# Economy System

> **Status**: In Design (Under Revision)
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Auction as signature · No idle spectating · Deep emergence

## Overview

The Economy System manages the two independent currencies that drive all player decisions in Lanes and Lies: **mana** (spent to play cards each round) and **gold** (spent to acquire cards through the shop and auction). Mana is current-round only — it resets each DRAFT phase to `min(round_number, mana_cap)` and any unspent mana is discarded at round end. Gold persists across rounds and earns compound interest for hoarding. Every player maintains separate gold and mana pools; currencies are never shared between players. A third resource, **reserve mana**, is universal and persists between rounds — it serves as a carry mechanism available to all classes, though Xelor's cards interact with it most aggressively. The Economy System owns all currency state, all income calculations, and all spend-validation rules. It does not own card costs or auction prices — it enforces the player's ability to pay them.

## Player Fantasy

The gold counter is never idle. Every round, two voices argue: the *miser* says hold it — interest compounds, 10 gold becomes 12 becomes pressure that bends the auction — and the *gambler* says spend it, that Rare closes a lane *this round*. The fantasy is the squeeze of watching gold cross each threshold (5g, 10g) and deciding whether this card is worth breaking the streak.

And then the auction hits, and you realize your opponent was watching your gold the whole time.

Economy is not just a budget — it is a read. A player who has held 8 gold for two rounds is either disciplined or scared. A player who blew it at round 4 is either ahead or desperate. The gold totals are public during auctions, and the smart player uses that information before the first bid lands. The "game knows what I'm building" feeling comes not just from the shop weighting, but from the moment your opponent over-bids on a card you didn't want because they misread your gold as a bluff.

Mana adds a second texture: it resets every round, but Reserve turns leftover mana into savings. The satisfying moment is slotting that last 1 mana into Reserve after a tight play — the game noticing your efficiency without announcing it.

## Detailed Rules

### Core Rules

**1. Player economy state**

Each player holds three independent currency pools:

| Resource | Type | Persists between rounds? | Cap |
|---|---|---|---|
| `current_mana` | u32 | No — resets each DRAFT phase | `GameConfig.mana_cap` (default 10) |
| `reserve_mana` | u32 | Yes | None (no cap — organic board pressure is the intended limiter) |
| `gold` | u32 | Yes | None |

All three pools are per-player and independent. Currencies are never shared between players.

---

**2. Current-round mana**

**Gain:** At the start of each DRAFT phase, the server sets:
```
current_mana = min(round_number, mana_cap)
```

**Spend:** Players spend current mana by playing cards during DRAFT and PLACEMENT phases. Spend validation: `current_mana + reserve_mana >= card_cost` must be true before the server accepts a card play. Players allocate payment between current and reserve — see Rule 4.

**Discard:** All unspent `current_mana` is set to 0 at the end of RESOLUTION. It does not carry over.

**Exception — Gelure (Xelor spell):** Transfers all remaining `current_mana` to `reserve_mana` before the discard step — `reserve_mana += current_mana; current_mana = 0`. The player chooses when to play Gelure during DRAFT or PLACEMENT.

---

**3. Reserve mana**

Reserve is a universal mechanic available to all classes. Xelor's identity is not the reserve itself — it is that Xelor's class spells interact with reserve far more aggressively than any other class.

**Gain sources (all classes):**
- Playing the "+1 reserve" spell card from a Lane 2 or Lane 4 prism: `reserve_mana += 1`

**Gain sources (Xelor only):**
- Gelure: `reserve_mana += current_mana` (see Rule 2)
- Miss Nuit (passive): `reserve_mana += 1` each time the opponent plays a card
- Additional Xelor class spells — see Class System GDD

**Spend:** Reserve mana can be applied to any card cost at any time during DRAFT or PLACEMENT, in any amount up to available reserve. See Rule 4 for spending allocation.

**No cap.** `reserve_mana` has no maximum for any class. Organic board pressure — the need to deploy cards and units — is the intended limiter on reserve accumulation. Miss Nuit's per-round reserve gain is hard-capped at +2 regardless of how many cards the opponent plays in a round (Class System GDD). Reserve can grow across the full game with no ceiling.

