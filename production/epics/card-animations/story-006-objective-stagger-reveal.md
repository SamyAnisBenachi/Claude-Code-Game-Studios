# Story 006: Multi-objective stagger reveal (F1 formula)

> **Epic**: Card Animations
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-001`, `TR-CAN-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `StagedObjectiveRevealQueue` resource (`VecDeque<(u8, Timer)>`) owned by Card Animations; `ResolutionObjectiveReveal` system drains queue timers (ticked by `Time<Virtual>`) and spawns objective-reveal animators after `AnimQueue` drains; `GroupDrainedSignal` drives GAME_OVER skip path which also runs `ResolutionObjectiveReveal` for buffered events.

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: HIGH
**Engine Notes**: No `AnimatorState` dependency in this story's ACs — all assertions are `Time<Virtual>`-driven timer checks. `Time<Virtual>` advances via `world.resource_mut::<Time<Virtual>>().advance_by(Duration::from_millis(N))`. `StagedObjectiveRevealQueue` is a Card Animations-owned resource inserted in Story 001.

**Control Manifest Rules (Presentation Layer)**:
- Required: Stagger by ascending lane at `stagger_cadence_ms` cadence from `Res<GameConfig>`. Board Rendering pre-sorts `ObjectiveDestroyedAnimReady` events ascending by lane — Card Animations does NOT re-sort (defensive sort optional). `StagedObjectiveRevealQueue` cleared on `BoardRebuildRequested`. Objective destruction reveal: 3-frame step function (80% → 60% → 30% Prism White over 240 ms) — NOT a smooth interpolation.
- Forbidden: Per-lane stagger during placement reveal (stagger applies to objective destruction only). Sequential blocking stagger (all reveals start at their offset but run concurrently). Direct `S2C*` subscription.
- Guardrail: Presentation steady-state < 1 ms per frame.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [ ] **CA-10** — GIVEN `GameConfig.stagger_cadence_ms=100` and `ObjectiveDestroyed` events for lanes 3 and 5, WHEN the reveal sequence starts, THEN lane 3's `Animator` is spawned at t=0 ms and lane 5's `Animator` enters `Playing` at t=100 ms (±16 ms frame tolerance). Verified via `Time<Virtual>` injection: `App::update()` at t=0 ms asserts lane 3 started; `advance_by(100 ms)` + `update()` asserts lane 5 started. **[BLOCKING]**
- [ ] **CA-11** — GIVEN `GameConfig.stagger_cadence_ms=0` and two `ObjectiveDestroyed` events, WHEN the reveal sequence starts, THEN both reveal animations start in the same frame and no panic or undefined behavior occurs. F1: `reveal_start_ms[i] = 0` for all `i`. **[BLOCKING]**
- [ ] **CA-16** — GIVEN `GameConfig.stagger_cadence_ms=120` (maximum defined cadence) and `ObjectiveDestroyed` events for 4 lanes (i=0 through i=3), WHEN the reveal sequence starts, THEN the `i=3` lane's `Animator` enters `Playing` at 360 ms (±16 ms tolerance). Verified via `Time<Virtual>`: `advance_by(Duration::from_millis(360))` + `update()`. Tests F1's upper bound at its defined domain limit. **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD F1 formula, Rule C-8:*

1. **F1 formula (GDD §Formulas):** `reveal_start_ms[i] = i × stagger_cadence_ms`. `i` = zero-indexed position in ascending-lane-sorted `ObjectiveDestroyed` list. Maximum `i=3` (both players lose 2 objectives each). Board Rendering pre-sorts; Card Animations does NOT re-sort (add defensive sort as a safety net if needed).

2. **Concurrent-with-stagger execution (GDD F1):** Each reveal starts at its `reveal_start_ms[i]` offset but runs concurrently with subsequent reveals. `StagedObjectiveRevealQueue: VecDeque<(lane_id: u8, stagger_timer: Timer)>` — `ResolutionObjectiveReveal` system polls each frame, pops entries whose stagger timer elapsed, spawns the reveal `Animator`.

3. **`stagger_cadence_ms=0` degenerate (GDD Edge Cases §8):** F1 produces `reveal_start_ms[i] = 0` for all i. `Timer::new(Duration::ZERO, TimerMode::Once)` fires on first tick. All reveals start same frame. No panic — permitted behavior even below the 80 ms advisory minimum.

4. **`StagedObjectiveRevealQueue` lifecycle (GDD Rule C-8):** Cleared on `BoardRebuildRequested` (Story 002). Cleared after all entries drain. `ResolutionObjectiveReveal` runs after `ResolutionExecuting` in `PresentationSet::AnimationTick`.

