# Class System

> **Status**: In Review (2026-04-30 pass 3 — 11 GDD-local blockers resolved in-session; see review log)
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-30
> **Implements Pillar**: Deep emergence · Simple surface

## Overview

The Class System is the authoritative design reference for the six playable classes in Lanes and Lies — **Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida** — and the rules that govern class-specific cards, tokens, and mechanical extensions. As a data layer, it specifies how class is established (chosen at LOBBY, immutable for the duration of the game, publicly visible to the opponent), how class gates the personal shop (each shop slot rolls 50/50 between the player's class card pool and the neutral pool — class cards never appear at auction, which is neutral-only by design), and the registry of class-specific tokens and transformations that exist outside the base 148-card class library: **Mummy** (Xelor DEATH spawns), **Chacha Noir** (Ecaflip transformation target), **Sinistro** (Xelor objective-attached spell), **Graine/Seed**, **Madoll**, **La Gonflable**, **La Sacrifiée** (Sadida token suite). As a player-facing system, it specifies each class's identity in one line — *Iop is aggro rush, Cra is range control, Sacrier is masochist sacrifice, Xelor is reserve tempo, Ecaflip is luck chaos, Sadida is seed setup* — and the signature Krosmic cards that anchor each archetype's strategic spine. The Class System defines the cross-class interaction matrix: which classes mechanically intersect (Sacrier's *Sang Méprise* reveals all of both players' objectives via Objective System's unicast reveal channel; Xelor's *Xelorium* steals only opponent **current mana**, leaving their reserve untouched per Economy's pool independence; Punition's self-destruction triggers the loss condition through Objective System's standard HP-zero check), and which class-specific keyword overrides extend (never contradict) the base Keyword System catalog (e.g., Sacrier's INJURED activates additive bonus stats; Cra's RANGE feeds the RANGE attack stack already specified in Combat Resolution sub-step 6).

The Class System owns the **what** of each class — identity, card library scope, token spawn rules, class-bound dice/RNG hooks — but never the **how** of the underlying mechanics it leans on. Base mana ramp belongs to Economy. Base shop weighting belongs to Card Acquisition. Base keyword definitions belong to Keyword System. Server-side RNG chains for Ecaflip dice belong to Server-side RNG. This GDD's job is to formalize and gap-fill the per-class details that the master GDD §3.9 sketches but that no other GDD owns. For the player, class is the most personal expression of choice in the game: you pick at lobby, you inhabit for the round, and the 24-25 cards in your shop slot pool become the strategic vocabulary you compose with for the rest of the game. Pillar alignment: **Deep emergence** — six classes × ~25 cards each × interaction matrix produces the strategy fanout that justifies replay; **Simple surface** — each class identity fits in one phrase, first-read legible.

## Player Fantasy

The Class System serves the fantasy of **authorship-via-class-rhythm** — each class is a different kind of clockwork the player chose to wind, and mastery is feeling the gear-ratio of *your* machine.

Where the Keyword System made every card a small contract — *FIRST STRIKE fires before retaliation, BODYGUARD stands in front, INJURED unlocks under damage* — the Class System sits one layer above. It tells the player: *which clockwork you committed to at lobby is the timer of every round you'll play.* Iop is wound short and loud — three rounds, charge, swing, decide. Xelor is wound long — accumulate reserve over five rounds, fire Rollback once, watch the entire board jump four cells in unison. Sadida is wound across — seeds laid round one, walked over round three, converted to Madolls round five. Cra wound thin and far. Sacrier wound *underarmored on purpose*. Ecaflip wound on dice rhythm — variance is the gear-ratio.

**What the player should feel:**
- **"I am winding *this* machine."** — When the Iop player commits everything to a charging mid-lane round 3, that is not a decision their hand made — it is a decision their *class* made through them. The player adopted the Iop tempo at lobby and is now executing the Iop closing window. Six lobbies, six different games on the same board.
- **"My class made that loop possible."** — Round 3, Xelor. You held back placement last round on purpose to fatten reserve. This round Rollback fires and your three units leap four cells in unison. You didn't cast a spell — you closed a circuit you wired two rounds ago. No other class could have completed that circuit. The class is the tempo signature.
- **"The same neutral card means different things in my hand."** — The neutral Tofu you bought at shop is a chump-blocker in a Cra hand, a Seed-cell occupier in a Sadida hand, a charging body in an Iop hand. The class isn't *the* gears in the machine — it is the *gear ratio* that determines what every other card means in your timeline.

**The six tempo signatures:**

| Class | Tempo signature |
|---|---|
| Iop | Short and loud — wound tight at round 1, releases in rounds 2-4. The closing window opens early. |
| Cra | Long and thin — fires in lines from outside the trade range. Range is the rhythm. |
| Sacrier | Underarmored on purpose — every wound is a wind-up. Payoff scales with damage taken. |
| Xelor | Slow accumulator — reserve ticks for five rounds, detonates once. The longest gear. |
| Ecaflip | Dice rhythm — variance is the gear-ratio. Plan ranges, not certainties. |
| Sadida | Laid across — seeds round 1, walked round 3, converted round 5. The widest gear, three-rounds-deep. |

**The four rhythm archetypes.** "Authorship-via-class-rhythm" encompasses four distinct modes of player agency, all valid expressions of the same pillar:

- **Authored outcome** (Iop, Xelor, Cra): the player directly triggers a deterministic effect at a chosen moment — Authority charges, Rollback fires, Harcèlement deals damage. The player controls both the timing and the result.
- **Authored sacrifice** (Sacrier): the player deliberately accepts damage and loss (INJURED activation, Punition self-sacrifice) as the mechanism that winds the clock. The authorship is in the *acceptance* — each wound is a chosen input, not an accident.
- **Prepared battlefield** (Sadida): the player authors the spatial trap architecture — which lanes carry seeds, at what density, in what configuration. The payoffs activate when opponents traverse the prepared field. The authorship is the *preparation itself*, not the trigger moment. A Sadida who places seeds precisely is the architect of the board; the clock is wound during placement, not at activation.
- **Authored risk exposure** (Ecaflip): the player chooses when and how to deploy variance — which die spells to cast, when to accept a coin-flip trade, how to sequence plays around outcome ranges. The authorship is the *risk architecture*, not the specific coin-flip result. Ecaflip players who lose by coin-flip chose to play that coin; the machine is wound by that choice.

These are distinct gear-ratios under the same pillar, not exceptions to it. A Sadida and an Iop both author their board state; the difference is whether the authorship fires immediately (Iop) or across opponent traversal (Sadida).

**What to avoid:** The class fantasy must never collapse into "powerful spells go boom." The Krosmic cards are the *teeth* of the gear, not the fantasy itself. The fantasy is the *rhythm* — that the Xelor player who cashes Rollback feels the same authorship as the Iop player who lands Authority on round 3, even though the cards do completely different things. Equally: class must never erase the universal game beneath it. All players have a reserve; all players read the same shop pool; all players play on the same 4×5 board. Class is the *ratio* the player chose to apply, not a separate game.

*Pillar alignment: "Authorship through anticipation" — the class is the longest gear in the player's clockwork, the one wound at lobby and ticked across the entire game. "Deep emergence" — six tempo signatures × the universal mechanics they re-frame produce the strategic fanout. "Simple surface" — each class's tempo fits in one phrase, first-read legible.*

## Detailed Design

### Core Rules

**Class is a session-scoped player property.** It is selected during LOBBY phase, locked when the player commits via the "Ready" signal, frozen at the LOBBY → DRAFT_INITIAL transition, and never changes for the remainder of the session. Each player's class is publicly visible to the opponent at LOBBY and throughout the game. The class choice gates: (1) which class card library is sampled when a personal shop slot rolls a class card; (2) which class-specific tokens that player can spawn via card effects.

**The six classes.** Each class commits the player to a fixed card library and a tempo signature.

| Class | Card count | Tempo signature | Signature Krosmics |
|---|---|---|---|
| **Iop** | 24 | Aggro rush — short, loud, front-loaded | *Authority*, *Heure de Gloire*, *Felida*, *Appel à la Baston*, *Katsu Mi* |
| **Cra** | 25 | Range control — long and thin | *Criblage*, *Harcèlement*, *Flèche Destructrice*, *Guy Yomtella*, *Lucy Fayre* |
| **Sacrier** | 25 | Masochist sacrifice — underarmored on purpose | *Sang Méprise*, *Punition*, *Fulgurance*, *Jet le Pied Volant*, *L'Enklarveur* |
| **Xelor** | 25 | Reserve tempo — slow accumulator, fires once | *Rollback*, *Garde-Temps*, *Miss Nuit*, *Dévouement*, *Xelorium* |
| **Ecaflip** | 24 | Luck chaos — variance is the gear-ratio | *Craps*, *Miranda*, *Defhi Croquets*, *Chacha/Bow Meow* |
| **Sadida** | 25 | Setup nature — laid across, three-rounds-deep | *Pollinisation*, *Savoir Sadida*, *Sacrifice Poupesque*, *Sylvine Folherbe* |

**Class card filtering boundary.** Class System defines *what cards qualify as a player's class library*; it does **not** define the shop-slot generation algorithm. Personal shop slot mechanics (the 50/50 class vs. neutral roll, weighting, refresh) belong to Card Acquisition GDD. Auctions are neutral-only by design — class cards never appear there. This boundary is binding: Class System never overrides Card Acquisition's roll machinery, and Card Acquisition never owns class-card identity.

**Cross-class card flow exception (Drheller and similar uniform draws).** The Lane 3 prism's *Draw 1 card* effect, the Drheller family's DEATH-draw effect, and any other "draw a random card" effect uses Server-side RNG's `draw_random` chain (uniform pick, no 50/50 class filter applied). Cross-class cards entering a player's hand via these effects are legal and expected — a Cra player can receive an Iop card via Drheller. This is a deliberate design feature: it produces rare cross-class moments without systematizing them. Implementers must NOT apply the shop's 50/50 class filter to triggered draws.

**Token registry.** Class System owns the spawn rules and stat blocks for these class-specific token entities. Tokens are not in the 148-card class library — they cannot be drafted, drawn, or auctioned. They appear only as effects of class cards.

