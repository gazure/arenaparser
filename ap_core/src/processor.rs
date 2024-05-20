
#[derive(Debug)]
pub struct LogProcessor {
    current_json_str: Option<String>,
    bracket_depth: usize,
}

// I have a crippling OOP addiction, don't I
impl LogProcessor {
    pub fn new(
    ) -> Self {
        Self {
            current_json_str: None,
            bracket_depth: 0,
        }
    }

    // try to find the json strings in the logs. ignoring all other info
    // purges whitespace from the internal json strings, but I don't think that will cause
    // any issues given the log entries I've read
    pub fn process_line(&mut self, log_line: &str) -> Vec<String>{
        let mut completed_json_strings = Vec::new();
        for char in log_line.chars() {
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
                            completed_json_strings.push(json_str.clone());
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
        completed_json_strings
    }
}