// server/src/core/session -- Game Session System core scaffold.
#![allow(dead_code, unused_imports)]

// S3-01 defines scaffold types before downstream GSS stories consume them.

pub mod config;
pub mod events;
pub mod plugin;
pub mod state;

pub use config::{build_session_config, SessionConfig};
pub use events::{SessionCancelled, SessionCancelledReason, SessionReady};
pub use plugin::GameSessionPlugin;
pub use state::{
    ClassSelections, LobbyDeadline, LobbyHeartbeats, LobbyState, RoomCode, SessionId, SessionSlot,
    SessionSlots, SessionToken, TeamId,
};
