# Story 001: State & API Scaffold

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: TR-CDP-01 through TR-CDP-05, TR-CDP-08 (pool initialization, sole-mutation discipline, read-only queries, `total_acquired` derived field, hard catalog validation)

**ADR Governing Implementation**: ADR-006: Card Data Schema and Pool State Architecture
**ADR Decision Summary**: `CardCatalog` is immutable, server-lifetime, loaded from `cards.json`. `PlayerPool { copies_remaining: HashMap<CardId, u32>, initial_count: HashMap<CardId, u32> }` is mutable, session-scoped, held inside `PlayerPools(HashMap<PlayerId, PlayerPool>)` resource. `distribute()` is the SOLE mutation on `copies_remaining` — never decrements below 0. `total_acquired(id)` is derived as `initial_count[id] - copies_remaining[id]`; no separate field. `pool_copies_override <= 0` is a soft error (log + use rarity default + continue). `load_card_catalog()` performs hard validation — duplicate IDs, missing rarity, `SHOP_WEIGHT_CAP <= 0` all abort startup.

**Engine**: Bevy 0.18 | **Risk**: LOW
**Engine Notes**: `state.rs` and the catalog loader are pure Rust — no Bevy ECS surface except `#[derive(Resource)]` on `PlayerPools`, `ShopSlots`, `InitialDraftOffering`, and `ManualRefreshCount`. No post-cutoff API risk on plain data types. `liv-bevy-018` is still mandatory on any file using `#[derive(Resource)]` to ensure the correct `bevy::prelude::Resource` import path.

**Control Manifest Rules (Core layer)**:
- Required: `PlayerPool` fields `copies_remaining` and `initial_count` are `HashMap<CardId, u32>`. `PlayerPools` is the only `Resource` wrapping pool state in this story.
- Required: `distribute()` in `pool/api.rs` is the SOLE site that mutates `copies_remaining`. Direct `HashMap` writes outside `pool/api.rs` are forbidden and enforced by CI grep gate.
- Required: `total_acquired(id)` is computed as `initial_count[id].saturating_sub(copies_remaining[id])` — no separate stored counter.
- Required: `pool_copies_override <= 0` logs a warning and uses the rarity default; server NEVER aborts for a soft error.
- Required: `load_card_catalog()` uses typed `CatalogLoadError` variants — no bare `unwrap()` in load path.
- Forbidden: No `unwrap()` in `distribute()`, `is_available()`, `copies_remaining()`, or `total_acquired()` — use `.copied().unwrap_or(0)` pattern.
- Forbidden: `EPIC_POOL_COPIES` and `LEGENDARY_POOL_COPIES` must not be read from `GameConfig` — they are compile-time consts in `shared/src/card.rs`.

---

## Acceptance Criteria

- [ ] `server/src/core/pool/state.rs` exists and defines:
  - `PlayerPool { copies_remaining: HashMap<CardId, u32>, initial_count: HashMap<CardId, u32> }` with `#[derive(Clone, Debug)]`
  - `PlayerPools(pub HashMap<PlayerId, PlayerPool>)` with `#[derive(Resource)]`
  - `ShopSlots(pub HashMap<PlayerId, Vec<CardId>>)` with `#[derive(Resource)]`
  - `InitialDraftOffering(pub HashMap<PlayerId, Vec<CardId>>)` with `#[derive(Resource)]`
  - `ManualRefreshCount(pub HashMap<PlayerId, u32>)` with `#[derive(Resource)]`
  - `PoolError` enum with variants: `CardNotInCatalog`, `CopiesExhausted`, `EmptyFilteredPool`
  - `DistributeError` enum with variants: `Exhausted`, `UnknownCard`
  - `PoolFilter` struct: `class: Option<ClassId>`, `family: Option<FamilyId>`, `max_rarity: Option<Rarity>` — derives `Default`
- [ ] `server/src/core/pool/api.rs` exists and exports:
  - `distribute(pool: &mut PlayerPool, card_id: CardId) -> Result<(), DistributeError>` — sole mutation; returns `Err(Exhausted)` when `copies_remaining == 0`; returns `Err(UnknownCard)` when card absent from pool
  - `is_available(pool: &PlayerPool, card_id: CardId) -> bool` — O(1); returns false for absent cards
  - `copies_remaining(pool: &PlayerPool, card_id: CardId) -> u32` — returns 0 for absent cards
  - `total_acquired(pool: &PlayerPool, card_id: CardId) -> u32` — derived; no stored field
  - `acquire_card(pool: &mut PlayerPool, card_id: CardId) -> Result<(), PoolError>` — delegates to `distribute()`; semantic alias for the shop-purchase path
- [ ] `server/src/core/pool/loader.rs` exists and exports:
  - `load_card_catalog(path: &str) -> Result<CardCatalog, CatalogLoadError>`
  - `CatalogLoadError` enum with variants: `FileNotFound { path: String }`, `ParseError { path: String, details: String }`, `DuplicateIds { ids: Vec<CardId> }`, `MissingRarity { card_id: CardId }`
  - Hard validation at load time: duplicate IDs → `DuplicateIds`, unrecognized/missing rarity → `MissingRarity`, `SHOP_WEIGHT_CAP <= 0` in loaded `GameConfig` → logged fatal error and `Err`
