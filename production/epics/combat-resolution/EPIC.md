# Epic: Combat Resolution

> **Layer**: Feature (M2)
> **GDD**: design/gdd/combat-resolution.md
> **Architecture Module**: `server/feature/combat/`
> **Status**: Ready
> **Stories**: 11 stories created — see table below

## Overview

Combat Resolution implements the server-side deterministic 6-step algorithm that executes every round after both players' placements are committed. The system runs as a single Bevy 0.18 exclusive system (`resolve_combat(world: &mut World)`) triggered by the RSM's `BeginResolution` message. It executes all sub-steps (placement effects, CHARGE X movement, FIRST STRIKE attacks, dead-unit removal, standard movement, and standard combat + objective damage) in one frame, accumulates a structured `ResolutionLog`, broadcasts `S2CResolutionEvent` to clients, and writes `ResolutionComplete` to notify the RSM. The combat modifier stack (`apply_combat_modifier_stack`) is a pure function that handles the full ATK/AR/LEADER/type-advantage/VULNERABILITY/RESISTANCE/ARMOR-PIERCING modifier chain with a two-pass algorithm for simultaneous bilateral combat. This epic owns the most algorithmically complex system in the game — all 45 BLOCKING acceptance criteria from `combat-resolution.md` must be satisfied before the epic is closed.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-017: Combat Resolution Execution Architecture | Exclusive system (`&mut World`) runs all 6 sub-steps synchronously in one frame; `apply_combat_modifier_stack` is a pure function; `S2CResolutionEvent` is a single batch broadcast after sub-step 6 completes | HIGH |
| ADR-010: RSM Phase Event Bus | `BeginResolution` and `ResolutionComplete` travel on the Bevy-internal buffered Message bus (`MessageWriter<T>` / `MessageReader<T>`); Combat Resolution subscribes to `ResolutionPhaseEntered` | HIGH |
| ADR-009: RSM Phase State | RSM owns the phase boundary; only `advance_phase` may write `RoundState`; `resolve_combat` exits immediately when no `BeginResolution` message is present | HIGH |
| ADR-002: Client-Server Authority | All sub-step computation is server-side only; clients receive `S2CResolutionEvent` and replay it at animation tempo; no client-side game logic | HIGH |
| ADR-005: Server-Side RNG | Ecaflip dice rolls pre-computed from `ServerRng` before sub-step 1; RNG state never transmitted to clients | LOW |

## GDD Requirements

