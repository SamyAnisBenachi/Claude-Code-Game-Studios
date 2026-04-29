# Combat Resolution

> **Status**: In Design
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

[To be designed]

## Edge Cases

[To be designed]

## Dependencies

[To be designed]

## Tuning Knobs

[To be designed]

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

[To be designed]

## Open Questions

[To be designed]
