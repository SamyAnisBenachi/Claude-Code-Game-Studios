# Objective System

> **Status**: In Review — revised post /design-review R2 (2026-04-29)
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Simple surface · Deep emergence · No idle spectating

## Overview

The Objective System is the server-side authority for each player's five objectives — three real and two fake — throughout the game session. It owns each objective's identity (real or fake, visible only to the owning player), HP state (5 HP each, 0 AR, cannot be healed), and cumulative destruction record. On any objective's HP reaching zero, the Objective System executes the correct consequence path: awarding gold to the attacker via the Economy System, recording fake destruction facts/counters for Board/Lane to project into spawn range, applying the server-RNG-drawn fake reward (mana cap +1 or free card pick at 50/50), or advancing the loss condition count for real objective destruction. It exposes two authoritative read interfaces: `real_objectives_destroyed(player)`, read by the Round State Machine after each RESOLUTION to evaluate GAME_OVER, and `fake_objectives_destroyed(player)`, read by the Board/Lane System to update `SpawnRangeState`. Objective System does not own live spawn range projection, `PlayerSnapshot.spawn_range_cells`, or `SpawnRangeChanged` transport. Initial fake-lane assignment uses two Server-side RNG seeds per player drawn at DRAFT_INITIAL; reward randomization uses one seed per fake destruction event.

## Player Fantasy

**The Liar King.** Five of your objectives face the opponent — three real, two counterfeit. You built this trap before the first card was played. Now you watch them walk into it.

The fantasy the Objective System creates is not *"I smashed the castle."* It is colder and more gleeful: *"I built a castle out of lies, and they attacked the wrong one."* Every fake destroyed is the trap working as designed. The attacker grows strong on your deception — expanded spawn range, a windfall bonus, three gold — and you remain in the game, one step safer, watching them believe they've found the path.

**The anchor moment:** A unit steps onto Cell 8 in Lane 3. The objective's HP ticks down. Neither player moves. Both are running the same calculation in opposite directions — *was that a tell?* — then the next round the attacker pivots to Lane 1. The defender exhales. Or curses. Depending on what was actually there. The moment is the silence between the strike and the reveal.

**As the attacker,** the fantasy inverts: you read silence, hunt tells, convert visible damage numbers into deduction. Every HP tick is evidence. Every pivot costs tempo. The game is a negotiation where the currency is conviction.

**What the player must never feel:** that attacking a fake was wasted. The fake rewards are real rewards — the defender placed them there. The system must make both sides feel they played well regardless of which objective was real.

## Detailed Rules

### Core Rules

**Rule 1 — Objective set ownership:**
Each player begins with exactly 5 objective slots, one per lane (lanes 1–5). Each slot holds: a lane index, a current HP value (initialized to `objective_hp = 5`), and a real/fake identity set at game start. The Objective System is the sole server-side authority for all objective state.

**Rule 2 — Fake lane assignment:**
At DRAFT_INITIAL, the Server-side RNG provides 2 seeds per player:
- Seed 1: `gen_range(0..5)` → first fake lane index (0–4)
- Seed 2: `gen_range(0..4)` → index into the 4 remaining lanes → second fake lane
The remaining 3 lanes are real objectives. This assignment is immutable for the game session.

**Rule 3 — HP properties:**
Objectives have `objective_hp = 5` HP and 0 AR. They cannot be healed — any healing effect targeting an objective is a no-op. HP cannot go below 0.

**Rule 4 — HP visibility and hidden information:**
Both players see each opposing objective's current HP as a number (5 → 3 → 0). The owner also sees which of their 5 slots are real vs. fake; the attacker does not — real/fake identity is hidden until destruction. This split is enforced via two separate replicated components:
- `ObjectiveHp { hp: u32 }` — replicated to both players
- `ObjectiveIdentity { is_fake: bool }` — replicated to the owning player only

On destruction, the Objective System broadcasts `ObjectiveDestroyed { target_player_id: PlayerId, lane: LaneId, was_fake: bool }` to both players. The `target_player_id` field identifies whose objective was destroyed — required when multiple objectives fall in the same RESOLUTION (one per player) to prevent ambiguous client rendering. The attacker learns real/fake at the RESOLUTION-end sync — not mid-sub-step.

> **Design note — exact HP integers (deliberate)**: The attacker sees precise HP numbers (5 → 3 → 0) rather than qualitative wound states (Healthy/Wounded/Critical). Design review flagged this as a risk to the bluff-deduction experience. It is retained as a conscious design decision: exact HP creates a valid information game ("how many more hits to finish this lane?") and avoids UI ambiguity, at the cost of some bluff depth. Revisit in playtesting if the "I fooled them" feeling is underdelivered.

**Rule 5 — Damage interface:**
All damage to objectives flows through one interface: `take_damage(lane: LaneId, attacker_player: PlayerId, amount: u32)`. Combat Resolution calls this during RESOLUTION sub-step 6 for units at Cell 8. Spell effects call the same interface with the spell controller as `attacker_player`. The Objective System handles all HP reduction, destruction checks, and consequence dispatch.

**Rule 6 — Damage ordering:**
Within sub-step 6, `take_damage()` calls are processed in ascending lane order (lane 1 → 2 → 3 → 4 → 5). A unit that is destroyed in lane 1 combat does not also deal objective damage. HP values broadcast to clients are batched at the end of the sub-step, not per-call.

**Rule 7 — Destruction consequence path:**
When `take_damage()` reduces HP to 0, the Objective System executes this sequence:

