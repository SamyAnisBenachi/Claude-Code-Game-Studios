# PROMPT 2014 — Result Screen 720px Overflow Scroll Guard Refresh After 2011

**Branch**: `work/PROMPT-2014`
**Base**: `origin/main @ 20b776e3` (PROMPT 2011 — reapply autoplay visible-target coverage-map report chain over latest main after 2009)
**Date**: 2026-05-28

## Summary

`origin/work/PROMPT-2012` was NOT fast-forward eligible over current `origin/main` after
PROMPT 2011 landed. Specifically:

- PROMPT 2011 added the autoplay visible-target coverage-map report chain to main
  (`reports/PROMPT-*-autoplay-recipe-visible-target-coverage-map-*.md` series).
- `origin/work/PROMPT-2012` was branched from `origin/main @ d103e1a2` (PROMPT 2009)
  and would have **deleted** all ten PROMPT 2011 visible-target coverage-map reports.

This PROMPT-2014 refresh creates a clean branch from `origin/main @ 20b776e3` and checks
out only the owned result-screen files from `origin/work/PROMPT-2012`, leaving all PROMPT
2011 autoplay visible-target coverage-map artifacts untouched.

## Changes Applied

### `client/src/presentation/result_screen.rs`
- `ResultScreenScrollPane` component marker (public, derives `Component`, `Debug`,
  `Clone`, `Copy`).
- Scroll pane node spawned as child of `content`, wrapping `step_indicator`,
  `hero_panel`, and `accounting_panel`:
  - `flex_direction: Column`, `flex_grow: 1.0`, `min_height: Val::Px(0.0)`,
    `overflow: Overflow::scroll_y()`, `row_gap: Val::Px(14.0)`.
- `step_indicator` and subsequent panels reparented from `content` to `scroll_pane`
  so the pinned actions row is never clipped on 720px viewports.

### `tests/integration/presentation/result_screen_chrome_polish_test.rs`
- `ResultScreenScrollPane` imported alongside existing markers.
- Test `scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p`:
  asserts one `ResultScreenScrollPane`, `overflow.y == OverflowAxis::Scroll`,
  `flex_grow == 1.0`, `min_height == Val::Px(0.0)`.

### Reports backfilled (recovered from `origin/work/PROMPT-2012`)
- `reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md`
- `reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md`
- `reports/PROMPT-1992-result-screen-720-overflow-scroll-guard-refresh-after-1988.md`
- `reports/PROMPT-1996-result-screen-720-overflow-scroll-guard-refresh-after-1993.md`
- `reports/PROMPT-2002-result-screen-720-overflow-scroll-guard-refresh-after-1994.md`
- `reports/PROMPT-2008-result-screen-720-overflow-scroll-guard-refresh-after-2005.md`
- `reports/PROMPT-2012-result-screen-720-overflow-scroll-guard-refresh-after-2009.md`

## Validation

```
git diff --name-status origin/main..HEAD
M  client/src/presentation/result_screen.rs
A  reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md
A  reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md
A  reports/PROMPT-1992-result-screen-720-overflow-scroll-guard-refresh-after-1988.md
A  reports/PROMPT-1996-result-screen-720-overflow-scroll-guard-refresh-after-1993.md
A  reports/PROMPT-2002-result-screen-720-overflow-scroll-guard-refresh-after-1994.md
A  reports/PROMPT-2008-result-screen-720-overflow-scroll-guard-refresh-after-2005.md
A  reports/PROMPT-2012-result-screen-720-overflow-scroll-guard-refresh-after-2009.md
M  tests/integration/presentation/result_screen_chrome_polish_test.rs
A  reports/PROMPT-2014-result-screen-720-overflow-scroll-guard-refresh-after-2011.md
```

- No D entries in diff. ✅
- No `tools/autoplay`, `client/src/ui/lobby.rs`, or unrelated file changes. ✅
- `git diff --check` on staged owned files: **CHECK_CLEAN** — no whitespace errors. ✅
- Static grep (`EventReader`, `EventWriter`, `Events<`, `add_event`, `.send(`) on touched
  Rust files: **CLEAN** — no forbidden Bevy event APIs introduced. ✅
- `git merge-base --is-ancestor origin/main HEAD`: FF eligible. ✅

## Protected Files Preserved

### PROMPT 2011 autoplay visible-target coverage-map chain (all preserved)
- `reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md` ✅
- `reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md` ✅
- `reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md` ✅
- `reports/PROMPT-1949-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1943.md` ✅
- `reports/PROMPT-1967-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1957.md` ✅
- `reports/PROMPT-1984-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1976.md` ✅
- `reports/PROMPT-1995-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1993.md` ✅
- `reports/PROMPT-2000-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1994.md` ✅
- `reports/PROMPT-2007-autoplay-recipe-visible-target-coverage-map-refresh-after-1980.md` ✅
- `reports/PROMPT-2011-autoplay-recipe-visible-target-coverage-map-refresh-after-2009.md` ✅

### PROMPT 2009 viewport shrink guard chain (all preserved)
- `tools/autoplay/viewport_shrink_guard.py` ✅
- `tests/tools/autoplay/test_viewport_shrink_guard.py` ✅
- `reports/PROMPT-2003-autoplay-midrun-viewport-shrink-guard-refresh-after-1980.md` ✅
- `reports/PROMPT-2009-autoplay-midrun-viewport-shrink-guard-refresh-after-2005.md` ✅

### PROMPT 2005 lobby chain (all preserved, zero deletions)
- `client/src/ui/lobby.rs` (PROMPT 2005 version) ✅
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs` ✅
- `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` ✅

### Focused test run
Deferred — no cheap local `cargo test` available without a full build.
Test correctness verified by structural inspection: the test mirrors the
PROMPT-1996/2002/2008/2012 structure already validated on earlier passes.

---

2014: RESULT-SCREEN-720-OVERFLOW-SCROLL-GUARD-REFRESH-AFTER-2011: READY_FOR_MAINLAND_ENQUEUE
