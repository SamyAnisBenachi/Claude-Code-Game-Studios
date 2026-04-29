# Auction System

> **Status**: In Design
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Auction as signature · No idle spectating · Deep emergence

## Overview

The Auction System runs one open ascending auction each time the Round State Machine enters DRAFT_AUCTION (every third round, starting from round 3). The system owns three core state values: the **current auction card** (drawn from the neutral card pool via Server-side RNG at auction start), the **current highest bid and current leader**, and the **auction countdown timer**. When a player places a bid, the Economy System validates gold availability and reserves the bid amount; if accepted, the timer resets to `min(remaining + 5 seconds, 20 seconds)` and all players receive the updated price and leader. When the timer reaches zero, the current leader wins — gold is spent and the card is added to their hand (or discarded if hand is full). If no bids are placed before the timer expires, the card is removed from the pool for this game and does not return.

Only neutral cards of Rare rarity or higher appear at auction — never class-specific cards and never Common or Uncommon cards. Starting bid is set by rarity: Rare = 3g, Epic = 4g, Legendary = 5g. Each valid bid must exceed the current price by at least 1 gold. Note: Neutral Epics do not exist in Krosmaga Extension=1 and must be authored as original card designs; until they are, the auction pool consists of Rare and Legendary neutrals only.

The Auction System is the game's signature mechanic. Every accepted bid is simultaneously a price update and an intelligence signal: the price reveals how much the opponent wants the card, and their gold total (always visible to all players) reveals whether they can outbid you. The decision to stay, drop, or bluff-bid one more time — knowing your opponent is running the same calculation — is the core skill moment the game is built around.

## Player Fantasy

The auction is a lie detector with a price tag.

When the card appears and the timer starts, the player is not staring at a card — they are staring at an opponent. Every bid placed is a question lobbed across the table: *do you want this?* Every silence that follows is an answer. The price is a probe. The timer is the pressure that forces honest answers. A player who stays calm and bids last teaches their opponent nothing. A player who bids early reveals exactly where their ceiling is.

The fantasy is **predatory patience** — watching the price climb in real time, reading the opponent's gold total as it sits untouched, timing one more bid at the exact moment it costs the most to answer. The card on the table is almost incidental. What the player is really acquiring is information: how badly the opponent wanted it, what it tells you about their lane plan, whether the 9g they just committed leaves them weak for the next placement round. You can lose the auction and still win the read.

The timer never stops. It only slows. Every accepted bid resets it — not by much, just enough that the clock becomes a conversation between two players who share no words. One bid says *I'm serious*. A counter-bid says *so am I*. A silence for four full seconds says *I've reached my limit* — and you feel it in your chest before the bar finishes draining. When the bar empties and the card is yours, the quiet that follows is not relief. It's confirmation: you read the price of their nerve exactly, and stopped one bid above it.

**Pillar alignment:** This system is the direct embodiment of "Auction as signature" and "No idle spectating." Even a player who has already dropped out of a round's auction is reading the bidding live — watching the opponent's gold change in real time, calibrating their next bluff. The auction is never background noise. It is the game.

## Detailed Design

### Core Rules

**Rule 1 — Auction trigger**
The Auction System receives a `StartAuction(round_number)` Bevy Message from the RSM on DRAFT_AUCTION entry. This is the sole trigger. The Auction System does not evaluate `is_auction_round` — that logic belongs to the RSM.

**Rule 2 — Card selection**
On receiving `StartAuction`:
1. If the shared neutral auction pool has zero copies remaining across all eligible rarities (Rare + neutral Epic + Legendary combined), the Auction System fires `AuctionSettled { winner: None, amount: 0 }` immediately (treated as a no-bid outcome) and returns to IDLE. The RSM transitions to DRAFT_SHOP normally.
2. Otherwise: call `draw_auction_card()` on the Card Data & Pool system. This uses one seed from the Server-side RNG SHOP chain, draws one card, and calls `distribute(card_id)` **immediately at draw time** — unconditionally, regardless of whether the card is ultimately won or not. The card is consumed from the pool at draw; it does not reappear in subsequent auctions.
3. Set `starting_price` from rarity: `{ Rare: 3, Epic: 4, Legendary: 5 }` (gold)
4. Set `timer_remaining_ms = GameConfig.auction_timer_seconds * 1000`
5. Set `current_price = starting_price`, `current_leader = None`

