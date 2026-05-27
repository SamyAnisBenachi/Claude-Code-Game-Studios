# PROMPT 1656 — Composite Evidence Validator Docs Wiring

**Status:** SHIPPED  
**Date:** 2026-05-27  
**Branch:** `prompt-1656-composite-evidence-validator-docs-wiring`

---

## Summary

Wired `tools/autoplay/validate_composite_run.py` (landed in PROMPT 1651) into
operator-facing documentation. No validator code was changed.

---

## Changes

### `docs/autoplay/evidence-operator-guide.md`

Added **Section 10 — Validating a Composite Run**:

- What the validator checks (8 checks listed verbatim from the docstring)
- Quickstart command with a real example path
- Options table: `EVIDENCE_DIR`, `--recipe`, `--strict`
- Exit codes table (0 = PASS, 1 = check failures, 2 = fatal/missing)
- Annotated output example showing PASS-with-warnings vs FAIL
- Explicit note that validator PASS ≠ human sign-off for `AUTOPLAY-VS-BOT-QA-001`
- ASCII diagram showing the launcher → validator relationship

### `docs/autoplay/autoplay-vs-bot-flow.md`

Added **"Validating the evidence directory"** subsection inside the Evidence
Output section:

- One-liner command with the timestamped path pattern
- Exit-code meanings inline
- `--strict` note for local vs. cross-machine use
- Cross-link to `evidence-operator-guide.md § 10`
- Updated footer from PROMPT 1644 → PROMPT 1656

---

## Validation

| Check | Result |
|---|---|
| `git diff --check` | Clean — no whitespace errors |
| `python validate_composite_run.py --help` | Confirmed — matches documented flags exactly |
| PROMPT 1655 owned files untouched | `Run-AutoplaySmoke.ps1` not modified |
| Rust / Cargo files untouched | Confirmed |
| Sprint / session-state files untouched | Confirmed |

---

1656: COMPOSITE-EVIDENCE-VALIDATOR-DOCS-WIRING: SHIPPED
