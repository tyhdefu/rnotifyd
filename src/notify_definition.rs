use std::time::{SystemTime, UNIX_EPOCH};
use rnotifylib::message::author::Author;
use rnotifylib::message::component::Component;
use rnotifylib::message::{Level, Message};
use crate::config::JobDefinitionId;
use serde::{Serialize, Deserialize};
use crate::job_result::JobResult;

#[derive(Serialize, Deserialize, Clone)]
pub struct NotifyDefinition {
    title: String,
    component: Component,
    report_if_success: bool,
}

impl NotifyDefinition {
    pub fn create_message(&self, job_id: &JobDefinitionId, job_result: JobResult) -> Option<Message> {
        let author = Author::parse(format!("rnotifyd/{}", job_id));
        let unix_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Failed to get duration since unix epoch")
            .as_millis();

        if let JobResult::Ok(_) = job_result {
            if !self.report_if_success {
                return None;
            }
        }

        let level = match job_result {
            JobResult::Ok(_) => Level::Info,
            JobResult::Invalid(_) => Level::SelfError,
            JobResult::Failed(_) => Level::Error,
        };

        Some(Message::new(level, Some(self.title.clone()), job_result.take_detail(),
                     Some(self.component.clone()), author,
                     unix_timestamp as i64))
    }
}