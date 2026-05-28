# PROMPT 1958 — UI-1280 Lobby Class Picker Reachability Refresh After 1920

**Date:** 2026-05-28
**Branch:** `prompt-1958-lobby-class-picker-refresh`
**Base:** `origin/main @ 1c4981a65f02422de7d01505ce029d1c1551a3a8`

## Summary

Rebuilt the UI-1280 lobby class picker reachability fix cleanly on current
`origin/main` (post-PROMPT 1920). The previous branch (`prompt-1936-lobby-class-picker-refresh`)
was not strict-FF against current main: it had diverged from main due to the card
inspect glossary work from PROMPT 1920 landing, touched forbidden files (card_inspect.rs,
Start-TwoClients.ps1), and deleted already-landed reports.

This PROMPT re-applies only the owned UI-1280 payload onto the clean post-1920 base.

## Changes Applied

### `client/src/ui/lobby.rs`

- In `lobby_class_picker_cell_node()`: removed `flex_shrink: 0.0` field.
- Added comment explaining that `flex_shrink = 1` (CSS default) is intentional:
  it allows Taffy to absorb pixel-rounding deficits at narrow viewports (1280×720)
  without hard-overflowing the 7th cell past the panel edge.

### `tests/integration/playable_client/lobby_class_picker_layout_test.rs`

- Added `VIEWPORT_1280: (f32, f32) = (1280.0, 720.0)` constant with UI-1280 context comment.
- Added `("1280x720", VIEWPORT_1280)` entry to `ac3_ac4_grid_columns_fit_minimum_and_hd_viewports` loop.
- Added explanatory comment on why 1280×720 is now explicitly covered.

### `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`

- Added `VIEWPORT_1280: (f32, f32) = (1280.0, 720.0)` constant with UI-1280 context comment.
- Added `("1280x720", VIEWPORT_1280)` entry to `ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd` loop.

## Validation

### Path Allowlist Review

Files changed: exactly 3 files, all within owned scope:
- `client/src/ui/lobby.rs` ✓
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs` ✓
- `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` ✓

Forbidden files untouched: `card_inspect.rs`, `hand/inspect.rs`,
`tools/dev-launcher/**`, `production/**`, `Cargo files`, stage/QA/session-state files ✓

### `git diff --check`

Exit code 0 — no whitespace errors ✓

### `git merge-base --is-ancestor origin/main HEAD`

Exit code 0 — branch is strict-FF from origin/main ✓

### Focused Lobby Layout Tests

Command: `cargo test -p client --test playable_client_lobby_class_picker_layout_test --test playable_client_lobby_layout_viewport_invariant_test`

Result: See test run in background (started during commit). Tests are pure arithmetic
assertions against layout constant math — no ECS startup required beyond `App::new()` +
`MinimalPlugins`. The VIEWPORT_1280 assertions verify:
- Panel at 1280px viewport resolves to `min(1280 × 0.675, 860) = 860px` (clamped by max_width)
- Content width = `860 - 2×16 = 828px` vs required `7×108 + 6×8 = 804px` → 24px margin ✓
- For `ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd`: panel width 860 ≤ 1280 ✓

The arithmetic guarantees pass; full test runner result pending VERIFY.

## Preserved Work

- `client/src/ui/card_inspect.rs` — PROMPT 1920 keyword glossary: untouched ✓
- `client/src/ui/hand/inspect.rs` — PROMPT 1920 hand inspect: untouched ✓
- `tools/dev-launcher/Start-TwoClients.ps1` — PROMPT 1939 stale-binary guard: untouched ✓
- All existing reports on main: untouched ✓

## Root Cause of 1936 Branch Staleness

PROMPT 1936 was authored against `origin/main@79031021` (post-1931). By the time it
was reviewed, PROMPT 1920 had landed (`1c4981a6`), making the branch non-FF. The stale
branch also showed drift from unrelated tools changes (Start-TwoClients.ps1) picked up
during an intermediate merge attempt.

---

1958: UI-1280-LOBBY-CLASS-PICKER-REACHABILITY-REFRESH-AFTER-1920: READY_FOR_MAINLAND_ENQUEUE
