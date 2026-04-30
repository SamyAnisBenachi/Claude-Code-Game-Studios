// server/src/core/session/state.rs -- Session-scoped immutable configuration.

use std::collections::HashMap;

use bevy::prelude::Resource;
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

/// Team identifier assigned by the Game Session System at SessionReady.
pub type TeamId = u8;

/// Immutable session configuration inserted once when SessionReady fires.
#[derive(Resource, Clone, Debug)]
pub struct SessionConfig {
    pub mode: GameMode,
    pub player_count: u8,
    pub team_map: HashMap<PlayerId, TeamId>,
    pub class_map: HashMap<PlayerId, ClassId>,
}

impl SessionConfig {
    pub fn players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.team_map.keys().copied()
    }
}
