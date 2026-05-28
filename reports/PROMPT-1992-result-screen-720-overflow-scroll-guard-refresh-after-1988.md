# PROMPT 1992 — Result-Screen 720px Overflow Scroll Guard Refresh After 1988

**Date**: 2026-05-28
**Branch**: `work/PROMPT-1992`
**Base**: `origin/main` @ `32ca23e87fa34d5b4484c4a4a42a03a5c2953919`

## Summary

PROMPT 1983's branch (`origin/work/PROMPT-1983`, commit `2f3dcf3d`) was NOT_FF against
current `origin/main@32ca23e8` (PROMPT 1988 tip) and deleted 8 protected report files
(1933/1935/1961/1970/1974/1985/1986/1988 chains). This PROMPT reapplies the
result-screen 720px overflow scroll guard payload via file-level transplant onto a fresh
branch based at `origin/main@32ca23e8`. No stale deletes are included.

## Method

1. Confirmed `work/PROMPT-1992` was behind `origin/main`; fast-forwarded to `32ca23e8`
   via `git merge --ff-only origin/main`.
2. Used `git checkout origin/work/PROMPT-1983 -- <file>` to transplant only the four
   owned files — no stale report deletions carried over.
3. Wrote PROMPT-1992 report (this file).

## Changes

### `client/src/presentation/result_screen.rs`

- Added `ResultScreenScrollPane` marker component (`pub`, `#[derive(Component, Debug, Clone, Copy)]`)
  with doc-comment explaining its role.
- Spawned the scroll pane node as a `ChildOf(content)` child filling available vertical space:
  - `flex_grow: 1.0` — grows to fill space above the pinned actions row
  - `min_height: Val::Px(0.0)` — allows shrinking below intrinsic content height
  - `overflow: Overflow::scroll_y()` — enables scrolling on 720px-tall viewports
  - `row_gap: Val::Px(14.0)` — preserves spacing between wrapped children
- Changed `ResultScreenStepIndicator`, `ResultScreenHeroPanel`, and `ResultScreenAccountingPanel`
  from `ChildOf(content)` to `ChildOf(scroll_pane)`.
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

### `reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md`

Transplanted from `origin/work/PROMPT-1983`. Was absent from `origin/main`
(reports/ is gitignored on the stale branch; file was only on the rejected 1962 branch).

### `reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md`

Transplanted from `origin/work/PROMPT-1983`.

## Preserved (Untouched)

Protected report chains — all confirmed present on branch:
- `reports/PROMPT-1933-krosmaga-auction-tier-border-asset-binding-refresh-after-1929.md` ✓
- `reports/PROMPT-1935-bot-autoplay-story-readiness-report-refresh-after-1931.md` ✓
- `reports/PROMPT-1961-krosmaga-auction-tier-border-1933-report-backfill-after-1957.md` ✓
- `reports/PROMPT-1970-bot-autoplay-story-readiness-report-refresh-after-1959.md` ✓
- `reports/PROMPT-1974-krosmaga-auction-tier-border-1933-report-backfill-after-1972.md` ✓
- `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md` ✓
- `reports/PROMPT-1986-krosmaga-auction-tier-border-1933-report-backfill-after-1976.md` ✓
- `reports/PROMPT-1988-krosmaga-auction-tier-border-1933-report-backfill-after-1985.md` ✓

All non-owned source files preserved; no touches outside owned scope.

## Validation

### Path Allowlist Review

Files changed vs `origin/main`:
```
M  client/src/presentation/result_screen.rs
M  tests/integration/presentation/result_screen_chrome_polish_test.rs
A  reports/PROMPT-1962-result-screen-720-overflow-scroll-guard-refresh-after-1957.md
A  reports/PROMPT-1983-result-screen-720-overflow-scroll-guard-refresh-after-1976.md
A  reports/PROMPT-1992-result-screen-720-overflow-scroll-guard-refresh-after-1988.md
```

All within owned scope. No forbidden files touched. Zero D (deleted) lines.

### `git diff --check`

PASS — no whitespace errors in owned files.

### Zero-Delete Validation

`git diff --name-status origin/main..HEAD` shows only M and A entries — zero D lines. PASS.

### Strict FF Check

`git merge-base --is-ancestor origin/main HEAD` → PASS (branch was fast-forwarded to origin/main
before applying changes; all owned commits are strictly on top).

### Focused Test

```
cargo test --test result_screen_chrome_polish_test
```

See test run output in commit evidence. All 6 tests pass (5 pre-existing + 1 new scroll guard test).

---

1992: RESULT-SCREEN-720-OVERFLOW-SCROLL-GUARD-REFRESH-AFTER-1988: READY_FOR_MAINLAND_ENQUEUE
