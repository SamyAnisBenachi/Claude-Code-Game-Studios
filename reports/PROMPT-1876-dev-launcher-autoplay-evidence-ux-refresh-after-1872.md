# PROMPT 1876 — Dev-Launcher Autoplay Evidence UX Refresh (after 1872)

**Date:** 2026-05-28  
**Branch:** `wt-1876-dev-launcher-ux-refresh`  
**Base:** `origin/main` @ `2ce3dc6b` (includes PROMPT 1872 artifacts)  
**Status:** SHIPPED

---

## Summary

PROMPT 1874 added section 10 (fast-lane evidence paths + window-size visibility) to
`tools/dev-launcher/Start-AutoplayVsBot.ps1`, but its branch (`wt-1874-dev-launcher-ux-refresh`)
was based on `origin/main@5c91918d` (pre-1872). Merging it would have deleted the
PROMPT 1846, PROMPT 1859, and PROMPT 1872 report artifacts.

This prompt (1876) recreates the same payload on a fresh branch from latest
`origin/main@2ce3dc6b`, preserving all prior report artifacts.

---

## Changes Applied

### `tools/dev-launcher/Start-AutoplayVsBot.ps1`

Appended **section 10** after the existing smoke-exit block and before `exit $smokeExit`:

- **Evidence paths block**: prints `$evidenceDir`, `$autoplayArtifactDir`, `$compositeSummaryPath`.
  Also prints the `validate_composite_run.py` command hint on clean runs (non-dry-run, exit 0).
- **Window config block**: surfaces `CCGS_WINDOW_WIDTH`, `CCGS_WINDOW_HEIGHT`,
  `CCGS_WINDOW_POSITION`, `WINIT_X11_SCALE_FACTOR` so operators can confirm viewport
  was correct without scrolling back through build output. Warns if width/height unset.

No other files modified.

---

## Validation

| Check | Result |
|-------|--------|
| `git diff --check` | CLEAN (no whitespace errors) |
| PowerShell `[Parser]::ParseFile` | PARSE OK |
| `git diff --name-status origin/main..HEAD` | Only `tools/dev-launcher/Start-AutoplayVsBot.ps1` + this report |
| PROMPT 1846 report present | ✓ `reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md` |
| PROMPT 1859 report present | ✓ `reports/PROMPT-1859-autoplay-evidence-analyzer-latest-run-report-backfill.md` |
| PROMPT 1872 report present | ✓ `reports/PROMPT-1872-autoplay-evidence-analyzer-latest-run-refresh-after-1858.md` |
| PROMPT 1833 tooling present | ✓ `tools/autoplay/analyze_evidence_run.py` |
| PROMPT 1844 report present | ✓ `reports/PROMPT-1844-autoplay-vsbot-viewport-click-target-audit.md` |

---

## Diff Summary

```
M  tools/dev-launcher/Start-AutoplayVsBot.ps1   (+35 lines, section 10 appended)
A  reports/PROMPT-1876-dev-launcher-autoplay-evidence-ux-refresh-after-1872.md
```

No deletions. No Bevy/Rust source touched. No sprint-status or session-state files touched.

---

1876: DEV-LAUNCHER-AUTOPLAY-EVIDENCE-UX-REFRESH-AFTER-1872: SHIPPED
