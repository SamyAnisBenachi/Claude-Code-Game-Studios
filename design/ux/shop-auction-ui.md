# UX Spec: Shop / Auction UI

> **Status**: In Design - UX review repair applied
> **Author**: user + ux-designer
> **Last Updated**: 2026-05-05
> **Journey Phase(s)**: DRAFT_INITIAL, DRAFT_AUCTION, DRAFT_SHOP
> **Template**: UX Spec
> **Input Methods**: Mouse + Keyboard (primary: mouse click). No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard
> **Source GDD**: `design/gdd/shop-auction-ui.md`

---

## Purpose & Player Need

The Shop / Auction UI is the player's economic decision surface. It answers three questions:

1. In DRAFT_INITIAL: which one-time cards define my starting direction?
2. In DRAFT_AUCTION: how much gold am I willing to reveal and reserve for the contested card?
3. In DRAFT_SHOP: what do I buy, refresh, or skip before placement begins?

The player arrives wanting to make a fast, informed economic decision under a visible timer. The UI must show cost, affordability, opponent gold, timer pressure, and outcome state without advising the player which choice is best.

If this screen is hard to use, the game loses the "Auction as signature" pillar. Players misread affordability, click unavailable actions, miss the timer, or feel punished by hidden server state rather than outplayed by the opponent.

---

## Player Context on Arrival

No `design/player-journey.md` exists yet, so this spec assumes context from the GDD and HUD design.

| Phase | Arrival Context | Expected Player State | Design Response |
|---|---|---|---|
| DRAFT_INITIAL | First game economy decision after lobby/start | Curious, plan-forming, moderately time-aware | Bright 3x3 offering, clear one-time header, visible 45s timer, no refresh affordance |
| DRAFT_AUCTION | Auction round begins, contested card announced | Pressured, watchful, adversarial | Board replacement, large card and price, own/opponent free gold side by side, three immediate bid buttons |
| DRAFT_SHOP | After auction settlement or non-auction shop entry | Calculating, lower-pressure | Compact bottom shop panel, three slots, refresh cost, ready/retract status |

Players can arrive voluntarily only through normal game phase progression. They do not navigate here from menus. Server messages and the Round State Machine own all entries and exits.

---

## Navigation Position

```text
IN-GAME HUD
  -> DRAFT_INITIAL panel
      -> PLACEMENT
  -> DRAFT_AUCTION panel
      -> settlement
      -> DRAFT_SHOP panel
      -> PLACEMENT
  -> DRAFT_SHOP panel (non-auction round)
      -> PLACEMENT
```

This is not a standalone screen in menu navigation. It is a phase-specific in-game UI layer inside `ShopAuctionUiRoot`, above Board Rendering and below HUD chips.

HUD continuity requirements:

- Top strip remains visible in all states: objective dots, phase label, round number, class figurines.
- HUD gold counters remain visible and topmost, never occluded by panels or settlement overlays.
- Hand row remains visible at the bottom edge where the phase allows it; acquisition feedback targets the hand area.
- During DRAFT_AUCTION, the board zone is replaced by the auction panel. This is the explicit exception to the general "board must not be occluded" rule.

---

## Entry & Exit Points

### Entry

| Entry Source | Trigger | Player Carries This Context |
|---|---|---|
| RSM phase change plus offering data | `S2CPhaseChanged(DRAFT_INITIAL)` and `S2CDraftOffering` both available | Starting gold, hand size, 9-card draft offering, 45s timer |
| Auction card first | `S2CAuctionCard` arrives before `S2CPhaseChanged(DRAFT_AUCTION)` | Card art, name, rarity, starting price, no active countdown |
| Auction phase first | `S2CPhaseChanged(DRAFT_AUCTION)` arrives before `S2CAuctionCard` | Phase and timer duration buffered internally; visible UI does not activate yet |
| Auction active | `S2CAuctionCard` and `S2CPhaseChanged(DRAFT_AUCTION)` both available | Featured card, current price, own free gold, opponent free gold, locked shop footer slots |
| Shop after auction | `S2CAuctionSettled` terminal event; buffered shop slots applied before transition | Settlement outcome, pre-populated shop slots, timer starts after expand animation completes |
| Non-auction shop | `S2CPhaseChanged(DRAFT_SHOP)` and `S2CShopSlots` both available | Three shop slots, refresh count, refresh cost, local hand/gold state |

### Exit

| Exit Destination | Trigger | Notes |
|---|---|---|
| PLACEMENT | `S2CPhaseChanged(PLACEMENT)` during DRAFT_INITIAL | Draft panel dismisses; further purchase sends blocked |
| PLACEMENT | `S2CPhaseChanged(PLACEMENT)` during DRAFT_SHOP | Shop panel dismisses; further purchase/refresh sends blocked |
| DRAFT_SHOP | `S2CAuctionSettled` plus settlement/transition sequence | Auction is terminal; accepted/rejected bid effects after settlement are ignored |
| INACTIVE / next phase | Non-auction phase while `AUCTION_PREPARING` | Preparing panel clears buffer and dismisses |
| AUCTION_PREPARING error display | 10s elapse without `S2CPhaseChanged(DRAFT_AUCTION)` | Panel remains preparing with connection error until a phase change resolves it |

