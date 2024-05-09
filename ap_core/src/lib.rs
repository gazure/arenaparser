pub mod arena_event_parser;
pub mod mtga_events;
pub mod replay;

use std::fmt::Display;
use crate::mtga_events::gre::GameObject;
use crate::mtga_events::mgrsc::MatchPlayer;
use anyhow::Result;
use lazy_static::lazy_static;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use mtga_events::primitives::{Annotation, MulliganType, ResultListEntry, TurnInfo, Zone};

#[derive(Debug, Clone, PartialEq)]
pub enum MatchEvent {
    MatchBegin {
        match_id: String,
        players: Vec<MatchPlayer>,
    },
    MatchComplete {
        match_id: String,
        result_list: Vec<ResultListEntry>,
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

        Ok(Self { db: cards_db })
    }


    pub fn get_pretty_name<T>(&self, grp_id: &T) -> Result<String> where T: Display + ?Sized {
        let grp_id = grp_id.to_string();
        self.db
            .get(&grp_id)
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

    pub fn get_pretty_name_defaulted(&self, grp_id: &str) -> String {
        self.get_pretty_name(grp_id)
            .unwrap_or_else(|_| grp_id.to_string())
    }
}
