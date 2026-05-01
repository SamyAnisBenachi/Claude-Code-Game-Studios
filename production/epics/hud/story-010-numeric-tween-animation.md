# Story 010: Numeric Tween Animation

> **Epic**: HUD
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-010`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Numeric value updates (gold, reserved_gold, mana, reserve mana) tween over ≤300ms via `bevy_tweening`. The backing `f32` field in `GoldDisplayState` is the tween target. A change-detection system in `StateSync` reads the current backing value each frame and writes the formatted string to `Text`/`TextSpan`. Cancel-and-replace via `Animator::set_tweenable()` — never despawn+respawn. Do NOT implement tween directly on `GoldDisplayState` via `Lens<GoldDisplayState>` — three simultaneous writers conflict.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `bevy_tweening 0.18` — verify `Animator<T>::set_tweenable()` method name before implementing (ADR-021 implementation gate: run `cargo check` against a stub). `Lens<T>` `lerp()` method name — verify against crate source. `Animator<T>` is added as a component; do NOT write `Transform` or `GoldDisplayState` directly while an `Animator` is active (animator overwrites next frame).

**Control Manifest Rules (Presentation Layer)**:
- Required: Tween cancel-and-replace via `set_tweenable()`. Backing `f32` in `GoldDisplayState` is the tween target — NOT the `Text` string directly. Change-detection in `StateSync` derives string from the current `f32` value each frame.
- Forbidden: Never despawn+respawn for tween cancel-and-replace. Never write `GoldDisplayState` directly while `Animator<GoldDisplayState>` is active. Never attach `Animator<T>` to phase label, round counter, or dot entities.
- Guardrail: Tween duration ≤ 300ms. No animation competes with central decision surfaces (auction panel, hand fan, board).

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story — all ADVISORY (correctness gates are in Stories 002 and 007):*

- [ ] **HUD-12** (ADVISORY): GIVEN HUD in ECONOMY_BASIC, WHEN a gold or mana value changes, THEN the displayed value animates to the new number within 300ms; verified by elapsed-time measurement between message receipt and final `GoldDisplayState` value stabilising.
- [ ] **Cancel-and-replace** (ADVISORY): If a new authoritative value arrives while a tween is in progress, the current tween is cancelled and a new tween starts from the current `f32` value (wherever interpolation has reached) to the new authoritative value. Duration ≤ 300ms. No string parsing needed.
- [ ] **Layout zones legible** (ADVISORY): At 1280×720 and 1920×1080, all four HUD zones are legible with no overlap with board or sister UIs. Confirmed by screenshots.

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

- Do NOT implement tween as `Lens<GoldDisplayState>` directly — three simultaneous writers on same component (`Animator`, `handle_gold_update_system`, `handle_gold_broadcast_system`).
- Implementation pattern: use a separate `GoldTweenTarget { gold: f32, reserved_gold: f32 }` component (or equivalent naming) as the `Animator`'s target. A `StateSync` system reads `GoldTweenTarget.gold` and writes the derived string to `Text`. `GoldDisplayState` remains the authoritative backing; when a new value arrives, start tween from `GoldTweenTarget.gold` (current display value) toward `GoldDisplayState.gold` (new authoritative value).
- `Animator::set_tweenable(new_tween)` for cancel-and-replace. Verify method name with `cargo check` before implementing.
- Duration: `hud_tween_duration_ms` from `HudConfig` (default 300ms).
- Do NOT animate: phase label, round counter, scoreboard dots. These update instantly (Stories 003, 004).
- GAME_OVER snap (implemented in Story 007): when FROZEN mode is entered, cancel tween and snap directly. Story 010 does not own this invariant.
- `bevy_tweening` implementation gate (from ADR-021): run `cargo check` against a `GoldTweenLens` stub before committing to implementation details.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 007]: GAME_OVER snap on in-flight tween (BLOCKING correctness invariant — must be done there)
- [Story 002]: Multi-update collapse (BLOCKING correctness invariant — last-value-wins before tween starts)

---

## QA Test Cases

*Written by qa-lead at story creation. All ADVISORY.*

**Manual check: HUD-12** — Tween duration ≤ 300ms
  - Setup: HUD visible in ECONOMY_BASIC; own gold = 5g
  - When: `S2CGoldUpdate{gold=15}` processed
  - Verify: Gold label animates from "5g" toward "15g" over approximately 300ms
  - Pass condition: Label stabilises at "15g" within 300ms of message receipt; no abrupt snap

**Manual check: Cancel-and-replace** — Mid-tween new value
  - Setup: Tween in progress from 5.0 toward 15.0 (currently ~8.0 displayed)
  - When: New `S2CGoldUpdate{gold=20}` arrives mid-tween
  - Verify: Tween restarts from ~8.0 toward 20.0; no jump to 5.0 or snap to 15.0
  - Pass condition: Smooth continuation without visible reset

**Manual check: Layout zones** — 1280×720 and 1920×1080
  - Setup: HUD visible in ECONOMY_AUCTION mode (widest label format); place some units and objectives
  - Verify: Top-left, top-center, top-right, bottom-left zones are visible with no overlap; no overlap with board units, hand fan, auction bid box
  - Pass condition: Screenshot shows all four zones legible at both resolutions

---

## Test Evidence

**Story Type**: Visual/Feel
**Required evidence**: `production/qa/evidence/numeric-tween-evidence.md` (screenshots at 1280×720 and 1920×1080) + lead sign-off

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (`GoldDisplayState` backing fields), Story 007 (GAME_OVER snap already handled — this story adds the smooth animation on top)
- Unlocks: None (polish story — final HUD story)