---

## Layout Specification

### Information Hierarchy

#### Global Priority

1. Current phase decision: draft pick, bid amount, shop purchase/refresh
2. Time remaining
3. Own and opponent gold, with own free gold used for affordability
4. Card identity, rarity, and cost/price
5. Ready/waiting state
6. Lower-priority future context, such as read-only shop footer during auction

#### DRAFT_INITIAL Priority

1. 9-card offering grid
2. One-time-only/no-refresh header
3. Own gold and hand capacity state
4. Timer bar and seconds
5. Ready/Retract Ready
6. First-session tooltip

#### DRAFT_AUCTION Priority

1. Current price and three bid commitments
2. Own free gold and opponent free gold
3. Featured card
4. Timer bar and leader state
5. Settlement/expired status
6. Read-only shop footer

#### DRAFT_SHOP Priority

1. Three shop slots
2. Purchase affordability and hand capacity
3. Refresh button and cost
4. Timer
5. Ready/Retract Ready status

### Layout Zones

Reference target: 16:9 desktop browser at 1920x1080. Layout must remain usable from 1366x768 upward and support in-game UI scaling from 75% to 150%.

| Zone | Screen Position | Contents | Behavior |
|---|---|---|---|
| HUD top strip | Top edge, unchanged from `design/ux/hud.md` | Objective dots, phase label, round, class figurines | Always visible above Shop/Auction UI |
| HUD bottom resources | Bottom edge, left/right clusters | Gold counters, timer readout, mana displays, hand row | Always visible; may dim during auction but never hidden |
| Board safe area | Center/top majority | Board rendering outside auction | Must not be covered by DRAFT_INITIAL or DRAFT_SHOP steady states |
| Panel area | Lower 35% of screen for DRAFT_INITIAL/DRAFT_SHOP | Draft grid or shop row | Uses bottom overlay/panel geometry |
| Auction takeover area | Center zone between top HUD and bottom HUD/hand | Auction panel | Replaces board zone during DRAFT_AUCTION |
| Overlay layer | Within active panel bounds only | Tooltip, toast, settlement overlay | Never covers HUD chips or top strip |

#### Vertical Layout Contract

The screen is divided into three protected vertical bands plus one active content band:

- **Top HUD reserve**: 96px at 100% UI scale, clamped by the global UI scale range. Shop/Auction panels, tooltips, toasts, and settlement overlays must not enter this reserve.
- **Bottom HUD/hand reserve**: 170px at 100% UI scale, clamped by the global UI scale range. The hand tray and bottom resource strip live here and remain visible in every Shop/Auction phase.
- **Active content band**: the remaining space between the top and bottom reserves. DRAFT_AUCTION owns this whole band as a board takeover. DRAFT_INITIAL and DRAFT_SHOP may use only the lower portion of this band.
- **Panel cap**: DRAFT_SHOP steady state should stay between 220px and 300px at 100% UI scale and never exceed 32% of viewport height. DRAFT_INITIAL may expand higher for the 3x3 grid but must preserve at least 42% of viewport height as readable board area above the panel at 1366x768.

HUD z-order is fixed: board world-space, Shop/Auction panels, panel-scoped overlays/toasts/tooltips, then HUD chips and hand tray. A card-acquisition animation may target the hand tray centerline, but its overlay mask must remain within the active panel band until the hand-owned animation takes over.

#### DRAFT_INITIAL Layout

The DRAFT_INITIAL panel sits in the bottom panel area but expands vertically enough to hold a readable 3x3 grid. The board behind it is dimmed to 40% opacity but remains spatially readable. The panel must not cover top HUD or HUD gold counters.

```text
+----------------------------------------------------------------+
| [class]       [objectives | phase | round]        [class]       |
|                                                                |
|                         BOARD DIMMED 40%                       |
|                                                                |
|     DRAFT OFFERING - ONE TIME ONLY - NO REFRESH                |
|     [first-session tooltip, above grid, non-occluding]         |
|                                                                |
|        [card] [card] [card]                                    |
|        [card] [card] [card]        [timer bar] [Ready]          |
|        [card] [card] [card]        [waiting text]               |
|                                                                |
| [own gold] [opp gold] [phase timer]      [mana/reserve/cap]     |
| [hand row]                                                      |
+----------------------------------------------------------------+
```

#### DRAFT_AUCTION Layout

The auction panel replaces the board zone. The contested card is centered. Price and bid controls anchor the player's eye below/right of the featured card. Own and opponent free gold sit side by side at equal visual weight. The read-only shop footer sits at the bottom of the auction panel above the hand/HUD strip.

