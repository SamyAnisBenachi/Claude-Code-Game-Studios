# PROMPT 1971 — Autoplay Mid-Run Viewport Shrink Guard Refresh After 1959

**Branch**: `worker/1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959`
**Base**: `origin/main @ 7fc1706e` (PROMPT 1959 — krosmaga-ui-stage3 reports)
**Date**: 2026-05-28
**Status**: READY_FOR_MAINLAND_ENQUEUE

---

## Context

PROMPT 1954 rebuilt the viewport shrink guard onto `origin/main@1c4981a6` (post-1920), but
its worker branch (`origin/worker/1954-autoplay-midrun-viewport-shrink-guard-refresh`) was
NOT fast-forward against current `origin/main@7fc1706e` after PROMPT 1957 and PROMPT 1959
landed. Wholesale merge or cherry-pick would overwrite PROMPT 1920 card-inspect changes,
PROMPT 1957 auction tier-border work, and PROMPT 1959 krosmaga-UI-stage3 reports.

This PROMPT performs a clean file-level transplant of only the owned scope.

---

## Owned Scope Transplanted

| File | Source | Action |
|------|--------|--------|
| `tools/autoplay/viewport_shrink_guard.py` | `origin/worker/1954-autoplay-midrun-viewport-shrink-guard-refresh @ 934e462a` | Transplanted (file-level `git show`) |
| `tests/tools/autoplay/test_viewport_shrink_guard.py` | same | Transplanted (file-level `git show`) |
| `reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md` | same | Transplanted |
| `reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md` | same | Transplanted |
| `reports/PROMPT-1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959.md` | this file | New |

No changes to `client/**`, `server/**`, `Cargo.*`, `production/**`, `tests/unit/**`,
`tools/autoplay/driver.py`, or any unrelated reports.

---

## Validation

### FF Check
```
git merge-base --is-ancestor origin/main HEAD → exit 0 (IS_ANCESTOR_OK)
```

### Diff Scope
```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md
A  reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md
A  reports/PROMPT-1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959.md
A  tests/tools/autoplay/test_viewport_shrink_guard.py
A  tools/autoplay/viewport_shrink_guard.py
```
All 5 files are additions only; no deletions; all within owned scope.

### Whitespace Check
```
git diff --check origin/main..HEAD → no output (CHECK_CLEAN)
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

Confirmed unchanged on this branch vs `origin/main`:
- PROMPT 1920 card-inspect hover glossary (`src/ui/card_inspect.rs`, related reports)
- PROMPT 1957 auction tier-border asset binding (`src/ui/auction.rs`, reports)
- PROMPT 1959 krosmaga-UI-stage3 reports

---

1971: AUTOPLAY-MIDRUN-VIEWPORT-SHRINK-GUARD-REFRESH-AFTER-1959: READY_FOR_MAINLAND_ENQUEUE
