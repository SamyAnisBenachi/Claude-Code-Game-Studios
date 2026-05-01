//! Server-authoritative Board / Lane System resource scaffold.
//!
//! BOARD-001 defines the persistent board-grid, occupancy, spawn-range, prism,
//! and structural config resources used by later placement and movement stories.
#![allow(dead_code, unused_imports)]

pub mod plugin;
pub mod state;

pub use plugin::BoardPlugin;
pub use state::{
    BoardCell, BoardConfig, BoardGrid, BoardOccupancy, LaneId, PrismState, SpawnRangeState,
    BOARD_CELLS_PER_LANE, BOARD_LANE_COUNT, BOARD_PLAYER_COUNT,
};