```text
+----------------------------------------------------------------+
| [class]       [objectives | DRAFT_AUCTION | round] [class]      |
|                                                                |
|          YOUR FREE GOLD        CURRENT PRICE      OPP FREE GOLD |
|              8g                    5g                 6g       |
|                                                                |
|                         [ featured card ]                      |
|                         [ 240 x 360   ]                        |
|                                                                |
|      [leader/no leader]        [timer bar + seconds]            |
|                                                                |
|               [6g (+1)] [8g (+3)] [10g (+5)]                   |
|               or [YOU ARE LEADING]                             |
|                                                                |
|      Read-only next shop: [slot] [slot] [slot]                  |
| [own gold] [opp gold] [phase timer]      [mana/reserve/cap]     |
| [hand row]                                                      |
+----------------------------------------------------------------+
```

#### DRAFT_SHOP Layout

The shop is a bottom panel that leaves the board readable above it. Three slots are horizontally arranged. Refresh and Ready sit to the right of the row at desktop width; at narrower widths they stack below the row within the same panel.

```text
+----------------------------------------------------------------+
| [class]       [objectives | DRAFT_SHOP | round]    [class]      |
|                                                                |
|                             BOARD                              |
|                                                                |
|   SHOP                                                         |
|   [slot card]       [slot card]       [slot card]               |
|   cost/name         cost/name         cost/name                 |
|                                            [REFRESH · 1g]       |
|                                            [Ready]              |
|                                            [waiting text]       |
|   [timer bar]                                                   |
| [own gold] [opp gold] [phase timer]      [mana/reserve/cap]     |
| [hand row]                                                      |
+----------------------------------------------------------------+
```

### Component Inventory

| Component | Type | Content | Interactive | Pattern |
|---|---|---|---|---|
| DraftInitialPanel | Panel | Header, grid, timer, ready state | Container only | PTN-OVR-003 Phase Economy Panel |
| Draft offering slot | Purchasable card slot | Art, name, rarity, cost, bought/locked/pending state | Yes when affordable and timer active | PTN-INP-005 Shop Item Card |
| First-session tooltip | Tutorial callout | "These 9 cards are your only offering. No refresh. 5g to spend." | Dismiss only | PTN-FDB-006 Dismissible Tutorial Tooltip |
| Draft timer bar | Countdown bar + seconds | 45s timer, neutral/yellow/red states | No | PTN-DSP-005 variant |
| Ready button | Action button | Ready / Retract Ready | Yes | PTN-NAV-001 |
| Waiting text | Status label | "Waiting for opponent..." | No | PTN-DSP-004 style |
| AuctionPanel | Board-replacement panel | Featured card, price, gold, timer, bid controls, footer | Container only | PTN-OVR-004 Auction Decision Panel |
| Featured auction card | Card zoom | Art, name, rarity, starting/current price context | No | Card Frame Container variant |
| Current price counter | Resource counter | Current bid price | No | PTN-DSP-001 |
| Free-gold counters | Resource counters | Own and opponent `gold - reserved_gold` | No | PTN-DSP-001 |
| Bid buttons | Preset action buttons | `[current_price + offset]g (+offset)` for +1/+3/+5 | Yes when enabled | PTN-INP-004 Auction Bid Button |
| You are leading badge | State replacement | "YOU ARE LEADING" | No | PTN-DSP-011 Leader Badge |
| Auction timer bar | Countdown bar + seconds | 20s timer, green/yellow/red, correction easing | No | PTN-DSP-005 variant |
| ShopFooterStrip | Read-only preview strip | Three shop slots at 30% opacity | No | PTN-DSP-012 Read-only Shop Footer |
| Toast | Non-blocking feedback | Bid rejection, refresh failure | No | PTN-FDB-005 Notification Toast |
| Settlement overlay | Panel-scoped overlay | YOU WON, OPPONENT WON, NO BIDS / CARD LOST | No | PTN-OVR-002 Settlement Overlay |
| ShopPanel | Bottom economy panel | Three slots, refresh, ready, timer | Container only | PTN-OVR-003 Phase Economy Panel |
| Shop slot | Purchasable card slot | Art, name, rarity, cost, empty/dead state | Yes when valid | PTN-INP-005 Shop Item Card |
| Refresh button | Async action button | `REFRESH · 1g`, `REFRESH · 2g` | Yes when affordable and not in-flight | PTN-NAV-001 plus PTN-FDB-001 |
| Hand-full banner | Status banner | Phase-specific lockout text | No | PTN-FDB-002 style without error input |

Shop/Auction pattern dependencies resolved in `design/ux/interaction-patterns.md`:

- Auction Bid Button
- Shop Item Card
- Read-only Shop Footer
- Settlement Overlay
- Notification Toast
- Dismissible Tutorial Tooltip
- Leader Badge
- Phase Economy Panel
- Auction Decision Panel

---

## States & Variants

