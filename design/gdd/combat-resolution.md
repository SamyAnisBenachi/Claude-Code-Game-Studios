# Combat Resolution

> **Status**: In Design (8 required sections complete — Visual/Audio, Open Questions pending)
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Simple surface · Deep emergence · No idle spectating

## Overview

Combat Resolution is the phase where both players' secretly-placed cards are revealed and processed simultaneously. During the 10-second PLACEMENT window, each player selects which cards to play without knowing the opponent's choices; at RESOLUTION, all selections are exposed at once and a deterministic 6-step algorithm executes every effect across all five lanes in parallel. The six steps — applying placement effects, CHARGE bonus movement, FIRST STRIKE attacks, dead-unit removal, standard movement, and standard combat — are global passes: each step completes across all lanes before the next begins, ensuring outcomes depend entirely on unit positions and abilities, not server processing order. At the end of RESOLUTION, any unit occupying the opponent's far edge deals damage directly to the objective in that lane. For the player, RESOLUTION is the dramatic payoff of every round — the moment all economic and positional bets are settled simultaneously, where a correctly-read lane wins a combat and an unanticipated unit reaching the objective shifts the entire game.

## Player Fantasy

**Combat Resolution is the information payoff of every round.**

The player should feel like a tactician whose patience just paid off — not because they won the exchange, but because they *learned something*. RESOLUTION is when the lying stops. For ten seconds during PLACEMENT, both players committed to their bets in secret; RESOLUTION forces the board to tell the truth. Five lanes light up simultaneously and the player watches them all — not because they can change anything, but because they are reading.

**The specific moment:** The instant both placements appear simultaneously on screen. The player scans: *They sent nothing to lane 3. They doubled down on lane 4 with a FIRST STRIKE unit. Lane 5 just took my objective damage again.* That scan takes three seconds and yields enough information to restructure next round's plan entirely.

**What the player should feel:**
- **Active, not passive** — watching RESOLUTION is not downtime. The player is continuously updating their mental model of the opponent's deck, priorities, and which objectives might be real.
- **Narrative completion** — the 10-second PLACEMENT window was a question ("what are they doing?"); RESOLUTION answers it. Win or lose, the loop closes satisfyingly.
- **Strategic continuity** — by the time the gold ticks up at round end, the player has already started next round's placement in their head. The board does not pause. Neither does the player.

**What to avoid:** RESOLUTION must never feel like passive watching. The animation playback and the information reveal should arrive together — the player should be reading outcomes as they happen, not watching a cutscene. If a player has "nothing to do" during RESOLUTION, the UI has failed: they should be tracking unit positions, watching objective HP, noting which of the opponent's lanes stayed empty, and recalculating their economic position.

*Pillar alignment: "No idle spectating" — watching IS playing. "Deep emergence" — the strategic depth of hidden information becomes legible during this window.*

## Detailed Design

### Core Rules

**Resolution entry.** The Round State Machine enters RESOLUTION after all placement submissions are received (or the placement timer expires). The PlacementBuffer is committed atomically; all cards in it take effect starting at sub-step 1. The server broadcasts `S2CPlacementReveal` immediately before sub-step 1 executes, revealing both players' placements simultaneously to clients.

**Global pass structure.** All six sub-steps are global passes — each sub-step completes across all five lanes before the next begins. Lane order (Lane 1 → Lane 5) is the tiebreaker within a sub-step only (used to determine sequential damage order when multiple sources hit a single target in the same sub-step).

---

**Sub-step 1 — Apply Placement Effects (global)**

All played cards enter the board simultaneously across all lanes. Placement effects:
- Units enter at their controller's specified cell (within their valid spawn range at time of submission)
- APPEARANCE triggers fire immediately when a unit enters the board
- If an APPEARANCE trigger kills a unit, that unit's DEATH trigger fires after **all** APPEARANCE effects from sub-step 1 have resolved (sequential chain — not simultaneous)
- Spell, Trap, and Field effects specified in the PlacementBuffer execute in this sub-step
- Cross-lane triggers (CHANGE LANE, Strich auto-switch) that fire during sub-step 1 execute **after** all sub-step 1 effects complete and **before** sub-step 2 begins

**CHARGE keyword and sub-step 1:** A unit with the combat keyword CHARGE (can act this round, no summoning sickness) enters play in sub-step 1 and participates normally in all subsequent sub-steps of this round. CHARGE is not a sub-step 1 action — it is a flag that removes the "cannot act in round of placement" restriction.

---

**Sub-step 2 — CHARGE X Bonus Movements (global)**

