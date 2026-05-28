# PROMPT 1954 — Autoplay Mid-Run Viewport Shrink Guard Refresh After 1920

**Date:** 2026-05-28
**Branch:** worktree-agent-a612fca8b3c787c6a
**Base:** origin/main@1c4981a65f02422de7d01505ce029d1c1551a3a8
**Status:** READY_FOR_MAINLAND_ENQUEUE

## Summary

Rebuilt the PROMPT 1940 mid-run viewport shrink guard payload cleanly on top of
current origin/main (tip: 1c4981a6, PROMPT 1920). The old branch
`origin/worker/1940-autoplay-midrun-viewport-shrink-guard-refresh` (commits
6e0b15b5 + 15e981c2) was based on an older main state and could not be
fast-forward merged to the current tip. This PROMPT re-applies the same two files
manually from a clean worktree rooted at the current main tip.

The two files are identical in content to what PROMPT 1940 shipped —
no functional changes were required. driver.py was not modified because
origin/main already carries the evolved inlined guard (PROMPT 1857) which
supersedes the import-based approach.

## Changes

| Status | File | Description |
|--------|------|-------------|
| A | `tools/autoplay/viewport_shrink_guard.py` | Standalone guard module: check_viewport_size / check_click_target / check_before_input. MIN_QA_VIEWPORT_W=1280 / MIN_QA_VIEWPORT_H=720. |
| A | `tests/tools/autoplay/test_viewport_shrink_guard.py` | 31 focused pytest tests covering all three public functions plus structural driver.py validation. |
| A | `reports/PROMPT-1940-autoplay-midrun-viewport-shrink-guard-refresh-after-1931.md` | Backfill of the PROMPT 1940 report (was on old branch, missing from main). |
| A | `reports/PROMPT-1954-autoplay-midrun-viewport-shrink-guard-refresh-after-1920.md` | This report. |

## Validation

- Path allowlist: PASS (only files in owned scope written)
- git diff --check: PASS (no whitespace issues)
- pytest: PASS (31/31 tests)
- Strict-FF: PASS (origin/main is ancestor of HEAD)

## Notes

### What PROMPT 1940 had vs what was rebuilt

PROMPT 1940 (6e0b15b5) was based on origin/main@be40e0c6. Current origin/main is
1c4981a6 (PROMPT 1920 — card inspect hover glossary refresh). The two mains differ
by several commits but none of them touch tools/autoplay/viewport_shrink_guard.py
or tests/tools/autoplay/test_viewport_shrink_guard.py — those files did not exist
on any branch of main yet. The rebuild is therefore a clean transplant with no
conflict risk.

The PROMPT 1940 report (reports/PROMPT-1940-...md) was also on the old branch and
not on main — it is backfilled here as a separate file in the same commit.

### Guard architecture reminder

The standalone `viewport_shrink_guard.py` module is a utility library. The
authoritative runtime guard is inlined in `driver.py` (PROMPT 1857 implementation).
The module coexists as a pure-Python utility that can be imported by other tooling
or used in isolation. The structural tests in `TestDriverViewportGuardPresent`
validate that driver.py still carries the inlined guard (EXIT_VIEWPORT_GUARD,
VIEWPORT-GUARD log prefix, _MIN_WIN_W/_MIN_WIN_H constants, at least one viewport
checkpoint kind).