**Does not reset.** Reserve is never automatically discarded at round end.

---

**4. Mana spending rules**

When a player plays a card with cost `C`:

1. Server checks `current_mana + reserve_mana >= C`; if false → card play rejected, no resources deducted
2. If the card has no "from reserve" restriction: server applies **auto-split** — draws from `current_mana` first up to `C`, draws remaining from `reserve_mana`. Players cannot override the allocation (simplifies UI; no strategic reason to prefer reserve over current for normal cards)
3. If the card has "costs from reserve" text (e.g., Garde-Temps costs 20 reserve mana): server validates `reserve_mana >= C`; if false → rejected regardless of `current_mana`. Allocation is `reserve_mana -= C`, `current_mana` untouched.

**Example:** Player has 3 current mana, 4 reserve. Plays a 5-cost card. Auto-split: `current_mana -= 3`, `reserve_mana -= 2`. Result: 0 current, 2 reserve.

**Example:** Player has 0 current, 5 reserve. Plays a 3-cost card with no restriction. Auto-split: `current_mana -= 0`, `reserve_mana -= 3`. Result: 0 current, 2 reserve. ← Accepted.

**Placement explicit split exception:** PLACEMENT submissions do not use auto-split. The player chooses an explicit split per staged card through Hand UI, and `C2SSubmitPlacement` sends `current_mana_spend` plus `reserve_mana_spend` for each entry. Economy exposes `validate_explicit_mana_split` and `apply_explicit_mana_split` so Board/Lane can validate and later deduct exactly that split. This exception exists only for placement submit batches; non-placement card plays keep the auto-split rules above.

---

**5. Mana cap**

Default: `mana_cap = GameConfig.mana_cap` (default 10).

**Increase:** When a fake objective is destroyed and the server randomly selects "mana cap +1" (50/50 against "free card pick"):
- The attacking player's `mana_cap += 1` — permanent for this game session
- Takes effect at the **start of the NEXT DRAFT phase** (not retroactive to current round)
- Maximum: `mana_cap = 12` (if both fake objective rewards yield mana cap — each is a 50/50 draw)

---

**6. Gold — income**

**Starting gold:** `gold = 5` granted once before round 1 DRAFT begins.

**Per-round income — applied at the start of each DRAFT phase in this order:**

Step 1 — Calculate interest (snapshot from RESOLUTION end):
```
interest = min(floor(gold_at_end_of_RESOLUTION / 5), GameConfig.interest_max_bonus)
```
The snapshot is taken from gold held **at the end of RESOLUTION** — after all kill/objective rewards have fired, before the new DRAFT begins. The baseline +2 has NOT yet been added when the snapshot is taken; it is added in Step 2.

Step 2 — Apply baseline and interest:
```
gold += GameConfig.gold_baseline_per_round + interest
```

**Interest table:**

| Gold held at RESOLUTION end | Interest bonus |
|---|---|
| 0–4 | +0 |
| 5–9 | +1 |
| 10+ | +2 (maximum) |

**Kill rewards:** `gold += GameConfig.kill_gold_reward` (default: 1) — applied immediately to the killing player during RESOLUTION when a unit's HP reaches 0.

**Objective rewards:** `gold += GameConfig.objective_gold_reward` (default: 3) — applied immediately to the attacking player during RESOLUTION when an objective's HP reaches 0. Note: if an objective reward pushes the killing player past an interest threshold at RESOLUTION end, that higher interest applies at the next DRAFT start.

**No gold cap.** `gold` has no maximum.

**Gold visibility:** Both players can always see each other's current `gold` total throughout all game phases — not only during auctions. This is intentional: gold is the primary strategic information channel for auction reads and bluffing.

**Design note — board pressure as the forcing function:** The interest mechanic rewards gold accumulation, but pure gold hoarding is not a free action. A player who withholds all gold spending must also deploy fewer or weaker units (since unit quality comes from card acquisition), which exposes their objectives to destruction. Board pressure — not an explicit gold-decay rule — is the organic counterweight to passive accumulation. The miser/gambler tension plays out across both currencies simultaneously.

