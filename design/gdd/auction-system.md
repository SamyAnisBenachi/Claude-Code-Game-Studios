# Auction System

> **Status**: In Review
> **Author**: User + Agents
> **Last Updated**: 2026-04-29 (post-/design-review revision: 14 blockers resolved)
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
The Auction System receives a `StartAuction(round_number)` Bevy Message from the RSM on DRAFT_AUCTION entry. This is the sole trigger. The Auction System does not evaluate `is_auction_round` — that logic belongs to the RSM. **Guard:** If `StartAuction` arrives in any state other than `IDLE`, it is silently discarded and a server error is logged. This indicates a bug in the RSM.

**Rule 2 — Card selection**
On receiving `StartAuction`:
1. If the shared neutral auction pool has zero copies remaining across all eligible rarities (Rare + neutral Epic + Legendary combined), the Auction System fires `AuctionSettled { winner: None, amount: 0 }` immediately (treated as a no-bid outcome) and returns to IDLE. The RSM transitions to DRAFT_SHOP normally.
2. Otherwise: call `draw_auction_card()` on the Card Data & Pool system. This uses one seed from the Server-side RNG SHOP chain, draws one card, and calls `distribute(card_id)` **immediately at draw time** — unconditionally, regardless of whether the card is ultimately won or not. The card is consumed from the pool at draw; it does not reappear in subsequent auctions.
3. Set `starting_price` from rarity: `{ Rare: 3, Epic: 4, Legendary: 5 }` (gold)
4. Set `timer_remaining_ms = GameConfig.auction_timer_seconds * 1000`
5. Set `current_price = starting_price`, `current_leader = None`

**Neutral Epic note:** Neutral Epic cards do not exist in Krosmaga Extension=1. They must be authored as original designs for this game. Until neutral Epics are designed, the shared auction pool consists of Rare and Legendary neutrals only. See Open Questions.

**Legendary pool stratification:** Legendary cards do not enter the auction draw pool until Round 6 (the second auction). At Round 3 (first auction), only Rare (and Epic, once designed) cards are eligible draws. Rationale: the minimum first bid on a Legendary is 6g, but typical round-3 gold after spending is 5–10g — making a round-3 Legendary effectively aspirational and likely to produce a no-bid outcome. The stratification ensures the first auction is always a meaningful economic decision, not an unreachable card. `draw_auction_card()` must filter by `round_number`: pass `eligible_rarities` based on this rule.

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

`AlreadyLeader` and `HandFull` are both in `network-protocol.md` ✓ (added 2026-04-29).

**Rule 5 — Accepted bid processing**
When all validation passes, in this exact order:
1. `release_gold_reservation(prev_leader)` — releases previous leader's reserved gold. **If `current_leader == None` (first bid of this auction), this is a no-op — do not call the function or pass `None`; skip step 1 entirely.** This must not panic. Implementer must check `current_leader` before calling.
2. `reserve_gold(new_leader, amount)` — reserves new leader's gold
3. Update state: `current_price = amount`, `current_leader = Some(new_leader)`
4. Reset timer: `timer_remaining_ms = min(timer_remaining_ms + 5000, GameConfig.auction_timer_seconds * 1000)`
5. Broadcast `S2CAuctionBidAccepted { bidder: new_leader, amount, new_timer_ms: timer_remaining_ms }`

**Reservation invariant:** At most one player's gold is reserved at any time. Steps 1–2 are atomic. There is never a state where two players simultaneously have an active reservation.

**Rule 6 — Timer tick processing order**
Each server tick, in this exact order:
1. Drain and process all pending `C2SPlaceBid` messages in arrival order
2. Decrement `timer_remaining_ms` by the tick delta using **saturating subtraction** (`timer_remaining_ms = timer_remaining_ms.saturating_sub(tick_delta_ms)`). A plain u32 subtraction will panic in debug builds and wrap to ~4.2 billion in release builds when `tick_delta > remaining`. This is the same pattern used by `objective_damage`. Tick delta is sourced from Bevy `Time::delta().as_millis() as u32`; truncation to integer milliseconds is accepted.
3. If `timer_remaining_ms == 0`: transition to RESOLVING

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

**System ordering requirement:** The reconnect snapshot system must be scheduled **before** the auction tick/resolution system in the Bevy `Update` schedule. This ensures `S2CGameSnapshot` is always enqueued before any same-frame `S2CAuctionSettled`, preventing a race where the client receives a settled state before the auction panel has appeared. The timer in the snapshot will be slightly stale by the RTT (typically 50–250 ms); the reconnecting client accepts this and counts down from the snapshot value immediately on receipt.

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
| `RESOLVING` | Executing winner resolution or no-bid cleanup. Firing `AuctionSettled`. Synchronous — must complete within the same server tick as LIVE_BIDDING exit. | → `IDLE` (after `AuctionSettled` Bevy Message fired). If RESOLVING does not complete within one tick (server bug), the RSM should treat it as an internal error and fire `AbortAuction`. |

