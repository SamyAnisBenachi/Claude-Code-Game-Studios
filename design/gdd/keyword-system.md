# Keyword System

> **Status**: Designed (all sections complete — /design-review pending in fresh session)
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-30
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
| COUNTERATTACK | SS3 or SS6 | Same-cell OR collision-halted adjacent-cell contact; never RANGE |
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

**COUNTERATTACK** — fires immediately when this unit receives damage in SS3 or SS6. Proximity requirement: the attacker must be on the same cell OR halted on an adjacent cell from a sub-step 5 collision. RANGE attackers that did not advance to the target's cell cannot trigger COUNTERATTACK. If a unit receives damage from multiple sources in the same sub-step, COUNTERATTACK fires once after all damage is applied.

**INJURED** — a unit is INJURED when `current_HP < max_HP`. INJURED is a persistent state re-evaluated at each sub-step boundary. A unit damaged in SS3 is INJURED from SS4 onward for that RESOLUTION. SILENCE strips INJURED-granted keywords (e.g., FIRST STRIKE); INJURED itself is not a keyword and cannot be silenced.

**START OF TURN** — fires at DRAFT phase entry, after mana ramp and gold income are applied (RSM Rule 3). Cards placed this round first trigger START OF TURN on round R+1.

**END OF TURN** — fires after RESOLUTION sub-step 6 completes, before the RSM round counter increments. Cards alive at this point can trigger it even if they entered play this round.

---

#### Combat Keyword Catalog

**FIRST STRIKE** — attacks in sub-step 3 before standard combat. Kills before retaliation. Two FIRST STRIKE units facing each other deal damage simultaneously (no priority). FIRST STRIKE does not advance objective damage to SS3.

**HASTE** *(renamed from CHARGE)* — unit can act (move, attack) the round it enters play. Without HASTE: SS1 entry only, no SS2/SS3/SS5/SS6 participation this round. STUN applied in SS1 overrides HASTE.

**RANGE 1-X** — attacks the nearest enemy in the forward direction within X cells; does not advance. Equidistant targets: server selects randomly (OQ-KS1 — needs RESOLUTION RNG seed slot in server-rng.md). RANGE + FIRST STRIKE: attacks in SS3 AND SS6. RANGE attacks bypass BODYGUARD. COUNTERATTACK cannot be triggered by RANGE attackers.

**WALL** — stationary (MP=0, cannot self-move). Advancing enemies stop at WALL's cell and fight it in SS6 (WALL deals 0 damage). Not IRREMOVABLE by default — can be displaced by REPEL/ATTRACT/TELEPORT.

**BODYGUARD** — on entry (SS1): controller chooses one other friendly unit on the board. That unit cannot be targeted by opponent Spells or Orders while this BODYGUARD is alive. RANGE attacks bypass BODYGUARD. Objective damage from movement is unaffected. Protection ends when BODYGUARD dies.

**IRREMOVABLE** — cannot be displaced by REPEL, ATTRACT, TELEPORT, Spells, or Orders. Does not affect the unit's own movement (MP, CHARGE X, CHANGE LANE).

**UNTARGETABLE** — cannot be named as the target of opponent Spells or Orders. Does not prevent standard combat or RANGE attacks. Friendly non-targeted AoE effects may still apply.

**RESISTANCE X** — incoming ATK reduced by X before the AR step: `ATK_effective = max(0, ATK_raw − X)`. Not bypassed by ARMOR-PIERCING (which only affects AR, not RESISTANCE).

**VULNERABILITY X** — incoming ATK increased by X before the AR step: `ATK_effective = ATK_raw + X`.

**SILENCE** — strips all keywords and keyword-granted effects for the duration. Affects: FIRST STRIKE, HASTE, CHARGE X, RANGE, WALL movement-lock, BODYGUARD protection, UNTARGETABLE, RESISTANCE, VULNERABILITY, ARMOR-PIERCING, SHIELD, LEADER bonus grant, OUTNUMBERED condition, and all trigger hooks (DEATH/FINAL BLOW/COUNTERATTACK/INJURED bonuses). INJURED state cannot be silenced.

