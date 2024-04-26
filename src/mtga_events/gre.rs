use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestTypeGREToClientEvent {
    pub gre_to_client_event: GREToClientEvent,
    pub request_id: Option<i32>,
    pub timestamp: String,
    pub transaction_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GREToClientEvent {
    pub gre_to_client_messages: Vec<GREToClientMessage>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum GREToClientMessage {
    #[serde(rename = "GREMessageType_ConnectResp")]
    ConnectResp(ConnectRespWrapper),
    #[serde(rename = "GREMessageType_DieRollResultsResp")]
    DieRollResults(DieRollResultsRespWrapper),
    #[serde(rename = "GREMessageType_GameStateMessage")]
    GameStateMessage(GameStateMessageWrapper),
    #[serde(rename = "GREMessageType_ChooseStartingPlayerReq")]
    ChooseStartingPlayerReq(ChooseStartingPlayerReqWrapper),
    #[serde(rename = "GREMessageType_MulliganReq")]
    MulliganReq(MulliganReqWrapper),
    #[serde(rename = "GREMessageType_SelectNReq")]
    SelectNReq(SelectNReqWrapper),
    #[serde(rename = "GREMessageType_ActionsAvailableReq")]
    ActionsAvailableReq(ActionsAvailableReq),
    #[serde(rename = "GREMessageType_SetSettingsResp")]
    SetSettingsResp(SetSettingsRespWrapper),
    #[serde(rename = "GREMessageType_SelectTargetsReq")]
    SelectTargetsReq(SelectTargetsReqWrapper),
    #[serde(rename = "GREMessageType_SubmitTargetsResp")]
    SubmitTargetsResp(SubmitTargetsRespWrapper),
    #[serde(rename = "GREMessageType_CastingTimeOptionsReq")]
    CastingTimeOptionsReq(CastingTimeOptionsReqWrapper),
    #[serde(rename = "GREMessageType_PayCostsReq")]
    PayCostsReq(PayCostsReqWrapper),
    #[serde(rename = "GREMessageType_SelectNResp")]
    SelectNResp(SelectNRespWrapper),
    #[serde(rename = "GREMessageType_DeclareAttackersReq")]
    DeclareAttackersReq(DeclareAttackersReqWrapper),
    #[serde(rename = "GREMessageType_SubmitAttackersResp")]
    SubmitAttackersResp(SubmitAttackersRespWrapper),
    #[serde(rename = "GREMessageType_DeclareBlockersReq")]
    DeclareBlockersReq(DeclareBlockersReqWrapper),
    #[serde(rename = "GREMessageType_SubmitBlockersResp")]
    SubmitBlockersResp(SubmitBlockersRespWrapper),
    #[serde(rename = "GREMessageType_IntermissionReq")]
    IntermissionReq(IntermissionReqWrapper),
    #[serde(rename = "GREMessageType_PromptReq")]
    PromptReq(PromptReqWrapper),
    #[serde(rename = "GREMessageType_QueuedGameStateMessage")]
    QueuedGameStateMessage(QueuedStateMessageWrapper),
    #[serde(rename = "GREMessageType_TimerStateMessage")]
    TimerStateMessage(TimerStateMessageWrapper),
    #[serde(rename = "GREMessageType_UIMessage")]
    UIMessage(UIMessageWrapper),
    #[serde(rename = "GREMessageType_SubmitDeckConfirmation")]
    SubmitDeckConfirmation(SubmitDeckConfirmationWrapper),
    #[serde(rename = "GREMessageType_OrderReq")]
    OrderReq(OrderReqWrapper),
    #[serde(rename = "GREMessageType_SubmitDeckReq")]
    SubmitDeckReq(SubmitDeckReqWrapper),
    #[serde(rename = "GREMessageType_SearchReq")]
    SearchReq(SearchReqWrapper),
    #[serde(rename = "GREMessageType_OptionalActionMessage")]
    OptionalActionMessage(OptionalActionMessageWrapper),
    #[serde(rename = "GREMessageType_GroupReq")]
    GroupReq(GroupReqWrapper),
    #[serde(rename = "GREMessageType_GroupResp")]
    GroupRespWrapper(GroupRespWrapper),
    #[default]
    Default,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreMeta {
    #[serde(default)]
    pub msg_id: i32,
    #[serde(default)]
    pub system_seat_ids: Vec<i32>,
    pub game_state_id: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRespWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalActionMessageWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitDeckReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitBlockersRespWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclareBlockersReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIMessageWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitDeckConfirmationWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAttackersRespWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclareAttackersReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectNRespWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayCostsReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntermissionReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastingTimeOptionsReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChooseStartingPlayerReqWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectNReqWrapper {
    pub select_n_req: SelectNReq,
    pub prompt: Option<Prompt>,
    pub allow_cancel: Option<String>,
    #[serde(default)]
    pub allow_undo: bool,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTargetsRespWrapper {
    pub submit_targets_resp: SubmitTargetsResp,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectTargetsReqWrapper {
    pub select_targets_req: SelectTargetsReq,
    pub prompt: Option<Prompt>,
    pub allow_cancel: Option<String>,
    #[serde(default)]
    pub allow_undo: bool,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRespWrapper {
    pub connect_resp: ConnectResp,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DieRollResultsRespWrapper {
    pub die_roll_results_resp: DieRollResultsResp,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsAvailableReqWrapper {
    pub actions_available_req: ActionsAvailableReq,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MulliganReqWrapper {
    pub mulligan_req: MulliganReq,
    pub prompt: Option<Prompt>,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptReqWrapper {
    pub prompt: Prompt,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSettingsRespWrapper {
    pub set_settings_resp: SetSettingsResp,
    #[serde(flatten)]
    pub meta: GreMeta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedStateMessageWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerStateMessageWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectNReq {
    pub context: String,
    pub id_type: Option<String>,
    #[serde(default)]
    pub ids: Vec<i32>,
    pub list_type: String,
    pub max_sel: i32,
    #[serde(default)]
    pub max_weight: i32,
    pub min_sel: i32,
    #[serde(default)]
    pub min_weight: i32,
    pub option_context: String,
    pub prompt: Option<Prompt>,
    pub source_id: Option<i32>,
    #[serde(default)]
    pub unfiltered_ids: Vec<i32>,
    pub validation_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsAvailableReq {
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub inactive_actions: Vec<Action>,
    pub prompt: Option<Prompt>,
    pub game_state_id: i32,
    pub msg_id: i32,
    pub system_seat_ids: Vec<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectTargetsReq {
    pub ability_grp_id: i32,
    pub source_id: Option<i32>,
    pub targets: Vec<SelectTargetsSpec>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectTargetsSpec {
    pub max_targets: i32,
    #[serde(default)]
    pub min_targets: i32,
    pub prompt: Option<Prompt>,
    pub selected_targets: Option<i32>,
    pub target_idx: i32,
    pub targeting_ability_grp_id: i32,
    pub targets: Vec<Target>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTargetsResp {
    pub result: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSettingsResp {
    pub settings: Settings,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mulliganType")]
pub enum MulliganReq {
    #[default]
    #[serde(rename = "MulliganType_London")]
    London,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub prompt_id: Option<i32>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub parameter_name: String,
    pub reference: Option<Reference>,
    pub prompt_id: Option<i32>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub number_value: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub id: i32,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResp {
    pub deck_message: DeckMessage,
    pub gre_changelist: i32,
    pub gre_version: GreVersion,
    pub grp_version: GrpVersion,
    pub proto_ver: String,
    pub settings: Settings,
    pub skins: Vec<Skin>,
    pub status: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckMessage {
    pub deck_cards: Vec<i32>,
    pub sideboard_cards: Vec<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreVersion {
    pub build_version: i32,
    pub major_version: i32,
    pub minor_version: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrpVersion {
    pub major_version: i32,
    pub minor_version: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_optional_payment_cancellation_setting: String,
    pub auto_pass_option: String,
    pub auto_select_replacement_setting: Option<String>,
    pub auto_tap_stops_setting: String,
    pub default_auto_pass_option: String,
    pub graveyard_order: String,
    pub mana_selection_type: String,
    pub smart_stops_setting: String,
    pub stack_auto_pass_option: String,
    pub stops: Vec<Stop>,
    pub transient_stops: Vec<Stop>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub applies_to: String,
    pub status: String,
    pub stop_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub catalog_id: i32,
    pub skin_code: String,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateMessageWrapper {
    #[serde(flatten)]
    pub meta: GreMeta,
    pub game_state_message: GameStateMessage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateMessage {
    #[serde(default)]
    pub actions: Vec<ActionWrapper>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default)]
    pub diff_deleted_instance_ids: Vec<i32>,
    #[serde(default)]
    pub diff_deleted_persistent_annotation_ids: Vec<i32>,
    #[serde(default)]
    pub game_objects: Vec<GameObject>,
    pub game_state_id: i32,
    #[serde(default)]
    pub persistent_annotations: Vec<Annotation>,
    #[serde(default)]
    pub players: Vec<Player>,
    pub prev_game_state_id: Option<i32>,
    #[serde(default)]
    pub timers: Vec<Timer>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub update: String,
    #[serde(default)]
    pub zones: Vec<Zone>,
    pub turn_info: Option<TurnInfo>,
    pub pending_message_count: Option<i32>,
    pub game_info: Option<GameInfo>,
    pub teams: Option<Vec<Team>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionWrapper {
    pub action: Action,
    pub seat_id: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub action_type: String,
    pub instance_id: Option<i32>,
    #[serde(default)]
    pub mana_cost: Vec<ManaCost>,
    pub ability_grp_id: Option<i32>,
    #[serde(default)]
    pub mana_payment_options: Vec<ManaPaymentOption>,
    pub facet_id: Option<i32>,
    pub grp_id: Option<i32>,
    pub should_stop: Option<bool>,
    pub auto_tap_solution: Option<AutoTapSolution>,
    #[serde(default)]
    pub targets: Vec<TargetCollection>,
    pub is_batchable: Option<bool>,
    pub unique_ability_id: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManaCost {
    pub color: Vec<String>,
    pub count: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManaPaymentOption {
    pub mana: Vec<Mana>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mana {
    pub ability_grp_id: i32,
    pub color: String,
    pub mana_id: i32,
    pub specs: Vec<Spec>,
    pub src_instance_id: i32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Spec {
    #[default]
    #[serde(rename = "ManaSpecType_Predictive")]
    Predictive,
    #[serde(rename = "ManaSpecType_FromCave")]
    FromCave,
    #[serde(rename = "ManaSpecType_FromCreature")]
    FromCreature,
    #[serde(rename = "ManaSpecType_Restricted")]
    Restricted,
    #[serde(rename = "ManaSpecType_FromTreasure")]
    FromTreasure,
    #[serde(rename = "ManaSpecType_AdditionalEffect")]
    AdditionalEffect,
    #[serde(rename = "ManaSpecType_CantBeCountered")]
    CantBeCountered,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTapSolution {
    pub auto_tap_actions: Vec<Action>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetCollection {
    pub target_idx: i32,
    pub targets: Vec<Target>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub highlight: Option<String>,
    pub target_instance_id: i32,
    pub legal_action: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub affected_ids: Vec<i32>,
    pub affector_id: Option<i32>,
    pub id: i32,
    #[serde(rename = "type")]
    pub type_field: Vec<String>,
    #[serde(default)]
    pub details: Vec<AnnotationDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationType {
    ResolutionStart,
    ResolutionComplete,
    RevealedCardCreated,
    RevealedCardDeleted,
    ObjectIdChanged,
    ZoneTransfer,
    SyntheticEvent,
    ModifiedLife,
    EnteredZoneThisTurn,
    PhaseOrStepModified,
    NewTurnStarted,
    UserActionTaken,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationDetail {
    pub key: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub value_int32: Vec<i32>,
    #[serde(default)]
    pub value_string: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameObject {
    #[serde(default)]
    pub abilities: Vec<i32>,
    #[serde(default)]
    pub card_types: Vec<String>,
    #[serde(default)]
    pub color: Vec<String>,
    pub controller_seat_id: i32,
    pub grp_id: i32,
    pub instance_id: i32,
    pub name: Option<i32>,
    pub overlay_grp_id: Option<i32>,
    pub owner_seat_id: i32,
    #[serde(rename = "type")]
    pub type_field: GameObjectType,
    #[serde(default)]
    pub viewers: Vec<i32>,
    pub visibility: String,
    pub zone_id: Option<i32>,
    #[serde(default)]
    pub subtypes: Vec<String>,
    #[serde(default)]
    pub super_types: Vec<String>,
    pub base_skin_code: Option<String>,
    pub skin_code: Option<String>,
    pub is_tapped: Option<bool>,
    pub power: Option<Power>,
    pub toughness: Option<Toughness>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameObjectType {
    #[default]
    #[serde(rename = "GameObjectType_Card")]
    Card,
    #[serde(rename = "GameObjectType_RevealedCard")]
    RevealedCard,
    #[serde(rename = "GameObjectType_TriggerHolder")]
    TriggerHolder,
    #[serde(rename = "GameObjectType_MDFCBack")]
    MDFCBack,
    #[serde(rename = "GameObjectType_Ability")]
    Ability,
    #[serde(rename = "GameObjectType_Token")]
    Token,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Power {
    #[serde(default)]
    pub value: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Toughness {
    #[serde(default)]
    pub value: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub deck_constraint_info: DeckConstraintInfo,
    pub game_number: i32,
    #[serde(rename = "matchID")]
    pub match_id: String,
    pub match_state: String,
    pub match_win_condition: String,
    pub mulligan_type: String,
    pub stage: String,
    pub super_format: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub variant: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckConstraintInfo {
    pub max_deck_size: i32,
    pub max_sideboard_size: i32,
    pub min_deck_size: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsmPlayer {
    pub controller_seat_id: i32,
    pub controller_type: String,
    pub life_total: i32,
    pub max_hand_size: i32,
    pub starting_life_total: i32,
    pub system_seat_number: i32,
    pub team_id: i32,
    pub timer_ids: Vec<i32>,
    pub pending_message_type: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: i32,
    pub player_ids: Vec<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub behavior: String,
    pub duration_sec: i32,
    pub timer_id: i32,
    #[serde(rename = "type")]
    pub type_field: String,
    pub warning_threshold_sec: Option<i32>,
    pub running: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnInfo {
    pub active_player: Option<i32>,
    pub decision_player: Option<i32>,
    pub next_phase: Option<String>,
    pub next_step: Option<String>,
    pub phase: Option<String>,
    pub priority_player: Option<i32>,
    pub turn_number: Option<i32>,
    pub step: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub owner_seat_id: Option<i32>,
    #[serde(rename = "type")]
    pub type_field: ZoneType,
    pub visibility: Visibility,
    pub zone_id: i32,
    #[serde(default)]
    pub viewers: Vec<i32>,
    #[serde(default)]
    pub object_instance_ids: Vec<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    #[default]
    #[serde(rename = "Visibility_Public")]
    Public,
    #[serde(rename = "Visibility_Private")]
    Private,
    #[serde(rename = "Visibility_Hidden")]
    Hidden,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum ZoneType {
    #[default]
    #[serde(rename = "ZoneType_Battlefield")]
    Battlefield,
    #[serde(rename = "ZoneType_Stack")]
    Stack,
    #[serde(rename = "ZoneType_Exile")]
    Exile,
    #[serde(rename = "ZoneType_Graveyard")]
    Graveyard,
    #[serde(rename = "ZoneType_Hand")]
    Hand,
    #[serde(rename = "ZoneType_Library")]
    Library,
    #[serde(rename = "ZoneType_Limbo")]
    Limbo,
    #[serde(rename = "ZoneType_Sideboard")]
    Sideboard,
    #[serde(rename = "ZoneType_Pending")]
    Pending,
    #[serde(rename = "ZoneType_Suppressed")]
    Suppressed,
    #[serde(rename = "ZoneType_Revealed")]
    Revealed,
    #[serde(rename = "ZoneType_RevealedSideboard")]
    RevealedSideboard,
    #[serde(rename = "ZoneType_RevealedExile")]
    RevealedExile,
    #[serde(rename = "ZoneType_RevealedGraveyard")]
    RevealedGraveyard,
    #[serde(rename = "ZoneType_RevealedHand")]
    RevealedHand,
    #[serde(rename = "ZoneType_RevealedLibrary")]
    RevealedLibrary,
    #[serde(rename = "ZoneType_RevealedLimbo")]
    RevealedLimbo,
    #[serde(rename = "ZoneType_RevealedStack")]
    RevealedStack,
    #[serde(rename = "ZoneType_RevealedBattlefield")]
    RevealedBattlefield,
    #[serde(rename = "ZoneType_RevealedCommand")]
    RevealedCommand,
    #[serde(rename = "ZoneType_Command")]
    Command,
    #[serde(rename = "ZoneType_RevealedCommandZone")]
    RevealedCommandZone,
    #[serde(rename = "ZoneType_RevealedTemporary")]
    RevealedTemporary,
    #[serde(rename = "ZoneType_Temporary")]
    Temporary,
    #[serde(rename = "ZoneType_RevealedTemporaryZone")]
    RevealedTemporaryZone,
    #[serde(rename = "ZoneType_RevealedToken")]
    RevealedToken,
    #[serde(rename = "ZoneType_Token")]
    Token,
    #[serde(rename = "ZoneType_RevealedTokenZone")]
    RevealedTokenZone,
    #[serde(rename = "ZoneType_RevealedPlayer")]
    RevealedPlayer,
    #[serde(rename = "ZoneType_Player")]
    Player,
    #[serde(rename = "ZoneType_RevealedPlayerZone")]
    RevealedPlayerZone,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DieRollResultsResp {
    pub player_die_rolls: Vec<PlayerDieRoll>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDieRoll {
    pub roll_value: i32,
    pub system_seat_id: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub controller_seat_id: i32,
    pub controller_type: String,
    #[serde(default)]
    pub life_total: i32,
    pub max_hand_size: i32,
    pub starting_life_total: i32,
    pub system_seat_number: i32,
    pub team_id: i32,
    pub timer_ids: Vec<i32>,
    pub pending_message_type: Option<String>,
    pub turn_number: Option<i32>,
}