---

### Interactions with Other Systems

| System | Direction | Interface |
|---|---|---|
| **Round State Machine** | Bidirectional | RSM → Auction: `StartAuction(round_number)` Bevy Message; `AbortAuction` Bevy Message. Auction → RSM: `AuctionSettled { winner: Option<PlayerId>, amount: u32 }` Bevy Message. `AuctionSettled` is NOT fired on AbortAuction. |
| **Economy System** | Auction → Economy | `can_afford_bid(player, amount)` — read-only bid check; `reserve_gold(player, amount)` — on new leader; `release_gold_reservation(player)` — on outbid or AbortAuction; `spend_reserved_gold(player)` — on win (deducts reserved, zeroes reservation; does not re-validate affordability). |
| **Card Data & Pool** | Auction → Pool | `draw_auction_card()` — draws one neutral Rare/Epic/Legendary using the Server RNG SHOP chain, calls `distribute(card_id)` unconditionally at draw time. Card is permanently consumed from the shared neutral auction pool regardless of whether it is won or not. |
| **Network Protocol / Lightyear** | Auction → Protocol | Broadcasts: `S2CAuctionCard` (before `S2CPhaseChanged`), `S2CAuctionBidAccepted`, `S2CAuctionSettled`. Unicasts: `S2CAuctionBidRejected` (bidder only), `S2CCardAcquired` (winner only). Receives: `C2SPlaceBid`. `S2CGoldBroadcast { player_id, gold, reserved_gold }` is fired by the Economy System on every auction-related gold mutation (`reserve_gold`, `release_gold_reservation`, `spend_reserved_gold`). The `reserved_gold` field is required so opponents can compute free gold (`gold - reserved_gold`) — the figure the Player Fantasy depends on. `S2CGoldBroadcast` now includes `reserved_gold: u32` (updated in network-protocol.md 2026-04-29 re-review). |
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
timer_remaining_ms = min(timer_remaining_ms + auction_timer_reset_seconds × 1000, auction_timer_seconds × 1000)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Timer before reset | `timer_remaining_ms` | u32 | 0–20000 | Milliseconds remaining before the accepted bid |
| Reset increment | `auction_timer_reset_seconds × 1000` | u32 | 3000–10000 ms | Configurable; default 5000 ms |
| Timer cap | `auction_timer_seconds × 1000` | u32 | 10000–30000 ms | Maximum reset target; default 20000 ms |
| Output | `timer_remaining_ms` | u32 | reset_ms–cap_ms | Timer value after the bid is accepted |

**Output Range:** `auction_timer_reset_seconds × 1000` ms (min) to `auction_timer_seconds × 1000` ms (max/cap). Default: 5000 ms min to 20000 ms max.
**Example:** Timer at 3000 ms, bid accepted → `min(3000 + 5000, 20000) = 8000 ms`. Timer at 17000 ms, bid accepted → `min(22000, 20000) = 20000 ms` (capped — net extension only 3000 ms).

**Design note:** The timer can never be extended past its initial value. The cap creates increasing urgency as bidding continues — late bids compress the window rather than restore it. **Dead zone:** any bid placed with more than `(auction_timer_seconds - auction_timer_reset_seconds) × 1000` ms remaining provides less than the full reset extension (at default config: bids placed with > 15 000 ms remaining get < 5 s extension). This is intentional — early bids do not restart the clock.

## Edge Cases

- **If `bidder == current_leader` (self-bid):** Rejected with `BidRejectedReason::AlreadyLeader`. Gold unchanged. Timer unchanged. The current leader cannot push the price themselves — price movements only occur when a new player commits gold.

- **If a bid and the timer-zero event arrive in the same server tick:** The server drains all pending `C2SPlaceBid` messages before evaluating whether `timer_remaining_ms <= 0`. A bid in the same tick as timer-zero is processed first. The client's local timer display is advisory only — `S2CAuctionSettled` is the authoritative terminal signal.

- **If `C2SPlaceBid` arrives after `S2CAuctionSettled` has been dispatched:** Silently discarded. No `S2CAuctionBidRejected` is sent — `S2CAuctionSettled` is the terminal signal. The client should display the bid as "pending" until either `S2CAuctionBidAccepted` or `S2CAuctionSettled` resolves it.

- **If the timer expires with no bids placed:** No gold changes hands. The drawn card was distributed from the shared pool at draw time — it does not return. Pool depletion from a no-bid is permanent. Both players carry their full gold into DRAFT_SHOP, benefiting from higher interest at RESOLUTION end. This is a mutual gold lock, not a neutral outcome.

