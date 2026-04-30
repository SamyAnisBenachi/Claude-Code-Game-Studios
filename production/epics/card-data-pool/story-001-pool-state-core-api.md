# Story 001: Pool State + Core API

> **Epic**: Card Data & Pool
> **Status**: Complete
> **Layer**: Core
> **Type**: Logic
> **Estimate**: 4–6 hours
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirements**: `TR-CDP-04`, `TR-CDP-05`, `TR-CDP-08`
*(TR-IDs are informal — `docs/architecture/tr-registry.yaml` is unpopulated. Run `/architecture-review` to register them formally.)*

**ADR Governing Implementation**: [ADR-006: Card Data Schema and Pool State Architecture](../../../docs/architecture/adr-006-card-data-schema.md)
**ADR Decision Summary**: Card data is split into an immutable `CardCatalog` (server-lifetime) and a mutable `PlayerPool` (session-scoped, per-player). `distribute()` is the sole mutation. `total_acquired` is derived from `initial_count - copies_remaining` — no separate tracking field.

**Engine**: Bevy 0.18 + Rust stable | **Risk**: LOW (pure Rust data structures, no Bevy-version coupling)
**Engine Notes**: `PlayerPools` struct carries `#[derive(bevy::prelude::Resource)]` — Bevy 0.18 Required Components API requires this derive be present. No bundles used. `shared/` crate must NOT derive `Resource` — server wraps it.

**Control Manifest Rules (Core layer)**:
- Required: `CardCatalog` is server-lifetime, immutable `Res<CardCatalog>`. `PlayerPool` is session-scoped per player.
- Required: `distribute()` is the sole pool mutation function. `copies_remaining` never goes below 0.
- Required: `total_acquired(id)` is derived: `initial_count[id] - copies_remaining[id]`. No separate stored field.
- Required: All pool draw functions return `Option<T>`. Never panic on empty pool.
- Forbidden: Direct mutation of `copies_remaining` outside `pool/api.rs` (or the impl method). CI grep gate enforces this.
- Forbidden: `unwrap()` in production paths. Use `?` or `expect("diagnostic")`.
- Required: `pool_copies_override <= 0` is a soft error — log, use rarity default, continue. Never abort startup.

---

## Acceptance Criteria

*From GDD `design/gdd/card-data-pool.md` §8, scoped to this story:*

- [ ] **AC-1 (CP1a)**: GIVEN a `CardCatalog` built from N card definitions (fixture), WHEN `PlayerPool::initialize(catalog, config)` completes, THEN `copies_remaining.len() == N` and `initial_count.len() == N`.
- [ ] **AC-2 (CP1b)**: GIVEN a catalog with at least one card of each rarity, WHEN `initialize()` completes, THEN `copies_remaining(id) >= 1` for every card in the catalog.
- [ ] **AC-3 (CP2)**: GIVEN a catalog containing a card with `pool_copies_override: Some(0)` or `Some(-1)`, WHEN `PlayerPool::initialize()` runs, THEN (a) no panic/abort; (b) `copies_remaining(id)` equals the rarity default for that card; (c) a `tracing::error!` log entry is produced containing the card's ID.
- [ ] **AC-4 (CP4)**: GIVEN a Rare card with `pool_copies_override: None`, WHEN `initialize()` completes, THEN `copies_remaining(card_id) == 4`.
- [ ] **AC-5 (CP5)**: GIVEN `copies_remaining(card_id) == N` (N > 0), WHEN `distribute(card_id)` is called, THEN returns `Ok(())` and `copies_remaining(card_id) == N - 1`.
- [ ] **AC-6 (CP5b)**: GIVEN a Rare card with `pool_copies_override: Some(2)`, WHEN `initialize()` completes, THEN `copies_remaining(card_id) == 2` (override applied, not rarity default of 4).
- [ ] **AC-7 (CP5c)**: GIVEN `copies_remaining(card_id) == 0`, WHEN `distribute(card_id)` is called, THEN returns `Err(DistributeError::Exhausted)` and `copies_remaining` remains 0.
- [ ] **AC-8 (CP6a)**: GIVEN `copies_remaining(card_id) == 0`, WHEN `is_available(card_id)`, THEN returns `false`.
- [ ] **AC-9 (CP6b)**: GIVEN `copies_remaining(card_id) >= 1`, WHEN `is_available(card_id)`, THEN returns `true`.
- [ ] **AC-10 (CP-IC)**: GIVEN `initial_count(id) == N`, WHEN `distribute(id)` called K times (K < N), THEN `initial_count(id) == N` (unchanged), `copies_remaining(id) == N - K`, `total_acquired(id) == K`.

