# Story 001: CardAnimationsPlugin scaffold + 5 custom lenses + cargo-check gates

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
**ADR Decision Summary**: `PresentationPlugin` composing five ordered sub-plugins; `bevy_tweening` `Animator<T>` cancel-and-replace contract defined centrally; custom `SpriteAlphaLens` (no built-in alpha lens); `AnimQueue` Resource drives sub-step playback; `CardAnimationsPlugin` must be first in registration order.

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: HIGH
**Engine Notes**: bevy_tweening is a third-party crate not covered by engine-reference docs. Two pre-implementation gates from ADR-021 MUST be verified with `cargo check` BEFORE implementation begins:
1. `Lens<T>::lerp()` method name — historical name; verify still correct in 0.18-compatible release.
2. `Animator<T>::set_tweenable()` — verify API exists in target bevy_tweening version.
Also resolve: OQ-CA-05 (`bevy_tweening = "0.18"` on crates.io — highest-priority OQ, blocks all implementation); OQ-CA-01 (`AnimatorState` enum name + public visibility, needed for test harnesses in Stories 002+); OQ-CA-04 (`TextColor` satisfies `Component` bound for `TextColorLens`); OQ-CA-10 (`Sprite.color.with_alpha()` path — `set_alpha()` does not exist in Bevy 0.18). ADR-021 already specifies `target.color = target.color.with_alpha(alpha)` as the correct pattern.

**Control Manifest Rules (Presentation Layer)**:
- Required: `CardAnimationsPlugin` must be first in `PresentationPlugin` registration order (runtime panic otherwise). `SpriteAlphaLens::lerp()`: use `target.color = target.color.with_alpha(alpha)`. `Color::srgba` / `Color::srgb` constructors. `app.add_message::<T>()` for all intra-client `Message` types (NOT `add_event`). `#[derive(Message)]` on all intra-client event structs.
- Forbidden: Any `*Bundle` type (`SpriteBundle`, `NodeBundle`, etc.). `Color::rgba()`. `EventWriter<T>` / `EventReader<T>` / `Events<T>` — do not exist in Bevy 0.17+. `Handle<TextureAtlas>` (removed in 0.18).
- Guardrail: Presentation steady-state < 1 ms per frame.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [x] **CA-1** — GIVEN `CardAnimationsPlugin` is registered, WHEN `App::new()` builds with the plugin and executes one update, THEN the app completes without panic; each of the 5 custom lens types (`SpriteAlphaLens`, `BackgroundColorAlphaLens`, `SpriteColorLens`, `TransformScaleXLens`, `TextColorLens`) can be constructed in a `World`-based unit test and inserted into a `Tween` without compile or runtime error. **[BLOCKING]**
- [x] **CA-20** — GIVEN the client app runs with `CardAnimationsPlugin`, WHEN each domain event type registered by upstream plugins is fired in a controlled test, THEN no missing-consumer panic or warning is logged. (Smoke test — confirms all `Message` types registered before `CardAnimationsPlugin` readers run.) **[ADVISORY]**

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines and GDD Rule C-9:*

1. **`CardAnimationsPlugin::build()` responsibilities (GDD Rule C-9):**
   - Add `bevy_tweening::TweeningPlugin` if not already registered.
   - Register the 5 custom lens types.
   - `app.add_message::<T>()` for each domain `Message` type Card Animations consumes. `add_message` is idempotent in Bevy 0.18 — no-op if already registered.
   - Insert `StagedObjectiveRevealQueue` resource (`app.insert_resource(StagedObjectiveRevealQueue::default())`).
   - `app.add_message::<GroupDrainedSignal>()`.

2. **5 custom lenses to implement (GDD Rule C-6):**
   - `SpriteAlphaLens { start: f32, end: f32 }` — target `Sprite`, `target.color = target.color.with_alpha(alpha)`. `EaseOutQuad` exits, `EaseInQuad` entrances.
   - `BackgroundColorAlphaLens { start: f32, end: f32 }` — target `BackgroundColor`. Clamp `[0.0, 1.0]` at lens level. `EaseOutCubic`.
   - `SpriteColorLens { start: Color, end: Color }` — target `Sprite`, full RGBA interpolation. 300 ms default.
   - `TransformScaleXLens { start: f32, end: f32 }` — target `Transform`, `transform.scale.x` only (Y/Z untouched). Clamp `>= 0.0`.
   - `TextColorLens { start: Color, end: Color }` — target `TextColor` newtype. `EaseOutCubic`.

