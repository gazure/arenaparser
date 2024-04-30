use std::collections::BTreeMap;
use ap_core::mtga_events::gre::{GameObject, GREToClientMessage, TurnInfo, Zone};
use ap_core::mtga_events::gre::Annotation;
use ap_core::mtga_events::gre::MulliganType;
use ap_core::mtga_events::mgrsc::{ResultList, StateType};
use ap_core::mtga_events::mgrsc::Player;
use std::path::PathBuf;

use anyhow::Result;
use bevy::app::AppExit;
use bevy::prelude::*;
use clap::Parser;
use crossbeam::channel::{Receiver, unbounded};
use ap_core::CardsDatabase;
use ap_core::mtga_events::client::ClientMessage;

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
    match_replay_log: PathBuf,
}

#[derive(Debug, Resource)]
struct CardsDB (CardsDatabase);



#[derive(Debug, Resource)]
struct MatchLogEvents{
    mgrsc_events: Vec<ap_core::mtga_events::mgrsc::RequestTypeMGRSCEvent>,
    gre_events: Vec<GREToClientMessage>,
    client_events: Vec<ClientMessage>,
}

#[derive(Debug, Component)]
struct Team(i32);

#[derive(Debug, Component)]
struct PlayerComp(String);

impl Default for MatchLogEvents {
    fn default() -> Self {
        Self {
            mgrsc_events: Vec::new(),
            gre_events: Vec::new(),
            client_events: Vec::new(),
        }
    }
}


fn ctrl_c_channel() -> Result<Receiver<()>> {
    let (ctrl_c_tx, ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        ctrl_c_tx.send(()).unwrap_or(());
    })?;
    Ok(ctrl_c_rx)
}

fn setup(mle: Res<MatchLogEvents>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    for mgrsc_event in &mle.mgrsc_events {
        if mgrsc_event.mgrsc_event.game_room_info.state_type == StateType::Playing {
            let players = mgrsc_event.mgrsc_event.game_room_info.players.clone().unwrap_or(Vec::new());
            for (i, player) in players.iter().enumerate() {
                let style = if i == 0 {
                    Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(10.0),
                        left: Val::Px(10.0),
                        ..default()
                    }
                } else {
                    Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.0),
                        left: Val::Px(10.0),
                        ..default()
                    }
                };

                commands.spawn((
                    PlayerComp(player.player_name.clone()),
                    Team(player.team_id.clone()),
                    TextBundle::from_section(
                        player.player_name.clone(),
                        TextStyle {
                            font_size: 40.0,
                            color: Color::AZURE,
                            ..default()
                        },
                    ).with_style(style),
                ));
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let replay_log = std::fs::read_to_string(args.match_replay_log)?;
    let mut match_log_events = MatchLogEvents::default();

    for line in replay_log.split('\n') {
        let parse_output = ap_core::arena_event_parser::parse(line)?;
        if let Some(gre_message) = parse_output.gre_message {
            let messages = gre_message.gre_to_client_event.gre_to_client_messages;
            for message in messages {
                match_log_events.gre_events.push(message);
            }
        }
        if let Some(client_message) = parse_output.client_message {
            match_log_events.client_events.push(client_message.payload);
        }
        if let Some(mgrc_message) = parse_output.mgrc_message {
            match_log_events.mgrsc_events.push(mgrc_message);
        }
    }




    let db = CardsDB(CardsDatabase::new()?);

    let ctrl_c_rx = ctrl_c_channel()?;
    let exit_system = move |mut exit: EventWriter<AppExit>| {
        if ctrl_c_rx.try_recv().is_ok() {
            exit.send(AppExit);
        }
    };


    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(db)
        .insert_resource(match_log_events)
        .add_systems(Startup, setup)
        // .add_systems(Update, (process_match_event, echo_game_state).chain())
        .add_systems(Update, exit_system)
        .run();

    Ok(())
}

