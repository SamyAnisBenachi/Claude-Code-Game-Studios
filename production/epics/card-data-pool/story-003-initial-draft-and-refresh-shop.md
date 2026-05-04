# Story 003: Initial Draft & Refresh Shop

> **Epic**: Card Data & Pool
> **Status**: Retired - superseded
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

> **Retired**: 2026-05-04 during S5-22 hygiene cleanup.
> **Canonical story**: [Story 003: refresh_shop + Slot Variants](story-003-refresh-shop-slot-variants.md)
> **Reason**: Older duplicate remained marked Ready after the canonical story completed; retained for historical reference only.

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: TR-CDP-09 (initial draft offering; shop refresh 3/9 slots; auction card draw)

**ADR Governing Implementation**: ADR-006: Card Data Schema and Pool State Architecture
**ADR Decision Summary**: `draw_initial_draft()` draws 9 distinct cards without replacement from class + Neutral eligible set; `distribute()` is NOT called for undrafted cards. `refresh_shop()` is atomic: it draws N cards (3 or 9) and distributes each on success, returning the drawn vector for the caller to write to `ShopSlots`/`InitialDraftOffering`. Partial fill on exhaustion (fewer cards than `slot_count`) is legal — remaining slots surfaced as the caller's responsibility. `draw_auction_card()` draws from Neutral Rare/Legendary only. `FamilyIndex` is a server-side derived map built from `CardCatalog` at startup — not part of `shared/`.

**Engine**: Bevy 0.18 | **Risk**: LOW
**Engine Notes**: All functions in this story are pure Rust with no Bevy ECS surface. `FamilyIndex` is `HashMap<String, Vec<CardId>>`. `liv-bevy-018` mandatory on all `.rs` files.

**Control Manifest Rules (Core layer)**:
- Required: `draw_initial_draft()` returns `Vec<CardId>` — distinct IDs, never duplicates. `distribute()` is NOT called inside this function. Callers call `distribute()` only for cards the player actually purchases from the draft.
- Required: `refresh_shop()` calls `distribute()` for each successfully drawn card before returning. The returned vector must have `len() <= slot_count`. Empty or partial vectors are valid (pool exhausted).
- Required: `build_family_index()` must be a pure function — no side effects, no mutation, no Bevy deps.
- Forbidden: `draw_initial_draft()` must not panic when the eligible set has fewer than `count` cards — return all available cards up to `count`.
- Forbidden: `draw_auction_card()` must not include Epic cards — class-specific Epics are excluded by the Neutral + Rare/Legendary filter.

---

## Acceptance Criteria

- [ ] `server/src/core/pool/api.rs` is extended with:
  - `draw_initial_draft(pool: &PlayerPool, catalog: &CardCatalog, class: ClassId, count: u8, rng: &mut ServerRng) -> Vec<CardId>` — draws `count` (default 9) distinct cards without replacement; eligible = class cards + Neutral cards; any rarity eligible; `distribute()` NOT called
  - `refresh_shop(pool: &mut PlayerPool, catalog: &CardCatalog, family_index: &FamilyIndex, rng: &mut ServerRng, config: &GameConfig, slot_count: usize) -> Vec<CardId>` — atomic: draws up to `slot_count` cards; calls `distribute()` for each; returns drawn vector; partial fill valid on exhaustion
  - `draw_auction_card(auction_pool: &PlayerPool, catalog: &CardCatalog, rng: &mut ServerRng) -> Option<CardId>` — Neutral Rare or Legendary only; returns `None` when auction pool exhausted
- [ ] `server/src/core/pool/family_index.rs` (or inline in `api.rs`) exports:
  - `build_family_index(catalog: &CardCatalog) -> FamilyIndex` where `FamilyIndex = HashMap<String, Vec<CardId>>` — groups all Neutral cards by their `family` field; cards with `family == None` are skipped
  - Pure function: deterministic, no side effects
