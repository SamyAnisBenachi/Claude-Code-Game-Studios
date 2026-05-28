# PROMPT 1952 — Autoplay Placement-Reject Recipe Refresh After 1937

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-placement-reject-recipe-1952`
**Worktree:** `D:\tmp\wt-1952-placement-reject`
**Base:** `origin/main` @ `241e33a83bdd99f05dbae3b1fe8fa47bab6f727c` (PROMPT 1950 reports reapply)
**Commit:** rebased onto origin/main after initial commit at b58cdd66

## Summary

Reapplied the `placement-reject-probe` autoplay recipe payload cleanly onto
`origin/main` after PROMPT 1937, without wholesale merge from the rejected 1928
branch. The 1928 branch was rejected due to NOT_FF conflicts, deletion of landed
reports, and stale `tools/dev-launcher/Start-TwoClients.ps1` changes. Only the
owned recipe payload was extracted and re-applied.

## Why 1928 Was Rejected

The `origin/integrate/autoplay-placement-reject-recipe-1928` branch:
- Was NOT fast-forward mergeable onto main (commits diverged after PROMPT 1912)
- Deleted 5 already-landed reports (PROMPT 1838, 1862, 1899, 1932, 1950 series)
- Carried stale changes to `tools/dev-launcher/Start-TwoClients.ps1`

## Approach

- Created dedicated worktree at `D:\tmp\wt-1952-placement-reject` branched from
  `origin/main` as `integrate/autoplay-placement-reject-recipe-1952`.
- Manually extracted only the 3 owned recipe files from the 1928 branch via
  `git show origin/integrate/autoplay-placement-reject-recipe-1928:<path>`.
- Applied changes: added `BOARD_DEEP_CELL` coord to `_coords.py`, added import
  and registry entry to `__init__.py`, created `placement_reject_probe.py`.
- Committed, then rebased onto latest `origin/main` (which had advanced to
  `241e33a8` with PROMPT 1950 report reapply since worktree creation).
- Extracted the PROMPT 1928 report from the rejected branch into this worktree
  as the continuity record (per owned scope).

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/recipes/_coords.py` | Added `BOARD_DEEP_CELL: FracPoint(0.5, 0.30)` with explanatory comment |
| `tools/autoplay/recipes/__init__.py` | Added `placement_reject_probe` import and REGISTRY entry |
| `tools/autoplay/recipes/placement_reject_probe.py` | New file — full recipe implementation |
| `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md` | Carried forward from rejected 1928 branch (continuity) |
| `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md` | This report |

## Forbidden Files — Not Touched

Confirmed clean: `tools/dev-launcher/**`, `client/**`, `production/**`,
`Cargo.toml`, no report deletions.

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

## Recipe Observable Limitations (Preserved from 1928)

These were documented in PROMPT 1928 and remain valid caveats:

- No `autoplay/status` rejection-state signal — rejection can only be confirmed
  by visual review of the `placement-reject-feedback` checkpoint screenshot.
- `BOARD_DEEP_CELL` default `fy=0.30` is heuristic; if it falls within the
  player's spawn range it will be accepted, not rejected. Override with
  `CCGS_AUTOPLAY_BOARD_DEEP_CELL=fx,fy` for a known-invalid cell.
- No explicit Cancel CTA in the current UI (PROMPT 1468); recovery is
  drag-retarget, not a dedicated cancel button.

## Integration Branch

Push target: `origin/integrate/autoplay-placement-reject-recipe-1952`
Push status: PUSHED

FF-merge command (orchestrator):
```
git merge --ff-only origin/integrate/autoplay-placement-reject-recipe-1952
```

---

1952: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1937: READY_FOR_MAINLAND_ENQUEUE
