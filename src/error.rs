pub struct Error {
    msg: String,
    fname: String,
    src_line: u32,
    src_column: u32,
}

impl Error {
    pub fn new(msg: &str, file: &str, ln: u32, col: u32) -> Self {
        Self {
            msg: String::from(msg),
            fname: String::from(file),
            src_line: ln,
            src_column: col,
        }
    }

    pub fn message(&self) -> &str {
        &self.msg[..]
    }

    pub fn file_name(&self) -> &str {
        &self.fname[..]
    }

    pub fn line(&self) -> u32 {
        self.src_line
    }

    pub fn column(&self) -> u32 {
        self.src_column
    }
}