- [ ] **CP-C**: GIVEN a catalog with 20 class+Neutral cards, WHEN `draw_initial_draft(pool, catalog, class, 9, rng)` called, THEN returns `Vec<CardId>` with exactly 9 entries AND all IDs are distinct (no duplicates)
- [ ] **CP-C2**: GIVEN a catalog mixing class cards (e.g., `ClassId::Iop`) and Neutral cards and another class (e.g., `ClassId::Cra`), WHEN `draw_initial_draft(pool, catalog, ClassId::Iop, 9, rng)` called, THEN all returned IDs have `class == ClassId::Iop` OR `class == ClassId::Neutral` in the catalog; no `ClassId::Cra` cards appear
- [ ] **CP-C3**: GIVEN a successful `draw_initial_draft()` call returning 9 IDs, THEN `copies_remaining[id]` is UNCHANGED for all 9 IDs (i.e., `distribute()` was not called inside the function)
- [ ] **CP-A**: GIVEN all Neutral Rare and Legendary cards have `copies_remaining = 0`, WHEN `draw_auction_card(auction_pool, catalog, rng)` called, THEN returns `None` (no panic)
- [ ] `refresh_shop()` for 3 slots: GIVEN a pool with >= 3 eligible cards, WHEN `refresh_shop(pool, catalog, family_index, rng, config, 3)` called, THEN returns `Vec` with exactly 3 IDs AND `copies_remaining[id]` decremented by 1 for each returned ID
- [ ] `refresh_shop()` for 9 slots: GIVEN a pool with >= 9 eligible cards, WHEN `refresh_shop(pool, catalog, family_index, rng, config, 9)` called, THEN returns `Vec` with exactly 9 IDs AND all 9 copies distributed
- [ ] `refresh_shop()` partial fill: GIVEN a pool where only 2 eligible cards remain, WHEN `refresh_shop(pool, catalog, family_index, rng, config, 3)` called, THEN returns `Vec` with exactly 2 IDs (not 3); no panic; a debug-level warning is logged
- [ ] `draw_initial_draft()` with fewer than `count` eligible cards: GIVEN only 6 class+Neutral cards in catalog, WHEN `draw_initial_draft(pool, catalog, class, 9, rng)` called, THEN returns all 6 (not panicking on missing 3)
- [ ] `draw_auction_card()` exclusion: GIVEN an auction pool containing a Neutral Epic card, WHEN `draw_auction_card()` called, THEN the Epic card is never returned (Epic excluded by filter)
- [ ] `build_family_index()` correctness: GIVEN a catalog with 10 neutral cards across 3 families and 5 class cards, WHEN `build_family_index()` called, THEN returned index has exactly 3 keys; each key maps to the correct card IDs; class cards absent
- [ ] Each `draw_initial_draft()` call consumes exactly `count` seeds from `ServerRng` (one `next_seed()` call per drawn card)
- [ ] `cargo check -p server` passes after extending `api.rs` and adding `family_index.rs`

---

## Implementation Notes

*Derived from EPIC.md §Deliverables, ADR-006 §Decision, and `card-data-pool.md` Rules 4–5:*

**`draw_initial_draft()` algorithm:**
1. Collect eligible: all catalog cards where `class == player_class` OR `class == ClassId::Neutral`.
2. Draw without replacement: shuffle eligible via Fisher-Yates using a seeded `ChaCha8Rng`; take first `count` elements.
3. Do NOT call `distribute()`. Callers are responsible for calling `distribute()` only for actually purchased cards.
4. If `eligible.len() < count`: return all eligible (no padding, no panic).

**`refresh_shop()` algorithm:**
```
let mut drawn = Vec::with_capacity(slot_count);
for slot_index in 0..slot_count {
    let seed = rng.next_seed(RngEvent::DrawShopSlot { player_id, slot_index, round });
    match draw(pool, catalog, PoolFilter::default(), seed, config) {
        Some(card_id) => {
            distribute(pool, card_id).ok(); // Already checked available
            drawn.push(card_id);
        }
        None => {
            warn!("pool exhausted at slot {slot_index}");
            break;
        }
    }
}
drawn
```

The atomicity guarantee: all `distribute()` calls happen within this function. The caller receives the vector and writes it to `ShopSlots` or `InitialDraftOffering` — there is no intermediate state where some slots are distributed and others are not (partial fill is handled by `break`, not by leaving undistributed gaps).

