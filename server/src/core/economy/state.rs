// server/src/core/economy/state.rs -- Economy state declarations.
//
// Pure data only. Mutating behavior lives in api.rs so economy writes stay
// auditable and grep-gatable.

use std::collections::HashMap;

use bevy::prelude::Resource;
use shared::session::PlayerId;

/// Per-player authoritative economy state for one game session.
#[derive(Clone, Debug)]
pub struct PlayerEconomy {
    /// Persistent gold available across rounds.
    pub gold: u32,
    /// Mana available in the current round only.
    pub current_mana: u32,
    /// Persistent reserve mana with no cap.
    pub reserve_mana: u32,
    /// Current per-player mana ceiling.
    pub mana_cap: u32,
    /// Gold reserved by an active auction bid.
    // Scaffold API consumed by downstream stories.
    #[allow(dead_code)]
    pub reserved_gold: u32,
}

/// Authoritative collection of all player economy states for one session.
#[derive(Resource, Default)]
pub struct PlayerEconomies(pub HashMap<PlayerId, PlayerEconomy>);

/// Gold snapshots captured at RESOLUTION end for next-DRAFT interest.
#[derive(Resource, Default)]
pub struct InterestSnapshots(pub HashMap<PlayerId, u32>);

/// Errors returned by economy validation APIs.
// Scaffold API consumed by downstream stories.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpendError {
    /// Player does not have enough available currency.
    InsufficientFunds,
    /// A reserve-only payment was attempted with current mana allocation.
    ReserveOnlyButCurrentProvided,
    /// Hand-size validation failed in a caller-owned system.
    HandFull,
    /// Phase-gate validation failed in a caller-owned system.
    WrongPhase,
    /// The requested player does not exist in `PlayerEconomies`.
    PlayerNotFound,
}
