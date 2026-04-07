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

        Self { src, line_nums }
    }

    pub fn get_line_nums(&self) -> &Vec<usize> {
        &self.line_nums
    }
}

pub struct Loc {
    line: usize,
    col: usize,
    length: usize,
}

impl Loc {
    pub fn new(line: usize, col: usize, length: usize) -> Self {
        Self { line, col, length }
    }

    pub fn get_line(src: &ProgramSrc, line: usize) -> &str {
        // check that line is a valid line
        let line_nums = src.get_line_nums();
        let first = *line_nums.get(line).unwrap(); // error message
        let last = *line_nums.get(line + 1).unwrap() - 1; // check that last is valid
        &src.src[first..last]
    }

    pub fn format(&self, src: &ProgramSrc) -> String {
        let prev = Self::get_line(src, self.line - 1);
        let this = Self::get_line(src, self.line);
        let next = Self::get_line(src, self.line + 1);

        let prev_fmt = format!("{} | {}", self.line - 1, prev);
        let this_fmt = format!("{} | {}", self.line, this);
        let next_fmt = format!("{} | {}", self.line + 1, next);

        let underline = format!("{}{}", " ".repeat(self.col), "^".repeat(self.length));
        debug_assert!(this.len() >= self.col + self.length);
        
        format!("{prev_fmt}\n{this_fmt}\n  | {underline}\n{next_fmt}")
    }
}
