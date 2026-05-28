# PROMPT 1942 — Krosmaga Auction Tier-Border Asset Binding Refresh after PROMPT 1939

**Date**: 2026-05-28  
**Branch**: `integrate/krosmaga-auction-tier-border-assets-1942`  
**Commit**: `996c1f7d`  
**Base**: `origin/main@be40e0c6` (PROMPT 1939 — stale-binary rebuild guard)

---

## Summary

Reapplied the PROMPT 1853 SLICE-B auction tier-border asset binding cleanly onto
`origin/main` after PROMPT 1939 landed. Source payload transplanted via
`git checkout 8c6792e3 -- <owned files>` (wt-1933-tier-border-refresh). No
cherry-pick of stale branches; no unrelated files touched.

---

## Source Commits / Branches

| Role | Ref | Notes |
|------|-----|-------|
| Original PROMPT 1898 worker | `17e6c665` (origin/integrate/krosmaga-auction-tier-border-assets-1898) | NOT_FF — carries unrelated deletes/reverts; not used directly |
| PROMPT 1933 refresh | `8c6792e3` (origin/wt-1933-tier-border-refresh) | NOT_FF after PROMPT 1939 — would revert Start-TwoClients.ps1 and delete PROMPT 1939 reports; payload transplanted file-by-file instead |
| New base | `be40e0c6` (origin/main) | PROMPT 1939 stale-binary rebuild guard |
| **This commit** | `996c1f7d` | Clean transplant onto current main |

---

## Files Changed

```
M  client/Cargo.toml
M  client/src/asset_wiring.rs
M  client/src/ui/shop_auction/mod.rs
A  tests/unit/asset_wiring/auction_tier_border_asset_test.rs
```

All four files are within the owned scope. No launcher, autoplay, production,
or unrelated report files were touched.

### client/src/asset_wiring.rs
- Added 10 path constants: `AUCTION_TIER_BORDER_{1-4}_ASSET` (4 tier-border overlays)
  and `AUCTION_GEM_{RARE,EPIC,LEGENDARY}_{24,32}_ASSET` (6 gem icons), all under
  `art/ui/shop_auction/`.
- Added `auction_tier_border_asset(tier: u8) -> &'static str` selector; out-of-range
  values clamp to tier 1.
- Extended `PlaceholderAssets` struct, `insert_placeholder_assets`, and
  `placeholder_assets_for_tests` with 10 new `Handle<Image>` fields.

### client/src/ui/shop_auction/mod.rs
- Imported the 4 tier-border constants.
- In `sync_auction_panel_system`: after setting `BorderColor` on
  `auction_featured_card_frame`, inserted an `ImageNode` using the tier-border
  path selected by `auction_border_color_tier(auction_state.current_price)` →
  Formula D.6.

### client/Cargo.toml
- Registered `[[test]] name = "auction_tier_border_asset_test"` target.

### tests/unit/asset_wiring/auction_tier_border_asset_test.rs (new file)
- 8 unit tests covering tier selector correctness (tiers 1–4), out-of-range
  clamp, path-prefix allowlist for all 4 border + 6 gem constants, and
  distinctness across tiers.

---

## Validation

### Scope gate
```
$ git diff --name-status origin/main..HEAD
M       client/Cargo.toml
M       client/src/asset_wiring.rs
M       client/src/ui/shop_auction/mod.rs
A       tests/unit/asset_wiring/auction_tier_border_asset_test.rs
```
Only owned-scope files. No launcher, autoplay, production, or unrelated UI files.

### Whitespace check
```
$ git diff --check origin/main..HEAD
(no output — WHITESPACE_CLEAN)
```

### FF readiness
```
$ git merge-base --is-ancestor origin/main HEAD
(exit 0 — FF_READY)
```
Branch `integrate/krosmaga-auction-tier-border-assets-1942` is strict-FF over
`origin/main@be40e0c6`.

### Focused test
```
$ cargo test --test auction_tier_border_asset_test --manifest-path client/Cargo.toml
   Finished `test` profile [optimized + debuginfo] target(s) in 6m 08s
    Running ../tests/unit/asset_wiring/auction_tier_border_asset_test.rs

running 8 tests
test test_auction_gem_constants_all_under_shop_auction_prefix ... ok
test test_auction_tier_border_asset_tier3_returns_tier3_path ... ok
test test_auction_tier_border_asset_tier1_returns_tier1_path ... ok
test test_auction_tier_border_asset_tier2_returns_tier2_path ... ok
test test_auction_tier_border_asset_out_of_range_clamps_to_tier1 ... ok
test test_auction_tier_border_constants_all_under_shop_auction_prefix ... ok
test test_auction_tier_border_selector_covers_all_four_tiers_distinct ... ok
test test_auction_tier_border_asset_tier4_returns_tier4_path ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Result: 8/8 PASS. Exit code 0. No errors. 101 pre-existing deprecation warnings (not new).**

---

## Root Checkout Status

The root checkout at `D:\_DEV\Work\Claude-Code-Game-Studios` remained on `main`
throughout. No edits were made to the root working tree.

---

1942: KROSMAGA-AUCTION-TIER-BORDER-ASSET-BINDING-REFRESH-AFTER-1939: READY_FOR_MAINLAND_ENQUEUE
