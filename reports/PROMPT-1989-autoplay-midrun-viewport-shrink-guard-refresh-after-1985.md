# PROMPT 1989 — Autoplay Mid-Run Viewport Shrink Guard Refresh After 1985

**Branch**: `work/PROMPT-1989`
**Base**: `origin/main @ b354bee6` (PROMPT 1985 — bot/autoplay story readiness report refresh after 1976)
**Date**: 2026-05-28
**Status**: READY_FOR_MAINLAND_ENQUEUE

---

## Context

PROMPT 1975 performed a clean transplant of the viewport shrink guard onto
`origin/main@7b259e91` (post-1972), but its local worktree
(`D:\tmp\wt-1975-viewport-shrink-guard @ 88c6e233`) was NOT fast-forward
against `origin/main@b354bee6` after PROMPT 1985 landed (bot/autoplay story
readiness reports). Wholesale cherry-pick of 88c6e233 would have been clean
(no deletions), but the branch was based on an older tip.

This PROMPT performs a clean file-level transplant of only the owned scope onto
fresh `origin/main@b354bee6`, preserving all PROMPT 1985 and prior work.

---

## Owned Scope Transplanted

| File | Source | Action |
|------|--------|--------|
| `tools/autoplay/viewport_shrink_guard.py` | `D:\tmp\wt-1975-viewport-shrink-guard @ 88c6e233` | Transplanted (file-level `git show`) |
| `tests/tools/autoplay/test_viewport_shrink_guard.py` | same | Transplanted (file-level `git show`) |
| `reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md` | same | Transplanted |
| `reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md` | same | Transplanted |
| `reports/PROMPT-1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959.md` | same | Transplanted |
| `reports/PROMPT-1975-autoplay-midrun-viewport-shrink-guard-refresh-after-1972.md` | same | Transplanted |
| `reports/PROMPT-1989-autoplay-midrun-viewport-shrink-guard-refresh-after-1985.md` | this file | New |

No changes to `client/**`, `server/**`, `Cargo.*`, `production/**`, `tests/unit/**`,
or any unrelated reports.

---

## Validation

### FF Check
```
git merge-base --is-ancestor origin/main HEAD → exit 0 (IS_ANCESTOR_OK)
```
Branch is strict fast-forward over `origin/main@b354bee6`.

### Diff Scope
```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md
A  reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md
A  reports/PROMPT-1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959.md
A  reports/PROMPT-1975-autoplay-midrun-viewport-shrink-guard-refresh-after-1972.md
A  reports/PROMPT-1989-autoplay-midrun-viewport-shrink-guard-refresh-after-1985.md
A  tests/tools/autoplay/test_viewport_shrink_guard.py
A  tools/autoplay/viewport_shrink_guard.py
```
7 files, additions only; zero D lines; all within owned scope.

### Whitespace Check
```
git diff --check origin/main..HEAD → exit 0 (CHECK_CLEAN)
```

### Pytest
```
pytest tests/tools/autoplay/test_viewport_shrink_guard.py -v
Platform: win32 — Python 3.12.10, pytest-9.0.3
31 collected items
31 passed in 0.09s
```

Full test classes:
- `TestCheckViewportSize` — 8 tests — all PASSED
- `TestCheckClickTarget` — 9 tests — all PASSED
- `TestCheckBeforeInput` — 9 tests — all PASSED
- `TestDriverViewportGuardPresent` — 5 structural tests — all PASSED
  (driver.py on `origin/main` already has `EXIT_VIEWPORT_GUARD`, `VIEWPORT-GUARD`,
  `_MIN_WIN_W`, `_MIN_WIN_H`, viewport checkpoint kinds, and the guard inside
  the `method == "autoplay/input"` branch from earlier landed PROMPTs 1857/1880)

---

## Implementation Notes

- `viewport_shrink_guard.py` exposes three pure functions: `check_viewport_size`,
  `check_click_target`, `check_before_input` with no side effects.
- Minimum QA viewport: `MIN_QA_VIEWPORT_W=1280.0`, `MIN_QA_VIEWPORT_H=720.0`
  (per PROMPT 1894 baseline).
- Guards run against `window_logical_size` from the live `autoplay/status` response,
  catching mid-run shrink events on the tick they first appear.
- `driver.py` already has inlined viewport guard code from PROMPT 1857/1880; the
  standalone `viewport_shrink_guard.py` module coexists as a reusable utility.

---

## Preserved Work

Confirmed unchanged on this branch vs `origin/main@b354bee6`:
- PROMPT 1985 bot/autoplay story readiness reports (refresh after 1976)
- PROMPT 1976 operator contract backfill reports (1861/1914/1941/1964/1968)
- PROMPT 1972 autoplay vsbot signoff-pack reports (refresh after 1959)
- PROMPT 1959 krosmaga-UI-stage3 reports
- PROMPT 1957 auction tier-border asset binding (`src/ui/auction.rs`, reports)
- PROMPT 1920 card-inspect hover glossary (`src/ui/card_inspect.rs`, related reports)

### Protected Reports Still Present
- `reports/PROMPT-1935-bot-autoplay-story-readiness-report-refresh-after-1931.md` ✓
- `reports/PROMPT-1970-bot-autoplay-story-readiness-report-refresh-after-1959.md` ✓
- `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md` ✓

---

1989: AUTOPLAY-MIDRUN-VIEWPORT-SHRINK-GUARD-REFRESH-AFTER-1985: READY_FOR_MAINLAND_ENQUEUE
