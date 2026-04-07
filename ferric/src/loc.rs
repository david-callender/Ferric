use std::fmt::format;

fn num_digits(n: usize) -> usize {
    n.checked_ilog10().unwrap_or(0) as usize + 1
}

#[derive(Debug)]
pub struct ProgramSrc {
    src: String,
    line_nums: Vec<usize>,
}

impl ProgramSrc {
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

    pub fn get_line_nums(&self) -> &Vec<usize> {
        &self.line_nums
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    line: usize,
    col: usize,
}

impl Loc {
    pub fn new(line: usize, col: usize) -> Self {
        assert_ne!(line, 0);
        Self { line, col }
    }

    fn get_line(src: &ProgramSrc, line: usize) -> Option<&str> {
        if line == 0 {
            return None;
        }
        let line_nums = src.get_line_nums();
        let first = *line_nums.get(line)?;
        let last = *line_nums.get(line + 1)?;
        Some(&src.src[first..last - 1]) // remove final newline
    }

    pub fn format_with_len(&self, src: &ProgramSrc, message: &str, len: usize) -> String {
        let prev = Self::get_line(src, self.line - 1);
        let this = Self::get_line(src, self.line);
        let next = Self::get_line(src, self.line + 1);

        let prev_num_len = num_digits(self.line - 1);
        let this_num_len = num_digits(self.line);
        let next_num_len = num_digits(self.line + 1);

        let num_space = next_num_len;

        let make_line = |line: Option<&str>, num_len| {
            line.map(|prev| {
                format!(
                    "{}{} | {}",
                    " ".repeat(num_space - num_len),
                    self.line - 1,
                    prev
                )
            })
            .unwrap_or_default()
        };

        let prev_fmt = make_line(prev, prev_num_len);
        let this_fmt = make_line(this, this_num_len);
        let next_fmt = make_line(next, next_num_len);

        let underline = format!(
            "{}|{}{} {}",
            " ".repeat(num_space + 1),
            " ".repeat(self.col + 1),
            "^".repeat(len),
            message
        );

        format!("{prev_fmt}\n{this_fmt}\n{underline}\n{next_fmt}")
    }
    
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
    pub fn new(start: Loc, end: Loc) -> Self {
        assert!(end >= start);
        Self { start, end }
    }
    
    pub fn format(&self, src: &ProgramSrc, message: &str) -> String {
        if self.start.line == self.end.line {
            return self.start.format_with_len(src, message, self.end.col - self.start.col);
        }
        
        let prev = Loc::get_line(src, self.start.line - 1);
        let start = Loc::get_line(src, self.start.line);
        
        let middle = (self.start.line+1..=self.end.line).map(|i| (i, Loc::get_line(src, i)));
        
        let next = Loc::get_line(src, self.end.line + 1);
        
        let num_space = num_digits(self.end.line + 1);
        
        let prev_fmt = prev.map(|prev| format!("{}{} |   {}", " ".repeat(num_space - num_digits(self.start.line - 1)), self.start.line - 1, prev)).unwrap_or_default();
        let start_fmt = start.map(|start| format!("{}{} |   {}", " ".repeat(num_space - num_digits(self.start.line)), self.start.line, start)).unwrap_or_default();
        let start_underline = format!("{}|  {}^", " ".repeat(num_space+1), "_".repeat(self.start.col + 1));
        
        let middle_fmt = middle.map(|(i, line)| line.map(|line| format!("{}{} | | {}", " ".repeat(num_space - num_digits(i)), i, line)).unwrap_or_default() ).collect::<Vec<_>>();
        
        let end_underline = format!("{}| |{}^", " ".repeat(num_space+1),"_".repeat(self.end.col + 1));
        let next_fmt = next.map(|next| format!("{}{} |   {}", " ".repeat(num_space - num_digits(self.end.line + 1)), self.end.line + 1, next)).unwrap_or_default();
        
        format!("{prev_fmt}\n{start_fmt}\n{start_underline}\n{}\n{end_underline}\n{next_fmt}", middle_fmt.join("\n"))
    }
}