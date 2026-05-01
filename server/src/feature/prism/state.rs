use bevy::prelude::Resource;
use shared::card::CardId;
use shared::session::PlayerId;

/// Number of prism lanes owned by each player.
pub const PRISM_LANE_COUNT: usize = 5;

/// Current scaffold capacity for authoritative player-indexed prism state.
pub const MAX_PLAYERS: usize = 2;

/// Server-authoritative prism collection state.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PrismState {
    /// Collected flags indexed as `[player_index][lane_index]`.
    pub collected: [[bool; PRISM_LANE_COUNT]; MAX_PLAYERS],
    /// Transient per-RESOLUTION respawn flags.
    pub pending_respawn: [bool; MAX_PLAYERS],
}

/// Test-facing discard record for stale collection messages.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct DiscardLog {
    pub entries: Vec<(PlayerId, u8)>,
}

/// RNG audit log entries produced by Prism reward draws.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct AuditLog {
    pub entries: Vec<PrismAuditEntry>,
}

/// One server-only Prism RNG audit entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrismAuditEntry {
    pub player_id: PlayerId,
    pub lane: u8,
    pub seed_index: u32,
    pub result: Option<CardId>,
}
