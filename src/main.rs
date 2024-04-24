use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use crossbeam::channel::{Receiver, select, unbounded};
use notify::event::ModifyKind;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use crate::arena_event_parser::ArenaEventParser;
use crate::processor::LogProcessor;

mod processor;
mod arena_event_parser;
mod gre;
mod mgrc_event;
mod client_event;


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

    let ctrl_c_rx = ctrl_c_channel()?;
    let (file_tx, file_rx) = unbounded();
    let (lines_tx, lines_rx) = unbounded();
    let (json_tx, json_rx) = unbounded();
    let (arena_event_tx, arena_event_rx) = unbounded();

    let mut log_processor = LogProcessor::new(lines_rx, json_tx, json_rx, arena_event_tx, output);
    let mut arena_event_processor = ArenaEventParser::new(arena_event_rx);

    std::thread::spawn(move || {
        log_processor.process();
    });
    std::thread::spawn(move || {
        arena_event_processor.process();
    });

    let mut watcher = RecommendedWatcher::new(file_tx, Config::default())?;
    watcher.watch(args.player_log.as_ref(), RecursiveMode::NonRecursive)?;

    loop {
        select! {
            recv(ctrl_c_rx) -> _ => {
                eprintln!("exiting...");
                break;
            }
            recv(file_rx) -> msg => {
                match msg {
                    Ok(Ok(event)) => {
                        if event.paths.contains(&args.player_log) {
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
