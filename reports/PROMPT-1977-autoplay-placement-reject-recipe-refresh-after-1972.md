# PROMPT 1977 — Autoplay Placement-Reject Recipe Refresh After 1972

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-placement-reject-recipe-1977`
**Worktree:** `D:\tmp\wt-1977-placement-reject-refresh`
**Base:** `origin/main` @ `32a59256` (PROMPT 1976 — backfill 1861/1914/1941/1964/1968 operator contract reports)
**Commits:** `d49f9b74` (feat), `908e7bc1` (docs chain reports), `<this-commit>` (this report)

## Summary

Reapplied the `placement-reject-probe` autoplay recipe payload cleanly onto
`origin/main` after PROMPT 1972. The previous 1960 branch was rejected because
it was NOT_FF against current `origin/main` — `origin/main` had advanced to
`7b259e91` (PROMPT 1972 signoff-pack reports) then further to `32a59256`
(PROMPT 1976 operator-contract backfill), both past the 1960 base of `2bf3960d`.

## Why 1960 Was Rejected

The `origin/integrate/autoplay-placement-reject-recipe-1960` branch:
- Was NOT fast-forward mergeable onto current `origin/main` (diverged after PROMPT 1957)
- Would have deleted current-main PROMPT 1972 signoff-pack reports and PROMPT 1976
  operator-contract backfill reports if merged wholesale.

## Approach

- Fetched fresh `origin/main` (`32a59256` at start; `7b259e91` at initial worktree
  creation — rebased after discovering main advanced during the session).
- Created dedicated worktree at `D:\tmp\wt-1977-placement-reject-refresh` branched
  from `origin/main` as `integrate/autoplay-placement-reject-recipe-1977`.
- Root checkout (`D:\_DEV\Work\Claude-Code-Game-Studios`) is orchestrator-only — all
  edits confined to dedicated worktree.
- Manually extracted the 3 owned recipe files from
  `origin/integrate/autoplay-placement-reject-recipe-1960` via `git show <ref>:<path>`.
  No wholesale cherry-pick or merge of the stale branch.
- Applied changes: `BOARD_DEEP_CELL` coord in `_coords.py`, `placement_reject_probe`
  import and REGISTRY entry in `__init__.py`, new `placement_reject_probe.py`.
- Carried forward PROMPT-1928, PROMPT-1952, PROMPT-1960 reports as continuity records.
- Rebased onto current `origin/main` when main advanced mid-session (PROMPT 1976 landed).
- No `client/**`, `server/**`, `Cargo.*`, `production/**`, or unrelated report files touched.

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/recipes/_coords.py` | Added `BOARD_DEEP_CELL: FracPoint(0.5, 0.30)` with explanatory comment |
| `tools/autoplay/recipes/__init__.py` | Added `placement_reject_probe` import and REGISTRY entry |
| `tools/autoplay/recipes/placement_reject_probe.py` | New file — full recipe implementation |
| `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md` | Carried forward (continuity) |
| `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md` | Carried forward (continuity) |
| `reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md` | Carried forward (continuity) |
| `reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md` | This report |

## Forbidden Files — Not Touched

Confirmed clean: `client/**`, `server/**`, `Cargo.*`, `production/**`,
`tests/unit/**`. No existing reports deleted.

## Validation Results

| Check | Result |
|-------|--------|
| `git merge-base --is-ancestor origin/main HEAD` | **PASS** (exit 0) |
| `git diff --name-status origin/main..HEAD` — no deletions | **PASS** (only A + M) |
| `git diff --check origin/main..HEAD` | **PASS** (exit 0, no whitespace errors) |
| Python import: `placement-reject-probe` in `REGISTRY` | **PASS** |
| Python import: `BOARD_DEEP_CELL` in `_coords.DEFAULTS` | **PASS** |

All recipes in registry after change:
`add-bot-lobby`, `class-select`, `draft-auction-probe`, `full-game`,
`game-over-observe`, `idle`, `lobby-create`, `placement-drag-probe`,
`placement-reject-probe`, `resolution-observe`, `round-loop`, `smoke`, `vs-bot`

## Recipe Observable Limitations (Preserved from 1928/1952/1960)

- No `autoplay/status` rejection-state signal — rejection confirmed only by
  visual review of the `placement-reject-feedback` checkpoint screenshot.
- `BOARD_DEEP_CELL` default `fy=0.30` is heuristic; if it falls within the
  player's spawn range it will be accepted, not rejected. Override with
  `CCGS_AUTOPLAY_BOARD_DEEP_CELL=fx,fy` for a known-invalid cell.
- No explicit Cancel CTA in the current UI (PROMPT 1468); recovery is
  drag-retarget, not a dedicated cancel button.

## Integration Branch

Push target: `origin/integrate/autoplay-placement-reject-recipe-1977`

FF-merge command (orchestrator):
```
git merge --ff-only origin/integrate/autoplay-placement-reject-recipe-1977
```

---

1977: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1972: READY_FOR_MAINLAND_ENQUEUE
