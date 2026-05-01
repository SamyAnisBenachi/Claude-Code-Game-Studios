use bevy::prelude::*;

use crate::feature::board::{BoardConfig, BoardGrid, BoardOccupancy, PrismState, SpawnRangeState};

/// Registers server-only board resources.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardGrid>()
            .init_resource::<BoardOccupancy>()
            .init_resource::<SpawnRangeState>()
            .init_resource::<PrismState>()
            .insert_resource(BoardConfig::default());
    }
}