1. Mark slot destroyed. HP clamped to 0. Queue `ObjectiveDestroyed { target_player_id: defending_player, lane, was_fake }` for RESOLUTION-end broadcast.
2. **Gold award** (if `attacker_player ≠ defending_player`): emit `AwardGold { player: attacker_player, amount: 3 }` to Economy System.
3. **Fake-specific rewards** (if fake AND `attacker_player ≠ defending_player`):
   - Increment `fake_objectives_destroyed(attacker_player)` by 1. This is an objective destruction fact/counter only. Board/Lane System consumes it to update live `SpawnRangeState`, snapshot `spawn_range_cells`, and the ordered `SpawnRangeChanged` resolution-log entry.
   - Draw 1 reward seed from Server-side RNG (`gen_range(0..2)`):
     - `0` → emit `ManaCapIncreased { player: attacker_player, amount: 1 }`. Economy System applies at RESOLUTION end; takes effect the following DRAFT.
     - `1` → check `attacker_player`'s hand size first: if hand is at max capacity (10 cards), skip `draw_random()` and emit `AwardGold { player: attacker_player, amount: 1 }` as fallback (see OS-15); this branch terminates here. Otherwise: draw 1 free-card seed from Server-side RNG, then call `draw_random(filter: PoolFilter { rarity: None, class: None, card_type: None, max_cost: None }, seed)` on Card Data & Pool. On `Some(card_id)`: call `distribute(card_id)` on the pool and add the card to the player's hand server-side; client sees it at RESOLUTION-end sync. On `None` (pool exhausted): no-op (see OS-22).
4. **Real objective destroyed** (if real): increment `real_objectives_destroyed(defending_player)` by 1.
5. **Self-destruction** (`attacker_player == defending_player`): steps 2 and 3 are skipped — no gold, no fake rewards for destroying your own objective. Step 4 executes normally if real.

**Rule 8 — Loss condition exposure:**
The Objective System exposes `real_objectives_destroyed(player): u32` as a read interface. The Round State Machine reads this after all 6 RESOLUTION sub-steps complete. A player whose count reaches ≥ 2 triggers GAME_OVER. If multiple players simultaneously reach ≥ 2 in the same RESOLUTION, the result is `GameOverReason::Draw`.

**Rule 9 — Destroyed slot display:**
Destroyed slots are removed from the board at RESOLUTION-end sync. No persistent "was fake / was real" tombstone is displayed. Board Rendering receives `ObjectiveDestroyed` and clears the visual slot.

**Rule 10 — Mana cap ceiling:**
`ManaCapIncreased` is a no-op if `mana_cap_effective` is already at `mana_cap_max` (= `GameConfig.mana_cap + fake_count`). At defaults: `mana_cap_max = 10 + 2 = 12`. The Economy System enforces this ceiling when processing the event. If `GameConfig.mana_cap` is tuned above 10, the ceiling scales proportionally.

> **Design note — fake reward > real reward (deliberate)**: Destroying a fake objective yields more combined reward (+3g + spawn expansion + mana cap or card pick) than destroying a real objective (+3g + loss condition advance). This asymmetry is intentional: it compensates the attacker for being deceived, keeping the defender's bluff credible rather than punishing attackers for targeting fakes. Without this compensation, attackers who consistently hunt the wrong lanes would have no strategic upside; the bluff would become a pure penalty. The system is designed so both players feel they played well regardless of which objective was real.

> **Design note — interest bracket capture (deliberate)**: Objective reward gold (+3) is awarded at RESOLUTION, before the Economy System's interest snapshot (per RSM Rule 4: "after all combat, kill rewards, and objective rewards"). A player at 7g who destroys an objective ends RESOLUTION at 10g, capturing the maximum interest bracket (+2). This bracket capture is intentional — it rewards aggressive play that creates gold income, consistent with the miser/gambler tension the economy system is designed to produce.

### States and Transitions

| State | Condition | Attacker sees | Owner sees |
|---|---|---|---|
| **Intact** | HP = 5 | HP = 5, identity hidden | HP = 5, real/fake known |
| **Damaged** | 0 < HP < 5 | Current HP, identity hidden | Current HP, real/fake known |
| **Destroyed** | HP = 0 | Real/fake revealed (at RESOLUTION end) | Real/fake confirmed |

Transitions: `Intact → Damaged` (any damage, HP > 0 remaining) → `Damaged → Destroyed` (HP reaches 0). `Destroyed` is terminal.

### Reveal Moment — Design Specification

**This is the primary emotional payload of the Objective System.** Every rule in this document exists to set up this single beat: the moment between HP reaching 0 and the fake/real identity appearing.

**Delivery:** `ObjectiveDestroyed { was_fake }` events from a given RESOLUTION are delivered in a single batch at RESOLUTION-end sync (consistent with Rule 6 batching). Do not reveal identities one-by-one as damage resolves mid-sub-step.

**Mandatory gap:** There MUST be a minimum display hold between HP reaching 0 (the strike) and `was_fake` being shown (the reveal). **Minimum: 500ms.** This gap is "the silence" — it is not a polish consideration and not an editorial choice for Board Rendering. It is the mechanical moment the "Liar King" fantasy is built on.

**Reveal ordering:** When multiple objectives are destroyed in the same RESOLUTION, reveal in ascending lane order. Reveal the acting attacker's results first, then the defender's.

