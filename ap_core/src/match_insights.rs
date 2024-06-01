use crate::mtga_events::gre::DeckMessage;
use anyhow::Result;
use derive_builder::Builder;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use rusqlite::{Connection, Params as RusqliteParams, Result as RusqliteResult};
use rusqlite_migration::Migrations;
use tracing::info;
use crate::cards::CardsDatabase;
use crate::replay::MatchReplay;
use crate::storage_backends::ArenaMatchStorageBackend;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> =
        Migrations::from_directory(&MIGRATIONS_DIR).unwrap();
}

#[derive(Debug, Clone, Builder)]
pub struct MulliganInfo {
    pub match_id: String,
    pub game_number: i32,
    pub number_to_keep: i32,
    pub hand: String,
    pub play_draw: String,
    pub opponent_identity: String,
    pub decision: String,
}

#[derive(Debug)]
pub struct MatchInsightDB {
    pub conn: Connection,
    pub cards_database: CardsDatabase,
}

impl MatchInsightDB {
    pub fn new(conn: Connection, cards_database: CardsDatabase) -> Self {
        Self { conn, cards_database }
    }
    pub fn init(&mut self) -> Result<()> {
        MIGRATIONS.to_latest(&mut self.conn)?;
        Ok(())
    }

    pub fn execute(&mut self, query: &str, params: impl RusqliteParams) -> RusqliteResult<usize> {
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

    pub fn insert_mulligan_info(&mut self, mulligan_info: MulliganInfo) -> Result<()> {
        self.conn.execute(
            "INSERT INTO mulligans (match_id, game_number, number_to_keep, hand, play_draw, opponent_identity, decision)\
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)\
             ON CONFLICT (match_id, game_number, number_to_keep) \
             DO UPDATE SET hand = excluded.hand, play_draw = excluded.play_draw, opponent_identity = excluded.opponent_identity, decision = excluded.decision",
            (
                mulligan_info.match_id,
                mulligan_info.game_number,
                mulligan_info.number_to_keep,
                mulligan_info.hand,
                mulligan_info.play_draw,
                mulligan_info.opponent_identity,
                mulligan_info.decision,
            ),
        )?;
        Ok(())
    }

    pub fn insert_match_result(
        &mut self,
        match_id: &str,
        game_number: Option<i32>,
        winning_team_id: i32,
        result_scope: String,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO match_results (match_id, game_number, winning_team_id, result_scope)\
             VALUES (?1, ?2, ?3, ?4)\
             ON CONFLICT (match_id, game_number)\
             DO UPDATE SET winning_team_id = excluded.winning_team_id, result_scope = excluded.result_scope",
            (match_id, game_number, winning_team_id, result_scope)
        )?;

        Ok(())
    }

    pub fn get_match_results(&mut self, match_id: &str) -> Result<Vec<(i32, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_number, result_scope FROM match_results WHERE match_id = ?1")?;
        let results = stmt
            .query_map([match_id], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<Vec<(i32, String)>>>()?;
        Ok(results)
    }

    fn persist_mulligans(&mut self, match_replay: &MatchReplay) -> Result<()> {
        let mulligan_infos = match_replay.get_mulligan_infos(&self.cards_database)?;
        mulligan_infos.iter().try_for_each(|mulligan_info| {
            self.insert_mulligan_info(mulligan_info.clone())
        })?;
        Ok(())
    }
}


impl ArenaMatchStorageBackend for MatchInsightDB {
    fn write(&mut self, match_replay: &MatchReplay) -> Result<()> {
        // TODO: move write_to_db out of match_replay
        info!("Writing match replay to database");
        // write match replay to database
        let controller_seat_id = match_replay.get_controller_seat_id()?;
        let match_id = &match_replay.match_id;
        let (controller_name, opponent_name) = match_replay.get_player_names(controller_seat_id)?;

        self.insert_match(
            match_id,
            controller_seat_id,
            &controller_name,
            &opponent_name,
        )?;

        let decklists = match_replay.get_decklists()?;
        for (game_number, deck) in decklists.iter().enumerate() {
            self.insert_deck(&match_replay.match_id, (game_number + 1) as i32, deck)?;
        }

        self.persist_mulligans(match_replay)?;

        // not too keen on this data model
        let match_results = match_replay.get_match_results()?;
        for (i, result) in match_results.result_list.iter().enumerate() {
            if result.scope == "MatchScope_Game" {
                self.insert_match_result(
                    match_id,
                    Some((i + 1) as i32),
                    result.winning_team_id,
                    result.scope.clone(),
                )?;
            } else {
                self.insert_match_result(
                    match_id,
                    None,
                    result.winning_team_id,
                    result.scope.clone(),
                )?;
            }
        }
        Ok(())
    }
}