- [ ] `PlayerPool::initialize(catalog: &CardCatalog, config: &GameConfig) -> PlayerPool` applies rarity-default copy counts with `pool_copies_override` precedence:
  - `Some(n)` where `n >= 1` → use `n`
  - `Some(n)` where `n <= 0` → log warning, use rarity default, continue (NEVER abort)
  - `None` → use rarity default
  - Epic and Legendary copy counts read from `EPIC_POOL_COPIES` and `LEGENDARY_POOL_COPIES` consts, not from `GameConfig`
- [ ] **CP1a**: GIVEN a fixture `cards.json` with N entries, WHEN `load_card_catalog()` succeeds, THEN `catalog.len() == N`
- [ ] **CP1b**: GIVEN a valid catalog and `GameConfig`, WHEN `PlayerPool::initialize()` runs, THEN `copies_remaining[id] >= 1` for every `id` in the catalog
- [ ] **CP2**: GIVEN a card definition with `pool_copies_override = Some(-1)`, WHEN `PlayerPool::initialize()` processes it, THEN `copies_remaining[id]` equals the rarity default, AND a warning is logged, AND no panic occurs
- [ ] **CP3a**: GIVEN a `cards.json` that does not exist at the given path, WHEN `load_card_catalog()` is called, THEN returns `Err(CatalogLoadError::FileNotFound)`
- [ ] **CP3b**: GIVEN a `cards.json` with invalid JSON syntax, WHEN `load_card_catalog()` is called, THEN returns `Err(CatalogLoadError::ParseError)`
- [ ] **CP3c**: GIVEN a `cards.json` where two entries share the same `id`, WHEN `load_card_catalog()` is called, THEN returns `Err(CatalogLoadError::DuplicateIds)` listing all conflicting IDs
- [ ] **CP3d**: GIVEN a `cards.json` where one card has an unrecognized `rarity` string, WHEN `load_card_catalog()` is called, THEN returns `Err(CatalogLoadError::MissingRarity)` with the offending card ID
- [ ] **CP4**: GIVEN a Rare card with no `pool_copies_override`, WHEN `PlayerPool::initialize()` runs with `GameConfig.rare_pool_copies = 4`, THEN `copies_remaining[id] == 4`
- [ ] **CP5**: GIVEN `copies_remaining[id] = 2`, WHEN `distribute(pool, id)` called once, THEN `copies_remaining[id] == 1`, returns `Ok(())`
- [ ] **CP5b**: GIVEN a card with `pool_copies_override = Some(7)`, WHEN `PlayerPool::initialize()` runs, THEN `copies_remaining[id] == 7`
- [ ] **CP5c**: GIVEN `copies_remaining[id] = 0`, WHEN `distribute(pool, id)` called, THEN returns `Err(DistributeError::Exhausted)`, `copies_remaining[id]` remains 0 (not decremented below 0)
- [ ] **CP6a**: GIVEN `copies_remaining[id] = 0`, WHEN `is_available(pool, id)` called, THEN returns `false`
- [ ] **CP6b**: GIVEN `copies_remaining[id] >= 1`, WHEN `is_available(pool, id)` called, THEN returns `true`
- [ ] **CP-IC**: GIVEN `initial_count[id] = 4` and two successful `distribute()` calls, WHEN `total_acquired(pool, id)` called, THEN returns `2`; `initial_count[id]` remains `4` (immutable after init)
- [ ] CI grep gate: `grep -rE "copies_remaining\.(insert|remove|entry)" server/src/ | grep -v "core/pool/"` returns zero matches
- [ ] CI grep gate: `grep -rE "panic!|unwrap\(\)" server/src/core/pool/api.rs` returns zero matches
- [ ] `cargo check -p server` passes after adding `state.rs`, `api.rs`, and `loader.rs`

---

## Implementation Notes

*Derived from EPIC.md §Deliverables, ADR-006 §Decision, and `card-data-pool.md` Rules 1–4:*

**Resource layout:**
```rust
// server/src/core/pool/state.rs
use bevy::prelude::Resource;
use std::collections::HashMap;
use shared::card::{CardId, CardCatalog, Rarity, ClassId, EPIC_POOL_COPIES, LEGENDARY_POOL_COPIES};

#[derive(Clone, Debug)]
pub struct PlayerPool {
    pub copies_remaining: HashMap<CardId, u32>,
    pub initial_count: HashMap<CardId, u32>,
}

#[derive(Resource)]
pub struct PlayerPools(pub HashMap<PlayerId, PlayerPool>);

#[derive(Resource)]
pub struct ShopSlots(pub HashMap<PlayerId, Vec<CardId>>);

#[derive(Resource)]
pub struct InitialDraftOffering(pub HashMap<PlayerId, Vec<CardId>>);

#[derive(Resource)]
pub struct ManualRefreshCount(pub HashMap<PlayerId, u32>);
```

