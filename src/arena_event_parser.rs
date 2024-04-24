use crate::client_event::{ClientMessage, RequestTypeClientToMatchServiceMessage};
use crate::gre::RequestTypeGREToClientEvent;
use crate::mgrc_event::{RequestTypeMGRSCEvent, StateType};
use anyhow::Result;
use crossbeam::channel::Receiver;
use serde_json::Value;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MatchPlayer {
    pub player_name: String,
    pub team_id: i32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FinalMatchResult {
    pub match_winner: i32,
    pub game_winners: Vec<i32>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ArenaMatch {
    pub match_id: String,
    pub players: Vec<MatchPlayer>,
    pub final_match_result: Option<FinalMatchResult>,
}

impl Display for ArenaMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Match ID: {}\nPlayers: {} vs. {}\n",
            self.match_id, self.players[0].player_name, self.players[1].player_name
        )
        .unwrap();
        if let Some(final_match_result) = &self.final_match_result {
            let game_winner_names: Vec<String> = final_match_result.game_winners.iter().map(|team_id| {
                self.players
                    .iter()
                    .find(|player| player.team_id == *team_id)
                    .unwrap()
                    .player_name
                    .clone()
            }).collect();
            let match_winner_name = self
                .players
                .iter()
                .find(|player| player.team_id == final_match_result.match_winner)
                .unwrap()
                .player_name
                .clone();
            write!(
                f,
                "Match Winner: {}\nGame Winners: {:?}",
                match_winner_name, game_winner_names
            )
            .unwrap();
        }
        write!(f, "")
    }
}

#[derive(Debug)]
pub struct ArenaEventParser {
    arena_event_rx: Receiver<String>,
    cards_db: Value,
    current_match: Option<ArenaMatch>,
}

impl ArenaEventParser {
    pub fn new(arena_event_rx: Receiver<String>) -> Self {
        let cards_db_path = "data/cards.json";
        let cards_db_file = std::fs::File::open(cards_db_path).unwrap();
        let cards_db_reader = std::io::BufReader::new(cards_db_file);
        let cards_db = serde_json::from_reader(cards_db_reader).unwrap();

        Self {
            arena_event_rx,
            cards_db,
            current_match: None,
        }
    }

    pub fn process(&mut self) {
        while let Ok(event) = self.arena_event_rx.recv() {
            let result = self.process_event(&event);
            if let Err(e) = result {
                eprintln!("Error processing event: {}\n{}", e, event);
            }
        }
    }

