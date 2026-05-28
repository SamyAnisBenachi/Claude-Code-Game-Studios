# PROMPT 1837 — DEV-LAUNCHER-AUTOPLAY-EVIDENCE-UX-FAST-LANE

_Generated: 2026-05-28 | Branch: `wt-1837-dev-launcher-evidence-ux` | Commit: d0c28cd7_

---

## Problem

After `Start-AutoplayVsBot.ps1` completes a run (success, failure, or blocked),
the operator's terminal shows an outcome line but **no evidence paths**. The paths
are printed mid-run (section 4 "Evidence dir" and section 7 "Autoplay artifact
dir:") buried above minutes of build and soak output. Finding them requires
scrolling back through the full terminal log.

---

## Inspection Summary

| File | Finding |
|---|---|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | Sections 1–9 complete. Final block (human-readable outcome, lines 415–428) prints PASS/FAIL text but zero evidence paths. |
| `tools/autoplay/Run-AutoplaySmoke.ps1` | Prints `[autoplay-smoke] artifacts under $ArtifactDir` at end — analogous pattern, but Run-AutoplaySmoke is called as a child process; its terminal output doesn't persist in the parent window on all configurations. |
| `docs/autoplay/evidence-operator-guide.md` | §3 documents the layout; §10 documents `validate_composite_run.py`. Both are well-maintained. No doc update needed. |
| PROMPT 1832/1833 conflict check | Neither touches `tools/dev-launcher/Start-AutoplayVsBot.ps1`. Conflict: NONE. |

---

## Change Implemented

**File:** `tools/dev-launcher/Start-AutoplayVsBot.ps1`  
**Scope:** Section 10 appended before `exit $smokeExit` (lines 430–447 post-edit)

The new section always emits (regardless of exit code):

```
---- Evidence paths ----
Composite dir:  <evidenceDir>
Autoplay run:   <autoplayArtifactDir>
Summary JSON:   <compositeSummaryPath>

  Validate: python "<validate_composite_run.py path>" "<evidenceDir>"
------------------------
```

The validate hint is only printed on `exit 0` + non-dry-run and only when
`tools/autoplay/validate_composite_run.py` exists — so it degrades gracefully
on setups where the validator is missing.

---

## Validation

- `git diff --check` — clean (no trailing-whitespace issues)
- PowerShell `Parser::ParseFile` — PARSE OK
- Path allowlist: only `tools/dev-launcher/Start-AutoplayVsBot.ps1` and
  `reports/PROMPT-1837-*.md` touched — within owned scope.
- Conflict check: no edits to `tools/autoplay/` recipe/analyzer files.

---

## Files Changed

| File | Action |
|---|---|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | +17 lines (section 10 — evidence fast-lane) |
| `reports/PROMPT-1837-dev-launcher-autoplay-evidence-ux-fast-lane.md` | new — this report |

---

1837: DEV-LAUNCHER-AUTOPLAY-EVIDENCE-UX-FAST-LANE: SHIPPED
