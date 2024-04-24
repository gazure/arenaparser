use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct RequestTypeMGRSCEvent {
    #[serde(rename = "matchGameRoomStateChangedEvent")]
    pub match_game_room_state_changed_event: MatchGameRoomStateChangedEvent,
    #[serde(rename = "requestId")]
    pub request_id: i32,
    pub timestamp: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct MatchGameRoomStateChangedEvent {
    #[serde(rename = "gameRoomInfo")]
    pub game_room_info: GameRoomInfo,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct GameRoomInfo {
    #[serde(rename = "gameRoomConfig")]
    pub game_room_config: GameRoomConfig,
    pub players: Option<Vec<Player>>,
    #[serde(rename = "finalMatchResult")]
    pub final_match_result: Option<FinalMatchResult>,
    #[serde(rename = "stateType")]
    pub state_type: StateType,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum StateType {
    #[serde(rename = "MatchGameRoomStateType_MatchCompleted")]
    MatchCompleted,
    #[serde(rename = "MatchGameRoomStateType_Playing")]
    Playing,
}

impl Default for StateType {
    fn default() -> Self {
        StateType::Playing
    }
}


#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct FinalMatchResult {
    #[serde(rename = "matchId")]
    pub match_id: String,
    #[serde(rename = "resultList")]
    pub result_list: Vec<ResultList>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResultList {
    pub scope: String,
    #[serde(rename = "winningTeamId")]
    pub winning_team_id: i32,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct GameRoomConfig {
    #[serde(rename = "matchId")]
    pub match_id: String,
}
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Player {
    #[serde(rename = "playerName")]
    pub player_name: String,
    #[serde(rename = "systemSeatId")]
    pub system_seat_id: i32,
    #[serde(rename = "teamId")]
    pub team_id: i32,
    #[serde(rename = "userId")]
    pub user_id: String,
}
