# Story 003: Shop Draw Pipeline — Auto-Refresh, Dedup, and 50/50 Split

> **Epic**: Card Acquisition
> **Status**: Complete
> **Layer**: Feature (M2)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/card-acquisition.md`
**Requirements**: `TR-CA-003`, `TR-CA-005`, `TR-CA-010`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-015: Card Acquisition Shop State Machine Architecture
**ADR Decision Summary**: On `ShopRefreshTrigger::AuctionLock` or `ShopOpen`, the tick system draws 3 slots via a per-slot pipeline: Phase 1 (50/50 seed → `SlotType`), Phase 2 (class/neutral draw), optional Phase 3 (family→card), dedup check against `displayed_this_draft` with up to 20 retries, K≥N short-circuit to empty slot. All slots added to `displayed_this_draft` after draw. Dedup set is NOT cleared on `ShopUnlock` (auction-round DRAFT_SHOP entry).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `MessageReader<ShopRefreshTriggered>` — Bevy internal message bus; inject via `World::new()` in tests
- `app.add_message::<ShopRefreshTriggered>()` in plugin setup — required before `MessageReader` is usable
- `HashSet<CardId>` membership check is O(1) — safe in the retry loop
- `ServerRng::next_seed()` consumed per Phase 1 roll, Phase 2 draw, Phase 3 draw, and each fallback retry

**Control Manifest Rules (Feature layer — from ADR-015):**
- Required: `displayed_this_draft` NOT cleared on `ShopUnlock` — dedup accumulates across DRAFT_AUCTION + DRAFT_SHOP in auction rounds
- Required: `displayed_this_draft` cleared only on `DraftInitial`, `AuctionLock`, `ShopOpen` triggers (fresh DRAFT phase entry)
- Required: K≥N short-circuit before retry loop begins — never enter the loop if no unique card is possible
- Forbidden: No client-side RNG — all Phase 1/2/3 seeds come from `ServerRng::next_seed()`

---

## Acceptance Criteria

*From GDD `design/gdd/card-acquisition.md`, scoped to this story:*

- [x] **CA6** — GIVEN DRAFT_SHOP begins and auto-refresh fires, WHEN `S2CShopSlots` is sent, THEN all non-null slot IDs are absent from `displayed_this_draft` before the refresh, all are added to `displayed_this_draft` after, and all non-null IDs within the same message are mutually distinct (no intra-message duplicates).
- [x] **CA12** — GIVEN all eligible cards for a slot type are already in `displayed_this_draft` (K ≥ N), WHEN auto-refresh or manual refresh assigns this slot, THEN the slot is set to empty without any retry attempts.
- [x] **CA16** — GIVEN a player triggers a manual refresh after already receiving auto-refresh slots this DRAFT phase, WHEN `S2CShopSlots` is sent, THEN none of the 3 new card IDs match any card ID sent in any prior `S2CShopSlots` message since this DRAFT phase began.
- [x] **CA19** — GIVEN N = 0 (no eligible cards exist for a slot type — test fixture only), WHEN any refresh assigns this slot, THEN slot is set to empty immediately with no probability computation or retry.

---

## Implementation Notes

*Derived from ADR-015 Decision:*

**Per-slot draw pipeline** (called for each of 3 slots during auto-refresh):

```
Step 1 — Phase 1: consume seed → gen_range(0..2) → SlotType (Class or Neutral)
Step 2a — Phase 2 (Class): draw_class_card(class, next_seed()) → Option<CardId>
Step 2b — Phase 2 (Neutral): draw_neutral_family(next_seed()) → Option<FamilyId>
          if Some(family) → Phase 3: draw_family_card(family, next_seed()) → Option<CardId>
Step 3 — Fallback: if Phase 2 Class returned None → retry as Neutral (new seeds)
Step 4 — Dedup:
          K = eligible_in_displayed_this_draft_count()
          N = eligible_distinct_count()
          if K >= N → empty slot immediately (no retries)
          else → retry up to 20 times until candidate NOT in displayed_this_draft
          if 20 retries exhausted → empty slot
