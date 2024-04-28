use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossbeam::channel::{Receiver, unbounded, select};

use crate::processor::{clean_json, LogProcessor, process_line};
mod processor;


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
#[command(about = "Tries to scrape useful data from mtga detailed logs")]
struct Args {
    #[arg(short, long)]
    player_log: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
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

    let log_source = File::open(&args.player_log)?;
    let output = File::options()
        .append(true)
        .create(true)
        .open(&args.output)?;
    let mut reader = BufReader::new(log_source);
    let mut processor = LogProcessor::new(output);

    let ctrl_c_rx = ctrl_c_channel()?;


    loop {
        select! {
            recv(ctrl_c_rx) -> _ => {
                break;
            }
            default(Duration::from_secs(1)) => {
                let lines = get_log_lines(&mut reader);
                for line in lines {
                    let json_lines = process_line(&mut processor, &line);
                    for json_line in json_lines {
                        let cleaned_json = clean_json(&json_line);
                        if let Some(cleaned_json) = cleaned_json {
                            processor.write_json(cleaned_json);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

