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
