# Story 002: Weighted Draw

> **Epic**: Card Data & Pool
> **Status**: Retired - superseded
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

> **Retired**: 2026-05-04 during S5-22 hygiene cleanup.
> **Canonical story**: [Story 002: Weighted Draw Functions](story-002-weighted-draw-functions.md)
> **Reason**: Older duplicate remained marked Ready after the canonical story completed; retained for historical reference only.

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: TR-CDP-06, TR-CDP-07 (Formula 2 weighted draw; all draw functions return `Option<CardId>`; empty filtered pool returns `None`)

**ADR Governing Implementation**: ADR-006: Card Data Schema and Pool State Architecture
**ADR Decision Summary**: The pool owns no RNG source — callers supply seeds from `Res<ServerRng>`. `draw()` with a `PoolFilter` implements Formula 2 CDF weighting: `raw_weight(t) = (1/|eligible|) + SHOP_WEIGHT_PER_CARD_OWNED × total_acquired(t)`, clamped to `SHOP_WEIGHT_CAP`, normalized to sum 1.0 ± 1e-6. Weight cap clamping prevents any single card dominating the draw. All draw functions return `Option<CardId>` — never panic on an empty or exhausted filtered pool. Convenience wrappers (`draw_class_card`, `draw_neutral_family`, `draw_family_card`, `draw_random`) are thin shims over `draw()`.

**Engine**: Bevy 0.18 | **Risk**: LOW
**Engine Notes**: `api.rs` draw functions are pure Rust — no Bevy ECS surface. `rand` and `rand_chacha` are stable, version-pinned crates (no post-cutoff risk). `liv-bevy-018` skill is still mandatory on all `.rs` files in `server/src/`.

**Control Manifest Rules (Core layer)**:
- Required: Every draw function returns `Option<CardId>` or `Option<String>` — never `CardId` directly. No panic path exists for empty pools.
- Required: The CDF normalization sum must equal `1.0 ± 1e-6` (verified by CP-NW test). Floating-point edge case (`draw == 1.0`) handled by returning `eligible.last().copied()`.
- Required: `draw()` calls `rng.next_seed(RngEvent::DrawShopSlot { ... })` exactly once per invocation — one seed consumption per draw. No internal RNG state held.
- Required: Weight cap clamping uses `.min(config.shop_weight_cap)` before normalization — raw weights above `SHOP_WEIGHT_CAP` are silently capped.
- Forbidden: Empty `eligible` set must return `None` immediately — no CDF construction on empty input.
- Forbidden: `shop_weight_cap <= 0.0` must be rejected at startup validation (Story 001 `load_card_catalog`). The draw function may `debug_assert!(config.shop_weight_cap > 0.0)` but must not itself abort.

---

## Acceptance Criteria

- [ ] `server/src/core/pool/api.rs` is extended with:
  - `draw(pool: &PlayerPool, catalog: &CardCatalog, filter: PoolFilter, rng: &mut ServerRng, config: &GameConfig) -> Option<CardId>` — implements Formula 2 CDF weighting; returns `None` on empty filtered pool
  - `draw_class_card(pool, catalog, class: ClassId, rng, config) -> Option<CardId>` — convenience wrapper: filter by `class`, eligible only where `copies_remaining > 0`
  - `draw_neutral_family(pool, catalog, family_index: &FamilyIndex, rng, config) -> Option<String>` — weighted pick over eligible neutral families; weight = sum of `total_acquired` across family cards
  - `draw_family_card(pool, family: &str, family_index: &FamilyIndex, rng) -> Option<CardId>` — uniform pick among available cards in a selected family
  - `draw_random(pool, catalog, filter: &PoolFilter, rng) -> Option<CardId>` — uniform (unweighted) pick over cards matching filter and having `copies_remaining > 0`
- [ ] Formula 2 implementation in `weighted_cdf_draw()`:
  - `raw_weight(t) = (1.0 / |eligible|) + config.shop_weight_per_card × total_acquired(t)`
  - `raw_weight(t) = raw_weight(t).min(config.shop_weight_cap)`
  - `normalized_weight(t) = raw_weight(t) / Σ raw_weight(t')`
  - CDF built from normalized weights; uniform `[0, 1)` draw selects first bucket where cumulative sum >= draw value
  - Floating-point fallback: if no bucket selected (draw == 1.0 rounding), return `eligible.last().copied()`
