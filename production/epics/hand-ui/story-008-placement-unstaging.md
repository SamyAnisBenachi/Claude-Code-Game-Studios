# Story 008: PLACEMENT Un-Staging — Board Ghosts & Instant Fan Slot

> **Epic**: Hand UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Un-staging is triggered by three gestures. For board ghosts (BoardCell, TargetUnit, TargetObj, LaneWide), Board Rendering owns the ghost entity and emits `GhostClickedEvent` or `GhostDragStartEvent` (both Bevy-internal messages). Hand UI reads these messages and runs the un-stage operation. For Instant cards, the dimmed fan slot itself is the un-stage surface — Hand UI detects the click directly.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `GhostClickedEvent { card_id }` and `GhostDragStartEvent { card_id }` are Bevy-internal `#[derive(Message)]` — NOT Lightyear messages. Register via `app.add_message::<T>()`. For HU-21b, Hand UI takes drag ownership from the point `GhostDragStartEvent` is received; it then tracks cursor until mouse-up. The fan zone boundary must be accessible as a resource or computed from the fan plate entity's AABB — implementer must expose `Res<FanZoneBounds>` (or equivalent screen-space rect) to enable testability.

**Control Manifest Rules (Presentation Layer)**:
- Required: `GhostClickedEvent` and `GhostDragStartEvent` are Bevy-internal `#[derive(Message)]` — use `MessageReader<T>`.
- Required: `GhostPlacementChanged { target: None, card_id: Some(card_id) }` written to clear Board Rendering's ghost on un-stage.
- Required: Un-stage is an atomic operation: remove from pending queue + write GhostPlacementChanged + set FanSlotState::Active + decrement count.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rule 8, scoped to this story:*

- [ ] **HU-21**: GIVEN a card is staged with a `BoardCell`, `TargetUnit`, `TargetObj`, or `LaneWide` target (board ghost active), WHEN Board Rendering writes a `GhostClickedEvent { card_id }` for that card's id, THEN Hand UI:
  - (a) Removes the card from the local pending queue
  - (b) Writes `GhostPlacementChanged { target: None, card_id: Some(card_id) }` to the Bevy message bus
  - (c) The fan slot for that card enters `FanSlotState::Active`
  - (d) The Submit count decrements by 1 (Submit text updates accordingly)

- [ ] **HU-21b**: GIVEN a card is staged with a board target (BoardCell, TargetUnit, TargetObj, or LaneWide), WHEN Board Rendering writes a `GhostDragStartEvent { card_id }` (player mouse-down on the board ghost) AND the player subsequently releases the mouse inside the hand fan zone (as determined by `Res<FanZoneBounds>` or equivalent), THEN Hand UI runs the same un-stage operation as HU-21. Submit count decrements.
  - *Implementer note: Expose `Res<FanZoneBounds>` (a screen-space rect) to allow the test to inject a valid release position.*
  - *If released outside the fan zone: the ghost returns to its board position — no un-stage.*

- [ ] **HU-21c**: GIVEN a card is staged with `PlayTarget::Instant` (no board ghost; only a dimmed fan slot ghost), WHEN the player clicks the dimmed fan slot for that card, THEN Hand UI runs the same un-stage operation as HU-21. Submit count decrements.

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule 8:*

1. **Drain `GhostClickedEvent`** in `PresentationSet::MessageDrain`. For each event, run the un-stage operation: remove the `card_id` from `PendingPlacements`, write `GhostPlacementChanged { target: None, card_id: Some(card_id) }`, set fan slot to `FanSlotState::Active`, decrement submit count.

2. **Drain `GhostDragStartEvent`** in `PresentationSet::MessageDrain`. Hand UI takes drag ownership: track cursor position per frame. On mouse-up, check if cursor is inside `Res<FanZoneBounds>` AABB:
   - Inside fan zone → run un-stage operation (same as HU-21)
   - Outside fan zone → ghost returns to board position (no state change for Hand UI pending queue; write `GhostPlacementChanged` with the ORIGINAL target to restore Board Rendering's ghost, or rely on Board Rendering's own state management)

3. **Instant un-stage** (HU-21c): Fan slots in `FanSlotState::Ghost` with `PlayTarget::Instant` must be click-detectable. On click on a ghost-state Instant fan slot, run the un-stage operation.

4. **Un-stage is atomic**: All four actions (queue removal, GhostPlacementChanged None, FanSlotState::Active, count decrement) happen within the same `App::update()` tick. No partial un-stage state.

5. **Reserve strip on un-stage**: When a card is un-staged, its reserve strip entity becomes `Visibility::Hidden` (inverse of the staging action in Story 005 HU-13(d)).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Core staging (GhostPlacementChanged Some, FanSlotState::Ghost, Submit count increment)
- [Story 007]: Instant card staging to fan plate

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-21**: Board ghost click → un-stage
  - Given: PLACEMENT; card C staged to `BoardCell { lane:2, cell:4 }`; pending queue has 1 entry; Submit text == `"Submit (1 cards)"`
  - When: Inject `GhostClickedEvent { card_id: C }` to Bevy message bus; `App::update()` runs
  - Then: `PendingPlacements` is empty; `GhostPlacementChanged { target: None, card_id: Some(C) }` written; fan slot for C has `FanSlotState::Active`; Submit text == `"Submit (0 cards)"`; reserve strip for C has `Visibility::Hidden`
  - Edge cases: `GhostClickedEvent` with unknown card_id → silently ignored (no panic, no state change)

- **HU-21b**: Board ghost drag-back → un-stage on fan-zone release
  - Given: PLACEMENT; card C staged to `BoardCell`; `Res<FanZoneBounds>` = Rect { x_min:100, x_max:900, y_min:600, y_max:680 }
  - When: Inject `GhostDragStartEvent { card_id: C }`; simulate mouse-up at position (450, 640) (inside fan zone); `App::update()` runs
  - Then: Same un-stage assertions as HU-21
  - When: (separate test) Inject `GhostDragStartEvent { card_id: C }`; simulate mouse-up at position (450, 200) (outside fan zone)
  - Then: `PendingPlacements` still has entry for C; fan slot for C still has `FanSlotState::Ghost` (no un-stage)
  - Note: Implementer must expose `Res<FanZoneBounds>` for this test to inject positions.

- **HU-21c**: Instant fan slot click → un-stage
  - Given: PLACEMENT; Instant card D staged (`PlayTarget::Instant`); fan slot for D has `FanSlotState::Ghost`; pending queue has 1 entry
  - When: Simulate click on fan slot D (ghost state, Instant type)
  - Then: Same un-stage assertions as HU-21 — `PendingPlacements` empty, `GhostPlacementChanged { target: None, card_id: Some(D) }` written, fan slot Active, Submit text decremented

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/hand-ui/placement_unstaging_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 (staging core — un-stage reverses staging), Story 006 (board targets staged here), Story 007 (Instant staging — HU-21c un-stages Instants)
- Unlocks: None directly (final piece of the staging state machine)
