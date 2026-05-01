use bevy::prelude::Component;
use serde::{Deserialize, Serialize};
use shared::session::PlayerId;

/// Stable key for one player's prism in one lane.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrismLaneKey {
    pub player: PlayerId,
    /// One-indexed lane number.
    pub lane: u8,
}

/// Public prism visibility state replicated to all clients.
#[derive(Component, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrismPresence {
    pub collected: bool,
}
