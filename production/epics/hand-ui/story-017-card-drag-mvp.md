# Story 017: HU-Card-Drag MVP — Drag Producers (Press / Move / Release)

> **Epic**: Hand UI
> **Status**: In Progress
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-11

## Context

**GDD**: `design/ux/hand-ui.md` (State machine — Dragging card / Valid board target hover / Staged board card / Un-staging, L210–230)

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Hand UI owns the placement drag lifecycle from input to ghost. Drag-start / cursor-move / drag-end messages are produced by the Hand UI from `bevy_picking` pointer events. Downstream consumers (`handle_placement_drag_started_system`, `handle_placement_cursor_moved_system`, `handle_placement_drag_ended_system`) already exist and consume those messages; only the producers were missing.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Producer surface uses `On<Pointer<Press>>` / `On<Pointer<Move>>` / `On<Pointer<Release>>` global observers added via `App::add_observer`. Filtering by `FanSlotIndex` via `Query` inside the observer body is the project-idiomatic pattern (see `client/src/presentation/board_rendering.rs:891–1128` for the ghost equivalent). `HandDragSprite` is a `ChildOf(fan_root)` UI node with `Visibility::Hidden` at spawn; the drag-start consumer already flips visibility — this story only needs to keep `Node.left` / `Node.top` synced to `active_drag.cursor_world_position` every frame while the drag is active.

**Control Manifest Rules (Presentation Layer)**:
- Required: All drag producer observers / systems gated on `HandUiMode::Staging` (or `active_drag.is_active()` for follow/move/end).
- Required: Pointer button checks must require `PointerButton::Primary` before writing a drag start.
- Required: `liv-bevy-018` skill applies to all `On<Pointer<...>>` observer signatures.
- Forbidden: Synthesising bespoke `WindowEvent` / cursor-position polling; rely on `bevy_picking` events for input.
- Forbidden: Direct mutation of `HandDragSprite` visibility from this story — that path is owned by the existing `handle_placement_drag_started_system` / `handle_placement_drag_ended_system`.

---

## Acceptance Criteria

Scoped to the producer surface (the FEATURE-GAP proved by PROMPT 683 Phase 2 diagnostic — `HandDragSprite` exists, message consumers wired, message producers missing entirely in gameplay):

- [ ] **HU-DRAG-01**: GIVEN `HandUiMode::Staging` (PLACEMENT entered), WHEN `Pointer<Press>` with `PointerButton::Primary` fires on a `FanSlotIndex` entity, THEN exactly one `HandUiPlacementDragStarted { card: <slot entity>, owner_id }` is emitted that same tick.

- [ ] **HU-DRAG-02**: GIVEN an active placement drag (`ActivePlacementDrag::is_active()`), WHEN `Pointer<Move>` fires anywhere (entity-agnostic), THEN:
  - Exactly one `HandUiPlacementCursorMoved { world_position: Some(<screen_pos>) }` is emitted per move event
  - The follow system writes `HandDragSprite`'s `Node.left` / `Node.top` to that screen position the same tick the cursor message is consumed

- [ ] **HU-DRAG-03**: GIVEN an active placement drag, WHEN `Pointer<Release>` with `PointerButton::Primary` fires anywhere, THEN exactly one `HandUiPlacementDragEnded` is emitted and the existing `handle_placement_drag_ended_system` runs its cleanup (Instant drop resolution / `active_drag.clear()`).

- [ ] **HU-DRAG-04**: GIVEN an active placement drag, WHEN the per-frame follow system runs (`HandUiSystemSet::StateSync`), THEN `HandDragSprite`'s `Visibility` is `Visible` (flipped by existing drag-start consumer) AND its `Node.left` / `Node.top` match the latest `active_drag.cursor_world_position` value. WHEN `HandUiPlacementDragEnded` consumed, THEN `Visibility` returns to `Hidden` (existing consumer path).

---

## Implementation Notes

1. **Press producer**: Global observer `on_fan_slot_press` registered with `app.add_observer(on_fan_slot_press)`. Body queries `Query<&FanSlotIndex>` on `trigger.entity`; bails on non-fan-slot, on non-primary button, on `HandUiMode != Staging`. Looks up the local owner id from a resource (`PlacementBoardView::local_player_id`) and writes `HandUiPlacementDragStarted { card: trigger.entity, owner_id }`.

