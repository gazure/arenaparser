use crate::arena_event_parser::{ParseOutput};
use crate::mtga_events::client::{ClientMessage, RequestTypeClientToMatchServiceMessage};
use crate::mtga_events::gre::{DeckMessage, GREToClientMessage, RequestTypeGREToClientEvent};
use crate::mtga_events::mgrsc::{RequestTypeMGRSCEvent, StateType};
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use serde::{Serialize, Serializer};
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
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        match self {
            Self::MGRSC(event) => {
                event.serialize(serializer)
            }
            Self::GRE(event) => {
                event.serialize(serializer)
            }
            Self::Client(event) => {
                event.serialize(serializer)
            }
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
    fn get_controller_seat_id(&self) -> Result<i32> {
        let gre_messages = self.client_server_messages.iter().filter_map(|mre| {
            match mre {
                MatchReplayEvent::GRE(message) => {
                    Some(message)
                },
                _ => None
            }
        });


        for gre_payload in gre_messages {
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

    fn get_initial_decklist(&self) -> Result<DeckMessage> {
        let gre_messages = self.client_server_messages.iter().filter_map(|mre| {
            match mre {
                MatchReplayEvent::GRE(message) => {
                    Some(message)
                },
                _ => None
            }
        });
        for gre_payload in gre_messages {
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
        let client_messages = self.client_server_messages.iter().filter_map(|mre| {
            match mre {
                MatchReplayEvent::Client(message) => {
                    Some(message)
                }
                _ => None
            }
        });
        let mut decklists = Vec::new();
        for message in client_messages {
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
    type Item = MatchReplayEventRef<'a>;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut events = Vec::new();
        events.push(MatchReplayEventRef::MGRSC(&self.match_start_message));
        self.client_server_messages.iter().for_each(|mre| {
            let mre_ref  = match mre {
                MatchReplayEvent::GRE(event) => {
                    MatchReplayEventRef::GRE(event)
                }
                MatchReplayEvent::Client(event) => {
                    MatchReplayEventRef::Client(event)
                }
                MatchReplayEvent::MGRSC(event) => {
                    MatchReplayEventRef::MGRSC(event)
                }
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
    pub client_server_messages: Vec<MatchReplayEvent>
}

impl MatchReplayBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn ingest_event(&mut self, event: ParseOutput) -> bool {
        match event {
            ParseOutput::GREMessage(gre_message) => {
                self.client_server_messages.push(MatchReplayEvent::GRE(gre_message))
            },
            ParseOutput::ClientMessage(client_message) => {
                self.client_server_messages.push(MatchReplayEvent::Client(client_message))
            }
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
