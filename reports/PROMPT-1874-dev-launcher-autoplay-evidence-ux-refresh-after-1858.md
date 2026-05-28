# PROMPT 1874 — Dev-Launcher Autoplay Evidence UX Refresh (after PROMPT 1858)

**Date**: 2026-05-28
**Branch**: `wt-1874-dev-launcher-ux-refresh`
**Status**: SHIPPED

## Context

PROMPT 1837 shipped branch `origin/wt-1837-dev-launcher-evidence-ux` with a
fast-lane UX block appended to `tools/dev-launcher/Start-AutoplayVsBot.ps1`.
That branch diverged from main before PROMPTs 1833/1844/1845/1858 landed and
could not be fast-forward merged without deleting those artifacts.

This prompt re-applies the 1837 payload onto a fresh branch from `origin/main`
(at `5c91918d` — PROMPT 1858 tip) using allowlist-only cherry-pick approach.

## Changes Applied

### `tools/dev-launcher/Start-AutoplayVsBot.ps1`

Appended **Section 10: Fast-lane evidence paths + window-size visibility** just
before the final `exit $smokeExit`:

- Prints `Composite dir`, `Autoplay run`, and `Summary JSON` paths in cyan after
  every run, so the operator does not need to scroll back through build output.
- If run succeeded (exit 0, non-dry-run) and `validate_composite_run.py` exists,
  prints the ready-to-paste validate command.
- Prints all four window-config env vars
  (`CCGS_WINDOW_WIDTH`, `CCGS_WINDOW_HEIGHT`, `CCGS_WINDOW_POSITION`,
  `WINIT_X11_SCALE_FACTOR`) so operators can confirm the viewport was correct
  for click-target alignment.
- Emits a yellow WARNING if window size is unset (game opens at OS default;
  bot click targets may be offscreen if < 1280×720).

The section comment references both `PROMPT 1837` (origin) and `PROMPT 1874`
(this refresh) for traceability.

## Validation

| Check | Result |
|-------|--------|
| `git diff --check` | PASS — no whitespace errors |
| PowerShell parse (`Parser::ParseFile`) | PASS — no syntax errors |
| `git diff --name-status origin/main..HEAD` | Only `tools/dev-launcher/Start-AutoplayVsBot.ps1` + this report |

```
git diff --name-status origin/main..HEAD
M       tools/dev-launcher/Start-AutoplayVsBot.ps1
A       reports/PROMPT-1874-dev-launcher-autoplay-evidence-ux-refresh-after-1858.md
```

## Artifacts Preserved (unchanged from main)

- `tools/autoplay/analyze_evidence_run.py` (PROMPT 1833)
- `reports/PROMPT-1844-autoplay-vsbot-viewport-evidence-audit.md` (PROMPT 1844)
- `reports/PROMPT-1845-*.md` (PROMPT 1845)
- `reports/PROMPT-1858-*.md` (PROMPT 1858)
- All QA evidence binaries/images in `production/qa/evidence/`

## Files Modified

| File | Action |
|------|--------|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | Modified — section 10 appended |
| `reports/PROMPT-1874-dev-launcher-autoplay-evidence-ux-refresh-after-1858.md` | Created |

---

1874: DEV-LAUNCHER-AUTOPLAY-EVIDENCE-UX-REFRESH-AFTER-1858: SHIPPED