**Board Rendering GDD constraints (mandatory, not suggestions):**
- The reveal animation MUST be a distinct visual event from HP reaching 0 — two separate beats, not one continuous animation.
- If Sang Méprise was active this RESOLUTION, suppress the "surprise reveal" animation for any objective whose identity was already known to the viewer — the surprise cannot fire twice.
- Audio cue at the reveal beat is required — this is the primary audio moment of the entire system. Coordinate with Audio Director before Board Rendering GDD is authored.

**What the player must experience:**
- **As defender:** A moment of held breath between the unit landing and the identity appearing. The trap either worked or it didn't — but you know before they do.
- **As attacker:** Conviction or dread in the silence. The reveal answers the calculation you've been running all round.

### Interactions with Other Systems

| System | Direction | Interface |
|---|---|---|
| **Server-side RNG** | ← Consumes | 2 seeds/player at DRAFT_INITIAL (fake assignment); per fake destruction: 1 seed for D4 reward draw + 1 additional seed for `draw_random` if outcome is FreeCardPick (total: **1 seed** if ManaCapIncreased, **2 seeds** if FreeCardPick) |
| **Game Config** | ← Reads | `objective_hp`, `fake_count`, `objective_gold_reward` at session init |
| **Combat Resolution** | ← Receives | `take_damage(lane, attacker_player, amount)` during sub-step 6 |
| **Economy System** | → Emits | `AwardGold { player, amount: 3 }` on destruction (attacker ≠ owner); `ManaCapIncreased { player, amount: 1 }` for mana cap reward |
| **Card Data & Pool** | → Calls | `draw_random(PoolFilter, seed)` for free card pick reward; `distribute(card_id)` on non-None result; card placed in hand server-side |
| **Board/Lane System** | → Exposes | `fake_objectives_destroyed(player): u32` destruction fact/counter; BLS consumes this to update `SpawnRangeState` |
| **Round State Machine** | → Exposes | `real_objectives_destroyed(player): u32` read interface; RSM queries after each RESOLUTION |
| **HUD** | → Replicates | `ObjectiveHp` to both players; `ObjectiveIdentity` to owner only; `ObjectiveDestroyed` to both on destruction |
| **Board Rendering** | → Replicates | `ObjectiveHp`, destroyed state; cleared slot on destruction |

## Formulas

### D1: objective_damage

`HP_new = max(0, HP_current − amount)`

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Current HP | `HP_current` | u32 | 0–5 | Objective HP before this call; 0 = already destroyed (destruction guard fails — no-op) |
| Damage amount | `amount` | u32 | 0–∞ | ATK of attacker unit (at Cell 8), or spell damage value; `amount=0` is short-circuited (see Notes) |
| New HP | `HP_new` | u32 | 0–5 | HP after reduction; destruction fires when this equals 0 |

**Output Range:** 0 to 5.
**Example:** `HP_current = 3`, `amount = 5` → `max(0, 3 − 5) = 0`. Objective destroyed.
**Notes:** AR is 0 on all objectives — no AR reduction step. In Rust: use `HP_current.saturating_sub(amount)`. If `amount == 0`, short-circuit (no destruction check). Self-destruction (`attacker_player == defending_player`) follows the same formula; consequence path in Rule 7 step 5 skips gold and fake rewards.

---

### D2: loss_condition

`is_eliminated(player) = (real_objectives_destroyed(player) >= loss_threshold)`

where `loss_threshold = 2` (fixed design constant — do NOT use the derived expression in code)

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Real objectives destroyed | `real_objectives_destroyed` | u32 | 0–(lane_count − fake_count) | Accumulated count for this player this session; max = 3 at default fake_count=2, up to 4 at fake_count=1 |
| Loss threshold | `loss_threshold` | u32 | 2 (fixed) | Fixed design constant. Historical derivation `(lane_count − fake_count) − 1` equals 2 at defaults but **must NOT be used in code** — always hardcode 2. Tuning `fake_count` does not change this value. |

**Output Range:** Boolean. Evaluated by the Round State Machine after all 6 RESOLUTION sub-steps complete.
**Example:** `real_objectives_destroyed(Player B) = 2`, `loss_threshold = 2` → `true` → GAME_OVER.
**Notes:** `loss_threshold = 2` is a **fixed design constant** (master GDD: "You LOSE when 2 of your own REAL objectives are destroyed"). It does NOT derive from `fake_count` — tuning `fake_count` changes how many reals exist, but the threshold stays at 2. If both players reach the threshold in the same RESOLUTION, result is `GameOverReason::Draw` (RSM Rule 14 — owned by `round-state-machine.md`). The Objective System exposes the count; the RSM owns the elimination decision.

---

### D3: mana_cap_after_fakes

`mana_cap_effective(player) = min(mana_cap_base + fakes_rewarded_mana(player), mana_cap_max)`

where `mana_cap_max = mana_cap_base + fake_count = 10 + 2 = 12`

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Base mana cap | `mana_cap_base` | u32 | 10 | GameConfig default |
| Mana cap rewards received | `fakes_rewarded_mana(player)` | u32 | 0–2 | Count of `ManaCapIncreased` events this session for this player |
| Max mana cap | `mana_cap_max` | u32 | 12 at defaults | `mana_cap_base + fake_count`; scales with config |

**Output Range:** 10 to 12 at defaults.
**Example:** 2 mana cap rewards received: `min(10 + 2, 12) = 12`. At defaults with `fake_count=2`, only 2 fakes exist — a third `ManaCapIncreased` is structurally impossible in normal play (the ceiling guard is relevant only when `fake_count` is tuned higher). Feeds into `mana_ramp`: `current_mana(R) = min(R, mana_cap_effective)`.

