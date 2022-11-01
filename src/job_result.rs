use rnotifylib::message::{MessageDetail};

pub enum JobResult {
    /// The job successfully ran
    Ok(MessageDetail),
    /// The job failed to run because its configuration was wrong.
    Invalid(MessageDetail),
    /// The job ran, and detected a problem.
    Failed(MessageDetail),
}

impl JobResult {
    pub fn take_detail(self) -> MessageDetail {
        match self {
            JobResult::Ok(detail) => detail,
            JobResult::Invalid(detail) => detail,
            JobResult::Failed(detail) => detail,
        }
    }

    pub fn type_str(&self) -> &str {
        match self {
            JobResult::Ok(_) => "ok",
            JobResult::Invalid(_) => "invalid",
            JobResult::Failed(_) => "failed",
        }
    }
}