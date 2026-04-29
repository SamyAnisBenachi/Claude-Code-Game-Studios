# Card Acquisition

> **Status**: In Design
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Deep emergence · Auction as signature

## Overview

Card Acquisition is the server-side system that governs how players obtain cards throughout a game session. It owns three distinct operations: the one-time **Draft Initial** (9 cards displayed at game start; player selects freely within a 5-gold budget), the **Personal Shop** (3 slots refreshed at the start of each DRAFT phase; cards purchased individually with gold), and **Hand State** (the bounded list of up to 10 cards each player holds at any moment). For each shop slot, Card Acquisition performs the 50/50 class/neutral split roll, calls Card Data & Pool's weighted draw functions with seeds from Server-side RNG, enforces within-round deduplication (the same card cannot appear in two slots in the same DRAFT phase), and routes confirmed purchases to `distribute()`. Manual refreshes cost `refresh_base_cost` (default 1g) plus 1g per additional refresh in the same phase, validated by the Economy System. Shop slots are visible but not interactive during DRAFT_AUCTION — purchases and manual refreshes are only accepted during DRAFT_SHOP. Card Acquisition does not own the auction, the free card pick from fake objective destruction, or any visual display of cards; those belong to the Auction System, Objective System, and the Hand UI / Shop/Auction UI GDDs respectively. The shop exists as strategic scaffolding: it is where players invest in an archetype before the auction forces a single high-stakes decision on the cards that matter most.

## Player Fantasy

The shop is not where you win — it is where you **decide**. While the auction tests conviction, the shop builds it. Purchases here are commitments: each Gobball you buy tilts the next slot toward Gobball, and the player can feel the system leaning their way, rewarding the player who chose a side over the player who kept options open.

**The five feelings this system serves:**

1. **"I committed."** — Round 3. Two class cards already in hand. The shop refreshes and a third appears in slot 2 next to a tempting neutral. You buy the third. You're not ahead. You're not even certain the archetype will land. But you've stopped hedging. The weighting will now tilt your way for the rest of the game — and you can feel that tilt begin. The shop becomes a *covenant*: you hold your end, the system holds its.

2. **"The weighting paid off."** — You bought one card from your chosen family two rounds ago. On refresh today, slot 1 shows another from the same family — improbably, precisely what you needed. This is the shop noticing your commitment without announcing that it does. No UI pop. No explicit reward. Just probability in your favor.

3. **"I disguised my plan."** — You spend on a neutral filler in slot 1 while watching your opponent's gold count. They think you're hedging. You know you're saving for an archetype card hidden in a refresh. For those two rounds, the shop is theater as much as economy.

4. **"I walked away."** — The shop offered the strongest neutral of the round. You had the gold. You skipped it and held for the auction. That restraint is a skill expression the shop makes visible; it teaches the player that the auction's gravity is always present even before it arrives.

**What the shop is NOT:** It is not a slot machine (refreshes are deliberate operations, not pulls), not a vending machine (purchases are commitments with permanent downstream effects from the weighting), and not a private moment (the opponent watches gold totals and infers intent — the shop is the opening move of the information war the auction concludes).

## Detailed Design

### Core Rules

**Rule 1 — Hand State**
Each player holds a `hand: Vec<CardId>` capped at 10 cards. The hand is server-authoritative; the client receives hand state updates over the network. Before any purchase is processed, the server checks `hand.len() < 10`. If false: purchase is rejected, gold is unchanged, the slot remains available for re-attempt, and the timer is not affected.

