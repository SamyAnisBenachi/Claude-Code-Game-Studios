use bevy::prelude::*;
use shared::session::PlayerId;
use std::collections::HashMap;

/// Forward-declared contract owned by the Objective System epic.
///
/// The RSM reads this resource to evaluate GAME_OVER after RESOLUTION. The
/// Objective System is responsible for populating it with real objective losses.
#[derive(Resource, Default, Clone, Debug)]
pub struct ObjectiveCounters {
    pub destroyed_per_player: HashMap<PlayerId, u32>,
}

impl ObjectiveCounters {
    pub fn real_objectives_destroyed(&self, player: PlayerId) -> u32 {
        self.destroyed_per_player.get(&player).copied().unwrap_or(0)
    }
}
