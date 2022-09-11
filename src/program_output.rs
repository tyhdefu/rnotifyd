#[derive(Debug)]
pub struct ProgramOutput {
    std_out: String,
    std_err: String,
    exit_code: i32,
}

impl ProgramOutput {
    pub fn new(std_out: String, std_err: String, exit_code: i32) -> Self {
        Self {
            std_out,
            std_err,
            exit_code
        }
    }

    pub fn trim_to(&mut self, max_len: usize) {
        Self::trim_string(&mut self.std_out, max_len);
        Self::trim_string(&mut self.std_err, max_len);
    }

    fn trim_string(s: &mut String, max_len: usize) {
        let len = s.len();
        if len > max_len {
            let hope_to_chop_at_start = len - max_len;

            let chop_start = s.char_indices()
                .map(|(i, _c)| i)
                .filter(|i| i > &hope_to_chop_at_start)
                .min();

            if let Some(chop_start) = chop_start {
                *s = s[chop_start..].to_owned();
            }
        }
    }


    pub fn get_stdout(&self) -> &str {
        &self.std_out
    }

    pub fn get_stderr(&self) -> &str {
        &self.std_err
    }

    pub fn get_exit_code(&self) -> i32 {
        self.exit_code
    }
}