**Rule 2 — Draft Initial (once per game, pre-round-1)**
On RSM entry into `DRAFT_INITIAL`:
1. Card Acquisition calls `draw_initial_draft(class, 9, seed)` — returns 9 distinct card IDs (any rarity, no duplicates).
2. All 9 IDs are added to `displayed_this_draft` and sent via `S2CDraftOffering { card_ids: [CardId; 9] }` (unicast, reliable).
3. Player purchases cards one at a time (click-to-buy). Each purchase validates: `gold >= card_cost` AND `hand.len() < 10`. If both pass: calls `spend_gold(player, cost)` then `distribute(card_id)`; card added to hand. If either fails: rejected with no state change.
4. Manual refresh is **not available** during DRAFT_INITIAL — the 9-card display is fixed for the 45s timer.
5. Unspent gold at timer end carries over to round 1 DRAFT. The 5g is a budget ceiling, not a use-it-or-lose-it pool.

**Rule 3 — Personal Shop Auto-Refresh**
On RSM entry into `DRAFT_SHOP` or `DRAFT_AUCTION` (triggered by `refresh_shop(player)` from RSM Rule 5):
1. Clear `displayed_this_draft` (new DRAFT phase begins clean).
2. For each of 3 slots, run the draw pipeline:
   - **Phase 1:** consume one seed → `gen_range(0..2)` → `SlotType` (Class or Neutral)
   - **Phase 2 (Class):** `draw_class_card(class, next_seed())` → `Option<CardId>`
   - **Phase 2 (Neutral):** `draw_neutral_family(next_seed())` → `Option<FamilyId>`; if `Some` → **Phase 3:** `draw_family_card(family, next_seed())` → `Option<CardId>`
   - **Fallback:** if Phase 2 Class returns `None` (class pool exhausted), retry as Neutral with new seeds
   - **Dedup check:** if candidate is in `displayed_this_draft`, retry up to 20 times; if still unresolved, leave slot empty
3. Add all successful card IDs to `displayed_this_draft`.
4. Send `S2CShopSlots { slots: Vec<Option<CardId>> }` to this player (unicast, reliable).

**Rule 4 — DRAFT_AUCTION: Shop Locked**
During `DRAFT_AUCTION`, the shop slots populated by Rule 3 are visible to the player alongside the auction UI. The auction screen is shared by all players simultaneously and is the primary UI focus. Card Acquisition accepts no `C2SPurchaseCard` or `C2SRefreshShop` messages during this phase — any received are rejected with `ERR_WRONG_PHASE`. No shop state mutations occur until `DRAFT_SHOP` begins.

**Rule 5 — Manual Refresh (DRAFT_SHOP only)**
Available from `DRAFT_SHOP` entry until the phase ends (PLACEMENT begins). Not available in any other phase.
- Cost: `refresh_cost = refresh_base_cost + (refresh_count_this_draft × 1)` gold; `refresh_count_this_draft` starts at 0 each `DRAFT_SHOP` entry.
- Manual refresh does **not** clear `displayed_this_draft` — it extends it. Any card shown this phase (including from auto-refresh) cannot reappear.
- On successful refresh: draw 3 new slots per Rule 3 pipeline (with accumulated dedup), update `displayed_this_draft`, send `S2CShopSlots`.
- Cost validated by Economy System before draw. Rejected refresh (insufficient gold) changes no state.

**Rule 6 — Purchase (DRAFT_SHOP only)**
Purchase is not available during any other phase. Pre-purchase checks (in order):
1. `hand.len() < 10` — hand not full
2. `gold >= card_cost` — gold sufficiency (Economy System validates)
3. `pool.is_available(card_id)` — copy still in pool

If all pass: `spend_gold(player, cost)` → `distribute(card_id)` → card added to hand. Purchased slot is removed from the display. If any check fails: rejected, no state change, slot remains available.

