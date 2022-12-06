use std::error::Error;
use std::fmt::Debug;
use std::process::{Command, Stdio};
use rnotifylib::message::MessageDetail;
use serde::{Serialize, Deserialize};
use crate::job_result::JobResult;
use crate::program_output::ProgramOutput;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ProgramOutputFormat {
    SimpleIfSuccess,
    StdoutIfSuccess,
    AlwaysDetailed,
}

impl Default for ProgramOutputFormat {
    fn default() -> Self {
        ProgramOutputFormat::SimpleIfSuccess
    }
}

pub async fn execute(cmd: &str, format: &ProgramOutputFormat) -> JobResult {
    match run_program(cmd) {
        Ok(mut output) => {
            let success = output.is_success();
            output.trim_to(500);
            let detail = output.to_detail(format);
            match success {
                true => JobResult::Ok(detail),
                false => JobResult::Failed(detail),
            }
        },
        Err(e) => JobResult::Invalid(MessageDetail::Raw(format!("Failed to run command: '{}'\nError: {e}", &cmd)))
    }
}

#[cfg(target_family = "windows")]
fn make_command() -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C");
    cmd
}

#[cfg(target_family = "unix")]
fn make_command() -> Command {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c");
    cmd
}

fn run_program(job: &str) -> Result<ProgramOutput, Box<dyn Error>> {
    let mut cmd = make_command();
    cmd.arg(job);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let process = cmd.spawn()?;

    let output = process.wait_with_output()?;

    let std_out = String::from_utf8_lossy(&output.stdout);
    let std_err = String::from_utf8_lossy(&output.stdout);

    Ok(ProgramOutput::new(std_out.into(), std_err.into(), output.status.code().unwrap_or(-1)))
}