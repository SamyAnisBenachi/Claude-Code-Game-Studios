# PROMPT 1875 — Autoplay Composite Window-Resize Verdict Refresh After PROMPT 1872

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-composite-window-resize-verdict-1875`
**Base:** `origin/main@2ce3dc6b` (PROMPT 1872)
**Method:** Cherry-pick of `8da9e2e2` (PROMPT 1873 commit) onto latest main

---

## Context

PROMPT 1873 delivered the composite window-resize verdict payload onto
`origin/main@5c91918d` (PROMPT 1858). Since then, origin/main advanced to
`2ce3dc6b` (PROMPT 1872, adding 3 analyzer report backfills). The 1873
integrate branch was no longer FF-ready, and merging it would clobber the
PROMPT 1846/1859/1872 report artifacts.

This prompt creates a fresh branch from `origin/main@2ce3dc6b` and cleanly
cherry-picks the 1873 commit onto it.

---

## Allowlisted Payload (carried from PROMPT 1873)

| File | Change |
|---|---|
| `tools/autoplay/analyze_evidence_run.py` | Window size tracking + win32 quality verdict logic |
| `tools/autoplay/validate_composite_run.py` | Window/capture integrity guard |
| `tests/tools/autoplay/test_window_resize_verdict.py` | 25 tests (all pass) |
| `reports/PROMPT-1850-autoplay-composite-window-resize-verdict-downgrade.md` | Backfill report |
| `reports/PROMPT-1864-autoplay-composite-window-resize-verdict-mainland-refresh-after-1844.md` | Backfill report |
| `reports/PROMPT-1873-autoplay-composite-window-resize-verdict-refresh-after-1858.md` | Prior-refresh report |

---

## New File Added

- `reports/PROMPT-1875-autoplay-composite-window-resize-verdict-refresh-after-1872.md` (this file)

---

## Validation Results

### git diff --check
```
PASSED — no whitespace errors
```

### git diff --name-status origin/main..HEAD
```
A  reports/PROMPT-1850-autoplay-composite-window-resize-verdict-downgrade.md
A  reports/PROMPT-1864-autoplay-composite-window-resize-verdict-mainland-refresh-after-1844.md
A  reports/PROMPT-1873-autoplay-composite-window-resize-verdict-refresh-after-1858.md
A  reports/PROMPT-1875-autoplay-composite-window-resize-verdict-refresh-after-1872.md
A  tests/tools/autoplay/test_window_resize_verdict.py
M  tools/autoplay/analyze_evidence_run.py
M  tools/autoplay/validate_composite_run.py
```
All changes are in allowlisted scope. No forbidden files touched.

### Protected Reports Preserved
| Report | Present |
|---|---|
| PROMPT-1833-autoplay-evidence-distinctness-analyzer.md | ✓ |
| PROMPT-1844-autoplay-vsbot-viewport-click-evidence-audit.md | ✓ |
| PROMPT-1845-post-1833-evidence-analyzer-focused-verify.md | ✓ |
| PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md | ✓ |
| PROMPT-1858-post-1833-evidence-analyzer-verify-report-backfill.md | ✓ |
| PROMPT-1859-autoplay-evidence-analyzer-latest-run-report-backfill.md | ✓ |
| PROMPT-1872-autoplay-evidence-analyzer-latest-run-refresh-after-1858.md | ✓ |

### pytest tests/tools/autoplay/test_window_resize_verdict.py
```
25 passed in 0.28s
```

---

## Outcome

Cherry-pick was clean — no conflicts. All 25 tests pass. Branch
`integrate/autoplay-composite-window-resize-verdict-1875` is FF-ready onto
`origin/main@2ce3dc6b`.

---

1875: AUTOPLAY-COMPOSITE-WINDOW-RESIZE-VERDICT-REFRESH-AFTER-1872: SHIPPED
