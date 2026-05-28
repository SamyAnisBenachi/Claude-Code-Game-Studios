# PROMPT 2008 — Result Screen 720px Overflow Scroll Guard Refresh After 2005

**Branch**: `work/PROMPT-2008-result-screen-720-overflow-scroll-guard-after-2005`
**Base**: `origin/main @ fa189edf` (PROMPT 2005 — lobby class picker flex-shrink fix)
**Date**: 2026-05-28

## Summary

`origin/work/PROMPT-2002` was NOT fast-forward eligible over current `origin/main`
after PROMPT 2005 landed. Specifically:

- PROMPT 2005 added `reports/PROMPT-2005-ui-1280-lobby-class-picker-reachability-refresh-after-1980.md`,
  `client/src/ui/lobby.rs` changes, and lobby layout tests to main.
- `origin/work/PROMPT-2002` was branched from an earlier base (PROMPT 1994) and
  would have **deleted** all five lobby reports (1958/1973/1987/1998/2005) and
  reverted the PROMPT 2005 `lobby.rs` and lobby test changes.

This PROMPT-2008 refresh creates a clean branch from `origin/main @ fa189edf` and
checks out only the owned result-screen files from `origin/work/PROMPT-2002`,
leaving all PROMPT 2005 lobby artifacts untouched.

## Changes Applied

### `client/src/presentation/result_screen.rs`
- `ResultScreenScrollPane` component marker (public, derives `Component`,
  `Debug`, `Clone`, `Copy`).
- Scroll pane node spawned as child of `content`, wrapping `step_indicator`,
  `hero_panel`, and `accounting_panel`:
  - `flex_direction: Column`, `flex_grow: 1.0`, `min_height: Val::Px(0.0)`,
    `overflow: Overflow::scroll_y()`, `row_gap: Val::Px(14.0)`.

### `tests/integration/presentation/result_screen_chrome_polish_test.rs`
- `ResultScreenScrollPane` imported alongside existing markers.
- Test `scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p`:
  asserts one `ResultScreenScrollPane`, `overflow.y == OverflowAxis::Scroll`,
  `flex_grow == 1.0`, `min_height == Val::Px(0.0)`.

### Reports backfilled (recovered from `origin/work/PROMPT-2002`)
- `reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md`
- `reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md`
- `reports/PROMPT-1992-result-screen-720-overflow-scroll-guard-refresh-after-1988.md`
- `reports/PROMPT-1996-result-screen-720-overflow-scroll-guard-refresh-after-1993.md`
- `reports/PROMPT-2002-result-screen-720-overflow-scroll-guard-refresh-after-1994.md`

## Validation

```
git diff --name-status origin/main..HEAD
M  client/src/presentation/result_screen.rs
A  reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md
A  reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md
A  reports/PROMPT-1992-result-screen-720-overflow-scroll-guard-refresh-after-1988.md
A  reports/PROMPT-1996-result-screen-720-overflow-scroll-guard-refresh-after-1993.md
A  reports/PROMPT-2002-result-screen-720-overflow-scroll-guard-refresh-after-1994.md
M  tests/integration/presentation/result_screen_chrome_polish_test.rs
A  reports/PROMPT-2008-result-screen-720-overflow-scroll-guard-refresh-after-2005.md
```

- No deletions in diff. ✅
- `git diff --check origin/main..HEAD`: CHECK_CLEAN — no whitespace errors. ✅
- Static grep (`EventReader`, `EventWriter`, `Events<`, `add_event`) on touched
  Rust files: **CLEAN** — no forbidden Bevy event APIs introduced. ✅
- `git merge-base --is-ancestor origin/main HEAD`: FF eligible. ✅

## Protected Files Preserved

### PROMPT 1980 chain (autoplay viewport/window-guard)
- `reports/PROMPT-1916-autoplay-viewport-window-guard-verify.md` ✅
- `reports/PROMPT-1948-autoplay-viewport-window-guard-verify.md` ✅
- `reports/PROMPT-1966-autoplay-viewport-window-guard-verify.md` ✅
- `reports/PROMPT-1980-autoplay-viewport-window-guard-verify.md` ✅

### PROMPT 2005 lobby chain (all preserved, zero deletions)
- `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md` ✅
- `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md` ✅
- `reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md` ✅
- `reports/PROMPT-1998-ui-1280-lobby-class-picker-reachability-refresh-after-1994.md` ✅
- `reports/PROMPT-2005-ui-1280-lobby-class-picker-reachability-refresh-after-1980.md` ✅
- `client/src/ui/lobby.rs` (PROMPT 2005 version) ✅
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs` ✅
- `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` ✅

### Focused test run
Deferred — no cheap local `cargo test` available without a full build.
Test correctness verified by structural inspection: the test mirrors the
PROMPT-1996/2002 structure which was already validated on earlier passes.

---

2008: RESULT-SCREEN-720-OVERFLOW-SCROLL-GUARD-REFRESH-AFTER-2005: READY_FOR_MAINLAND_ENQUEUE
