// Shared protocol types: C2S/S2C message definitions and registration manifest.
// ADR-003 keeps shared/ dependency-light: pure serde data, no Bevy plugins.
// ADR-008 assigns every message to exactly one of two Lightyear channels.

use crate::card::{CardId, ClassId};
use crate::session::PlayerId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Ordered, guaranteed delivery.
/// ADR-008: all game-state and control messages use this channel.
pub struct ReliableChannel;

/// Best-effort delivery.
/// ADR-008: only heartbeat messages use this channel in the current protocol.
pub struct UnreliableChannel;

/// Direction used by the protocol registration manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolDirection {
    ClientToServer,
    ServerToClient,
}

/// Channel assignment used by the protocol registration manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolChannel {
    Reliable,
    Unreliable,
}

/// Minimal dependency-free registration adapter.
///
/// Server/client networking stories adapt this manifest to Lightyear 0.26:
/// `app.add_channel::<C>(ChannelSettings { .. })` and
/// `app.register_message::<M>().add_direction(NetworkDirection::...)`.
pub trait ProtocolRegistry {
    fn add_channel<C: Send + Sync + 'static>(&mut self, channel: ProtocolChannel);
    fn add_message<M: Serialize + DeserializeOwned + Send + Sync + 'static>(
        &mut self,
        direction: ProtocolDirection,
        channel: ProtocolChannel,
    );
}

/// Registers all protocol channels and message assignments.
///
/// Lightyear 0.26 API shape is verified in
/// `tests/evidence/lightyear-026-verification.md` items 1-3.
pub fn register_protocol(registry: &mut impl ProtocolRegistry) {
    // Lightyear 0.26: channel syntax verified in tests/evidence/lightyear-026-verification.md items 1-2.
    registry.add_channel::<ReliableChannel>(ProtocolChannel::Reliable);
    // Lightyear 0.26: channel syntax verified in tests/evidence/lightyear-026-verification.md items 1-2.
    registry.add_channel::<UnreliableChannel>(ProtocolChannel::Unreliable);

    register_c2s::<C2SHello>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SCreateRoom>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SJoinRoom>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SClassChoice>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SSelectClass>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SConfirmClass>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SPurchaseCard>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SRefreshShop>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SActivateCard>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SSignalReady>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SPlaceBid>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SSubmitPlacement>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SAcknowledgeResult>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SHeartbeat>(registry, ProtocolChannel::Unreliable);

    register_s2c::<S2CHandshake>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CHandshakeRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPhaseChanged>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGameOver>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGoldUpdate>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGoldBroadcast>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CCardAcquired>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CShopSlots>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CDraftOffering>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPoolUpdate>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPlacementReveal>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CResolutionEvent>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionCard>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionBidAccepted>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionSettled>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionBidRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2COpponentDisconnected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2COpponentReconnected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CRoomCreated>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CCreateRoomRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CJoinAck>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CJoinRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CSlotUpdated>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CClassLocked>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CClassesRevealed>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CConfirmClassRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CSessionCancelled>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CObjectiveIdentities>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CSangMepriseReveal>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGameSnapshot>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CHeartbeat>(registry, ProtocolChannel::Unreliable);
}

fn register_c2s<M: Serialize + DeserializeOwned + Send + Sync + 'static>(
    registry: &mut impl ProtocolRegistry,
    channel: ProtocolChannel,
) {
    // Lightyear 0.26: message direction is set with add_direction, verified in tests/evidence/lightyear-026-verification.md item 3.
    registry.add_message::<M>(ProtocolDirection::ClientToServer, channel);
}

fn register_s2c<M: Serialize + DeserializeOwned + Send + Sync + 'static>(
    registry: &mut impl ProtocolRegistry,
    channel: ProtocolChannel,
) {
    // Lightyear 0.26: message direction is set with add_direction, verified in tests/evidence/lightyear-026-verification.md item 3.
    registry.add_message::<M>(ProtocolDirection::ServerToClient, channel);
}