**STUN** — unit cannot act this round: SS2 (CHARGE X), SS3 (FIRST STRIKE), SS5 (standard movement), SS6 (attacks) are all suppressed. Unit remains on board and takes damage normally. Lasts current RESOLUTION only.

**ARMOR-PIERCING** — attacker treats defender's AR as 0 for outgoing damage. RESISTANCE on the defender is applied independently (before AR step) and is unaffected. Attacker's own AR (including RPS bonus) is unaffected.

**SHIELD** — absorbs all incoming damage from one sub-step (SS3 or SS6). Sub-step scoped: consumed once; does not protect in a different sub-step. Persists across rounds until triggered. Two simultaneous attackers in the same sub-step are both absorbed by one SHIELD consumption.

**LEADER** — grants a stat bonus to all friendly units of the same family (bonus type and value per card). Snapshotted at RESOLUTION entry; persists even if LEADER dies in SS4. Recalculated fresh each round. A SILENCEd LEADER does not grant its bonus. The LEADER counts as a family member for "X [family] in play" effects.

**OUTNUMBERED** — this unit's OUTNUMBERED bonus is active when the controlling player has fewer units on the board (all lanes, Minions + Structures) than the opponent. Evaluated at the start of each sub-step. Can activate or deactivate mid-RESOLUTION as deaths change counts.

---

#### Movement Keyword Catalog

**CHARGE X** — advances X extra cells during sub-step 2, before FIRST STRIKE. Applies the same WALL-blocking and collision rules as sub-step 5. Suppressed by STUN. Independent of HASTE.

**REPEL X** — pushes target X cells toward its own side. Direction: toward Cell 1 (Player A units) or Cell 8 (Player B units). Formula: `new_cell = clamp(target_cell + (−advance_dir(target.owner)) × X, 1, 8)`. Triggered effect — fires within whatever sub-step activates it. IRREMOVABLE: no effect. WALL can be pushed. Trap cells trigger on entry during displacement. Pushing a unit to Cell 8 (for Player B) places it at its own spawn — no objective damage.

**ATTRACT X** — pulls target toward caster's cell; stops at caster's cell if X would carry it further. Formula: `effective_X = min(X, |caster_cell − target_cell|)`. Lane-local. Can target friendly or enemy per card text. IRREMOVABLE: no effect. Trap triggers on traversed cells. Pulling an enemy to your own objective cell is valid per card text — that unit will deal objective damage to your objective at SS6.

**TELEPORT** — repositions a unit to a specified cell. Destination, target filter, and range restriction are per-card (no keyword-level range cap). Does NOT trigger APPEARANCE or COUNTERATTACK. Cannot return a unit to hand (valid destination: cells 1–8). IRREMOVABLE: no effect. Co-occupation is allowed. Spawn-range restrictions do not apply (PLACEMENT rule only). Cross-lane TELEPORT only if card text specifies a different lane.

**CHANGE LANE** — moves this unit to an adjacent lane at the same cell position. Executes after the triggering sub-step, before the next begins. Rejected silently if the destination lane already holds a friendly Minion (1-Minion-per-lane slot rule). *Strich:* automatically triggers CHANGE LANE when an enemy unit enters play in Strich's current lane; server selects randomly if both adjacent lanes are valid.

---

### States and Transitions

| State | Source | Active from | Expires when |
|---|---|---|---|
| INJURED | HP drops below max_HP | Sub-step boundary after damage | HP restored to max_HP or unit dies |
| SHIELD | Unit enters with SHIELD | Board entry | Absorbs an attack (consumed in SS3 or SS6) |
| LEADER bonus | LEADER alive at RESOLUTION entry | RESOLUTION entry (snapshotted) | RESOLUTION ends; recalculated next round |
| OUTNUMBERED | Board count check | Start of each sub-step | Counts equalize or favor controller |
| STUN | Source effect | Immediately on application | End of current RESOLUTION |
| SILENCE | Source effect | Immediately on application | Duration per card (typically one RESOLUTION) |
| BODYGUARD protection | BODYGUARD enters play | SS1 | BODYGUARD unit leaves board |