---

### D4: fake_reward_probability

`reward = RNG.gen_range(0..2)` → `{ 0: ManaCapIncreased, 1: FreeCardPick }`

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| RNG draw | `gen_range(0..2)` | u32 | {0, 1} | Server-side draw; exclusive upper bound = uniform 50/50 |
| P(ManaCapIncreased) | — | float | 0.5 | Per fake destruction event; independent draws |
| P(FreeCardPick) | — | float | 0.5 | Per fake destruction event; independent draws |

**Output Range:** Two discrete outcomes, P = 0.5 each. Each destruction event is an independent Bernoulli trial.
**Example:** Fake destroyed in lane 2 → `gen_range(0..2) = 0` → `ManaCapIncreased { player: A, amount: 1 }`. Mana cap: 10 → 11.
**Notes:** If `mana_cap_effective` is already 12 when a mana cap reward fires, Economy System applies a no-op. Card selection for `FreeCardPick` is owned by Card Data & Pool.

---

### D5: fake_lane_assignment

Two-step RNG draw at DRAFT_INITIAL per player:

```
// LaneId is 1-indexed: value 1–5 corresponds directly to display lane 1–5
lanes = [1, 2, 3, 4, 5]
fake_1 = lanes[RNG_1.gen_range(0..5)]      // RNG yields array index 0–4; result is LaneId 1–5
fake_2 = remaining[RNG_2.gen_range(0..4)]  // index 0–3 into the 4 remaining LaneIds
real_lanes = lanes \ {fake_1, fake_2}
```

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| First RNG draw | `RNG_1.gen_range(0..5)` | u32 | 0–4 | Index into full 5-lane list |
| Second RNG draw | `RNG_2.gen_range(0..4)` | u32 | 0–3 | Index into remaining 4 lanes after `fake_1` removed |
| First fake lane | `fake_1` | LaneId | 1–5 | LaneId is 1-indexed — value equals the display lane number |
| Second fake lane | `fake_2` | LaneId | 1–5, ≠ fake_1 | Distinct by construction |

**Output Range:** C(5,2) = 10 distinct lane pairs, each with probability 1/10 = 10%.
**Example:** `RNG_1 = 3` → array index 3 into [1,2,3,4,5] → fake_1 = LaneId 4 (display: lane 4). Remaining = [1,2,3,5]. `RNG_2 = 1` → array index 1 into [1,2,3,5] → fake_2 = LaneId 2 (display: lane 2). `fake_lanes = {2, 4}`, `real_lanes = {1, 3, 5}`.
**Notes:** Two independent `gen_range(0..5)` draws without removal would produce a 20% collision rate — distinctness requires the removal approach. Uses exactly 2 seeds per player from the DRAFT_INITIAL RNG chain. Spawn range projection is owned by `board-lane-system.md`; this system only provides the `fake_objectives_destroyed(player)` destruction fact/counter.

## Edge Cases

- **If a fake objective is destroyed**: The loss condition counter does NOT advance. Only `fake_objectives_destroyed(attacker_player)` increments. `real_objectives_destroyed(defending_player)` is unchanged. Destroying both fakes and still being in the game is intended and expected.

- **If Punition (Sacrier spell) forces self-destruction of your own objective** (`attacker_player == defending_player`): Gold award and fake rewards are skipped (Rule 7, step 5). If the destroyed objective is real, `real_objectives_destroyed` advances by 1 (self-inflicted loss condition advance). If fake, no consequence beyond removing the slot.

- **If a double-tranchant card destroys one of your own fake objectives**: Same as self-destruction — no spawn expansion, no mana cap reward, no gold.

- **If "Sang Méprise" (Sacrier Krosmic) is played during RESOLUTION**: The server sends `S2CSangMepriseReveal { identities: Vec<(LaneId, is_fake)> }` as a targeted reliable unicast to the opponent only (Option B — see OQ5). The opponent's client stores these identities in local state for the duration of this RESOLUTION; local state is cleared at RESOLUTION end. Replication scope for `ObjectiveIdentity` is never changed. Destruction events during Sang Méprise still fire `ObjectiveDestroyed { was_fake }` at RESOLUTION end as the authoritative record — the event fires regardless of prior visibility. Board Rendering should suppress the "surprise reveal" animation for any objective whose identity was already known via this reveal. **Reconnect gap:** `S2CSangMepriseReveal` is a one-shot unicast not included in `S2CGameSnapshot`. A player who reconnects mid-RESOLUTION after this event fired will not receive the revealed identities. The Network Protocol GDD must address this in the snapshot schema or reconnect re-delivery path.

- **If a unit persists at Cell 8 in a lane where the objective is already destroyed**: `take_damage()` is called again next RESOLUTION sub-step 6. `HP_current = 0`; `saturating_sub` returns 0. Destruction guard: `HP_new == 0 AND HP_current > 0` — fails. No consequence path fires. Unit persists at Cell 8 until killed.

- **If a FreeCardPick reward fires and the attacker's hand is at 10 cards**: `draw_random()` is not called. Emit `AwardGold { player: attacker_player, amount: 1 }` as a fallback instead. Applied at RESOLUTION end with other Economy events. Note: the attacker cannot time this fallback — fake lane identity is unknown until the moment of destruction, so hand management before the fact is not possible. The +1g fallback is a consequence of capacity, not a strategic choice.

- **If multiple damage sources target the same objective in the same sub-step**: The first `take_damage()` call (in lane-ascending order) reduces HP to 0 and fires the full consequence path. Subsequent calls have `HP_current = 0`, fail the destruction guard, and are no-ops. The consequence path fires exactly once per objective per game.

