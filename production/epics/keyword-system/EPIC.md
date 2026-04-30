# Epic: Keyword System

> **Layer**: Feature (M3)
> **GDD**: design/gdd/keyword-system.md
> **Architecture Module**: `server/feature/keyword/` + `protocol/src/keyword.rs`
> **Status**: Ready
> **Stories**: 16 stories created (all BLOCKED — pending ADR-018 + ADR-022 Accepted)

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Module Scaffold + UnitKeywordState + Protocol Types](story-001-module-scaffold.md) | Logic | Blocked | ADR-018 + ADR-022 |
| 002 | [Movement Formulas — repel_destination + attract_destination](story-002-movement-formulas.md) | Logic | Blocked | ADR-018 |
| 003 | [FIRST STRIKE + HASTE Keywords](story-003-first-strike-haste.md) | Logic | Blocked | ADR-018 |
| 004 | [STUN State Management](story-004-stun-state.md) | Logic | Blocked | ADR-018 |
| 005 | [SHIELD Sub-Step Scope](story-005-shield-scope.md) | Logic | Blocked | ADR-018 |
| 006 | [SILENCE + INJURED State System](story-006-silence-injured.md) | Logic | Blocked | ADR-018 + ADR-022 |
| 007 | [WALL + IRREMOVABLE + UNTARGETABLE](story-007-wall-displacement-immunity.md) | Logic | Blocked | ADR-018 |
| 008 | [LEADER Snapshot System](story-008-leader-snapshot.md) | Logic | Blocked | ADR-018 |
| 009 | [OUTNUMBERED Board Count Evaluation](story-009-outnumbered.md) | Logic | Blocked | ADR-018 |
| 010 | [BODYGUARD Bond Management](story-010-bodyguard-bond.md) | Logic | Blocked | ADR-018 |
| 011 | [RANGE Targeting + Modifier Stack](story-011-range-modifier-stack.md) | Logic | Blocked | ADR-018 |
| 012 | [Timing Trigger — APPEARANCE + DEATH Chain](story-012-appearance-death-chain.md) | Logic | Blocked | ADR-022 |
| 013 | [Timing Trigger — FINAL BLOW + COUNTERATTACK](story-013-final-blow-counterattack.md) | Logic | Blocked | ADR-022 |
| 014 | [Timing Trigger — INJURED Bonus Activation](story-014-injured-bonus-timing.md) | Logic | Blocked | ADR-022 |
| 015 | [Timing Trigger — START/END OF TURN](story-015-start-end-of-turn.md) | Logic | Blocked | ADR-022 |
| 016 | [Displacement Keywords — REPEL + ATTRACT + TELEPORT + CHANGE LANE](story-016-displacement-keywords.md) | Logic | Blocked | ADR-018 |

## Overview

This epic implements the Keyword System — the authoritative resolution engine for all
~28 card abilities in Lanes and Lies. It covers three categories of keyword behavior:
timing triggers (APPEARANCE, DEATH, FINAL BLOW, COUNTERATTACK, INJURED, START OF TURN,
END OF TURN), combat keywords (FIRST STRIKE, HASTE, RANGE, WALL, BODYGUARD, IRREMOVABLE,
UNTARGETABLE, RESISTANCE, VULNERABILITY, ARMOR-PIERCING, SHIELD, LEADER, OUTNUMBERED,
SILENCE, STUN), and movement keywords (CHARGE X, REPEL X, ATTRACT X, TELEPORT,
CHANGE LANE).

The module is structured as a pure-function / callback interface into
`server/feature/combat/`: Combat Resolution owns *when* keywords execute within its
six-sub-step structure; this epic owns *what each keyword does* within that timing.
Persistent runtime state for all six keyword states (SHIELD, STUN, SILENCE, LEADER bonus,
BODYGUARD bond, OUTNUMBERED) is co-located in a single `UnitKeywordState` component per
board unit entity. Timing triggers are dispatched via Bevy Observers (APPEARANCE, DEATH,
FINAL BLOW, START OF TURN, END OF TURN) or inline conditional calls (COUNTERATTACK,
INJURED). Network protocol types (`KeywordKind`, `KeywordPayload`, `DisplacementEvent`)
live in `protocol/src/keyword.rs` and are consumed by `S2CResolutionEvent`.

## Governing ADRs

