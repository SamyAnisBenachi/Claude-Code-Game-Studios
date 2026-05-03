# Story 007: PLACEMENT Instant Card Staging

> **Epic**: Hand UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Instant cards stage to the fan plate, not to a board cell. The fan plate is a bevy_ui `Node` element. `GhostPlacementChanged { target: Some(Instant), card_id }` is sent for protocol completeness; Board Rendering ignores it (no board ghost for Instant). The fan plate highlight (`FanPlateHighlighted`) is the only valid drop zone for Instant cards.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `FanPlateHighlighted` is a marker component on the fan plate `Node` entity. Drop-zone detection is bevy_ui — cursor-overlap with the plate entity's computed bounding rect. `PickingBehavior` guard required. Plate background brightens on drag-over (VA-7 visual spec) — rendering is ADVISORY; the marker component is BLOCKING.

**Control Manifest Rules (Presentation Layer)**:
- Required: Fan drag-sprite is bevy_ui `Node` — preserves z-order above board content.
- Required: `GhostPlacementChanged` is Bevy-internal `#[derive(Message)]` — register via `app.add_message::<GhostPlacementChanged>()`.
- Required: `PickingBehavior` only inside `#[cfg(feature = "ui_picking")]`.
- Forbidden: Board cell highlight (`BoardCellHighlighted`) during Instant card drag.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rules 6 (step 2 Instant subcase) and 7, scoped to this story:*

- [x] **HU-18**: GIVEN the player drag-starts an Instant card during PLACEMENT, WHEN the drag sprite becomes visible, THEN:
  - (a) The fan plate entity receives a `FanPlateHighlighted` marker component
  - (b) The `BoardCellHighlighted` marker set on all board cell entities is empty (Instant cards do NOT highlight board cells)
  - Prism White border pulse rendering (`#EEF4FF` 3px at 60% opacity, 0.5Hz) is ADVISORY.

- [x] **HU-19**: GIVEN the player drops an Instant card on the highlighted fan plate zone, WHEN the drop fires, THEN:
  - (a) The card stages with `PlayTarget::Instant` in the local pending queue
  - (b) `GhostPlacementChanged { target: Some(PlayTarget::Instant), card_id: Some(card_id) }` Bevy-internal message is written
  - (c) The Submit count increments by 1 (Submit text updates to "Submit (N cards)")
  - The 80ms Arcane Gold flash rendering is ADVISORY.

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule 7:*

1. **Drag-start for Instant card**: On drag-start detection for a card with `CardType::Instant`:
   - Show drag sprite (hidden → visible, scaled to 1.10×)
   - Add `FanPlateHighlighted` to the fan plate entity
   - Do NOT add any `BoardCellHighlighted` markers to board cell entities

2. **Drop-zone detection**: The fan plate drop zone is the bevy_ui `Node` element serving as the fan background panel. A drop "on the plate" is detected by cursor position falling within the plate entity's screen-space AABB at mouse-up time. The plate's screen rect must be accessible for this calculation.

3. **Valid drop**: Cursor position within plate AABB at mouse-up → stage the card:
   - Add to local `PendingPlacements` as `PlayTarget::Instant`
   - Write `GhostPlacementChanged { target: Some(PlayTarget::Instant), card_id: Some(card_id) }`
   - Fan slot enters `FanSlotState::Ghost` (from Story 005 core path)
   - Submit count increments
   - Reserve strip becomes Visible (from Story 011; cost=0 Instant cards will immediately hide it per HU-27)
   - Remove `FanPlateHighlighted`

4. **Invalid drop** (outside plate): Run the Story 005 HU-14 invalid-drop path — drag sprite hidden, fan slot reverts to Active, no message written. Remove `FanPlateHighlighted`.

5. **Un-staging Instant cards**: Handled by Story 008 HU-21c — click on dimmed fan slot ghost. This story does NOT implement un-staging.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Core staging mechanics (GhostPlacementChanged, FanSlotState::Ghost, Submit count)
- [Story 008]: Instant card un-staging (HU-21c — click on dimmed fan slot)
- [Story 006]: Non-Instant drag highlight sets (BoardCellHighlighted for Minion/TargetObj/LaneWide)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-18**: Instant drag — plate highlighted, no cell highlights
  - Given: PLACEMENT; hand contains Instant card at slot 2
  - When: Simulate drag-start on slot 2 (card type = Instant); drag sprite becomes visible
  - Then: Fan plate entity has `FanPlateHighlighted` marker; query all entities with `BoardCellHighlighted` → count == 0
  - Edge cases: Drag-start on non-Instant card → `FanPlateHighlighted` NOT added (handled by Story 006)

- **HU-19**: Instant drop on plate → staged
  - Given: PLACEMENT; Instant card at slot 2 being dragged; fan plate has `FanPlateHighlighted`
  - When: Simulate mouse-up with cursor inside fan plate AABB
  - Then: Local `PendingPlacements` contains `{ card_id: slot2_id, target: PlayTarget::Instant }`; `GhostPlacementChanged { target: Some(Instant), card_id: Some(slot2_id) }` written to message bus; Submit text == `"Submit (1 cards)"`
  - Edge cases: Drop outside plate → fan slot reverts to Active; no `GhostPlacementChanged`; `FanPlateHighlighted` removed

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/placement_instant_staging_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Completion Notes

**Completed**: 2026-05-03
**Verdict**: COMPLETE
**Criteria**: 2/2 passing; HU-18 and HU-19 are covered by `tests/unit/hand-ui/placement_instant_staging_test.rs`.
**Test Evidence**: `cargo test -p client --test hand_ui_placement_instant_staging_test` passed 3/3 locally. Main integration at `d3a16d1` also passed `cargo fmt -p client -- --check`, `cargo test -p client --test hand_ui_placement_submit_core_test`, `cargo test -p client --test hand_ui_placement_drag_highlights_test`, `cargo check -p client`, and `cargo check -p client --features ui_picking`.
**Verification**: Instant placement drags show the drag sprite, clear all `BoardCellHighlighted` markers, add `FanPlateHighlighted` to the fan plate, stage valid plate drops as `PlayTarget::Instant`, write `GhostPlacementChanged { target: Some(PlayTarget::Instant), card_id: Some(card_id) }`, update Submit text to `"Submit (1 cards)"`, and restore invalid outside-plate drops without ghost messages.
**Deviations**: None blocking.
**Code Review**: Skipped - Lean mode.
**Tech Debt**: None logged.
**Sprint Status**: Unchanged per user instruction; no matching `HAND-UI-007` row exists in `production/sprint-status.yaml`.
**Next Recommended**: Hand UI Story 008 PLACEMENT Un-staging (`production/epics/hand-ui/story-008-placement-unstaging.md`) after readiness check.

---

## Dependencies

- Depends on: Story 005 (core stage/unstage path that this story extends for Instant card type)
- Unlocks: Story 008 (un-staging includes Instant fan slot click HU-21c)
