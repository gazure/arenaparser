use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use anyhow::anyhow;
use serde::Serialize;
use crate::mtga_events::client::{ClientMessage, RequestTypeClientToMatchServiceMessage};
use crate::mtga_events::gre::RequestTypeGREToClientEvent;
use crate::mtga_events::mgrsc::{RequestTypeMGRSCEvent, StateType};


fn write_line<T>(writer: &mut BufWriter<File>, line: &T)  -> anyhow::Result<()> where T: Serialize {
    let line_str = serde_json::to_string(line)?;
    writer.write_all(line_str.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct MatchReplayBuilder {
    pub match_id: String,
    pub mgrsc_messages: Vec<RequestTypeMGRSCEvent>,
    pub gre_messages: Vec<RequestTypeGREToClientEvent>,
    pub client_messages: Vec<RequestTypeClientToMatchServiceMessage>,
}

impl MatchReplayBuilder {
    pub fn reset(&mut self) {
        self.mgrsc_messages.clear();
        self.gre_messages.clear();
        self.client_messages.clear();
        self.match_id = "".to_string();
    }

    fn write_match_replay(&self) -> anyhow::Result<()> {
        // write match replay to file
        if self.match_id == "" {
            return Err(anyhow!("Match ID is empty, not writing match replay"));
        }
        println!("Writing match replay to file");
        let path = PathBuf::from("match_replays").join(format!("{}.json", self.match_id));
        println!(
            "Match replay file created file: {}",
            &path.to_str().unwrap()
        );
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);

        self.mgrsc_messages.iter()
            .try_for_each(|mgrc_event| {
                write_line(&mut writer, mgrc_event)
            })?;

        self.gre_messages.iter().try_for_each(|gre_message| {
            write_line(&mut writer, gre_message)
        })?;

        self.client_messages.iter()
            .filter(|client_message| {
                match client_message.payload {
                    ClientMessage::UIMessage(_) | ClientMessage::SetSettingsReq(_) => false,
                    _ => true,
                }
            })
            .try_for_each(|client_message| {
                write_line(&mut writer, client_message)
            })?;
        Ok(())
    }

    pub fn ingest_mgrc_event(&mut self, mgrsc_event: RequestTypeMGRSCEvent) {
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
                match self.write_match_replay() {
                    Ok(_) => println!("Match replay written to file"),
                    Err(e) => eprintln!("Error writing match replay: {}", e),
                }
                self.reset()
            }
            StateType::Playing => {
                println!("found match: {}", match_id);
                self.match_id = match_id;
            }
        }
    }
}