**`distribute()` implementation discipline:** Never use `*count -= 1` without the exhaustion guard. The function must check `copies_remaining == 0` first and return `Err` before any decrement. This is the critical invariant the CI grep gate enforces.

**`total_acquired` formula:** `initial_count[id].saturating_sub(copies_remaining[id])`. Use `saturating_sub` to prevent any u32 underflow in case of a bug where `copies_remaining` somehow exceeds `initial_count`.

**`pool_copies_override` precedence (Formula 1 from GDD):**
```
copies = match card.pool_copies_override {
    Some(n) if n >= 1 => n as u32,
    Some(n)           => { warn!(...); rarity_default(card.rarity, config) }
    None              => rarity_default(card.rarity, config)
}
```

**Rarity defaults:** `Common=6, Uncommon=5, Rare=4, Epic=EPIC_POOL_COPIES=1, Legendary=LEGENDARY_POOL_COPIES=1`. `GameConfig` fields (`common_pool_copies`, `uncommon_pool_copies`, `rare_pool_copies`) provide the tunable values for Common/Uncommon/Rare. Epic and Legendary are intentional consts — see ADR-006 §Constraints.

**`PlayerId` type**: Import from `shared::protocol::PlayerId`. If not yet defined when this story lands, use `type PlayerId = u32` placeholder with `// TODO: import from shared/` comment.

**`FamilyId` type**: `PoolFilter.family` field — use `Option<String>` if a `FamilyId` newtype is not yet defined in `shared/`. Add a `// TODO: replace String with FamilyId newtype` comment.

**`load_card_catalog` validation order**: Check for duplicate IDs first (full pass), then missing rarity on each card. Return the first `Err` encountered. All duplicate IDs should be collected before returning (do not short-circuit on first duplicate).

---

## Out of Scope

- Story 002: `draw()` with weighted CDF logic (Formula 2)
- Story 003: `refresh_shop()`, `draw_initial_draft()`, `draw_auction_card()`
- Story 004: `on_session_ready_init` Observer and `on_shop_refresh_needed` subscriber
- Story 005: Manual refresh cost escalation and Economy integration
- Story 006: Network dispatch wiring
- `CardPoolPlugin` registration — authored in Story 004
- `FamilyIndex` server-side derived map — authored in Story 003
- `CardData`, `CardId`, `Rarity`, `ClassId` type definitions — `workspace-and-shared-types` Story 002

---

## QA Test Cases

- **CP1a/CP1b: Catalog load and pool init happy path**
  - Given: fixture `cards.json` with 5 cards (one per rarity), valid `GameConfig`
  - When: `load_card_catalog()` then `PlayerPool::initialize()`
  - Then: `catalog.len() == 5`; every card has `copies_remaining >= 1`

- **CP2: Soft error on invalid override**
  - Given: card definition with `pool_copies_override = Some(-1)`, rarity `Rare`, `GameConfig.rare_pool_copies = 4`
  - When: `PlayerPool::initialize()` runs
  - Then: `copies_remaining[id] == 4`, warning logged, no panic

- **CP5c: distribute() at zero copies returns error without underflow**
  - Given: `copies_remaining[id] = 0`
  - When: `distribute(pool, id)` × 2
  - Then: both calls return `Err(Exhausted)`; `copies_remaining[id] == 0` (saturated)

- **CP-IC: total_acquired is derived, initial_count is immutable**
  - Given: card initialized with `initial_count = 4`, `copies_remaining = 4`
  - When: `distribute()` × 3
  - Then: `total_acquired() == 3`, `initial_count[id] == 4` (unchanged)

- **CP3c: Duplicate ID detection collects all conflicts**
  - Given: `cards.json` with three cards sharing `id = 42`
  - When: `load_card_catalog()`
  - Then: Returns `Err(DuplicateIds { ids })` where `ids` contains `42` (reported once)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/pool/state_api_test.rs` — all acceptance criteria passing; covers CP1a, CP1b, CP2, CP3a–d, CP4, CP5, CP5b, CP5c, CP6a, CP6b, CP-IC
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `workspace-and-shared-types` Story 002 (provides `CardData`, `CardId`, `Rarity`, `ClassId`, `EPIC_POOL_COPIES`, `LEGENDARY_POOL_COPIES` in `shared/src/card.rs`)
- Depends on: `workspace-and-shared-types` Story 003 (provides `GameConfig` pod struct with pool copy count fields)
- Depends on: `workspace-and-shared-types` Story 001 (workspace scaffolding; `server` crate must exist)
- Unlocks: Story 002 (weighted draw — requires `PlayerPool`, `PoolFilter`, `distribute()`)
- Unlocks: Story 003 (shop refresh — requires all of Story 002's API)
- Unlocks: Story 004 (session init Observer — requires `PlayerPools`, `ShopSlots`, `InitialDraftOffering`, `ManualRefreshCount` resource types)
