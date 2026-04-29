# ADR-006: Card Data Schema and Pool State Architecture

## Status
Accepted

## Date
2026-04-29

## Last Verified
2026-04-29

## Decision Makers
User + Lead Programmer + technical-director

## Summary
Card data is split into two distinct concerns: an immutable `CardCatalog`
(`HashMap<CardId, CardData>`) loaded from `assets/data/cards.json` at server
startup and held for the server's lifetime, and a mutable `PlayerPool` scoped
to each game session that tracks per-player copy counts and shop-slot state.
The client loads `cards.json` independently for display lookup; only the server
mutates pool state. All random draws accept explicit seeds from `ServerRng` —
the pool owns no randomness source.

---

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core — Data / Asset Loading |
| **Knowledge Risk** | LOW — `serde_json`, `HashMap`, and `serde` are stable Rust std/crates with no Bevy-version coupling. Asset loading touches `bevy_asset_loader` (MEDIUM risk, see notes). |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`; `design/gdd/card-data-pool.md`; `design/gdd/game-config.md`; `docs/architecture/architecture.md` |
| **Post-Cutoff APIs Used** | `bevy_asset_loader` — requires `#[derive(TypePath)]` in Bevy 0.18. `ron` must be a direct dependency (no longer re-exported from `bevy_asset`). Verify `bevy_asset_loader` version on crates.io for 0.18 compatibility before implementing (see game-config.md OQ1, OQ2, OQ3). |
| **Verification Required** | Confirm `#[derive(Asset, TypePath)]` compiles on `CardCatalogAsset` wrapper with target `bevy_asset_loader` version. Confirm `serde_json` parses the full ~315-card `cards.json` without allocation pathology at server startup (smoke test). |

---

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-003: Cargo Workspace Structure — Accepted (2026-04-29); ADR-004: Asset Loading Pipeline — Accepted (2026-04-29). This ADR defines the schema and pool model. Implementation may start now that both workspace and asset-loading ADRs are accepted. |
| **Enables** | Economy System (card mana costs read from `CardData.cost`); Objective System (free-card draw uses `PlayerPool::draw_random()`); Auction System M2 (shared neutral auction pool drawing from `PlayerPool::draw_auction_card()`); Card Acquisition M2 (shop slot generation); Combat Resolution M2 (reads `CardData` stats and keywords by `CardId`). |
| **Blocks** | Card Pool implementation sprint cannot start until this ADR is accepted; Shop Refresh implementation; any story that reads `CardData` fields or mutates `copies_remaining`. |
| **Ordering Note** | `shared/src/card.rs` must be written before `server/src/core/pool/state.rs` — the server crate depends on `shared`. Client and server both load `cards.json` independently; no coordination protocol is needed because `cards.json` is read-only after loading. |

---

## Context

### Problem Statement

Lanes and Lies needs a card data system that answers two distinct questions with
very different mutability requirements:

1. **"What is this card?"** — Definition lookup: stats, cost, type, rarity,
   keywords. This data is immutable for the lifetime of the server. It must be
   loaded once and shared read-only across all game sessions.

2. **"How many copies of this card are left for this player?"** — Pool state:
   copy counts, shop-slot assignments. This data is per-player, per-session,
   and mutated every time a card is purchased or displayed. It must be
   initialized fresh at session start and destroyed at session end.

Conflating these two concerns in a single mutable structure would create
unnecessary write contention, complicate session teardown, and prevent the
catalog from serving as a stable read reference for the client.

A secondary problem is the weighted draw algorithm: the shop must lean toward
cards the player has already acquired (reinforcing archetype commitment), but
this weighting must be computed from data the pool already owns (`initial_count`
vs. `copies_remaining`) rather than requiring a separate tracking structure.

### Constraints

- All randomness is server-seeded (`ServerRng`) — the pool must never own an
  `Rng` source; callers supply explicit seeds.
- The client loads `cards.json` for display only; no client-side pool state
  exists. Pool mutations are server-authoritative.
- `Epic` (1 copy) and `Legendary` (1 copy) pool counts are load-bearing design
  pillars, not tuning knobs — they are Rust consts, not `GameConfig` fields.
- Load failure of `cards.json` is fatal — the server must not start if the
  catalog is missing, invalid, or contains duplicate IDs.
- The shared neutral auction pool (used by the Auction System) is architecturally
  distinct from each player's personal shop pool. This ADR defines the interface
  (`draw_auction_card`) but delegates shared pool management to the Auction
  System GDD.

### Requirements

- O(1) catalog lookup by `CardId`.
- `distribute()` (the sole pool mutation) must never underflow below 0; it must
  return a typed error if called on an exhausted card.
- Weighted draw (Formula 2 from `card-data-pool.md`) must be computable from
  `initial_count` and `copies_remaining` already held in `PlayerPool` — no
  separate acquired-count tracking is needed (`total_acquired = initial_count -
  copies_remaining`).
- Pool never panics on empty results — all draw functions return `Option<T>`.
- `cards.json` hard validation at load time: duplicate IDs, missing `rarity`,
  and `SHOP_WEIGHT_CAP <= 0` all abort server startup with logged errors.
- `pool_copies_override <= 0` is a soft error: card receives rarity default,
  server continues, error is logged.

---

## Decision

Card data is split into two Rust types with different lifetimes, locations, and
mutability contracts.

### Part 1: CardCatalog — Immutable, Server Lifetime

