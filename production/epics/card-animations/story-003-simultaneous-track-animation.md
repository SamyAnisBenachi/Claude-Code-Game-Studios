# Story 003: Simultaneous Transform controller animation

> **Epic**: Card Animations
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Mandatory simultaneous-start parallelism for same-event animations. OQ-CA-02 resolved after ADR creation: `Tracks<T>` is removed in the local Bevy-0.18-compatible `bevy_tweening v0.15.0` API. Same-entity same-component parallel `Transform` animation uses independent `TweenAnim` controller entities targeting the unit with `AnimTarget::component::<Transform>(target)`, with field-specific or composite lenses to prevent write-order conflicts.

**Engine**: Bevy 0.18 + bevy_tweening v0.15.0 (Bevy-0.18-compatible) | **Risk**: HIGH
**Engine Notes**: OQ-CA-01 and OQ-CA-02 are resolved. Use `TweenAnim`, `AnimTarget::component`, `PlaybackState::Playing`, and `TweenState` in tests. Do not use `Tracks<T>`; the crate changelog explicitly removed it and the crate docs direct parallel animation to independently enqueued animations.

**Control Manifest Rules (Presentation Layer)**:
- Required: Same-event animations start in the same frame. `TweenAnim` cancel-and-replace via `set_tweenable`. Same-component parallel `Transform` animation uses independent controller entities with `AnimTarget::component::<Transform>(target)`.
- Forbidden: Multiple `TweenAnim` components on the same ECS entity for the same target path; use separate controller entities instead. Despawn+respawn for animation management. Any `*Bundle` type.
- Guardrail: Presentation steady-state < 1 ms per frame.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [ ] **CA-15** — GIVEN a unit entity needs simultaneous REPEL displacement (`Tween<Transform>`) and placement-reveal flip (`SpriteColorLens` + `TransformScaleXLens`), WHEN both tweens are spawned, THEN the relevant `TweenAnim` controllers exist simultaneously and are in `PlaybackState::Playing` after one tick. **[BLOCKING]**
- [ ] **CA-18** — GIVEN a unit entity at `Transform::IDENTITY` requires two simultaneous `Tween<Transform>` animations implemented as separate `TweenAnim` controller entities targeting the unit via `AnimTarget::component::<Transform>(target)` (tween A: X 0→100, 600 ms via X-only translation lens; tween B: Y 0→60, 600 ms via Y-only translation lens), WHEN `Time<Virtual>` is advanced by 16 ms and `App::update()` runs, THEN: (a) exactly two `TweenAnim` controllers target the unit's `Transform` and both are in `PlaybackState::Playing`, AND (b) `Transform.translation.x` is in range (0.0, 100.0) exclusive, AND (c) `Transform.translation.y` is in range (0.0, 60.0) exclusive. **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule C-7:*

1. **OQ-CA-02 resolved — `Tracks<T>` removed.** Use independent `TweenAnim` controller entities with `AnimTarget::component::<Transform>(target)`. This is the documented replacement path in `bevy_tweening v0.15.0`.

2. **Same-component parallel usage (GDD Rule C-7):** When one entity needs two simultaneous `Transform` animations, spawn one controller entity per tween and target the board unit with `AnimTarget::component::<Transform>(unit)`. Do not attach multiple `TweenAnim` components to the unit itself.

3. **Simultaneous-start invariant (GDD Rule C-4):** Both controllers are spawned in the same system pass and start in the same frame. Events in the same `AnimGroup` spawn tweens in a single system pass — no `apply_deferred` injection between entries.

4. **Lens write-scope requirement:** Parallel same-`Transform` tweens must not both write the same field unless they are combined into one composite lens. For CA-18, use X-only and Y-only translation lenses. For REPEL plus placement reveal scale, the displacement lens writes translation and `TransformScaleXLens` writes scale.x, so two independent controllers are valid.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002](story-002-tween-cancel-replace-lifecycle.md): Single-component cancel-replace; concurrent `Animator<Transform>`/`Animator<Sprite>` on different components
- [Story 005](story-005-placement-reveal-parallelism.md): 5-lane placement-reveal parallelism (uses single-component animators per lane)

---

## QA Test Cases

*OQ gates resolved; tests can be written during implementation.*

**CA-15 — REPEL + placement-reveal flip: 3 animators on 1 entity simultaneously**

- Given: Entity with `Sprite` and `Transform`; `DisplacementAnimRequested` spawns a `TweenAnim` controller targeting `Transform`; `PlacementRevealAnimReady` for the same entity spawns a `TweenAnim` controller targeting `Sprite` (`SpriteColorLens`) and a separate `TweenAnim` controller targeting `Transform` scale; `Time<Virtual>` advanced 1 ms
- When: `app.update()` called once
- Then: all relevant `TweenAnim` controllers are in `PlaybackState::Playing`
- Edge cases: only displacement fires alone — no cross-contamination; only reveal fires alone — no displacement spawned

**CA-18 — Parallel Transform controllers advance X and Y simultaneously**

- Given: Entity at `Transform::IDENTITY`; two `TweenAnim` controller entities target the same unit `Transform` via `AnimTarget::component`; tween A uses an X-only translation lens (X: 0→100, 600 ms), and tween B uses a Y-only translation lens (Y: 0→60, 600 ms); `Time<Virtual>` advanced 16 ms
- When: `app.update()` called once
- Then: (a) exactly two `TweenAnim` controllers target the entity's `Transform` and are `Playing`; (b) `Transform.translation.x` in (0.0, 100.0); (c) `Transform.translation.y` in (0.0, 60.0)
- Edge cases: advance 600 ms → x=100.0, y=60.0 exactly; advance 0 ms → x=0.0, y=0.0 (no movement)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/tracks_animation_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: [Story 002](story-002-tween-cancel-replace-lifecycle.md) DONE (cancel-replace contract established); OQ-CA-01 resolved (`PlaybackState` test API); OQ-CA-02 resolved (`Tracks<T>` removed, independent controller design selected)
- Unlocks: None directly (enables REPEL/ATTRACT displacement animation implementation)
