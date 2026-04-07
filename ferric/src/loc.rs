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

pub struct Loc {
    line: usize,
    col: usize,
}

impl Loc {
    pub fn new(line: usize, col: usize) -> Self {
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

    pub fn format(&self, src: &ProgramSrc, message: &str) -> String {
        let prev = Self::get_line(src, self.line - 1);
        let this = Self::get_line(src, self.line);
        let next = Self::get_line(src, self.line + 1);

        let prev_fmt = prev
            .map(|prev| format!("{} | {}", self.line - 1, prev))
            .unwrap_or_default();
        let this_fmt = this
            .map(|this| format!("{} | {}", self.line, this))
            .unwrap_or_default();
        let next_fmt = next
            .map(|next| format!("{} | {}", self.line + 1, next))
            .unwrap_or_default();

        let underline = format!("{}^ {}", " ".repeat(self.col), message);

        format!("{prev_fmt}\n{this_fmt}\n  | {underline}\n{next_fmt}")
    }
}
