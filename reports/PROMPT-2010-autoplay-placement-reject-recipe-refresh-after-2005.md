# PROMPT-2010 — Autoplay Placement-Reject Recipe Refresh After 2005

**Branch**: `work/PROMPT-2010-autoplay-placement-reject-recipe-after-2005`
**Base**: `origin/main @ fa189edf403dcf9bc12eddc315caf23bb6095e9b`
**Date**: 2026-05-28

## Context

PROMPT-2006 was rejected by orchestrator verification: its branch was not
fast-forward over origin/main after PROMPT-2005 landed, and it deleted protected
lobby reports (`1958/1973/1987/1998/2005`) while also touching forbidden lobby
files (`client/src/ui/lobby.rs`, lobby layout tests).

This PROMPT-2010 creates a clean branch directly from `origin/main @ fa189edf`
and recovers only the owned placement-reject autoplay recipe payload from
`origin/work/PROMPT-2006-autoplay-placement-reject-whitespace-fix`.

## Owned Files Applied

Source branch: `origin/work/PROMPT-2006-autoplay-placement-reject-whitespace-fix @ 7a1b123a`

Checked out via `git checkout <source> -- <file>` (no merge/cherry-pick):

```
tools/autoplay/recipes/__init__.py          M
tools/autoplay/recipes/_coords.py           M
tools/autoplay/recipes/placement_reject_probe.py  A
tests/tools/autoplay/test_recipe_static.py  M
reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md  A
reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md  A
reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md  A
reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md  A
reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md  A
reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md  A
reports/PROMPT-2004-autoplay-placement-reject-recipe-refresh-after-1980.md  A
reports/PROMPT-2006-autoplay-placement-reject-recipe-refresh-whitespace-fix.md  A
```

## Forbidden Files — Confirmed Not Touched

- `client/src/ui/lobby.rs` — not checked out
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs` — not checked out
- `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` — not checked out

## Protected Reports — Confirmed Preserved

PROMPT-1980 reports and PROMPT-2005 lobby reports remain intact on this branch:

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
M  tests/tools/autoplay/test_recipe_static.py
M  tools/autoplay/recipes/__init__.py
M  tools/autoplay/recipes/_coords.py
A  tools/autoplay/recipes/placement_reject_probe.py
A  reports/PROMPT-2010-autoplay-placement-reject-recipe-refresh-after-2005.md
```
Zero deletions. Zero forbidden files.

### git diff --check origin/main..HEAD
Clean — exit 0.

### pytest tests/tools/autoplay/test_recipe_static.py
```
83 passed in 0.16s
```

### git merge-base --is-ancestor origin/main HEAD
Exit 0 — strict fast-forward confirmed.

2010: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-2005: READY_FOR_MAINLAND_ENQUEUE