| State / Variant | Trigger | What Changes |
|---|---|---|
| Inactive | PLACEMENT, RESOLUTION, GAME_OVER | All panel roots use no active layout; no Shop/Auction input accepted |
| DRAFT_INITIAL buffering | Offering or phase missing | Panel blank; no timer; no card slots visible |
| DRAFT_INITIAL active | Offering and phase available | 3x3 sorted grid, timer, Ready visible |
| DRAFT_INITIAL first-session tooltip | First activation and tooltip not dismissed | Tooltip above grid/header, not covering cards |
| DRAFT_INITIAL purchase pending | Valid slot click sent | Clicked slot enters pending state only; other valid slots remain interactive |
| DRAFT_INITIAL purchased | `S2CCardAcquired` plus `S2CGoldUpdate` confirms | Slot remains in place, dimmed with "BOUGHT" overlay |
| DRAFT_INITIAL insufficient gold | Clicked card cost exceeds gold | No send; own gold counter flashes amber/red for 200ms |
| DRAFT_INITIAL hand full | `local_hand_size == 10` | Unowned slots locked; banner: "Hand full - cannot buy more cards." |
| DRAFT_INITIAL ready | Ready clicked | Button becomes Retract Ready; waiting text appears; grid remains interactive |
| DRAFT_INITIAL expired | Timer reaches 0 before PLACEMENT | Timer shows 0; panel freezes; clicks silently ignored until phase change |
| AUCTION_PREPARING | `S2CAuctionCard` before phase | Card and price visible; timer greyed; "Auction starting..." label; no countdown or bids |
| AUCTION_PREPARING timeout | 10s without auction phase | Label changes to "Connection error - awaiting server..." |
| AUCTION_ACTIVE | Card and phase both available | Featured card, current price, free gold, leader, timer, bid buttons, locked footer |
| Auction hand full | Hand size 10 at auction entry | Bid area disabled with "Hand full - no bids possible this auction" |
| Auction insufficient gold | `local_free_gold < current_price + 1` | All bid buttons disabled; "Insufficient gold" message in bid area |
| Auction partial affordability | Some preset commitments exceed free gold | Only unaffordable buttons disabled; labels remain visible |
| Auction bid in-flight | Enabled bid clicked | Clicked button reads "BIDDING..."; other buttons disabled; no further bid send |
| Auction local leader | Accepted local bid | Bid buttons hidden; "YOU ARE LEADING" badge fills button area |
| Auction opponent leader update gated | Opponent accepted bid before local gold broadcast | Buttons remain disabled/hidden until local `S2CGoldBroadcast` arrives |
| Auction rejected | `S2CAuctionBidRejected` | In-flight clears, buttons re-evaluate, mapped toast appears |
| Auction locally expired | Timer reaches 0 before settlement | Bar frozen at 0; bids disabled; "Auction ending..." then "Awaiting server..." if unresolved |
| Auction timer restored | Accepted bid arrives while locally expired | Bar eases to new fill, labels clear, bids re-evaluate |
| Auction settled local win | `winner == local_player` | "YOU WON" overlay; card feedback toward hand; transition to shop |
| Auction settled opponent win | `winner == opponent` | "OPPONENT WON" overlay; no local hand movement; transition to shop |
| Auction settled no bids | `winner == None` | "NO BIDS - CARD LOST"; card fades; transition to shop |
| DRAFT_SHOP buffering | Non-auction shop phase without slots | Panel not interactive until slots arrive |
| DRAFT_SHOP active | Phase and slots available, or post-auction transition complete | Three slots, refresh, ready, timer |
| DRAFT_SHOP purchase pending | Valid slot click sent | Only clicked slot pending; other slots remain individually interactive |
| DRAFT_SHOP purchase confirmed | `S2CCardAcquired` plus `S2CGoldUpdate` confirms clicked slot | Purchased card fades out; slot becomes a fixed-position empty/dead well, non-focusable and non-clickable, with no hover lift |
| DRAFT_SHOP empty/dead slot | Server slot is empty, pool-depleted, or purchase already confirmed | Column remains reserved; shows an inert "EMPTY" or "SOLD" well with no cost, no card art hover, and no C2S action |
| DRAFT_SHOP refresh in-flight | Refresh clicked | Refresh disabled same frame; all slots grey with "Refreshing..." |
| DRAFT_SHOP refresh timeout | 5s without `S2CShopSlots` | Refresh re-enabled; toast "Refresh failed - try again" for 2.0s; count unchanged |
| DRAFT_SHOP hand full | `local_hand_size == 10` | Slots locked; Refresh remains available if affordable |
| DRAFT_SHOP ready | Ready clicked | Button becomes Retract Ready; waiting text appears; shop remains interactive |
| Reduced motion | Accessibility setting enabled | Replaces scale/pulse/slide emphasis with fades/cuts and static tint changes |

---

## Interaction Map

Input scope: Mouse click primary. Keyboard Tab and Enter must reach all interactive controls. Esc dismisses tutorial tooltip when it is visible.