**Location:** `shared/src/card.rs` (plain Rust, no Bevy derives — `shared/`
purity constraint from `architecture.md` Boundary 1).

```rust
// shared/src/card.rs

use serde::{Deserialize, Serialize};

/// Stable numeric identifier matching Krosmaga Extension=1 card IDs.
/// Newtype wrapper prevents accidental integer arithmetic on IDs.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CardId(pub u32);

/// Rarity tier. Determines base pool copy count and auction eligibility.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Card category. Determines which stat fields are present and valid.
///
/// - `Minion`: has `cost`, `atk`, `hp`, `mp`, `ar`
/// - `Spell`: has `cost` only (beyond base fields)
/// - `Trap`: has `cost` only
/// - `Structure`: has `cost`, `atk`=0, `mp`=0, `hp`, `ar`
/// - `Field`: has `cost` only; effect in `effect_text`
/// - `Order`: has `cost` only
/// - `DoubleFace`: second-face schema TBD (GDD OQ6)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardType {
    Minion,
    Spell,
    Trap,
    Structure,
    Field,
    Order,
    DoubleFace,
}

/// Class affiliation. `Neutral` cards belong to no class and appear in
/// every player's shop neutral pool.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClassId {
    Iop,
    Cra,
    Sacrier,
    Xelor,
    Ecaflip,
    Sadida,
    Neutral,
}

/// Unit type used for archetype/interaction tagging.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitType {
    Blade,
    Arcane,
    Shield,
    Neutral,
}

/// Keyword variants.
///
/// No-parameter keywords are plain enum variants.
/// Parameterized keywords carry their value inline.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Keyword {
    // No-parameter keywords
    Simple(SimpleKeyword),
    // Parameterized keywords
    RangeX     { #[serde(rename = "kw")] kw: String, max_range: u8 },
    ChargeXMove{ #[serde(rename = "kw")] kw: String, cells: u8 },
    ResistanceX{ #[serde(rename = "kw")] kw: String, value: u8 },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimpleKeyword {
    FirstStrike,
    Charge,
    AppearanceTrigger,
    DeathTrigger,
    FinalBlowTrigger,
    CounterattackTrigger,
    StartOfTurnTrigger,
    EndOfTurnTrigger,
    // Additional keywords added here as card content expands
}

/// Immutable definition of one card.
///
/// Loaded from `assets/data/cards.json` at server startup into `CardCatalog`.
/// Never mutated after load. Both server and client hold a copy.
///
/// Stat fields (`atk`, `hp`, `mp`, `ar`, `cost`) are present on all cards but
/// carry zero values where they are semantically absent (e.g., `mp = 0` on
/// Structures). Systems must check `card_type` before using stats.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CardData {
    /// Stable numeric ID. Primary key. Matches Krosmaga Extension=1 ID where applicable.
    pub id: CardId,

    /// Canonical French display name.
    pub name_fr: String,

    /// Canonical English display name.
    pub name_en: String,

    /// Class affiliation. `ClassId::Neutral` for neutral cards.
    pub class: ClassId,

    /// Optional family grouping (neutral cards only; e.g., "Gobball").
    /// `None` for class cards.
    pub family: Option<String>,

    /// Rarity tier. Determines base pool copy count and auction eligibility.
    pub rarity: Rarity,

    /// Card category. Determines valid stat fields and game rules.
    pub card_type: CardType,

    /// Unit archetype tag. Default `UnitType::Neutral` for all cards.
    pub unit_type: UnitType,

    /// Mana cost to play. Present on all card types (0 if not applicable).
    pub cost: u32,

    /// Attack. Minions and Structures only; 0 on all other types.
    pub atk: u8,

    /// Hit points. Minions and Structures only; 0 on all other types.
    pub hp: u8,

    /// Movement points. Minions only; 0 on Structures (WALL) and all other types.
    pub mp: u8,

    /// Armor. Minions and Structures only; 0 on all other types.
    pub ar: u8,

    /// Keyword list. Empty array if none.
    pub keywords: Vec<Keyword>,

    /// Human-readable effect description. Required for trigger cards; empty
    /// string for stat-only Minions.
    pub effect_text: String,

    /// Sprite atlas key for rendering.
    pub art_id: String,

    /// Optional per-card pool copy count override.
    ///
    /// `None` → use rarity default from `GameConfig`.
    /// `Some(n)` where `n >= 1` → use `n` copies.
    /// `Some(n)` where `n <= 0` → soft error: log and use rarity default.
    pub pool_copies_override: Option<i32>,
}

/// Immutable map of all card definitions. Built once at server startup.
///
/// Server inserts as `Res<CardCatalog>` after `LoadingState` completes.
/// Client builds an identical map for display lookup.
///
/// Invariant: never mutated after construction. Session teardown does not
/// affect this resource — it lives for the server's lifetime.
pub type CardCatalog = std::collections::HashMap<CardId, CardData>;

/// Rarity-based base copy count constants.
/// Epic and Legendary are intentionally consts, not `GameConfig` fields.
/// Their scarcity is a load-bearing design pillar (see card-data-pool.md
/// Player Fantasy section).
pub const EPIC_POOL_COPIES: u32 = 1;
pub const LEGENDARY_POOL_COPIES: u32 = 1;
```

**Family index (server-side only):** The server builds a supplementary
`FamilyIndex: HashMap<String, Vec<CardId>>` from the `CardCatalog` at startup
for O(1) neutral family draws. This index is derived data and not part of the
shared `CardData` schema.

