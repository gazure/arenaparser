use std::fs::File;
use std::io::{BufWriter, Write};

use bevy::prelude::{EventReader, EventWriter, ResMut, Resource};
use serde_json::Value;

use crate::{ArenaEvent, JsonEvent, LogEvent};

const REQUEST: &str = "request";
const PAYLOAD: &str = "Payload";

#[derive(Debug, Resource)]
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

}

/// some json logs are nested and re-encoded as strings, this function will attempt to clean them up
pub fn clean_json(mut lp: ResMut<LogProcessor>, mut json_event_reader: EventReader<JsonEvent>, mut arena_event: EventWriter<ArenaEvent>) {
    for json_event in json_event_reader.read() {
        if let Ok(mut decoded_value) = serde_json::from_str::<Value>(&json_event.json_str) {
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
            write_json(&mut lp.writer, decoded_value.clone());
            arena_event.send(ArenaEvent{event: decoded_value.to_string()});
        }
    }
}

fn write_json(writer: &mut File, json_value: Value) {
    let json_str = serde_json::to_string(&json_value).unwrap();
    let mut writer = BufWriter::new(writer);
    writer.write_all(json_str.as_bytes()).unwrap();
    writer.write_all(b"\n").unwrap();
}

// try to find the json strings in the logs.
// they could be seperated by newlines and such
// so we have this LeetCode-ass looking solution
// to account for that.
pub fn process_line(mut processor: ResMut<LogProcessor>, mut event: EventReader<LogEvent>, mut json_writer: EventWriter<JsonEvent>) {
    for line_event in event.read() {
        for char in line_event.line.chars() {
            match char {
                '{' => {
                    if processor.current_json_str.is_none() {
                        processor.current_json_str = Some(String::new());
                    }
                    processor.current_json_str.as_mut().unwrap().push('{');
                    processor.bracket_depth += 1;
                }
                '}' => {
                    let processor = processor.as_mut();
                    if let Some(json_str) = &mut processor.current_json_str {
                        json_str.push('}');
                        processor.bracket_depth -= 1;
                        if processor.bracket_depth == 0 {
                            json_writer.send(JsonEvent{json_str: json_str.clone()});
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
    }
}
