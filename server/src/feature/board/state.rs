use std::collections::HashMap;

use bevy::prelude::{Entity, Resource};
use shared::session::PlayerId;

/// Number of independent lanes in the authoritative board grid.
pub const BOARD_LANE_COUNT: usize = 5;

/// Number of absolute cells in each lane.
pub const BOARD_CELLS_PER_LANE: usize = 8;

/// Number of player sides tracked by server board resources.
pub const BOARD_PLAYER_COUNT: usize = 2;

/// One-indexed lane identifier used by board placement and occupancy state.
pub type LaneId = u8;

/// A live board-cell occupant recorded in the authoritative grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCell {
    /// Entity occupying this grid cell.
    pub entity: Entity,
}

impl BoardCell {
    /// Build a board-cell record for an occupying entity.
    pub const fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

/// Server-authoritative 5-lane by 8-cell spatial grid.
///
/// Cell index 0 maps to absolute cell 1. Cell index 7 maps to absolute cell 8.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct BoardGrid {
    /// Board cells indexed as `[lane_index][cell_index]`.
    pub lanes: [[Option<BoardCell>; BOARD_CELLS_PER_LANE]; BOARD_LANE_COUNT],
}

impl Default for BoardGrid {
    fn default() -> Self {
        Self {
            lanes: [[None; BOARD_CELLS_PER_LANE]; BOARD_LANE_COUNT],
        }
    }
}

/// Server-only occupancy state used by placement validation.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct BoardOccupancy {
    /// Minion occupancy keyed by `(player, lane)`.
    pub minion_slots: HashMap<(PlayerId, LaneId), Entity>,
    /// Trap occupancy keyed by `(player, lane, absolute_cell)`.
    pub traps: HashMap<(PlayerId, LaneId, u8), Entity>,
    /// Structure occupancy keyed by `(player, lane, absolute_cell)`.
    pub structures: HashMap<(PlayerId, LaneId, u8), Entity>,
    /// Field occupancy keyed by `(player, lane)`.
    pub fields: HashMap<(PlayerId, LaneId), Entity>,
}

impl Default for BoardOccupancy {
    fn default() -> Self {
        Self {
            minion_slots: HashMap::new(),
            traps: HashMap::new(),
            structures: HashMap::new(),
            fields: HashMap::new(),
        }
    }
}

/// Per-player fake-objective destruction state for spawn-range validation.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpawnRangeState {
    /// Destroyed fake objective counts indexed by player side, clamped by later stories.
    pub fakes_destroyed: [u8; BOARD_PLAYER_COUNT],
    /// Already-applied fake-destruction facts that the scheduled message bridge should skip.
    pub applied_fake_objective_facts: [u32; BOARD_PLAYER_COUNT],
}

/// Per-player prism presence by lane.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrismState {
    /// Prism presence indexed as `[player_index][lane_index]`.
    pub present: [[bool; BOARD_LANE_COUNT]; BOARD_PLAYER_COUNT],
}

impl Default for PrismState {
    fn default() -> Self {
        Self {
            present: [[true; BOARD_LANE_COUNT]; BOARD_PLAYER_COUNT],
        }
    }
}

/// Structural board constants used by board formulas and validation.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardConfig {
    /// Player A advances toward larger absolute cell values.
    pub player_a_direction: i16,
    /// Player B advances toward smaller absolute cell values.
    pub player_b_direction: i16,
    /// Player A's structural spawn cell.
    pub player_a_spawn_cell: u8,
    /// Player B's structural spawn cell.
    pub player_b_spawn_cell: u8,
    /// Player A's objective target cell.
    pub player_a_objective_cell: u8,
    /// Player B's objective target cell.
    pub player_b_objective_cell: u8,
    /// Number of lanes in the board.
    pub lane_count: u8,
    /// Number of cells per lane.
    pub cells_per_lane: u8,
    /// Minimum valid absolute cell number.
    pub cell_min: u8,
    /// Maximum valid absolute cell number.
    pub cell_max: u8,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            player_a_direction: 1,
            player_b_direction: -1,
            player_a_spawn_cell: 1,
            player_b_spawn_cell: 8,
            player_a_objective_cell: 8,
            player_b_objective_cell: 1,
            lane_count: BOARD_LANE_COUNT as u8,
            cells_per_lane: BOARD_CELLS_PER_LANE as u8,
            cell_min: 1,
            cell_max: BOARD_CELLS_PER_LANE as u8,
        }
    }
}
