use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize, Deserializer};
use crate::frequency::Frequency;
use crate::notify_definition::NotifyDefinition;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    jobs: HashMap<JobDefinitionId, JobDefinition>
}

impl Config {
    pub fn entries(&self) -> &HashMap<JobDefinitionId, JobDefinition> {
        &self.jobs
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct JobDefinition {
    cmd: String,
    #[serde(default)] // false by default.
    allow_parallel: bool,
    frequency: Frequency,
    #[serde(rename = "notification")]
    notify_definition: NotifyDefinition,
}

impl JobDefinition {
    pub fn get_cmd(&self) -> &String {
        &self.cmd
    }

    pub fn get_frequency(&self) -> &Frequency {
        &self.frequency
    }

    pub fn get_notify_definition(&self) -> &NotifyDefinition {
        &self.notify_definition
    }

    pub fn allow_parallel(&self) -> bool {
        self.allow_parallel
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
#[derive(Serialize)]
pub struct JobDefinitionId {
    id: String
}

impl JobDefinitionId {
    pub fn try_new(s: String) -> Result<Self, String> {
        if !inflections::case::is_kebab_case(&s) {
            return Err("Not camel case.".to_owned());
        }
        Ok(Self {
            id: s
        })
    }
}

impl Display for JobDefinitionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<'de> Deserialize<'de> for JobDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let string = String::deserialize(deserializer)?;
        Ok(JobDefinitionId::try_new(string).expect("Invalid job definition id."))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rnotifylib::message::component::Component;
    use crate::action::ProgramOutputFormat;
    use crate::config::{Config, JobDefinition, JobDefinitionId};
    use crate::frequency::{FixedPeriodInner, Frequency};
    use crate::notify_definition::NotifyDefinition;

    #[test]
    fn test_config() {
        let s = std::fs::read_to_string("test/example_config.yaml").expect("Failed to read file.");
        let config: Config = serde_yaml::from_str(&s).expect("Failed to deserialize config");

        let mut jobs = HashMap::new();
        let job = JobDefinition {
            cmd: "ping 192.168.0.10".to_string(),
            allow_parallel: false,
            frequency: Frequency::FixedPeriod(FixedPeriodInner::new(0, 30, 0)),
            notify_definition: NotifyDefinition::new("Ping 192.168.0.10".to_string(), Component::from("ping"),
                                                     false, ProgramOutputFormat::StdoutIfSuccess),
        };
        jobs.insert(JobDefinitionId::try_new("check-devices".to_string()).unwrap(), job);
        let expected = Config { jobs };

        assert_eq!(expected, config);
    }
}