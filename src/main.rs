use std::cmp::min;
use std::path::PathBuf;
use std::time::Duration;
use chrono::{Local, SecondsFormat};
use getopts::Options;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;
use tokio::time::MissedTickBehavior;
use all_config::AllConfig;
use next_run::NextRun;
use rnotifydlib::action;
use rnotifydlib::config::JobDefinitionId;
use rnotifydlib::job_result::JobResult;
use rnotifydlib::notify_definition::NotifyDefinition;
use crate::run_log::RunLog;
use crate::running_jobs::RunningJobs;

const RNOTIFY_CONFIG_ARG: &str = "rnotify-config";
const RNOTIFYD_CONFIG_ARG: &str = "config";
const RNOTIFY_RUN_LOG_ARG: &str = "runlog";

const CHECK_INTERVAL: Duration = Duration::from_secs(60);

mod run_log;
mod all_config;
mod next_run;
mod running_jobs;

#[tokio::main(worker_threads = 1)]
async fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(3)
        .enable_time()
        .build()
        .unwrap();

    println!("-- Started at: {} --", Local::now().to_rfc3339_opts(SecondsFormat::Millis, true));
    let mut opts = Options::new();
    opts.optopt("", RNOTIFY_CONFIG_ARG, "The rnotify.toml file.", "RNOTIFY");
    opts.reqopt("", RNOTIFYD_CONFIG_ARG, "The rnotifyd.yaml file.", "RNOTIFYD");
    opts.optopt("", RNOTIFY_RUN_LOG_ARG, "The run log file.", "RUNLOG");
    let args: Vec<_> = std::env::args().collect();
    let parsed = match opts.parse(args) {
        Ok(matches) => matches,
        Err(err) => panic!("Invalid arguments: {}", err),
    };
    let configs = all_config::read_configs(&parsed);

    let run_log = run_log::read_run_log(&configs.get_run_log_path());
    println!("RunLog: {:?}", run_log);
    let handle = runtime.handle().clone();

    main_loop(configs, run_log, runtime).await
}

async fn main_loop(config: AllConfig, mut run_log: RunLog, rt: Runtime) {
    let guard = rt.enter();

    println!("Beginning main loop.");
    let mut interval = tokio::time::interval(CHECK_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let job_config = config.get_job_config().clone();
    let mut next_run = NextRun::new();

    // Currently running non-parallel allowed jobs
    let mut running = RunningJobs::new();

    // Sender to report when jobs finish.
    let (send, mut recv) = tokio::sync::mpsc::channel(10);

    loop {
        let now = Local::now();
        let timestamp_now = now.timestamp() as u64;
        for (id, definition) in job_config.entries() {
            let next = next_run.update_and_get(id, definition.get_frequency(), now, &run_log, &running);
            if timestamp_now >= next {
                if !definition.allow_parallel() && running.any_running(&id) {
                    println!("Job {} is due to run, but is already running, so it will not be run yet.", id);
                    continue;
                }
                println!("Job {} is due to run.", id);

                running.add(id.clone(), timestamp_now);
                next_run.invalidate(id);
                next_run.update_and_get(id, definition.get_frequency(), now + chrono::Duration::seconds(1), &run_log, &running);

                // Run task.
                spawn_job(id.clone(), definition.get_cmd().clone(), definition.get_notify_definition().clone(),
                          config.get_rnotify_config().clone(), timestamp_now, send.clone());
            }
        }

        let short_wait = next_run.get_wait(timestamp_now);
        let sleep = min(short_wait, CHECK_INTERVAL);

        tokio::select!(
            _ = tokio::time::sleep(sleep) => {

            }
            _ = tokio::signal::ctrl_c() => {
                eprintln!("Terminating");
                let s: String = running.get_running().iter()
                    .filter(|(_a, b)| !b.is_empty())
                    .map(|(a, b)| format!("{a} x{}", b.len()))
                    .collect::<Vec<String>>()
                    .join(", ");

                if !s.is_empty() {
                    eprintln!("Some jobs were still running when program aborted: {}", s);
                }
                rt.shutdown_timeout(Duration::from_millis(500));
                return;
            }
            job_finish = recv.recv() => {
                if job_finish.is_none() {
                    eprintln!("Job finish receiver had the other end dropped");
                    return;
                }
                let job_finish = job_finish.unwrap();
                println!("Job finished: {:?}", job_finish);
                running.mark_completed(&job_finish.id, job_finish.started);

                run_log.record(job_finish.id, job_finish.started);
                spawn_runlog_write(run_log.write_to_string(), config.get_run_log_path().clone());
            }
        );
    }
}

#[derive(Debug)]
pub struct JobFinish {
    id: JobDefinitionId,
    started: u64,
    success: bool,
}

impl JobFinish {
    fn new(id: JobDefinitionId, started: u64, success: bool) -> Self {
        Self {
            id,
            started,
            success,
        }
    }
}

fn spawn_runlog_write(s: String, loc: PathBuf) {
    tokio::spawn(async move {
        match std::fs::write(loc, s) {
            Ok(_) => {
                println!("Wrote run log.");
            }
            Err(err) => {
                eprintln!("Error writing run log: {err}");
            }
        }
    });
}

fn spawn_job(id: JobDefinitionId, cmd: String, notify_definition: NotifyDefinition,
             rnotify_config: rnotifylib::config::Config, start_timestamp: u64, job_finish_sender: Sender<JobFinish>) {
    tokio::task::spawn(run_job(id, cmd, notify_definition, rnotify_config, start_timestamp, job_finish_sender));
}

async fn run_job(id: JobDefinitionId, cmd: String, notify_definition: NotifyDefinition,
                 rnotify_config: rnotifylib::config::Config, start_timestamp: u64, job_finish_sender: Sender<JobFinish>) {
    println!("[{id}] Running at {}", Local::now().to_rfc3339_opts(SecondsFormat::Millis, true));
    let output = action::execute(&cmd, notify_definition.get_output_format()).await;
    if let JobResult::Invalid(err) = &output {
        eprintln!("[{id}] Failed to run job: {:?}", err);
    }
    let succ = matches!(&output, JobResult::Ok(_));
    let job_finish = JobFinish::new(id.clone(), start_timestamp, succ);

    println!("[{id}] Job had outcome {}", output.type_str());
    match notify_definition.create_message(&id, output) {
        None => println!("[{id}] Didn't need a rnotify message to be sent"),
        Some(message) => {
            match rnotifylib::send_message(message, &rnotify_config) {
                Ok(()) => println!("[{id}] Sent a message to rnotify."),
                Err(errs) => println!("[{id}] Failed to send a message to rnotify {}", errs),
            }
        }
    }

    if let Err(e) = job_finish_sender.send(job_finish).await {
        eprintln!("Failed to record job finishing: {}", e);
    }

}
