# SAU-011 Auction Bid Target Size and Focus Evidence

Status: Implementation and automated ECS evidence added; browser/WASM screenshot capture pending.

Story: `production/epics/shop-auction-ui/story-011-auction-bid-target-size-and-focus-evidence.md`
QA condition: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
Source row: A11Y-ST-12

## Scope

Implemented:

- +1, +3, and +5 auction bid controls use fixed 108x44 CSS-px Bevy UI
  target nodes at 100 percent UI scale.
- Bid labels preserve total commitment primary text and visible increment
  secondary text: `(+1)`, `(+3)`, and `(+5)`.
- Enabled bid controls expose test-observable focus order +1 -> +3 -> +5.
- Focused bid controls expose a 2px Prism White-equivalent focus ring state.
- Disabled unaffordable controls and hidden local-leader controls are skipped
  by keyboard focus traversal.
- Enter/Space keyboard activation and pointer `Interaction::Pressed` route
  through the existing `ShopAuctionBidButtonClicked` path, preserving one-send
  semantics, in-flight disable, and `BIDDING...` feedback.

Excluded:

- No Settings/Accessibility work.
- No SAU-007 settlement transition work.
- No auction server/gameplay semantic changes.
- No QA-COND-0005 closure.

## Automated Evidence

Required focused test:

```text
cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test
```

Coverage in `tests/integration/shop_auction_ui/auction_bid_target_focus_test.rs`:

- Target size: `AuctionBidTargetBounds` and `Node` width/height are at least
  44x44 for all three controls.
- Labels: visible text remains split as total commitment plus `(+1)`,
  `(+3)`, and `(+5)`.
- Focus order/ring: Tab traversal reaches +1 then +3; focused control exposes
  `AuctionBidFocusState { focus_ring_visible: true, focus_ring_width_px: 2.0 }`
  and a 2px node border.
- Disabled/hidden focus: unaffordable controls are not focusable; local
  `YOU ARE LEADING` replacement hides all bid controls and leaves no focused
  bid-area item.
- Behavior preservation: Enter and pointer interaction send exactly one
  `C2SPlaceBid { amount }`, switch the clicked control to `BIDDING...`, and
  block additional sends while in flight.

## Browser/WASM Capture Status

Browser/WASM screenshot artifacts were not captured in this worker pass.

Required before story closure:

- 100 percent UI scale target-bound measurements for +1, +3, and +5.
- Focus-bound measurements for +1, +3, and +5.
- Screenshot evidence for enabled affordable buttons with visible `(+1)`,
  `(+3)`, and `(+5)` labels.
- Screenshot evidence for unaffordable disabled bid buttons skipped by
  keyboard focus.
- Screenshot evidence for clicked-button `BIDDING...` with other bid controls
  non-interactive.
- Screenshot evidence for local `YOU ARE LEADING` replacement with no
  focusable bid-area control.

Implementation note: the visible button target itself was enlarged to 44px high;
no equivalent invisible hit box is used.

## QA-COND-0005 Impact

SAU-011 implements the A11Y-ST-12 code path for auction bid target size, focus
visibility, immediate preset commitment labels, affordability gating, in-flight
disable, one-send semantics, and `BIDDING...` feedback.

This evidence file does not close A11Y-ST-12 by itself until the required
browser/WASM screenshot artifacts are attached. QA-COND-0005 remains Open until
all remaining Standard-tier rows are implemented and evidenced, reclassified,
or accepted as risk.