- **If a spell targets an already-destroyed objective** (HP = 0, destroyed flag set): `take_damage()` call is a no-op (destruction guard fails). Server accepts silently; client-side UI should gate spell targeting to intact objectives but server must handle stale targeting commands gracefully.

- **If `take_damage()` is called with `amount == 0`**: Short-circuit immediately — no `saturating_sub`, no destruction guard, no events. Prevents accidental consequence path triggers from 0-damage effect calls.

- **If a FreeCardPick reward fires but the card pool returns `None`** (no cards available): No card granted, no gold fallback, no re-roll to mana cap. Re-rolling would bias the D4 50/50. Server-side no-op; log for debug.

- **If both fake objectives are destroyed in the same RESOLUTION and both draw `ManaCapIncreased`**: Economy System receives two `ManaCapIncreased { amount: 1 }` events in the same batch. Applied in lane order at RESOLUTION end: first `10 → 11`, second `11 → 12`. Both valid. If `mana_cap_effective` was already 11 before this RESOLUTION, first raises to 12, second is a ceiling no-op.

- **If Sang Méprise is active during a RESOLUTION in which an objective is destroyed**: Both players already see all identities. `ObjectiveDestroyed { was_fake }` still fires at RESOLUTION end as the authoritative record. Board Rendering should suppress the "surprise reveal" animation if Sang Méprise was active this RESOLUTION — flag for Board Rendering GDD.

- **If Garde-Temps (Xelor Krosmic, costs 20 reserve mana) targets an objective**: Treated as `take_damage(lane, attacker_player, objective_hp)` — lethal damage regardless of current HP. Full consequence path fires. If the targeted objective is already destroyed, no-op.

- **If both players' real objectives simultaneously reach the loss threshold in the same RESOLUTION**: Both `real_objectives_destroyed` counts cross ≥ 2 during the same RESOLUTION sub-step sequence. RSM evaluates after all sub-steps complete, finds both players qualify → `GameOverReason::Draw`. No winner declared.

- **If `fake_count > lane_count - loss_threshold` in GameConfig** (e.g., `fake_count = 4` with `lane_count = 5`, `loss_threshold = 2`): Only 1 or fewer real objectives exist per player; `loss_threshold = 2` can never be reached by normal play. The game cannot end by win condition. **This configuration is invalid.** The server must assert `fake_count <= lane_count - loss_threshold` at session initialization and refuse to start the session if violated. At defaults: `assert!(fake_count <= 3)`. Note: `fake_count = 3` is valid — with 2 reals, the attacker must destroy both. `fake_count = 4` is the first invalid value. See AC OS-23a.

- **If `objective_hp = 0` in GameConfig**: Objectives would spawn with HP = 0, immediately satisfying the destruction guard. The consequence path behavior at spawn is undefined. The server must assert `objective_hp >= 1` at session initialization. See AC OS-23b.

- **If `fake_count = 0` in GameConfig**: D5 unconditionally draws two RNG seeds and assigns two fake lanes regardless of `fake_count`. With `fake_count = 0`, two lanes would be marked fake despite zero fakes being intended. The server must assert `fake_count >= 1` at session initialization. See AC OS-23b.

- **If a FreeCardPick reward fires and `draw_random()` returns `None`** (pool exhausted): No card is granted, no gold fallback, no re-roll to mana cap. Re-rolling would bias the D4 50/50. Silent no-op — distinct from the hand-full fallback (OS-15) which emits +1g. See AC OS-22.

## Dependencies

| System | Direction | Coupling | Interface | Notes |
|---|---|---|---|---|
| **Server-side RNG** | Upstream | Hard | 2 seeds/player at DRAFT_INITIAL (fake assignment); 1 seed per fake destruction (reward draw) | Cannot initialize without RNG. Seeds must be drawn before any placement is accepted. |
| **Game Config** | Upstream | Hard | Reads `objective_hp`, `fake_count`, `objective_gold_reward`, `fake_objective_spawn_advance` at session init | All locked values; all parameters source from Game Config. |
| **Round State Machine** | Downstream | Hard | RSM reads `real_objectives_destroyed(player)` after each RESOLUTION | Read-only. RSM owns the GAME_OVER decision; this system provides the count. Bidirectional: RSM GDD Rule 11 confirms dependency. |
| **Board / Lane System** | Downstream | Hard | BLS consumes `fake_objectives_destroyed(player)` to update `SpawnRangeState` | Read-only destruction fact from Objective. BLS owns live spawn range projection, snapshot source, and `SpawnRangeChanged` transport. |
| **Combat Resolution** | Upstream caller | Hard | Combat Resolution calls `take_damage(lane, attacker_player, amount)` during sub-step 6 | Not yet designed. Must reference this interface when authored. |
| **Economy System** | Downstream receiver | Hard (one-way) | Receives `AwardGold { player, amount }` and `ManaCapIncreased { player, amount: 1 }` events at RESOLUTION end | Economy GDD should reference this system as the event source when updated. |
| **Card Data & Pool** | Downstream caller | Hard (one-way) | Objective System calls `draw_random(PoolFilter, seed)` for free card pick; `distribute(card_id)` on non-None result | Pool returns `None` if no cards available; treated as no-op. |
| **HUD** | Downstream view | Soft | Reads `ObjectiveHp` (both players) and `ObjectiveIdentity` (owner only) via Lightyear replication | Not yet designed. Must list this system as a data dependency when authored. |
| **Board Rendering** | Downstream view | Soft | Reads `ObjectiveHp`, destroyed state via replication; clears slot on `ObjectiveDestroyed` | Not yet designed. Must list this system as a data dependency when authored. |

