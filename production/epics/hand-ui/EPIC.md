# Epic: Hand UI

> **Layer**: Presentation
> **GDD**: design/gdd/hand-ui.md
> **Architecture Module**: `client/src/presentation/hand/` — `HandUiPlugin` (sub-plugin #3 inside `PresentationPlugin`)
> **Status**: Ready
> **Stories**: Created — see Stories table below

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
| TR-HU-004 | Reserve mana split control: +/- strip per staged card; ceiling = `player.reserve_mana - sum(other_staged.reserve_mana_spend)` | ADR-021 + ADR-019 ✅ |
| TR-HU-005 | DRAFT_INITIAL grid overlay: 9-card display, purchase feedback, `purchase_timeout_ms` revert | ADR-021 + ADR-004 ✅ |
| TR-HU-006 | TargetUnit hover feedback: `TargetUnitHover` marker on valid units; no `BoardCellHighlighted` markers | ADR-021 (rendering boundary) ✅ |
| TR-HU-007 | PLACEMENT timer urgency: 5s threshold fires `TimerUrgencyAudio` event exactly once; tween to Amber/Crimson | ADR-021 (AnimationTick set + tween lifecycle) ✅ |
| TR-HU-008 | Submit pre-validation: client-side mana/reserve overdraw check before `C2SSubmitPlacement`; server validation authoritative | ADR-002 + ADR-021 ✅ |

**Untraced Requirements**: None

## Sprint 6 Accessibility Gate

QA-COND-0005 remains Open for Standard-tier accessibility remediation. Story 014
is Complete and evidences A11Y-ST-14 PLACEMENT staged disclosure as a Hand UI
UI story. Story 015 scopes the Hand UI slice of A11Y-ST-02 card cost, ATK, HP,
and keyword readability across hand/fan, staged fan, and acquisition feedback
surfaces before the final cross-surface Presentation evidence pass.

## Pre-Implementation Gates

These are not design gaps — the GDD is Approved. They gate specific stories within the epic:

| Gate | Blocks | Action Required |
|------|--------|-----------------|
| **OQ8** — `S2CActivationRejected` not registered in NP GDD | HU-28 / HU-28b (activation-lock story) | Add `S2CActivationRejected` to `design/gdd/network-protocol.md` before the activation-lock story opens |
| **OQ5/OQ6** — Card data pipeline ADR; atlas-sharing confirmation | Asset-pipeline story (card TextureAtlas frame index resolution) | ADR-021 resolves direction (`Res<CardAtlas>` shared from `BoardRenderingPlugin`); confirm `CardAtlas::frame_index(card_id)` shared method exists before asset story starts |
| **HAND-UI-010 prerequisites** — placement submit pre-validation prerequisites | Story 010 | Resolved; Story 010 is Complete and its submit pre-validation behavior must be preserved by Stories 014 and 015 |

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
- All Visual/Feel and UI stories have evidence docs with lead sign-off in `production/qa/evidence/`
- Pre-implementation gates OQ8 and OQ5/OQ6 are resolved and their gated stories are implemented and closed

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Plugin Scaffold — Pre-Pooled Entity Spawning](story-001-plugin-scaffold.md) | Logic | Complete | ADR-021 |
| 002 | [Fan Layout Formula — Card Position & Rotation](story-002-fan-layout-formula.md) | Logic | Complete | ADR-021 |
| 003 | [Phase State Machine — Visibility & Input Gating](story-003-phase-state-machine.md) | Logic | Complete | ADR-021, ADR-009 |
| 004 | [DRAFT_INITIAL Grid — Display & Purchase Flow](story-004-draft-initial-grid.md) | Integration | Complete | ADR-021, ADR-004 |
| 005 | [PLACEMENT Entry — Submit Button & Core Stage/Unstage](story-005-placement-submit-core.md) | Logic | Complete | ADR-021, ADR-002 |
| 006 | [PLACEMENT Drag — Highlight Sets & TargetUnit](story-006-placement-drag-highlights.md) | Logic | Complete | ADR-021 |
| 007 | [PLACEMENT Instant Card Staging](story-007-placement-instant-staging.md) | Logic | Complete | ADR-021 |
| 008 | [PLACEMENT Un-Staging — Board Ghosts & Instant Fan Slot](story-008-placement-unstaging.md) | Integration | Complete | ADR-021 |
| 009 | [PLACEMENT Timer — Urgency, Grace Window & Submit Checkmark](story-009-placement-timer.md) | Integration | Complete | ADR-021 |
| 010 | [Submit Pre-Validation — Mana & Reserve Checks](story-010-submit-prevalidation.md) | Logic | Complete | ADR-021, ADR-002 |
| 011 | [Reserve Mana Split Strip — Per-Staged-Card Controls](story-011-reserve-mana-strip.md) | Logic | Complete | ADR-021 |
| 012 | [Activation Lock — DRAFT_SHOP Instant Card Lock & Timeout](story-012-activation-lock.md) | Integration | Blocked | ADR-021, ADR-002 |
| 013 | [Reconnect Rebuild — PLACEMENT State Recovery](story-013-reconnect-rebuild.md) | Integration | Ready | ADR-021, ADR-002, ADR-009 |
| 014 | [PLACEMENT Staged Disclosure Accessibility](story-014-placement-staged-disclosure-accessibility.md) | UI | Complete | ADR-021, ADR-002, ADR-023 |
| 015 | [Card Text, Stat, and Keyword Accessibility](story-015-card-text-stat-keyword-accessibility.md) | UI | Ready | ADR-021, ADR-002, ADR-019 |
| 018 | [S11-DRAG-RUNTIME-RETEST — Drag-and-Drop Runtime Divergence Retest + S1-S5 Truth-Table Lock](story-018-drag-runtime-retest.md) | Integration | Draft (Sprint 11 not activated) | ADR-021, ADR-002, ADR-009 |
| 020 | [S12-UX-HAND-DRAG-STATE-VISUALS-001 — Hand-Card Drag-State Visual Differentiation](story-020-hand-drag-state-visuals.md) | UI | Complete (Sprint 15 Should Have; closed PROMPT 1009 on `origin/main` after PROMPT 1003 dev-story + PROMPT 1008 integration `88a6db1`) | ADR-021, ADR-002, ADR-012 |
| 021 | [Hand UI Fan Root `B0004` Hierarchy Warning Cleanup](story-021-hand-fan-root-b0004-hierarchy.md) | Integration | Draft -- Sprint 17 candidate (Nice to Have, `S17-UI-HAND-B0004-CLEANUP-001`; PROMPT 1076 AUDIT-1076-14), NOT activated | ADR-021 |
| 022 | [S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001 — Mana Preview During PLACEMENT Drag](story-022-hand-mana-preview-during-drag.md) | Logic + Integration | Draft -- future Sprint 18 candidate (`S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001`; PROMPT 1127 §R2 missing-feature), NOT activated | ADR-021, ADR-002, ADR-012, ADR-009, ADR-019 |
| 023 | [S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001 — Idle Hand Playable Affordance](story-023-hand-idle-playable-affordance.md) | UI + Integration | Draft -- future Sprint 18 candidate (`S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001`; PROMPT 1127 §R3 missing-feature designed-out), NOT activated | ADR-021, ADR-002, ADR-012, ADR-009 |
| 025 | [S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001 — Passive Hand Click Inspect / Selection Feedback](story-025-hand-fan-passive-click-affordance.md) | UI + Integration | Draft -- future Sprint 18 Wave-A candidate (`S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001`; PROMPT 1201 HUNT-1201-06 + PROMPT 1203 B-1203-PLA-08 + PROMPT 1287 §5 Wave-A), NOT activated | ADR-021, ADR-002 |
| 026 | [S18-HAND-FAN-Z-LAYER-AUCTION-001 — Hand Fan Visibility / Z-Layer During DraftAuction](story-026-hand-fan-z-layer-auction.md) | UI + Integration | Draft -- future Sprint 18 Wave-A candidate (`S18-HAND-FAN-Z-LAYER-AUCTION-001`; PROMPT 1201 HUNT-1201-09 + PROMPT 1180 H-05 + PROMPT 1287 §5 Wave-A), NOT activated | ADR-021, ADR-002 |