**Neutral Epic note:** Neutral Epic cards do not exist in Krosmaga Extension=1. They must be authored as original designs for this game. Until neutral Epics are designed, the shared auction pool consists of Rare and Legendary neutrals only. See Open Questions.

**Rule 3 — Auction announcement (ordering invariant)**
Before the RSM sends `S2CPhaseChanged(DRAFT_AUCTION)`, the Auction System broadcasts `S2CAuctionCard { card_id, starting_price }` on the reliable channel. This is an invariant — clients must know which card is being auctioned before they enter the DRAFT_AUCTION UI state. The RSM phase entry sequence (F2 Step 4/5) enforces this: `StartAuction` fires and the Auction System handles it fully before `S2CPhaseChanged` is sent.

**Rule 4 — Bid validation**
A `C2SPlaceBid { amount }` is accepted if and only if ALL of the following hold:

| Condition | Rejection reason |
|---|---|
| `phase == DRAFT_AUCTION` AND `timer_remaining_ms > 0` | `AuctionExpired` |
| `amount >= current_price + 1` | `AmountTooLow` |
| `bidder != current_leader` | `AlreadyLeader` |
| `bidder.gold - bidder.reserved_gold >= amount` | `InsufficientGold` |
| `bidder.hand_size < 10` | `HandFull` |

On rejection: unicast `S2CAuctionBidRejected { reason: BidRejectedReason }` to the bidder only. No auction state changes.

**`BidRejectedReason` enum (5 variants):** `InsufficientGold` · `AmountTooLow` · `AuctionExpired` · `AlreadyLeader` · `HandFull`

`AlreadyLeader` and `HandFull` are additions to the enum currently defined in `network-protocol.md`. That file must be updated to match.

**Rule 5 — Accepted bid processing**
When all validation passes, in this exact order:
1. `release_gold_reservation(prev_leader)` — releases previous leader's reserved gold (no-op if `current_leader == None`)
2. `reserve_gold(new_leader, amount)` — reserves new leader's gold
3. Update state: `current_price = amount`, `current_leader = Some(new_leader)`
4. Reset timer: `timer_remaining_ms = min(timer_remaining_ms + 5000, GameConfig.auction_timer_seconds * 1000)`
5. Broadcast `S2CAuctionBidAccepted { bidder: new_leader, amount, new_timer_ms: timer_remaining_ms }`

**Reservation invariant:** At most one player's gold is reserved at any time. Steps 1–2 are atomic. There is never a state where two players simultaneously have an active reservation.

**Rule 6 — Timer tick processing order**
Each server tick, in this exact order:
1. Drain and process all pending `C2SPlaceBid` messages in arrival order
2. Decrement `timer_remaining_ms` by the tick delta
3. If `timer_remaining_ms <= 0`: transition to RESOLVING

A bid arriving in the same server tick as the timer-zero event is processed before resolution fires. The client shows an advisory countdown only; `S2CAuctionSettled` is the terminal signal — clients must not pre-empt it with their local timer display.

**Rule 7 — Resolution (RESOLVING state)**

**Case A — Current leader exists (`current_leader != None`):**
1. Call `spend_reserved_gold(leader)`: `gold -= reserved_gold; reserved_gold = 0`. This is not a `can_afford_bid` re-check — the reservation IS the commitment.
2. If `leader.hand_size < 10`: add `card_id` to leader's hand. Unicast `S2CCardAcquired { card_id, source: CardSource::AuctionWon }` to the winner.
3. If `leader.hand_size == 10`: card discarded; gold already deducted. **This case is unreachable under correct RSM enforcement** (no card acquisition is valid during DRAFT_AUCTION; hand size cannot increase between bid and resolution). Log as a server error if reached.
4. Broadcast `S2CAuctionSettled { winner: Some(leader), amount: current_price }`
5. Fire `AuctionSettled` Bevy Message → RSM transitions to DRAFT_SHOP

**Case B — No bids placed (`current_leader == None`):**
1. No gold changes hands. No reservation to release.
2. The card was already distributed from the pool at draw time (Rule 2). It does not return.
3. Broadcast `S2CAuctionSettled { winner: None, amount: 0 }`
4. Fire `AuctionSettled` Bevy Message → RSM transitions to DRAFT_SHOP

