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

#[derive(Debug, Clone, Copy, Resource)]
pub struct LobbyDeadline(pub f64);

#[derive(Debug, Resource)]
pub struct LobbyHeartbeats(pub HashMap<PlayerId, f64>);
