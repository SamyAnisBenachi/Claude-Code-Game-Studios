# Story 005: Placement-reveal parallelism + PLACEMENT 250ms budget + PlacementCancelAllAnimsRequested

> **Epic**: Card Animations
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-004`, `TR-CAN-005`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Mandatory simultaneous-start parallelism for placement reveal (all 5 lanes flip same frame); PLACEMENT hard cap 250 ms enforced at `Tween` construction time; `PlacementCancelAllAnimsRequested` emitted by Board Rendering on `S2CPhaseChanged(RESOLUTION)` — Card Animations cancels all `PlacementPhaseAnimator`-marked entities and overwrites `Transform` via `BoardLayout.cell_to_world()`; `CardAnimationsSet::React.after(BoardRenderSet::ScheduleTweens)` guarantees same-frame delivery.

**Engine**: Bevy 0.18 + bevy_tweening v0.15.0 (Bevy-0.18-compatible) | **Risk**: HIGH
**Engine Notes**: Design gates resolved 2026-05-02. `board-rendering.md` now defines `PlacementRevealAnimReady { entries: Vec<PlacementRevealEntry> }`, `PlacementRevealEntry { unit: Entity, lane: u8, cell: u8 }`, `LaneCell { lane: u8, cell: u8 }`, and `BoardLayout.cell_to_world(lane, cell) -> Vec2`. OQ-CA-01 resolved via Story 001/002: tests assert `TweenAnim.playback_state == PlaybackState::Playing`, not `AnimatorState::Playing`.

NOTE: CA-12 (PLACEMENT tween duration clamp to 250 ms) has no payload dependency and can be extracted as an independent unit test once Story 002 is DONE. Consider implementing CA-12 early if other blockers are resolved slowly.

**Control Manifest Rules (Presentation Layer)**:
- Required: Mandatory simultaneous-start for placement reveal — all 5 entries in single system pass, no `apply_deferred` between. `BoardLayout.cell_to_world()` as coordinate source for placement snap. `CardAnimationsSet::React.after(BoardRenderSet::ScheduleTweens)` ordering for same-frame `PlacementCancelAllAnimsRequested` delivery. PLACEMENT 250 ms hard cap applied at `Tween` construction: `duration.min(Duration::from_millis(250))`.
- Forbidden: Per-lane stagger during placement reveal (all 5 must start same frame). `Transform.translation` written directly while a `TweenAnim` is actively mutating translation. Any `*Bundle` type.
- Guardrail: PLACEMENT hard cap 250 ms on ALL animation durations.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [ ] **CA-3** — GIVEN `PlacementRevealAnimReady { entries }` is received with 5 `PlacementRevealEntry { unit, lane, cell }` entries, WHEN one `App::update()` tick runs, THEN all 5 unit entities have reveal `TweenAnim` controllers in `PlaybackState::Playing`. Implementation note: system must iterate all 5 entries in a single pass before any `apply_deferred` flush separates them — scheduling invariant validated in code review. **[BLOCKING]**
- [ ] **CA-4b** — GIVEN CA-4's placement animation cancellation scenario, WHEN the frame renders, THEN no partially-tweened `Transform` visual position persists on screen. *(Screenshot evidence; manual QA only.)* **[ADVISORY]**
- [ ] **CA-12** — GIVEN any PLACEMENT-phase animation is requested with any `GameConfig` value for `snap_back_duration_ms`, WHEN the `Tween` is constructed, THEN the tween's duration is clamped to ≤ 250 ms at construction time (`duration.min(Duration::from_millis(250))`). Verified for each PLACEMENT animation type: drag-lift, snap-back, hover, cell-highlight. **[BLOCKING]**
- [ ] **CA-21** — GIVEN a PLACEMENT-phase animation is in `PlaybackState::Playing` on an entity/controller with `PlacementPhaseAnimator` marker and the target board entity has `LaneCell { lane: u8, cell: u8 }`, WHEN `PlacementCancelAllAnimsRequested` is processed (same-frame delivery via SystemSet ordering), THEN (a) all `PlacementPhaseAnimator` controllers/entities are no longer in `Playing` state AND (b) `world.get::<Transform>(entity).unwrap().translation.truncate() == board_layout.cell_to_world(lane_cell.lane, lane_cell.cell)` with the entity's Z preserved. **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD Rules C-4, C-7, C-10, Edge Cases §2:*

1. **Simultaneous-start invariant (GDD Rule C-4):** `PlacementRevealAnimReady` handler iterates all entries in one pass. All 5 reveal `TweenAnim` controller insertions happen in the same `commands` batch before any flush. No `apply_deferred` between entries.

2. **PLACEMENT 250 ms hard cap (GDD Rule C-2):** At `Tween` construction: `duration = duration.min(Duration::from_millis(250))`. Applied to drag-lift, snap-back, hover scale, cell-highlight. This is a runtime clamp from `GameConfig.snap_back_duration_ms` knob (range 100–250 ms).

3. **`PlacementCancelAllAnimsRequested` handler (GDD Rule C-10, Edge Cases §2):** Queries `With<PlacementPhaseAnimator>`. For each controller/entity: cancel `TweenAnim` via `set_tweenable` or pause; overwrite the target entity's `Transform.translation.xy` using `Res<BoardLayout>.cell_to_world(lane_cell.lane, lane_cell.cell)` from the target entity's `LaneCell` component, preserving existing Z. Same-frame delivery guaranteed by `CardAnimationsSet::React.after(BoardRenderSet::ScheduleTweens)`.

4. **Placement-reveal flip timing (GDD Rule C-2, TR-CAN-005):** 80–100 ms flip duration. 3-frame squash: back-of-card silhouette → Prism White edge-on squash flash → front-face sprite. Uses `SpriteColorLens` + `TransformScaleXLens`. Coordinate source: `BoardLayout.cell_to_world(lane, cell)`.

5. **`PlacementCancelAllAnimsRequested` vs `BoardRebuildRequested` (GDD Edge Cases §2 vs §3):** `PlacementCancelAllAnimsRequested` is the phase-change path (normal game flow, every round); `BoardRebuildRequested` is the reconnect path. Both result in no in-flight PLACEMENT animators remaining. Story 002 handles `BoardRebuildRequested`; this story handles the phase-change path.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001](story-001-plugin-scaffold-custom-lenses.md): Plugin scaffold, lens infrastructure
- [Story 002](story-002-tween-cancel-replace-lifecycle.md): `BoardRebuildRequested` cancel-replace (reconnect path)
- [Story 003](story-003-simultaneous-track-animation.md): same-component simultaneous `Transform` animation controllers

---

## QA Test Cases

*Design gates for CA-3 and CA-21 are resolved. Tests can be written during implementation.*

**CA-3 — PlacementRevealAnimReady spawns reveal controllers on all 5 entities in single pass**

- Given: World with `CardAnimationsPlugin`; 5 pre-spawned unit entities with `LaneCell`; `PlacementRevealAnimReady { entries }` message written with 5 `PlacementRevealEntry { unit, lane, cell }` entries
- When: `app.update()` called once
- Then: all 5 reveal `TweenAnim` controllers are in `PlaybackState::Playing`
- Edge cases: 1-entity payload — single `Animator` spawned; 0-entity payload — no panic; scheduling invariant (no `apply_deferred` mid-system) verified in code review

**CA-12 — PLACEMENT tween duration clamped to 250 ms at construction time**

- Given: World with `CardAnimationsPlugin`; `GameConfig.snap_back_duration_ms=300` (above cap)
- When: Snap-back animation requested (snap-back event fires); `Tween<Transform>` constructed
- Then: `tween.duration().as_millis() <= 250`
- Repeat for: drag-lift, hover scale, cell-highlight — each PLACEMENT animation type in separate tests
- Edge cases: `snap_back_duration_ms=250` → accepted (at boundary); `snap_back_duration_ms=251` → clamped to 250; `snap_back_duration_ms=200` (below cap) → 200 ms used unchanged

**CA-21 — PlacementCancelAllAnimsRequested cancels and snaps to cell position**

- Given: World with `CardAnimationsPlugin` + `BoardLayout` resource; 2 board entities with `LaneCell` (lane=1/cell=1; lane=2/cell=3) and active PLACEMENT `TweenAnim` controllers marked `PlacementPhaseAnimator`; `PlacementCancelAllAnimsRequested` message written
- When: `app.update()` called once
- Then: (a) no `PlacementPhaseAnimator` controller remains in `PlaybackState::Playing`; (b) `Transform.translation.truncate() == board_layout.cell_to_world(lane, cell)` for each entity and Z is preserved
- Edge cases: no `PlacementPhaseAnimator` entities in world — no panic, no-op; entity with `PlacementPhaseAnimator` but no `LaneCell` — error logged, no crash

**CA-4b — No partially-tweened Transform position after cancellation (Visual)**

Manual check: No visual artifact after PLACEMENT animation cancellation
  - Setup: Game in PLACEMENT phase; drag a card partway across the board; wait for RESOLUTION to begin
  - Verify: Dragged card snaps to its committed cell; no card visible at an intermediate screen position after RESOLUTION starts
  - Pass condition: No ghost card visible at a mid-drag position once RESOLUTION begins (screenshot evidence in `production/qa/evidence/placement-reveal-evidence.md`)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- Integration: `tests/integration/card-animations/placement_reveal_test.rs` — must exist and pass
- Visual: `production/qa/evidence/placement-reveal-evidence.md` (CA-4b screenshot + sign-off)

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: [Story 002](story-002-tween-cancel-replace-lifecycle.md) DONE; `board-rendering.md` `PlacementRevealAnimReady` payload schema defined; `LaneCell` component defined; OQ-CA-01 resolved (`PlaybackState` test API)
- Unlocks: None directly
