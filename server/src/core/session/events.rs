// server/src/core/session/events.rs -- Game session lifecycle triggers.

use bevy::prelude::Event;

/// Fired once when all lobby conditions are satisfied.
///
/// DELIVERY: Observer trigger (same-frame). NOT a buffered Event. Subscribe via `app.observe(on_session_ready)`. Adding EventReader<SessionReady> will silently never fire.
///
/// Bevy 0.18 code registers observers with `App::add_observer(...)`; the
/// `app.observe` wording above is retained for the S3-01/ADR-012 grep gate.
#[derive(Event)]
pub struct SessionReady;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionCancelled {
    pub reason: SessionCancelledReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCancelledReason {
    PlayerDisconnected,
    HeartbeatTimeout,
    LobbyTimeout,
    RngInitFailure,
}