---

### Interactions with Other Systems

| System | Data In | Data Out | Interface Contract |
|---|---|---|---|
| Card Data & Pool | Keyword array per card, parameterized values (RANGE max_range, RESISTANCE value, etc.) | — | This GDD is the authoritative spec for what each keyword declaration in `cards.json` means. |
| Combat Resolution | Sub-step structure, modifier stack (steps 1–10), trigger timing | Keyword effect execution within sub-steps; trigger chain rules | Combat Resolution owns execution timing; this GDD owns what each keyword does within that timing. Keywords must not contradict CR sub-step assignments. |
| Round State Machine | DRAFT phase entry, RESOLUTION_COMPLETE | START/END OF TURN effects per card | RSM fires phase events; Keyword System specifies what START/END OF TURN cards do. |
| Board/Lane System | Cell positions, lane layout, CHANGE LANE slot validity, collision model | REPEL/ATTRACT/TELEPORT/CHANGE LANE displacement results | Movement keywords use the F1 formula and collision rules from board-lane-system.md. |
| Server-side RNG | RESOLUTION RNG chain | RANGE equidistant selection + TELEPORT random-destination | OQ-KS1: one seed slot covers both; must be registered in server-rng.md before RANGE/TELEPORT implementation. |
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
| Effective pull | `effective_pull` | u8 | 0–6 | Capped so target never passes caster's cell |
| Direction sign | `sign(...)` | i8 | −1, 0, or +1 | Direction from target toward caster; 0 if already co-located |
| Output | `attract_destination` | u8 | 1–8 | Destination cell after pull |

**Output Range:** [1, 8]. `effective_pull = 0` if caster and target already share a cell (no movement). Destination is always between `target_cell` and `caster_cell` inclusive — the target cannot overshoot the caster.

**Example:** Caster at Cell 5, target (Player B unit) at Cell 7, ATTRACT 4. `effective_pull = min(4, |5 − 7|) = min(4, 2) = 2`. `attract_destination = 7 + sign(5 − 7) × 2 = 7 + (−1) × 2 = 5`. Target lands at Cell 5, co-located with caster.

---

### Formula 3: OUTNUMBERED Board Count

The `outnumbered` condition is defined as:

`outnumbered(player) = count(alive_units(player)) < count(alive_units(opponent))`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Player unit count | `count(alive_units(player))` | u8 | 0–10 | All alive Minions + Structures owned by this player, across all lanes. Traps excluded (face-down; not participating in board count). |
| Opponent unit count | `count(alive_units(opponent))` | u8 | 0–10 | Same, for opponent. |
| Output | `outnumbered` | bool | false/true | True only when strictly fewer; equal counts = false. |

**Output Range:** `false` (not outnumbered) or `true` (outnumbered). Evaluated at the start of each sub-step — can flip mid-RESOLUTION as units die.

**Example:** Player has 2 Minions on board; opponent has 4 Minions + 1 Structure = 5 units. `2 < 5 = true`. OUTNUMBERED bonus is active.

---

### Formula 4: Damage Modifier Reference (owned by combat-resolution.md)

RESISTANCE X, VULNERABILITY X, and ARMOR-PIERCING are evaluated within the `net_damage` formula (owned by `combat-resolution.md`, registered in `design/registry/entities.yaml`). They are not re-defined here; the full modifier stack is in `combat-resolution.md` Detailed Design — Combat Modifier Stack, steps 1–10.

For reference: `RESISTANCE X` reduces incoming `ATK_raw` by X (floor 0) before the AR step. `VULNERABILITY X` increases incoming `ATK_raw` by X before the AR step. `ARMOR-PIERCING` sets `AR_effective = 0` after RESISTANCE is applied.

