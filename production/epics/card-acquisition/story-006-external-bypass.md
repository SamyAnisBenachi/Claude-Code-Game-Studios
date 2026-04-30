# Story 006: External Bypasses — PlayerHands Shared API

> **Epic**: Card Acquisition
> **Status**: Ready
> **Layer**: Feature (M2)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/card-acquisition.md`
**Requirement**: `TR-CA-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-015: Card Acquisition Shop State Machine Architecture
**ADR Decision Summary**: `PlayerHands` is a shared Resource. Cards added by Prism System (Lane 3 draw) or Objective System (free card pick) bypass `card_acquisition_tick_system` entirely — they write `ResMut<PlayerHands>` directly. These writes happen exclusively during RESOLUTION phase. `card_acquisition_tick_system` holds `ResMut<PlayerHands>` only during DRAFT phase. RSM phase exclusion prevents concurrent write conflicts; no scheduling guard is strictly required, but `CardAcquisitionSet::Tick.before(PrismSet::Tick)` is added as a compile-time invariant when both sets exist.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `ResMut<PlayerHands>` is safe for multiple writers because DRAFT and RESOLUTION are mutually exclusive RSM phases — confirmed by Bevy schedule graph inspection
- `CardAcquisitionSet::Tick.before(PrismSet::Tick)` ordering must be configured in plugin setup even if current phase exclusion prevents same-frame access — guards against future scheduler drift
- Bevy 0.18: if two systems with overlapping `ResMut<T>` are scheduled in the same system set without ordering, Bevy will panic at schedule build time — the explicit `.before()` ordering prevents this

**Control Manifest Rules (Feature layer — from ADR-015):**
- Required: `PlayerHands::push_card()` is the sole write interface — no direct `HashMap` manipulation outside this method
- Required: External writers (Prism, Objective) call `hand_len()` before `push_card()` to enforce the 10-card cap — CA does not re-check for bypasses; each bypass system owns its own cap check
- Forbidden: Card Acquisition system in the call chain for Prism Lane 3 or Objective free-card-pick draws
- Guardrail: `CardAcquisitionSet::Tick.before(PrismSet::Tick)` and `.before(ObjectiveSet::Tick)` must be configured when those sets are added

---

## Acceptance Criteria

*From GDD `design/gdd/card-acquisition.md`, scoped to this story:*

- [ ] **CA17** — GIVEN a Lane 3 prism is collected during RESOLUTION, WHEN the Prism System processes the reward, THEN `hand.len() == hand_len_before + 1`, `gold == gold_before`, and no `C2SPurchaseCard` event was written to the message queue.

---

## Implementation Notes

*Derived from ADR-015 Decision:*

This story has two implementation tasks:

**Task A — Verify `PlayerHands::push_card()` is callable from outside Card Acquisition.** The `PlayerHands` type defined in Story 001 must be `pub` with a `pub fn push_card()` method. Confirm it is usable from `server/feature/prism/` and `server/feature/objective/` without a module-boundary compile error.

**Task B — Integration test simulating Prism bypass.** Set up a `World::new()` with:
- `PlayerHands` inserted (Story 001)
- `ShopStates` inserted in `Inactive` phase (RESOLUTION — CA is dormant)
- `PlayerEconomies` for the player
- Simulate a Lane 3 prism event by calling `hands.push_card(player_id, card_id)` directly (or via the Prism System's system) without routing through `card_acquisition_tick_system`

Assert:
- `hand_len(player) == prior_len + 1`
- Economy gold unchanged
- No `C2SPurchaseCard` message in the Lightyear send buffer

**CA17 test note**: The Prism System (Story 003 of the prism-system epic) will own the full `resolve_prism_draws` system. For this story, the integration test only needs to confirm that `PlayerHands` is correctly accessible from outside CA — it does not need a full Prism System implementation. A direct `world.resource_mut::<PlayerHands>().push_card(...)` call is sufficient to verify the boundary contract.

**Scheduling guard** (add in CA plugin when Prism plugin exists):
```rust
app.configure_sets(Update,
    CardAcquisitionSet::Tick.before(PrismSet::Tick)
);
app.configure_sets(Update,
    CardAcquisitionSet::Tick.before(ObjectiveSet::Tick)
);
```

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Prism System epic (prism-system stories): the actual `resolve_prism_draws` system that calls `push_card` — this story only validates the shared API boundary
- Objective System epic: the free-card-pick path — same boundary, verified when that epic's stories run
- Story 001: The `PlayerHands` struct and `push_card` method definition

---

## QA Test Cases

- **CA17**: Prism bypass writes hand without gold change or CA involvement
  - Given: `PlayerHands` has N cards for player; economy.gold = G; `ShopStates` in `Inactive` phase
  - When: `hands.push_card(player_id, card_id)` called directly (simulating Prism System)
  - Then: `hand_len(player) == N + 1`; economy.gold == G (unchanged); no `C2SPurchaseCard` in message queue; `ShopStates` unchanged
  - Edge cases: hand at 9 (adds to 10); hand at 10 (bypass system must check cap — CA17 only tests the N<10 case since Prism System owns the cap check for bypasses)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/card_acquisition/external_bypass_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (`state-scaffold`) must be Done — `PlayerHands::push_card()` must be defined and public
- Unlocks: Prism System epic stories (prism-system) and Objective System stories that write to `PlayerHands` — they can begin once this story confirms the shared API contract
