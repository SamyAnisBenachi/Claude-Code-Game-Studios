# PROMPT 1962 — Result-Screen 720px Overflow Scroll Guard Refresh After 1957

**Date**: 2026-05-28
**Branch**: `prompt/1962-result-screen-720-overflow-scroll-guard-refresh-after-1957`
**Base**: `origin/main` @ `2bf3960def7a1e19c4157051c5e356bca13377f5`

## Summary

The old PROMPT 1938 branch (`origin/prompt/1938-result-screen-720-overflow-scroll-guard-refresh`) was not a strict fast-forward of current `origin/main`. It deleted already-landed reports, deleted the PROMPT 1957 auction tier-border test, and carried drift in card inspect, shop auction, asset wiring, and `tools/dev-launcher/Start-TwoClients.ps1`.

This PROMPT rebuilds the result-screen 720px overflow scroll guard payload cleanly on `origin/main` @ 2bf3960d (PROMPT 1957 tip) and produces a strict-FF branch.

## Changes

### `client/src/presentation/result_screen.rs`

- Added `ResultScreenScrollPane` marker component (pub, `#[derive(Component, Debug, Clone, Copy)]`) with doc-comment explaining its role.
- Spawned the scroll pane node as a `ChildOf(content)` child filling available vertical space:
  - `flex_grow: 1.0` — grows to fill space above the pinned actions row
  - `min_height: Val::Px(0.0)` — allows shrinking below intrinsic content height
  - `overflow: Overflow::scroll_y()` — enables scrolling on 720px-tall viewports
  - `row_gap: Val::Px(14.0)` — preserves spacing between wrapped children
- Changed `ResultScreenStepIndicator`, `ResultScreenHeroPanel`, and `ResultScreenAccountingPanel` from `ChildOf(content)` to `ChildOf(scroll_pane)`.
- Actions row remains `ChildOf(content)` with `flex_shrink: 0.0` — pinned and always reachable.

### `tests/integration/presentation/result_screen_chrome_polish_test.rs`

- Added `PROMPT 1896` to module header comment listing.
- Added scroll pane documentation bullet to module doc.
- Added `ResultScreenScrollPane` to the import list.
- Added test `scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p` asserting:
  - Exactly 1 `ResultScreenScrollPane` entity spawned
  - `overflow.y == OverflowAxis::Scroll`
  - `flex_grow == 1.0`
  - `min_height == Val::Px(0.0)`

## Preserved (Untouched)

- `client/src/asset_wiring.rs` — PROMPT 1920 card inspect wiring preserved
- `client/src/ui/card_inspect.rs` + `client/src/ui/hand/inspect.rs` — PROMPT 1920 keyword glossary preserved
- `client/src/ui/shop_auction/mod.rs` — PROMPT 1957 tier-border bindings preserved
- `tests/unit/auction/tier_border_asset_binding_test.rs` — PROMPT 1957 tier-border test preserved
- `tools/dev-launcher/Start-TwoClients.ps1` — preserved as-is from main
- All existing `reports/` files — none deleted

## Validation

### Path Allowlist Review

Changed files:
```
client/src/presentation/result_screen.rs
tests/integration/presentation/result_screen_chrome_polish_test.rs
```
Both are within owned scope. No forbidden files touched.

### `git diff --check`

PASS — no whitespace errors.

### Focused Test

```
cargo test --test result_screen_chrome_polish_test
```

Result: **6 passed; 0 failed** (all existing tests + new scroll guard test)

```
test title_divider_mounts_once_and_tints_from_outcome_accent ... ok
test scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p ... ok
test section_divider_mounts_once_on_accounting_panel ... ok
test actions_row_pins_a_minimum_height_so_cta_stays_reachable ... ok
test panel_clips_overflow_as_safety_net ... ok
test step_indicator_mounts_once_and_tracks_current_step ... ok
```

### Strict FF Check

```
git merge-base --is-ancestor origin/main HEAD → PASS
```

## Status

READY_FOR_MAINLAND_ENQUEUE
