# Story 007: Damage number lifecycle (F2 despawn timer)

> **Epic**: Card Animations
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `DamageNumberSpawnRequested` domain event (from Board Rendering) triggers damage number entity spawn; custom `TextColorLens` and `Tween<Transform>` run concurrently on the same entity; `DespawnAfter` timer initialized from F2 (`max(float_tween_duration_ms, fade_tween_duration_ms)`) at spawn time — NOT from tween completion; entity carries `DamageNumber` marker for cleanup sweep on `BoardRebuildRequested`.

**Engine**: Bevy 0.18 + bevy_tweening v0.15.0 (Bevy-0.18-compatible) | **Risk**: HIGH
**Engine Notes**: Design gates resolved 2026-05-02. F3 now defines `jitter_table_len = 8` and the static `Vec2` offsets. `DamageNumberSpawnRequested { target: Entity, damage_value: u32, event_id: u32 }` is confirmed in `card-animations.md`. Damage numbers use world-space `Text2d` + `TextFont` + `TextColor` + `LineHeight` + `Transform`, plus `DamageNumber` and `DespawnAfter(Timer)`. OQ-CA-04 is resolved by Story 001 evidence: `TextColorLens` compiled and passed the scaffold test.

**Control Manifest Rules (Presentation Layer)**:
- Required: `DamageNumber` marker component on all damage number entities (enables `BoardRebuildRequested` cleanup sweep via `Query<Entity, With<DamageNumber>>`). `DespawnAfter(Timer)` initialized at spawn time from F2. No client-side RNG — use `jitter_table[event_id % 8]` for deterministic position jitter. Concurrent `TweenAnim` float targeting `Transform` + `TweenAnim` fade targeting `TextColor`.
- Forbidden: Entity despawn on float tween completion (F2 uses `max()` — always despawn at the later of the two). Entity reuse for multiple simultaneous damage numbers (each spawns its own entity). Direct `S2C*` subscription.
- Guardrail: `max(float_tween_duration_ms, fade_tween_duration_ms) + 50 < resolution_sub_step_duration_ms` (strict `<`). Startup assert in `CardAnimationsPlugin` validates this constraint. At defaults: 500+50=550 < 600 ✅.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [ ] **CA-8** — GIVEN a damage number entity pre-spawned with `float_tween_duration_ms=500`, `fade_tween_duration_ms=500`, and `DespawnAfter(Timer::new(Duration::from_millis(500), TimerMode::Once))`, WHEN `Time<Virtual>` is advanced by 500 ms and `App::update()` runs, THEN the entity is despawned (`World::get_entity()` returns `Err`). F2: `max(500, 500)=500`. Despawn timer set at spawn time from F2 — NOT from tween-completion event. **[BLOCKING]**
- [ ] **CA-9** — GIVEN `float_tween_duration_ms=400`, `fade_tween_duration_ms=600`, and `DespawnAfter(Timer::new(Duration::from_millis(600), TimerMode::Once))`, WHEN `Time<Virtual>` is advanced by 400 ms and `App::update()` runs, THEN entity still exists; WHEN advanced a further 200 ms (total 600 ms) and `App::update()` runs, THEN entity is despawned. F2: `max(400, 600)=600`. Entity must NOT be despawned at float tween's 400 ms completion. **[BLOCKING]**
- [ ] **CA-25** — GIVEN `DamageNumberSpawnRequested { target: entity, damage_value: 15, event_id: 0 }` is written, WHEN `App::update()` runs, THEN (a) exactly one entity with `DamageNumber` marker exists, AND (b) the entity has world-space text components `Text2d`, `TextFont`, `TextColor`, `LineHeight`, and `Transform`, AND (c) float and fade `TweenAnim` controllers are in `PlaybackState::Playing`, AND (d) entity carries `DespawnAfter(Timer)` initialized from F2 (`max(float_tween_duration_ms, fade_tween_duration_ms)`). F3 applies `jitter_table[event_id % 8]` to the spawn origin. **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule C-7, F2, F3 formulas:*

1. **F2 despawn formula (GDD §Formulas):** `despawn_delay_ms = max(float_tween_duration_ms, fade_tween_duration_ms)`. `DespawnAfter(Timer::new(Duration::from_millis(despawn_delay_ms), TimerMode::Once))` initialized at spawn time. Timer advances via `Time<Virtual>`. Entity despawned when timer finishes — NOT when either tween completes. Float tween may finish first; entity coasts at final float position while alpha fade continues draining to zero.