| Element | Action | Input | Immediate Feedback | Outcome |
|---|---|---|---|---|
| First-session tooltip | Dismiss | Click `Got it`, click outside tooltip, or Esc | Tooltip fades/cuts out | Dismiss flag stored; tooltip not shown again |
| Draft offering slot | Purchase | Click or keyboard Enter on focused slot | Slot enters pending state only if valid | Sends `C2SPurchaseCard { card_id }` |
| Draft offering slot, unaffordable | Attempt purchase | Click or Enter | Own gold counter flashes 200ms | No C2S send |
| Draft Ready button | Ready | Click or Enter | Button label changes to Retract Ready; waiting text appears | Sends `C2SSignalReady { retract: false }` |
| Draft Retract Ready button | Retract | Click or Enter | Button label changes to Ready; waiting text clears | Sends `C2SSignalReady { retract: true }` |
| Bid +1/+3/+5 button | Place bid | Click or Enter | Clicked button reads "BIDDING..."; all bids disabled | Sends `C2SPlaceBid { amount }` |
| Disabled bid button | Attempt bid | Click ignored | Disabled state remains; reason text already visible if panel-level | No C2S send |
| You are leading badge | None | None | Passive state | No action available |
| Shop slot | Purchase | Click or Enter | Clicked slot enters pending state only if valid | Sends `C2SPurchaseCard { card_id }` |
| Shop Refresh button | Refresh | Click or Enter | Button disables same frame; slots grey with "Refreshing..." | Sends `C2SRefreshShop {}` |
| Shop Ready button | Ready | Click or Enter | Button label changes to Retract Ready; waiting text appears | Sends `C2SSignalReady { retract: false }` |
| Shop Retract Ready button | Retract | Click or Enter | Button label changes to Ready; waiting text clears | Sends `C2SSignalReady { retract: true }` |
| Toast | Auto-dismiss | Timer only | Fades/cuts after display duration | No gameplay event |
| Settlement overlay | None | None | Outcome displayed | Transition proceeds automatically unless phase interrupt arrives |

Shop/Auction notification toasts use PTN-FDB-005 timing: 120ms fade in, 2.0s hold at full opacity, 120ms fade out. A replacement toast resets the hold timer rather than stacking vertically.

Keyboard focus order:

- DRAFT_INITIAL: tooltip dismiss if present, slots row-major 1-9, Ready/Retract Ready.
- DRAFT_AUCTION: +1, +3, +5 bid buttons when visible. If "YOU ARE LEADING" replaces them, no focusable item exists in the bid area.
- DRAFT_SHOP: slot 1, slot 2, slot 3, Refresh, Ready/Retract Ready.

No disabled control receives keyboard focus. A hidden control is removed from the focus order.

---

## Events Fired

| Player Action | Event / Message Fired | Payload / Data |
|---|---|---|
| Dismiss draft tooltip | Local preference write | `shop_auction_draft_tooltip_dismissed = true` |
| Purchase DRAFT_INITIAL card | `C2SPurchaseCard` | `{ card_id }` |
| Purchase DRAFT_SHOP slot | `C2SPurchaseCard` | `{ card_id }` |
| Click Refresh | `C2SRefreshShop` | `{}` |
| Click Ready in DRAFT_INITIAL | `C2SSignalReady` | `{ retract: false }` |
| Click Retract Ready in DRAFT_INITIAL | `C2SSignalReady` | `{ retract: true }` |
| Click Ready in DRAFT_SHOP | `C2SSignalReady` | `{ retract: false }` |
| Click Retract Ready in DRAFT_SHOP | `C2SSignalReady` | `{ retract: true }` |
| Click bid +1 | `C2SPlaceBid` | `{ amount: current_price + 1 }` |
| Click bid +3 | `C2SPlaceBid` | `{ amount: current_price + 3 }` |
| Click bid +5 | `C2SPlaceBid` | `{ amount: current_price + 5 }` |
| Click unaffordable card/bid/refresh | None | Client-side rejection only |
| Toast auto-dismiss | None | Visual state only |
| Settlement overlay complete | Local transition event | Starts auction-down/shop-up transition unless interrupted |

Persistent or server-authoritative state changes must not be committed optimistically. Purchases, refreshes, bid leadership, price, gold, and card ownership all wait for S2C confirmation.

---

## Transitions & Animations

### Panel Entry and Exit

| Transition | Standard Motion | Reduced Motion |
|---|---|---|
| DRAFT_INITIAL entry | Cards fan/stagger into 3x3 over 200ms; board dims to 40% | Grid appears instantly with 80ms fade; no stagger |
| DRAFT_INITIAL exit | Panel fades out on phase change | Instant cut or 80ms fade |
| AUCTION_PREPARING entry | Card/price appear in inactive panel; grey timer bar | Instant appearance with static grey bar |
| AUCTION_ACTIVE entry | Auction panel replaces board; card and price become visual apex | Instant board replacement with 80ms fade; no scale expansion |
| AUCTION to SHOP | Settlement overlay, auction panel slides down, shop panel expands up over 350ms | Overlay holds; auction hides and shop appears with crossfade; no slide/expand |
| DRAFT_SHOP entry non-auction | Bottom panel fades in with populated slots | Instant or 80ms fade |
| DRAFT_SHOP exit | Panel fades out on phase change | Instant cut or 80ms fade |

### Timer Behavior