All units with the movement keyword CHARGE X advance an additional X cells simultaneously, applying the same collision rules as sub-step 5. This sub-step affects ONLY units with a numeric CHARGE X value. The combat keyword CHARGE ("can act this round") has no effect in this sub-step.

---

**Sub-step 3 — FIRST STRIKE Attacks (global)**

All units with the FIRST STRIKE keyword deal damage simultaneously across all lanes:
- **RANGE + FIRST STRIKE:** A unit with both keywords also attacks in this sub-step (in addition to sub-step 6). It targets the nearest enemy unit within its forward RANGE. If multiple targets are equidistant, the server resolves randomly.
- **Damage application:** When multiple sources hit the same unit in sub-step 3, damage is applied sequentially in lane order (Lane 1 first, Lane 5 last). Each source is resolved separately — HP is updated between each source.
- **COUNTERATTACK:** If a unit with COUNTERATTACK receives damage in sub-step 3, its COUNTERATTACK effect fires immediately in sub-step 3 (before sub-step 4 begins). COUNTERATTACK is only triggered by physical proximity. A RANGE attacker that does not occupy the target's cell **cannot** be counter-attacked.
- **STUN:** A STUNned unit does not attack in sub-step 3 and does not move in sub-step 5. It is completely frozen for the round.
- Dead units (HP reduced to 0) from sub-step 3 damage are NOT removed until sub-step 4. A unit killed in sub-step 3 can still deal FIRST STRIKE damage in the same sub-step, and may trigger COUNTERATTACK.

---

**Sub-step 4 — Remove Dead Units (global)**

All units at 0 HP across all lanes are removed from the board:
- DEATH triggers fire for each removed unit (in lane order if multiple die simultaneously)
- DEATH trigger chains are sequential: if A's DEATH trigger kills B, B's DEATH trigger fires after A's completes
- FINAL BLOW fires in the sub-step where the kill occurred (sub-step 3 for FIRST STRIKE kills; sub-step 6 for standard combat kills) — not consolidated to sub-step 4
- **Kill gold:** +1 gold awarded immediately to the controlling player of the unit whose attack dealt the final damage. Objective destruction does NOT award +1 kill gold (objectives are not units).

---

**Sub-step 5 — Standard Movement (global)**

All non-STUNned units advance toward the opponent's side by their MP value. Movement is resolved step-by-step (1 cell per tick), simultaneously across all lanes:

