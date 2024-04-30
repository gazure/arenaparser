#![allow(unused)]
use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use bevy::prelude::*;
use bevy::input::ButtonInput;
use clap::Parser;

use ap_core::mtga_events::client::ClientMessage;
use ap_core::mtga_events::gre::{GREToClientMessage, GameObjectType, GameStateMessage, ZoneType};
use ap_core::mtga_events::mgrsc::StateType;
use ap_core::CardsDatabase;

#[derive(Debug, Component)]
pub struct Abilities(Vec<i32>);

#[derive(Debug, Component)]
pub struct GRP {
    id: i32,
    name: String,
}

#[derive(Debug, Component)]
pub struct GOType(GameObjectType);

#[derive(Debug, Component)]
pub struct Zone(i32);

#[derive(Debug, Component)]
pub struct Owner(i32);

#[derive(Debug, Component)]
pub struct Tapped;

#[derive(Debug, Component)]
pub struct Instance(i32);

#[derive(Debug, Component)]
pub struct ZoneInfo {
    pub id: i32,
    pub owner_id: i32,
    pub default_visibility: Visibility,
    pub type_field: ZoneType,
}

#[derive(Debug, Parser)]
#[command(about = "Tries to scrape useful data from mtga detailed logs")]
struct Args {
    #[arg(short, long)]
    match_replay_log: PathBuf,
}

#[derive(Debug, Resource)]
struct CardsDB(CardsDatabase);

#[derive(Debug)]
struct MatchPlayer {
    player_name: String,
    team_id: i32,
    die_roll_result: i32,
}

#[derive(Debug, Resource)]
struct MatchLogEvents {
    controller_seat_id: i32,
    controller_main_deck: Vec<String>,
    controller_sideboard: Vec<String>,
    players: BTreeMap<i32, MatchPlayer>,

    mgrsc_events: Vec<ap_core::mtga_events::mgrsc::RequestTypeMGRSCEvent>,
    gre_events: Vec<GREToClientMessage>,
    game_state_message_idx: usize,
    game_state_messages: Vec<GameStateMessage>,
    client_events: Vec<ClientMessage>,
}

#[derive(Debug, Component)]
struct Team(i32);

#[derive(Debug, Component)]
struct PlayerComp(String);

impl Default for MatchLogEvents {
    fn default() -> Self {
        Self {
            controller_seat_id: 0,
            controller_main_deck: Vec::new(),
            controller_sideboard: Vec::new(),
            players: BTreeMap::new(),
            mgrsc_events: Vec::new(),
            gre_events: Vec::new(),
            game_state_message_idx: 0,
            game_state_messages: Vec::new(),
            client_events: Vec::new(),
        }
    }
}

fn the_update_system(input: Res<ButtonInput<KeyCode>>, db: Res<CardsDB>, mut commands: Commands, mut mle: ResMut<MatchLogEvents>, ie_query: Query<(Entity, &Instance)>) {
    if input.just_pressed(KeyCode::Space) {
        if let Some(gsm) = mle.game_state_messages.get(mle.game_state_message_idx) {
            for deleted_instance_id in &gsm.diff_deleted_instance_ids {
                for (entity, instance) in ie_query.iter() {
                    if instance.0 == *deleted_instance_id {
                        commands.entity(entity).despawn();
                    }
                }
            }
            let mut new_instance_ids = Vec::<i32>::new();

            for go in &gsm.game_objects {
                let mut go_entity = None;
                for (entity, instance) in ie_query.iter() {
                    if instance.0 == go.instance_id {
                        go_entity = Some(entity);
                        break;
                    }
                }
                let entity = match go_entity {
                    Some(entity) => {
                        let mut e_commands = commands.entity(entity);
                        match go.zone_id {
                            Some(zone_id) => {
                                e_commands.insert(Zone(zone_id));
                            }
                            None => {
                                e_commands.remove::<Zone>();
                            }
                        }
                        e_commands.insert(Owner(go.owner_seat_id));
                        e_commands.insert(GRP{id: go.grp_id, name: db.0.get_pretty_name_defaulted(&go.grp_id.to_string())});
                        match go.is_tapped {
                            Some(true) => {
                                e_commands.insert(Tapped);
                            }
                            _ => {
                                e_commands.remove::<Tapped>();
                            }
                        }
                        entity
                    },
                    None => {
                        let entity_id = commands.spawn((Instance(go.instance_id), Owner(go.owner_seat_id), GRP{id: go.grp_id, name: db.0.get_pretty_name_defaulted(&go.grp_id.to_string())})).id();
                        let mut e_commands = commands.entity(entity_id);
                        match go.zone_id {
                            Some(zone_id) => {
                                e_commands.insert(Zone(zone_id));
                            }
                            None => {
                                e_commands.remove::<Zone>();
                            }
                        }
                        match go.is_tapped {
                            Some(true) => {
                                e_commands.insert(Tapped);
                            }
                            _ => {
                                e_commands.remove::<Tapped>();
                            }
                        }
                        entity_id
                    }
                };
                for zone in &gsm.zones {
                    for instance_id in &zone.object_instance_ids {
                        let mut found_entity = false;
                        for (entity, instance) in ie_query.iter() {
                            if instance.0 == *instance_id {
                                commands.entity(entity).insert(Zone(zone.zone_id));
                                found_entity = true;
                                break;
                            }
                        }
                        if !found_entity {
                            commands.spawn((Instance(*instance_id), Zone(zone.zone_id)));
                        }
                    }
                }
            }
            mle.game_state_message_idx += 1;
            info!("Advanced Game State Message Index to {}", mle.game_state_message_idx);
        }
    }
}

