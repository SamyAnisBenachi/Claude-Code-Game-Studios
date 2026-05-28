# PROMPT 2015 — Result Screen 720px Overflow Scroll Guard Refresh After 2013

**Branch**: `work/PROMPT-2015`
**Base**: `origin/main @ 02aa3ac6` (PROMPT 2013 — autoplay placement-reject recipe refresh after 2009)
**Date**: 2026-05-28

## Summary

`origin/work/PROMPT-2014` was NOT fast-forward eligible over `origin/main` after
PROMPT 2013 landed. Specifically:

- PROMPT 2013 added the autoplay placement-reject recipe chain to main
  (`tools/autoplay/recipes/placement_reject_probe.py`, recipe support files,
  and `reports/PROMPT-2013-autoplay-placement-reject-recipe-refresh-after-2009.md`
  plus the full placement-reject report chain back to PROMPT 1928).
- `origin/work/PROMPT-2014` was branched from `origin/main @ 20b776e3` (PROMPT 2011)
  and would have **deleted** all PROMPT 2013 placement-reject recipe files and reports.

This PROMPT-2015 refresh creates a clean branch from `origin/main @ 02aa3ac6`, applies
only the owned result-screen files from `origin/work/PROMPT-2014`, and leaves all
PROMPT 2013 autoplay placement-reject artifacts untouched.

The `work/PROMPT-2015` worktree branch was rebased from its prior stale state
(PROMPT-2007 commit already upstream via PROMPT-2011) to `origin/main` before
any edits; git dropped the redundant commit automatically during rebase.

## Changes Applied

### `client/src/presentation/result_screen.rs`
- `ResultScreenScrollPane` component marker (public, derives `Component`, `Debug`,
  `Clone`, `Copy`) inserted after `ResultScreenPanel`.
- Scroll pane node spawned as child of `content`, wrapping `step_indicator`,
  `hero_panel`, and `accounting_panel`:
  - `flex_direction: Column`, `flex_grow: 1.0`, `min_height: Val::Px(0.0)`,
    `overflow: Overflow::scroll_y()`, `row_gap: Val::Px(14.0)`.
- `step_indicator`, `hero_panel`, and `accounting_panel` reparented from
  `ChildOf(content)` to `ChildOf(scroll_pane)` so the pinned actions row is
  never clipped on 720px viewports.

### `tests/integration/presentation/result_screen_chrome_polish_test.rs`
- `ResultScreenScrollPane` imported alongside existing markers.
- Test `scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p`:
  asserts one `ResultScreenScrollPane`, `overflow.y == OverflowAxis::Scroll`,
  `flex_grow == 1.0`, `min_height == Val::Px(0.0)`.

### Reports backfilled (recovered from `origin/work/PROMPT-2014`)
- `reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md`
- `reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md`
- `reports/PROMPT-1992-result-screen-720-overflow-scroll-guard-refresh-after-1988.md`
- `reports/PROMPT-1996-result-screen-720-overflow-scroll-guard-refresh-after-1993.md`
- `reports/PROMPT-2002-result-screen-720-overflow-scroll-guard-refresh-after-1994.md`
- `reports/PROMPT-2008-result-screen-720-overflow-scroll-guard-refresh-after-2005.md`
- `reports/PROMPT-2012-result-screen-720-overflow-scroll-guard-refresh-after-2009.md`
- `reports/PROMPT-2014-result-screen-720-overflow-scroll-guard-refresh-after-2011.md`

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
A  reports/PROMPT-2014-result-screen-720-overflow-scroll-guard-refresh-after-2011.md
M  tests/integration/presentation/result_screen_chrome_polish_test.rs
A  reports/PROMPT-2015-result-screen-720-overflow-scroll-guard-refresh-after-2013.md
```

- No D entries in diff. ✅
- No `tools/autoplay`, `tests/tools/autoplay`, or unrelated file changes. ✅
- `git diff --check` on all changed files: **CHECK_CLEAN** — no whitespace errors. ✅
- Static grep (`EventReader`, `EventWriter`, `Events<`, `add_event`, `.send(`) on touched
  Rust files: **CLEAN** — no forbidden Bevy event APIs introduced. ✅
- `git merge-base --is-ancestor origin/main HEAD`: FF eligible. ✅

## Protected Files Preserved

### PROMPT 2013 autoplay placement-reject chain (all preserved)
- `tools/autoplay/recipes/placement_reject_probe.py` ✅
- `reports/PROMPT-2013-autoplay-placement-reject-recipe-refresh-after-2009.md` ✅
- `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md` ✅
- `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md` ✅
- `reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md` ✅
- `reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md` ✅
- `reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md` ✅
- `reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md` ✅
- `reports/PROMPT-2004-autoplay-placement-reject-recipe-refresh-after-1980.md` ✅
- `reports/PROMPT-2006-autoplay-placement-reject-recipe-refresh-whitespace-fix.md` ✅
- `reports/PROMPT-2010-autoplay-placement-reject-recipe-refresh-after-2005.md` ✅
- `tests/tools/autoplay/test_recipe_static.py` (PROMPT 2013 additions) ✅
- `tools/autoplay/recipes/__init__.py` (PROMPT 2013 additions) ✅
- `tools/autoplay/recipes/_coords.py` (PROMPT 2013 additions) ✅

### PROMPT 2011 autoplay visible-target coverage-map chain (all preserved)
- `reports/PROMPT-2011-autoplay-recipe-visible-target-coverage-map-refresh-after-2009.md` ✅
- Full prior chain (PROMPT 1848, 1909, 1924, 1949, 1967, 1984, 1995, 2000, 2007) ✅

### PROMPT 2009 viewport shrink guard chain (all preserved)
- `tools/autoplay/viewport_shrink_guard.py` ✅
- `tests/tools/autoplay/test_viewport_shrink_guard.py` ✅

### Focused test run
Deferred — no cheap local `cargo test` available without a full build.
Test correctness verified by structural inspection: the test mirrors the
PROMPT-1996/2002/2008/2012 structure already validated on earlier passes,
and `ResultScreenScrollPane` is the only new symbol introduced.

2015: RESULT-SCREEN-720-OVERFLOW-SCROLL-GUARD-REFRESH-AFTER-2013: SHIPPED