| Token | Class | Source cards | Stats / effect |
|---|---|---|---|
| **Mummy / Momie** | Xelor | Corum (DEATH), Quartz (DEATH), Dévouement (transform target) | Unit token (HP=2/ATK=2/MP=3); passive: gains +1 reserve whenever it suffers damage |
| **Sinistro** | Xelor | Sinistro spell, Diod Dewit (APPEARANCE) | Spell-attached, lives on a friendly objective; deals 1 damage/RESOLUTION to opposing-lane objective; destroyed if its parent objective takes damage |
| **Chacha Noir** | Ecaflip | Chacha/Bow Meow (transform target) | Unit token (HP=2/ATK=2/MP=6); replaces the target unit at the target's cell; no passive effect |
| **Graine / Seed** | Sadida | Pollinisation, Sac de Graines, Ronce (DEATH), Sadida cards with seed-place effects | Cell-attached marker; +1 AR to friendly walk-over (permanent on unit), 1 damage to enemy walk-over; persistent until explicitly consumed/converted; **max 1 seed per cell** |
| **Madoll / La Folle** | Sadida | Graines de Folie (convert from Seeds), Sylvine Folherbe (hand grant) | Unit token (HP=3/ATK=1/MP=3); passive: Spell-type cards cost 1 less while in play (Trap and Order cards unaffected — OQ-CS-4 closed ruling) |
| **La Gonflable** | Sadida | Sylvine Folherbe + class spawn paths | Unit token (HP=3/ATK=2/MP=3); END OF MOVEMENT: heals other friendly units in its lane for 2 HP |
| **La Sacrifiée** | Sadida | Sacrifice Poupesque, Sadida class spawn paths | Unit token (HP=2/ATK=2/MP=3); DEATH: 1 damage to enemy units in its lane |