## Tuning Knobs

The Objective System has no independently owned tuning knobs. All configurable parameters are defined in `game-config.md` and read at session initialization.

| Parameter | Source | Default | Safe Range | Effect on Objective System |
|---|---|---|---|---|
| `objective_hp` | game-config.md | 5 | 3–8 | HP of each objective. Lower = faster objective kills, more explosive tempo. Higher = more sustained unit pressure needed; more comeback potential. |
| `fake_count` | game-config.md | 2 | 1–3 | Fakes per player. Controls bluff depth and mana cap ceiling (`mana_cap_base + fake_count`). Note: `loss_threshold` stays at 2 regardless. At `fake_count = 1`: 4 reals, attacker needs 2 of 4 (easier to close). At `fake_count = 3`: 2 reals, attacker must destroy both (harder to close; near-perfect information needed). |
| `objective_gold_reward` | game-config.md | 3 | 2–5 | Gold awarded on any objective destruction (attacker ≠ owner). Higher = more gold snowball from first destruction. |
| `fake_objective_spawn_advance` | game-config.md / Board-Lane System | 1 | 1–2 | Spawn range expansion (cells) per fake destroyed. Listed for cross-system context only; Board/Lane reads this knob and owns the live projection. |

**Derived values (not independently configurable):**
- `loss_threshold = 2` — fixed design constant; does not change with `fake_count`
- `mana_cap_max = mana_cap_base + fake_count` — ceiling on mana cap increases from fake rewards
- `fake_reward_probability = 0.5 / 0.5` — hardcoded uniform Bernoulli trial; not a GameConfig field

**Required configuration invariants (server assertion at session init):**
- `fake_count >= 1` — `fake_count = 0` causes D5 to unconditionally assign two fake lanes despite no fakes being intended.
- `fake_count <= lane_count - loss_threshold` (i.e., `fake_count <= 3` at defaults) — violating this makes the loss condition unreachable; the game cannot end. (`fake_count = 3` is valid; `fake_count = 4` is the first invalid value.)
- `objective_hp >= 1` — `objective_hp = 0` causes objectives to spawn destroyed.
- These are mandatory startup guards, not advisory ranges. Session must be refused if any is violated.

## Visual/Audio Requirements

Visual display of objectives is owned by the Board Rendering GDD. Audio feedback for objective destruction (including the fake reveal moment) is deferred to the Board Rendering and HUD GDDs. See Open Questions #4.

## UI Requirements

Objective status display (5 dots per side, HP numbers) is owned by the HUD GDD. The HUD consumes `ObjectiveHp` (both players) and `ObjectiveIdentity` (owner only) from this system's replicated components.

## Acceptance Criteria

