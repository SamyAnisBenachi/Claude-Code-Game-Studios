# Story 004: AnimQueue RESOLUTION drain + GAME_OVER skip path + empty-queue handling

> **Epic**: Card Animations
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-002`, `TR-CAN-006`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `AnimQueue` Resource drives RESOLUTION sub-step playback via `group_timer.finished()` checked by `ResolutionExecuting` system each frame; `GroupDrainedSignal` Bevy `#[derive(Message)]` emitted to Board Rendering for GAME_OVER path; `Time<Virtual>`-driven group and inter-step timers; GAME_OVER skip path holds `current_index` at 0 and runs `ResolutionObjectiveReveal`.

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: HIGH
**Engine Notes**: No `AnimatorState` dependency in this story's ACs — all assertions are resource-state and timer-based. `MessageWriter<GroupDrainedSignal>` is a Bevy-internal `Message` (NOT Lightyear `MessageSender`). `Time<Virtual>` advances via `world.resource_mut::<Time<Virtual>>().advance_by(Duration::from_millis(N))`. `MessageWriter<T>` cannot be used inside an exclusive system (not relevant here — `ResolutionExecuting` is a regular system).

**Control Manifest Rules (Presentation Layer)**:
- Required: `AnimQueue` tick system runs first within `PresentationSet::AnimationTick`. `GroupDrainedSignal` registered via `app.add_message::<GroupDrainedSignal>()` in `CardAnimationsPlugin`. `AnimQueue.groups` cleared on `BoardRebuildRequested`. `StagedObjectiveRevealQueue` cleared on `BoardRebuildRequested`. Sub-step grouping invariant: all events sharing a `sub_step` are in the same `AnimGroup`.
- Forbidden: `EventWriter<T>` / `EventReader<T>`. `AnimQueue` blocking phase advance. Direct `S2C*` subscription in `card_animations/` module. Streaming per-sub-step S2C events (one batch `S2CResolutionEvent` only — consumed upstream by Board Rendering, which populates `AnimQueue`).
- Guardrail: Presentation steady-state < 1 ms per frame. Phase-boundary frame < 3 ms spike.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [ ] **CA-5a** — GIVEN `AnimQueue` pre-loaded with two `AnimGroup`s (group 0: 600 ms empty; group 1: 600 ms containing a `ResolutionEvent` that would spawn `Animator<Transform>` on pre-spawned `entity_1`) AND `PendingPhaseChange` set to `GAME_OVER`, WHEN `Time<Virtual>` is advanced by 600 ms and `App::update()` runs, THEN (a) `world.resource::<AnimQueue>().current_index == 0` AND (b) `world.get::<Animator<Transform>>(entity_1).is_none()`. Clause (b) is non-vacuous: if skip failed, entity_1 would have an `Animator`. **[BLOCKING]**
- [ ] **CA-5b** — GIVEN same setup as CA-5a PLUS `PendingObjectiveDestroyedEvents` pre-populated for 1 lane, WHEN `Time<Virtual>` advanced by 600 ms and `App::update()` runs, THEN the objective-reveal `Animator` for that lane's entity exists and is in `Playing` state. **[BLOCKING]**
- [ ] **CA-5c** — GIVEN same setup as CA-5b, WHEN `Time<Virtual>` advanced by 599 ms and `App::update()` runs, THEN `current_index == 0` AND objective-reveal `Animator` does NOT exist on that entity; WHEN advanced a further 1 ms and `App::update()` runs, THEN (a) `current_index == 0` still (skip fired; group 1 not started) AND (b) objective-reveal `Animator` for the lane exists in `Playing` state. **[BLOCKING]**
- [ ] **CA-17** — GIVEN `AnimQueue` with `groups.len() == 0`, WHEN the `ResolutionExecuting` drain system runs and `Time<Virtual>` is advanced by `pre_animation_pause_ms`, THEN no tween-spawning commands are issued, no panic, and `PendingPhaseChange` is drained (buffered `S2CPhaseChanged` emitted). Verified by asserting zero new `Animator` inserts and `PendingPhaseChange` is `None` after the tick. **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule C-8:*

1. **`ResolutionExecuting` system (GDD Rule C-8, ADR-021 §AnimationTick):** Each frame: check `AnimQueue.group_timer.finished()`. When finished: start `inter_step_timer`. When inter-step timer finishes: advance `current_index`, spawn next group's tweens, reset `group_timer`.

2. **GAME_OVER skip path (GDD Rule C-8):** When `group_timer.finished()` AND `PendingPhaseChange` contains `GAME_OVER`: (1) hold `current_index` at 0 — do NOT advance to group 1, (2) run `ResolutionObjectiveReveal` for any buffered `ObjectiveDestroyed` events, (3) emit `GroupDrainedSignal` via `MessageWriter<GroupDrainedSignal>`.

3. **`ResolutionObjectiveReveal` system (GDD Rule C-9, item 5a):** Drains `StagedObjectiveRevealQueue` timers (ticked by `Time<Virtual>`). Spawns objective-reveal animators. Runs after `ResolutionExecuting` in `AnimationTick` PresentationSet.

