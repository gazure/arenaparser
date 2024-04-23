use anyhow::Result;
use crossbeam::channel::Receiver;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
struct RequestTypeMGRSCEvent {
    #[serde(rename = "matchGameRoomStateChangedEvent")]
    match_game_room_state_changed_event: MatchGameRoomStateChangedEvent,
    #[serde(rename = "requestId")]
    request_id: i32,
    timestamp: String,
    #[serde(rename = "transactionId")]
    transaction_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct MatchGameRoomStateChangedEvent {
    #[serde(rename = "gameRoomInfo")]
    game_room_info: GameRoomInfo,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameRoomInfo {
    #[serde(rename = "gameRoomConfig")]
    game_room_config: GameRoomConfig,
    players: Option<Vec<Player>>,
    #[serde(rename = "finalMatchResult")]
    final_match_result: Option<FinalMatchResult>,
    #[serde(rename = "stateType")]
    state_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FinalMatchResult {
    #[serde(rename = "matchId")]
    match_id: String,
    #[serde(rename = "resultList")]
    result_list: Vec<ResultList>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResultList {
    scope: String,
    #[serde(rename = "winningTeamId")]
    winning_team_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameRoomConfig {
    #[serde(rename = "matchId")]
    match_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Player {
    #[serde(rename = "playerName")]
    player_name: String,
    #[serde(rename = "systemSeatId")]
    system_seat_id: i32,
    #[serde(rename = "teamId")]
    team_id: i32,
    #[serde(rename = "userId")]
    user_id: String,
}


#[derive(Debug, Deserialize, Serialize)]
struct RequestTypeClientToMatchServiceMessage {
    #[serde(rename = "clientToMatchServiceMessageType")]
    client_to_match_service_message_type: String,
    #[serde(rename = "requestId")]
    request_id: i32,
    #[serde(rename = "payload")]
    payload: ClientMessage,
    timestamp: String,
    #[serde(rename = "transactionId")]
    transaction_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "ClientMessageType_PerformActionResp")]
    PerformActionResp(PerformActionResp),
    #[serde(rename = "ClientMessageType_MulliganResp")]
    MulliganResp(MulliganResp),
    #[serde(rename = "ClientMessageType_UIMessage")]
    UIMessage(UIMessage),
    #[serde(rename = "ClientMessageType_SelectNResp")]
    SelectNResp(SelectNResp),
}

#[derive(Debug, Deserialize, Serialize)]
struct SelectNResp {}

#[derive(Debug, Deserialize, Serialize)]
struct UIMessage {
    #[serde(rename = "systemSeatId")]
    system_seat_id: i32,
    #[serde(rename = "uiMessage")]
    ui_message: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct MulliganResp {
    #[serde(rename = "gameStateId")]
    game_state_id: i32,
    #[serde(rename = "mulliganResp")]
    mulligan_response: MulliganDecision,
    #[serde(rename = "respId")]
    response_id: i32,
}
#[derive(Debug, Deserialize, Serialize)]
struct MulliganDecision {
    decision: String,
}


#[derive(Debug, Deserialize, Serialize)]
struct PerformActionResp {
    #[serde(rename = "gameStateId")]
    game_state_id: i32,
    #[serde(rename = "performActionResp")]
    perform_action_response: PerformActionResponse,
    #[serde(rename = "respId")]
    response_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct PerformActionResponse {
    #[serde(rename = "actions")]
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Action {
    #[serde(rename = "actionType")]
    action_type: String,
    #[serde(rename = "facetId")]
    facet_id: i32,
    #[serde(rename = "grpId")]
    grp_id: i32,
    #[serde(rename = "instanceId")]
    instance_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RequestTypeGREToClientEvent {
    #[serde(rename = "greToClientEvent")]
    gre_to_client_event: GREToClientEvent,
    #[serde(rename = "requestId")]
    request_id: i32,
    timestamp: String,
    #[serde(rename = "transactionId")]
    transaction_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GREToClientEvent {
    #[serde(rename = "greToClientMessages")]
    gre_to_client_messages: Vec<GREToClientMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum GREToClientMessage {
    #[serde(rename = "GREMessageType_ConnectResp")]
    ConnectResp(ConnectResp),
    #[serde(rename = "GREMessageType_DieRollResultsResp")]
    DieRollResults(DieRollResultsResp),
    #[serde(rename = "GREMessageType_GameStateMessage")]
    GameStateMessage(GameStateMessage),
    #[serde(rename = "GREMessageType_ChooseStartingPlayerReq")]
    ChooseStartingPlayerReq(ChooseStartingPlayerReq),
    #[serde(rename = "GREMessageType_MulliganReq")]
    MulliganReq(MulliganReq),
    #[serde(rename = "GREMessageType_SelectNReq")]
    SelectNReq(SelectNReq),
    #[serde(rename = "GREMessageType_ActionsAvailableReq")]
    ActionsAvailableReq(ActionsAvailableReq),
    #[serde(rename = "GREMessageType_SetSettingsResp")]
    SetSettingsResp(SetSettingsResp),
    #[serde(rename = "GREMessageType_SelectTargetsReq")]
    SelectTargetsReq(SelectTargetsReq),
    #[serde(rename = "GREMessageType_SubmitTargetsResp")]
    SubmitTargetsResp(SubmitTargetsResp),
    #[serde(rename = "GREMessageType_CastingTimeOptionsReq")]
    CastingTimeOptionsReq(CastingTimeOptionsReq),
    #[serde(rename = "GREMessageType_PayCostsReq")]
    PayCostsReq(PayCostsReq),
    #[serde(rename = "GREMessageType_SelectNResp")]
    SelectNResp(SelectNResp),
    #[serde(rename = "GREMessageType_DeclareAttackersReq")]
    DeclareAttackersReq(DeclareAttackersReq),
    #[serde(rename = "GREMessageType_SubmitAttackersResp")]
    SubmitAttackersResp(SubmitAttackersResp),
    #[serde(rename = "GREMessageType_IntermissionReq")]
    IntermissionReq(IntermissionReq),
    #[serde(rename = "GREMessageType_PromptReq")]
    PromptReq(PromptReq),
    #[serde(rename = "GREMessageType_QueuedGameStateMessage")]
    QueuedGameStateMessage(QueuedGameStateMessage),
}

#[derive(Debug, Deserialize, Serialize)]
struct QueuedGameStateMessage {}

#[derive(Debug, Deserialize, Serialize)]
struct PromptReq {}

#[derive(Debug, Deserialize, Serialize)]
struct IntermissionReq {}

#[derive(Debug, Deserialize, Serialize)]
struct SubmitAttackersResp {}

#[derive(Debug, Deserialize, Serialize)]
struct DeclareAttackersReq {}

#[derive(Debug, Deserialize, Serialize)]
struct PayCostsReq {}

#[derive(Debug, Deserialize, Serialize)]
struct CastingTimeOptionsReq {}

#[derive(Debug, Deserialize, Serialize)]
struct SubmitTargetsResp {}

#[derive(Debug, Deserialize, Serialize)]
struct SelectTargetsReq {}

#[derive(Debug, Deserialize, Serialize)]
struct SetSettingsResp {}

#[derive(Debug, Deserialize, Serialize)]
struct DieRollResultsResp {}

#[derive(Debug, Deserialize, Serialize)]
struct ActionsAvailableReq {}

#[derive(Debug, Deserialize, Serialize)]
struct SelectNReq {}

#[derive(Debug, Deserialize, Serialize)]
struct MulliganReq {}

#[derive(Debug, Deserialize, Serialize)]
struct ChooseStartingPlayerReq {}

#[derive(Debug, Deserialize, Serialize)]
struct GameStateMessage {}

#[derive(Debug, Deserialize, Serialize)]
struct ConnectResp {
    #[serde(rename = "connectResp")]
    connect_response: ConnectResponse,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConnectResponse {
    #[serde(rename = "deckMessage")]
    deck_message: DeckMessage,
    settings: Option<serde_json::Value>,
    skins: Option<serde_json::Value>,
    status: String
}

#[derive(Debug, Deserialize, Serialize)]
struct DeckMessage {
    #[serde(rename = "deckCards")]
    deck_cards: Vec<i32>,
    #[serde(rename = "sideboardCards")]
    sideboard_cards: Vec<i32>,
}

#[derive(Debug)]
pub struct ArenaEventParser {
    arena_event_rx: Receiver<serde_json::Value>,
    cards_db: serde_json::Value,
}

impl ArenaEventParser {
    pub fn new(arena_event_rx: Receiver<serde_json::Value>) -> Self {
        let cards_db_path = "data/cards.json";
        let cards_db_file = std::fs::File::open(cards_db_path).unwrap();
        let cards_db_reader = std::io::BufReader::new(cards_db_file);
        let cards_db = serde_json::from_reader(cards_db_reader).unwrap();

        Self {
            arena_event_rx,
            cards_db,
        }
    }

    pub fn process(&mut self) {
        while let Ok(event) = self.arena_event_rx.recv() {
            self.process_event(event).unwrap_or(())
        }
    }

    fn process_event(&self, event: serde_json::Value) -> Result<()> {
        if let Ok(client_to_match_service_message) =
            serde_json::from_value::<RequestTypeClientToMatchServiceMessage>(event.clone())
        {
            match client_to_match_service_message.payload {
                ClientMessage::PerformActionResp(payload) => {
                    let action_resp_payload = payload.perform_action_response;
                    for action in action_resp_payload.actions {
                        let grp_id = action.grp_id.to_string();
                        let card = self.cards_db.get("cards").unwrap().get(&grp_id).unwrap();
                        let pretty_name = card.get("pretty_name").unwrap();
                        println!(
                            "Action Type: {}, card_name: {}",
                            action.action_type, pretty_name
                        );
                    }
                }
                ClientMessage::MulliganResp(payload) => {
                    let mulligan_resp_payload = payload.mulligan_response;
                    println!("Mulligan Decision: {}", mulligan_resp_payload.decision);
                }
                _ => {}
            }
        } else if let Ok(mgrsc_event) =
            serde_json::from_value::<RequestTypeMGRSCEvent>(event.clone())
        {
            let game_room_info = mgrsc_event
                .match_game_room_state_changed_event
                .game_room_info;
            let game_room_config = game_room_info.game_room_config;
            let players = game_room_info.players;
            let match_id = game_room_config.match_id;
            println!("Match ID: {}", match_id);
            if let Some(players) = players {
                for player in players {
                    println!("Player Name: {}, Team #{}", player.player_name, player.team_id);
                }
            }
            if let Some(final_match_result) = game_room_info.final_match_result {
                for result in final_match_result.result_list {
                    if result.scope == "MatchScope_Match" {
                        println!("Winning Team: {}", result.winning_team_id);
                    }
                    if result.scope == "MatchScope_Match" {
                        println!("Match Winning Team: {}", result.winning_team_id);
                    }
                }
            }
        } else if let Ok(request_gre_to_client_event)  = serde_json::from_value::<RequestTypeGREToClientEvent>(event.clone()) {
            let gre_to_client_event = request_gre_to_client_event.gre_to_client_event;
            for gre_to_client_message in gre_to_client_event.gre_to_client_messages {
                match gre_to_client_message {
                    GREToClientMessage::ConnectResp(payload) => {
                        let connect_resp_payload = payload.connect_response;
                        let deck_message = connect_resp_payload.deck_message;
                        let main_deck_card_names = deck_message.deck_cards.iter().map(|card_id| {
                            let card_id = card_id.to_string();
                            let card = self.cards_db.get("cards").unwrap().get(&card_id).unwrap();
                            card.get("pretty_name").unwrap().as_str().unwrap().to_string()
                        }).collect::<Vec<String>>();
                        let sideboard_card_names = deck_message.sideboard_cards.iter().map(|card_id| {
                            let card_id = card_id.to_string();
                            let card = self.cards_db.get("cards").unwrap().get(&card_id).unwrap();
                            card.get("pretty_name").unwrap().as_str().unwrap().to_string()
                        }).collect::<Vec<String>>();
                        println!("Main Deck: {:?}", main_deck_card_names);
                        println!("Sideboard: {:?}", sideboard_card_names);
                    },
                    _ => {}
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
