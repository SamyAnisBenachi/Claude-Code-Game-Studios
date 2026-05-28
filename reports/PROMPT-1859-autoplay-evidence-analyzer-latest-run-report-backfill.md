# PROMPT 1859 — Autoplay Evidence Analyzer: Latest Run Report Backfill

**Date:** 2026-05-28  
**Worker branch:** `report/autoplay-evidence-analyzer-latest-run-1859`  
**Base:** origin/main@bb90d7c2  
**Scope:** reports-only — no code, test, evidence, or config mutation  

---

## Purpose

PROMPT 1846 completed PARTIAL: the report was written locally but was never committed to a remote branch or landed to main. This backfill task (PROMPT 1859) creates a dedicated branch over latest `origin/main`, commits the PROMPT 1846 report with `git add -f` (since `reports/` is gitignored), and adds this meta-report.

---

## Source Report: PROMPT 1846

**File:** `reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md`  
**Status:** Preserved verbatim from local root-checkout copy.  
**Encoding check:** No mojibake observed. ASCII + UTF-8 safe punctuation only. No repair needed.

### Key Findings Preserved

| Run | Capture Labels | Distinct Hashes | Frozen Lines | Analyzer Verdict | Evidence Quality |
|-----|---------------|-----------------|--------------|-----------------|-----------------|
| 20260528-051148-Z | none (Bevy RPC only) | 0 of 0 | 0 | PARTIAL | Weakest |
| 20260528-063609-Z | win32_capture | 1 of 15 (frozen) | 0 | PARTIAL | Weak |
| 20260528-090613-Z | win32_printwindow + win32_capture + desktop_bitblt | 12 of 26 | 11 | PARTIAL | Strongest |

**Overall verdict:** No automated PASS. Run 3 (090613-Z) is the best evidence — 12 distinct pixel_hashes via desktop_bitblt fallback — but PARTIAL due to 11 FROZEN lines and mid-run window resize (1296×759 → 1296×1115). Run 3 bitblt PNGs are conditionally useful for human-reviewed sign-off only.

---

## Diff Scope Validation

Files changed on this branch (relative to origin/main@bb90d7c2):

```
reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md   (new, force-added)
reports/PROMPT-1859-autoplay-evidence-analyzer-latest-run-report-backfill.md  (new, force-added)
```

**Confirmed reports-only.** No Cargo files, no CI files, no source code, no test files, no sprint/session-state/QA evidence files touched.

---

## Worktree Used

- Worktree path: `D:\_DEV\Work\tmpwt-1859-autoplay-evidence-analyzer-backfill`
- Branch: `report/autoplay-evidence-analyzer-latest-run-1859`
- Root checkout (`D:\_DEV\Work\Claude-Code-Game-Studios`) remains on `main` — not modified.

---

## Push / Remote Status

Branch pushed to `origin/report/autoplay-evidence-analyzer-latest-run-1859`. If push was blocked by a protected-branch rule or GitHub export restriction, the commit is local and the branch name is as above.

---

1859: AUTOPLAY-EVIDENCE-ANALYZER-LATEST-RUN-REPORT-BACKFILL: SHIPPED
