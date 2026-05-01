# Epic: Hand UI

> **Layer**: Presentation
> **GDD**: design/gdd/hand-ui.md
> **Architecture Module**: `client/src/presentation/hand/` — `HandUiPlugin` (sub-plugin #3 inside `PresentationPlugin`)
> **Status**: Ready
> **Stories**: Not yet created — run `/create-stories hand-ui`

## Overview

Hand UI implements the client-side card fan display and all card-play interaction across every game phase. It lives inside the `PresentationPlugin` as `HandUiPlugin`, registered third (after `CardAnimationsPlugin` and `BoardRenderingPlugin`) because it reads `Res<CardAtlas>` and `Res<BoardLayout>` that `BoardRenderingPlugin` inserts on session entry. All 10 fan slots, 9 DRAFT_INITIAL grid slots, and 1 drag sprite are pre-pooled at session start and toggled via `Visibility` — no per-round spawn/despawn. Phase transitions arrive via `Res<CurrentClientPhase>` (written by the shared `phase_sink_system`); Hand UI never drains `MessageReceiver<S2CPhaseChanged>` directly. The drag sprite is a bevy_ui `Node`, not a world-space `Sprite`, to preserve z-order above board content during PLACEMENT. The epic covers the full six-state machine (`HIDDEN → GRID → PASSIVE → PASSIVE_LOCKED → STAGING → SUBMITTED`), the drag-to-stage flow, Instant card staging, reserve mana split strip, submit pre-validation, PLACEMENT timer urgency, reconnect rebuild, and all associated visual state components.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md) | `HandUiPlugin` composes into `PresentationPlugin`; reads `Res<CardAtlas>` + `Res<BoardLayout>` from `BoardRenderingPlugin`; phase state via `Res<CurrentClientPhase>` only; drag sprite is bevy_ui `Node`; pre-pooled entities; `PresentationSet` ordering contract | HIGH |
| [ADR-002: Client-Server Authority](../../docs/architecture/adr-002-client-server-authority.md) | Client is a view — no hand state mutation; all visual changes driven by S2C messages | LOW |
| [ADR-004: Asset Loading Pipeline](../../docs/architecture/adr-004-asset-loading-pipeline.md) | `bevy_asset_loader` LoadingState; `Res<CardAtlas>` promoted before sub-plugins initialise | MEDIUM |
| [ADR-009: RSM Phase State](../../docs/architecture/adr-009-rsm-phase-state.md) | `S2CPhaseChanged` → shared `phase_sink_system` → `Res<CurrentClientPhase>`; Hand UI reads resource, never Lightyear buffer | HIGH |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-HU-001 | Hand fan pre-pooled: 10 card slots + 9 DRAFT_INITIAL grid slots; fixed entity count regardless of hand size | ADR-021 Impl Guideline 3 ✅ |
| TR-HU-002 | PLACEMENT drag-to-stage state machine: Idle → Dragging → Staged → Committed; cursor-to-cell mapping via `Res<BoardLayout>` | ADR-021 (rendering boundary + BoardLayout) ✅ |
| TR-HU-003 | Instant card staging: cards with Instant type stage to fan plate; no board cell highlight | ADR-021 + ADR-002 ✅ |
| TR-HU-004 | Reserve mana split control: +/- strip per staged card; ceiling = `player.reserve_mana − sum(other_staged.reserve_amount)` | ADR-021 + ADR-019 ✅ |
| TR-HU-005 | DRAFT_INITIAL grid overlay: 9-card display, purchase feedback, `purchase_timeout_ms` revert | ADR-021 + ADR-004 ✅ |
| TR-HU-006 | TargetUnit hover feedback: `TargetUnitHover` marker on valid units; no `BoardCellHighlighted` markers | ADR-021 (rendering boundary) ✅ |
| TR-HU-007 | PLACEMENT timer urgency: 5s threshold fires `TimerUrgencyAudio` event exactly once; tween to Amber/Crimson | ADR-021 (AnimationTick set + tween lifecycle) ✅ |
| TR-HU-008 | Submit pre-validation: client-side mana/reserve overdraw check before `C2SSubmitPlacement`; server validation authoritative | ADR-002 + ADR-021 ✅ |

**Untraced Requirements**: None

## Pre-Implementation Gates

These are not design gaps — the GDD is Approved. They gate specific stories within the epic:

| Gate | Blocks | Action Required |
|------|--------|-----------------|
| **OQ8** — `S2CActivationRejected` not registered in NP GDD | HU-28 / HU-28b (activation-lock story) | Add `S2CActivationRejected` to `design/gdd/network-protocol.md` before the activation-lock story opens |
| **OQ5/OQ6** — Card data pipeline ADR; atlas-sharing confirmation | Asset-pipeline story (card TextureAtlas frame index resolution) | ADR-021 resolves direction (`Res<CardAtlas>` shared from `BoardRenderingPlugin`); confirm `CardAtlas::frame_index(card_id)` shared method exists before asset story starts |

