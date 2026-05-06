# SAU-011 Auction Bid Target Size and Focus Evidence

Status: Implementation, automated ECS evidence, and browser/WASM screenshot
evidence captured.

Story: `production/epics/shop-auction-ui/story-011-auction-bid-target-size-and-focus-evidence.md`
QA condition: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
Source row: A11Y-ST-12

## Scope

Implemented:

- +1, +3, and +5 auction bid controls use fixed 108x44 CSS-px Bevy UI
  target nodes at 100 percent UI scale.
- Bid labels preserve total commitment primary text and visible increment
  secondary text: `(+1)`, `(+3)`, and `(+5)`.
- Enabled bid controls expose focus order +1 -> +3 -> +5.
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

## Browser/WASM Capture Evidence

Capture command:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/capture.ps1 -Url http://127.0.0.1:8082/shop-auction-bid-target-focus-harness.html -ReadyTimeoutSeconds 240
```

Capture summary:

- Summary JSON: `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/capture-summary.json`
- Capture tool: PowerShell Chrome DevTools Protocol
- Browser: Chrome `147.0.7727.139`
- Captured at: `2026-05-06T11:10:58.8758988Z`
- Viewport: 1366x768, device scale factor 1, UI scale 1.0

Summary verdict:

- Target bounds 44px minimum: PASS.
- Focus bounds 44px minimum: PASS.
- Affordable labels: PASS.
- Unaffordable keyboard skip: PASS.
- `BIDDING...` feedback: PASS.
- `YOU ARE LEADING` replacement with no focusable bid control: PASS.

Measured 100 percent UI scale target bounds:

| Scenario | +1 | +3 | +5 |
| --- | --- | --- | --- |
| Affordable | 108x44 | 108x44 | 108x44 |
| Focus +1 | 108x44, 2px ring | 108x44 | 108x44 |
| Focus +3 | 108x44 | 108x44, 2px ring | 108x44 |
| Focus +5 | 108x44 | 108x44 | 108x44, 2px ring |
| Unaffordable | 108x44, focused | 108x44, skipped | 108x44, skipped |
| Bidding | 108x44, disabled | 108x44, `BIDDING...` | 108x44, disabled |
| Leading | hidden, not focusable | hidden, not focusable | hidden, not focusable |

Artifact set:

- Affordable enabled buttons with `(+1)`, `(+3)`, `(+5)`:
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-affordable-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-affordable-1366x768-report.json`
- Focus +1:
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-focus-plus-1-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-focus-plus-1-1366x768-report.json`
- Focus +3:
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-focus-plus-3-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-focus-plus-3-1366x768-report.json`
- Focus +5:
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-focus-plus-5-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-focus-plus-5-1366x768-report.json`
- Unaffordable disabled bid buttons skipped by keyboard focus:
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-unaffordable-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-unaffordable-1366x768-report.json`
- Clicked-button `BIDDING...` with other bid controls non-interactive:
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-bidding-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-bidding-1366x768-report.json`
- Local `YOU ARE LEADING` replacement with no focusable bid-area control:
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-leading-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/sau-011-leading-1366x768-report.json`

Implementation note: the visible button target itself was enlarged to 44px
high; no equivalent invisible hit box is used.

## QA-COND-0005 Impact

SAU-011 implements and evidences A11Y-ST-12 for auction bid target size, focus
visibility, immediate preset commitment labels, affordability gating, in-flight
disable, one-send semantics, and `BIDDING...` feedback.

This story does not close QA-COND-0005 by itself. QA-COND-0005 remains Open
until all remaining Standard-tier rows are implemented and evidenced,
reclassified, or accepted as risk.