## Edge Cases

**If SILENCE is applied via APPEARANCE in SS1:** The silenced unit's DEATH trigger (and all other trigger hooks) are stripped for the duration. When that unit later dies in SS4, DEATH does not fire. SILENCE is not sub-step scoped for stripping — once applied, the loss persists for the SILENCE duration. If the SILENCEd unit had FINAL BLOW, FINAL BLOW also does not fire when it kills an enemy.

**If a DEATH trigger chain kills multiple units in sequence:** No hard chain depth cap. Termination is structural: a unit can only die once, and with at most 10 units on the board, the chain terminates within 9 links. The server tracks an "already-dead" set during SS4; a unit already queued or removed cannot die again. RSM 60-second RESOLUTION safety timeout is the backstop. FINAL BLOW does NOT apply to kills caused by DEATH trigger chains — FINAL BLOW requires an attack in SS3 or SS6.

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
| Network Protocol | `network-protocol.md` | Needs `DisplacementEvent { unit_id, keyword: DisplacementKind, from_cell, to_cell, sub_step }` variant in `S2CResolutionEvent` (OQ-NP1) | Soft — additive |

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
| REPEL X / ATTRACT X distance | Per-card `cards.json` | 1–3 | 1–6 | Above 4: can displace unit across an entire player side in one effect |
| SILENCE duration | Per-card `effect_text` | 1 RESOLUTION | 1 RESOLUTION | Multi-round SILENCE is not designed; keep to 1 RESOLUTION unless explicitly playtested |
| STUN duration | Hardcoded rule | 1 RESOLUTION | 1 RESOLUTION | Multi-round STUN is too punishing; do not increase without playtesting |

**Future GameConfig candidates:** If COUNTERATTACK proximity rule ever needs to be tuned (`same-cell only` vs. `same-cell or collision-halted adjacent`), add `counterattack_requires_same_cell: bool` to `game_config.ron`. Currently hardcoded to `false`. If OUTNUMBERED threshold needs adjustment from strict `<` to `≤`, add `outnumbered_threshold_mode` similarly.

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

No interactive elements. No modal dialogs. Keyword tooltips are not specified — keywords are intended to be self-describing from their card text.

## Acceptance Criteria

### Timing Triggers

| # | Criterion | Type |
|---|---|---|
| KW-001 | GIVEN a unit with an APPEARANCE trigger enters the board in SS1, WHEN sub-step 1 resolves, THEN the APPEARANCE effect executes before any DEATH trigger chains that result from APPEARANCE-caused kills. | BLOCKING |
| KW-002 | GIVEN two units in different lanes are killed in the same sub-step, WHEN sub-step 4 removes dead units, THEN DEATH triggers fire in lane order (Lane 1 before Lane 5); Lane 2 unit's DEATH trigger resolves completely before Lane 4 unit's begins. | BLOCKING |
| KW-003 | GIVEN unit A has a DEATH trigger that deals lethal damage to unit B (also has a DEATH trigger), WHEN unit A is removed in SS4, THEN A's DEATH trigger resolves completely, then B is removed, then B's DEATH trigger fires. | BLOCKING |
| KW-004 | GIVEN a unit with FINAL BLOW is killed by a FIRST STRIKE attacker in SS3, WHEN the killing blow reduces HP to 0, THEN FINAL BLOW fires in SS3 (not SS4); a unit killed in SS6 by standard combat triggers FINAL BLOW in SS6. | BLOCKING |
| KW-005 | GIVEN a COUNTERATTACK unit receives damage from a RANGE attacker that did not advance to the COUNTERATTACK unit's cell, WHEN the RANGE attack resolves, THEN COUNTERATTACK does NOT fire. | BLOCKING |
| KW-006 | GIVEN a COUNTERATTACK unit is halted at an adjacent cell from a sub-step 5 collision, WHEN the two units exchange melee damage in SS6, THEN COUNTERATTACK fires. | BLOCKING |
| KW-007 | GIVEN a unit with full HP receives damage in SS3 (reducing HP below max_HP), WHEN SS3 resolves, THEN INJURED-granted keywords are NOT active during SS3; INJURED is active from SS4 onward in the same RESOLUTION. | BLOCKING |
| KW-008 | GIVEN a unit is INJURED and subsequently SILENCEd, WHEN SILENCE applies, THEN INJURED-granted keywords are stripped, but the INJURED state itself persists. | BLOCKING |
| KW-009 | GIVEN a unit with START OF TURN is alive at the start of round R+1, WHEN the DRAFT phase begins after mana ramp + gold income, THEN START OF TURN fires; a unit that entered play on round R does NOT trigger START OF TURN on round R. | BLOCKING |
| KW-010 | GIVEN a unit with END OF TURN is alive at the end of RESOLUTION, WHEN SS6 completes, THEN END OF TURN fires before the RSM round counter increments; a unit that entered play this round and survived can trigger END OF TURN. | BLOCKING |

