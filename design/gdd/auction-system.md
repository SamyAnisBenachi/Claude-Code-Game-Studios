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

All formulas below are locked from master GDD §4.4. They are formally restated here in the canonical format for this GDD. Values are registered in `design/registry/entities.yaml` as constants and must not be changed without a master GDD revision.

---

**Formula 1: Starting Price**

```
starting_price(rarity) = { Rare: 3, Epic: 4, Legendary: 5 }
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Rarity | `rarity` | enum | Rare \| Epic \| Legendary | Rarity of the card drawn for this auction |
| Output | `starting_price` | u32 | 3–5 | Initial `current_price` and first valid bid threshold |

**Output Range:** 3 to 5 gold.
**Example:** Legendary drawn → `current_price = 5`. First valid bid must be ≥ 6g.

---

**Formula 2: Minimum Bid**

```
minimum_bid = current_price + 1
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Current price | `current_price` | u32 | 3–∞ | Last accepted bid amount (or starting price if no bids yet) |
| Output | `minimum_bid` | u32 | 4–∞ | Minimum amount a `C2SPlaceBid` must carry to pass `AmountTooLow` validation |

**Output Range:** 4 gold (first bid on Rare) to unbounded. In practice limited by players' gold.
**Example:** Current price 7g → minimum bid 8g. A bid of 7g or less is rejected with `AmountTooLow`.

---

**Formula 3: Timer Reset on Accepted Bid**