pub type SessionToken = [u8; 16];
pub type EntityId = u64;
pub type AcquisitionSource = CardSource;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameMode {
    OneVOne,
    TwoVTwo,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RoundPhase {
    #[default]
    Handshaking,
    Lobby,
    DraftInitial,
    DraftShop,
    DraftAuction,
    Placement,
    Resolution,
    GameOver,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DraftPhase {
    Initial,
    Auction,
    Shop,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameOverReason {
    ObjectivesDestroyed,
    Disconnect,
    Draw,
    ResolutionTimeout,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionCancelledReason {
    LobbyTimeout,
    PlayerDisconnected,
    ServerRngFail,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinRejectedReason {
    SlotOccupied,
    SessionFull,
    RoomNotFound,
    InvalidSlot,
    AlreadyInSession,
    SessionNotJoinable,
    SessionInProgress,
    InvalidMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreateRoomRejectedReason {
    AlreadyInSession,
    InvalidMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BidRejectedReason {
    InsufficientGold,
    AmountTooLow,
    AuctionExpired,
    AlreadyLeader,
    HandFull,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfirmClassRejectedReason {
    ClassAlreadyConfirmed,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardSource {
    ShopPurchase,
    DraftInitial,
    AuctionWon,
    FreeCardPick,
    PrismLane1,
    PrismLane2,
    PrismLane3,
    PrismLane4,
    PrismLane5,
    KeywordEffect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlacedCard {
    pub card_id: CardId,
    pub owner_id: PlayerId,
    pub target: PlayTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PlayTarget {
    BoardCell { lane: u8, cell: u8 },
    TargetUnit { lane: u8, unit_id: EntityId },
    TargetObj { player_id: PlayerId, lane: u8 },
    LaneWide { lane: u8 },
    Instant,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TaggedEvent {
    pub sub_step: u8,
    pub event: ResolutionEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ResolutionEvent {
    UnitMoved {
        unit_id: EntityId,
        from_lane: u8,
        to_lane: u8,
        from_cell: u8,
        to_cell: u8,
    },
    UnitDied {
        unit_id: EntityId,
        lane: u8,
        cell: u8,
        killer_id: Option<EntityId>,
    },
    TrapTriggered {
        trap_id: EntityId,
        triggering_unit_id: EntityId,
        lane: u8,
        cell: u8,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SessionSlot {
    pub slot: u8,
    pub team: u8,
    pub player_id: Option<PlayerId>,
    pub class_id: Option<ClassId>,
    pub class_confirmed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SHello {
    pub protocol_version: u32,
    pub session_token: Option<SessionToken>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SCreateRoom {
    pub mode: GameMode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SJoinRoom {
    pub room_code: String,
    pub requested_slot: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SClassChoice {
    pub class: ClassId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SSelectClass {
    pub class_id: ClassId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SConfirmClass {
    pub class_id: ClassId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SPurchaseCard {
    pub card_id: CardId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SRefreshShop {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SActivateCard {
    pub card_id: CardId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SSignalReady {
    pub retract: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SPlaceBid {
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SSubmitPlacement {
    pub placements: Vec<PlacedCard>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SAcknowledgeResult {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SHeartbeat {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CHandshake {
    pub protocol_version: u32,
    pub session_id: u64,
    pub session_token: SessionToken,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CHandshakeRejected {
    pub server_version: u32,
    pub client_version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CPhaseChanged {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub timer_duration_ms: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CGameOver {
    pub loser: Option<PlayerId>,
    pub round: u32,
    pub reason: GameOverReason,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CGoldUpdate {
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CGoldBroadcast {
    pub player_id: PlayerId,
    pub gold: u32,
    pub reserved_gold: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CCardAcquired {
    pub card_id: CardId,
    pub source: CardSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CShopSlots {
    pub slots: Vec<Option<CardId>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CDraftOffering {
    pub card_ids: Vec<CardId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CPoolUpdate {
    pub updates: Vec<(CardId, u8)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CPlacementReveal {
    pub placements: Vec<PlacedCard>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CResolutionEvent {
    pub events: Vec<TaggedEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CAuctionCard {
    pub card_id: CardId,
    pub starting_price: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CAuctionBidAccepted {
    pub bidder: PlayerId,
    pub amount: u32,
    pub new_timer_ms: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CAuctionSettled {
    pub winner: Option<PlayerId>,
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CAuctionBidRejected {
    pub reason: BidRejectedReason,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2COpponentDisconnected {
    pub player_id: PlayerId,
    pub grace_remaining_ms: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2COpponentReconnected {
    pub player_id: PlayerId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CRoomCreated {
    pub session_id: String,
    pub room_code: String,
    pub mode: GameMode,
    pub slots: Vec<SessionSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CCreateRoomRejected {
    pub reason: CreateRoomRejectedReason,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CJoinAck {
    pub session_id: String,
    pub mode: GameMode,
    pub slots: Vec<SessionSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CJoinRejected {
    pub reason: JoinRejectedReason,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CSlotUpdated {
    pub slots: Vec<SessionSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CClassLocked {
    pub class_id: ClassId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CClassesRevealed {
    pub player_class_map: Vec<(PlayerId, ClassId)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CConfirmClassRejected {
    pub reason: ConfirmClassRejectedReason,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CSessionCancelled {
    pub reason: SessionCancelledReason,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CObjectiveIdentities {
    pub identities: Vec<(u8, bool)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CSangMepriseReveal {
    pub identities: Vec<(u8, bool)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub player_id: PlayerId,
    pub class_id: ClassId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitBoardLocation {
    BoardCell { lane: u8, cell: u8 },
    ObjectiveAttachment { lane: u8 },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitStatsSnapshot {
    pub hp: u8,
    pub atk: u8,
    pub mp: u8,
    pub ar: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitBoardState {
    pub unit_id: EntityId,
    pub owner_id: PlayerId,
    pub location: UnitBoardLocation,
    pub stats: Option<UnitStatsSnapshot>,
    pub source_class: Option<ClassId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CGameSnapshot {
    // TODO(GSS epic): expand to full S2CGameSnapshot schema.
    pub player_id: PlayerId,
    pub players: Vec<PlayerSnapshot>,
    pub units: Vec<UnitBoardState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CHeartbeat {}