---

**7. Gold — spending**

| Purchase | Cost | Phase |
|---|---|---|
| Common card (shop) | 1g | DRAFT |
| Uncommon card (shop) | 2g | DRAFT |
| Rare card (shop) | 3g | DRAFT |
| Epic card (shop) | 4g | DRAFT |
| Manual shop refresh | 1g (1st), +1g per additional refresh this DRAFT phase | DRAFT (before PLACEMENT begins) |
| Auction winning bid | Bid amount | Auction (during DRAFT phase) |

**Refresh cost escalation:** The first manual shop refresh in a DRAFT phase costs `GameConfig.refresh_base_cost` (default: 1g). Each subsequent refresh costs `refresh_base_cost + min(refresh_count_this_draft, refresh_cap)` — at defaults (base=1, cap=1): 1g first refresh, 2g all subsequent (escalation is capped). The counter resets at the start of each DRAFT phase.

**Spend validation:** Server checks `gold >= cost` before accepting any purchase. If false: rejected, gold not deducted. No partial payment.

**Auction bid validation and gold reservation:** Economy validates each auction bid at placement time via `can_afford_bid(player, amount)` (read-only check: `gold − reserved_gold >= bid_amount`). When a player becomes the current highest bidder, the bid amount is **reserved** via `reserve_gold(player, amount)` and is unavailable for shop purchases. When outbid, the reservation is released via `release_gold_reservation(player)`. `spend_gold(player, bid_amount)` is called only at auction resolution if the player wins. This prevents a player from draining reserved gold through concurrent shop purchases.

**Auction hand-full rule:** A player with 10 cards in hand is blocked from placing auction bids. The server rejects any bid attempt from a player whose `hand_size == 10`. The player must play at least one card during DRAFT to create room before bidding is permitted. See Auction System GDD for enforcement details.

---

### States and Transitions

| State | Description | Economy actions valid |
|---|---|---|
| `Pre-Game` | Before round 1; starting gold granted | Gold: initial 5 granted |
| `Draft` | Mana reset; interest+baseline gold applied | Gold: shop purchases, refresh, auction bids; Mana: card plays |
| `Placement` | Cards selected for this round; no new income | Mana: card plays (if not yet played) |
| `Resolution` | Combat resolves; kill/objective rewards fire; mana discarded at end | Gold: kill/objective rewards applied; Mana: discard at resolution end |

---

### Interactions with Other Systems

| System | Direction | What flows |
|---|---|---|
| **Game Config** | Economy ← Config | `mana_cap`, `starting_gold`, `gold_baseline_per_round`, `interest_max_bonus`, `kill_gold_reward`, `objective_gold_reward` |
| **Round State Machine** | RSM → Economy | Triggers mana reset (DRAFT start), interest+baseline application (DRAFT start), mana discard (RESOLUTION end) |
| **Card Acquisition (Shop)** | Shop → Economy | Calls `spend_gold(player, amount)`; Economy validates |
| **Auction System** | Auction → Economy | Calls `can_afford_bid(player, amount)` per bid placement; `reserve_gold(player, amount)` when bid becomes highest; `release_gold_reservation(player)` when outbid; `spend_gold(player, bid_amount)` on win |
| **Combat Resolution** | Combat → Economy | Calls `apply_gold_award(player, kill_reward)` per kill |
| **Objective System** | Objectives → Economy | Calls `apply_gold_award(player, objective_reward)` on destruction; calls `increment_mana_cap(player)` on fake destroy mana reward |
| **Prism System** | Prism → Economy | Hands player a "+1 reserve" spell card (Lane 2/4 only); playing that card calls `add_reserve(player, 1)` |
| **Class System (Xelor)** | Class → Economy | Xelor spells call `add_reserve(player, n)` and spend via normal Rule 4 validation |
| **HUD / Shop/Auction UI / Board Rendering** | Economy → UI | Broadcasts `current_mana`, `reserve_mana`, `gold`, `mana_cap`, and auction `reserved_gold` projection for display and affordability each round |

