use std::fs;
use std::path::PathBuf;
use getopts::Matches;
use crate::{RNOTIFYD_CONFIG_ARG,  RNOTIFY_CONFIG_ARG, RNOTIFY_RUN_LOG_ARG};

pub struct AllConfig {
    rnotify: rnotifylib::config::Config,
    job_config: rnotifydlib::config::Config,
    run_log: PathBuf,
}

impl AllConfig {
    pub fn get_job_config(&self) -> &rnotifydlib::config::Config {
        &self.job_config
    }

    pub fn get_rnotify_config(&self) -> &rnotifylib::config::Config {
        &self.rnotify
    }

    pub fn get_run_log_path(&self) -> &PathBuf {
        &self.run_log
    }
}

pub fn read_configs(parsed: &Matches) -> AllConfig {
    let rnotify_config_path = parsed.opt_str(RNOTIFY_CONFIG_ARG)
        .map(PathBuf::from)
        .unwrap_or_else(|| rnotifylib::config::get_default_config_path());

    let rnotifyd_config_path = get_string_arg(parsed, RNOTIFYD_CONFIG_ARG);

    let rnotify_storage_path = parsed.opt_str(RNOTIFY_RUN_LOG_ARG)
        .unwrap_or_else(|| String::from("run_log.txt"));


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
            Err(err) => panic!("Error reading rnotifyd (yaml) config file {}", err),
        };
        match serde_yaml::from_str(&rnotifyd_config_str) {
            Ok(c) => c,
            Err(err) => panic!("Error parsing rnotifyd (yaml) config file: {}", err),
        }
    };

    AllConfig {
        rnotify: rnotify_config,
        job_config: rnotifyd_config,
        run_log,
    }
}

fn get_string_arg(matches: &Matches, arg_name: &str) -> String {
    match matches.opt_str(arg_name) {
        None => panic!("Missing argument: {}", arg_name),
        Some(s) => s
    }
}