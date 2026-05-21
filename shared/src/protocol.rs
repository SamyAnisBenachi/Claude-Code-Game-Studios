// Shared protocol types: C2S/S2C message definitions and registration manifest.
// ADR-003 keeps shared/ dependency-light: pure serde data, no Bevy plugins.
// ADR-008 assigns every message to exactly one of two Lightyear channels.

use crate::card::{CardId, ClassId};
use crate::keyword::KeywordKind;
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
    register_c2s::<C2SCreateBotRoom>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SAddBot>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SRemoveBot>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SJoinRoom>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SListRooms>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SClassChoice>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SSelectClass>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SConfirmClass>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SPurchaseCard>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SRefreshShop>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SActivateCard>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SSignalReady>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SPlaceBid>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SSetPlacementTimerMultiplier>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SSubmitPlacement>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SAcknowledgeResult>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SRequestSnapshot>(registry, ProtocolChannel::Reliable);
    register_c2s::<C2SHeartbeat>(registry, ProtocolChannel::Unreliable);

    register_s2c::<S2CHandshake>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CHandshakeRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPhaseChanged>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGameOver>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGoldUpdate>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGoldBroadcast>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CCardAcquired>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPrismRewardDropped>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPrismRespawned>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CShopSlots>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CDraftOffering>(registry, ProtocolChannel::Reliable);
    // S2CPoolUpdate removed by S13-PROTO-ORPHAN-DRAIN-001 (Path B): no server
    // producer and no client consumer ever existed; private pool state lives on
    // the server only and reaches the client through `S2CGameSnapshot.PlayerSnapshot.pool_snapshot`.
    register_s2c::<S2CPlacementReveal>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPlacementAccepted>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CPlacementRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CResolutionEvent>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionCard>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionBidAccepted>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionSettled>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CAuctionBidRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2COpponentDisconnected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2COpponentReconnected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CRoomCreated>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CCreateRoomRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CBotActionRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CRoomList>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CJoinAck>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CJoinRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CSlotUpdated>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CClassLocked>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CClassesRevealed>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CConfirmClassRejected>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CSessionSettingsUpdated>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CSessionCancelled>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CObjectiveIdentities>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CSangMepriseReveal>(registry, ProtocolChannel::Reliable);
    register_s2c::<S2CGameSnapshot>(registry, ProtocolChannel::Reliable);
    // PROMPT 1614 (BOT-DEBUG-OVERLAY-IMPLEMENTATION): debug-only god-mode
    // bot-state push gated server-side by `CCGS_BOT_DEBUG_UI=1` and
    // client-side rendered only when `CCGS_DEBUG_UI=1`. Registered
    // unconditionally so the channel binding is symmetric across builds; the
    // server never emits the message unless its config gate is on, and the
    // client overlay never spawns UI unless its own gate is on.
    register_s2c::<S2CDebugBotStatePush>(registry, ProtocolChannel::Reliable);
    // S2CHeartbeat removed by S13-PROTO-ORPHAN-DRAIN-001 (Path B): the GDD's
    // Rule 8 disconnect-detection contract uses `C2SHeartbeat` only (client →
    // server on the unreliable channel); no S2C heartbeat was ever produced or
    // consumed, and the unreliable channel binding has no other S2C message.
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

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum PlacementTimerMultiplier {
    #[default]
    X1,
    X1_5,
    X2,
    X3,
}

impl PlacementTimerMultiplier {
    pub const MULTIPLAYER_STANDARD_VALUES: [Self; 4] = [Self::X1, Self::X1_5, Self::X2, Self::X3];

    pub const fn ratio(self) -> (u32, u32) {
        match self {
            Self::X1 => (1, 1),
            Self::X1_5 => (3, 2),
            Self::X2 => (2, 1),
            Self::X3 => (3, 1),
        }
    }

    pub fn apply_to_ms(self, base_ms: u32) -> u32 {
        let (numerator, denominator) = self.ratio();
        let value = u128::from(base_ms) * u128::from(numerator) / u128::from(denominator);
        u32::try_from(value).unwrap_or(u32::MAX)
    }

