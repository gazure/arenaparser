use ap_core::mtga_events::gre::{GameObject, TurnInfo, Zone};
use ap_core::mtga_events::gre::Annotation;
use ap_core::mtga_events::gre::MulliganType;
use ap_core::mtga_events::mgrc_event::ResultList;
use ap_core::mtga_events::mgrc_event::Player;
use std::path::PathBuf;

use anyhow::Result;
use bevy::app::AppExit;
use bevy::prelude::*;
use clap::Parser;
use crossbeam::channel::{Receiver, unbounded};
use ap_core::CardsDatabase;


use crate::match_event_handler::{echo_game_state, process_match_event};

mod match_event_handler;

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

#[derive(Debug, Parser)]
#[command(about = "Tries to scrape useful data from mtga detailed logs")]
struct Args {
    #[arg(short, long)]
    player_log: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Debug, Resource)]
struct CardsDB (CardsDatabase);

fn ctrl_c_channel() -> Result<Receiver<()>> {
    let (ctrl_c_tx, ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        ctrl_c_tx.send(()).unwrap_or(());
    })?;
    Ok(ctrl_c_rx)
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let db = CardsDB(CardsDatabase::new());

    let ctrl_c_rx = ctrl_c_channel()?;
    let exit_system = move |mut exit: EventWriter<AppExit>| {
        if ctrl_c_rx.try_recv().is_ok() {
            exit.send(AppExit);
        }
    };


    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(db)
        .add_systems(Update, (process_match_event, echo_game_state).chain())
        .add_systems(Update, exit_system)
        .run();

    Ok(())
}

