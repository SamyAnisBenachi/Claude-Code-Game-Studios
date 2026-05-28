# PROMPT 1928 — Autoplay Placement-Reject Recipe Refresh After 1912

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-placement-reject-recipe-1928`
**Worktree:** `D:\tmp\wt-1928-placement-reject-refresh`
**Base:** `origin/main` @ `1c945fd2` (PROMPT 1912 whitespace cleanup)
**Commit:** `60b866f4`

## Summary

Refreshed the `placement-reject-probe` recipe payload from
`origin/integrate/autoplay-placement-reject-recipe-1881` onto current
`origin/main` after PROMPT 1912, without wholesale merge. Applied only the
owned files in strict cherry-apply fashion to avoid carrying stale deletions
or reversions from the 1881 branch.

## Approach

- Created dedicated worktree at `D:\tmp\wt-1928-placement-reject-refresh`
  branched from `origin/main` as `integrate/autoplay-placement-reject-recipe-1928`.
- Inspected `git diff origin/main...origin/integrate/autoplay-placement-reject-recipe-1881
  --name-status` to confirm no forbidden files in the source branch diff.
- Applied payload manually (no cherry-pick, no merge) to keep only owned scope.

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/recipes/_coords.py` | Added `BOARD_DEEP_CELL: FracPoint(0.5, 0.30)` entry with explanatory comment |
| `tools/autoplay/recipes/__init__.py` | Added `placement_reject_probe` import and registry entry |
| `tools/autoplay/recipes/placement_reject_probe.py` | New file — full recipe implementation |

## Forbidden Files — Not Touched

Confirmed clean: `client/src/autoplay.rs`, `tools/autoplay/driver.py`,
`Run-AutoplaySmoke.ps1`, `Start-AutoplayVsBot.ps1` — all untouched.

## Validation Results

| Check | Result |
|-------|--------|
| `git merge-base --is-ancestor origin/main HEAD` | PASS (exit 0) |
| `git diff --check origin/main..HEAD` | PASS (exit 0, no trailing whitespace) |
| `git diff --name-status origin/main..HEAD` — no deletes | PASS (only M + A) |
| Python import: `placement-reject-probe` in `REGISTRY` | PASS |
| Python import: `BOARD_DEEP_CELL` in `DEFAULTS` | PASS |

All recipes visible after change:
`add-bot-lobby`, `class-select`, `draft-auction-probe`, `full-game`,
`game-over-observe`, `idle`, `lobby-create`, `placement-drag-probe`,
`placement-reject-probe`, `resolution-observe`, `round-loop`, `smoke`, `vs-bot`

## PROMPT 1912 Preservation

Confirmed origin/main base includes:
- `e02d132f` — PROMPT 1912: AC-VPT-01 window-size default repair
- `fe2a9e88` — PROMPT 1912: refresh report
- `1c945fd2` — PROMPT 1912: whitespace cleanup

These are untouched; branch is strictly FF-ready on top of them.

## Integration Branch

Push target: `origin/integrate/autoplay-placement-reject-recipe-1928`

Push command (orchestrator):
```
git push origin integrate/autoplay-placement-reject-recipe-1928
```

---

1928: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1912: SHIPPED