- **If the shared neutral auction pool is empty on `StartAuction`:** The Auction System fires `AuctionSettled { winner: None, amount: 0 }` immediately without broadcasting `S2CAuctionCard`. The RSM transitions to DRAFT_SHOP. No auction panel is shown to clients. This is a defensive guard — unreachable in a standard game with the default pool.

- **If the winner's `hand_size == 10` at resolution:** Gold is deducted; card discarded. This case is unreachable under correct RSM enforcement (no card acquisition is valid during DRAFT_AUCTION; hand size cannot increase between bid and resolution). Log a server error if reached — it indicates a validation bug.

- **If a player bids with `hand_size == 10`:** Rejected with `BidRejectedReason::HandFull`. The player must play at least one card during DRAFT to create room. **UI requirement:** At DRAFT_AUCTION entry, if the local player's `hand_size == 10`, immediately surface a warning on the auction panel: "Hand full — play a card to bid." The bid button must be visually disabled (not just post-rejection) so the player sees the lockout before spending time deliberating. This is a Pillar 2 constraint — a silently-locked player is idle spectating.

- **If `AbortAuction` arrives during LIVE_BIDDING:** Cancel timer. If `current_leader != None`, call `release_gold_reservation(current_leader)`. Return to IDLE. Do NOT fire `AuctionSettled`. The RSM has already committed to GAME_OVER before sending `AbortAuction`.

- **If `AbortAuction` arrives during SELECTING or RESOLVING:** Same cleanup: release any reservation, cancel any pending work, return to IDLE silently. No `AuctionSettled` fired.

- **If a player reconnects during LIVE_BIDDING:** The reconnecting player receives `S2CGameSnapshot` with `auction_state != None` — containing current card, price, leader, and timer. `S2CAuctionCard` is not re-sent. The player can bid immediately on receipt of the snapshot.

