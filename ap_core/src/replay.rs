use crate::arena_event_parser::{ParseOutput};
use crate::mtga_events::client::{ClientMessage, RequestTypeClientToMatchServiceMessage};
use crate::mtga_events::gre::{DeckMessage, GREToClientMessage, RequestTypeGREToClientEvent};
use crate::mtga_events::mgrsc::{RequestTypeMGRSCEvent, StateType};
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::vec::IntoIter;
use lazy_static::lazy_static;
use crate::CardsDatabase;

// TODO: figure out better way of doing cards.db
lazy_static!(
    static ref CARDS_DB: CardsDatabase = CardsDatabase::new().unwrap();
);


fn write_line<T>(writer: &mut BufWriter<File>, line: &T) -> Result<()>
where
    T: Serialize,
{
    let line_str = serde_json::to_string(line)?;
    writer.write_all(line_str.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}

#[derive(Debug)]
pub enum MatchIteratorItem<'a> {
    GREMessage(&'a RequestTypeGREToClientEvent),
    ClientMessage(&'a RequestTypeClientToMatchServiceMessage),
    MGRSCMessage(&'a RequestTypeMGRSCEvent),
}

#[derive(Debug, Default)]
pub struct MatchReplay {
    pub match_id: String,
    pub mgrsc_messages: Vec<RequestTypeMGRSCEvent>,
    pub gre_messages: Vec<RequestTypeGREToClientEvent>,
    pub client_messages: Vec<RequestTypeClientToMatchServiceMessage>,
}

impl MatchReplay {
    pub fn write(&mut self, path: PathBuf) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for match_item in self.into_iter() {
           match match_item {
               MatchIteratorItem::MGRSCMessage(message) => {
                   write_line(&mut writer, message)?
               }
               MatchIteratorItem::GREMessage(message) => {
                   write_line(&mut writer, message)?
               }
               MatchIteratorItem::ClientMessage(message) => {
                   write_line(&mut writer, message)?
               }
           }
        }
        Ok(())
    }

    fn get_controller_seat_id(&self) -> Result<i32> {
        for gre_payload in &self.gre_messages {
            for gre_message in &gre_payload.gre_to_client_event.gre_to_client_messages {
                match gre_message {
                    GREToClientMessage::ConnectResp(wrapper) => {
                        return Ok(wrapper.meta.system_seat_ids[0]);
                    }
                    _ => {}
                }
            }
        }
        Err(anyhow!("Controller seat ID not found"))
    }

    fn get_player_names(&self, seat_id: i32) -> Result<(String, String)> {
        for mgrsc_message in &self.mgrsc_messages {
            if let Some(players) = &mgrsc_message.mgrsc_event.game_room_info.players {
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
        }
        Err(anyhow!("player names not found"))
    }

    fn get_initial_decklist(&self) -> Result<DeckMessage> {
        for gre_payload in &self.gre_messages {
            for gre_message in &gre_payload.gre_to_client_event.gre_to_client_messages {
                match gre_message {
                    GREToClientMessage::ConnectResp(wrapper) => {
                        return Ok(wrapper.connect_resp.deck_message.clone());
                    },
                    _ => {}
                }
            }
        }
        Err(anyhow!("Initial decklist not found"))
    }

    fn get_sideboarded_decklists(&self) -> Vec<DeckMessage> {
        let mut decklists = Vec::new();
        for message in &self.client_messages {
            match &message.payload {
                ClientMessage::SubmitDeckResp(submit_deck_resp) => {
                    decklists.push(submit_deck_resp.submit_deck_resp.deck.clone());
                }
                _ => {}
            }
        }
        decklists
    }

    fn get_decklists(&self) -> Result<Vec<DeckMessage>> {
        let mut decklists = vec![self.get_initial_decklist()?];
        decklists.append(&mut self.get_sideboarded_decklists());
        Ok(decklists)
    }

    pub fn write_to_db(&self, conn: &Connection) -> Result<()> {
        // write match replay to database
        let controller_seat_id = self.get_controller_seat_id()?;
        let (controller_name, opponent_name) = self.get_player_names(controller_seat_id)?;

        conn.execute(
            "INSERT INTO matches (id, controller_seat_id, controller_player_name, opponent_player_name) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO NOTHING",
            (&self.match_id, controller_seat_id, controller_name, opponent_name),
        )?;

        let decklists = self.get_decklists()?;
        for (game_number, deck) in decklists.iter().enumerate() {
            conn.execute(
                "INSERT INTO decks
                    (match_id, game_number, deck_cards, sideboard_cards)
                    VALUES (?1, ?2, ?3, ?4)
                    ON CONFLICT (match_id, game_number)
                    DO UPDATE SET deck_cards = excluded.deck_cards, sideboard_cards = excluded.sideboard_cards",
                (&self.match_id, (game_number + 1) as i32, serde_json::to_string(&deck.deck_cards)?, serde_json::to_string(&deck.sideboard_cards)?),
            )?;
        }

        Ok(())
    }
}

impl<'a> IntoIterator for &'a MatchReplay {
    type Item = MatchIteratorItem<'a>;
    type IntoIter = IntoIter<MatchIteratorItem<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut events = Vec::new();
        events.push(MatchIteratorItem::MGRSCMessage(&self.mgrsc_messages[0]));

        let mut gre_iter = self.gre_messages.iter().peekable();
        let mut client_iter = self.client_messages.iter().peekable();

        loop {
            match (gre_iter.peek(), client_iter.peek()) {
                (None, None) => {
                    break;
                }
                (Some(_), None) => {
                    events.push(MatchIteratorItem::GREMessage(gre_iter.next().unwrap()));
                }
                (None, Some(_)) => {
                    events.push(MatchIteratorItem::ClientMessage(client_iter.next().unwrap()));
                }
                (Some(gre), Some(client)) => {
                    if let Some(gre_request_id) = gre.request_id {
                        if gre_request_id <= client.request_id {
                            events.push(MatchIteratorItem::GREMessage(gre_iter.next().unwrap()));
                        } else {
                            events.push(MatchIteratorItem::ClientMessage(client_iter.next().unwrap()))
                        }
                    } else {
                        let _ = gre_iter.next().unwrap();
                    }
                }
            }
        }
        events.push(MatchIteratorItem::MGRSCMessage(&self.mgrsc_messages[1]));
        events.into_iter()
    }
}



#[derive(Debug, Default)]
pub struct MatchReplayBuilder {
    pub match_id: String,
    pub mgrsc_messages: Vec<RequestTypeMGRSCEvent>,
    pub gre_messages: Vec<RequestTypeGREToClientEvent>,
    pub client_messages: Vec<RequestTypeClientToMatchServiceMessage>,
}

impl MatchReplayBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn ingest_event(&mut self, event: ParseOutput) -> Option<MatchReplay> {
        match event {
            ParseOutput::GREMessage(gre_message) => self.ingest_gre_message(gre_message),
            ParseOutput::ClientMessage(client_message) => {
                self.ingest_client_message(client_message)
            }
            ParseOutput::MGRSCMessage(mgrsc_event) => {
                return self.ingest_mgrc_event(mgrsc_event);
            }
            ParseOutput::NoEvent => {}
        }
        None
    }

    pub fn ingest_gre_message(&mut self, gre_message: RequestTypeGREToClientEvent) {
        self.gre_messages.push(gre_message);
    }

    pub fn ingest_client_message(
        &mut self,
        client_message: RequestTypeClientToMatchServiceMessage,
    ) {
        self.client_messages.push(client_message);
    }

    pub fn ingest_mgrc_event(&mut self, mgrsc_event: RequestTypeMGRSCEvent) -> Option<MatchReplay> {
        let state_type = mgrsc_event.mgrsc_event.game_room_info.state_type.clone();
        let match_id = mgrsc_event
            .mgrsc_event
            .game_room_info
            .game_room_config
            .match_id
            .clone();
        self.mgrsc_messages.push(mgrsc_event);
        match state_type {
            StateType::MatchCompleted => {
                // match is over
                // write match replay to file
                // reset match replay builder
                self.build()
            }
            StateType::Playing => {
                println!("found match: {}", match_id);
                self.match_id = match_id;
                None
            }
        }
    }

    pub fn build(&mut self) -> Option<MatchReplay> {
        let match_replay = MatchReplay {
            match_id: self.match_id.clone(),
            mgrsc_messages: self.mgrsc_messages.clone(),
            gre_messages: self.gre_messages.clone(),
            client_messages: self.client_messages.clone(),
        };
        Some(match_replay)
    }
}
