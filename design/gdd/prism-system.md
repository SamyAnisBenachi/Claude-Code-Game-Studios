# Prism System

> **Status**: Designed — /design-review pending (fresh session)
> **Author**: User + Agents
> **Last Updated**: 2026-04-30
> **Implements Pillar**: No idle spectating · Deep emergence · Simple surface

## Overview

The Prism System owns the per-lane reward tokens that turn the spawn cell into a contested resource. Each player holds 5 prisms — one at their own spawn cell of each lane — and collects them by ending a unit's standard movement at that cell. Collection delivers a lane-specific reward: Lane 1/5 grant a "1 damage to a chosen objective" spell card to the hand (3 mana to play), Lane 2/4 grant a "+1 mana to reserve" spell card, and Lane 3 draws one random card from the shared shop pool. Once a player has collected all 5 of their own prisms, that player's full set respawns; the opponent's collection pace does not affect this cycle.

Mechanically the system is a thin reward dispatcher: it consumes the `PrismCollected(player, lane)` message that Board / Lane System emits in RESOLUTION sub-step 5, runs the Lane→reward routing, and emits the corresponding hand-add or random-draw call into Card Data & Pool, plus a per-player respawn check. Lane 3's draw consumes one server-side seed via `ServerRng::next_seed()` (event_type = `"draw_random"`), with execution order locked by ADR-005's `ResolvePrism` slot in the RESOLUTION schedule (`apply_placement_effects → resolve_ecaflip_triggers → resolve_prism_draws → award_fake_objective_rewards`). The system itself owns no formulas or balance values beyond two tuning constants — it is the connector between board geometry and the economy/hand layers, sitting under "No idle spectating" (every lane stays decision-relevant) and "Deep emergence" (WALL-parking, prism races, and lane priority emerge without explicit teaching).

## Player Fantasy

The fantasy is colonial-economic: **you don't just play the lanes — you own them, and they pay you rent.** Round 6. Lane 2 has been a stalemate for three rounds; you've stopped pushing units into the meat grinder and parked a WALL on your spawn cell instead. Every round the WALL sits there, immovable, MP=0, and a "+1 mana to reserve" spell card lands in your hand like clockwork. Your opponent can't shut it off without committing real units to a lane they've already conceded ground on — and the moment they do, your other four lanes get easier. That is the loop the system serves: a lane you stopped fighting in does not become dead space, it becomes a private tax stream.

The cunning move is choosing **which** lane to convert to revenue. Lane 1 or 5 prints "1 damage to chosen objective" cards — small but bypass-everything, and three of them is a real objective gone. Lane 3 prints raw card draw, the closest thing to a deck-builder ramp this game has. Lanes 2 and 4 print reserve mana, the currency that lets a Xelor's *Garde-Temps* or your own seventh-round combo come online a turn early. You watch your opponent's spawn cells too — every round they don't claim their own prism is a round their tax meter sits idle, and the moment you see a WALL go down on their side, you know exactly what kind of long game they're building.

Two pillars pay this off. **No idle spectating** — even a lane that has gone quiet has live tax flowing through it; abandonment is never neutral, only revenue or refusal. **Deep emergence** — WALL-parking, lane abandonment-as-strategy, prism races, and the per-player respawn cycle are never explained in any tutorial card; players discover them by watching the board do its work, then build whole strategies around being the kind of player who notices.

## Detailed Design

### Core Rules

**Rule 1 — System ownership.** The Prism System owns:
- A `PrismState` resource: `collected[lane: 1..=5][player: PlayerId] -> bool`. `false` = prism present and collectible; `true` = collected, not yet respawned. Naming aligns with `PrismBoardState.collected: bool` in network-protocol.md.
- Two static card definitions in `assets/data/cards.json`: `prism_strike` (Spell, cost 3 mana, target: `TargetObj { player_id, lane }`, effect: "deal 1 damage to chosen objective", bypasses lane position; self-targeting own objectives is legal — see Edge Cases) and `prism_reserve` (Spell, cost 0 mana, target: `Instant`, effect: `add_reserve(player, 1)`; playable during DRAFT phase only — see Rule 13).

