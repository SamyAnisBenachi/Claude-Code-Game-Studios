//! Server-authoritative Board / Lane System resource scaffold.
//!
//! BOARD-001 defines the persistent board-grid, occupancy, spawn-range, prism,
//! and structural config resources used by later placement and movement stories.
#![allow(dead_code, unused_imports)]

pub mod movement;
pub mod placement;
pub mod plugin;
pub mod state;

pub use movement::{
    advance_direction, apply_charge_movement, apply_f1, apply_standard_movement, ChargeBonus,
};
pub use placement::{
    close_placement_phase, get_units_at_cell, handle_placement_submission, is_field_slot_available,
    is_minion_slot_available, is_structure_slot_available, is_trap_slot_available,
    placement_buffer_open, process_placement_submission, requires_spawn_range_validation,
    validate_spawn_range, PendingPlacements, PlacementCommitTrace, PlacementCommitTraceEntry,
    PlacementCommitted, PlacementSubmissionReceived, PlacementSubmissionResult, PlayerSubmission,
};
pub use plugin::{BoardPlugin, BoardSystemSet};
pub use state::{
    BoardCell, BoardConfig, BoardGrid, BoardOccupancy, LaneId, PrismState, SpawnRangeState,
    BOARD_CELLS_PER_LANE, BOARD_LANE_COUNT, BOARD_PLAYER_COUNT,
};