## Key ADR-021 Constraints for This Epic

Implementation notes that stories must embed (from ADR-021 and GDD Rule 1):

1. **Plugin registration order is a contract.** `HandUiPlugin` is third — `BoardRenderingPlugin` must already be registered when `HandUiPlugin::build()` runs, or `Res<CardAtlas>` / `Res<BoardLayout>` are not yet inserted (runtime panic).
2. **Session-scoped Resources.** All systems reading `Res<CardAtlas>` or `Res<BoardLayout>` must be in `in_state(ClientState::InSession)`.
3. **No `MessageReceiver<S2CPhaseChanged>` in HandUiPlugin.** Phase state is read from `Res<CurrentClientPhase>` exclusively.
4. **Drag sprite is a bevy_ui `Node`.** Not a world-space `Sprite`. This preserves z-order above board sprites during PLACEMENT (ADR-021 Impl Guideline 8).
5. **Pre-pooled entities.** All 10 fan slots + 9 grid slots + 1 drag sprite spawned on `OnEnter(ClientState::InSession)`, despawned on `OnExit`. Toggle via `Visibility` only — no per-round spawn/despawn.
6. **Tween cancel-and-replace via `set_tweenable()`.** Never despawn + respawn hand entities mid-animation — game-state components (slot state markers) must survive the cancel.
7. **`ui_picking` feature guard.** Any `PickingBehavior` insertion must be inside `#[cfg(feature = "ui_picking")]`.
8. **`GhostPlacementChanged` is a Bevy-internal message** (`MessageWriter<T>` / `MessageReader<T>`), not a Lightyear message. Board Rendering reads it; Hand UI writes it.

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All acceptance criteria from `design/gdd/hand-ui.md` are verified (HU-01 through HU-30, excluding HU-28/HU-28b until OQ8 resolves)
- All Logic and Integration stories have passing test files in `tests/unit/hand-ui/` or `tests/integration/hand-ui/`
- All Visual/Feel stories have evidence docs with lead sign-off in `production/qa/evidence/`
- Pre-implementation gates OQ8 and OQ5/OQ6 are resolved and their gated stories are implemented and closed

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Plugin Scaffold — Pre-Pooled Entity Spawning](story-001-plugin-scaffold.md) | Logic | Ready | ADR-021 |
| 002 | [Fan Layout Formula — Card Position & Rotation](story-002-fan-layout-formula.md) | Logic | Ready | ADR-021 |
| 003 | [Phase State Machine — Visibility & Input Gating](story-003-phase-state-machine.md) | Logic | Ready | ADR-021, ADR-009 |
| 004 | [DRAFT_INITIAL Grid — Display & Purchase Flow](story-004-draft-initial-grid.md) | Integration | Ready | ADR-021, ADR-004 |
| 005 | [PLACEMENT Entry — Submit Button & Core Stage/Unstage](story-005-placement-submit-core.md) | Logic | Ready | ADR-021, ADR-002 |
| 006 | [PLACEMENT Drag — Highlight Sets & TargetUnit](story-006-placement-drag-highlights.md) | Logic | Ready | ADR-021 |
| 007 | [PLACEMENT Instant Card Staging](story-007-placement-instant-staging.md) | Logic | Ready | ADR-021 |
| 008 | [PLACEMENT Un-Staging — Board Ghosts & Instant Fan Slot](story-008-placement-unstaging.md) | Integration | Ready | ADR-021 |
| 009 | [PLACEMENT Timer — Urgency, Grace Window & Submit Checkmark](story-009-placement-timer.md) | Integration | Ready | ADR-021 |
| 010 | [Submit Pre-Validation — Mana & Reserve Checks](story-010-submit-prevalidation.md) | Logic | Ready | ADR-021, ADR-002 |
| 011 | [Reserve Mana Split Strip — Per-Staged-Card Controls](story-011-reserve-mana-strip.md) | Logic | Ready | ADR-021 |
| 012 | [Activation Lock — DRAFT_SHOP Instant Card Lock & Timeout](story-012-activation-lock.md) | Integration | Blocked | ADR-021, ADR-002 |
| 013 | [Reconnect Rebuild — PLACEMENT State Recovery](story-013-reconnect-rebuild.md) | Integration | Ready | ADR-021, ADR-002, ADR-009 |

**Story counts**: 8 Logic · 5 Integration · 1 Blocked (OQ8)
**Dependency order**: 001 → 002, 003; 003 → 004, 005, 013; 005 → 006, 007, 008, 009, 010, 011
