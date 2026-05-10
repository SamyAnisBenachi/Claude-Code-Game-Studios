# Story 011: Reserve Mana Split Strip — Per-Staged-Card Controls

> **Epic**: Hand UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: One reserve strip per staged card. Strip attaches to the fan ghost slot (anchored 8px above the dimmed slot). Strip appears when card stages; disappears when card un-stages. No slider or drag — single-click step controls only. The ceiling formula ensures the pool ceiling for card B accounts for all `reserve_amount` already committed by other staged cards. No auto-decrement of other cards.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Strip is a bevy_ui `Node` subtree (one per fan slot). `[ − ]` and `[ + ]` buttons use `Interaction` component for click detection. Disabled buttons use a `Disabled` marker (or custom component) to suppress click processing. All strip positioning via `Node::left`/`Node::bottom` absolute values — not flexbox.

**Control Manifest Rules (Presentation Layer)**:
- Required: UI always bevy_ui `Node` — no world-space sprites for strip elements.
- Required: `PickingBehavior` inside `#[cfg(feature = "ui_picking")]` guard.
- Required: All strip systems `in_state(ClientState::InSession)`.
- Forbidden: `NodeBundle` — use `Node { .. }` Required Components.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rule 13, scoped to this story:*

- [x] **HU-25**: GIVEN a card with `cost = 5` is staged AND `player.reserve_mana = 3`, WHEN the player clicks `[ + ]` on its reserve strip:
  - First click: `reserve_amount` increments to 1; `[ + ]` remains Enabled (ceiling = min(5, 3) = 3; 1 < 3)
  - Second click: `reserve_amount` increments to 2; `[ + ]` remains Enabled (2 < 3)
  - Third click: `reserve_amount` increments to 3; `[ + ]` immediately enters `Disabled` state (3 == min(5, 3) = ceiling)
  - Fourth click on now-Disabled `[ + ]`: no state change; `reserve_amount` remains 3

- [x] **HU-26**: GIVEN card A is staged with `reserve_amount = 2` AND `player.reserve_mana = 3`, WHEN card B (`cost ≥ 2`) is staged (default `reserve_amount = 0`) AND the player presses `[ + ]` on card B's reserve strip ONCE:
  - (a) Card B's `reserve_amount` increments to 1 (ceiling = `player.reserve_mana − sum_other = 3 − 2 = 1`)
  - (b) Card B's `[ + ]` button immediately enters `Disabled` state (1 == ceiling of 1)
  - (c) Card A's `reserve_amount` remains 2 (no auto-decrement of other staged cards occurs)

- [x] **HU-27**: GIVEN a card with `cost = 0` is staged, WHEN the staged ghost renders, THEN the reserve strip entity for that card has `Visibility::Hidden` (no decision to make — free cards have no reserve split).

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule 13:*

1. **Ceiling formula**: For each staged card B, the `[ + ]` button ceiling is `min(card_B.cost, player.reserve_mana − sum(other_staged.reserve_amount))`. This must be recomputed on every `[ + ]` press and also when any other card's `reserve_amount` changes. Use `PresentationSet::StateSync` for re-evaluation each frame.

2. **Disable logic**: After each `[ + ]` press, check if `reserve_amount == ceiling`. If so, set the `[ + ]` button entity to `Disabled` immediately in the same system invocation. Disabled `[ + ]` processes no clicks.

3. **`[ − ]` clamp**: `reserve_amount` cannot go below 0. After a `[ − ]` press, if `reserve_amount > 0`, decrement. If `reserve_amount == 0`, `[ − ]` does nothing (but does not disable — the button is always active; it just has no effect at 0).

4. **`player.reserve_mana == 0`**: All `[ + ]` buttons are Disabled. All strips display `"0 / cost"`. No clicks have effect on `[ + ]`.

5. **`cost == 0`** (HU-27): The strip entity for that fan slot is `Visibility::Hidden`. Do not process any click events for it.

6. **Strip visibility**: Strip becomes `Visibility::Visible` when card stages (from Story 005 HU-13(d)), `Visibility::Hidden` when card un-stages (from Story 008). This story implements the strip's internal logic (HU-25/26/27), not its visibility toggle lifecycle.

7. **Pre-submit validation interaction**: The Rule 10 pre-validation (Story 010) sums all `reserve_amount` values; the strip `[ + ]` disable logic proactively prevents overdraw in most cases.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Strip visibility toggle on stage (Visible) / un-stage (Hidden)
- [Story 010]: Pre-validation sums `reserve_amount` as a final gate

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-25**: Reserve strip ceiling and disable
  - Given: Card with `cost=5` staged; `player.reserve_mana=3`; strip `reserve_amount=0`; `[ + ]` Enabled
  - When: Click `[ + ]` → assert `reserve_amount=1`; `[ + ]` Enabled; click `[ + ]` → `reserve_amount=2`; Enabled; click `[ + ]` → `reserve_amount=3`; `[ + ]` Disabled
  - Then: Additional click on Disabled `[ + ]` → `reserve_amount` still 3; no state change
  - Edge cases: `player.reserve_mana=5, cost=3` → ceiling = min(3,5) = 3; Disabled at 3; `player.reserve_mana=0` → `[ + ]` Disabled immediately on stage