> Trace source: `docs/architecture/tr-registry.yaml` version 10. Keep this table aligned with active registry IDs.

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-CR-001 | Sub-step 1 placement effects: PendingPlacements drains atomically into BoardState; APPEARANCE triggers fire before SS2; DEATH from APPEARANCE damage is deferred until all SS1 APPEARANCE effects complete; CHANGE LANE/cross-lane effects execute before SS2; STUN applied by APPEARANCE suppresses SS2 CHARGE X and SS5 movement (CR-24, CR-38, CR-39, CR-40) | ADR-017 |
| TR-CR-002 | Sub-step 3 FIRST STRIKE damage resolves before retaliation in sub-step 6; FINAL BLOW is eligible from SS3 kills (CR-2) | ADR-017, ADR-022 |
| TR-CR-003 | Sub-step 5 step-by-step collision detection handles path crossing and same-cell halting; deviation from Board/Lane skip-intermediate rule documented in ADR-017 (CR-9) | ADR-017 |
| TR-CR-004 | Sub-step 6 bilateral pair simultaneous damage uses pre-combat HP snapshots; two-pass algorithm computes all damage before applying (CR-1) | ADR-017 |
| TR-CR-005 | SHIELD persists across rounds until consumed; when triggered, its pre-check absorbs all incoming damage for that sub-step, consumes exactly once, and later sub-steps apply damage normally after consumption (CR-6, CR-7, CR-29, CR-36) | ADR-017, ADR-018 |
| TR-CR-006 | COUNTERATTACK fires after incoming damage or SHIELD absorption for melee contact only (same-cell or collision-halt adjacent); RANGE attackers at distance never trigger it; chains once; multi-attacker simultaneous retaliation supported (CR-20, CR-21, CR-35) | ADR-018, ADR-022 |
| TR-CR-007 | FINAL BLOW fires in the kill sub-step, not SS4; eligibility is evaluated against the attacker who delivered lethal damage (CR-22, CR-23) | ADR-018, ADR-022 |
| TR-CR-008 | Kill gold +1 is awarded by `resolve_combat`; objective gold +3 lands before economy interest snapshots (CR-16) | ADR-017, ADR-019 |
| TR-CR-009 | Objective damage bypasses AR, uses ATK_effective only, and mutates objective HP with `saturating_sub` (CR-10, CR-27) | ADR-017 |
| TR-CR-010 | INJURED activates at sub-step boundaries; state is re-evaluated and not retroactive within a sub-step (CR-26) | ADR-018 |
| TR-CR-011 | OUTNUMBERED is counted at sub-step entry using per-player global board count and strict less-than comparison (KW-027a, KW-027b) | ADR-018 |
| TR-CR-012 | Type advantage formula grants ATK+1 and AR+1 in bilateral matchups only, from `GameConfig.type_beats` (CR-15) | ADR-017 |
| TR-CR-013 | `resolve_combat` exclusive-system scaffold reads `BeginResolution`, exits idle without mutations, executes SS1-SS6, writes `ResolutionComplete` only after `S2CResolutionEvent`, and aborts at 10,000 iterations with `GameOver{Draw}` (CR-41) | ADR-017 |
| TR-CR-014 | `S2CPlacementReveal` is enqueued before any SS1 entity spawn or ECS mutation; empty PendingPlacements still sends an empty reveal (CR-30) | ADR-007, ADR-017, ADR-020 |
| TR-CR-015 | `S2CResolutionEvent` is enqueued before `ResolutionComplete`; RSM phase change can only follow after `ResolutionComplete`, guaranteeing clients receive the full log before phase change (CR-32) | ADR-008, ADR-017 |
| TR-CR-016 | STUN suppresses all RESOLUTION actions for the affected unit: no CHARGE X movement in SS2, no FIRST STRIKE attack in SS3, no standard movement in SS5, and no standard attack in SS6 (CR-5) | ADR-017, ADR-018, ADR-022 |
| TR-CR-017 | SS5 WALL collision halt stops an advancing enemy at the WALL cell; SS6 damage targets that WALL, WALL has 0 ATK, and dead WALL removal occurs at the next DEATH-processing point (CR-8) | ADR-017 |
| TR-CR-018 | CHARGE X movement in SS2 advances X cells using the movement/collision rules, then SS5 movement advances MP as a separate movement (CR-31) | ADR-017, ADR-018 |
| TR-CR-019 | RANGE unit already in range of a WALL does not halt at or advance toward the WALL in SS5; SS6 attacks the WALL from the current cell and emits CombatDamage targeting the WALL (CR-44) | ADR-017, ADR-018 |
| TR-CR-020 | RANGE + FIRST STRIKE units emit two distinct damage events during RESOLUTION: one CombatDamage in SS3 and one CombatDamage in SS6 (CR-4) | ADR-017, ADR-018 |
| TR-CR-021 | Multiple FIRST STRIKE attackers targeting the same unit apply damage in lane order and award FINAL BLOW credit to the lethal source (CR-37) | ADR-017, ADR-018, ADR-022 |
| TR-CR-022 | SS4 DEATH chains are sequential through ChainDeathBuffer queue ordering rather than recursive observer dispatch (CR-25) | ADR-017, ADR-022 |
| TR-CR-023 | SS6 RANGE target selection attacks the nearest forward enemy within RANGE X; single-nearest consumes no RNG, equidistant nearest consumes exactly one RangeEquidistantSelect seed with deterministic eligible ordering (CR-3) | ADR-005, ADR-017 |
| TR-CR-024 | RANGE filtering is forward-only for both players; enemies behind the attacker are never valid RANGE targets (CR-28) | ADR-017 |
| TR-CR-025 | RANGE + FIRST STRIKE units select their SS6 target fresh after SS4 removal and can acquire a surviving enemy if the SS3 target died (CR-45) | ADR-017 |

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All 45 BLOCKING acceptance criteria from `design/gdd/combat-resolution.md` are verified
- All Logic stories have passing unit tests in `tests/unit/combat/`
- All Integration stories have passing integration tests in `tests/integration/combat/`
- `fn apply_combat_modifier_stack` has unit tests covering CR-12, CR-13, CR-14, CR-15, CR-42, CR-43 — all pass without Bevy context
- `fn resolve_combat(world: &mut World)` compiles and registers without error on Bevy 0.18 (ADR-017 Validation Criterion 1)
- `network-protocol.md` D.2 references the canonical `ResolutionEvent` enum from ADR-017 (OQ5 close — ADR-017 Validation Criterion 6)

