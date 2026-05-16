# Sprint 14 Auction Lead/Loss State Evidence

Story: `S12-UX-AUCTION-LEAD-LOSS-STATE-001`

Prompt: PROMPT 971

## No-Claim Banner

Story 018 implements auction leading / losing / neutral state visual on the
featured card only. It does not claim Standard-tier colorblind conformance and
does not advance `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-002-a`,
`PAW-TD-003-a`, `S8-QA-001-W1`, the PROMPT 761 Polish-to-Release gate-check,
or any release-readiness claim. All conditions remain accept-risk / open per
their existing dispositions.

## Producer Decision Applied

PROMPT 967 Option A was applied: the existing Story 016
`AuctionFeaturedCardFrame` is reused and recolored with static token colors.
No pulse, badge, chevron, new art, server change, protocol change, or new
Lightyear drain was added.

Token mapping:

- Neutral / pre-bid: existing `ACCENT #F2C94C`.
- Leading: `SEMANTIC_SUCCESS #27AE60`.
- Losing: `SEMANTIC_ERROR #EB5757`.

## Automated Evidence

New integration test:

- `tests/integration/shop_auction_ui/auction_lead_loss_state_test.rs`

Coverage:

- Neutral frame state carries the strict `AuctionFeaturedCardLeadLossState::Neutral`
  marker and the accent border before any bid.
- Local leader state carries `Leading`, success border color, and preserves the
  existing `YOU ARE LEADING` text fallback.
- Opponent leader state carries `Losing`, error border color, and exposes an
  `OPPONENT LEADING` text fallback.
- State transitions are strict single-enum transitions: neutral to leading to
  losing, with no multiple marker components.

## Browser/WASM Capture Limitation

Browser/WASM screenshot capture at 1920 x 1080 and 1366 x 768 was not performed
in this worker environment. The worker evidence is ECS integration coverage
against the real `ShopAuctionUiPlugin`, stable marker components, frame border
colors, and text fallback. Runtime PNG capture remains a manual/browser follow-up
for `/story-done` or later QA if required.

## Adjacent Contracts Preserved

The implementation keeps the existing auction panel, featured card, free-gold
counters, bid buttons, timer, and settlement systems. Story 004 / 005 / 006 /
011 / 013 regression coverage is expected from the targeted test commands in the
worker report.
