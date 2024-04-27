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
use serde_json::Value;

use crate::arena_event_parser::process_arena_event;
use crate::match_event_handler::{echo_game_state, process_match_event};
use crate::mtga_events::gre::{Annotation, GameObject, MulliganType, TurnInfo, Zone};
use crate::mtga_events::mgrc_event::{Player, ResultList};
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

#[derive(Debug, Event)]
struct GameStateUpdated;

#[derive(Debug, Clone, PartialEq, Event)]
pub enum MatchEvent {
    MatchBegin {
        match_id: String,
        players: Vec<Player>,
    },
    MatchComplete {
        match_id: String,
        result_list: Vec<ResultList>,
    },
    StartingPlayerResponse(i32),
    ClientAction {
        action_type: String,
        card_name: String,
    },
    ServerMulliganRequest {
        cards_in_hand: i32,
        seat_id: i32,
        mulligan_count: i32,
        mulligan_type: MulliganType,
    },
    MulliganDecision(String),
    DeckMessage(Vec<i32>, Vec<i32>),
    GameStateMessage {
        game_state_id: i32,
        annotations: Vec<Annotation>,
        game_objects: Vec<GameObject>,
        zones: Vec<Zone>,
        turn_info: Option<TurnInfo>,
    },
}

#[derive(Debug, Resource)]
pub struct CardsDatabase {
    pub db: Value,
}

impl CardsDatabase {
    pub fn new() -> Self {
        let cards_db_path = "data/cards.json";
        let cards_db_file = File::open(cards_db_path).unwrap();
        let cards_db_reader = BufReader::new(cards_db_file);
        let cards_db: Value = serde_json::from_reader(cards_db_reader).unwrap();
        let cards_db = cards_db.get("cards").unwrap().clone();

        Self {
            db: cards_db,
        }
    }

    pub fn get_pretty_name(&self, grp_id: &str) -> Result<String> {
        self.db.get(grp_id)
            .ok_or_else(|| anyhow::anyhow!("Card not found in database"))
            .and_then(|card| card.get("pretty_name")
                .ok_or_else(|| anyhow::anyhow!("Card does not have a pretty name"))
                .and_then(|pretty_name| pretty_name.as_str()
                    .ok_or_else(|| anyhow::anyhow!("Pretty name is not a string"))
                    .map(|pretty_name| pretty_name.to_string())))
    }
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
    let arena_event_processor = CardsDatabase::new();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_crossbeam_event::<LogEvent>()
        .add_crossbeam_event::<JsonEvent>()
        .add_crossbeam_event::<ArenaEvent>()
        .add_crossbeam_event::<MatchEvent>()
        .add_crossbeam_event::<GameStateUpdated>()
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
        .add_systems(Update, (process_match_event, echo_game_state).chain())
        .add_systems(Update, exit_system)
        .run();

    Ok(())
}

