# Story 007: Damage number lifecycle (F2 despawn timer)

> **Epic**: Card Animations
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `DamageNumberSpawnRequested` domain event (from Board Rendering) triggers damage number entity spawn; custom `TextColorLens` and `Tween<Transform>` run concurrently on the same entity; `DespawnAfter` timer initialized from F2 (`max(float_tween_duration_ms, fade_tween_duration_ms)`) at spawn time — NOT from tween completion; entity carries `DamageNumber` marker for cleanup sweep on `BoardRebuildRequested`.

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: HIGH
**Engine Notes**: ⚠️ **BLOCKED — three pre-implementation gates:**
1. **OQ-CA-11**: Jitter table for F3 (`event_id % jitter_table_len`) undefined — must define `jitter_table_len` and table `Vec2` contents before simultaneous-damage edge case tests can be written. Blocks CA-25.
2. **CA-25 text entity layout**: `DamageNumberSpawnRequested` payload schema unconfirmed; text entity component layout (`Text2d` vs `Text`, `TextFont`, `TextColor`, `LineHeight`) unspecified. Blocks CA-25.
3. **`DespawnAfter` component**: Not yet defined — blocks CA-8, CA-9, CA-25. Once `DespawnAfter` is defined, CA-8 and CA-9 (F2 timer math tests) can be implemented independently of OQ-CA-11 and CA-25's other blockers.

Also: OQ-CA-04 (`TextColor` satisfies `Component` bound for `TextColorLens`) must be resolved (Story 001 cargo-check gate).

**Control Manifest Rules (Presentation Layer)**:
- Required: `DamageNumber` marker component on all damage number entities (enables `BoardRebuildRequested` cleanup sweep via `Query<Entity, With<DamageNumber>>`). `DespawnAfter(Timer)` initialized at spawn time from F2. No client-side RNG — use `event_id % jitter_table_len` for deterministic position jitter. Concurrent `Animator<Transform>` (float) + text component animator (fade) on same entity.
- Forbidden: Entity despawn on float tween completion (F2 uses `max()` — always despawn at the later of the two). Entity reuse for multiple simultaneous damage numbers (each spawns its own entity). Direct `S2C*` subscription.
- Guardrail: `max(float_tween_duration_ms, fade_tween_duration_ms) + 50 < resolution_sub_step_duration_ms` (strict `<`). Startup assert in `CardAnimationsPlugin` validates this constraint. At defaults: 500+50=550 < 600 ✅.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [ ] **CA-8** — GIVEN a damage number entity pre-spawned with `float_tween_duration_ms=500` and `fade_tween_duration_ms=500`, WHEN `Time<Virtual>` is advanced by 500 ms and `App::update()` runs, THEN the entity is despawned (`World::get_entity()` returns `Err`). F2: `max(500, 500)=500`. Despawn timer set at spawn time from F2 — NOT from tween-completion event. **[BLOCKING]** *(Can be implemented once `DespawnAfter` is defined, independently of other CA-25 blockers.)*
- [ ] **CA-9** — GIVEN `float_tween_duration_ms=400` and `fade_tween_duration_ms=600`, WHEN `Time<Virtual>` is advanced by 400 ms and `App::update()` runs, THEN entity still exists; WHEN advanced a further 200 ms (total 600 ms) and `App::update()` runs, THEN entity is despawned. F2: `max(400, 600)=600`. Entity must NOT be despawned at float tween's 400 ms completion. **[BLOCKING]** *(Can be implemented once `DespawnAfter` is defined.)*
- [ ] **CA-25** — GIVEN `DamageNumberSpawnRequested { target: entity, damage_value: 15, event_id: 0 }` is written, WHEN `App::update()` runs, THEN (a) exactly one entity with `DamageNumber` marker exists, AND (b) the entity has `Animator<Transform>` and text component both in `Playing` state, AND (c) entity carries `DespawnAfter(Timer)` initialized from F2 (`max(float_tween_duration_ms, fade_tween_duration_ms)`). **PRE-IMPLEMENTATION GATE: BLOCKED until payload schema confirmed, text entity component layout specified (`Text2d`/`Text` + `TextFont` + `TextColor` + `LineHeight`), `DespawnAfter` component defined, and OQ-CA-11 resolved.** **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule C-7, F2, F3 formulas:*