| # | Criterion | Type |
|---|---|---|
| OS-1 | **GIVEN** a new game session starts and DRAFT_INITIAL fires, **WHEN** objective state is queried for any player, **THEN** that player has exactly 5 objective slots, each with HP = 5, AR = 0, and a real/fake identity assigned and immutable for the session. | BLOCKING |
| OS-2 | **GIVEN** fake lane assignment runs at DRAFT_INITIAL using two RNG seeds per player, **WHEN** the two fake lane indices are recorded, **THEN** the two assigned fake lanes are always distinct — no player receives the same lane index as both fakes. | BLOCKING |
| OS-3 | **GIVEN** an objective at HP = 3, **WHEN** `take_damage(lane, attacker, 2)` is called, **THEN** objective HP = 1 and no destruction event or consequence path fires. | BLOCKING |
| OS-4 | **GIVEN** an objective at HP = 2, **WHEN** `take_damage(lane, attacker, 5)` is called, **THEN** objective HP = 0 (not negative) and the destruction consequence path fires exactly once. | BLOCKING |
| OS-5 | **GIVEN** an objective already at HP = 0 (destroyed), **WHEN** `take_damage(lane, attacker, 3)` is called, **THEN** HP remains 0, no destruction event fires, and no rewards are emitted. | BLOCKING |
| OS-6 | **GIVEN** an objective at HP = 3, **WHEN** any healing effect targets it, **THEN** HP remains 3 — healing is a no-op on objectives. | BLOCKING |
| OS-7 | **GIVEN** any objective (real or fake) is destroyed by the opponent (`attacker_player ≠ defending_player`), **WHEN** the consequence path runs, **THEN** `AwardGold { player: attacker_player, amount: 3 }` is emitted exactly once. | BLOCKING |
| OS-8 | **GIVEN** a fake objective is destroyed by the opponent, **WHEN** the consequence path runs, **THEN** `fake_objectives_destroyed(attacker)` increments by 1, AND exactly one of `{ManaCapIncreased, FreeCardPick}` is emitted (not both, not neither). | BLOCKING |
| OS-9 | **GIVEN** `real_objectives_destroyed(player) = 1`, **WHEN** that player's second real objective is destroyed, **THEN** `real_objectives_destroyed(player) = 2`. *(Unit test scope: assert the count only. RSM transition to GAME_OVER is verified in RSM integration tests, not here — testing it here against a mocked RSM produces false confidence.)* | BLOCKING |
| OS-10 | **GIVEN** both players each have `real_objectives_destroyed` = 1, **WHEN** one unit in each player's lane deals lethal damage to a real opponent objective in the same RESOLUTION sub-step sequence, **THEN** `real_objectives_destroyed(player_a) = 2` AND `real_objectives_destroyed(player_b) = 2` after all sub-steps complete. *(Unit test scope: assert the counts only. RSM evaluation of `GameOverReason::Draw` when both counts reach the threshold is verified in RSM integration tests — see RSM Rule 14.)* | BLOCKING |
| OS-11 | **GIVEN** both of a player's fake objectives are destroyed and both rewards are `ManaCapIncreased`, **WHEN** mana_cap_effective is queried, **THEN** `mana_cap_effective = min(mana_cap_base + 2, mana_cap_max)` where `mana_cap_max = GameConfig.mana_cap + fake_count`. At defaults: `min(10 + 2, 12) = 12`. | BLOCKING |
| OS-12 | **GIVEN** a fake objective is destroyed, **WHEN** the D4 reward draw produces `ManaCapIncreased`, **THEN** `ManaCapIncreased { player: attacker_player, amount: 1 }` is emitted exactly once regardless of the current mana cap value. *(The ceiling no-op — that `mana_cap_effective` does not exceed `mana_cap_max` — is enforced by the Economy System and verified in Economy System ACs, not here.)* | BLOCKING |
| OS-13a | **GIVEN** an objective is destroyed by the opponent, **WHEN** the RESOLUTION-end sync fires, **THEN** `ObjectiveDestroyed { target_player_id, lane, was_fake: bool }` is emitted with the correct payload for the destroyed lane. *(Objective System unit test — event emission only.)* | BLOCKING |
| OS-13b | **GIVEN** `ObjectiveDestroyed` is received by Board Rendering, **WHEN** it processes the event, **THEN** the visual slot for that lane is no longer displayed. *(ADVISORY — Board Rendering scope. Screenshot evidence + lead sign-off.)* | ADVISORY |
| OS-14 | **GIVEN** `attacker_player == defending_player` (self-destruction via Punition or double-tranchant), **WHEN** a real objective is destroyed, **THEN** no gold is awarded, no fake rewards fire, and `real_objectives_destroyed(defending_player)` increments by 1. | BLOCKING |
| OS-15 | **GIVEN** a FreeCardPick reward fires for a player whose hand is at 10 cards, **WHEN** the consequence path processes the reward, **THEN** `draw_random()` is NOT called and `AwardGold { player, amount: 1 }` is emitted instead. | BLOCKING |
| OS-16 | **GIVEN** `take_damage(lane, attacker, 0)` is called, **WHEN** the system processes it, **THEN** HP is unchanged, no destruction check runs, and no events are emitted. | BLOCKING |
| OS-17 | **GIVEN** two connected clients (owner and attacker), **WHEN** the attacker's state for an opponent's intact objective is queried, **THEN** `ObjectiveHp` is present and the attacker does NOT receive `S2CObjectiveIdentities` for the opponent's lanes. *(ADR-001 resolved OQ4 — unicast architecture adopted. Test form: message dispatch assertion verifying no `S2CObjectiveIdentities` is sent to the attacker's `ClientId`. Remains ADVISORY because this is a two-client integration test requiring a live Lightyear session — not a `World::new()` unit test.)* | ADVISORY |
| OS-18a | **GIVEN** units at Cell 8 in both lane 1 and lane 3, each dealing lethal damage to the opponent's objective in the same RESOLUTION sub-step 6, **WHEN** `take_damage()` is processed, **THEN** the lane 1 consequence path fires before the lane 3 path (verified by event emission order in unit test). | BLOCKING |
| OS-18b | **GIVEN** two `take_damage()` calls targeting the same objective in sub-step 6 (same lane), **WHEN** both calls are processed, **THEN** the final `ObjectiveHp` value is correct (unit-testable) and the consequence path fires exactly once (unit-testable). *(Batching guarantee — that no intermediate HP replication update is sent to clients between calls — is a Lightyear transport property, not ECS state. Verified as a network integration test once the OQ4 ADR and sub-step scheduling contract are established. Currently ADVISORY.)* | ADVISORY |
| OS-19 | **GIVEN** a fake objective is destroyed with a known server-side RNG seed producing `gen_range(0..2) = 0`, **WHEN** the reward draw executes, **THEN** `ManaCapIncreased` is emitted (not `FreeCardPick`). **GIVEN** a seed producing `gen_range(0..2) = 1`, **THEN** `FreeCardPick` is emitted (not `ManaCapIncreased`). *(Requires seeded `ChaCha` RNG in test — not live randomness.)* | BLOCKING |
| OS-20 | **GIVEN** an objective at HP = 3 and two sequential `take_damage(lane, attacker, 5)` calls in the same sub-step, **WHEN** both calls are processed, **THEN** HP = 0, the consequence path fires exactly once (one `AwardGold`, one `ObjectiveDestroyed`), and the second call is a no-op. | BLOCKING |
| OS-21 | **GIVEN** `attacker_player == defending_player` (self-destruction via Punition or double-tranchant) and the destroyed objective is a **fake**, **WHEN** the consequence path runs, **THEN** `fake_objectives_destroyed(defender)` is unchanged, no `AwardGold` is emitted, and no mana cap or card reward fires. The slot is marked destroyed and `ObjectiveDestroyed` is broadcast. | BLOCKING |
| OS-22 | **GIVEN** a FreeCardPick reward fires and `draw_random(PoolFilter, seed)` returns `None` (pool exhausted), **WHEN** the consequence path processes the reward, **THEN** no card is granted, no gold is emitted, and no re-roll to mana cap occurs. *(Distinct from OS-15: pool-exhausted → no-op; hand-full → +1g.)* | BLOCKING |
| OS-23a | **GIVEN** a game session initialization attempt with `fake_count > lane_count - loss_threshold` (e.g., `fake_count = 4`, `lane_count = 5`, `loss_threshold = 2`), **WHEN** the server evaluates the upper-bound config invariant, **THEN** the session is refused with an error before LOBBY state is entered. *(Config guard — verify via startup assertion test with `GameConfig { fake_count: 4, .. }`.)* | BLOCKING |
| OS-23b | **GIVEN** a game session initialization attempt with `objective_hp = 0` OR `fake_count = 0` in GameConfig, **WHEN** the server evaluates the lower-bound config invariants, **THEN** the session is refused with an error before LOBBY state is entered. *(Two separate startup assertions; test each independently.)* | BLOCKING |
| OS-24 | **GIVEN** Sang Méprise is active during a RESOLUTION in which a fake objective is also destroyed, **WHEN** the consequence path executes and RESOLUTION-end sync fires, **THEN** `ObjectiveDestroyed { was_fake: true }` is broadcast to both players exactly once — Sang Méprise visibility does not suppress or duplicate the authoritative destruction event. | BLOCKING |
| OS-25 | **GIVEN** Garde-Temps (Xelor Krosmic) targets an intact objective at HP = 3 (below `objective_hp`), **WHEN** the effect resolves as `take_damage(lane, attacker_player, objective_hp)` (damage = `objective_hp` = 5), **THEN** HP = 0 and the full consequence path fires (gold, fake rewards if applicable, `ObjectiveDestroyed` broadcast). *(Confirm lethal-damage routing through the standard interface when Class System GDD is authored — see OQ1.)* | BLOCKING |
| OS-26 | **GIVEN** both fake objectives are destroyed in the same RESOLUTION with one D4 draw producing 0 (ManaCapIncreased) and the other producing 1 (FreeCardPick), **WHEN** the consequence path processes both rewards, **THEN** `ManaCapIncreased { player, amount: 1 }` is emitted once AND `draw_random(PoolFilter, seed)` is called once (for the FreeCardPick path). *(Requires two seeded ChaCha draws producing results 0 and 1.)* | BLOCKING |
| OS-27 | **GIVEN** a FreeCardPick reward fires and the attacker's hand is below max capacity, **WHEN** `draw_random` is called, **THEN** it is called with `PoolFilter { rarity: None, class: None, card_type: None, max_cost: None }` — all fields None. *(Verifies the filter contract, not the draw outcome. Inspect call arguments in test.)* | BLOCKING |
| OS-28 | **GIVEN** a game session initializes with `fake_count = 1` in GameConfig, **WHEN** DRAFT_INITIAL runs, **THEN** exactly one fake lane is assigned per player, the remaining 4 lanes are real, all 5 objective slots initialize with HP = `objective_hp`, and `real_objectives_destroyed(player) = 0` for all players. | BLOCKING |