    fn process_event(&mut self, event: &String) -> Result<()> {
        if event.contains("clientToMatchServiceMessage") {
            let client_to_match_service_message: RequestTypeClientToMatchServiceMessage =
                serde_json::from_str(event)?;

            match client_to_match_service_message.payload {
                ClientMessage::PerformActionResp(payload) => {
                    let action_resp_payload = payload.perform_action_response;
                    for action in action_resp_payload.actions {
                        if let Some(grp_id) = action.grp_id {
                            let grp_id = grp_id.to_string();
                            let card = self.cards_db.get("cards").unwrap().get(&grp_id).unwrap();
                            let pretty_name = card.get("pretty_name").unwrap();
                            println!(
                                "Action Type: {}, card_name: {}",
                                action.action_type, pretty_name
                            );
                        }
                    }
                }
                ClientMessage::MulliganResp(payload) => {
                    let mulligan_resp_payload = payload.mulligan_response;
                    println!("Mulligan Decision: {}", mulligan_resp_payload.decision);
                }
                ClientMessage::DeclareAttackersReq(payload) => {
                    println!("Declare Attackers Request");
                    payload.extra.iter().for_each(|(key, value)| {
                        println!("{}: {}", key, *value);
                    });
                }
                _ => {}
            }
        } else if event.contains("matchGameRoomStateChangedEvent") {
            let mgrsc_event: RequestTypeMGRSCEvent = serde_json::from_str(event)?;

            let game_room_info = mgrsc_event
                .match_game_room_state_changed_event
                .game_room_info;
            let game_room_config = game_room_info.game_room_config;
            let players = game_room_info.players;
            let match_id = game_room_config.match_id;
            println!("Match ID: {}", match_id);
            if game_room_info.state_type == StateType::Playing {
                if let Some(players) = players {
                    self.current_match = Some(ArenaMatch {
                        match_id,
                        players: players
                            .iter()
                            .map(|player| MatchPlayer {
                                player_name: player.player_name.clone(),
                                team_id: player.team_id,
                            })
                            .collect(),
                        final_match_result: None,
                    });
                }
            } else if game_room_info.state_type == StateType::MatchCompleted {
                let final_match_result = game_room_info.final_match_result.unwrap();
                if let Some(current_match) = &mut self.current_match {
                    let mut match_result = FinalMatchResult {
                        match_winner: 0,
                        game_winners: Vec::new(),
                    };
                    for result in final_match_result.result_list {
                        if result.scope == "MatchScope_Match" {
                            match_result.match_winner = result.winning_team_id;
                        } else {
                            match_result.game_winners.push(result.winning_team_id);
                        }
                    }
                    current_match.final_match_result = Some(match_result);
                }
            }
            if let Some(current_match) = &self.current_match {
                println!("Current Match: {}", current_match);
            }
        } else if event.contains("greToClientEvent") {
            let request_gre_to_client_event: RequestTypeGREToClientEvent =
                serde_json::from_str(event)?;
            let gre_to_client_event = request_gre_to_client_event.gre_to_client_event;
            for gre_to_client_message in gre_to_client_event.gre_to_client_messages {
                match gre_to_client_message {
                    _ => {
                        //TODO: figure out how to handle all the GRE to client events
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_event() -> Result<()> {
        let event = json!({
            "clientToMatchServiceMessageType": "ClientToMatchServiceMessage",
            "requestId": 6,
            "payload": {
                "gameStateId": 23,
                "performActionResp": {
                    "actions": [
                        {
                            "actionType": "ActionType_Play",
                            "facetId": 163,
                            "grpId": 58445,
                            "instanceId": 163,
                            "shouldStop": true
                        }
                    ],
                    "autoPassPriority": "AutoPassPriority_Yes"
                },
                "respId": 37,
                "type": "ClientMessageType_PerformActionResp"
            },
            "timestamp": "638494452827308839",
            "transactionId": "7bf08a41-032c-4dc8-a842-762a0b71c04b"
        });
        let client_to_match_service_message: RequestTypeClientToMatchServiceMessage =
            serde_json::from_value(event)?;
        assert_eq!(client_to_match_service_message.request_id, 6);
        match client_to_match_service_message.payload {
            ClientMessage::PerformActionResp(payload) => {
                let action_resp_payload = payload.perform_action_response;
                for action in action_resp_payload.actions {
                    assert_eq!(action.action_type, "ActionType_Play");
                    assert_eq!(action.facet_id, 163);
                    assert_eq!(action.grp_id, 58445);
                    assert_eq!(action.instance_id, 163);
                }
            }
            _ => {
                assert!(false, "Expected PerformActionResp")
            }
        }

        Ok(())
    }

    #[test]
    fn test_parse_mgrsc_event() {
        let event = json!({
          "matchGameRoomStateChangedEvent": {
            "gameRoomInfo": {
              "gameRoomConfig": {
                "matchId": "a75fba61-77a2-4cee-bfbd-039cd95ba1d7",
                "reservedPlayers": [
                  {
                    "connectionInfo": {
                      "connectionState": "ConnectionState_Open"
                    },
                    "courseId": "Avatar_Basic_Aragorn_LTR",
                    "eventId": "AIBotMatch",
                    "platformId": "Windows",
                    "playerName": "tehsbe",
                    "sessionId": "be39d6c8-e288-47ff-a33c-fa982e8bc1d8",
                    "systemSeatId": 1,
                    "teamId": 1,
                    "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU"
                  },
                  {
                    "connectionInfo": {
                      "connectionState": "ConnectionState_Open"
                    },
                    "courseId": "Avatar_Basic_Sparky",
                    "eventId": "AIBotMatch",
                    "isBotPlayer": true,
                    "playerName": "Sparky",
                    "systemSeatId": 2,
                    "teamId": 2,
                    "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU_Familiar"
                  }
                ]
              },
              "players": [
                {
                  "playerName": "tehsbe",
                  "systemSeatId": 1,
                  "teamId": 1,
                  "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU"
                },
                {
                  "playerName": "Sparky",
                  "systemSeatId": 2,
                  "teamId": 2,
                  "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU_Familiar"
                }
              ],
              "stateType": "MatchGameRoomStateType_Playing"
            }
          },
          "requestId": 2,
          "timestamp": "1713848448700",
          "transactionId": "ab322eb2-a095-4ad4-91a0-d6bf08bf9482"
        });
        let mgrsc_event: RequestTypeMGRSCEvent = serde_json::from_value(event).unwrap();
        assert_eq!(mgrsc_event.request_id, 2);
    }
}
