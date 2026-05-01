# Story 003: Simultaneous-track animation (Tracks<Transform>)

> **Epic**: Card Animations
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Mandatory simultaneous-start parallelism for same-event animations; `Tracks<T>` required when one entity needs two `Transform` animations simultaneously (REPEL displacement + simultaneous advance); NOT two `Animator<Transform>` components on same entity (would conflict).

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: HIGH
**Engine Notes**: ⚠️ **BLOCKED — OQ-CA-02**: `Tracks<T>` API in bevy_tweening 0.18 is unconfirmed. GDD Rule C-7 requires `Tracks<Transform>` for same-component parallel animations; if removed, workaround is child entities for the secondary transform (requires design review before implementation). Also blocked on OQ-CA-01 (`AnimatorState` enum name) for test harness. **Must resolve OQ-CA-02 before any implementation. Must resolve OQ-CA-01 before writing tests. Story 001 resolves both as part of its cargo-check gates.**

**Control Manifest Rules (Presentation Layer)**:
- Required: Use `Tracks<T>` for same-entity same-component simultaneous animations. `Animator<T>` cancel-and-replace via `set_tweenable`. Simultaneous-start: all tweens in same `AnimGroup` start in same frame — no `apply_deferred` between.
- Forbidden: Two `Animator<Transform>` on the same entity (component conflict). Despawn+respawn for animation management. Any `*Bundle` type.
- Guardrail: Presentation steady-state < 1 ms per frame.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [ ] **CA-15** — GIVEN a unit entity needs simultaneous REPEL displacement (`Tween<Transform>`) and placement-reveal flip (`SpriteColorLens` + `TransformScaleXLens`), WHEN both tweens are spawned, THEN all three animators exist on the entity simultaneously and are in `AnimatorState::Playing` after one tick. **[BLOCKING]**
- [ ] **CA-18** — GIVEN a unit entity at `Transform::IDENTITY` requires two simultaneous `Tween<Transform>` animations wrapped in a `Tracks<Transform>` (tween A: X 0→100, 600 ms; tween B: Y 0→60, 600 ms), WHEN `Time<Virtual>` is advanced by 16 ms and `App::update()` runs, THEN: (a) exactly one `Animator<Transform>` on the entity, AND (b) `Transform.translation.x` in range (0.0, 100.0) exclusive, AND (c) `Transform.translation.y` in range (0.0, 60.0) exclusive. **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule C-7:*

1. **BLOCKED — resolve OQ-CA-02 first.** Confirm `Tracks<T>` is present in bevy_tweening 0.18 and wraps into `Animator<T>` directly. If removed, workaround = child entity for secondary transform (requires design review before implementing this story).

2. **`Tracks<Transform>` usage (GDD Rule C-7):** When one entity needs two simultaneous `Transform` animations, wrap both in `Tracks<Transform>` and pass to a single `Animator<Transform>`. Never add two `Animator<Transform>` to the same entity.

3. **Simultaneous-start invariant (GDD Rule C-4):** Both tweens in a `Tracks<Transform>` start in the same frame. Events in the same `AnimGroup` spawn tweens in a single system pass — no `apply_deferred` injection between entries.

4. **Mixed Animator<Transform> + Animator<Sprite> still valid (Story 002):** REPEL displacement (`Animator<Transform>`) and placement-reveal color flip (`Animator<Sprite>` via `SpriteColorLens`) can coexist because they target different components. The `TransformScaleXLens` squash for the placement-reveal must be bundled into the `Tracks<Transform>` alongside the REPEL translation tween — not a separate `Animator<Transform>`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002](story-002-tween-cancel-replace-lifecycle.md): Single-component cancel-replace; concurrent `Animator<Transform>`/`Animator<Sprite>` on different components
- [Story 005](story-005-placement-reveal-parallelism.md): 5-lane placement-reveal parallelism (uses single-component animators per lane)

---

## QA Test Cases

*PRE-IMPLEMENTATION GATE — tests to be written once OQ-CA-01 and OQ-CA-02 resolve via Story 001.*

**CA-15 — REPEL + placement-reveal flip: 3 animators on 1 entity simultaneously**

- Given: Entity with `Sprite` and `Transform`; `DisplacementAnimRequested` spawns `Animator<Transform>`; `PlacementRevealAnimReady` for same entity spawns `Animator<Sprite>` (`SpriteColorLens`) + `Animator<Transform>` scale (via `Tracks<Transform>`); `Time<Virtual>` advanced 1 ms
- When: `app.update()` called once
- Then: `Animator<Transform>` in `Playing`; `Animator<Sprite>` in `Playing` (verify exact component count matches bevy_tweening API once OQ-CA-02 resolved)
- Edge cases: only displacement fires alone — no cross-contamination; only reveal fires alone — no displacement spawned

**CA-18 — Tracks<Transform> advances X and Y simultaneously**

- Given: Entity at `Transform::IDENTITY`; `Tracks<Transform>` wrapping tween A (X: 0→100, 600 ms) and tween B (Y: 0→60, 600 ms); `Animator<Transform>` installed; `Time<Virtual>` advanced 16 ms
- When: `app.update()` called once
- Then: (a) exactly one `Animator<Transform>` on entity; (b) `Transform.translation.x` in (0.0, 100.0); (c) `Transform.translation.y` in (0.0, 60.0)
- Edge cases: advance 600 ms → x=100.0, y=60.0 exactly; advance 0 ms → x=0.0, y=0.0 (no movement)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- Logic: `tests/unit/card-animations/tracks_animation_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: [Story 002](story-002-tween-cancel-replace-lifecycle.md) must be DONE (cancel-replace contract established); OQ-CA-02 resolved (Tracks<T> API confirmed); OQ-CA-01 resolved (AnimatorState enum for tests)
- Unlocks: None directly (enables REPEL/ATTRACT displacement animations once unblocked)
