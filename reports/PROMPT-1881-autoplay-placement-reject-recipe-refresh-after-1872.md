# PROMPT 1881 — Autoplay Placement-Reject Recipe Refresh After 1872

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-placement-reject-recipe-1881`
**HEAD:** `23c3e901bea20e812c417482bab12626bae9f7fe`
**Base SHA (origin/main):** `2ce3dc6b` (post-PROMPT-1872)
**Worker:** Claude Sonnet 4.6

## Context

PROMPT 1860 shipped the placement-reject-probe recipe on branch
`integrate/autoplay-placement-reject-recipe-1860` but that branch was based
on `origin/main@bb90d7c2` (pre-PROMPT 1845/1846/1858/1859/1872). A direct
fast-forward onto current `origin/main@2ce3dc6b` would have deleted those
report commits. PROMPT 1860 was NOT main-landed.

This worker creates a fresh worktree branch from latest `origin/main@2ce3dc6b`
and reapplies only the placement-reject recipe payload (allowlist-scoped).

## Files Carried

| File | Status |
|------|--------|
| `tools/autoplay/recipes/__init__.py` | M — `placement_reject_probe` import + REGISTRY entry |
| `tools/autoplay/recipes/_coords.py` | M — `BOARD_DEEP_CELL` coord (0.5, 0.30) |
| `tools/autoplay/recipes/placement_reject_probe.py` | A — new recipe (121 lines) |
| `reports/PROMPT-1849-autoplay-placement-reject-recovery-recipe-integration-refresh.md` | A — 1849 integration report |
| `reports/PROMPT-1860-autoplay-placement-reject-recipe-mainland-refresh-after-1844.md` | A — 1860 mainland refresh report |

## Validation

| Check | Result |
|---|---|
| `git diff --check origin/main..HEAD` | PASS — no whitespace errors |
| `git diff --name-status origin/main..HEAD` | PASS — 5 owned files (3A, 2M); no deletions |
| PROMPT 1845/1846/1858/1859/1872 reports preserved | PASS — not in diff |
| `tools/autoplay/driver.py` untouched | PASS |
| `tools/autoplay/analyze_evidence_run.py` untouched | PASS |
| Python REGISTRY check | PASS — 13 recipes; `placement-reject-probe` present |
| Push to origin | PASS — `integrate/autoplay-placement-reject-recipe-1881` |

### Registry (13 recipes)
```
add-bot-lobby, class-select, draft-auction-probe, full-game, game-over-observe,
idle, lobby-create, placement-drag-probe, placement-reject-probe,
resolution-observe, round-loop, smoke, vs-bot
```

## FF Readiness

Branch is fast-forward ready onto current `origin/main@2ce3dc6b`:
```
git merge --ff-only integrate/autoplay-placement-reject-recipe-1881
```

**Recommendation:** MAINLAND_ENQUEUE — safe to fast-forward merge.

---

1881: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1872: SHIPPED
