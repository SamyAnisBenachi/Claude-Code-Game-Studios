# Keyword System

> **Status**: Needs Revision — R3 MAJOR REVISION IN PROGRESS. Decisions collected 2026-05-01. Edits applied inline. R4 re-review required.
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-05-01
> **Implements Pillar**: Deep emergence · Simple surface · No idle spectating

## Overview

The Keyword System is the authoritative design reference for all card abilities in Lanes and Lies. It specifies the exact behavior of approximately 28 keywords across three categories: **timing triggers** that fire in response to game events (APPEARANCE, DEATH, FINAL BLOW, COUNTERATTACK, INJURED, START OF TURN, END OF TURN); **combat keywords** that extend the base ATK/HP/AR resolution stack (FIRST STRIKE, CHARGE, RANGE 1-X, WALL, BODYGUARD, ARMOR-PIERCING, SHIELD, LEADER, RESISTANCE X, VULNERABILITY X, SILENCE, STUN, IRREMOVABLE, UNTARGETABLE, OUTNUMBERED); and **movement keywords** that alter unit positioning beyond standard MP-based advancement (CHARGE X, REPEL X, ATTRACT X, TELEPORT, CHANGE LANE). Keywords are declared as structured data in each card's `cards.json` definition; the Combat Resolution system evaluates them within its six-sub-step global pass framework, but what each keyword means — its trigger condition, its effect, its priority relative to other keywords, and its edge-case resolution — is owned exclusively by this document. For the player, keywords are the tactical vocabulary printed on every card: "FIRST STRIKE" tells a player everything they need to know about attack timing without reading a rules manual. The strategic depth of the game emerges primarily from keyword interactions — a WALL anchoring a lane against a FIRST STRIKE attacker, a RANGE unit attacking from outside COUNTERATTACK range, an INJURED unit unlocking bonus stats after absorbing a hit. The ~28 keyword definitions, combined with lane positioning and hidden placement, generate a combinatorial strategy space that rewards deep familiarity while remaining legible to newcomers on round one — the "Deep emergence" pillar operating within the "Simple surface" constraint.

## Player Fantasy

The Keyword System serves the fantasy of **authorship through anticipation** — the feeling that the board you built is a clockwork you wound up, and RESOLUTION is the moment you let it run.

During PLACEMENT, the player is not just choosing which cards to play. They are composing a sequence: *BODYGUARD enters first. FIRST STRIKE fires before anything can reach it. The CHARGE unit behind it advances two extra cells and lands at the objective.* Each keyword on each card is a gear the player slots into place — silently, with 10 seconds on the clock, knowing the opponent is building their own machine at the same time. The player commits, submits, and steps back.

Then RESOLUTION starts. And the player watches their clockwork fire.

**What the player should feel:**
- **"I built that."** — When FIRST STRIKE kills the blocker before the CHARGE unit passes, the player did not *react*. They *constructed*. That outcome existed in their head during PLACEMENT. The system confirmed it.
- **"I need to read theirs."** — The clockwork framing makes RESOLUTION active, not passive. The player watches their own machine run while simultaneously reading the opponent's: *they used IRREMOVABLE on lane 2, that's why the REPEL didn't work. They have a LEADER somewhere.* Watching is intelligence gathering.
- **"Each keyword is a gear that keeps its promise."** — At the card level, every keyword is a small contract: WALL stays put, SHIELD absorbs one hit, BODYGUARD stands in front. When a keyword fires exactly as named, the game feels legible and fair. When two keywords collide unexpectedly — SILENCE stripping INJURED's bonus mid-combat — the game rewards the player who read the interaction before placing.

**What to avoid:** Keywords must never feel like hidden gotchas. A player who loses a lane to COUNTERATTACK should leave thinking "I didn't account for that" — not "I couldn't have known." Every keyword name must describe its effect well enough that a first-time player reading the card gets the right mental model. *Simple surface* applies at the individual keyword level; *deep emergence* is what happens when the clockwork gears mesh in unexpected combinations.

*Pillar alignment: "No idle spectating" — RESOLUTION is the payoff phase of authorship, not downtime. "Deep emergence" — ~28 keywords combining with positions and card stats create the strategy space. "Simple surface" — each keyword fits its name, first read.*

**SILENCE as the totalizing counter:** SILENCE strips all keywords simultaneously — including WALL's blocking behavior, BODYGUARD's protection, LEADER's bonus, and all trigger hooks. This all-or-nothing scope is intentional design, not a documentation gap. SILENCE is the game's "I read them perfectly" payoff: the player who plays SILENCE on a WALL+BODYGUARD+LEADER cluster has read the opponent's entire clockwork and applied the one card that dismantles it. The keyword system's clockwork-authorship fantasy includes a wrench. This only feels fair if the receiving player could have anticipated it — anticipation must be possible from board state alone (the SILENCE card exists in the card pool; a BODYGUARD protecting the LEADER is the textbook counter-play). The GDD's design test: "If a player loses to SILENCE and says 'I couldn't have known,' the design has failed." A player who loses and says "I should have protected that LEADER" has experienced the design correctly. The high-visibility SILENCE application animation (see UI Requirements) is the delivery mechanism for the "I should have known" moment.

## Detailed Design

### Core Rules

**Keyword categories.** All card abilities belong to one of three categories:
- **Timing triggers** — hooks that fire in response to game events. The keyword flag declares that the effect exists and when it fires; the card's `effect_text` and code-side logic define what actually happens.
- **Combat keywords** — modifiers applied during Combat Resolution sub-steps. Most are stateless per-attack modifications to the damage formula or attack-sequencing rules; some create persistent states.
- **Movement keywords** — displacement effects beyond standard MP-based advancement. All are triggered effects that fire within whatever sub-step or card context activates them.

**HASTE disambiguation.** The combat keyword formerly known as CHARGE is renamed **HASTE** in all card data and documents going forward.
- **HASTE** (combat keyword): the unit can act (move and attack) in the round it enters play. Without HASTE, a unit placed in sub-step 1 is affected by summoning sickness — it enters the board but skips movement and attacks this round.
- **CHARGE X** (movement keyword): advances X extra cells during sub-step 2.
- A unit may have both. STUN applied in SS1 overrides HASTE — the STUNned unit still cannot act this round.

**Sub-step timing reference.**

| Keyword | Executes at | Note |
|---|---|---|
| APPEARANCE | Sub-step 1 | All SS1 appearances resolve before any DEATH chains begin |
| HASTE | Flag — SS1 | Removes summoning sickness; no sub-step of its own |
| CHARGE X | Sub-step 2 | Bonus movement; suppressed by STUN |
| FIRST STRIKE | Sub-step 3 | Kills before retaliation |
| RANGE + FIRST STRIKE | SS3 AND SS6 | Two separate attacks; SHIELD consumed in SS3 doesn't protect in SS6 |
| COUNTERATTACK | SS3 or SS6 | Fires on any non-RANGE melee attack against this unit; no proximity restriction |
| FINAL BLOW | SS3 or SS6 | In the sub-step of the kill, not consolidated to SS4 |
| DEATH | Sub-step 4 | Sequential chains; lane order for simultaneous deaths |
| Standard movement | Sub-step 5 | WALL blocks; STUN suppresses; cross-lane triggers after SS5 |
| RANGE (standard) | Sub-step 6 | RANGE without FIRST STRIKE attacks in SS6 only |
| SHIELD | SS3 and/or SS6 | Sub-step scoped; absorbed once per sub-step |
| INJURED | Sub-step boundary | Re-evaluated after each sub-step; never strips own condition |
| START OF TURN | DRAFT entry | After mana ramp + gold income (RSM Rule 3) |
| END OF TURN | RESOLUTION end | After SS6, before round counter increments |
| REPEL X / ATTRACT X / TELEPORT | Context-dependent | Fires within triggering sub-step |
| CHANGE LANE | After triggering SS | Before next sub-step begins |

---

#### Timing Trigger Catalog

**APPEARANCE** — fires immediately when the unit enters the board from the PlacementBuffer during sub-step 1. All SS1 APPEARANCE effects across all lanes resolve before any DEATH chains from APPEARANCE-caused deaths begin. The killed unit is still present during any FINAL BLOW that results.

**DEATH** — fires when a unit is removed from the board in sub-step 4. Chains are sequential: if A's DEATH trigger kills B, B's DEATH trigger fires after A's completes. Simultaneous deaths fire in lane order (Lane 1–5). Kill gold (+1) is awarded to the player whose attack dealt the final damage.

**FINAL BLOW** — fires in the sub-step where the kill occurred (SS3 for FIRST STRIKE kills, SS6 for standard). If two sequential damage sources in the same sub-step kill a unit, the second source (the one that reduced HP to 0) receives FINAL BLOW credit.