| ADR | Decision Summary | Status | Engine Risk |
|-----|-----------------|--------|-------------|
| ADR-018: Keyword System — ECS State Architecture | Monolithic `UnitKeywordState` component; `server/feature/keyword/` module tree with `components.rs`, `effects.rs`, `state_eval.rs`, `movement.rs`; extended `Keyword`/`SimpleKeyword` enum (ADR-006 amendment); `KeywordKind`/`KeywordPayload`/`DisplacementEvent` in `protocol/src/keyword.rs` | **Proposed** | HIGH |
| ADR-022: Keyword System — Timing Trigger Observer Architecture | 5 Observer-dispatched timing triggers; `ChainDeathBuffer` for sequential lane-ordered DEATH chain; COUNTERATTACK and INJURED as inline proximity-gated / scan-based dispatch; `start_of_turn_dispatch_system` as normal Bevy system for DRAFT-phase dispatch | **Proposed** | HIGH |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-KW-001 | HASTE rename from CHARGE; `SimpleKeyword::Haste` in extended enum (ADR-006 amendment) | ADR-018 Part 3 ✅ |
| TR-KW-002 | CHARGE X bonus movement at SS2; cells parameter via board F1 | ADR-018 (effects.rs) ✅ |
| TR-KW-003 | FIRST STRIKE attacks in SS3; standard unit retaliates in SS6 only | ADR-018 (effects.rs) ✅ |
| TR-KW-004 | COUNTERATTACK proximity-gated inline dispatch; same-cell or collision-halt adjacency; RANGE excluded | ADR-022 Part 5 ✅ |
| TR-KW-005 | INJURED re-evaluated at sub-step boundaries (SS3→SS4, SS5, SS6); not retroactive within sub-step | ADR-022 Part 5 ✅ |
| TR-KW-006 | SILENCE strips all keywords including INJURED-granted bonuses; `silenced_until_round: Option<u32>` | ADR-018 Part 1 ✅ |
| TR-KW-007 | STUN suppresses SS2/SS3/SS5/SS6; `stun_active: bool`; clears at RESOLUTION end | ADR-018 Part 1 ✅ |
| TR-KW-008 | SHIELD absorbs all damage from one sub-step; persists across rounds until consumed | ADR-018 Part 1 ✅ |
| TR-KW-009 | WALL halts advancing enemies at its cell; deals 0 damage; displaceable unless IRREMOVABLE | ADR-018 (effects.rs) ✅ |
| TR-KW-010 | OUTNUMBERED global board count `<` strict; re-evaluated at each sub-step boundary | ADR-018 Part 6 ✅ |
| TR-KW-011 | RANGE target selection nearest-enemy; equidistant tie-break via `range_equidistant_select` RNG slot | ADR-018 Part 7 ✅ |
| TR-KW-012 | BODYGUARD bond as `Option<Entity>` on BODYGUARD entity; stable across CHANGE LANE | ADR-018 Part 1 ✅ |

**Untraced requirements**: None — all 12 TRs are covered by ADR-018 or ADR-022.

## Pre-Implementation Gates

All stories in this epic are **BLOCKED** until the following gates are resolved:

| Gate | Status | Blocks |
|------|--------|--------|
| ADR-018 Accepted | ❌ Proposed | All keyword stories |
| ADR-022 Accepted | ❌ Proposed | All timing trigger stories |
| ADR-006 amendment — extend `SimpleKeyword` enum (Charge→Haste, add VulnerabilityX/RepelX/AttractX; serde adjacent tag; round-trip test) | ❌ Pending | All keyword card data encoding |
| ADR-005 — add 3 RESOLUTION seed slots (`range_equidistant_select`, `teleport_random_dest`, `strich_change_lane_select`) | ❌ Pending | RANGE/TELEPORT/Strich stories; KW-033b permanently BLOCKED until this resolves |
| keyword-system.md R3 re-review (GDD is "Needs Revision") | ❌ Recommended | Story authoring accuracy |
| ADR-022 Verification Required × 5 (Bevy 0.18 API: `world.trigger_targets()`, `Trigger<T>` param, `ResMut<T>` in Observer, `MessageWriter<T>` in Observer, `commands.trigger_targets()` flush) | ❌ Pending | Any story using Observer or message dispatch |
| OQ-KS-new — `silence_duration_rounds: u8` field added to `cards.json` schema in card-data-pool.md | ❌ Open | SILENCE implementation stories |

**Permanently BLOCKED story**: KW-033b (Strich CHANGE LANE with RNG tie-break) — BLOCKED
until `strich_change_lane_select` seed slot registered in ADR-005; annotate as BLOCKED
in the story file when created.

