// server/src/core/session/state.rs -- Lobby and session scaffold state.

use std::collections::HashMap;

use bevy::prelude::Resource;
use lightyear::prelude::PeerId;
use shared::card::ClassId;
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

/// Server-level one-active-session guard.
///
/// Maps each player currently occupying a room slot to the session they are in.
/// Cleanup is owned by later disconnect/game-over stories.
#[derive(Debug, Default, Resource)]
pub struct ActiveSessions(pub HashMap<PlayerId, SessionId>);

/// Transient network connection identity for each stable session player.
#[derive(Debug, Default, Resource)]
pub struct PlayerConnectionMap(pub HashMap<PeerId, PlayerId>);

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