fn the_echo_system(input: Res<ButtonInput<KeyCode>>, mle: Res<MatchLogEvents>, query: Query<(&Instance, &Zone, Option<&GRP>)>) {
    if input.just_pressed(KeyCode::KeyF) {
        let mut zone_to_igrp = BTreeMap::<i32, Vec<i32>>::new();
        for (instance, zone, grp) in query.iter() {
            info!("Instance: {}, Zone: {}, GRP: {:?}", instance.0, zone.0, grp);
        }
    }
}

fn setup(mle: Res<MatchLogEvents>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    for (team_id, player) in mle.players.iter() {
        let style = if *team_id == mle.controller_seat_id {
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
        info!("Player {} rolled a {}", player.player_name, player.die_roll_result);

        commands.spawn((
            PlayerComp(player.player_name.clone()),
            Team(*team_id),
            TextBundle::from_section(
                format!("{}", player.player_name),
                TextStyle {
                    font_size: 40.0,
                    color: Color::AZURE,
                    ..default()
                },
            )
            .with_style(style),
        ));
    }
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let replay_log = std::fs::read_to_string(args.match_replay_log)?;
    let mut match_log_events = MatchLogEvents::default();
    let db = CardsDB(CardsDatabase::new()?);

    for line in replay_log.split('\n') {
        let parse_output = ap_core::arena_event_parser::parse(line)?;
        if let Some(gre_message) = parse_output.gre_message {
            let messages = gre_message.gre_to_client_event.gre_to_client_messages;
            for message in messages {
                match &message {
                    GREToClientMessage::GameStateMessage(gsm_wrapper) => {
                        match_log_events
                            .game_state_messages
                            .push(gsm_wrapper.game_state_message.clone());
                    }
                    GREToClientMessage::ConnectResp(connect_wrapper) => {
                        let seat_id =
                            connect_wrapper
                                .meta
                                .system_seat_ids
                                .get(0)
                                .ok_or(anyhow::anyhow!(
                                "No seat id found, ConnectResp appears to be missing information"
                            ))?;
                        let deck_message = &connect_wrapper.connect_resp.deck_message;
                        let main_deck = deck_message
                            .deck_cards
                            .iter()
                            .map(|card| {
                                db.0.get_pretty_name(&card.to_string())
                                    .unwrap_or(card.to_string())
                            })
                            .collect();
                        let sideboard = deck_message
                            .sideboard_cards
                            .iter()
                            .map(|card| {
                                db.0.get_pretty_name(&card.to_string())
                                    .unwrap_or(card.to_string())
                            })
                            .collect();

                        match_log_events.controller_seat_id = *seat_id;
                        match_log_events.controller_main_deck = main_deck;
                        match_log_events.controller_sideboard = sideboard;
                    }
                    GREToClientMessage::DieRollResults(wrapper) => {
                        let die_roll_results = &wrapper.die_roll_results_resp.player_die_rolls;
                        for die_roll_result in die_roll_results {
                            match_log_events
                                .players
                                .get_mut(&die_roll_result.system_seat_id)
                                .map(|player| player.die_roll_result = die_roll_result.roll_value);
                        }
                    }
                    _ => {}
                }
                match_log_events.gre_events.push(message);
            }
        }
        if let Some(client_message) = parse_output.client_message {
            match_log_events.client_events.push(client_message.payload);
        }
        if let Some(mgrsc_message) = parse_output.mgrc_message {
            if mgrsc_message.mgrsc_event.game_room_info.state_type == StateType::Playing {
                let players = mgrsc_message
                    .mgrsc_event
                    .game_room_info
                    .players
                    .clone()
                    .ok_or(anyhow::anyhow!("No players found in game room info"))?;
                for player in players {
                    match_log_events.players.insert(
                        player.system_seat_id,
                        MatchPlayer {
                            player_name: player.player_name.clone(),
                            team_id: player.team_id,
                            die_roll_result: 0,
                        },
                    );
                }
            }
            match_log_events.mgrsc_events.push(mgrsc_message);
        }
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(db)
        .insert_resource(match_log_events)
        .add_systems(Startup, setup)
        .add_systems(Update, the_update_system)
        .add_systems(Update, the_echo_system)
        .run();

    Ok(())
}
