# PROMPT 1871 — AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1858

**Date:** 2026-05-28  
**Status:** SUPERSEDED by PROMPT 1877  
**Branch:** `wt/1871-truth-refresh` from `origin/main @ 5c91918d`  
**Superseded by:** PROMPT 1877 — same correction re-applied on top of `origin/main @ 2ce3dc6b`  
**Scope:** Reports-only backfill — no source code, tests, or tool changes.

---

## 1. Purpose

PROMPT 1866 produced the correct truth reconciliation for PROMPT 1831/1840 on
branch `origin/wt/1866-report-truth-reconcile`. That branch was built from
`origin/main @ bb90d7c2` and would delete the PROMPT 1845 and 1858 report files
when merged to the current main (`5c91918d`) — because those files landed in
commits after the 1866 branch base.

This prompt re-applied the same correction content cleanly on top of the then-
current `origin/main` without touching PROMPT 1845/1858 artifacts.

---

## 2. Root Cause: Why 1866 Could Not Be Main-Landed

| Fact | Detail |
|------|--------|
| 1866 base commit | `origin/main @ bb90d7c2` (PROMPT 1844) |
| Current main HEAD at time of 1871 | `5c91918d` (PROMPT 1858 backfill) |
| Commits between base and HEAD | `bb90d7c2` → `5c91918d` (2 commits: PROMPT 1845 + 1858 reports) |
| 1866 diff includes deletions of | `reports/PROMPT-1845-*` and `reports/PROMPT-1858-*` |
| Reason for deletions | Branch was built when those files didn't exist; git diff shows them as 1866-deleted |
| Safe action | Re-apply 1866 content (1831 correction + 1866 reconcile report) as new files on top of current main |

---

## 3. Why 1871 Was Also Not Main-Landed

PROMPT 1871 landed on `origin/wt/1871-truth-refresh` from base `5c91918d`.
After PROMPT 1872 landed on origin/main (`2ce3dc6b`), the 1871 branch could not
be FF-merged because it would delete the PROMPT 1846, 1859, and 1872 report
files (which landed after the 1871 branch base). PROMPT 1877 is the next
re-application, on top of `2ce3dc6b`.

---

## 4. Truth Established (Unchanged — see PROMPT 1877 for final record)

### 4.1 NEEDS_HUMAN_GUI (PROMPT 1831 relay)

**Correctly explained by PROMPT 1840 — preserved unchanged:**  
`NEEDS_HUMAN_GUI` was a transient preflight gate state while the launcher waited
for the operator to open a live game window. It is not a final failure verdict.
The operator launched the game; the run proceeded. This explanation stands.

### 4.2 Run 20260528-090613-Z

| Dimension | Original claim | Corrected verdict |
|-----------|---------------|-------------------|
| Automated PASS | PASS ✅ | **CONDITIONAL — NOT an automated PASS** |
| Capture quality | "10 distinct hashes" | Hashes came entirely from `desktop_bitblt` fallback; `win32_printwindow` was frozen 11/11 captures |
| Click accuracy | Implied correct | **Invalid for ticks 128–260** — mid-run DWM resize (720→505→1076) invalidated baked click coords |
| Checkpoint validity | 15/15 = verified | Time-based only — does not verify click accuracy or UI state |
| Analyzer verdict | Not checked | PARTIAL (FROZEN label 11 times) |

**Corrected run verdict:** `CONDITIONAL — strongest available human-review evidence,
but NOT an automated PASS for AUTOPLAY-VS-BOT-QA-001.`

### 4.3 Post-1818 Capture Chain

**Confirmed working.** See PROMPT 1877 for the definitive record.

### 4.4 Automated PASS Gate

NOT satisfied. See PROMPT 1877 for current status.

---

## 5. Branch and Commit

- **Worktree (at time of 1871 execution):** `D:\tmp\wt-1871-truth-refresh`  
- **Branch:** `wt/1871-truth-refresh`  
- **Base:** `origin/main @ 5c91918d`  
- **Not main-landed** — superseded by PROMPT 1877.

---

1871: AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1858: SUPERSEDED (PROMPT 1877 is the final application)
