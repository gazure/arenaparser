#![allow(unused)]
use anyhow::Result;

use crate::{CardsDatabase, CARDS_DB};
use crate::mtga_events::client::{ClientMessage, RequestTypeClientToMatchServiceMessage};
use crate::mtga_events::gre::{GreMeta, GREToClientMessage, MulliganReq, MulliganReqWrapper, Parameter, RequestTypeGREToClientEvent};
use crate::mtga_events::mgrc_event::{
    RequestTypeMGRSCEvent, StateType,
};

pub fn do_process_arena_event(event: &str) -> Result<()> {
    if event.contains("clientToMatchServiceMessage") {
        let client_to_match_service_message: RequestTypeClientToMatchServiceMessage =
            serde_json::from_str(event)?;
        match client_to_match_service_message.payload {
            ClientMessage::PerformActionResp(payload) => {
                let action_resp_payload = payload.perform_action_response;
                for action in action_resp_payload.actions {
                    if let Some(grp_id) = action.grp_id {
                        let grp_id = grp_id.to_string();
                        let pretty_name = CARDS_DB.get_pretty_name(&grp_id).unwrap();
                    }
                }
            }
            ClientMessage::MulliganResp(payload) => {
            }
            ClientMessage::ChooseStartingPlayerResp(resp) => {
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
        match game_room_info.state_type {
            StateType::Playing => {
                if let Some(players) = players {
                }
            }
            StateType::MatchCompleted => {
                let final_match_result = game_room_info.final_match_result.unwrap();
            }
        }
    } else if event.contains("greToClientEvent") {
        let request_gre_to_client_event: RequestTypeGREToClientEvent =
            serde_json::from_str(event)?;
        let gre_to_client_event = request_gre_to_client_event.gre_to_client_event;
        for gre_to_client_message in gre_to_client_event.gre_to_client_messages {
            process_gre_message(gre_to_client_message, );
        }
    }
    Ok(())
}
pub fn process_arena_event(cards: CardsDatabase) {
    let cards_db = &cards.db;
}

fn process_gre_message(message: GREToClientMessage) {
    match message {
        GREToClientMessage::GameStateMessage(wrapper) => {
            let game_state = wrapper.game_state_message;
            let annotations = game_state.annotations;
            let game_objects = game_state.game_objects;
            let zones = game_state.zones;
            let turn_info = game_state.turn_info;
        }
        GREToClientMessage::ConnectResp(wrapper) => {
            let connect = wrapper.connect_resp;
            let maindeck = connect.deck_message.deck_cards;
            let sideboard = connect.deck_message.sideboard_cards;
        }
        GREToClientMessage::ChooseStartingPlayerReq(_) => {
            // Nothing of interest in here, check Client message
        }
        GREToClientMessage::MulliganReq(wrapper) => match wrapper {
            MulliganReqWrapper {
                mulligan_req: MulliganReq {
                    mulligan_count,
                    type_field: mulligan_type,
                },
                prompt: Some(prompt),
                meta:
                    GreMeta {
                        system_seat_ids: seat_ids,
                        game_state_id: _,
                        msg_id: _,
                    },
            } => {
                let seat_id = match seat_ids.as_slice() {
                    [seat_id] => seat_id,
                    _ => {
                        return;
                    }
                };
                match prompt.parameters.as_slice() {
                    [Parameter {
                        number_value: Some(cards_in_hand),
                        ..
                    }] => {
                    }
                    _ => {}
                }
            }
            _ => {}
        },
        GREToClientMessage::DieRollResults(wrapper) => {
            let _results = wrapper.die_roll_results_resp.player_die_rolls;
            // TODO send to match handler
        }
        _ => {}
    }
}

// #[cfg(test)]
// mod tests {
//     use serde_json::json;
//
//     use super::*;
//
//     #[test]
//     fn test_process_event() -> Result<()> {
//         let event = json!({
//             "clientToMatchServiceMessageType": "ClientToMatchServiceMessage",
//             "requestId": 6,
//             "payload": {
//                 "gameStateId": 23,
//                 "performActionResp": {
//                     "actions": [
//                         {
//                             "actionType": "ActionType_Play",
//                             "facetId": 163,
//                             "grpId": 58445,
//                             "instanceId": 163,
//                             "shouldStop": true
//                         }
//                     ],
//                     "autoPassPriority": "AutoPassPriority_Yes"
//                 },
//                 "respId": 37,
//                 "type": "ClientMessageType_PerformActionResp"
//             },
//             "timestamp": "638494452827308839",
//             "transactionId": "7bf08a41-032c-4dc8-a842-762a0b71c04b"
//         });
//         let client_to_match_service_message: RequestTypeClientToMatchServiceMessage =
//             serde_json::from_value(event)?;
//         assert_eq!(client_to_match_service_message.request_id, 6);
//         match client_to_match_service_message.payload {
//             ClientMessage::PerformActionResp(payload) => {
//                 let action_resp_payload = payload.perform_action_response;
//                 for action in action_resp_payload.actions {
//                     assert_eq!(action.action_type, "ActionType_Play");
//                     assert_eq!(action.facet_id, 163);
//                     assert_eq!(action.grp_id, 58445);
//                     assert_eq!(action.instance_id, 163);
//                 }
//             }
//             _ => {
//                 assert!(false, "Expected PerformActionResp")
//             }
//         }
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_parse_mgrsc_event() {
//         let event = json!({
//           "matchGameRoomStateChangedEvent": {
//             "gameRoomInfo": {
//               "gameRoomConfig": {
//                 "matchId": "a75fba61-77a2-4cee-bfbd-039cd95ba1d7",
//                 "reservedPlayers": [
//                   {
//                     "connectionInfo": {
//                       "connectionState": "ConnectionState_Open"
//                     },
//                     "courseId": "Avatar_Basic_Aragorn_LTR",
//                     "eventId": "AIBotMatch",
//                     "platformId": "Windows",
//                     "playerName": "tehsbe",
//                     "sessionId": "be39d6c8-e288-47ff-a33c-fa982e8bc1d8",
//                     "systemSeatId": 1,
//                     "teamId": 1,
//                     "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU"
//                   },
//                   {
//                     "connectionInfo": {
//                       "connectionState": "ConnectionState_Open"
//                     },
//                     "courseId": "Avatar_Basic_Sparky",
//                     "eventId": "AIBotMatch",
//                     "isBotPlayer": true,
//                     "playerName": "Sparky",
//                     "systemSeatId": 2,
//                     "teamId": 2,
//                     "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU_Familiar"
//                   }
//                 ]
//               },
//               "players": [
//                 {
//                   "playerName": "tehsbe",
//                   "systemSeatId": 1,
//                   "teamId": 1,
//                   "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU"
//                 },
//                 {
//                   "playerName": "Sparky",
//                   "systemSeatId": 2,
//                   "teamId": 2,
//                   "userId": "CJUMDQAAGNBUTMMN72XYYWDEBU_Familiar"
//                 }
//               ],
//               "stateType": "MatchGameRoomStateType_Playing"
//             }
//           },
//           "requestId": 2,
//           "timestamp": "1713848448700",
//           "transactionId": "ab322eb2-a095-4ad4-91a0-d6bf08bf9482"
//         });
//         let mgrsc_event: RequestTypeMGRSCEvent = serde_json::from_value(event).unwrap();
//         assert_eq!(mgrsc_event.request_id, 2);
//     }
// }