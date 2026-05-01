// server/src/core/session -- Game Session System core scaffold.
#![allow(dead_code, unused_imports)]

// S3-01 defines scaffold types before downstream GSS stories consume them.

pub mod config;
pub mod events;
pub mod plugin;
pub mod snapshot;
pub mod state;
pub mod system;

pub use config::{build_session_config, SessionConfig};
pub use events::{SessionCancelled, SessionCancelledReason, SessionReady};
pub use plugin::GameSessionPlugin;
pub use snapshot::build_snapshot;
pub use state::{
    ActiveSessions, ClassSelections, LobbyDeadline, LobbyHeartbeats, LobbyState,
    PlayerConnectionMap, PlayerSessionData, PlayerSessions, RoomCode, RoomSession, RoomSessions,
    SessionId, SessionSlot, SessionSlots, SessionToken, TeamId,
};
pub use system::{
    create_room, generate_unique_room_code, handle_create_room, handle_join_room, initialise_slots,
    join_room, normalise_room_code, protocol_slots, room_code_from_bytes, CreateRoomOutcome,
    JoinRoomOutcome, ROOM_CODE_LEN,
};
