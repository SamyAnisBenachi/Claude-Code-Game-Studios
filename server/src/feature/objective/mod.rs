//! Server-authoritative Objective System state model.
//!
//! Objective identity remains server-only per ADR-001. Only `ObjectiveHp` is
//! registered for replication.

pub mod plugin;
pub mod state;
pub mod system;

pub use crate::core::objective_contract::ObjectiveCounters;
pub use plugin::ObjectivePlugin;
pub use state::{HiddenObjectives, ObjectiveHp, ObjectiveSlot, OBJECTIVE_LANE_COUNT};
pub use system::initialize_objectives_on_draft_initial;
