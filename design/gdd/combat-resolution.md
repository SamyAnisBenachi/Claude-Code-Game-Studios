# Combat Resolution

> **Status**: Designed — /design-review complete 2026-04-30 (MAJOR REVISION NEEDED → revised in-session; P0 fixes: SHIELD canonical pre-check rule, COUNTERATTACK formula defined, OQ3 seed registered, bilateral+multi-source overlap specified, INJURED/OUTNUMBERED ordering fixed, kill-log mechanism added; OQ1/OQ5 closed)
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-30
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

All units with the movement keyword CHARGE X advance an additional X cells simultaneously, applying the same collision rules as sub-step 5. This sub-step affects ONLY units with a numeric CHARGE X value. The combat keyword CHARGE ("can act this round") has no effect in this sub-step. **STUNned units skip this sub-step** — STUN suppresses CHARGE X bonus movement (same as it suppresses standard movement in sub-step 5).

---

**Sub-step 3 — FIRST STRIKE Attacks (global)**

All units with the FIRST STRIKE keyword deal damage simultaneously across all lanes:
- **RANGE + FIRST STRIKE:** A unit with both keywords also attacks in this sub-step (in addition to sub-step 6). It targets the nearest enemy unit within its forward RANGE. If multiple targets are equidistant, the server resolves randomly.
- **Damage application:** When multiple sources hit the same unit in sub-step 3, damage is applied sequentially in lane order (Lane 1 first, Lane 5 last). Each source is resolved separately — HP is updated between each source.
- **COUNTERATTACK:** If a unit with COUNTERATTACK is attacked in sub-step 3 by one or more melee attackers (same-cell contact), its COUNTERATTACK effect fires once after all sub-step 3 damage to it is resolved — including SHIELD pre-check absorption. COUNTERATTACK fires even if SHIELD absorbed all incoming damage. COUNTERATTACK is triggered by direct melee contact only. A RANGE attacker that did not advance to the target's cell **cannot** trigger COUNTERATTACK. See *COUNTERATTACK Retaliation Formula* in the Combat Modifier Stack section for damage, FINAL BLOW eligibility, and chain behavior.
- **STUN:** A STUNned unit does not attack in sub-step 3 and does not move in sub-step 5. It is completely frozen for the round.
- Dead units (HP reduced to 0) from sub-step 3 damage are NOT removed until sub-step 4. A unit killed in sub-step 3 can still deal FIRST STRIKE damage in the same sub-step, and may trigger COUNTERATTACK. *(Design note: this is an explicit design choice following the standard simultaneous-resolution model. Visually, the attack animation plays from the dying unit before it collapses — the death animation is deferred to sub-step 4 so that the unit's own FIRST STRIKE attack resolves first. This is a known exception to the "board tells the truth" pillar; it is accepted because simultaneous resolution is the intended strategic model. Both attack animations should overlap in time, with the kill animation completing after the killing-blow impact.)*

---

**Sub-step 4 — Remove Dead Units (global)**

All units at 0 HP across all lanes are removed from the board:
- DEATH triggers fire for each removed unit (in lane order if multiple die simultaneously)
- DEATH trigger chains are sequential: if A's DEATH trigger kills B, B's DEATH trigger fires after A's completes
- FINAL BLOW fires in the sub-step where the kill occurred (sub-step 3 for FIRST STRIKE kills; sub-step 6 for standard combat kills) — not consolidated to sub-step 4
- **Kill gold:** +1 gold awarded immediately to the controlling player of the unit whose attack dealt the final damage. Objective destruction does NOT award +1 kill gold (objectives are not units).

---

**Sub-step 5 — Standard Movement (global)**

All non-STUNned units advance toward the opponent's side by their MP value. Movement is resolved step-by-step (1 cell per tick), simultaneously across all lanes.

**Pre-loop rules:**
- Each unit's destination is computed **once at sub-step 5 entry** as `clamp(current_cell + direction × MP, 1, 8)`. It is not re-evaluated per tick.
- Units with MP=0 (e.g., WALL structures) are **excluded from the tick loop** — they never attempt to advance.
- **Collision detection is per-lane only.** Units in different lanes cannot collide with each other.

**Tick loop:**
1. Each tick: all units attempt to advance 1 cell toward the opponent's side.
2. **WALL exception:** A unit stops the moment its next step would bring it to a cell occupied by an enemy WALL unit. The advancing unit fights the WALL in sub-step 6 (WALL has 0 ATK, so it takes damage but deals none back).
3. **Collision — two sub-cases:**
   - **(A) Same-cell landing:** If two enemy units would both advance to the same cell in the same tick, both land there and fight in sub-step 6.
   - **(B) Path crossing:** If two enemy units would swap positions (each moving to the other's cell in the same tick), both halt at their cells from the **previous tick** (adjacent facing cells) and fight in sub-step 6.
4. Movement continues tick by tick until all units have reached their destinations or been halted.

*Note: a unit blocked on tick 0 (its very first advance would be blocked) stays at its entry cell for this sub-step.*

> **Design note:** This step-by-step collision model is a deliberate deviation from the Board/Lane System GDD's "skip intermediate cells" rule. Only enemy units (and WALL specifically) create collision halts. Structures and friendly units never block movement. An ADR will document this deviation.

Cross-lane triggers (CHANGE LANE, Strich auto-switch) caused by sub-step 5 movement execute after all sub-step 5 movement completes and before sub-step 6 begins.

---

**Sub-step 6 — Standard Combat and Objective Damage (global)**

**Standard combat:** All enemy unit pairs sharing a cell (or halted facing each other from sub-step 5 collision) deal damage to each other simultaneously:

**Two damage scopes:**
- **(A) Bilateral pair combat** (two units fighting each other): damage is computed simultaneously — both calculate against their pre-combat HP snapshots; neither sees the other's damage before computing their own. This is the primary melee/RANGE exchange model.
- **(B) Multi-source on single target** (unit C is attacked by two units from different lanes): damage is applied **sequentially in lane order** (Lane 1 first, Lane 5 last). HP is updated between each source. Each attacker runs the modifier stack against C's current HP.

**SHIELD special rule:** Before running any modifier stack, check if the defender has SHIELD. If yes, negate all damage from this sub-step's attacks (from all attackers simultaneously) and consume SHIELD once. Do not run the per-attacker modifier stacks. This pre-check is an exception to the multi-source sequential rule — SHIELD always absorbs the entire sub-step regardless of source count.

- Both units in bilateral pair combat calculate and apply damage at the same time; no unit has combat priority over the other in sub-step 6
- **RANGE units:** attack the nearest enemy unit in the forward direction within their range. RANGE units without FIRST STRIKE attack only in this sub-step. RANGE + FIRST STRIKE units also attacked in sub-step 3 and attack again here — two separate attacks, each capable of consuming SHIELD independently.
- **COUNTERATTACK:** fires once after all sub-step 6 damage to the unit is resolved — including SHIELD pre-check absorption. Fires even if SHIELD absorbed all incoming damage. Applies to same-cell combat and to **collision-halt adjacent-cell combat**. RANGE attackers that did not advance to the target's cell cannot trigger COUNTERATTACK. See *COUNTERATTACK Retaliation Formula* in the Combat Modifier Stack section for damage, FINAL BLOW eligibility, and chain behavior.
- **Sequential damage:** multiple sources hitting one unit in sub-step 6 are applied in lane order (Lane 1 first) per the multi-source rule above

**Objective damage:** After all unit combat resolves, any unit occupying the opponent's Cell 8 deals its ATK value as direct damage to the objective:
- Formula: `HP_new = max(0, HP_current - attacker.ATK)` (objective-system.md owns this formula)
- Objectives have 0 AR; the combat modifier stack does not apply — raw ATK is used (including active LEADER/spell buffs for this round)
- FIRST STRIKE does NOT advance objective damage to sub-step 3; objective damage is always sub-step 6
- On HP → 0: award +3 gold to attacking player; apply fake rewards if fake; check loss condition if real
- Attacking unit remains at Cell 8 and attacks again next round unless killed

---

### Combat Modifier Stack

Applied in this order for each individual attack (one attacker, one defender):

**SHIELD pre-check (runs before the modifier stack):** If the defender has SHIELD, negate all incoming damage from this sub-step's attacks, consume SHIELD once, and skip the modifier stack entirely. This is a sub-step-level absorption — SHIELD blocks all simultaneous attackers at once regardless of source count. COUNTERATTACK fires after this pre-check (not before it). The attacker's own SHIELD absorbs COUNTERATTACK retaliation via the same pre-check logic.

1. **SILENCE** — strip all keywords from the attacker for this combat
2. **STUN** — if attacker is STUNned, attack does not execute; skip all remaining steps
3. **LEADER bonus** — apply LEADER-granted ATK bonuses (snapshotted at round start; persist until end of RESOLUTION)
4. **Type advantage (ATK)** — if attacker's type beats defender's type: `ATK_combat += 1`
5. **VULNERABILITY X** — if defender has VULNERABILITY X: `ATK_effective = ATK_combat + X`
6. **RESISTANCE X** — if defender has RESISTANCE X: `ATK_effective = ATK_effective - X` (floor 0)
7. **ARMOR-PIERCING** — if attacker has ARMOR-PIERCING: `AR_defender = 0` (RESISTANCE was applied in step 6 independently and is unaffected)
8. **Type advantage (AR)** — if attacker's type beats defender's type: `AR_attacker_combat += 1`
9. **Formula:** `net_damage = max(0, ATK_effective − AR_defender)`

---

### Persistent Keyword States

**INJURED** — A unit is INJURED when `current_HP < max_HP`. State is evaluated at each sub-step boundary (not mid-sub-step). Damage in sub-step 3 activates INJURED at sub-step 4; effects apply from sub-step 4 onward. SILENCE strips INJURED-granted keywords (e.g., FIRST STRIKE granted by INJURED); the INJURED condition itself is not a keyword and is never silenced.

**SHIELD** — Persists until consumed. A SHIELD that is not triggered in a round carries into subsequent rounds. SHIELD absorbs any damage source: melee, RANGE, FIRST STRIKE, spell.

**LEADER** — Stat bonuses snapshotted at RESOLUTION entry. Persist until RESOLUTION ends regardless of LEADER death within that round. Recalculated each round.

**OUTNUMBERED** — Board count (friendly vs. enemy units) evaluated at the start of each sub-step. A unit becomes OUTNUMBERED if the count at sub-step entry favors the opponent.

**Sub-step boundary evaluation order:** At each sub-step boundary (e.g., SS3→SS4, SS5→SS6), evaluations occur in this fixed order:
1. Remove dead units (HP ≤ 0) from the board; fire DEATH triggers (SS4 only; SS6 dead-unit removal runs as a post-combat cleanup pass after all SS6 attacks resolve)
2. Recompute OUTNUMBERED based on post-removal board state
3. Evaluate INJURED status for all surviving units based on HP mutations in the completed sub-step

This ordering ensures OUTNUMBERED counts reflect the final board state after deaths and INJURED reflects actual damage dealt. A unit that becomes simultaneously INJURED and OUTNUMBERED at the SS3→SS4 boundary gains both bonuses at SS4 entry — the OUTNUMBERED count uses the post-death board (deaths from SS3 are removed first).

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

**Reward ownership:** Kill gold (+1) is awarded when the dead unit is formally removed (sub-step 4 for SS3 kills; post-combat cleanup pass at end of sub-step 6 for SS6 kills). Objective gold (+3) and fake rewards fire during sub-step 6 when objective HP reaches 0. All gold events are embedded as `GoldAwarded` entries in `S2CResolutionEvent` (batch-only; no standalone `S2CGoldUpdate` during RESOLUTION). The RSM does not manage rewards.

**Kill-gold attribution mechanism:** The exclusive system maintains an internal `kill_log: Vec<KillRecord { killer_player_id: PlayerId, victim_id: UnitId, lethal_sub_step: u8 }>`. When a unit's HP reaches 0 during any sub-step, a `KillRecord` is appended immediately at the point of lethal damage. For SS3 kills: the `kill_log` is drained at sub-step 4 when those units are formally removed. For SS6 kills: the `kill_log` is drained during the post-SS6-combat cleanup pass. Each drain emits one `GoldAwarded { player_id: killer_player_id, amount: 1 }` entry into the `ResolutionLog`.

**Internal iteration budget:** The `resolve_combat` exclusive system tracks a monotonically increasing internal iteration counter across all sub-step loops (tick loop in SS5, DEATH trigger chains in SS4, COUNTERATTACK chains). If the counter exceeds 10,000 total iterations for a single RESOLUTION, the algorithm aborts and the RSM is notified to broadcast `S2CGameOver { loser: None, reason: Draw }`. This guards against infinite loops in pathological trigger configurations. The 60-second RSM safety timeout remains as an outer backstop for cases where the exclusive system itself hangs (process-level watchdog territory).

---

### Interactions with Other Systems

| System | Data In | Data Out | Interface Contract |
|---|---|---|---|
| Board/Lane System | Unit positions, spawn ranges, lane layout, cell occupancy | Updated unit positions after sub-step 5 | Combat Resolution reads `BoardState`; writes updated positions. WALL collision uses cell occupancy. |
| Objective System | `objective_damage(HP_current, amount)` formula | Updated objective HP; `ObjectiveDestroyed` events | Owned by objective-system.md. Called at end of sub-step 6. |
| Economy System | `kill_gold_reward` = 1g, `objective_gold_reward` = 3g | Gold updates (`GoldAwarded` entry in `S2CResolutionEvent` batch) | During RESOLUTION, all gold rewards are embedded as `GoldAwarded` entries in `S2CResolutionEvent`. No standalone `S2CGoldUpdate` is sent during RESOLUTION. Kill gold is applied at sub-step 4 (when the dead unit is formally removed). Objective gold is applied at sub-step 6 (when objective HP reaches 0). |
| Network Protocol | `C2SSubmitPlacement` (from PlacementBuffer) | `S2CPlacementReveal`, `S2CResolutionEvent` | PlacementReveal sent before sub-step 1; ResolutionEvent sent after all sub-steps complete (batched log). Both messages are enqueued in the same `resolve_combat` exclusive-system frame. Pre-implementation gate: verify or enforce that Lightyear 0.26 delivers same-frame reliable-channel messages in enqueue order (PlacementReveal before ResolutionEvent). See ADR-011 risk table. |
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
| Type advantage AR bonus | `AR_attacker_combat` | u8 | 0 or 1 | +1 when `type_beats(attacker_type, defender_type) = true`. Applied to the attacker's own AR **when the attacker is defending against the simultaneous return-strike** (see two-pass algorithm below). Does not modify `AR_effective` of this attack. |
| Net damage | `net_damage` | u8 | 0+ | `max(0, ATK_effective − AR_effective)` |

**Full modifier application order:**
```
// ⚠️ Rust note: compute in i32 (or use saturating_sub on u8) before clamping.
// Naive u8 subtraction will panic in debug or wrap in release if ATK_resist > sum.

ATK_effective = max(0i32,
  ATK_base as i32
  + ATK_leader as i32
  + ATK_type as i32   // +1 if type advantage
  + ATK_vuln as i32   // +X from VULNERABILITY
  - ATK_resist as i32 // -X from RESISTANCE
) as u8

AR_effective = if ARMOR_PIERCING { 0 } else { AR_base }
// AR_attacker_combat is NOT applied here — it applies in the opponent's counter-attack computation (see two-pass algorithm below)

net_damage = max(0, ATK_effective - AR_effective)
```

**Two-pass algorithm for simultaneous bilateral combat (sub-step 6):**

When unit A and unit B are fighting simultaneously (bilateral pair), run two separate modifier stack computations:

```
// Pass 1: A attacks B
(net_damage_A_to_B, AR_attacker_combat_A) = run_modifier_stack(attacker=A, defender=B)

// Pass 2: B attacks A — A's type-advantage AR bonus applies here
AR_effective_A_as_defender = AR_base_A + AR_attacker_combat_A  // AR_attacker_combat from Pass 1
(net_damage_B_to_A, _) = run_modifier_stack(attacker=B, defender=A,
                           override_defender_AR = AR_effective_A_as_defender)

// Apply both results simultaneously
A.hp -= net_damage_B_to_A
B.hp -= net_damage_A_to_B
```

*Note: `AR_attacker_combat_A` is 0 if A does not have type advantage over B; 1 if it does. B's AR bonus is discarded (`_`) because the cyclic triangle guarantees `type_beats(B, A) = false` whenever `type_beats(A, B) = true` — this discard is only safe under strict anti-symmetric typing. Any future non-cyclic type extension must revise this algorithm.*

**Output Range:** 0 (fully absorbed) to **31 maximum** (ATK_base=20 + ATK_leader=5 + ATK_type=1 + ATK_vuln=5, vs AR=0 with ARMOR-PIERCING). u8 storage is safe (max 31 < 255). Typical range in normal play: 0–8 per hit.

**Note on arithmetic:** Compute all modifier sums in i32 (not u8) for both addition and subtraction before clamping. Naive u8 addition (`ATK_base + ATK_leader`) is safe at current stat caps (max 31) but will silently overflow if ATK_base range is ever extended past 224. The formula comment applies to the full expression.

**Bilateral + cross-lane attacker overlap rule:**

When a bilateral pair (A↔B) also receives cross-lane damage in the same sub-step (e.g., unit C from a different lane attacks B):

1. **Bilateral first:** compute A↔B bilateral exchange using pre-combat HP snapshots for both A and B. Apply both results simultaneously.
2. **Cross-lane sequential after:** apply C's damage against B's post-bilateral HP, in lane order relative to other cross-lane attackers. HP is updated between each cross-lane source.

The SHIELD pre-check for B against C uses B's post-bilateral SHIELD state — if A's bilateral attack consumed B's SHIELD, it is unavailable for C's cross-lane attack. If B's SHIELD is intact after the bilateral exchange (because A is a RANGE attacker that did not trigger SHIELD, or A's damage was 0), B's SHIELD absorbs C's cross-lane attack.

**Example:** Blade unit (ATK=3, AR=1) attacks Arcane unit (ATK=2, AR=1, RESISTANCE 1). Type advantage: ATK_type=+1, AR_attacker_combat=+1.
- Pass 1 (Blade→Arcane): ATK_effective = max(0, 3+1−1) = 3. AR_effective = 1. `net_damage_Blade_to_Arcane = 2`.
- Pass 2 (Arcane→Blade): AR_effective_Blade = AR_base_Blade(1) + AR_attacker_combat(1) = 2. ATK_effective_Arcane = max(0, 2+0−0) = 2. `net_damage_Arcane_to_Blade = max(0, 2−2) = 0`.
- Result: Blade takes 0 damage (type advantage AR reduced incoming damage to 0), Arcane takes 2 damage.

---

### COUNTERATTACK Retaliation Formula

When COUNTERATTACK fires (once per sub-step, after all damage to the defending unit is resolved):

**Timing:** After all incoming attacks for the sub-step are processed (damage applied or absorbed via SHIELD pre-check). COUNTERATTACK fires even if all incoming damage was absorbed by SHIELD.

**Targets:** All melee attackers that attacked this unit in this sub-step (same-cell or collision-halt adjacency). RANGE attackers who did not advance to the target's cell are excluded.

**Retaliation damage:** runs the full modifier stack (steps 1–9 above), treating the COUNTERATTACK unit as attacker and the original attacker as defender. The original attacker's SHIELD pre-check applies to each retaliation independently — if the original attacker has SHIELD, COUNTERATTACK damage is absorbed by their SHIELD.

**FINAL BLOW:** eligible. If COUNTERATTACK retaliation reduces an attacker to 0 HP, FINAL BLOW fires for the COUNTERATTACK unit in the current sub-step.

**Chain (fires once):** If the original attacker also has COUNTERATTACK, they retaliate back once against the COUNTERATTACK unit (running the same full modifier stack). The chain stops there — no further COUNTERATTACK fires from either side.

**Multiple attackers:** If two or more melee attackers attacked the COUNTERATTACK unit in the same sub-step, the COUNTERATTACK unit retaliates against all of them simultaneously using pre-retaliation HP snapshots for each bilateral pair.

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
- Cell range clamped to [1, 8]; `range_X` must be ≥ 1 (RANGE 0 is not a valid keyword value)
- Target priority: nearest enemy first (minimum cell-distance). If equidistant, server selects randomly (see OQ3 — RNG seed slot required)
- Walls and Structures within range are valid targets; friendly units are never valid targets
- Output: target unit reference, or null (no valid targets = no attack this sub-step)
- **RANGE unit at Cell 8:** When a RANGE unit reaches Cell 8, no forward enemy cells exist within range. The `valid_targets` function returns null. However, the **objective damage rule** still applies — any unit occupying Cell 8 at the end of sub-step 6 deals objective damage regardless of whether it is a RANGE unit. The RANGE targeting formula does not govern objective damage; the sub-step 6 objective damage rule does.

## Edge Cases

**If two FIRST STRIKE units face each other in sub-step 3:** Both calculate and apply damage simultaneously using pre-combat stats. Neither has priority. If both die, both DEATH triggers fire (in lane order).

**If a RANGE + FIRST STRIKE unit attacks in sub-step 3 and the target survives:** The unit attacks again in sub-step 6. SHIELD consumed in sub-step 3 does not protect in sub-step 6 (separate sub-steps).

**If a RANGE + FIRST STRIKE unit kills in sub-step 3:** Target is removed in sub-step 4; the unit may attack a different valid target in sub-step 6 if one exists within range.

**If STUN is applied by an APPEARANCE trigger in sub-step 1:** STUN takes effect immediately. The STUNned unit cannot move or attack for the rest of this RESOLUTION, even if it has CHARGE.

**If a SHIELD unit is hit by multiple simultaneous attackers in the same sub-step:** SHIELD absorbs the entire sub-step's incoming damage (all simultaneous hits). SHIELD is consumed once. Sub-steps 3 and 6 are separate — SHIELD consumed in sub-step 3 does not protect in sub-step 6.

**If a RANGE attacker deals damage to a unit with COUNTERATTACK:** COUNTERATTACK does not fire. COUNTERATTACK requires direct melee contact. A RANGE unit that did not advance to the target's cell cannot be counter-attacked.

**If two melee units fight from adjacent cells (collision-halt sub-step 5):** COUNTERATTACK fires normally. Adjacent-cell collision-halt combat counts as direct melee contact for COUNTERATTACK purposes. Both units are fighting each other at close range; the same-cell distinction applies only to RANGE combat (where the attacker stays far away).

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

**If a unit with COUNTERATTACK is hit by an attacker while SHIELD is active:** COUNTERATTACK fires after all sub-step damage (including SHIELD absorption) is resolved. SHIELD absorbs the incoming damage; COUNTERATTACK fires anyway — the unit was attacked regardless of absorption outcome. The COUNTERATTACK retaliation is a separate damage event; the original attacker's SHIELD pre-check applies to the retaliation independently.

**If a STUNned unit also has SHIELD:** STUN suppresses the unit's outgoing actions (movement and attacks) only. SHIELD is a passive defensive state — it absorbs incoming damage regardless of STUN. A STUNned unit with SHIELD still absorbs the first incoming attack via the SHIELD pre-check and consumes SHIELD as normal. STUN does not strip or disable SHIELD.

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
| Type advantage ATK bonus | `type_advantage_atk_bonus` | +1 | +1–+2 | +2 would make type dominant; keep at +1 unless RPS feels weak in playtests. **Action: add this field to game-config.md.** |
| Type advantage AR bonus | `type_advantage_ar_bonus` | +1 | +1–+2 | Paired with ATK bonus; change both together or not at all. **Action: add this field to game-config.md.** |

**RANGE + FIRST STRIKE power note:** A unit with both keywords attacks twice per round (sub-step 3 and sub-step 6). At ATK=4 with no defenders, it can deal up to 8 objective damage in one round — 160% of default objective HP. Against a SHIELD unit, it strips SHIELD in sub-step 3 and deals full damage in sub-step 6. This is a premium keyword combo: **card design must not create RANGE+FIRST STRIKE units with ATK > 3 unless the card has significant HP or cost constraints.** Validate this combo's power ceiling in early playtests before nerfing — it may be intentionally strong as a rare late-game threat.

**Not in this system (but affect it):**
- Mana ramp (`mana_cap`) — controls cards-per-round; indirectly scales combat density
- RESISTANCE X, VULNERABILITY X, RANGE X — per-card values, not global knobs

**Interaction note:** `kill_gold_reward` and `objective_gold_reward` interact with the interest formula. Higher combat rewards accelerate the economic snowball — do not tune in isolation from `interest_threshold_gold`.

## Visual/Audio Requirements

> **Art Direction:** Ankama/Wakfu cel-shaded 2D, bold clean outlines, rich saturated colors. No blur, glow, or bloom on unit sprites — impact flashes are flat 1-frame color fills. Single exception: objective destruction uses a full-screen opacity overlay to simulate bloom without a GPU post-process pass.

### Color Conventions (Combat)

Color encodes timing across all combat events — these four roles must never overlap or drift:

| Color | Hex | Meaning |
|---|---|---|
| Prism White flash | `#EEF4FF` | FIRST STRIKE impact (timing advantage) |
| Warm orange flash | `#E07020` | Standard combat impact (normal timing) |
| Arcane Gold pulse | `#F5C842` | Keyword trigger fired (APPEARANCE, DEATH, LEADER bonus) |
| Crimson Slate | `#8B1A2F` | Damage received (numbers + death tint) |

### Mandatory Pause Gates

Three pauses between sub-steps serve "No idle spectating" — they are active board-reading time, not dead time:
- **100ms** after last CHARGE X unit lands → before FIRST STRIKE attacks begin
- **100ms** after all sub-step 5 movement settles → before sub-step 6 combat begins
- **100ms** after sub-step 6 combat resolves → before objective damage numbers appear

### Placement Reveal

- All 5 lanes reveal simultaneously in one frame — no per-lane stagger. Stagger implies sequence where none exists.
- 3-frame flip per unit: back-of-card silhouette → Prism White edge-on squash flash → front-face sprite. Total: 80–100ms.
- Frame 3: base ring flashes player-side color for 1 frame (Sky Blue `#3A8EDB` for Player A, Terracotta `#D45C22` for Player B), then snaps to normal.

### CHARGE X vs Standard Movement

| Attribute | CHARGE X (sub-step 2) | Standard (sub-step 5) |
|---|---|---|
| Motion trail | 2–3 fading sprite copies at 40% opacity, 1 cell behind, fade over 150ms | None |
| Speed | 150ms per cell | 120ms per cell |

Trail is the differentiator: trail = CHARGE X; no trail = standard.

### FIRST STRIKE vs Standard Attack

| Attribute | FIRST STRIKE (sub-step 3) | Standard (sub-step 6) |
|---|---|---|
| Impact flash | Prism White `#EEF4FF` | Warm orange `#E07020` |
| Lanes animated simultaneously | Yes (global pass) | Yes (global pass) |

Attack animation per hit: smear frame (1 frame, limb stretched toward target) + impact flash (1 frame) + recover. Total 200–250ms per Art Bible Section 5.5.

### Damage Numbers

- Position: at impact point, above target unit (never below/beside). Minimum 24px at full-board zoom.
- Style: Crimson Slate `#8B1A2F`, 2.5× base typography, Heavy weight.
- Float: +40px upward over 500ms; fade begins at 250ms.
- Two sources hitting same target in lane order: first source offset left, second right — never overlap.
- Spawned as world-space entities at the impact cell (not HUD overlay) — position stays correct regardless of camera.
- **SHIELD absorption exception:** No damage number when SHIELD absorbs. Absence of a number after an attack connects is the information signal.

### SHIELD Visual States

| Moment | Visual |
|---|---|
| SHIELD active | Hexagonal Prism White glyph (8×8px) on base ring |
| SHIELD absorbing damage | Hex glyph scales to 1.5× over 100ms → snaps back and disappears |
| SHIELD consumed | 3-particle Prism White burst (4px particles, radial 20px outward, 250ms fade) |

SHIELD glyph consumed in sub-step 3 does not reappear for sub-step 6. Glyph absence = SHIELD gone.

### Death Animation

Frame 1 (full opacity) → Frame 2 (50% vertical squash, Crimson Slate 60% tint) → Frame 3 (vanish with 3–4 Crimson Slate particles, radial outward, 200ms fade). Total: 350ms. Fast and brutal — not cinematic.

If DEATH trigger: Arcane Gold ring pulse radiates from unit base before Frame 3. DEATH trigger effect plays after pulse fades. Sequential — never overlap death pulses from chained units.

### APPEARANCE Trigger

Arcane Gold aura pulse on entry: radiates from base ring, reaches ~2× unit width diameter, fades 200ms. If the APPEARANCE effect applies to a target, the target's reaction plays separately — caster's gold pulse = "I did something"; target's visual = "something happened to me."

### RANGE Attack

No movement. Unit plays a 5px forward-lean toward target (80ms, springs back). Projectile (`vfx_ranged_bolt_[type]_small.png`, tinted by attacker class color) travels at 600–800px/s. On arrival: warm orange `#E07020` impact flash + damage number.

### Persistent State Indicators

All indicators attach to the unit entity (survive movement). They must never animate except on state change; never disappear during other animations.

| State | Visual | Location |
|---|---|---|
| STUN | 3 Prism White rotating stars, 6px each, 1 rev/sec | Orbiting base ring |
| SHIELD active | Hexagonal Prism White glyph, 8×8px | On base ring |
| INJURED | Outline pulses Void `#0D0D14` ↔ rust-brown `#5C2E10` at 0.5 Hz | Unit outline |
| LEADER | Arcane Gold crown glyph, 8×8px | Above unit head |
| LEADER family buff | Arcane Gold 20% opacity base ring tint on buffed allies | On base ring |
| SILENCE | Outline desaturates Void → grey `#666666` | Unit outline |
| OUTNUMBERED | Crimson Slate arrow-down on player's lane-line side | Lane edge (per lane) |

LEADER bonus tint fades over 300ms after LEADER dies — the bonus persists this round per rules, so the fading tint correctly shows "bonus outlasted its source."

### Objective Damage and Destruction

**Taking damage:** HP pips flash Crimson Slate per pip removed. Damage number (Crimson Slate, 2.5×, float +60px, 500ms fade — taller clearance than unit damage). Attacking unit plays a 10px forward-lean.

**Destruction:** 3-frame Prism White full-screen overlay (80% → 60% → 30% opacity). Pedestal swaps to cracked-sprite variant. Flame animation stops permanently.
- **Real objective:** warm gold fill floods lane column for 400ms.
- **Fake objective:** `?` glyph scales to 3× over 200ms, rotates 45°, dissolves. No gold fill.

**HUD objective dot:** Real destroyed → Crimson Slate filled `×`. Fake destroyed → mid-grey `?`.

### Kill / Objective Gold Feedback

| Event | Visual | Duration |
|---|---|---|
| Kill gold +1 | `+1` Arcane Gold, 1.5× base, floats +20px above killing unit | 400ms fade |
| Objective gold +3 | `+3` Arcane Gold, 2× base, HUD gold counter tick with bloom pulse | 600ms fade |

### Animation Constraints (Bevy 0.18 / WASM)

| Constraint | Value |
|---|---|
| RESOLUTION display time target | ≤3,000ms single exchange; ≤5,000ms full 5-lane contested round |
| Tweening easing | `EaseOutQuad` for all combat translates. Spring/elastic reserved for UI confirmation events only. |
| Tween scoping | One `Animator<Transform>` per unit entity. No shared-clock chains. |
| Damage number entities | World-space transient entities (`Tween<Transform>` float + `Tween<Sprite>` alpha fade) |
| VFX atlas budget | ≤120 of 256 available frames in `atlas_vfx` at 64×64px. Combat VFX estimate: 21 frames. |
| 5-lane parallelism | All lanes animate simultaneously during all global passes. No lane-by-lane sequential playback. |

### Audio Events (Direction Pending)

Audio specification requires separate direction from a sound designer. Events below require distinct audio cues; timing gated on animation completion, not on server event receipt:

- Placement reveal (5-lane simultaneous flip)
- FIRST STRIKE impact (distinct timbre from standard)
- Standard combat impact
- Unit death
- SHIELD absorption (blocked hit, no damage) + SHIELD break (consumed)
- Objective damage hit
- Objective destruction — real
- Objective destruction — fake reveal
- Kill gold reward +1
- COUNTERATTACK response

### Implementation Priority

1. Placement reveal flip (emotional hook — must land before any playtest)
2. Damage numbers + impact flashes (Prism White FIRST STRIKE, orange standard)
3. Unit movement translates (standard + CHARGE X trail)
4. Death sequence
5. SHIELD absorption visual + persistent SHIELD glyph
6. Objective damage + destruction burst
7. Kill/objective gold floats
8. Persistent state indicators (STUN stars, SHIELD glyph, LEADER crown, INJURED outline)
9. APPEARANCE / DEATH trigger gold pulses
10. COUNTERATTACK animation, RANGE projectile, WALL trim
11. OUTNUMBERED lane indicator
12. SILENCE grey outline

## UI Requirements

Combat Resolution does not own any screen or interactive panel. Its UI requirements are a **display contract** on downstream systems (`board-rendering.md`, `card-animations.md`, `hud.md`) that must be met for RESOLUTION to be legible.

**R1 — Simultaneous placement reveal**
When `S2CPlacementReveal` arrives, all new units across all 5 lanes must appear simultaneously — not sequentially per lane, not unit-by-unit. Sequential reveal breaks the simultaneous-placement fantasy.
*Owner:* Board Rendering GDD.

**R2 — Sub-step visual separation**
`S2CResolutionEvent` delivers a sequenced event log. The client replay must introduce enough visual pause between sub-steps that players can parse what caused what. Each sub-step boundary (1→2, 2→3, ..., 5→6) must be legible. Sub-step 3 effects must complete visually before sub-step 5 movement begins.
*Owner:* Card Animations GDD.

**R3 — Per-unit status indicators visible during RESOLUTION**
At any point during RESOLUTION playback, the following states must be identifiable by sight without inspecting the unit:
- SHIELD active: distinct visual indicator (e.g., bubble or border) — disappears when consumed
- STUN active: frozen/locked visual — must be distinct from WALL (also immobile)
- INJURED: required only if the unit gains new keywords via INJURED (visual reflects enhanced state, not HP loss)
- LEADER bonus: units receiving the LEADER ATK buff carry a visible aura for the duration of RESOLUTION
*Owner:* Board Rendering GDD.

**R4 — Damage numbers attributable to source**
`net_damage` values appear as floating text over the target. Multiple sources hitting the same target in one sub-step stagger by a few frames to remain individually readable. Both players see all damage — no fog of war on combat results.
*Owner:* Board Rendering GDD.

**R5 — Kill and objective gold reward display**
+1 kill gold and +3 objective gold must display contextually (near the killed unit / destroyed objective), not only in the HUD gold counter. The player must be able to see mid-RESOLUTION that they earned gold and why, without looking away from the board.
*Owner:* HUD GDD (reward popup), Board Rendering GDD (contextual origin point).

**R6 — Objective HP updates during sub-step 6 execution**
Objective HP bars must decrement as damage is dealt within sub-step 6, not at RESOLUTION end. If an objective reaches 0 HP, the destruction visual must play before `S2CPhaseChanged(DRAFT_SHOP)` arrives.
*Owner:* Board Rendering GDD.

**R7 — No interactive elements during RESOLUTION**
Hand cards, shop slots, and all placement controls must be non-interactive and visually suppressed during RESOLUTION_EXECUTING. The player is in observation mode. Only read-only HUD elements (gold, HP bars) remain visible. No modal dialogs.
*Owner:* Board Rendering / HUD GDD coordination.

## Acceptance Criteria

| # | Criterion | Type |
|---|---|---|
| CR-1 | GIVEN a unit with FIRST STRIKE in any lane, WHEN sub-step 3 executes, THEN that unit deals net_damage to any enemy unit sharing its cell before sub-step 5 movement occurs. | BLOCKING |
| CR-2 | GIVEN two FIRST STRIKE units sharing a cell, WHEN sub-step 3 executes, THEN both deal damage simultaneously (HP snapshots taken before either mutation is applied); if both receive lethal damage, both die and both DEATH triggers fire. | BLOCKING |
| CR-3 | GIVEN a unit with RANGE 1-X at cell C with a single nearest enemy, WHEN sub-step 6 executes, THEN it attacks that nearest enemy (Player A: minimum cell-distance in cells C+1 to C+X; Player B: C-X to C-1); it does not advance to do so. Equidistant target selection: promote to BLOCKING once the `range_equidistant_select` seed slot is wired up in the RANGE keyword story. Until then treat as ADVISORY for the equidistant case only. | ADVISORY (equidistant case) / BLOCKING (single-nearest case) |
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
| CR-21 | GIVEN a unit with COUNTERATTACK receives damage from a melee attacker (same-cell contact OR adjacent-cell collision-halt combat) in sub-step 3 or sub-step 6, WHEN the damage event is received (before SHIELD absorption check), THEN COUNTERATTACK fires immediately in that same sub-step (before the next sub-step begins). | BLOCKING |
| CR-22 | GIVEN a unit kills another unit in sub-step 3 (FIRST STRIKE), WHEN FINAL BLOW fires, THEN it fires in sub-step 3 (before sub-step 4); the killed unit is still present on the board during FINAL BLOW resolution. | BLOCKING |
| CR-23 | GIVEN a unit kills another unit in sub-step 6 (standard combat), WHEN FINAL BLOW fires, THEN it fires in sub-step 6 (not consolidated to sub-step 4). | BLOCKING |
| CR-24 | GIVEN a unit with an APPEARANCE ability enters play in sub-step 1, WHEN sub-step 1 executes, THEN the APPEARANCE ability fires before sub-step 2 begins. | BLOCKING |
| CR-25 | GIVEN unit A's DEATH trigger kills unit B in sub-step 4, WHEN DEATH triggers process, THEN B's DEATH trigger fires AFTER A's DEATH trigger completes (sequential chain, not simultaneous). | BLOCKING |
| CR-26 | GIVEN a unit takes damage in sub-step 3 that puts HP below maximum (activating INJURED), WHEN sub-step 3 completes, THEN the INJURED bonus is NOT active in sub-step 3; it IS active **from sub-step 4 onward** for this RESOLUTION (INJURED activates at the sub-step 4 boundary — consistent with Persistent Keyword States). Since sub-step 4 has no attacks, INJURED-granted FIRST STRIKE is not exercised until sub-step 3 of the next round (see CR-34). | BLOCKING |
| CR-27 | GIVEN a unit at Cell 8 with ATK=3 attacks an objective with HP=2, WHEN sub-step 6 completes, THEN objective HP = 0 (not −1; floor at 0 applies) and the objective is destroyed. | BLOCKING |
| CR-28 | GIVEN a RANGE unit with enemies both forward and behind it (both within range X), WHEN sub-step 6 executes, THEN only the forward enemy is targeted; the enemy behind is never a valid RANGE target. | BLOCKING |
| CR-29 | GIVEN a RANGE + FIRST STRIKE unit attacks a SHIELD unit in sub-step 3 (consuming SHIELD), WHEN sub-step 6 executes the second attack from the same unit, THEN the attack deals full damage (SHIELD consumed in sub-step 3 does not protect in sub-step 6). | BLOCKING |
| CR-30 | GIVEN S2CPlacementReveal is broadcast, WHEN RESOLUTION begins, THEN PlacementReveal is sent before any sub-step 1 effects execute and contains both players' full placements in one atomic message. | BLOCKING |
| CR-31 | GIVEN a unit with CHARGE X, WHEN sub-step 2 executes, THEN the unit advances X additional cells (subject to WALL-blocking and crossing rules); WHEN sub-step 5 executes, THEN the unit additionally advances its MP value as a separate movement. | BLOCKING |
| CR-32 | GIVEN RESOLUTION completes all 6 sub-steps, WHEN RESOLUTION_COMPLETE fires, THEN S2CResolutionEvent MUST contain: exactly one SubStepEntry per executed sub-step, one CombatDamage record per damage application (including non-lethal hits), one UnitRemovedRecord per killed unit, one GoldAwarded record per gold event, and one KeywordTriggered record per APPEARANCE/DEATH/COUNTERATTACK/FINAL BLOW activation — all in chronological (sub_step, trigger_index) order. S2CPhaseChanged(DRAFT_SHOP) must NOT be observed by any client before S2CResolutionEvent is received. | BLOCKING |
| CR-33 | GIVEN a LEADER unit (grants +1 ATK to family units) is killed in sub-step 4 of round N, WHEN round N sub-steps 5 and 6 execute, THEN family units' ATK_effective includes the +1 LEADER bonus; WHEN round N+1 RESOLUTION begins with LEADER still dead, THEN family units' ATK_effective equals ATK_base only (verified by asserting damage dealt equals ATK_base-derived formula with no LEADER term). | BLOCKING |
| CR-34 | GIVEN a unit gains FIRST STRIKE via INJURED (activated at sub-step boundary after sub-step 3 damage), WHEN sub-step 3 of the NEXT round executes, THEN the unit attacks as a FIRST STRIKE unit. | BLOCKING |

| CR-35 | GIVEN two melee units that halted on adjacent cells after a path-crossing collision in sub-step 5, WHEN sub-step 6 combat resolves, THEN COUNTERATTACK fires for any unit with the COUNTERATTACK keyword when it receives damage (collision-halt adjacency satisfies melee contact per the proximity definition). | BLOCKING |
| CR-36 | GIVEN a unit with SHIELD is attacked simultaneously by two FIRST STRIKE units from different lanes in sub-step 3, WHEN sub-step 3 resolves, THEN the SHIELD unit takes 0 damage from both attackers AND SHIELD is consumed exactly once. | BLOCKING |
| CR-37 | GIVEN unit X in Lane 2 (ATK=3, FIRST STRIKE) and unit Y in Lane 4 (ATK=3, FIRST STRIKE) both target unit Z (HP=4, AR=0) in sub-step 3, WHEN sub-step 3 resolves, THEN Lane 2 damage is applied first (Z HP → 1), then Lane 4 damage (Z HP → 0, Z killed); FINAL BLOW credit is awarded to the Lane 4 unit's controller. | BLOCKING |
| CR-38 | GIVEN unit A's APPEARANCE trigger deals lethal damage to unit B in sub-step 1, AND unit C also has an APPEARANCE trigger in sub-step 1, WHEN sub-step 1 executes, THEN unit C's APPEARANCE fires before unit B's DEATH trigger; unit B's DEATH trigger fires only after all sub-step 1 APPEARANCE effects complete. | BLOCKING |
| CR-39 | GIVEN a unit with a CHANGE LANE trigger activates in sub-step 1, WHEN all sub-step 1 effects complete, THEN the CHANGE LANE executes before sub-step 2 begins; the unit's new lane position is used for sub-step 2 CHARGE X movement. | BLOCKING |
| CR-40 | GIVEN unit A's APPEARANCE trigger applies STUN to unit B (which has CHARGE X) in sub-step 1, WHEN sub-step 2 executes, THEN unit B does NOT advance via CHARGE X (STUN suppresses sub-step 2); WHEN sub-step 5 executes, THEN unit B does NOT advance (STUN suppresses sub-step 5). | BLOCKING |
| CR-41 | GIVEN RESOLUTION_EXECUTING has been active for > 60 seconds (simulated via injected RSM safety timeout), WHEN the timeout fires, THEN the server broadcasts S2CGameOver { loser: None, reason: Draw } and RESOLUTION_COMPLETE does not fire for that round. | BLOCKING |
| CR-42 | GIVEN a unit with VULNERABILITY 2 (AR=1) is attacked by a unit with ATK=3, WHEN combat resolves, THEN ATK_effective = 3+2 = 5; net_damage = max(0, 5−1) = 4. | BLOCKING |
| CR-43 | GIVEN a unit with FIRST STRIKE and ARMOR-PIERCING is SILENCEd before RESOLUTION, WHEN sub-step 3 executes, THEN the unit does NOT attack (FIRST STRIKE stripped by SILENCE); WHEN sub-step 6 executes, THEN the unit's attack does not apply ARMOR-PIERCING (stripped by SILENCE); the defender's AR_base is used normally. | BLOCKING |
| CR-44 | GIVEN a RANGE 1-3 unit at cell C with a WALL unit at cell C+2, WHEN sub-step 5 executes, THEN the RANGE unit's cell position is unchanged (it does not halt or advance toward the WALL); WHEN sub-step 6 executes, THEN the RANGE unit attacks the WALL from cell C and a CombatDamage record is emitted with target = WALL. | BLOCKING |
| CR-45 | GIVEN a RANGE + FIRST STRIKE unit kills its sub-step 3 target, AND a different enemy unit exists within range at sub-step 6 entry, WHEN sub-step 4 removes the killed unit AND sub-step 6 executes, THEN the RANGE unit acquires the surviving enemy as its sub-step 6 target and a CombatDamage record is emitted for that target. | BLOCKING |

## Open Questions

**OQ1 — RESOLVED.** ADR-017 Decision 2 formally documents the step-by-step collision detection boundary. Rule A: the destination formula (Board/Lane GDD F1) governs Trap/Prism triggering and applies to all movement. Rule B: the tick-by-tick enemy collision loop (Combat Resolution SS5) is an additional layer for enemy unit obstruction only. These rules are complementary, not contradictory.

**OQ2 — RESOLVED.** Type advantage ATK and AR bonuses moved to `game_config.ron` as `type_advantage_atk_bonus` and `type_advantage_ar_bonus` (both default +1). **Action required:** Add these two fields to `game-config.md` before Combat Resolution epic begins. This was a coding-standards violation (technical-preferences.md forbids hardcoded balance values).

**OQ3 — RESOLVED (seed registered).** `range_equidistant_select` seed slot added to the server-rng.md RESOLUTION caller table. Consumes 1 seed per equidistant RANGE attack (0 seeds when a single nearest target exists). The seed name is `range_equidistant_select`. CR-3 remains ADVISORY until the RANGE keyword story begins; promote to BLOCKING at that point.

**OQ4 — RESOLVED.** COUNTERATTACK fires for all direct melee contact including collision-halt adjacent-cell combat. COUNTERATTACK is a "defensive reactive strike" — any unit fighting via direct melee (same-cell or adjacent halted) can trigger it. RANGE attackers that stayed at distance cannot. Updated in CR-21, sub-step 6 rules, and Edge Cases.

**OQ5 — RESOLVED.** `CombatDamage` and `KeywordTriggered` variants are now defined in both network-protocol.md Section D.2 and ADR-017 Decision 3. Pre-implementation gate: reconcile the `UnitId` type name (ADR-017) vs. `EntityId` (NP D.2) across both documents — they must match before code is written.
