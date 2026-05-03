# Story 006: PLACEMENT Drag — Highlight Sets & TargetUnit

> **Epic**: Hand UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-006` (TargetUnit hover), `TR-HU-002` (drag highlight sets)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Board cell highlights use `BoardCellHighlighted` marker components on board cell entities. TargetUnit hover uses `TargetUnitHover` marker component. `Res<BoardLayout>` from Board Rendering provides cursor-to-cell coordinate conversion. Hand UI does NOT add `BoardCellHighlighted` markers for TargetUnit drags — unit-targeting and cell-targeting are mutually exclusive per drag type.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Cursor-to-board-cell mapping uses `Res<BoardLayout>::cell_to_world()` inverse. Board cell entity queries use component markers. `GlobalTransform` for cursor-world-position lookup; do not read `Transform` directly — use `GlobalTransform` for rendered positions. `PickingBehavior` guard required if mouse-over detection uses bevy_ui picking (must be inside `#[cfg(feature = "ui_picking")]`).

**Control Manifest Rules (Presentation Layer)**:
- Required: `Res<BoardLayout>` for cursor-to-cell mapping — do not re-derive the formula independently.
- Required: `PickingBehavior` only inside `#[cfg(feature = "ui_picking")]`.
- Required: All drag systems `in_state(ClientState::InSession)`.
- Forbidden: `BoardCellHighlighted` markers on board cells during TargetUnit drags — unit-targeting only uses `TargetUnitHover` marker on the unit entity.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rule 6 (drag over board), scoped to this story:*

