# PROMPT 2004 — Autoplay Placement-Reject Recipe Refresh After 1980

**Date:** 2026-05-28
**Branch:** work/PROMPT-2004
**Commit:** 2e23e85b8772cbd79f64218c5ae3a8775f556795
**Base origin/main SHA:** f16d60416651cbbaa9443ec76da25fae2f552af9

## Summary

Recovered the placement-reject autoplay recipe payload onto a clean branch
from `origin/main`. The prior worker branch `origin/work/PROMPT-1999`
(af5c0388) was rejected by orchestrator as NOT_FF over current main, and
would have deleted the viewport/window-guard reports from PROMPT 1980
(`1916/1948/1966/1980`).

This refresh creates a strict-FF branch by:
1. Fast-forwarding `work/PROMPT-2004` to `origin/main` (f16d6041)
2. Recovering only the 10 owned files via `git checkout origin/work/PROMPT-1999 -- <files>`
3. Verifying no deletions; running focused Python test suite
4. Committing the owned payload

## Owned Files Recovered

| File | Status |
|------|--------|
| `tools/autoplay/recipes/__init__.py` | Modified |
| `tools/autoplay/recipes/_coords.py` | Modified |
| `tools/autoplay/recipes/placement_reject_probe.py` | Added |
| `tests/tools/autoplay/test_recipe_static.py` | Modified |
| `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md` | Added |
| `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md` | Added |
| `reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md` | Added |
| `reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md` | Added |
| `reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md` | Added |
| `reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md` | Added |

## PROMPT-1980 Reports Preservation

All four protected reports confirmed present on branch:

- `reports/PROMPT-1916-post-1912-autoplay-viewport-window-guard-focused-verify.md` ✓
- `reports/PROMPT-1948-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1943.md` ✓
- `reports/PROMPT-1966-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1957.md` ✓
- `reports/PROMPT-1980-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1976.md` ✓

## Validation

### `git diff --name-status origin/main..HEAD`

```
A  reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md
A  reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md
A  reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md
A  reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md
A  reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md
A  reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md
A  reports/PROMPT-2004-autoplay-placement-reject-recipe-refresh-after-1980.md
M  tests/tools/autoplay/test_recipe_static.py
M  tools/autoplay/recipes/__init__.py
M  tools/autoplay/recipes/_coords.py
A  tools/autoplay/recipes/placement_reject_probe.py
```

Zero deletions. All entries are A (added) or M (modified).

### `git diff --check origin/main..HEAD`

Clean — no whitespace errors.

### `git merge-base --is-ancestor origin/main HEAD`

Exit code 0 — STRICT_FF: PASS

### Python tests: `python -m pytest tests/tools/autoplay/test_recipe_static.py`

```
83 passed in 0.10s
```

All 83 tests pass, including `TestCheckpointContracts::test_placement_reject_probe_checkpoints`,
`test_placement_reject_probe_checkpoint_order`, and `test_placement_reject_probe_does_not_block`.

---

2004: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1980: READY_FOR_MAINLAND_ENQUEUE