**Rule 8 — AbortAuction (RSM-initiated disconnect cleanup)**
When the RSM sends `AbortAuction` (triggered by player disconnect during DRAFT_AUCTION):
1. Cancel the timer.
2. If `current_leader != None`: call `release_gold_reservation(current_leader)`.
3. Return to IDLE. **Do not fire `AuctionSettled`.**

The RSM has already committed to GAME_OVER before sending `AbortAuction`. No settlement signal is needed or expected.

**Rule 9 — Reconnect support**
The Auction System exposes a read-only `auction_snapshot()` query used by the reconnect handler to populate `S2CGameSnapshot.auction_state`:

```
AuctionSnapshot {
    card_id: CardId,
    last_accepted_bid: u32,       // equals starting_price if no bids placed yet
    current_leader: Option<PlayerId>,
    timer_remaining_ms: u32,
}
```

Returns `None` when in IDLE state. `S2CAuctionCard` is not re-sent on reconnect — the snapshot is the sole source.

**Rule 10 — Timer units**
`GameConfig.auction_timer_seconds` (integer seconds) is converted to `timer_remaining_ms: u32` (milliseconds) on `StartAuction` entry. All internal timer operations and all wire message fields use milliseconds. Timer reset: `min(timer_remaining_ms + 5000, auction_timer_seconds * 1000)`.

---

### States and Transitions

```
IDLE ──► SELECTING ──► LIVE_BIDDING ──► RESOLVING ──► IDLE
              │               │
         (pool empty)    (AbortAuction)
              │               │
         AuctionSettled{None} └──► IDLE (silent; no event)
              │
           ──► IDLE
```

| State | Description | Exit conditions |
|---|---|---|
| `IDLE` | No auction in progress. `auction_snapshot()` returns `None`. | → `SELECTING` on `StartAuction` |
| `SELECTING` | Drawing card from pool; initialising timer and price state. Synchronous — no player input accepted. | → `LIVE_BIDDING` (card drawn successfully); → `IDLE` (pool exhausted — fires `AuctionSettled{None}`, RSM transitions to DRAFT_SHOP) |
| `LIVE_BIDDING` | Timer counting down. Accepting `C2SPlaceBid`. Broadcasting bid results. | → `RESOLVING` (timer ≤ 0 after draining bid queue); → `IDLE` (AbortAuction — silent cleanup, no event fired) |
| `RESOLVING` | Executing winner resolution or no-bid cleanup. Firing `AuctionSettled`. Synchronous. | → `IDLE` (after `AuctionSettled` Bevy Message fired) |

---

### Interactions with Other Systems

| System | Direction | Interface |
|---|---|---|
| **Round State Machine** | Bidirectional | RSM → Auction: `StartAuction(round_number)` Bevy Message; `AbortAuction` Bevy Message. Auction → RSM: `AuctionSettled { winner: Option<PlayerId>, amount: u32 }` Bevy Message. `AuctionSettled` is NOT fired on AbortAuction. |
| **Economy System** | Auction → Economy | `can_afford_bid(player, amount)` — read-only bid check; `reserve_gold(player, amount)` — on new leader; `release_gold_reservation(player)` — on outbid or AbortAuction; `spend_reserved_gold(player)` — on win (deducts reserved, zeroes reservation; does not re-validate affordability). |
| **Card Data & Pool** | Auction → Pool | `draw_auction_card()` — draws one neutral Rare/Epic/Legendary using the Server RNG SHOP chain, calls `distribute(card_id)` unconditionally at draw time. Card is permanently consumed from the shared neutral auction pool regardless of whether it is won or not. |
| **Network Protocol / Lightyear** | Auction → Protocol | Broadcasts: `S2CAuctionCard` (before `S2CPhaseChanged`), `S2CAuctionBidAccepted`, `S2CAuctionSettled`. Unicasts: `S2CAuctionBidRejected` (bidder only), `S2CCardAcquired` (winner only). Receives: `C2SPlaceBid`. |
| **Reconnect handler** | Auction → Handler | Read-only `auction_snapshot()` query populates `S2CGameSnapshot.auction_state`. Returns `None` in IDLE. |
| **Objective System** | Shared dependency | Also a consumer of `draw_auction_card()` for the fake-objective free-card-pick reward (Economy GDD OQ1 resolution). Both the Auction System and Objective System draw from the same shared neutral pool. Objective System draws reduce auction pool availability in subsequent rounds. |

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
