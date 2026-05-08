// server/src/core/session/state.rs -- Lobby and session scaffold state.

use std::collections::{HashMap, HashSet};

use bevy::prelude::{Entity, Resource};
use lightyear::prelude::PeerId;
use shared::card::ClassId;
use shared::protocol::{
    CardSource, S2CAuctionBidAccepted, S2CAuctionBidRejected, S2CDraftOffering, S2CGameOver,
    S2CGameSnapshot, S2CGoldUpdate, S2CObjectiveIdentities, S2CSessionCancelled,
    S2CSessionSettingsUpdated, S2CShopSlots,
};
use shared::session::PlayerId;
use uuid::Uuid;

/// Team identifier assigned by the Game Session System at SessionReady.
pub type TeamId = u8;

#[derive(Debug, Clone, PartialEq)]
pub struct SessionSlot {
    pub index: u8,
    pub team: TeamId,
    pub player: Option<PlayerId>,
    pub class: Option<ClassId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum LobbyState {
    LobbyWaiting,
    LobbyReady,
    GameActive,
    LobbyCancelled,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomCode(pub String);

pub type SessionToken = [u8; 16];

#[derive(Debug, Clone, PartialEq, Resource)]
pub struct SessionSlots(pub Vec<SessionSlot>);

#[derive(Debug, Default, Resource)]
pub struct ClassSelections(pub HashMap<PlayerId, ClassId>);

#[derive(Debug, Default, Resource)]
pub struct ClassPreviews(pub HashMap<PlayerId, ClassId>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Resource)]
pub struct PlacementTimerMultiplierRequests(
    pub HashMap<PlayerId, shared::protocol::PlacementTimerMultiplier>,
);

/// Authoritative per-player class identity for one game session.
#[derive(Debug, Resource, Default)]
pub struct PlayerSessions {
    pub players: HashMap<PlayerId, PlayerSessionData>,
}

#[derive(Debug, Clone)]
pub struct PlayerSessionData {
    /// ClassId::Neutral is the "not yet chosen" sentinel during LOBBY.
    pub class: ClassId,
    /// Set once the LOBBY -> DRAFT_INITIAL gate succeeds.
    pub class_locked: bool,
}

impl Default for PlayerSessionData {
    fn default() -> Self {
        Self {
            class: ClassId::Neutral,
            class_locked: false,
        }
    }
}

impl PlayerSessions {
    /// Returns the session class for a registered player.
    pub fn class_of(&self, player_id: PlayerId) -> ClassId {
        self.players
            .get(&player_id)
            .expect("class_of: player not registered in PlayerSessions")
            .class
    }

    /// Returns true when the player is registered and class-locked.
    pub fn is_locked(&self, player_id: PlayerId) -> bool {
        self.players
            .get(&player_id)
            .map(|player| player.class_locked)
            .unwrap_or(false)
    }

    /// Gate check for the LOBBY -> DRAFT_INITIAL transition.
    pub fn all_classes_chosen(&self) -> bool {
        self.players
            .values()
            .all(|player| player.class != ClassId::Neutral)
    }

