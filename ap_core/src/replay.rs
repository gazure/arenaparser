use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::vec::IntoIter;

use anyhow::{anyhow, Result};
use serde::{Serialize, Serializer};
use tracing::debug;

use crate::arena_event_parser::ParseOutput;
use crate::cards::CardsDatabase;
use crate::match_insights::MatchInsightDB;
use crate::mtga_events::client::{
    ClientMessage, MulliganOption, MulliganRespWrapper, RequestTypeClientToMatchServiceMessage,
};
use crate::mtga_events::gre::{
    DeckMessage, GREToClientMessage, GameObjectType, GameStateMessage, MulliganReqWrapper,
    RequestTypeGREToClientEvent,
};
use crate::mtga_events::mgrsc::{FinalMatchResult, RequestTypeMGRSCEvent, StateType};
use crate::mtga_events::primitives::ZoneType;

fn write_line<T>(writer: &mut BufWriter<File>, line: &T) -> Result<()>
where
    T: Serialize,
{
    let line_str = serde_json::to_string(line)?;
    writer.write_all(line_str.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct MatchReplay {
    pub match_id: String,
    pub match_start_message: RequestTypeMGRSCEvent,
    pub match_end_message: RequestTypeMGRSCEvent,
    pub client_server_messages: Vec<MatchReplayEvent>,
}

#[derive(Debug, Clone)]
pub enum MatchReplayEvent {
    GRE(RequestTypeGREToClientEvent),
    Client(RequestTypeClientToMatchServiceMessage),
    MGRSC(RequestTypeMGRSCEvent),
}

pub enum MatchReplayEventRef<'a> {
    GRE(&'a RequestTypeGREToClientEvent),
    Client(&'a RequestTypeClientToMatchServiceMessage),
    MGRSC(&'a RequestTypeMGRSCEvent),
}

impl<'a> Serialize for MatchReplayEventRef<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::MGRSC(event) => event.serialize(serializer),
            Self::GRE(event) => event.serialize(serializer),
            Self::Client(event) => event.serialize(serializer),
        }
    }
}

impl MatchReplay {
    pub fn write(&self, path: PathBuf) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for match_item in self.into_iter() {
            write_line(&mut writer, &match_item)?;
        }
        Ok(())
    }
    fn gre_events_iter(&self) -> impl Iterator<Item = &RequestTypeGREToClientEvent> {
        self.client_server_messages
            .iter()
            .filter_map(|mre| match mre {
                MatchReplayEvent::GRE(message) => Some(message),
                _ => None,
            })
    }

    fn gre_messages_iter(&self) -> impl Iterator<Item = &GREToClientMessage> {
        self.gre_events_iter()
            .flat_map(|gre| &gre.gre_to_client_event.gre_to_client_messages)
    }

    fn game_state_messages_iter(&self) -> impl Iterator<Item = &GameStateMessage> {
        self.gre_messages_iter()
            .filter_map(|gre_message| match gre_message {
                GREToClientMessage::GameStateMessage(wrapper) => Some(&wrapper.game_state_message),
                _ => None,
            })
    }

    fn client_messages_iter(
        &self,
    ) -> impl Iterator<Item = &RequestTypeClientToMatchServiceMessage> {
        self.client_server_messages
            .iter()
            .filter_map(|mre| match mre {
                MatchReplayEvent::Client(message) => Some(message),
                _ => None,
            })
    }
    fn get_controller_seat_id(&self) -> Result<i32> {
        for gre_message in self.gre_messages_iter() {
            if let GREToClientMessage::ConnectResp(wrapper) = gre_message {
                return Ok(wrapper.meta.system_seat_ids[0]);
            }
        }
        Err(anyhow!("Controller seat ID not found"))
    }

    fn get_player_names(&self, seat_id: i32) -> Result<(String, String)> {
        if let Some(players) = &self.match_start_message.mgrsc_event.game_room_info.players {
            let controller = players
                .iter()
                .find(|player| player.system_seat_id == seat_id);
            let opponent = players
                .iter()
                .find(|player| player.system_seat_id != seat_id);
            if let Some(controller) = controller {
                if let Some(opponent) = opponent {
                    return Ok((controller.player_name.clone(), opponent.player_name.clone()));
                }
            }
        }
        Err(anyhow!("player names not found"))
    }

    fn get_opponent_cards(&self) -> Result<Vec<i32>> {
        let controller_id = self.get_controller_seat_id()?;
        let opponent_cards = self
            .game_state_messages_iter()
            .flat_map(|gsm| &gsm.game_objects)
            .filter(|game_object| game_object.owner_seat_id != controller_id)
            .filter(|game_object| {
                game_object.type_field == GameObjectType::Card
                    || game_object.type_field == GameObjectType::MDFCBack
            })
            .map(|game_object| game_object.grp_id)
            .collect();
        Ok(opponent_cards)
    }

    fn get_opponent_color_identity(&self, cards_db: &CardsDatabase) -> Result<String> {
        let opponent_cards = self.get_opponent_cards()?;
        let mut color_identity = BTreeSet::new();
        for card in opponent_cards {
            if let Some(card_db_entry) = cards_db.get(&card) {
                let colors = if card_db_entry.name == "jegantha_the_wellspring" {
                    vec!["R".to_string(), "G".to_string()]
                } else {
                    card_db_entry.color_identity.clone()
                };
                debug!("card: {}, colors: {:?}", card_db_entry.name, colors);
                color_identity.extend(colors);
            }
        }
        Ok(color_identity.into_iter().collect::<Vec<_>>().join(""))
    }

    fn get_match_results(&self) -> Result<FinalMatchResult> {
        self.match_end_message
            .mgrsc_event
            .game_room_info
            .final_match_result
            .clone()
            .ok_or(anyhow!("Match results not found"))
    }

    fn get_initial_decklist(&self) -> Result<DeckMessage> {
        for gre_message in self.gre_messages_iter() {
            if let GREToClientMessage::ConnectResp(wrapper) = gre_message {
                return Ok(wrapper.connect_resp.deck_message.clone());
            }
        }
        Err(anyhow!("Initial decklist not found"))
    }

    fn get_sideboarded_decklists(&self) -> Vec<DeckMessage> {
        let mut decklists = Vec::new();

        for message in self.client_messages_iter() {
            if let ClientMessage::SubmitDeckResp(submit_deck_resp) = &message.payload {
                decklists.push(submit_deck_resp.submit_deck_resp.deck.clone());
            }
        }
        decklists
    }

    fn get_decklists(&self) -> Result<Vec<DeckMessage>> {
        let mut decklists = vec![self.get_initial_decklist()?];
        decklists.append(&mut self.get_sideboarded_decklists());
        Ok(decklists)
    }

    fn persist_mulligans(&self, db: &mut MatchInsightDB, cards_db: &CardsDatabase) -> Result<()> {
        let controller_id = self.get_controller_seat_id()?;

        let mut game_number = 1;
        let mut opening_hands = BTreeMap::<i32, Vec<Vec<i32>>>::new();
        let mut mulligan_requests = BTreeMap::<i32, Vec<&MulliganReqWrapper>>::new();
        let mut play_or_draw: BTreeMap<i32, String> = BTreeMap::new();
        let opponent_color_identity = self.get_opponent_color_identity(cards_db)?;

        for gre in self.gre_messages_iter() {
            match gre {
                GREToClientMessage::GameStateMessage(wrapper) => {
                    let gsm = &wrapper.game_state_message;

                    if gsm.players.len() == 2
                        && gsm.players.iter().all(|player| {
                            player.pending_message_type
                                == Some("ClientMessageType_MulliganResp".to_string())
                        })
                    {
                        if let Some(turn_info) = &gsm.turn_info {
                            if let Some(decision_player) = turn_info.decision_player {
                                if decision_player == controller_id {
                                    println!("game_number: {}, play_or_draw: Play", game_number);
                                    play_or_draw.insert(game_number, "Play".to_string());
                                } else {
                                    println!("game_number: {}, play_or_draw: Draw", game_number);
                                    play_or_draw.insert(game_number, "Draw".to_string());
                                }
                            }
                        }
                    }

                    if gsm.players.iter().any(|player| {
                        player.controller_seat_id == controller_id
                            && player.pending_message_type
                                == Some("ClientMessageType_MulliganResp".to_string())
                    }) {
                        let controller_hand_zone_id = gsm
                            .zones
                            .iter()
                            .find(|zone| {
                                zone.type_field == ZoneType::Hand
                                    && zone.owner_seat_id == Some(controller_id)
                            })
                            .ok_or(anyhow!("Controller hand zone not found"))?
                            .zone_id;
                        let game_objects_in_hand: Vec<i32> = gsm
                            .game_objects
                            .iter()
                            .filter(|go| {
                                go.zone_id.is_some()
                                    && go.zone_id.unwrap() == controller_hand_zone_id
                                    && go.type_field == GameObjectType::Card
                            })
                            .map(|go| go.grp_id)
                            .collect();
                        opening_hands
                            .entry(game_number)
                            .or_default()
                            .push(game_objects_in_hand);
                    }
                }
                GREToClientMessage::MulliganReq(wrapper) => {
                    mulligan_requests
                        .entry(game_number)
                        .or_default()
                        .push(wrapper);
                }
                GREToClientMessage::IntermissionReq(_) => {
                    game_number += 1;
                }
                _ => {}
            }
        }

        let mulligan_responses: BTreeMap<i32, &MulliganRespWrapper> = self
            .client_messages_iter()
            .filter_map(|client_message| match &client_message.payload {
                ClientMessage::MulliganResp(wrapper) => {
                    Some((wrapper.meta.game_state_id?, wrapper))
                }
                _ => None,
            })
            .collect();

        for (game_number, hands) in opening_hands {
            let mut mulligan_requests_iter = mulligan_requests
                .get(&game_number)
                .ok_or(anyhow!(
                    "No mulligan requests found for game {}",
                    game_number
                ))?
                .iter();
            let play_draw = play_or_draw.get(&game_number).ok_or(anyhow!(
                "No play/draw decision found for game {}",
                game_number
            ))?;
            for hand in hands {
                let hand_string = hand
                    .iter()
                    .map(|grp_id| grp_id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                if let Some(mulligan_request) = mulligan_requests_iter.next() {
                    let game_state_id = mulligan_request.meta.game_state_id.unwrap();
                    let number_to_keep = 7 - mulligan_request.mulligan_req.mulligan_count;
                    let decision = match mulligan_responses.get(&game_state_id) {
                        Some(mulligan_response) => match mulligan_response.mulligan_resp.decision {
                            MulliganOption::AcceptHand => "Keep",
                            MulliganOption::Mulligan => "Mulligan",
                        },
                        None => "Match Ended",
                    };
                    let opp_identity = if game_number == 1 {
                        "Unknown"
                    } else {
                        &opponent_color_identity
                    };
                    db.insert_mulligan_info(
                        &self.match_id,
                        game_number,
                        number_to_keep,
                        &hand_string,
                        play_draw,
                        opp_identity,
                        decision,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn write_to_db(&self, conn: &mut MatchInsightDB, cards_db: &CardsDatabase) -> Result<()> {
        // write match replay to database
        let controller_seat_id = self.get_controller_seat_id()?;
        let (controller_name, opponent_name) = self.get_player_names(controller_seat_id)?;

        conn.insert_match(
            &self.match_id,
            controller_seat_id,
            &controller_name,
            &opponent_name,
        )?;

        let decklists = self.get_decklists()?;
        for (game_number, deck) in decklists.iter().enumerate() {
            conn.insert_deck(&self.match_id, (game_number + 1) as i32, deck)?;
        }

        self.persist_mulligans(conn, cards_db)?;

        // not too keen on this data model
        let match_results = self.get_match_results()?;
        for (i, result) in match_results.result_list.iter().enumerate() {
            if result.scope == "MatchScope_Game" {
                conn.insert_match_result(
                    &self.match_id,
                    Some((i + 1) as i32),
                    result.winning_team_id,
                    result.scope.clone(),
                )?;
            } else {
                conn.insert_match_result(
                    &self.match_id,
                    None,
                    result.winning_team_id,
                    result.scope.clone(),
                )?;
            }
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a MatchReplay {
    type Item = MatchReplayEventRef<'a>;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut events = Vec::new();
        events.push(MatchReplayEventRef::MGRSC(&self.match_start_message));
        self.client_server_messages.iter().for_each(|mre| {
            let mre_ref = match mre {
                MatchReplayEvent::GRE(event) => MatchReplayEventRef::GRE(event),
                MatchReplayEvent::Client(event) => MatchReplayEventRef::Client(event),
                MatchReplayEvent::MGRSC(event) => MatchReplayEventRef::MGRSC(event),
            };
            events.push(mre_ref);
        });
        events.push(MatchReplayEventRef::MGRSC(&self.match_end_message));
        events.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct MatchReplayBuilder {
    pub match_id: String,
    pub match_start_message: RequestTypeMGRSCEvent,
    pub match_end_message: RequestTypeMGRSCEvent,
    pub client_server_messages: Vec<MatchReplayEvent>,
}

impl MatchReplayBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn ingest_event(&mut self, event: ParseOutput) -> bool {
        match event {
            ParseOutput::GREMessage(gre_message) => self
                .client_server_messages
                .push(MatchReplayEvent::GRE(gre_message)),
            ParseOutput::ClientMessage(client_message) => self
                .client_server_messages
                .push(MatchReplayEvent::Client(client_message)),
            ParseOutput::MGRSCMessage(mgrsc_event) => {
                return self.ingest_mgrc_event(mgrsc_event);
            }
            ParseOutput::NoEvent => {}
        }
        false
    }

    pub fn ingest_mgrc_event(&mut self, mgrsc_event: RequestTypeMGRSCEvent) -> bool {
        let state_type = mgrsc_event.mgrsc_event.game_room_info.state_type.clone();
        let match_id = mgrsc_event
            .mgrsc_event
            .game_room_info
            .game_room_config
            .match_id
            .clone();
        match state_type {
            StateType::MatchCompleted => {
                // match is over
                self.match_end_message = mgrsc_event;
                return true;
            }
            StateType::Playing => {
                println!("found match: {}", match_id);
                self.match_id = match_id;
                self.match_start_message = mgrsc_event;
            }
        }
        false
    }

    pub fn build(self) -> Result<MatchReplay> {
        // TODO: add some Err states to this
        let match_replay = MatchReplay {
            match_id: self.match_id,
            match_start_message: self.match_start_message,
            match_end_message: self.match_end_message,
            client_server_messages: self.client_server_messages,
        };
        Ok(match_replay)
    }
}
