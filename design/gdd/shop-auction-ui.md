# Shop / Auction UI

> **Status**: In Review (post-/design-review revision 2026-04-30: 11 BLOCKING items resolved; re-review pending in fresh session)
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-30
> **Implements Pillar**: No idle spectating · Auction as signature · Simple surface

## Overview

Shop / Auction UI is the client-side presentation system that surfaces every economic decision a player faces each round. It owns three distinct panels corresponding to three RSM phases: the **Draft Offering** (DRAFT_INITIAL — a fixed 9-card grid sent by the server at game start, purchasable within a 5-gold budget over 45 seconds), the **Shop Panel** (DRAFT_SHOP — three refreshable slots drawn from the player's personal pool, purchased with gold, refreshable at escalating cost), and the **Auction Panel** (DRAFT_AUCTION — the game's signature interaction: a live bid timer, current price counter, leader display, and bid input the player uses to contest the neutral card on the block).

The system consumes six server-to-client messages — `S2CDraftOffering`, `S2CShopSlots`, `S2CAuctionCard`, `S2CAuctionBidAccepted`, `S2CAuctionSettled`, `S2CAuctionBidRejected` — and `S2CGoldUpdate` for real-time economy display. All panel state transitions follow RSM phase changes. During DRAFT_AUCTION the Shop Panel remains visible but all purchase and refresh interactions are locked; the Auction Panel is the primary UI focus. During DRAFT_SHOP the Auction Panel is dismissed and the Shop Panel becomes fully interactive. The system produces `C2SPurchaseCard`, `C2SRefreshShop`, and `C2SPlaceBid` messages from player input.

The player fantasy this system serves is economic agency under imperfect information: the player sees their own gold and the opponent's visible free gold, sees the card on the block, and chooses how much conviction to reveal with each bid. The shop builds an archetype in silence; the auction forces a single, legible decision.

## Player Fantasy

The Shop / Auction UI is your dashboard. It surfaces three things at any moment: how much you have, what it would buy, and how long until the moment passes. The panel does not editorialize. It does not tell you which slot to take or when to bid. It shows you the cost. The decision is yours — and the UI's restraint is what makes the decision feel like one.

The opponent's gold is visible the same way the price is visible: as a number on screen. You can read whether they can outbid you. You can read whether the +5g you are about to send pushes them to their ceiling. The inference itself — *what does that price tell me about their plan?* — happens in your head, not in any UI affordance. The panel is honest about the data; it does not editorialize the read.

In DRAFT_INITIAL the panel asks *who are you?* Nine cards, a 5-gold budget, no refresh. You are reading the lineup while your opponent reads theirs, and every card you pick is a card they cannot have. In DRAFT_SHOP it asks *who are you becoming?* Three slots refresh toward your archetype, and you feel the tilt if you have committed: a second Gobball appearing beside the first, probability leaning your way without announcing itself. In DRAFT_AUCTION it asks *how badly?* Five gold left, eight seconds on the timer, the +1g button under your cursor. The auction panel does not help. It just holds the number up and waits.

The other moment this system owns: buying nothing. You close DRAFT_SHOP with your gold intact and cards unspent. The opponent sees the same: your gold total unchanged on their screen. Whether they read that as restraint or as inability is up to them. The UI guarantees the data is shared; the meaning is not.

**Pillar alignment:** "No idle spectating" — every state of this UI offers a live, meaningful signal or a decision. "Auction as signature" — the auction takeover is the panel's highest-stakes configuration; it is designed to be the most consequential 20 seconds of the round. "Simple surface" — at any one moment, the panel asks one question and surfaces only the values needed to answer it (see VA.1 prominence tiers).

## Detailed Design

### Core Rules

**DRAFT_INITIAL Panel**

**Rule 1 — Activation.** The panel activates on receipt of `S2CDraftOffering { card_ids }`. Before this message arrives, the panel is blank. The panel header reads permanently: **"DRAFT OFFERING — ONE TIME ONLY · NO REFRESH · 5g BUDGET"**. On a player's first session, a dismissible callout tooltip appears over the grid: *"These 9 cards are your only offering. No refresh. 5g to spend."* The tooltip disappears on first explicit dismiss and does not reappear. No disabled refresh button exists — the affordance is absent, not locked, to avoid teaching the wrong mental model.

**Rule 2 — Grid layout.** 9 cards in a 3×3 grid. Sort order: rarity descending (Legendary → Epic → Rare → Uncommon → Common), then cost descending within rarity. This resolves Card Acquisition Open Question 5.

**Rule 3 — Accepted input.** The only accepted input is a click on a card slot to purchase it. Client pre-validates before sending: `local_gold >= card_cost` AND `local_hand_size < 10`. If either fails, the click is silently ignored — no server message, no visual error. On valid click: send `C2SPurchaseCard { card_id }`, await `S2CGoldUpdate` + `S2CCardAcquired`.

**Rule 4 — Slot state after purchase.** On confirmed purchase: the slot gains a "BOUGHT" overlay and grays out — it does not disappear. Slots stay in position to preserve spatial memory during the 45-second window. Gold budget counter animates down from `S2CGoldUpdate`.

**Rule 5 — Timer.** Countdown bar initialized from `S2CPhaseChanged { timer_duration_ms }`. Color urgency: yellow below 15s, red below 5s. At zero: panel freezes (all clicks silently ignored). Panel does not auto-dismiss — waits for `S2CPhaseChanged(PLACEMENT)`.

**Rule 6 — Hand full lockout.** When `local_hand_size == 10`: all unowned slots display locked (greyed, non-clickable). Banner: *"Hand full — cannot buy more cards."*

**Rule 7 — Ready signal.** Per RSM Rule 12 + RSM Player Inputs table, `C2SSignalReady { retract: bool }` is a valid DRAFT_INITIAL action: a player who has finished spending may signal ready to exit the phase early (RSM transitions to PLACEMENT when all players signal ready, even before the 45s timer expires). The DRAFT_INITIAL panel exposes a "Ready" button positioned beside the timer bar. On click: send `C2SSignalReady { retract: false }`; button label changes to "Retract Ready"; "Waiting for opponent..." appears beneath. Grid remains fully interactive while ready (player may still purchase additional slots — RSM accepts purchases until the all-ready transition fires). On click of "Retract Ready": send `C2SSignalReady { retract: true }`; button reverts to "Ready"; status text clears. Panel freezes on `S2CPhaseChanged(PLACEMENT)`.

---

**DRAFT_AUCTION Panel**

**Rule 1 — Activation sequence.** Panel activation requires **both** `S2CAuctionCard { card_id, starting_price }` and `S2CPhaseChanged(DRAFT_AUCTION)`. The client buffers whichever message arrives first and activates the panel only when both are present. **No assumption is made about cross-type FIFO ordering on the Lightyear reliable channel** (NP OQ3 unresolved as of 2026-04-30) — Auction System Rule 3 declares an enqueue-order invariant on the server, but the wire protocol does not guarantee receipt order between distinct message types. The buffer is the primary activation path, not an edge-case guard.

Activation steps (when both messages have been received):
- Render card art, name, rarity badge (Rare=blue · Epic=purple · Legendary=gold), starting price as current price.
- Timer bar at 100%, "No leader yet", preset bid buttons enabled per affordability check.
- Start client-side timer countdown.

If only `S2CAuctionCard` has arrived: render card art and price in a "preparing..." state with timer bar greyed; do not start countdown. If only `S2CPhaseChanged(DRAFT_AUCTION)` has arrived: hold the previous panel state (auction not yet visible); do not start countdown.

**Rule 2 — Shop footer during DRAFT_AUCTION.** The shop slots (populated at DRAFT_AUCTION entry by auto-refresh) are visible as a footer strip below the auction panel. The strip is fully locked: slots are read-only (greyed, non-clickable), the refresh button is **hidden** (not just disabled). The footer is intentionally visible so the player can evaluate upcoming shop options while bidding. No `C2SPurchaseCard` or `C2SRefreshShop` is sent.

**Rule 3 — Timer bar.** Drains continuously from `auction_timer_ms` (20s default). Color urgency: green above 10s · yellow 5–10s · red below 5s. Zone color transitions cross-fade over 300ms — no snap. Timer authority is local; server corrections arrive only via `S2CAuctionBidAccepted.new_timer_ms`.

**Rule 4 — Bid input: preset buttons only.** The bid input consists of three preset increment buttons — no free-form text field:
- **+1g** — bids `current_price + 1` (minimum bid)
- **+3g** — bids `current_price + 3`
- **+5g** — bids `current_price + 5`

Each button fires immediately on click (no separate Confirm step). On click: re-validate, send `C2SPlaceBid { amount: current_price + offset }`, then enter the in-flight state:
- The clicked button's label changes from "+Xg" to **"BIDDING..."** in the same Heavy weight at 80% opacity.
- The other two buttons render in standard generic-disabled state (greyed, ~30% opacity).
- All three buttons are non-interactive until `S2CAuctionBidAccepted` or `S2CAuctionBidRejected` arrives.

