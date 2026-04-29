# Story 002: Weighted Draw Functions

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirements**: `TR-CDP-06`, `TR-CDP-07`
*(TR-IDs are informal — `docs/architecture/tr-registry.yaml` is unpopulated.)*

**ADR Governing Implementation**: [ADR-006: Card Data Schema and Pool State Architecture](../../../docs/architecture/adr-006-card-data-schema.md)
**ADR Decision Summary**: All draw functions return `Option<T>` — never panic on empty pool. Weighted draw (Formula 2) uses `total_acquired = initial_count - copies_remaining` derived from existing fields — no extra tracking. The pool never owns an RNG source; callers supply explicit seeds from `ServerRng`.

**Engine**: Bevy 0.18 + Rust stable | **Risk**: LOW (pure data structures; rand_chacha seeded RNG is stable)
**Engine Notes**: `rand_chacha::ChaCha8Rng::seed_from_u64(seed)` is used for all draws. This crate is confirmed in `Cargo.toml` (server). The `rand` version is `0.9` — confirm `seq::SliceRandom` and `Rng` traits still match the `0.9` API before implementing.

**Control Manifest Rules (Core layer)**:
- Required: All pool draw functions return `Option<T>`. Never panic on empty pool — return `None` and let caller handle it.
- Required: `total_acquired(id)` is derived: `initial_count[id] - copies_remaining[id]`. No separate stored field.
- Forbidden: `unwrap()` in production paths.
- Forbidden: Client-side RNG. Pool receives seeds from `ServerRng` — it never creates or holds an RNG.
- Guardrail: O(1) `CardCatalog` lookup by `CardId` via `HashMap`.

---

## Acceptance Criteria

*From GDD `design/gdd/card-data-pool.md` §8, scoped to this story:*

- [ ] **AC-1 (CP7)**: GIVEN a pool where all cards for `ClassId::Iop` have `copies_remaining == 0` and at least one Neutral card has `copies_remaining >= 1`, WHEN `draw_class_card(Iop, seed)` is called 100 times with distinct seeds, THEN every call returns `None`.
- [ ] **AC-2 (CP-SHC)**: GIVEN a pool with eligible Iop class cards, WHEN `draw_class_card(Iop, seed)`, THEN the returned `CardId` (if `Some`) has `class == ClassId::Iop` and `copies_remaining(card_id) >= 1` at call time.
- [ ] **AC-3 (CP-SHN)**: GIVEN a pool with eligible neutral families (family "Gobball" has >= 1 available card), WHEN `draw_neutral_family(seed)` returns `Some("Gobball")` and then `draw_family_card("Gobball", seed2)`, THEN returned `CardId` has `class == Neutral`, `copies_remaining >= 1`, and belongs to family "Gobball".
- [ ] **AC-4 (CP-NW)**: GIVEN a pool with eligible types and mixed `total_acquired` values, WHEN normalized weights are computed for all eligible types, THEN `|Σ normalized_weight(t) - 1.0| < 1e-6`.
- [ ] **AC-5 (CP-A)**: GIVEN all Neutral cards with `rarity ∈ {Rare, Legendary}` have `copies_remaining == 0`, WHEN `draw_auction_card(auction_pool, catalog, seed)`, THEN returns `None`.
- [ ] **AC-6 (CP-B)**: GIVEN all cards matching a given `PoolFilter` have `copies_remaining == 0`, WHEN `draw_random(catalog, filter, seed)`, THEN returns `None` and `distribute()` is never called.
- [ ] **AC-7 (CP-C)**: GIVEN a catalog with >= 9 eligible cards (Iop + Neutral combined), WHEN `draw_initial_draft(catalog, ClassId::Iop, 9, seed)`, THEN returns `Vec<CardId>` of length 9 with no duplicate IDs.
- [ ] **AC-8 (CP-C2)**: GIVEN `draw_initial_draft(catalog, ClassId::Iop, 9, seed)` returns 9 IDs, WHEN each ID is looked up in the catalog, THEN every card has `class == ClassId::Iop` OR `class == ClassId::Neutral` (no other class present).
- [ ] **AC-9 (CP-C3)**: `draw_initial_draft()` does NOT call `distribute()` internally — `copies_remaining` is unchanged for all returned cards after the call.
- [ ] **AC-10 (CP9)**: GIVEN 25 eligible Iop class cards where card T has `total_acquired == 3` and all 24 others have `total_acquired == 0`, with `shop_weight_per_card=0.10`, `shop_weight_cap=0.65`, WHEN Formula 2 weights are computed for T, THEN `raw_weight(T) == 0.34 ± 1e-6` and `normalized_weight(T) == 0.2615 ± 1e-4`.
- [ ] **AC-11 (CP10)**: GIVEN `shop_weight_cap=0.65`, `shop_weight_per_card=0.10`, 25 eligible types, card T has `total_acquired == 7`, WHEN raw weight is computed (`1/25 + 0.10 × 7 = 0.74`), THEN the clamped raw weight output equals `0.65` (cap applied, not 0.74).