### Part 2: PlayerPool — Mutable, Session-Scoped

**Location:** `server/src/core/pool/state.rs`

```rust
// server/src/core/pool/state.rs

use std::collections::HashMap;
use shared::card::{CardId, CardCatalog, Rarity, ClassId, EPIC_POOL_COPIES,
                   LEGENDARY_POOL_COPIES};
use crate::foundation::config::GameConfig;

/// Error returned when `distribute()` is called on an exhausted card.
#[derive(Debug, PartialEq, Eq)]
pub enum DistributeError {
    /// `copies_remaining` is already 0; cannot decrement further.
    Exhausted,
    /// `card_id` is not present in this player's pool.
    UnknownCard,
}

/// Filter applied to `draw_random()`. All fields `None` by default (no restriction).
#[derive(Default)]
pub struct PoolFilter {
    /// Restrict to one card type (e.g., Minion only).
    pub card_type: Option<shared::card::CardType>,
    /// Restrict to a specific class or Neutral.
    pub class: Option<ClassId>,
    /// Restrict to specific rarities.
    pub rarity: Option<Vec<Rarity>>,
    /// Maximum mana cost (inclusive).
    pub max_cost: Option<u32>,
}

/// Per-player pool state, scoped to one game session.
///
/// Initialized from `CardCatalog` + `GameConfig` at `SessionReady`.
/// Destroyed with the session on `GameOverEmitted`.
///
/// Invariant: `copies_remaining[id] + total_acquired[id] == initial_count[id]`
///   for all `id` in the pool. Verified by `distribute()` and enforced by
///   the pool initialization.
pub struct PlayerPool {
    /// Current available copies per card. Decremented by `distribute()`.
    /// Never below 0.
    pub copies_remaining: HashMap<CardId, u32>,

    /// Copy count at pool initialization. Never mutated after construction.
    /// Required to compute `total_acquired` for Formula 2 weighting without
    /// a separate tracking field:
    ///   `total_acquired(id) = initial_count[id] - copies_remaining[id]`
    pub initial_count: HashMap<CardId, u32>,

    /// Current shop display (3 slots). `None` = empty slot (pool exhausted).
    /// Populated by `refresh_shop()`. Never persisted across DRAFT phases.
    pub shop_slots: Vec<Option<CardId>>,
}

/// Authoritative collection of all per-player pools for a session.
///
/// Inserted as `Res<PlayerPools>` by the pool initialization system at
/// `SessionReady`. Removed by GSS on `GameOverEmitted`.
#[derive(bevy::prelude::Resource)]
pub struct PlayerPools {
    pub pools: HashMap<shared::session::PlayerId, PlayerPool>,
}

impl PlayerPool {
    /// Initialize a new pool from the catalog.
    ///
    /// Copy counts follow this precedence:
    ///   1. `pool_copies_override` if `Some(n)` and `n >= 1`
    ///   2. Rarity default from `GameConfig` (Common/Uncommon/Rare) or const
    ///      (Epic/Legendary)
    ///   3. Soft error: `pool_copies_override <= 0` → log error, use rarity
    ///      default, do NOT abort.
    pub fn initialize(catalog: &CardCatalog, config: &GameConfig) -> Self {
        let mut copies_remaining = HashMap::new();
        let mut initial_count = HashMap::new();

        for (id, card) in catalog {
            let base = Self::rarity_copies(card.rarity, config);
            let copies = match card.pool_copies_override {
                Some(n) if n >= 1 => n as u32,
                Some(n) => {
                    // Soft error: log and fall back to rarity default.
                    // Server does not abort (card-data-pool.md Rule 1, Formula 1).
                    tracing::error!(
                        card_id = ?id,
                        override_value = n,
                        rarity_default = base,
                        "pool_copies_override <= 0 — using rarity default"
                    );
                    base
                }
                None => base,
            };
            copies_remaining.insert(*id, copies);
            initial_count.insert(*id, copies);
        }

        Self {
            copies_remaining,
            initial_count,
            shop_slots: vec![None; 3],
        }
    }

    /// Returns the rarity-default copy count from `GameConfig` or const.
    fn rarity_copies(rarity: Rarity, config: &GameConfig) -> u32 {
        match rarity {
            Rarity::Common    => config.common_pool_copies,
            Rarity::Uncommon  => config.uncommon_pool_copies,
            Rarity::Rare      => config.rare_pool_copies,
            Rarity::Epic      => EPIC_POOL_COPIES,
            Rarity::Legendary => LEGENDARY_POOL_COPIES,
        }
    }

    // ─── Read-only queries ────────────────────────────────────────────────

    /// O(1) availability check.
    pub fn is_available(&self, card_id: CardId) -> bool {
        self.copies_remaining.get(&card_id).copied().unwrap_or(0) > 0
    }

    /// Current copy count for UI display (`copies_remaining` indicator).
    pub fn copies_remaining(&self, card_id: CardId) -> u32 {
        self.copies_remaining.get(&card_id).copied().unwrap_or(0)
    }

    /// How many copies have been distributed (purchased) since session start.
    ///
    /// Derived: `initial_count - copies_remaining`. Does not require a
    /// separate field.
    pub fn total_acquired(&self, card_id: CardId) -> u32 {
        let initial = self.initial_count.get(&card_id).copied().unwrap_or(0);
        let remaining = self.copies_remaining.get(&card_id).copied().unwrap_or(0);
        initial.saturating_sub(remaining)
    }

    // ─── Draw functions ───────────────────────────────────────────────────

    /// Draw 9 distinct card IDs for the initial draft offering.
    ///
    /// Eligible set: union of `class` cards + all Neutral cards.
    /// Any rarity is eligible (including Epic and Legendary).
    /// Drawn without replacement — no duplicate IDs in the returned Vec.
    /// `distribute()` is NOT called — undrafted cards remain available.
    ///
    /// Callers must call `distribute()` only for cards the player purchases.
    pub fn draw_initial_draft(
        &self,
        catalog: &CardCatalog,
        class: ClassId,
        count: u8,
        seed: u64,
    ) -> Vec<CardId> {
        // Collect eligible card IDs (class + Neutral).
        // Eligibility is catalog-based, not pool-copy-based, for initial draft.
        let eligible: Vec<CardId> = catalog
            .iter()
            .filter(|(_, card)| card.class == class || card.class == ClassId::Neutral)
            .map(|(id, _)| *id)
            .collect();

        // Uniform draw without replacement using Fisher-Yates on a seeded ChaCha RNG.
        // Uses the caller-supplied seed — no internal RNG state.
        use rand::SeedableRng;
        use rand::seq::SliceRandom;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let mut shuffled = eligible;
        shuffled.shuffle(&mut rng);
        shuffled.into_iter().take(count as usize).collect()
    }

    /// Weighted draw for a class shop slot (Phase 2 of shop slot generation).
    ///
    /// Implements Formula 2 from card-data-pool.md.
    /// Eligible: cards where `class == player_class` and `copies_remaining > 0`.
    /// Returns `None` only when all eligible class cards are exhausted.
    ///
    /// Phase 1 (50/50 slot type split) and fallback logic (class exhausted →
    /// retry as neutral) are the calling system's responsibility
    /// (Card Acquisition). The pool does not perform fallback internally.
    pub fn draw_class_card(
        &self,
        catalog: &CardCatalog,
        class: ClassId,
        seed: u64,
        config: &GameConfig,
    ) -> Option<CardId> {
        let eligible: Vec<CardId> = self.copies_remaining
            .iter()
            .filter(|(id, &count)| {
                count > 0 && catalog.get(id).map(|c| c.class == class).unwrap_or(false)
            })
            .map(|(id, _)| *id)
            .collect();

        if eligible.is_empty() {
            return None;
        }

        self.weighted_cdf_draw(&eligible, seed, config)
    }

    /// Weighted draw for a neutral family (Phase 2 for neutral slots).
    ///
    /// Eligible: neutral families where at least one card has `copies_remaining > 0`.
    /// Weight is computed per-family (sum of `total_acquired` across all family cards).
    /// Returns `None` only when all neutral families are fully exhausted.
    pub fn draw_neutral_family(
        &self,
        catalog: &CardCatalog,
        family_index: &std::collections::HashMap<String, Vec<CardId>>,
        seed: u64,
        config: &GameConfig,
    ) -> Option<String> {
        // Collect eligible families (at least one available card).
        let eligible_families: Vec<&String> = family_index
            .keys()
            .filter(|family| {
                family_index[*family]
                    .iter()
                    .any(|id| self.is_available(*id))
            })
            .collect();

        if eligible_families.is_empty() {
            return None;
        }

        // Compute per-family total_acquired (Formula 2 — neutral slot variant).
        let family_acquired: Vec<u32> = eligible_families
            .iter()
            .map(|family| {
                family_index[*family]
                    .iter()
                    .map(|id| self.total_acquired(*id))
                    .sum()
            })
            .collect();

        let selected_idx = Self::weighted_cdf_select(
            &family_acquired,
            eligible_families.len(),
            seed,
            config,
        )?;

        Some(eligible_families[selected_idx].clone())
    }

    /// Uniform draw of one card within a family (Phase 3 for neutral slots).
    ///
    /// Precondition: `draw_neutral_family` returned `Some(family)` — the
    /// family is guaranteed to have at least one available card.
    /// Returns `None` only if the family is fully exhausted (should not occur
    /// in normal flow when called after `draw_neutral_family` returned `Some`).
    pub fn draw_family_card(
        &self,
        family: &str,
        family_index: &std::collections::HashMap<String, Vec<CardId>>,
        seed: u64,
    ) -> Option<CardId> {
        let available: Vec<CardId> = family_index
            .get(family)?
            .iter()
            .filter(|id| self.is_available(**id))
            .copied()
            .collect();

        if available.is_empty() {
            return None;
        }

        // Uniform selection (Phase 3 uses no weighting).
        use rand::SeedableRng;
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let idx = rng.gen_range(0..available.len());
        Some(available[idx])
    }

    /// Draw from the shared neutral auction pool.
    ///
    /// Eligible: Neutral cards with `rarity ∈ {Rare, Legendary}` and
    /// `copies_remaining > 0`. Epic cards are excluded (class-specific;
    /// no Neutral Epics exist in the catalog).
    ///
    /// Note: The shared auction pool is architecturally distinct from the
    /// per-player shop pool. Full management of the shared pool (initialization,
    /// depletion tracking, multi-player coordination) is the Auction System
    /// GDD's responsibility. This function operates on the shared pool passed
    /// by the Auction System.
    pub fn draw_auction_card(
        auction_pool: &PlayerPool,
        catalog: &CardCatalog,
        seed: u64,
    ) -> Option<CardId> {
        use rand::SeedableRng;
        use rand::seq::SliceRandom;
        let eligible: Vec<CardId> = auction_pool.copies_remaining
            .iter()
            .filter(|(id, &count)| {
                count > 0 && catalog.get(id).map(|c| {
                    c.class == ClassId::Neutral
                        && matches!(c.rarity, Rarity::Rare | Rarity::Legendary)
                }).unwrap_or(false)
            })
            .map(|(id, _)| *id)
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        eligible.choose(&mut rng).copied()
    }

    /// Uniform draw over cards matching a filter.
    ///
    /// Used by draw effects and Prism Lane 3. The caller is responsible for
    /// calling `distribute()` if the draw effect consumes the card.
    /// Returns `None` when no eligible cards match the filter.
    ///
    /// This function does NOT call `distribute()` internally — callers own
    /// the consumption decision.
    pub fn draw_random(
        &self,
        catalog: &CardCatalog,
        filter: &PoolFilter,
        seed: u64,
    ) -> Option<CardId> {
        use rand::SeedableRng;
        use rand::seq::SliceRandom;
        let eligible: Vec<CardId> = self.copies_remaining
            .iter()
            .filter(|(id, &count)| {
                count > 0 && Self::matches_filter(id, catalog, filter)
            })
            .map(|(id, _)| *id)
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        eligible.choose(&mut rng).copied()
    }

    // ─── Mutation ─────────────────────────────────────────────────────────

    /// Decrement `copies_remaining` by 1.
    ///
    /// The sole pool mutation. Called on card purchase (shop), auction win,
    /// or draw effect consumption.
    ///
    /// Returns `Err(DistributeError::Exhausted)` if `copies_remaining == 0`.
    /// `copies_remaining` is never decremented below 0.
    ///
    /// Callers should check `is_available()` before calling to avoid the error
    /// path in hot-path code.
    pub fn distribute(&mut self, card_id: CardId) -> Result<(), DistributeError> {
        match self.copies_remaining.get_mut(&card_id) {
            None => Err(DistributeError::UnknownCard),
            Some(count) if *count == 0 => Err(DistributeError::Exhausted),
            Some(count) => {
                *count -= 1;
                Ok(())
            }
        }
    }

    // ─── Private helpers ──────────────────────────────────────────────────

    /// CDF-based weighted selection implementing Formula 2 (card-data-pool.md).
    ///
    /// ```text
    /// raw_weight(t) = (1 / |eligible|) + SHOP_WEIGHT_PER_CARD_OWNED × total_acquired(t)
    /// raw_weight(t) = clamp(raw_weight(t), 0.0, SHOP_WEIGHT_CAP)
    /// normalized_weight(t) = raw_weight(t) / Σ raw_weight(t')
    /// ```
    ///
    /// Selection: build CDF from normalized weights, draw uniform [0, 1) from
    /// seeded RNG, find first bucket where cumulative sum >= draw value.
    ///
    /// Precondition: `eligible` is non-empty (guaranteed by callers).
    /// Returns `None` only if normalization produces a zero-sum (impossible with
    /// valid `SHOP_WEIGHT_CAP > 0` and non-empty eligible set).
    fn weighted_cdf_draw(
        &self,
        eligible: &[CardId],
        seed: u64,
        config: &GameConfig,
    ) -> Option<CardId> {
        let per_card_weight = config.shop_weight_per_card;
        let weight_cap = config.shop_weight_cap;
        let base_weight = 1.0_f32 / eligible.len() as f32;

        // Compute raw weights with ownership bonus, clamped to cap.
        let raw_weights: Vec<f32> = eligible
            .iter()
            .map(|id| {
                let bonus = per_card_weight * self.total_acquired(*id) as f32;
                (base_weight + bonus).min(weight_cap)
            })
            .collect();

        let total: f32 = raw_weights.iter().sum();
        if total <= 0.0 {
            return None; // Should never occur with valid config
        }

        // Build CDF and select via uniform draw.
        use rand::SeedableRng;
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let draw: f32 = rng.gen_range(0.0_f32..1.0_f32);
        let mut cumulative = 0.0_f32;

        for (i, &w) in raw_weights.iter().enumerate() {
            cumulative += w / total;
            if draw <= cumulative {
                return Some(eligible[i]);
            }
        }

        // Floating point edge: draw == 1.0 or rounding — return last element.
        eligible.last().copied()
    }

    /// CDF selection over a Vec of integer acquired-counts (neutral family variant).
    /// Returns the selected index or None on empty input.
    fn weighted_cdf_select(
        acquired_counts: &[u32],
        count: usize,
        seed: u64,
        config: &GameConfig,
    ) -> Option<usize> {
        let weight_cap = config.shop_weight_cap;
        let per_card_weight = config.shop_weight_per_card;
        let base_weight = 1.0_f32 / count as f32;

        let raw_weights: Vec<f32> = acquired_counts
            .iter()
            .map(|&acq| {
                let bonus = per_card_weight * acq as f32;
                (base_weight + bonus).min(weight_cap)
            })
            .collect();

        let total: f32 = raw_weights.iter().sum();
        if total <= 0.0 {
            return None;
        }

        use rand::SeedableRng;
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let draw: f32 = rng.gen_range(0.0_f32..1.0_f32);
        let mut cumulative = 0.0_f32;

        for (i, &w) in raw_weights.iter().enumerate() {
            cumulative += w / total;
            if draw <= cumulative {
                return Some(i);
            }
        }

        Some(raw_weights.len() - 1)
    }

    /// Applies a `PoolFilter` to one card. Returns `true` if the card passes.
    fn matches_filter(id: &CardId, catalog: &CardCatalog, filter: &PoolFilter) -> bool {
        let Some(card) = catalog.get(id) else { return false; };

        if let Some(ct) = filter.card_type {
            if card.card_type != ct { return false; }
        }
        if let Some(class) = filter.class {
            if card.class != class { return false; }
        }
        if let Some(ref rarities) = filter.rarity {
            if !rarities.contains(&card.rarity) { return false; }
        }
        if let Some(max_cost) = filter.max_cost {
            if card.cost > max_cost { return false; }
        }
        true
    }
}
```

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  shared/src/card.rs  (no Bevy deps)                             │
│                                                                  │
│  CardId · CardData · Rarity · CardType · ClassId · Keyword      │
│  CardCatalog = HashMap<CardId, CardData>  (type alias)          │
│  EPIC_POOL_COPIES = 1  (const)                                  │
│  LEGENDARY_POOL_COPIES = 1  (const)                             │
└────────────────────┬────────────────────────────────────────────┘
                     │ imported by both
          ┌──────────┴───────────┐
          │                      │
┌─────────▼──────────┐  ┌────────▼──────────────────────────────┐
│  client/           │  │  server/                               │
│  (display only)    │  │                                        │
│  Builds CardCatalog│  │  foundation/: loads cards.json →       │
│  from cards.json   │  │    CardCatalog (Res<CardCatalog>)       │
│  for rendering     │  │    FamilyIndex (Res<FamilyIndex>)       │
│  No pool state     │  │                                        │
└────────────────────┘  │  core/pool/state.rs:                   │
                        │    PlayerPools (Res<PlayerPools>)       │
                        │      ├─ PlayerPool [Player A]           │
                        │      │    copies_remaining: HashMap     │
                        │      │    initial_count: HashMap        │
                        │      │    shop_slots: Vec<Option<CardId>>│
                        │      └─ PlayerPool [Player B]           │
                        │           (same structure)              │
                        │                                         │
                        │  Shared auction pool:                   │
                        │    AuctionPool (Res<AuctionPool>)        │
                        │    [Auction System GDD — M2]            │
                        └─────────────────────────────────────────┘
```

