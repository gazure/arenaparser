use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_crossbeam_event::CrossbeamEventApp;
use clap::Parser;
use crossbeam::channel::{Receiver, unbounded};

use crate::arena_event_parser::{ArenaEventParser, MatchEvent, process_arena_event};
use crate::match_event_handler::process_match_event;
use crate::processor::{clean_json, LogProcessor, process_line};

mod arena_event_parser;
mod match_event_handler;
mod mtga_events;
mod processor;

#[derive(Debug, Resource)]
struct LogReader {
    reader: BufReader<File>,
    timer: Timer,
}

#[derive(Debug, Event)]
struct LogEvent {
    line: String,
}

#[derive(Debug, Event)]
struct JsonEvent {
    json_str: String,
}

#[derive(Debug, Event)]
struct ArenaEvent {
    event: String,
}

fn check_new_log_lines(
    mut lr: ResMut<LogReader>,
    time: Res<Time>,
    mut lines_tx: EventWriter<LogEvent>,
) {
    lr.timer.tick(time.delta());
    if lr.timer.finished() {
        let mut reader = &mut lr.reader;
        let lines = get_log_lines(&mut reader);
        for line in lines {
            lines_tx.send(LogEvent { line });
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
    let reader = BufReader::new(log_source);

    let ctrl_c_rx = ctrl_c_channel()?;
    let exit_system = move |mut exit: EventWriter<AppExit>| {
        if ctrl_c_rx.try_recv().is_ok() {
            exit.send(AppExit);
        }
    };

    let log_processor = LogProcessor::new(output);
    let arena_event_processor = ArenaEventParser::new();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_crossbeam_event::<LogEvent>()
        .add_crossbeam_event::<JsonEvent>()
        .add_crossbeam_event::<ArenaEvent>()
        .add_crossbeam_event::<MatchEvent>()
        .insert_resource(LogReader {
            reader,
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        })
        .insert_resource(log_processor)
        .insert_resource(arena_event_processor)
        .add_systems(Update, check_new_log_lines)
        .add_systems(Update, process_line)
        .add_systems(Update, clean_json)
        .add_systems(Update, process_arena_event)
        .add_systems(Update, process_match_event)
        .add_systems(Update, exit_system)
        .run();

    Ok(())
}
