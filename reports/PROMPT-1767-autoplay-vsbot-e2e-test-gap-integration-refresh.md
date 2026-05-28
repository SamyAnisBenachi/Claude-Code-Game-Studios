# PROMPT 1767 — AUTOPLAY-VSBOT-E2E-TEST-GAP-INTEGRATION-REFRESH

Generated: 2026-05-28

## Summary

Integration refresh of PROMPT 1764 (vs-bot checkpoint validation) onto latest `origin/main`.
Cherry-pick landed cleanly with no conflicts; all 111 Python tests pass.

## Branch / Commit

| Field | Value |
|---|---|
| Integration branch | `wt/1767-vsbot-integration` |
| Integration commit | `d503a3ba` |
| Base `origin/main` SHA | `7ca41fc49dbfc8f0175b87818860c0c5bebce718` |
| Source commit (1764) | `0f57014d` |
| Cherry-pick result | Clean — no conflicts |
| FF-ready | YES (1 commit ahead of origin/main) |

## Changes Integrated

### `tools/autoplay/validate_composite_run.py`
- Added `vs-bot` entry to `RECIPE_REQUIRED_CHECKPOINTS` requiring:
  `lobby-loaded`, `bot-added`, `lobby-confirmed`, `class-select-loaded`,
  `placement-loaded`, `placement-submitted`

### `tests/tools/autoplay/test_validate_composite_run.py`
- Added `vs-bot` to `TestCheckpointRegistry.KNOWN_RECIPES`
- Added `TestCheckpointRegistry.test_vs_bot_has_bot_added_checkpoint`
- Added `TestVsBotCheckpointValidation` (4 tests):
  - `test_validate_vs_bot_required_checkpoints_present_passes`
  - `test_validate_vs_bot_missing_bot_added_checkpoint_fails`
  - `test_validate_vs_bot_missing_placement_submitted_fails`
  - `test_validate_vs_bot_blocked_outcome_skips_checkpoints`

## Validation

### `git diff --check`
```
DIFF CHECK CLEAN
```

### Python Tests
```
tests/tools/autoplay/test_validate_composite_run.py  — 34 passed
tests/tools/autoplay/test_recipe_static.py           — 77 passed
Total: 111 passed in 0.71s
```

### Push Status
```
* [new branch] wt/1767-vsbot-integration -> wt/1767-vsbot-integration
```

## Mainland Enqueue

Branch `wt/1767-vsbot-integration` is strictly FF-ready for `origin/main`.
Enqueue via normal merge/rebase flow — no squash needed (1 clean commit).

---

1767: AUTOPLAY-VSBOT-E2E-TEST-GAP-INTEGRATION-REFRESH: SHIPPED
