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

> ⚠️ **No TR-IDs registered** — `docs/architecture/tr-registry.yaml` has no `system: combat-resolution` entries. Requirements below use `TR-CR-???` placeholders. Register stable IDs before `/create-stories` assigns them to stories.

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-CR-001 | Execution scaffold: `resolve_combat(world: &mut World)` exclusive system registered via `add_systems(Update, resolve_combat)`; reads `MessageReader<BeginResolution>`, exits if none present; executes sub-steps 1–6; writes `MessageWriter<ResolutionComplete>`; broadcasts `S2CResolutionEvent` via Lightyear (CR-30, CR-32, CR-41) | ADR-017 ✅ |
| TR-CR-002 | Combat modifier stack pure function: `apply_combat_modifier_stack(attacker, defender) -> CombatResult`; applies SILENCE→STUN→LEADER→type-advantage-ATK→VULNERABILITY→RESISTANCE→ARMOR-PIERCING→type-advantage-AR in that order; all intermediate arithmetic in i32; `net_damage = max(0, ATK_effective − AR_effective)` (CR-12, CR-13, CR-14, CR-15, CR-42, CR-43) | ADR-017 ✅ |
| TR-CR-003 | Two-pass bilateral algorithm: Pass 1 (A→B) computes `net_damage_A_to_B` + `AR_attacker_combat_A`; Pass 2 (B→A) uses `AR_base_A + AR_attacker_combat_A` as defender AR; both HP mutations applied simultaneously from pre-combat snapshots | ADR-017 ✅ |
| TR-CR-004 | Sub-step 1: all played cards enter board simultaneously; APPEARANCE triggers fire immediately; cross-lane triggers (CHANGE LANE) execute after all SS1 effects complete and before SS2; STUN applied by APPEARANCE takes effect immediately (CR-24, CR-38, CR-39, CR-40) | ADR-017 ✅ |
| TR-CR-005 | Sub-step 2: CHARGE X bonus movement; STUNned units skip; collision rules same as SS5; distinct from the CHARGE combat keyword (CR-31) | ADR-017 ✅ |
| TR-CR-006 | Sub-step 3: FIRST STRIKE attacks fire simultaneously across all lanes; multi-source sequential in lane order; dead units NOT removed until SS4; STUN suppresses SS3 attack (CR-1, CR-2, CR-4, CR-22, CR-37) | ADR-017 ✅ |
| TR-CR-007 | Sub-step 4: all units at 0 HP removed; DEATH trigger chains fire sequentially in lane order; kill gold log drained here for SS3 kills; FINAL BLOW fires in the sub-step of the kill, not consolidated to SS4 (CR-16, CR-22, CR-23, CR-25, CR-26) | ADR-017 ✅ |
| TR-CR-008 | Sub-step 5: destination formula F1 computed once at SS5 entry (`clamp(current_cell + direction × MP, 1, 8)`); tick-by-tick enemy collision loop detects WALL halt (CR-8) and path-crossing halt (CR-9); friendly units + Traps + Prisms not checked in collision loop; STUN suppresses SS5 movement (CR-5, CR-8, CR-9, CR-31, CR-44) | ADR-017 ✅ ADR-002 ✅ |
| TR-CR-009 | Sub-step 6: bilateral pair combat (2-pass simultaneous); multi-source sequential in lane order; RANGE targeting (nearest forward enemy, equidistant random from `range_equidistant_select` seed); objective damage at Cell 8 after all unit combat; kill gold log drained here for SS6 kills; GAME_OVER check fires when objective HP reaches 0 (CR-3, CR-6, CR-10, CR-11, CR-17, CR-18, CR-19, CR-27, CR-28, CR-29, CR-35, CR-36, CR-45) | ADR-017 ✅ |
| TR-CR-010 | SHIELD pre-check: runs before modifier stack; negates all damage from all attackers in same sub-step simultaneously; consumed once; persists between rounds until consumed; COUNTERATTACK fires after pre-check regardless of absorption (CR-6, CR-7, CR-29, CR-36) | ADR-017 ✅ |
| TR-CR-011 | COUNTERATTACK: melee-only (same-cell or collision-halt adjacent); fires once per sub-step after all incoming damage is resolved; runs full modifier stack; FINAL BLOW eligible; chains once if attacker also has COUNTERATTACK; multiple attackers retaliates simultaneously using pre-retaliation HP snapshots (CR-20, CR-21, CR-35) | ADR-017 ✅ |
| TR-CR-012 | Persistent keyword states: INJURED evaluated at sub-step boundary (SS4 onward, not mid-SS3); OUTNUMBERED re-evaluated at each sub-step entry from post-removal board state; LEADER snapshot taken at RESOLUTION entry, persists through RESOLUTION even if LEADER dies; STUN suppresses all outgoing actions but not passive SHIELD (CR-5, CR-26, CR-33, CR-34, CR-40) | ADR-017 ✅ |
| TR-CR-013 | Kill-log attribution: internal `kill_log: Vec<KillRecord>` appended at lethal HP reduction; drained at SS4 (SS3 kills) and post-SS6-combat cleanup (SS6 kills); each drain emits one `GoldAwarded` entry in `ResolutionLog`; objective destruction awards +3g to attacker, NOT +1 kill gold (CR-16, CR-17) | ADR-017 ✅ |
| TR-CR-014 | Iteration budget guard: monotonic internal counter across all sub-step loops; if > 10,000 iterations, abort and RSM broadcasts `S2CGameOver { loser: None, reason: Draw }`; RSM 60s safety timeout remains as outer backstop (CR-41) | ADR-017 ✅ |
| TR-CR-015 | S2CResolutionEvent batch delivery: single reliable-broadcast Lightyear message after all 6 sub-steps complete; `events: Vec<ResolutionEvent>` in chronological `(sub_step, trigger_index)` order; `S2CPhaseChanged(DRAFT_SHOP)` must not be observable before `S2CResolutionEvent` is received; `S2CPlacementReveal` broadcast before sub-step 1 executes (CR-30, CR-32) | ADR-017 ✅ ADR-008 ✅ |

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

1. **Register TR-CR-001..015** in `docs/architecture/tr-registry.yaml` — stories cannot reference stable IDs until this is done
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
