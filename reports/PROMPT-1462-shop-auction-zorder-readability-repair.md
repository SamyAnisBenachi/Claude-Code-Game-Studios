# PROMPT 1462 - Shop/Auction Z-Order Readability Repair

Final relay line: `1462: SHOP-AUCTION-ZORDER-READABILITY-REPAIR: BRANCH_PUSHED`

## Branch Package

- Branch: `work/shop-auction-zorder-readability-repair-1462`
- Base: latest fetched `origin/main` at worktree creation
- Worktree: `.codex-worktrees/PROMPT-1462-shop-auction-zorder`
- Push target: `origin work/shop-auction-zorder-readability-repair-1462`

## Findings

- Latest forensic QA identified auction panel overlap/readability failures, especially leader/status copy competing with featured card content, timer/price controls being visually ambiguous, and shop slots blending into low-contrast panel chrome.
- The cited PROMPT 1450 and PROMPT 1451 refresh reports were not present in `reports/` in the root checkout. The repair preserves their requested behavior by leaving leader perspective logic and shop-slot receive/buffering behavior intact.
- No board rendering, QA snapshot, server, shared, protocol, sprint status, session state, sprint plan, QA plan, or stage files are part of this branch.

## Files Changed

- `client/src/ui/shop_auction/mod.rs`
  - Added fixed auction readability constants for a left featured-card lane and a right info/control lane.
  - Moved auction status, timer bar, bid status, bid buttons, pass button, and free-gold counters out of the featured-card footprint.
  - Kept numeric timer/price surfaces visible by sizing the timer fill inside the fixed-width info lane.
  - Preserved shop slots as visible opaque button wells with separate child affordance copy.

- `tests/integration/shop_auction_ui/responsive_layout_repair_test.rs`
  - Added PROMPT 1462 regression coverage proving auction status/timer/bid status/counters/controls do not overlap the featured card at the minimum viewport.
  - Added semantic assertions that auction controls remain clickable controls while status text remains an info label.

- `tests/integration/shop_auction_ui/shop_panel_test.rs`
  - Added PROMPT 1462 regression coverage proving server-supplied shop slots are visible button wells and their buy-cost affordance copy remains non-clickable info text.

- `tests/integration/shop_auction_ui/auction_featured_card_layout_test.rs`
  - Updated the featured-card layout expectation from centered-panel placement to the fixed left readability lane.

- `tests/integration/shop_auction_ui/auction_free_gold_counters_layout_test.rs`
  - Updated the counter adjacency expectation so counters sit above the bid cluster in the right-lane control column rather than occupying the same row.

- `reports/PROMPT-1462-shop-auction-zorder-readability-repair.md`
  - Records this branch package, changed files, verification, and remaining live visual QA.

## Verification

Changed file list was limited to the six PROMPT 1462 owned files:

- `client/src/ui/shop_auction/mod.rs`
- `tests/integration/shop_auction_ui/responsive_layout_repair_test.rs`
- `tests/integration/shop_auction_ui/shop_panel_test.rs`
- `tests/integration/shop_auction_ui/auction_featured_card_layout_test.rs`
- `tests/integration/shop_auction_ui/auction_free_gold_counters_layout_test.rs`
- `reports/PROMPT-1462-shop-auction-zorder-readability-repair.md`

`git diff --check` passed.

Cargo MSVC policy was applied before Cargo:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Targeted command:

```powershell
cargo test -p client --test shop_auction_ui_responsive_layout_repair_test --test shop_auction_ui_shop_panel_test --test shop_auction_ui_auction_featured_card_layout_test --test shop_auction_ui_auction_free_gold_counters_layout_test
```

Result: passed.

- `shop_auction_ui_auction_featured_card_layout_test`: 7 passed
- `shop_auction_ui_auction_free_gold_counters_layout_test`: 5 passed
- `shop_auction_ui_responsive_layout_repair_test`: 6 passed
- `shop_auction_ui_shop_panel_test`: 11 passed

## Remaining Visual QA

- A live two-client screenshot pass is still needed to confirm the new readability lanes under actual board backdrop, stale placement overlay, settlement overlay, and connection-loss overlay conditions.
- Full workspace tests were intentionally not run per prompt scope.

1462: SHOP-AUCTION-ZORDER-READABILITY-REPAIR: BRANCH_PUSHED
