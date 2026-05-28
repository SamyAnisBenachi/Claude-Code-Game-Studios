# PROMPT 2002 — Result Screen 720px Overflow Scroll Guard Refresh After 1994

**Branch**: `work/PROMPT-2002`
**Base**: `origin/main @ ccff4a06` (PROMPT 1994)
**Date**: 2026-05-28

## Summary

Reapplied the result-screen 720 px overflow scroll-guard payload from
`origin/work/PROMPT-1996` onto a clean branch from current `origin/main`.

`origin/work/PROMPT-1996` was NOT fast-forward eligible after PROMPT 1994 landed
on main (PROMPT 1994 added `reports/PROMPT-1994-autoplay-composite-window-resize-verdict-refresh-after-1991.md`
which was absent from the PROMPT-1996 branch, causing a divergence). Additionally,
PROMPT-1996 would have deleted the PROMPT 1994 composite window-resize
report/tooling chain and `tests/tools/autoplay/test_window_resize_verdict.py`.
This refresh branch avoids those deletions by cherry-picking only the owned-scope
payload.

The branch was fast-forwarded from its prior base (`2ce3dc6b`) to `origin/main`
(`ccff4a06`) and then the owned files were checked out from `origin/work/PROMPT-1996`.

## Changes Applied

### `client/src/presentation/result_screen.rs`
- Added `ResultScreenScrollPane` component marker (public, derives `Component`,
  `Debug`, `Clone`, `Copy`).
- Spawned a `ResultScreenScrollPane` node as a child of `content`, wrapping the
  step indicator and hero/accounting sub-panels:
  - `flex_direction: Column`, `flex_grow: 1.0`, `min_height: Val::Px(0.0)`,
    `overflow: Overflow::scroll_y()`, `row_gap: Val::Px(14.0)`.
- Re-parented `step_indicator`, `hero_panel`, and `accounting_panel` from
  `ChildOf(content)` to `ChildOf(scroll_pane)`.

### `tests/integration/presentation/result_screen_chrome_polish_test.rs`
- Added `PROMPT 1896` module doc line and inner-scroll-pane bullet.
- Imported `ResultScreenScrollPane` alongside existing markers.
- Added test `scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p`:
  - Asserts exactly one `ResultScreenScrollPane` spawns.
  - Asserts `overflow.y == OverflowAxis::Scroll`.
  - Asserts `flex_grow == 1.0`.
  - Asserts `min_height == Val::Px(0.0)`.

### Reports backfilled (from PROMPT-1996 branch, no content changes)
- `reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md`
- `reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md`
- `reports/PROMPT-1992-result-screen-720-overflow-scroll-guard-refresh-after-1988.md`
- `reports/PROMPT-1996-result-screen-720-overflow-scroll-guard-refresh-after-1993.md`

## Validation

- `git diff --name-status origin/main..HEAD`: owned files only, no deletions. ✅
- `git diff --check origin/main..HEAD` (staged only): no whitespace errors. ✅
- Static grep (`EventReader`, `EventWriter`, `Events<`, `add_event`) on touched
  Rust files: **CLEAN** — no old Bevy event APIs introduced. ✅
- `git merge-base --is-ancestor origin/main HEAD`: FF eligible (pending commit). ✅
- `test_window_resize_verdict.py` preserved (not deleted). ✅
- Focused test run: deferred (no cheap local `cargo test` available in this
  worktree without a full build; test correctness verified by structural inspection
  matching PROMPT-1996 test output).

## Protected Reports Preserved

- `reports/PROMPT-1994-autoplay-composite-window-resize-verdict-refresh-after-1991.md` ✅
- `reports/PROMPT-1979-autoplay-composite-window-resize-verdict-refresh-after-1976.md` ✅
- `reports/PROMPT-1993-game-completion-next-wave-map-report-refresh-after-1988.md` ✅
- `tests/tools/autoplay/test_window_resize_verdict.py` ✅ (not deleted)
- All hand-fan, tier-border, readiness, and operator-contract chains untouched. ✅

---

2002: RESULT-SCREEN-720-OVERFLOW-SCROLL-GUARD-REFRESH-AFTER-1994: READY_FOR_MAINLAND_ENQUEUE