## Formulas

All formulas are locked from master GDD §4.1–4.3. They are formally restated here in the canonical format for this GDD. These values are in the entity registry and must not be changed without a master GDD revision.

---

**Formula 1: Mana Ramp**

```
current_mana(R) = min(R, mana_cap)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Round number | `R` | u32 | 1–∞ | Current round (1-indexed) |
| Mana cap | `mana_cap` | u32 | 10–12 | From `GameConfig.mana_cap`; default 10; max 12 |
| Output | `current_mana` | u32 | 1–12 | Mana available at DRAFT start |

**Output Range:** 1 to `mana_cap`. Monotonically non-decreasing until cap is reached.
**Example:** Round 3 → 3. Round 12 → 10. Round 12 with both fake mana rewards → 12.

---

**Formula 2: Interest**

```
interest(g) = min(floor(g / interest_threshold_gold), interest_max_bonus)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Gold at RESOLUTION end | `g` | u32 | 0–∞ | Snapshot before baseline is added |
| Interest max bonus | `interest_max_bonus` | u32 | 2 (default) | From `GameConfig.interest_max_bonus` |
| Output | `interest` | u32 | 0–2 | Gold bonus granted at next DRAFT start |

**Output Range:** 0 to 2.

| `g` range | Interest |
|---|---|
| 0–4 | +0 |
| 5–9 | +1 |
| 10+ | +2 |

**Example:** 8g at RESOLUTION end → `min(floor(8/5), 2) = 1`.

---

**Formula 3: Gold Income at DRAFT Start**

```
gold_new = gold_RESOLUTION_end + interest(gold_RESOLUTION_end) + gold_baseline_per_round
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Gold at RESOLUTION end | `gold_RESOLUTION_end` | u32 | 0–∞ | After all kill/objective rewards fire |
| Interest | `interest(g)` | u32 | 0–2 | Per Formula 2 |
| Baseline | `gold_baseline_per_round` | u32 | 2 | From `GameConfig.gold_baseline_per_round` |
| Output | `gold_new` | u32 | gold_RESOLUTION_end + 2 to gold_RESOLUTION_end + 4 | Gold available at DRAFT start |

**Output Range:** Minimum `gold_RESOLUTION_end + 2`; maximum `gold_RESOLUTION_end + 4`.
**Example:** 8g at RESOLUTION end → 8 + 1 (interest) + 2 (baseline) = **11g** at DRAFT start.
**Note:** `interest_threshold_gold` is a `GameConfig` field (default 5, safe range 5–10). The formula reads this value at runtime — changing it in `game_config.ron` changes where interest brackets fall without a code change.

---

**Formula 4: Mana Cap Maximum**

```
mana_cap_achieved = GameConfig.mana_cap + fake_objective_mana_rewards
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Base mana cap | `GameConfig.mana_cap` | u32 | 10 | Default for all players |
| Mana cap rewards earned | `fake_objective_mana_rewards` | u32 | 0–2 | Count of fake destructions yielding mana reward (50/50 per fake) |
| Output | `mana_cap_achieved` | u32 | 10–12 | Player's mana cap for this session |

**Output Range:** 10 (guaranteed minimum) to 12 (requires both fake destruction rewards to yield mana, each independently 50/50).

## Edge Cases

- **Round 1 current mana is 1:** `min(1, 10) = 1`. Players start round 1 with only 1 mana. Cards costing 2+ cannot be played round 1 unless the player has reserve. Expected behavior — Hearthstone-style mana ramp.

- **Current mana = 0, reserve = 0:** Player cannot play any card. Existing board state continues normally. No compensation or forced action.

- **Gelure played when current_mana = 0:** `reserve_mana += 0`. Legal — the card is consumed from hand but has no mechanical effect. Server does not reject zero-value transfers.

- **Interest when gold = 0 at RESOLUTION end:** `interest = 0`. Player receives only the +2 baseline at next DRAFT.