3. **Alpha API (OQ-CA-10):** `SpriteAlphaLens` MUST use `target.color = target.color.with_alpha(alpha)`. ADR-021 Key Interfaces already specifies this. No `set_alpha()` method exists in Bevy 0.18.

4. **`bevy_tweening = "0.18"` in `client/Cargo.toml`.** Verify crate is published and Bevy-0.18-compatible before committing (OQ-CA-05).

5. **Module structure (GDD Rule C-9):** Single `CardAnimationsPlugin`, internal modules `lenses/`, `animators/`, `queue/`, `events/`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002](story-002-tween-cancel-replace-lifecycle.md): Tween cancel-replace lifecycle (`set_tweenable`, `BoardRebuildRequested` handler)
- [Story 004](story-004-anim-queue-resolution-drain.md): `AnimQueue` Resource, `ResolutionExecuting` drain system
- All upstream event handler systems (those produce tweens; Story 001 only scaffolds the infrastructure)

---

## QA Test Cases

*Written by qa-lead at story creation.*

**AC-1a — Plugin builds without panic**

- Given: Minimal `App` with `CardAnimationsPlugin` registered; no other plugins
- When: `App::new()` + `plugin.build(&mut app)` + `app.update()` called once
- Then: No panic; clean exit

**AC-1b — Each custom lens constructable and insertable into a Tween**

- Given: `World::new()`; entity spawned with the relevant target component (`Sprite` for `SpriteAlphaLens`/`SpriteColorLens`; `BackgroundColor` for `BackgroundColorAlphaLens`; `Transform` for `TransformScaleXLens`; `TextColor` for `TextColorLens`)
- When: `Tween::new(Duration::from_millis(100), EaseFunction::Linear, lens_instance)` constructed; `Animator::new(tween)` inserted on entity
- Then: No compile error; no runtime panic; `world.get::<Animator<T>>(entity)` returns `Some(_)`
- Edge cases: `lerp()` at ratio=0.0 → start value; at ratio=1.0 → end value; `BackgroundColorAlphaLens` at ratio=1.5 → clamped to 1.0; `TransformScaleXLens` at ratio=-0.1 → clamped to 0.0; `TextColorLens` — confirm `TextColor` satisfies `Component` bound (OQ-CA-04)

**AC-20 — Smoke test: no missing-consumer warning**

Manual check: All domain Message types fire without consumer error
  - Setup: App with `CardAnimationsPlugin` + all upstream plugins (or test harness that registers consumed message types manually); fire one instance of each domain event type
  - Verify: No panic; no missing-consumer warning logged
  - Pass condition: `app.update()` completes without error for all domain event types

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/plugin_scaffold_test.rs` — must exist and pass

**Status**: [x] Created and passed locally with `cargo test -p client --test card_animations_plugin_scaffold_test --target-dir target\codex-card-animations-test`

---

## Dependencies

- Depends on: None — first story; resolves all ADR-021 cargo-check gates and OQ-CA-01/05
- Unlocks: [Story 002](story-002-tween-cancel-replace-lifecycle.md), [Story 004](story-004-anim-queue-resolution-drain.md)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 2/2 passing. CA-1 covered by the plugin/lens unit tests; CA-20 covered for the scaffolded registered domain messages.
**Deviations**: Advisory only: current `design/gdd/card-animations.md` also names `TimerColorZoneRequested` and `NoBidsTransitionRequested`, but those message stubs are not present in the Story 001 scaffold. Advisory only: story/GDD/TR text says `bevy_tweening 0.18`, while the workspace pins `bevy_tweening = "0.15"` as the Bevy 0.18-compatible release.
**Test Evidence**: Logic: `tests/unit/card-animations/plugin_scaffold_test.rs`; `cargo test -p client --test card_animations_plugin_scaffold_test --target-dir target\codex-card-animations-test` passed 8/8 tests.
**Code Review**: Skipped - Lean mode.