The Prism System does NOT own: hand storage (writes via Card Acquisition's hand-mutation API), Lane 1/5 spell damage routing (Objective System resolves the spell when played), Lane 2/4 reserve grant (Economy System resolves the spell when played), or board geometry/collection trigger (Board / Lane System emits `PrismCollected`).

**Rule 2 — Initial state.** At session start: `collected[lane][player] = false` for all 5 lanes × all players. All 10 prism tokens are present at game start.

**Rule 3 — Collection trigger contract.** The system consumes `PrismCollected(player, lane)` messages emitted by Board / Lane System during RESOLUTION sub-step 5. In Bevy 0.18, `PrismCollected` is a `#[derive(Message)]` type (not an `Event`). Board / Lane System emits via `MessageWriter`; `resolve_prism_draws` consumes via `MessageReader`. The message buffer persists until drained in `resolve_prism_draws`. Receipt requires `collected[lane][player] == false`; if `true`, the message is a stale duplicate — silently discarded with a server-side warning log. (Defensive guard. Board / Lane System should never emit on a collected prism, but the check protects the audit log.)

**Rule 4 — Reward routing per lane (deterministic lanes 1/2/4/5).** On a valid event for Lane L ∈ {1, 2, 4, 5}: set `collected[L][player] = true`, then call hand-add for the appropriate static card:
- L = 1 or 5 → `prism_strike`
- L = 2 or 4 → `prism_reserve`

Hand-add invokes the same hand-write API used by Card Acquisition (e.g., `HandState::push(player, card_id)` — exact API name pending architecture confirmation, see OQ3). On successful add, broadcast `S2CCardAcquired { card_id, source: PrismLane{L} }` (reliable, unicast to owning player). On hand-full rejection, see Rule 7 — no broadcast, no error.

Lanes 1/2/4/5 consume **zero** RNG seeds.

**Rule 5 — Reward routing (Lane 3, RNG draw).** On a valid event for Lane 3:
1. **Hand-full pre-check**: if `hand[player].len() >= 10`, set `collected[3][player] = true`, do NOT call `next_seed()`, do NOT broadcast. The prism is still consumed.
2. Otherwise: call `ServerRng::next_seed()` (event_type `"draw_random"`), call `CardDataPool::draw_random(filter, seed)` where `filter = PoolFilter { card_type: Some(Minion | Spell), class: None, rarity: None, max_cost: None }` over the player's per-player pool.
   - On `Some(card_id)`: call `distribute(card_id)`, hand-add, broadcast `S2CCardAcquired { card_id, source: PrismLane3 }`. Audit log: `("draw_random", seed_index, Some(card_id))`.
   - On `None` (pool exhausted): no hand add, no broadcast. Audit log: `("draw_random", seed_index, None)`. Seed IS consumed.
3. Set `collected[3][player] = true`.

**Rule 6 — Inter-player and inter-lane ordering.** When multiple `PrismCollected` events are pending for the same RESOLUTION, `resolve_prism_draws` processes them in:
1. Ascending `player_id`
2. Within a player: ascending lane index (1 → 5)

This ordering applies to all reward delivery, audit log writes, `collected[]` mutations, and `S2CCardAcquired` broadcast emission. It is required for determinism (per ADR-005 / server-rng.md Rule 6).

**Rule 7 — Hand-full handling.** Before any hand add (Rule 4 or Rule 5 step 2), check `hand[player].len() < 10`. If hand is full:
- Lanes 1/2/4/5: spell card silently dropped. Prism state is still updated (`collected = true`). No broadcast. No refund. No queue.
- Lane 3: short-circuit at Rule 5 step 1 — seed is NOT consumed, prism is still consumed.

Rationale: a full hand signals strategic abundance — the organic ceiling on WALL-park farming. Skipping seed consumption on Lane 3 hand-full keeps the audit log semantically clean (a `draw_random` entry means a card actually entered play). Non-deferred, non-refunded reward is consistent with auction "binding bid" model (you committed the unit's MP toward the spawn cell; that was the cost).

**Rule 8 — Respawn detection.** After all `PrismCollected` events for the current RESOLUTION have been processed in Rule 6 order, check each player: if `count(collected[lane][player] == true for lane in 1..=5) == 5`, mark `pending_respawn[player] = true`. Do NOT mutate `collected[]` yet.

**Rule 9 — Respawn timing.** At the END of `resolve_prism_draws`, after all reward broadcasts have been issued, for each player where `pending_respawn[player] == true`: set `collected[lane][player] = false` for all 5 lanes. Reset `pending_respawn[player] = false`. The `PrismPresence` component replication (unreliable channel, per network-protocol.md) propagates to clients on the next frame.

A unit that ended sub-step 5 at the spawn cell of a lane that is being respawned in this same RESOLUTION does NOT collect the just-respawned prism — collection events were already processed in Rule 6 before respawn fires. Respawned prisms become collectible starting the **next** RESOLUTION's sub-step 5.

**Rule 10 — Respawn delivers no reward.** Respawn is a pure state reset: no card, no gold, no `S2CCardAcquired`. Clients infer the new prism presence from `PrismPresence` component replication. No distinct "prism respawned" reliable message is sent.

**Rule 11 — Prisms grant no gold.** The Prism System never calls into the Economy System for gold awards. The `GoldAwardReason::PrismReward` enum variant present in `network-protocol.md` (line 383) is unused and should be removed in a forthcoming Network Protocol revision (see OQ2).

**Rule 12 — Lane symmetry is intentional.** Lanes 1 and 5 are mirrors (both grant `prism_strike`). Lanes 2 and 4 are mirrors (both grant `prism_reserve`). Lane 3 is unique (random draw). This symmetry is by design — the board has axial reflection over the central lane, and prism rewards reinforce that axis. It is not redundancy.

**Rule 13 — Spell card play-phase constraint.** `prism_reserve` and `prism_strike` cards may be played from hand at any point during the DRAFT phase only. Attempting to play either card during PLACEMENT or RESOLUTION is rejected by the card-play validation system (owned by Card Acquisition / Round State Machine). On rejection: no state mutation occurs, no mana is deducted, no error message is sent to the client — the play attempt is silently discarded server-side.

### States and Transitions

| State | Meaning | Storage |
|---|---|---|
| `Present` | `collected[lane][player] == false`. Token visible at the spawn cell; collectible. | Server-authoritative `PrismState` resource; replicated to clients via `PrismPresence` component |
| `Collected` | `collected[lane][player] == true`. Token absent; awaiting respawn. | Same |
| `pending_respawn[player]` | Transient flag set at end of Rule 6, cleared at end of Rule 9. | Local to `resolve_prism_draws` execution scope |

| From | Trigger | To | When |
|---|---|---|---|
| *(Uninitialized)* | Session start — Rule 2 initialization | `Present` (all 10 tokens, all players) | Before first RESOLUTION; never changes except via `resolve_prism_draws` |
| `Present` | Valid `PrismCollected(p, l)` consumed by Rule 4 or Rule 5 | `Collected` | Within `resolve_prism_draws` |
| `Collected` (any lane, player p) | `count(Collected for p) == 5` after all reward delivery | `pending_respawn[p] = true` | End of Rule 6 reward loop |
| `Collected` (all lanes for player p) | Respawn fires (Rule 9) | `Present` (all 5 lanes) | End of `resolve_prism_draws`, after broadcasts |

Invariant: `Collected` state persists across all phases and rounds until `prism_respawn_due(player)` returns `true`. No phase transition (DRAFT, PLACEMENT) mutates prism state.

`Present` and `Collected` persist across DRAFT, PLACEMENT, and PLACEMENT phases — only `resolve_prism_draws` mutates them.

### Interactions with Other Systems

| System | Direction | Interface |
|---|---|---|
| **Board / Lane System** (Approved) | Board → Prism (input) | Emits `PrismCollected(player, lane)` during sub-step 5 when a unit ends standard movement at its own player's spawn cell. Board owns the cell-position check; Prism System trusts the event. |
| **Card Data & Pool** (Approved) | Prism → Pool (Lane 3 only) | `draw_random(PoolFilter { card_type: Some(Minion \| Spell), class: None, rarity: None, max_cost: None }, seed) -> Option<CardId>`; on `Some`, follow with `distribute(card_id)`. Lanes 1/2/4/5 read static `prism_strike` / `prism_reserve` definitions but call no Pool API. |
| **Card Acquisition** (Approved) | None — explicitly bypassed | Per `card-acquisition.md` line 80 contract, prism rewards bypass Card Acquisition entirely. Prism System writes to hand directly using the same hand-mutation API Card Acquisition uses internally (exact API name TBD per OQ3). Hand-full check is owned by Prism System (Rule 7). |
| **Economy System** (Approved) | Indirect — via played card | Prism System adds `prism_reserve` to hand (Lane 2/4); when the player later plays it, the spell's effect calls `add_reserve(player, 1)`. Economy System owns reserve grant; Prism System owns delivery of the spell card only. |
| **Objective System** (Approved) | Indirect — via played card | Prism System adds `prism_strike` to hand (Lane 1/5); when the player later plays it with a `TargetObj { player_id, lane }` target, Objective System applies `objective_damage(HP, 1) = max(0, HP - 1)`. Spell bypasses lane position (per AC P6). |
| **Server-side RNG** (Approved) | Prism → RNG (Lane 3 only) | One `next_seed()` call per Lane 3 collection where hand is not full. Audit log entry per ADR-005 / server-rng.md Section 8: `event_type = "draw_random"`, result = `Some(card_id)` or `None`. Lanes 1/2/4/5 consume no seeds; Lane 3 hand-full short-circuits before consuming a seed. |
| **Round State Machine** (Approved) | RSM → Prism (schedule) | `resolve_prism_draws` runs only during the RESOLUTION phase, in the schedule slot defined by ADR-005 / server-rng.md Rule 5: `apply_placement_effects → resolve_ecaflip_triggers → resolve_prism_draws → award_fake_objective_rewards`. |
| **Network Protocol** (Approved) | Prism → Network | `S2CCardAcquired { card_id, source: PrismLane{1..5} }` — reliable unicast to owning player on each successful hand add. The `S2CCardAcquired` message and its `source` enum already exist in `network-protocol.md` (line 396–400) — formal payload schema confirmation pending (see OQ4). `PrismPresence` component replication (unreliable channel) carries `collected: bool` per (player, lane). |
| **Board Rendering** (Designed) | Prism → Rendering (indirect) | Rendering reads `PrismPresence` replicated state for token visibility and consumes `PrismCollected` via the resolution event log for collection VFX. No direct interaction. |
| **Class System** (Not Started) | Forward-looking | Xelor reserve-mana ramp stacks normally with Lane 2/4 grants — a Xelor player who collects both Lane 2 and Lane 4 holds two `prism_reserve` cards, plays them for +2 reserve, then layers Gelure / Miss Nuit on top. No special-case rules. |

## Formulas

### F1 — Prism Respawn Condition

The `prism_respawn_due` formula is defined as:

`prism_respawn_due(player) = (count(collected[lane][player] == true for lane in 1..=5) == 5)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Lane index | `lane` | u8 | 1–5 | Iteration index over the 5 board lanes |
| Player ID | `player` | PlayerId | session-scoped | Identifies which player's prism set is being checked |
| Collected flag | `collected[lane][player]` | bool | {false, true} | `true` = prism collected, awaiting respawn; `false` = prism present |

**Output Range:** `false | true`. `true` triggers a single respawn pass at end of `resolve_prism_draws` (Rule 9), which then sets `collected[lane][player] = false` for all 5 lanes of the player.

**Worked example:**
- After round 4: `collected[1..=5][player_a] = [true, true, false, true, true]`. `prism_respawn_due(player_a) = (4 == 5) = false`. No respawn.
- After round 7, a Lane 3 collection completes the set: `collected[1..=5][player_a] = [true, true, true, true, true]`. `prism_respawn_due(player_a) = true`. Respawn fires at end of `resolve_prism_draws`; `collected[1..=5][player_a] = [false, false, false, false, false]`. All 5 prisms available again starting next RESOLUTION's sub-step 5.

### Referenced operations (owned by other GDDs)

The Prism System triggers but does not own the following math-touching operations — they are listed for completeness:

| Operation | Owned by | Triggered by | Output |
|---|---|---|---|
| `objective_damage(HP, 1)` | `objective-system.md` (registry: formula `objective_damage`) | Played `prism_strike` spell (Lane 1/5) targeting `TargetObj { player_id, lane }` | `HP_new = max(0, HP - 1)` — exactly 1 damage to chosen objective |
| `add_reserve(player, 1)` | `economy-system.md` (API, not a formula) | Played `prism_reserve` spell (Lane 2/4) | `reserve_mana[player] += 1` — uncapped per economy-system.md Rule 3 |
| `draw_random(filter, seed)` | `card-data-pool.md` (API, not a formula) | Lane 3 collection with `hand[player].len() < 10` | `Option<CardId>` — uniform pick over distinct eligible cards matching filter; consumes 1 server seed |

The Prism System itself owns no balance formulas beyond F1. Two tuning constants are listed in Section G — Tuning Knobs.

## Edge Cases

**If a `PrismCollected(player, lane)` event arrives with `collected[lane][player] == true`** (stale duplicate): event is silently discarded with a server-side warning log. No state mutation. Defensive guard — Board / Lane System should never emit on a collected prism.

**If Player A's hand has exactly 10 cards when their unit collects a Lane 1/2/4/5 prism**: spell card silently dropped. `collected[lane][player]` is still set to `true`, the `collected_this_round[player]` counter still increments, no `S2CCardAcquired` is broadcast, no refund. The prism is consumed; the reward is not delivered. Rationale: a full hand signals strategic abundance — the organic ceiling on WALL-park farming.

**If Player A's hand has exactly 10 cards when their unit collects a Lane 3 prism**: hand-full pre-check (Rule 5 step 1) short-circuits. `next_seed()` is NOT called, `seed_index` does NOT advance, no `draw_random` audit log entry is appended, no card is added, no broadcast. `collected[3][player]` is still set to `true`. Rationale: consuming a seed for an undeliverable card pollutes the audit log and incorrectly mutates `copies_remaining` via `distribute()`. Skipping the seed keeps the audit semantically clean — a `draw_random` entry means a card actually entered play.

**If Lane 3 `draw_random` returns `None`** (eligible pool exhausted): seed IS consumed, audit log appends `("draw_random", seed_index, None)`, no hand add, no `S2CCardAcquired` broadcast. `collected[3][player]` is set to `true`. The player receives no card and no notification.

**If Player A's units collect prisms on lanes 1, 3, and 5 in the same RESOLUTION**: `resolve_prism_draws` processes them in ascending lane order (Rule 6). Lane 1 emits `S2CCardAcquired { source: PrismLane1 }` first; Lane 3 then consumes one seed and emits `S2CCardAcquired { source: PrismLane3 }`; Lane 5 emits `S2CCardAcquired { source: PrismLane5 }` last. Audit log records the Lane 3 entry between the two deterministic events.

**If Player A and Player B both collect a Lane 3 prism in the same RESOLUTION**: per Rule 6 inter-player ordering, Player A (lower `player_id`) processes first, consuming seed N; Player B consumes seed N+1. Audit log entries appear in that order. Both `S2CCardAcquired` broadcasts are sent, each unicast to its owning player.

**If Player A collects their 5th prism in the same RESOLUTION as Player B collects their 1st prism**: Player A's respawn (Rule 9) fires at end of `resolve_prism_draws`, after both reward broadcasts. Player B's respawn does NOT fire (count = 1, not 5). Each respawn cycle is independent (locked by master GDD AC P5).

**If Player A had collected lanes 1–4 in prior rounds and a Lane 5 collection completes their set this RESOLUTION**: the 5th `PrismCollected` is processed normally (reward delivered, `collected[5][player_a] = true`); Rule 8 then sets `pending_respawn[player_a] = true`; Rule 9 resets all 5 lanes to `false` at end of `resolve_prism_draws`. The just-collected Lane 5 IS included in the respawn — it flips back to `false` along with the others.

**If a unit ends sub-step 5 at the spawn cell of a lane that is being respawned in this same RESOLUTION**: the unit does NOT collect the just-respawned prism. Reward delivery (Rule 6) processes all `PrismCollected` events first; respawn (Rule 9) fires only after reward delivery is complete. Respawned prisms become collectible starting the next RESOLUTION's sub-step 5.

**If a unit reaches the prism cell via TELEPORT, REPEL, ATTRACT, or CHANGE LANE** (not standard sub-step 5 movement): no `PrismCollected` is emitted by Board / Lane System (locked by board-lane-system.md Rule 11 + edge cases). Prism state is unchanged.

**If a unit reaches the prism cell via CHARGE X** (sub-step 2 bonus movement): no `PrismCollected` is emitted — collection is gated to sub-step 5 standard movement only. A unit that ends sub-step 5 at the prism cell after a CHARGE-X-and-MP combined movement DOES collect, since sub-step 5 is the relevant trigger.

**If a Player A WALL unit (MP=0) is parked at cell 1 of lane N**: every RESOLUTION's sub-step 5, the unit ends at cell 1. While `collected[N][player_a] == false`, every RESOLUTION fires `PrismCollected(player_a, N)` and delivers the lane-N reward. Once collected, no event fires until that prism respawns (i.e., until Player A collects all 5 of their own). This WALL-park farming is the canonical "tax stream" loop — locked by master GDD §3.4 + board-lane-system.md edge cases.

**Counterplay for WALL-parking:** The expected counterplay is combat at the spawn cell. A WALL must have low HP/AR (its role is positional value, not combat durability) so that an opponent unit arriving at cell 1 destroys it in one RESOLUTION. An opponent who routes one unit into the WALL's lane forces a choice: the WALL farmer loses the prism source permanently (until they redeploy a new unit) OR they spend their own unit's slot to fight the threat. The balance mechanism is asymmetric only if the opponent unit's travel time is too long — this should be validated in Combat Resolution GDD (unit HP/AR of WALL archetype). Note that the WALL's card cost (drafted/auctioned) and its permanent consumption of one Minion slot across multiple rounds ARE the primary opportunity cost, not the opponent's ability to evict it. WALL-parking in a lane the opponent does not contest is an accepted dominant line; the system does not require active countermeasures in every lane simultaneously.

**If a Player B unit reaches cell 1** (Player A's prism cell): no `PrismCollected` is emitted for Player B — prisms are owned by the player whose spawn cell they occupy. Player B's prism for that lane is at cell 8, not cell 1. Locked by board-lane-system.md AC BL-13.

**If two teammates in 2v2 both have units that end sub-step 5 at their shared spawn cell of the same lane**: each player has their own prism at that cell (per master GDD §3.4 — "each player tracks their own 5 prisms independently"). Both `PrismCollected` events fire — one for each player. Prism state is keyed on `(player_id, lane)`, not `(team_id, lane)`. Both rewards are delivered (each unicast to its owning player); audit log orders them by ascending `player_id`.

**If the server crashes mid-RESOLUTION between prism collection and respawn**: per `server-rng.md` Rule 7, the entire session is forfeit and clients receive a disconnect notification. There is no partial-RESOLUTION recovery. On the next session, RNG starts fresh and prisms initialize per Rule 2.

**If a player reconnects mid-game**: the reconnect snapshot (`S2CGameSnapshot.PrismBoardState { lane, collected }`) carries the authoritative `collected: bool` per (player, lane) pair. Per network-protocol.md Rule 2, this is component replication — not re-sent on reconnect, the snapshot includes it. Client board view repopulates accordingly.

**If RNG seed allocation changes** (e.g., a future spec adds a new seed-consuming step before `resolve_prism_draws`): the audit log seed_index counter shifts, but the Prism System's logic is unaffected. Determinism is preserved within any single session.

## Dependencies

### Upstream Dependencies

| System | Status | Type | Interface | Notes |
|---|---|---|---|---|
| **Board / Lane System** | Approved | Hard | Emits `PrismCollected(player, lane)` during RESOLUTION sub-step 5; owns `PrismPresence` per-lane component replication | Trigger contract is locked in Rule 11 + edge cases. Prism System trusts the event — no re-validation of cell position. |
| **Card Data & Pool** | Approved | Hard | `draw_random(filter, seed) -> Option<CardId>` for Lane 3; `distribute(card_id)` to mutate `copies_remaining`; static `prism_strike` and `prism_reserve` card definitions read from `assets/data/cards.json` | Lane 3 uses `PoolFilter { card_type: Some(Minion \| Spell), class: None, rarity: None, max_cost: None }` over the player's per-player pool. |
| **Server-side RNG** | Approved | Hard (Lane 3 only) | `ServerRng::next_seed()` per Lane 3 collection where hand is not full; audit log entry `("draw_random", seed_index, result)` | ADR-005 schedule slot: `apply_placement_effects → resolve_ecaflip_triggers → resolve_prism_draws → award_fake_objective_rewards`. Lanes 1/2/4/5 do NOT consume seeds. |
| **Round State Machine** | Approved | Hard | Drives RESOLUTION phase entry; `resolve_prism_draws` registered only on RESOLUTION schedule | No DRAFT or PLACEMENT interaction. |
| **Network Protocol** | Approved | Hard | `S2CCardAcquired { card_id, source: PrismLane{1..5} }` reliable unicast on each successful hand add; `PrismPresence { collected: bool }` component replication | Source enum already exists (line 396–400). `S2CCardAcquired` payload schema confirmation is OQ4. |
| **Game Config** | Approved | Soft | Two tuning knobs: `prism_strike_damage` (default 1) and `prism_strike_mana_cost` (default 3) — both read at session start | See Section G. Knobs in `assets/config/game_config.ron`. |
| **Card Acquisition** | Approved | None | Explicitly bypassed (per `card-acquisition.md` line 80) | Hand-mutation API (which Card Acquisition owns internally) is shared — Prism System must call the same API to avoid dual-ownership of the hand vector. Exact API name is OQ3. |

### Downstream Dependents

| System | Status | Type | Interface | Notes |
|---|---|---|---|---|
| **Economy System** | Approved | Indirect | When a player plays a `prism_reserve` spell card from hand, the spell's effect calls `add_reserve(player, 1)` | Economy System owns the reserve grant; Prism System owns delivery of the spell card. Lane 2/4 → `prism_reserve` → reserve pool. |
| **Objective System** | Approved | Indirect | When a player plays a `prism_strike` spell card with a `TargetObj { player_id, lane }` target, Objective System applies `objective_damage(HP, prism_strike_damage)` | Spell bypasses lane position (master GDD AC P6). Per-objective damage of 1 (default `prism_strike_damage`). |
| **Board Rendering** | Designed | Soft | Reads `PrismPresence` component for prism token visibility; consumes `PrismCollected` via the resolution event log for collection burst VFX | No direct interaction with Prism System internals. |
| **Hand UI** | Designed | Soft | Renders `prism_strike` and `prism_reserve` cards in the hand fan with appropriate visuals (cost badge, target type, effect text) | New cards must be added to the hand-card asset set. |
| **HUD** | Not Started | Soft | May display prism collection progress (`5 - count(collected[lane][player] == true)`) as a respawn counter; may display opponent's prism state | Not blocking — HUD GDD will define what it actually displays. |
| **Class System** | Not Started | Soft | Xelor reserve-mana ramp stacks normally with Lane 2/4 grants | No special-case rules. |
| **Card Animations** | Not Started | Soft | Tweens for: prism collection burst, prism token respawn fade-in, `prism_strike` projectile to objective, `prism_reserve` reserve-bar ping | No blocking interface; visuals are Card Animations' concern. |

The bidirectional consistency check: Board / Lane System lists Prism System under "Downstream Dependents" with the matching event `PrismCollected(player, lane)` (board-lane-system.md line 307). Card Data & Pool lists prism Lane 3 as a use case for `draw_random` (card-data-pool.md). Economy System lists Prism System under "Downstream (soft)" for the +1 reserve grant (economy-system.md line 315). Objective System's `objective_damage` formula is referenced for the Lane 1/5 spell. All upstream dependencies have matching back-references — no one-directional dependencies detected.

## Tuning Knobs

| Knob | Config Key | Default | Safe Range | Too Low | Too High |
|---|---|---|---|---|---|
| `prism_strike_damage` | `game_config.prism_strike_damage` | 1 | 1–3 | Below 1 is meaningless (always 0 after `max(0, HP - 0)`). At 1: three spells kill a fresh 5-HP objective — a meaningful but slow burn that rewards sustained lane presence. | At 3: two `prism_strike` cards plus any other damage sources eliminate a real objective quickly; edges toward "lane 1/5 prisms dominate all strategy." |
| `prism_strike_mana_cost` | `game_config.prism_strike_mana_cost` | 3 | 1–5 | At 1: the spell effectively costs one unit's worth of mana per damage; early rounds it competes directly with cheap unit placement. At 2: the spell outperforms combat on damage-per-mana for most early rounds. Floor is 1 — a cost of 0 eliminates all opportunity cost and is not a valid balance target. | At 4–5: the spell sits in hand unplayed for early rounds; players feel the prism-lane reward is taxed more than the farming cost justified. |

**Cross-referenced knobs (owned by other GDDs — not tunable here):**

| Knob | Value | Source GDD | Relevance to Prism System |
|---|---|---|---|
| `objective_hp` | 5 | game-config.md | Determines how many `prism_strike` casts are needed to destroy an undefended objective (default: 5 casts) |
| `hand_size_max` | 10 | (implicit, game-wide) | Drives the hand-full ceiling that caps WALL-park farming value; not a GameConfig field — the named constant `HAND_SIZE_MAX: usize = 10` serves as the canonical source of truth across all systems (Economy, Prism, Objective) |
| `lane_count` | 5 | board-lane-system.md | Structural constant; changing it requires a full Prism System redesign (5 prism types matched to 5 lanes) |

**Reserve mana accumulation model (Xelor / dual-WALL interaction):**

Reserve mana has no cap (by design — see economy-system.md OQ2 resolution). A player who WALLs Lanes 2 and 4 accumulates +2 reserve mana per completed 5-prism cycle. At maximum efficiency (WALL on all 5 lanes, one cycle per 5 rounds assuming all prisms are available): +2 reserve mana per 5 rounds from prisms alone. The Class System GDD must model the Xelor Garde-Temps (20 reserve cost) interaction explicitly and set a hard cap on Miss Nuit's per-round grant that accounts for prism contribution. Worked example: a non-Xelor player running dual-WALL with an 8-round game accumulates approximately +3–4 reserve from prisms (~1.5 cycles × 2 reserve/cycle). This is meaningful but not degenerate; the Minion slots consumed by 2 WALLs represent real opportunity cost. Xelor layering is the Class System's concern to model.

**Lane 3 frequency model (Legendary draw rate):**

Lane 3 uses `rarity: None` — all rarities are eligible. With pool sizes Common=6, Uncommon=5, Rare=4, Epic=1, Legendary=1 per player and a pool of approximately 250 Minion+Spell cards total across all rarities: expected Legendary frequency per draw ≈ 1/250 ≈ 0.4%. Across a 10-round game with one Lane 3 collection per respawn cycle (~2 cycles), expected Legendary draws via Lane 3 per game ≈ 0.8% — well below 1 per game. This rate is accepted as background RNG noise relative to the auction pathway (Legendaries available from round 6+ at bid 5g+). The draw rate increases in late game as pool depletes, but pool depletion itself means fewer eligible cards remain — the expected value of Lane 3 decreases, not increases, as the game progresses (depleted Commons/Uncommons mean less expected variety, not better cards).

## Visual/Audio Requirements

*Deferred to Art Bible phase. Board Rendering GDD (`board-rendering.md`) already specifies the prism token sprite (`env_prism_idle_32x32`, 32×32), idle spin animation, and collection burst VFX. Card Animations GDD will own tweens for `prism_strike` projectile and `prism_reserve` reserve-bar ping. Run `/asset-spec system:prism-system` after the Art Bible is approved.*

## UI Requirements

*No dedicated UI surface. Prism rewards are delivered as cards added to the Hand UI (owned by `hand-ui.md`). Any prism progress tracker or respawn counter is a HUD concern (`hud.md`). No action required here.*

## Acceptance Criteria

| ID | Criterion | Gate |
|---|---|---|
| PS-01 | **GIVEN** a player's unit occupies their own spawn cell in Lane 1 or Lane 5 at the end of RESOLUTION sub-step 5 AND the prism token for that lane is present, **WHEN** `resolve_prism_draws` runs, **THEN** exactly one `prism_strike` spell card is added to that player's hand and the prism token is marked collected. | BLOCKING |
| PS-02 | **GIVEN** a player's unit occupies their own spawn cell in Lane 2 or Lane 4 at the end of RESOLUTION sub-step 5 AND the prism token is present, **WHEN** `resolve_prism_draws` runs, **THEN** exactly one `prism_reserve` spell card is added to that player's hand and the prism token is marked collected. | BLOCKING |
| PS-03 | **GIVEN** a player's unit occupies their own spawn cell in Lane 3 at end of sub-step 5 AND the prism token is present AND the player's pool contains at least one Minion or Spell card, **WHEN** `resolve_prism_draws` runs, **THEN** exactly 1 card is drawn via `draw_random(filter=Minion\|Spell, seed)` and added to that player's hand, and exactly 1 server RNG seed is consumed. | BLOCKING |
| PS-04 | **GIVEN** a player has a `prism_reserve` spell card in hand, **WHEN** the player plays it during the DRAFT phase, **THEN** that player's reserve mana increases by exactly 1, the card costs 0 mana to play, and it may be played at any point during DRAFT. | BLOCKING |
| PS-05 | **GIVEN** a player has collected all 5 of their own prism tokens (in same or across multiple RESOLUTIONs), **WHEN** `resolve_prism_draws` finishes delivering all reward broadcasts for that RESOLUTION, **THEN** all 5 prism tokens for that player reset to `collected = false` at end of `resolve_prism_draws`, and the opponent's prism state is unchanged. | BLOCKING |
| PS-06 | **GIVEN** a player has a `prism_strike` spell card in hand, **WHEN** the player plays it targeting any objective (real or fake), **THEN** that objective takes exactly `prism_strike_damage` (default 1) damage, the spell costs exactly `prism_strike_mana_cost` (default 3) mana, and no lane position requirement applies. | BLOCKING |
| PS-07 | **GIVEN** a WALL unit (MP=0) is parked at a player's own spawn cell in any lane AND the prism token for that lane is present, **WHEN** RESOLUTION sub-step 5 completes (zero movement; unit remains at spawn cell), **THEN** the prism token is collected and the lane reward is granted — repeating every RESOLUTION as long as the token is present. | BLOCKING |
| PS-08 | **GIVEN** a unit arrives at a player's own spawn cell via TELEPORT, REPEL, ATTRACT, or CHARGE-X displacement (not sub-step 5 standard movement), **WHEN** `resolve_prism_draws` evaluates collection eligibility, **THEN** no prism token is collected and no reward is granted. | BLOCKING |
| PS-09 | **GIVEN** a player's hand contains exactly 10 cards AND the player's unit collects a prism in Lane 1, 2, 4, or 5, **WHEN** `resolve_prism_draws` attempts to add the spell card, **THEN** the spell card is silently dropped (not added to hand), no error is raised, the prism token IS still marked collected, and no replacement reward is granted. | BLOCKING |
| PS-10 | **GIVEN** a player's hand contains exactly 10 cards AND the player's unit collects a prism in Lane 3, **WHEN** `resolve_prism_draws` evaluates the collection, **THEN** `draw_random` is NOT called, no server RNG seed is consumed, no card is added to hand, and the prism token IS still marked collected. | BLOCKING |
| PS-11 | **GIVEN** a player's unit collects a prism in Lane 3 AND `draw_random(filter=Minion\|Spell, seed)` returns `None` (pool exhausted), **WHEN** `resolve_prism_draws` processes the result, **THEN** no card is added to hand, exactly 1 server RNG seed IS consumed, the audit log entry has `result: None`, and the prism token is marked collected. | BLOCKING |
| PS-12 | **GIVEN** `resolve_prism_draws` receives a `PrismCollected` event for a (player, lane) pair whose token is already marked collected (stale duplicate), **WHEN** the event is evaluated, **THEN** the event is discarded silently: no reward granted, no seed consumed, no error raised. | BLOCKING |
| PS-13 | **GIVEN** a player collects their 5th prism token in RESOLUTION N, **WHEN** `resolve_prism_draws` completes all reward broadcasts for RESOLUTION N, **THEN** the full respawn (all 5 tokens reset to `collected = false`) occurs AFTER the last broadcast — any unit at a spawn cell in that same RESOLUTION does NOT collect the freshly respawned token within RESOLUTION N. | BLOCKING |
| PS-14 | **GIVEN** a player's prism set respawns after full collection, **WHEN** the respawn state is inspected, **THEN** no additional reward (card, gold, mana, or otherwise) is granted by the respawn event itself — it is a state reset only. | BLOCKING |
| PS-15 | **GIVEN** any prism token is collected (any lane, any player), **WHEN** the player's resource totals are read after RESOLUTION, **THEN** the player's gold total is unchanged — prisms grant zero gold. | BLOCKING |
| PS-16 | **GIVEN** a 2v2 game where Player A and Player B are on the same team, **WHEN** Player A collects the Lane 3 prism keyed on `(player_A_id, lane_3)`, **THEN** Player B's prism token at `(player_B_id, lane_3)` is unaffected, and Player A's respawn cycle runs on Player A's individual count (0–5) independently. | BLOCKING |
| PS-21 | **GIVEN** Player A collects their 5th prism (triggering full respawn) AND Player B has collected 3 of 5 in the same RESOLUTION, **WHEN** `resolve_prism_draws` completes, **THEN** Player A's prisms all reset to uncollected, Player B retains 3 collected, and Player B's respawn does not trigger. | BLOCKING |
| PS-17 | **GIVEN** multiple players each have a unit eligible for prism collection in the same RESOLUTION, **WHEN** `resolve_prism_draws` processes all collections, **THEN** rewards are delivered in ascending `player_id` order, and within the same player, in ascending lane index order. | ADVISORY |
| PS-18 | **GIVEN** `GameConfig` is loaded with `prism_strike_damage = 2`, **WHEN** `prism_strike` is played, **THEN** the objective takes exactly 2 damage — confirming the value is read from config, not hardcoded. | ADVISORY |
| PS-19 | **GIVEN** `GameConfig` is loaded with `prism_strike_mana_cost = 0`, **WHEN** `prism_strike` is displayed and played, **THEN** the card shows 0 mana cost and playing it deducts 0 mana. | ADVISORY |
| PS-20 | **GIVEN** a unit collects a prism via valid sub-step 5 standard movement, **WHEN** collection is processed, **THEN** a `PrismCollected { player_id, lane }` event is emitted, providing an auditable trail for QA replay. | ADVISORY |
| PS-22 | **GIVEN** `prism_strike_damage` is set outside the documented safe range (< 1 or > 3) in `game_config.ron`, **WHEN** the server reads the config at startup, **THEN** the value is rejected with an explicit `expect()` message — no silent out-of-range value reaches gameplay. | ADVISORY |

*22 total: 17 BLOCKING, 5 ADVISORY. Every Core Rule from Section C (R1–R12) maps to at least one criterion.*

## Open Questions

| ID | Question | Owner | Status |
|---|---|---|---|
| OQ1 | **Hand-write API unification.** Prism System must write to hand using the same API as Card Acquisition to avoid dual-ownership of the hand vector. The exact call signature (`HandState::push(player, card_id)` vs. a Bevy message `AddCardToHand { player, card_id }`) is not yet decided. | Lead Programmer | Open — must resolve before Prism epic implementation |
| OQ2 | **`S2CCardAcquired` formal schema registration.** The message and its `source: PrismLane{1..5}` enum are referenced in `network-protocol.md` (lines 396–400) but not registered in `design/registry/entities.yaml` `network_messages` section, and the full payload `{ card_id: CardId, source: AcquisitionSource }` is not formally defined. Registration needed before the Network Protocol revision. | Network Protocol GDD / Lead Programmer | Open — NP GDD update required |
| OQ3 | **`GoldAwardReason::PrismReward` removal.** `network-protocol.md` line 383 contains a `PrismReward` variant in the `GoldAwardReason` enum with "amount defined in Prism System GDD." Prisms grant zero gold by design. The variant is unused and should be removed in the next NP GDD revision to avoid implementer confusion. | Network Protocol GDD | Open — minor housekeeping |
| OQ4 | **server-rng.md caller table conditional note.** The `draw_random` row in `server-rng.md` Rule 3 currently reads "1 per draw event." A conditional note is needed: "0 seeds consumed if player's hand is full at collection time (Prism Lane 3 only) — hand-full check short-circuits before `next_seed()` is called." Without this note, any audit replay tool that assumes fixed seed count per Lane 3 event will produce a misaligned log. | Server-side RNG GDD | Open — server-rng.md update required |
| OQ5 | **`prism_strike` self-targeting.** Can a player target one of their OWN objectives with `prism_strike`? Per `lanes-and-lies-gdd.md` §4.7, spell-based objective damage with self-targeting destroys the objective without +3 gold reward, and the destroyed objective counts toward the player's own loss condition. This is intentionally "double-tranchant" behavior — consistent with Sacrier's Punition. Confirm this is the intended interaction and document in Objective System or `prism_strike` card effect text. | Objective System / Game Design | Open — confirmation needed before NP finalises `TargetObj` validation |