**Story counts**: 8 Logic · 5 Integration · 2 UI; status counts: 12 Complete · 2 Ready · 1 Blocked (OQ8). Story 018 is a Sprint 11 DRAFT retest/paperwork story — not counted in the active completion ratios; status tracked separately under `production/sprints/sprint-11.md` `S11-DRAG-RUNTIME-RETEST-001`. Stories 016 (`card-slot-chrome-layout`) and 017 (`card-drag-mvp`) exist as files but predate the most recent count refresh and are not yet folded into the totals — see those files for their authoritative status. Story 019 (`drag-runtime-retest-tighter-capture`) is the Sprint 11/12 diagnostic-only follow-on to 018, closed `Done` with `closed-with-conditions / cannot-reproduce` per PROMPT 814 (`production/sprints/sprint-12.md` `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`); not counted in the active completion ratios. Story 020 (`hand-drag-state-visuals`, S12-UX-HAND-DRAG-STATE-VISUALS-001) is Done as a Sprint 15 Should Have row (PROMPT 1003 dev-story + PROMPT 1008 integration `88a6db1` + PROMPT 1009 /story-done closure); the post-refactor `HAND_UI_ENTITY_COUNT` was bumped by +(HAND_FAN_SLOT_COUNT * 2 + 1) to account for the new overlay children of existing pre-pooled fan-slot / fan_root entities; not yet folded into the active completion ratios. Stories 022 (`hand-mana-preview-during-drag`, `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001`) and 023 (`hand-idle-playable-affordance`, `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001`) are PROMPT 1136 authoring outputs covering PROMPT 1127 §R2 and §R3 respectively — both Draft, both future Sprint 18 candidates, NEITHER activated by PROMPT 1136 (`production/sprints/sprint-18.md` is absent on `origin/main` at authoring time); not yet folded into the active completion ratios. Stories 025 (`hand-fan-passive-click-affordance`, `S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001`) and 026 (`hand-fan-z-layer-auction`, `S18-HAND-FAN-Z-LAYER-AUCTION-001`) are PROMPT 1294 Wave-A authoring outputs covering PROMPT 1201 HUNT-1201-06 / -09 + PROMPT 1203 B-1203-PLA-08 + PROMPT 1180 H-05 — both Draft, both future Sprint 18 Wave-A candidates per PROMPT 1287 §5, NEITHER activated by PROMPT 1294. Story 025 introduces a passive-hand click-intent gate / inspect-state marker distinct from `DragStateOverlay` (Story 020) and `FanSlotPlayableAffordanceOverlay` (Story 023). Story 026 narrows the `RoundPhase::DraftAuction` (`HandUiMode::PassiveLocked`) hand-fan disposition to hidden / subordinate / non-overlapping (worker discretion within three documented paths) and asserts the disposition via QA-snapshot evidence (`hand_fan_visible` and/or `auction_state` top-level keys lifted by PROMPT 1229). Neither story claims `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `S8-QA-001-W1`, or PROMPT 761 `Polish->Release` retry; not yet folded into the active completion ratios. Story 022 introduces a HUD reactive mana-projection display and adds a proposed `TR-HU-009` row to `docs/architecture/tr-registry.yaml` at `/dev-story` time. Story 023 introduces an idle-hand playable-affordance overlay that is deliberately distinct from Story 020's `DragStateOverlay` marker (preserves Story 020 AC2 by construction; no edit to Story 020 file is required or performed) and adds a proposed `TR-HU-010` row at `/dev-story` time. Both stories are R1-independent at the integration-test level (drive state via direct resource insertion; do not require the PROMPT 1127 §R1 bevy_picking repair to be merged first).
**Dependency order**: 001 → 002, 003; 003 → 004, 005, 013; 005 → 006, 007, 008, 009, 010, 011; 006, 007, 008, 010, 011 → 014; 002, 004, 005, 007, 008, 010, 011, 014 → 015; 017 → 018 (story 018 retests the drag MVP landed by 017 + PROMPT 697 / 706 / 709 follow-on work; depends on Sprint 11 activation); 017 → 020 (story 020 builds drag-state visual differentiation on top of the drag MVP producers + `HandDragSprite` + `Res<ActivePlacementDrag>` landed by 017; depends on Sprint 15 activation; **does NOT** depend on, repair, or retest the Sprint 12 story 019 underlying runtime question — disposition preserved verbatim).