- **If the reconnect snapshot has `auction_state.timer_remaining_ms == 0`:** The auction is still resolving on the server (timer hit zero in the same tick the snapshot was produced, or the client's RTT consumed the remaining time). The client must NOT treat the auction as settled. Immediately enter the "locally expired, awaiting settlement" state: display the auction panel with timer bar frozen at 0% and a "Auction resolving…" label in place of the bid button area. Wait for `S2CAuctionSettled` as the terminal signal. This is the reconnect path into the locally-expired state — the same handling applies whether the player was connected throughout or just reconnected.

- **If the client's local timer reaches zero before `S2CAuctionSettled` arrives ("locally expired, awaiting settlement" state):** This occurs when a same-tick bid on the server extends the timer, but the client has already decremented to zero locally. The client must NOT finalize the auction state locally. While in this state: timer bar freezes at 0%, bid button remains disabled (since the server authority is unknown), the client waits for either `S2CAuctionBidAccepted` (resume — animate bar from 0% to `new_timer_ms / cap` before resuming drain) or `S2CAuctionSettled` (finalize). This is the most tension-laden moment of the signature mechanic; visual behavior must not be undefined.

- **Auction pool card count visibility:** The number of cards remaining in the shared neutral auction pool is **not shown to players**. This is an intentional design choice: hidden pool state preserves the "is the next auction worth saving for?" tension that informs no-bid decisions. Revealing the pool count would remove a layer of strategic uncertainty that gives experienced players an edge. The UI must not display pool remaining count before, during, or after an auction.

- **Interest snapshot invariant:** `reserved_gold == 0` for all players at every RESOLUTION end, guaranteed by RSM phase ordering (DRAFT_AUCTION precedes RESOLUTION; all auction settlement completes before RESOLUTION begins). The interest formula must read `gold`, not `gold - reserved_gold`. A non-zero `reserved_gold` at RESOLUTION end is a server bug.

- **If two bids carry the same `amount` in the same server tick:** Only the first-received is processed as a valid bid (it raises `current_price`). The second arrives with `amount <= current_price` and is rejected with `AmountTooLow`. Arrival order within a tick is determined by message queue position.

- **If the Objective System calls `draw_auction_card()` for a fake-objective free-card-pick:** This occurs during RESOLUTION, which is a mutually exclusive RSM phase with DRAFT_AUCTION. There is no concurrent pool access. Pool state is consistent across phase boundaries.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **Round State Machine** | Upstream (hard) | RSM drives all phase transitions. Sends `StartAuction(round_number)` on DRAFT_AUCTION entry and `AbortAuction` on disconnect. Auction System returns `AuctionSettled` to trigger DRAFT_SHOP. |
| **Economy System** | Upstream (hard) | Auction System calls `can_afford_bid`, `reserve_gold`, `release_gold_reservation`, `spend_reserved_gold`. Economy System validates all gold operations — Auction System does not touch gold directly. |
| **Card Data & Pool** | Upstream (hard) | Auction System calls `draw_auction_card()` to draw from the shared neutral pool (Rare/Epic/Legendary). Pool owns copy counts; Auction System is a consumer only. |
| **Game Config** | Upstream (hard) | Reads `auction_timer_seconds`, `auction_timer_reset_seconds`, `auction_max_duration_seconds`, `auction_floor_rare`, `auction_floor_epic`, `auction_floor_legendary`, `legendary_pool_entry_round`. All fields registered in game-config.md (2026-04-29). |
| **Server-side RNG** | Upstream (indirect) | RNG seed consumed inside `draw_auction_card()` via the SHOP RNG chain. Auction System does not call RNG directly. |
| **Network Protocol / Lightyear** | Downstream (hard) | Auction System produces all auction wire messages. Protocol delivers them via reliable broadcast or unicast. |
| **Shop/Auction UI** | Downstream (hard) | UI renders bid panel, timer, price, and leader from auction state. Must not render until `S2CAuctionCard` has been received. |
| **Objective System** | Shared dependency | Also a consumer of `draw_auction_card()` for the fake-objective free-card-pick (Economy GDD OQ1). Both systems share the same neutral pool; pool depletion from one affects the other. |

**Bidirectionality:** RSM GDD lists Auction System as a downstream dependent ✓. Economy GDD lists Auction System as a downstream caller ✓. Card Data & Pool GDD must be updated to list Auction System as a `draw_auction_card()` consumer and to document the neutral Epic rarity bucket.

## Tuning Knobs

All Auction System knobs live in `GameConfig`. All fields below are registered in `game-config.md`, `game_config.ron`, and the `GameConfig` Rust struct.

| Knob | Default | GameConfig field | Safe Range | Too Low | Too High |
|---|---|---|---|---|---|
| `auction_timer_seconds` | 20s | `GameConfig.auction_timer_seconds` | 10–30s | Insufficient time to process information; bluff decisions become reflex | Auction phase exceeds DRAFT_SHOP in duration; pacing sags |
| `auction_timer_reset_seconds` | 5s | `GameConfig.auction_timer_reset_seconds` | 3–10s | Each bid barely extends the timer; contested auctions end abruptly | Near-full reset on every bid; auctions can run very long on active cards |
| `auction_max_duration_seconds` | 120s | `GameConfig.auction_max_duration_seconds` | 60–300s | May cut off a legitimate bidding war before natural resolution | No practical concern; safety timeout should never fire in normal play |
| `auction_floor_rare` | 3g | `GameConfig.auction_floor_rare` | 2–5g | Below 2g: floor indistinguishable from shop card costs | Excludes players who spent gold heavily this round |
| `auction_floor_epic` | 4g | `GameConfig.auction_floor_epic` | 3–6g | Same as Rare concern | — |
| `auction_floor_legendary` | 5g | `GameConfig.auction_floor_legendary` | 4–8g | Legendary too accessible early; no prestige signal | May gate cashflow-poor players |
| `legendary_pool_entry_round` | 6 | `GameConfig.legendary_pool_entry_round` | 3–9 | Legendaries appear at Round 3 when most players have 5–10g — 6g minimum bid is uncontestable | Legendaries may never appear in shorter games |

**Knob interactions:**
- `auction_timer_seconds` × `auction_timer_reset_seconds` determine maximum theoretical auction duration. The RSM's `auction_max_duration_seconds` safety timeout must always exceed `auction_timer_seconds + (realistic_max_bids × auction_timer_reset_seconds)`.
- All `auction_floor_*` values must remain above 2g to distinguish auction rarities from shop card costs (Common=1g, Uncommon=2g).
- `legendary_pool_entry_round` should not be set below `3` (first auction round), as a round-3 Legendary requires 6g minimum bid and most players will have 5–10g at that point, making the card effectively uncontestable.

**Pool size (claim validation):** The shared neutral auction pool consists of neutral Rare and Legendary cards (neutral Epic cards are not yet designed). The pool size in draws before exhaustion must be specified in `card-data-pool.md` (OQ6). Until that number is documented, the GDD's claim that "pool exhaustion is unreachable in a standard game" cannot be validated. Both the Objective System (fake-objective free-card-pick via `draw_auction_card()`) and the Auction System share this pool — pool depletion rate is joint.

## Visual/Audio Requirements

This system is the game's signature mechanic. Audio and visual urgency are load-bearing — they create the "brink" feeling the Player Fantasy describes.

**Visual requirements:**

| Event | Visual requirement |
|---|---|
| DRAFT_AUCTION entry (`S2CAuctionCard` received) | Auction panel slides in. Card art, name, rarity badge, and starting price displayed. Timer bar appears at full (20s). |
| Bid accepted (`S2CAuctionBidAccepted`) | Price counter animates to new value. Current leader name/avatar updates immediately. Timer bar resets to new value with a flash. |
| Player is current leader | "YOU ARE LEADING" indicator active. Bid button state changes to "Raise bid." |
| Timer countdown | Timer bar drains continuously. Color urgency: green (>10s) → yellow (5–10s) → red (<5s). |
| Timer reset on bid | Timer bar briefly fills back before resuming drain — visible "extension" animation. |
| `S2CAuctionSettled` (winner) | Card animates to winner's hand area. Gold total animates down. Win/loss overlay per player. |
| `S2CAuctionSettled` (no bids) | Card fades out with "NO BIDS — CARD LOST" overlay. Timer collapses. |
| Post-settlement transition | Auction panel slides out. DRAFT_SHOP panel slides in. |

**Audio requirements:**

| Event | Audio requirement |
|---|---|
| DRAFT_AUCTION entry | Ambient urgency tone begins (distinct from DRAFT_SHOP calm) |
| Each accepted bid | Short ascending SFX — escalating pitch series on rapid bids |
| Timer < 5s | Audible countdown tick sound |
| Timer reset on bid | Brief "extension" reverse-tick sound |
| Auction won by self | Victory sting |
| Auction won by opponent | Neutral resolved sting |
| No-bid settlement | Muted "card gone" sound — minor key |

📌 **Asset Spec** — Visual/Audio requirements are defined. After the art bible is approved, run `/asset-spec system:auction-system` to produce per-asset visual descriptions and generation prompts.

## UI Requirements

The auction bid panel is a time-critical interactive UI. All elements must update within one frame of receiving the relevant `S2C` message.

| Element | Spec |
|---|---|
| **Card display** | Card art, name, rarity badge with **both** color (Rare=blue, Epic=purple, Legendary=gold) **and** text label ("RARE" / "EPIC" / "LEGENDARY") on the badge — color alone fails the project accessibility checklist (deuteranopia/tritanopia risk on blue vs. purple). Current price (large, bold). |
| **Timer bar** | Horizontal bar draining continuously. Color urgency (green → yellow → red). Value driven by `timer_remaining_ms` from `S2CAuctionBidAccepted`, not by local timer drift. **Smoothing:** when `S2CAuctionBidAccepted` arrives, the bar chases `new_timer_ms / (auction_timer_seconds × 1000)` using an ease-out curve over ~100 ms — drain and interpolation run **concurrently** (the bar moves toward the new target while also continuing to drain). Do not hard-jump and do not pause drain during interpolation. `cap = auction_timer_seconds × 1000` ms. |
| **Current leader** | Player name and avatar. "You" vs opponent label. "No leader yet" state when `current_leader == None`. Updates each `S2CAuctionBidAccepted`. When the local player is outbid: "YOU ARE LEADING" indicator clears, price counter shows the new amount, and the bid button reverts to "Place bid" — all in the same frame the `S2CAuctionBidAccepted` is processed. |
| **Bid input** | **Preset buttons (primary):** "Min" (= `current_price + 1`), "+3g", "+5g". **Custom** fallback: free-form amount input clamped ≥ `minimum_bid` and ≤ `player.gold - player.reserved_gold`. Confirm button. Preset buttons are the primary interaction for time-critical play; Custom is for unusual amounts. Bid size is the signal. |
| **Available gold** | Shows `gold - reserved_gold` (free gold) — not raw gold — so the player sees exactly what they can commit to a new bid. |
| **"You are leading" indicator** | Visually distinct when the local player is `current_leader`. Bid button reads "Raise bid" (not "Place bid"). |
| **Personal shop** | Shop slots are visible but **NOT interactable** during DRAFT_AUCTION. Visual locked state: desaturated overlay on all shop card slots. Any click or hover attempt on a shop slot displays a tooltip: "Shop available during Draft Shop." Shop panel must not occlude the auction panel. Shop purchases and manual refresh are accepted only during DRAFT_SHOP. This matches RSM Rule 5 and Card Acquisition Rule 4 (`ERR_WRONG_PHASE`). |
| **Hand-full reactive state** | Bid button disabled state is re-evaluated on every incoming `S2C` message that could change `hand_size` — not only at DRAFT_AUCTION entry. If the local player's `hand_size` reaches 10 mid-auction (edge case; acknowledged unreachable under correct RSM enforcement per Rule 7), the bid button must disable immediately and the "Hand full — play a card to bid" warning must re-surface without requiring a panel reload. |

📌 **UX Flag — Auction System:** This system has complex time-critical UI. Run `/ux-design` for the `shop-auction-ui` screen before writing epics. Stories referencing auction UI must cite `design/ux/shop-auction-ui.md`, not this GDD directly.

## Acceptance Criteria

*(Master GDD §8 ACs A1–A10 are required and apply — see [lanes-and-lies-gdd.md](lanes-and-lies-gdd.md) §8. The criteria below cover Auction System-specific behaviors not fully captured by A1–A10.)*

| # | Criterion | Type |
|---|---|---|
| AU1-a | **GIVEN** `StartAuction(R)` is processed by the Auction System, **WHEN** execution completes, **THEN** internal state is `LIVE_BIDDING`, `card_id != None`, and `current_price == starting_price_for_drawn_rarity`. (Proves full initialisation before returning to RSM.) | BLOCKING — Logic/Unit |
| AU1-b | **GIVEN** the RSM enters DRAFT_AUCTION, **WHEN** `S2CPhaseChanged(DRAFT_AUCTION)` is dispatched, **THEN** an `S2CAuctionCard` message was already queued in the same or earlier frame. *BLOCKED pending NP Open Question 3 (Lightyear reliable channel FIFO guarantee across message types). Implement as integration test (`App::new()` with both RSM and Auction System plugins). **Fallback if NP OQ3 cannot resolve before sprint ends:** replace with a code-review assertion — confirm via Bevy system schedule inspection that the RSM system ordering places the `StartAuction` handler before `S2CPhaseChanged` dispatch in the same frame, and document the review in `production/qa/evidence/au1b-ordering-review.md`.* | BLOCKING — Integration (pending NP OQ3) |
| AU2 | **GIVEN** an auction is LIVE_BIDDING and `bidder == current_leader`, **WHEN** `C2SPlaceBid` is received, **THEN** `S2CAuctionBidRejected { reason: AlreadyLeader }` is unicast to the bidder and no auction state changes. | BLOCKING |
| AU3 | **GIVEN** an auction is LIVE_BIDDING and `bidder.hand_size == 10`, **WHEN** `C2SPlaceBid` is received, **THEN** `S2CAuctionBidRejected { reason: HandFull }` is unicast and no state changes. | BLOCKING |
| AU4 | **GIVEN** Player A has `gold = 10, reserved_gold = 5` (current leader) and Player B has `gold = 10, reserved_gold = 0`, **WHEN** Player B's bid of 6g is accepted, **THEN** `Player_A.reserved_gold == 0` AND `Player_B.reserved_gold == 6`. No player has non-zero `reserved_gold` for a bid they are no longer leading. | BLOCKING — Logic/Unit |
| AU5 | **GIVEN** an accepted bid with `timer_remaining_ms = 3000` and `auction_timer_seconds = 20`, `auction_timer_reset_seconds = 5` *(inject these as test constants — do not read from live `GameConfig` to avoid config-drift failures)*, **WHEN** the bid is processed, **THEN** `timer_remaining_ms = min(3000 + 5000, 20000) = 8000`. | BLOCKING |
| AU6 | **GIVEN** an accepted bid with `timer_remaining_ms = 17000` and `auction_timer_seconds = 20`, `auction_timer_reset_seconds = 5` *(same injection requirement — assert the formula, not the literal 20000)*, **WHEN** the bid is processed, **THEN** `timer_remaining_ms = min(17000 + 5000, 20000) = 20000` (capped). | BLOCKING |
| AU7-a | **GIVEN** the timer reaches 0 with a current leader and `leader.hand_size < 10`, **WHEN** resolution fires, **THEN** `spend_reserved_gold(leader)` is called: `leader.gold` is decremented by the bid amount, **`leader.reserved_gold == 0`** (the reservation is zeroed — not just decremented), the card is added to the leader's hand (`hand_size` increases by 1), and `S2CAuctionSettled { winner: Some(leader), amount }` is broadcast. Assert all three output values. | BLOCKING |
| AU7-b | **GIVEN** the timer reaches 0 with a current leader and `leader.hand_size == 10` (injected artificially), **WHEN** resolution fires, **THEN** `spend_reserved_gold(leader)` is still called (gold deducted), the card is NOT added to the hand (`hand_size` remains 10), and `S2CAuctionSettled { winner: Some(leader), amount }` is broadcast. (See master GDD A9. This path is documented as unreachable under correct RSM enforcement — this test confirms the guard works.) | BLOCKING |
| AU8 | **GIVEN** the timer reaches 0 with no bids placed, **WHEN** resolution fires, **THEN** no gold changes for any player and `S2CAuctionSettled { winner: None, amount: 0 }` is broadcast. Additionally, **GIVEN** `draw_auction_card()` was called at SELECTING entry, **WHEN** the auction ends (any outcome), **THEN** the drawn card's `copies_remaining` in the shared neutral pool has been decremented by 1 (`distribute()` was called at draw time). *(Merges former AU12. AU12 retired — merged into AU8 on 2026-04-29.)* **Type note:** The no-gold assertion is Logic/Unit; the `copies_remaining` assertion requires calling into Card Data & Pool system state and is **Integration**. Either split into AU8 (Logic/Unit) and AU8-pool (Integration), or classify AU8 overall as Integration. | BLOCKING — Integration |
| AU9 | **GIVEN** the RSM sends `AbortAuction` with `current_leader == Some(Player_A)` and `Player_A.reserved_gold == 5`, **WHEN** the Auction System processes `AbortAuction`, **THEN** `Player_A.reserved_gold == 0`, Auction System state is `IDLE`, and the `Events<AuctionSettled>` resource contains zero events (verified by reading the `Events` resource after the system runs). | BLOCKING |
| AU10 | **GIVEN** the Auction System is in LIVE_BIDDING with `timer_remaining_ms = T` (any T ≥ 0), **WHEN** `auction_snapshot()` is called, **THEN** the result is `Some(AuctionSnapshot)` where: `card_id` matches the drawn card, `last_accepted_bid == starting_price` if no bids placed (or last accepted amount otherwise), `current_leader == None` if no bids (or last bidder otherwise), and `timer_remaining_ms == T`. Snapshot is `None` only in IDLE. *(The "> 0" assertion is removed — a snapshot with `timer_remaining_ms == 0` is valid during same-tick resolution.)* | BLOCKING |
| AU11 | **GIVEN** Player A has `gold = 5` and `reserved_gold = 5` (active highest bid), **WHEN** Player A attempts a 1g shop purchase, **THEN** the purchase is rejected (`gold - reserved = 0 < 1`). | BLOCKING |
| AU13 | **GIVEN** two `C2SPlaceBid` messages arrive in the same server tick with the same `amount = P`, **WHEN** processed in arrival order, **THEN** only the first is accepted (`current_price` raised to P); the second is rejected with `AmountTooLow` (`P >= P + 1` is false). | BLOCKING |
| AU14 | **GIVEN** any DRAFT_AUCTION phase begins, **WHEN** the Auction System state is inspected at SELECTING entry, **THEN** `reserved_gold == 0` for all players — no stale reservation from prior rounds or prior auctions. | BLOCKING |
| AU15 | **GIVEN** the shared neutral auction pool has `copies_remaining == 0` across all eligible rarities, **WHEN** `StartAuction` is received, **THEN** `AuctionSettled { winner: None, amount: 0 }` Bevy Message fires immediately, Auction System state is `IDLE`, and no `S2CAuctionCard` message is queued. | BLOCKING |
| AU16 | **GIVEN** an auction is LIVE_BIDDING and `bidder.gold - bidder.reserved_gold < bid_amount`, **WHEN** `C2SPlaceBid { amount: bid_amount }` is received, **THEN** `S2CAuctionBidRejected { reason: InsufficientGold }` is unicast to the bidder and no auction state changes. | BLOCKING |
| AU17 | **GIVEN** an auction is LIVE_BIDDING with `current_price = P`, **WHEN** `C2SPlaceBid { amount: P }` is received (at current price, not above it), **THEN** `S2CAuctionBidRejected { reason: AmountTooLow }` is unicast and no state changes. Also test `amount = P - 1`. | BLOCKING |
| AU18 | **GIVEN** the Auction System is in IDLE (after `S2CAuctionSettled` was dispatched this round), **WHEN** a stale `C2SPlaceBid` message is received, **THEN** no `S2CAuctionBidRejected` is queued, no state changes, and no error is logged. *(Silent discard — distinct from `AuctionExpired` which applies during LIVE_BIDDING.)* | BLOCKING |
| AU19 | **GIVEN** the Auction System is in SELECTING or RESOLVING, **WHEN** `AbortAuction` is received, **THEN** Auction System returns to IDLE, `reserved_gold == 0` for all players, and `AuctionSettled` Bevy Message is NOT fired. *(Test each state separately: AU19-a for RESOLVING, AU19-b for SELECTING — a combined test can pass by covering only RESOLVING.)* | BLOCKING |
| AU20 | **GIVEN** Player A is current leader with `reserved_gold = A_amt`, **WHEN** Player B's bid is accepted (outbidding Player A), **THEN** at no point during processing are both `Player_A.reserved_gold > 0` AND `Player_B.reserved_gold > 0` simultaneously. Verify by asserting: (1) `Player_A.reserved_gold == 0` at the point after Step 1 (release) and before Step 2 (reserve), and (2) final state `Player_B.reserved_gold == B_amt`. Tests Rule 5 Step 1 → Step 2 ordering: release-before-reserve invariant. | BLOCKING — Logic/Unit |
| M7-a | **GIVEN** an accepted bid is processed, **WHEN** the Economy System fires `S2CGoldBroadcast` for the bidding player, **THEN** the broadcast payload includes both `gold: u32` and `reserved_gold: u32` for the affected player, so the receiving client can compute `free_gold = gold - reserved_gold`. *`S2CGoldBroadcast` is updated in network-protocol.md to include `reserved_gold` (resolved 2026-04-29 re-review).* **Sprint prerequisite:** this story cannot be implemented until the `network-protocol.md` schema update is confirmed. Track it as a hard dependency before sprint assignment — a programmer who implements the entire auction system and reaches M7-a last will find a late-sprint cross-team blocker. | BLOCKING |
| M7-b | **GIVEN** `S2CGoldBroadcast` is received by the client during DRAFT_AUCTION, **WHEN** the auction panel renders the next frame, **THEN** the opponent's gold total (with reserved amount if applicable) is visible in the auction panel. Evidence: screenshot of auction panel with both players' gold visible. | ADVISORY — UI/Visual |

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ1 | **Neutral Epic card designs.** No neutral Epics exist in Krosmaga Extension=1. Neutral Epics at auction require original card designs. Until they exist, the auction pool is Rare + Legendary only. Epic floor price (4g) reserved for when they are designed. | Game Designer | Before M2 implementation |
| OQ2 | ~~**GameConfig additions.**~~ **RESOLVED 2026-04-29.** `auction_floor_rare`, `auction_floor_epic`, `auction_floor_legendary` are in `game-config.md` and `GameConfig` struct. `legendary_pool_entry_round` also added (design review 2026-04-29). | — | Closed |
| OQ3 | ~~**`BidRejectedReason` enum additions.**~~ **RESOLVED 2026-04-29.** `AlreadyLeader` and `HandFull` are in the `BidRejectedReason` enum in `network-protocol.md`. | — | Closed |
| OQ4 | **`spend_reserved_gold` API name.** The economy-system.md describes the pattern (gold -= reserved; reserved = 0) but does not name the function explicitly. Verify exact API name before implementation — may be `spend_gold(player, reserved_amount)` + `release_gold_reservation`. | Gameplay Programmer | Before Auction System story implementation |
| OQ5 | **2v2/3v3 auction behavior.** Master GDD OQ3: "1 card per auction — consider 1 per N players in larger modes." Current spec assumes 1v1. Multi-player auction dynamics (4+ bidders, card count scaling) unresolved. Hackathon scope: 1v1 only. | Game Designer | Post-hackathon |
| OQ6 | **Card Data & Pool GDD update needed.** `card-data-pool.md` must be updated to: (1) add neutral Epic as a rarity bucket in the shared auction pool, (2) document Auction System as a `draw_auction_card()` consumer, (3) specify the initial copy count for the shared neutral pool, (4) document Legendary pool stratification (not eligible before Round 6). | Game Designer | Before Card Acquisition GDD is authored |
| OQ7 | **Late-game wealth disparity — ⚠️ Pillar Risk: Acknowledged.** At 35g vs 10g, the wealthy player can open at `opponent_gold + 1g` and win without a real contest. This is a known violation of "No idle spectating" for the economically trailing player. **Not blocked:** the fix requires real card-pool + economy data from M2 playtesting. **Decision deadline: before M3 closes.** Monitor gold differential at each auction during M2 playtesting — if gap regularly exceeds 20g before Round 9, escalate to a design sprint between M2 and M3. Candidate mitigations under consideration: (1) trailing-player interest multiplier (e.g., +50% interest rate when gold deficit > 15g); (2) free-bid floor (trailing player's opening bid of each auction costs 0g if deficit > threshold, removing the "can't afford to probe" problem); (3) scaled gold consolation on auction loss (e.g., +1g for losing bidder when the spread exceeds a threshold). | Game Designer | Before M3 close |
| OQ8 | **No-bid frequency risk.** Late-game cards below ~6–7g effective value (considering opportunity cost vs. interest) may rationally receive no bids from both players. Pool burn is permanent. **⚠️ DO NOT default to a "first-bid gold bonus" mitigation.** A gold bonus for the first bid creates a "claim the bounty" incentive that competes with the "signal interest" incentive — once both motivations coexist in the same bid action, bids no longer read as intent signals, which destroys the lie-detector mechanic. Alternative approaches to investigate via M2 playtesting: (1) lower rarity-appropriate floor prices to expand the rational bid range; (2) adjust interest curve to make hoarding less rewarding at low gold totals (increasing the opportunity cost of a no-bid); (3) accept no-bid as a valid strategic outcome and verify it is self-limiting (pool depletion creates scarcity pressure in later auctions). | Game Designer | During M2 playtesting |
| OQ9 | **AuctionExpired bid rejection reachability.** Rule 4 first condition: bid rejected if `phase != DRAFT_AUCTION OR timer_remaining_ms == 0`. Since Rule 6 transitions to RESOLVING the same tick the timer hits zero, it's unclear whether LIVE_BIDDING with `timer == 0` is a reachable state (if transition is atomic). Confirm with the Bevy ECS system schedule: if LIVE_BIDDING→RESOLVING is atomic within a single tick, `AuctionExpired` is unreachable and AC AU17c should document this. If there is a window between timer-zero and RESOLVING, add AU17c covering `AuctionExpired`. | Gameplay Programmer | Before Auction System story implementation |
