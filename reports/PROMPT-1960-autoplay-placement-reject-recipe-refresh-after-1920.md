# PROMPT 1960 — Autoplay Placement-Reject Recipe Refresh After 1920

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-placement-reject-recipe-1960`
**Worktree:** `D:\tmp\wt-1960-placement-reject`
**Base:** `origin/main` @ `2bf3960d` (PROMPT 1957 — krosmaga auction tier-border asset binding refresh after 1920)

## Summary

Reapplied the `placement-reject-probe` autoplay recipe payload cleanly onto
`origin/main` after PROMPT 1920. The previous 1952 branch was rejected because
it was not strict-FF onto the post-1920 main (1952 was built from a pre-1920
base), deleted already-landed PROMPT 1868/1920 card-inspect reports, and
carried drift in `client/src/ui/card_inspect.rs` and
`client/src/ui/hand/inspect.rs`.

## Why 1952 Was Rejected

The `origin/integrate/autoplay-placement-reject-recipe-1952` branch:
- Was NOT fast-forward mergeable onto post-1920 `origin/main` (diverged after PROMPT 1937)
- Deleted already-landed PROMPT 1868/1920 card-inspect reports
- Carried stale changes to `client/src/ui/card_inspect.rs` and `client/src/ui/hand/inspect.rs`

## Approach

- Created dedicated worktree at `D:\tmp\wt-1960-placement-reject` branched from
  `origin/main` (2bf3960d) as `integrate/autoplay-placement-reject-recipe-1960`.
- Confirmed root checkout is shared/orchestrator-only — all edits in worktree only.
- Manually extracted the 3 owned recipe files from commit `e1b35071` (1952 feat commit)
  via `git show e1b35071:<path>` — no wholesale branch merge.
- Applied changes: `BOARD_DEEP_CELL` coord in `_coords.py`, `placement_reject_probe`
  import and REGISTRY entry in `__init__.py`, new `placement_reject_probe.py`.
- Extracted PROMPT-1928 and PROMPT-1952 reports from old commits as continuity records.
- No `client/src/**` files touched. No existing reports deleted.

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/recipes/_coords.py` | Added `BOARD_DEEP_CELL: FracPoint(0.5, 0.30)` with explanatory comment |
| `tools/autoplay/recipes/__init__.py` | Added `placement_reject_probe` import and REGISTRY entry |
| `tools/autoplay/recipes/placement_reject_probe.py` | New file — full recipe implementation |
| `reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md` | Carried forward (continuity) |
| `reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md` | Carried forward (continuity) |
| `reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md` | This report |

## Forbidden Files — Not Touched

Confirmed clean: `client/src/**`, `tools/dev-launcher/**`, `production/**`,
`Cargo.toml`. No report deletions from main.

## Validation Results

| Check | Result |
|-------|--------|
| `git merge-base --is-ancestor origin/main HEAD` (post-commit) | PASS |
| `git diff --check` | PASS (no trailing whitespace errors) |
| Path allowlist: only owned `tools/autoplay/recipes/` files + owned reports | PASS |
| No `client/src/**` changes | PASS |
| No deletions of current-main reports | PASS |
| pytest/static recipe import check | DEFERRED — no Python env in this shell; recipe structure identical to 1952 which passed |

Recipe registry after change includes:
`add-bot-lobby`, `class-select`, `draft-auction-probe`, `full-game`,
`game-over-observe`, `idle`, `lobby-create`, `placement-drag-probe`,
`placement-reject-probe`, `resolution-observe`, `round-loop`, `smoke`, `vs-bot`

## Recipe Observable Limitations (Preserved from 1928/1952)

- No `autoplay/status` rejection-state signal — rejection confirmed only by
  visual review of the `placement-reject-feedback` checkpoint screenshot.
- `BOARD_DEEP_CELL` default `fy=0.30` is heuristic; if it falls within the
  player's spawn range it will be accepted, not rejected. Override with
  `CCGS_AUTOPLAY_BOARD_DEEP_CELL=fx,fy` for a known-invalid cell.
- No explicit Cancel CTA in the current UI (PROMPT 1468); recovery is
  drag-retarget, not a dedicated cancel button.

## Integration Branch

Push target: `origin/integrate/autoplay-placement-reject-recipe-1960`
Push status: PUSHED

FF-merge command (orchestrator):
```
git merge --ff-only origin/integrate/autoplay-placement-reject-recipe-1960
```

---

1960: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-AFTER-1920: READY_FOR_MAINLAND_ENQUEUE
