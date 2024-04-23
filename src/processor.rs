use crossbeam::channel::{Receiver, Sender};
use std::fs::File;
use crossbeam::select;
use serde_json::Value;
use std::io::{BufWriter, Write};

const REQUEST: &str = "request";
const PAYLOAD: &str = "Payload";

#[derive(Debug)]
pub struct LogProcessor {
    lines_rx: Receiver<String>,
    json_tx: Sender<String>,
    json_rx: Receiver<String>,
    arena_event_tx: Sender<Value>,
    writer: File,
    current_json_str: Option<String>,
    bracket_depth: usize,
}

impl LogProcessor {
    pub fn new(
        lines_rx: Receiver<String>,
        json_tx: Sender<String>,
        json_rx: Receiver<String>,
        arena_event_tx: Sender<Value>,
        writer: File,
    ) -> Self {
        Self {
            lines_rx,
            json_tx,
            json_rx,
            arena_event_tx,
            writer,
            current_json_str: None,
            bracket_depth: 0,
        }
    }

    // not sure if it's better to seperate the 2 parts of this of this processor
    // but meh, seems like a non-issue.
    pub fn process(&mut self) {
        loop {
            select! {
                recv(self.lines_rx) -> msg => {
                    match msg {
                        Ok(line) => {
                            self.process_line(line);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                recv(self.json_rx) -> msg => {
                    match msg {
                        Ok(json_str) => {
                            match self.clean_json(json_str) {
                                Ok(json_value) => {
                                    self.write_json(json_value);
                                }
                                Err(e) => {
                                    eprintln!("Error cleaning json: {:?}", e);
                                }
                            }
                        }
                        Err(_) => {
                            break
                        }
                    }
                }
            }
        }
    }

    /// some json logs are nested and re-encoded as strings, this function will attempt to clean them up
    fn clean_json(&self, json_log_str: String) -> anyhow::Result<Value> {
        let mut decoded_value = serde_json::from_str::<Value>(&json_log_str)?;
        if let Some(request) = decoded_value.get(REQUEST).unwrap_or(&Value::Null).as_str() {
            let mut decoded_request = serde_json::from_str::<Value>(request)?;
            if let Some(payload) = decoded_request
                .get(PAYLOAD)
                .unwrap_or(&Value::Null)
                .as_str()
            {
                let decoded_payload = serde_json::from_str::<Value>(payload)?;
                decoded_request[PAYLOAD] = decoded_payload;
            }
            decoded_value[REQUEST] = decoded_request;
        }
        Ok(decoded_value)
    }

    fn write_json(&mut self, json_value: Value) {
        let json_str = serde_json::to_string(&json_value).unwrap();
        let mut writer = BufWriter::new(&self.writer);
        writer.write_all(json_str.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
        self.arena_event_tx.send(json_value).unwrap();
    }

    // try to find the json strings in the logs.
    // they could be seperated by newlines and such we have this LeetCode-ass looking solution
    // to account for that.
    fn process_line(&mut self, line: String) {
        for char in line.chars() {
            match char {
                '{' => {
                    if self.current_json_str.is_none() {
                        self.current_json_str = Some(String::new());
                    }
                    self.current_json_str.as_mut().unwrap().push('{');
                    self.bracket_depth += 1;
                }
                '}' => {
                    if let Some(json_str) = &mut self.current_json_str {
                        json_str.push('}');
                        self.bracket_depth -= 1;
                        if self.bracket_depth == 0 {
                            self.json_tx.send(json_str.clone()).unwrap();
                            self.current_json_str = None;
                        }
                    }
                }
                ' ' | '\n' | '\r' => {}
                _ => {
                    if let Some(json_str) = &mut self.current_json_str {
                        json_str.push(char);
                    }
                }
            }
        }
    }
}