- **Kill reward crosses an interest threshold:** Player ends RESOLUTION with 9g. A kill fires (+1g) → 10g. Next DRAFT interest = `floor(10/5) = 2`, not 1. Winning rounds has a mild compound gold benefit. Intended.

- **Mana cap increases mid-game:** Takes effect at the start of the **next DRAFT phase**. If the fake objective is destroyed during RESOLUTION, the current round is not extended retroactively.

- **Both fake mana rewards granted (mana_cap = 12):** Legal. Maximum mana cap. If both fakes instead yield free card picks, mana_cap remains at 10 for the game.

- **Auction bid leaves player at 0g:** Accepted. No floor on post-bid gold. Player will receive no interest at next DRAFT, but the bid is valid.

- **Manual shop refresh when gold = 0:** Rejected. `gold >= 1` required. Gold not deducted on rejection.

- **Shop purchase when hand has 10 cards:** Rejected at validation (hand full). Gold not deducted.

- **Auction win when hand has 10 cards:** Gold IS deducted; won card is discarded. Binding commitment — see Auction System GDD.

- **Two units die simultaneously from the same player's kill:** Both kill rewards apply: `gold += 2 × kill_gold_reward`. Applied immediately in RESOLUTION.

- **Self-inflicted objective damage (Sacrier Punition, double-tranchant):** If `attacker_player == defending_player`, the Economy System does NOT call `apply_gold_award`. Loss condition still applies. Per master GDD §5.

- **Mid-RESOLUTION disconnect:** Per RSM Rule 13, if a disconnect aborts RESOLUTION before all sub-steps complete, the interest snapshot does NOT fire for that round. All partial RESOLUTION gold awards already applied before the abort (kills, objectives that resolved) remain. On reconnect (within `disconnect_grace_seconds`), the Economy System re-syncs all currency values to the client. The skipped interest snapshot is not retroactively applied.

- **Reserved gold and shop purchases:** A player's active auction bid reservation reduces available gold for shop validation. Shop spend check: `gold − reserved_gold >= shop_cost`. Example: player has 8g, active bid reservation = 5g → at most 3g available for shop cards. A 4g shop purchase is rejected even though raw `gold = 8`.

- **Zero-cost card (cost = 0):** Auto-split deducts 0 from both pools. Neither `current_mana` nor `reserve_mana` is modified. Always succeeds if the card play is otherwise valid (hand not empty, phase correct).

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **Game Config** | Upstream (hard) | Reads `mana_cap`, `starting_gold`, `gold_baseline_per_round`, `interest_threshold_gold`, `interest_max_bonus`, `kill_gold_reward`, `objective_gold_reward`, `refresh_base_cost` |
| **Round State Machine** | Upstream (coordination) | Triggers: mana reset at DRAFT start, interest+baseline application at DRAFT start, mana discard at RESOLUTION end |
| **Card Acquisition (Shop)** | Downstream (hard) | Calls `spend_gold(player, amount)` for shop purchases; Economy validates |
| **Auction System** | Downstream (hard) | Calls `can_afford_bid(player, amount)`; `reserve_gold(player, amount)`; `release_gold_reservation(player)`; `spend_gold(player, bid_amount)` on win |
| **Shop / Auction UI** | Downstream (read-only) | Consumes `S2CGoldBroadcast { gold, reserved_gold }` to compute `local_free_gold = gold - reserved_gold` for auction display and bid affordability |
| **Combat Resolution** | Downstream (hard) | Calls `apply_gold_award(player, kill_gold_reward)` per kill during RESOLUTION |
| **Objective System** | Downstream (hard) | Calls `apply_gold_award(player, objective_gold_reward)` on objective destruction; calls `increment_mana_cap(player)` on fake destruction mana reward |
| **Prism System** | Downstream (soft) | Lane 2/4 prism grants "+1 reserve" spell card; playing it calls `add_reserve(player, 1)` |
| **Class System** | Downstream (soft) | Xelor spells call `add_reserve(player, n)` and interact with reserve via normal Rule 4 spend validation |
| **HUD / Board Rendering** | Downstream (read-only) | Reads `current_mana`, `reserve_mana`, `gold`, `mana_cap` per player for display |

