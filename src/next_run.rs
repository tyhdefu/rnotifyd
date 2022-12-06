use std::collections::HashMap;
use chrono::{DateTime, Local};
use rnotifydlib::config::JobDefinitionId;
use rnotifydlib::frequency::Frequency;
use std::time::Duration;
use crate::RunLog;

pub struct NextRun {
    map: HashMap<JobDefinitionId, u64>,
}

impl NextRun {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Calculates, caches, and gets the timestamp of the next time the job with the given id should be run.
    /// run_log: completed successful jobs
    /// provisional_runs: jobs that have started but not yet finished, so may fail,
    ///                   but we assume that they will succeed.
    pub fn update_and_get<P: ProvisionalJobRuns>(&mut self, id: &JobDefinitionId, frequency: &Frequency, now: DateTime<Local>,
                      run_log: &RunLog, provisional_runs: &P) -> u64 {
        *self.map.entry(id.clone()).or_insert_with(|| {
            let last_run = provisional_runs.get_latest(id)
                .or_else(|| run_log.get_last_successful_run_time(id));

            frequency.next(&now, last_run)
        })
    }

    /// Invalidate the cached timestamp for a particular job id
    /// Should be used if the data it was calculated on is now incorrect:
    /// - The job has since run
    /// - The job ran, but failed, so actually has not (successfully) ran recently
    pub fn invalidate(&mut self, id: &JobDefinitionId) {
        self.map.remove(id);
    }

    /// Gets the duration to wait until the next job is due to be run.
    pub fn get_wait(&self, now: u64) -> Duration {
        Duration::from_secs(self.map.values()
            .map(|i| i - now).min()
            .unwrap_or(u64::MAX))
    }
}

pub trait ProvisionalJobRuns {
    fn get_latest(&self, id: &JobDefinitionId) -> Option<u64>;
}
