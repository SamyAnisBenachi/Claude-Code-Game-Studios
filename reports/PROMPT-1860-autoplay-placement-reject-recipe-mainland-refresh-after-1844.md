# PROMPT 1860 — Autoplay Placement-Reject Recipe Mainland Refresh After 1844

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-placement-reject-recipe-1860`
**Base SHA:** `bb90d7c2` (origin/main after PROMPT 1844 main-land)
**Source branch:** `origin/integrate/autoplay-placement-reject-recipe-1849 @ ff2f5b1c`
**Worker worktree:** `tmpwt-1860-placement-reject-mainland`

## Context

PROMPT 1849 shipped the placement-reject-probe recipe on branch
`integrate/autoplay-placement-reject-recipe-1849` but that branch was based
before PROMPT 1844 main-landed. A direct diff showed it would delete
`reports/PROMPT-1844-autoplay-vsbot-viewport-click-evidence-audit.md`.

This worker refreshes the recipe payload onto the current `origin/main@bb90d7c2`
by cherry-picking only the two relevant commits from 1849:

| Commit | What |
|--------|------|
| `1847459f` | feat(autoplay): PROMPT 1832 — placement-reject-probe recipe |
| `ff2f5b1c` | docs(reports): PROMPT 1849 — placement-reject-recipe integration refresh report |

## Files Carried

| File | Status |
|------|--------|
| `tools/autoplay/recipes/__init__.py` | M — recipe registered |
| `tools/autoplay/recipes/_coords.py` | M — BOARD_DEEP_CELL coord added |
| `tools/autoplay/recipes/placement_reject_probe.py` | A — new recipe |
| `reports/PROMPT-1849-autoplay-placement-reject-recovery-recipe-integration-refresh.md` | A — 1849 report |
| `reports/PROMPT-1860-autoplay-placement-reject-recipe-mainland-refresh-after-1844.md` | A — this report |

## Validation

### Merge-base ancestry
```
git merge-base --is-ancestor origin/main HEAD → exit 0 (PASS)
```

### Diff vs origin/main (no deletions)
```
A  reports/PROMPT-1849-autoplay-placement-reject-recovery-recipe-integration-refresh.md
M  tools/autoplay/recipes/__init__.py
M  tools/autoplay/recipes/_coords.py
A  tools/autoplay/recipes/placement_reject_probe.py
A  reports/PROMPT-1860-autoplay-placement-reject-recipe-mainland-refresh-after-1844.md
```

### Protected files preserved
- `reports/PROMPT-1844-autoplay-vsbot-viewport-click-evidence-audit.md` — NOT deleted ✓
- `tools/autoplay/analyze_evidence_run.py` (PROMPT 1833) — NOT touched ✓

### Registry check
```python
'placement-reject-probe' in REGISTRY  # → True
```

Full registry (13 recipes):
`smoke, idle, add-bot-lobby, lobby-create, class-select, draft-auction-probe,
placement-drag-probe, placement-reject-probe, resolution-observe, game-over-observe,
round-loop, full-game, vs-bot`

### git diff --check
No whitespace errors.

## FF Readiness

Branch is fast-forward ready onto `origin/main@bb90d7c2`:
```
git merge --ff-only integrate/autoplay-placement-reject-recipe-1860
```

## Status

MAINLAND_ENQUEUE ready.

---

1860: AUTOPLAY-PLACEMENT-REJECT-RECIPE-MAINLAND-REFRESH-AFTER-1844: SHIPPED
