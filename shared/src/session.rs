// shared/src/session.rs — Session identity types
// Shared between server and client for player identity at the protocol boundary.
// ADR-011: SessionToken for reconnection; PlayerId for in-session routing.
// ADR-012: PlayerId is inserted into SessionConfig at SessionReady; never reassigned.

use serde::{Deserialize, Serialize};

/// Stable per-player identity within one game session.
///
/// Maps to Lightyear ClientId (u64) at the S2C/C2S boundary.
/// Server creates one PlayerId per connected client at lobby formation.
/// The value is opaque — callers must not interpret or construct it manually.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u64);
