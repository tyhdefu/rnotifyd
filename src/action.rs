use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Read;
use std::net::IpAddr;
use std::process::{Command, Stdio};
use rnotifylib::message::MessageDetail;
use serde::{Serialize, Deserialize};
use surge_ping::SurgeError;
use crate::job_result::JobResult;
use crate::program_output::ProgramOutput;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Action {
    /// Runs a program. If it fails, pipe the output to rnotify.
    Program(ProgramAction),
    /// Checks if a systemd service is running on the current machine
    SystemdActiveLocal { service: String },
    /// Checks if a device responds to a ping.
    Ping{ host: String },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProgramAction {
    program: String,
    args: Vec<String>,
    #[serde(default)]
    output_format: ProgramOutputFormat
}

#[derive(Serialize, Deserialize, Clone)]
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

impl Action {
    pub async fn execute(&self) -> JobResult {
        return match &self {
            Action::Program(action) => {
                let mut cmd = Command::new(&action.program);
                cmd.args(&action.args);
                match run_program(cmd) {
                    Ok(mut output) => {
                        let success = output.is_success();
                        output.trim_to(500);
                        let detail = output.to_detail(&action.output_format);
                        match success {
                            true => JobResult::Ok(detail),
                            false => JobResult::Failed(detail),
                        }
                    },
                    Err(e) => JobResult::Invalid(MessageDetail::Raw(format!("Program: {}' with args '{:?}'\nError: {e}", &action.program, &action.args)))
                }
            }
            Action::SystemdActiveLocal{ service } => {
                let mut cmd = Command::new("systemctl");
                cmd.arg("is-active");
                cmd.arg(service);
                match run_program(cmd) {
                    Ok(output) => {
                        match output.is_success() {
                            true => JobResult::Ok(MessageDetail::Raw(format!("Systemd service {service} is running."))),
                            false => JobResult::Failed(MessageDetail::Raw(format!("Systemd service {service} is not running"))),
                        }
                    },
                    Err(err) => JobResult::Invalid(MessageDetail::Raw(format!("Failed to check whether systemd service is running: {err:?}")))
                }
            }
            Action::Ping { host } => {
                let ip: IpAddr = match host.parse() {
                    Ok(ip) => ip,
                    Err(err) => return JobResult::Invalid(MessageDetail::Raw(format!("Failed to parse host '{host}': {err:?}"))),
                };
                let payload = [0u8, 1, 2, 3, 4, 5, 6, 7];
                match surge_ping::ping(ip, &payload).await {
                    Ok((_, duration)) => JobResult::Ok(MessageDetail::Raw(format!("Host '{host}' responded to ping after {}ms", duration.as_millis()))),
                    Err(err) => {
                        match err {
                            SurgeError::Timeout { .. } => JobResult::Failed(MessageDetail::Raw(format!("Pinging '{host}' timed out."))),
                            error => JobResult::Invalid(MessageDetail::Raw(format!("Failed to ping host {host}: {error:?}")))
                        }
                    }
                }
            }
        };
    }
}

fn run_program(mut cmd: Command) -> Result<ProgramOutput, Box<dyn Error>> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut process = cmd.spawn()?;
    let status = process.wait()?;

    let mut std_out = String::new();
    process.stdout.take().unwrap().read_to_string(&mut std_out)?;

    let mut std_err = String::new();
    process.stderr.take().unwrap().read_to_string(&mut std_err)?;

    Ok(ProgramOutput::new(std_out, std_err, status.code().unwrap_or(-1)))
}

#[derive(Debug)]
struct FailedToResolveHost {
    host: String,
}

impl FailedToResolveHost {
    pub fn new(host: String) -> Self {
        Self {
            host
        }
    }
}

impl Error for FailedToResolveHost {}

impl Display for FailedToResolveHost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to resolve '{}' to an Ip.", self.host)
    }
}