    pub fn from_standard_ratio(numerator: u32, denominator: u32) -> Option<Self> {
        match (numerator, denominator) {
            (1, 1) => Some(Self::X1),
            (3, 2) => Some(Self::X1_5),
            (2, 1) => Some(Self::X2),
            (3, 1) => Some(Self::X3),
            _ => None,
        }
    }

    pub fn from_ratio_capped(numerator: u32, denominator: u32) -> Option<Self> {
        if denominator == 0 || numerator < denominator {
            return None;
        }

        if u128::from(numerator) >= 3 * u128::from(denominator) {
            return Some(Self::X3);
        }
        if u128::from(numerator) * 2 >= 4 * u128::from(denominator) {
            return Some(Self::X2);
        }
        if u128::from(numerator) * 2 >= 3 * u128::from(denominator) {
            return Some(Self::X1_5);
        }
        Some(Self::X1)
    }
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

/// PROMPT 1430 (S19-BOT-PROTOCOL-FOUNDATIONS): bot flavour selector carried by
/// `C2SCreateBotRoom` and `C2SAddBot`. Defaults to `Default` so the first wave
/// of UI workers can ship without picking a flavour. Future flavours (e.g.
/// Aggressive, Defensive) extend the enum without a wire-format revision.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BotKind {
    #[default]
    Default,
}

/// PROMPT 1430: reasons the server rejects `C2SCreateBotRoom`, `C2SAddBot`,
/// or `C2SRemoveBot`. Mirrors `S2CJoinRejected` ergonomics so the client can
/// surface a corrective lobby UI state.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BotActionRejectedReason {
    NotOwner,
    SlotOccupied,
    SlotNotBot,
    InvalidSlot,
    UnknownSession,
    SessionNotJoinable,
    BotCapReached,
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