---

## Implementation Notes

*From ADR-006 Implementation Guidelines:*

**File location**: `server/src/core/pool/api.rs` (draw functions) — or as `impl PlayerPool` methods in `state.rs`. The EPIC hint specifies free functions; ADR-006 shows impl methods. Either is acceptable — pick one layout and apply consistently.

**Formula 2 algorithm** (`draw_class_card` and `draw_neutral_family`):
```
raw_weight(t)        = (1 / |eligible_types|) + SHOP_WEIGHT_PER_CARD × total_acquired(t)
raw_weight(t)        = clamp(raw_weight(t), 0.0, SHOP_WEIGHT_CAP)
normalized_weight(t) = raw_weight(t) / Σ raw_weight(t')    for all t' in eligible_types
```
CDF draw: build cumulative sum, draw uniform [0.0, 1.0) from seeded `ChaCha8Rng`, find first bucket ≥ draw value. Floating-point edge (`draw == 1.0`): return last element.

**`draw_class_card`** — eligible set: cards where `class == player_class` AND `copies_remaining > 0`. Weight unit = individual `CardId`. Returns `None` on empty eligible set.

**`draw_neutral_family`** — eligible set: neutral families where `≥ 1 card` has `copies_remaining > 0`. Weight unit = family (aggregate `total_acquired` across all family cards). Returns `None` on empty eligible set.

**`draw_family_card`** — uniform pick from family's available cards (all members with `copies_remaining > 0`). No Formula 2 weighting — Phase 3 is uniform. Returns `None` only if family is fully exhausted.

**`draw_auction_card`** — static function receiving `auction_pool: &PlayerPool`. Eligible: `class == Neutral` AND `rarity ∈ {Rare, Legendary}`. Epic excluded (no Neutral Epics). Uniform draw (not Formula 2). Returns `None` if empty.

**`draw_random`** — uniform draw filtered by `PoolFilter`. Does NOT call `distribute()` internally. Caller owns consumption decision. Returns `None` if no cards match filter with `copies_remaining > 0`.

**`draw_initial_draft`** — Fisher-Yates shuffle of eligible cards (class + Neutral, any rarity, catalog-based not pool-copy-based). Use `SliceRandom::shuffle` with seeded `ChaCha8Rng`. Return first `count` IDs. Does NOT call `distribute()`. No duplicates guaranteed by shuffle.

**IMPORTANT — expose `pub(crate)` weight computation helper** (per QA Lead recommendation):
Expose a `pub(crate) fn compute_weights(eligible: &[CardId], pool: &PlayerPool, config: &GameConfig) -> Vec<f32>` to allow AC-10/AC-11 deterministic assertions without statistical sampling. This is the only sound way to test Formula 2 output precisely.

**RNG seed discipline**: Each draw function consumes exactly one seed via one `ChaCha8Rng::seed_from_u64(seed)` call. Do not call `next_seed()` inside pool functions — the caller (Card Acquisition / system layer) supplies the pre-generated seed. This is the RNG audit trail contract (ADR-005).

**`FamilyIndex`** is server-side only: `HashMap<String, Vec<CardId>>` built from `CardCatalog` at startup. `draw_neutral_family()` and `draw_family_card()` receive it as a parameter.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001]: `PlayerPool` struct, `distribute()`, `is_available()`, `total_acquired()` — must be DONE first
- [Story 003]: `refresh_shop()` — orchestrates multiple draw calls
- [Story 004]: Bevy system that calls these draw functions in response to `ShopRefreshNeeded`

---

## QA Test Cases

*Written by QA Lead at story creation. Implement against these — do not invent new test cases.*

- **AC-1** — `test_draw_class_card_all_exhausted_returns_none`
  - Given: Pool where all Iop cards have `copies_remaining == 0`; at least one Neutral card available
  - When: `draw_class_card(pool, catalog, ClassId::Iop, seed_0..seed_99, config)` called 100 times
  - Then: Every call returns `None`
  - Edge cases: Only one class card exists and it's exhausted → None; pool has zero Iop cards at all → None

- **AC-2** — `test_draw_class_card_returns_correct_class`
  - Given: Pool with 5 eligible Iop cards (all `copies_remaining >= 1`), `family_index` built
  - When: `draw_class_card(pool, catalog, ClassId::Iop, seed_42, config)` called
  - Then: Returns `Some(id)` where `catalog[id].class == ClassId::Iop` and `pool.copies_remaining[id] >= 1`
  - Edge cases: Run 20 times with different seeds — every result is an Iop card

