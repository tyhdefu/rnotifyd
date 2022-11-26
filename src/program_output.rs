use rnotifylib::message::formatted_detail::{FormattedString, Style};
use rnotifylib::message::message_detail_builder::MessageDetailBuilder;
use rnotifylib::message::MessageDetail;
use crate::action::ProgramOutputFormat;

#[derive(Debug)]
pub struct ProgramOutput {
    std_out: String,
    std_err: String,
    exit_code: i32,
}

impl ProgramOutput {
    pub fn new(std_out: String, std_err: String, exit_code: i32) -> Self {
        Self {
            std_out,
            std_err,
            exit_code
        }
    }

    pub fn trim_to(&mut self, max_len: usize) {
        Self::trim_string(&mut self.std_out, max_len);
        Self::trim_string(&mut self.std_err, max_len);
    }

    fn trim_string(s: &mut String, max_len: usize) {
        let len = s.len();
        if len > max_len {
            let hope_to_chop_at_start = len - max_len;

            let chop_start = s.char_indices()
                .map(|(i, _c)| i)
                .filter(|i| i > &hope_to_chop_at_start)
                .min();

            if let Some(chop_start) = chop_start {
                *s = format!("...{}", &s[chop_start..]);
            }
        }
    }


    pub fn get_stdout(&self) -> &str {
        &self.std_out
    }

    pub fn get_stderr(&self) -> &str {
        &self.std_err
    }

    pub fn get_exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn to_detail(mut self, format: &ProgramOutputFormat) -> MessageDetail {
        self.trim_to(500);
        let suc = self.is_success();
        match (format, suc) {
            (ProgramOutputFormat::SimpleIfSuccess, true) => MessageDetail::Raw("Program Succeeded".to_owned()),
            (ProgramOutputFormat::SimpleIfSuccess, false) => to_detail_verbose(&self),

            (ProgramOutputFormat::StdoutIfSuccess, true) => {
                MessageDetailBuilder::new()
                    .text(vec![FormattedString::plain("Program Succeeded")])
                    .section("Stdout", |section| {
                        section.append_styled(self.std_out, Style::Monospace);
                    })
                    .build()
            },
            (ProgramOutputFormat::StdoutIfSuccess, false) => to_detail_verbose(&self),
            (ProgramOutputFormat::AlwaysDetailed, _) => to_detail_verbose(&self),
        }
    }
}

fn to_detail_verbose(output: &ProgramOutput) -> MessageDetail {
    let success = output.is_success();
    let raw = format!("{:?}", output);

    let exit_code_str = format!("exit code {:?}", output.get_exit_code());
    let topline = format!("Program {} with {}", if success {"successful"} else {"failed"}, exit_code_str);

    MessageDetailBuilder::with_raw(raw)
        .text(vec![FormattedString::plain(topline)])
        .section("Stderr", |section| {
            section.append_styled(output.get_stderr(), Style::Monospace);
        })
        .section("Stdout", |section| {
            section.append_styled(output.get_stdout(), Style::Monospace);
        })
        .build()
}