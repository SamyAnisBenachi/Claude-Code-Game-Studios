use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use shared::session::PlayerId;

use crate::feature::board::LaneId;

/// Number of objective lanes owned by each player.
pub const OBJECTIVE_LANE_COUNT: u8 = 5;

/// Public objective health replicated to all clients.
#[derive(Component, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ObjectiveHp {
    /// Current objective health.
    pub hp: u32,
}

/// Server-side objective slot metadata.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectiveSlot {
    /// One-indexed lane containing this objective.
    pub lane: LaneId,
    /// Player whose objective occupies this slot.
    pub player: PlayerId,
    /// Whether the objective has already been destroyed.
    pub destroyed: bool,
}

/// Server-only hidden objective identity map.
///
/// `true` means the objective is fake. This resource is never replicated and is
/// populated by the fake-assignment story.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct HiddenObjectives {
    /// Hidden identity keyed by `(owning_player, lane)`.
    pub identities: HashMap<(PlayerId, LaneId), bool>,
}
