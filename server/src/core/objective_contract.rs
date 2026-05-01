use bevy::prelude::*;
use shared::session::PlayerId;
use std::collections::HashMap;

/// Objective destruction counters exposed to core systems.
///
/// The Objective System owns mutations; the RSM reads real-objective losses to
/// evaluate GAME_OVER after RESOLUTION without importing Feature-layer modules.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub struct ObjectiveCounters {
    /// Destroyed real objectives keyed by defending player.
    pub real_destroyed: HashMap<PlayerId, u32>,
    /// Destroyed fake objectives keyed by attacking player for spawn range.
    pub fake_destroyed: HashMap<PlayerId, u32>,
}

impl ObjectiveCounters {
    /// Reset both counter maps and seed each active player at zero.
    pub fn reset_for_players(&mut self, players: impl IntoIterator<Item = PlayerId>) {
        self.real_destroyed.clear();
        self.fake_destroyed.clear();

        for player in players {
            self.real_destroyed.insert(player, 0);
            self.fake_destroyed.insert(player, 0);
        }
    }

    /// Number of real objectives destroyed for `player`.
    pub fn real_objectives_destroyed(&self, player: PlayerId) -> u32 {
        self.real_destroyed.get(&player).copied().unwrap_or(0)
    }

    /// Number of fake objectives rewarded for `player`.
    #[allow(dead_code)]
    pub fn fake_objectives_destroyed(&self, player: PlayerId) -> u32 {
        self.fake_destroyed.get(&player).copied().unwrap_or(0)
    }
}
