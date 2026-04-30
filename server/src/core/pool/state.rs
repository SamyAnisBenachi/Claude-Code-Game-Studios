// server/src/core/pool/state.rs — Pool data structure declarations (ADR-006 Part 2)
//
// Pure type declarations only — no impl methods.
// All methods (including the sole mutation `distribute()`) live in api.rs.
//
// CI invariant: `grep -rE "copies_remaining\.(insert|remove|entry)" server/src/`
// must return matches in api.rs only. Direct field mutation outside api.rs is
// forbidden and caught by the CI grep gate at story close.

use std::collections::HashMap;

use shared::card::{CardId, CardType, ClassId, Rarity};
use shared::session::PlayerId;

/// Error returned when `distribute()` is called incorrectly.
///
/// ADR-006: callers should check `is_available()` before calling `distribute()`
/// to avoid the error path in hot-path code.
#[derive(Debug, PartialEq, Eq)]
pub enum DistributeError {
    /// `copies_remaining` is already 0; cannot decrement further.
    Exhausted,
    /// `card_id` is not present in this player's pool.
    UnknownCard,
}

/// Filter applied to `draw_random()`.
///
/// All fields `None` by default (no restriction). A filter with all `None`
/// matches every card in the pool. Used by draw effects and Prism Lane 3.
#[derive(Default)]
pub struct PoolFilter {
    /// Restrict to one card type (e.g., Minion only).
    pub card_type: Option<CardType>,
    /// Restrict to a specific class or Neutral.
    pub class: Option<ClassId>,
    /// Restrict to specific rarities.
    pub rarity: Option<Vec<Rarity>>,
    /// Maximum mana cost (inclusive).
    pub max_cost: Option<u32>,
}

/// Per-player pool state, scoped to one game session.
///
/// Initialized from `CardCatalog` + `GameConfig` at `SessionReady` via
/// `PlayerPool::initialize()`. Destroyed with the session on `GameOverEmitted`.
///
/// # Invariant
/// `copies_remaining[id] + total_acquired(id) == initial_count[id]` for all `id`.
/// Maintained exclusively by `distribute()` in `api.rs`. Direct writes to
/// `copies_remaining` outside `api.rs` violate this invariant and are caught
/// by the CI grep gate.
pub struct PlayerPool {
    /// Current available copies per card. Decremented by `distribute()`.
    /// Never below 0. Direct mutation outside `api.rs` is forbidden.
    pub copies_remaining: HashMap<CardId, u32>,

    /// Copy count at pool initialization. Never mutated after construction.
    /// Required to compute `total_acquired` for Formula 2 weighting:
    ///   `total_acquired(id) = initial_count[id] - copies_remaining[id]`
    pub initial_count: HashMap<CardId, u32>,

    /// Current shop display slots (3 entries). `None` = empty slot.
    /// Populated by `refresh_shop()` (Story 003). Not used in Story 001.
    pub shop_slots: Vec<Option<CardId>>,
}

/// Authoritative collection of all per-player pools for one game session.
///
/// Inserted as `Res<PlayerPools>` by `CardPoolPlugin` at `SessionReady`.
/// Removed by GSS on `GameOverEmitted` (wired in a future story).
///
/// ADR-003: `#[derive(Resource)]` must NOT appear in `shared/` — only here.
#[derive(bevy::prelude::Resource, Default)]
pub struct PlayerPools {
    pub pools: HashMap<PlayerId, PlayerPool>,
}

/// Current compact shop slots per player.
///
/// Vec length may be below the requested slot count when the pool is partially
/// exhausted. Callers render missing entries as empty slots.
#[derive(bevy::prelude::Resource, Default)]
pub struct ShopSlots(pub HashMap<PlayerId, Vec<CardId>>);

/// Initial 9-card draft offerings per player.
///
/// Cleared after DRAFT_INITIAL by the future subscriber/system story.
#[derive(bevy::prelude::Resource, Default)]
pub struct InitialDraftOffering(pub HashMap<PlayerId, Vec<CardId>>);

/// Manual shop refresh count per player for the active DRAFT phase.
///
/// Reset to 0 when the automatic DRAFT-entry refresh is processed.
#[derive(bevy::prelude::Resource, Default)]
pub struct ManualRefreshCount(pub HashMap<PlayerId, u32>);

impl ManualRefreshCount {
    /// Reset one player's manual refresh counter at DRAFT entry.
    ///
    /// Inserts a zero entry for players that have not refreshed yet so future
    /// systems can read a stable value.
    pub fn reset_for_player(&mut self, player: PlayerId) {
        self.0.insert(player, 0);
    }
}