### Key Interfaces

**Catalog loading** (server startup, `LoadingState`):
```rust
// server/src/core/pool/loader.rs
pub fn load_card_catalog(path: &str) -> Result<CardCatalog, CatalogLoadError>;

pub enum CatalogLoadError {
    FileNotFound { path: String },
    ParseError   { path: String, details: String },
    DuplicateIds { ids: Vec<CardId> },
    MissingRarity{ card_id: CardId },
}
```

**Pool initialization** (at `SessionReady`):
```rust
PlayerPool::initialize(catalog: &CardCatalog, config: &GameConfig) -> PlayerPool
```

**Draw functions** (per-slot, per-round):
```rust
PlayerPool::draw_initial_draft(catalog, class, count=9, seed) -> Vec<CardId>
PlayerPool::draw_class_card(catalog, class, seed, config)     -> Option<CardId>
PlayerPool::draw_neutral_family(catalog, family_index, seed, config) -> Option<String>
PlayerPool::draw_family_card(family, family_index, seed)      -> Option<CardId>
PlayerPool::draw_auction_card(auction_pool, catalog, seed)    -> Option<CardId>
PlayerPool::draw_random(catalog, filter, seed)                -> Option<CardId>
```

**Sole mutation**:
```rust
PlayerPool::distribute(card_id) -> Result<(), DistributeError>
```

