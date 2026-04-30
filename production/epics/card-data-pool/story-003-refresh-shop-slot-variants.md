# Story 003: refresh_shop + Slot Variants

> **Epic**: Card Data & Pool
> **Status**: Complete
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: `TR-CDP-09` (partial — `refresh_shop` function; system-level subscriber is Story 004)
*(TR-IDs are informal — `docs/architecture/tr-registry.yaml` is unpopulated.)*

**ADR Governing Implementation**: [ADR-006: Card Data Schema and Pool State Architecture](../../../docs/architecture/adr-006-card-data-schema.md)
**ADR Decision Summary**: `refresh_shop` is an atomic distribute-and-collect — it draws N cards and calls `distribute()` for each successful draw before returning. Returns `Vec<CardId>` (possibly shorter than `slot_count` if pool is partially exhausted). Never panics on empty pool. Slot count is 9 for `DRAFT_INITIAL`, 3 for `DRAFT_SHOP` or manual refresh.

**Engine**: Bevy 0.18 + Rust stable | **Risk**: LOW (pure function; no Bevy API dependency)
**Engine Notes**: `ShopSlots`, `InitialDraftOffering`, and `ManualRefreshCount` are plain Bevy `Resource`s wrapping `HashMap`s. `#[derive(Resource)]` required on each. No bundle usage.

**Control Manifest Rules (Core layer)**:
- Required: `distribute()` is the sole pool mutation — `refresh_shop` must call it (not write to `copies_remaining` directly).
- Required: All draw functions return `Option<T>`. `refresh_shop` must handle `None` gracefully — partial fills are valid.
- Forbidden: Panicking on exhausted pool. Partial fill (< `slot_count` cards) is correct behavior, not an error.
- Guardrail: Server tick budget ≤ 5ms steady state. `refresh_shop` runs per-player per DRAFT — must be O(N) in eligible pool size.

---

## Acceptance Criteria

*From EPIC.md deliverables and GDD §8, scoped to this story:*

- [ ] **AC-1**: GIVEN a `PlayerPool` with >= 3 eligible cards and `slot_count = 3`, WHEN `refresh_shop(pool, catalog, rng_seed, config, 3)` is called, THEN the returned `Vec<CardId>` has length 3; each returned card had `copies_remaining >= 1` before the call; each returned card's `copies_remaining` is decremented by 1 after the call.
- [ ] **AC-2**: GIVEN a `PlayerPool` with >= 9 distinct eligible cards and `slot_count = 9`, WHEN `refresh_shop(pool, catalog, rng_seed, config, 9)`, THEN returned `Vec<CardId>` has length 9; all 9 are distinct; each copy decremented.
- [ ] **AC-3** (partial fill): GIVEN a `PlayerPool` where only 2 eligible cards have `copies_remaining >= 1` (all others == 0) and `slot_count = 3`, WHEN `refresh_shop(pool, catalog, rng_seed, config, 3)`, THEN returned `Vec<CardId>` has length 2 (partial fill); no panic; no placeholder `None`s in the `Vec` — callers use `Vec.len()` to detect partial fill.
- [ ] **AC-4**: GIVEN `ManualRefreshCount[player] == 3` from a prior DRAFT phase, WHEN a new `ShopRefreshNeeded` is processed for that player at a DRAFT entry (by the system in Story 004), THEN `ManualRefreshCount[player] == 0` after processing. *(This story initialises the `ManualRefreshCount` resource and resets logic; the trigger is tested in Story 004 as an integration test.)*

---

## Implementation Notes

*From ADR-006 and EPIC.md deliverables:*

**`refresh_shop` signature** (in `server/src/core/pool/api.rs`):
```rust
/// Atomic shop refresh: draws up to `slot_count` cards, distributes each on success.
/// Returns the drawn Vec. Length may be < slot_count if pool is partially exhausted.
/// Never panics. Caller handles the partial-fill case.
pub fn refresh_shop(
    pool: &mut PlayerPool,
    catalog: &CardCatalog,
    family_index: &HashMap<String, Vec<CardId>>,
    rng: &mut ServerRng,
    config: &GameConfig,
    slot_count: usize,
) -> Vec<CardId>
```

**Atomic draw-and-distribute loop**:
For each slot (up to `slot_count`):
1. Draw one card via `draw_class_card` or `draw_neutral_family` + `draw_family_card` (caller of `refresh_shop` determines the class context; alternatively, the function does one weighted draw per slot)
2. If draw returns `Some(card_id)`: call `distribute(pool, card_id)` — card is now committed
3. If draw returns `None`: stop — pool is exhausted; do not add `None` to the result
4. Return collected `Vec<CardId>` (may be shorter than `slot_count`)

**Note on draw strategy within `refresh_shop`**: The EPIC's `refresh_shop` calls the draw functions with seed per slot. The per-slot draw type (class vs. neutral split) is determined by Card Acquisition's Phase 1 split roll — `refresh_shop` itself does not do Phase 1; it draws from the appropriate sub-function. The system layer (Story 004) passes the correct context. For implementation simplicity, `refresh_shop` may use `draw()` with a `PoolFilter` or call `draw_class_card`/`draw_neutral_family`/`draw_family_card` in sequence per slot.