**`draw_auction_card()` filter:** `class == ClassId::Neutral AND rarity IN [Rare, Legendary]`. Epic cards are class-specific in the lore; no Neutral Epics exist in the catalog. The filter can be implemented as a hard enum match rather than a configurable list — this is an invariant, not a tuning knob.

**`FamilyIndex` builder:**
```rust
pub fn build_family_index(catalog: &CardCatalog) -> FamilyIndex {
    let mut index: FamilyIndex = HashMap::new();
    for (id, card) in catalog {
        if card.class == ClassId::Neutral {
            if let Some(ref family) = card.family {
                index.entry(family.clone()).or_default().push(*id);
            }
        }
    }
    index
}
```
Store the result as `Res<FamilyIndex>` inserted at server startup (alongside `Res<CardCatalog>`). It is read-only after construction.

**RNG consumption order discipline (ADR-005):** For DRAFT_INITIAL, `refresh_shop(slot_count=9)` is called once per player in ascending `player_id` order; within each player's call, slot_index increments 0..9. For DRAFT_SHOP, same pattern for 0..3. The audit log compares replayed sessions to recorded outputs — any reordering breaks replay determinism. The `on_session_ready_init` system (Story 004) must iterate `PlayerIds` in deterministic order.

---

## Out of Scope

- Story 002: `draw()` core CDF logic — `refresh_shop()` calls `draw()` from Story 002
- Story 004: System-layer wiring (`on_session_ready_init`, `on_shop_refresh_needed` that calls `refresh_shop()`)
- Auction System GDD: shared auction pool initialization, multi-player coordination, `AuctionPool` resource — this story only provides the draw function; the Auction System consumes it
- Hand management (10-card cap): Card Acquisition epic — not this epic

---

## QA Test Cases

- **CP-C: Initial draft returns 9 distinct IDs**
  - Given: catalog with 30 class+Neutral cards; pool initialized
  - When: `draw_initial_draft(pool, catalog, class, 9, rng)`
  - Then: returns 9 IDs; `Set::from(ids).len() == 9`; each ID exists in catalog

- **CP-C3: distribute() not called during draft**
  - Given: pool initialized; record `copies_remaining` snapshot for all 9 drawn IDs
  - When: `draw_initial_draft()` called
  - Then: `copies_remaining[id]` unchanged for all 9 drawn IDs

- **refresh_shop partial fill — no panic**
  - Given: pool with exactly 2 cards having `copies_remaining > 0` (all others exhausted)
  - When: `refresh_shop(pool, catalog, family_index, rng, config, 3)`
  - Then: returns Vec with len == 2; no panic; warning logged

- **CP-A: Auction draw None on exhaustion**
  - Given: auction_pool with all Neutral Rare and Legendary cards at `copies_remaining = 0`
  - When: `draw_auction_card(auction_pool, catalog, rng)`
  - Then: returns `None`

- **FamilyIndex correctness**
  - Given: catalog with 4 Neutral cards in family "Gobball" and 3 in family "Tofu"; 5 Iop class cards (no family)
  - When: `build_family_index(catalog)`
  - Then: `index.len() == 2`; `index["Gobball"].len() == 4`; `index["Tofu"].len() == 3`; no Iop card IDs in index

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/pool/refresh_shop_test.rs` — all acceptance criteria passing; covers CP-C, CP-C2, CP-C3, CP-A, refresh_shop (3 slots), refresh_shop (9 slots), partial fill, `build_family_index()` correctness
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (provides `PlayerPool`, `PoolFilter`, `distribute()`, `DistributeError`, resource types)
- Depends on: Story 002 (provides `draw()` CDF implementation — `refresh_shop()` calls it per slot)
- Depends on: `workspace-and-shared-types` Story 002 (provides `CardCatalog`, `ClassId`, `Rarity`)
- Depends on: `server-rng` Story 001 (provides `ServerRng.next_seed()` — seed source for all draws)
- Unlocks: Story 004 (session init Observer and shop refresh subscriber — requires `refresh_shop()` and `draw_initial_draft()`)
- Unlocks: Story 005 (manual refresh — calls `refresh_shop(3)`)
- Partially unlocks: Auction System epic (Story 003 delivers `draw_auction_card()` — Auction System GDD can reference this API once accepted)
