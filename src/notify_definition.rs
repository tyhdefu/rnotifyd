use std::time::{SystemTime, UNIX_EPOCH};
use rnotifylib::message::author::Author;
use rnotifylib::message::component::Component;
use rnotifylib::message::{Level, Message, MessageDetail};
use rnotifylib::message::formatted_detail::{FormattedMessageComponent, FormattedMessageDetail, FormattedString, Style};
use crate::config::JobDefinitionId;
use serde::{Serialize, Deserialize};
use crate::program_output::ProgramOutput;

#[derive(Serialize, Deserialize, Clone)]
pub struct NotifyDefinition {
    title: String,
    component: Component,
    level: Level,
    message_generator: MessageGenerator,
}

impl NotifyDefinition {
    pub fn create_message(&self, job_id: &JobDefinitionId, output: ProgramOutput) -> Option<Message> {
        let author = Author::parse(format!("rnotifyd/{}", job_id));
        let unix_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Failed to get duration since unix epoch")
            .as_millis();

        let detail = self.message_generator.generate(output);
        if detail.is_none() {
            return None;
        }

        Some(Message::new(self.level.clone(), Some(self.title.clone()), detail.unwrap(),
                     Some(self.component.clone()), author,
                     unix_timestamp as i64))
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MessageGenerator {
    FromOutputBasic,
}

impl MessageGenerator {
    pub fn generate(&self, output: ProgramOutput) -> Option<MessageDetail> {
        // TODO: Check if program errored.

        let raw = format!("{:?}", output);
        let mut components = vec![];
        components.push(FormattedMessageComponent::Text(vec![FormattedString::new(format!("Failed with non-zero exit code {:?}", output.get_exit_status()), vec![])]));

        let std_err = FormattedMessageComponent::Section("Stderr".to_owned(), vec![FormattedString::new(output.get_stderr().to_owned(), vec![Style::Monospace])]);
        let std_out = FormattedMessageComponent::Section("Stdout".to_owned(), vec![FormattedString::new(output.get_stdout().to_owned(), vec![Style::Monospace])]);

        components.push(std_err);
        components.push(std_out);

        Some(MessageDetail::Formatted(FormattedMessageDetail::new(raw, components)))
    }
}