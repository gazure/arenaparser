use anyhow::Result;
use include_dir::{Dir, include_dir};
use lazy_static::lazy_static;
use rusqlite::{Connection, Params as RusqliteParams, Result as RusqliteResult};
use rusqlite_migration::Migrations;
use tracing::info;
use crate::cards::CardsDatabase;
use crate::models::deck::Deck;
use crate::models::match_result::{MatchResult, MatchResultBuilder};
use crate::models::mtga_match::{MTGAMatch, MTGAMatchBuilder};
use crate::models::mulligan::MulliganInfo;
use crate::replay::MatchReplay;
use crate::storage_backends::ArenaMatchStorageBackend;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> =
        Migrations::from_directory(&MIGRATIONS_DIR).unwrap_or(Migrations::new(Vec::new()));
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

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
    /// or if the migrations cannot be applied
    pub fn init(&mut self) -> Result<()> {
        MIGRATIONS.to_latest(&mut self.conn)?;
        Ok(())
    }

    /// # Errors
    ///
    /// passes along errors from Rusqlite
    pub fn execute(&mut self, query: &str, params: impl RusqliteParams) -> RusqliteResult<usize> {
        self.conn.execute(query, params)
    }

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
    pub fn insert_match(
        &mut self,
        mtga_match: &MTGAMatch,
    ) -> Result<()> {
        let params = (
            &mtga_match.id,
            &mtga_match.controller_seat_id,
            &mtga_match.controller_player_name,
            &mtga_match.opponent_player_name,
            &mtga_match.created_at,
        );

        self.conn.execute(
            "INSERT INTO matches \
            (id, controller_seat_id, controller_player_name, opponent_player_name, created_at)\
            VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(id) DO NOTHING",
            params
        )?;
        Ok(())
    }

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
    pub fn insert_deck(
        &mut self,
        match_id: &str,
        deck: &Deck,
    ) -> Result<()> {
        let deck_string = serde_json::to_string(&deck.mainboard)?;
        let sideboard_string = serde_json::to_string(&deck.sideboard)?;

        self.conn.execute(
            "INSERT INTO decks
                    (match_id, game_number, deck_cards, sideboard_cards)
                    VALUES (?1, ?2, ?3, ?4)
                    ON CONFLICT (match_id, game_number)
                    DO UPDATE SET deck_cards = excluded.deck_cards, sideboard_cards = excluded.sideboard_cards",
            (match_id, deck.game_number, deck_string, sideboard_string)
        )?;
        Ok(())
    }

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
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

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
    pub fn insert_match_result(
        &mut self,
        match_result: &MatchResult
    ) -> Result<()> {
        let params = (
            &match_result.match_id,
            &match_result.game_number,
            &match_result.winning_team_id,
            &match_result.result_scope,
        );

        self.conn.execute(
            "INSERT INTO match_results (match_id, game_number, winning_team_id, result_scope)\
             VALUES (?1, ?2, ?3, ?4)\
             ON CONFLICT (match_id, game_number)\
             DO UPDATE SET winning_team_id = excluded.winning_team_id, result_scope = excluded.result_scope",
            params
        )?;

        Ok(())
    }

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
    pub fn get_match_results(&mut self, match_id: &str) -> Result<Vec<MatchResult>> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_number, winning_team_id, result_scope FROM match_results WHERE match_id = ?1")?;
        let results = stmt.query_map(
            [match_id],
            |row| {
                let game_number: i32 = row.get(0)?;
                let winning_team_id: i32 = row.get(1)?;
                let result_scope: String = row.get(2)?;

                Ok(MatchResult {
                    match_id: match_id.to_string(),
                    game_number: Some(game_number),
                    winning_team_id,
                    result_scope,
                })
            },
        )?.collect::<rusqlite::Result<Vec<MatchResult>>>()?;
        Ok(results)
    }

    /// # Errors
    ///
    /// will return an error if mulligans cannot be found in `match_replay` or
    /// the database cannot be contacted for some reason
    fn persist_mulligans(&mut self, match_replay: &MatchReplay) -> Result<()> {
        let mulligan_infos = match_replay.get_mulligan_infos(&self.cards_database)?;
        mulligan_infos.iter().try_for_each(|mulligan_info| {
            self.insert_mulligan_info(mulligan_info.clone())
        })?;
        Ok(())
    }

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
    pub fn get_decklists(&mut self, match_id: &str) -> Result<Vec<Deck>> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_number, deck_cards, sideboard_cards FROM decks WHERE match_id = ?1")?;
        let deck = stmt
            .query_map([match_id], |row| {
                let game_number: i32 = row.get(0)?;
                let deck_cards: String = row.get(1)?;
                let sideboard_cards: String = row.get(2)?;

                Ok(Deck::from_raw_decklist(
                    "Found Deck".to_string(),
                    game_number,
                    &deck_cards,
                    &sideboard_cards,
                ))
            })?
            .collect::<rusqlite::Result<Vec<Deck>>>()?;
        Ok(deck)
    }

    /// # Errors
    ///
    /// will return an error if the database cannot be contacted for some reason
    pub fn get_mulligans(&mut self, match_id: &str) -> Result<Vec<MulliganInfo>> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_number, number_to_keep, hand, play_draw, opponent_identity, decision FROM mulligans WHERE match_id = ?1")?;
        let mulligans = stmt
            .query_map([match_id], |row| {
                let game_number: i32 = row.get(0)?;
                let number_to_keep: i32 = row.get(1)?;
                let hand: String = row.get(2)?;
                let play_draw: String = row.get(3)?;
                let opponent_identity: String = row.get(4)?;
                let decision: String = row.get(5)?;

                Ok(MulliganInfo {
                    match_id: match_id.to_string(),
                    game_number,
                    number_to_keep,
                    hand,
                    play_draw,
                    opponent_identity,
                    decision,
                })
            })?
            .collect::<rusqlite::Result<Vec<MulliganInfo>>>()?;
        Ok(mulligans)
    }
}


impl ArenaMatchStorageBackend for MatchInsightDB {

    // TODO: transactions?
    /// # Errors
    ///
    /// will return an error if a `controller_seat_id` cannot be found
    /// or if the match replay cannot be written to the database due to missing data
    /// or connection error
    fn write(&mut self, match_replay: &MatchReplay) -> Result<()> {
        info!("Writing match replay to database");
        let controller_seat_id = match_replay.get_controller_seat_id()?;
        let match_id = &match_replay.match_id;
        let (controller_name, opponent_name) = match_replay.get_player_names(controller_seat_id)?;

        let mtga_match = MTGAMatchBuilder::default()
            .id(match_id.to_string())
            .controller_seat_id(controller_seat_id)
            .controller_player_name(controller_name.clone())
            .opponent_player_name(opponent_name.clone())
            .build()?;


        self.insert_match(
            &mtga_match
        )?;

        match_replay.get_decklists()?.iter().try_for_each(|deck| {
            self.insert_deck(&match_replay.match_id, deck)
        })?;

        self.persist_mulligans(match_replay)?;

        // not too keen on this data model
        let match_results = match_replay.get_match_results()?;
        for (i, result) in match_results.result_list.iter().enumerate() {
            let game_number = if result.scope == "MatchScope_Game"{i32::try_from(i + 1).ok()} else {None};

            let match_result = MatchResultBuilder::default()
                .match_id(match_id.to_string())
                .game_number(game_number)
                .winning_team_id(result.winning_team_id)
                .result_scope(result.scope.clone())
                .build()?;

            self.insert_match_result(&match_result)?;
        }
        Ok(())
    }
}