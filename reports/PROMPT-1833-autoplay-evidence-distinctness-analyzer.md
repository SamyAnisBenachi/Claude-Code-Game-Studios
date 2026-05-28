# PROMPT 1833 — Autoplay Evidence Distinctness Analyzer

**Date**: 2026-05-28
**Branch**: wt-1833-evidence-analyzer
**Status**: SHIPPED

## Summary

Added `tools/autoplay/analyze_evidence_run.py` — a lightweight, read-only Python
analyzer for a single autoplay-run evidence directory. Companion test suite added at
`tests/tools/autoplay/test_analyze_evidence_run.py` (21 tests, all passing).

## Deliverables

| File | Action | Purpose |
|---|---|---|
| `tools/autoplay/analyze_evidence_run.py` | NEW | Reusable analyzer script |
| `tests/tools/autoplay/test_analyze_evidence_run.py` | NEW | 21 focused pytest tests |

## Analyzer Capabilities

The script accepts a path to an autoplay-run directory (e.g.
`production/qa/evidence/autoplay-runs/20260528-063609-Z`) and produces:

| Section | Source | What it extracts |
|---|---|---|
| Launcher / Driver | `launcher-status.json` | `outcome`, `driver_exit_code`, `client_exit_code` |
| Capture labels | `driver.log` | Families seen: `win32_capture`, `win32_printwindow`, `desktop_bitblt`; FROZEN line count |
| Screenshot counts | filesystem | Root `win32_tick_*.png` count + `screenshots/*.png` count |
| pixel_hash | `driver.log` | Total captures, distinct count, distinct values, frozen-pattern flag |
| Verdict | derived | PASS / PARTIAL / FAIL / NEEDS_HUMAN_GUI + one-line reason |

### Verdict Logic

| Verdict | Condition |
|---|---|
| `NEEDS_HUMAN_GUI` | `launcher_outcome` in `{blocked-human-gui, blocked-precondition, blocked-recipe-guard, blocked-soak-timeout}` |
| `FAIL` | `driver_exit_code != 0`, OR zero total screenshots |
| `PARTIAL` | All pixel_hashes identical (frozen renderer), OR FROZEN label in log |
| `PASS` | None of the above |

### Exit Codes

| Code | Meaning |
|---|---|
| 0 | PASS |
| 1 | PARTIAL |
| 2 | FAIL |
| 3 | NEEDS_HUMAN_GUI |
| 4 | Cannot analyse (directory not found) |

### Output Modes

- Default: human-readable summary with labelled sections
- `--json`: machine-readable JSON with all fields

## Smoke Test Against Real Evidence

Run against `production/qa/evidence/autoplay-runs/20260528-063609-Z`:

```
VERDICT: PARTIAL
REASON : all 15 pixel_hash captures share the same value (0x26207c4c) — renderer may be frozen
```

Correctly identifies the frozen win32 renderer pattern first diagnosed in PROMPT 1829.

## Test Results

```
21 passed in 0.30s
```

All 21 tests pass covering: PASS, PARTIAL (frozen hashes, FROZEN label), FAIL (exit code,
no screenshots, missing dir), NEEDS_HUMAN_GUI (all blocked outcomes), missing-file
resilience, pixel_hash deduplication, JSON output structure, human output smoke.

## Path Allowlist Review

- `tools/autoplay/analyze_evidence_run.py` — within allowed scope
- `tests/tools/autoplay/test_analyze_evidence_run.py` — within allowed scope
- `reports/PROMPT-1833-autoplay-evidence-distinctness-analyzer.md` — within allowed scope

No QA evidence files mutated. No Rust source touched. No Cargo invoked.

---

1833: AUTOPLAY-EVIDENCE-DISTINCTNESS-ANALYZER: SHIPPED