| Timer | Duration Source | Color Zones | Motion |
|---|---|---|---|
| DRAFT_INITIAL | `S2CPhaseChanged.timer_duration_ms`, default 45s | Neutral >15s, yellow <=15s, red <=5s | 300ms cross-fade between zones; red pulse below 5s |
| DRAFT_AUCTION | `S2CPhaseChanged.timer_duration_ms`, default 20s; corrected by `S2CAuctionBidAccepted.new_timer_ms` | Green >10s, yellow <=10s and >5s, red <=5s | 300ms cross-fade; accepted bid eases bar to target over 120-150ms plus 60ms white flash |
| DRAFT_SHOP | `S2CPhaseChanged.timer_duration_ms`, default 30s; post-auction starts after shop expand complete | Neutral >5s, red <=5s | 300ms cross-fade; red pulse below 5s |

Reduced motion removes timer bar scale pulse and glow pulsing. Numeric seconds, fill length, and color/tint remain.

### Tint and Escalation

- Auction border uses the GDD tier map: 0-3 Pale Ink Blue, 4-6 Auction Amber, 7-9 Deep Amber, 10+ Crimson-Amber.
- Tier transitions cross-fade over 300ms in standard motion.
- Reduced motion keeps the final tier color but removes frame flicker, particle pulse, and repeated glow.
- Gold decrease uses a brief Auction Amber tint during the numeric delta.
- Disabled/read-only footer slots render at 30% opacity with no hover affordance.

### Settlement

| Settlement Case | Standard Motion | Reduced Motion |
|---|---|---|
| Local player wins | "YOU WON" overlay for 400ms; card moves/arcs to hand target; gold counter animates down | Overlay appears for 400ms; card appears in hand via non-moving flash/cut |
| Opponent wins | "OPPONENT WON" overlay for 400ms; no local card movement | Overlay appears/cuts; no motion |
| No bids | Panel desaturates over 200ms; card fades out over 400ms; "NO BIDS - CARD LOST" holds 1.0s | Static desaturated panel; card cuts/fades without scale |

`S2CPhaseChanged(PLACEMENT)` interrupts any settlement animation immediately. Phase entry is never delayed by visual polish.

---

## Data Requirements

| Data | Source System | Read / Write | Notes |
|---|---|---|---|
| Current phase | RSM / phase sink | Read | Sole authority for activation and dismissal |
| Phase timer duration | `S2CPhaseChanged.timer_duration_ms` | Read | Initializes DRAFT_INITIAL, DRAFT_AUCTION, DRAFT_SHOP timers |
| Draft offering cards | Card Acquisition: `S2CDraftOffering` | Read | 9 card ids; activate only with phase |
| Shop slots | Card Acquisition: `S2CShopSlots` | Read | 3 slots; buffer if received during auction |
| Card display data | Card database/assets | Read | Art, name, rarity, cost, badge, card id |
| Starting auction card | Auction System: `S2CAuctionCard` | Read | Card id and starting price; can create preparing state |
| Auction accepted bid | Auction System: `S2CAuctionBidAccepted` | Read | Bidder, amount, `new_timer_ms`; updates price/leader/timer target |
| Auction rejected bid | Auction System: `S2CAuctionBidRejected` | Read | Rejection reason maps to exact toast copy |
| Auction settlement | Auction System: `S2CAuctionSettled` | Read | Winner and amount; terminal for auction state |
| Own gold/current mana/reserve mana/mana cap | Economy: `S2CGoldUpdate` | Read | Drives HUD and purchase affordability for own player |
| Own and opponent free gold | Economy: `S2CGoldBroadcast` | Read | `gold - reserved_gold`; local broadcast gates opponent-accepted bid re-enable |
| Card acquired confirmation | Card Acquisition: `S2CCardAcquired` | Read | Confirms DRAFT_INITIAL and DRAFT_SHOP purchases; paired with `S2CGoldUpdate` before final purchased/empty slot state is shown |
| Hand size | Hand UI / card ownership state | Read | Hand full locks purchase/bid slots according to phase |
| Refresh count this draft | Card Acquisition/UI confirmed state | Read | Increments only on `S2CShopSlots` confirmation, not on send |
| Refresh cost config | Game Config | Read | Default 1g then 2g cap |
| Tooltip dismissed flag | Local preferences backed by browser `localStorage` | Read / Write | Boolean key `lanes_and_lies.shop_auction.draft_tooltip_dismissed`; write immediately on explicit button, outside click, or Esc dismiss |
| UI scale | Settings | Read | 75%-150% supported |
| Reduced motion | Settings | Read | Changes animation policy |
| Purchase intent | UI to Network Protocol | Write | `C2SPurchaseCard { card_id }`; no optimistic ownership |
| Refresh intent | UI to Network Protocol | Write | `C2SRefreshShop {}`; no optimistic refresh count |
| Bid intent | UI to Network Protocol | Write | `C2SPlaceBid { amount }`; no optimistic price/leader |
| Ready intent | UI to Network Protocol | Write | `C2SSignalReady { retract }` |

Architectural constraints:

- UI owns presentation state and local input gating only.
- UI does not own card pool, auction truth, gold truth, or phase truth.
- Late confirmations after phase change restore or ignore visual state according to GDD edge cases.
- Reconnect snapshot behavior is outside this UX file's visual layout scope, but reconnect must rebuild a coherent panel state without replaying transient animations.