- **AC-3** — `test_draw_neutral_family_then_draw_family_card`
  - Given: `family_index` with family "Gobball" → 3 Neutral cards; all 3 have `copies_remaining == 2`
  - When: `draw_neutral_family(pool, catalog, family_index, seed_1, config)` → `Some("Gobball")`; then `draw_family_card(pool, "Gobball", family_index, seed_2)` → `Some(card_id)`
  - Then: `catalog[card_id].class == ClassId::Neutral`; `catalog[card_id].family == Some("Gobball")`; `pool.copies_remaining[card_id] >= 1`
  - Edge cases: All families fully exhausted → `draw_neutral_family` returns `None`

- **AC-4** — `test_normalized_weights_sum_to_one`
  - Given: Pool with 25 Iop cards; 3 cards have `total_acquired == 2`, rest have `total_acquired == 0`; `shop_weight_per_card=0.10`, `shop_weight_cap=0.65`
  - When: `compute_weights(eligible_ids, pool, config)` called; weights normalized
  - Then: `|sum(normalized_weights) - 1.0| < 1e-6`
  - Edge cases: Single eligible type → normalized_weight == 1.0; all same total_acquired → uniform weights

- **AC-5** — `test_draw_auction_card_exhausted_returns_none`
  - Given: Auction pool where all Neutral Rare and Neutral Legendary cards have `copies_remaining == 0`; some non-eligible cards (Epic, non-Neutral) still available
  - When: `draw_auction_card(auction_pool, catalog, seed)`
  - Then: `None`
  - Edge cases: Only Neutral Common available → also None (wrong rarity); only Neutral Epic → None (Epics excluded)

- **AC-6** — `test_draw_random_exhausted_filter_returns_none`
  - Given: Pool where all Legendary Neutral cards have `copies_remaining == 0`; filter = `PoolFilter { rarity: Some(vec![Rarity::Legendary]), class: Some(ClassId::Neutral) }`
  - When: `draw_random(pool, catalog, filter, seed)` called
  - Then: Returns `None`; verify `copies_remaining` unchanged for all cards (no distribute called)
  - Edge cases: All-None filter on empty pool → None; filter with no matching cards → None

- **AC-7** — `test_draw_initial_draft_9_distinct_ids`
  - Given: Catalog with 20 eligible cards (15 Iop, 5 Neutral), all with `copies_remaining >= 1`
  - When: `draw_initial_draft(pool, catalog, ClassId::Iop, 9, seed_42)`
  - Then: Returns `Vec<CardId>` of exactly length 9; all 9 IDs are distinct (no duplicates)
  - Edge cases: Count=9 exactly available → returns all 9; count > available → returns all available (no panic)

- **AC-8** — `test_draw_initial_draft_class_and_neutral_only`
  - Given: Catalog with Iop, Cra, and Neutral cards
  - When: `draw_initial_draft(pool, catalog, ClassId::Iop, 9, seed)` returns 9 IDs
  - Then: Every returned ID has `class == Iop OR class == Neutral`; no Cra or other class present
  - Edge cases: All 9 come from Neutral (valid); all 9 come from Iop (valid)

- **AC-9** — `test_draw_initial_draft_does_not_call_distribute`
  - Given: Pool with all `copies_remaining` values set and recorded
  - When: `draw_initial_draft(pool, catalog, class, 9, seed)` called
  - Then: After the call, all `copies_remaining` values are identical to before the call
  - Edge cases: Multiple calls with different seeds → copies still unchanged

- **AC-10** — `test_formula2_raw_weight_at_3_owned`
  - Given: 25 eligible Iop card IDs; card T has `total_acquired == 3`, all 24 others have `total_acquired == 0`; config `shop_weight_per_card=0.10`, `shop_weight_cap=0.65`
  - When: `compute_weights(eligible_ids, pool, config)` called for all 25
  - Then: `raw_weight(T) == 1.0/25.0 + 0.10 * 3.0 == 0.34 ± 1e-6`; `normalized_weight(T) == 0.2615 ± 1e-4`; `normalized_weight(T) > normalized_weight(U)` for all unowned U
  - Edge cases: `total_acquired == 0` for all → uniform weights (`1/N` each)

- **AC-11** — `test_formula2_weight_clamped_at_cap`
  - Given: 25 eligible types; card T has `total_acquired == 7`; config `shop_weight_per_card=0.10`, `shop_weight_cap=0.65`
  - When: Pre-clamp raw weight computed: `1/25 + 0.10 * 7 = 0.74`; clamp applied
  - Then: The stored/used raw weight for T == `0.65` (not 0.74)
  - Edge cases: `total_acquired` such that raw == exactly cap → cap applied (not slightly over); weight_cap == 0.0 would be a GameConfig validation error (should not reach here)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/pool/weighted_draw_test.rs` — must exist and all tests must pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (Pool State + Core API) must be **DONE** — `PlayerPool`, `distribute()`, `total_acquired()` required
- Unlocks: Story 003 (refresh_shop needs the draw functions)
