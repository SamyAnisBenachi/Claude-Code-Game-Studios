# Story 002: Tween cancel-replace lifecycle

> **Epic**: Card Animations
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `Animator<T>` cancel-and-replace contract defined centrally — `set_tweenable()` replaces in-place, never despawn/respawn; `Animator` component remains on entity after `TweenCompleted` (archetype churn avoidance); `BoardRebuildRequested` cancels all in-flight tweens cancel-in-place; concurrent `Animator<Transform>` and `Animator<Sprite>` may coexist on one entity targeting different components.

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: HIGH
**Engine Notes**: OQ-CA-01 — `AnimatorState` enum name in bevy_tweening 0.18 is unconfirmed. If the state is not publicly inspectable via an enum, all 5 ACs (CA-2, CA-4, CA-6, CA-7, CA-19) require reformulation before the test harness is finalized. **Do not write the test file until Story 001 resolves OQ-CA-01 via cargo check.** Reformulation path: switch to `Time<Virtual>`-based completion assertions if `AnimatorState` inspection is unavailable. `BoardRebuildRequested` is a `#[derive(Message)]` intra-client type — use `MessageReader<BoardRebuildRequested>`, NOT `EventReader`.

**Control Manifest Rules (Presentation Layer)**:
- Required: Tween cancel-and-replace via `Animator<T>::set_tweenable(new_tween)`. `Animator<T>` component NOT removed on tween completion. Concurrent `Animator<Transform>` + `Animator<Sprite>` on same entity (different components, advanced independently). `PlacementPhaseAnimator` marker component inserted alongside any PLACEMENT-phase `Animator<T>`.
- Forbidden: `commands.entity(e).despawn()` + respawn for tween cancel. `commands.entity(e).remove::<Animator<T>>()` on tween completion. Writing `Transform.translation` directly while `Animator<Transform>` active (animator overwrites next frame). Any `*Bundle` type.
- Guardrail: Presentation steady-state < 1 ms per frame.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [x] **CA-2** — GIVEN board entities exist with `Animator<Transform>` or `Animator<Sprite>` in `AnimatorState::Playing`, WHEN `BoardRebuildRequested` is written and `App::update()` is called once, THEN (a) no entity that had a Playing animator is in `AnimatorState::Playing` after the tick, AND (b) those entities STILL HAVE their `Animator<T>` components present — verified by `world.get::<Animator<Transform>>(entity).is_some()`. Clause (b) confirms cancel-in-place rather than removal. **[BLOCKING]**
- [x] **CA-4** — GIVEN a PLACEMENT-phase animation is in `AnimatorState::Playing`, WHEN `BoardRebuildRequested` fires in the same frame, THEN the `Animator` is no longer in `AnimatorState::Playing` after that frame. **[BLOCKING]**
- [x] **CA-6** — GIVEN a unit entity has an `Animator<Transform>` in `TweenCompleted` state, WHEN a new tween is requested, THEN the entity retains its ECS entity ID (no despawn/respawn) AND the new animation begins within one `App::update()` tick. **[BLOCKING]**
- [x] **CA-7** — GIVEN a unit needs a simultaneous advance `Tween<Transform>` and death `SpriteAlphaLens` fade in the same `AnimGroup`, WHEN both tweens are spawned, THEN both `Animator<Transform>` and `Animator<Sprite>` exist on the entity and are both in `AnimatorState::Playing` after one tick. **[BLOCKING]**
- [x] **CA-19** — GIVEN an `Animator<Transform>` reaches `TweenCompleted` on a unit entity, WHEN no new tween is requested, THEN the `Animator<Transform>` component remains on the entity. **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines and GDD Rule C-7:*

1. **Cancel-in-place (GDD Rule C-7, ADR-021):** `BoardRebuildRequested` handler queries all entities with `Animator<T>` and calls `animator.set_tweenable(idle_tween)` or equivalent pause. The component MUST NOT be removed — only tween state reset. Avoids archetype migration cost.

2. **`set_tweenable` is unconditional (GDD Rule C-7):** Replaces regardless of current `AnimatorState` (Playing, TweenCompleted, or Paused). No guard clause on state before calling.

3. **Animator persists after completion (GDD Rule C-7):** Do NOT call `commands.entity(e).remove::<Animator<T>>()` in any tween-completion callback. The animator remains in `TweenCompleted` state, available for reuse via `set_tweenable`.

4. **Concurrent animators (GDD Rule C-7):** `Animator<Transform>` and `Animator<Sprite>` target different components and advance independently. Two `Animator<Transform>` on the same entity conflict — use `Tracks<Transform>` (Story 003).

5. **`PlacementPhaseAnimator` marker (GDD Rule C-7):** Every entity receiving a PLACEMENT-phase animation (`Animator<T>`) must also have `PlacementPhaseAnimator` marker inserted in the same frame. NOT removed on cancel — persists until entity despawn or `BoardRebuildRequested` board reset.

6. **`BoardRebuildRequested` message registration:** Registered by Board Rendering plugin in its `build()`. `CardAnimationsPlugin` consumes via `MessageReader<BoardRebuildRequested>` — no re-registration needed.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001](story-001-plugin-scaffold-custom-lenses.md): Plugin scaffold, lens construction, cargo-check gates
- [Story 003](story-003-simultaneous-track-animation.md): `Tracks<Transform>` for same-component parallel animations
- [Story 005](story-005-placement-reveal-parallelism.md): `PlacementCancelAllAnimsRequested` (phase-change path); `BoardRebuildRequested` here covers the reconnect path only

