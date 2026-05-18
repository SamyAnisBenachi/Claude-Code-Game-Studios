// server/src/core/session -- Game Session System core scaffold.
#![allow(dead_code, unused_imports)]

// S3-01 defines scaffold types before downstream GSS stories consume them.

pub mod config;
pub mod events;
pub mod plugin;
pub mod reconnect;
pub mod snapshot;
pub mod snapshot_request;
pub mod state;
pub mod system;

pub use config::{
    build_session_config, build_session_config_with_settings, effective_placement_timer_multiplier,
    SessionConfig,
};
pub use events::{SessionCancelled, SessionCancelledReason, SessionReady};
pub use plugin::GameSessionPlugin;
pub use reconnect::handle_reconnect as reconnect_snapshot_system;
pub use reconnect::{
    defer_unicast_for_reconnect, flush_deferred_queue, handle_reconnect, hello_timeout_watchdog,
    initialise_reconnect_tracker, on_reconnect_connected, process_reconnect_hello, ReconnectClose,
    ReconnectDispatch, ReconnectProcessResult,
};
pub use snapshot::{build_game_snapshot, build_snapshot};
pub use snapshot_request::{handle_request_snapshot, SnapshotRequestCooldowns};
pub use state::{
    ActiveSessions, ClassPreviews, ClassSelections, DeferredMessage, EndedSessionResultState,
    LobbyDeadline, LobbyHeartbeats, LobbyState, NextFreshPlayerId, PendingHello,
    PlacementTimerMultiplierRequests, PlayerConnectionMap, PlayerSessionData, PlayerSessions,
    ReconnectNetworkOutbox, ReconnectTracker, RoomCode, RoomSession, RoomSessions, SessionId,
    SessionNetworkOutbox, SessionSlot, SessionSlots, SessionToken, TeamId,
};
pub use system::{
    all_classes_confirmed, all_slots_filled, apply_placement_timer_multiplier_request_batch,
    apply_result_acknowledgement, build_room_list, cancel_lobby_by_session, cancel_lobby_for_player,
    cleanup_ended_session_reconnect_state, confirm_class, create_room,
    effective_session_settings_update, evaluate_room_session_ready, evaluate_session_ready,
    f4_session_ready, generate_unique_room_code, handle_confirm_class, handle_create_room,
    handle_game_over_teardown, handle_join_room, handle_list_rooms, handle_lobby_disconnect,
    handle_lobby_heartbeat, handle_placement_timer_multiplier_requests,
    handle_result_acknowledgements, handle_select_class, initialise_slots, join_room,
    lobby_timeout_check, normalise_room_code, protocol_slots, resolve_result_acknowledgement,
    room_code_from_bytes, select_class, tick_ended_session_result_timeout, tick_lobby_heartbeats,
    ConfirmClassOutcome, CreateRoomOutcome, JoinRoomOutcome, ResultAcknowledgementOutcome,
    SelectClassOutcome, ServerRngFactory, ServerRngInitError, SessionSystemSet, ROOM_CODE_LEN,
};