2. **Move producer**: Global observer `on_pointer_move_during_drag`. Body checks `active_drag.is_active()`; bails otherwise. Writes `HandUiPlacementCursorMoved { world_position: Some(trigger.pointer_location.position) }`. The `pointer_location.position` is a screen-space `Vec2` — consistent with `cursor_over_fan_plate`'s viewport-space contract elsewhere in `mod.rs`.

3. **Release producer**: Global observer `on_pointer_release_during_drag`. Body checks `active_drag.is_active()` and `trigger.button == PointerButton::Primary`; bails otherwise. Writes `HandUiPlacementDragEnded`.

4. **Follow system** `sync_hand_drag_sprite_position_system`: Runs in `HandUiSystemSet::StateSync`. Reads `Res<ActivePlacementDrag>`; if `is_active()` and `cursor_world_position.is_some()`, writes the `Node.left` / `Node.top` of the entity with `HandDragSprite`. No-ops otherwise.

5. **Idempotence**: Existing consumers (`handle_placement_drag_started_system`, `handle_placement_cursor_moved_system`, `handle_placement_drag_ended_system`) already flip visibility, set `active_drag.cursor_world_position`, and route Instant drops. This story only emits the producer messages; it does NOT re-implement that logic.

---

## Out of Scope

- Drop completion logic for BoardCell / TargetUnit / TargetObj / LaneWide targets — landed by Story 005 / 006 and a follow-up PROMPT 697.
- Snap-back animation on invalid drop — covered by existing tween code paths.
- Keyboard equivalent of drag (focus + Enter + arrow keys) — covered by accessibility stories 014 / 015.
- `Pointer<Drag>` / `Pointer<DragStart>` / `Pointer<DragEnd>` events — this story uses Press / Move / Release per the PROMPT 696 specification.

---

## QA Test Cases

- **HU-DRAG-01**: Press emits drag-started
  - Given: `HandUiMode::Staging`, slot 0 has a `HandSlotCard`
  - When: `Pointer<Press>` (primary button) on slot 0
  - Then: One `HandUiPlacementDragStarted` with `card == slot_entity`
  - Edge: Press on non-Primary button → no emit; Press while `HandUiMode != Staging` → no emit

- **HU-DRAG-02**: Move during active drag emits cursor-moved + updates sprite Node
  - Given: An active drag (post drag-started)
  - When: `Pointer<Move>` event at `Vec2::new(620.0, 400.0)`
  - Then: One `HandUiPlacementCursorMoved` with that position; the `HandDragSprite` entity has `Node.left = Val::Px(620.0)` and `Node.top = Val::Px(400.0)` after the same `app.update()`
  - Edge: Move without active drag → no emit

- **HU-DRAG-03**: Release ends drag
  - Given: Active drag
  - When: `Pointer<Release>` (primary)
  - Then: One `HandUiPlacementDragEnded`; `active_drag.is_active()` becomes false after consumer runs

- **HU-DRAG-04**: Sprite follows cursor through full sequence
  - Given: Hand contains two acquired cards, phase set to Placement (Staging), viewport 1280×720
  - When: Press on slot 0 → Move(300,400) → Move(620,500) → Release(over board cell)
  - Then: `HandDragSprite` visibility goes Hidden → Visible → Hidden; `Node.left/top` track each Move event; final `HandUiPlacementDragEnded` is emitted

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/hand-ui/hand_ui_drag_to_board_cell_test.rs` — must exist and pass.

**Status**: [ ] Created and passing

---

## Dependencies

- Depends on: Story 005 (PLACEMENT entry + staging core — existing drag-start / move / end consumers live there)
- Depends on: Story 006 (drag highlights — consumes the messages this story produces)
- Unlocks: PROMPT 697 (drop completion: Pointer<Release> over board cell → `HandUiPlacementDropResolved { target: Some(PlayTarget::BoardCell { … }) }`). PROMPT 697 will add the BoardCell branch to `handle_placement_drag_ended_system`; this story stops at the drag-ended emit, per Phase 3 scope guard.