## Pre-Implementation Gates

Before the first story can be marked Ready for implementation:

1. **DONE 2026-05-01 / revised 2026-05-03 / repaired 2026-05-04**: `TR-CR-001..025` are registered in `docs/architecture/tr-registry.yaml`
2. **`type_advantage_atk_bonus` and `type_advantage_ar_bonus`** must be added to `game-config.md` and `assets/config/game_config.ron` (OQ2 from GDD — action required before combat story begins)
3. **Update `network-protocol.md` D.2** to reference the canonical `ResolutionEvent` enum from ADR-017 (OQ5 close)
4. **Verify `UnitId` vs `EntityId` naming** across `network-protocol.md` D.2 and ADR-017 — must match before code is written (OQ5 gate)

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [resolve_combat Scaffold + Safety Timeout](story-001-resolve-combat-scaffold.md) | Integration | Ready | ADR-017, ADR-010 |
| 002 | [Combat Modifier Stack — Pure Function](story-002-combat-modifier-stack.md) | Logic | Ready | ADR-017 |
| 003 | [Sub-step 1 — Placement Commit + APPEARANCE](story-003-substep1-placement-appearance.md) | Logic | Ready | ADR-017 |
| 004 | [Sub-steps 2 & 5 — Movement + Collision](story-004-movement-collision.md) | Logic | Ready | ADR-017 |
| 005 | [Sub-step 3 — FIRST STRIKE Attacks](story-005-substep3-first-strike.md) | Logic | Ready | ADR-017 |
| 006 | [Sub-step 4 — Dead Removal + DEATH Chains + Kill Gold](story-006-substep4-dead-removal.md) | Logic | Ready | ADR-017 |
| 007 | [Sub-step 6 — Standard Combat + SHIELD + COUNTERATTACK](story-007-substep6-combat-shield-counterattack.md) | Logic | Ready | ADR-017 |
| 008 | [Sub-step 6 — RANGE Targeting](story-008-range-targeting.md) | Logic | Ready | ADR-017 |
| 009 | [Sub-step 6 — Objective Damage + GAME_OVER](story-009-objective-damage-gameover.md) | Logic | Ready | ADR-017, ADR-002 |
| 010 | [Persistent Keyword States — INJURED, LEADER, OUTNUMBERED](story-010-persistent-keyword-states.md) | Logic | Ready | ADR-017 |
| 011 | [ResolutionEvent Log Completeness](story-011-resolution-event-log.md) | Integration | Ready | ADR-017, ADR-008 |

## Next Step

Run `/story-readiness production/epics/combat-resolution/story-001-resolve-combat-scaffold.md` to validate the first story before implementation begins.