- [ ] **CP7**: GIVEN all class cards have `copies_remaining = 0`, WHEN `draw_class_card(pool, catalog, class, rng, config)` called, THEN returns `None` (no panic)
- [ ] **CP8a**: GIVEN all class and all neutral cards exhausted, WHEN `draw(pool, catalog, PoolFilter::default(), rng, config)` called, THEN returns `None`
- [ ] **CP-SHC**: GIVEN a player of class `Iop`, WHEN `draw_class_card(pool, catalog, ClassId::Iop, rng, config)` called with available Iop cards, THEN returned `CardId` maps to a card with `class == ClassId::Iop` in the catalog
- [ ] **CP-SHN**: GIVEN available neutral families with at least one available card, WHEN `draw_neutral_family()` then `draw_family_card()` called, THEN returned `CardId` maps to a card with the correct family and `class == ClassId::Neutral`
- [ ] **CP-NW**: GIVEN an eligible set of 5 cards with varying `total_acquired` counts, WHEN Formula 2 weights are computed and normalized, THEN `Σ normalized_weight(t)` equals `1.0 ± 1e-6`
- [ ] **CP9**: GIVEN card X with `total_acquired = 3`, `shop_weight_per_card = 0.1`, `|eligible| = 5`, WHEN `raw_weight(X)` computed, THEN `raw_weight(X) == (1.0/5) + 0.1 × 3 == 0.5`
- [ ] **CP10**: GIVEN card X with `total_acquired = 7`, `shop_weight_per_card = 0.1`, `shop_weight_cap = 0.5`, WHEN raw weight computed, THEN clamped result `== 0.5` (not `0.2 + 0.7 = 0.9`)
- [ ] **CP-B**: GIVEN a `PoolFilter` matching zero cards, WHEN `draw_random(pool, catalog, filter, rng)` called, THEN returns `None` (empty filtered pool does not panic)
- [ ] `draw()` calls `rng.next_seed(RngEvent::DrawShopSlot { ... })` exactly once per call; this is verified by checking `Res<ServerRng>.audit_log` after a draw
- [ ] Weight cap clamping: card with highest `total_acquired` in a test set does NOT exceed `config.shop_weight_cap` in its pre-normalization raw weight
- [ ] `cargo check -p server` passes after extending `api.rs`

---

## Implementation Notes

*Derived from EPIC.md §Deliverables, ADR-006 §Decision, and `card-data-pool.md` Formula 2:*

**Formula 2 pseudocode:**
```rust
fn weighted_cdf_draw(
    pool: &PlayerPool,
    eligible: &[CardId],
    seed: u64,
    config: &GameConfig,
) -> Option<CardId> {
    let base = 1.0_f32 / eligible.len() as f32;
    let raw: Vec<f32> = eligible.iter().map(|id| {
        let bonus = config.shop_weight_per_card * pool.total_acquired(*id) as f32;
        (base + bonus).min(config.shop_weight_cap)
    }).collect();

    let total: f32 = raw.iter().sum();
    if total <= 0.0 { return None; }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let draw: f32 = rng.gen_range(0.0_f32..1.0_f32);
    let mut cumulative = 0.0_f32;
    for (i, &w) in raw.iter().enumerate() {
        cumulative += w / total;
        if draw <= cumulative { return Some(eligible[i]); }
    }
    eligible.last().copied()   // float edge case
}
```

**`draw()` top-level filter application**: Before building the eligible set, apply `PoolFilter` fields in order: `class` filter, then `family` filter, then `max_rarity` filter. Only cards with `copies_remaining > 0` are eligible after all filters pass.

**`draw_neutral_family()` weighting**: Weight per family = sum of `total_acquired` across all cards in that family. A family with all copies remaining has weight `0 + base_weight`. A family where the player owns many copies has elevated weight — this is the archetype-commitment signal for neutral cards.

**`draw_family_card()` is uniform**: After a family is selected by the weighted phase, the specific card within the family is drawn uniformly (`SliceRandom::choose` on available cards). No Formula 2 weighting at the intra-family level.