**Read-only queries**:
```rust
PlayerPool::is_available(card_id)     -> bool         // O(1)
PlayerPool::copies_remaining(card_id) -> u32
PlayerPool::total_acquired(card_id)   -> u32          // derived; no extra field
```

---

## Alternatives Considered

### Alternative 1: Single Unified Struct (CardEntry with mutable copy count)

**Description:** One struct per card combining `CardData` with a mutable
`copies_remaining` field. A single `HashMap<CardId, CardEntry>` per player.

**Pros:** Simpler type surface; one lookup gives both definition and count.

**Cons:** Catalog cannot be shared immutably between sessions. Every session
requires a deep clone of all card definitions. Client cannot hold the same
catalog type without copying pool state it has no use for. Mutating counts
pollutes the definition hashmap with write contention during concurrent session
reads. Tests must construct full card definitions to test copy-count logic.

**Rejection Reason:** Immutable definitions and mutable pool state have
different lifetimes, different owners, and different readers. Merging them
violates Single Responsibility and makes CardCatalog impossible to share
cheaply across sessions.

### Alternative 2: Full ECS Components for Pool State

**Description:** Represent each "card slot" in a player's pool as a Bevy
entity with components: `CardDefinition`, `CopiesRemaining`, `PlayerOwner`.

**Pros:** Native ECS queries for filtering by rarity, class, or availability.
Lightyear could potentially replicate `CopiesRemaining` components directly.

