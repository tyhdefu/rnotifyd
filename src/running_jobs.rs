use std::collections::HashMap;
use rnotifydlib::config::JobDefinitionId;
use crate::next_run::ProvisionalJobRuns;

pub struct RunningJobs {
    map: HashMap<JobDefinitionId, Vec<u64>>,
}

impl RunningJobs {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: JobDefinitionId, started: u64) {
        let vec = self.map.entry(id).or_insert_with(|| vec![]);
        vec.push(started);
    }

    pub fn mark_completed(&mut self, id: &JobDefinitionId, started: u64) {
        match self.map.get_mut(id) {
            None => eprintln!("No job to mark completed."),
            Some(vec) => {
                vec.retain(|i| *i != started);
            }
        }
    }

    pub fn any_running(&self, id: &JobDefinitionId) -> bool {
        self.map.get(id)
            .filter(|vec| !vec.is_empty())
            .is_some()
    }

    pub fn get_running(&self) -> &HashMap<JobDefinitionId, Vec<u64>> {
        &self.map
    }
}

impl ProvisionalJobRuns for RunningJobs {
    fn get_latest(&self, id: &JobDefinitionId) -> Option<u64> {
        self.map.get(id)
            .map(|vec| vec.iter().max())
            .flatten()
            .copied()
    }
}