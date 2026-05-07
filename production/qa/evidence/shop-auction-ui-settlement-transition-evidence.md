# Shop/Auction UI Settlement Transition Evidence

Story: SAU-007 Auction Settlement and Shop Transition
Date: 2026-05-07
Scope: client-side Shop/Auction UI presentation behavior only.

## Covered

- `S2CAuctionSettled` enters terminal settlement state for local winner, opponent winner, and no-bid outcomes.
- Local winner settlement clears bid gates, requests card-acquired animation feedback, increments the Shop/Auction hand-size projection, and shows the settlement overlay.
- Terminal settlement clears in-flight bid state, pending accepted/gold gate flags, timer target, and stale toast state.
- Accepted/rejected messages after terminal settlement are ignored and do not update price, leader, timer target, bid buttons, or toast state.
- Same-update accepted-plus-settled processing leaves only the terminal settlement state rendered.
- Auction-to-shop transition uses the 350 ms standard timing, applies buffered shop slots before reveal, and defers the Shop/Auction shop timer until transition completion.
- Reduced motion completes the transition immediately while preserving settlement-before-shop timer ordering.
- `PLACEMENT` phase interrupt cancels settlement immediately and does not delay phase entry.

## Evidence Commands

- `cargo test -p client --test shop_auction_ui_auction_settlement_test`
- Adjacent regressions to run with this story: `shop_auction_ui_auction_feedback_test`, `shop_auction_ui_auction_bid_buttons_test`, `shop_auction_ui_auction_activation_test`, `shop_auction_ui_shop_panel_test`
- Required workspace checks: `cargo fmt -p client -- --check`, `cargo check -p client`, `git diff --check`

## Worker Verification Results

- PASS: `cargo test -p client --test shop_auction_ui_auction_settlement_test` (7 passed).
- PASS: `cargo test -p client --test shop_auction_ui_auction_feedback_test --test shop_auction_ui_auction_bid_buttons_test --test shop_auction_ui_auction_activation_test --test shop_auction_ui_shop_panel_test` (32 passed).
- PASS: `cargo test -p server --test playable_client_real_e2e_loop_test real_lightyear_two_client_draft_shop_auction_placement_resolution_reaches_next_loop` (1 passed).
- PASS: `cargo fmt -p client -- --check`.
- PASS: `cargo check -p client` using reduced-debug single-threaded local MSVC settings after the first run exhausted local target disk space.
- PASS: `git diff --check`.

## Scope Notes

This evidence does not claim public release readiness, broad accessibility completion, playtest validation, full manual QA, full playable-client completion, or full game completion. QA-COND-0005 and QA-COND-0006 are unchanged.
