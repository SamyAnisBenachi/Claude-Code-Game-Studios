# PROMPT 1651 — AUTOPLAY-COMPOSITE-EVIDENCE-VALIDATOR

**Branch:** `work/composite-evidence-validator-1651`
**Source base:** `origin/main@178a8471`
**Date:** 2026-05-27

## Summary

Added a standalone Python CLI validator for autoplay-vs-bot composite evidence
directories produced by `tools/dev-launcher/Start-AutoplayVsBot.ps1` (PROMPT 1644).
Also added 29 unit tests exercising all check paths using synthetic fixture
directories (no GUI, no Cargo).

## Files Added

| File | Purpose |
|------|---------|
| `tools/autoplay/validate_composite_run.py` | Validator CLI |
| `tests/tools/autoplay/test_validate_composite_run.py` | 29 pytest unit tests |

## Validator: `tools/autoplay/validate_composite_run.py`

**Usage:**

```
python tools/autoplay/validate_composite_run.py <evidence-dir>
python tools/autoplay/validate_composite_run.py <evidence-dir> [--recipe NAME] [--strict]
python tools/autoplay/validate_composite_run.py --help
```

**Checks performed:**

1. Evidence directory exists.
2. `composite-summary.json` present and valid JSON.
3. `schema` == `"autoplay_vs_bot_composite_summary_v1"`.
4. `outcome` field present and non-empty string.
5. `live_pass_status` contains `NOT-CLAIMED` — guards against false PASS claims for AUTOPLAY-VS-BOT-QA-001.
6. `autoplay-run-path.txt` present.
7. Path in `autoplay-run-path.txt` matches `autoplay_artifact_dir` in the summary (when both non-empty).
8. If artifact dir exists: `launcher-status.json` present.
9. If `checkpoints.jsonl` exists: expected checkpoint labels for the recipe are a subset of observed labels.

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | One or more checks failed |
| 2 | Evidence directory or summary not found / unparseable (argparse error) |

**`--strict` flag:** treats a missing autoplay artifact dir as a failure (default: warning only, for dry-run or cross-machine scenarios).

**`--recipe NAME`:** override the recipe name for checkpoint validation instead of reading from the summary.

**Checkpoint expectations** (from `RECIPE_REQUIRED_CHECKPOINTS`):

| Recipe | Required checkpoint labels |
|--------|---------------------------|
| `smoke`, `idle` | *(none)* |
| `lobby-create` | `lobby-loaded`, `lobby-confirmed` |
| `add-bot-lobby` | `lobby-loaded`, `bot-added`, `lobby-confirmed` |
| `class-select` | `class-select-loaded`, `class-confirmed` |
| `draft-auction-probe` | `shop-loaded`, `auction-ready` |
| `placement-drag-probe` | `placement-loaded`, `placement-submitted` |
| `resolution-observe` | `resolution-started`, `resolution-complete` |
| `game-over-observe` | `game-over-screen`, `winner-confirmed` |
| `full-game` | `lobby-loaded`, `lobby-confirmed`, `class-select-loaded`, `placement-loaded`, `placement-submitted` |
| `round-loop` | `round-loop-complete` |

Blocked outcomes (`blocked-recipe-guard`, `blocked-human-gui`, `blocked-precondition`, `blocked-soak-timeout`) skip checkpoint validation — no checkpoints are expected.

## Tests: `tests/tools/autoplay/test_validate_composite_run.py`

All 29 tests pass. Test classes:

| Class | Coverage area |
|-------|--------------|
| `TestValidHappyPath` (5) | Valid evidence, dry-run, blocked outcome, recipe override, absent checkpoints.jsonl |
| `TestSummarySchemaChecks` (7) | Wrong schema, missing fields, empty outcome, missing summary, invalid JSON, missing dir |
| `TestLivePassStatusCheck` (3) | Claimed PASS fails, empty fails, NOT-CLAIMED passes |
| `TestRunPathFileChecks` (3) | Missing run-path file, path mismatch, path match |
| `TestArtifactDirChecks` (2) | Missing launcher-status.json, strict mode on absent artifact dir |
| `TestCheckpointChecks` (7) | Missing checkpoint fails, all present passes, extra allowed, per-recipe cases, bad JSONL warns |
| `TestCheckpointRegistry` (2) | All 11 known recipes in registry, values are list[str] |

## Validation Run

```
pytest tests/tools/autoplay/test_validate_composite_run.py -v
29 passed in 0.41s

python tools/autoplay/validate_composite_run.py --help
(help text rendered correctly)

git diff --check
(clean — no whitespace errors)
```

## Constraints Respected

- No Rust / Cargo touched.
- No GUI launched.
- No evidence files mutated.
- Stays within owned scope: `tools/autoplay/`, `tests/tools/autoplay/`, `reports/`.
- Does not touch `docs/autoplay.md`, `production/session-state/`, or sprint files.

1651: AUTOPLAY-COMPOSITE-EVIDENCE-VALIDATOR: SHIPPED