---

## Implementation Notes

*From ADR-006 Implementation Guidelines:*

**Module layout** (`server/src/core/pool/`):
- `state.rs` — data structures only: `PlayerPool`, `PlayerPools`, `DistributeError`, `PoolFilter`, copy-count constants
- `api.rs` — sole-mutation discipline: `distribute()`, `acquire_card()`, `total_acquired()`, `is_available()`, `copies_remaining()` as free functions OR `impl PlayerPool` methods (either layout is valid; pick one and stay consistent)
- `plugin.rs` — `CardPoolPlugin` (registers resources; Story 004 fills this out)

**`PlayerPool` struct**:
```rust
pub struct PlayerPool {
    pub copies_remaining: HashMap<CardId, u32>,  // mutated ONLY via distribute()
    pub initial_count:    HashMap<CardId, u32>,  // never mutated after construction
    pub shop_slots:       Vec<Option<CardId>>,   // 3-slot current shop display
}
```

**`PlayerPools` resource** (Bevy 0.18):
```rust
#[derive(bevy::prelude::Resource)]
pub struct PlayerPools {
    pub pools: HashMap<shared::session::PlayerId, PlayerPool>,
}
```

**`PlayerPool::initialize()` precedence** (Formula 1 from GDD):
1. `pool_copies_override: Some(n)` where `n >= 1` → use `n`
2. `pool_copies_override: Some(n)` where `n <= 0` → soft error: `tracing::error!` with card ID + override value; use rarity default
3. `pool_copies_override: None` → use rarity default from `GameConfig`
4. Rarity defaults: Common=6, Uncommon=5, Rare=4; Epic/Legendary use consts `EPIC_POOL_COPIES=1`, `LEGENDARY_POOL_COPIES=1`

**`distribute()` invariant**: Match on `copies_remaining.get_mut(&card_id)`:
- `None` → `Err(DistributeError::UnknownCard)`
- `Some(0)` → `Err(DistributeError::Exhausted)` — do NOT decrement
- `Some(n)` → `*n -= 1; Ok(())`

**`total_acquired()`**: `initial_count[id].saturating_sub(copies_remaining[id])` — no separate field needed.

**CI grep gate** (enforced at story done): `grep -rE "copies_remaining\.(insert|remove|entry)" server/src/ | grep -v "core/pool/api.rs"` must return zero matches.

**Expose `compute_weights()` as `pub(crate)`** for Story 002's Formula 2 tests — do not implement here but design the struct to make this extraction easy (avoid deeply private state).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: All draw functions (`draw_class_card`, `draw_neutral_family`, `draw_initial_draft`, etc.) and Formula 2 weighting
- [Story 003]: `refresh_shop()` function and `ShopSlots`/`InitialDraftOffering` resources
- [Story 004]: Bevy systems (`on_session_ready_init`, `on_shop_refresh_needed`) and `CardPoolPlugin`
- Foundation `game-config-pipeline` epic: `load_card_catalog()`, hard validation of `cards.json` (duplicate IDs, invalid JSON, missing rarity field). This story assumes `Res<CardCatalog>` is already built and available.

---

## QA Test Cases

*Written by QA Lead at story creation. Implement against these — do not invent new test cases.*

- **AC-1** — `test_initialize_catalog_length`
  - Given: `CardCatalog` built with 5 cards (mix of rarities, some with override=None)
  - When: `PlayerPool::initialize(&catalog, &config)` called
  - Then: `pool.copies_remaining.len() == 5` and `pool.initial_count.len() == 5`
  - Edge cases: Empty catalog → both maps len == 0, no panic

