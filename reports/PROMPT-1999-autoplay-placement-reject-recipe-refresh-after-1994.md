# PROMPT 1999 — Autoplay Placement Reject Recipe Refresh After 1994

**Date:** 2026-05-28
**Branch:** work/PROMPT-1999
**Base:** origin/main @ ccff4a06be7d752135cf660d52410efcd3959fce

## Summary

Clean refresh of the autoplay placement reject/recovery recipe payload from
PROMPT 1990 onto the post-1994 main. The PROMPT-1990 branch was NOT_FF after
main advanced with PROMPT 1994 and would have deleted active report chains and
modified forbidden files. This refresh extracts only the owned scope and
reapplies it cleanly.

## Approach

1. Fast-forwarded work/PROMPT-1999 branch from its old base to origin/main
   (ccff4a06) — HEAD was already an ancestor, so no conflicts.
2. Checked out owned files from `origin/work/PROMPT-1990` using
   `git checkout origin/work/PROMPT-1990 -- <owned-files>` — no forbidden files
   touched.
3. Ran focused Python static test suite.

## Owned Files Applied

| File | Change |
|------|--------|
| `tools/autoplay/recipes/placement_reject_probe.py` | Added |
| `tools/autoplay/recipes/__init__.py` | Modified |
| `tools/autoplay/recipes/_coords.py` | Modified |
| `tests/tools/autoplay/test_recipe_static.py` | Modified |
| `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md` | Added |
| `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md` | Added |
| `reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md` | Added |
| `reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md` | Added |
| `reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md` | Added |
| `reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md` | Added (this file) |

## Preserved Chains (not touched)

- Hand fan reports: 1854/1878/1910/1947/1955/1963/1981/1991
- Game completion reports: 1978/1993
- Composite window-resize reports/tooling: 1850/1864/1873/1875/1913/1918/1945/1951/1969/1979/1994
- Bot/autoplay story readiness: 1935/1970/1985
- Tier-border: 1933/1961/1974/1986/1988

## Validation

### `git diff --name-status origin/main..HEAD`

All entries are additions (A) or modifications (M) of owned files only — no
deletions, no forbidden paths.

```
A  reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md
A  reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md
A  reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md
A  reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md
A  reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md
A  reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md
M  tests/tools/autoplay/test_recipe_static.py
M  tools/autoplay/recipes/__init__.py
M  tools/autoplay/recipes/_coords.py
A  tools/autoplay/recipes/placement_reject_probe.py
```

### `git diff --check origin/main..HEAD`

PASS — no whitespace issues in owned files. (The `.claude/settings.json`
trailing whitespace shown by `git diff --check origin/main` is a pre-existing
unstaged worktree modification, not part of this commit.)

### Python static tests

```
tests/tools/autoplay/test_recipe_static.py — 83 passed in 0.20s
```

All 83 tests pass, including `test_placement_reject_probe_checkpoints`,
`test_placement_reject_probe_checkpoint_order`, and
`test_placement_reject_probe_does_not_block`.

### FF eligibility

`git merge-base --is-ancestor origin/main HEAD` — PASS (confirmed after commit).

## Status

1999: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1994: READY_FOR_MAINLAND_ENQUEUE