1. **F2 despawn formula (GDD §Formulas):** `despawn_delay_ms = max(float_tween_duration_ms, fade_tween_duration_ms)`. `DespawnAfter(Timer::new(Duration::from_millis(despawn_delay_ms), TimerMode::Once))` initialized at spawn time. Timer advances via `Time<Virtual>`. Entity despawned when timer finishes — NOT when either tween completes. Float tween may finish first; entity coasts at final float position while alpha fade continues draining to zero.

2. **Concurrent animators (GDD Rule C-7):** `Animator<Transform>` (+60 px float, `EaseOutCubic`, 500 ms default) and text component animator (`TextColorLens` fade) start in the same frame as entity spawn. Both ticked independently.

3. **F3 position jitter (GDD F3 — BLOCKED on OQ-CA-11):** When two simultaneous `DamageNumberSpawnRequested` events target the same unit in one sub-step, second entity position offset = `jitter_table[event_id % jitter_table_len]`. `jitter_table` = static `Vec2` array. No client-side RNG.

4. **Damage number origin (GDD V.1):** Float starts from unit torso world position, NOT health bar position. Prevents number occlusion of the HP bar.

5. **Startup assert (GDD §Tuning Knobs):** `max(float_tween_duration_ms, fade_tween_duration_ms) + 50 < resolution_sub_step_duration_ms` (strict `<`). Post-deserialization validation on complete `GameConfig` struct. Panic with descriptive message if violated. Note: `< sub_step` strict, not `<=`. 550 ms at sub_step=600 ms fails the assert.

6. **`DamageNumber` marker for cleanup (GDD Edge Cases §4):** `BoardRebuildRequested` handler includes `Query<Entity, With<DamageNumber>>` despawn sweep. Stale floating numbers must not survive reconnect.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001](story-001-plugin-scaffold-custom-lenses.md): Plugin scaffold, `TextColorLens` definition
- [Story 002](story-002-tween-cancel-replace-lifecycle.md): `BoardRebuildRequested` handler, `DamageNumber` entity cleanup sweep

---

## QA Test Cases

*Written by qa-lead at story creation. CA-8 and CA-9 reviewable once `DespawnAfter` defined. CA-25 is PRE-IMPL GATE.*

**CA-8 — F2 symmetric despawn: max(500, 500)=500 ms**

- Given: Pre-spawned damage number entity with `DespawnAfter(Timer::new(Duration::from_millis(500), TimerMode::Once))`, `Animator<Transform>` (float 500 ms), text animator (fade 500 ms), `DamageNumber` marker
- When: `advance_by(Duration::from_millis(500))`; `app.update()` called once
- Then: `world.get_entity(entity)` returns `Err` (despawned)
- Edge cases: advance 499 ms — entity still exists; advance 501 ms — entity still despawned; no `app.update` after advance — entity not yet despawned (system must tick)

**CA-9 — F2 asymmetric despawn: max(400, 600)=600 ms**

- Given: Pre-spawned damage number entity with `DespawnAfter` timer=600 ms (F2: max(400,600)=600), `Animator<Transform>` float=400 ms, text fade=600 ms
- When (t=400 ms): `advance_by(400)`; `app.update()` — entity still exists
- Then: `world.get_entity(entity)` is `Ok` (float tween completed at 400 ms but despawn timer at 400/600 ms — entity NOT despawned early)
- When (t=600 ms): `advance_by(200)`; `app.update()`
- Then: `world.get_entity(entity)` returns `Err` (despawned at 600 ms)
- Edge cases: verify no early despawn triggered by float tween callback — entity presence at 400 ms is the critical assertion

**CA-25 — DamageNumberSpawnRequested spawns entity with correct components (PRE-IMPL)**

- Given: World with `CardAnimationsPlugin`; target board entity pre-spawned; `DamageNumberSpawnRequested { target, damage_value: 15, event_id: 0 }` written via `MessageWriter`
- When: `app.update()` called once
- Then: (a) exactly one `DamageNumber`-marked entity in world; (b) entity has `Animator<Transform>` and text component both in `Playing` state; (c) entity has `DespawnAfter(Timer)` initialized to `max(float_tween_duration_ms, fade_tween_duration_ms)` from `GameConfig`
- Edge cases: two simultaneous events (event_id=0, event_id=1) — two distinct entities; second entity's position offset by `jitter_table[1 % jitter_table_len]` (OQ-CA-11); damage_value=0 — entity still spawned

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/damage_number_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: [Story 002](story-002-tween-cancel-replace-lifecycle.md) must be DONE (concurrent animator contract); `DespawnAfter` component defined; OQ-CA-11 resolved; CA-25 pre-impl gates cleared (payload schema, text entity layout)
- Unlocks: None
