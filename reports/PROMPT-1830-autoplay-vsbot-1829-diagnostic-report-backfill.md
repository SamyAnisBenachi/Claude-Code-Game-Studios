# PROMPT 1830 — AUTOPLAY-VSBOT-1829-DIAGNOSTIC-REPORT-BACKFILL

**Status:** SHIPPED  
**Date:** 2026-05-28  
**Branch:** `wt-1830-report-backfill`  
**Base:** `origin/main@0399ba0b`

---

## 1. Objective

Backfill the PROMPT 1829 diagnostic report from its worktree into the root
`reports/` directory, and document the key diagnostic conclusion for the
orchestrator record.

---

## 2. Files Written

| File | Action |
|---|---|
| `reports/PROMPT-1829-autoplay-vsbot-stale-win32-label-root-cause.md` | Backfilled verbatim from `tmpwt-1829-stale-win32-label-root-cause/reports/` |
| `reports/PROMPT-1830-autoplay-vsbot-1829-diagnostic-report-backfill.md` | This report |

No source files were modified. No QA evidence, sprint state, or Cargo files
were touched.

---

## 3. Key Diagnostic Conclusion (PROMPT 1829 Summary)

The stale `win32_capture=OK` labels in evidence run `20260528-063609-Z` are
expected and not a bug. The run executed at 06:36:09 UTC on 2026-05-28, which
is **76 minutes before** the PROMPT 1818 commit (`d8b41463`, merged 07:52:42
UTC) that renamed the label to `win32_printwindow=OK` and introduced
frozen-frame detection.

At current HEAD (`ae06e9b1`):
- `tools/autoplay/driver.py` uses the correct post-1818 labels and frozen
  fallback logic.
- Script path chain is clean — no stale copies, no secondary checkout.
- Stale-pyc guard is double-layered (both launcher and smoke scripts clear
  `__pycache__` before launch).

No source repair was needed. The outstanding gap is that no **post-1818** live
vs-bot evidence run has been captured yet. The next step is a fresh
`Start-AutoplayVsBot.ps1 -Recipe vs-bot` run to produce post-1818 evidence and
close AUTOPLAY-VS-BOT-QA-001.

---

## 4. Path Allowlist Verification

- `reports/PROMPT-1829-autoplay-vsbot-stale-win32-label-root-cause.md` — allowed
- `reports/PROMPT-1830-autoplay-vsbot-1829-diagnostic-report-backfill.md` — allowed
- No other files written.

`git diff --check` clean (no trailing whitespace or conflict markers).

---

1830: AUTOPLAY-VSBOT-1829-DIAGNOSTIC-REPORT-BACKFILL: SHIPPED