**Bidirectionality:** Game Config GDD lists Economy System as a downstream consumer ✓. Shop/Auction UI lists Economy System as the source for `S2CGoldBroadcast` / free-gold display ✓. Round State Machine (not yet authored) must list Economy as a dependency when written. All other downstream GDDs must list Economy System when written.

## Tuning Knobs

All Economy System tuning knobs are defined in `GameConfig` (see [game-config.md](game-config.md) Section G for defaults, safe ranges, and impact descriptions). Do not duplicate them here.

| Knob | Default | GameConfig field | What it adjusts |
|---|---|---|---|
| `mana_cap` | 10 | `GameConfig.mana_cap` | Peak mana per round; reached at round 10; max 12 with fake rewards |
| `starting_gold` | 5 | `GameConfig.starting_gold` | Initial draft budget; affects opening card choice range |
| `gold_baseline_per_round` | 2 | `GameConfig.gold_baseline_per_round` | Floor income every round; affects overall economy pace |
| `interest_max_bonus` | 2 | `GameConfig.interest_max_bonus` | Maximum interest per round; cap on hoard incentive |
| `kill_gold_reward` | 1 | `GameConfig.kill_gold_reward` | Gold from aggression; snowball potential of ahead player |
| `objective_gold_reward` | 3 | `GameConfig.objective_gold_reward` | Gold from objective destruction; biggest per-event income source |
| `refresh_base_cost` | 1 | `GameConfig.refresh_base_cost` | Base gold cost of the first manual shop refresh per DRAFT phase; each additional refresh in the same phase costs +1g more |

**`interest_threshold_gold`** (default 5, safe range 5–10): the divisor in the interest formula. Owned by `game-config.md`. Do not set below 5 — at threshold 3 or 4, starting gold (5g) immediately exceeds the maximum-interest bracket, removing the miser/gambler decision pressure.

**Miss Nuit per-round cap:** Miss Nuit's reserve gain is hard-capped at +2 per round (regardless of how many cards the opponent plays). This is not a GameConfig field — it is hardcoded in the Class System GDD.

**GameConfig status:** `refresh_base_cost` added to `game-config.md` 2026-04-29. `reserve_mana_cap` was briefly added then removed by design decision (no total reserve cap; organic pressure is the intended limiter — see OQ2).

## Visual/Audio Requirements

None specific to this system. Economy state is displayed by the HUD — see HUD GDD for visual requirements. Kill/objective gold award events may have SFX (coin sound) owned by the Audio system, not Economy.

## UI Requirements

Economy data (gold, mana, reserve, mana_cap) is read by the HUD for display. Shop/Auction UI also consumes gold plus `reserved_gold` projection to display auction free gold and gate bid affordability. UI requirements for mana bars, gold counter, reserve counter, interest threshold indicators, and auction free-gold panels are owned by the consuming UI GDDs. Economy System only owns the data; it does not own any UI directly.

**📌 UX Flag — Economy System:** This system feeds the HUD. In Phase 4 (Pre-Production), run `/ux-design` for the HUD screen before writing epics. Stories referencing mana/gold display should cite `design/ux/hud.md`, not this GDD.

## Acceptance Criteria

*(Core economy ACs E1–E11 from master GDD §8 apply and are required — see [lanes-and-lies-gdd.md](lanes-and-lies-gdd.md) §8. The ACs below cover behavior specific to this GDD: reserve mechanics, auto-split rules, mana cap, and spend validation edge cases.)*

### Mana Spending (Auto-Split)