- **HU-26**: Multi-card reserve ceiling — no auto-decrement
  - Given: Card A staged with `reserve_amount=2`; `player.reserve_mana=3`; card B staged with `reserve_amount=0`; ceiling for B = 3−2 = 1
  - When: Click `[ + ]` on card B → `reserve_amount=1`; `[ + ]` for B now Disabled (1 == ceiling)
  - Then: Card A `reserve_amount` still == 2 (no change)
  - Edge cases: Press `[ − ]` on A → A.reserve_amount=1; B's ceiling recalculates to 3−1=2; B's `[ + ]` now Enabled; press `[ + ]` on B → B.reserve_amount=2

- **HU-27**: Free card strip hidden
  - Given: Card with `cost=0` staged; fan slot in `FanSlotState::Ghost`
  - When: Strip render system runs
  - Then: Reserve strip entity for that fan slot has `Visibility::Hidden`
  - Edge cases: `cost=0` card un-staged and re-staged → strip still `Visibility::Hidden`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/reserve_mana_strip_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 005 (staging core — reserve strip shown/hidden by staging state)
- Unlocks: Story 010 (pre-validation reads reserve_amount values set by this strip)

## Completion Notes

**Completed**: 2026-05-04
**Criteria**: 3/3 passing (HU-25, HU-26, HU-27)
**Deviations**:
- Advisory: `TR-HU-004` in `docs/architecture/tr-registry.yaml` does not currently describe the reserve strip behavior covered by HU-25/HU-26/HU-27. Current `design/gdd/hand-ui.md` Rule 13 is the verified behavior source.
- Advisory: VA-9 specifies a 96 px strip width; the implementation uses 104 px. The logic criteria pass; visual sizing can be reconciled in UI polish if needed.
**Test Evidence**: Logic: `tests/unit/hand-ui/reserve_mana_strip_test.rs`; `cargo test -p client --test hand_ui_reserve_mana_strip_test` passed 3/3.
**Code Review**: Skipped - lean mode.

### Finding B v2 — Verdict 2 Reconciliation (2026-05-10)

PROMPT 623's read-only diagnostic of the runtime "RESERVE 0 CURRENT 0" leak at PLACEMENT entry traced the symptom to a child-visibility regression in `client/src/ui/hand/mod.rs`:

- `spawn_reserve_strip` spawned `ReserveStripValueText` with `Visibility::Visible` (line 2649).
- `spawn_reserve_strip_button` spawned the `[ - ]` and `[ + ]` `ReserveStripButton` entities with `Visibility::Visible` (line 2683).

The strip parent (`ReserveStripForFanSlot`) was correctly `Visibility::Hidden` at spawn (line 2629), and HU-13(d) (Story 005) toggled it to `Visible`/`Hidden` on stage/un-stage. But Bevy's visibility model treats `Visibility::Visible` on a child as an explicit override of the parent's computed state — children with `Visible` ignore a `Hidden` parent. The fan-slot chrome convention (lines 2447, 2455, 2463, 2471, 2479, 2487, 2495) and the board-rendering convention (`client/src/presentation/board_rendering.rs` lines 2138, 2155, 2274, 2305) both use `Visibility::Inherited` for children that must follow a parent gating decision.

Effect on this story's ACs:
- **AC-27 (HU-27)** "free card strip hidden": the strip parent was `Hidden` as designed, but the value text and `[ - ] / [ + ]` buttons still rendered, leaking the strip onto the fan even for `cost == 0` staged cards.
- **AC-13(d)** (Story 005 contract relied on here) "un-stage hides the strip": un-stage set the strip parent to `Hidden`, but the value text and buttons remained painted because their `Visible` overrode inheritance.

The original `reserve_mana_strip_test.rs` exercised only the in-strip stepper logic on a freshly constructed parent and never asserted child propagation, so the regression slipped past the AC-27 contract.

**Verdict 2 repair (this branch — `work/finding-b-v2-reserve-strip-child-visibility`):**

- `client/src/ui/hand/mod.rs:2649` — `Visibility::Visible` → `Visibility::Inherited` on `ReserveStripValueText`.
- `client/src/ui/hand/mod.rs:2683` — `Visibility::Visible` → `Visibility::Inherited` on `ReserveStripButton` (both `-` and `+` via the shared helper).
- New regression test `tests/integration/hand-ui/placement_entry_post_acquisition_test.rs` drives 3 `HandUiCardAcquiredReceived` events in `DraftInitial`, transitions to `Placement`, and asserts: every `ReserveStripForFanSlot` parent is `Hidden`; every `ReserveStripValueText` child is `Visibility::Inherited`; every `ReserveStripButton` child is `Visibility::Inherited`.

AC-27 and AC-13(d) status is re-affirmed `[x]` — the AC contract is unchanged; the propagation path it relied on is now enforced by an integration test rather than an implicit spawn-time convention. The branch HEAD will be assigned by the orchestrator on cherry-pick into `main`.