**Slot count semantics**:
- `slot_count = 9` → `DRAFT_INITIAL` (9-card selection)
- `slot_count = 3` → `DRAFT_SHOP` or manual refresh (3-card shop)
The system layer (Story 004) determines which count to use.

**`ShopSlots` resource**:
```rust
#[derive(Resource, Default)]
pub struct ShopSlots(pub HashMap<PlayerId, Vec<CardId>>);
// Vec length may be < 3 on partial fill; client must handle this
```

**`InitialDraftOffering` resource**:
```rust
#[derive(Resource, Default)]
pub struct InitialDraftOffering(pub HashMap<PlayerId, Vec<CardId>>);
// 9 cards for DRAFT_INITIAL; cleared after DRAFT_INITIAL
```

**`ManualRefreshCount` resource**:
```rust
#[derive(Resource, Default)]
pub struct ManualRefreshCount(pub HashMap<PlayerId, u32>);
// Reset to 0 at each DRAFT phase entry per player
// nth refresh costs refresh_base_cost + (n - 1) gold
```

**Reset semantics for `ManualRefreshCount`**: Reset to 0 for each player when `ShopRefreshNeeded` is processed (auto DRAFT refresh). This story declares the resource and the reset helper; the reset call lives in Story 004's system.

**Partial fill behaviour** (per ADR-006 "Empty-filtered-pool" section): When the pool has fewer eligible cards than `slot_count`, the returned `Vec` is compact — it contains only the successfully drawn cards. The client renders missing slots as empty via `Vec.len() < slot_count` check. No `None` padding in the returned `Vec`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: Individual draw functions (`draw_class_card`, `draw_neutral_family`, etc.) — must be DONE first
- [Story 004]: Bevy system (`on_shop_refresh_needed`) that calls `refresh_shop` in response to messages; `ManualRefreshCount` reset as a system side effect
- [Story 005]: `on_manual_refresh` system; cost escalation logic; Economy `validate_spend`/`apply_spend` integration

---

## QA Test Cases

*Written by QA Lead at story creation. Implement against these — do not invent new test cases.*

- **AC-1** — `test_refresh_shop_3_slots_full`
  - Given: Pool with 10 eligible cards (various classes + neutrals), all `copies_remaining >= 2`; `slot_count = 3`; mock seeds
  - When: `refresh_shop(pool, catalog, family_index, &seeds, config, 3)` called
  - Then: `result.len() == 3`; each returned card had `copies_remaining >= 1` pre-call; each card's `copies_remaining` decremented by exactly 1 post-call
  - Edge cases: All 3 drawn cards are distinct (verify no double-distribution of same card in one refresh call)

- **AC-2** — `test_refresh_shop_9_slots_initial_draft`
  - Given: Pool with 15 eligible cards (class + neutral mix), all `copies_remaining >= 2`; `slot_count = 9`; mock seeds
  - When: `refresh_shop(pool, catalog, family_index, &seeds, config, 9)` called
  - Then: `result.len() == 9`; all 9 IDs are distinct; `copies_remaining` decremented for each
  - Edge cases: Exactly 9 eligible cards → returns all 9 (no shortage)

- **AC-3** — `test_refresh_shop_partial_fill`
  - Given: Pool where only 2 cards have `copies_remaining >= 1`, all others == 0; `slot_count = 3`
  - When: `refresh_shop(pool, catalog, family_index, &seeds, config, 3)` called
  - Then: `result.len() == 2` (not 3); no panic; `copies_remaining` for both returned cards == 0 post-call
  - Edge cases: Pool fully exhausted (0 eligible) → `result.len() == 0`; `copies_remaining` unchanged; no panic

- **AC-4** — `test_manual_refresh_count_reset_on_draft_entry`
  - Given: `ManualRefreshCount(map with player_A → 3)` in a Bevy `World`
  - When: Reset helper called for `player_A` (simulating DRAFT entry)
  - Then: `ManualRefreshCount[player_A] == 0`
  - Edge cases: Player with no existing entry → 0 after reset; multiple players → only the targeted player reset

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/pool/refresh_shop_test.rs` — must exist and all tests must pass

**Status**: [x] Created and passing in CI run `25167672501`

---

## Completion Notes

**Completed**: 2026-04-30
**Criteria**: 4/4 passing
**Deviations**: None blocking. `refresh_shop` uses the weighted draw path directly and leaves future class/neutral split policy to the subscriber story, matching the story note.
**Test Evidence**: Logic evidence at `tests/unit/pool/refresh_shop_test.rs`; runnable tests embedded in `server/src/core/pool/api.rs` and covered by `cargo test -p server` in CI run `25167672501`.
**Implementation Commits**: `901823d` (S2-04 pool refresh implementation), `e4ac84e` (integration repair + doctest CI fix)
**Code Review**: Lean mode skipped; CI green.

---

## Dependencies

- Depends on: Story 002 (Weighted Draw Functions) must be **DONE** — `draw_class_card`, `draw_neutral_family`, `draw_family_card` required
- Unlocks: Story 004 (ShopRefreshNeeded Subscriber — calls `refresh_shop`)
