// Shared protocol types — C2S/S2C message definitions
// ADR-008: two channels only (ReliableChannel, UnreliableChannel)
// ADR-002: all message types here; zero game logic
//
// ADR-003 fallback (2026-04-29): lightyear `shared` feature does not exist in 0.26.
// register_protocol() lives in server/main.rs and client/main.rs, not here.
// shared/ only defines plain serde types — zero Lightyear or Bevy plugin deps.

use serde::{Deserialize, Serialize};

/// Reliable ordered channel — all game-state and control messages.
/// ADR-008: use for everything except C2SHeartbeat and S2CAuctionUpdate.
pub struct ReliableChannel;

/// Best-effort channel — high-frequency display-only messages.
/// ADR-008: use ONLY for C2SHeartbeat and S2CAuctionUpdate.
pub struct UnreliableChannel;

/// Current game phase — shared between server (RoundState) and wire protocol.
/// ADR-009: defined here so S2CPhaseChanged and S2CGameSnapshot can embed it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub enum RoundPhase {
    #[default]
    Lobby,
    DraftInitial,
    DraftAuction,
    DraftShop,
    Placement,
    Resolution,
    GameOver,
}

/// Draft sub-phase used by RSM phase-entry messages.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DraftPhase {
    Initial,
    Auction,
    Shop,
}

/// Game modes supported — all modes ship (ADR-003, milestones.md).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GameMode {
    OneVOne,
    TwoVTwo,
    ThreeVThree,
    OneVOneVOne,
    TwoVTwoVTwo,
}

/// Opaque session identity token issued at first connect.
/// ADR-011: sole identity bridge across Lightyear ClientId reassignment.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SessionToken(pub [u8; 16]);

/// Reason for game over — round-state-machine.md Rule 14.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GameOverReason {
    ObjectivesDestroyed,
    Disconnection,
    Draw,
}

// C2S messages (client → server) — stubs, expanded per network-protocol.md
// ADR-002: every handler validates phase + sender identity before processing

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct C2SHello {
    pub protocol_version: u32,
    pub session_token: Option<SessionToken>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct C2SHeartbeat;

// S2C messages (server → client) — stubs, expanded per network-protocol.md

/// No-op S2C message — mini-spike proof that S2C registration compiles end-to-end.
// TODO(Epic 4): replace with real S2C stub once protocol is populated.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct S2CHeartbeat;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S2CHandshake {
    pub session_token: SessionToken,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S2CPhaseChanged {
    pub phase: RoundPhase,
    pub round: u32,
    pub timer_ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct S2CGameOver {
    pub loser: Option<String>, // PlayerId — placeholder until shared/ defines it
    pub reason: GameOverReason,
}
