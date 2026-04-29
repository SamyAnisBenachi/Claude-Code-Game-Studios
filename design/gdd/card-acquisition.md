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