**Rule 7 — External Hand Additions (not Card Acquisition's responsibility)**
Cards added to hand via Prism Lane 3 or the free card pick (fake objective reward) bypass Card Acquisition entirely. The Prism System and Objective System each call `draw_random()` and `distribute()` on Card Data & Pool directly, then write the result to the player's hand. The hand-size limit check for these events is owned by the calling system, not Card Acquisition.

---

### States and Transitions

| State | When | Card Acquisition actions valid |
|---|---|---|
| `INACTIVE` | PLACEMENT / RESOLUTION | None |
| `DRAFT_INITIAL` | RSM is in DRAFT_INITIAL (once per game) | Purchase (click-to-buy, live budget) |
| `SHOP_ACTIVE` | RSM is in DRAFT_SHOP | Purchase, manual refresh |
| `AUCTION_LOCK` | RSM is in DRAFT_AUCTION | None (slots visible, read-only) |

```
INACTIVE → DRAFT_INITIAL     on RSM: DRAFT_INITIAL entry (refresh_shop fires)
DRAFT_INITIAL → INACTIVE     on RSM: DRAFT_INITIAL → PLACEMENT
INACTIVE → AUCTION_LOCK      on RSM: DRAFT_AUCTION entry (refresh_shop fires)
AUCTION_LOCK → SHOP_ACTIVE   on RSM: DRAFT_AUCTION → DRAFT_SHOP (refresh_shop fires again)
INACTIVE → SHOP_ACTIVE       on RSM: DRAFT_SHOP entry (non-auction round, refresh_shop fires)
SHOP_ACTIVE → INACTIVE       on RSM: DRAFT_SHOP → PLACEMENT
```

---

### Interactions with Other Systems

| System | Direction | What flows |
|---|---|---|
| **Round State Machine** | RSM → Card Acquisition | `refresh_shop(player)` fires on DRAFT_INITIAL, DRAFT_AUCTION, and DRAFT_SHOP entry — the only trigger for auto-refresh |
| **Card Data & Pool** | Card Acquisition → Pool | `draw_initial_draft(class, 9, seed)` at DRAFT_INITIAL; `draw_class_card()`, `draw_neutral_family()`, `draw_family_card()` per slot per refresh; `distribute(card_id)` on confirmed purchase |
| **Server-side RNG** | Card Acquisition → RNG | 1 seed per Phase 1 split roll + 1 seed per Phase 2 draw + 1 seed per Phase 3 draw (neutral slots) + 1 seed per fallback retry, per slot per refresh |
| **Economy System** | Card Acquisition → Economy | `spend_gold(player, card_cost)` on purchase; `spend_gold(player, refresh_cost)` on manual refresh |
| **Network Protocol** | Card Acquisition → Network | `S2CDraftOffering` once at DRAFT_INITIAL; `S2CShopSlots` after every auto-refresh and manual refresh (unicast, reliable) |
| **Prism System** | (no interaction) | Lane 3 prism draw is Prism System → Pool directly; Card Acquisition is not in this call chain |
| **Objective System** | (no interaction) | Free card pick draw is Objective System → Pool directly; Card Acquisition is not in this call chain |

## Formulas

### Formula 1: Refresh Cost

```
refresh_cost = refresh_base_cost + min(refresh_count_this_draft, refresh_cap)
```

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Base refresh cost | `refresh_base_cost` | u32 | 1–3 | From `GameConfig.refresh_base_cost`; default 1g; cost of the first refresh in a DRAFT_SHOP phase |
| Refresh cost cap | `refresh_cap` | u32 | 1–5 | From `GameConfig.refresh_cap`; default 1; maximum additional gold above base (caps the escalation) |
| Refreshes done this phase | `refresh_count_this_draft` | u32 | 0–∞ | Count of manual refreshes already completed this DRAFT_SHOP phase; resets to 0 on each DRAFT_SHOP entry |
| Output | `refresh_cost` | u32 | 1–(base+cap) | Gold required for the next manual refresh |

**Output Range:** `refresh_base_cost` (first refresh, count=0) to `refresh_base_cost + refresh_cap` (all subsequent). Default configuration: 1g first, 2g all others.

**Example:** `refresh_base_cost=1`, `refresh_cap=1`, player has refreshed twice (`refresh_count_this_draft=2`). Third refresh: `1 + min(2, 1) = 2g`.

**Interest interaction note:** A player holding exactly 5–9g earns +1 interest next round. Spending 2g on a refresh can drop them below the interest threshold; the effective cost of refreshing at the 5g bracket is **3g effective** (2g paid + 1g lost interest). This is intentional — the miser/gambler tension from the Economy System extends into refresh decisions. It is not a bug if players recognize and weigh this tradeoff.

---

### Formula 2: Dedup Retry Success Probability

```
P_unique = (N - K) / N
```

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Eligible distinct cards | `N` | u32 | 1–pool_size | Distinct card IDs eligible for this slot type with `copies_remaining > 0` |
| Already displayed | `K` | u32 | 0–N | Count of cards in `displayed_this_draft` that are also in the eligible set |
| Output | `P_unique` | float | 0.0–1.0 | Probability a single random draw produces a non-duplicate |

**Output Range:** 0.0 (eligible set entirely exhausted relative to displayed set) to 1.0 (no overlap yet).

**Implementation invariant:** If `K ≥ N`, skip all retries and assign empty slot immediately (short-circuit before the retry loop begins).

**Example:** 12 eligible neutral cards, 2 already displayed this phase: `P_unique = 10/12 ≈ 0.833`. Probability all 20 retries fail: `0.167^20 ≈ negligible`. At the tightest realistic case (N=3, K=2): `P_unique = 0.333`; probability exhausting 20 retries: `0.667^20 ≈ 0.0003`. Empty slot from dedup exhaustion is a statistical curiosity, not a normal gameplay occurrence.

---

### Reference — Shop Slot Weighted Draw

The probability that a specific card or family appears in a shop slot is computed by **Card Data & Pool Formula 2 (Shop Slot Weighted Selection)** in `design/gdd/card-data-pool.md`. Card Acquisition calls `draw_class_card()` / `draw_neutral_family()` / `draw_family_card()` — it does not implement the weighting formula. See card-data-pool.md Formula 2 for variable definitions, the full expression, and worked examples.

## Edge Cases

- **If a purchase is rejected for any reason (hand full, insufficient gold, or pool unavailable):** Gold is unchanged, pool state is unchanged, the slot remains visible and available for re-attempt. The timer is not affected. No partial state is applied under any rejection path.

- **If the player attempts to buy a card during DRAFT_INITIAL when `hand.len() == 10`:** Rejected. Slot stays. (Theoretical — hand starts at 0 and the 5g budget caps purchases well below 10. Enforced regardless for defense-in-depth.)

- **If the player attempts a manual refresh during DRAFT_INITIAL:** Rejected with `ERR_WRONG_PHASE`. The 9-card display is fixed for the timer duration. No gold deducted, `refresh_count_this_draft` unchanged.

- **If the player attempts a manual refresh or purchase during DRAFT_AUCTION:** Rejected with `ERR_WRONG_PHASE`. Shop is read-only in `AUCTION_LOCK` state.

- **If the player attempts a manual refresh with insufficient gold:** Rejected. `refresh_count_this_draft` is NOT incremented — a rejected refresh does not consume a refresh slot or change the cost of the next attempt.

- **If `K ≥ N` before any draw attempt for a slot** (all eligible cards already shown this phase): Short-circuit immediately to empty slot. No retries attempted.

- **If a card's `copies_remaining` reaches 0 between when it was placed in a shop slot and when the player attempts to purchase it** (another system consumed the last copy — e.g., opponent auction draw): Rule 6 check 3 (`pool.is_available(card_id)`) rejects the purchase. The slot remains visible displaying a card the player can no longer buy (a "dead slot"). The server does not automatically refill it. The player can trigger a manual refresh to replace it.

- **If `distribute()` returns `Err(DistributeError::Exhausted)` after all three Rule 6 checks passed** (TOCTOU bug — should not occur in correct implementation): Treat as purchase rejection. Log a server error. Do not apply gold spend. No state change. Gold must never be deducted unless `distribute()` returns `Ok(())`.

- **If fewer than 9 unique eligible cards exist in the pool at DRAFT_INITIAL** (only possible with a stripped test fixture — not in live play with ~298 cards): `draw_initial_draft()` returns however many distinct IDs are available (fewer than 9). The offering displays N < 9 cards with no padding or error. Player's 5g budget still applies to what is shown.

- **If the class pool is fully exhausted** (`draw_class_card` returns `None` every call): Card Acquisition falls back to neutral for all 3 shop slots for the remainder of the game. This is intentional — the player has purchased every available copy of every class card. The shop shows only neutral cards until the session ends.

- **If both class and neutral pools are exhausted:** All 3 shop slots display as empty for that DRAFT_SHOP phase. No error. The player retains cards already in hand.

- **If the DRAFT_INITIAL timer expires with zero purchases:** The player's hand remains empty and their 5g carries over to round 1 DRAFT. The 5g is a budget ceiling, not a use-it-or-lose-it pool.

- **If the Prism System (Lane 3) or Objective System (free card pick) adds a card that brings `hand.len()` to exactly 10** between a player's auto-refresh and their next purchase attempt: Auto-refreshed slots remain displayed. Any purchase attempt fires the `hand.len() < 10` check and is rejected normally. Card Acquisition takes no proactive action on the hand-full event; it only checks at purchase time.

- **If `refresh_count_this_draft` resets across rounds:** `refresh_count_this_draft` resets to 0 on every DRAFT_SHOP entry — including the next round's entry. A player who refreshed 4 times last round pays `refresh_base_cost` again on the first refresh of the following round.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **Card Data & Pool** | Hard upstream | Calls `draw_initial_draft()` at DRAFT_INITIAL; `draw_class_card()`, `draw_neutral_family()`, `draw_family_card()` per slot per refresh; `is_available()` before purchase; `distribute()` on confirmed purchase |
| **Economy System** | Hard upstream | Calls `spend_gold(player, cost)` for purchases and manual refreshes; Economy validates all gold checks before CA proceeds |
| **Server-side RNG** | Hard upstream | Consumes seeds for all draw operations (one per Phase 1 roll, Phase 2 draw, Phase 3 draw, fallback retry per slot); seed table lives in `server-rng.md` |
| **Round State Machine** | Hard upstream (coordination) | RSM fires `refresh_shop(player)` on DRAFT_INITIAL, DRAFT_AUCTION, and DRAFT_SHOP entry; RSM phase transitions determine CA's active state |
| **Game Config** | Hard upstream | Reads `refresh_base_cost`, `refresh_cap` (new — to be added to game-config.md) |
| **Network Protocol** | Downstream (send) | Produces `S2CDraftOffering { card_ids }` once at DRAFT_INITIAL; produces `S2CShopSlots { slots }` after every auto-refresh and manual refresh. Both unicast to the affected player on the reliable channel. Formal schemas to be registered in network-protocol.md. |
| **Hand UI** | Downstream (read-only) | Reads player hand contents for display; receives hand state via network messages |
| **Shop / Auction UI** | Downstream (read-only) | Reads shop slot state from `S2CShopSlots`; renders slots during DRAFT_SHOP, read-only view during DRAFT_AUCTION |
| **Prism System** | No direct interaction | Lane 3 prism draw bypasses CA (Prism → Pool directly); CA only observes the resulting hand state |
| **Objective System** | No direct interaction | Free card pick bypasses CA (Objective → Pool directly); CA only observes the resulting hand state |

**Bidirectionality notes:**
- `game-config.md` must be updated to list Card Acquisition as a downstream consumer of `refresh_base_cost` and `refresh_cap`.
- `network-protocol.md` must formally register `S2CDraftOffering` and `S2CShopSlots` (currently pending, referenced in card-data-pool.md).
- Downstream GDDs (Hand UI, Shop/Auction UI) must list Card Acquisition as upstream when they are authored.

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
