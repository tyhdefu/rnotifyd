use std::io::Read;
use std::net::IpAddr;
use std::process::{Command, Stdio};
use serde::{Serialize, Deserialize};
use crate::program_output::ProgramOutput;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Action {
    /// Runs a program. If it fails, pipe the output to rnotify.
    Program{ program: String, args: Vec<String> },
    /// Checks if a systemd service is running on the current machine
    SystemdActiveLocal(String),
    /// Checks if a device responds to a ping.
    Ping(IpAddr),
}

impl Action {
    pub fn execute(&self) -> std::io::Result<ProgramOutput> {

        fn run_program(mut cmd: Command) -> std::io::Result<ProgramOutput> {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut process = cmd.spawn()?;
            let status = process.wait()?;

            let mut std_out = String::new();
            process.stdout.take().unwrap().read_to_string(&mut std_out)?;

            let mut std_err = String::new();
            process.stderr.take().unwrap().read_to_string(&mut std_err)?;

            Ok(ProgramOutput::new(std_out, std_err, status))
        }

        return match &self {
            Action::Program{ program, args } => {
                let mut cmd = Command::new(program);
                cmd.args(args);
                run_program(cmd)
            }
            Action::SystemdActiveLocal(_) => {
                let mut cmd = Command::new("systemctl");
                cmd.arg("is-active");
                run_program(cmd)
            }
            Action::Ping(ip) => {
                let mut cmd = Command::new("ping");
                cmd.arg(ip.to_string());
                run_program(cmd)
            }
        };
    }
}