**Cons:** ~315 cards × 2 players = 630 entities per session spawned at
`SessionReady` and despawned at `GameOverEmitted`. Query overhead and
structural changes during card exhaustion would fire archetype migrations.
Replication of `CopiesRemaining` via Lightyear would broadcast opponent pool
state to both clients unless component-level replication scope is available —
which ADR-001 established is not supported in Lightyear 0.26 at entity
granularity. This pattern is architecturally fragile for this data shape.

**Rejection Reason:** Pool state is a plain data structure, not a game entity.
The ECS is the right home for game-world objects (units, objectives). Using it
for a data table creates unnecessary entity lifecycle overhead and replication
complexity. A `Resource`-based `HashMap` is the idiomatic Bevy pattern for
session-scoped non-spatial data.

### Alternative 3: Shared Global Pool (All Players Draw from One Pool)

**Description:** One pool for all players, as in Teamfight Tactics. Card
purchases by one player reduce availability for all.

**Pros:** Emergent rivalry — taking a card denies it from opponents.

**Cons:** Contradicts the GDD Player Fantasy: "The pool is finite and personal
— each player's copy counts are theirs alone, unaffected by what others buy."
The design specifically rejects the TFT model. This alternative is a design
conflict, not a technical trade-off.

**Rejection Reason:** Explicitly rejected by `card-data-pool.md` Rule 2
("This game does NOT use a shared global pool").

