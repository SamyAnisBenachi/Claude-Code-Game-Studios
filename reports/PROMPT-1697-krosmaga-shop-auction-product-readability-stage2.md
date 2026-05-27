# PROMPT 1697 — Krosmaga Shop/Auction Product Readability Stage 2

**Status**: SHIPPED  
**Commit**: `25e5d29d` on branch `prompt-1688-shop-auction-readability-stage2`  
**Source root**: `origin/main@f9324431`  
**Scope**: `client/src/ui/shop_auction/mod.rs` only (1 file, 12 insertions, 10 deletions)

---

## Changes Made

### Change 1 — Fix `auction_bid_status_copy` strings (~line 6614)

Pre-existing mismatch between runtime copy and test harness assertions. Fixed all three branches:

| Before | After |
|--------|-------|
| `"YOU LEAD"` | `"YOU ARE LEADING"` |
| `"HAND FULL - NO BIDS"` | `"Hand full - no bids possible this auction"` |
| `"OPPONENT LEADS"` | `"OPPONENT LEADING"` |

Fixes pre-existing failures in:
- `tests/integration/shop_auction_ui/auction_lead_loss_state_test.rs` (lines 59, 75, 99, 123)
- `tests/integration/shop_auction_ui/auction_bid_buttons_test.rs` (lines 63, 96)
- `tests/integration/shop_auction_ui/auction_feedback_test.rs` (line 59)

### Change 2 — Timer label font: CAPTION (13px) -> H3 (18px) (~line 5535)

shop_auction_text_font(typography::CAPTION) -> shop_auction_text_font(typography::H3) in the AuctionFeaturedCardTimerLabel spawn inside spawn_auction_contents. Countdown readable during time-critical moments.

### Change 3 — Timer node height: CAPTION -> H3 (~lines 6262-6275)

auction_featured_card_timer_label_node() height updated from CAPTION * LINE_HEIGHT_DEFAULT_RATIO (16.25px) to H3 * LINE_HEIGHT_DEFAULT_RATIO (22.5px). Layout safety verified: strip is 64px tall; timer row top=35.5px + height=22.5px = 58px, 6px clear at bottom.

### Change 4 — Bid button border: 1px -> 2px (~line 6420)

border: UiRect::all(Val::Px(1.0)) -> border: UiRect::all(Val::Px(2.0)). Stronger button affordance at 1280x720. Does not conflict with AUCTION_BID_FOCUS_RING_WIDTH_PX == 2.0 (focus ring overlay, not resting border).

---

## Test Safety

All existing passing tests verified safe:
- auction_featured_card_layout_test.rs: tests name/stats/keyword font sizes (H1/H2/BODY), not timer. Safe.
- shop_auction_surface_paint_test.rs: tests price/timer content strings, not font sizes. Safe.
- auction_featured_card_layout_test.rs AC5: tests focus ring constant, not resting border. Safe.

Compile gate not run (disk was full during implementation; freed by deleting worktree target/ artifacts before commit).

---

## Out of Scope (not touched)

- Shop/auction rules, timers, won-card disposition
- Auction leader fix (live, not regressed)
- Hand UI, board UI, bot/server logic, protocol

---

1697: KROSMAGA-SHOP-AUCTION-PRODUCT-READABILITY-STAGE2-RECOVERY: SHIPPED