---

## QA Test Cases

*Written by qa-lead at story creation. All ACs assert `AnimatorState::Playing` — confirm exact enum name via OQ-CA-01 (Story 001 cargo check) before writing test file.*

**CA-2 — BoardRebuildRequested cancels Playing animators; Animator<T> component stays present**

- Given: World with `CardAnimationsPlugin`; 3 board entities each with `Animator<Transform>` and/or `Animator<Sprite>` in `AnimatorState::Playing` (1000 ms tween, `Time<Virtual>` advanced 1 ms to start); `MessageWriter<BoardRebuildRequested>` writes one message
- When: `app.update()` called once
- Then: (a) no entity has `Animator<T>` in `Playing` state; (b) `world.get::<Animator<Transform>>(e).is_some()` for all entities (component present, not removed)
- Edge cases: entity with `Animator<Sprite>` only — same guarantee; `BoardRebuildRequested` with zero Playing animators — no panic; fired twice same tick — idempotent

**CA-4 — PLACEMENT animator cancelled same frame as BoardRebuildRequested**

- Given: Entity with `Animator<Transform>` (snap-back, 250 ms, `EaseOutBack`) in `Playing`, carrying `PlacementPhaseAnimator` marker; `Time<Virtual>` advanced 1 ms; `BoardRebuildRequested` written same tick before update
- When: `app.update()` called once
- Then: `world.get::<Animator<Transform>>(entity)` not in `Playing` state
- Edge cases: tween at t=0 (never ticked) — cancelled; tween at t=249 ms (1 ms from completion) — cancelled before natural completion

**CA-6 — set_tweenable on TweenCompleted entity preserves ECS ID, begins new animation**

- Given: Entity with unique marker component `EntityIdentity` and `Animator<Transform>` advanced past duration (`TweenCompleted` state); record entity id = E
- When: System calls `animator.set_tweenable(new_100ms_tween)` on E; `app.update()` called once (`Time<Virtual>` advanced 1 ms)
- Then: `world.get_entity(E)` is `Ok` (not despawned — same ECS id); `Animator<Transform>` in `Playing` state
- Edge cases: multiple `set_tweenable` calls same tick — last-write wins, no panic

**CA-7 — Simultaneous Animator<Transform> and Animator<Sprite> on same entity**

- Given: Entity with `Sprite` and `Transform`; two `ResolutionEvent`s in same `AnimGroup` frame — one spawns `Animator<Transform>` (advance, 600 ms), one spawns `Animator<Sprite>` (`SpriteAlphaLens` fade, 200 ms); `Time<Virtual>` advanced 1 ms
- When: `app.update()` called once
- Then: `world.get::<Animator<Transform>>(entity)` in `Playing`; `world.get::<Animator<Sprite>>(entity)` in `Playing`
- Edge cases: `Animator<Sprite>` completes at 200 ms while `Animator<Transform>` still playing — `Sprite` transitions to `TweenCompleted`; `Transform` still `Playing`; no conflict

**CA-19 — Animator<Transform> remains after TweenCompleted (no removal)**

- Given: Entity with `Animator<Transform>` (tween duration 100 ms)
- When: `Time<Virtual>` advanced 100 ms; `app.update()` called once (tween completes naturally)
- Then: `world.get::<Animator<Transform>>(entity).is_some()` == true (component still present)
- Edge cases: additional `app.update()` calls after completion — still present; two entities completing same world — both retain independently

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/tween_lifecycle_test.rs` — must exist and pass

**Status**: [x] Created and passed locally with `cargo test -p client --test card_animations_tween_lifecycle_test --target-dir target\codex-card-anim-002-test`

---

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-custom-lenses.md) must be DONE (lens infrastructure; OQ-CA-01 resolved for test harness)
- Unlocks: [Story 005](story-005-placement-reveal-parallelism.md), [Story 007](story-007-damage-number-lifecycle.md), [Story 008](story-008-input-gating.md)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 5/5 passing. CA-2, CA-4, CA-6, CA-7, and CA-19 are covered by `tests/unit/card-animations/tween_lifecycle_test.rs`.
**Deviations**: Advisory only: story/GDD/ADR wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`; the compiled workspace API is `TweenAnim` with `PlaybackState`/`TweenState` from `bevy_tweening v0.15.0`, the Bevy 0.18-compatible crate version used here. Advisory only: current `TR-CAN-007` registry text maps to CA-25/scaffold-style lifecycle wording, while this story closes the GDD-scoped CA-2, CA-4, CA-6, CA-7, and CA-19 criteria.
**Test Evidence**: Logic: `tests/unit/card-animations/tween_lifecycle_test.rs`; `cargo test -p client --test card_animations_tween_lifecycle_test --target-dir target\codex-card-anim-002-test` passed 5/5. Paired scaffold+lifecycle command passed 8/8 and 5/5. `cargo check -p client --target-dir target\codex-card-anim-002-test` passed.
**Code Review**: Skipped - Lean mode.
