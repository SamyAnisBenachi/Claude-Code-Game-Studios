# Epic: Card Animations

> **Layer**: Presentation
> **GDD**: design/gdd/card-animations.md
> **Architecture Module**: `client/src/ui/anim/`
> **Status**: Ready
> **Stories**: 9 stories created — 6 Ready, 3 Blocked

## Overview

Card Animations is the `bevy_tweening`-based animation pipeline living entirely in
`client/src/ui/anim/`. It owns the shared library of lenses, easing curves, duration
constants, the `AnimQueue` Resource, and the `Animator<T>` cancel-and-replace tween
lifecycle that all five presentation sub-systems consume. It serves Hand UI (card-to-fan
slides, drag-lift, snap-back), Board Rendering (unit advance, fog-lift, HP-bar fill,
objective destruction), Combat Resolution visuals (placement-reveal flip, damage-number
floats, death fades, COUNTERATTACK and RANGE projectiles), Shop/Auction UI (panel
transitions, gold-counter ticking, timer-bar ease, NO BIDS desaturation), and Keyword
System visuals (REPEL/ATTRACT displacement, TRAP flip). It contains zero gameplay logic,
never delays a phase transition, never holds server-authoritative state, and is never
replayed on reconnect — its sole job is to translate the server's authoritative S2C event
stream into legible motion within strict per-phase animation budgets.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-021: Presentation Layer Architecture | `PresentationPlugin` composing five ordered sub-plugins; `bevy_tweening` `Animator<T>` cancel-and-replace contract defined centrally; `AnimQueue` Resource drives sub-step playback; custom `SpriteAlphaLens`; `CurrentClientPhase` single phase-sink pattern; `BoardLayout` `Res` shared across sub-systems | HIGH |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-CAN-001 | Decoration Test: every animation must pass ≥1 of 5 informational tests (state-change, removal, parallel, input-gating, teach-by-showing) | ADR-021 ✅ |
| TR-CAN-002 | Per-phase animation budget enforced: PLACEMENT hard cap 250 ms; RESOLUTION 600 ms per sub-step; `AnimQueue` Resource | ADR-021 ✅ |
| TR-CAN-003 | Input-gating rules: bid buttons gated on settlement; card slides non-blocking; UI input map per phase | ADR-021 ✅ |
| TR-CAN-004 | Mandatory simultaneous-start parallelism for placement reveal; max 2 UI region tweens at once; `AnimGroup` drain | ADR-021 ✅ |
| TR-CAN-005 | Placement-reveal flip 80–100 ms; 5 lanes simultaneous; `LaneCell` + `BoardLayout.cell_to_world()` coordinate source | ADR-021 ✅ |
| TR-CAN-006 | Sub-step pause gates: 100 ms after SS1, SS5, SS6 settlement before next sub-step plays | ADR-021 ✅ |
| TR-CAN-007 | `bevy_tweening 0.18` hard dependency; custom `SpriteAlphaLens` (no built-in alpha lens); 5 custom lenses; lifecycle managed via `DamageNumberSpawnRequested` + `GroupDrainedSignal` | ADR-021 ✅ |

> **TR registry note**: The `tr-registry.yaml` comment on TR-CAN says "ADR coverage: NONE —
> Presentation Layer ADR (ADR-021) required". This comment was written before ADR-021 was
> created. ADR-021 is Accepted (2026-04-30) and covers all 7 TR-CAN entries. The comment is
> stale and safe to ignore.

## Pre-Implementation Gates

Two items in ADR-021 must be verified with `cargo check` before the first Card Animations
story can be opened:

1. **`Lens<T>::lerp()` method name** — bevy_tweening is a third-party crate not covered by
   engine-reference docs. Verify the method name is still `lerp()` in the 0.18-compatible
   release by running `cargo check` against a `SpriteAlphaLens` stub.
2. **`Animator<T>::set_tweenable()`** — verify this API exists in the target bevy_tweening
   version with a stub `cargo check` call.

These are story-level gates. The epic itself is Ready; the first story must include a
`cargo check` verification step before any implementation begins.

## Definition of Done

This epic is complete when:

- All stories are implemented, reviewed, and closed via `/story-done`
- All acceptance criteria from `design/gdd/card-animations.md` are verified
- All Logic and Integration stories have passing test files in `tests/`
- All Visual/Feel stories have screenshot evidence + lead sign-off in `production/qa/evidence/`
- Both ADR-021 pre-implementation gates are cleared (cargo check passing)
- `AnimQueue`, all 5 custom lenses, and the cancel-and-replace contract are implemented
  and verified against the GDD phase budgets

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [CardAnimationsPlugin scaffold + 5 custom lenses + cargo-check gates](story-001-plugin-scaffold-custom-lenses.md) | Logic | Ready | ADR-021 |
| 002 | [Tween cancel-replace lifecycle](story-002-tween-cancel-replace-lifecycle.md) | Logic | Ready | ADR-021 |
| 003 | [Simultaneous Transform controller animation](story-003-simultaneous-track-animation.md) | Logic | Ready | ADR-021 |
| 004 | [AnimQueue RESOLUTION drain + GAME_OVER skip path](story-004-anim-queue-resolution-drain.md) | Logic | Ready | ADR-021 |
| 005 | [Placement-reveal parallelism + PLACEMENT 250ms budget + PlacementCancelAllAnimsRequested](story-005-placement-reveal-parallelism.md) | Integration | Ready | ADR-021 |
| 006 | [Multi-objective stagger reveal (F1 formula)](story-006-objective-stagger-reveal.md) | Logic | Ready | ADR-021 |
| 007 | [Damage number lifecycle (F2 despawn timer)](story-007-damage-number-lifecycle.md) | Logic | Ready | ADR-021 |
| 008 | [Input-gating: timer bar, drag latency, bid button state, de-hover cancel-replace](story-008-input-gating.md) | Integration | Ready | ADR-021 |
| 009 | [CI boundary enforcement (no direct S2C subscription)](story-009-ci-boundary-enforcement.md) | Integration | Ready | ADR-021 |

## Next Step

Run `/story-readiness production/epics/card-animations/story-001-plugin-scaffold-custom-lenses.md` then `/dev-story` to begin implementation. Work through stories in dependency order — each story's `Depends on:` field specifies what must be DONE before it can start.

**Blocker-clear note (2026-05-02):**
1. OQ-CA-01/OQ-CA-05 resolved by Story 001/002 evidence: implementation uses `bevy_tweening v0.15.0` for Bevy 0.18, with `TweenAnim`, `PlaybackState`, and `TweenState`.
2. OQ-CA-02 resolved: `Tracks<T>` is removed; Story 003 now uses independent `TweenAnim` controller entities with `AnimTarget::component`.
3. Story 005 unblocked: `board-rendering.md` defines `PlacementRevealAnimReady`, `PlacementRevealEntry`, `LaneCell`, and confirms `BoardLayout.cell_to_world(lane, cell) -> Vec2`.
4. Story 007 unblocked: F3 jitter table, `DamageNumberSpawnRequested` payload, `Text2d` layout, and `DespawnAfter(Timer)` are defined in the GDDs.
