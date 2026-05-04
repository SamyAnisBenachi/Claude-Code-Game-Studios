//! Server-authoritative Objective System state model.
//!
//! Objective identity remains server-only per ADR-001. Only `ObjectiveHp` is
//! registered for replication.

pub mod plugin;
pub mod state;
pub mod system;

pub use crate::core::objective_contract::ObjectiveCounters;
pub use plugin::ObjectivePlugin;
pub use state::{
    HiddenObjectives, ObjectiveDestroyed, ObjectiveHp, ObjectiveSlot, PendingObjectiveEvents,
    OBJECTIVE_LANE_COUNT,
};
#[allow(unused_imports)]
pub use system::{
    apply_consequence_path, broadcast_objective_events, deliver_objective_identities_on_ready,
    draw_fake_reward, initialize_objectives_on_draft_initial, objective_resolution_ready,
    take_damage, ObjectiveIdentitiesReady, ObjectiveNetworkOutbox, ObjectiveResolutionState,
    FAKE_REWARD_POOL_FILTER,
};
