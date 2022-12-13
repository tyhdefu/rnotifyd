use std::str::FromStr;
use log::error;
use rnotifylib::message::formatted_detail::{FormattedString, Style};
use rnotifylib::message::message_detail_builder::MessageDetailBuilder;
use rnotifylib::message::MessageDetail;
use crate::program_output::ProgramOutput;

/// Converts the program output into a list of failures and successes.
pub fn to_detail_from_list(output: &ProgramOutput) -> MessageDetail {
    let mut successful_components = vec![];
    let mut failed_components = vec![];

    for (i, line) in output.std_out.lines().enumerate() {
        let mut split = line.splitn(3, ":");

        let component = split.next();
        let result: Option<ComponentResult> = split.next()
            .map(|str| str.parse().ok())
            .flatten();

        if component.is_none() || result.is_none() {
            // TODO: Provide information about what command / job was run.
            error!("Invalid list output. Please ensure that programs conform to the list output format.");
            return generate_invalid_format_message(output, i + 1);
        }
        let component = component.unwrap();
        let result = result.unwrap();

        match result {
            ComponentResult::Success => {
                successful_components.push(component)
            }
            ComponentResult::Failure => {
                let reason = split.next().unwrap_or("No reason provided");
                failed_components.push((component, reason))
            }
        }
    }

    let mut builder = MessageDetailBuilder::with_raw(output.get_stdout().to_string());

    builder = builder.section("Failed components", |s| {
        if failed_components.is_empty() {
            s.append_plain("None");
        }
        for (i, (failure, reason)) in failed_components.iter().enumerate() {
            if i != 0 {
                s.append_plain("\n");
            }

            s.append_plain(format!("- {} ", failure));
            s.append_styled(reason, Style::Monospace);
        }
    });

    builder = builder.section("Successful components", |s| {
        if successful_components.is_empty() {
            s.append_plain("None");
        }
        for (i, success) in successful_components.iter().enumerate() {
            if i != 0 {
                s.append_plain("\n");
            }

            s.append_plain(format!("- {}", success));
        }
    });

    if !output.get_stderr().is_empty() {
        builder = builder.section("Stderr", |s| {
            s.append_styled(output.get_stderr(), Style::Monospace);
        });
    }
    builder.build()
}

enum ComponentResult {
    Success,
    Failure,
}

impl FromStr for ComponentResult {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(ComponentResult::Success),
            "failure" => Ok(ComponentResult::Failure),
            _ => Err(())
        }
    }
}

fn generate_invalid_format_message(output: &ProgramOutput, problem_line_num: usize) -> MessageDetail {
    MessageDetailBuilder::with_raw(format!("Output from program did not conform to output format. Encountered first issue on line {}", problem_line_num))
        .section("Received stdout", |s| {
            s.append_styled(output.get_stdout(), Style::Monospace);
        })
        .section("Stderr (Not parsed)", |s| {
            s.append_styled(output.get_stderr(), Style::Monospace);
        })
        .build()
}