# PROMPT 1990 — Autoplay Placement-Reject Recipe Refresh After 1988

**Date:** 2026-05-28
**Branch:** `work/PROMPT-1990`
**Base:** `origin/main@32ca23e8`
**Source material:** `origin/integrate/autoplay-placement-reject-recipe-1977`

---

## Task

Reapply the `placement-reject-probe` recipe payload from the stale PROMPT 1977 branch
(`origin/integrate/autoplay-placement-reject-recipe-1977`) onto current `origin/main`
(`32ca23e8`) without carrying stale report deletions or disturbing the report chains
protected by the orchestrator (1933/1935/1961/1970/1974/1985/1986/1988).

---

## Execution

### 1. Branch setup

- Worktree already existed at `work/PROMPT-1990`.
- Fast-forwarded branch from stale `2ce3dc6b` to `origin/main@32ca23e8` via `git merge --ff-only`.

### 2. Payload extraction

Checked out only owned files from `FETCH_HEAD` (1977 stale branch):
- `tools/autoplay/recipes/__init__.py` — adds `placement-reject-probe` to registry
- `tools/autoplay/recipes/_coords.py` — adds `BOARD_DEEP_CELL` coordinate constant
- `tools/autoplay/recipes/placement_reject_probe.py` — new recipe (121 lines)
- `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md`
- `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md`
- `reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md`
- `reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md`

No deletions carried over (`git diff --name-status` showed A/M only).

### 3. Test fix

`tests/tools/autoplay/test_recipe_static.py` had a hardcoded `EXPECTED_RECIPES` set of
12 entries that did not include `placement-reject-probe`. Two tests failed:
- `TestRegistry::test_expected_recipe_count` (12 vs 13)
- `TestRegistry::test_expected_recipe_names_present` (unexpected: `placement-reject-probe`)

Fix: added `"placement-reject-probe"` to `EXPECTED_RECIPES` (now 13 entries) and added
three new checkpoint contract tests:
- `test_placement_reject_probe_checkpoints` — verifies `placement-reject-loaded`,
  `placement-reject-feedback`, `placement-reject-recovery-submitted` present
- `test_placement_reject_probe_checkpoint_order` — verifies loaded < feedback < recovery-submitted
- `test_placement_reject_probe_does_not_block` — verifies no `local.block` (no env gate)

---

## Validation

### `git diff --name-status origin/main..HEAD` (pre-report commit)

```
A  reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md
A  reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md
A  reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md
A  reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md
M  tests/tools/autoplay/test_recipe_static.py
M  tools/autoplay/recipes/__init__.py
M  tools/autoplay/recipes/_coords.py
A  tools/autoplay/recipes/placement_reject_probe.py
```

Zero `D` lines. All paths are in owned scope.

### `git diff --check origin/main..HEAD`

PASS — no whitespace errors.

### Focused Python tests

```
pytest tests/tools/autoplay/test_recipe_static.py -v
83 passed in 0.12s
```

### Protected report chain verification

All forbidden-to-delete reports confirmed present on branch:

| Report | Status |
|--------|--------|
| `reports/PROMPT-1933-krosmaga-auction-tier-border-asset-binding-refresh-after-1929.md` | ✓ present |
| `reports/PROMPT-1935-bot-autoplay-story-readiness-report-refresh-after-1931.md` | ✓ present |
| `reports/PROMPT-1961-krosmaga-auction-tier-border-1933-report-backfill-after-1957.md` | ✓ present |
| `reports/PROMPT-1970-bot-autoplay-story-readiness-report-refresh-after-1959.md` | ✓ present |
| `reports/PROMPT-1974-krosmaga-auction-tier-border-1933-report-backfill-after-1972.md` | ✓ present |
| `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md` | ✓ present |
| `reports/PROMPT-1986-krosmaga-auction-tier-border-1933-report-backfill-after-1976.md` | ✓ present |
| `reports/PROMPT-1988-krosmaga-auction-tier-border-1933-report-backfill-after-1985.md` | ✓ present |

---

## Files Delivered

| File | Change |
|------|--------|
| `tools/autoplay/recipes/placement_reject_probe.py` | Added (121 lines) |
| `tools/autoplay/recipes/__init__.py` | Modified — registry entry added |
| `tools/autoplay/recipes/_coords.py` | Modified — `BOARD_DEEP_CELL` constant added |
| `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md` | Carried forward |
| `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md` | Carried forward |
| `reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md` | Carried forward |
| `reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md` | Carried forward |
| `reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md` | This report |
| `tests/tools/autoplay/test_recipe_static.py` | Modified — EXPECTED_RECIPES + 3 new tests |

---

1990: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1988: READY_FOR_MAINLAND_ENQUEUE
