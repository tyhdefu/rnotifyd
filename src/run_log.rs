use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use rnotifydlib::config::JobDefinitionId;

#[derive(Debug, PartialEq, Eq)]
pub struct RunLog {
    last_run: HashMap<JobDefinitionId, u64>,
}

impl RunLog {
    pub fn get_last_successful_run_time(&self, id: &JobDefinitionId) -> Option<u64> {
        self.last_run.get(id).copied()
    }

    pub fn record(&mut self, id: JobDefinitionId, timestamp: u64){
        self.last_run.insert(id, timestamp);
    }

    pub fn read_from_string(s: &str) -> Result<RunLog, String> {

        let mut map = HashMap::new();

        for line in s.lines()
            .filter(|line| !line.starts_with("#"))
            .filter(|line| !line.is_empty()) {

            let mut split = line.split(":");
            let id_part = split.next().ok_or("Missing 1st part of colon seperated entry.".to_owned())?;
            let id = JobDefinitionId::try_new(id_part.to_owned())?;
            let last_ran = split.next().ok_or("Missing 2nd part of colon seperated entry.".to_owned())?;
            let unix_time: u64 = last_ran.parse().map_err(|err| format!("Error converting {} to i64: {}", last_ran, err))?;
            map.insert(id, unix_time);
        }
        Ok(RunLog {
            last_run: map
        })
    }

    pub fn write_to_string(&self) -> String {
        let mut s = String::new();
        for (id, unix_time) in &self.last_run {
            s.push_str(&format!("{id}:{unix_time}\n"))
        }
        s
    }
}

impl Default for RunLog {
    fn default() -> Self {
        Self {
            last_run: HashMap::new(),
        }
    }
}

pub fn read_run_log(path: &PathBuf) -> RunLog {
    if !path.exists() {
        eprintln!("Cannot find run log file, assuming nothing has run.");
        return RunLog::default();
    }
    let run_log_str = fs::read_to_string(path).expect("Failed to read run log"); // TODO: Check if exists etc.

    RunLog::read_from_string(&run_log_str).expect("Failed to parse run log.")
}

pub fn write_run_log(run_log: &RunLog, path: &PathBuf) -> std::io::Result<()> {
    let s = run_log.write_to_string();
    fs::write(path, s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read() {
        let s = "hello-world:1670340125\nbeep-boop:1670370255";
        let parsed = RunLog::read_from_string(s).unwrap();
        let mut expected_map = HashMap::new();
        expected_map.insert(JobDefinitionId::try_new("hello-world".into()).unwrap(), 1670340125);
        expected_map.insert(JobDefinitionId::try_new("beep-boop".into()).unwrap(), 1670370255);
        let expected = RunLog {
            last_run: expected_map,
        };

        assert_eq!(expected, parsed);
    }
}