## Definition of Done

This epic is complete when:

- All stories are implemented, reviewed, and closed via `/story-done`
- All BLOCKING acceptance criteria from `design/gdd/keyword-system.md` pass (KW-001
  through KW-057, excluding KW-033b which is gated on ADR-005 seed slot registration
  and KW-051/KW-052 which are gated on OQ-KS4 Trap design)
- All Logic stories (timing trigger unit tests, formula tests, state machine tests) have
  passing test files in `tests/unit/keyword/`
- All Integration stories (COUNTERATTACK chain, SILENCE+INJURED cross-keyword, LEADER
  snapshot + death, OUTNUMBERED flip mid-RESOLUTION) have tests in `tests/integration/keyword/`
- Movement formula tests for `repel_destination()` and `attract_destination()` pass
  covering KW-029a, KW-029b, KW-030 before any combat resolution story opens
- All Visual/Feel evidence (displacement animations, BODYGUARD connector, IRREMOVABLE
  flash, SILENCE glyph strip) documented in `production/qa/evidence/`
- `bodyguard_cleanup_system` integration test confirms bond cleared within one frame
  of BODYGUARD despawn
- `ChainDeathBuffer` confirmed empty at RESOLUTION end by integration test
- Pre-implementation gates all resolved (see table above)

## Module Structure (from ADR-018 + ADR-022)

```
server/feature/keyword/
  mod.rs          ← KeywordPlugin: registers 5 observers, ChainDeathBuffer,
                    start_of_turn_dispatch_system, app.add_message::<KeywordTriggered>()
  components.rs   ← UnitKeywordState (6 keyword states co-located)
  events.rs       ← UnitAppeared, UnitDied, FinalBlowDealt,
                    StartOfTurnTriggered, EndOfTurnTriggered
  observers.rs    ← on_unit_appeared, on_unit_died, on_final_blow_dealt,
                    on_start_of_turn, on_end_of_turn,
                    start_of_turn_dispatch_system
  resources.rs    ← ChainDeathBuffer (VecDeque — DEATH chain queue)
  effects.rs      ← keyword effect functions (called BY combat resolution)
  state_eval.rs   ← leader_snapshot_system, eval_outnumbered_system,
                    eval_injured_bonuses, bodyguard_cleanup_system
  movement.rs     ← repel_destination(), attract_destination() (pure functions)

protocol/src/keyword.rs
  KeywordKind, KeywordPayload, InjuredGrantedKeyword (renamed from KeywordKind)
  DisplacementEvent, DisplacementKind
  (consumed by S2CResolutionEvent in network-protocol.md)
```

## Recommended Story Grouping (hint for `/create-stories`)

Stories should be grouped by keyword category and resolved in dependency order:

1. **Scaffold** — `UnitKeywordState` component + `KeywordPlugin` registration + `protocol/src/keyword.rs` types + module stubs (all gates must clear before this story opens)
2. **Movement formulas** — `repel_destination()` + `attract_destination()` pure functions + tests (KW-029a/b, KW-030); unblocks combat resolution movement stories
3. **Timing triggers** — APPEARANCE (KW-001), DEATH chain (KW-002/003), FINAL BLOW (KW-004a/b), COUNTERATTACK (KW-005/006), INJURED (KW-007/008a/b), START OF TURN (KW-009a/b), END OF TURN (KW-010a/b)
4. **Combat keywords** — FIRST STRIKE (KW-011/012), HASTE (KW-013/014), STUN (KW-015a/b), RANGE + BODYGUARD (KW-016), RANGE+FIRST STRIKE (KW-017), WALL (KW-018), BODYGUARD bond (KW-019), IRREMOVABLE (KW-020a/b/c), UNTARGETABLE (KW-021), RESISTANCE/AP (KW-022), SILENCE (KW-023), SHIELD sub-step scope (KW-024), LEADER snapshot (KW-025/026), OUTNUMBERED (KW-027a/b)
5. **Movement keywords** — CHARGE X (KW-028), REPEL (KW-029a/b), ATTRACT (KW-030), TELEPORT (KW-031a/b), CHANGE LANE (KW-032/033a/c) [KW-033b BLOCKED]
6. **Cross-keyword interactions** — KW-034 through KW-057 (R2 additions; many depend on prior categories)

## Next Step

Resolve all Pre-Implementation Gates above, then run `/create-stories keyword-system`
to break this epic into implementable stories.
