# PROMPT 1997 — Autoplay Mid-Run Viewport Shrink Guard Refresh After 1993

**Branch**: `work/PROMPT-1997`
**Base**: `origin/main @ 56839ef1` (PROMPT 1993 — reapply game-completion next-wave map PROMPT 1978 report after 1991 mainland)
**Date**: 2026-05-28
**Status**: READY_FOR_MAINLAND_ENQUEUE

---

## Context

PROMPT 1989 performed a clean transplant of the viewport shrink guard onto
`origin/main@b354bee6` (post-1985), but its branch (`origin/work/PROMPT-1989`)
was NOT fast-forward against `origin/main@56839ef1` after PROMPTs 1991 and 1993
landed (hand fan readability Stage3-D + game-completion next-wave map report).
Wholesale reuse of `origin/work/PROMPT-1989` would have deleted current hand-fan
reports 1854/1878/1910/1947/1955/1963/1981/1991 and game-completion reports
1978/1993, and touched unrelated hand UI files/tests.

This PROMPT performs a clean file-level transplant of only the owned scope onto
fresh `origin/main@56839ef1`, preserving all PROMPT 1991/1993 and prior work.

---

## Owned Scope Transplanted

| File | Source | Action |
|------|--------|--------|
| `tools/autoplay/viewport_shrink_guard.py` | `origin/work/PROMPT-1989 @ 842838d1` | Transplanted (file-level `git checkout`) |
| `tests/tools/autoplay/test_viewport_shrink_guard.py` | same | Transplanted |
| `reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md` | same | Transplanted |
| `reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md` | same | Transplanted |
| `reports/PROMPT-1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959.md` | same | Transplanted |
| `reports/PROMPT-1975-autoplay-midrun-viewport-shrink-guard-refresh-after-1972.md` | same | Transplanted |
| `reports/PROMPT-1989-autoplay-midrun-viewport-shrink-guard-refresh-after-1985.md` | same | Transplanted |
| `reports/PROMPT-1997-autoplay-midrun-viewport-shrink-guard-refresh-after-1993.md` | this file | New |

No changes to `client/**`, `server/**`, `Cargo.*`, `production/**`, `tests/unit/**`,
`tests/integration/hand-ui/**`, or any unrelated reports.

---

## Validation

### FF Check
```
git merge-base --is-ancestor origin/main HEAD → exit 0 (IS_ANCESTOR_OK)
```
Branch is strict fast-forward over `origin/main@56839ef1`.

### Diff Scope
```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md
A  reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md
A  reports/PROMPT-1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959.md
A  reports/PROMPT-1975-autoplay-midrun-viewport-shrink-guard-refresh-after-1972.md
A  reports/PROMPT-1989-autoplay-midrun-viewport-shrink-guard-refresh-after-1985.md
A  reports/PROMPT-1997-autoplay-midrun-viewport-shrink-guard-refresh-after-1993.md
A  tests/tools/autoplay/test_viewport_shrink_guard.py
A  tools/autoplay/viewport_shrink_guard.py
```
8 files, additions only; zero D lines; all within owned scope.

### Whitespace Check
```
git diff --check origin/main..HEAD → exit 0 (CHECK_CLEAN)
```

### Pytest
```
pytest tests/tools/autoplay/test_viewport_shrink_guard.py -v
Platform: win32 — Python 3.12.10, pytest-9.0.3
31 collected items
31 passed in 0.05s
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

Confirmed unchanged on this branch vs `origin/main@56839ef1`:
- PROMPT 1991 hand fan readability Stage3-D (`client/src/ui/hand/**`, reports)
- PROMPT 1993 game-completion next-wave map report
- PROMPT 1988 tier-border reports (1933/1961/1974/1986)
- PROMPT 1985 bot/autoplay story readiness reports
- All hand-fan reports: 1854/1878/1910/1947/1955/1963/1981/1991 ✓
- All game-completion reports: 1978/1993 ✓

### Protected Reports Still Present
- `reports/PROMPT-1935-bot-autoplay-story-readiness-report-refresh-after-1931.md` ✓
- `reports/PROMPT-1970-bot-autoplay-story-readiness-report-refresh-after-1959.md` ✓
- `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md` ✓
- `reports/PROMPT-1978-game-completion-next-wave-map-report-refresh-after-1976.md` ✓
- `reports/PROMPT-1993-game-completion-next-wave-map-report-refresh-after-1988.md` ✓
- `reports/PROMPT-1991-krosmaga-hand-fan-readability-stage3-refresh-after-1988.md` ✓

---

1997: AUTOPLAY-MIDRUN-VIEWPORT-SHRINK-GUARD-REFRESH-AFTER-1993: READY_FOR_MAINLAND_ENQUEUE
