//! Server-authoritative Prism System scaffold.
//!
//! PRISM-001 defines session-scoped prism resources, replicated presence
//! entities, lifecycle cleanup, and the no-op resolver scaffold used by later
//! reward-routing stories.
#![allow(dead_code, unused_imports)]

pub mod components;
pub mod messages;
pub mod plugin;
pub mod state;
pub mod system;

pub use components::{PrismLaneKey, PrismPresence};
pub use messages::PrismCollected;
pub use plugin::{PrismPlugin, PrismSystemSet};
pub use state::{AuditLog, DiscardLog, PrismAuditEntry, PrismState, MAX_PLAYERS, PRISM_LANE_COUNT};
pub use system::{
    cleanup_prism_session, initialize_prism_session, resolve_prism_draws,
    PrismCardAcquiredDispatch, PrismNetworkOutbox, PrismRewardDroppedDispatch,
};