/// Why the server rejected a `C2SSubmitPlacement` batch. Mirrors the rejection
/// variants of the server's internal `PlacementSubmissionResult` so the
/// originating client can surface a corrective UI state instead of remaining
/// stuck on the optimistic Submitted view.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlacementRejectedReason {
    WrongPhase,
    UnknownPlayer,
    DuplicateFinalSubmission,
    MissingCatalog,
    MissingEconomy,
    CardMissingFromCatalog,
    CardNotInHand,
    DuplicateCardId,
    InvalidTarget,
    SpawnRangeRejected,
    OccupancyRejected,
    InsufficientMana,
    OwnerMismatch,
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
pub enum PlayTarget {
    BoardCell { lane: u8, cell: u8 },
    TargetUnit { lane: u8, unit_id: EntityId },
    TargetObj { player_id: PlayerId, lane: u8 },
    LaneWide { lane: u8 },
    Instant,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlacedCardSubmit {
    pub card_id: CardId,
    pub target: PlayTarget,
    pub current_mana_spend: u32,
    pub reserve_mana_spend: u32,
}

impl PlacedCardSubmit {
    /// Intended total payment for this card; BLS-011 owns validation.
    pub const fn total_mana_spend(&self) -> u32 {
        self.current_mana_spend
            .saturating_add(self.reserve_mana_spend)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlacedCardReveal {
    pub owner_id: PlayerId,
    pub card_id: CardId,
    pub target: PlayTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TaggedEvent {
    pub sub_step: u8,
    pub trigger_index: u32,
    pub event: ResolutionEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ResolutionEvent {
    SubStepBegin,
    UnitPlaced {
        unit_id: EntityId,
        player: PlayerId,
        lane: u8,
        cell: u8,
    },
    UnitMoved {
        unit_id: EntityId,
        lane: u8,
        from_cell: u8,
        to_cell: u8,
    },
    UnitChangedLane {
        unit_id: EntityId,
        from_lane: u8,
        to_lane: u8,
    },
    CombatDamage {
        attacker_id: EntityId,
        defender_id: EntityId,
        damage_amount: u8,
        defender_hp_after: u8,
        was_blocked_by_shield: bool,
    },
    UnitRemoved {
        unit_id: EntityId,
        lane: u8,
        cell: u8,
    },
    KeywordTriggered {
        unit_id: EntityId,
        keyword: KeywordKind,
    },
    GoldAwarded {
        player: PlayerId,
        amount: u32,
        reason: GoldReason,
    },
    ObjectiveDamage {
        attacker_id: Option<EntityId>,
        target_player_id: PlayerId,
        lane: u8,
        damage_amount: u32,
        objective_hp_after: u32,
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
    ObjectiveDestroyed {
        target_player_id: PlayerId,
        lane: u8,
        was_fake: bool,
    },
    SpawnRangeChanged {
        player_id: PlayerId,
        new_spawn_range_cells: u8,
    },
    GameOver {
        loser: Option<PlayerId>,
        reason: GameOverReason,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoldReason {
    Kill,
    ObjectiveDestroyed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SessionSlot {
    pub slot: u8,
    pub team: u8,
    pub player_id: Option<PlayerId>,
    pub class_id: Option<ClassId>,
    pub class_confirmed: bool,
    /// PROMPT 1430 (S19-BOT-PROTOCOL-FOUNDATIONS): true when this slot is held
    /// by a server-authored bot rather than a remote human peer. Server-only
    /// authority; the client treats this field as read-only. Existing
    /// human-only rooms serialize this as `false`.
    #[serde(default)]
    pub is_bot: bool,
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

/// PROMPT 1430 (S19-BOT-PROTOCOL-FOUNDATIONS): host requests a fresh room
/// pre-seeded with a bot in the first opposing-team slot. Mirrors
/// `C2SCreateRoom` so the lobby UX can offer a "Play vs Bot" CTA without a
/// follow-up Add Bot round-trip. Server is authoritative — if the caller is
/// already in a session the request is rejected via `S2CBotActionRejected`
/// rather than `S2CCreateRoomRejected`, so the client can distinguish the
/// botted entry point from the human-only path.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SCreateBotRoom {
    pub mode: GameMode,
    pub bot_kind: BotKind,
}

/// PROMPT 1430: room owner requests that a bot occupy a specific empty slot.
/// Server validates ownership + slot before mutating room state and emits an
/// `S2CSlotUpdated` to the room or `S2CBotActionRejected` to the caller.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SAddBot {
    pub slot: u8,
    pub bot_kind: BotKind,
}

/// PROMPT 1430: room owner requests that the bot in `slot` be removed and the
/// seat re-opened. Rejection is unicast to the caller; success broadcasts
/// `S2CSlotUpdated` to the room.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SRemoveBot {
    pub slot: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SJoinRoom {
    pub room_code: String,
    pub requested_slot: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct C2SListRooms {}

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
pub struct C2SSetPlacementTimerMultiplier {
    pub multiplier: PlacementTimerMultiplier,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SSubmitPlacement {
    pub placements: Vec<PlacedCardSubmit>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SAcknowledgeResult {}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct C2SRequestSnapshot {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SHeartbeat {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CHandshake {
    pub protocol_version: u32,
    pub session_id: u64,
    pub session_token: SessionToken,
    pub player_id: PlayerId,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CCardAcquired {
    pub card_id: CardId,
    pub source: CardSource,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CPrismRewardDropped {
    pub player_id: PlayerId,
    pub lane: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CPrismRespawned {
    pub player_id: PlayerId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CShopSlots {
    pub slots: Vec<Option<CardId>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CDraftOffering {
    pub card_ids: Vec<CardId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CPlacementReveal {
    pub placements: Vec<PlacedCardReveal>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CResolutionEvent {
    pub round: u32,
    pub events: Vec<TaggedEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CAuctionCard {
    pub card_id: CardId,
    pub starting_price: u32,
    /// Auction live-bidding countdown duration (ms), sourced from
    /// `GameConfig::auction_timer_seconds` server-side. Replaces the prior
    /// client-side reliance on `S2CPhaseChanged::timer_duration_ms` which
    /// reports 0 for the auction phase per `draft_timer_ms`.
    pub timer_duration_ms: u32,
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
    /// PROMPT 1513 — wire-authoritative card id for the settled auction.
    /// Always present (the server knows the auction card at settle time on
    /// both the winner and no-bid paths). Clients use this to arm winner
    /// disposition state (e.g. `AuctionWonPending`) without depending on
    /// local `auction_state.card_id`, which is cleared shortly after the
    /// settling transition.
    pub card_id: CardId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CAuctionBidRejected {
    pub reason: BidRejectedReason,
}

/// Server-authoritative acceptance feedback for a `C2SSubmitPlacement` batch.
///
/// PROMPT 1546 — sent unicast to the originating client immediately after
/// `process_placement_submission` returns `Accepted`. Pairs with
/// `S2CPlacementRejected` so every `C2SSubmitPlacement` is matched by exactly
/// one server-authored ACK (`Accepted` or `Rejected`) before `S2CPlacementReveal`
/// fires at phase close. This closes the "silent accept" gap surfaced by
/// PROMPT 1476 (P0) and PROMPT 1478, and is the only positive signal the
/// submitter receives for effect-only (Spell/Order/Instant) placements that
/// never appear in reveal payloads and never spawn replicated entities.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct S2CPlacementAccepted {
    /// Server's view of how many placements were committed. Lets the client
    /// cross-check its own staged count.
    pub placements_len: u8,
    /// Mirrors `PlayerSubmission.is_final` server-side. Always `true` today
    /// (`process_placement_submission` only commits final batches); the field
    /// exists for forward compatibility if non-final submissions are added.
    pub is_final: bool,
}

/// Server-authoritative rejection feedback for a `C2SSubmitPlacement` batch.
///
/// Sent unicast to the originating client whenever the server's
/// `handle_placement_submission` logs a `submission rejected` decision so the
/// client can revert stale Submitted state and surface a corrective UI step.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct S2CPlacementRejected {
    pub reason: PlacementRejectedReason,
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

/// PROMPT 1430 (S19-BOT-PROTOCOL-FOUNDATIONS): unicast feedback when the
/// server refuses a `C2SCreateBotRoom`, `C2SAddBot`, or `C2SRemoveBot`.
/// The reason variants mirror `JoinRejectedReason` semantics so the lobby UI
/// can render a single corrective message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CBotActionRejected {
    pub reason: BotActionRejectedReason,
}

/// One joinable room in the lobby browser. See `S2CRoomList` for filters/order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RoomListEntry {
    /// 6-char uppercase alphanumeric code — passable directly to `C2SJoinRoom`.
    pub room_code: String,
    /// Mode determines `slots_max` (2 for OneVOne, 4 for TwoVTwo).
    pub mode: GameMode,
    /// Number of slots already filled. Includes the owner. <= `slots_max`.
    /// PROMPT 1430: bot-occupied slots count toward `slots_filled` so a fully
    /// botted room is not surfaced as joinable.
    pub slots_filled: u8,
    /// Total slots in this room (2 for OneVOne, 4 for TwoVTwo).
    pub slots_max: u8,
    /// First slot index whose `player` is `None`. `None` only appears for
    /// transient cases — `S2CRoomList` itself filters out fully occupied rooms.
    pub first_open_slot: Option<u8>,
    /// PROMPT 1430: number of slots in this room currently held by bots.
    /// Existing human-only rooms serialize this as `0`.
    #[serde(default)]
    pub bot_count: u8,
    /// PROMPT 1430: `true` when at least one non-owner slot is either empty
    /// or held by a remote human peer. Lets the lobby browser flag rooms that
    /// are effectively single-player practice (owner + bots only).
    #[serde(default = "default_has_human_opponent")]
    pub has_human_opponent: bool,
}

/// `has_human_opponent` defaults to `true` when deserializing legacy payloads
/// so older snapshots that pre-date PROMPT 1430 keep their previous semantics
/// (no bots present means a human opponent is at least possible).
fn default_has_human_opponent() -> bool {
    true
}

/// Response to `C2SListRooms`. Only rooms in `LobbyState::LobbyWaiting` with at
/// least one open slot are included. Sorted by `room_code` ascending for stable
/// rendering and snapshot-friendly tests. Empty `Vec` when no rooms are joinable.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct S2CRoomList {
    pub rooms: Vec<RoomListEntry>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CSessionSettingsUpdated {
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
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
    pub gold: u32,
    pub reserved_gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub spawn_range_cells: u8,
    pub mana_cap: u8,
    pub submitted: bool,
    pub hand: Vec<CardId>,
    pub shop_slots: Vec<Option<CardId>>,
    pub pool_snapshot: Vec<(CardId, u8)>,
    pub objectives: Vec<ObjectiveSnapshot>,
    pub opponent_objectives: Vec<OpponentObjectiveSnapshot>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveSnapshot {
    pub lane: u8,
    pub hp: u8,
    pub is_real: bool,
    pub is_destroyed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpponentObjectiveSnapshot {
    pub lane: u8,
    pub hp: u8,
    pub is_destroyed: bool,
    pub was_fake: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveReveal {
    pub player_id: PlayerId,
    pub lane: u8,
    pub is_fake: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct BoardSnapshot {
    pub units: Vec<UnitBoardState>,
    pub traps: Vec<TrapBoardState>,
    pub structures: Vec<StructureBoardState>,
    pub fields: Vec<FieldBoardState>,
    pub prisms: Vec<PrismBoardState>,
    pub seeds: Vec<SeedBoardState>,
    pub sinistros: Vec<SinistroState>,
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
    pub card_id: Option<CardId>,
    pub stats: Option<UnitStatsSnapshot>,
    pub source_class: Option<ClassId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapBoardState {
    pub trap_id: EntityId,
    pub owner: PlayerId,
    pub lane: u8,
    pub cell: u8,
    pub card_id: Option<CardId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureBoardState {
    pub structure_id: EntityId,
    pub card_id: Option<CardId>,
    pub owner: PlayerId,
    pub lane: u8,
    pub cell: u8,
    pub max_hp: u8,
    pub current_hp: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldBoardState {
    pub field_id: EntityId,
    pub card_id: Option<CardId>,
    pub owner: PlayerId,
    pub lane: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrismBoardState {
    pub player_id: PlayerId,
    pub lane: u8,
    pub collected: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedBoardState {
    pub owner: PlayerId,
    pub lane: u8,
    pub cell: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SinistroState {
    pub owner: PlayerId,
    pub lane: u8,
    pub damage: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionSnapshot {
    pub card_id: CardId,
    pub starting_price: u32,
    pub last_accepted_bid: u32,
    pub current_leader: Option<PlayerId>,
    pub timer_remaining_ms: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CGameSnapshot {
    pub protocol_version: u32,
    pub recipient_player_id: PlayerId,
    pub round_number: u32,
    pub phase: RoundPhase,
    pub timer_remaining_ms: Option<u32>,
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
    pub players: Vec<PlayerSnapshot>,
    pub board: BoardSnapshot,
    pub auction_state: Option<AuctionSnapshot>,
    pub active_sang_meprise_reveals: Option<Vec<ObjectiveReveal>>,
}

// ---------------------------------------------------------------------------
// PROMPT 1614 — debug-only bot-state push (data contract: PROMPT 1604).
// ---------------------------------------------------------------------------

/// Debug-only god-mode view of every bot participant in the session.
///
/// The server only emits this message when `CCGS_BOT_DEBUG_UI=1` is set in
/// its environment **and** at least one bot is in the session. Production
/// servers never emit it. The reliable channel is used so the client can
/// trust ordering against `S2CGameSnapshot`, which makes the overlay a
/// non-flickery aid during human QA.
///
/// Carries fields the client cannot reach through normal protocol because
/// they are either redacted in `S2CGameSnapshot.players` for non-recipient
/// players (`hand`) or live only inside server resources (`BotDecisionLog`,
/// `BotState.rng_word_counter`, `AuctionRoundContext.valuation`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CDebugBotStatePush {
    /// One entry per bot in the session. Ordered by `PlayerId` numeric
    /// value for stable rendering.
    pub bots: Vec<DebugBotStateEntry>,
    /// Total entries in the server-side `BotDecisionLog`, so the client
    /// overlay can render "showing N of M" without inspecting the tail
    /// vector length.
    pub decision_log_total: u32,
    /// Server wallclock ms (`Time::elapsed`) at the moment the push was
    /// assembled. The client uses this to label the overlay row with a
    /// staleness indicator.
    pub assembled_at_ms: u64,
}

/// Per-bot god-mode payload bundled inside [`S2CDebugBotStatePush`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DebugBotStateEntry {
    pub player_id: PlayerId,
    pub class_id: Option<ClassId>,
    pub gold: u32,
    pub current_mana: u32,
    pub mana_cap: u8,
    pub submitted: bool,
    /// Full hand (god-mode). Sent only inside the debug push.
    pub hand: Vec<CardId>,
    /// Tail of the bot's decision log (capped by the server).
    pub decision_tail: Vec<DebugBotDecisionEntry>,
    /// Most recent auction-bid valuation produced by the bot's heuristic.
    /// `None` when the bot has not bid this session yet or the last bid
    /// pre-dates the configured tail cap.
    pub last_bid_valuation: Option<u32>,
}

/// Serialisable mirror of one server-side `BotDecisionEntry`. Kept flat —
/// the `kind_label` is the lowercase decision-variant name (e.g.
/// `"auction_bid"`) and `detail` holds a short human-readable summary
/// (e.g. `"card=42 amt=4 val=5"`). Decoupled from the server-only
/// `BotDecisionKind` enum so the shared crate never depends on server
/// internals (per ADR-003).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DebugBotDecisionEntry {
    pub round_number: u32,
    pub phase: RoundPhase,
    pub timestamp_ms: u64,
    pub kind_label: String,
    pub detail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingRegistry {
        messages: Vec<(&'static str, ProtocolDirection, ProtocolChannel)>,
    }

    impl ProtocolRegistry for RecordingRegistry {
        fn add_channel<C: Send + Sync + 'static>(&mut self, _channel: ProtocolChannel) {}

        fn add_message<M: Serialize + DeserializeOwned + Send + Sync + 'static>(
            &mut self,
            direction: ProtocolDirection,
            channel: ProtocolChannel,
        ) {
            self.messages
                .push((std::any::type_name::<M>(), direction, channel));
        }
    }

    #[test]
    fn submit_and_reveal_payloads_use_direction_specific_shapes() {
        let submit = C2SSubmitPlacement {
            placements: vec![PlacedCardSubmit {
                card_id: CardId(7),
                target: PlayTarget::BoardCell { lane: 2, cell: 3 },
                current_mana_spend: 2,
                reserve_mana_spend: 1,
            }],
        };
        let submit_json = serde_json::to_value(&submit).expect("submit payload should serialize");
        assert_eq!(submit.placements[0].total_mana_spend(), 3);
        assert!(submit_json["placements"][0].get("card_id").is_some());
        assert!(submit_json["placements"][0]
            .get("current_mana_spend")
            .is_some());
        assert!(submit_json["placements"][0]
            .get("reserve_mana_spend")
            .is_some());
        assert!(submit_json["placements"][0].get("owner_id").is_none());

        let reveal = S2CPlacementReveal {
            placements: vec![PlacedCardReveal {
                owner_id: PlayerId(11),
                card_id: CardId(7),
                target: PlayTarget::BoardCell { lane: 2, cell: 3 },
            }],
        };
        let reveal_json = serde_json::to_value(&reveal).expect("reveal payload should serialize");
        assert!(reveal_json["placements"][0].get("owner_id").is_some());
        assert!(reveal_json["placements"][0].get("card_id").is_some());
        assert!(reveal_json["placements"][0].get("target").is_some());
        assert!(reveal_json["placements"][0]
            .get("current_mana_spend")
            .is_none());
        assert!(reveal_json["placements"][0]
            .get("reserve_mana_spend")
            .is_none());
        assert!(reveal_json["placements"][0].get("reserve_amount").is_none());
    }

    #[test]
    fn placement_messages_remain_registered_on_reliable_channel() {
        let mut registry = RecordingRegistry::default();
        register_protocol(&mut registry);

        assert_eq!(
            registry
                .messages
                .iter()
                .find(|(name, _, _)| { *name == std::any::type_name::<C2SSubmitPlacement>() }),
            Some(&(
                std::any::type_name::<C2SSubmitPlacement>(),
                ProtocolDirection::ClientToServer,
                ProtocolChannel::Reliable,
            ))
        );
        assert_eq!(
            registry
                .messages
                .iter()
                .find(|(name, _, _)| { *name == std::any::type_name::<S2CPlacementReveal>() }),
            Some(&(
                std::any::type_name::<S2CPlacementReveal>(),
                ProtocolDirection::ServerToClient,
                ProtocolChannel::Reliable,
            ))
        );
        assert_eq!(
            registry
                .messages
                .iter()
                .find(|(name, _, _)| { *name == std::any::type_name::<C2SRequestSnapshot>() }),
            Some(&(
                std::any::type_name::<C2SRequestSnapshot>(),
                ProtocolDirection::ClientToServer,
                ProtocolChannel::Reliable,
            ))
        );
        assert_eq!(
            registry.messages.iter().find(|(name, _, _)| {
                *name == std::any::type_name::<C2SSetPlacementTimerMultiplier>()
            }),
            Some(&(
                std::any::type_name::<C2SSetPlacementTimerMultiplier>(),
                ProtocolDirection::ClientToServer,
                ProtocolChannel::Reliable,
            ))
        );
        assert_eq!(
            registry.messages.iter().find(|(name, _, _)| {
                *name == std::any::type_name::<S2CSessionSettingsUpdated>()
            }),
            Some(&(
                std::any::type_name::<S2CSessionSettingsUpdated>(),
                ProtocolDirection::ServerToClient,
                ProtocolChannel::Reliable,
            ))
        );
    }

    #[test]
    fn placement_timer_multiplier_values_are_multiplayer_safe_and_integer_backed() {
        assert_eq!(
            PlacementTimerMultiplier::MULTIPLAYER_STANDARD_VALUES,
            [
                PlacementTimerMultiplier::X1,
                PlacementTimerMultiplier::X1_5,
                PlacementTimerMultiplier::X2,
                PlacementTimerMultiplier::X3
            ]
        );
        assert_eq!(PlacementTimerMultiplier::X1_5.apply_to_ms(12_000), 18_000);
        assert_eq!(PlacementTimerMultiplier::X3.apply_to_ms(10_000), 30_000);
        assert_eq!(
            PlacementTimerMultiplier::from_standard_ratio(1, 2),
            None,
            "0.5x is not a multiplayer Standard-tier value"
        );
        assert_eq!(
            PlacementTimerMultiplier::from_ratio_capped(4, 1),
            Some(PlacementTimerMultiplier::X3)
        );
    }

    #[test]
    fn lobby_room_browser_messages_are_registered_reliable() {
        let mut registry = RecordingRegistry::default();
        register_protocol(&mut registry);

        assert_eq!(
            registry
                .messages
                .iter()
                .find(|(name, _, _)| { *name == std::any::type_name::<C2SListRooms>() }),
            Some(&(
                std::any::type_name::<C2SListRooms>(),
                ProtocolDirection::ClientToServer,
                ProtocolChannel::Reliable,
            )),
            "C2SListRooms must be registered ClientToServer/Reliable"
        );
        assert_eq!(
            registry
                .messages
                .iter()
                .find(|(name, _, _)| { *name == std::any::type_name::<S2CRoomList>() }),
            Some(&(
                std::any::type_name::<S2CRoomList>(),
                ProtocolDirection::ServerToClient,
                ProtocolChannel::Reliable,
            )),
            "S2CRoomList must be registered ServerToClient/Reliable"
        );
    }

    #[test]
    fn bot_protocol_messages_are_registered_reliable() {
        let mut registry = RecordingRegistry::default();
        register_protocol(&mut registry);

        for name in [
            std::any::type_name::<C2SCreateBotRoom>(),
            std::any::type_name::<C2SAddBot>(),
            std::any::type_name::<C2SRemoveBot>(),
        ] {
            let entry = registry.messages.iter().find(|(n, _, _)| *n == name);
            assert_eq!(
                entry,
                Some(&(
                    name,
                    ProtocolDirection::ClientToServer,
                    ProtocolChannel::Reliable
                )),
                "{name} must be registered ClientToServer/Reliable"
            );
        }

        let s2c_name = std::any::type_name::<S2CBotActionRejected>();
        let entry = registry.messages.iter().find(|(n, _, _)| *n == s2c_name);
        assert_eq!(
            entry,
            Some(&(
                s2c_name,
                ProtocolDirection::ServerToClient,
                ProtocolChannel::Reliable,
            )),
            "S2CBotActionRejected must be registered ServerToClient/Reliable"
        );
    }

    #[test]
    fn session_slot_serializes_is_bot_and_defaults_to_false_on_missing() {
        let slot = SessionSlot {
            slot: 1,
            team: 1,
            player_id: Some(crate::session::PlayerId(42)),
            class_id: None,
            class_confirmed: false,
            is_bot: true,
        };
        let value = serde_json::to_value(&slot).expect("session slot should serialize");
        assert_eq!(
            value.get("is_bot").and_then(|v| v.as_bool()),
            Some(true),
            "is_bot must be present on wire when bot-occupied"
        );

        let human = SessionSlot {
            slot: 0,
            team: 0,
            player_id: None,
            class_id: None,
            class_confirmed: false,
            is_bot: false,
        };
        let human_value = serde_json::to_value(&human).expect("session slot should serialize");
        assert_eq!(
            human_value.get("is_bot").and_then(|v| v.as_bool()),
            Some(false),
            "is_bot must serialize as false for human-only / empty slots"
        );

        // Legacy payload — no is_bot — must deserialize to is_bot=false.
        let legacy = serde_json::json!({
            "slot": 0,
            "team": 0,
            "player_id": null,
            "class_id": null,
            "class_confirmed": false,
        });
        let decoded: SessionSlot =
            serde_json::from_value(legacy).expect("legacy SessionSlot must decode");
        assert!(
            !decoded.is_bot,
            "legacy SessionSlot payloads default to is_bot=false"
        );
    }

    #[test]
    fn room_list_entry_serializes_bot_count_and_has_human_opponent_with_safe_defaults() {
        let entry = RoomListEntry {
            room_code: "ABCDEF".to_string(),
            mode: GameMode::OneVOne,
            slots_filled: 2,
            slots_max: 2,
            first_open_slot: None,
            bot_count: 1,
            has_human_opponent: false,
        };
        let value = serde_json::to_value(&entry).expect("entry should serialize");
        assert_eq!(
            value.get("bot_count").and_then(|v| v.as_u64()),
            Some(1),
            "bot_count must serialize on wire"
        );
        assert_eq!(
            value.get("has_human_opponent").and_then(|v| v.as_bool()),
            Some(false),
            "has_human_opponent must serialize on wire"
        );

        // Legacy payload without the new fields decodes safely: bot_count=0,
        // has_human_opponent=true (the previous implicit semantic).
        let legacy = serde_json::json!({
            "room_code": "XYZ123",
            "mode": "OneVOne",
            "slots_filled": 1,
            "slots_max": 2,
            "first_open_slot": 1,
        });
        let decoded: RoomListEntry =
            serde_json::from_value(legacy).expect("legacy RoomListEntry must decode");
        assert_eq!(decoded.bot_count, 0);
        assert!(decoded.has_human_opponent);
    }

    #[test]
    fn bot_kind_default_is_default_variant_and_round_trips() {
        assert_eq!(BotKind::default(), BotKind::Default);

        let payload = C2SCreateBotRoom {
            mode: GameMode::OneVOne,
            bot_kind: BotKind::Default,
        };
        let json = serde_json::to_value(&payload).expect("C2SCreateBotRoom should serialize");
        assert_eq!(json["mode"], serde_json::json!("OneVOne"));
        assert_eq!(json["bot_kind"], serde_json::json!("Default"));
    }

    #[test]
    fn bot_action_rejected_payload_round_trips() {
        let payload = S2CBotActionRejected {
            reason: BotActionRejectedReason::NotOwner,
        };
        let value = serde_json::to_value(&payload).expect("rejection should serialize");
        assert_eq!(value["reason"], serde_json::json!("NotOwner"));
    }

    #[test]
    fn session_settings_update_payload_has_no_requester_identity() {
        let update = S2CSessionSettingsUpdated {
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X3,
        };
        let value = serde_json::to_value(update).expect("settings update should serialize");

        assert!(value.get("placement_timer_multiplier_effective").is_some());
        assert!(value.get("player_id").is_none());
        assert!(value.get("requester").is_none());
        assert!(value.get("requester_id").is_none());
        assert!(value.get("connection_id").is_none());
    }
}
