# Epic: HUD

> **Layer**: Presentation
> **GDD**: design/gdd/hud.md
> **Architecture Module**: `client/src/ui/hud/` — `HudPlugin` within `PresentationPlugin`
> **Status**: Ready
> **Stories**: 12 stories created/updated through 2026-05-05; 3 Sprint 14
> layout-composition candidates appended 2026-05-14 by PROMPT 879.
> Story 017 closed by PROMPT 976 on 2026-05-16.

## Overview

This epic implements the `HudPlugin` — the client-side persistent readout layer that
surfaces economic and tactical state to both players at all times. The HUD owns four
screen-edge zones (top-left phase/round, top-center scoreboard, top-right gold, bottom-left
mana), pre-pools all 23 entities at session start, and reacts to four server signals:
`S2CGoldUpdate` (own economy), `S2CGoldBroadcast` (both players' gold), `S2CPhaseChanged`
(round + phase via `Res<CurrentClientPhase>`), and a Bevy Observer for `HudObjectiveUpdate`
(re-emitted by Board Rendering after draining `ObjectiveDestroyed`). It rebuilds fully from
`S2CGameSnapshot` on reconnect. The HUD produces zero client-to-server messages and never
asserts game state — it is read-only and server-authoritative.

`HudPlugin` is registered 4th in `PresentationPlugin` (after `CardAnimationsPlugin`,
`BoardRenderingPlugin`, `HandUiPlugin`). Its systems slot into the `PresentationSet`
(PhaseTransition → MessageDrain → StateSync → AnimationTick) defined by ADR-021.
Gold/mana numerics tween via `bevy_tweening` using the cancel-and-replace contract from
ADR-021; dot state flips and phase/round labels are instantaneous (StateSync, no AnimationTick).

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-021: Presentation Layer Architecture | PRIMARY — `HudPlugin` 4th in `PresentationPlugin`; `PresentationSet` (PhaseTransition → MessageDrain → StateSync → AnimationTick); 23 pre-pooled entities after Sprint 14 HUD opponent figurine; `PickingBehavior` guard behind `#[cfg(feature = "ui_picking")]`; bevy_tweening cancel-and-replace via `Animator::set_tweenable()` | HIGH |
| ADR-002: Client-Server Authority | HUD is read-only; zero C2S messages; all state received as S2C messages; no client-side game logic | LOW |
| ADR-001: Objective Identity Unicast | `ObjectiveIdentity` is server-only; `HudObjectiveUpdate` strips `was_fake` at Board Rendering boundary — HUD scoreboard architecturally cannot reveal real/fake | LOW |
| ADR-008: Lightyear Channel Config | `S2CGoldUpdate`, `S2CGoldBroadcast`, `S2CPhaseChanged`, `S2CGameSnapshot` all on `ReliableChannel`; drained via `MessageReceiver<T>` in `MessageDrain` set; single-drain constraint: one reader per message type per frame | HIGH |
| ADR-011: Reconnect + Snapshot | `S2CGameSnapshot` triggers full HUD rebuild (all 23 entities rewritten, no respawn); deferred queue flushed after snapshot; FROZEN mode tiebreak — snapshot always wins, FROZEN re-applies | HIGH |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-HUD-001 | Gold label format adaptive (ECONOMY_BASIC `Xg` / ECONOMY_AUCTION `Xg (Yr)`); 23 pre-pooled entities after Sprint 14 HUD opponent figurine: 6 label parents + 2 TextSpan children (gold label `(Yr)` spans) + 10 scoreboard dots + top/bottom strip and HUD chrome entities including own/opponent figurines | ADR-021 ✅ |
| TR-HUD-002 | Mana display `current_mana / mana_cap`; reserve mana label visible iff `reserve_mana > 0`; sourced from `S2CGoldUpdate` in `MessageDrain` set | ADR-021 ✅ |
| TR-HUD-003 | Phase label strings for DRAFT\_INITIAL / DRAFT\_SHOP / DRAFT\_AUCTION / PLACEMENT / RESOLUTION / GAME\_OVER; sourced via `Res<CurrentClientPhase>` (never direct `MessageReceiver<S2CPhaseChanged>`) | ADR-021 ✅ |
| TR-HUD-004 | Scoreboard dot state machine: all 10 dots ALIVE at session start; `HudObjectiveUpdate` Bevy Observer transitions single dot ALIVE → DESTROYED; `HudPlugin` registers `app.observe(handle_hud_objective_update)` | ADR-021 ✅ |
| TR-HUD-005 | Real/fake identity never rendered on scoreboard; `was_fake` stripped by Board Rendering before triggering `HudObjectiveUpdate` | ADR-001 ✅ |
| TR-HUD-006 | RESOLUTION persistence: HUD root `Visibility::Visible` while Hand UI and Shop/Auction UI hide; enforced by `PresentationSet` ordering — all phase handlers run together in `PhaseTransition` | ADR-021 ✅ |
| TR-HUD-007 | `S2CGoldUpdate` / `S2CGoldBroadcast` tie-break: `handle_gold_broadcast_system` scheduled `.before(handle_gold_update_system)` within `MessageDrain`; own `GoldDisplayState.gold` always wins from `S2CGoldUpdate` | ADR-021 ✅ |
| TR-HUD-008 | Dot ALIVE → DESTROYED: instantaneous state flip via Observer; handled in `StateSync` set; no tween, no `Animator<T>` attached to dot entities | ADR-021 ✅ |
| TR-HUD-009 | FROZEN mode on GAME\_OVER: incremental updates (`S2CGoldUpdate`, `HudObjectiveUpdate`) rejected after `phase == GAME_OVER`; `S2CGameSnapshot` bypasses FROZEN (snapshot always wins), then FROZEN re-applies | ADR-021 ⚠️ partial — pattern via `Res<CurrentClientPhase>` phase check; FROZEN behavior is GDD Rule 10 + Rule 13 spec; no dedicated ADR for freeze semantics |
| TR-HUD-010 | Numeric tween ≤ 300ms; in-flight cancel-and-replace via `Animator::set_tweenable()` — never despawn + respawn; snap to authoritative value on GAME\_OVER entry | ADR-021 ✅ |

**10 / 10 TRs covered** (9 fully by Accepted ADRs; TR-HUD-009 partial — follow GDD Rules 10 and 13 directly for FROZEN behavior).

## Sprint 6 Accessibility Gate Addendum

QA-COND-0005 remains Open for Standard-tier accessibility remediation. The
Sprint 6 accessibility evidence register marks A11Y-ST-13, "Mana pools:
distinct container shapes," as a must-implement row. HUD-011 is the narrow HUD
story for that row: current mana must read as a bar-shaped container and
reserve mana must read as a diamond-shaped container, with browser/WASM evidence
showing the distinction is not color-only.

The same register marks A11Y-ST-01 and A11Y-ST-03 as evidence-only blockers for
text size and contrast. HUD-012 is the narrow HUD story for those rows: HUD gold,
mana, reserve, phase, and round text must meet the required text-size floors;
DRAFT_AUCTION inline gold/reserved labels must be captured as auction-linked HUD
counters; and HUD-owned text/background pairs must have browser/WASM contrast
evidence. The actual auction price counter remains Shop/Auction UI owned.

## Pre-Implementation Gates

The following open questions from `design/gdd/hud.md` must be resolved before the
corresponding stories can be implemented (not blockers for epic creation):

| OQ | Blocks | Owner |
|----|--------|-------|
| OQ-HUD-01 — `S2CSessionPaused` / `S2CSessionResumed` undefined in network-protocol.md | "Waiting for opponent…" pause overlay story | Network Protocol GDD |
| OQ-HUD-04 — `LANE_MIDPOINT_X` sharing mechanism (Board Rendering → HUD) | Dot horizontal alignment story | Tech Lead / Board Rendering epic |
| OQ-HUD-05 — `HudObjectiveUpdate` trigger type crate location | Observer registration story | Tech Lead / Lead Programmer |

Stories for OQ-HUD-01 / OQ-HUD-04 / OQ-HUD-05 will be flagged **BLOCKED** in the story file until the respective OQ resolves. All other stories are implementable from the GDD and ADRs as-is.

## Key Bevy 0.18 API Notes

> ⚠️ `liv-bevy-018` skill is **mandatory** on every `.rs` file in this epic.

- **TextSpan children**: gold label uses parent `Text`/`TextFont`/`TextColor` entity + child `TextSpan`/`TextFont`/`TextColor` entity for `(Yr)` suffix — Bevy 0.18 multi-span pattern. Empty string `""` in ECONOMY\_BASIC mode (never despawn the child entity).
- **BorderRadius inside Node**: scoreboard dots use `Node { border_radius: BorderRadius::all(Val::Px(r)), width: Val::Px(d), height: Val::Px(d), .. }` — standalone `BorderRadius` component does not exist in 0.18.
- **LineHeight required component**: any `Text` entity needs `LineHeight` if custom line spacing is needed; omit only if default is acceptable.
- **PickingBehavior guard**: `#[cfg(feature = "ui_picking")]` on HUD root `Node` insertion — inserting without feature compiled panics (unregistered component). If `ui_picking` absent from `Cargo.toml`, root Node is already non-interactive.
- **GoldDisplayState backing field**: tween targets `f32` fields in `GoldDisplayState`; a change-detection system reads the current value and writes formatted strings to `Text`/`TextSpan`. Do NOT implement as `Lens<GoldDisplayState>` directly — three simultaneous writers on same component.
- **Observer not EventReader**: `app.observe(handle_hud_objective_update)` — never `EventReader<HudObjectiveUpdate>` and never `MessageReader<ObjectiveDestroyed>` (Board Rendering is the sole drain of that Lightyear channel).
- **`HudConfig` not `GameConfig`**: `hud_tween_duration_ms` and layout constants live in a client-side `HudConfig` struct — cosmetic preferences, not server-authoritative game parameters.

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All acceptance criteria from `design/gdd/hud.md` (28 BLOCKING + 4 ADVISORY) are verified
- All Logic and Integration stories have passing test files in `tests/unit/hud/` or `tests/integration/hud/`
- Visual/UI stories have screenshot evidence + lead sign-off in `production/qa/evidence/`
- HUD-011 has browser/WASM A11Y-ST-13 evidence at `production/qa/evidence/hud-011-mana-shapes-evidence.md`
- HUD-012 has browser/WASM A11Y-ST-01 and HUD-owned A11Y-ST-03 evidence at `production/qa/evidence/hud-012-text-size-contrast-accessibility.md`
- HUD-20 (same-tick tie-break) uses `App::new()` with `HudPlugin` registered — not `World::new()` — to verify plugin system ordering (GDD AC note)
- `cargo build -p client` without `ui_picking` feature compiles without panic (ADR-021 validation criterion)
- OQ-HUD-04 resolved: dot horizontal alignment verified against `BoardLayout` / `LANE_MIDPOINT_X` at 1280×720 and 1920×1080

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [HUD Plugin Scaffold and Pre-Pooled Entity Tree](story-001-hud-plugin-scaffold.md) | Logic | Ready | ADR-021 |
| 002 | [Gold and Mana Display (ECONOMY_BASIC)](story-002-gold-mana-display.md) | Logic | Ready | ADR-021 |
| 003 | [Phase Label, Round Counter, and Instantaneous Transitions](story-003-phase-label-round-counter.md) | Logic | Ready | ADR-021 |
| 004 | [Scoreboard Dot Observer and State Machine](story-004-scoreboard-dot-observer.md) | Integration | Blocked (OQ-HUD-05) | ADR-021, ADR-001 |
| 005 | [Phase Transitions and RESOLUTION Persistence](story-005-phase-transitions.md) | Logic | Ready | ADR-021 |
| 006 | [ECONOMY_AUCTION Inline Gold Format and TextSpan](story-006-economy-auction-inline-gold.md) | UI | Ready | ADR-021 |
| 007 | [GAME_OVER Freeze Mode](story-007-game-over-freeze.md) | Logic | Ready | ADR-021 |
| 008 | [Reconnect Snapshot Rebuild](story-008-reconnect-snapshot-rebuild.md) | Integration | Ready | ADR-021, ADR-011 |
| 009 | [Same-Tick Gold Tie-Break (Plugin-Level Integration)](story-009-same-tick-tie-break.md) | Integration | Ready | ADR-021 |
| 010 | [Numeric Tween Animation](story-010-numeric-tween-animation.md) | Visual/Feel | Ready | ADR-021 |
| 011 | [Current and Reserve Mana Shape Distinction](story-011-current-reserve-mana-shapes.md) | UI | Ready | ADR-021, ADR-002 |
| 012 | [HUD Text Size and Contrast Accessibility Evidence](story-012-text-size-and-contrast-accessibility.md) | UI | Ready | ADR-021, ADR-002 |
| 014 | [HUD Timer Eyeball Visual Check](story-014-hud-timer-eyeball-visual-check.md) | Visual/Feel | Draft -- Sprint 13 candidate (Should Have, `S11-HUD-TIMER-EYEBALL-VISUAL-001`), NOT activated | ADR-021, ADR-002 |
| 015 | [HUD Top Strip Layout (Composition Only)](story-015-hud-top-strip-layout.md) | UI | Draft -- Sprint 14 candidate (Must framing per `docs/ux/ui-clean-pass-roadmap.md` rank 7, `S11-UX-HUD-TOP-STRIP-LAYOUT`), NOT activated | ADR-021, ADR-002 |
| 016 | [HUD Bottom Strip Layout (Composition Only)](story-016-hud-bottom-strip-layout.md) | UI | Draft -- Sprint 14 candidate (Tier 1 Must per `docs/ux/ui-clean-pass-roadmap.md` rank 8, `S11-UX-HUD-BOTTOM-STRIP-LAYOUT`), NOT activated | ADR-021, ADR-002 |
| 017 | [HUD Opponent Figurine Composition (Layout Only)](story-017-hud-opponent-figurine.md) | UI | Done via PROMPT 976 (Sprint 14 Nice to Have, `S11-UX-HUD-OPP-FIGURINE`) | ADR-021, ADR-002, ADR-012 |
| 018 | [HUD Opponent Figurine + OPP Label + Mana Duplicate Cleanup](story-018-opp-figurine-mana-cleanup.md) | Integration | Draft -- Sprint 17 candidate (Should Have, `S17-UI-HUD-OPP-MANA-CLEANUP-001`; PROMPT 1076 AUDIT-1076-10 + -16 + -17 bundle), NOT activated | ADR-021, ADR-002, ADR-012 |

**15 stories total: 5 Logic · 6 UI · 3 Integration · 1 Visual/Feel**
Story 004 blocked on OQ-HUD-05 (HudObjectiveUpdate trigger type crate location).
Story 014 is a Sprint 13 candidate (Sprint 12 close-out deferral; Sprint 10 smoke retry-7 W2 carry); NOT activated.
Stories 015 / 016 / 017 began as Sprint 14 candidates from PROMPT 802 Expert UI
Layout audit roadmap (reconciled by `docs/ux/ui-clean-pass-roadmap.md` ranks
7, 8, and Tier 1 Should-Priority Adjacent Rows table). Story 017 is Done via
PROMPT 976; the row remains layout-composition only with no final-art claim and
no Standard-tier accessibility claim. PAW-TD-004-a placeholder-art accept-risk
and QA-COND-0005 accept-risk are preserved verbatim. Stories 015 + 016 are
siblings on `spawn_hud`; story 017 consumes story 016 because the opponent
figurine is hosted inside the bottom-strip flex parent.

## Sprint 14 UI Layout Candidate Dependency Chain

Before any future work on stories 015 / 016 / 017 enters `/dev-story` in Sprint 14, the
following Tier 0 foundational stories MUST be Done (not just Ready) per
`docs/ux/ui-clean-pass-roadmap.md` sequencing rules:

- `S11-TD-UI-ZINDEX-LAYERS` (rank 1, Tier 0 Must) — z-index layer module
- `S11-TD-UI-FLEX-STRIPS` (rank 3, Tier 0 Must) — flex strip primitives
- `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (rank 6, Tier 0 Must) — global UI
  design spec (numeric inputs for spacing/gap/padding/line-height)

Optional but recommended (for assertion coverage, not blocking):

- `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4, Tier 0 Must) — viewport
  invariant test bin scaffolding

None of these foundational stories exist as story files in this epic yet;
they live in their respective epics under `production/epics/`. The Sprint
14 activation orchestrator is responsible for confirming each foundational
story has a Done status before pulling stories 015 / 016 / 017 into the
`/dev-story` queue.

## Next Step

Run `/story-readiness production/epics/hud/story-012-text-size-and-contrast-accessibility.md` before assigning the Sprint 6 A11Y-ST-01/A11Y-ST-03 HUD evidence story. Work through stories in dependency order — each story's `Depends on:` field tells you what must be DONE first.

Stories 015 / 016 were Sprint 14 layout candidates and story 017 is now Done
via PROMPT 976. Future HUD layout work should still respect the Tier 0
foundational dependencies above.