---

## Consequences

### Positive

- `CardCatalog` is zero-cost to share across sessions — it is `Res<CardCatalog>`,
  immutable, and lives for the server's lifetime. No per-session deep clone.
- `total_acquired` is derived from fields already held (`initial_count -
  copies_remaining`) — no additional tracking field needed.
- `distribute()` is the single mutation point. All copy-count changes flow
  through one function, which is trivially unit-testable.
- Draw functions return `Option<T>` — callers never panic on exhausted pools.
- Client holds the same `CardCatalog` type as the server for display lookup
  but has no pool state type dependency.
- Epic and Legendary copy counts are compile-time consts — they cannot be
  accidentally placed in `GameConfig` by a future contributor.

### Negative

- Two separate data structures (`CardCatalog` and `PlayerPool`) must be kept
  in sync on lookup: every `draw_*` function takes both as parameters.
  Contributors must remember to pass the catalog alongside the pool.
- `FamilyIndex` is a derived server-only structure not in `shared/`. Any system
  that needs family-level queries must depend on the server-side index rather
  than deriving it from `CardCatalog` directly.
- The `CardData` struct uses `Option<String>` for `family` — the absence of a
  typed `FamilyId` newtype means family string comparisons are used in the
  family index. This is acceptable at ~50 neutral families but is a mild smell
  if the family list grows significantly.

### Risks

- **`bevy_asset_loader` version compatibility with Bevy 0.18:** If the correct
  version is not available on crates.io, the `LoadingState`-based catalog load
  must be implemented manually. Mitigation: verify before sprint start
  (game-config.md OQ1).
- **`pool_copies_override` soft-error ambiguity:** A card with `override = -1`
  silently falls back to the rarity default. Content authoring tools must
  validate this field before submitting `cards.json`. Without tooling, bad
  overrides produce silent behavior drift. Mitigation: implement a
  `validate_card_catalog()` linter as part of the content pipeline.
- **Shared auction pool architecture:** `draw_auction_card()` is stubbed to
  receive an `auction_pool: &PlayerPool` parameter. The Auction System GDD must
  specify initialization, depletion handling, and multi-player behavior of this
  pool. Until that ADR is written, the auction draw interface is provisional.
- **Float precision in Formula 2 normalization:** `Σ normalized_weight(t)` must
  equal 1.0 within ±1e-6 (CP-NW). The CDF draw adds floating-point rounding;
  the `eligible.last().copied()` fallback handles the edge case where `draw ==
  1.0`. This is tested explicitly by CP-NW.

---

## Performance Implications

- **CPU:** `CardCatalog` lookup is `HashMap::get` — O(1). `PlayerPool::draw_*`
  iterates the eligible subset (up to ~315 entries at initialization, shrinking
  as copies deplete). Draw runs once per shop slot per DRAFT phase — three calls
  per round at most. No hot-path concern.
- **Memory:** `CardCatalog` holds ~315 `CardData` structs with `String` fields
  for name, effect_text, art_id, family. Estimated ~50–100 KB per catalog. Two
  `PlayerPool` structs (per-player `HashMap<CardId, u32>` × 2 maps × 315 entries)
  add ~20 KB per session. Negligible against the 256 MB WASM heap budget.
- **Load Time:** Parsing `cards.json` (~315 cards, estimated ~200 KB) with
  `serde_json` is a one-time startup cost, expected under 10ms on server-class
  hardware. Not on the critical path for WASM client load time.
- **Network:** `S2CPoolUpdate` sends only changed `copies_remaining` deltas per
  round (typically 1–3 entries). `S2CPoolSnapshot` on reconnect sends all 315
  entries — estimated ~3 KB per player.

---

## Migration Plan

This is a greenfield system — no existing pool code to migrate. First
implementation follows this sequence:

1. Write `shared/src/card.rs` with all type definitions.
2. Write `cards.json` fixture with representative card data for tests.
3. Write `server/src/core/pool/loader.rs` with `load_card_catalog()` and all
   hard validation (duplicate IDs, missing rarity, `SHOP_WEIGHT_CAP <= 0`).
4. Write `server/src/core/pool/state.rs` with `PlayerPool::initialize()`,
   read-only queries, and `distribute()`.
5. Write unit tests covering all `card-data-pool.md` BLOCKING acceptance criteria
   (CP1a through CP-C3) in `tests/unit/pool/`.
6. Write `server/src/core/pool/draw.rs` implementing the weighted draw functions.
7. Write unit tests for Formula 2 weighting (CP9, CP10) and draw behavior
   (CP7, CP7b, CP8a, CP-IC, CP-SHC, CP-SHN, CP-NW).