```
timer_remaining_ms = min(timer_remaining_ms + 5000, auction_timer_seconds × 1000)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Timer before reset | `timer_remaining_ms` | u32 | 0–20000 | Milliseconds remaining before the accepted bid |
| Reset increment | — | u32 | 5000 ms | Fixed: `auction_timer_reset_seconds × 1000` |
| Timer cap | — | u32 | `auction_timer_seconds × 1000` | Maximum reset target; default 20000 ms |
| Output | `timer_remaining_ms` | u32 | 5000–20000 | Timer value after the bid is accepted |

**Output Range:** 5000 ms (min) to 20000 ms (max/cap).
**Example:** Timer at 3000 ms, bid accepted → `min(3000 + 5000, 20000) = 8000 ms`. Timer at 17000 ms, bid accepted → `min(22000, 20000) = 20000 ms` (capped).

**Design note:** The timer can never be extended past its initial value. The cap creates increasing urgency as bidding continues — late bids compress the window rather than restore it.

## Edge Cases

- **If `bidder == current_leader` (self-bid):** Rejected with `BidRejectedReason::AlreadyLeader`. Gold unchanged. Timer unchanged. The current leader cannot push the price themselves — price movements only occur when a new player commits gold.

- **If a bid and the timer-zero event arrive in the same server tick:** The server drains all pending `C2SPlaceBid` messages before evaluating whether `timer_remaining_ms <= 0`. A bid in the same tick as timer-zero is processed first. The client's local timer display is advisory only — `S2CAuctionSettled` is the authoritative terminal signal.

- **If `C2SPlaceBid` arrives after `S2CAuctionSettled` has been dispatched:** Silently discarded. No `S2CAuctionBidRejected` is sent — `S2CAuctionSettled` is the terminal signal. The client should display the bid as "pending" until either `S2CAuctionBidAccepted` or `S2CAuctionSettled` resolves it.

- **If the timer expires with no bids placed:** No gold changes hands. The drawn card was distributed from the shared pool at draw time — it does not return. Pool depletion from a no-bid is permanent. Both players carry their full gold into DRAFT_SHOP, benefiting from higher interest at RESOLUTION end. This is a mutual gold lock, not a neutral outcome.

- **If the shared neutral auction pool is empty on `StartAuction`:** The Auction System fires `AuctionSettled { winner: None, amount: 0 }` immediately without broadcasting `S2CAuctionCard`. The RSM transitions to DRAFT_SHOP. No auction panel is shown to clients. This is a defensive guard — unreachable in a standard game with the default pool.

- **If the winner's `hand_size == 10` at resolution:** Gold is deducted; card discarded. This case is unreachable under correct RSM enforcement (no card acquisition is valid during DRAFT_AUCTION; hand size cannot increase between bid and resolution). Log a server error if reached — it indicates a validation bug.

- **If a player bids with `hand_size == 10`:** Rejected with `BidRejectedReason::HandFull`. The player must play at least one card during DRAFT to create room. This check occurs at bid time — a player with a full hand when the auction begins must take action before participating.

- **If `AbortAuction` arrives during LIVE_BIDDING:** Cancel timer. If `current_leader != None`, call `release_gold_reservation(current_leader)`. Return to IDLE. Do NOT fire `AuctionSettled`. The RSM has already committed to GAME_OVER before sending `AbortAuction`.

- **If `AbortAuction` arrives during SELECTING or RESOLVING:** Same cleanup: release any reservation, cancel any pending work, return to IDLE silently. No `AuctionSettled` fired.

- **If a player reconnects during LIVE_BIDDING:** The reconnecting player receives `S2CGameSnapshot` with `auction_state != None` — containing current card, price, leader, and timer. `S2CAuctionCard` is not re-sent. The player can bid immediately on receipt of the snapshot.

- **Interest snapshot invariant:** `reserved_gold == 0` for all players at every RESOLUTION end, guaranteed by RSM phase ordering (DRAFT_AUCTION precedes RESOLUTION; all auction settlement completes before RESOLUTION begins). The interest formula must read `gold`, not `gold - reserved_gold`. A non-zero `reserved_gold` at RESOLUTION end is a server bug.

- **If two bids carry the same `amount` in the same server tick:** Only the first-received is processed as a valid bid (it raises `current_price`). The second arrives with `amount <= current_price` and is rejected with `AmountTooLow`. Arrival order within a tick is determined by message queue position.

- **If the Objective System calls `draw_auction_card()` for a fake-objective free-card-pick:** This occurs during RESOLUTION, which is a mutually exclusive RSM phase with DRAFT_AUCTION. There is no concurrent pool access. Pool state is consistent across phase boundaries.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **Round State Machine** | Upstream (hard) | RSM drives all phase transitions. Sends `StartAuction(round_number)` on DRAFT_AUCTION entry and `AbortAuction` on disconnect. Auction System returns `AuctionSettled` to trigger DRAFT_SHOP. |
| **Economy System** | Upstream (hard) | Auction System calls `can_afford_bid`, `reserve_gold`, `release_gold_reservation`, `spend_reserved_gold`. Economy System validates all gold operations — Auction System does not touch gold directly. |
| **Card Data & Pool** | Upstream (hard) | Auction System calls `draw_auction_card()` to draw from the shared neutral pool (Rare/Epic/Legendary). Pool owns copy counts; Auction System is a consumer only. |
| **Game Config** | Upstream (hard) | Reads `auction_timer_seconds`, `auction_timer_reset_seconds`. Starting price fields (`auction_floor_rare`, `auction_floor_epic`, `auction_floor_legendary`) must be added to GameConfig. |
| **Server-side RNG** | Upstream (indirect) | RNG seed consumed inside `draw_auction_card()` via the SHOP RNG chain. Auction System does not call RNG directly. |
| **Network Protocol / Lightyear** | Downstream (hard) | Auction System produces all auction wire messages. Protocol delivers them via reliable broadcast or unicast. |
| **Shop/Auction UI** | Downstream (hard) | UI renders bid panel, timer, price, and leader from auction state. Must not render until `S2CAuctionCard` has been received. |
| **Objective System** | Shared dependency | Also a consumer of `draw_auction_card()` for the fake-objective free-card-pick (Economy GDD OQ1). Both systems share the same neutral pool; pool depletion from one affects the other. |

**Bidirectionality:** RSM GDD lists Auction System as a downstream dependent ✓. Economy GDD lists Auction System as a downstream caller ✓. Card Data & Pool GDD must be updated to list Auction System as a `draw_auction_card()` consumer and to document the neutral Epic rarity bucket.

## Tuning Knobs

All Auction System knobs live in `GameConfig`. The three starting-price fields must be added to `game-config.md` and `game_config.ron` — they are not yet registered.

| Knob | Default | GameConfig field | Safe Range | Too Low | Too High |
|---|---|---|---|---|---|
| `auction_timer_seconds` | 20s | `GameConfig.auction_timer_seconds` | 10–30s | Insufficient time to process information; bluff decisions become reflex | Auction phase exceeds DRAFT_SHOP in duration; pacing sags |
| `auction_timer_reset_seconds` | 5s | `GameConfig.auction_timer_reset_seconds` | 3–10s | Each bid barely extends the timer; contested auctions end abruptly | Near-full reset on every bid; auctions can run very long on active cards |
| `auction_floor_rare` | 3g | `GameConfig.auction_floor_rare` *(add)* | 2–5g | Below 2g: floor indistinguishable from shop card costs | Excludes players who spent gold heavily this round |
| `auction_floor_epic` | 4g | `GameConfig.auction_floor_epic` *(add)* | 3–6g | Same as Rare concern | — |
| `auction_floor_legendary` | 5g | `GameConfig.auction_floor_legendary` *(add)* | 4–8g | Legendary too accessible early; no prestige signal | May gate cashflow-poor players |

**Knob interactions:**
- `auction_timer_seconds` × `auction_timer_reset_seconds` determine maximum theoretical auction duration. The RSM's `auction_max_duration_seconds` safety timeout must always exceed `auction_timer_seconds + (realistic_max_bids × auction_timer_reset_seconds)`.
- All `auction_floor_*` values must remain above 2g to distinguish auction rarities from shop card costs (Common=1g, Uncommon=2g).

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

*(Master GDD §8 ACs A1–A10 are required and apply — see [lanes-and-lies-gdd.md](lanes-and-lies-gdd.md) §8. The criteria below cover Auction System-specific behaviors not fully captured by A1–A10.)*

| # | Criterion | Type |
|---|---|---|
| AU1 | **GIVEN** `is_auction_round(R) == true` and the RSM enters DRAFT_AUCTION, **WHEN** `StartAuction(R)` is received, **THEN** `S2CAuctionCard { card_id, starting_price }` is broadcast before `S2CPhaseChanged(DRAFT_AUCTION)` is sent. | BLOCKING |
| AU2 | **GIVEN** an auction is LIVE_BIDDING and `bidder == current_leader`, **WHEN** `C2SPlaceBid` is received, **THEN** `S2CAuctionBidRejected { reason: AlreadyLeader }` is unicast to the bidder and no auction state changes. | BLOCKING |
| AU3 | **GIVEN** an auction is LIVE_BIDDING and `bidder.hand_size == 10`, **WHEN** `C2SPlaceBid` is received, **THEN** `S2CAuctionBidRejected { reason: HandFull }` is unicast and no state changes. | BLOCKING |
| AU4 | **GIVEN** Player A is current leader with 5g reserved and Player B bids 6g (accepted), **WHEN** the bid is processed, **THEN** `release_gold_reservation(Player_A)` fires before `reserve_gold(Player_B, 6)`. At no point are both players' reservations active simultaneously. | BLOCKING |
| AU5 | **GIVEN** an accepted bid with `timer_remaining_ms = 3000` and `auction_timer_seconds = 20`, **WHEN** the bid is processed, **THEN** `timer_remaining_ms = 8000` (`min(3000 + 5000, 20000)`). | BLOCKING |
| AU6 | **GIVEN** an accepted bid with `timer_remaining_ms = 17000` and `auction_timer_seconds = 20`, **WHEN** the bid is processed, **THEN** `timer_remaining_ms = 20000` (capped — not 22000). | BLOCKING |
| AU7 | **GIVEN** the timer reaches 0 with a current leader, **WHEN** resolution fires, **THEN** `spend_reserved_gold(leader)` is called (not `can_afford_bid`), the card is added to the winner's hand (if `hand_size < 10`), and `S2CAuctionSettled { winner: Some(leader), amount }` is broadcast. | BLOCKING |
| AU8 | **GIVEN** the timer reaches 0 with no bids placed, **WHEN** resolution fires, **THEN** no gold changes, `S2CAuctionSettled { winner: None, amount: 0 }` is broadcast, and the card drawn at auction start does not appear in future auctions this game. | BLOCKING |
| AU9 | **GIVEN** the RSM sends `AbortAuction` with an active leader (gold reserved), **WHEN** the Auction System processes it, **THEN** `release_gold_reservation(leader)` fires, the Auction System returns to IDLE, and `AuctionSettled` Bevy Message is NOT fired. | BLOCKING |
| AU10 | **GIVEN** a player reconnects during LIVE_BIDDING, **WHEN** `S2CGameSnapshot` is received, **THEN** `auction_state != None` with: `card_id`, `last_accepted_bid` (= `starting_price` if no bids), `current_leader` (`None` if no bids), and `timer_remaining_ms > 0`. | BLOCKING |
| AU11 | **GIVEN** Player A has `gold = 5` and `reserved_gold = 5` (active highest bid), **WHEN** Player A attempts a 1g shop purchase, **THEN** the purchase is rejected (`gold - reserved = 0 < 1`). | BLOCKING |
| AU12 | **GIVEN** an auction card is drawn at SELECTING entry, **WHEN** the auction ends (any outcome), **THEN** the card's `copies_remaining` in the shared neutral pool is already decremented — it does not appear at subsequent auctions. (`distribute()` was called at draw time.) | BLOCKING |
| AU13 | **GIVEN** two `C2SPlaceBid` messages arrive in the same server tick with the same `amount`, **WHEN** processed in arrival order, **THEN** only the first is accepted; the second is rejected with `AmountTooLow` (first bid already raised `current_price`). | BLOCKING |
| AU14 | **GIVEN** any DRAFT_AUCTION phase begins, **WHEN** the Auction System state is inspected at SELECTING entry, **THEN** `reserved_gold == 0` for all players — no stale reservation from prior rounds or prior auctions. | BLOCKING |

## Open Questions

[To be designed]