This in-flight visual is the player's confirmation that the click was received and the bid is in transit (50–250ms typical RTT). On bid resolution, the "BIDDING..." label reverts to its "+Xg" form and per-button affordability re-evaluates per Rule 5.

The `current_price` used for each button is the last value received from `S2CAuctionBidAccepted.amount` (or `starting_price` if no bids yet).

**Rule 4a — Border temperature ramp.** The auction panel's border color escalates with `current_price` to reinforce auction intensity at-a-glance. The mapping is a discrete tier function (see Formula D.6):

| `current_price` range | Border color tier | Hex |
|---|---|---|
| 1–3 g | Ink Blue | `#1A2D5A` |
| 4–6 g | Auction Amber | `#E87C1E` |
| 7–9 g | Deep Amber | `#C2630E` |
| 10+ g | Crimson-Amber | `#9C2000` |

Tier transitions cross-fade over 300ms (same animation policy as timer color zones, Formula D.3). The border ramp is independent of the timer color zone — they may render different colors simultaneously (e.g., 8g auction price with 4s on the timer = Deep Amber border + Crimson-Amber timer bar).

**Rule 5 — Proactive button lockouts.**

| Condition | Affected buttons | Display |
|---|---|---|
| `hand_size == 10` | All three disabled | "Hand full — no bids possible this auction" (cards cannot be played during DRAFT_AUCTION; the lockout is unrecoverable for this auction) |
| `free_gold < current_price + 1` | All three disabled | "Insufficient gold" |
| `free_gold < current_price + offset` (per button) | That button only | Button greyed |
| `player == current_leader` | All three disabled | "YOU ARE LEADING" badge; no raise action available |
| Bid in flight (clicked button awaiting server response) | All three disabled; clicked button shows "BIDDING..." | (see Rule 4) |
| Locally expired, awaiting settlement | All three disabled | (see Rule 8) |

