use crate::arena_event_parser::ParseOutput;
use crate::mtga_events::client::{ClientMessage, RequestTypeClientToMatchServiceMessage};
use crate::mtga_events::gre::{GREToClientMessage, RequestTypeGREToClientEvent};
use crate::mtga_events::mgrsc::{RequestTypeMGRSCEvent, StateType};
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

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
    pub mgrsc_messages: Vec<RequestTypeMGRSCEvent>,
    pub gre_messages: Vec<RequestTypeGREToClientEvent>,
    pub client_messages: Vec<RequestTypeClientToMatchServiceMessage>,
}

impl MatchReplay {
    pub fn write(&mut self, path: PathBuf) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        self.mgrsc_messages
            .iter()
            .try_for_each(|mgrc_event| write_line(&mut writer, mgrc_event))?;

        self.gre_messages.iter_mut().try_for_each(|gre_message| {
            gre_message
                .gre_to_client_event
                .gre_to_client_messages
                .iter_mut()
                .for_each(|message| match message {
                    GREToClientMessage::GameStateMessage(gsm) => {
                        gsm.game_state_message.actions = Vec::new();
                    }
                    _ => {}
                });
            write_line(&mut writer, gre_message)
        })?;

        self.client_messages
            .iter()
            .filter(|client_message| match client_message.payload {
                ClientMessage::UIMessage(_) | ClientMessage::SetSettingsReq(_) => false,
                _ => true,
            })
            .try_for_each(|client_message| write_line(&mut writer, client_message))?;
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

    pub fn write_to_db(&self, conn: &Connection) -> Result<()> {
        // write match replay to database
        let controller_seat_id = self.get_controller_seat_id()?;
        let (controller_name, opponent_name) = self.get_player_names(controller_seat_id)?;
        conn.execute(
            "INSERT INTO matches (id, controller_seat_id, controller_player_name, opponent_player_name) VALUES (?1, ?2, ?3, ?4)",
            (&self.match_id, controller_seat_id, controller_name, opponent_name),
        )?;
        Ok(())
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

    // fn write_match_replay(&self) -> Result<()> {
    //     // write match replay to file
    //     if self.match_id == "" {
    //         return Err(anyhow!("Match ID is empty, not writing match replay"));
    //     }
    //     println!("Writing match replay to file");
    //     let path = PathBuf::from("match_replays").join(format!("{}.json", self.match_id));
    //     println!(
    //         "Match replay file created file: {}",
    //         &path.to_str().unwrap()
    //     );
    //     let file = File::create(path).unwrap();
    //     let mut writer = BufWriter::new(file);
    //
    //     self.mgrsc_messages.iter()
    //         .try_for_each(|mgrc_event| {
    //             write_line(&mut writer, mgrc_event)
    //         })?;
    //
    //     self.gre_messages.iter().try_for_each(|gre_message| {
    //         write_line(&mut writer, gre_message)
    //     })?;
    //
    //     self.client_messages.iter()
    //         .filter(|client_message| {
    //             match client_message.payload {
    //                 ClientMessage::UIMessage(_) | ClientMessage::SetSettingsReq(_) => false,
    //                 _ => true,
    //             }
    //         })
    //         .try_for_each(|client_message| {
    //             write_line(&mut writer, client_message)
    //         })?;
    //     Ok(())
    // }
    //
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
