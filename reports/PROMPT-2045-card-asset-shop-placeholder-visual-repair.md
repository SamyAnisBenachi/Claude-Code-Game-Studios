# PROMPT-2045 — Card-Asset / Shop Placeholder Visual Repair

**Date**: 2026-05-28
**Source-of-truth at start**: `origin/main@135ca0b0`
**Branch**: `work/PROMPT-2045`
**Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2045`
**Related**: `reports/PROMPT-2038-card-asset-shop-placeholder-binding-repair.md`,
`reports/PROMPT-2026-visible-screen-screenshot-visual-bug-audit.md`,
`production/qa/bugs/current-unplayable-bug-register-2026-05-28.md`

## Outcome

**STATUS**: SHIPPED.

PROMPT 2038's `BID_BUTTON_HOVER_ASSET` fix was authored on a side branch
(commit `77c406f7`) and never landed on `origin/main`. Hovering the bid button
on the first-round shop / auction surface therefore still replaced the live
chrome with the universal `?` placeholder (`art/characters/ui_unit_placeholder_default_board.png`).

This PROMPT cherry-picks `77c406f7` cleanly onto current `origin/main@135ca0b0`
so the fix actually ships, with no other source edits.

## What landed

Cherry-picked commit (now `744f4772` on `work/PROMPT-2045`) carrying:

1. `client/src/asset_wiring.rs`
   - `BID_BUTTON_HOVER_ASSET` re-pointed from the universal `?` placeholder
     PNG to `BID_BUTTON_NORMAL_ASSET` (active state) until a real hover PNG is
     authored. Single-source change at the asset-wiring chokepoint.

2. `tests/integration/presentation/asset_wiring_foundation_test.rs`
   - `test_bid_button_selector_covers_all_variants` rewritten as
     `test_bid_button_selector_never_routes_to_placeholder`. The prior
     "distinct path per variant" guard tacitly blessed the placeholder
     re-point. The new assertion expresses the intent directly: no
     `BidButtonChromeState` variant may resolve to
     `PLACEHOLDER_FALLBACK_ASSET`.

3. `reports/PROMPT-2038-card-asset-shop-placeholder-binding-repair.md`
   - The original PROMPT 2038 report is brought across so the bug register's
     cross-reference resolves on main.

## Audit findings on remaining placeholder routes (verified)

Re-walked the asset-wiring chokepoint and the shop/auction binding chain
against current main:

| Surface | Binding path | Status |
|---|---|---|
| Shop slot card art | `apply_card_display_art` -> `art/cards/display/card_{art_id}_art_display.png` | OK — all 16 cards in `assets/data/cards.json` have a matching PNG on disk in `assets/art/cards/display/`. Verified by listing. |
| Draft-initial featured card | same chokepoint | OK |
| Auction featured card | same chokepoint | OK |
| Shop well chrome | `SHOP_SLOT_WELL_IDLE_ASSET` | OK — real file on disk; chrome preservation strategy in `apply_card_display_art` keeps the well drawn when `art_id` is empty/unresolved (SOURCE-1077-01). |
| Bid button Normal | `art/ui/auction/ui_bid_button_active.png` | OK |
| Bid button Hover | now `BID_BUTTON_NORMAL_ASSET` | **Fixed by this PROMPT** (was routed to `?` placeholder PNG). |
| Bid button Disabled | `art/ui/auction/ui_bid_button_disabled.png` | OK |
| Auction tier borders 1-4 | `art/ui/shop_auction/ui_auction_border_tier{1..4}_hud.png` | OK |
| Auction rarity gems 24/32 | `art/ui/shop_auction/ui_gem_*_default_*.png` | OK |

`client/src/ui/shop_auction/mod.rs` does not bind the universal placeholder
PNG directly anywhere — the only `placeholder`/`fallback` mentions are in
comments/doc strings or unrelated color fallbacks (lines 145, 3748, 4621,
4728, 5430, 6183, 6325, 6613). `client/src/ui/card_inspect.rs` contains no
direct placeholder/`art_id` references. No further shop-surface edits needed.

Remaining `NO ANALOGUE on disk` constants still pointing at the universal
`?` placeholder PNG (out of scope for this PROMPT, owned by hand / HUD
pipelines per PROMPT-2045 forbidden scope):

- `STAT_BADGE_AR_ASSET` (hand card armor badge)
- `HUD_PHASE_TIMER_BAR_ASSET` (HUD phase timer)
- `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` (HUD objective dot)

These should be tracked under a hand/HUD repair PROMPT — flagged here for
the bug register, not patched.

## Validation performed

- **Path allowlist review**: only `client/src/asset_wiring.rs`,
  `tests/integration/presentation/asset_wiring_foundation_test.rs`, and
  `reports/PROMPT-204{5,8}-*.md` modified. All within the PROMPT-2045 owned
  scope.
- **`git diff --check origin/main..HEAD`**: clean (no whitespace errors).
- **Focused test**:
  `cargo test --package client --test asset_wiring_foundation_test test_bid_button_selector_never_routes_to_placeholder`
  → `test result: ok. 1 passed; 0 failed`.
- **Visual polish closure**: NOT claimed without screenshot evidence.
  The fix is a constant re-point; behavior-equivalence vs Normal hover state
  is demonstrated by the test. Visual confirmation must come from a future
  autoplay/manual capture run.

## Out of scope (flagged for follow-up)

- `STAT_BADGE_AR_ASSET` / `HUD_PHASE_TIMER_BAR_ASSET` /
  `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` `?` placeholder routing (hand/HUD).
- BUG-01 (client never leaves Lobby) and BUG-08/09 (board/auction layout)
  from PROMPT 2026 — unrelated to this asset-binding chokepoint.
- `ui_bid_button_hover.png` final asset authoring — once on disk, re-point
  `BID_BUTTON_HOVER_ASSET` back to its dedicated path with no test change.

## Final line

2045: CARD-ASSET-SHOP-PLACEHOLDER-VISUAL-REPAIR: SHIPPED