- **AC-2** — `test_initialize_all_cards_have_copies`
  - Given: Catalog with one card of each rarity (Common, Uncommon, Rare, Epic, Legendary), all with `pool_copies_override: None`
  - When: `PlayerPool::initialize(&catalog, &config)` with `config.common_pool_copies=6` etc.
  - Then: `copies_remaining(common_id)==6`, `copies_remaining(uncommon_id)==5`, `copies_remaining(rare_id)==4`, `copies_remaining(epic_id)==1`, `copies_remaining(legendary_id)==1`
  - Edge cases: All five rarity variants must produce ≥ 1 copy

- **AC-3** — `test_soft_error_override_zero_and_negative`
  - Given: Catalog with a Rare card (`pool_copies_override: Some(0)`) and another Rare card (`pool_copies_override: Some(-3)`)
  - When: `PlayerPool::initialize()` called
  - Then: Neither init panics; `copies_remaining(id_zero) == 4` (rarity default); `copies_remaining(id_neg) == 4`; two `tracing::error!` events captured (use `tracing_test` or similar)
  - Edge cases: Override == i32::MIN; override == -1; ensure no abort

- **AC-4** — `test_rare_no_override_gets_rarity_default`
  - Given: Catalog with exactly one Rare card, `pool_copies_override: None`, `config.rare_pool_copies = 4`
  - When: `initialize()` called
  - Then: `copies_remaining(id) == 4`
  - Edge cases: Config value `rare_pool_copies = 1` → copies == 1

- **AC-5** — `test_distribute_decrements_correctly`
  - Given: Pool initialized for a Rare card (4 copies); `copies_remaining(id) == 4`
  - When: `distribute(id)` called once
  - Then: Returns `Ok(())`; `copies_remaining(id) == 3`
  - Edge cases: Call distribute 4 times in sequence → copies go 4→3→2→1→0; last Ok(()) returns before Exhausted

- **AC-6** — `test_positive_override_applied`
  - Given: Rare card with `pool_copies_override: Some(2)`
  - When: `initialize()` called
  - Then: `copies_remaining(id) == 2` (not 4)
  - Edge cases: Override == 1 (minimum valid); Override == 99 (max expected)

- **AC-7** — `test_distribute_exhausted_error`
  - Given: Pool with card at `copies_remaining == 0` (achieved by distributing all copies)
  - When: `distribute(id)` called
  - Then: Returns `Err(DistributeError::Exhausted)`; `copies_remaining(id)` still == 0
  - Edge cases: Call distribute twice in a row on exhausted card → both return Exhausted; `copies_remaining` never goes negative

- **AC-8** — `test_is_available_false_at_zero`
  - Given: Pool with `copies_remaining(id) == 0`
  - When: `is_available(id)`
  - Then: `false`
  - Edge cases: Unknown card_id → `false` (not panic)

- **AC-9** — `test_is_available_true_above_zero`
  - Given: Pool with `copies_remaining(id) == 1`
  - When: `is_available(id)`
  - Then: `true`
  - Edge cases: copies == 4 → also true; copies == u32::MAX → true

- **AC-10** — `test_initial_count_immutable_total_acquired_correct`
  - Given: Pool with card at `initial_count(id) == 4`
  - When: `distribute(id)` called 3 times (K=3)
  - Then: `initial_count(id) == 4` (unchanged); `copies_remaining(id) == 1`; `total_acquired(id) == 3`
  - Edge cases: K == 0 → `total_acquired == 0`; K == N (all copies) → `total_acquired == N`, `copies_remaining == 0`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/pool/pool_state_test.rs` — must exist and all tests must pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Foundation `workspace-and-shared-types` epic (Story 001–004) — `shared/src/card.rs` with `CardId`, `CardData`, `Rarity`, `ClassId` must be DONE. Foundation `game-config-pipeline` epic — `GameConfig` with pool copy fields must be DONE and `Res<CardCatalog>` available.
- Unlocks: Story 002 (weighted draw — needs `PlayerPool` struct and `distribute()`)

## Completion Notes
**Completed**: 2026-04-30
**Criteria**: 10/10 passing
**Deviations**: None — ADR-006 module layout followed (state.rs/api.rs/plugin.rs). CI grep gate PASS.
**Test Evidence**: Logic — 21 embedded tests in `server/src/core/pool/api.rs` + evidence doc `tests/unit/pool/pool_state_test.rs`. CI green.
**Code Review**: Skipped (Lean mode)
