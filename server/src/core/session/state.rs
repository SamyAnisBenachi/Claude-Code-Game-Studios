// server/src/core/session/state.rs -- Lobby and session scaffold state.

use std::collections::HashMap;

use bevy::prelude::Resource;
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

#[derive(Debug, Clone, PartialEq, Resource)]
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

#[derive(Debug, Resource)]
pub struct SessionSlots(pub Vec<SessionSlot>);

#[derive(Debug, Resource)]
pub struct ClassSelections(pub HashMap<PlayerId, ClassId>);

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

#[derive(Debug, Clone, Copy, Resource)]
pub struct LobbyDeadline(pub f64);

#[derive(Debug, Resource)]
pub struct LobbyHeartbeats(pub HashMap<PlayerId, f64>);