**COUNTERATTACK** — fires once per sub-step after all incoming damage to this unit in that sub-step is resolved (including SHIELD pre-check absorption). Fires even if SHIELD absorbed all damage — the unit was attacked regardless. Fires on any non-RANGE melee attack (same-cell contact OR collision-halted adjacent-cell contact). RANGE attackers cannot trigger COUNTERATTACK. Does NOT fire when the unit is STUNned. **Retaliation damage:** runs the full combat modifier stack (steps 1–9; the original attacker's SHIELD pre-check applies to the COUNTERATTACK damage independently). **FINAL BLOW eligible:** if COUNTERATTACK retaliation reduces an attacker to 0 HP, FINAL BLOW fires for the COUNTERATTACK unit. **Chain (once):** if the original attacker also has COUNTERATTACK, they retaliate back once; the chain stops there (no counter-of-counter). **Multiple attackers:** if multiple melee attackers hit the unit in the same sub-step, the COUNTERATTACK unit retaliates against all of them simultaneously using pre-retaliation HP snapshots for each bilateral pair.

**INJURED** — a unit is INJURED when `current_HP < max_HP`. INJURED is a persistent state re-evaluated at each sub-step boundary. A unit damaged in SS3 is INJURED from SS4 onward for that RESOLUTION. SILENCE strips INJURED-granted keywords (e.g., FIRST STRIKE); INJURED itself is not a keyword and cannot be silenced.

**START OF TURN** — fires at DRAFT phase entry, after mana ramp and gold income are applied (RSM Rule 3). Cards placed this round first trigger START OF TURN on round R+1.

**END OF TURN** — fires after RESOLUTION sub-step 6 completes, before the RSM round counter increments. Cards alive at this point can trigger it even if they entered play this round.

---

#### Combat Keyword Catalog

**FIRST STRIKE** — attacks in sub-step 3 before standard combat. Kills before retaliation. Two FIRST STRIKE units facing each other deal damage simultaneously (no priority). FIRST STRIKE does not advance objective damage to SS3.

**HASTE** *(renamed from CHARGE)* — unit can act (move, attack) the round it enters play. Without HASTE: SS1 entry only, no SS2/SS3/SS5/SS6 participation this round. STUN applied in SS1 overrides HASTE.

**RANGE 1-X** — attacks the nearest enemy in the forward direction within X cells; does not advance. Equidistant targets: server selects randomly (`range_equidistant_select` seed slot — registered in ADR-005, OQ-KS1 RESOLVED). RANGE + FIRST STRIKE: attacks in SS3 AND SS6. RANGE attacks bypass BODYGUARD. COUNTERATTACK cannot be triggered by RANGE attackers. **RANGE + WALL:** WALL is a valid RANGE target. If WALL is the nearest enemy, RANGE attacks WALL. WALL's blocking behavior (movement halt in SS5) does not affect RANGE targeting — RANGE selects by cell proximity, not by whether the target blocks movement.

**WALL** — stationary (MP=0, cannot self-move). Advancing enemies stop at WALL's cell and fight it in SS6 (WALL deals 0 damage). Not IRREMOVABLE by default — can be displaced by REPEL/ATTRACT/TELEPORT. **FIRST STRIKE + WALL:** a FIRST STRIKE unit can attack WALL in SS3. If FIRST STRIKE kills WALL in SS3, WALL is removed in SS4 and its blocking anchor is gone — advancing enemies no longer halt at its former cell in SS5. This is intentional counter-play (FIRST STRIKE + CHARGE X can clear a WALL anchor before standard movement resolves).

**BODYGUARD** — on entry (SS1): controller chooses one other friendly unit on the board. That unit cannot be targeted by opponent Spells or Orders while this BODYGUARD is alive. RANGE attacks bypass BODYGUARD. Objective damage from movement is unaffected. Protection ends when BODYGUARD dies.

**IRREMOVABLE** — cannot be displaced by REPEL, ATTRACT, TELEPORT, Spells, or Orders. Does not affect the unit's own movement (MP, CHARGE X, CHANGE LANE).

**UNTARGETABLE** — cannot be named as the target of opponent Spells or Orders. Does not prevent standard combat or RANGE attacks. Friendly non-targeted AoE effects may still apply.

**RESISTANCE X** — incoming ATK reduced by X before the AR step: `ATK_effective = max(0, ATK_raw − X)`. Not bypassed by ARMOR-PIERCING (which only affects AR, not RESISTANCE).

**VULNERABILITY X** — incoming ATK increased by X before the AR step: `ATK_effective = ATK_raw + X`.

**SILENCE** — strips all keywords and keyword-granted effects for the duration. Affects: FIRST STRIKE, HASTE, CHARGE X, RANGE, WALL movement-lock, BODYGUARD protection, UNTARGETABLE, RESISTANCE, VULNERABILITY, ARMOR-PIERCING, SHIELD, LEADER bonus grant, OUTNUMBERED condition, and all trigger hooks (DEATH/FINAL BLOW/COUNTERATTACK/INJURED bonuses). INJURED state cannot be silenced.

**STUN** — unit cannot act this round: SS2 (CHARGE X), SS3 (FIRST STRIKE), SS5 (standard movement), SS6 (attacks) are all suppressed. STUN also suppresses all reactive keyword hooks: a STUNned unit does NOT fire COUNTERATTACK when attacked, does NOT fire DEATH trigger when killed, and does NOT fire any other trigger-based keyword. Unit remains on board and takes incoming damage normally. Lasts current RESOLUTION only.

**ARMOR-PIERCING** — attacker treats defender's AR as 0 for outgoing damage. RESISTANCE on the defender is applied independently (before AR step) and is unaffected. Attacker's own AR (including RPS bonus) is unaffected.

**SHIELD** — absorbs all incoming damage from one sub-step (SS3 or SS6). Sub-step scoped: consumed once; does not protect in a different sub-step. Persists across rounds until triggered. Two simultaneous attackers in the same sub-step are both absorbed by one SHIELD consumption.

**LEADER** — grants a stat bonus to all friendly units of the same family (bonus type and value per card). Snapshotted after SS1 completes (after all SS1 APPEARANCE effects resolve, before SS2 begins); persists even if LEADER dies in SS4. Recalculated fresh each round. A LEADER placed in SS1 of round R (with or without HASTE) IS included in the round R snapshot, because the snapshot is taken post-SS1 — the LEADER is already on the board. A SILENCEd LEADER does not grant its bonus. **LEADER stacking rule:** If two LEADER units of the same family are both alive when the SS1-end snapshot is taken, only the one placed earlier in the current session grants its bonus; the second LEADER's bonus is suppressed (bonuses do not stack). The earlier-placed LEADER is always deterministic from placement order. Two LEADERs of different families each grant their own family's bonus independently.

**OUTNUMBERED** — this unit's OUTNUMBERED bonus is active when the controlling player has fewer units on the board (all lanes, Minions + Structures, excluding Traps and Fields) than the opponent. Evaluated at each sub-step boundary — after the preceding sub-step fully completes, before the current sub-step begins. Can activate or deactivate mid-RESOLUTION as deaths change counts. **Board count definition:** Traps are face-down and excluded. Fields are passive lane-wide effects with no HP and are excluded (same rationale as Traps — only entities that fight count). Maximum per player: 10 (5 lanes × 1 Minion + 5 lanes × 1 Structure cap — verify Structure-slot cap against `board-lane-system.md`; confirmed max = 10 for 1v1).

---

#### Movement Keyword Catalog

**CHARGE X** — advances X extra cells during sub-step 2, before FIRST STRIKE. Applies the same WALL-blocking and collision rules as sub-step 5. Suppressed by STUN. Independent of HASTE.

**REPEL X** — pushes target X cells toward its own side. Direction: toward Cell 1 (Player A units) or Cell 8 (Player B units). Formula: `new_cell = clamp(target_cell + (−advance_dir(target.owner)) × X, 1, 8)`. Triggered effect — fires within whatever sub-step activates it. IRREMOVABLE: no effect. WALL can be pushed. Trap cells trigger on entry during displacement. Pushing a unit to Cell 8 (for Player B) places it at its own spawn — no objective damage.

**ATTRACT X** — pulls target toward caster's cell. Lane-local. Can target friendly or enemy per card text. IRREMOVABLE: no effect. Trap triggers on traversed cells. **Collision rule for enemy targets:** opposing units can never occupy the same cell — an enemy unit pulled by ATTRACT stops 1 cell short of the caster's cell (see Formula 2). For friendly targets, the unit may stop at the caster's cell (same-player co-occupation is allowed). TELEPORT is the only effect that bypasses this rule.

**TELEPORT** — repositions a unit to a specified cell. Destination, target filter, and range restriction are per-card (no keyword-level range cap). Does NOT trigger APPEARANCE or COUNTERATTACK. Cannot return a unit to hand (valid destination: cells 1–8). IRREMOVABLE: no effect. Co-occupation is allowed. Spawn-range restrictions do not apply (PLACEMENT rule only). Cross-lane TELEPORT only if card text specifies a different lane.

**CHANGE LANE** — moves this unit to an adjacent lane at the same cell position. Executes after the triggering sub-step, before the next begins. Rejected silently if the destination lane already holds a friendly Minion (1-Minion-per-lane slot rule). *Strich:* automatically triggers CHANGE LANE when an enemy unit enters play in Strich's current lane; server selects randomly if both adjacent lanes are valid.

---

### States and Transitions

| State | Source | Active from | Expires when |
|---|---|---|---|
| INJURED | HP drops below max_HP | Sub-step boundary after damage | HP restored to max_HP or unit dies |
| SHIELD | Unit enters with SHIELD | Board entry | Absorbs an attack (consumed in SS3 or SS6) |
| LEADER bonus | LEADER alive when SS1 completes | After SS1 (snapshotted post-SS1, pre-SS2) | RESOLUTION ends; recalculated next round |
| OUTNUMBERED | Board count check | Sub-step boundary (after prior SS completes) | Counts equalize or favor controller |
| STUN | Source effect | Immediately on application | End of current RESOLUTION |
| SILENCE | Source effect | Immediately on application | Duration per card (typically one RESOLUTION) |
| BODYGUARD protection | BODYGUARD enters play | SS1 | BODYGUARD unit leaves board |

---

### Replication Contract

> **Authoritative source for client display of all persistent keyword states.** The server owns ground truth; the client must be able to render every glyph and bond defined in Visual/Audio Requirements and reconstruct all states after a mid-RESOLUTION reconnect. Each state below specifies its replication path. New variants introduced here MUST be added to `network-protocol.md` before implementation.

| State | Replication Path | Lifetime | Reconnect Recovery |
|---|---|---|---|
| **SHIELD** | `UnitBoardState.shield_active: bool` field on `S2CGameSnapshot`; `KeywordTriggered { keyword: ShieldConsumed, sub_step }` event in `S2CResolutionEvent` when absorbed | Persists until consumed in SS3 or SS6 of any RESOLUTION | From snapshot |
| **STUN** | `UnitBoardState.stunned_until_round: Option<u32>` (field renamed from `stun_active: bool` in NP R6 — BREAKING CHANGE); `KeywordTriggered { payload: KeywordPayload::StunApplied { duration_rounds: u8 }, sub_step }` event when applied. **SERVER MUST emit `duration_rounds = 1` always** — the `u8` field exists for forward-compatibility only; any value > 1 is a server bug in the current design (multi-round STUN is not designed: see Tuning Knobs). | Current RESOLUTION only | From snapshot — `Some(current_round)` during the RESOLUTION it was applied; `None` at RESOLUTION end. Stars glyph visible to both players. |
| **SILENCE** | `UnitBoardState.silenced_until_round: Option<u32>` (NP R6 authoritative type — was incorrectly `Option<u8>` in prior versions). **Server computes:** `silenced_until_round = current_round + silence_duration_rounds - 1` (expiry-inclusive). For N=1: `silenced_until_round = current_round`. **Client renders SILENCE while:** `current_round <= silenced_until_round`. Worked example for N=1: applied in round 5 → `silenced_until_round = 5`; in round 5 `5 <= 5 = true` (active); in round 6 `6 <= 5 = false` (expired) ✓ exactly 1 RESOLUTION. See OQ-KS-new for the `silence_duration_rounds` structured field requirement. **On receipt of `SilenceApplied`:** client MUST clear all runtime INJURED-granted keyword state for that unit (including bonuses tracked from prior `InjuredBonusActive` events). SILENCE is full-RESOLUTION scoped — it does not expire mid-sub-step (see OQ-KS6). | Per-card structured `silence_duration_rounds` field (see OQ-KS-new — this field must exist in `cards.json` before SILENCE can be implemented). | From snapshot — `Some(R)` where R = round silence expires; client renders SILENCE while `current_round <= silenced_until_round`. |
| **INJURED** | Derived client-side from live HP comparison: `UnitStats.hp < UnitBoardState.max_hp`. **Note:** `max_hp` lives only in `UnitBoardState` (reconnect snapshot), NOT in the `UnitStats` replicated component (`UnitStats { hp: u8, atk: u8, ar: u8 }` has no `max_hp` field). Client MUST cache `UnitBoardState.max_hp` from the last snapshot and use it alongside live `UnitStats.hp` to compute INJURED mid-RESOLUTION. No separate INJURED field needed — it is always derived. INJURED-granted keyword bonuses surface via `KeywordTriggered { source_unit_id: Some(unit_id), sub_step, payload: KeywordPayload::InjuredBonusActive { granted_keyword: GrantedKeyword } }` — see NP GDD D.3 for the authoritative `GrantedKeyword` enum (4 variants: `FirstStrike, Counterattack, Range, Shield`). | While `current_hp < max_hp` | From snapshot (HP comparison: `UnitBoardState.current_hp < UnitBoardState.max_hp`) |
| **HASTE** | `UnitBoardState.haste_active: bool` — set to `true` for any unit with the HASTE keyword that has already acted in SS1 of the current RESOLUTION; `false` otherwise. Required for mid-RESOLUTION reconnect so the client can determine whether to show the unit as having already moved in SS1. `KeywordTriggered { source_unit_id: Some(unit_id), sub_step: 2, payload: KeywordPayload::HasteActivated }` event emitted at SS2 when a HASTE unit moves. **SERVER MUST NOT emit `HasteActivated` for a HASTE unit that was STUNned in SS1** — `haste_active` stays `false` for STUNned HASTE units. | Set `true` at SS1 entry for HASTE units that acted (not STUNned); reset `false` at RESOLUTION end. | From snapshot — client skips "not yet moved" rendering for units where `haste_active = true`. |
| **LEADER bonus** | `UnitBoardState.leader_bonus_atk: u8` and `leader_bonus_hp: u8` on every eligible family unit, written post-SS1 (after all SS1 APPEARANCE effects resolve, before SS2); `KeywordTriggered { keyword: LeaderSnapshotTaken, leader_unit_id }` emitted at that moment for client tint application | Persists for entire RESOLUTION even if LEADER dies in SS4; recalculated next round | From snapshot (the buffed stats reflect the snapshot; on reconnect the tint is reapplied to all units whose `leader_bonus_atk > 0`) |
| **OUTNUMBERED** | Computed client-side from `S2CGameSnapshot` board counts at sub-step entry events; the server emits `KeywordTriggered { keyword: OutnumberedFlipped, player_id, active: bool, sub_step }` ONLY when the boolean transitions, to keep bandwidth low | Re-evaluated at sub-step boundaries | From snapshot (count both sides) |
| **BODYGUARD bond** | `UnitBoardState.bodyguard_protects: Option<EntityId>` on the BODYGUARD unit; the protected unit carries no field (the bond is unidirectional, server-tracked). Bond established once via `KeywordTriggered { keyword: BodyguardBondCreated, bodyguard_id, protected_id, sub_step: 1 }`; broken via `KeywordTriggered { keyword: BodyguardBondBroken, bodyguard_id }` when BODYGUARD dies | Until BODYGUARD dies | From snapshot — client redraws connector procedurally between BODYGUARD's glyph and the protected unit's base ring |

**Implementation contract:** BODYGUARD protection MUST be stored as a unit-to-unit entity bond (`Option<EntityId>` on the BODYGUARD's component), NOT as a lane-scoped attribute. This guarantees the bond survives any CHANGE LANE the protected unit may execute — the entity reference is stable across position changes.

**OUTNUMBERED indicator note:** the visual indicator from `combat-resolution.md` is a per-lane arrow (legacy spec). The OUTNUMBERED rule is a global board count. To resolve this mismatch, the indicator must surface **per-unit** on each unit that carries the OUTNUMBERED keyword (not per-lane), reading from the global computed boolean. `combat-resolution.md` Visual section needs updating accordingly (added as recommended fix to that GDD's review backlog).

**LEADER snapshot client display:** the Arcane Gold 20% opacity base ring tint specified in `combat-resolution.md` Visual section persists on every family unit whose `leader_bonus_atk > 0` until RESOLUTION ends. If the LEADER dies in SS4, the tint stays — the buffed stat field persists in the replicated component. Client does not require special "LEADER-died" logic; the field naturally clears at RESOLUTION end.

**60-second RESOLUTION safety timeout:** if the timeout fires, the server broadcasts `S2CGameOver { loser: None, reason: ResolutionTimeout }`. The client cancels all in-flight keyword animations and shows the timeout result screen. Distinct from a genuine mutual-objective `Draw`.

**SILENCE + INJURED interaction:** When `SilenceApplied` is received, the client MUST clear all runtime INJURED-granted keyword state (e.g., stop showing FIRST STRIKE glyph on a SILENCEd INJURED unit). The server has already stripped the bonus; the client must not render it.

**HASTE + STUN:** SERVER MUST NOT emit `HasteActivated` for a STUNned HASTE unit. STUN suppresses the act; no HASTE activation event is emitted.

---

### Interactions with Other Systems

| System | Data In | Data Out | Interface Contract |
|---|---|---|---|
| Card Data & Pool | Keyword array per card, parameterized values (RANGE max_range, RESISTANCE value, etc.) | — | This GDD is the authoritative spec for what each keyword declaration in `cards.json` means. |
| Combat Resolution | Sub-step structure, modifier stack (steps 1–10), trigger timing | Keyword effect execution within sub-steps; trigger chain rules | Combat Resolution owns execution timing; this GDD owns what each keyword does within that timing. Keywords must not contradict CR sub-step assignments. |
| Round State Machine | DRAFT phase entry, RESOLUTION_COMPLETE | START/END OF TURN effects per card | RSM fires phase events; Keyword System specifies what START/END OF TURN cards do. |
| Board/Lane System | Cell positions, lane layout, CHANGE LANE slot validity, collision model | REPEL/ATTRACT/TELEPORT/CHANGE LANE displacement results | Movement keywords use the F1 formula and collision rules from board-lane-system.md. |
| Server-side RNG | RESOLUTION RNG chain | RANGE equidistant selection, TELEPORT random-destination, Strich lane selection | All three seed slots registered in ADR-005 (2026-05-01): `RangeEquidistantSelect` (Orders 4), `TeleportRandomDest` (Order 5), `StrichChangeLaneSelect` (Order 6). OQ-KS1 RESOLVED. |
| Network Protocol | `S2CResolutionEvent` | New `DisplacementEvent` variant needed | REPEL/ATTRACT/TELEPORT animation requires `{ unit_id, keyword: DisplacementKind, from_cell, to_cell, sub_step }` in ResolutionEvent. Must be added to network-protocol.md (OQ-NP1). |
| Class System | SILENCE, STUN, LEADER base rules | Class-specific keyword interactions | Class System GDD defines class overrides. This GDD defines base rules; class overrides are additive, never contradictory. |

## Formulas

### Formula 1: REPEL Displacement

The `repel_destination` formula is defined as:

`repel_destination = clamp(target_cell + (−advance_dir(target.owner)) × X, 1, 8)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Target current cell | `target_cell` | u8 | 1–8 | Cell the displaced unit currently occupies |
| Push distance | `X` | u8 | 1–6 | Keyword parameter; maximum cells pushed |
| Owner direction | `advance_dir(owner)` | i8 | +1 or −1 | Player A = +1 (advances toward Cell 8); Player B = −1 (advances toward Cell 1) |
| Negated direction | `−advance_dir` | i8 | −1 or +1 | "Toward own side" = opposite of advance direction |
| Output | `repel_destination` | u8 | 1–8 | Destination cell after clamping |

**Output Range:** [1, 8] — always valid; pushing past Cell 1 or Cell 8 clamps at the board edge (unit is not destroyed or returned to hand).

**Implementation note (Rust integer types):** the intermediate expression `target_cell + (−advance_dir) × X` can produce negative values (e.g., Player A at Cell 2, REPEL 6 → intermediate = −4). Compute the intermediate in `i32` (or at least `i16`), apply `clamp(_, 1, 8)`, then cast the clamped result to `u8`. A naive `u8` arithmetic implementation will underflow (saturate or panic in debug; wrap to 250+ in release) — same convention as `combat-resolution.md` `net_damage`. **Traversal iteration bound:** when iterating intermediate cells (for Trap-trigger purposes), iterate cells *strictly between* the start cell and the clamped final destination — exclusive of the start cell. A unit clamped to its current cell traverses zero cells.

**Example:** Player A unit at Cell 6, REPEL 3. `advance_dir(Player A) = +1`. `repel_destination = clamp(6 + (−1) × 3, 1, 8) = clamp(3, 1, 8) = 3`.

---

### Formula 2: ATTRACT Displacement

The `attract_destination` formula is defined as:

`effective_pull = min(X, |caster_cell − target_cell|)`
`attract_destination = target_cell + sign(caster_cell − target_cell) × effective_pull`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Caster current cell | `caster_cell` | u8 | 1–8 | Cell of the unit applying ATTRACT |
| Target current cell | `target_cell` | u8 | 1–8 | Cell of the unit being pulled |
| Pull distance | `X` | u8 | 1–6 | Keyword parameter; maximum cells pulled |
| Effective pull | `effective_pull` | u8 | 0–6 | Capped so target never passes caster's cell (friendly) or 1 cell short of it (enemy) |
| Direction sign | `sign(...)` | i8 | −1, 0, or +1 | Direction from target toward caster; 0 if already co-located |
| Output | `attract_destination` | u8 | 1–8 | Destination cell after pull |

**Precondition:** Caster and target must be in the same lane. Cross-lane ATTRACT is not supported by this formula. Cross-lane repositioning requires TELEPORT with card text specifying the destination lane. The formula has no `lane` parameter — lane-locality is enforced by the server before the formula runs, not by the formula itself.

**Collision rule — enemy vs friendly targets:** Opposing units (from different players) can never occupy the same cell. For **enemy targets**, the server applies the 1-cell-apart collision rule: `effective_pull = min(X, max(0, |caster_cell − target_cell| − 1))`. This ensures the enemy target stops 1 cell short of the caster's cell. For **friendly targets** (same player), co-occupation is allowed: `effective_pull = min(X, |caster_cell − target_cell|)` (the formula as written). The formula above shows the friendly-target form; the server must branch on target ownership before applying it.

**Implementation note (Rust integer types):** the intermediate expression `target_cell + sign(...) × effective_pull` can produce negative values if done in `u8`. Compute the intermediate in `i32`, apply `clamp(_, 1, 8)` if needed, then cast the clamped result to `u8`. Same convention as Formula 1.

**Output Range:** [1, 8]. `effective_pull = 0` if caster and target already share a cell (no movement; only valid for friendly targets). **`sign(0)` note:** when `caster_cell == target_cell`, `effective_pull = min(X, 0) = 0`, so `sign(0)` does not affect the output (0 × anything = 0). The implementation may use `i8::signum()` safely.

**Example (friendly pull):** Caster (Player A) at Cell 5, friendly target at Cell 7, ATTRACT 4. `effective_pull = min(4, |5 − 7|) = min(4, 2) = 2`. `attract_destination = 7 + sign(5 − 7) × 2 = 7 + (−1) × 2 = 5`. Target lands at Cell 5, co-located with caster ✓ (same player, co-occupation allowed).

**Example (enemy pull):** Caster (Player A) at Cell 5, enemy target (Player B) at Cell 7, ATTRACT 4. `effective_pull = min(4, max(0, |5 − 7| − 1)) = min(4, 1) = 1`. `attract_destination = 7 + sign(5 − 7) × 1 = 7 − 1 = 6`. Enemy lands at Cell 6 (1 cell short of caster) ✓ (1-cell-apart collision rule enforced).

**Implementation note (Rust integer types):** `sign(caster_cell − target_cell)` where both are `u8`. In Rust, `u8 − u8` underflows when `target_cell > caster_cell` (panics in debug, wraps in release). Compute `(caster_cell as i32 − target_cell as i32)` in `i32`, apply `i32::signum()`, then use the result. Same class of defect as Formula 1 — same fix required.

**Collision rule for enemy targets:** the "1-cell-apart" rule for opposing units applies. An enemy unit pulled by ATTRACT stops 1 cell short of the caster's cell — it cannot share the caster's cell. `attract_destination` as computed by the formula gives the mathematical destination; the server then applies the collision rule: if the target is an enemy and `attract_destination == caster_cell`, the actual destination is `caster_cell − advance_dir(caster.owner)` (1 cell short on the approach side). For friendly targets, the unit can stop at the caster's cell. TELEPORT is the only displacement that bypasses the 1-cell-apart rule (co-occupation explicitly allowed for TELEPORT).

---

### Formula 3: OUTNUMBERED Board Count

The `outnumbered` condition is defined as:

`outnumbered(player) = count(alive_units(player)) < count(alive_units(opponent))`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Player unit count | `count(alive_units(player))` | u8 | 0–10 | All alive Minions + Structures owned by this player, across all lanes. **Excluded:** Traps (face-down, not fighting), Fields (passive lane-wide effects with no HP — excluded same as Traps). **Confirmed max = 10** (5 lanes × 1 Minion + 5 lanes × 1 Structure; verified against `board-lane-system.md` 1-Minion-per-lane + 1-Structure-per-lane model). |
| Opponent unit count | `count(alive_units(opponent))` | u8 | 0–10 | Same, for opponent. |
| Output | `outnumbered` | bool | false/true | True only when strictly fewer; equal counts = false. |

**Output Range:** `false` (not outnumbered) or `true` (outnumbered). Evaluated at each sub-step boundary — after the preceding sub-step fully completes, before the current sub-step begins. Cannot flip intra-sub-step (e.g., mid-SS4 death chain). The OUTNUMBERED state used in SS5 reflects the full board state after all SS4 deaths resolve. The server maintains a per-player `outnumbered_cache: bool` (internal state, not replicated) to detect transitions and emit `OutnumberedFlipped` only when the boolean changes.

**Example:** Player has 2 Minions on board; opponent has 4 Minions + 1 Structure = 5 units. `2 < 5 = true`. OUTNUMBERED bonus is active.

---

### Formula 4: Damage Modifier Reference (owned by combat-resolution.md)

RESISTANCE X, VULNERABILITY X, and ARMOR-PIERCING are evaluated within the `net_damage` formula (owned by `combat-resolution.md`, registered in `design/registry/entities.yaml`). They are not re-defined here; the full modifier stack is in `combat-resolution.md` Detailed Design — Combat Modifier Stack, steps 1–10.

For reference: `VULNERABILITY X` increases `ATK_effective` by X at modifier-stack step 5. `RESISTANCE X` then reduces `ATK_effective` by X (floor 0) at step 6, *after* LEADER bonus, type advantage, and VULNERABILITY have been applied — the term `ATK_effective` refers to the running value at that step in the stack, not the base card stat. `ARMOR-PIERCING` sets `AR_effective = 0` at step 7, independently of RESISTANCE (RESISTANCE is not bypassed by ARMOR-PIERCING). See `combat-resolution.md` Detailed Design — Combat Modifier Stack for the full step ordering.

## Edge Cases

**INJURED-grantable keyword exhaustive list:** INJURED can grant exactly these four keywords: `FIRST STRIKE`, `COUNTERATTACK`, `RANGE`, `SHIELD`. These correspond to the `GrantedKeyword` enum in the Network Protocol GDD (D.3): `FirstStrike`, `Counterattack`, `Range`, `Shield`. This is a closed list — adding a new INJURED-grantable keyword requires updating both this GDD and the NP GDD's `GrantedKeyword` enum. Design rationale: all four are defensive or reactive (the "wounded animal" pattern). Movement keywords (HASTE, CHARGE), identity keywords (WALL, LEADER), and status-application keywords (STUN, SILENCE, ATTRACT, REPEL) are explicitly excluded. **INJURED-granted SHIELD timing:** SHIELD granted via INJURED activates at the sub-step boundary where INJURED was acquired — the same rule as FIRST STRIKE (KW-007). A unit damaged in SS3 becomes INJURED at the SS3→SS4 boundary; SHIELD is available from SS4 onward in the same RESOLUTION, not retroactively in SS3. **Outnumbered note:** whether OUTNUMBERED can be granted via INJURED is an open design question (OQ-KS-new-2 — see future playtesting gate).

**LEADER stacking edge case:** Two LEADER units of the same family alive at RESOLUTION entry — only the earlier-placed LEADER grants its bonus; the later LEADER's bonus is suppressed. "Earlier-placed" is determined by placement timestamp within the session, which is deterministic and observable. If a player has two LEADER-A units and one is killed in SS4, the surviving LEADER-A continues to grant its bonus (whichever one survives). Two LEADERs of different families (LEADER-A and LEADER-B) each grant their own bonuses independently.

**SILENCE on WALL — worked example:** Player A builds around WALL at Cell 4 as a blocking anchor. Player B plays a SILENCE card (APPEARANCE trigger, SS1) targeting the WALL. Result: WALL's `keywords` array entry is stripped, losing its blocking behavior. Enemy units no longer halt at Cell 4 — they advance past it and fight in SS6. WALL retains its MP=0 card stat (the unit still physically can't self-move), so it stays at Cell 4, but it no longer acts as a blocking anchor. The WALL unit now takes damage from enemies who pass through. **How anticipation was possible:** the SILENCE card is in the card pool (players can read opponent's acquisition history). The counter-play was available: BODYGUARD protecting the WALL would have shielded it from Spell/Order targeting — but BODYGUARD itself is also stripped by SILENCE. The deep counter-play is board position (a WALL in a lane the opponent can't easily reach to play a SILENCE-targeted spell). If the receiving player set up their WALL in a reachable position without protecting it, SILENCE delivered the "I read them" payoff cleanly.

**If SILENCE is applied via APPEARANCE in SS1:** The silenced unit's DEATH trigger (and all other trigger hooks) are stripped for the duration. When that unit later dies in SS4, DEATH does not fire. SILENCE is not sub-step scoped for stripping — once applied, the loss persists for the SILENCE duration. If the SILENCEd unit had FINAL BLOW, FINAL BLOW also does not fire when it kills an enemy.

**If a DEATH trigger chain kills multiple units in sequence:** No hard chain depth cap. The server tracks an "already-dead" set during SS4; a unit already queued or removed cannot die again. **Note on the "9-link bound":** the structural bound (a unit can only die once, max 10 units on board → max 9 links) is only valid when no DEATH trigger can spawn new units. If any card has a DEATH trigger that summons a new unit (e.g., "DEATH: spawn a token"), the chain can exceed 9 links as the board count fluctuates below the cap. Card authors must be aware of this: DEATH-trigger spawners in combination can produce arbitrarily long chains. The 60-second RESOLUTION safety timeout is the hard backstop. FINAL BLOW does NOT apply to kills caused by DEATH trigger chains — FINAL BLOW requires an attack in SS3 or SS6.

**If Strich's auto-CHANGE LANE trigger fires but both adjacent lanes are full:** CHANGE LANE is rejected silently. Strich stays in its current lane and fights the enemy unit that entered. The trigger fired; there was no valid destination. Card text should state "if a valid adjacent lane exists."

**If BODYGUARD is placed and the controller wants to protect a unit in a different lane:** Allowed. BODYGUARD protection is board-wide — the controller may name any other friendly unit on the board regardless of lane. Protection is unit-scoped and follows the protected unit even if it CHANGE LANEs.

**If a RANGE attacker targets an UNTARGETABLE unit:** RANGE attacks are combat targeting, not Spell/Order targeting. UNTARGETABLE only blocks Spells and Orders. A RANGE unit's nearest-enemy proximity selection is unaffected by UNTARGETABLE.

**If ATTRACT X pulls a unit through a Trap cell:** The unit traverses intermediate cells sequentially for Trap-trigger purposes. If a Trap is on an intermediate cell, the Trap triggers on entry. If the Trap kills the displaced unit, displacement ends at that cell. If the Trap STUNs the unit, displacement ends at that cell.

**If a unit with CHARGE X is TELEPORTed in SS1 by an APPEARANCE effect:** CHARGE X in SS2 executes from the post-TELEPORT cell, not the placement cell. A unit TELEPORTed to Cell 5 in SS1 with CHARGE 2 advances from Cell 5 to Cell 7 in SS2.

**If a SILENCEd unit has the WALL keyword:** SILENCE strips the WALL keyword. The unit retains its MP=0 card stat so it still moves 0 cells per round. However, it is no longer a blocking anchor — advancing enemy units no longer halt at its cell.

**If OUTNUMBERED status would flip mid-DEATH chain (SS4):** OUTNUMBERED is evaluated at sub-step boundaries, not intra-sub-step. Mid-chain deaths do not retroactively update OUTNUMBERED status during SS4. The count used for SS5 reflects the full board state after all SS4 deaths resolve.

**If a unit gains COUNTERATTACK via INJURED, and it was not INJURED at SS6 entry:** If INJURED activates at the SS3→SS4 boundary (unit damaged in SS3), COUNTERATTACK is available from SS4 onward, including SS6. If the unit becomes INJURED only at the SS6→END boundary (damaged in SS6), COUNTERATTACK is not available in SS6; it activates for the next round's SS3/SS6.

**If a WALL unit has SHIELD:** SHIELD absorbs the first SS6 attack (no damage). The attacking unit remains at the WALL's cell. The second round's SS6 attack deals damage normally. WALL + SHIELD is a two-hit blocker by design.

**If BODYGUARD is killed by a FINAL BLOW-triggering attacker:** BODYGUARD protection ends at the instant BODYGUARD's HP reaches 0, before FINAL BLOW resolves. The formerly-protected unit is exposed from that point forward, starting the next PLACEMENT phase.

**INJURED classification:** INJURED is a *state*, not a keyword. The Timing Trigger Catalog lists it as a hook because cards can grant bonuses *while* INJURED, but the state itself is computed (`current_HP < max_HP`) and is never carried in the `cards.json` `keywords` array. SILENCE strips INJURED-granted bonuses (e.g., FIRST STRIKE while INJURED) but does not clear the INJURED state. The "~28 keywords" count in the Overview is approximate and does NOT include INJURED.

**BODYGUARD bond storage:** BODYGUARD protection MUST be implemented as a unit-to-unit entity reference (`bodyguard_protects: Option<EntityId>` on the BODYGUARD's component), NOT as a lane-scoped attribute. The bond survives any CHANGE LANE the protected unit executes — the entity reference is stable across position changes. A naive lane-scoped storage would silently orphan the bond when the protected unit moves lanes; this is forbidden.

**LEADER placed this round:** A LEADER unit placed in SS1 of round R IS included in the SS1-end snapshot (taken after all SS1 APPEARANCE effects resolve, before SS2). The LEADER grants its bonus in round R. The LEADER receives its own family bonus as well (it counts as a family member). If the LEADER is killed by an APPEARANCE effect in SS1 before the snapshot is taken, it is absent from the snapshot and grants no bonus this round. HASTE does not affect snapshot timing (snapshot is taken at SS1-end regardless of HASTE). This rule supersedes the prior "deferred to R+1" design (changed in R3 per design decision D8). *(Note: Combat Resolution GDD must be updated to reflect the SS1-end snapshot timing — tracked in OQ-KS9.)*

**REPEL displacement Trap traversal:** REPEL follows the same Trap-traversal rules as ATTRACT. The unit traverses intermediate cells sequentially (strictly between start and final destination, exclusive of start). If a Trap on an intermediate cell kills the displaced unit, displacement ends at that cell. If a Trap STUNs the unit, displacement ends at that cell. Non-lethal, non-STUN Trap damage does NOT terminate displacement — the unit continues to its computed destination, and any subsequent intermediate Trap also fires.

**STUN + COUNTERATTACK:** A STUNned unit does NOT fire COUNTERATTACK when attacked. STUN suppresses all reactive keyword hooks — COUNTERATTACK, DEATH trigger, and all other trigger-based keywords. The STUNned unit takes incoming damage normally but produces no retaliation.

**FIRST STRIKE + WALL kill in SS3:** A FIRST STRIKE unit that deals lethal damage to a WALL in SS3 removes the WALL anchor. WALL is removed in SS4. Advancing enemies in SS5 no longer halt at the former WALL cell. This is the intended counter-play to lane-anchor WALL strategies.

**RANGE + WALL interaction:** A RANGE unit targets enemies by cell proximity, not by blocking behavior. If a WALL is the nearest enemy, RANGE attacks WALL. WALL's movement-halt rule applies to advancing enemies in SS5 — it does not create a targeting shield against RANGE. RANGE cannot "shoot through" a WALL to hit a more distant enemy.

**ATTRACT enemy collision rule:** The 1-cell-apart rule for opposing units applies during ATTRACT. An enemy unit pulled by ATTRACT stops 1 cell short of the caster's cell. If the formula result equals the caster's cell, the server reduces the destination by 1 cell (on the approach side). For friendly targets, co-location with the caster is permitted. TELEPORT explicitly bypasses this rule (co-occupation allowed).

**SILENCE + IRREMOVABLE:** SILENCE strips all keywords including IRREMOVABLE. A SILENCEd IRREMOVABLE unit can be displaced by REPEL, ATTRACT, or TELEPORT for the duration of the SILENCE. The SILENCE + REPEL two-card combo is an intentional counter-play to IRREMOVABLE lane anchors. Once SILENCE expires, IRREMOVABLE returns and the unit is immovable again.

**SILENCE + UNTARGETABLE:** SILENCE strips UNTARGETABLE. A SILENCEd UNTARGETABLE unit becomes a valid Spell/Order target for the SILENCE duration. This is intentional — SILENCE is the game's universal keyword strip. If the unit also has BODYGUARD protection from another unit, the BODYGUARD's protection continues to block Spell/Order targeting of the formerly-UNTARGETABLE unit (BODYGUARD protection is separate from UNTARGETABLE).

**BODYGUARD + UNTARGETABLE on the same unit — immune to SILENCE:** A BODYGUARD unit that also has UNTARGETABLE cannot be targeted by opponent Spells (including SILENCE — SILENCE is a Spell/Order). Without targeting BODYGUARD directly, an opponent cannot strip BODYGUARD's protection via SILENCE. This combo's only counters are combat damage to BODYGUARD or RANGE attacks on the protected unit. See Dangerous Combinations table.

**BODYGUARD with no valid target at entry:** If a BODYGUARD unit enters the board when no other friendly unit is alive, it enters with no protection bond (`bodyguard_protects = None`). The bond can only be established at SS1 entry — if no valid target existed at that moment, no bond is formed. BODYGUARD fights normally but provides no protection this RESOLUTION.

**BODYGUARD executes CHANGE LANE:** If the BODYGUARD unit itself (not the protected unit) executes CHANGE LANE, the protection bond persists — the entity reference is stable across position changes. The connector visual updates to reflect the new BODYGUARD position. Protection remains board-wide regardless of which lanes BODYGUARD and the protected unit occupy.

**INJURED via APPEARANCE in SS1:** A unit damaged by an APPEARANCE effect in SS1 becomes INJURED at the SS1→SS2 boundary. INJURED-granted bonuses (FIRST STRIKE, COUNTERATTACK, RANGE, SHIELD) are active from SS2 onward in the same RESOLUTION — including SS3 (for INJURED-granted FIRST STRIKE). This is a powerful card synergy: an APPEARANCE effect that deals 1 damage to a friendly unit can intentionally trigger INJURED bonuses, allowing FIRST STRIKE activation in SS3 of the same round. Card-pool authors must be aware of this interaction when designing APPEARANCE effects with self-damage.

**SHIELD persisting across rounds:** SHIELD has no round limit — it persists until consumed in SS3 or SS6 of any RESOLUTION, or until the unit leaves the board. A unit with SHIELD that is never attacked retains SHIELD indefinitely across multiple rounds.

**TELEPORT + STUN:** STUN persists through TELEPORT. STUN is an entity-level state (`stunned_until_round` on the unit's component), not a positional attribute. A STUNned unit TELEPORTed to a new cell remains STUNned for the remainder of the current RESOLUTION.

**IRREMOVABLE + CHANGE LANE:** IRREMOVABLE prevents displacement by opponent effects (REPEL, ATTRACT, TELEPORT, Spells, Orders). It does NOT affect the unit's own movement. An IRREMOVABLE unit can freely execute CHANGE LANE (own movement).

**DEATH chain re-entry prevention:** The server maintains an "already-dead" set during SS4. A unit that has already been removed from the board (or is queued for removal) cannot be killed again and cannot re-enter the chain. This prevents DEATH-trigger loops. If unit A's DEATH trigger would deal damage to unit A again (e.g., area-of-effect), the second death is silently suppressed for already-dead entities.

## Dependencies

### Upstream (Keyword System depends on these)

| System | GDD | Interface | Nature |
|---|---|---|---|
| Card Data & Pool | `card-data-pool.md` | Keyword array per card; parameterized values (RANGE max_range, RESISTANCE/VULNERABILITY value, CHARGE X cells); `cards.json` keyword schema | Hard |
| Combat Resolution | `combat-resolution.md` | Sub-step ordering (SS1–SS6); modifier stack (steps 1–10); trigger timing; INJURED/DEATH/FINAL BLOW execution framework | Hard |
| Board/Lane System | `board-lane-system.md` | Cell positions [1–8]; lane layout; 1-Minion-per-lane slot rule (CHANGE LANE validation); F1 movement formula; collision model | Hard |
| Round State Machine | `round-state-machine.md` | DRAFT phase entry event (START OF TURN timing, after RSM Rule 3); RESOLUTION_COMPLETE event (END OF TURN timing) | Hard |
| Game Config | `game-config.md` | Keyword parameter safe ranges (future use if global keyword knobs are added) | Soft |
| Server-side RNG | `server-rng.md` | RESOLUTION RNG chain — seed slot for RANGE equidistant target selection and TELEPORT random destination (OQ-KS1) | Hard |

### Downstream (these depend on Keyword System)

| System | GDD | What it consumes | Nature |
|---|---|---|---|
| Class System | `class-system.md` | Base keyword definitions; SILENCE, STUN, LEADER, INJURED rules as extension points | Soft — extends, never contradicts |
| Board Rendering | `board-rendering.md` | Persistent state indicator specs; displacement event animations | Soft |
| Card Animations | `card-animations.md` | APPEARANCE/DEATH/FINAL BLOW/COUNTERATTACK trigger visual sequences; displacement animations from `S2CResolutionEvent.DisplacementEvent` | Soft |
| Network Protocol | `network-protocol.md` | `DisplacementEvent` fully defined in NP GDD — OQ-NP1 RESOLVED. Authoritative schema: `DisplacementEvent { unit_id: EntityId, attacker_id: Option<EntityId>, from_lane: u8, from_cell: u8, to_lane: u8, to_cell: u8, kind: DisplacementKind, block_reason: Option<DisplacementBlockReason>, sub_step: u8 }`. Fields renamed from keyword GDD's original spec: `keyword`→`kind`; `was_blocked: bool`→`block_reason: Option<DisplacementBlockReason>` (richer encoding); `from_lane`/`to_lane` added for cross-lane TELEPORT. | Soft — additive |

### Bidirectional consistency note

Combat Resolution is both an upstream dependency (sub-step structure) and has the Keyword System as a downstream dependent (keywords extend the modifier stack). The interface contract: Combat Resolution owns *when* things execute; Keyword System owns *what each keyword does*. Keywords must not reassign sub-step timing. Sub-step timing changes require modifying `combat-resolution.md` first.

## Tuning Knobs

> Keyword parameters (RANGE X, RESISTANCE X, VULNERABILITY X, CHARGE X cells, REPEL X, ATTRACT X) are **per-card values in `assets/data/cards.json`**, not global `game_config.ron` fields. There are no system-level global knobs for the Keyword System. All tuning is at the card data level.

| Knob | Location | Default | Safe Range | Gameplay Impact |
|---|---|---|---|---|
| RANGE 1-X (max_range) | Per-card `cards.json` | 1–3 | 1–6 | Above 4: unit can reach objectives from mid-board |
| RESISTANCE X value | Per-card `cards.json` | 1–2 | 1–4 | Higher = nearly unkillable by low-ATK attackers; pairs with AR |
| VULNERABILITY X value | Per-card `cards.json` | 1–2 | 1–4 | Glass-cannon identity; stackable with ARMOR-PIERCING |
| CHARGE X cells | Per-card `cards.json` | 1–3 | 1–6 | Above 4: unit can cross the midboard and hit objective in round of play |
| REPEL X / ATTRACT X distance | Per-card `cards.json` | 1–3 | 1–6 | Above 4: can displace unit across an entire player side in one effect. **Card authoring rule: REPEL 0 and ATTRACT 0 are forbidden.** X=0 produces a no-op displacement — the server silently skips it (no DisplacementEvent emitted, no animation). Card authors must not create cards with X=0 displacement values. |
| SILENCE duration | Per-card structured field (see OQ-KS-new) | 1 RESOLUTION | 1 RESOLUTION | Multi-round SILENCE is not designed; keep to 1 RESOLUTION unless explicitly playtested. **Note:** `effect_text` is display-only; server reads `silence_duration_rounds: u8` (a structured field that must be added to `cards.json` schema — tracked in OQ-KS-new). |
| STUN duration | Hardcoded rule | 1 RESOLUTION | 1 RESOLUTION | Multi-round STUN is too punishing; do not increase without playtesting. **Wire protocol note:** `KeywordPayload::StunApplied { duration_rounds: u8 }` carries a `duration_rounds` field for forward-compatibility — the server MUST always emit `duration_rounds = 1`. Any value > 1 is a server bug in current design. |

**Future GameConfig candidates:** If OUTNUMBERED threshold needs adjustment from strict `<` to `≤`, add `outnumbered_threshold_mode` to `game_config.ron`.

**OUTNUMBERED flip risk (design tension):** OUTNUMBERED is evaluated per sub-step boundary and can deactivate mid-RESOLUTION. A player who builds around OUTNUMBERED may watch their highly effective FIRST STRIKE unit kill its way to parity in SS3, losing the OUTNUMBERED bonus before SS5/SS6. This is intentional tension: the bonus rewards being behind at each moment, not staying behind. Card-pool authors should design OUTNUMBERED units with this in mind — the bonus may not be active when it is most needed if the unit performs well in SS3.

### Dangerous Combinations (card-pool authoring guidance)

Per-card values stay in safe ranges, but **specific keyword stacks become degenerate at high X**. Card-pool authors must consult this list before assigning combinations to a single card. None of these are forbidden; they are flagged as Legendary/Epic-only stat budgets that should not appear on common-rarity cards.

| Combination | Failure mode | Author rule |
|---|---|---|
| **CHARGE 5–6 + HASTE + MP ≥ 1** | Round-1 objective damage from spawn (Cell 1 → Cell 8 in SS2 + SS5). | Reserve CHARGE 5+ to Legendary cards with HP ≤ 3 and no HASTE; OR HASTE units with CHARGE ≤ 2. |
| **RANGE 5–6 + FIRST STRIKE + HASTE** | Cell-1 unit hits Cell 6–7 in SS3 of placement round, immune to COUNTERATTACK. | Reserve to Legendary; pair with low HP (≤ 3) and high cost (≥ 6 mana). |
| **WALL + SHIELD + IRREMOVABLE + RESISTANCE X** | 2+ round lane stall with no displacement counter; opponent's only answer is high-ATK + ARMOR-PIERCING. | Forbid in card pool: at most 3 of these 4 keywords on any single WALL unit. |
| **BODYGUARD + UNTARGETABLE on same unit** | Spell/Order targeting completely defeated for both this unit and the protected ally. **IMMUNE TO SILENCE** — neither the BODYGUARD (UNTARGETABLE) nor the protected unit (BODYGUARD-protected) can be targeted by SILENCE. Only counters: combat damage to BODYGUARD, or RANGE attacks on the protected unit. | Reserve to Legendary; pair with high cost (≥ 7 mana). |
| **SILENCE + REPEL (or ATTRACT/TELEPORT)** | Effective two-card counter to IRREMOVABLE — SILENCE strips IRREMOVABLE, then REPEL displaces the now-movable unit. Both cards must be played same round. | Not forbidden; budget as two-card combo cost (≥ combined mana). |
| **LEADER + SILENCE-strip immunity (hypothetical IRREMOVABLE-class keyword)** | Permanent global buff impossible to counter. | Not currently in keyword set; flag if introduced. |

### HASTE design-lever note

The HASTE rename from CHARGE was adopted to disambiguate from CHARGE X (movement). The game-designer review flagged that "HASTE" does not self-describe its trigger condition (no-summoning-sickness) any better than the original CHARGE. **If playtest shows HASTE/CHARGE confusion persisting**, fold HASTE into a no-summoning-sickness default on all cards (i.e., remove summoning sickness as a default rule and use a `Slow` keyword for the exception), then rename CHARGE X → RUSH X. Reserve as a post-vertical-slice design lever — not a Day-1 change, since the rename has already cascaded into `cards.json` schema, OQ-KS2 audit, and downstream Hand UI / Card Animations contracts.

## Visual/Audio Requirements

> Visual conventions from `combat-resolution.md` apply to all keyword visuals — color palette, impact flash timing, and STUN/SHIELD/INJURED/LEADER/SILENCE/OUTNUMBERED indicator specs are defined there. This section specifies only indicators and animations unique to the Keyword System.

### New State Indicators

**BODYGUARD active — two-element indicator:**
- **On the BODYGUARD unit:** Prism White `#EEF4FF` shield-arc glyph (6×8px) at top-left of base ring. Static. On BODYGUARD death: 2-frame split into ±30° diagonal shards, each fading over 200ms (reuses existing 4px particle sprites — 0 additional atlas frames).
- **On the protected unit:** Three Prism White 3px dots tracing the shortest path between BODYGUARD's glyph and the protected unit's base ring (procedural connector, not a sprite). Cross-lane connections allowed. Static while bond is active; disappears when BODYGUARD dies.
- Atlas cost: 1 static frame (BODYGUARD arc glyph).

**IRREMOVABLE:**
- Void `#0D0D14` chain-link glyph (6×6px — two interlocked 45°-rotated squares) at bottom-center of base ring. Static.
- On displacement attempt: 1-frame Void flat flash (full-sprite 15% opacity, fades 100ms). No displacement animation plays.
- Atlas cost: 1 frame.

**UNTARGETABLE:**
- Ivory `#F7F0DC` diamond outline (6×6px) with a 2px diagonal cross-stroke, at top-right of base ring. Static. Represents "targeting reticle broken."
- Atlas cost: 1 frame.

### Glyph Position Map (all indicators, no conflicts)

| Indicator | Position | Color |
|---|---|---|
| STUN stars | Orbiting base ring | Prism White `#EEF4FF` |
| SHIELD hex | Base ring center | Prism White `#EEF4FF` |
| LEADER crown | Above unit head | Arcane Gold `#F5C842` |
| BODYGUARD arc | Top-left of base ring | Prism White `#EEF4FF` |
| IRREMOVABLE chain | Bottom-center of base ring | Void `#0D0D14` |
| UNTARGETABLE diamond | Top-right of base ring | Ivory `#F7F0DC` |
| INJURED outline | Full unit perimeter | Void ↔ rust-brown pulse |
| SILENCE outline | Full unit perimeter | Desaturate to `#666666` |

A SILENCED + INJURED unit simultaneously shows a grey outline that oscillates between `#666666` and rust-brown — both states readable in one visual.

### Displacement Animations (REPEL, ATTRACT, TELEPORT)

All complete in ≤480ms (6-cell max at 80ms/cell).

**REPEL X** — `EaseInQuad` straight-line slide toward target's own side (accelerates away = physical push). Warm orange `#E07020` flat flash (40% opacity) at impact, fades 80ms. Trail: 1 fading sprite copy (35% opacity, 0.5 cells behind, 120ms). Arrival: 1-frame Void shadow-burst (20% opacity, 4px radius), fades 80ms. **Mnemonic: orange = pushed.**

**ATTRACT X** — `EaseOutQuad` straight-line slide toward caster's cell (decelerates into arrival = pulled in gently). Arcane Gold `#F5C842` flat flash (35% opacity) at pull initiation, fades 100ms. Trail: 1 fading sprite copy (35% opacity, 0.5 cells behind). No arrival burst. **Mnemonic: gold = pulled.**

**TELEPORT** — No translate. Two-beat blink:
- *Exit:* sprite squashes to 0px height over 80ms → 1-frame Prism White `#EEF4FF` horizontal bar (4px height, full sprite width at base-ring level), fades 60ms.
- *Entry:* Prism White bar appears at destination for 1 frame → sprite expands from 0px to full height over 80ms (`EaseOutQuad`). Total: ~300ms.
- No trail. Does NOT play Arcane Gold APPEARANCE aura pulse (TELEPORT does not trigger APPEARANCE). **Mnemonic: Prism White blink = magical repositioning.**

### START OF TURN / END OF TURN Trigger Visual

**START OF TURN** — Arcane Gold `#F5C842` base ring pulse (radiates to ~2× unit width, fades 200ms — identical to APPEARANCE/DEATH pulse). During DRAFT phases only: Ivory `#F7F0DC` floating label `"START OF TURN"` (0.8× base typography), fades 600ms. Non-blocking.

**END OF TURN** — Identical Arcane Gold pulse (200ms). All END OF TURN triggers fire simultaneously (global pass). No text label during RESOLUTION_COMPLETE (would compete with objective damage floats). If overlapping with gold reward floats: END OF TURN pulse reduces to 50% opacity for 250ms overlap window (combat economy feedback has visual priority).

### Atlas Budget

| Category | Frames |
|---|---|
| Combat VFX (from `combat-resolution.md`) | ~21 |
| IRREMOVABLE chain-link glyph | 1 |
| UNTARGETABLE diamond-cross glyph | 1 |
| TELEPORT Prism White bar (exit + entry) | 2 |
| **Total** | **~25 / 120 budget** |

BODYGUARD, REPEL, ATTRACT, START/END OF TURN all reuse existing atlas sprites — 0 additional frames.

📌 **Asset Spec** — Visual/Audio requirements defined. After the art bible is approved, run `/asset-spec system:keyword-system` to produce per-asset visual descriptions and generation prompts for the new glyph sprites.

## UI Requirements

The Keyword System owns no interactive UI screen or panel. Keyword display is a display contract on downstream systems:

- **Card face display** (Hand UI GDD): keywords rendered as bold badge tags on each card in hand (e.g., `RANGE 1-3`, `RESISTANCE 2`). HASTE badge must reflect the rename — update card text strings in `cards.json` alongside the keyword schema change.
- **Board unit state indicators** (Board Rendering GDD): all persistent keyword state glyphs defined in Visual/Audio Requirements above — BODYGUARD, IRREMOVABLE, UNTARGETABLE, plus all indicators from `combat-resolution.md`.
- **Displacement animation playback** (Card Animations GDD): REPEL/ATTRACT/TELEPORT animation specs from Visual/Audio Requirements, driven by the `S2CResolutionEvent.DisplacementEvent` variant (see OQ-NP1).

No interactive elements. No modal dialogs. Keywords are intended to be self-describing from their card text. COUNTERATTACK is now fully card-text-derivable: "this unit retaliates when attacked by any non-RANGE attacker" — no tooltip exception required (simplified in R3, design decision D3).

**SILENCE high-visibility application moment:** When SILENCE is applied to a unit during RESOLUTION, the client MUST play a high-visibility animation showing all stripped keyword glyphs fading off the unit simultaneously (glyph icons briefly highlighted, then dissolve over 200ms). This is a VFX requirement, not a tooltip. The animation communicates to the affected player exactly what was lost — supporting the "this felt fair" perception of SILENCE as a totalizing read. See Visual/Audio Requirements for SILENCE's desaturation indicator spec.

## Acceptance Criteria

### Timing Triggers

| # | Criterion | Type |
|---|---|---|
| KW-001 | GIVEN a unit with an APPEARANCE trigger enters the board in SS1, WHEN sub-step 1 resolves, THEN the APPEARANCE effect executes before any DEATH trigger chains that result from APPEARANCE-caused kills. | BLOCKING |
| KW-002 | GIVEN two units in different lanes are killed in the same sub-step, WHEN sub-step 4 removes dead units, THEN DEATH triggers fire in lane order (Lane 1 before Lane 5); Lane 2 unit's DEATH trigger resolves completely before Lane 4 unit's begins. | BLOCKING |
| KW-003 | GIVEN unit A has a DEATH trigger that deals lethal damage to unit B (also has a DEATH trigger), WHEN unit A is removed in SS4, THEN A's DEATH trigger resolves completely, then B is removed, then B's DEATH trigger fires. | BLOCKING |
| KW-004a | GIVEN a unit with FINAL BLOW is killed by a FIRST STRIKE attacker in SS3, WHEN the killing blow reduces HP to 0 in SS3, THEN FINAL BLOW fires in SS3 and NOT in SS4. | BLOCKING |
| KW-004b | GIVEN a unit with FINAL BLOW is killed by standard combat in SS6, WHEN the killing blow reduces HP to 0 in SS6, THEN FINAL BLOW fires in SS6 and NOT in SS4. | BLOCKING |
| KW-005 | GIVEN a COUNTERATTACK unit receives damage from a RANGE attacker, WHEN the RANGE attack resolves, THEN COUNTERATTACK does NOT fire — RANGE attacks never trigger COUNTERATTACK. | BLOCKING |
| KW-006 | GIVEN a COUNTERATTACK unit receives damage from a non-RANGE attacker in SS6, WHEN the attack resolves, THEN COUNTERATTACK fires. | BLOCKING |
| KW-007 | GIVEN unit X has max_HP=4, current_HP=4, and gains FIRST STRIKE when INJURED; X receives 2 damage in SS3 (reducing HP to 2), WHEN SS3 resolves, THEN X does NOT receive the INJURED-granted FIRST STRIKE during SS3; INJURED activates at the SS3→SS4 boundary, granting the bonus from SS4 onward in the same RESOLUTION. | BLOCKING |
| KW-008a | GIVEN a unit is INJURED (current_HP < max_HP) and gains FIRST STRIKE from INJURED, WHEN SILENCE is applied, THEN the INJURED-granted FIRST STRIKE is stripped and the unit no longer attacks in SS3. | BLOCKING |
| KW-008b | GIVEN a unit is INJURED and then SILENCEd, WHEN SILENCE applies, THEN the `injured` state flag on the unit remains true — SILENCE does not clear the INJURED state. | BLOCKING |
| KW-009a | GIVEN a unit with START OF TURN is alive at the start of round R+1, WHEN the DRAFT phase begins after mana ramp + gold income are applied, THEN the START OF TURN effect fires. | BLOCKING |
| KW-009b | GIVEN a unit with START OF TURN enters play on round R, WHEN round R's DRAFT phase begins, THEN START OF TURN does NOT fire for that unit on round R; it fires on round R+1 if the unit survives. | BLOCKING |
| KW-010a | GIVEN a unit with END OF TURN is alive when SS6 completes, WHEN RESOLUTION ends, THEN END OF TURN fires before the RSM round counter increments. | BLOCKING |
| KW-010b | GIVEN a unit with END OF TURN entered play on round R and survives SS6, WHEN RESOLUTION ends for round R, THEN END OF TURN fires — a unit that entered play this round is eligible. | BLOCKING |

### Combat Keywords

| # | Criterion | Type |
|---|---|---|
| KW-011 | GIVEN a FIRST STRIKE unit is in the same cell as a standard enemy unit, WHEN SS3 resolves, THEN the FIRST STRIKE unit deals damage in SS3; the enemy does NOT deal damage in SS3; if the enemy survives, it attacks in SS6. | BLOCKING |
| KW-012 | GIVEN two FIRST STRIKE units are co-located, WHEN SS3 resolves, THEN both deal damage simultaneously using pre-combat HP snapshots; neither's damage is computed after seeing the other's result. | BLOCKING |
| KW-013 | GIVEN a unit with HASTE (no STUN, no FIRST STRIKE, no CHARGE X) is placed in SS1, WHEN RESOLUTION proceeds, THEN the unit participates in SS5 movement and SS6 attacks in the same round it entered play. (FIRST STRIKE+HASTE and CHARGE X+HASTE combos are covered by KW-034 and KW-013-extension scenarios in card-pool tests.) | BLOCKING |
| KW-014 | GIVEN a unit with HASTE has STUN applied via an SS1 APPEARANCE trigger, WHEN RESOLUTION proceeds, THEN the unit skips SS2, SS3, SS5, and SS6; HASTE does not override STUN. (Combat-keyword angle on the same invariant as KW-034 cross-keyword test; one implementation test suffices.) | BLOCKING |
| KW-015a | GIVEN a STUNned unit is in the path of an enemy attack in SS3 or SS6, WHEN the attack resolves, THEN the STUNned unit takes incoming damage according to the normal damage formula; it does not attack or advance. | BLOCKING |
| KW-015b | GIVEN a unit was STUNned during RESOLUTION R, WHEN RESOLUTION R+1 begins, THEN the STUN state is cleared; the unit participates in SS2, SS3, SS5, and SS6 normally. | BLOCKING |
| KW-016 | GIVEN a RANGE 1-X unit has an enemy BODYGUARD protecting another unit within range, WHEN the RANGE unit selects its target, THEN RANGE selects by proximity (nearest cell); BODYGUARD's Spell/Order protection does not intercept RANGE targeting. | BLOCKING |
| KW-017 | GIVEN a unit with RANGE and FIRST STRIKE, WHEN RESOLUTION executes, THEN the unit attacks in SS3 AND again in SS6; SHIELD consumed in SS3 does NOT protect the same unit in SS6. | BLOCKING |
| KW-018 | GIVEN a WALL unit is at Cell 4 and an enemy unit has MP sufficient to reach or pass Cell 4, WHEN SS5 movement resolves, THEN the enemy unit stops at Cell 4 and fights WALL in SS6; WALL deals 0 damage. | BLOCKING |
| KW-019 | GIVEN unit B is protected by BODYGUARD unit G; G receives lethal damage and its HP reaches 0 in SS3 or SS6, WHEN G's HP is reduced to 0, THEN unit B's Spell/Order protection ends at that instant — not at SS4 removal; starting from the next PLACEMENT phase, B is targetable by opponent Spells and Orders. | BLOCKING |
| KW-020a | GIVEN an IRREMOVABLE unit is the target of REPEL X, WHEN REPEL resolves, THEN the unit's cell position does not change; IRREMOVABLE does not prevent the unit's own movement (MP, CHARGE X, CHANGE LANE). | BLOCKING |
| KW-020b | GIVEN an IRREMOVABLE unit is the target of ATTRACT X, WHEN ATTRACT resolves, THEN the unit's cell position does not change. | BLOCKING |
| KW-020c | GIVEN an IRREMOVABLE unit is the target of TELEPORT, WHEN TELEPORT resolves, THEN the unit's cell position does not change. | BLOCKING |
| KW-021 | GIVEN an UNTARGETABLE unit is in combat range of an enemy RANGE unit, WHEN SS6 resolves, THEN the RANGE attack hits the UNTARGETABLE unit normally; UNTARGETABLE only blocks Spell/Order targeting. | BLOCKING |
| KW-022 | GIVEN a defender with RESISTANCE 2 is attacked by a unit with ARMOR-PIERCING, WHEN the modifier stack resolves, THEN RESISTANCE 2 reduces ATK_effective by 2 first; ARMOR-PIERCING sets AR_defender to 0 independently; RESISTANCE is not bypassed by ARMOR-PIERCING. | BLOCKING |
| KW-023 | GIVEN a SILENCEd unit has COUNTERATTACK, DEATH trigger, FIRST STRIKE, and is INJURED, WHEN SILENCE applies, THEN all keyword hooks are stripped; the unit does not counterattack or trigger on death; INJURED state persists. | BLOCKING |
| KW-024 | GIVEN a unit with SHIELD is attacked by a RANGE+FIRST STRIKE attacker in SS3, then by two enemy melee units simultaneously in SS6, WHEN RESOLUTION executes, THEN SHIELD absorbs the SS3 attack and is consumed in SS3 (chronological order); in SS6, SHIELD is no longer active and both melee attackers deal full damage. | BLOCKING |
| KW-025 | GIVEN a LEADER unit is present at SS1-end (bonus snapshotted) and is killed in SS4, WHEN SS6 resolves, THEN the ATK bonus remains active for all eligible family units in SS6. | BLOCKING |
| KW-026 | GIVEN a LEADER unit is SILENCEd at SS1-end snapshot time, WHEN the bonus snapshot is computed, THEN the SILENCEd LEADER grants no bonus to family units for this RESOLUTION. | BLOCKING |
| KW-027a | GIVEN Player A has 3 units on board and Player B has 3 units on board at SS2 entry, WHEN OUTNUMBERED is evaluated for Player A, THEN the result is false — the bonus is NOT active (equal counts do not qualify). | BLOCKING |
| KW-027b | GIVEN Player A has 2 units on board and Player B has 4 units on board at SS2 entry, WHEN OUTNUMBERED is evaluated for Player A, THEN the result is true — the bonus IS active. | BLOCKING |

### Movement Keywords

| # | Criterion | Type |
|---|---|---|
| KW-028 | GIVEN a unit with CHARGE 2 is in a lane where an enemy WALL is 1 cell ahead, WHEN SS2 resolves, THEN the unit is blocked at the WALL's cell and does not pass through it. | BLOCKING |
| KW-029a | GIVEN a Player A unit at Cell 2 is REPELled 3 cells by a Player B effect, WHEN the `repel_destination` formula resolves, THEN the unit lands at Cell `clamp(2 + (−1)×3, 1, 8) = 1` (clamped at board edge). | BLOCKING |
| KW-029b | GIVEN a WALL unit at Cell 5 is REPELled 2 cells by a Player B unit (push toward Cell 8), WHEN REPEL resolves, THEN WALL moves to `repel_destination = clamp(5 + (+1)×2, 1, 8) = 7`. | BLOCKING |
| KW-029c | GIVEN a Player A unit at Cell 1 is REPELled 6 cells (maximum X), WHEN `repel_destination` resolves, THEN `clamp(1 + (−1)×6, 1, 8) = clamp(−5, 1, 8) = 1`; unit stays at Cell 1; zero intermediate cells are traversed (no Trap triggers at Cell 1 itself). | BLOCKING |
| KW-029d | GIVEN a Player B unit at Cell 8 is REPELled 6 cells (maximum X) by any effect, WHEN `repel_destination` resolves, THEN `clamp(8 + (+1)×6, 1, 8) = clamp(14, 1, 8) = 8`; unit stays at Cell 8; zero intermediate cells traversed. | BLOCKING |
| KW-030a | GIVEN Player A caster at Cell 3 ATTRACTs a friendly target at Cell 7 with ATTRACT 6, WHEN ATTRACT resolves, THEN `effective_pull = min(6, |3−7|) = 4`; target lands at Cell 3 (co-located with caster — same player co-occupation allowed). | BLOCKING |
| KW-030b | GIVEN Player A caster at Cell 3 ATTRACTs an enemy (Player B) target at Cell 7 with ATTRACT 6, WHEN ATTRACT resolves, THEN `effective_pull = min(6, max(0, |3−7|−1)) = min(6, 3) = 3`; enemy lands at Cell 4 (1 cell short of caster — 1-cell-apart collision rule enforced). | BLOCKING |
| KW-031a | GIVEN a unit is TELEPORTed to a cell occupied by an enemy unit, WHEN TELEPORT resolves, THEN no APPEARANCE trigger fires on the teleported unit. | BLOCKING |
| KW-031b | GIVEN a unit is TELEPORTed to a cell occupied by an enemy unit, WHEN TELEPORT resolves, THEN no COUNTERATTACK fires from the enemy unit at that cell. | BLOCKING |
| KW-032 | GIVEN a unit attempts CHANGE LANE to an adjacent lane that already has a friendly Minion, WHEN CHANGE LANE resolves, THEN the lane change does not execute; the unit remains in its current lane; no error state is created. | BLOCKING |
| KW-033a | GIVEN Strich is in Lane 3 and exactly one adjacent lane (Lane 2 or Lane 4) is valid (the other is full with a friendly Minion), WHEN an enemy unit enters Lane 3 in SS1, THEN Strich automatically executes CHANGE LANE to the only valid adjacent lane. | BLOCKING |
| KW-033b | GIVEN Strich is in Lane 3, both Lane 2 and Lane 4 are valid, and the seeded RNG selects Lane 2, WHEN an enemy unit enters Lane 3 in SS1, THEN Strich moves to Lane 2. (BLOCKED — requires `strich_change_lane_select` seed slot per OQ-KS1 resolution before this AC can be implemented as a deterministic test.) | BLOCKING |
| KW-033c | GIVEN Strich is in Lane 3 and both adjacent lanes (Lane 2, Lane 4) already contain a friendly Minion, WHEN an enemy unit enters Lane 3 in SS1, THEN CHANGE LANE is rejected; Strich remains in Lane 3; no error state is created. | BLOCKING |

### Cross-Keyword Interactions

| # | Criterion | Type |
|---|---|---|
| KW-034 | GIVEN a HASTE unit has STUN applied in SS1, WHEN RESOLUTION proceeds, THEN the STUNned HASTE unit skips SS2, SS3, SS5, and SS6; HASTE does not partially override STUN. (Canonical cross-keyword test for HASTE+STUN; KW-014 covers the same invariant from the combat-keyword angle.) | BLOCKING |
| KW-035a | GIVEN unit A (FIRST STRIKE) kills unit B (has DEATH trigger and FINAL BLOW) in SS3, WHEN SS3 resolves, THEN FINAL BLOW's effect is reflected in the server's authoritative per-player gold resource (exact Rust type to be named at implementation; see TODO in combat system) at SS3 completion, before SS4 begins; unit B is still present in `BoardSnapshot.units` at SS3 completion (removal is deferred to SS4). **Assertion target:** read the server's authoritative gold resource after SS3 system runs and before SS4 system runs — the gold delta from FINAL BLOW must already be recorded. Do NOT assert on event-emission ordering — ECS schedule ordering is fragile for this assertion. | BLOCKING |
| KW-035b | GIVEN unit B has a DEATH trigger and was killed in SS3 by FIRST STRIKE, WHEN SS4 resolves, THEN unit B is removed from the board AND B's DEATH trigger effect executes; kill gold is added to the attacker's gold total in SS4. | BLOCKING |
| KW-036 | GIVEN a WALL unit is SILENCEd, WHEN SS5 resolves, THEN the SILENCEd WALL loses its blocking behavior; advancing enemies no longer halt at its cell; unit still has MP=0 and does not self-move. | BLOCKING |
| KW-037 | GIVEN unit X has SHIELD, and a RANGE+FIRST STRIKE attacker hits X in SS3 (consuming SHIELD), WHEN SS6 executes the RANGE unit's second attack, THEN the SS6 attack applies full damage to X — SHIELD is no longer present in SS6 (consumed in SS3); X's HP after SS6 = HP-after-SS3 minus SS6 net damage. | BLOCKING |
| KW-038 | GIVEN unit X is BODYGUARD-protected and an enemy RANGE unit's proximity selection identifies X as the nearest enemy, WHEN the RANGE attack resolves, THEN BODYGUARD does not intercept; X can be hit by RANGE regardless of BODYGUARD. | BLOCKING |
| KW-039 | GIVEN a LEADER is un-SILENCEd at SS1-end (bonus snapshotted), then SILENCEd during SS3, WHEN SS6 resolves, THEN the snapshot bonus remains active for all eligible family units; mid-RESOLUTION SILENCE does not retroactively invalidate a legally-taken snapshot. | BLOCKING |
| KW-040 | GIVEN a DEATH trigger chain changes board counts mid-chain in SS4, WHEN OUTNUMBERED is evaluated for SS5, THEN the count reflects the full board state after all SS4 deaths resolve — not any intermediate count during the chain. | BLOCKING |
| ~~KW-041~~ | ~~ATTRACT backfire to objective cell~~ — **REMOVED in R3.** Fundamental rule: opposing units can never occupy the same cell; when they make contact they are always 1 cell apart. An enemy unit ATTRACTed by a Player A unit can never reach Player A's Cell 1 (the 1-cell-apart collision rule stops it 1 cell short of the caster). The original premise was wrong. *See OQ-KS10 for formal definition of the 1-cell-apart rule in board-lane-system.md.* | — |

### Additional Combat Keyword Interactions (new — R2 additions)

| # | Criterion | Type |
|---|---|---|
| KW-042 | GIVEN a unit with HASTE and FIRST STRIKE is placed in SS1 of round R, WHEN RESOLUTION proceeds through SS3, THEN the unit executes its FIRST STRIKE attack in SS3 of round R (the same round it entered play); it attacks again in SS6 of round R via standard melee or RANGE resolution. HASTE removes summoning sickness for all sub-steps including SS3. | BLOCKING |
| KW-043 | GIVEN a unit with HASTE and CHARGE 2 is placed in SS1 of round R at Cell 1, WHEN RESOLUTION proceeds, THEN the unit advances 2 extra cells in SS2 (landing at Cell 3), advances its MP in SS5, and attacks in SS6 — all within round R. HASTE removes summoning sickness for SS2 (CHARGE X) as well as SS5/SS6. | BLOCKING |
| KW-044 | GIVEN unit X has FIRST STRIKE and COUNTERATTACK; unit X is SILENCEd for exactly 1 RESOLUTION during round R, WHEN round R's RESOLUTION ends and round R+1 RESOLUTION begins, THEN unit X has FIRST STRIKE and COUNTERATTACK active again — SILENCE has expired; the unit attacks in SS3 of round R+1 via FIRST STRIKE and retaliates in any sub-step where it is attacked. | BLOCKING |
| KW-045 | GIVEN unit X with FIRST STRIKE and COUNTERATTACK is SILENCEd at the end of SS3 (after SS3 has resolved), WHEN SS6 resolves, THEN unit X does NOT use FIRST STRIKE in SS6 (SILENCE already active) AND unit X does NOT fire COUNTERATTACK in SS6 if attacked (SILENCE strips COUNTERATTACK); SS3 COUNTERATTACK damage already dealt before SILENCE applied is NOT reversed. | BLOCKING |
| KW-046 | GIVEN a LEADER unit with HASTE is placed in SS1 of round R, WHEN the LEADER bonus snapshot is computed at SS1-end (after all APPEARANCE effects resolve, before SS2), THEN eligible family units receive the LEADER bonus in round R — the LEADER was placed in SS1 and is present at snapshot time. (Note: this reverses the prior R2 ruling; changed in R3 design decision D8. The prior AC tested the opposite behavior.) | BLOCKING |
| KW-047 | GIVEN Player A has two LEADER units of the same family (LEADER-1 placed in round R, LEADER-2 placed in round R+1) both alive at RESOLUTION entry of round R+1, WHEN the LEADER bonus snapshot is computed, THEN eligible family units receive only LEADER-1's bonus (earlier-placed); LEADER-2's bonus is suppressed. The total bonus applied to each eligible unit equals LEADER-1's bonus value, not the sum of both. | BLOCKING |
| KW-048 | GIVEN unit A (ATK=5, COUNTERATTACK) and unit B (ATK=3, COUNTERATTACK) fight in SS6 — unit A attacks unit B; WHEN B's COUNTERATTACK fires against A, THEN A's COUNTERATTACK does NOT fire a second time (chain terminates after one COUNTERATTACK); final HP of A reflects: A's initial attack resolved by B's defense, then B's COUNTERATTACK resolved by A's defense; no further retaliation occurs. | BLOCKING |
| KW-049 | GIVEN unit X (ATK=4, COUNTERATTACK) is attacked simultaneously in SS6 by attacker A (ATK=3) and attacker B (ATK=2); WHEN SS6 resolves, THEN X retaliates against A for 4 damage (post-modifier, computed from X's pre-retaliation state) AND X retaliates against B for 4 damage (post-modifier, same pre-retaliation snapshot used for both); the HP snapshot used for X's outgoing COUNTERATTACK damage to each attacker is taken before any retaliation damage is applied to X. | BLOCKING |
| KW-050 | GIVEN unit X has COUNTERATTACK and active SHIELD; in SS6, attacker A deals damage to X, WHEN SHIELD absorbs all incoming damage (X takes 0 damage), THEN X's COUNTERATTACK still fires against A; A takes COUNTERATTACK damage equal to X's ATK (post-modifier); X's HP remains unchanged (SHIELD worked); X's SHIELD is now consumed (was used in SS6). | BLOCKING |
| KW-051 | GIVEN a Player A unit at Cell 5 is REPELled 4 cells toward its own side (Player A advance_dir = +1, so REPEL pushes to Cell 1); Cell 3 has a lethal Trap owned by Player B; WHEN REPEL resolves, THEN the unit enters Cell 4 (1 cell traversal), Cell 3 (2nd traversal — Trap triggers on entry, deals lethal damage); unit dies at Cell 3; displacement ends at Cell 3; unit does not continue to Cell 1. (BLOCKED until OQ-KS4 Trap design is resolved — annotate as BLOCKED with this note.) | BLOCKING |
| KW-052 | GIVEN caster at Cell 2, target at Cell 6, ATTRACT 5; Cell 4 has a lethal Trap; WHEN ATTRACT resolves, THEN the target traverses Cell 5 (no Trap), Cell 4 (Trap triggers — lethal); target dies at Cell 4; displacement ends at Cell 4; target does not continue to Cell 2. (BLOCKED until OQ-KS4 Trap design is resolved.) | BLOCKING |
| KW-053 | GIVEN BODYGUARD unit G (Lane 3) protects unit P (Lane 3); unit P executes CHANGE LANE from Lane 3 to Lane 2; WHEN P is in Lane 2, THEN P is still protected by G — an opponent Spell/Order targeting P is still blocked; G's `bodyguard_protects` field still references P's entity ID. The bond survived the lane change because it is stored as a unit-to-unit entity reference, not a lane-scoped attribute. | BLOCKING |
| KW-054 | GIVEN Player A has 2 units on board and Player B has 4 units on board at SS3 entry (Player A is OUTNUMBERED); Player A's FIRST STRIKE units kill 3 opponent units in SS3 (opponent count drops to 1); WHEN SS5 is evaluated, THEN OUTNUMBERED for Player A is false — `count(A)=2, count(B)=1, 2 < 1 = false`; the bonus is inactive in SS5 and SS6. | BLOCKING |
| KW-055 | GIVEN unit X has COUNTERATTACK granted via INJURED; X has ATK=3 and current_HP=2 (max_HP=4, so INJURED=true); in SS6, attacker A deals 1 damage to X (reducing HP to 1); WHEN SS6 resolves, THEN X's COUNTERATTACK fires against A for 3 damage (ATK post-modifier, using pre-retaliation snapshot); the INJURED-granted COUNTERATTACK was active because X was INJURED before SS6 began. | BLOCKING |
| KW-056 | GIVEN unit X has RANGE granted via INJURED; X's keyword array normally has no RANGE entry; WHEN INJURED is active at SS6 entry, THEN X attacks the nearest enemy within RANGE (from the `max_range` value specified in the INJURED-RANGE card definition) without advancing; X does not trigger COUNTERATTACK from the RANGE attack (COUNTERATTACK cannot be triggered by RANGE attackers per KW-005). | BLOCKING |
| KW-057 | GIVEN unit X gains SHIELD via INJURED at the SS3→SS4 boundary (was damaged in SS3), WHEN SS6 attacker A deals damage to X, THEN SHIELD (granted at the SS3→SS4 boundary) absorbs the SS6 attack — the granted SHIELD is active from SS4 onward; it is NOT retroactive to SS3. | BLOCKING |

### Additional ACs (new — R3 additions)

| # | Criterion | Type |
|---|---|---|
| KW-058 | GIVEN a STUNned unit receives melee damage in SS3 or SS6, WHEN the attack resolves, THEN COUNTERATTACK does NOT fire — STUN suppresses all keyword hooks including reactive triggers. | BLOCKING |
| KW-059 | GIVEN a unit with FIRST STRIKE is co-located with an enemy WALL at SS3 entry, WHEN SS3 resolves, THEN the FIRST STRIKE unit deals damage to the WALL in SS3; the WALL deals 0 damage in response. If the FIRST STRIKE damage reduces the WALL's HP to 0, the WALL is removed in SS4. | BLOCKING |
| KW-060 | GIVEN an enemy WALL was killed by FIRST STRIKE in SS3 and removed in SS4, WHEN SS5 resolves, THEN advancing friendly units are no longer blocked at the WALL's former cell — they pass through freely. | BLOCKING |
| KW-061 | GIVEN a RANGE unit has an enemy WALL as the nearest enemy within range, WHEN SS6 (or SS3 for FIRST STRIKE) resolves, THEN the RANGE unit attacks the WALL, not any unit beyond it. | BLOCKING |
| KW-062 | GIVEN a BODYGUARD unit enters when no other friendly unit is alive on the board, WHEN SS1 resolves, THEN `bodyguard_protects = None`; the BODYGUARD fights normally this round; no error state is created. | BLOCKING |
| KW-063 | GIVEN a LEADER unit is placed in SS1 of round R (with or without HASTE), WHEN the LEADER bonus snapshot is computed at SS1-end, THEN eligible family units receive the LEADER bonus in round R — the LEADER is present at snapshot time. | BLOCKING |
| KW-064 | GIVEN a unit with SHIELD has never been attacked in round R, WHEN round R+1's RESOLUTION begins, THEN the unit's SHIELD is still active — SHIELD has no round-expiry; it persists until consumed. | BLOCKING |
| KW-065 | GIVEN unit A's DEATH trigger deals damage to unit B in SS4; unit B's DEATH trigger would deal damage to unit A (which is already dead), WHEN the DEATH chain resolves, THEN the already-dead set prevents unit A from entering the chain a second time; unit B's DEATH fires once (against any valid live targets) without causing a loop. | BLOCKING |
| KW-066 | GIVEN an IRREMOVABLE unit triggers its own CHANGE LANE movement, WHEN CHANGE LANE resolves, THEN the unit successfully moves to the adjacent lane — IRREMOVABLE does not suppress own movement. | BLOCKING |
| KW-067 | GIVEN Player A unit at Cell 1 is REPELled 6 cells (maximum), WHEN `repel_destination` resolves, THEN `clamp(1 + (−1)×6, 1, 8) = clamp(−5, 1, 8) = 1`; unit does not move; zero cells are traversed (no Trap triggers). | BLOCKING |
| KW-068 | GIVEN a Player A unit with RANGE is placed with an ATTRACT X ≥ 4 targeting an enemy unit at Cell 5, caster at Cell 2, WHEN ATTRACT resolves, THEN enemy stops at Cell 3 (1 cell short of caster's Cell 2 — 1-cell-apart collision rule applied); the enemy is NOT on Cell 2. | BLOCKING |
| KW-069 | GIVEN a unit receives damage from an APPEARANCE trigger in SS1 (becoming INJURED at the SS1→SS2 boundary) and has a card-granted FIRST STRIKE when INJURED, WHEN SS3 resolves, THEN the INJURED-granted FIRST STRIKE IS active — INJURED was acquired at SS1→SS2 boundary, which is before SS3. | BLOCKING |

## Open Questions

| # | Question | Owner | Action Required |
|---|---|---|---|
| OQ-KS1 | Three distinct keyword RNG events need separate seed slots in the RESOLUTION chain (the original "one slot covers both" claim was incorrect — different sub-step ordering creates non-determinism). Required slots: (a) `range_equidistant_select` — fires in SS3 for RANGE+FIRST STRIKE attackers, SS6 for standard RANGE attackers, when multiple targets are equidistant; (b) `teleport_random_dest` — fires within whatever sub-step a TELEPORT card text triggers, when destination is randomised; (c) `strich_change_lane_select` — fires after the triggering sub-step when both adjacent lanes are valid. Inter-player ordering follows `server-rng.md` Rule 6 (ascending player_id, then ascending lane). | Server-side RNG + Network Protocol | Register all three slots in `server-rng.md` Rule 5 RESOLUTION chain with explicit `event_type` strings before any keyword implementation. Resolves OQ3 in `combat-resolution.md`. |
| OQ-KS2 | HASTE rename (from CHARGE): all Extension=1 cards with the CHARGE combat keyword must be audited and updated to `"Haste"` in `cards.json`. Schema field update required in `card-data-pool.md`. | Card Data & Pool + Game Designer | Audit before any card data encoding begins. |
| OQ-KS3 | COUNTERATTACK rule simplified in R3 (design decision D3): fires on any non-RANGE attack. No proximity condition. Update `combat-resolution.md` OQ4 to Resolved and update any COUNTERATTACK references to reflect simplified rule. | Combat Resolution GDD | Update `combat-resolution.md` before keyword implementation. |
| OQ-KS4 | ATTRACT and REPEL traversal triggers Traps on intermediate cells (cells strictly between the start cell and the final destination, exclusive of start). Non-lethal/non-STUN Trap damage does NOT terminate displacement; only kill or STUN do. The Trap GDD (part of `card-data-pool.md` OQ1 original designs) must specify that Traps fire on cell entry regardless of how the unit entered. | Trap design | Include in Trap card design spec when original Trap cards are authored. |
| OQ-KS5 | `combat-resolution.md` Visual section currently specifies an OUTNUMBERED indicator scoped per-lane (Crimson Slate arrow on the lane line). The OUTNUMBERED rule is a global board count, not per-lane. The indicator must move to per-unit (on each unit carrying the OUTNUMBERED keyword) reading from the global boolean. | Combat Resolution Visual + Board Rendering | Update `combat-resolution.md` Visual subsection before Board Rendering implementation. |
| ~~OQ-NP1~~ | ~~`S2CResolutionEvent` needs a `DisplacementEvent` variant.~~ **RESOLVED 2026-04-30.** `DisplacementEvent` is now fully defined in the NP GDD. Authoritative schema (note field renames vs. this GDD's original spec): `DisplacementEvent { unit_id: EntityId, attacker_id: Option<EntityId>, from_lane: u8, from_cell: u8, to_lane: u8, to_cell: u8, kind: DisplacementKind, block_reason: Option<DisplacementBlockReason>, sub_step: u8 }`. `kind` = renamed `keyword`; `block_reason: Option<DisplacementBlockReason>` = supersedes `was_blocked: bool`; `from_lane`/`to_lane` added. See NP GDD R3+ and downstream Interactions table updated. ✓ | — | Closed. |
| OQ-NP2 | ~~`GameOverReason` enum needs a `ResolutionTimeout` variant distinct from `Draw`.~~ **RESOLVED 2026-04-30.** `ResolutionTimeout` added to `GameOverReason` enum in `round-state-machine.md` Rule 14 (enum owner). RSM-38 now emits `reason=ResolutionTimeout` on 60s safety timeout. NP synced. Registry updated. | — | Closed |
| OQ-NP3 | DEATH chain link order is animation-load-bearing (combat-resolution.md requires sequential, non-overlapping death pulses for chained units), but `UnitDied { unit_id, lane, cell, killer_id }` does not encode chain position. Either (a) document `killer_id = Some(triggering_unit_id)` semantics for trigger kills, or (b) add `caused_by_death_trigger_of: Option<EntityId>` to the variant. | Network Protocol GDD | Resolve in `network-protocol.md` before Card Animations implementation. |
| ~~OQ-NP4~~ | ~~`S2CGameSnapshot.UnitBoardState` needs new fields.~~ **RESOLVED 2026-04-30 (NP R3+).** All fields are present in NP GDD `UnitBoardState`: `shield_active: bool`, `stunned_until_round: Option<u32>` (was `stun_active: bool` — renamed in NP R6), `silenced_until_round: Option<u32>` (was `Option<u8>` — corrected in NP R6), `leader_bonus_atk: u8`, `leader_bonus_hp: u8`, `bodyguard_protects: Option<EntityId>`, `haste_active: bool` (added NP R6). ✓ | — | Closed. |
| ~~OQ-NP5~~ | ~~`S2CResolutionEvent` needs a `KeywordTriggered` variant.~~ **RESOLVED 2026-04-30 (NP R3+).** `KeywordTriggered { source_unit_id: Option<EntityId>, sub_step: u8, payload: KeywordPayload }` is fully defined in NP GDD with 10 payload variants (ShieldConsumed, StunApplied, SilenceApplied, InjuredBonusActive, LeaderSnapshotTaken, OutnumberedFlipped, BodyguardBondCreated, BodyguardBondBroken, CounterattackFired, HasteActivated). Note: NP GDD uses `source_unit_id: Option<EntityId>` (not `unit_id`) — None for board-global events like OutnumberedFlipped. ✓ | — | Closed. |
| OQ-KS-new | **SILENCE structured duration field** — a structured `silence_duration_rounds: u8` field must be added to the `cards.json` schema and to `card-data-pool.md` before SILENCE can be implemented. Server computes `silenced_until_round = current_round + silence_duration_rounds - 1` (expiry-inclusive). **Priority: HIGH — blocks SILENCE implementation.** Server startup validation must reject `silence_duration_rounds == 0` or `> 1` (multi-round SILENCE is not designed for current scope). | Card Data & Pool GDD | Add `silence_duration_rounds: u8` to `cards.json` keyword schema in `card-data-pool.md`. Update this OQ once added. |
| OQ-KS6 | **STUN suppresses all keyword hooks including reactive triggers** — RESOLVED in R3 (design decision D4). COUNTERATTACK does NOT fire when the unit is STUNned. Add this ruling to `combat-resolution.md` STUN definition and any cross-references. | Combat Resolution GDD | Update `combat-resolution.md` STUN section before keyword implementation. |
| OQ-KS7 | **`SilenceApplied` event payload must include stripped-keywords list** — the UI requires "all stripped keyword glyphs dissolve simultaneously." `KeywordPayload::SilenceApplied { duration_rounds: u8 }` cannot fulfill this — the client needs to know which keywords were stripped (including runtime INJURED-granted keywords not in `cards.json`). Proposed addition: `SilenceApplied { duration_rounds: u8, stripped_keywords: Vec<StrippedKeyword> }`. Must be resolved in `network-protocol.md` before SILENCE implementation. **Priority: HIGH.** | Network Protocol GDD | Add `stripped_keywords` field to `SilenceApplied` payload in NP GDD before SILENCE implementation. |
| OQ-KS8 | **`CounterattackFired` event must include `target_id`** — for multi-attacker COUNTERATTACK (multiple simultaneous non-RANGE attackers), multiple `CounterattackFired` events fire. Without a `target_id: EntityId` field, the animation system cannot pair each retaliation with its target without fragile positional ordering. Must be resolved in `network-protocol.md` before COUNTERATTACK implementation. | Network Protocol GDD | Add `target_id: EntityId` to `CounterattackFired` payload in NP GDD. |
| OQ-KS9 | **LEADER snapshot timing change must propagate to Combat Resolution GDD** — R3 changed LEADER snapshot from "RESOLUTION entry" to "after SS1 completes (post-SS1, pre-SS2)." `combat-resolution.md` must be updated to reflect this timing in the combat resolution pass sequence. | Combat Resolution GDD | Update `combat-resolution.md` LEADER snapshot section before keyword implementation. |
| OQ-KS10 | **`board-lane-system.md` must formally define the 1-cell-apart collision rule** — "opposing units from different players can never occupy the same cell; when they make contact they stop 1 cell apart." This rule is referenced in the ATTRACT formula (enemy target cap), COUNTERATTACK (simplified rule basis), and WALL behavior. The authoritative definition must live in `board-lane-system.md` and be cross-referenced here. | Board/Lane System GDD | Add explicit "opposing unit collision model" rule to `board-lane-system.md` before movement keyword implementation. |
