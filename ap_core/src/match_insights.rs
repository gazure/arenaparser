use crate::mtga_events::gre::DeckMessage;
use anyhow::Result;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use rusqlite::{Connection, Params, Result as RusqliteResult};
use rusqlite_migration::Migrations;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> =
        Migrations::from_directory(&MIGRATIONS_DIR).unwrap();
}

#[derive(Debug)]
pub struct MatchInsightDB {
    conn: Connection,
}

impl MatchInsightDB {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    pub fn init(&mut self) -> Result<()> {
        MIGRATIONS.to_latest(&mut self.conn)?;
        Ok(())
    }

    pub fn execute(&mut self, query: &str, params: impl Params) -> RusqliteResult<usize> {
        self.conn.execute(query, params)
    }

    pub fn insert_match(
        &mut self,
        id: &str,
        seat_id: i32,
        name: &str,
        opp_name: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO matches \
            (id, controller_seat_id, controller_player_name, opponent_player_name)\
            VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO NOTHING",
            (id, seat_id, name, opp_name),
        )?;
        Ok(())
    }

    pub fn insert_deck(
        &mut self,
        match_id: &str,
        game_number: i32,
        deck_message: &DeckMessage,
    ) -> Result<()> {
        let deck_string = serde_json::to_string(&deck_message.deck_cards)?;
        let sideboard_string = serde_json::to_string(&deck_message.sideboard_cards)?;

        self.conn.execute(
            "INSERT INTO decks
                    (match_id, game_number, deck_cards, sideboard_cards)
                    VALUES (?1, ?2, ?3, ?4)
                    ON CONFLICT (match_id, game_number)
                    DO UPDATE SET deck_cards = excluded.deck_cards, sideboard_cards = excluded.sideboard_cards",
            (match_id, game_number, deck_string, sideboard_string)
        )?;
        Ok(())
    }

    pub fn insert_mulligan_info(
        &mut self,
        match_id: &str,
        game_number: i32,
        number_to_keep: i32,
        hand: &str,
        play_draw: &str,
        opp_identity: &str,
        decision: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO mulligans (match_id, game_number, number_to_keep, hand, play_draw, opponent_identity, decision)\
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)\
             ON CONFLICT (match_id, game_number, number_to_keep) \
             DO UPDATE SET hand = excluded.hand, play_draw = excluded.play_draw, opponent_identity = excluded.opponent_identity, decision = excluded.decision",
            (match_id, game_number, number_to_keep, hand, play_draw, opp_identity, decision),
        )?;
        Ok(())
    }

    pub fn insert_match_result(&mut self, match_id: &str, game_number: Option<i32>, winning_team_id: i32, result_scope: String) -> Result<()> {
        self.conn.execute(
            "INSERT INTO match_results (match_id, game_number, winning_team_id, result_scope) VALUES (?1, ?2, ?3, ?4)",
            (match_id, game_number, winning_team_id, result_scope)
        )?;

        Ok(())
    }
}
