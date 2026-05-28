# PROMPT 1940 — Autoplay Mid-Run Viewport Shrink Guard Refresh After 1931

**Date:** 2026-05-28
**Branch:** `worker/1940-autoplay-midrun-viewport-shrink-guard-refresh`
**Commit:** `6e0b15b5`
**Source commit:** `f370a714` (PROMPT 1922)
**Base:** `origin/main @ be40e0c6`

---

## Summary

Reapplied the PROMPT 1922 viewport shrink guard payload cleanly onto current
`origin/main` (after PROMPT 1931 landed, which itself post-dated 1922).

Cherry-pick of `f370a714` conflicted in `driver.py` because `origin/main` already
carries an evolved inlined implementation of the same guard (introduced by PROMPT
1857, post-dating 1922). The cherry-pick was aborted; the three-file payload was
applied manually using the minimal-patch approach.

---

## Files Changed

| Status | File |
|--------|------|
| A | `tools/autoplay/viewport_shrink_guard.py` |
| A | `tests/tools/autoplay/test_viewport_shrink_guard.py` |

`tools/autoplay/driver.py` was **not modified** — `origin/main` already has the
evolved inlined guard (PROMPT 1857) which supersedes the import-based approach from
1922. The standalone module is retained as an independent utility.

---

## Conflict Resolution Details

`f370a714` driver.py patch:
- Added `from viewport_shrink_guard import check_before_input as _viewport_check`
- Called `_viewport_check(params, status)` before `autoplay/input` dispatch
- Logged `VIEWPORT-SHRINK-BLOCK` / emitted `viewport_shrink_block` checkpoint kind

`origin/main` driver.py (PROMPT 1857 evolution):
- Removed the external import
- Inlined full guard logic: `_validate_cursor_coords`, `_parse_window_size`,
  `_check_window_minimum`, `_check_window_drift`
- Defined `EXIT_VIEWPORT_GUARD = 5`, `_MIN_WIN_W = 1280.0`, `_MIN_WIN_H = 720.0`
- Logs `VIEWPORT-GUARD` prefix; emits `viewport_shrink_abort`, `viewport_guard_oob`,
  `viewport_guard_cursor_none`, `viewport_drift` checkpoint kinds
- Additional post-foreground size re-check (AC-VPT-08) not in 1922

The inlined implementation is a strict superset of the 1922 guard functionality.

---

## Test Adaptation

The original 1922 `TestDriverImportsViewportGuard` class was adapted to
`TestDriverViewportGuardPresent` which validates the current inlined
implementation:

| 1922 test (was) | 1940 test (now) |
|-----------------|-----------------|
| `driver imports viewport_shrink_guard` | `driver has EXIT_VIEWPORT_GUARD constant` |
| `driver calls _viewport_check(` | `driver logs VIEWPORT-GUARD prefix` |
| `driver logs VIEWPORT-SHRINK-BLOCK` | `driver emits a viewport checkpoint kind (any of 5)` |
| `driver emits viewport_shrink_block checkpoint` | `driver has _MIN_WIN_W / _MIN_WIN_H constants` |
| ordering: guard after input branch | `VIEWPORT-GUARD` appears after `method == "autoplay/input"` (using `str.find` from that index) |

---

## Validation

### `git diff --check`
```
WHITESPACE_OK (no issues)
```

### `git diff --name-status origin/main..HEAD`
```
A    tests/tools/autoplay/test_viewport_shrink_guard.py
A    tools/autoplay/viewport_shrink_guard.py
```
Only owned-scope files. No deleted reports, no unrelated changes.

### pytest
```
31 passed in 0.05s
```
Full test run: 31/31 PASSED across:
- `TestCheckViewportSize` (8 tests)
- `TestCheckClickTarget` (9 tests)
- `TestCheckBeforeInput` (9 tests)
- `TestDriverViewportGuardPresent` (5 structural tests)

---

## QA-001 C0 / Human-Review Caveat

Preserved as required. Automated evidence from the viewport shrink guard is useful
for detecting bot click misses caused by an undersized window, but human
viewport/operator verification remains the recommended gate until GUI conditions
(full 1280x720 desktop window, no compositor scaling) are confirmed on the
operator's machine. The guard provides a loud EXIT_VIEWPORT_GUARD (rc=5) signal
to surface the problem; it does not replace manual confirmation of visual state.

---

## FF-Readiness

Branch is **1 commit ahead** of `origin/main` (`be40e0c6`), no divergence.
Fast-forward merge is clean.

1940: AUTOPLAY-MIDRUN-VIEWPORT-SHRINK-GUARD-REFRESH-AFTER-1931: READY_FOR_MAINLAND_ENQUEUE
