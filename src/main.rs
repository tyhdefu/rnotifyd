use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use chrono::{DateTime, Local};
use getopts::{Matches, Options};
use tokio::select;
use tokio::time::MissedTickBehavior;
use rnotifydlib::action::Action;
use rnotifydlib::config::{JobDefinition, JobDefinitionId};
use rnotifydlib::notify_definition::NotifyDefinition;
use crate::run_log::RunLog;

const RNOTIFY_CONFIG_ARG: &str = "rnotify-config";
const RNOTIFYD_CONFIG_ARG: &str = "config";
const RNOTIFY_RUN_LOG_ARG: &str = "runlog";

const CHECK_INTERVAL: Duration = Duration::from_secs(60);

mod run_log;

#[tokio::main]
async fn main() {
    let mut opts = Options::new();
    opts.optopt("", RNOTIFY_CONFIG_ARG, "The rnotify.toml file.", "RNOTIFY");
    opts.optopt("", RNOTIFYD_CONFIG_ARG, "The rnotifyd.json file.", "RNOTIFYD");
    opts.optopt("", RNOTIFY_RUN_LOG_ARG, "The run log file.", "RNOTIFYD RUNLOG");
    let args: Vec<_> = std::env::args().collect();
    let parsed = match opts.parse(args) {
        Ok(matches) => matches,
        Err(err) => panic!("Invalid arguments: {}", err),
    };
    let configs = read_configs(&parsed);

    let run_log = read_run_log(&configs.run_log);
    main_loop(configs, run_log).await;
}

async fn main_loop(config: AllConfig, mut run_log: RunLog) {
    println!("Beginning main loop.");
    let mut interval = tokio::time::interval(CHECK_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let job_config = config.get_job_config().clone();
    let mut next_run = HashMap::new();

    fn update_next_run(next_run: &mut HashMap<JobDefinitionId, u64>, now: DateTime<Local>,
                       id: &JobDefinitionId, definition: &JobDefinition,
                        run_log: &RunLog) -> u64 {
        *next_run.entry(id.clone()).or_insert_with(|| {
            let timestamp_now = now.timestamp() as u64;

            let last_run = run_log.get_last_run(id);
            let next = definition.get_frequency().next(&now, last_run);
            println!("Next {}, now: {}, diff: {}", next, timestamp_now, next - timestamp_now);
            next
        })
    }

    loop {
        let now = Local::now();
        let timestamp_now = now.timestamp() as u64;
        for (id, definition) in job_config.entries() {

            let next = update_next_run(&mut next_run, now, id, definition, &run_log);
            if timestamp_now >= next {
                next_run.remove(id);
                println!("Job {} is due to run.", id);
                // Run task.
                spawn_job(id.clone(), definition.get_action().clone(), definition.get_notify_definition().clone(), config.rnotify.clone());
                run_log.insert(id.clone(), timestamp_now);
                update_next_run(&mut next_run, now + chrono::Duration::seconds(1), id, definition, &run_log);
            }
        }
        let min_wait = next_run.values().map(|i| i - timestamp_now).min().unwrap_or(u64::MAX);
        println!("Min wait: {}", min_wait);
        let min_wait_duration = Duration::from_secs(min_wait);
        let quick_sleep = tokio::time::sleep(min_wait_duration);
        tokio::pin!(quick_sleep);
        select!(
            _ = tokio::signal::ctrl_c() => {
                eprintln!("Received control-c");
                break;
            }
            _ = interval.tick() => {
                println!("Tick: {}", Local::now());
            }
            _ = quick_sleep, if min_wait_duration < CHECK_INTERVAL => {
                println!("Quick tick.");
            }
        );
    }
}

fn spawn_job(id: JobDefinitionId, action: Action, notify_definition: NotifyDefinition, rnotify_config: rnotifylib::config::Config) {
    tokio::task::spawn_blocking(move || {
        println!("Running job: {} at {}", id, Local::now());
        let output = action.execute();
        if let Err(err) = output {
            eprintln!("Failed to run job: {}", err);
            return;
        }
        let output = output.unwrap();
        println!("Job {} returned exit code {:?}", id, output.get_exit_status());
        match notify_definition.create_message(&id, output) {
            None => println!("Job {} ran and didn't need a rnotify message to be sent about it.", id),
            Some(message) => {
                println!();
                match rnotifylib::send_message(message, &rnotify_config) {
                    Ok(()) => println!("Job {} ran, and successfully sent a message to rnotify.", id),
                    Err(errs) => println!("Job {} ran, but failed to send a message to rnotify {}", id, errs),
                }
            }
        }
        // TODO: Record job run.
    });
}

fn get_string_arg(matches: &Matches, arg_name: &str) -> String {
    match matches.opt_str(arg_name) {
        None => panic!("Missing argument: {}", arg_name),
        Some(s) => s
    }
}

fn read_configs(parsed: &Matches) -> AllConfig {
    let rnotify_config_path = get_string_arg(parsed, RNOTIFY_CONFIG_ARG);
    let rnotifyd_config_path = get_string_arg(parsed, RNOTIFYD_CONFIG_ARG);
    let rnotify_storage_path = get_string_arg(parsed, RNOTIFY_RUN_LOG_ARG);

    let run_log: PathBuf = rnotify_storage_path.into();

    let rnotify_config: rnotifylib::config::Config = {
        let rnotify_config_str = match fs::read_to_string(rnotify_config_path) {
            Ok(s) => s,
            Err(err) => panic!("Error reading rnotify (toml) config file: {}", err),
        };
        match toml::from_str(&rnotify_config_str) {
            Ok(c) => c,
            Err(err) => panic!("Error parsing rnotify (toml) config file: {}", err),
        }
    };

    let rnotifyd_config: rnotifydlib::config::Config = {
        let path: PathBuf = rnotifyd_config_path.into();
        if !path.exists() {
            panic!("Config file: '{:?}' does not exist.", path);
        }
        let rnotifyd_config_str = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) => panic!("Error reading rnotifyd (json) config file {}", err),
        };
        match serde_json::from_str(&rnotifyd_config_str) {
            Ok(c) => c,
            Err(err) => panic!("Error parsing rnotifyd (json) config file: {}", err),
        }
    };

    AllConfig {
        rnotify: rnotify_config,
        job_config: rnotifyd_config,
        run_log,
    }
}

fn read_run_log(path: &PathBuf) -> RunLog {
    if !path.exists() {
        eprintln!("Cannot find run log file, assuming nothing has run.");
        return RunLog::default();
    }
    let run_log_str = fs::read_to_string(path).expect("Failed to read run log"); // TODO: Check if exists etc.

    RunLog::read_from_string(&run_log_str).expect("Failed to parse run log.")
}

fn write_run_log(run_log: &RunLog, path: &PathBuf) -> std::io::Result<()>{
    let s = run_log.write_to_string();
    fs::write(path, s)
}

pub struct AllConfig {
    rnotify: rnotifylib::config::Config,
    job_config: rnotifydlib::config::Config,
    run_log: PathBuf,
}

impl AllConfig {
    pub fn get_job_config(&self) -> &rnotifydlib::config::Config {
        &self.job_config
    }
}