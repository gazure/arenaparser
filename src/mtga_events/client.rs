use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestTypeClientToMatchServiceMessage {
    #[serde(rename = "clientToMatchServiceMessageType")]
    pub client_to_match_service_message_type: String,
    #[serde(rename = "requestId")]
    pub request_id: i32,
    #[serde(rename = "payload")]
    pub payload: ClientMessage,
    pub timestamp: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "ClientMessageType_ChooseStartingPlayerResp")]
    ChooseStartingPlayerResp(ChooseStartingPlayerResp),
    #[serde(rename = "ClientMessageType_SubmitDeckResp")]
    SubmitDeckResp(SubmitDeckResp),
    #[serde(rename = "ClientMessageType_SetSettingsReq")]
    SetSettingsReq(SetSettingsReq),
    #[serde(rename = "ClientMessageType_PerformActionResp")]
    PerformActionResp(PerformActionResp),
    #[serde(rename = "ClientMessageType_MulliganResp")]
    MulliganResp(MulliganResp),
    #[serde(rename = "ClientMessageType_UIMessage")]
    UIMessage(UIMessage),
    #[serde(rename = "ClientMessageType_SelectNResp")]
    SelectNResp(SelectNResp),
    #[serde(rename = "ClientMessageType_SubmitTargetsReq")]
    SubmitTargetsReq(SubmitTargetsReq),
    #[serde(rename = "ClientMessageType_SubmitTargetsResp")]
    SubmitTargetsResp(SubmitTargetsResp),
    #[serde(rename = "ClientMessageType_SelectTargetsResp")]
    SelectTargetsResp(SelectTargetsResp),
    #[serde(rename = "ClientMessageType_SubmitAttackersReq")]
    SubmitAttackersReq(SubmitAttackersReq),
    #[serde(rename = "ClientMessageType_DeclareAttackersReq")]
    DeclareAttackersReq(DeclareAttackersReq),
    #[serde(rename = "ClientMessageType_DeclareAttackersResp")]
    DeclareAttackersResp(DeclareAttackersResp),
    #[serde(rename = "ClientMessageType_SubmitBlockersReq")]
    SubmitBlockersReq(SubmitBlockersReq),
    #[serde(rename = "ClientMessageType_SubmitBlockersResp")]
    SubmitBlockersResp(SubmitBlockersResp),
    #[serde(rename = "ClientMessageType_DeclareBlockersResp")]
    DeclareBlockersResp(DeclareBlockersResp),
    #[serde(rename = "ClientMessageType_ConcedeReq")]
    ConcedeReq(ConcedeReq),
    #[serde(rename = "ClientMessageType_EffectCostResp")]
    EffectCostResp(EffectCostResp),
    #[serde(rename = "ClientMessageType_CastingTimeOptionsResp")]
    CastingTimeOptionsResp(CastingTimeOptionsResp),
    #[serde(rename = "ClientMessageType_CancelActionReq")]
    CancelActionReq(CancelActionReq),
    #[serde(rename = "ClientMessageType_OrderResp")]
    OrderResp(OrderResp),
    #[serde(rename = "ClientMessageType_SearchResp")]
    SearchResp(SearchResp),
    #[serde(rename = "ClientMessageType_OptionalActionResp")]
    OptionalActionResp(OptionalActionResp),
    #[serde(rename = "ClientMessageType_PerformAutoTapActionsResp")]
    PerformAutoTapActionsResp(PerformAutoTapActionsResp),
    #[serde(rename = "ClientMessageType_EnterSideboardingReq")]
    EnterSideboardingReq(EnterSideboardingReq),
    #[serde(rename = "ClientMessageType_GroupResp")]
    GroupResp(GroupResp),
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct GroupResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct EnterSideboardingReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct PerformAutoTapActionsResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct OptionalActionResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SearchResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct CancelActionReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct CastingTimeOptionsResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SetSettingsReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SubmitDeckResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}


#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChooseStartingPlayerResp {
    #[serde(default)]
    pub system_seat_id: i32,
    pub team_id: i32,
    pub team_type: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SubmitBlockersReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SubmitBlockersResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeclareBlockersResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct EffectCostResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct ConcedeReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SubmitAttackersReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeclareAttackersReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeclareAttackersResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SelectTargetsResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SubmitTargetsReq {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SubmitTargetsResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct SelectNResp {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct UIMessage {
    #[serde(rename = "systemSeatId")]
    #[serde(default)]
    pub system_seat_id: i32,
    #[serde(rename = "uiMessage")]
    pub ui_message: Value,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct MulliganResp {
    #[serde(rename = "gameStateId")]
    pub game_state_id: i32,
    #[serde(rename = "mulliganResp")]
    pub mulligan_response: MulliganDecision,
    #[serde(rename = "respId")]
    pub response_id: i32,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct MulliganDecision {
    pub decision: String,
}


#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct PerformActionResp {
    #[serde(rename = "gameStateId")]
    pub game_state_id: i32,
    #[serde(rename = "performActionResp")]
    pub perform_action_response: PerformActionResponse,
    #[serde(rename = "respId")]
    pub response_id: i32,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct PerformActionResponse {
    #[serde(rename = "actions")]
    pub actions: Vec<Action>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Action {
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(rename = "facetId")]
    pub facet_id: Option<i32>,
    #[serde(rename = "grpId")]
    pub grp_id: Option<i32>,
    #[serde(rename = "instanceId")]
    pub instance_id: Option<i32>,
}