### Combat Keywords

| # | Criterion | Type |
|---|---|---|
| KW-011 | GIVEN a FIRST STRIKE unit is in the same cell as a standard enemy unit, WHEN SS3 resolves, THEN the FIRST STRIKE unit deals damage in SS3; the enemy does NOT deal damage in SS3; if the enemy survives, it attacks in SS6. | BLOCKING |
| KW-012 | GIVEN two FIRST STRIKE units are co-located, WHEN SS3 resolves, THEN both deal damage simultaneously using pre-combat HP snapshots; neither's damage is computed after seeing the other's result. | BLOCKING |
| KW-013 | GIVEN a unit with HASTE is placed in SS1, WHEN RESOLUTION proceeds, THEN the unit advances in SS5, attacks in SS6 (and SS3 if FIRST STRIKE), executes CHARGE X in SS2 if present — all in the same round it entered play. | BLOCKING |
| KW-014 | GIVEN a unit with HASTE has STUN applied via an SS1 APPEARANCE trigger, WHEN RESOLUTION proceeds, THEN the unit skips SS2, SS3, SS5, and SS6; HASTE does not override STUN. | BLOCKING |
| KW-015 | GIVEN a STUNned unit is in the path of an enemy attack, WHEN SS3 or SS6 resolves, THEN the STUNned unit takes incoming damage normally; it does not attack, move, or advance; STUN expires at end of current RESOLUTION. | BLOCKING |
| KW-016 | GIVEN a RANGE 1-X unit has an enemy BODYGUARD protecting another unit within range, WHEN the RANGE unit selects its target, THEN RANGE selects by proximity (nearest cell); BODYGUARD's Spell/Order protection does not intercept RANGE targeting. | BLOCKING |
| KW-017 | GIVEN a unit with RANGE and FIRST STRIKE, WHEN RESOLUTION executes, THEN the unit attacks in SS3 AND again in SS6; SHIELD consumed in SS3 does NOT protect the same unit in SS6. | BLOCKING |
| KW-018 | GIVEN a WALL unit is at Cell 4 and an enemy unit has MP sufficient to reach or pass Cell 4, WHEN SS5 movement resolves, THEN the enemy unit stops at Cell 4 and fights WALL in SS6; WALL deals 0 damage. | BLOCKING |
| KW-019 | GIVEN unit B is protected by BODYGUARD unit G; G is killed in SS4, WHEN G's removal is processed, THEN unit B is no longer protected from opponent Spells and Orders immediately. | BLOCKING |
| KW-020 | GIVEN an IRREMOVABLE unit is the target of REPEL X, ATTRACT X, or TELEPORT, WHEN the displacement effect is applied, THEN the IRREMOVABLE unit does not change position; IRREMOVABLE does not prevent the unit's own movement. | BLOCKING |
| KW-021 | GIVEN an UNTARGETABLE unit is in combat range of an enemy RANGE unit, WHEN SS6 resolves, THEN the RANGE attack hits the UNTARGETABLE unit normally; UNTARGETABLE only blocks Spell/Order targeting. | BLOCKING |
| KW-022 | GIVEN a defender with RESISTANCE 2 is attacked by a unit with ARMOR-PIERCING, WHEN the modifier stack resolves, THEN RESISTANCE 2 reduces ATK_effective by 2 first; ARMOR-PIERCING sets AR_defender to 0 independently; RESISTANCE is not bypassed by ARMOR-PIERCING. | BLOCKING |
| KW-023 | GIVEN a SILENCEd unit has COUNTERATTACK, DEATH trigger, FIRST STRIKE, and is INJURED, WHEN SILENCE applies, THEN all keyword hooks are stripped; the unit does not counterattack or trigger on death; INJURED state persists. | BLOCKING |
| KW-024 | GIVEN a SHIELD unit is attacked by two enemies simultaneously in SS6 and by a RANGE+FIRST STRIKE attacker in SS3, WHEN both sub-steps resolve, THEN SHIELD absorbs all SS6 damage and is consumed; the SS3 attack is unaffected if it is in a different sub-step. | BLOCKING |
| KW-025 | GIVEN a LEADER unit is alive at RESOLUTION entry (bonus snapshotted) and is killed in SS4, WHEN SS6 resolves, THEN the ATK bonus remains active for all eligible family units in SS6. | BLOCKING |
| KW-026 | GIVEN a LEADER unit is SILENCEd at RESOLUTION entry, WHEN the bonus snapshot is computed, THEN the SILENCEd LEADER grants no bonus to family units for this RESOLUTION. | BLOCKING |
| KW-027 | GIVEN Player A has 3 units and Player B has 3 units at SS2 entry, WHEN OUTNUMBERED is evaluated, THEN 3 < 3 = false — bonus is NOT active. GIVEN Player A has 2 and Player B has 4, THEN 2 < 4 = true — bonus IS active. | BLOCKING |