Tokens carry a `source_class` tag for two purposes: (1) LEADER family bonus checks (LEADER cards specifying "Sadida_Token family" boost only Sadida-sourced tokens, regardless of current controller); (2) Miranda-stolen token integrity (a stolen token retains its `source_class` and cannot be boosted by the new controller's class-LEADER cards). Tokens do **not** inherit the owning player's class. SILENCE on a parent unit does NOT propagate to its previously-spawned tokens — tokens are independent entities once spawned.

### States and Transitions

**Class assignment lifecycle.**

| State | Trigger | Class state | Mutability |
|---|---|---|---|
| LOBBY entry | Player joins lobby | `class = None` | Editable — player can pick and re-pick |
| LOBBY ready | Player clicks "Ready" | `class = Some(C); locked = true` for that player | Frozen for that player; other players still editing |
| DRAFT_INITIAL entry | All players Ready, RSM transitions LOBBY → DRAFT_INITIAL | All players' `class = Some(C); locked = true` | Immutable for the rest of the session |
| Reconnect | Player rejoins after disconnect | `class` restored from server state | Immutable |

**Class invariant.** At the moment LOBBY → DRAFT_INITIAL fires, every active player MUST have `class = Some(C)`. If any player has `class = None`, the transition is refused and the lobby remains in LOBBY (this is a Game Session System / RSM concern; Class System merely declares the invariant). `class = None` is architecturally unreachable in any phase ≥ DRAFT_INITIAL.

**Per-round Class System trackers.** Class System has **no round-scoped state of its own**. The two trackers below are owned by other systems and merely *consumed* by class effects:

- `miss_nuit_reserve_gained_this_round: u32` — owned by Economy System; tracks reserve actually granted this round (not raw card-play count); reset to 0 at DRAFT phase entry.
- `garde_temps_used_this_game: u32` — owned by **Game Session System** (game-scoped, not round-scoped); initialized to 0 at `on_lobby_to_draft_initial` transition; persists across all rounds; never reset mid-game; discarded at session end. This is the enforcement counter for `garde_temps_per_game_cap`.
- `sang_meprise_active: bool` — owned by Objective System; cleared at RESOLUTION end.

**Token state.** Tokens occupy normal board state once spawned and are owned by Combat Resolution / Board-Lane System. Class System does not maintain a "live tokens" registry per round.

### Interactions with Other Systems

The cross-class interaction matrix below names every class-mechanic that crosses a system boundary, and which document owns each side. Class System owns the *trigger and intent*; the receiving system owns the *effect application*.

| Mechanic | Class | Trigger | Data flow | Effect owner |
|---|---|---|---|---|
| **Gelure** (transfer) | Xelor | Spell played | `current_mana → reserve`; `current_mana := 0` | Economy System |
| **Xelorium** (steal current mana) | Xelor | Spell at sub-step 1 | `opponent.current_mana → self.reserve`; `opponent.current_mana := 0`; opponent reserve untouched | Economy System |
| **Sablier** (swap reserve) | Xelor | Spell | `opponent.reserve -= 1` (saturating); `self.reserve += 1` | Economy System |
| **Many de Brakmar** (drain reserve) | Xelor | APPEARANCE | `opponent.reserve -= 2` (saturating) | Economy System |
| **Brûlure Temporelle** | Xelor | Spell | 2 damage to target + `self.reserve += 1` | Combat Resolution + Economy |
| **Mummy passive** | Xelor | Whenever a Mummy takes damage during sub-step | `self.reserve += 1` (no cap — each hit adds 1 reserve; Mummy's HP=2 is the de facto accumulation ceiling in normal combat) | Economy System |
| **Garde-Temps** (reserve cost gate) | Xelor | Spell, costs 20 reserve | Server gates: play accepted iff `self.reserve >= 20`; on accept, deduct 20 reserve and destroy chosen enemy objective | Economy + Objective System |
| **Rollback** (charge by reserve) | Xelor | Spell at sub-step 2 | `n := self.reserve`; `self.reserve := 0`; each friendly unit charges `n` cells (lane-clamped) | Keyword System (CHARGE X) + Economy |
| **Sarcophage** | Xelor | Spell | Draw 1, or 2 if `self.reserve >= 5` | Card Acquisition |
| **Aiguille** | Xelor | Spell | 2 damage, or 5 damage if `self.reserve >= 5` | Combat Resolution |
| **Miss Nuit** (reserve passive) | Xelor | While in play, on opponent card play | `self.reserve += 1`; capped at +2 per round (`miss_nuit_cap`) | Economy System |
| **Patek Tag** (prism destroy) | Xelor | APPEARANCE | Destroys 1 enemy prism; downstream: opponent loses one Lane reward source | Board / Lane System |
| **Sang Méprise** (objective reveal) | Sacrier | Spell at sub-step 1 | Server unicasts identity of every alive objective (both players) to both players for current RESOLUTION; client clears at RESOLUTION end | Objective System (reveal channel) |
| **Punition** (self-destroy + AOE) | Sacrier | Spell at sub-step 1 | Sacrifice 1 chosen alive real objective of self; then 3 damage to each alive opponent objective; if self loss-condition triggers (≥ 3 real destroyed), RSM declares Sacrier loser | Objective System + RSM |
| **Fulgurance** (position swap) | Sacrier | Spell | Swap positions of 2 chosen friendly units | Combat Resolution / Board |
| **Jet le Pied Volant** | Sacrier | Whenever a friendly unit or objective takes damage | This unit charges 1 cell | Combat Resolution (CHARGE) |
| **INJURED bonus stats** | Sacrier (and any) | Each sub-step boundary while `current_HP < max_HP` | Class-specific additive bonuses (e.g., Edass +2 ATK + FIRST STRIKE while INJURED) — applied/removed via Keyword System's INJURED re-evaluation | Keyword System |
| **Ecaflip 1d6 / coin flip** | Ecaflip | Card-specific (APPEARANCE / spell / DEATH) | Roll computed via RESOLUTION RNG chain (server-rng.md Rule 5); broadcast as `S2CResolutionEvent` outcome | Server-side RNG + Combat Resolution |
| **Dé du Chateux** (1d6 + reveal-on-low) | Ecaflip | Spell | `roll = 1..=6`; deal `roll` damage; if `roll <= dé_chateux_reveal_threshold` (default 3), reveal the targeted-row enemy objective (unicast to Ecaflip player only — narrower than Sang Méprise) | Server-RNG + Combat Resolution + Objective System |
| **Miranda** (control transfer) | Ecaflip | APPEARANCE, persists while alive | Adjacent enemy units (geometric binding: units in lanes `own_lane − 1` and `own_lane + 1`, at any cell — i.e., the entire directly neighbouring lanes; see Combat Resolution GDD adjacency definition for the authoritative spec) transfer to Ecaflip player's control; on Miranda's death, units revert to original controller. Stolen tokens retain their `source_class` (no class-LEADER boost from new controller). | Combat Resolution / Board |
| **Sadida Seed** (passive cell hazard) | Sadida | Walk-over event — each seeded cell the unit's path traverses during sub-step 5 (intermediate + destination cells both trigger) | Friendly: `unit.AR += 1` (permanent until unit destroyed); enemy: 1 damage pre-AR (passes through AR pipeline). Seed persists; max 1 per cell. | Combat Resolution |
| **Graines de Folie** (Seeds → Madolls) | Sadida | Spell | For each board Seed: remove Seed, spawn Madoll on Seed's cell | Board / Lane System (spawn) |
| **Pollinisation** | Sadida | Spell | 3 damage to enemy units + place Seed on each cell where a unit died this resolution | Combat Resolution + Board |
| **Sacrifice Poupesque** | Sadida | Spell | Sacrifice all friendly Sadida_Token units; each deals 1 damage to enemy objective in its lane | Combat Resolution + Objective |

**Boundary rules:**
1. Class System never re-defines a base mechanic. It declares *which class-card invokes which base-system effect with which arguments*.
2. When a class effect needs RNG, it consumes from the appropriate Server-side RNG chain — never from a class-private RNG.
3. When a class effect mutates Economy (current_mana / reserve / gold), Economy is the writer. Class System provides the delta and target.
4. When a class effect mutates Objective state (HP, reveal, destruction), Objective System is the writer. Class System provides the trigger.
5. Class-specific keyword *parameters* (e.g., "INJURED: +2 ATK + FIRST STRIKE" — the specific +2/+FS values) are owned by the card data file (`cards.json` per the Card Data & Pool GDD), not by Class System or Keyword System. Class System defines the *semantic of "INJURED bonus is class-typical for Sacrier"*; the data file holds the values.

## Formulas

All formulas in this section consume inputs already defined by upstream systems (Economy, Combat Resolution, Server-side RNG, Objective System) and produce mutations in those systems. Class System owns only the formulas listed below; everything else (mana ramp, damage, RNG generation, objective destruction) lives in the upstream GDD. Each formula references its `cards.json` source(s) so implementers can trace value → effect.

### CS-1 — Gelure (Xelor: mana → reserve transfer)

The Gelure transfer formula is defined as:

`reserve_new = reserve + current_mana ;  current_mana_new = 0`

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `current_mana` | u32 | 0 – `mana_cap` (default 10, max 12) | Player current-round mana at cast time |
| `reserve` | u32 | 0 – unbounded | Player reserve before transfer |

**Output Range:** `reserve_new ∈ [reserve, reserve + mana_cap]`; `current_mana_new = 0` always.
**Example:** current_mana = 4, reserve = 2 → reserve = 6, current_mana = 0.
**Edge case:** current_mana = 0 → no-op transfer; the Gelure cast itself costs 0 so no waste penalty.

### CS-2 — Xelorium (Xelor: steal opponent current_mana)

The Xelorium steal formula is defined as:

`self.reserve_new = self.reserve + opponent.current_mana ;  opponent.current_mana_new = 0`

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `self.reserve` | u32 | 0 – unbounded | Xelor reserve before steal |
| `opponent.current_mana` | u32 | 0 – `mana_cap` | Opponent's remaining current mana at sub-step 1 |

**Output Range:** `self.reserve_new ∈ [self.reserve, self.reserve + mana_cap]`. Opponent reserve is **not** touched.
**Example:** self.reserve = 3, opponent.current_mana = 5 → self.reserve = 8, opponent.current_mana = 0.
**Timing:** Resolves at sub-step 1 (PLACEMENT commit), not at DRAFT-phase cast time. The opponent's current_mana is stolen as it stood when both players' placements arrive at the server. *(See Open Question OQ-Xelorium-timing for the alternative interpretation.)*
**Cost deduction order:** Xelorium's 4-mana cost is deducted from `self.current_mana` per Economy Rule 4 **before** the steal formula fires. The steal formula's input (`opponent.current_mana`) is computed on post-cost state. Implementers must not optimistically read `opponent.current_mana` before the cost deduction completes.
**Edge case:** Opponent already spent all current_mana before sub-step 1 → steal of 0; Xelorium's own cost (4 mana) is still paid by Xelor.

### CS-3 — Rollback (Xelor: reserve → friendly charge distance)

The Rollback charge-distribution formula is defined as:

```
n = self.reserve
self.reserve_new = 0
for each friendly Minion u in self.units:  -- Minion-type only; Structures and Traps are excluded
  destination(u) = clamp(u.cell + direction(u.owner) * n, 1, 8)
```

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `n` | u32 | 0 – unbounded (practical max ~15-20) | Reserve consumed; charge distance per unit |
| `u.cell` | u32 | 1 – 8 | Unit's cell pre-charge |
| `direction(owner)` | i32 | {+1, −1} | +1 for Player A, −1 for Player B |

**Output Range:** Per unit destination ∈ [1, 8] (clamped at board edges). Reserve always becomes 0.
**Example:** Xelor (Player A) reserve = 4. Three units at cells 2, 3, 5 → destinations 6, 7, 8 (last one clamped).
**Edge case (n = 0):** All units charge 0 cells; reserve stays 0; spell cost still paid. Client UI should warn before submission.
**Timing:** Rollback fires at sub-step 2 (CHARGE X bonus movement). Units placed this round with HASTE are eligible for Rollback's movement (HASTE removed summoning sickness; Rollback is movement, not action). Units with STUN do NOT charge — STUN suppresses sub-step 2.
**Strategic tradeoff (binding in Mummy-light games):** Spending reserve via Rollback in rounds 3+ forecloses Garde-Temps access in standard accumulation paths — recovering from 0 to 20 reserve in 2 remaining rounds via Miss Nuit + Gelure + Sablier alone (no Mummy) yields ~15–19 reserve, falling short of the 20-threshold. This is Xelor's central strategic decision: commit to Rollback for immediate board movement, or hold reserve for Garde-Temps later. Both are valid play-lines; neither is a mistake. Players must make this choice consciously by round 3. **Mummy caveat:** In games with 2+ active Mummies, passive reserve from combat damage (+1 per hit, uncapped) can bridge the gap after a round-3 Rollback drain. Mummy's HP=2 is the de facto cap — tokens that die quickly in combat limit their reserve contribution naturally. Verify the tradeoff holds in high-Mummy-density games during M2 playtesting; adjust `garde_temps_reserve_cost` if both Rollback and Garde-Temps become trivially achievable in the same game.

### CS-4 — Garde-Temps (Xelor: 20-reserve gate, destroy enemy objective)

The Garde-Temps reserve-gate formula is defined as:

```
target_valid = (chosen_enemy_objective.is_alive = true)
playable = (self.reserve >= garde_temps_cost) AND target_valid
        AND (self.garde_temps_used_this_game < garde_temps_per_game_cap)
if playable:
  self.reserve_new = self.reserve - garde_temps_cost
  self.garde_temps_used_this_game += 1
  take_damage(lane=chosen_enemy_objective.lane, attacker=self, amount=chosen_enemy_objective.hp)
  -- lethal damage; `chosen_enemy_objective.hp` = GameConfig.objective_hp (max HP constant, see variable table); full consequence path fires via objective-system.md Rule 7
else:
  reject_play  -- reserve/mana untouched; reject also if target already HP=0 or cap reached
```

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `self.reserve` | u32 | 0 – unbounded | Player reserve at play attempt |
| `garde_temps_cost` | u32 | 20 (GameConfig knob) | Reserve cost; replaces standard mana cost path |
| `garde_temps_per_game_cap` | u32 | 1 (GameConfig knob) | Max number of Garde-Temps activations per game per player |
| `self.garde_temps_used_this_game` | u32 | 0 – cap | Counter incremented on each successful activation; persisted in session state |
| `playable` | bool | {false, true} | Server-side acceptance gate |
| `chosen_enemy_objective.hp` | u32 | `GameConfig.objective_hp` (default 5) | **Max HP constant from GameConfig — NOT the objective's current HP.** Always equals the max-HP constant regardless of how much damage the objective has already taken. This makes the call always lethal. Consistent with objective-system.md Edge Cases: "Treated as `take_damage(lane, attacker_player, objective_hp)`". |

**Output Range:** On accept, `self.reserve_new = self.reserve − 20` (≥ 0 by gate); one enemy objective receives lethal `take_damage` (routes through objective-system.md Rule 7 consequence path — gold award, fake reward, real-objectives counter, and loss condition). On reject, no state change.
**Example:** reserve = 23, garde_temps_used_this_game = 0 → playable = true → reserve_new = 3; objective take_damage fires.
**Interface alignment:** Uses `take_damage()` not `destroy()` — consistent with objective-system.md Edge Cases ("If Garde-Temps targets an objective: Treated as `take_damage(lane, attacker_player, objective_hp)`) and with Punition below.
**Note:** Garde-Temps' card-data `mana_cost` field is 0 (or absent); Economy System's "from reserve" path (Rule 4) is the only valid payment route. Server validates `reserve >= 20` BEFORE accepting the placement.
**Counter ownership:** `self.garde_temps_used_this_game` is owned by **Game Session System**, initialized to 0 at `on_lobby_to_draft_initial` transition, and persists for the full game session. It is never reset mid-game. The registry formula for `garde_temps_gate` MUST include the `garde_temps_used_this_game < garde_temps_per_game_cap` guard — this is a required enforcement check, not optional.
**Strategic context:** Garde-Temps is the payoff for the *hold-reserve* line. A Xelor player who has spent reserve via Rollback cannot realistically reach 20 reserve again in a standard 5-round game. The 20-reserve threshold at the default setting is calibrated to this tradeoff — see CS-3 strategic tradeoff note.

### CS-5 — Sang Méprise (Sacrier: full objective reveal)

The Sang Méprise reveal-set formula is defined as:

```
reveal_set = { o | o ∈ all_objective_slots(Player_A) ∪ all_objective_slots(Player_B), o.is_alive = true }
for each player P in {Player_A, Player_B}:
  unicast(P, reveal_set)  -- includes both players' alive objectives
```

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `all_objective_slots(P)` | set of 5 slots | exactly 5 per player | All real + fake slots for player P |
| `o.is_alive` | bool | {false, true} | Slot HP > 0 |
| `reveal_set` size | u32 | 0 – 10 | Alive slots across both players |

**Output Range:** Each alive slot's `is_fake` boolean is unicast to both players; client stores for current RESOLUTION only; cleared at RESOLUTION end.
**Example (Round 5):** Player A has 3 alive (1 still-fake); Player B has 4 alive (2 still-fake). The `reveal_set` contains 7 entries (all alive objective slots from both players). Sang Méprise unicasts this full 7-slot `reveal_set` to **both** players via separate unicasts — Player A receives it, Player B receives it. Each player now knows every alive objective's `is_fake` status for the current RESOLUTION. Already-destroyed slots are excluded — they were revealed at destruction. *(Note: the formula is authoritative; prior session text suggesting a directional per-player vector was an error.)*
**Reconnect gap:** `S2CSangMepriseReveal` is not in `S2CGameSnapshot`. A player reconnecting mid-RESOLUTION while Sang Méprise is active will not receive the reveal. → flagged as Open Question OQ-NP-snapshot.

### CS-6 — Punition (Sacrier: self-destroy + AOE)

The Punition self-destroy + damage formula is defined as:

```
has_eligible_real = ( count({o ∈ self.objectives | o.is_real ∧ o.is_alive}) >= 1 )
if has_eligible_real:
  take_damage(lane=chosen_real_objective.lane, attacker=self, amount=chosen_real_objective.hp)
  -- lethal self-damage; full consequence path fires via objective-system.md Rule 7
  for each o ∈ opponent.objectives where o.is_alive:
    take_damage(lane=o.lane, attacker=self, amount=3)
else:
  reject_play
```

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `has_eligible_real` | bool | {false, true} | Sacrier has ≥ 1 alive real objective to sacrifice |
| `chosen_real_objective` | slot ref | exactly 1 | Player-selected; must be real and alive |
| `opponent.alive_objectives` count | u32 | 0 – 5 | Damage targets |

**Output Range:** 3 damage to EACH alive opponent objective (e.g., 4 alive → 12 total damage spread, not concentrated).
**Loss condition interaction:** Sacrificing a real decrements Sacrier's alive-real count. If post-sacrifice `real_objectives_destroyed(self) >= 3`, Punition can self-eliminate the Sacrier — RSM declares game over. Document this explicitly so it is not treated as a bug.
**Edge case:** `has_eligible_real = false` → server rejects play; mana not consumed.

### CS-7 — Sadida Seed (passive cell-hazard)

The Seed walk-over formula is defined per direction. **Walk-over definition (binding):** A Seed triggers for every cell the unit's movement path traverses during sub-step 5 — including intermediate cells and the final landing cell. A unit with MP=3 moving over 2 seeded intermediate cells before stopping on a third seeded cell triggers 3 Seeds.

**Friendly walks over Seed:** `unit.AR_new = unit.AR + 1` (permanent until unit destroyed); seed persists.

**Enemy walks over Seed:** `damage_to_unit = 1` (then routed through Combat Resolution's damage pipeline; AR applies); seed persists.

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `unit.AR` | u32 | 0 – unbounded | Unit armor before seed |
| `damage_to_unit` | u32 | 1 (pre-AR) | Damage dealt to enemy walker |

**Output Range:** Friendly AR +1 per walk-over (a unit walking over multiple seeds in one movement gains +1 each — but `max 1 seed per cell` caps any single cell to one bonus). Enemy 1 damage pre-AR.
**Counter:** The **ARMOR-PIERCING** keyword (defined in `keyword-system.md`) treats the defender's AR as 0 for that attacker, effectively dealing full damage regardless of AR value. ARMOR-PIERCING cards are the intended design counter to high-AR Sadida units. There is no numeric AR cap applied to units; ARMOR-PIERCING is the design ceiling. Pre-implementation gate: verify ARMOR-PIERCING cards are available in the relevant class pools before Class System stories begin.
**Stacking rule:** **Max 1 seed per cell.** Attempting to place a second seed on an occupied cell discards the new placement; client UI tooltip informs the player.
**Lifetime:** Seeds persist indefinitely. Removed only by: (a) explicit consume/convert (Graines de Folie, etc.), or (b) game session end. Seeds on cells with no walk-over remain dormant board hazards.

### CS-8 — Madoll Spawn from Graines de Folie

The seeds-to-Madolls conversion formula is defined as:

```
for each seed S in self.seeds_on_board:
  remove(S)
  spawn_unit(token=Madoll, cell=S.cell, lane=S.lane, owner=self)
```

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `S.cell` | u32 | 1 – 4 (Sadida side only) | Cell of the seed |
| `S.lane` | u32 | 1 – 5 | Lane of the seed |
| `Madoll` stats | unit | HP=3 / ATK=1 / MP=3 | Per Krosmaga reference |

**Output Range:** One Madoll per board seed; spawn at exact seed cell.
**Edge case (cell occupied / lane at unit cap):** Board / Lane System rejects over-capacity spawn; that seed's Madoll is skipped (seed is still consumed). Logged as a board-warning.

### CS-9 — Ecaflip 1d6 with Reveal-On-Low (Dé du Chateux)

The Dé du Chateux roll-and-reveal formula is defined as:

```
seed = server_rng.next(RESOLUTION_chain, this_trigger_index)
roll = uniform(seed, 1..=6)
damage = roll
reveal = (roll <= dé_chateux_reveal_threshold)  -- default 3
```

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `roll` | u32 | 1 – 6 | Uniform die result |
| `dé_chateux_reveal_threshold` | u32 | 1 – 6 (GameConfig) | Roll ≤ threshold → reveal |
| `damage` | u32 | 1 – 6 | Damage to target |
| `reveal` | bool | {false, true} | Whether targeted-row enemy objective is revealed |

**Output Range:** damage ∈ [1, 6]; reveal = true on rolls {1, 2, 3} (50 % at default).
**Target:** The unit or objective selected by the Ecaflip player at card-play time. Target type (unit vs. objective vs. player-choice) is specified in `cards.json` for this card. Damage routes through the standard `take_damage(target, roll)` pipeline: for units, AR reduction applies; for objectives, HP reduction via `objective-system.md` Rule 9.
**Example:** roll = 2 → damage = 2 applied to target; reveal = true → enemy objective in target lane revealed (unicast to Ecaflip player only — narrower than Sang Méprise).
**RNG sourcing:** Consumed from the RESOLUTION chain per server-rng.md Rule 5; ordering ascending player_id → lane → trigger_index_within_card.

### CS-10 — Ecaflip Coin Flip (Chatar / Shava Shavien / Craps)

The coin-flip formula is defined as:

```
seed = server_rng.next(RESOLUTION_chain, this_trigger_index)
flip = uniform(seed, 0..=1)  -- 0 = outcome_A, 1 = outcome_B
```

**Per-card outcome table:**

| Card | flip = 0 (A) | flip = 1 (B) |
|---|---|---|
| Chatar (APPEARANCE) | unit gains +2 ATK | unit suffers 2 self-damage |
| Shava Shavien (DEATH) | card returns to **owner's** hand | card returns to **opponent's** hand |
| Craps (Krosmic spell) | 8 damage spread among alive opponent objectives | 4 damage spread among alive opponent objectives |

**Design note (Shava Shavien):** flip=tails returning to the *opponent's* hand is intentional extreme-variance design — Shava Shavien is Ecaflip's highest-risk card. A tails result gifts the opponent a Krosmic card from Ecaflip's class. This is not a bug. Deploying Shava Shavien is the authored risk; the player who plays it chose to accept this downside. No mitigation mechanic exists by design. Documenting explicitly so implementers do not treat it as an error.

**Craps damage distribution sub-formula (equal-share):**

```
alive = count(opponent.alive_objectives)
share = floor(total / alive)
remainder = total mod alive
-- distribute remainder to first `remainder` lanes in ascending lane order
```

**Example (Craps heads, 3 alive opponents):** total = 8, alive = 3 → share = 2, remainder = 2 → Lanes 1 and 2 receive 3 damage; Lane 3 receives 2.
**Edge case (alive = 0):** Game already over; Craps is no-op. **Implementation guard:** Server MUST evaluate `if alive == 0 { return no_op; }` **before** computing `share = total / alive`. Use `checked_div` or an explicit early-return to prevent integer division-by-zero panic in Rust.
**alive = 1:** `share = floor(total / 1) = total`, `remainder = 0`. All damage concentrates on the last objective (8 or 4 HP from Craps). If this exceeds the objective's remaining HP, the objective is destroyed via the standard `take_damage` path — this is the intended win-condition case.

### CS-11 — Miss Nuit Reserve Trigger (per-round cap)

The Miss Nuit reserve trigger formula is defined as:

```
-- Fires on each opponent_card_played_event during PLACEMENT commit (sub-step 1)
if Miss_Nuit.is_alive AND NOT Miss_Nuit.is_silenced AND miss_nuit_reserve_gained_this_round < miss_nuit_cap:
  self.reserve += 1
  miss_nuit_reserve_gained_this_round += 1
```

**Variables:**

| Variable | Type | Range | Description |
|---|---|---|---|
| `miss_nuit_reserve_gained_this_round` | u32 | 0 – `miss_nuit_cap` | Reserve actually granted this round (tracks grant count, not raw card-play count — these are equivalent under current rules but semantically distinct) |
| `miss_nuit_cap` | u32 | 2 (GameConfig) | Per-round Miss Nuit reserve gain ceiling |

**Output Range:** Reserve gain ∈ [0, +2] per round.
**"Plays a card" qualifier (binding):**
- **Counts:** Spell cast or minion summon committed at PLACEMENT sub-step 1 (the card was in opponent's hand and is now leaving it).
- **Does NOT count:** Token spawns (Mummy, Madoll, Bow Meow); free card grants from Lane 3 prism; Drheller-style triggered draws.
- **Does NOT count:** Xelor's own card plays — only **opponent** plays trigger Miss Nuit.
**Edge case (Miss Nuit silenced or destroyed mid-round):** Subsequent opponent plays in the same round do not trigger; the trigger is gated on Miss Nuit being alive AND not silenced at the moment opponent's card commits.

### CS-12 — Cra RANGE Mechanic (class anchor, cross-reference)

Cra's class rhythm — "Long and thin — fires from outside trade range. Range is the rhythm." — is mechanically anchored in the **RANGE keyword** and its attack resolution at **Combat Resolution sub-step 6**. Unlike Xelor (CS-1 through CS-4) or Sadida (CS-7/8), Cra has no unique formula beyond what RANGE defines — the class rhythm *is* the RANGE mechanic applied systematically.

**Cra cards feeding the RANGE mechanic:**

| Card | Class contribution |
|---|---|
| *Criblage* | RANGE damage to targets across columns |
| *Harcèlement* | Persistent RANGE damage per RESOLUTION while in play |
| *Flèche Destructrice* | Single-shot RANGE with conditional multiplier |
| *Guy Yomtella* | RANGE unit — attacks from outside melee reach |
| *Lucy Fayre* | RANGE + FIRST STRIKE unit — fires before melee exchange |

**Formula reference:** The RANGE attack formula lives in `combat-resolution.md` sub-step 6: RANGE attacks fire at cell-distance ≥ 2, before melee contact (sub-step 3). Cra's tempo signature is the *sub-step ordering advantage* — RANGE attacks create board pressure before opponents can engage in melee. The Cra player's rhythm is timed around maintaining units in the 3–5 cell range corridor while denying opponent advance.

**Cross-reference:** `keyword-system.md` — RANGE keyword definition and stacking rules. `combat-resolution.md` sub-step 6 — RANGE attack resolution formula (authoritative). This Class System section is the class-identity anchor; the mechanics live in those documents.

**Binding tempo rule (Cra):** Cra's rhythm requires maintaining ≥ 1 RANGE unit in play to operate the range-control tempo. A Cra player with zero RANGE units on the board has lost their class mechanism — they are playing only from neutral cards. No special subsystem beyond RANGE keyword resolution is needed; the class identity is expressed through card selection.

**Melee-push ruling:** If a Cra RANGE unit is pushed into melee contact with an opponent unit (cell distance < 2) by any effect (ATTRACT, Fulgurance, Rollback, etc.), the RANGE unit loses its RANGE attack eligibility for that sub-step and participates in normal melee resolution at sub-step 3. RANGE attack eligibility is recalculated at each sub-step boundary based on current cell distance. This is handled entirely by combat-resolution.md sub-step 6 pre-check; no special Cra-specific logic is required.

### CS-13 — Iop Class Scope (Declarative)

Iop's tempo signature — "aggro rush — short, loud, front-loaded" — is delivered through its card library (Authority, Heure de Gloire, Felida, Appel à la Baston, Katsu Mi). These cards use base mechanics already owned by Combat Resolution (CHARGE, FIRST STRIKE, BODYGUARD) and Keyword System. No Iop-specific formula is owned by the Class System GDD. Implementers building Iop mechanics read `cards.json` for card-by-card effect definitions.

Class System's contribution for Iop: (1) class identity and tempo signature (Section B/C above), (2) shop-slot class filtering to Iop's 24-card library, (3) CS-AC-38 below. Iop is not a design gap — it is a class whose mechanics are fully composable from the base systems without requiring class-specific formula ownership.

---

## Edge Cases

Edge cases are grouped by category. Each entry follows the form **If [condition] → [exact resolution]. [Rationale]**. Trivial edges (current_mana=0 transfers, reserve=0 charges, target-already-destroyed no-ops) are documented inline in Section D formulas and not duplicated here.

#### Concurrency & simultaneity

- **If both players play Sang Méprise in the same RESOLUTION** → Both submissions land at sub-step 1. The first reveal sets `sang_meprise_active = true` and unicasts the full alive-objectives vector to both players. The second is idempotent: server still pays the spell cost but issues no second reveal (state already set; client silently ignores). Reveal state is set-once-per-RESOLUTION; double-firing has no additional information yield.
- **If Xelor plays Xelorium AND Gelure in the same PLACEMENT batch** → Both resolve at sub-step 1 in ascending trigger_index order. Xelorium fires first (reserve gains opponent.current_mana; opponent.current_mana := 0), then Gelure fires (reserve gains self residual current_mana; self.current_mana := 0). Both cards' mana costs are paid before effects fire. Legal high-burst combo, not a bug. **Burst ceiling (by design):** `reserve_gain = opponent.current_mana + (self.current_mana − Xelorium_cost)`. Example at mana_cap=10, Xelorium cost=4, opponent at full mana: `10 + (10 − 4) = +16 reserve` in one round. Round-2 Garde-Temps is achievable via this combo if both cards are in hand simultaneously, Xelor pays the 4-mana cost, and the opponent has spent 0 mana before sub-step 1 — a converging set of conditions. This burst ceiling is intended; it is Xelor's highest-requirement, highest-yield play. Balancers should monitor whether the combo trivializes the Rollback↔Garde-Temps tradeoff in M2 playtesting.
- **If two opponent cards commit in the same sub-step batch and one of them SILENCEs Miss Nuit** → Cards committing before the SILENCE trigger Miss Nuit's reserve gain (subject to `miss_nuit_cap`); cards committing after the SILENCE do not. Reserve already awarded before the SILENCE lands is **not** retroactively revoked. Ordering is ascending trigger_index within the batch — deterministic, no rollback semantics.
- **If Punition simultaneously destroys the opponent's last alive real objective AND the Sacrier's own loss condition is met in the same sub-step** → Both players reach `real_objectives_destroyed >= 3` in sub-step 1. RSM evaluates both loss conditions simultaneously. Per RSM's mutual-destruction rule, the result is a **Draw** — not a Sacrier loss. This is the only scenario where Punition produces a Draw rather than a Sacrier loss or an opponent destruction.

#### Token transfer (Miranda + class-counted effects)

- **If Miranda steals a Sadida unit that previously placed Seeds** → Existing Seeds remain on the board (Seeds are board-state, never unit-attached). If the Ecaflip-controlled Sadida unit walks over a Seed, the walk-over resolves against the **current controller** (Ecaflip) — the unit is "friendly" for that walk and gains +1 AR. Seed trigger checks controller, not source_class.
- **If Sacrifice Poupesque is cast while Miranda controls a Sadida_Token** → Filter is `controller = self AND source_class = Sadida_Token`. Miranda-stolen tokens have `controller = Ecaflip` and are excluded. Both conditions are conjunctive; stolen Madolls do not sacrifice and do not deal damage.
- **If Criblage (Cra) counts "each Cra in play" and Miranda has stolen a Cra unit** → Count uses `source_class = Cra AND controller = self`. Stolen Cra units are excluded from the original Cra player's count. Ecaflip player gets no benefit either (LEADER/family checks exclude stolen tokens for the new controller).

#### SILENCE/STUN on class triggers

- **If Sinistro is targeted by SILENCE** → Server rejects as InvalidTarget. SILENCE is a unit-only status (Keyword System); Sinistro is a spell-attached entity on a friendly objective, not a unit. To remove Sinistro, the opponent must damage or destroy its parent objective.
- **If Jet le Pied Volant is SILENCEd mid-RESOLUTION** → Charges already triggered before SILENCE landed are not reversed. Damage to friendly units/objectives occurring after SILENCE does not trigger further charges. SILENCE wear-off (end-of-round keyword reset) restores the trigger next round.
- **If Sinistro's parent objective is destroyed (HP → 0)** → Sinistro is destroyed in the same Rule 7 consequence path (objective-system.md), before the RESOLUTION-end sync. No RESOLUTION-end damage fires from a Sinistro that is destroyed this RESOLUTION — the end-of-RESOLUTION trigger check must gate on Sinistro still being alive. The client removes the Sinistro indicator as part of the objective-destruction animation (triggered by `SinistroDestroyed` event — see NP-6). Objective System Rule 7 must also include a Sinistro cleanup step when the parent objective is destroyed.

#### Reserve-math edges

- **If Xelor reserve exceeds 8 (Rollback's effective board ceiling)** → Reserve has no cap. Rollback charges units by `reserve` cells, clamped at [1, 8]. Once reserve ≥ 8, additional accumulation is wasted for Rollback but still useful for Garde-Temps (cost = 20). No soft cap or warning imposed.
- **If Sablier is cast while opponent.reserve = 0** → `opponent.reserve = saturating_sub(0, 1) = 0`; `self.reserve += 1`. Xelor gains +1 even when the opponent has nothing to swap from. Asymmetric by design — the gain is the mechanic.
- **If Garde-Temps reserve gate fails (reserve < 20)** → Server rejects the placement; spell cost is **not** deducted (validation precedes deduction, per Economy Rule 4). Implementers must not deduct optimistically.
- **If Garde-Temps targets an already-destroyed (HP=0) objective** → `target_valid = false`; server rejects. The 20 reserve is NOT deducted. A player who selects a destroyed lane as target receives a rejection without penalty. Client UI should grey out already-destroyed lanes as invalid targets.
- **If a Sadida unit with high MP traverses a fully-seeded lane (max seeds per cell)** → The unit gains +1 AR per seeded cell traversed. With all 4 cells on the Sadida player's side seeded and unit MP ≥ 4, a unit gains +4 AR from a single movement. This is the intended maximum from seed density. The `seed_ar_bonus` knob controls per-seed gain; actual AR accumulation depends on seed density × unit MP. Multiple seeds in one path are intended to stack. **The design counter to high-AR units is the ARMOR-PIERCING keyword** (see CS-7 counter note and `keyword-system.md`) — ARMOR-PIERCING treats the defender's AR as 0 for that attacker, restoring full damage. No numeric AR cap is applied; ARMOR-PIERCING is the design ceiling. Pre-implementation gate: verify ARMOR-PIERCING cards are available in the relevant card pools before Class System stories begin.

#### Card-data and cross-class legality

- **If a card has `card_class = null` or no class field in cards.json** → Treated as Neutral. May appear in any class shop's neutral-slot pool. Card-data validation tests must assert all 148 class cards carry correct class tags; runtime path trusts the declared field without re-validating intent.
- **If a player holds a card from another class and plays it** (via Drheller draw, Shava Shavien return, or data bug) → Server accepts the play. Class System imposes **no runtime gate** on `card_class ≠ player.class`. Once a card is in a player's hand by any means, it is playable. Consistent with the Drheller cross-class exception and Shava Shavien's design intent.

#### Disconnect during class-active effects

- **If Xelor disconnects after submitting a placement that includes Rollback** → C2SSubmitPlacement was already received; disconnect after submission does not cancel committed placements. Rollback fires at sub-step 2 as submitted. Grace timer (30 s) starts; reconnecting player receives snapshot + missed S2CResolutionEvents.
- **If Sacrier disconnects mid-RESOLUTION while Sang Méprise is active** → Reveal already broadcast. On reconnect, `S2CGameSnapshot.active_sang_meprise_reveals` (if non-None) carries the full reveal set; the client rebuilds reveal state from the snapshot field and does NOT require a second `S2CSangMepriseReveal` unicast. See NP D.1 `active_sang_meprise_reveals` field. This closes OQ-NP-snapshot (formerly OQ6 in network-protocol.md).

## Dependencies

| Direction | System | Interface | Hard/Soft |
|---|---|---|---|
| Upstream | **Card Data & Pool** (`card-data-pool.md`) | Class card library (148 cards across 6 classes); `card_class` field in cards.json; `draw_class_card(player_class)` API; uniform `draw_random` for prism/Drheller draws | Hard |
| Upstream | **Card Acquisition** (`card-acquisition.md`) | 50/50 class-vs-neutral shop slot roll; auction neutral-only constraint; refresh policy | Hard — shop generation mechanics belong to Card Acquisition; Class System only defines what qualifies as a class card |
| Upstream | **Economy System** (`economy-system.md`) | `current_mana`, `reserve_mana` mutation interface; Economy Rule 4 "from reserve" path; `miss_nuit_cards_played_this_round` tracker (owned by Economy) | Hard |
| Upstream | **Game Session System** (`game-session-system.md`) | LOBBY → DRAFT_INITIAL transition; class Ready gate; `player.class` session field storage | Hard |
| Upstream | **Round State Machine** (`round-state-machine.md`) | Phase sequence (LOBBY, DRAFT_INITIAL, DRAFT, PLACEMENT, RESOLUTION); RSM declares game over on Punition self-elimination | Hard |
| Upstream | **Combat Resolution** (`combat-resolution.md`) | Sub-step 1/2/3/6 execution; Fulgurance, Jet le Pied Volant, Pollinisation, Sacrifice Poupesque effect application | Hard |
| Upstream | **Objective System** (`objective-system.md`) | `take_damage` interface; `S2CSangMepriseReveal` unicast channel; Punition self-destroy → loss condition; `sang_meprise_active` flag | Hard |
| Upstream | **Keyword System** (`keyword-system.md`) | SILENCE, STUN, INJURED, HASTE, FIRST STRIKE, CHARGE X, COUNTERATTACK base definitions; INJURED re-evaluation timing | Soft — class effects extend, never redefine |
| Upstream | **Server-side RNG** (`server-rng.md`) | RESOLUTION RNG chain (Rule 5) for all Ecaflip dice and coin flips; `draw_random` for uniform draws | Hard — no class-private RNG |
| Upstream | **Network Protocol** (`network-protocol.md`) | `C2SClassChoice`, `S2CGameSnapshot` (class field), `S2CSangMepriseReveal`, `S2CResolutionEvent` roll outcomes; reconnect snapshot gap | Hard |
| Downstream | **Card Animations** (`card-animations.md`) | Krosmic play reveals; token spawn animations (Mummy, Chacha Noir, Seed, Madoll); Rollback mass-charge; Sang Méprise reveal flash | Soft — Class System does not depend on animations |
| Peer | **Prism System** (`prism-system.md`) | Lane 2/4 prism +1 reserve (all classes); Lane 3 draw (uniform pool); Patek Tag (Xelor) destroys a prism | Soft |

## Tuning Knobs

All knobs live in `assets/config/game_config.ron` and map to `GameConfig` fields. Knobs owned by upstream systems are referenced, not redefined here.

| Knob | GameConfig field | Default | Safe range | Too low | Too high |
|---|---|---|---|---|---|
| Garde-Temps reserve cost | `garde_temps_reserve_cost` | 20 | 10 – 30 | Objective destruction cheapened; Xelor bomb too accessible | Unreachable in normal play; Xelor loses late-game win condition |
| Garde-Temps per-game use cap | `garde_temps_per_game_cap` | 1 | 1 – 2 | At 1: Xelor gets one deterministic swing; combined with reserve economy this is sufficient to close a game once. At 2: two objectives can be destroyed by reserve alone — dominant strategy risk if Mummy passive cap is also absent. Raise to 2 only after M2 confirms reserve accumulation is manageable. | N/A (minimum is 1 — 0 disables the card entirely) |
| Miss Nuit per-round cap | `miss_nuit_cap` | 2 | 1 – 4 | Reserve gain too slow vs. active opponents | Mass-token opponents (Chafer, Bow Meow) flood Xelor's reserve; Rollback trivially recharged each round |
| Dé du Chateux reveal threshold | `dé_chateux_reveal_threshold` | 3 (≈50 % chance) | 1 – 5 | Objective reveal almost never fires; low-roll dice spell feels punishing with no upside | Near-guaranteed reveal every cast; Ecaflip gains too much information advantage |
| Seed AR bonus per walk-over | `seed_ar_bonus` | 1 | 1 – 2 | (minimum is 1 — below that removes the mechanic) | +2 per seed → extreme AR stacking mid-game; Sadida units gain very high AR by round 6 (counter: PIERCE keyword cards — see CS-7 and keyword-system.md) |
| Seed damage to enemy walkers | `seed_enemy_damage` | 1 | 1 – 2 | (minimum is 1) | 2 per step = too punishing for aggressive classes; rush strategies against Sadida become non-viable |

**Upstream knobs referenced (not owned here):**
- `mana_cap` default 10, max 12 — Economy System
- `reserve` no cap — Economy System (design decision)
- `shop_class_ratio` 50 % class / 50 % neutral — Card Acquisition

**Design rule (not a numeric knob):** Miss Nuit trigger scope = Spell + Minion cards played from hand only. Tokens, prism grants, and triggered draws excluded. Changing this is a design decision, not a config change.

## Visual/Audio Requirements

*Not in scope for this GDD. Presentation concerns (class card play reveals, token spawn animations, Rollback mass-charge, Sang Méprise reveal flash) are owned by Card Animations GDD.*

## UI Requirements

**Class picker (LOBBY):**
- Player selects from 6 classes before clicking Ready; each class shows its one-line tempo signature and 4 signature Krosmic card names. Hovering a card name should surface a tooltip with the card's effect summary — 4 names without context are insufficient for new players to make an informed class choice.
- Intermediate state: after Player A clicks Ready (one player committed, other not yet), Player A's class is locked and displayed to them with a visual lock indicator; Player B's slot shows "Waiting for opponent…". The opponent's locked class is revealed to both players simultaneously only when both have committed.
- Class cannot be changed after Ready is clicked; UI must disable the picker post-lock.
- Exception: if a player retracts their Ready (RSM ready-retractable rule — allowed before both players commit), the class picker re-enables for that player. The picker re-disables when they click Ready again.

**Reserve display (all classes):**
- Reserve mana is always visible in the HUD alongside current mana (per HUD GDD). All six classes see their reserve — reserve is not Xelor-exclusive.
- **Outside RESOLUTION:** Reserve value updates immediately on any mutation (Gelure, prism reward, Miss Nuit gain).
- **During RESOLUTION:** Reserve counter reflects the post-RESOLUTION state sync when the animation batch completes. Real-time mid-RESOLUTION reserve updates (Xelorium steal at sub-step 1, Rollback drain at sub-step 2, Mummy passive gains) are contingent on NP-5 being resolved. Until NP-5 closes, these values update only after the full RESOLUTION animation concludes.

**Garde-Temps gate feedback:**
- If player attempts to submit a placement containing Garde-Temps with `reserve < 20`, the card should be visually marked as unplayable and the submission blocked at the client side before sending to server.
- Reserve-insufficient must use a **visually distinct treatment from mana-insufficient** (e.g., different icon color, "R" indicator prefix). Using identical visuals for two different failure reasons teaches the wrong mental model about the reserve vs. mana distinction. Coordinate with Hand UI GDD to ensure both insufficient-mana and insufficient-reserve states are defined as a coherent system.
- After the once-per-game cap is consumed (`garde_temps_used_this_game = garde_temps_per_game_cap`), Garde-Temps shows a **permanently exhausted state** distinct from the temporary reserve-insufficient state. Tooltip: "Already used this game." This state persists for the remainder of the session.
- Server-side rejection is the authoritative gate; client-side feedback is ADVISORY but expected for UX.

**Rollback n=0 warning:**
- If Xelor player stages Rollback with `reserve = 0`, display an **inline warning in the staged-card area** (visible without hover): "No reserve — Rollback will have no effect." Player may still submit; it is not blocked. A hover tooltip is insufficient here — players staging under time pressure do not hover, so the warning must be visible in the staging area without requiring interaction.

**Xelorium drain feedback:**
- When Xelorium resolves and zeroes the opponent's current mana, the opponent's mana counter animates from its current value to 0 with a brief visual flash or glow marking the steal event.
- The Xelor player's reserve counter increments by the stolen amount. Update timing subject to NP-5 (see Reserve display above).
- This feedback is required — a mana pool silently hitting 0 mid-RESOLUTION with no visual signal is a readability failure for the opponent.

**Sinistro attachment display:**
- Sinistro's presence on a friendly objective is displayed as a persistent spell-indicator icon on the objective tile, visible to both players throughout the game (not hidden information).
- The icon is removed when Sinistro is destroyed (parent objective takes damage).
- The indicator must be visually distinct from objective HP displays, fake/real indicators, and other objective decorations.

**Sang Méprise reveal overlay:**
- When `S2CSangMepriseReveal` is received, the board displays real/fake indicators on all 10 objective slots for the duration of the RESOLUTION.
- Overlay is cleared automatically at RESOLUTION end. No player action required to dismiss.
- Reconnect gap: if a player reconnects mid-RESOLUTION after Sang Méprise has fired, the overlay is absent. Client must render gracefully without the overlay (objectives appear as unknown state).

**📌 UX Flag — Class System:** This system has UI requirements. Run `/ux-design class-picker` and `/ux-design reserve-hud` in Pre-Production Phase 4 before writing epics that cover class selection and mana display. Stories referencing class UI should cite `design/ux/class-picker.md` and `design/ux/hud.md`, not this GDD directly.

## Acceptance Criteria

All criteria are BLOCKING unless marked ADVISORY. Format: GIVEN [state] / WHEN [trigger] / THEN [measurable outcome].

#### Class lifecycle

**CS-AC-01** GIVEN a lobby with two players, WHEN Player A selects Xelor and clicks Ready, THEN Player A's `class` field is locked to `Xelor` on the server and subsequent class-change messages from Player A are rejected.

**CS-AC-02** GIVEN both players have locked their class, WHEN the RSM transitions LOBBY → DRAFT_INITIAL, THEN every active player's `class` field is `Some(C)` — no player may have `class = None`.

**CS-AC-03a** GIVEN both players have locked their classes and the RSM has transitioned to DRAFT_INITIAL, WHEN any player receives `S2CGameSnapshot`, THEN the `PlayerSnapshot` for each player contains a `class_id` field equal to that player's locked class.

**CS-AC-03b** (ADVISORY) GIVEN both players' classes are locked and the game is in progress, THEN the opponent's class name is rendered in the game UI header for the duration of the game. (UI-layer assertion — verify in manual walkthrough, not headless unit test.)

#### Xelor reserve formulas

**CS-AC-04** GIVEN Xelor player with `current_mana=5` and `reserve=2`, WHEN Gelure is played, THEN `current_mana=0` and `reserve=7`.

**CS-AC-05** GIVEN Xelor player with `current_mana=8, reserve=3` and opponent with `current_mana=6, reserve=8`, WHEN Xelorium (cost=4 mana, deducted first per Economy Rule 4) resolves at RESOLUTION sub-step 1, THEN `Xelor.current_mana=4`, `Xelor.reserve=9`, `opponent.current_mana=0`, and `opponent.reserve=8` (unchanged).

**CS-AC-05b** GIVEN Xelor player with `current_mana=4` (exactly the cost of Xelorium) and opponent with `current_mana=6`, WHEN Xelorium is played, THEN `Xelor.current_mana=0` (cost deducted), `Xelor.reserve` increases by 6 (the steal receives post-cost `opponent.current_mana=6`), and the play is not rejected as insufficient-mana (exact-cost payment is valid).

**CS-AC-06** GIVEN Xelor player with `reserve=4` and three friendly units at cells 2, 3, 5 on a board of cells [1–8] where Player A advances in the +1 direction, WHEN Rollback resolves, THEN `reserve=0` and units land at cells 6, 7, 8 (`clamp(2+4)=6`, `clamp(3+4)=7`, `clamp(5+4)=8`).

**CS-AC-07** GIVEN Xelor player with `current_mana=N` (N ≥ Rollback's mana cost per `cards.json`), `reserve=0`, WHEN Rollback is played, THEN `reserve=0`, all friendly units advance 0 cells, and `current_mana = N − Rollback_mana_cost` (mana cost deducted normally; zero-reserve cast is valid and not rejected).

**CS-AC-08** GIVEN Xelor player with `reserve=5`, one healthy unit at cell 2 and one STUNned unit at cell 4, WHEN Rollback resolves, THEN the healthy unit moves to cell 7; the STUNned unit does not move; `reserve=0`.

**CS-AC-09** GIVEN Xelor player with `reserve=15` (below `garde_temps_reserve_cost=20`), WHEN Garde-Temps play is submitted, THEN server rejects with insufficient-reserve error; no mana deducted; `reserve=15` unchanged.

**CS-AC-10** GIVEN Xelor player with `reserve=22`, WHEN Garde-Temps is accepted, THEN `reserve=2` and the chosen enemy objective HP=0.

**CS-AC-10b** GIVEN Xelor player with `reserve=22` and `garde_temps_used_this_game=1` (`garde_temps_per_game_cap=1`), WHEN Garde-Temps play is submitted, THEN server rejects with "per-game cap reached" error; `reserve=22` unchanged; `garde_temps_used_this_game=1` unchanged.

#### Miss Nuit

**CS-AC-11** GIVEN Miss Nuit is in play, WHEN opponent plays 3 cards (spell or minion) in one round, THEN `Xelor.reserve` increases by exactly 2 (`miss_nuit_cap=2` enforced, not 3).

**CS-AC-12** GIVEN Miss Nuit is in play, WHEN opponent spawns 3 token units via DEATH triggers in one round (not card-plays from hand), THEN `Xelor.reserve` is unchanged (token spawns do not qualify as card plays).

#### Sacrier formulas

**CS-AC-13** GIVEN Sacrier player who submitted Sang Méprise at PLACEMENT, WHEN RESOLUTION begins and placements are committed, THEN both players receive a unicast `S2CSangMepriseReveal` containing the `is_fake` status for every alive objective slot across both players. *(Integration test — requires NP-1 and Lightyear unicast infrastructure. Unit test: assert `sang_meprise_active = true` and `reveal_set` is correctly populated after placement commit, without asserting message delivery.)*

**CS-AC-14a** GIVEN Sang Méprise was active during a RESOLUTION, WHEN that RESOLUTION ends (RSM exits sub-step 6 or transitions to next phase), THEN the server's `sang_meprise_active` flag is `false`; subsequent `S2CResolutionEvent` messages for the next round do not include objective reveal data. *(The `sang_meprise_active=false` assertion is headless-testable. Asserting `S2CResolutionEvent` message content is integration-level — requires Lightyear infrastructure.)*

**CS-AC-14b** (ADVISORY) GIVEN Sang Méprise is no longer active, THEN the client renders opponent objectives as hidden (fog/unrevealed state) in the following PLACEMENT phase.

**CS-AC-15** GIVEN Sacrier player with 2 alive real objectives, WHEN Punition is played targeting one real objective, THEN that objective HP→0 AND 3 damage is applied to each alive opponent objective individually.

**CS-AC-16** GIVEN Sacrier player with 0 alive real objectives, WHEN Punition play is submitted, THEN server rejects; mana untouched.

**CS-AC-17** GIVEN Sacrier player has exactly 1 alive real objective remaining (2 already destroyed), WHEN Punition targets that last real objective, THEN the objective HP→0 AND the RSM transitions to GAME_OVER with the Sacrier as the losing player (self-elimination is valid, not a bug).

#### Sadida seeds and tokens

**CS-AC-18** GIVEN a Seed on cell 3 lane 2, WHEN a friendly unit's movement path passes through cell 3 lane 2 during sub-step 5 (whether as an intermediate cell or the final destination), THEN the unit gains +1 AR permanently; the Seed remains on the cell.

**CS-AC-19** GIVEN a Seed on cell 3 lane 2, WHEN an enemy unit's movement path passes through cell 3 lane 2 during sub-step 5 (whether intermediate or final cell), THEN that unit takes 1 damage pre-AR (effective damage = max(0, 1 − unit.AR), routed through the AR reduction pipeline); the Seed persists.

**CS-AC-20** GIVEN Sadida player has 3 Seeds on the board across different lanes, WHEN Graines de Folie is cast, THEN 3 Madolls (HP=3, ATK=1, MP=3) are spawned at the exact Seed cells and all 3 Seeds are removed from the board.

**CS-AC-21** GIVEN Sadida player has 2 Seeds — one on cell 2 lane 3 (empty) and one on cell 1 lane 1 (lane at unit capacity), WHEN Graines de Folie resolves, THEN 1 Madoll spawns at cell 2 lane 3; no Madoll spawns in lane 1; both Seeds are consumed.

#### Ecaflip RNG

> **RNG test setup (CS-AC-22/23/24/25):** ACs requiring specific roll/flip values must use one of: (a) extract resolution logic as a pure function taking `roll: u8` or `flip: u8` directly (preferred for unit tests), or (b) seed the RESOLUTION RNG chain to a ChaCha seed known to produce the required value at the relevant trigger index (required for full integration tests). Test setup must be specified alongside the test implementation.

**CS-AC-22** GIVEN Ecaflip player's Dé du Chateux server RNG roll = 2 (test setup: inject roll via pure function or seeded RNG), WHEN the effect resolves, THEN 2 damage is dealt to the target AND the enemy objective in the target lane is revealed (unicast to Ecaflip player only; roll ≤ 3).

**CS-AC-23** GIVEN Ecaflip player's Dé du Chateux server RNG roll = 5, WHEN the effect resolves, THEN 5 damage is dealt to target AND no objective reveal occurs (roll > 3).

**CS-AC-24** GIVEN Ecaflip player's Craps coin flip = heads (8 damage) with 3 alive opponent objectives, WHEN Craps resolves, THEN objectives in lanes 1 and 2 receive 3 damage each and lane 3 receives 2 damage (`floor(8/3)=2, remainder=2` → first 2 lanes get +1).

**CS-AC-25** GIVEN Ecaflip player's Shava Shavien dies with coin flip = tails, WHEN the DEATH trigger resolves, THEN the Shava Shavien card enters the **opponent's** hand.

#### Shop class filtering and cross-class legality

**CS-AC-26** GIVEN a player whose class is Sadida, WHEN shop slots are generated, THEN the class slot samples exclusively from the Sadida card library (25 cards); no Iop/Cra/Sacrier/Xelor/Ecaflip cards may appear in the class slot.

**CS-AC-27** GIVEN a player triggers a Drheller DEATH draw, WHEN the draw resolves, THEN the drawn card may be any card from the full pool with no class filter applied (cross-class card in hand is legal and playable).

**CS-AC-27b** GIVEN a player holds a cross-class card obtained via Drheller DEATH draw, WHEN that player attempts to play the cross-class card, THEN the server accepts the play (does not reject with any class-restriction error); the card's effect resolves normally.

#### Token passive behaviors

**CS-AC-28** GIVEN a Sinistro spell is placed on a friendly objective in lane 2, WHEN a RESOLUTION completes (after all sub-steps conclude), THEN the enemy objective in lane 2 has taken 1 damage; Sinistro remains attached to its parent objective.

**CS-AC-29** GIVEN a La Gonflable token (HP=3/ATK=2/MP=3) is in play in lane 3 and at least one other friendly unit is also present in lane 3, WHEN La Gonflable's movement ends during RESOLUTION sub-step 5, THEN each other friendly unit in lane 3 is healed for 2 HP (capped at that unit's max HP); La Gonflable itself is not healed.

**CS-AC-30** GIVEN a La Sacrifiée token (HP=2/ATK=2/MP=3) is in play in lane 4, WHEN La Sacrifiée is destroyed (HP → 0) during RESOLUTION, THEN each enemy unit present in lane 4 at the moment of destruction takes 1 damage (routed through the AR reduction pipeline; effective damage = max(0, 1 − unit.AR) per enemy unit in the lane).

#### Miranda control transfer

**CS-AC-31** GIVEN Miranda is in play in lane 3 and has transferred control of an adjacent enemy unit in lane 2, WHEN Miranda is destroyed (HP → 0) during RESOLUTION, THEN all units whose control was transferred by Miranda revert to their original controllers; revert fires before end-of-RESOLUTION sub-step processing concludes.

**CS-AC-32** GIVEN Miranda steals a Sadida-controlled Madoll token (source_class=Sadida), WHEN any LEADER-bonus check fires for the Ecaflip player, THEN the stolen Madoll's `source_class` remains `Sadida`; the Ecaflip player's LEADER cards that specify `Sadida_Token` family do not boost the stolen Madoll.

#### Sinistro destruction

**CS-AC-33** GIVEN a Sinistro spell is attached to a friendly objective in lane 2, WHEN that objective takes any damage amount > 0 during RESOLUTION, THEN Sinistro is destroyed (removed from the objective); subsequent RESOLUTION cycles in lane 2 do not apply Sinistro's 1 damage to the enemy objective.

#### Madoll passive

**CS-AC-34** GIVEN a Madoll token (HP=3/ATK=1/MP=3) is in play, WHEN a Spell-type card's mana cost is evaluated for playability, THEN the effective cost is reduced by 1 (minimum 0); Trap-type and Order-type card costs are unaffected (per OQ-CS-4 ruling).

#### Ecaflip — Craps edge cases

**CS-AC-35** GIVEN Ecaflip plays Craps and all opponent objectives have HP=0 at the time of resolution (alive=0), WHEN the Craps distribution sub-formula runs, THEN no damage is applied, no integer division executes (the `alive == 0` guard fires before any division), and game state is unchanged. *(S1 crash guard — use `checked_div` or explicit early-return in Rust.)*

#### Seed stacking

**CS-AC-36** GIVEN a Seed already occupies cell 3 lane 2, WHEN a Sadida card attempts to place a second Seed on cell 3 lane 2, THEN the placement is discarded; the cell retains exactly 1 Seed; the placing unit's state is unchanged.

#### Sacrier — Punition mutual destruction

**CS-AC-37** GIVEN Sacrier has exactly 1 alive real objective (2 already destroyed) AND opponent also has exactly 1 alive real objective (2 already destroyed), WHEN Punition targets Sacrier's last real objective and opponent's last real objective HP reaches 0 from the 3-damage AOE in the same sub-step, THEN the RSM evaluates both loss conditions simultaneously and declares a **Draw** (not Sacrier loss, not opponent loss).

#### Cra RANGE mechanics

**CS-AC-38** GIVEN a Cra player's class slot generates a card during shop phase, THEN the drawn card belongs to Cra's 25-card class library; no Iop/Sacrier/Xelor/Ecaflip/Sadida cards may appear in the Cra class slot.

**CS-AC-39** GIVEN a Cra RANGE unit at cell distance ≥ 2 from an opponent unit, WHEN RESOLUTION sub-step 6 runs, THEN the Cra unit fires its RANGE attack before any melee exchange (sub-step 3) resolves for that lane.

**CS-AC-40** GIVEN a Cra RANGE unit is pushed to cell distance < 2 by an opponent effect (e.g., ATTRACT), WHEN RESOLUTION sub-step 6 runs, THEN the Cra unit does NOT fire a RANGE attack; it participates in melee resolution at sub-step 3 instead (RANGE eligibility recalculated at each sub-step boundary).

#### Iop class filtering

**CS-AC-41** GIVEN a player whose class is Iop, WHEN shop slots are generated, THEN the class slot samples exclusively from Iop's 24-card class library; no Cra/Sacrier/Xelor/Ecaflip/Sadida cards may appear in the Iop class slot.

#### HASTE and Rollback

**CS-AC-08b** GIVEN Xelor player with `reserve=3` and one unit placed this round with HASTE at cell 2, WHEN Rollback resolves at sub-step 2, THEN the HASTE unit moves to cell 5 (`clamp(2+3)=5`); `reserve=0`. *(Positive case for the OQ-CS-3 ruling: HASTE removes summoning sickness; Rollback is external movement, not an action.)*

## Open Questions

| ID | Question | Owner | Status |
|---|---|---|---|
| OQ-CS-1 | **Xelorium timing** — Closed. Ruling: sub-step 1. Implementation note: Xelorium's own mana cost (4) MUST be deducted from Xelor's `current_mana` before the steal formula fires, or Xelor will steal back their own 4 mana. Deduction precedes effect application per Economy Rule 4. | Design | **Closed — sub-step 1** |
| OQ-CS-2 | **Sang Méprise reconnect gap** — Resolved. NP GDD closed this via snapshot-field approach: `active_sang_meprise_reveals: Option<Vec<(PlayerId, LaneId, bool)>>` in `S2CGameSnapshot`. Client rebuilds reveal state from the snapshot field on reconnect; no re-unicast path is needed. The Class GDD Edge Cases section (disconnect section) is authoritative. | Network Protocol | **Closed — NP GDD resolution (snapshot field)** |
| OQ-CS-3 | **Rollback and HASTE** — Closed. Ruling: HASTE units placed this round are eligible for Rollback movement. Rationale: HASTE removes summoning sickness for action; Rollback is external movement (sub-step 2), not an action. CS-AC-08 is correct as written. | Design | **Closed — HASTE units eligible** |
| OQ-CS-4 | **Madoll passive cost-reduction scope** — Closed. Ruling: Spell-type cards only. Rationale: consistent with Krosmaga original and with the Keyword System's spell/trap/order type distinction. Trap and Order cards are NOT affected by Madoll's passive. | Design | **Closed — Spell-type only** |
| NP-1 | **`PlayerSnapshot` missing `class_id` field** — **RESOLVED** in NP GDD (Pass 3 verification): `PlayerSnapshot` already contains `class_id: ClassId`. CS-AC-03a annotation updated: NP-1 dependency is satisfied; remaining dependency is Lightyear unicast infrastructure only. | Network Protocol | **Closed — field present in NP GDD** |
| NP-2 | **`UnitBoardState` missing `source_class` field** — Miranda-stolen tokens must retain `source_class` for LEADER bonus checks and client rendering. `UnitBoardState` needs `source_class: Option<ClassId>` (None for non-token units; Some(class) set at spawn, never mutated). Required for state reconstruction on crash/reconnect. | Network Protocol | Open — NP required change |
| NP-3 | **`S2CResolutionEvent` missing `UnitSpawned` variant** — Tokens (Mummy, Madoll, Chacha Noir, La Gonflable, La Sacrifiée) spawn during RESOLUTION. Without a `UnitSpawned` event variant, clients animating resolution replay will encounter `UnitMoved` for entities that don't yet exist locally. Add `UnitSpawned { unit_id, card_id, owner, lane, cell, source: SpawnSource }` where `SpawnSource` ∈ {DeathTrigger, AppearanceTrigger, ClassTokenConversion}. | Network Protocol | Open — NP required change |
| NP-4 | **No registered message type for Dé du Chateux single-lane reveal** — The single-lane unicast (Ecaflip-only, per Dé du Chateux roll ≤ threshold) cannot reuse `S2CSangMepriseReveal` (wrong payload shape, wrong semantic, wrong recipient scope). NP GDD must register `S2CSingleObjectiveReveal { player_id: PlayerId, lane: u8, is_fake: bool }` as a separate unicast message. | Network Protocol | Open — NP required change |
| NP-5 | **Reserve mutations during RESOLUTION have no communication path** — `S2CGoldUpdate` is suppressed during RESOLUTION per NP GDD rules. Four mutation sites have no specified event variant: Xelorium (sub-step 1), Rollback (sub-step 2), Mummy passive (per-hit during combat sub-steps), and **Garde-Temps deduction** (sub-step 1, deducts 20 reserve on activation). All four sites must be covered. Either add a `ReserveChanged { player_id, new_reserve }` variant to `S2CResolutionEvent`, or clarify that the suppression rule applies to gold-only updates and allows reserve-only unicasts. | Network Protocol | Open — NP required change |
| NP-6 | **Sinistro ResolutionEvent gaps** — Three missing protocol elements: (1) No `DamageKind` variant to discriminate Sinistro damage from spell damage during RESOLUTION replay; (2) Sinistro's sub-step assignment is unspecified (recommended: sub-step 6, after all combat); (3) No `SinistroDestroyed` event for the client to remove the Sinistro indicator mid-RESOLUTION when the parent objective takes damage. NP GDD must add all three. | Network Protocol | Open — NP required change |
| NP-7 | **Miranda control transfer has no message** — `CardOwner` component replication is suppressed during RESOLUTION animation. `UnitMoved` carries no `controller_id` field. Client cannot render Miranda-stolen units with the correct team color during RESOLUTION replay. Miranda revert on death has no `released_units` payload. NP GDD must add: `ResolutionEvent::ControlTransferred { unit_id, new_owner }` and `MirandaRevertFired { released_units: Vec<EntityId> }`. | Network Protocol | Open — NP required change |
| NP-8 | **Chacha Noir SpawnSource missing Replacement variant** — `SpawnSource ∈ {DeathTrigger, AppearanceTrigger, ClassTokenConversion}` is incomplete. Chacha Noir replaces a target unit (original dies, Chacha Noir spawns at its cell). Without `SpawnSource::Replacement { replaced_unit_id: EntityId }`, the client cannot causal-link the `UnitDied` and `UnitSpawned` events and will render them as unrelated animations rather than a replacement sequence. NP GDD must add this variant. | Network Protocol | Open — NP required change |
| NP-9 | **`SeedPlaced` and `SeedConsumed` events missing from `S2CResolutionEvent`** — When Pollinisation, Sac de Graines, or Ronce DEATH places a Seed during RESOLUTION, no event notifies the client. Without `SeedPlaced { owner: PlayerId, lane: u8, cell: u8 }`, the client cannot animate seed appearance during RESOLUTION and seeds will appear as a visual jump on snapshot reconciliation. Similarly, Graines de Folie's seed conversion requires `SeedConsumed { lane: u8, cell: u8 }` events (paired with `UnitSpawned` at the same cell) to animate the conversion batch. NP GDD must add both event variants. | Network Protocol | Open — NP required change |
