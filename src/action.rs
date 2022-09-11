use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::{Command, Stdio};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::program_output::ProgramOutput;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Action {
    /// Runs a program. If it fails, pipe the output to rnotify.
    Program{ program: String, args: Vec<String> },
    /// Checks if a systemd service is running on the current machine
    SystemdActiveLocal { service: String },
    /// Checks if a device responds to a ping.
    Ping{ host: String },
}

impl Action {
    pub async fn execute(&self) -> Result<ProgramOutput, Box<dyn std::error::Error>> {

        fn run_program(mut cmd: Command) -> Result<ProgramOutput, Box<dyn std::error::Error>> {
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

        return match &self {
            Action::Program{ program, args } => {
                let mut cmd = Command::new(program);
                cmd.args(args);
                run_program(cmd)
            }
            Action::SystemdActiveLocal{ service } => {
                let mut cmd = Command::new("systemctl");
                cmd.arg("is-active");
                cmd.arg(service);
                run_program(cmd)
            }
            Action::Ping { host } => {
                let ip: IpAddr = host.parse()?;
                let payload = [0u8, 1, 2, 3, 4, 5, 6, 7];
                surge_ping::ping(ip, &payload).await?;
                Ok(ProgramOutput::new(String::new(), String::new(), 0))
            }
        };
    }
}

//async fn resolve_hostname(host: &str) -> Result<IpAddr, Box<dyn Error>> {
//    let resolver = MdnsResolver::new().await?;
//    println!("Ping1.5");
//    let res = resolver.query_timeout(host, Duration::from_secs(3)).await?;
//    println!("Ping1.6");
//    let packet = res.to_packet()?;
//    let first_result = packet.answers.iter().next().ok_or_else(|| FailedToResolveHost::new(host.to_string()))?;
//    Ok(match &first_result.rdata {
//        RData::A(rec) => IpAddr::V4(Ipv4Addr::from(rec.address)),
//        RData::AAAA(rec) => IpAddr::V6(Ipv6Addr::from(rec.address)),
//        _ => Err(FailedToResolveHost::new(host.to_string()))?,
//    })
//}

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