| # | Criterion | Type |
|---|---|---|
| EC1 | **GIVEN** `current_mana = 2` and `reserve_mana = 3`, **WHEN** a player plays a 4-cost card (no reserve restriction), **THEN** `current_mana = 0` and `reserve_mana = 1` (auto-split: current drawn first, reserve as overflow). | BLOCKING |
| EC2 | **GIVEN** `current_mana = 0` and `reserve_mana = 5`, **WHEN** a player plays a 3-cost card (no reserve restriction), **THEN** `current_mana = 0` and `reserve_mana = 2`. | BLOCKING |
| EC3 | **GIVEN** `current_mana = 4` and `reserve_mana = 2`, **WHEN** a player plays a 4-cost card (no reserve restriction) where cost equals `current_mana` exactly, **THEN** `current_mana = 0` and `reserve_mana = 2` (reserve untouched). | BLOCKING |
| EC4 | **GIVEN** `current_mana = 1` and `reserve_mana = 1`, **WHEN** a player attempts to play a 3-cost card, **THEN** play is rejected and `current_mana = 1`, `reserve_mana = 1` (neither pool deducted). | BLOCKING |
| EC5 | **GIVEN** a card with "costs from reserve" text costs 4, and `reserve_mana = 3`, `current_mana = 10`, **WHEN** the player attempts to play it, **THEN** play is rejected (`reserve_mana < 4`; `current_mana` does not substitute). | BLOCKING |

### Reserve Persistence

| # | Criterion | Type |
|---|---|---|
| EC6 | **GIVEN** `reserve_mana = 7` at the end of RESOLUTION, **WHEN** the next DRAFT phase begins, **THEN** `reserve_mana = 7` (unchanged) and `current_mana = min(R, mana_cap)` (reset confirmed in same assertion). | BLOCKING |

### Gelure (Xelor Reserve Transfer)

| # | Criterion | Type |
|---|---|---|
| EC7 | **GIVEN** `current_mana = 5`, **WHEN** Gelure is played, **THEN** `current_mana = 0` and `reserve_mana` increases by exactly 5. | BLOCKING |
| EC8 | **GIVEN** `current_mana = 0`, **WHEN** Gelure is played, **THEN** `current_mana = 0` and `reserve_mana` is unchanged (legal no-op, no error). | BLOCKING |

### Mana Cap

| # | Criterion | Type |
|---|---|---|
| EC9 | **GIVEN** `mana_cap = 10` and `increment_mana_cap(player)` is called (testing cap application, not RNG draw), **WHEN** the next DRAFT phase begins, **THEN** `mana_cap = 11` and `current_mana = min(R, 11)`. | BLOCKING |
| EC10 | **GIVEN** `mana_cap = 12` (maximum already reached), **WHEN** `increment_mana_cap(player)` is called again, **THEN** `mana_cap` remains 12. | BLOCKING |

### Gold Awards

| # | Criterion | Type |
|---|---|---|
| EC11 | **GIVEN** `attacker_player == defending_player` (self-inflicted objective damage via Punition or double-tranchant), **WHEN** the objective's HP reaches 0, **THEN** the player's gold balance is unchanged (no objective gold reward). | BLOCKING |

### Gold Economy (Income and Spend)

| # | Criterion | Type |
|---|---|---|
| EC12 | **GIVEN** game start (pre-round-1 initialization), **WHEN** `initialize_player_economy(player)` is called, **THEN** `gold = 5`, `current_mana = 0`, `reserve_mana = 0`. | BLOCKING |
| EC13 | **GIVEN** `gold_at_RESOLUTION_end = 8`, **WHEN** next DRAFT phase begins, **THEN** `interest = 1` (`floor(8/5) = 1`). | BLOCKING |
| EC14 | **GIVEN** `gold_at_RESOLUTION_end = 10`, **WHEN** next DRAFT phase begins, **THEN** `interest = 2` (maximum). | BLOCKING |
| EC15 | **GIVEN** `gold_at_RESOLUTION_end = 8`, **WHEN** next DRAFT phase begins and baseline + interest are applied, **THEN** `gold = 11` (`8 + 1 interest + 2 baseline`). | BLOCKING |
| EC16 | **GIVEN** a player's unit kills an opponent unit during RESOLUTION, **WHEN** `apply_gold_award(player, kill_gold_reward)` fires, **THEN** player `gold` increases by exactly `kill_gold_reward` (default: 1). | BLOCKING |
| EC17 | **GIVEN** a player destroys an opponent objective (`attacker ≠ defender`) during RESOLUTION, **WHEN** `apply_gold_award(player, objective_gold_reward)` fires, **THEN** player `gold` increases by exactly `objective_gold_reward` (default: 3). | BLOCKING |
| EC18 | **GIVEN** `current_mana = 4` at the start of RESOLUTION, **WHEN** RESOLUTION phase ends (mana discard step), **THEN** `current_mana = 0`. | BLOCKING |