### Movement Keywords

| # | Criterion | Type |
|---|---|---|
| KW-028 | GIVEN a unit with CHARGE 2 is in a lane where an enemy WALL is 1 cell ahead, WHEN SS2 resolves, THEN the unit is blocked at the WALL's cell and does not pass through it. | BLOCKING |
| KW-029 | GIVEN (A) Player A unit at Cell 2 REPELled 3 cells: result = 1; (B) IRREMOVABLE unit REPELled: no displacement; (C) WALL unit REPELled: WALL moves to repel_destination, WHEN REPEL resolves for each, THEN each scenario produces its expected outcome. | BLOCKING |
| KW-030 | GIVEN caster at Cell 3, target at Cell 7, ATTRACT 6, WHEN ATTRACT resolves, THEN effective_pull = min(6, 4) = 4; target lands at Cell 3 (co-located with caster, NOT past Cell 3). | BLOCKING |
| KW-031 | GIVEN a unit is TELEPORTed to a cell occupied by an enemy unit, WHEN TELEPORT resolves, THEN no APPEARANCE trigger fires; no COUNTERATTACK fires; IRREMOVABLE blocks the teleport entirely. | BLOCKING |
| KW-032 | GIVEN a unit attempts CHANGE LANE to an adjacent lane that already has a friendly Minion, WHEN CHANGE LANE resolves, THEN the lane change does not execute; the unit remains in its current lane; no error state is created. | BLOCKING |
| KW-033 | GIVEN Strich is in Lane 3 and an enemy enters Lane 3 in SS1, WHEN SS1 completes, THEN Strich auto-triggers CHANGE LANE; server selects randomly if both adjacent lanes are valid; if neither is valid, CHANGE LANE is rejected silently. | BLOCKING |

### Cross-Keyword Interactions

