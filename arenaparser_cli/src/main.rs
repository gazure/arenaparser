use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossbeam::channel::{Receiver, select, unbounded};

use ap_core::arena_event_parser;
use ap_core::replay::MatchReplayBuilder;

use crate::processor::LogProcessor;

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
    #[arg(short, long, help = "Location of Player.log file")]
    player_log: PathBuf,
    #[arg(short, long, help = "directory to write replay output files")]
    output_dir: PathBuf,
    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "wait for new events on Player.log, useful if you are actively playing MTGA")]
    follow: bool,
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
    let mut reader = BufReader::new(log_source);
    let mut processor = LogProcessor::new();
    let mut match_replay_builder = MatchReplayBuilder::new(args.output_dir);
    let follow = args.follow;

    let ctrl_c_rx = ctrl_c_channel()?;

    loop {
        select! {
            recv(ctrl_c_rx) -> _ => {
                break;
            }
            default(Duration::from_secs(1)) => {
                let lines = get_log_lines(&mut reader);
                for line in lines {
                    let json_lines = processor.process_line(&line);
                    for json_line in json_lines {
                        let parse_output= arena_event_parser::parse(&json_line);
                        match parse_output {
                            Ok(po) => {
                                if let Some(mgrc_message) = po.mgrc_message {
                                    match_replay_builder.ingest_mgrc_event(mgrc_message);
                                }
                                if let Some(gre_message) = po.gre_message {
                                    match_replay_builder.gre_messages.push(gre_message);
                                }
                                if let Some(client_message) = po.client_message {
                                    match_replay_builder.client_messages.push(client_message);
                                }


                            },
                            Err(e) => eprintln!("Error parsing json: {}\n{}", e, &json_line),
                        }
                    }
                }
                if !follow {
                    break;
                }
            }
        }
    }

    Ok(())
}