8. Integrate with `bevy_asset_loader` `LoadingState` — verify `TypePath` derive.
9. Integration test: pool initialization → shop draw → distribute flow
   (CP1b, CP4, CP5, CP5b, CP5c).

---

## Validation Criteria

All BLOCKING acceptance criteria in `design/gdd/card-data-pool.md` sections
"Pool Loading," "Pool Queries & Distribution," and "Shop Weighting" must pass:

| Test ID | What it proves |
|---------|---------------|
| CP1a | Catalog length matches fixture |
| CP1b | Every card has ≥1 copy at init |
| CP2 | Soft error on `override <= 0` — log, continue, use rarity default |
| CP3a–d | Hard errors abort startup: missing file, bad JSON, duplicate IDs, unknown rarity |
| CP4 | Rare with no override → 4 copies |
| CP5 | `distribute()` decrements correctly |
| CP5c | `distribute()` returns `Err(Exhausted)` at 0 copies |
| CP5b | Positive `pool_copies_override` overrides rarity default |
| CP6a–b | `is_available()` correct at 0 and >0 |
| CP7 | `draw_class_card` returns None when all class cards exhausted |
| CP8a | Both subsets exhausted → `draw_shop_slot()` returns None |
| CP-IC | `initial_count` immutable; `total_acquired` computable |
| CP-SHC | Class draw returns a card of the correct class |
| CP-SHN | Neutral draw returns a card of the correct family and class |
| CP-NW | Σ `normalized_weight(t)` == 1.0 ± 1e-6 |
| CP-A | Auction draw returns None when auction pool exhausted |
| CP-B | `draw_random` with exhausted filter returns None |
| CP-C | Initial draft returns 9 distinct IDs |
| CP-C2 | Draft returns only class + Neutral cards |
| CP-C3 | `distribute()` not called for unselected draft cards |
| CP9 | Formula 2 raw_weight and normalized_weight at 3 owned copies |
| CP10 | Weight clamped at `SHOP_WEIGHT_CAP` at 7 owned copies |

---

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|---|---|---|
| `card-data-pool.md` Rule 1 | Load from `assets/data/cards.json`; hard validation; fatal on duplicate IDs or missing rarity | `load_card_catalog()` with typed `CatalogLoadError` variants; all validation cases covered |
| `card-data-pool.md` Rule 2 | Per-player independent pool; NOT shared global pool | `PlayerPools: HashMap<PlayerId, PlayerPool>` — each player has their own pool; no shared mutation |
| `card-data-pool.md` Rule 3 | Card definition schema with all fields | `CardData` struct matches all base and Minion-specific fields from the GDD table |
| `card-data-pool.md` Rule 4 | All draw operations including `draw_initial_draft`, `draw_class_card`, `draw_neutral_family`, `draw_family_card`, `draw_auction_card`, `draw_random`, `distribute` | All functions present; signatures match GDD interface table |
| `card-data-pool.md` Rule 5 | Depletion handling — return `None`, never panic | All draw functions return `Option<T>`; `distribute()` returns `Result` |
| `card-data-pool.md` Formula 1 | `pool_copies_override` precedence; soft error on ≤0 | `PlayerPool::initialize()` implements the three-branch precedence formula |
| `card-data-pool.md` Formula 2 | Weighted draw: base weight + ownership bonus, clamped, normalized | `weighted_cdf_draw()` and `weighted_cdf_select()` implement Formula 2; `total_acquired` derived from `initial_count - copies_remaining` |
| `card-data-pool.md` Tuning Knobs | `common_pool_copies`, `uncommon_pool_copies`, `rare_pool_copies`, `shop_weight_per_card`, `shop_weight_cap` | `rarity_copies()` reads from `GameConfig`; Epic/Legendary use consts |
| `game-config.md` Rule 2 | `GameConfig` struct fields for pool counts and shop weights | `PlayerPool::initialize()` and `weighted_cdf_draw()` read `Res<GameConfig>` — no hardcoded values |
| `game-config.md` Rule 5 | `SHOP_WEIGHT_CAP > 0` is a startup validation invariant | Pool loader validates `config.shop_weight_cap > 0.0` before initializing any pool |
| `design/gdd/card-data-pool.md` TR-CDP-01–09 | All tagged technical requirements | All nine TRs addressed: catalog loading, per-player pool, copy counts, weighted draw, depletion handling, distribution visibility, shared auction draw separation, initial draft, pool queries |

---

## Related

- `docs/architecture/adr-001-objective-identity-unicast.md` — Unicast pattern for secret data; objective pool state uses the same server-authoritative, never-replicated approach for `HiddenObjectives`
- `design/gdd/card-data-pool.md` — Authoritative GDD; Formulas F1–F3, Rules 1–8, all Acceptance Criteria
- `design/gdd/game-config.md` — `GameConfig` struct; pool copy count fields and shop weight fields
- `docs/architecture/architecture.md` — Boundary 5 (Card Pool public API); Core layer module ownership table
- ADR-003: Cargo Workspace Structure — Accepted (2026-04-29) — workspace crate layout for `shared/src/card.rs` is implementation-ready
- ADR-004: Asset Loading Pipeline — Accepted (2026-04-29) — `LoadingState`-based `cards.json` load is implementation-ready
- Pending: Auction System GDD (M2) — must define shared auction pool management before `draw_auction_card` integration is complete
