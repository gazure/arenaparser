pub mod mtga_events;
pub mod arena_event_parser;
pub mod replay;

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::BufReader;
use lazy_static::lazy_static;
use serde_json::Value;
use crate::mtga_events::gre::{Annotation, GameObject, MulliganType, TurnInfo, Zone};
use crate::mtga_events::mgrsc::{Player, ResultList};

#[derive(Debug, Clone, PartialEq)]
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

lazy_static! {
    pub(crate) static ref CARDS_DB: CardsDatabase = CardsDatabase::new().unwrap();
}

#[derive(Debug)]
pub struct CardsDatabase {
    pub db: Value,
}

impl CardsDatabase {
    pub fn new() -> Result<Self> {
        let cards_db_path = "data/cards.json";
        let cards_db_file = File::open(cards_db_path)?;
        let cards_db_reader = BufReader::new(cards_db_file);
        let cards_db: Value = serde_json::from_reader(cards_db_reader)?;
        let cards_db = cards_db.get("cards").ok_or(anyhow!("Cards not found"))?.clone();

        Ok(Self { db: cards_db })
    }

    pub fn get_pretty_name(&self, grp_id: &str) -> Result<String> {
        self.db
            .get(grp_id)
            .ok_or_else(|| anyhow::anyhow!("Card not found in database"))
            .and_then(|card| {
                card.get("pretty_name")
                    .ok_or_else(|| anyhow::anyhow!("Card does not have a pretty name"))
                    .and_then(|pretty_name| {
                        pretty_name
                            .as_str()
                            .ok_or_else(|| anyhow::anyhow!("Pretty name is not a string"))
                            .map(|pretty_name| pretty_name.to_string())
                    })
            })
    }
}