5. **240 ms Prism White overlay (GDD Rule C-12, V.1):** Objective destruction reveal uses a step function — 3 sequential `Tween`s each ~80 ms duration. NOT a smooth `SpriteColorLens` interpolation. Prism White = `Color::srgba(0.933, 0.957, 1.0, alpha)` (#EEF4FF).

6. **`ResolutionObjectiveReveal` system registration (GDD Rule C-9, item 5a):** Registered in `CardAnimationsPlugin::build()`. Runs after `ResolutionExecuting` in the same `PresentationSet::AnimationTick` set.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 004](story-004-anim-queue-resolution-drain.md): `AnimQueue` RESOLUTION drain, `StagedObjectiveRevealQueue` Resource definition, `ResolutionObjectiveReveal` system scaffold
- [Story 002](story-002-tween-cancel-replace-lifecycle.md): `BoardRebuildRequested` queue clear (reconnect path)

---

## QA Test Cases

*Written by qa-lead at story creation. No `AnimatorState` dependency — all `Time<Virtual>`-driven.*

**CA-10 — F1 two-lane stagger at 100 ms cadence**

- Given: `GameConfig.stagger_cadence_ms=100`; `ObjectiveDestroyedAnimReady` events for lanes 3 and 5 written; pre-spawned objective entities for both lanes (no `Time<Virtual>` advance — stagger timer i=0 fires on first tick)
- When (t=0): `app.update()` called once
- Then: lane 3 objective entity has `Animator` component (i=0, stagger=0 ms fires immediately); lane 5 does NOT have `Animator` yet
- When (t=100 ms): `advance_by(Duration::from_millis(100))`; `app.update()`
- Then: lane 5 objective entity has `Animator` component (i=1, stagger=100 ms elapsed); lane 3 still has `Animator` (may be Playing or TweenCompleted — still present)
- Edge cases: advance 99 ms — lane 5 `Animator` absent; advance 116 ms (tolerance ceiling) — lane 5 definitely present

**CA-11 — F1 degenerate: stagger_cadence_ms=0, both start same frame**

- Given: `GameConfig.stagger_cadence_ms=0`; 2 `ObjectiveDestroyedAnimReady` events; pre-spawned entities for both lanes
- When: `app.update()` called once (no time advance)
- Then: Both entities have `Animator` components; no panic; no undefined behavior
- Edge cases: 4 lanes with cadence=0 — all 4 start same frame, no panic; cadence=1 (near-zero) — lane i=1 fires after 1 ms advance, not same frame

**CA-16 — F1 upper bound: i=3, cadence=120 ms, start at 360 ms (±16 ms)**

- Given: `GameConfig.stagger_cadence_ms=120`; 4 `ObjectiveDestroyedAnimReady` events (ascending lanes, i=0..3); pre-spawned entities for all 4
- When: `advance_by(Duration::from_millis(360))`; `app.update()`
- Then: i=3 lane's entity has `Animator` component (3 × 120 ms = 360 ms elapsed)
- Edge cases: advance 344 ms (360−16 tolerance floor) — i=3 `Animator` absent; advance 376 ms (tolerance ceiling) — definitely present; i=3 only (isolating last stagger): timer starts at 360 ms, not 0 ms (sorted index is the stagger input, not lane number)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/objective_stagger_test.rs` — must exist and pass

**Status**: [x] Created and passing via `cargo test -p client --test card_animations_objective_stagger_test`

---

## Dependencies

- Depends on: [Story 004](story-004-anim-queue-resolution-drain.md) must be DONE (`StagedObjectiveRevealQueue` Resource inserted; `ResolutionObjectiveReveal` system scaffolded)
- Unlocks: None (final RESOLUTION animation story in this epic)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 3/3 passing
**Deviations**: Advisory only - `TR-CAN-001` and `TR-CAN-004` registry entries map to older CA-1/CA-4 wording, while this story verifies story-scoped GDD criteria CA-10, CA-11, and CA-16. Advisory only - story/GDD wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`, while the compiled workspace uses `TweenAnim` / `PlaybackState` / `TweenState`.
**Test Evidence**: Logic test file at `tests/unit/card-animations/objective_stagger_test.rs`; `cargo test -p client --test card_animations_objective_stagger_test` passed 3/3. Regression tests `cargo test -p client --test card_animations_anim_queue_test` passed 4/4 and `cargo test -p client --test card_animations_plugin_scaffold_test` passed 8/8. `cargo check -p client` passed.
**Code Review**: Skipped - Lean mode.