---

## Accessibility

Standard tier. Source: `design/accessibility-requirements.md`.

| Requirement | UX Requirement |
|---|---|
| Minimum pointer target | All bid, refresh, ready, tooltip dismiss, and purchasable slot targets are at least 44x44 CSS px at 100% UI scale and remain reachable at 75%-150% scale |
| Keyboard navigation | All interactive controls reachable by Tab and activated by Enter; Esc dismisses tooltip |
| Focus indicators | Focused controls show a 2px Prism White outline or equivalent high-contrast focus ring |
| Text contrast | Body text at least 4.5:1; auction price counter at least 7:1 |
| Minimum text size | Resource counters at least 20px at 1080p; auction price at least 40px; card keyword text floor from card spec remains readable on hover/zoom |
| Color-independent state | Rarity uses badge shape plus text; timer urgency uses fill length and numeric seconds; disabled state uses opacity plus text/affordance; auction escalation always includes numeric price |
| Motion reduction | Auction entry, bid pulse, timer pulse, settlement movement, and panel slide/expand have reduced-motion alternatives |
| Critical audio backup | Timer final-5s audio has visual timer, color, and text backups |
| Cognitive load | Each phase has one primary decision area; read-only footer has low opacity and no hover to prevent false affordance |
| Tutorial prompt | First-session tooltip is dismissible, non-occluding, and should be retrievable later through Help when that screen exists |
| Motor safety | Ready is retractable. Refresh disables in the same frame to prevent double-send |

Bid accessibility decision:

- Auction bidding keeps the GDD/story behavior: immediate preset bid buttons with no separate confirmation step.
- Misclick mitigation is handled by preset total-commitment labels (`8g (+1)`), 44x44 minimum targets, keyboard focus rings, per-button affordability gating, same-frame in-flight disable, exact one-send semantics, and visible "BIDDING..." feedback on only the clicked button.
- The in-flight state is feedback, not a reversible confirmation. Once a valid bid click is sent, the UI does not offer local undo; server acceptance/rejection remains authoritative.
- `design/accessibility-requirements.md` must match this decision. OQ-SAU-UX-4 is closed.

Screen reader support:

- In-game screen reader support is not currently in scope because Bevy 0.18 browser accessibility passthrough is unresolved.
- If accessible names/roles become available, all buttons need semantic labels: "Bid 8 gold, plus 1", "Refresh shop for 1 gold", "Buy [card name] for [cost] gold", "Ready", "Retract Ready".

---

## Localization Considerations

| Element | Risk | Requirement |
|---|---|---|
| DRAFT_INITIAL header | Medium | Allow 2-line wrap at smaller widths; no text overlap with timer/Ready |
| Tooltip body | Medium | Tooltip supports up to 3 lines and expands vertically without covering cards |
| Bid labels | Low | Numeric format stable: `8g (+1)`; localize gold suffix only if economy text style changes globally |
| "BIDDING..." | Medium | Button can show a spinner/icon plus shortened text if localization exceeds width |
| "YOU ARE LEADING" | Medium | Badge supports 2-line wrap; text remains centered |
| Rejection toasts | Medium | Toast area supports 2 lines; no layout reflow of bid buttons |
| Refresh label | Low | `REFRESH · 1g` can localize within fixed button min width; allow 2-line wrap at narrow desktop |
| Hand-full banners | High | Banner supports 2 lines and does not cover cards |
| Settlement overlays | Medium | "OPPONENT WON" and "NO BIDS - CARD LOST" support 2-line layout |

Localization layout limits:

- All localized text containers in this spec must tolerate 40% string expansion without overlapping adjacent controls at 1366x768, 1920x1080, and 150% UI scale.
- Fixed numeric zones reserve width for at least three currency digits plus suffix (`999g`) and three timer digits (`999s`). If a debug/config value exceeds that limit, render compact overflow (`999+g`, `999+s`) in the fixed zone and keep the exact value in diagnostics/accessibility metadata.
- Bid labels keep the total commitment and increment visible together. If localized suffixes exceed width, the button may wrap to two lines (`8g` / `(+1)`) but may not hide the increment.
- Toasts and banners may wrap to two lines; they do not resize or push bid buttons, shop slots, timers, HUD chips, or the hand tray.

All currency and countdown numbers use game-specific formatting, not locale-specific money/date formatting.

---

## Acceptance Criteria