| # | Criterion | Type |
|---|---|---|
| KW-034 | GIVEN a HASTE unit has STUN applied in SS1, WHEN RESOLUTION proceeds, THEN the STUNned HASTE unit skips SS2, SS3, SS5, and SS6; HASTE does not partially override STUN. | BLOCKING |
| KW-035 | GIVEN unit A (FIRST STRIKE) kills unit B (has DEATH trigger and FINAL BLOW) in SS3, THEN FINAL BLOW fires in SS3; DEATH trigger fires in SS4; kill gold is awarded at SS4. | BLOCKING |
| KW-036 | GIVEN a WALL unit is SILENCEd, WHEN SS5 resolves, THEN the SILENCEd WALL loses its blocking behavior; advancing enemies no longer halt at its cell; unit still has MP=0 and does not self-move. | ADVISORY |
| KW-037 | GIVEN a unit has SHIELD and is attacked by a RANGE+FIRST STRIKE unit in SS3 (consuming SHIELD), WHEN SS6 executes the second attack, THEN the SS6 attack deals full damage — SHIELD consumed in SS3 does not protect in SS6. | ADVISORY |
| KW-038 | GIVEN unit X is BODYGUARD-protected and an enemy RANGE unit's proximity selection identifies X as the nearest enemy, WHEN the RANGE attack resolves, THEN BODYGUARD does not intercept; X can be hit by RANGE regardless of BODYGUARD. | ADVISORY |
| KW-039 | GIVEN a LEADER is unsILENCEd at RESOLUTION entry (bonus snapshotted), then SILENCEd during SS3, WHEN SS6 resolves, THEN the snapshot bonus remains active; mid-RESOLUTION SILENCE does not retroactively invalidate a legally-taken snapshot. | ADVISORY |
| KW-040 | GIVEN a DEATH trigger chain changes board counts mid-chain in SS4, WHEN OUTNUMBERED is evaluated for SS5, THEN the count reflects the full board state after all SS4 deaths resolve — not any intermediate count during the chain. | ADVISORY |
| KW-041 | GIVEN Player A uses ATTRACT to pull a Player B unit to Player A's Cell 1, WHEN SS6 objective damage resolves, THEN the Player B unit deals its ATK as damage to Player A's objective — ATTRACT does not grant immunity from backfire positioning. | ADVISORY |

## Open Questions

| # | Question | Owner | Action Required |
|---|---|---|---|
| OQ-KS1 | RANGE equidistant target selection and TELEPORT random destination both require server-side RNG. No seed slot exists in the RESOLUTION RNG chain for either use case. One seed slot should cover both. | Server-side RNG + Network Protocol | Add seed slot to `server-rng.md` seed table before RANGE or TELEPORT implementation. Resolves OQ3 in `combat-resolution.md`. |
| OQ-KS2 | HASTE rename (from CHARGE): all Extension=1 cards with the CHARGE combat keyword must be audited and updated to `"Haste"` in `cards.json`. Schema field update required in `card-data-pool.md`. | Card Data & Pool + Game Designer | Audit before any card data encoding begins. |
| OQ-KS3 | OQ4 in `combat-resolution.md` (COUNTERATTACK proximity) is now resolved: fires for both same-cell AND collision-halted adjacent contact. | Combat Resolution GDD | Update `combat-resolution.md` OQ4 status to Resolved. |
| OQ-KS4 | ATTRACT and REPEL traversal triggers Traps on intermediate cells. The Trap GDD (part of `card-data-pool.md` OQ1 original designs) must specify that Traps fire on cell entry regardless of how the unit entered. | Trap design | Include in Trap card design spec when original Trap cards are authored. |
| OQ-NP1 | `S2CResolutionEvent` needs a `DisplacementEvent { unit_id, keyword: DisplacementKind, from_cell, to_cell, sub_step }` variant for REPEL/ATTRACT/TELEPORT animation on client. Currently unregistered. | Network Protocol GDD | Add variant to `S2CResolutionEvent` enum in `network-protocol.md` before Board Rendering implementation. |