## Open Questions

1. **Garde-Temps damage amount** — Garde-Temps "destroys a dofus" (Xelor Krosmic, 20 reserve mana). Treated here as `take_damage(lane, attacker_player, objective_hp)`. If Garde-Temps instead operates as an instant-kill flag bypassing the HP formula, the consequence path contract must be updated. Confirm when the Class System GDD is authored.

2. **FreeCardPick pool filter** — `draw_random(PoolFilter { all None }, seed)` draws uniformly from all available cards. The specific filter (rarity cap? TFT weighting?) is deferred to the Card Acquisition GDD. Confirm whether the draw should apply TFT weighting or be uniform.

3. **Fake reveal animation beat** — Visual treatment (timing, animation, particle effect) is deferred to the Board Rendering GDD. However, the Reveal Moment subsection above specifies the mandatory constraints (500ms minimum hold, distinct two-beat animation) that Board Rendering must implement.

4. **Lightyear replication strategy for `ObjectiveIdentity`** (**RESOLVED — see ADR-001**) — Technical spike (2026-04-29) confirmed: Lightyear 0.26 visibility primitives operate at entity granularity only. Per-component replication scope on a single entity is **not supported** as a first-class API. **Chosen architecture (ADR-001):** `ObjectiveIdentity` is never a replicated ECS component. The server sends `S2CObjectiveIdentities { identities: Vec<(LaneId, is_fake)> }` as a targeted reliable unicast to the owning player only at `DRAFT_INITIAL`. Server holds authoritative identity in a non-replicated `HiddenObjectives` resource. On reconnect, server re-sends `S2CObjectiveIdentities` to the reconnecting client. See `docs/architecture/adr-001-objective-identity-unicast.md` for full decision record, Rust API pattern, and implementation notes.

5. **Sang Méprise replication mechanism**: **Safe resolution: Option B (authoritative — do not implement Option A)** — send a targeted reliable unicast `S2CSangMepriseReveal { identities: Vec<(LaneId, is_fake)> }` to the opponent only; replication scope remains unchanged. The reveal persists in client local state for the RESOLUTION duration. Requires ADR to formalize the pattern. **Open gap (reconnect):** `S2CSangMepriseReveal` is not in `S2CGameSnapshot` — see Sang Méprise edge case for required Network Protocol GDD fix (snapshot must include `active_sang_meprise_identities: Option<Vec<(LaneId, is_fake)>>` or server must re-send reveal after reconnect snapshot if Sang Méprise is active).