Step 5 — On success: add candidate to displayed_this_draft, assign to current_slots[i]
```

**`ShopRefreshTrigger` branches:**
- `AuctionLock` → clear `displayed_this_draft`, draw 3 slots, set `phase = AuctionLock`, send `S2CShopSlots`
- `ShopOpen` → clear `displayed_this_draft`, draw 3 slots, set `phase = ShopActive`, send `S2CShopSlots`
- `ShopUnlock` → do NOT clear `displayed_this_draft`; do NOT draw; set `phase = ShopActive`; reset `refresh_count_this_draft = 0`. Same slots become purchasable.

**CA6 test fixture**: Inject `ShopRefreshTriggered { trigger: ShopOpen }` into a `World::new()` test with a mock pool returning distinct cards. Verify `displayed_this_draft` set membership after the call.

**CA12 fixture**: Set `displayed_this_draft` to contain all eligible neutral or class cards before triggering a refresh. Verify the slot is `None` in `current_slots`.

**CA16 fixture**: Run auto-refresh, record 3 card IDs. Run manual refresh (Story 004 gate: must pass gold check). Verify the 3 new IDs have no intersection with the prior 3.

**CA19 fixture**: Configure pool to return `None` for all draw calls (N=0). Verify slot assigned to `None`, no seed consumed after the K≥N short-circuit.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001: `ShopStates` / `PlayerHands` / `ShopRefreshTriggered` type definitions
- Story 002: DRAFT_INITIAL `draw_initial_draft` path
- Story 004: Manual refresh cost validation (gold spend before draw) and counter increment
- Story 005: Purchase flow, `is_available` check, `spend_gold`, CA18 rollback

---

## QA Test Cases

- **CA6**: Auto-refresh populates dedup set and sends distinct slots
  - Given: `displayed_this_draft` is empty; pool returns distinct cards for all draw calls
  - When: `ShopRefreshTriggered { trigger: ShopOpen }` processed
  - Then: all 3 non-null `current_slots` IDs are in `displayed_this_draft`; all 3 are mutually distinct
  - Edge cases: pool returns None for one slot (slot is null, others distinct); all 3 slots null

- **CA12**: K≥N short-circuits to empty slot
  - Given: pool has N=2 eligible cards for Neutral type; both already in `displayed_this_draft` (K=2)
  - When: auto-refresh assigns a Neutral slot
  - Then: slot set to `None`; retry loop never entered (verify via seed consumption count — no extra seeds consumed)
  - Edge cases: K=N exactly; K=0 (no prior display — normal draw proceeds)

- **CA16**: Manual refresh produces no duplicates relative to prior auto-refresh
  - Given: auto-refresh sent 3 cards (all in `displayed_this_draft`); economy has sufficient gold
  - When: manual refresh triggered (via Story 004 gold path)
  - Then: 3 new cards in `S2CShopSlots` have empty intersection with prior 3
  - Edge cases: pool has exactly 6 distinct eligible cards (auto 3 + manual 3 exactly exhausts unique set)

- **CA19**: N=0 assigns empty slot immediately
  - Given: pool configured to return `None` for all draw/family calls; `displayed_this_draft` is empty
  - When: any refresh triggers slot assignment
  - Then: `current_slots` all `None`; `displayed_this_draft` unchanged (nothing to add); no retry seeds consumed
  - Edge cases: N=0 for Class slot type only (Neutral may still draw successfully)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/card_acquisition/draw_pipeline_test.rs` — must exist and pass
*(Testable via `World::new()` + injected `ShopRefreshTriggered` messages — no Lightyear session required)*

**Status**: [x] Verified locally with `cargo test -p server --test card_acquisition_draw_pipeline_test`

---

## Dependencies

- Depends on: Story 001 (`state-scaffold`) must be Done — `ShopStates`, `ShopRefreshTriggered`, `ShopRefreshTrigger` must be defined
- Unlocks: Story 004 (`refresh-cost`) and Story 005 (`purchase-flow`) both depend on this draw pipeline being in place

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 4/4 passing (CA6, CA12, CA16, CA19)
**Deviations**: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01; no blocking GDD/ADR drift found.
**Test Evidence**: Logic evidence at `tests/unit/card_acquisition/draw_pipeline_test.rs`; `cargo test -p server --test card_acquisition_draw_pipeline_test` passed 5/5 tests. `cargo check -p server` also passed.
**Code Review**: Skipped - lean review mode; local implementation review found no blocking GDD/ADR deviations.