- [ ] `design/ux/shop-auction-ui.md` is referenced by Shop/Auction UI implementation stories before final visual evidence work begins.
- [ ] DRAFT_INITIAL opens only after both `S2CPhaseChanged(DRAFT_INITIAL)` and `S2CDraftOffering` are available, then displays a sorted 3x3 grid, timer, and Ready button.
- [ ] DRAFT_INITIAL first-session tooltip appears above the grid/header, does not cover any card slot, dismisses via explicit button/outside click/Esc, and does not reappear after dismissal.
- [ ] DRAFT_INITIAL purchase clicks send `C2SPurchaseCard` only for affordable cards while hand size is below 10 and timer is active.
- [ ] Purchased DRAFT_INITIAL slots remain in position with a "BOUGHT" overlay and no grid reflow.
- [ ] Ready/Retract Ready in DRAFT_INITIAL and DRAFT_SHOP sends the correct `C2SSignalReady { retract }` value and does not disable purchases before phase transition.
- [ ] DRAFT_AUCTION shows the featured card, current price, own free gold, opponent free gold, leader state, timer, and three preset bid buttons when active.
- [ ] Auction bid buttons show total commitment as primary text and increment as secondary text, for +1/+3/+5.
- [ ] Clicking a valid bid sends exactly one `C2SPlaceBid { amount }`, changes only the clicked button to "BIDDING...", and disables all bid buttons until S2C response.
- [ ] If the local player is leading, all bid buttons are hidden and "YOU ARE LEADING" fills the bid area.
- [ ] AUCTION_PREPARING displays card and price with a grey timer bar and no countdown, then either activates on DRAFT_AUCTION phase or shows connection error after 10s.
- [ ] DRAFT_AUCTION read-only shop footer shows three locked slots at 30% opacity and never sends purchase or refresh messages.
- [ ] Locally expired auction state freezes the timer at 0, disables bids, shows "Auction ending...", then shows "Awaiting server..." after 1500ms if unresolved.
- [ ] Settlement overlays display the correct local win, opponent win, or no-bid outcome and never cover HUD chips.
- [ ] Auction-to-shop transition pre-populates the ShopPanel before reveal, slides/dismisses auction and expands shop over 350ms in standard motion, and starts the DRAFT_SHOP timer only after expansion completes.
- [ ] DRAFT_SHOP shows three slots, Refresh with confirmed cost label, Ready/Retract Ready, and a timer without covering the top HUD or board-critical area.
- [ ] DRAFT_SHOP confirmed purchases fade the purchased card out into a fixed-position empty/dead slot well; remaining slots do not reflow, and empty/dead wells are non-focusable and send no purchase message.
- [ ] Refresh disables in the same frame as click, sends exactly one `C2SRefreshShop`, greys slots with "Refreshing...", and increments refresh count only on `S2CShopSlots`.
- [ ] All interactive controls are reachable by keyboard Tab/Enter in the focus orders defined above, with visible focus indicators.
- [ ] Reduced-motion mode removes panel slide/expand, repeated pulse, frame flicker, and card travel motion while preserving text, tint, numeric, and state-change feedback.
- [ ] First visible active panel content appears within 100ms after the required phase/data messages are both available; auction-to-shop reveal completes within the specified 350ms transition and never shows an empty ShopPanel flash.
- [ ] At 1366x768, 1920x1080, and 150% UI scale, no button text, tooltip, timer, toast, or overlay overlaps another required UI element.
- [ ] At 1366x768, 1920x1080, and 150% UI scale, all localized labels fit with 40% text expansion or use the specified two-line/compact numeric fallback without changing panel geometry.

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-SAU-UX-1 | ~~Exact persistent storage for the DRAFT_INITIAL tooltip dismissal flag.~~ **RESOLVED 2026-05-05** - store `lanes_and_lies.shop_auction.draft_tooltip_dismissed` in local preferences backed by browser `localStorage`. | - | Closed |
| OQ-SAU-UX-2 | ~~Hand tray / HUD / panel vertical contract missing for SAU-009.~~ **RESOLVED 2026-05-05** - top HUD reserve, bottom HUD/hand reserve, active content band, panel caps, and z-order are defined in Vertical Layout Contract. SAU-009 still verifies with screenshots. | - | Closed |
| OQ-SAU-UX-3 | ~~HUD/art direction says DRAFT_AUCTION replaces the board, while the Shop/Auction GDD UI Requirements say the board must not be occluded.~~ **RESOLVED 2026-05-04** — `design/gdd/shop-auction-ui.md` now states DRAFT_AUCTION is the explicit board-takeover exception; DRAFT_INITIAL and DRAFT_SHOP keep the board readable. | — | Closed |
| OQ-SAU-UX-4 | ~~Accessibility requirements request bid confirmation, but the current GDD and stories specify immediate preset bid buttons.~~ **RESOLVED 2026-05-05** - keep immediate preset bid buttons; misclick mitigations are target size, total labels, focus, affordability gating, in-flight disable, and one-send semantics. | - | Closed |
| OQ-SAU-UX-5 | Should "YOU ARE LEADING" include any passive opponent-activity or tension signal if playtests report the leader window as idle? GDD OQ9 flags this as high risk. | Designer | High |
| OQ-SAU-UX-6 | ~~Should toast display duration be globally standardized in the interaction pattern library, or should Shop/Auction toasts use phase-specific timing?~~ **RESOLVED 2026-05-05** - use PTN-FDB-005: 120ms fade in, 2.0s hold, 120ms fade out. | - | Closed |
| OQ-SAU-UX-7 | Should the DRAFT_INITIAL tutorial prompt be retrievable from a future Help/Pause screen immediately, or is first-session persistence enough for M2? | UX Designer + Producer | Low |