1. Each tick: all units attempt to advance 1 cell toward the opponent's side.
2. **WALL exception:** A unit stops the moment its next step would bring it to a cell occupied by an enemy WALL unit. The advancing unit fights the WALL in sub-step 6 (WALL has 0 ATK, so it takes damage but deals none back).
3. **Collision:** If two enemy units advancing toward each other would both land on the same cell, or would swap positions (cross through each other's cell), both halt at their current cells. Units halted by collision or crossing fight each other in sub-step 6.
4. Movement continues tick by tick until all units have reached their destinations or been halted.

> **Design note:** This step-by-step collision model is a deliberate deviation from the Board/Lane System GDD's "skip intermediate cells" rule. Only enemy units (and WALL specifically) create collision halts. Structures and friendly units never block movement. An ADR will document this deviation.

Cross-lane triggers (CHANGE LANE, Strich auto-switch) caused by sub-step 5 movement execute after all sub-step 5 movement completes and before sub-step 6 begins.

---

**Sub-step 6 — Standard Combat and Objective Damage (global)**

**Standard combat:** All enemy unit pairs sharing a cell (or halted facing each other from sub-step 5 collision) deal damage to each other simultaneously:
- Both units calculate and apply damage at the same time; no unit has combat priority over the other in sub-step 6
- **RANGE units:** attack the nearest enemy unit in the forward direction within their range. RANGE units without FIRST STRIKE attack only in this sub-step. RANGE + FIRST STRIKE units also attacked in sub-step 3 and attack again here — two separate attacks, each capable of consuming SHIELD independently.
- **COUNTERATTACK:** fires immediately when a unit receives damage in sub-step 6
- **Sequential damage:** multiple sources hitting one unit in sub-step 6 are applied in lane order (Lane 1 first)

**Objective damage:** After all unit combat resolves, any unit occupying the opponent's Cell 8 deals its ATK value as direct damage to the objective:
- Formula: `HP_new = max(0, HP_current - attacker.ATK)` (objective-system.md owns this formula)
- Objectives have 0 AR; the combat modifier stack does not apply — raw ATK is used (including active LEADER/spell buffs for this round)
- FIRST STRIKE does NOT advance objective damage to sub-step 3; objective damage is always sub-step 6
- On HP → 0: award +3 gold to attacking player; apply fake rewards if fake; check loss condition if real
- Attacking unit remains at Cell 8 and attacks again next round unless killed

---

### Combat Modifier Stack

Applied in this order for each individual attack (one attacker, one defender):

1. **SILENCE** — strip all keywords from the attacker for this combat
2. **STUN** — if attacker is STUNned, attack does not execute; skip all remaining steps
3. **LEADER bonus** — apply LEADER-granted ATK bonuses (snapshotted at round start; persist until end of RESOLUTION)
4. **Type advantage (ATK)** — if attacker's type beats defender's type: `ATK_combat += 1`
5. **VULNERABILITY X** — if defender has VULNERABILITY X: `ATK_effective = ATK_combat + X`
6. **RESISTANCE X** — if defender has RESISTANCE X: `ATK_effective = ATK_effective - X` (floor 0)
7. **ARMOR-PIERCING** — if attacker has ARMOR-PIERCING: `AR_defender = 0` (RESISTANCE was applied in step 6 independently and is unaffected)
8. **Type advantage (AR)** — if attacker's type beats defender's type: `AR_attacker_combat += 1`
9. **Formula:** `net_damage = max(0, ATK_effective − AR_defender)`
10. **SHIELD** — if defender has SHIELD: negate all damage from this sub-step's attacks (sub-step-level absorption; blocks all simultaneous attackers at once; consumed after blocking)

---

### Persistent Keyword States

**INJURED** — A unit is INJURED when `current_HP < max_HP`. State is evaluated at each sub-step boundary (not mid-sub-step). Damage in sub-step 3 activates INJURED at sub-step 4; effects apply from sub-step 4 onward. SILENCE strips INJURED-granted keywords (e.g., FIRST STRIKE granted by INJURED); the INJURED condition itself is not a keyword and is never silenced.

**SHIELD** — Persists until consumed. A SHIELD that is not triggered in a round carries into subsequent rounds. SHIELD absorbs any damage source: melee, RANGE, FIRST STRIKE, spell.

**LEADER** — Stat bonuses snapshotted at RESOLUTION entry. Persist until RESOLUTION ends regardless of LEADER death within that round. Recalculated each round.

**OUTNUMBERED** — Board count (friendly vs. enemy units) evaluated at the start of each sub-step. A unit becomes OUTNUMBERED if the count at sub-step entry favors the opponent.

---

### States and Transitions

| State | Entry Trigger | Actions on Entry | Exit Trigger |
|---|---|---|---|
| PLACEMENT | RSM broadcasts `S2CPhaseChanged(PLACEMENT)` | PlacementBuffer opens; placement timer starts | All players submit OR timer expires |
| RESOLUTION_EXECUTING | RSM fires `BeginResolution`; PlacementBuffer commits | Broadcast `S2CPlacementReveal`; snapshot unit stats and LEADER bonuses; execute sub-steps 1–6 | Sub-step 6 + all rewards applied |
| RESOLUTION_COMPLETE | Sub-step 6 complete | Fire `ResolutionComplete` to RSM; broadcast `S2CResolutionEvent` | RSM receipt |

**Internal sub-step sequence (within RESOLUTION_EXECUTING):**
```
APPLYING_PLACEMENTS → EXECUTING_CHARGE_X → EXECUTING_FIRST_STRIKE →
REMOVING_DEAD → EXECUTING_MOVEMENT → EXECUTING_COMBAT_AND_OBJECTIVE
```

Each internal state is logged for the RSM safety timeout. If RESOLUTION exceeds 60 seconds total, the round resolves as a Draw (RSM Rule 14).

**Reward ownership:** Kill gold (+1) fires at sub-step 4 (Combat Resolution owns this). Objective gold (+3) and fake rewards fire at sub-step 6 (Combat Resolution owns this). The RSM does not manage rewards.

---

### Interactions with Other Systems

| System | Data In | Data Out | Interface Contract |
|---|---|---|---|
| Board/Lane System | Unit positions, spawn ranges, lane layout, cell occupancy | Updated unit positions after sub-step 5 | Combat Resolution reads `BoardState`; writes updated positions. WALL collision uses cell occupancy. |
| Objective System | `objective_damage(HP_current, amount)` formula | Updated objective HP; `ObjectiveDestroyed` events | Owned by objective-system.md. Called at end of sub-step 6. |
| Economy System | `kill_gold_reward` = 1g, `objective_gold_reward` = 3g | Gold updates (`S2CGoldUpdate` unicast) | Combat Resolution fires gold rewards immediately on kill/destruction. |
| Network Protocol | `C2SSubmitPlacement` (from PlacementBuffer) | `S2CPlacementReveal`, `S2CResolutionEvent` | PlacementReveal sent before sub-step 1. ResolutionEvent sent after all sub-steps complete (batched log). |
| Server-side RNG | Ecaflip 1d6 dice rolls | Dice results broadcast to clients | Combat Resolution calls into RNG chain for Ecaflip keyword triggers. No other RNG in this system. |
| Round State Machine | `BeginResolution` (entry) | `ResolutionComplete` (exit) | RSM controls phase; Combat Resolution executes sub-steps and notifies on completion. |

## Formulas

### Formula 1: Combat Damage

The `net_damage` formula is defined as:

`net_damage = max(0, ATK_effective − AR_effective)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Base attack | `ATK_base` | u8 | 0–20 | Unit's card stat before modifiers |
| LEADER bonus ATK | `ATK_leader` | u8 | 0–5 | Granted by a living LEADER of the same family (snapshotted at round start) |
| Type advantage ATK | `ATK_type` | u8 | 0 or 1 | +1 if attacker's type beats defender's type |
| VULNERABILITY X | `ATK_vuln` | u8 | 0–5 | Defender keyword: incoming ATK increased by X |
| RESISTANCE X | `ATK_resist` | u8 | 0–5 | Defender keyword: incoming ATK decreased by X |
| Effective attack | `ATK_effective` | u8 | 0+ | `max(0, ATK_base + ATK_leader + ATK_type + ATK_vuln − ATK_resist)` |
| Base armor | `AR_base` | u8 | 0–10 | Unit's card stat before modifiers |
| ARMOR-PIERCING | `AR_pierce` | bool | — | If true: `AR_effective = 0` (applied after RESISTANCE) |
| Effective armor | `AR_effective` | u8 | 0+ | `AR_base` if no ARMOR-PIERCING; `0` if ARMOR-PIERCING is true |
| Net damage | `net_damage` | u8 | 0+ | `max(0, ATK_effective − AR_effective)` |

**Full modifier application order:**
```
ATK_effective = max(0,
  ATK_base
  + ATK_leader
  + ATK_type        // +1 if type advantage
  + ATK_vuln        // +X from VULNERABILITY
  - ATK_resist      // -X from RESISTANCE
)

AR_effective = if ARMOR_PIERCING { 0 } else { AR_base }
// Type advantage AR (+1) applies to the ATTACKER's own AR on retaliation — not to AR_effective

net_damage = max(0, ATK_effective - AR_effective)
```

**Output Range:** 0 (fully absorbed) to uncapped. Typical range in normal play: 0–8 per hit.

**Example:** Blade unit (ATK=3) attacks Shield unit (AR=2, RESISTANCE 1). Type advantage: ATK_type=+1. ATK_effective = max(0, 3+1−1) = 3. AR_effective = 2 (no ARMOR-PIERCING). `net_damage = max(0, 3−2) = 1`.

---

### Formula 2: Type Advantage

`type_beats(attacker_type, defender_type) → bool`

```
Blade > Arcane > Shield > Blade   (cyclic triangle)
Neutral: no advantage or disadvantage

type_beats(Blade, Arcane)  = true
type_beats(Arcane, Shield) = true
type_beats(Shield, Blade)  = true
type_beats(Neutral, _)     = false
type_beats(_, Neutral)     = false
type_beats(X, X)           = false  // same type, no advantage
```

**Output when `type_beats = true`:**
- `ATK_combat += 1` (step 4 in modifier stack)
- `AR_attacker_combat += 1` (attacker absorbs 1 more incoming retaliation damage this combat only)

These bonuses are per-combat-interaction only and do not modify base card stats.

**Current status:** Most cards are typed as Neutral (unassigned). Type assignment is progressive. Unassigned cards are treated as Neutral.

---

### Formula 3: Objective Damage (reference — owned by objective-system.md)

`HP_new = max(0, HP_current - ATK_effective)`

Where `ATK_effective` = attacker's ATK including active buffs (LEADER, spell buffs) but excluding all defensive modifiers (objectives have 0 AR; ARMOR-PIERCING, RESISTANCE, VULNERABILITY do not apply to objectives). See `objective-system.md` for full specification.

**Output Range:** 0 to `objective_hp` (5 by default). HP is monotonically non-increasing.

---

### Formula 4: RANGE Target Selection

`valid_targets(attacker, range_X) = enemy units at cells [attacker_cell+1 .. attacker_cell+range_X]` (Player A)
`valid_targets(attacker, range_X) = enemy units at cells [attacker_cell−range_X .. attacker_cell−1]` (Player B)

- Forward-only: targets cells toward the opponent's side exclusively
- Cell range clamped to [1, 8]
- Target priority: nearest enemy first (minimum cell-distance). If equidistant, server selects randomly.
- Walls and Structures within range are valid targets; friendly units are never valid targets
- Output: target unit reference, or null (no valid targets = no attack this sub-step)

## Edge Cases

**If two FIRST STRIKE units face each other in sub-step 3:** Both calculate and apply damage simultaneously using pre-combat stats. Neither has priority. If both die, both DEATH triggers fire (in lane order).

**If a RANGE + FIRST STRIKE unit attacks in sub-step 3 and the target survives:** The unit attacks again in sub-step 6. SHIELD consumed in sub-step 3 does not protect in sub-step 6 (separate sub-steps).

**If a RANGE + FIRST STRIKE unit kills in sub-step 3:** Target is removed in sub-step 4; the unit may attack a different valid target in sub-step 6 if one exists within range.

**If STUN is applied by an APPEARANCE trigger in sub-step 1:** STUN takes effect immediately. The STUNned unit cannot move or attack for the rest of this RESOLUTION, even if it has CHARGE.

**If a SHIELD unit is hit by multiple simultaneous attackers in the same sub-step:** SHIELD absorbs the entire sub-step's incoming damage (all simultaneous hits). SHIELD is consumed once. Sub-steps 3 and 6 are separate — SHIELD consumed in sub-step 3 does not protect in sub-step 6.

**If a RANGE attacker deals damage to a unit with COUNTERATTACK:** COUNTERATTACK does not fire. COUNTERATTACK requires physical proximity (same-cell contact). A RANGE unit that did not advance to the target's cell cannot be counter-attacked.

**If a WALL unit is in the path of an advancing enemy:** The enemy stops at the WALL's cell and attacks it in sub-step 6. WALL has 0 ATK and deals no damage back. A RANGE unit within range of a WALL attacks it from range without stopping.

**If two enemy units' paths would cross (each moving to the other's cell in the same tick):** Both halt at their cells from the previous tick (adjacent facing cells) and fight each other in sub-step 6.

**If two enemy units would land on the same cell in the same movement tick:** Both land there and fight normally in sub-step 6.

**If a unit at Cell 8 is killed by FIRST STRIKE in sub-step 3:** Removed in sub-step 4; does NOT deal objective damage in sub-step 6. Objective damage requires the unit to be alive at the end of sub-step 6.

**If two objectives from different players are both destroyed in the same sub-step 6:** Both loss conditions are checked simultaneously. If both players satisfy the loss condition, the game is a Draw — RSM broadcasts `S2CGameOver { loser: None, reason: Draw }`.

**If FINAL BLOW fires in sub-step 3:** The effect resolves immediately in sub-step 3, before sub-step 4. The killed unit is still on the board during FINAL BLOW resolution.

**If a unit is killed by the second of two sequential damage sources in sub-step 3 (lane-order application):** The source applied second (higher lane number) receives FINAL BLOW credit — it was the source whose damage reduced HP to 0.

**If INJURED grants FIRST STRIKE mid-RESOLUTION:** INJURED activates at the sub-step boundary after the damage that triggered it. A unit damaged in sub-step 3 does not retroactively attack in sub-step 3. It gains FIRST STRIKE from sub-step 4 onward — meaning FIRST STRIKE applies to sub-step 3 of the *next* round, not the current one.

**If a LEADER dies in sub-step 4:** The LEADER bonus (snapshotted at RESOLUTION entry) remains active for sub-steps 5 and 6 of the same RESOLUTION. Next round: if LEADER is still dead, no bonus is applied.

**If SILENCE is applied to a unit with an INJURED-granted FIRST STRIKE:** SILENCE strips the FIRST STRIKE keyword. The INJURED flag remains (SILENCE does not cure INJURED). If SILENCE ends, INJURED-granted FIRST STRIKE can return.

**If a STUNned unit has CHARGE X:** STUN suppresses all movement and attacks. CHARGE X bonus movement in sub-step 2 is also suppressed — STUN overrides CHARGE X.

**If an Ecaflip dice roll affects combat (e.g., Karla Blondie gains 1d6 ATK):** The RNG result is computed server-side via the RESOLUTION RNG chain before sub-step 1. Broadcast to clients via `S2CResolutionEvent`. No client-side RNG.

**If a unit is destroyed by Punition (self-targeting objective damage, Sacrier):** The self-destroyed objective triggers the loss condition check but the controller does not receive +3 gold (no attacker reward for self-destruction). See objective-system.md for the full self-destroy rule.

**If a unit dealing objective damage has an ATK-buffing spell active this round (e.g., Heure de Gloire):** The buffed ATK is used for objective damage. Objective damage uses `ATK_effective` including spell buffs but excluding ARMOR-PIERCING and RESISTANCE (inapplicable to objectives).

## Dependencies

### Upstream (Combat Resolution depends on these)

| System | GDD | Interface | Nature |
|---|---|---|---|
| Card Data & Pool | `card-data-pool.md` | Unit ATK, HP, AR, MP, type, keywords per card | Hard |
| Game Config | `game-config.md` | `kill_gold_reward`, `objective_gold_reward`, `placement_timer_seconds`, `objective_hp` | Hard |
| Server-side RNG | `server-rng.md` | RESOLUTION RNG chain; Ecaflip dice rolls | Hard |
| Economy System | `economy-system.md` | Kill/objective gold reward values; gold update event | Hard |
| Board/Lane System | `board-lane-system.md` | Cell positions, lane layout, spawn ranges, WALL cell occupancy | Hard |
| Round State Machine | `round-state-machine.md` | `BeginResolution` trigger; RESOLUTION safety timeout (60s) | Hard |
| Network Protocol | `network-protocol.md` | `S2CPlacementReveal`, `S2CResolutionEvent`, `C2SSubmitPlacement` | Hard |
| Objective System | `objective-system.md` | `objective_damage` formula; `ObjectiveDestroyed` event; fake reward dispatch | Hard |

### Downstream (these depend on Combat Resolution's output)

| System | GDD | What it consumes | Nature |
|---|---|---|---|
| Board Rendering | `board-rendering.md` | `S2CResolutionEvent` log for animation playback | Soft |
| Keyword System | `keyword-system.md` | Sub-step ordering, INJURED/DEATH/FINAL BLOW trigger timing | Soft — extends; must not contradict sub-step assignments |
| Prism System | `prism-system.md` | Unit positions at sub-step 5 (prism collection at spawn cell) | Soft |
| Class System | `class-system.md` | Kill/FINAL BLOW/DEATH trigger hooks; reserve mana interactions | Soft |
| Card Animations | `card-animations.md` | Sub-step event sequence from `S2CResolutionEvent` | Soft |

### Bidirectional consistency note

Board/Lane System GDD defines `unit_movement` as "skip intermediate cells." Combat Resolution defines step-by-step collision detection as a deliberate deviation. An ADR will document this divergence. The `unit_movement` formula (owned by board-lane-system.md) still defines the intended destination; Combat Resolution's step-by-step logic determines the actual final position after collision halts.

## Tuning Knobs

> All values below are configurable in `assets/config/game_config.ron`. Source of truth: `game-config.md`.

| Knob | Config Field | Default | Safe Range | Gameplay Impact |
|---|---|---|---|---|
| Kill gold reward | `kill_gold_reward` | 1g | 0–2g | 0 = removes combat economy loop; 2 = strong snowball incentive |
| Objective gold reward | `objective_gold_reward` | 3g | 2–5g | Higher = stronger swing on first destruction; lower = slower snowball |
| Objective HP | `objective_hp` | 5 HP | 3–8 HP | Lower = faster games; higher = more comeback potential |
| Placement timer | `placement_timer_seconds` | 10s | 5–20s | Shorter = more reflex pressure; longer = more deliberate tactical choice |
| Type advantage ATK bonus | *(hardcoded in resolution logic — not currently in GameConfig)* | +1 | +1–+2 | +2 would make type dominant; keep at +1 unless RPS feels weak in playtests |
| Type advantage AR bonus | *(hardcoded in resolution logic)* | +1 | +1–+2 | Paired with ATK bonus; change both together or not at all |

**Not in this system (but affect it):**
- Mana ramp (`mana_cap`) — controls cards-per-round; indirectly scales combat density
- RESISTANCE X, VULNERABILITY X, RANGE X — per-card values, not global knobs

**Interaction note:** `kill_gold_reward` and `objective_gold_reward` interact with the interest formula. Higher combat rewards accelerate the economic snowball — do not tune in isolation from `interest_threshold_gold`.

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

| # | Criterion | Type |
|---|---|---|
| CR-1 | GIVEN a unit with FIRST STRIKE in any lane, WHEN sub-step 3 executes, THEN that unit deals net_damage to any enemy unit sharing its cell before sub-step 5 movement occurs. | BLOCKING |
| CR-2 | GIVEN two FIRST STRIKE units sharing a cell, WHEN sub-step 3 executes, THEN both deal damage simultaneously (HP snapshots taken before either mutation is applied); if both receive lethal damage, both die and both DEATH triggers fire. | BLOCKING |
| CR-3 | GIVEN a unit with RANGE 1-X at cell C, WHEN sub-step 6 executes, THEN it attacks the nearest enemy unit in the forward direction (Player A: cells C+1 to C+X; Player B: cells C-X to C-1); it does not advance to do so; equidistant targets are selected randomly by the server. | BLOCKING |
| CR-4 | GIVEN a unit with RANGE 1-X AND FIRST STRIKE, WHEN RESOLUTION executes, THEN two distinct damage events are emitted: one in sub-step 3 (FIRST STRIKE pass) and one in sub-step 6 (standard combat pass). | BLOCKING |
| CR-5 | GIVEN a STUNned unit (including a CHARGE unit STUNned in sub-step 1), WHEN RESOLUTION executes, THEN the unit does not advance in sub-step 2 (CHARGE X suppressed), does not advance in sub-step 5, and does not attack in sub-steps 3 or 6. | BLOCKING |
| CR-6 | GIVEN a unit with SHIELD receives damage in sub-step 3, WHEN sub-step 3 resolves, THEN all sub-step 3 damage is negated and SHIELD is consumed; WHEN sub-step 6 attacks that same unit, THEN damage is applied normally (SHIELD already consumed). | BLOCKING |
| CR-7 | GIVEN a unit with SHIELD receives no damage during RESOLUTION, WHEN the next round's RESOLUTION begins, THEN SHIELD is still active (persists between rounds until consumed). | BLOCKING |
| CR-8 | GIVEN an advancing enemy unit whose next step would reach a WALL unit's cell, WHEN sub-step 5 executes, THEN the attacker halts at the WALL's cell; WHEN sub-step 6 executes, THEN the attacker deals net_damage to the WALL (WALL has 0 ATK, deals 0 damage back); if WALL HP reaches 0, the WALL is removed at the next DEATH-processing point and the attacker remains at that cell for the rest of this RESOLUTION. | BLOCKING |
| CR-9 | GIVEN two enemy units whose movement paths would cross in sub-step 5 (each moving to the other's cell in the same tick), WHEN sub-step 5 executes, THEN both halt at their pre-crossing cells; WHEN sub-step 6 executes, THEN both units fight each other. | BLOCKING |
| CR-10 | GIVEN a unit alive at Cell 8 at the end of sub-step 6, WHEN sub-step 6 completes, THEN that unit deals its ATK value as damage to the objective in that lane AND the unit remains at Cell 8 (attacks again next round unless killed). | BLOCKING |
| CR-11 | GIVEN a unit with FIRST STRIKE is at Cell 8 and is killed in sub-step 3, WHEN sub-step 4 removes it, THEN it does NOT deal objective damage in sub-step 6. | BLOCKING |
| CR-12 | GIVEN ATK_attacker = 3 and AR_defender = 5, WHEN combat resolves, THEN net_damage = 0 (damage cannot go negative). | BLOCKING |
| CR-13 | GIVEN a unit with RESISTANCE 2 (AR=1) attacked by a unit with ATK=4, WHEN combat resolves, THEN ATK_effective = max(0, 4−2) = 2; net_damage = max(0, 2−1) = 1. | BLOCKING |
| CR-14 | GIVEN a unit with ARMOR-PIERCING (ATK=3) attacks a unit with AR=4 and RESISTANCE 1, WHEN combat resolves, THEN ATK_effective = max(0, 3−1) = 2 (RESISTANCE applied first); AR_effective = 0 (ARMOR-PIERCING applied independently after RESISTANCE); net_damage = max(0, 2−0) = 2. | BLOCKING |
| CR-15 | GIVEN a Blade-type unit attacks an Arcane-type unit, WHEN combat resolves, THEN attacker's ATK_combat += 1 and attacker's AR_combat += 1 for this combat only; base card stats and other combats this round are unaffected. | BLOCKING |
| CR-16 | GIVEN a unit kills an enemy unit, WHEN sub-step 4 processes the dead unit, THEN the killing player immediately receives +1 gold. | BLOCKING |
| CR-17 | GIVEN a unit at Cell 8 destroys an objective, WHEN sub-step 6 completes, THEN the attacking player receives +3 gold and does NOT additionally receive +1 kill gold (objectives are not units). | BLOCKING |
| CR-18 | GIVEN the 2nd real objective of Player B is destroyed, WHEN the loss condition check runs, THEN the server broadcasts S2CGameOver { loser: Player B, reason: ObjectivesDestroyed } on the reliable channel. | BLOCKING |
| CR-19 | GIVEN both players' 2nd real objectives are destroyed in the same sub-step 6, WHEN the loss condition check runs, THEN the server broadcasts S2CGameOver { loser: None, reason: Draw }. | BLOCKING |
| CR-20 | GIVEN a unit with COUNTERATTACK receives damage from a RANGE attacker that did not occupy the target's cell, WHEN damage is applied, THEN COUNTERATTACK does NOT fire (physical proximity required). | BLOCKING |
| CR-21 | GIVEN a unit with COUNTERATTACK receives damage from a same-cell attacker in sub-step 3 or sub-step 6, WHEN damage is applied, THEN COUNTERATTACK fires immediately in that same sub-step (before the next sub-step begins). | BLOCKING |
| CR-22 | GIVEN a unit kills another unit in sub-step 3 (FIRST STRIKE), WHEN FINAL BLOW fires, THEN it fires in sub-step 3 (before sub-step 4); the killed unit is still present on the board during FINAL BLOW resolution. | BLOCKING |
| CR-23 | GIVEN a unit kills another unit in sub-step 6 (standard combat), WHEN FINAL BLOW fires, THEN it fires in sub-step 6 (not consolidated to sub-step 4). | BLOCKING |
| CR-24 | GIVEN a unit with an APPEARANCE ability enters play in sub-step 1, WHEN sub-step 1 executes, THEN the APPEARANCE ability fires before sub-step 2 begins. | BLOCKING |
| CR-25 | GIVEN unit A's DEATH trigger kills unit B in sub-step 4, WHEN DEATH triggers process, THEN B's DEATH trigger fires AFTER A's DEATH trigger completes (sequential chain, not simultaneous). | BLOCKING |
| CR-26 | GIVEN a unit takes damage in sub-step 3 that puts HP below maximum (activating INJURED), WHEN sub-step 3 completes, THEN the INJURED bonus is NOT active in sub-step 3; it IS active from sub-step 5 onward for this RESOLUTION. | BLOCKING |
| CR-27 | GIVEN a unit at Cell 8 with ATK=3 attacks an objective with HP=2, WHEN sub-step 6 completes, THEN objective HP = 0 (not −1; floor at 0 applies) and the objective is destroyed. | BLOCKING |
| CR-28 | GIVEN a RANGE unit with enemies both forward and behind it (both within range X), WHEN sub-step 6 executes, THEN only the forward enemy is targeted; the enemy behind is never a valid RANGE target. | BLOCKING |
| CR-29 | GIVEN a RANGE + FIRST STRIKE unit attacks a SHIELD unit in sub-step 3 (consuming SHIELD), WHEN sub-step 6 executes the second attack from the same unit, THEN the attack deals full damage (SHIELD consumed in sub-step 3 does not protect in sub-step 6). | BLOCKING |
| CR-30 | GIVEN S2CPlacementReveal is broadcast, WHEN RESOLUTION begins, THEN PlacementReveal is sent before any sub-step 1 effects execute and contains both players' full placements in one atomic message. | BLOCKING |
| CR-31 | GIVEN a unit with CHARGE X, WHEN sub-step 2 executes, THEN the unit advances X additional cells (subject to WALL-blocking and crossing rules); WHEN sub-step 5 executes, THEN the unit additionally advances its MP value as a separate movement. | ADVISORY |
| CR-32 | GIVEN RESOLUTION completes all 6 sub-steps, WHEN RESOLUTION_COMPLETE fires, THEN S2CResolutionEvent containing a sequenced log of all sub-step events is broadcast to all players before S2CPhaseChanged(DRAFT_SHOP). | ADVISORY |
| CR-33 | GIVEN a LEADER unit grants +1 ATK to family units and is killed in sub-step 4, WHEN sub-steps 5 and 6 execute, THEN family units retain the +1 ATK bonus; the following round with LEADER dead, the bonus is absent. | ADVISORY |
| CR-34 | GIVEN a unit gains FIRST STRIKE via INJURED (activated at sub-step boundary after sub-step 3 damage), WHEN sub-step 3 of the NEXT round executes, THEN the unit attacks as a FIRST STRIKE unit. | ADVISORY |

## Open Questions

[To be designed]
