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