2. **Concurrent tween controllers (GDD Rule C-7):** `TweenAnim` targeting `Transform` (+60 px float, `EaseOutCubic`, 500 ms default) and `TweenAnim` targeting `TextColor` (`TextColorLens` fade) start in the same frame as entity spawn. Both tick independently.

3. **F3 position jitter (GDD F3):** When simultaneous `DamageNumberSpawnRequested` events target the same unit in one sub-step, each entity position offset = `jitter_table[event_id % 8]`. The 8-entry static `Vec2` table is defined in `card-animations.md`. No client-side RNG.

4. **Damage number entity layout:** Spawn one world-space text entity with `Text2d`, `TextFont`, `TextColor`, `LineHeight`, `Transform`, `DamageNumber`, and `DespawnAfter(Timer)`. Text value is `damage_value.to_string()`.

5. **Damage number origin (GDD V.1):** Float starts from unit torso world position plus F3 jitter offset, NOT health bar position. Prevents number occlusion of the HP bar.

6. **Startup assert (GDD §Tuning Knobs):** `max(float_tween_duration_ms, fade_tween_duration_ms) + 50 < resolution_sub_step_duration_ms` (strict `<`). Post-deserialization validation on complete `GameConfig` struct. Panic with descriptive message if violated. Note: `< sub_step` strict, not `<=`. 550 ms at sub_step=600 ms fails the assert.

7. **`DamageNumber` marker for cleanup (GDD Edge Cases §4):** `BoardRebuildRequested` handler includes `Query<Entity, With<DamageNumber>>` despawn sweep. Stale floating numbers must not survive reconnect.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001](story-001-plugin-scaffold-custom-lenses.md): Plugin scaffold, `TextColorLens` definition
- [Story 002](story-002-tween-cancel-replace-lifecycle.md): `BoardRebuildRequested` handler, `DamageNumber` entity cleanup sweep

---

## QA Test Cases

*Design gates resolved; all tests can be written during implementation.*

**CA-8 — F2 symmetric despawn: max(500, 500)=500 ms**

- Given: Pre-spawned damage number entity with `DespawnAfter(Timer::new(Duration::from_millis(500), TimerMode::Once))`, float `TweenAnim` (500 ms), text fade `TweenAnim` (500 ms), `DamageNumber` marker
- When: `advance_by(Duration::from_millis(500))`; `app.update()` called once
- Then: `world.get_entity(entity)` returns `Err` (despawned)
- Edge cases: advance 499 ms — entity still exists; advance 501 ms — entity still despawned; no `app.update` after advance — entity not yet despawned (system must tick)

**CA-9 — F2 asymmetric despawn: max(400, 600)=600 ms**

- Given: Pre-spawned damage number entity with `DespawnAfter` timer=600 ms (F2: max(400,600)=600), float `TweenAnim`=400 ms, text fade `TweenAnim`=600 ms
- When (t=400 ms): `advance_by(400)`; `app.update()` — entity still exists
- Then: `world.get_entity(entity)` is `Ok` (float tween completed at 400 ms but despawn timer at 400/600 ms — entity NOT despawned early)
- When (t=600 ms): `advance_by(200)`; `app.update()`
- Then: `world.get_entity(entity)` returns `Err` (despawned at 600 ms)
- Edge cases: verify no early despawn triggered by float tween callback — entity presence at 400 ms is the critical assertion

**CA-25 — DamageNumberSpawnRequested spawns entity with correct components**

- Given: World with `CardAnimationsPlugin`; target board entity pre-spawned; `DamageNumberSpawnRequested { target, damage_value: 15, event_id: 0 }` written via `MessageWriter`
- When: `app.update()` called once
- Then: (a) exactly one `DamageNumber`-marked entity in world; (b) entity has `Text2d`, `TextFont`, `TextColor`, `LineHeight`, and `Transform`; (c) float and fade `TweenAnim` controllers are in `PlaybackState::Playing`; (d) entity has `DespawnAfter(Timer)` initialized to `max(float_tween_duration_ms, fade_tween_duration_ms)` from `GameConfig`
- Edge cases: two simultaneous events (event_id=0, event_id=1) — two distinct entities; second entity's position offset by `jitter_table[1 % 8]`; damage_value=0 — entity still spawned

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/damage_number_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: [Story 002](story-002-tween-cancel-replace-lifecycle.md) DONE (concurrent controller contract); `DespawnAfter` component defined in GDD; OQ-CA-11 resolved; CA-25 design gates cleared (payload schema, text entity layout)
- Unlocks: None
