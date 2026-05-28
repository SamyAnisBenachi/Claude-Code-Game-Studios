# PROMPT-2013 — Autoplay Placement-Reject Recipe Refresh After 2009

**Branch**: `work/PROMPT-2013`
**Base**: `origin/main @ 20b776e3` (includes PROMPT 2009 viewport shrink guard + PROMPT 2011)
**Date**: 2026-05-28

## Context

PROMPT-2010 was rejected by orchestrator verification: its branch
`origin/work/PROMPT-2010-autoplay-placement-reject-recipe-after-2005` was not
strict-FF over current main and its diff would delete PROMPT-2009 viewport shrink
guard files/reports (`viewport_shrink_guard.py`, `test_viewport_shrink_guard.py`,
`PROMPT-2003-*`, `PROMPT-2009-*`).

This PROMPT-2013 creates a clean branch from `origin/main @ 20b776e3` and recovers
only the owned placement-reject autoplay recipe payload from
`origin/work/PROMPT-2010-autoplay-placement-reject-recipe-after-2005`, via direct
path checkout (no merge/cherry-pick).

## Owned Files Applied

Source branch: `origin/work/PROMPT-2010-autoplay-placement-reject-recipe-after-2005`

Checked out via `git checkout <source> -- <file>` (no merge/cherry-pick):

```
tools/autoplay/recipes/__init__.py                                          M
tools/autoplay/recipes/_coords.py                                           M
tools/autoplay/recipes/placement_reject_probe.py                            A
tests/tools/autoplay/test_recipe_static.py                                  M
reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md  A
reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md  A
reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md  A
reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md  A
reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md  A
reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md  A
reports/PROMPT-2004-autoplay-placement-reject-recipe-refresh-after-1980.md  A
reports/PROMPT-2006-autoplay-placement-reject-recipe-refresh-whitespace-fix.md  A
reports/PROMPT-2010-autoplay-placement-reject-recipe-refresh-after-2005.md  A
```

## Forbidden Files — Confirmed Not Touched

- `tools/autoplay/viewport_shrink_guard.py` — not checked out
- `tests/tools/autoplay/test_viewport_shrink_guard.py` — not checked out
- `client/src/**` — not touched
- `production/**` — not touched

## Protected Files — Confirmed Preserved

PROMPT-2009 viewport shrink guard files remain intact:

- `tools/autoplay/viewport_shrink_guard.py` ✓
- `tests/tools/autoplay/test_viewport_shrink_guard.py` ✓
- `reports/PROMPT-2003-autoplay-midrun-viewport-shrink-guard-refresh-after-1980.md` ✓
- `reports/PROMPT-2009-autoplay-midrun-viewport-shrink-guard-refresh-after-2005.md` ✓

PROMPT-2005 lobby class picker files remain intact:

- `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md` ✓
- `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md` ✓
- `reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md` ✓
- `reports/PROMPT-1998-ui-1280-lobby-class-picker-reachability-refresh-after-1994.md` ✓
- `reports/PROMPT-2005-ui-1280-lobby-class-picker-reachability-refresh-after-1980.md` ✓

## Validation Output

### git diff --name-status origin/main..HEAD
```
A  reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md
A  reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md
A  reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md
A  reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md
A  reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md
A  reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md
A  reports/PROMPT-2004-autoplay-placement-reject-recipe-refresh-after-1980.md
A  reports/PROMPT-2006-autoplay-placement-reject-recipe-refresh-whitespace-fix.md
A  reports/PROMPT-2010-autoplay-placement-reject-recipe-refresh-after-2005.md
M  tests/tools/autoplay/test_recipe_static.py
M  tools/autoplay/recipes/__init__.py
M  tools/autoplay/recipes/_coords.py
A  tools/autoplay/recipes/placement_reject_probe.py
A  reports/PROMPT-2013-autoplay-placement-reject-recipe-refresh-after-2009.md
```
Zero deletions. Zero forbidden files. No viewport_shrink_guard changes.

### git diff --check origin/main..HEAD
Clean for all owned files (settings.json trailing whitespace is pre-existing, not staged).

### pytest tests/tools/autoplay/test_recipe_static.py
```
83 passed in 0.10s
```

### git merge-base --is-ancestor origin/main HEAD
Exit 0 — strict fast-forward confirmed.

2013: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-2009: READY_FOR_MAINLAND_ENQUEUE
