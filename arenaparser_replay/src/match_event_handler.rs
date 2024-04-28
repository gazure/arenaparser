#![allow(unused)]

use bevy::prelude::*;
use std::collections::BTreeMap;
use ap_core::mtga_events::gre::{GameObjectType, ZoneType};
// use serde_json::Value;

use crate::{CardsDB, GameStateUpdated, MatchEvent};

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

#[derive(Debug, Component)]
pub struct MTGAMatch {
    match_id: String,
    players: BTreeMap<i32, String>,
    event_feed: Vec<MatchEvent>,
}

#[derive(Debug, Default)]
pub struct InstanceEntityMapping {
    pub instance_to_entity: BTreeMap<i32, Entity>,
}

pub fn process_match_event(
    cards: Res<CardsDB>,
    mut query: Query<(Entity, &mut MTGAMatch)>,
    mut instances_query: Query<&mut Instance>,
    mut match_event_reader: EventReader<MatchEvent>,
    mut instance_mapping: Local<InstanceEntityMapping>,
    mut game_state_update_writer: EventWriter<GameStateUpdated>,
    mut commands: Commands,
) {
    for match_event in match_event_reader.read() {
        for (_, mut mtga_match) in query.iter_mut() {
            mtga_match.event_feed.push(match_event.clone());
        }
        match match_event {
            MatchEvent::MatchBegin { match_id, players } => {
                let mut players_map = BTreeMap::new();
                for player in players {
                    players_map.insert(player.team_id, player.player_name.clone());
                }
                commands.spawn(MTGAMatch {
                    match_id: match_id.clone(),
                    players: players_map,
                    event_feed: Vec::new(),
                });
            },
            MatchEvent::MatchComplete { match_id, result_list } => {
                for (match_entity, mtga_match) in query.iter_mut() {
                    if mtga_match.match_id.contains(match_id) {
                        for res in result_list {
                            if let Some(player_name)  = mtga_match.players.get(&res.winning_team_id) {
                                println!("Winner: {}", player_name);
                            }
                        }
                    }
                    for entity in instance_mapping.instance_to_entity.values() {
                        commands.entity(*entity).despawn();
                    }
                    commands.entity(match_entity).despawn();
                }
            }
            MatchEvent::DeckMessage(maindeck, sideboard) => {
                let maindeck_card_names = maindeck
                    .iter()
                    .map(|card_id| card_id.to_string())
                    .map(|card_id| cards.0.get_pretty_name(&card_id).unwrap_or(card_id))
                    .collect::<Vec<String>>();
                let sideboard_card_names = sideboard
                    .iter()
                    .map(|card_id| card_id.to_string())
                    .map(|card_id| cards.0.get_pretty_name(&card_id).unwrap_or(card_id))
                    .collect::<Vec<String>>();
                println!("Maindeck: {:?}", maindeck_card_names);
                println!("Sideboard: {:?}", sideboard_card_names);
            }
            MatchEvent::ClientAction {
                action_type,
                card_name,
            } => {
                println!("Client action: {} {}", action_type, card_name);
            }
            MatchEvent::GameStateMessage {
                game_state_id,
                annotations,
                game_objects,
                zones,
                turn_info,
            } => {
                for annotation in annotations {}
                for game_object in game_objects {
                    let instance_id = game_object.instance_id;
                    let go_type = game_object.type_field.clone();
                    let grp_id = game_object.grp_id.to_string();
                    let pretty_name = cards.0.get_pretty_name(&grp_id).unwrap_or(grp_id);
                    let zone_id = game_object.zone_id;
                    if let Some(entity) = instance_mapping.instance_to_entity.get(&instance_id) {
                        if let Some(mut entity_commands) = commands.get_entity(*entity) {
                            entity_commands.try_insert(GOType(go_type));
                            entity_commands.try_insert(GRP {
                                id: game_object.grp_id,
                                name: pretty_name,
                            });
                            if let Some(zone_id) = zone_id {
                                entity_commands.try_insert(Zone(zone_id));
                            } else {
                                entity_commands.remove::<Zone>();
                            }
                        }
                    } else {
                        println!("New Game Object Type: {:?}", game_object.type_field);
                        let entity_id = if let Some(zone_id) = zone_id {
                            commands.spawn((
                                Instance(instance_id),
                                GRP {
                                    id: game_object.grp_id,
                                    name: pretty_name,
                                },
                                GOType(game_object.type_field.clone()),
                                Zone(zone_id),
                            )).id()
                        } else {
                            commands.spawn((
                                Instance(instance_id),
                                GRP {
                                    id: game_object.grp_id,
                                    name: pretty_name,
                                },
                                GOType(game_object.type_field.clone()),
                            )).id()
                        };
                        instance_mapping.instance_to_entity.insert(instance_id, entity_id);
                    }
                }
                game_state_update_writer.send(GameStateUpdated);
            }
            _ => {}
        }
    }
}

pub fn echo_game_state(
    mut match_query: Query<&MTGAMatch>,
    grps_in_zones: Query<(&GRP, &Zone)>,
    mut event_reader: EventReader<GameStateUpdated>,
) {
    let mut events_iter = event_reader.read();
    if events_iter.next().is_some() {
        for mtga_match in match_query.iter_mut() {
            println!("Match: {}", mtga_match.match_id);
            for (team_id, player_name) in mtga_match.players.iter() {
                println!("Player {}: {}", team_id, player_name);
            }
        }
        for (grp, zone) in grps_in_zones.iter() {
            println!("GRP: {} in Zone: {}", grp.name, zone.0);
        }
    }
    for _ in events_iter {}
}
