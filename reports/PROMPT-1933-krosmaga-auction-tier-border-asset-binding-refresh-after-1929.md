# PROMPT 1933 — KROSMAGA-AUCTION-TIER-BORDER-ASSET-BINDING-REFRESH-AFTER-1929

**Date:** 2026-05-28
**Branch:** `wt-1933-tier-border-refresh`
**Commit:** `8c6792e3`
**Base:** `origin/main @ 79031021` (post-PROMPT 1931)

---

## Summary

Recreated the PROMPT 1853 SLICE-B auction tier-border asset binding payload
cleanly on current `origin/main`. The stale branches
(`origin/wt-1853-tier-border-slice-b`, `origin/integrate/krosmaga-auction-tier-border-assets-1898`)
were non-FF relative to main; their payload was recovered via `git diff` and
manually applied, preserving all autoplay/report/launcher changes on main.

---

## Changes Applied

### `client/src/asset_wiring.rs`
- Added 4 `AUCTION_TIER_BORDER_{1-4}_ASSET` path constants under `art/ui/shop_auction/`
- Added 6 `AUCTION_GEM_{RARE,EPIC,LEGENDARY}_{24,32}_ASSET` path constants under `art/ui/shop_auction/`
- Added `pub fn auction_tier_border_asset(tier: u8) -> &'static str` selector (1-indexed, clamps out-of-range to tier 1)
- Added 10 new fields to `PlaceholderAssets` struct (4 tier-border + 6 gem)
- Added 10 corresponding `asset_server.load(...)` calls in `insert_placeholder_assets`
- Added 10 `Handle::default()` entries in `placeholder_assets_for_tests`

### `client/src/ui/shop_auction/mod.rs`
- Extended `use crate::asset_wiring` import to include `AUCTION_TIER_BORDER_{1-4}_ASSET`
- In `sync_auction_panel_system`: after the `BorderColor` update block, inserted tier-border
  `ImageNode` binding on `entities.auction_featured_card_frame` using `auction_border_color_tier`
  formula (Formula D.6) to select path from `AuctionBorderColorTier` variant

### `tests/unit/asset_wiring/auction_tier_border_asset_test.rs` (new file)
- 8 unit tests: tier 1–4 path correctness, out-of-range clamp (0/5/255 → tier 1),
  prefix allowlist for all 4 border constants, prefix allowlist for all 6 gem constants,
  distinctness assertion across all 4 tier paths

### `client/Cargo.toml`
- Registered `auction_tier_border_asset_test` as a `[[test]]` target

---

## Validation

### Scope gate
```
git diff --name-status origin/main..HEAD
```
Files changed:
- `M client/Cargo.toml`
- `M client/src/asset_wiring.rs`
- `M client/src/ui/shop_auction/mod.rs`
- `A tests/unit/asset_wiring/auction_tier_border_asset_test.rs`

All within owned scope. No deletions of existing reports or tests.

### Whitespace gate
```
git diff --check origin/main..HEAD → PASS (no trailing whitespace)
```

### Focused test result
```
cargo test --test auction_tier_border_asset_test

running 8 tests
test test_auction_gem_constants_all_under_shop_auction_prefix ... ok
test test_auction_tier_border_asset_out_of_range_clamps_to_tier1 ... ok
test test_auction_tier_border_asset_tier1_returns_tier1_path ... ok
test test_auction_tier_border_asset_tier2_returns_tier2_path ... ok
test test_auction_tier_border_asset_tier3_returns_tier3_path ... ok
test test_auction_tier_border_asset_tier4_returns_tier4_path ... ok
test test_auction_tier_border_constants_all_under_shop_auction_prefix ... ok
test test_auction_tier_border_selector_covers_all_four_tiers_distinct ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Broad Cargo suite deferred per task rules (no-broad-suite-by-default). All
pre-existing warnings (deprecated `HudEntity`/`HandUiEntity`/`ShopAuctionUiEntity`
markers) are pre-existing on main and not introduced by this prompt.

---

## Commit

```
8c6792e3 feat(ui/auction): PROMPT 1933 — refresh 1853 tier-border asset binding onto post-1931 main
```

Branch pushed: `origin/wt-1933-tier-border-refresh`

---

1933: KROSMAGA-AUCTION-TIER-BORDER-ASSET-BINDING-REFRESH-AFTER-1929: SHIPPED
