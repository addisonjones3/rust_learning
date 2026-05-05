use serde::Deserialize;
use std::collections::hash_map::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    STARTED,
    ENDED,
    ERRORED,
}

pub struct ParseStatusError;

impl FromStr for Status {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "STARTED" => Ok(Self::STARTED),
            "ENDED" => Ok(Self::ENDED),
            "ERRORED" => Ok(Self::ERRORED),
            _ => Err(ParseStatusError),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateCommand {
    SetEnd,
    SetStatus,
    AddTag,
}

#[derive(Debug)]
pub struct ParseUpdateCommandError;

impl FromStr for UpdateCommand {
    type Err = ParseUpdateCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SET_END" => Ok(Self::SetEnd),
            "SET_STATUS" => Ok(Self::SetStatus),
            "ADD_TAG" => Ok(Self::AddTag),
            _ => Err(ParseUpdateCommandError),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SpanSummary {}

pub enum SpanError {
    ParseCommand(ParseUpdateCommandError),
    ParseStatus(ParseStatusError),
    EndTimeParse(String),
    EndTimeBeforeStart(usize),
    SpanNotFound(String),
    BadTagFormat,
}

#[derive(Clone)]
struct Span {
    span_id: String,
    trace_id: String,
    service_name: String,
    start_time: usize,
    end_time: Option<usize>,
    status: Status,
    tags: HashMap<String, String>,
}

impl Span {
    pub fn new(span_id: String, trace_id: String, service_name: String, start_time: usize) -> Span {
        Span {
            span_id,
            trace_id,
            service_name,
            start_time,
            end_time: None,
            status: Status::STARTED,
            tags: HashMap::new(),
        }
    }

    fn update(&mut self, status: Status) {
        match (&self.status, status) {
            (Status::STARTED, Status::ENDED) => {
                self.status = Status::ENDED;
            }
            (Status::STARTED, Status::ERRORED) => {
                self.status = Status::ERRORED;
            }
            (Status::STARTED, _) => {}
            (Status::ENDED, _) => {}
            (Status::ERRORED, _) => {}
        }
    }

    fn add_tag(&mut self, key: String, val: String) {
        let _ = &self.tags.entry(key).or_insert(val);
    }

    fn set_end(&mut self, time: usize) -> Result<(), SpanError> {
        if time < self.start_time {
            return Err(SpanError::EndTimeBeforeStart(time));
        }
        self.end_time = Some(time);
        Ok(())
    }

    pub fn get_summary(self) -> String {
        let mut tags_summary = String::from("none");
        let mut tags_iter = self.tags.iter().peekable();
        if tags_iter.peek().is_some() {
            let mut tags = tags_iter
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>();
            tags.sort();
            tags_summary = tags.join(" ");
        }

        let end_time: String = match self.end_time {
            Some(t) => t.to_string(),
            None => String::from("-1"),
        };
        format!(
            "{0} {1} {2} {3:?} {4} {5}",
            self.span_id, self.trace_id, self.service_name, self.status, end_time, tags_summary,
        )
    }
}

#[derive(Default)]
pub struct SpanCollection {
    spans: HashMap<String, Span>,
    count_started: usize,
    count_ended: usize,
    count_errored: usize,
}

// impl Default for SpanCollection {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl SpanCollection {
    pub fn new() -> SpanCollection {
        SpanCollection {
            spans: HashMap::new(),
            count_started: 0,
            count_ended: 0,
            count_errored: 0,
        }
    }
    pub fn handle_command(mut self, args: Vec<String>) -> Result<(), SpanError> {
        let command_str: &str = args[0].as_str();
        let command = match UpdateCommand::from_str(command_str) {
            Ok(c) => c,
            Err(e) => return Err(SpanError::ParseCommand(e)),
        };

        let span_id: &str = args[1].as_str();
        let span = match self.spans.get_mut(span_id) {
            Some(s) => s,
            None => return Err(SpanError::SpanNotFound(span_id.to_string())),
        };

        match command {
            UpdateCommand::SetEnd => {
                let end_time_str: &str = args[2].as_str();
                match end_time_str.parse::<usize>() {
                    Ok(end_time) => span.set_end(end_time)?,
                    Err(_) => return Err(SpanError::EndTimeParse(end_time_str.to_string())),
                };
            }
            UpdateCommand::SetStatus => {
                let stats_str: &str = args[2].as_str();
                match Status::from_str(stats_str) {
                    Ok(status) => {
                        span.update(status);
                        match status {
                            Status::ENDED => self.count_ended += 1,
                            Status::ERRORED => self.count_errored += 1,
                            _ => {}
                        }
                    }
                    Err(e) => return Err(SpanError::ParseStatus(e)),
                };
            }
            UpdateCommand::AddTag => {
                if args[2..].len() != 2 {
                    eprintln!("invalid format for add tag");
                    return Err(SpanError::BadTagFormat);
                }

                span.add_tag(args[2].clone(), args[3].clone());
            }
        };

        Ok(())
    }

    pub fn new_span(
        &mut self,
        span_id: String,
        trace_id: String,
        service_name: String,
        start_time: usize,
    ) {
        let _ = self.spans.entry(span_id.clone()).or_insert(Span::new(
            span_id,
            trace_id,
            service_name,
            start_time,
        ));
        self.count_started += 1;
    }

    pub fn update_span(&mut self, span_id: String, status: Status) -> Result<(), SpanError> {
        let span = self.get_span(span_id)?;
        span.update(status);
        Ok(())
    }

    fn get_span(&mut self, span_id: String) -> Result<&mut Span, SpanError> {
        match self.spans.get_mut(&span_id) {
            Some(s) => Ok(s),
            None => Err(SpanError::SpanNotFound(span_id)),
        }
    }

    pub fn summarize_all(&self) -> String {
        let mut span_summaries: Vec<String> = self
            .spans
            .values()
            .map(|s| s.clone().get_summary())
            .collect();
        span_summaries.sort();
        span_summaries.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const S1_START_TIME: usize = 1000;
    const S1_NAME: &str = "AuthService";
    const S1_TRACE_ID: &str = "t1";
    const S1_SPAN_ID: &str = "s1";

    fn new_s1() -> Span {
        Span::new(
            String::from(S1_SPAN_ID),
            String::from(S1_TRACE_ID),
            String::from(S1_NAME),
            S1_START_TIME,
        )
    }

    #[test]
    fn new_spans() {
        let s1 = new_s1();
        assert_eq!(s1.span_id, S1_SPAN_ID);
        assert_eq!(s1.trace_id, S1_TRACE_ID);
        assert_eq!(s1.status, Status::STARTED);
        assert_eq!(s1.start_time, S1_START_TIME);
        assert_eq!(s1.service_name, S1_NAME);
    }

    #[test]
    fn span_set_end() {
        let mut s1 = new_s1();
        s1.update(Status::ENDED);
        assert_eq!(s1.status, Status::ENDED)
    }

    #[test]
    fn span_set_error_after_end() {
        let mut s1 = new_s1();
        s1.update(Status::ENDED);
        s1.update(Status::ERRORED);
        assert_eq!(s1.status, Status::ENDED)
    }

    #[test]
    fn summary_format() {
        let mut s1 = new_s1();
        s1.add_tag("env".to_owned(), "production".to_owned());
        s1.add_tag("region".to_owned(), "us-east1".to_owned());

        let s1_summary = s1.get_summary();
        let expected_summary: &str = "s1 t1 AuthService STARTED -1 env=production region=us-east1";
        assert_eq!(s1_summary, expected_summary);
    }
}