### Auction Behavior

| # | Criterion | Type |
|---|---|---|
| EC21 | **GIVEN** player `gold = 3`, **WHEN** player attempts to place an auction bid of `5g`, **THEN** bid is rejected and `gold` is unchanged (`can_afford_bid` returns false). | BLOCKING |
| EC22 | **GIVEN** player `hand_size = 10`, **WHEN** player attempts to place any auction bid, **THEN** bid is rejected (hand full). | BLOCKING |
| EC23 | **GIVEN** player `gold = 8` with an active bid reservation of `5g`, **WHEN** player attempts a `4g` shop purchase, **THEN** purchase is rejected (`gold − reserved = 3 < 4`). | BLOCKING |

### Shop Refresh Cost Escalation

| # | Criterion | Type |
|---|---|---|
| EC24 | **GIVEN** `gold = 5` and no refreshes yet this DRAFT phase, **WHEN** player triggers manual refresh, **THEN** `gold = 4` (1g deducted for first refresh). | BLOCKING |
| EC25 | **GIVEN** `gold = 5` and one refresh already used this DRAFT phase, **WHEN** player triggers a second refresh, **THEN** `gold = 3` (2g deducted). | BLOCKING |
| EC26 | **GIVEN** two refreshes used this DRAFT phase, **WHEN** the NEXT DRAFT phase begins, **THEN** the refresh cost counter resets to base cost (first refresh in new phase costs 1g). | BLOCKING |
| EC27 | **GIVEN** `current_mana = 3`, `reserve_mana = 2`, and card cost `5`, **WHEN** Board/Lane validates placement split `current_mana_spend = 3`, `reserve_mana_spend = 2`, **THEN** validation succeeds. If either spend exceeds its matching pool or the two spends do not sum to card cost, validation rejects and neither pool changes. | BLOCKING |
| EC28 | **GIVEN** an explicit placement split has already validated, **WHEN** Board/Lane applies it at PLACEMENT close, **THEN** `current_mana` and `reserve_mana` are deducted by exactly the submitted amounts; no auto-split recomputation occurs. | BLOCKING |

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ1 | ~~Rarity ceiling for "free card pick" from fake objective destruction.~~ **RESOLVED (2026-04-29):** Free card pick draws from the shared auction pool (same pool as auction draws). Any rarity may be drawn, subject to pool availability (e.g., if the only Legendary copy is already in play, it cannot be picked). The drawn card is removed from the pool and cannot appear at subsequent auctions. No rarity cap. The Objective System GDD owns the draw implementation. | Game Designer | RESOLVED |
| OQ2 | ~~Reserve cap — balance TBD via playtesting.~~ **DESIGN DECISION (2026-04-29): No total reserve cap.** Reserve has no maximum. Organic board pressure (card deployment, objective pressure) is the intended limiter. Miss Nuit per-round gain is capped at +2 (Class System GDD). `reserve_mana_cap` was briefly added and then removed — it made Garde-Temps (cost 20 reserve) permanently unplayable at cap=10. Rely on playtesting to catch snowball if it emerges. | Game Designer | CLOSED |
| OQ3 | Interest as gold-read signal: the +2 cap means 8g and 14g look the same to opponents. Does this weaken "gold as a read"? Evaluate in playtesting; consider visible gold-advantage HUD indicator for large disparities. | Game Designer | After first playtesting session |
| OQ4 | Predictable hoarding windows: fixed auction rounds enable 2-round low-action buildup. Monitor in playtesting — if this conflicts with "no idle spectating," consider exclusive shop incentives to maintain spend pressure pre-auction. | Game Designer | After first playtesting session |
