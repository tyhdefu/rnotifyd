use std::process::ExitStatus;

#[derive(Debug)]
pub struct ProgramOutput {
    std_out: String,
    std_err: String,
    exit_status: ExitStatus,
}

impl ProgramOutput {
    pub fn new(std_out: String, std_err: String, exit_status: ExitStatus) -> Self {
        Self {
            std_out,
            std_err,
            exit_status,
        }
    }


    pub fn get_stdout(&self) -> &str {
        &self.std_out
    }

    pub fn get_stderr(&self) -> &str {
        &self.std_err
    }

    pub fn get_exit_status(&self) -> &ExitStatus {
        &self.exit_status
    }
}