**`draw_random()` is uniform**: Used for draw effects and Prism Lane 3. The `PoolFilter` struct from Story 001 (`class`, `family`, `max_rarity`) is applied; matching cards are collected; uniform selection via `SliceRandom::choose`. The caller is responsible for calling `distribute()` if the draw effect consumes the card.

**RNG seed consumption contract**: `draw()` obtains its seed by calling `rng.next_seed(RngEvent::DrawShopSlot { player_id, slot_index, round })`. The `slot_index` is provided by the caller (the `refresh_shop` loop in Story 003). The audit log entry is written by `ServerRng.next_seed()` — the draw function itself does not write to the audit log.

**`FamilyIndex` type**: `HashMap<String, Vec<CardId>>` — built server-side from `CardCatalog` at startup (Story 003 delivers `build_family_index()`). If not yet available when this story is implemented, stub as a parameter type and add a `// TODO: build in story-003` comment.

---

## Out of Scope

- Story 001: `PlayerPool` struct, `distribute()`, `is_available()`, `total_acquired()`, `PoolFilter` struct
- Story 003: `refresh_shop()` (the loop that calls `draw()` for each slot), `draw_initial_draft()`, `draw_auction_card()`, `FamilyIndex` builder
- Story 004: System-layer wiring — `on_shop_refresh_needed` that calls `draw()` per slot
- Story 005: Manual refresh integration
- Phase 1 (50/50 class/neutral slot split roll): this belongs to the Card Acquisition system, not the pool. The pool receives a `PoolFilter` specifying class or neutral; it does not roll the split itself.

---

## QA Test Cases

- **CP9: Formula 2 raw weight at 3 owned copies**
  - Given: `eligible = [A, B, C, D, E]` (5 cards); card A has `total_acquired = 3`; `shop_weight_per_card = 0.1`; `shop_weight_cap = 0.5`
  - When: raw weight for A computed
  - Then: `raw_weight(A) = (1.0/5) + 0.1 × 3 = 0.2 + 0.3 = 0.5`

- **CP10: Weight clamped at cap**
  - Given: card X has `total_acquired = 7`; `shop_weight_per_card = 0.1`; `shop_weight_cap = 0.5`
  - When: raw weight computed
  - Then: unclamped would be `0.2 + 0.7 = 0.9`; clamped result is `0.5`

- **CP-NW: Normalized weights sum to 1.0 ± 1e-6**
  - Given: 5-card eligible set with `total_acquired` = [0, 1, 2, 3, 7]; `shop_weight_per_card = 0.1`; `shop_weight_cap = 0.5`; `|eligible| = 5`
  - When: all normalized weights computed
  - Then: `Σ normalized_weight` is within `[0.999999, 1.000001]`

- **CP7: Empty class pool returns None**
  - Given: pool initialized; all Iop cards set to `copies_remaining = 0`
  - When: `draw_class_card(pool, catalog, ClassId::Iop, rng, config)`
  - Then: returns `None`, no panic

- **CP-B: Empty filtered pool returns None**
  - Given: filter for `Rarity::Legendary`; all Legendary cards set to `copies_remaining = 0`
  - When: `draw_random(pool, catalog, filter, rng)`
  - Then: returns `None`, no panic

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/pool/draw_test.rs` — all acceptance criteria passing; covers CP7, CP8a, CP-SHC, CP-SHN, CP-NW, CP9, CP10, CP-B, and RNG audit log entry count
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (provides `PlayerPool`, `PoolFilter`, `distribute()`, `total_acquired()`, `DistributeError`, resource types)
- Depends on: `workspace-and-shared-types` Story 002 (provides `CardCatalog`, `ClassId`, `Rarity`, `FamilyId` types)
- Depends on: `server-rng` Story 001 (provides `ServerRng` resource and `next_seed()` API — required to obtain draw seeds; `RngEvent::DrawShopSlot` event type)
- Unlocks: Story 003 (refresh shop — requires `draw_class_card`, `draw_neutral_family`, `draw_family_card`)
- Unlocks: Story 004 (system integration — Story 003 must be complete first)