**Bid stride note.** With three preset increments (+1g/+3g/+5g), the player can bid `current_price + {1, 3, 5}` only — never `+2` or `+4` from a given anchor price. Across successive bids the achievable bid amounts cover all integers (since `+1` from an opponent's `+1` reaches every parity), but the *single-action* squeeze-bid at exact opponent ceiling (e.g., bidding +2g to push opponent to 0) is not expressible. This is a deliberate design constraint — see OQ1 closure note for the fantasy tradeoff.

Individual button disable: +1g disabled if `free_gold < current_price + 1`; +3g disabled if `free_gold < current_price + 3`; +5g disabled if `free_gold < current_price + 5`. A player with exactly 2g free gold and `current_price = 0` sees +1g enabled, +3g disabled, +5g disabled.

**Rule 6 — Bid accepted: `S2CAuctionBidAccepted { bidder, amount, new_timer_ms }`.**
Update current price to `amount`. Update leader display. If `bidder == local_player`: activate "YOU ARE LEADING" badge; all three preset buttons disabled (cannot bid against self). If `bidder == opponent`: clear "YOU ARE LEADING" if active; re-enable preset buttons per current affordability (update `current_price = amount` first, then evaluate per-button disable). Timer bar: animate (ease-out, 120–150ms) from current display position to `new_timer_ms / cap_ms`, brief bar flash, then resume drain.

**Rule 7 — Bid rejected: `S2CAuctionBidRejected { reason }`.**
First, revert any "BIDDING..." label on a clicked button back to its "+Xg" form. Then re-enable preset buttons (per affordability). Toast notification per reason:

| Reason | Toast text |
|---|---|
| `InsufficientGold` | "Not enough gold" |
| `AmountTooLow` | "Bid must be at least [minimum_bid]g" |
| `AlreadyLeader` | "You are already leading" |
| `HandFull` | "Hand full — play a card to bid" |
| `AuctionExpired` | "Auction has ended" |

**Rule 8 — Locally expired, awaiting settlement.** Activates when the client's local timer drains to 0 before `S2CAuctionSettled` arrives.
1. Timer bar freezes at 0%.
2. All three preset buttons disabled.
3. After 500ms: subtle pulse animation on the timer bar **and** small "Bid placed — finalizing..." label appears beneath the bar (immediate, not delayed — the player needs feedback that the system is processing, not "broken").
4. After 1500ms: label changes to "Awaiting server..." if still unresolved (longer-than-typical RTT — escalation from "normal processing" to "noticeable lag").
5. Two resolutions: `S2CAuctionBidAccepted` → animate bar to `new_timer_ms / cap`, re-enable preset buttons (per affordability), resume drain, clear pulse + label; `S2CAuctionSettled` → proceed to Rule 9.

**Rule 9 — Settlement: `S2CAuctionSettled { winner, amount }`.**

| Case | Animation | Transition |
|---|---|---|
| `winner == local_player` | Card art animates to hand area; gold counter animates down; "YOU WON" overlay 1.5s | Auction panel slides DOWN · shop footer EXPANDS UPWARD · 350ms |
| `winner == opponent` | "OPPONENT WON" overlay 1.5s | Same transition |
| `winner == None` (no bids) | Card art fades out; "NO BIDS — CARD LOST" overlay 1.0s | Same transition |

DRAFT_SHOP timer starts when the panel expansion animation completes, not during the transition.

---

**DRAFT_SHOP Panel**

**Rule 1 — Activation.** Two paths: (a) **Auction rounds**: `S2CAuctionSettled` triggers the transition; shop slots already populated from DRAFT_AUCTION auto-refresh — no new `S2CShopSlots` arrives. (b) **Non-auction rounds**: `S2CPhaseChanged(DRAFT_SHOP)` and `S2CShopSlots` both received; panel activates when both present (buffer if one arrives first). Timer from `S2CPhaseChanged { timer_duration_ms }` (default 30s).

**Rule 2 — Layout.** Three slots in a horizontal row. Each slot: card art, name, rarity badge, cost. Refresh button and Ready button visible.

**Rule 3 — Refresh button label.** Updates live with current cost:
- Before any manual refresh this phase: **"REFRESH · 1g"**
- After first manual refresh: **"REFRESH · 2g"** (caps at `refresh_base_cost + refresh_cap` = 2g; stays at 2g for all subsequent refreshes this phase)
- Disabled (greyed) when `local_gold < refresh_cost`

**Rule 4 — Purchase flow.** Client pre-validates (`gold >= cost` AND `hand_size < 10`). On valid click: send `C2SPurchaseCard`. On confirmation (`S2CGoldUpdate` + `S2CCardAcquired`): purchased slot fades out. Remaining two slots stay at their positions.

**Rule 5 — Refresh flow.** On valid click: **immediately disable the Refresh button before sending `C2SRefreshShop`** (single click → single send is invariant; double-click cannot result in two requests). All three slots grey out with "Refreshing..." label. On `S2CShopSlots` arrival OR the 5-second timeout (see Edge Cases): new cards animate in (or "Refresh failed" toast appears); refresh button re-enables with updated cost label.

The disable is set in the same frame as the click handler — no intervening render frame can re-enable it. `refresh_count_this_draft` increments only on `S2CShopSlots` receipt (not on send), so a failed/timed-out refresh does not advance the cost counter.

**Rule 6 — Hand full lockout.** When `hand_size == 10`: all slots locked. Banner: *"Hand full — play cards during PLACEMENT to free space."* Refresh button **remains active** (player can read upcoming cards).

**Rule 7 — Ready signal.** "Ready" button available throughout DRAFT_SHOP. On click: sends `C2SSignalReady { retract: false }`; button changes to "Retract Ready"; "Waiting for opponent..." appears. Shop remains fully interactive while ready. Panel freezes on `S2CPhaseChanged(PLACEMENT)`.

---

### States and Transitions

| State | Active when | Interactions accepted |
|---|---|---|
| `INACTIVE` | PLACEMENT / RESOLUTION / GAME_OVER | None |
| `DRAFT_INITIAL` | RSM phase == DRAFT_INITIAL | Purchase click |
| `AUCTION_ACTIVE` | RSM phase == DRAFT_AUCTION | Preset bid buttons +1g/+3g/+5g (if not locked per affordability/state) |
| `SHOP_ACTIVE` | RSM phase == DRAFT_SHOP | Purchase, Refresh, Ready signal |

```
INACTIVE ──► DRAFT_INITIAL     on S2CPhaseChanged(DRAFT_INITIAL) + S2CDraftOffering
DRAFT_INITIAL ──► INACTIVE     on S2CPhaseChanged(PLACEMENT)
INACTIVE ──► AUCTION_ACTIVE    on S2CAuctionCard received (before S2CPhaseChanged(DRAFT_AUCTION))
AUCTION_ACTIVE ──► SHOP_ACTIVE on S2CAuctionSettled + 350ms transition animation
INACTIVE ──► SHOP_ACTIVE       on S2CPhaseChanged(DRAFT_SHOP) + S2CShopSlots (non-auction round)
SHOP_ACTIVE ──► INACTIVE       on S2CPhaseChanged(PLACEMENT)
```

Panel transitions:
- `DRAFT_INITIAL → INACTIVE`: panel fades out on phase change
- `AUCTION_ACTIVE → SHOP_ACTIVE`: auction panel slides down, shop footer expands upward (350ms); DRAFT_SHOP timer starts when expansion completes
- `SHOP_ACTIVE → INACTIVE`: panel fades out on phase change

---

### Interactions with Other Systems

| System | Direction | What flows |
|---|---|---|
| **Card Acquisition** | Upstream → UI | `S2CDraftOffering` (9 cards, DRAFT_INITIAL activation); `S2CShopSlots` (3 slots at each auto-refresh and manual refresh). UI holds no card pool knowledge — slot state is entirely message-driven. |
| **Auction System** | Upstream → UI | `S2CAuctionCard` (auction panel activation); `S2CAuctionBidAccepted` (price, leader, timer updates); `S2CAuctionBidRejected` (inline error + re-enable); `S2CAuctionSettled` (settlement animation + panel transition). UI → Auction: `C2SPlaceBid { amount }`. |
| **Economy System** | Upstream → UI | `S2CGoldUpdate { gold, current_mana, reserve_mana, mana_cap }` (own player — drives gold display, refresh cost disable, and buy lockout. Note: does NOT carry `reserved_gold`); `S2CGoldBroadcast { player_id, gold, reserved_gold }` (both own and opponent — drives `local_free_gold = gold - reserved_gold` for preset button affordability checks and available-gold display; opponent's broadcast drives opponent free-gold display in auction panel). |
| **Round State Machine** | Upstream → UI | `S2CPhaseChanged` (sole authority for panel state activation, timer initialization, and dismissal). UI owns no phase logic. |
| **UI → Network Protocol** | Downstream | `C2SPurchaseCard { card_id }` (DRAFT_INITIAL and DRAFT_SHOP); `C2SRefreshShop {}` (DRAFT_SHOP only); `C2SPlaceBid { amount }` (DRAFT_AUCTION only); `C2SSignalReady { retract: bool }` (DRAFT_INITIAL and DRAFT_SHOP — matches RSM Rule 12 + Player Inputs table). All locks enforced client-side before send. |

## Formulas

### Formula D.1 — `local_free_gold`

The preset bid buttons' affordability check and the auction panel's "available gold" display both read `local_free_gold`, not raw `gold`. A button for offset X is enabled only if `local_free_gold >= current_price + X`.

`local_free_gold = gold - reserved_gold`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Total gold | `gold` | u32 | 0–unbounded | Player's total gold, from `S2CGoldBroadcast.gold` (filtered `player_id == self`) |
| Reserved gold | `reserved_gold` | u32 | 0–`gold` | Amount locked by the player's current auction bid; guaranteed 0 outside DRAFT_AUCTION. From `S2CGoldBroadcast.reserved_gold` |
| Output | `local_free_gold` | u32 | 0–`gold` | Spendable gold; preset button affordability ceiling and "available gold" display |

**Output Range:** 0 to `gold`. `reserved_gold ≤ gold` is a server invariant, so the result is always non-negative. Implementation should use saturating subtraction as a defensive guard against server bugs.

**Example:** `gold = 8`, `reserved_gold = 5` (player is auction leader with a 5g bid). `local_free_gold = 3`. With `current_price = 0`: +1g enabled (3 ≥ 1), +3g enabled (3 ≥ 3), +5g disabled (3 < 5).

**Source note:** `reserved_gold` comes exclusively from `S2CGoldBroadcast`. `S2CGoldUpdate` carries `reserve_mana` (mana economy) — a different field. The opponent's free gold uses the same formula from their `S2CGoldBroadcast` entry.

---

### Formula D.2 — `timer_bar_fill_pct`

Drives the visual fill width of all three phase timer bars.

`fill_pct = clamp(timer_remaining_ms / timer_max_ms, 0.0, 1.0)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Remaining time | `timer_remaining_ms` | f32 | 0–timer_max_ms | Local countdown. Initialized from `S2CPhaseChanged.timer_duration_ms`. During DRAFT_AUCTION, corrected by `S2CAuctionBidAccepted.new_timer_ms` |
| Phase maximum | `timer_max_ms` | f32 | > 0 | Phase constant at phase entry: 45000 (DRAFT_INITIAL) · 30000 (DRAFT_SHOP) · 20000 (DRAFT_AUCTION) |
| Output | `fill_pct` | f32 | 0.0–1.0 | Fraction of the bar to render as filled (1.0 = full, 0.0 = empty) |

**Output Range:** 0.0–1.0. Clamp absorbs timer drift undershots and float rounding at phase boundaries.

**Example:** DRAFT_AUCTION, 6s remaining. `fill_pct = 6000 / 20000 = 0.30`. Bar renders at 30% width.

**Animation note:** On `S2CAuctionBidAccepted`, compute target `fill_pct` from `new_timer_ms / timer_max_ms`. Ease-out tween from current bar position to target over 120–150ms (with brief bar flash), then resume continuous drain. Do not snap.

---

### Formula D.3 — `timer_color_zone` (DRAFT_AUCTION)

Three-zone urgency color model for the auction timer bar.

```
timer_color_zone(timer_remaining_ms) =
  timer_remaining_ms > 10000         →  Green
  5000 < timer_remaining_ms ≤ 10000  →  Yellow
  timer_remaining_ms ≤ 5000          →  Red
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Remaining time | `timer_remaining_ms` | f32 | 0–20000 | Evaluated every frame (same value driving D.2) |
| Output | `timer_color_zone` | enum | Green \| Yellow \| Red | Target color zone; the cross-fade animates to this target |

**Boundaries:** >10001ms → Green. 10000ms → Yellow (first Yellow frame). 5001ms → Yellow. 5000ms → Red (first Red frame).

**Cross-fade rule:** Zone changes trigger a 300ms cross-fade from current rendered color to target. The formula produces a target, not an instantaneous rendered value. Scope: DRAFT_AUCTION only. DRAFT_INITIAL and DRAFT_SHOP use Formula D.5c.

---

### Formula D.4 — `displayed_refresh_cost` (reference)

Owned by `design/gdd/card-acquisition.md`, registered in `design/registry/entities.yaml` as `refresh_cost`. Reproduced here for implementer reference.

`displayed_refresh_cost = refresh_base_cost + min(refresh_count_this_draft, refresh_cap)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Base cost | `refresh_base_cost` | u32 | 1–3 | GameConfig field. Default: 1g |
| Cost cap | `refresh_cap` | u32 | 0–5 | GameConfig field. Default: 1 |
| Refreshes done | `refresh_count_this_draft` | u32 | 0–∞ | Manual refreshes completed since DRAFT_SHOP entry. Incremented on `S2CShopSlots` receipt (not on C2S send). Resets to 0 each DRAFT_SHOP entry |
| Output | `displayed_refresh_cost` | u32 | 1–2 (defaults) | Cost shown in Refresh button label; used for afford-check disable |

**Output Range (defaults):** 1g first refresh, 2g all subsequent. Never exceeds `refresh_base_cost + refresh_cap`.

**Button label:** "REFRESH · [displayed_refresh_cost]g". Disabled when `local_free_gold < displayed_refresh_cost`.

---

### Formula D.5a — Preset button affordability check

*No formula: the check is a direct per-button comparison using `local_free_gold` and `current_price` from Formula D.1 and Formula D.5b. No separate formula is needed — see Rule 5 in Detailed Design.*

Button X is enabled iff `local_free_gold >= current_price + X`, where X ∈ {1, 3, 5}.

---

### Formula D.5b — `minimum_bid` (reference)

Owned by `design/gdd/auction-system.md`, registered in `design/registry/entities.yaml`.

`minimum_bid = current_price + 1`

**UI usage:** `minimum_bid = current_price + 1` defines the effective amount sent by the **+1g button**. The +3g and +5g buttons send `current_price + 3` and `current_price + 5` respectively. `current_price` is updated locally on each `S2CAuctionBidAccepted.amount` receipt. If the server rejects a bid with `AmountTooLow`, the toast shows the actual `minimum_bid`.

---

### Formula D.6 — `auction_border_color_tier` (DRAFT_AUCTION)

Discrete tier function mapping `current_price` to the auction panel's border color. Drives the visual escalation defined in Rule 4a.

```
auction_border_color_tier(current_price) =
  current_price ≤ 3   →  Ink Blue       (#1A2D5A)
  4 ≤ current_price ≤ 6  →  Auction Amber  (#E87C1E)
  7 ≤ current_price ≤ 9  →  Deep Amber     (#C2630E)
  current_price ≥ 10  →  Crimson-Amber  (#9C2000)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Current auction price | `current_price` | u32 | 0–unbounded | Most recent value from `S2CAuctionBidAccepted.amount`, or `starting_price` if no bids placed |
| Output | `auction_border_color_tier` | enum | InkBlue \| AuctionAmber \| DeepAmber \| CrimsonAmber | Target border color tier; the cross-fade animates to this target |

**Boundaries:** `current_price = 0` (panel entry, no bids) → Ink Blue (treated as ≤ 3 case). 3 → Ink Blue. 4 → Auction Amber. 6 → Auction Amber. 7 → Deep Amber. 9 → Deep Amber. 10 → Crimson-Amber.

**Cross-fade rule:** Tier changes trigger a 300ms cross-fade from current rendered border color to target. Independent from Formula D.3 (timer zone color); both may transition simultaneously.

**Scope:** DRAFT_AUCTION only. The Ink Blue starting tier matches the panel background, so the border is visually subtle at low prices and grows louder as the auction escalates — reinforcing "the most consequential 20 seconds of the round."

---

### Formula D.5c — `draft_timer_color_zone` (DRAFT_INITIAL and DRAFT_SHOP)

Two-zone urgency model for non-auction phase timers.

**DRAFT_SHOP (two zones):**
```
draft_timer_color_zone(timer_remaining_ms) =
  timer_remaining_ms > 5000   →  Neutral (default bar color)
  timer_remaining_ms ≤ 5000   →  Red
```

**DRAFT_INITIAL (three zones):**
```
draft_timer_color_zone(timer_remaining_ms) =
  timer_remaining_ms > 15000                →  Neutral
  5000 < timer_remaining_ms ≤ 15000         →  Yellow
  timer_remaining_ms ≤ 5000                 →  Red
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Remaining time | `timer_remaining_ms` | f32 | phase-dependent | Same value driving D.2 |
| Output | `draft_timer_color_zone` | enum | Neutral \| Yellow \| Red | Target zone for bar color; cross-fade 300ms on zone change |

**Scope:** DRAFT_INITIAL uses the three-zone variant (neutral → yellow at 15s → red at 5s). DRAFT_SHOP uses the two-zone variant (neutral → red at 5s). DRAFT_AUCTION is governed by Formula D.3.

## Edge Cases

- **If `S2CAuctionCard` arrives and the local player's `hand_size == 10`:** The auction panel renders normally, but all three preset bid buttons are immediately disabled with the panel-level toast "Hand full — no bids possible this auction." Hand size cannot decrease during DRAFT_AUCTION (no placements or purchases are valid), so the player is locked out for the entire auction. The toast wording deliberately does not suggest "play a card" — that action is unavailable during DRAFT_AUCTION.

- **If `S2CPhaseChanged(DRAFT_AUCTION)` arrives before `S2CAuctionCard` (or vice versa):** This is the expected behavior, not an error. Per Rule 1, the panel buffers whichever message arrives first and activates only when both are present. The client must not crash, must not show a blank panel, and must not assume cross-type FIFO ordering on the Lightyear reliable channel.

- **If `S2CShopSlots` arrives during DRAFT_AUCTION (auto-refresh message):** Buffer it. Do not update the footer strip mid-auction. Apply when the transition animation completes and DRAFT_SHOP becomes active.

- **If `S2CAuctionSettled` arrives with `winner == None`:** Show "NO BIDS — CARD LOST" overlay. Do not animate the card toward either player. Transition to DRAFT_SHOP normally after 1s.

- **If `S2CAuctionSettled` arrives during the "locally expired" state:** Normal expected resolution. Finalize immediately. Cancel the 500ms/1500ms feedback timers.

- **If `S2CAuctionBidAccepted` arrives during the "locally expired" state:** A same-tick server bid extended the timer. Animate bar from 0% to the fill percentage computed from `new_timer_ms`. Re-enable preset buttons per current affordability (update `current_price = amount` first, then evaluate per-button disable). Clear pulse and "Awaiting server..." label. If the local player had a "BIDDING..." in-flight state on a button, revert that button's label to "+Xg" before re-enabling.

- **If `S2CAuctionBidAccepted` and `S2CAuctionSettled` arrive in the same delivery batch (same server tick — last-second bid + simultaneous settlement):** Apply both updates in arrival order before any visual update. Step 1: process `S2CAuctionBidAccepted` (update `current_price`, `current_leader`, timer); Step 2: immediately process `S2CAuctionSettled` (Rule 9 transition). Do not render an intermediate "in-flight resolved" frame between the two — the player should see the settlement animation directly with the final `current_leader` reflected. The "BIDDING..." label, if active, is bypassed and the panel transitions to settlement.

- **If two rapid purchases are sent during DRAFT_INITIAL or DRAFT_SHOP (different slots):** Each fires independently. Do not lock all slots on a single in-flight purchase — only the clicked slot goes pending. The server processes both in message order.

- **If the DRAFT_SHOP refresh confirmation (`S2CShopSlots`) does not arrive within 5 seconds of `C2SRefreshShop`:** Re-enable the refresh button. Display "Refresh failed — try again". Do not advance `refresh_count_this_draft` locally (it was never incremented on send; only incremented on confirmed receipt).

- **If a purchase in-flight confirmation arrives after `S2CPhaseChanged(PLACEMENT)` fires:** Restore the slot to its pre-click state. Gold remains unchanged. The phase-change wins; the purchase was silently rejected by the server.

- **If `S2CPhaseChanged(PLACEMENT)` arrives while the auction panel settlement animation is still playing:** Cancel the animation and transition immediately. The PLACEMENT 10s timer starts on this message — UI animations must never delay phase entry.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **Card Acquisition** | Hard upstream | Provides `S2CDraftOffering { card_ids }` at DRAFT_INITIAL entry (activates 9-card panel); provides `S2CShopSlots { slots }` at each auto-refresh and manual refresh (drives shop slot content). Listed as downstream in card-acquisition.md. |
| **Auction System** | Hard upstream | Provides `S2CAuctionCard` (panel activation), `S2CAuctionBidAccepted` (price/leader/timer updates), `S2CAuctionBidRejected` (inline errors), `S2CAuctionSettled` (settlement animation + transition). UI sends `C2SPlaceBid { amount }` back through Network Protocol. Lists Shop/Auction UI as a downstream dependent. |
| **Economy System** | Hard upstream | Provides `S2CGoldUpdate { gold, current_mana, reserve_mana, mana_cap }` (own player economy) and `S2CGoldBroadcast { player_id, gold, reserved_gold }` (own + opponent — drives `local_free_gold` and opponent free-gold display). All buy lockouts and bid field clamps derive from these messages exclusively. |
| **Round State Machine** | Hard upstream (coordination) | `S2CPhaseChanged` is the sole authority for panel activation, timer initialization, and panel dismissal. UI owns no phase logic. |
| **Network Protocol / Lightyear** | Transport layer | All S2C messages arrive on the reliable Lightyear channel. UI tolerates 50–250ms RTT. UI sends `C2SPurchaseCard`, `C2SRefreshShop`, `C2SPlaceBid`, `C2SSignalReady` through the protocol. |
| **Game Config** | Hard upstream (read-only) | Reads at phase entry: `draft_initial_timer_seconds` (45), `draft_shop_timer_seconds` (30), `auction_timer_seconds` (20). Reads for Formula D.4: `refresh_base_cost` (1), `refresh_cap` (1). Values loaded at session start via asset pipeline; not re-read mid-session. |
| **Hand UI** | Sibling (shared screen) | Independent system sharing screen space during DRAFT phases. No direct calls between them. Coordination is spatial only: the hand must not occlude the auction panel during DRAFT_AUCTION. |
| **Board Rendering** | Sibling (shared screen) | Board Rendering is visible during all phases. Shop/Auction UI panels must coexist with the board without occlusion. Exact layout split is a UI Requirements concern. |

**Bidirectionality notes:**
- card-acquisition.md lists Shop/Auction UI as downstream ✓
- auction-system.md lists Shop/Auction UI as downstream ✓
- economy-system.md should be updated to list Shop/Auction UI as a `S2CGoldBroadcast` consumer
- game-config.md should be updated to list Shop/Auction UI as a downstream consumer of `draft_initial_timer_seconds`, `draft_shop_timer_seconds`, `auction_timer_seconds`, `refresh_base_cost`, `refresh_cap`

## Tuning Knobs

This system owns no numerical knobs of its own — all gameplay values are sourced from upstream systems. Listed here for implementer reference.

**Knobs that affect this system (owned elsewhere):**

| Knob | Default | Owner | Effect on Shop/Auction UI |
|---|---|---|---|
| `draft_initial_timer_seconds` | 45s | game-config.md | Duration of the DRAFT_INITIAL panel. Drives `timer_max_ms` in Formula D.2. Color urgency: yellow <15s, red <5s. |
| `draft_shop_timer_seconds` | 30s | game-config.md | Duration of the DRAFT_SHOP panel. Drives `timer_max_ms` in Formula D.2. Color urgency: red <5s. |
| `auction_timer_seconds` | 20s | game-config.md | Auction panel duration and reset cap. Drives `timer_max_ms` in Formula D.2 and zone boundaries in Formula D.3. |
| `auction_timer_reset_seconds` | 5s | game-config.md | Timer bar extension per accepted bid. Determines magnitude of the ease-out fill animation. |
| `refresh_base_cost` | 1g | game-config.md / economy-system.md | First manual refresh cost in DRAFT_SHOP. Drives the initial Refresh button label. |
| `refresh_cap` | 1 | game-config.md / card-acquisition.md | Maximum escalation above base refresh cost. Determines when the Refresh button label stops increasing (stays at 2g by default). |
| `auction_floor_rare/epic/legendary` | 3/4/5g | auction-system.md / game-config.md | Starting prices displayed on auction panel activation. Passed as `S2CAuctionCard.starting_price` — no UI logic depends on these values directly. |

**UI-specific timing parameters (designer-to-developer guidance, not GameConfig):**

| Parameter | Recommended | Effect |
|---|---|---|
| Auction panel transition duration | 350ms | Down/up slide animation. <200ms = abrupt. >500ms = compresses DRAFT_SHOP timer perceived time. |
| Timer bar ease-out duration | 120–150ms | Bid-accepted bar fill animation. Should resolve within one network RTT. |
| Timer color cross-fade duration | 300ms | Zone transitions (green→yellow, yellow→red). Instant snap reads as a UI glitch. |
| "Locally expired" pulse delay | 500ms | Delay before pulsing indicator on frozen timer bar. |
| "Awaiting server..." label delay | 1500ms | Delay before text label appears beneath bar. |
| Settlement overlay duration | 1500ms (win/loss) / 1000ms (no bids) | Overlay display time before panel transitions. Shorter = abrupt. Longer = delays DRAFT_SHOP phase start. |

## Visual/Audio Requirements

### VA.1 — Panel Visual Style

**DRAFT_AUCTION Panel:** Warm-gold dominant. Panel border driven by Formula D.6 (`auction_border_color_tier`) — see Detailed Design Rule 4a for the full tier mapping. Border thickness: 4–6px. Background: Ink Blue `#1A2D5A` with subtle cel-shade gradient (lighter at top-center, ~15–20% luminance differential). Shape language: angular chips and geometric forms per art bible shape grammar.

**Visual prominence tiers (DRAFT_AUCTION).** The panel is laid out so that exactly one decision is visually anchored at any moment. Three tiers, in descending prominence:

| Tier | Elements | Treatment |
|---|---|---|
| PRIMARY (decision surface) | Three preset bid buttons (+1g/+3g/+5g); current price numeral; opponent free-gold numeral; player free-gold numeral | 100% opacity; Heavy weight; largest type/touch targets. The cluster the player's eye lands on. |
| SECONDARY (context surface) | Card art (240×360); rarity badge; auction timer bar; "YOU ARE LEADING" badge when active | 100% opacity; subordinate scale (card art is large but visually centered, not flashing/animated unless settlement). |
| TERTIARY (peripheral context) | Shop footer strip (3 cards); footer slot art | 30% opacity (revised from 60%); 80% scale relative to baseline shop slots; no hover affordance. Reads as "available later," not "actionable now." |

Settlement overlays and red-zone timer pulse temporarily promote their target element (timer bar, card art) above the PRIMARY tier — those are momentary, not steady-state.

The "Simple surface — one question at a time" pillar is satisfied by the prominence hierarchy: the player's eye is anchored to PRIMARY for the bid decision; SECONDARY supplies the context for that decision; TERTIARY is informational only.

**DRAFT_SHOP Panel:** Warm intimate — cooler than the auction, warmer than the board. Parchment-warm border (`#D4AF72`, 2–3px, no escalation ramp). Background: dark parchment-brown tint (`#1E1610`). Slot wells have a soft warm highlight strip at the top edge (baked art, not dynamic light).

**DRAFT_INITIAL Panel:** Bright, even, "Possibility" register. Minimal frame (2px Ivory outline per slot). Near-transparent overlay — board dimmed to 40% opacity behind the 3×3 grid. No competing panel chrome.

**Shop footer strip during DRAFT_AUCTION:** Same Ink Blue as auction panel at 60% brightness; slot cards at 30% opacity (TERTIARY tier per the prominence table); 80% scale relative to baseline shop slot dimensions; no gold trim; no hover affordance. Reads as peripheral inventory context, not actionable surface. The reduced opacity is deliberate — the auction is the decision; the footer is preparation for the decision after.

---

### VA.2 — Card Display Sizes

| Context | Tier | Size | Opacity |
|---|---|---|---|
| DRAFT_AUCTION featured card | Card Zoom | 240 × 360 px | 100% — brightest element on screen |
| DRAFT_SHOP slots (interactive) | Card Display | 120 × 180 px; hover: 160 × 240 px, 8px lift, gold outline pulse | 100% |
| DRAFT_INITIAL grid | Card Display | 120 × 180 px; hover: 160 × 240 px (same hover behavior) | 100%; purchased = 40% behind "BOUGHT" overlay |
| DRAFT_AUCTION footer strip | Card Display | 120 × 180 px | 60% |

**Rarity badge gems (art bible Section 7.5):**
- Rare: circular gem, Ink Blue outer / lighter blue center gradient
- Epic: diamond gem, `#6B35A0` outer / `#A060D8` center gradient
- Legendary: star/octagonal gem, Arcane Gold outer / Prism White highlight center

Badge dimensions: 28–36px at Card Zoom; 20–24px at Card Display. Always-on rarity text label (10px rendered minimum) beneath each badge (accessibility — see VA.7).

**"BOUGHT" overlay (DRAFT_INITIAL):** "BOUGHT" in Heavy weight, Ivory, centered over art. Card art dims to 40% behind overlay. Slot stays in position; no reflow.

---

### VA.3 — Timer Bar

Horizontal pill, full panel width, 12px tall (1080p ref), 6px rounded ends, 1px Void outline. Solid fill with 2px Ivory cel-shade highlight at top edge (40% opacity). No gradient along bar length.

| Phase | Neutral | Yellow | Red |
|---|---|---|---|
| DRAFT_INITIAL | Arcane Gold `#F5C842` | Auction Amber `#E87C1E` | Crimson-Amber `#9C2000` |
| DRAFT_SHOP | Arcane Gold `#F5C842` | — (two-zone only) | Crimson-Amber `#9C2000` |
| DRAFT_AUCTION | Timer Green `#3AB86A` | Auction Amber `#E87C1E` | Crimson-Amber `#9C2000` |

*Note: DRAFT_AUCTION uses green — a documented exception to the gold color vocabulary. Green unambiguously reads as "time available" in this isolated context.*

**Zone transitions:** 300ms cross-fade (never snap).

**Red zone urgency (<5s, all phases):**
1. Crimson-Amber fill + 4px diffuse outer glow at 60% opacity
2. Fill opacity pulses 85–100% at 2Hz
3. Bar scales 110% height → 100% per second (one vertical swell beat per second)

**DRAFT_AUCTION bid-accepted reset:** Ease-out tween from current fill to new position (120–150ms) + 60ms Prism White `#EEF4FF` flash at 80% opacity, then resume drain.

---

### VA.4 — Gold Display

**HUD chips (always visible):** Angular chip (Ink Blue bg), gold coin icon (24px, flat outlined), Heavy numeral in Arcane Gold with Void shadow. Player and opponent chips at equal visual weight — opponent's gold is tactical information during auctions.

**Auction panel gold display:** "YOUR GOLD / OPP GOLD" labels (Regular 1× Ivory) above Heavy 2.5× numerals for both player `local_free_gold` and opponent free gold. Equal weight, side by side — the most prominent economic display on the panel.

**Gold change animation (any `S2CGoldUpdate`/`S2CGoldBroadcast`):**
- Numeral ticks old→new over 200ms (single jump if delta >3g)
- Coin icon warm-gold bloom: 8px diffuse, 60% peak opacity, 300ms (80ms rise / 220ms decay)
- On decrease: numeral briefly shifts to Auction Amber `#E87C1E` during the tick, then returns to Arcane Gold

---

### VA.5 — Status and Settlement Overlays

**"YOU ARE LEADING" badge (in-panel, not full-screen):** Angular chip, Arcane Gold background, Ink Blue inverted text. 4px outer gold glow pulsing at 1Hz. Never occludes the gold counter.

**Settlement overlays — panel-level only; board, HUD, and gold counter remain visible throughout:**

| Case | Visual treatment | Duration |
|---|---|---|
| YOU WON | Arcane Gold overlay (85% opacity); "YOU WON" Heavy 3× Ink Blue with Prism White stroke; card art arcs to hand tray | 1.5s |
| OPPONENT WON | Ink Blue overlay (75% opacity); "OPPONENT WON" Heavy 2.5× Ivory | 1.5s |
| NO BIDS — CARD LOST | Panel desaturates to grayscale over 200ms; "NO BIDS" Heavy 2.5× + "CARD LOST" Regular 1.25× Ivory; card art fades to 0 over 400ms | 1.0s |

---

### VA.6 — Audio Requirements

**Auction-phase audio:** 7 events owned by `auction-system.md` Visual/Audio section. This GDD maps trigger moments only; tonal specs are owned there.

**Shop/Draft-specific audio (owned by this GDD):**

| Event | Trigger | Character |
|---|---|---|
| DRAFT_INITIAL entry | `S2CDraftOffering` received | Warm bright sting, 2–3s, ascending major phrase (Wakfu "Possibility" register) |
| DRAFT_INITIAL purchase confirm | `S2CCardAcquired` received | Single warm chime, higher pitch than shop confirm; 300–400ms |
| DRAFT_INITIAL budget depleted | `local_gold` reaches 0 | Single soft low bell; 1 beat |
| DRAFT_SHOP entry | Panel activates | Brief descending acoustic phrase, 1–2s; contrasts clearly with auction urgency tone |
| DRAFT_SHOP purchase confirm | `S2CCardAcquired` received in DRAFT_SHOP | Single warm chime, slightly lower pitch than DRAFT_INITIAL confirm; 300–400ms |
| Refresh swoosh | `C2SRefreshShop` sent (immediate feedback) | Paper/card shuffle, 200–300ms |
| Refresh failed | 5s timeout fires | Reversed swoosh, 150ms, low alarm register |
| Ready signal | `C2SSignalReady` sent | Short ascending two-note phrase, 200–300ms |
| Ready retracted | `C2SSignalReady { retract: true }` sent | Single descending counterpart note, same duration |
| Timer red zone entry (<5s) | First red-zone frame (all phases) | Shared countdown tick cue — same audio across all phases for "critical timer" |

**Audio principle:** No looping ambient underscore in DRAFT_INITIAL or DRAFT_SHOP. Any shop entry sting ducks to silence over 200ms when the auction urgency tone begins.

---

### VA.7 — Accessibility

**Rarity colorblind safety:**
1. Shape differentiation: Rare = circular gem · Epic = diamond gem · Legendary = star/octagonal gem (rarity distinguishable by shape independent of color)
2. Always-on rarity text label beneath each badge (10px minimum)

**Timer bar colorblind safety:**
1. Bar fill length is the primary zone-independent urgency signal
2. <5s vertical scale pulse is a shape signal independent of color
3. Countdown tick audio provides non-visual urgency signal
4. No separate colorblind mode required — shape, animation, and audio redundancies cover all common colorblind types

**Minimum tap targets:** Bid confirm, Refresh button, and DRAFT_INITIAL card slots all meet the 44×44 CSS px minimum per art bible Section 7.7.

📌 **Asset Spec** — Visual/Audio requirements are defined. After the art bible is approved, run `/asset-spec system:shop-auction-ui` to produce per-asset visual descriptions, dimensions, and generation prompts from this section.

## UI Requirements

The Shop/Auction UI is implemented as a `bevy_ui` node tree overlay coexisting with Board Rendering in screen space.

**Layout contract with Board Rendering:** The board occupies the majority of the screen (center + top). The Shop/Auction UI panel area sits at the bottom of the screen. Exact pixel split to be defined by the UX spec (`design/ux/shop-auction-ui.md`). The board must not be occluded in any Shop/Auction UI panel state.

**Node tree structure (intent):**
- `ShopAuctionUiRoot` — full-screen z-layer above board; pointer-events active only in the panel area
  - `DraftInitialPanel` — visible during DRAFT_INITIAL only
  - `AuctionPanel` — visible during DRAFT_AUCTION only
    - `ShopFooterStrip` — child of AuctionPanel; expands to become DRAFT_SHOP panel on transition
  - `ShopPanel` — visible during DRAFT_SHOP only (the expanded ShopFooterStrip)

**Transition implementation:** Panel transitions (auction down / shop expand up) driven by `bevy_tweening` over 350ms. **`Tween<Transform>` is NOT viable** — bevy_ui panel position is controlled by `Node` layout properties, not `Transform`. Implementer must provide a custom `bevy_tweening::Lens` that interpolates the relevant `Node` field (e.g., `bottom: Val::Percent(...)`) on the panel's `Node` component. Slot content is already populated before the transition begins; no data loading during animation. Verify final field name and Lens pattern with `liv-bevy-018`.

**bevy_ui component requirements (Bevy 0.18):**
- Panels use `Node`, `BackgroundColor`, `BorderColor`, `BorderRadius` (Required Components API — no `NodeBundle`)
- Text uses `Text` component; `LineHeight` is a required component on `Text` in Bevy 0.18 — every spawned text node must include it (verify exact constructor with `liv-bevy-018`). The Heavy 2.5× numerals in VA.4 must specify `LineHeight` at spawn time.
- Timer bar: width is set via the `Node` component's width field (`Val::Percent(fill_pct * 100.0)`) — `Style.width` does **not** exist in Bevy 0.18 (the `Style` component was merged into `Node` in 0.15). Update each frame the bar is visible (see Performance section for dirty-flag note).
- **Bid input — preset buttons only (OQ1 RESOLVED):** Three discrete `Button` widgets (+1g/+3g/+5g) per Detailed Design Rule 4. No text-input widget required. The earlier "HIGH risk" assessment around bevy_ui 0.18 text input is moot for this system.
- All panels use `Visibility::Hidden` when inactive (not despawned) to preserve state across phase transitions. **Note:** `Visibility::Hidden` suppresses rendering but layout participation behavior in Bevy 0.18 is unverified — if hidden nodes still participate in flex layout, the "zero layout cost" claim under Performance may not hold. Verify with `liv-bevy-018`; if Hidden retains layout cost, switch to `Display::None` (or 0.18 equivalent) for inactive panels.

**Z-ordering (lowest to highest):**
1. Board (rendered by Board Rendering)
2. Shop/Auction UI panels (auction / shop / DRAFT_INITIAL grid)
3. Settlement overlays (panel-scoped — never occlude HUD or board chrome; see VA.5)
4. HUD chips (gold counters, round number) — topmost, never occluded

Settlement overlays are bounded to the auction-panel area only; they may dim the panel content but must not extend over the HUD chips or the board.

**Performance:**
- Timer bar: dirty-flag update — only write the `Node` width field when `fill_pct` changes by >0.0005 (revised from 0.005 to avoid visible quantized stepping at 60 FPS — DRAFT_AUCTION drains ~0.00083 per frame, DRAFT_INITIAL ~0.00037 per frame).
- Gold counters: update only on `S2CGoldUpdate`/`S2CGoldBroadcast`, not continuously.
- All panels hidden during RESOLUTION — layout cost depends on whether `Visibility::Hidden` removes layout participation in Bevy 0.18 (see UI Requirements note above). If not, switch to `Display::None`.

📌 **UX Flag — Shop/Auction UI:** This system has UI requirements. Run `/ux-design` for the `shop-auction-ui` screen before writing epics. Stories referencing this UI must cite `design/ux/shop-auction-ui.md`, not this GDD directly.

## Acceptance Criteria

All BLOCKING criteria require an automated test in `tests/unit/shop_auction_ui/`. ADVISORY criteria require manual walkthrough or screenshot evidence in `production/qa/evidence/`.

### H.1 — Formula Unit Tests (all BLOCKING)

| # | Criterion | Gate |
|---|---|---|
| SAU-F1 | **GIVEN** `gold = 8`, `reserved_gold = 5` (from `S2CGoldBroadcast`), **WHEN** `local_free_gold` is computed, **THEN** result is 3 | BLOCKING |
| SAU-F2 | **GIVEN** `gold = 3`, `reserved_gold = 5` (server invariant violation), **WHEN** `local_free_gold` is computed, **THEN** result saturates to 0 (no u32 underflow) | BLOCKING |
| SAU-F3 | **GIVEN** `S2CGoldUpdate` arrives with `reserve_mana = 4` AND `S2CGoldBroadcast` has NOT arrived, **WHEN** preset button affordability is evaluated, **THEN** `reserved_gold = 0` (not sourced from `S2CGoldUpdate.reserve_mana`) | BLOCKING |
| SAU-F4 | **GIVEN** `timer_remaining_ms = 6000`, `timer_max_ms = 20000`, **WHEN** `timer_bar_fill_pct` is computed, **THEN** result = 0.30 (±f32 tolerance) | BLOCKING |
| SAU-F5 | **GIVEN** `timer_remaining_ms = -50.0` (drift undershot), **WHEN** `timer_bar_fill_pct` is computed, **THEN** result = 0.0 (clamped, not negative) | BLOCKING |
| SAU-F6 | **GIVEN** `timer_remaining_ms = 20100.0`, `timer_max_ms = 20000`, **WHEN** `timer_bar_fill_pct` is computed, **THEN** result = 1.0 (clamped at upper bound) | BLOCKING |
| SAU-F7 | **GIVEN** DRAFT_AUCTION, `timer_remaining_ms = 10000`, **WHEN** `timer_color_zone` is evaluated, **THEN** zone is Yellow (>10000 = Green; ≤10000 = Yellow) | BLOCKING |
| SAU-F8 | **GIVEN** DRAFT_AUCTION, `timer_remaining_ms = 5000`, **WHEN** `timer_color_zone` is evaluated, **THEN** zone is Red | BLOCKING |
| SAU-F9 | **GIVEN** DRAFT_AUCTION, `timer_remaining_ms = 15000`, **WHEN** `timer_color_zone` is evaluated, **THEN** zone is Green | BLOCKING |
| SAU-F10 | **GIVEN** `refresh_base_cost = 1`, `refresh_cap = 1`, `refresh_count_this_draft = 0`, **WHEN** `displayed_refresh_cost` is computed, **THEN** result = 1 | BLOCKING |
| SAU-F11 | **GIVEN** same config, `refresh_count_this_draft = 1`, **WHEN** `displayed_refresh_cost` is computed, **THEN** result = 2 | BLOCKING |
| SAU-F12 | **GIVEN** same config, `refresh_count_this_draft = 5`, **WHEN** `displayed_refresh_cost` is computed, **THEN** result = 2 (cap: `min(5, 1) = 1`) | BLOCKING |
| SAU-F13 | **GIVEN** `local_free_gold = 3`, `current_price = 0`, **WHEN** preset button affordability is evaluated, **THEN** +1g enabled (3 ≥ 1) AND +3g enabled (3 ≥ 3) AND +5g disabled (3 < 5) | BLOCKING |
| SAU-F13b | **GIVEN** `local_free_gold = 2`, `current_price = 0`, **WHEN** preset button affordability is evaluated, **THEN** +1g enabled (2 ≥ 1) AND +3g disabled (2 < 3) AND +5g disabled (2 < 5) | BLOCKING |
| SAU-F14 | **GIVEN** DRAFT_INITIAL: `timer_remaining_ms = 15000` → Yellow; 5000 → Red; 5001 → Yellow, **WHEN** `draft_timer_color_zone` is evaluated for each, **THEN** zones match | BLOCKING |
| SAU-F15 | **GIVEN** DRAFT_SHOP: `timer_remaining_ms = 5000` → Red; 5001 → Neutral, **WHEN** `draft_timer_color_zone` is evaluated, **THEN** zones match | BLOCKING |

### H.2 — State Machine / Logic Tests (BLOCKING unless noted)

| # | Criterion | Gate |
|---|---|---|
| SAU-DI1 | **GIVEN** DRAFT_INITIAL, `local_gold = 2`, `card_cost = 3`, **WHEN** slot is clicked, **THEN** no `C2SPurchaseCard` is sent | BLOCKING |
| SAU-DI2 | **GIVEN** DRAFT_INITIAL, `local_hand_size = 10`, **WHEN** any slot is clicked, **THEN** no `C2SPurchaseCard` is sent | BLOCKING |
| SAU-DI3 | **GIVEN** DRAFT_INITIAL, `local_gold = 5`, `card_cost = 2`, `local_hand_size = 9`, **WHEN** slot is clicked, **THEN** exactly one `C2SPurchaseCard { card_id }` is sent | BLOCKING |
| SAU-DI4 | **GIVEN** DRAFT_INITIAL, any refresh mechanism triggered, **WHEN** message queue is checked, **THEN** no `C2SRefreshShop` was sent | BLOCKING |
| SAU-DI5 | **GIVEN** DRAFT_INITIAL, `timer_remaining_ms = 0`, **WHEN** slot is clicked, **THEN** no `C2SPurchaseCard` sent AND panel remains visible awaiting `S2CPhaseChanged(PLACEMENT)` | BLOCKING |
| SAU-DI6 | **GIVEN** `S2CDraftOffering` contains a known set of cards with mixed rarities and costs, **WHEN** 3×3 grid is populated, **THEN** cards appear in rarity-descending then cost-descending order | BLOCKING |
| SAU-DA1 | **GIVEN** DRAFT_AUCTION, `local_free_gold = 6`, `current_price = 0`, **WHEN** preset buttons are evaluated, **THEN** all three (+1g/+3g/+5g) are enabled (6 ≥ 1, 6 ≥ 3, 6 ≥ 5) | BLOCKING |
| SAU-DA2 | **GIVEN** `S2CAuctionCard` arrives AND `local_hand_size = 10`, **WHEN** panel initializes, **THEN** all three preset buttons are immediately disabled with "Hand full — play a card to bid" | BLOCKING |
| SAU-DA3 | **GIVEN** DRAFT_AUCTION, `local_free_gold = 2`, `current_price = 0`, **WHEN** button states are evaluated, **THEN** +1g enabled AND +3g disabled AND +5g disabled | BLOCKING |
| SAU-DA4 | **GIVEN** all preset buttons are disabled ("Bid sent — waiting..."), **WHEN** `S2CAuctionBidRejected { reason }` arrives, **THEN** preset buttons re-enabled (per affordability) AND toast notification matches the reason code | BLOCKING (state); ADVISORY (toast text) |
| SAU-DA5 | **GIVEN** `S2CAuctionBidRejected` with each of: `InsufficientGold`, `AmountTooLow`, `AlreadyLeader`, `HandFull`, `AuctionExpired` (5 cases), **WHEN** each arrives, **THEN** mapped toast text matches the GDD table | BLOCKING |
| SAU-DA6 | **GIVEN** `S2CAuctionBidAccepted { bidder: local_player, amount: 5 }` arrives, **WHEN** leader state is updated, **THEN** `current_leader == local_player` AND all three preset buttons are disabled AND "YOU ARE LEADING" badge is shown | BLOCKING |
| SAU-DA7 | **GIVEN** `S2CAuctionBidAccepted { bidder: opponent, amount: 7 }` arrives, **WHEN** state is updated, **THEN** `current_price = 7` AND preset buttons re-enabled per new affordability (buttons needing `free_gold >= 7 + offset` evaluated) | BLOCKING |
| SAU-DA8 | **GIVEN** local timer drains to 0 AND no `S2CAuctionSettled` has arrived, **WHEN** locally expired state activates, **THEN** `timer_bar_fill_pct = 0.0` AND all three preset buttons are disabled | BLOCKING |
| SAU-DA9 | **GIVEN** `local_free_gold = 4`, `current_price = 0` (all buttons enabled), **WHEN** `S2CGoldBroadcast { gold: 2, reserved_gold: 0 }` arrives, **THEN** +1g enabled (2 ≥ 1) AND +3g disabled (2 < 3) AND +5g disabled (2 < 5) | BLOCKING |
| SAU-DA10 | **GIVEN** locally expired state, **WHEN** `S2CAuctionBidAccepted { new_timer_ms: 8000 }` arrives, **THEN** preset buttons re-enabled (per affordability) AND `timer_bar_fill_pct` target = 0.40 | BLOCKING (state); ADVISORY (animation) |
| SAU-DS1 | **GIVEN** non-auction round: `S2CPhaseChanged(DRAFT_SHOP)` arrives without `S2CShopSlots`, **WHEN** panel state is checked, **THEN** panel is not yet interactive; **WHEN** `S2CShopSlots` arrives, **THEN** panel activates | BLOCKING |
| SAU-DS2 | **GIVEN** `refresh_count_this_draft = 0`, `C2SRefreshShop` sent but `S2CShopSlots` not yet arrived, **WHEN** counter is read, **THEN** count = 0; **WHEN** `S2CShopSlots` arrives, **THEN** count = 1 | BLOCKING |
| SAU-DS3 | **GIVEN** `refresh_count_this_draft = 2` at end of DRAFT_SHOP, **WHEN** next DRAFT_SHOP begins, **THEN** count resets to 0 AND `displayed_refresh_cost = 1` | BLOCKING |
| SAU-DS4 | **GIVEN** `local_hand_size = 10` in DRAFT_SHOP, **WHEN** panel evaluates slot states, **THEN** all slots are non-clickable AND Refresh button remains enabled (if `local_gold >= refresh_cost`) | BLOCKING |
| SAU-DS5 | **GIVEN** `C2SRefreshShop` sent 5 seconds ago with no `S2CShopSlots` response, **WHEN** timeout fires, **THEN** Refresh button re-enabled AND `refresh_count_this_draft` unchanged | BLOCKING |
| SAU-DS6 | **GIVEN** two different slots clicked rapidly (both valid), **WHEN** message queue is checked, **THEN** two `C2SPurchaseCard` messages sent AND only each clicked slot is pending (not all slots locked) | BLOCKING |
| SAU-DS7 | **GIVEN** `C2SPurchaseCard` is in-flight AND `S2CPhaseChanged(PLACEMENT)` arrives, **WHEN** late confirmation arrives, **THEN** slot restored to pre-click state AND gold unchanged | BLOCKING |
| SAU-EG1 | **GIVEN** `S2CPhaseChanged(DRAFT_AUCTION)` arrives before `S2CAuctionCard`, **WHEN** state is checked, **THEN** auction panel NOT activated; **WHEN** `S2CAuctionCard` arrives, **THEN** panel activates normally | BLOCKING |
| SAU-EG2 | **GIVEN** AUCTION_ACTIVE state AND `S2CShopSlots` arrives, **WHEN** footer is checked, **THEN** NOT updated mid-auction; **WHEN** DRAFT_SHOP becomes active post-transition, **THEN** buffered slots applied | BLOCKING |
| SAU-EG3 | **GIVEN** `S2CAuctionBidAccepted` and `S2CAuctionSettled` arrive in the same delivery batch (last-tick bid), **WHEN** the client processes them in arrival order, **THEN** the bid-accepted state (current_price, current_leader, timer) is applied first, followed immediately by the settlement transition; no intermediate "in-flight resolved" frame is rendered | BLOCKING |
| SAU-EG4 | **GIVEN** only `S2CPhaseChanged(DRAFT_AUCTION)` has arrived (no `S2CAuctionCard`), **WHEN** state is checked, **THEN** auction panel NOT activated; **GIVEN** only `S2CAuctionCard` has arrived (no `S2CPhaseChanged`), **THEN** auction panel renders in "preparing..." state with timer greyed and countdown not started | BLOCKING |
| SAU-DI7 | **GIVEN** DRAFT_INITIAL is active and player clicks Ready, **WHEN** message queue is checked, **THEN** exactly one `C2SSignalReady { retract: false }` is sent AND button label changes to "Retract Ready" | BLOCKING |
| SAU-DI8 | **GIVEN** DRAFT_INITIAL is active and player has signalled ready, **WHEN** player clicks a purchasable slot, **THEN** `C2SPurchaseCard` is sent (grid remains interactive while ready) | BLOCKING |
| SAU-DI9 | **GIVEN** DRAFT_INITIAL Ready clicked, **WHEN** player clicks "Retract Ready", **THEN** exactly one `C2SSignalReady { retract: true }` is sent AND button label reverts to "Ready" | BLOCKING |
| SAU-DA11 | **GIVEN** preset bid button +3g is clicked, **WHEN** the in-flight state activates, **THEN** the +3g button label reads "BIDDING..." at 80% opacity AND the +1g/+5g buttons render in standard generic-disabled state (~30% opacity) | BLOCKING (state); ADVISORY (visual treatment) |
| SAU-DA12 | **GIVEN** in-flight "BIDDING..." state on +3g button, **WHEN** `S2CAuctionBidAccepted` or `S2CAuctionBidRejected` arrives, **THEN** the +3g button label reverts to "+3g" AND all three buttons re-evaluate per Rule 5 affordability | BLOCKING |
| SAU-DS8 | **GIVEN** DRAFT_SHOP refresh button is clicked, **WHEN** the click handler returns, **THEN** the refresh button is disabled in the same frame (before any subsequent click event can fire) AND exactly one `C2SRefreshShop` is sent | BLOCKING |
| SAU-F16 | **GIVEN** `current_price = 3`, **WHEN** `auction_border_color_tier` is computed, **THEN** result is InkBlue | BLOCKING |
| SAU-F17 | **GIVEN** `current_price = 4`, **WHEN** `auction_border_color_tier` is computed, **THEN** result is AuctionAmber | BLOCKING |
| SAU-F18 | **GIVEN** `current_price = 7`, **WHEN** `auction_border_color_tier` is computed, **THEN** result is DeepAmber | BLOCKING |
| SAU-F19 | **GIVEN** `current_price = 10`, **WHEN** `auction_border_color_tier` is computed, **THEN** result is CrimsonAmber | BLOCKING |

### H.3 — Visual / Interaction Evidence (ADVISORY)

| # | Criterion | Evidence |
|---|---|---|
| SAU-V1 | DRAFT_INITIAL header reads "DRAFT OFFERING — ONE TIME ONLY · NO REFRESH · 5g BUDGET" | Manual walkthrough |
| SAU-V2 | First-session tooltip appears on activation; dismisses and does not reappear | Manual walkthrough |
| SAU-V3 | Purchased slot shows "BOUGHT" overlay at same grid position; no reflow of remaining slots | Screenshot |
| SAU-V4 | DRAFT_INITIAL timer: neutral → yellow cross-fade at 15s → red cross-fade at 5s (no snap) | Manual walkthrough |
| SAU-V5 | DRAFT_AUCTION shop footer: read-only slots, no refresh button present | Screenshot |
| SAU-V6 | DRAFT_AUCTION timer bar colors: green >10s, yellow 5–10s, red ≤5s; all transitions cross-fade | Manual walkthrough |
| SAU-V7 | `S2CAuctionBidAccepted`: timer bar eases out to new position (~120–150ms, with brief bar flash); no snap | Manual walkthrough |
| SAU-V8 | "YOU ARE LEADING" badge visible when local player is leader; all three preset bid buttons are disabled | Screenshot |
| SAU-V9 | Locally expired (1500ms+): "Awaiting server..." label visible beneath frozen timer bar | Manual walkthrough |
| SAU-V10 | Settlement: correct overlay per outcome; panel slides down, shop expands up (~350ms) for all 3 `S2CAuctionSettled` cases | Manual walkthrough (3 cases) |
| SAU-V11 | DRAFT_SHOP timer starts when panel expansion animation completes (not during the 350ms transition) | Manual walkthrough |
| SAU-V12 | Refresh button label: "REFRESH · 1g" → "REFRESH · 2g" → stays at "REFRESH · 2g" on subsequent refreshes | Manual walkthrough |
| SAU-V13 | Opponent's free gold (`S2CGoldBroadcast.gold - reserved_gold`) visible in auction panel | Screenshot |
| SAU-V14 | `S2CPhaseChanged(PLACEMENT)` during settlement animation cancels animation immediately (no delay to phase entry) | Manual walkthrough |

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ1 | ~~Bid input text field implementation.~~ **RESOLVED** — Bid input changed to preset buttons (+1g/+3g/+5g) only. No free-form text field. **Fantasy tradeoff acknowledged:** preset stride was chosen for friend-scope implementation simplicity (no bevy_ui 0.18 text-input research, clearer touch targets, fewer error states). The cost: +2g/+4g squeeze-bids — explicitly called for by the auction-system fantasy ("calibrate next bluff at exact moment it costs the most") — are no longer expressible. The auction-system.md fantasy text should be updated in a future pass to match. | — | Closed 2026-04-30 |
| OQ2 | **DRAFT_INITIAL tooltip dismiss persistence.** The one-time "no refresh" tooltip must not reappear after the first dismiss. Storage location for this flag (browser `localStorage` for WASM target, or a player preferences resource?) must be resolved during WASM client architecture review. | Engine Programmer / Client Architecture | Before M2 UI story implementation |
| OQ3 | ~~`S2CGoldBroadcast` must include `reserved_gold`.~~ **RESOLVED** — `network-protocol.md` already includes `{ player_id: PlayerId, gold: u32, reserved_gold: u32 }` (NP line 104, added 2026-04-29). No further update needed. | — | Closed 2026-04-30 |
| OQ4 | **Screen layout split with Board Rendering.** Exact pixel/percentage allocation between the board area and the Shop/Auction UI bottom panel is undefined. **Prereq:** UX spec also needs hand-tray vertical extent (from `hand-ui.md`) and HUD chip position (from `hud.md`, not yet authored) before it can finalize a layout. Must be specified in the UX spec (`/ux-design shop-auction-ui`) before any layout implementation begins. | UX Designer | Before `/ux-design` is run; HUD GDD must exist |
| OQ5 | ~~`C2SSignalReady` message registration.~~ **RESOLVED** — `network-protocol.md` already registers `C2SSignalReady { retract: bool }` for both DRAFT_INITIAL and DRAFT_SHOP (NP line 48). GDD updated 2026-04-30 to add Ready button to DRAFT_INITIAL panel (matches RSM Rule 12 + Player Inputs table). | — | Closed 2026-04-30 |
| OQ6 | **Lightyear reliable channel cross-type FIFO ordering** is unverified — see `network-protocol.md` OQ3. This GDD has been authored to **not depend** on cross-type FIFO (Rule 1 buffers both `S2CAuctionCard` and `S2CPhaseChanged` until both arrive). If NP OQ3 resolves with strict FIFO guarantees, the buffer becomes a no-op for the common case but remains a correctness guard. No further GDD changes required regardless of NP OQ3 outcome. | Network Programmer (informational only) | Track NP OQ3; informational |
| OQ7 | **Bidirectional dependency updates pending** — `economy-system.md` should list this GDD as `S2CGoldBroadcast` consumer; `game-config.md` should list this GDD as downstream consumer of timer + refresh knobs. Self-flagged in Dependencies section "Bidirectionality notes" (lines for economy-system / game-config). | Whoever next edits those GDDs | Before M2 UI story implementation |
| OQ8 | **`S2CShopSlots` buffer application timing.** Rule 1a (DRAFT_SHOP, auction rounds) says "shop slots already populated — no new `S2CShopSlots` arrives." Edge Case (line ~309) says buffer arriving during DRAFT_AUCTION applies at transition completion. Same message — clarify whether the buffered `S2CShopSlots` is applied before or after the panel-expansion animation completes. Suggested resolution: apply at transition-complete moment (consistent with "DRAFT_SHOP timer starts when expansion completes"). | Lead Programmer | Before DRAFT_SHOP rendering story implementation |