    /// Locks every registered class atomically after the gate passes.
    pub fn lock_all_classes(&mut self) {
        for player in self.players.values_mut() {
            debug_assert!(
                player.class != ClassId::Neutral,
                "lock_all_classes: player has Neutral class -- gate should have blocked this"
            );
            player.class_locked = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct LobbyDeadline(pub f64);

#[derive(Debug, Clone, PartialEq, Resource)]
pub struct LobbyHeartbeats(pub HashMap<PlayerId, f64>);

#[derive(Debug, Clone)]
pub enum DeferredMessage {
    GameOver(S2CGameOver),
    SessionCancelled(S2CSessionCancelled),
    GoldUpdate(S2CGoldUpdate),
    ObjectiveIdentities(S2CObjectiveIdentities),
    DraftOffering(S2CDraftOffering),
    ShopSlots(S2CShopSlots),
    AuctionBidRejected(S2CAuctionBidRejected),
    AuctionBidAccepted(S2CAuctionBidAccepted),
    CardAcquired {
        card_id: shared::card::CardId,
        source: CardSource,
    },
    PrismRewardDropped {
        player_id: PlayerId,
        lane: u8,
    },
}

/// Per-session reconnect state. Story 007 owns snapshot construction; game-over
/// teardown only cleans this resource up if it is present.
#[derive(Debug, Default, Resource)]
pub struct ReconnectTracker {
    pub snapshot_sent: HashMap<PlayerId, bool>,
    pub deferred_queue: HashMap<PlayerId, Vec<DeferredMessage>>,
    pub token_map: HashMap<SessionToken, (SessionId, PlayerId)>,
    pub pending_hellos: HashMap<PeerId, PendingHello>,
    pub sang_meprise_sent_to: HashSet<PlayerId>,
}

/// Authoritative GAME_OVER result retained during the result acknowledgement
/// window. It is removed only after every participant acknowledges or the
/// configured acknowledgement timeout expires.
#[derive(Debug, Clone, Resource)]
pub struct EndedSessionResultState {
    pub result: S2CGameOver,
    pub participants: HashSet<PlayerId>,
    pub acknowledged: HashSet<PlayerId>,
    pub final_snapshots: HashMap<PlayerId, S2CGameSnapshot>,
    pub expires_at_ms: u64,
    pub session_ids: HashSet<SessionId>,
}

impl EndedSessionResultState {
    pub fn acknowledge(&mut self, player_id: PlayerId) -> bool {
        if !self.participants.contains(&player_id) {
            return false;
        }

        self.acknowledged.insert(player_id)
    }

    pub fn all_acknowledged(&self) -> bool {
        !self.participants.is_empty()
            && self
                .participants
                .iter()
                .all(|player| self.acknowledged.contains(player))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingHello {
    pub entity: Entity,
    pub remaining_ms: u32,
}

/// Testable reconnect dispatch log. The reconnect systems append every ordered
/// send and close request here before attempting the live Lightyear send.
#[derive(Debug, Default, Resource)]
pub struct ReconnectNetworkOutbox {
    dispatches: Vec<crate::core::session::reconnect::ReconnectDispatch>,
    closes: Vec<crate::core::session::reconnect::ReconnectClose>,
}

impl ReconnectNetworkOutbox {
    pub fn push_dispatch(&mut self, dispatch: crate::core::session::reconnect::ReconnectDispatch) {
        self.dispatches.push(dispatch);
    }

    pub fn extend_dispatches(
        &mut self,
        dispatches: impl IntoIterator<Item = crate::core::session::reconnect::ReconnectDispatch>,
    ) {
        self.dispatches.extend(dispatches);
    }

    pub fn push_close(&mut self, close: crate::core::session::reconnect::ReconnectClose) {
        self.closes.push(close);
    }

    pub fn extend_closes(
        &mut self,
        closes: impl IntoIterator<Item = crate::core::session::reconnect::ReconnectClose>,
    ) {
        self.closes.extend(closes);
    }

    pub fn dispatches(&self) -> &[crate::core::session::reconnect::ReconnectDispatch] {
        &self.dispatches
    }

    pub fn closes(&self) -> &[crate::core::session::reconnect::ReconnectClose] {
        &self.closes
    }
}

/// Testable network dispatch log for Game Session System S2C messages.
#[derive(Debug, Default, Clone, Resource)]
pub struct SessionNetworkOutbox {
    session_cancelled: Vec<S2CSessionCancelled>,
    session_settings_updated: Vec<S2CSessionSettingsUpdated>,
    game_over: Vec<S2CGameOver>,
}

impl SessionNetworkOutbox {
    pub fn push_session_cancelled(&mut self, message: S2CSessionCancelled) {
        self.session_cancelled.push(message);
    }

    pub fn push_game_over(&mut self, message: S2CGameOver) {
        self.game_over.push(message);
    }

    pub fn push_session_settings_updated(&mut self, message: S2CSessionSettingsUpdated) {
        self.session_settings_updated.push(message);
    }

    pub fn session_cancelled(&self) -> &[S2CSessionCancelled] {
        &self.session_cancelled
    }

    pub fn session_settings_updated(&self) -> &[S2CSessionSettingsUpdated] {
        &self.session_settings_updated
    }

    pub fn game_over(&self) -> &[S2CGameOver] {
        &self.game_over
    }
}

/// Server-level one-active-session guard.
///
/// Maps each player currently occupying a room slot to the session they are in.
/// Cleanup is owned by later disconnect/game-over stories.
#[derive(Debug, Default, Resource)]
pub struct ActiveSessions(pub HashMap<PlayerId, SessionId>);

/// Transient network connection identity for each stable session player.
#[derive(Debug, Default, Resource)]
pub struct PlayerConnectionMap(pub HashMap<PeerId, PlayerId>);

/// Monotonic allocator for fresh primary-client identities before room entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct NextFreshPlayerId(pub u64);

impl Default for NextFreshPlayerId {
    fn default() -> Self {
        Self(1)
    }
}

impl NextFreshPlayerId {
    pub fn allocate_avoiding(&mut self, used: &HashSet<PlayerId>) -> PlayerId {
        loop {
            let candidate = PlayerId(self.0.max(1));
            self.0 = self.0.saturating_add(1).max(1);

            if !used.contains(&candidate) {
                return candidate;
            }
        }
    }
}

/// All room records currently known to this server process.
#[derive(Debug, Default, Resource)]
pub struct RoomSessions {
    by_id: HashMap<SessionId, RoomSession>,
    by_code: HashMap<RoomCode, SessionId>,
}

#[derive(Debug, Clone)]
pub struct RoomSession {
    pub session_id: SessionId,
    pub room_code: RoomCode,
    pub owner: PlayerId,
    pub mode: shared::protocol::GameMode,
    pub state: LobbyState,
    pub slots: SessionSlots,
    pub lobby_deadline: LobbyDeadline,
    pub heartbeats: LobbyHeartbeats,
}

impl RoomSessions {
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn session_ids(&self) -> Vec<SessionId> {
        self.by_id.keys().copied().collect()
    }

    pub fn contains_room_code(&self, room_code: &RoomCode) -> bool {
        self.by_code.contains_key(room_code)
    }

    pub fn get(&self, session_id: SessionId) -> Option<&RoomSession> {
        self.by_id.get(&session_id)
    }

    pub fn get_mut(&mut self, session_id: SessionId) -> Option<&mut RoomSession> {
        self.by_id.get_mut(&session_id)
    }

    pub fn get_by_code(&self, room_code: &RoomCode) -> Option<&RoomSession> {
        let session_id = self.by_code.get(room_code).copied()?;
        self.by_id.get(&session_id)
    }

    pub fn get_mut_by_code(&mut self, room_code: &RoomCode) -> Option<&mut RoomSession> {
        let session_id = self.by_code.get(room_code).copied()?;
        self.by_id.get_mut(&session_id)
    }

    pub fn insert(&mut self, session: RoomSession) {
        self.by_code
            .insert(session.room_code.clone(), session.session_id);
        self.by_id.insert(session.session_id, session);
    }
}
