# PROMPT 1764 — AUTOPLAY-VSBOT-E2E-TEST-GAP-IMPLEMENTATION-MAP

**Branch:** `wt/1764-vsbot-test-gap`
**Commit:** `0f57014d`
**Date:** 2026-05-28

---

## Audit Summary

### Existing test coverage

| Path | What it covers | Status |
|---|---|---|
| `tests/tools/autoplay/test_recipe_static.py` | Registry completeness (12 recipes), method allowlist, env-gate blocking, checkpoint contracts for all recipes incl. vs-bot | PASS (111 total after this change) |
| `tests/tools/autoplay/test_validate_composite_run.py` | `validate_composite_run.py` schema, outcome, live-pass-status, run-path, artifact dir, checkpoint validation per recipe | GAP: `vs-bot` missing |
| `tools/dev-launcher/BuildProvenance.Tests.ps1` | Pester tests for `BuildProvenance.psm1` pure builder | Unrelated; not a gap |
| `tests/integration/autoplay*/` | **MISSING** — no Rust integration tests at this path | Advisory gap (see below) |

### Gap identified

`RECIPE_REQUIRED_CHECKPOINTS` in `tools/autoplay/validate_composite_run.py` (11 entries) was missing `vs-bot`. A composite run producing `"recipe": "vs-bot"` in `composite-summary.json` bypassed all checkpoint validation — the validator would declare PASS even with an empty `checkpoints.jsonl`.

`TestCheckpointRegistry.KNOWN_RECIPES` in `test_validate_composite_run.py` mirrored the same omission, so no test caught it.

### Advisory gap (not implemented here)

`tests/integration/autoplay/` and `tests/integration/autoplay_vs_bot/` are empty. These would be Rust ECS-level integration tests. They are **not appropriate for this prompt** because:
- They require `cargo test` (broad Cargo, forbidden by task scope).
- Live GUI behavior is operator-gated by design; no headless Rust test can replace that.
- No obvious cheap static Rust test exists that isn't already covered by the Python layer.

These remain as advisory follow-up (see deferred work register).

---

## Implementation

### Files changed

**`tools/autoplay/validate_composite_run.py`**
Added `vs-bot` to `RECIPE_REQUIRED_CHECKPOINTS`:
```python
"vs-bot": ["lobby-loaded", "bot-added", "lobby-confirmed",
            "class-select-loaded",
            "placement-loaded", "placement-submitted"],
```
Rationale: same phase-bookend pattern as `full-game`; `bot-added` is the key discriminator that proves `add-bot-lobby` ran instead of `lobby-create`.

**`tests/tools/autoplay/test_validate_composite_run.py`**
- Added `"vs-bot"` to `TestCheckpointRegistry.KNOWN_RECIPES` (causes `test_all_known_recipes_in_registry` to catch future omissions).
- Added `TestCheckpointRegistry.test_validate_composite_run_vs_bot_has_bot_added_checkpoint` — asserts `bot-added` is in the registry entry.
- Added `TestVsBotCheckpointValidation` class with 4 tests:
  1. `test_validate_vs_bot_required_checkpoints_present_passes` — happy path
  2. `test_validate_vs_bot_missing_bot_added_checkpoint_fails` — regression for the gap
  3. `test_validate_vs_bot_missing_placement_submitted_fails` — placement gate
  4. `test_validate_vs_bot_blocked_outcome_skips_checkpoints` — blocked run exempt

### Validation result

```
pytest tests/tools/autoplay/test_validate_composite_run.py tests/tools/autoplay/test_recipe_static.py -v
111 passed in 0.88s
```

No Cargo. No GUI. No live run.

---

## Deferred follow-up (advisory, not blocking)

| Item | Reason deferred |
|---|---|
| `tests/integration/autoplay/` Rust tests | Requires broad `cargo test`; no cheap headless static surface exists; live behavior is operator-gated |
| PowerShell `-DryRun` Pester tests for `Start-AutoplayVsBot.ps1` | `Start-AutoplayVsBot.ps1` logic is already validated end-to-end by the live smoke PASS (PROMPT 1757); Pester infrastructure for this script would require a separate setup story |

---

1764: AUTOPLAY-VSBOT-E2E-TEST-GAP-IMPLEMENTATION-MAP: SHIPPED
