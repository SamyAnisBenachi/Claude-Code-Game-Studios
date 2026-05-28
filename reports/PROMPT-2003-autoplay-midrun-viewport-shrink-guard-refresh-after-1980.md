# PROMPT 2003 — AUTOPLAY-MIDRUN-VIEWPORT-SHRINK-GUARD-REFRESH-AFTER-1980

**Date:** 2026-05-28
**Branch:** work/PROMPT-2003
**Base origin/main SHA:** f16d60416651cbbaa9443ec76da25fae2f552af9

## Summary

PROMPT 1997 was rejected because its branch (`origin/work/PROMPT-1997`) was
NOT fast-forward over current `origin/main` (which had landed PROMPT 1980 and
related reports after 1997's branch diverged).

This PROMPT-2003 worker created a clean refresh by:

1. Fast-forwarding `work/PROMPT-2003` to `origin/main` (FF merge, no conflicts)
2. Recovering the 8 owned files from `origin/work/PROMPT-1997` via `git show`
3. Confirming that all 6 report files in scope already exist verbatim on
   `origin/main` (landed by prior refresh cycles — no re-add needed)
4. Staging and committing the 2 tool files that were NOT yet on main:
   - `tools/autoplay/viewport_shrink_guard.py`
   - `tests/tools/autoplay/test_viewport_shrink_guard.py`

## Validation

### git diff --name-status (staged, pre-commit)

```
A  tests/tools/autoplay/test_viewport_shrink_guard.py
A  tools/autoplay/viewport_shrink_guard.py
```

Zero deletions. No files outside owned scope touched.

### git diff --check

Clean — no whitespace errors.

### FF check

```
git merge-base --is-ancestor origin/main HEAD → exit 0
```

Strict FF confirmed after commit.

### Python test suite

```
tests/tools/autoplay/test_viewport_shrink_guard.py — 31 passed in 0.07s
```

All 31 tests pass.

## PROMPT 1980 Report Preservation

The following reports (the PROMPT 1980 chain) were verified present on main and
remain untouched:

- `reports/PROMPT-1916-post-1912-autoplay-viewport-window-guard-focused-verify.md` ✓
- `reports/PROMPT-1948-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1943.md` ✓
- `reports/PROMPT-1966-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1957.md` ✓
- `reports/PROMPT-1980-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1976.md` ✓

None of these were modified, deleted, or touched by this PROMPT.

## Owned Files Status

| File | Action | Result |
|------|--------|--------|
| `tools/autoplay/viewport_shrink_guard.py` | Added (new to main) | ✓ committed |
| `tests/tools/autoplay/test_viewport_shrink_guard.py` | Added (new to main) | ✓ committed |
| `reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md` | Already on main (verbatim match) | ✓ no-op |
| `reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md` | Already on main (verbatim match) | ✓ no-op |
| `reports/PROMPT-1971-autoplay-midrun-viewport-shrink-guard-refresh-after-1959.md` | Already on main (verbatim match) | ✓ no-op |
| `reports/PROMPT-1975-autoplay-midrun-viewport-shrink-guard-refresh-after-1972.md` | Already on main (verbatim match) | ✓ no-op |
| `reports/PROMPT-1989-autoplay-midrun-viewport-shrink-guard-refresh-after-1985.md` | Already on main (verbatim match) | ✓ no-op |
| `reports/PROMPT-1997-autoplay-midrun-viewport-shrink-guard-refresh-after-1993.md` | Already on main (verbatim match) | ✓ no-op |
| `reports/PROMPT-2003-autoplay-midrun-viewport-shrink-guard-refresh-after-1980.md` | This report | ✓ committed |

---

2003: AUTOPLAY-MIDRUN-VIEWPORT-SHRINK-GUARD-REFRESH-AFTER-1980: READY_FOR_MAINLAND_ENQUEUE
