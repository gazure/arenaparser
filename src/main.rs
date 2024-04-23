use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use crossbeam::channel::{Receiver, select, Sender, unbounded};
use notify::{
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use notify::event::ModifyKind;
use serde_json::Value;

const REQUEST: &str = "request";
const PAYLOAD: &str = "Payload";

#[derive(Debug)]
struct JsonProcessor {
    json_rx: Receiver<String>,
    writer: File,
}

impl JsonProcessor {
    fn new(json_rx: Receiver<String>, writer: File) -> Self {
        Self { json_rx, writer }
    }


    /// some json logs are nested and re-encoded as strings, this function will attempt to clean them up
    fn clean_json(&self, json_log_str: String) -> Result<Value> {
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

    fn json_processor(&mut self) {
        loop {
            match self.json_rx.recv() {
                Ok(json_str) => match self.clean_json(json_str) {
                    Ok(json_value) => {
                        self.write_json(json_value);
                    }
                    Err(e) => {
                        eprintln!("Error cleaning json: {:?}", e);
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }
    }

    fn write_json(&mut self, json_value: Value) {
        let json_str = serde_json::to_string(&json_value).unwrap();
        let mut writer = BufWriter::new(&self.writer);
        writer.write_all(json_str.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }
}

#[derive(Debug)]
struct LogProcessor {
    lines_rx: Receiver<String>,
    json_tx: Sender<String>,
    current_json_str: Option<String>,
    bracket_depth: usize,
}

impl LogProcessor {
    fn new(lines_rx: Receiver<String>, json_tx: Sender<String>) -> Self {
        Self {
            lines_rx,
            json_tx,
            current_json_str: None,
            bracket_depth: 0,
        }
    }

    fn log_line_processor(&mut self) {
        loop {
            match self.lines_rx.recv() {
                Ok(line) => {
                    self.process_line(line);
                },
                Err(_) => {
                    break;
                }
            }
        }
    }

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

fn get_log_lines(reader: &mut impl BufRead) -> Vec<String> {
    // read lines from reader
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => lines.push(line),
            Err(e) => {
                eprintln!("Error reading line: {:?}", e);
                break;
            }
        }
    }
    lines
}

#[derive(Debug, Parser)]
#[command(about = "Tries to scrap useful data from mtga detailed logs")]
struct Args {
    #[arg(short, long)]
    player_log_path: PathBuf,
    #[arg(short, long)]
    output_path: PathBuf,
}

fn ctrl_c_channel() -> Result<Receiver<()>> {
    let (ctrl_c_tx, ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        ctrl_c_tx.send(()).unwrap_or(());
    })?;
    Ok(ctrl_c_rx)
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let log_source = File::open(&args.player_log_path)?;
    let output = File::options().append(true).create(true).open(&args.output_path)?;
    let mut reader = BufReader::new(log_source);

    let ctrl_c_rx = ctrl_c_channel()?;
    let (file_tx, file_rx) = unbounded();
    let (lines_tx, lines_rx) = unbounded();
    let (json_tx, json_rx) = unbounded();
    let mut processor = LogProcessor::new(lines_rx, json_tx);
    let mut json_processor = JsonProcessor::new(
        json_rx,
        output
    );

    std::thread::spawn(move || {
        processor.log_line_processor();
    });
    std::thread::spawn(move || {
        json_processor.json_processor();
    });

    let mut watcher= RecommendedWatcher::new(file_tx, Config::default())?;
    watcher.watch(args.player_log_path.as_ref(), RecursiveMode::NonRecursive)?;

    loop {
        select! {
            recv(ctrl_c_rx) -> _ => {
                eprintln!("exiting...");
                break;
            }
            recv(file_rx) -> msg => {
                match msg {
                    Ok(Ok(event)) => {
                        if event.paths.contains(&args.player_log_path) {
                            match event.kind {
                                EventKind::Create(_) => println!("Log rotated, unsure what to do with this yet"),
                                EventKind::Modify(ModifyKind::Any) => {
                                    let lines = get_log_lines(&mut reader);
                                    for line in lines {
                                        lines_tx.send(line).unwrap();
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            eprintln!("event did not contain Player.log");
                        }
                    }
                    Ok(Err(e)) => eprintln!("Ok Error? {:?}", e),
                    Err(e) => {
                        eprintln!("watch error: {:?}", e);
                        break;
                    }
                }
            }
            default(std::time::Duration::from_secs(1)) => {
                // Windows is really stingy about FS events apparently, let's just read the file every second
                let lines = get_log_lines(&mut reader);
                for line in lines {
                    lines_tx.send(line).unwrap();
                }
            }
        }
    }
    Ok(())
}
