#[must_use]
fn num_digits(n: usize) -> usize {
    n.checked_ilog10().unwrap_or(0) as usize + 1
}

#[derive(Debug)]
pub struct ProgramSrc {
    src: String,
    line_nums: Vec<usize>,
}

impl ProgramSrc {
    #[must_use]
    pub fn new(src: String) -> Self {
        // 1-indexed, so first line is line_nums[1]
        let mut line_nums = vec![0, 0];

        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_nums.push(i + 1);
            }
        }

        line_nums.push(src.len());

        Self { src, line_nums }
    }
    
    pub fn stream(&self) -> impl Iterator<Item = u8> {
        self.src.bytes()
    }

    #[must_use]
    fn get_line(&self, line: usize) -> Option<ProgramLine<'_>> {
        if line == 0 {
            return None;
        }
        let first = *self.line_nums.get(line)?;
        let last = *self.line_nums.get(line + 1)?;
        let contents = &self.src[first..last - 1]; // remove final newline
        Some(ProgramLine {
            contents,
            num: line,
        })
    }
}

#[derive(Debug, Clone)]
struct ProgramLine<'a> {
    contents: &'a str,
    num: usize,
}

impl ProgramLine<'_> {
    #[must_use]
    fn display(&self, gutter_size: usize, post_gutter: &str, pre_line: &str) -> String {
        format!(
            "{}{}{post_gutter} | {pre_line}{}",
            " ".repeat(gutter_size - num_digits(self.num)),
            self.num,
            self.contents
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    line: usize,
    col: usize,
}

impl Loc {
    #[must_use]
    pub fn new(line: usize, col: usize) -> Self {
        assert_ne!(line, 0, "lines are 1-indexed");
        assert_ne!(col, 0, "columns are 1-indexed");
        Self { line, col }
    }

    #[must_use]
    pub fn format_with_len(&self, src: &ProgramSrc, message: &str, len: usize) -> String {
        let prev = src.get_line(self.line - 1);
        let this = src.get_line(self.line);
        let next = src.get_line(self.line + 1);

        let gutter_size = num_digits(self.line + 1);

        let make_line = |line: Option<ProgramLine<'_>>| {
            line.map(|line| line.display(gutter_size, "", ""))
                .unwrap_or_default()
        };

        let prev_fmt = make_line(prev);
        let this_fmt = make_line(this);
        let next_fmt = make_line(next);

        let underline = format!(
            "{}|{}{} {}",
            " ".repeat(gutter_size + 1),
            " ".repeat(self.col),
            "^".repeat(len),
            message
        );

        format!("{prev_fmt}\n{this_fmt}\n{underline}\n{next_fmt}")
    }

    #[must_use]
    pub fn format(&self, src: &ProgramSrc, message: &str) -> String {
        self.format_with_len(src, message, 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    start: Loc,
    end: Loc,
}

impl Span {
    #[must_use]
    pub fn new(start: Loc, end: Loc) -> Self {
        assert!(end >= start);
        Self { start, end }
    }

    #[must_use]
    pub fn format(&self, src: &ProgramSrc, message: &str) -> String {
        if self.start.line == self.end.line {
            return self
                .start
                .format_with_len(src, message, self.end.col - self.start.col);
        }

        let prev = src.get_line(self.start.line - 1);
        let start = src.get_line(self.start.line);
        let middle = (self.start.line + 1..=self.end.line).map(|i| src.get_line(i));
        let next = src.get_line(self.end.line + 1);

        let num_space = num_digits(self.end.line + 1);

        let make_line = |line: Option<ProgramLine<'_>>, pre_line: &str| {
            line.map(|line| line.display(num_space, "", pre_line))
                .unwrap_or_default()
        };

        let prev_fmt = make_line(prev, "  ");
        let start_fmt = make_line(start, "  ");
        let start_underline = format!(
            "{}|  {}^",
            " ".repeat(num_space + 1),
            "_".repeat(self.start.col)
        );
        let middle_fmt = middle.map(|line| make_line(line, "| ")).collect::<Vec<_>>();
        let end_underline = format!(
            "{}| |{}^ {message}",
            " ".repeat(num_space + 1),
            "_".repeat(self.end.col)
        );
        let next_fmt = make_line(next, "  ");

        format!(
            "{prev_fmt}\n{start_fmt}\n{start_underline}\n{}\n{end_underline}\n{next_fmt}",
            middle_fmt.join("\n")
        )
    }
}