- [x] **HU-12**: GIVEN the player drag-starts a Minion card during PLACEMENT, WHEN the cursor enters the board area, THEN the highlighted-cell set (queryable via `BoardCellHighlighted` marker component on board cell entities) equals exactly:
  - (player's valid spawn cells for this round) minus (cells with prior-round board units) minus (cells already targeted by staged Minions in the local pending queue)
  - Sky Blue overlay rendering (`#3A8EDB` at 50%) is ADVISORY (lead sign-off).

- [x] **HU-12b**: GIVEN the player drag-starts a TargetObj card during PLACEMENT, WHEN the cursor enters the board area, THEN the highlighted-cell set equals exactly the surviving opponent objective cells (one per surviving lane; destroyed objectives produce no highlight for that lane).

- [x] **HU-12c**: GIVEN the player drag-starts a LaneWide (Field) card during PLACEMENT, WHEN the cursor enters the board area, THEN the highlighted-cell set equals all cells of all 5 lane columns (full board excluding objective cells).

- [x] **HU-12d**: GIVEN the player drag-starts a TargetUnit card during a round where ≥ 1 valid target unit exists on the board, WHEN the cursor hovers over a valid target unit entity, THEN:
  - The hovered unit entity receives a `TargetUnitHover` marker component
  - No `BoardCellHighlighted` marker components are added to any board cell entity
  - Prism White outline pulse rendering is ADVISORY.

- [x] **HU-20**: GIVEN the player drag-starts a TargetUnit card during a round where NO valid target units exist on the board, WHEN the drag sprite moves over the board, THEN:
  - (a) The `BoardCellHighlighted` marker set remains empty
  - (b) A `NoValidTargetsOverlay` marker entity exists with `Visibility::Visible`
  - (c) Dropping anywhere returns the card to its original fan slot via the normal invalid-drop path (Story 005 HU-14)
  - The full-dim overlay rendering is ADVISORY.

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule 6:*

1. **Highlight set computation** (runs every frame during active drag in `PresentationSet::StateSync`):
   - **Minion**: `valid_spawn_cells(player)` from `Res<BoardLayout>` (spawn range from `Res<PlayerEconomies>`) minus cells in `BoardState` (prior-round occupancy) minus cells already in local `PendingPlacements` with `PlayTarget::BoardCell` for Minion types. Add `BoardCellHighlighted` to matching cell entities; remove from all others.
   - **TargetObj**: Query all opponent objective cell entities with `ObjectiveAlive` marker; add `BoardCellHighlighted` to those. Destroyed objectives (no `ObjectiveAlive`) get no marker.
   - **LaneWide (Field)**: Add `BoardCellHighlighted` to all 5×8 = 40 non-objective board cell entities.
   - **TargetUnit**: Add `TargetUnitHover` to the unit entity under cursor; add NO `BoardCellHighlighted` to cells. If no valid unit exists, add `NoValidTargetsOverlay` visibility.

2. **Highlight cleanup**: On drag end (valid drop, invalid drop, or drag cancel), remove ALL `BoardCellHighlighted` markers and ALL `TargetUnitHover` markers from all entities. Remove `NoValidTargetsOverlay` visibility.

3. **Staged Minion exclusion** (HU-12): When computing valid cells for a new Minion drag, the local `PendingPlacements` vec is authoritative for excluding already-targeted cells — not the server's `BoardState` (which only knows prior-round units). This is a client-side exclusion for UX correctness.

4. **TargetUnit no-targets path** (HU-20): If card type is TargetUnit and `valid_target_units().is_empty()`, skip all highlight logic and set `NoValidTargetsOverlay` to Visible immediately on drag-start. Any drop fires the HU-14 invalid-drop cancel path.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Core staging on valid drop (this story computes highlights; Story 005 handles the drop)
- [Story 007]: Instant card plate highlight (`FanPlateHighlighted`) — different highlight surface
- [Story 008]: Un-staging gestures

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-12**: Minion highlight set (set arithmetic)
  - Given: PLACEMENT; player spawn cells = [lane1:cell1, lane1:cell2]; prior-round occupied = [lane1:cell1]; staged Minion pending at lane1:cell2
  - When: Drag-start a new Minion card; cursor enters board
  - Then: `BoardCellHighlighted` marker set is empty (both spawn cells are excluded: cell1 by occupancy, cell2 by staged pending)
  - Edge cases: No prior-round units, no staged → all spawn cells highlighted; spawn range of [1..3] per player direction

- **HU-12b**: TargetObj highlights surviving objectives only
  - Given: PLACEMENT; opponent has objectives in lanes 1–5; lane 3 objective is destroyed (no `ObjectiveAlive` marker)
  - When: Drag-start TargetObj card
  - Then: `BoardCellHighlighted` markers on objective cells for lanes 1,2,4,5 only; lane 3 objective cell has no marker
  - Edge cases: All opponent objectives destroyed → no highlighted cells → HU-20-like no-targets overlay (if TargetObj with no targets — edge case, confirm designer intent; for now: empty highlight set, card cannot be placed)

- **HU-12c**: LaneWide highlights all board columns
  - Given: PLACEMENT; standard 5-lane 8-cell board
  - When: Drag-start LaneWide card
  - Then: Count of entities with `BoardCellHighlighted` == 40 (5 lanes × 8 cells; objective cells excluded per GDD Rule 6)
  - Edge cases: Lane with destroyed objective still gets full column highlighted (only the 5 objective cells at far end are excluded)

- **HU-12d**: TargetUnit hover — unit gets marker, cells do not
  - Given: PLACEMENT; 2 valid target units on board (entity A in lane 1, entity B in lane 3)
  - When: Drag-start TargetUnit card; cursor moves over entity A
  - Then: Entity A has `TargetUnitHover` marker; entity B has no `TargetUnitHover`; count of `BoardCellHighlighted` entities == 0
  - When: Cursor moves to entity B
  - Then: Entity B gains `TargetUnitHover`; entity A loses it (only one unit hovered at a time)
  - Edge cases: Cursor moves off all units → no `TargetUnitHover` on any entity

- **HU-20**: TargetUnit — no valid targets overlay
  - Given: PLACEMENT; 0 valid target units on board
  - When: Drag-start TargetUnit card
  - Then: `BoardCellHighlighted` count == 0; `NoValidTargetsOverlay` entity has `Visibility::Visible`
  - When: Simulate drop anywhere on board
  - Then: Card returns to `FanSlotState::Active`; no `GhostPlacementChanged` written; `NoValidTargetsOverlay` has `Visibility::Hidden`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/placement_drag_highlights_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Completion Notes

**Completed**: 2026-05-03
**Criteria**: 5/5 passing (HU-12, HU-12b, HU-12c, HU-12d, HU-20)
**Deviations**: None blocking. Sky Blue overlay rendering, Prism White outline pulse rendering, and full-dim overlay visual sign-off remain advisory presentation evidence.
**Test Evidence**: `cargo test -p client --test hand_ui_placement_drag_highlights_test` passed 5/5; `cargo check -p client` passed; `cargo fmt -p client -- --check` passed.
**Code Review**: Skipped - Lean mode.
**Sprint Status**: Not updated; no matching `HAND-UI-006` row exists in `production/sprint-status.yaml`.

## Dependencies

- Depends on: Story 005 (PLACEMENT entry + staging core — this story extends the drag behavior with highlight logic)
- Unlocks: Story 008 (un-staging uses TargetUnit/BoardCell targeting state)