4. **Empty queue (CA-17):** `ResolutionExecuting` with `groups.len() == 0` advances the `pre_animation_pause_ms` timer (from `Res<GameConfig>`), then drains `PendingPhaseChange`. No tween-spawning commands issued. No panic.

5. **Queue lifecycle (GDD Rule C-8):** When RESOLUTION completes (all groups drained), `AnimQueue.groups` cleared, `current_index` reset to 0. On `BoardRebuildRequested`: `AnimQueue` reset to `Default::default()` immediately; `StagedObjectiveRevealQueue` also cleared.

6. **`GroupDrainedSignal` registration:** `app.add_message::<GroupDrainedSignal>()` in `CardAnimationsPlugin::build()` — registered in Story 001 scaffold.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001](story-001-plugin-scaffold-custom-lenses.md): Plugin scaffold, `StagedObjectiveRevealQueue` Resource definition
- [Story 002](story-002-tween-cancel-replace-lifecycle.md): `BoardRebuildRequested` cancel-replace (reconnect path, queue clear)
- [Story 006](story-006-objective-stagger-reveal.md): Multi-objective stagger reveal — `StagedObjectiveRevealQueue` drain + F1 formula

---

## QA Test Cases

*Written by qa-lead at story creation. No `AnimatorState` dependency — all `Time<Virtual>`-driven.*

**CA-5a — GAME_OVER skip: group 1 tweens never spawned; current_index stays at 0**

- Given: `World` with `CardAnimationsPlugin`; `AnimQueue` with 2 groups (group 0: 600 ms empty; group 1: 600 ms with `ResolutionEvent` → `Animator<Transform>` on pre-spawned `entity_1`); `PendingPhaseChange` resource set to `GAME_OVER`; `Time<Virtual>` advanced 600 ms
- When: `app.update()` called once (group 0 timer fires; GAME_OVER skip detected)
- Then: (a) `world.resource::<AnimQueue>().current_index == 0`; (b) `world.get::<Animator<Transform>>(entity_1).is_none()`
- Edge cases: `PendingPhaseChange` is `None` — normal drain, `current_index` advances to 1 after group 0; group 0 at 599 ms — skip not yet triggered

**CA-5b — GAME_OVER skip still runs ResolutionObjectiveReveal**

- Given: Same as CA-5a + `PendingObjectiveDestroyedEvents` populated for lane 2 (pre-spawned objective entity for lane 2); `Time<Virtual>` advanced 600 ms
- When: `app.update()` called once
- Then: Objective-reveal `Animator<T>` for lane 2 entity exists and is in `Playing` state
- Edge cases: multiple lanes in `PendingObjectiveDestroyedEvents` — all get reveal animators spawned; lane entity missing — error logged, no crash

**CA-5c — Temporal boundary: skip fires at exactly 600 ms, not 599 ms**

- Given: Same as CA-5b (2-group queue + GAME_OVER + 1 pending objective destroyed)
- When (part 1): `Time<Virtual>` advanced 599 ms; `app.update()`
- Then: `current_index == 0`; objective-reveal `Animator` does NOT exist on lane entity
- When (part 2): `advance_by(Duration::from_millis(1))`; `app.update()` (cumulative 600 ms)
- Then: (a) `current_index == 0` still (skip fired; group 1 not started); (b) objective-reveal `Animator` for lane exists in `Playing` state
- Edge cases: second `app.update()` after 600 ms — idempotent, no re-trigger

**CA-17 — Empty AnimQueue: no spawn, no panic, PendingPhaseChange drained**

- Given: `AnimQueue` with `groups.len() == 0`; `PendingPhaseChange` set to a pending value; `Time<Virtual>` advanced by `pre_animation_pause_ms` (400 ms default)
- When: `app.update()` called once
- Then: (a) zero new `Animator` inserts (query count unchanged); (b) no panic; (c) `world.resource::<PendingPhaseChange>()` is `None` (phase change emitted)
- Edge cases: `Time<Virtual>` advanced only 399 ms — `PendingPhaseChange` NOT yet drained (timer not finished); `AnimQueue` reset to empty mid-queue (reconnect) — same empty-queue behavior on next update

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/anim_queue_test.rs` — must exist and pass

**Status**: [x] Created and passing via `cargo test -p client --test card_animations_anim_queue_test`

---

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-custom-lenses.md) must be DONE (plugin scaffold, `AnimQueue` Resource, `StagedObjectiveRevealQueue` registered)
- Unlocks: [Story 006](story-006-objective-stagger-reveal.md) (multi-objective stagger uses `StagedObjectiveRevealQueue` from this story)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 4/4 passing
**Deviations**: Advisory only - story/GDD wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`, while the compiled workspace uses `TweenAnim` / `PlaybackState` / `TweenState` from `bevy_tweening 0.15`. Advisory only - `TR-CAN-006` registry text says 100 ms pause, while current GDD/code use 150 ms inter-step pause; this did not affect the four scoped ACs.
**Test Evidence**: Logic test file at `tests/unit/card-animations/anim_queue_test.rs`; `cargo test -p client --test card_animations_anim_queue_test` passed 4/4.
**Code Review**: Skipped - Lean mode.
