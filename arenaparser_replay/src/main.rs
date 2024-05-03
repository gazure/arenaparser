#![allow(unused)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::Result;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use clap::Parser;

use ap_core::mtga_events::client::ClientMessage;
use ap_core::mtga_events::gre::{GameObjectType, GameStateMessage, GREToClientMessage};
use ap_core::mtga_events::mgrsc::StateType;
use ap_core::CardsDatabase;
use ap_core::mtga_events::primitives::{Phase, Step, ZoneType};

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
pub struct LifeTotal(i32);

#[derive(Debug, Component)]
pub struct Instance(i32);

#[derive(Debug, Component)]
pub struct ZoneInfo {
    pub id: i32,
    pub owner_id: Option<i32>,
    pub default_visibility: ap_core::mtga_events::primitives::Visibility,
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

#[derive(Debug, Component)]
struct TurnInfo;

impl MatchLogEvents {
    fn get_player_name(&self, seat_id: i32) -> String {
        self.players
            .get(&seat_id)
            .map(|player| player.player_name.clone())
            .unwrap_or("Unknown".to_string())
    }
}

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

fn the_update_system(
    input: Res<ButtonInput<KeyCode>>,
    db: Res<CardsDB>,
    mut commands: Commands,
    mut mle: ResMut<MatchLogEvents>,
    ie_query: Query<(Entity, &Instance)>,
    zone_info_query: Query<(&ZoneInfo)>,
    mut turn_info_query: Query<(&mut Text), (With<TurnInfo>, Without<PlayerComp>)>,
    mut players_life_total_query: Query<
        (&mut Text, &mut LifeTotal, &Team),
        (With<PlayerComp>, Without<TurnInfo>),
    >,
) {
    if input.just_pressed(KeyCode::Space) {
        let mle = mle.into_inner();
        while let Some(gsm) = mle.game_state_messages.get(mle.game_state_message_idx) {
            mle.game_state_message_idx += 1;
            // Always update the turn info if applicable
            if let Some(ti) = &gsm.turn_info {
                for mut turn_info_text in turn_info_query.iter_mut() {
                    if let Some(active) = ti.active_player {
                        let player_name = mle.get_player_name(active);
                        turn_info_text.sections[0].value = player_name;
                    }
                    if let Some(turn_num) = ti.turn_number {
                        let turn_number = turn_num / 2 + turn_num % 2;
                        turn_info_text.sections[2].value = turn_number.to_string();
                    }
                    if let Some(phase) = ti.phase {
                        turn_info_text.sections[4].value = phase.to_string();
                    }
                    turn_info_text.sections[6].value = if let Some(step) = ti.step {
                         step.to_string()
                    } else {
                        "".to_string()
                    }
                }
            }
            if gsm.players.is_empty() && gsm.game_objects.is_empty() && gsm.zones.is_empty() {
                info!("Advanced Game State Message Index to {}, GameStateId: {}  -- no new zones or game objects", mle.game_state_message_idx, gsm.game_state_id);
                continue;
            }

            let game_state_id = gsm.game_state_id;
            for player in &gsm.players {
                if let Some((mut text, mut life_total, team)) = players_life_total_query
                    .iter_mut()
                    .find(|(_, _, team_id)| team_id.0 == player.team_id)
                {
                    life_total.0 = player.life_total;
                    text.sections[1].value = life_total.0.to_string();
                }
            }

            for deleted_instance_id in &gsm.diff_deleted_instance_ids {
                for (entity, instance) in ie_query.iter() {
                    if instance.0 == *deleted_instance_id {
                        commands.entity(entity).despawn();
                    }
                }
            }
            let mut new_instances = BTreeMap::new();

            for go in &gsm.game_objects {
                let mut go_entity = None;
                for (entity, instance) in ie_query.iter() {
                    if instance.0 == go.instance_id {
                        go_entity = Some(entity);
                        break;
                    }
                }
                let entity = go_entity.unwrap_or_else(|| {
                    let id = commands.spawn((Instance(go.instance_id),)).id();
                    new_instances.insert(go.instance_id, id);
                    id
                });
                let mut e_commands = commands.entity(entity);
                e_commands.insert(GRP {
                    id: go.grp_id,
                    name: db.0.get_pretty_name_defaulted(&go.grp_id.to_string()),
                });
                e_commands.insert(Owner(go.owner_seat_id));
                match go.is_tapped {
                    Some(true) => {
                        e_commands.insert(Tapped);
                    }
                    _ => {
                        e_commands.remove::<Tapped>();
                    }
                }
            }
            for zone in &gsm.zones {
                for instance_id in &zone.object_instance_ids {
                    let mut entity = match new_instances.get(instance_id) {
                        Some(entity) => *entity,
                        None => ie_query
                            .iter()
                            .find(|(_, instance)| instance.0 == *instance_id)
                            .map(|(entity, _)| entity)
                            .unwrap_or_else(|| {
                                commands.spawn((Instance(*instance_id),)).id()
                            })
                    };
                    commands.entity(entity).insert(Zone(zone.zone_id));
                }
                if let None = zone_info_query
                    .iter()
                    .find(|zone_info| zone_info.id == zone.zone_id)
                {
                    // TODO: From/Into?
                    commands.spawn((ZoneInfo {
                        id: zone.zone_id,
                        owner_id: zone.owner_seat_id,
                        default_visibility: zone.visibility,
                        type_field: zone.type_field,
                    },));
                }
            }
            info!(
                "Advanced Game State Message Index to {}, GameStateId: {}",
                mle.game_state_message_idx, game_state_id
            );
            break;
        }
    }
}

fn the_echo_system(
    input: Res<ButtonInput<KeyCode>>,
    mle: Res<MatchLogEvents>,
    db: Res<CardsDB>,
    query: Query<(&Instance, &Zone, Option<&GRP>)>,
    zones: Query<(&ZoneInfo)>,
) {
    if input.just_pressed(KeyCode::KeyF) {
        let mut zone_to_instances = BTreeMap::<i32, Vec<i32>>::new();
        let mut instance_to_grp = BTreeMap::<i32, String>::new();
        for (instance, zone, grp) in query.iter() {
            let zone_id = zone.0;
            zone_to_instances
                .entry(zone_id)
                .or_insert_with(Vec::new)
                .push(instance.0);
            if let Some(grp) = grp {
                instance_to_grp.insert(instance.0, grp.name.clone());
            }
        }

        for (zone_id, instance_ids) in zone_to_instances.iter() {
            if let Some(zone_info) = zones.iter().find(|zone_info| zone_info.id == *zone_id) {
                let zone_name = zone_info.type_field.to_string();
                let owner_id = zone_info
                    .owner_id
                    .and_then(|owner_id| {
                        mle.players
                            .get(&owner_id)
                            .map(|player| player.player_name.clone())
                    })
                    .unwrap_or("Public".to_string());

                info!("Zone {}: {} Owner: {}", zone_id, zone_name, owner_id);
                match zone_info.type_field {
                    ZoneType::Library | ZoneType::Limbo => {
                        let card_count = instance_ids.len();
                        info!("  {} cards", card_count);
                    }
                    _ => {
                        for instance_id in instance_ids {
                            let default_card_name = instance_id.to_string();
                            let card_name = instance_to_grp
                                .get(instance_id)
                                .unwrap_or(&default_card_name);
                            info!("  {} {}", card_name, instance_id);
                        }
                    }
                }
            }
        }
    }
}

fn setup(mle: Res<MatchLogEvents>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        TurnInfo,
        TextBundle::from_sections([
            TextSection::from_style(
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
            TextSection::new(
                " Turn ",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
            TextSection::new(
                " ",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
            TextSection::new(
                " ",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),
        ])
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(45.0),
            left: Val::Px(10.0),
            ..default()
        }),
    ));
    let mut texts = Vec::new();

    for (team_id, player) in mle.players.iter() {
        let default_life_total = 20;
        let style = if *team_id == mle.controller_seat_id {
            Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                align_self: AlignSelf::Center,
                ..default()
            }
        } else {
            Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                align_self: AlignSelf::Center,
                ..default()
            }
        };
        info!(
            "{} rolled a {}",
            player.player_name, player.die_roll_result
        );


        let text = commands.spawn((
            PlayerComp(player.player_name.clone()),
            LifeTotal(default_life_total),
            Team(*team_id),
            TextBundle::from_sections([
                TextSection::new(
                    format!("{}: ", player.player_name),
                    TextStyle {
                        font_size: 40.0,
                        ..default()
                    },
                ),
                TextSection::new(
                    default_life_total.to_string(),
                    TextStyle {
                        font_size: 60.0,
                        ..default()
                    },
                ),
            ])
            .with_text_justify(JustifyText::Center)
            .with_style(style),
        )).id();
        texts.push(text);
    }

    let mut parent = commands.spawn(NodeBundle{
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        ..default()
    });
    parent.push_children(&texts);
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
