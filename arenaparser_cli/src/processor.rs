use std::fs::File;
use std::io::{BufWriter, Write};

use serde_json::Value;

const REQUEST: &str = "request";
const PAYLOAD: &str = "Payload";

#[derive(Debug)]
pub struct LogProcessor {
    writer: File,
    current_json_str: Option<String>,
    bracket_depth: usize,
}

impl LogProcessor {
    pub fn new(
        writer: File,
    ) -> Self {
        Self {
            writer,
            current_json_str: None,
            bracket_depth: 0,
        }
    }

    pub fn write_json(&mut self, json_str: String) {
        let mut writer = BufWriter::new(&self.writer);
        writer.write_all(json_str.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }
}

/// some json logs are nested and re-encoded as strings, this function will attempt to clean them up
pub fn clean_json(json_event: &str) -> Option<String> {
    if let Ok(mut decoded_value) = serde_json::from_str::<Value>(json_event) {
        if let Some(request) = decoded_value.get(REQUEST).unwrap_or(&Value::Null).as_str() {
            if let Ok(mut decoded_request) = serde_json::from_str::<Value>(request) {
                if let Some(payload) = decoded_request
                    .get(PAYLOAD)
                    .unwrap_or(&Value::Null)
                    .as_str()
                {
                    if let Ok(decoded_payload) = serde_json::from_str::<Value>(payload) {
                        decoded_request[PAYLOAD] = decoded_payload;
                    }
                }
                decoded_value[REQUEST] = decoded_request;
            }
        }
        Some(serde_json::to_string(&decoded_value).unwrap())
    } else {
        None
    }
}

// try to find the json strings in the logs.
// they could be seperated by newlines and such
// so we have this LeetCode-ass looking solution
// to account for that.
pub fn process_line(processor: &mut LogProcessor, log_line: &str) -> Vec<String>{
    let mut completed_json_strings = Vec::new();
    for char in log_line.chars() {
        match char {
            '{' => {
                if processor.current_json_str.is_none() {
                    processor.current_json_str = Some(String::new());
                }
                processor.current_json_str.as_mut().unwrap().push('{');
                processor.bracket_depth += 1;
            }
            '}' => {
                if let Some(json_str) = &mut processor.current_json_str {
                    json_str.push('}');
                    processor.bracket_depth -= 1;
                    if processor.bracket_depth == 0 {
                        completed_json_strings.push(json_str.clone());
                        processor.current_json_str = None;
                    }
                }
            }
            ' ' | '\n' | '\r' => {}
            _ => {
                if let Some(json_str) = &mut processor.current_json_str {
                    json_str.push(char);
                }
            }
        }
    }
    completed_json_strings
}
