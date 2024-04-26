// use std::collections::BTreeMap;

use bevy::prelude::*;
// use serde_json::Value;

use crate::arena_event_parser::MatchEvent;
// use crate::mtga_events::mgrc_event::Player;
//
// #[derive(Debug)]
// pub struct MatchResult {
//     scope: String,
//     winning_team_id: i32,
// }
//
// #[derive(Debug)]
// pub struct ArenaMatch {
//     match_id: String,
//     players: Vec<Player>,
//     zones: BTreeMap<i32, Value>,
//     game_objects: BTreeMap<i32, Value>,
//     results: Vec<MatchResult>,
// }

pub fn process_match_event(mut match_event_reader: EventReader<MatchEvent>) {
    for match_event in match_event_reader.read() {
        println!("Match Event: {:?}", match_event);
    }
}