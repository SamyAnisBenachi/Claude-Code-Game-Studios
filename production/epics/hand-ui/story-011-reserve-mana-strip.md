# Story 011: Reserve Mana Split Strip — Per-Staged-Card Controls

> **Epic**: Hand UI
> **Status**: Ready
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

- [ ] **HU-25**: GIVEN a card with `cost = 5` is staged AND `player.reserve_mana = 3`, WHEN the player clicks `[ + ]` on its reserve strip:
  - First click: `reserve_amount` increments to 1; `[ + ]` remains Enabled (ceiling = min(5, 3) = 3; 1 < 3)
  - Second click: `reserve_amount` increments to 2; `[ + ]` remains Enabled (2 < 3)
  - Third click: `reserve_amount` increments to 3; `[ + ]` immediately enters `Disabled` state (3 == min(5, 3) = ceiling)
  - Fourth click on now-Disabled `[ + ]`: no state change; `reserve_amount` remains 3

- [ ] **HU-26**: GIVEN card A is staged with `reserve_amount = 2` AND `player.reserve_mana = 3`, WHEN card B (`cost ≥ 2`) is staged (default `reserve_amount = 0`) AND the player presses `[ + ]` on card B's reserve strip ONCE:
  - (a) Card B's `reserve_amount` increments to 1 (ceiling = `player.reserve_mana − sum_other = 3 − 2 = 1`)
  - (b) Card B's `[ + ]` button immediately enters `Disabled` state (1 == ceiling of 1)
  - (c) Card A's `reserve_amount` remains 2 (no auto-decrement of other staged cards occurs)

- [ ] **HU-27**: GIVEN a card with `cost = 0` is staged, WHEN the staged ghost renders, THEN the reserve strip entity for that card has `Visibility::Hidden` (no decision to make — free cards have no reserve split).

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

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 (staging core — reserve strip shown/hidden by staging state)
- Unlocks: Story 010 (pre-validation reads reserve_amount values set by this strip)
