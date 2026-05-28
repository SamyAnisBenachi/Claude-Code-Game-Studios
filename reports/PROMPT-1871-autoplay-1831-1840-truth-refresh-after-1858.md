# PROMPT 1871 — AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1858

**Date:** 2026-05-28  
**Status:** SHIPPED  
**Branch:** `wt/1871-truth-refresh` from `origin/main @ 5c91918d`  
**Scope:** Reports-only backfill — no source code, tests, or tool changes.

---

## 1. Purpose

PROMPT 1866 produced the correct truth reconciliation for PROMPT 1831/1840 on
branch `origin/wt/1866-report-truth-reconcile`. That branch was built from
`origin/main @ bb90d7c2` and would delete the PROMPT 1845 and 1858 report files
when merged to the current main (`5c91918d`) — because those files landed in
commits after the 1866 branch base.

This prompt re-applies the same correction content cleanly on top of the current
`origin/main` without touching PROMPT 1845/1858 artifacts.

---

## 2. Root Cause: Why 1866 Could Not Be Main-Landed

| Fact | Detail |
|------|--------|
| 1866 base commit | `origin/main @ bb90d7c2` (PROMPT 1844) |
| Current main HEAD | `5c91918d` (PROMPT 1858 backfill) |
| Commits between base and HEAD | `bb90d7c2` → `5c91918d` (2 commits: PROMPT 1845 + 1858 reports) |
| 1866 diff includes deletions of | `reports/PROMPT-1845-*` and `reports/PROMPT-1858-*` |
| Reason for deletions | Branch was built when those files didn't exist; git diff shows them as 1866-deleted |
| Safe action | Re-apply 1866 content (1831 correction + 1866 reconcile report) as new files on top of current main |

---

## 3. What Was Done

### 3.1 Files Written

| File | Action | Status |
|------|--------|--------|
| `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md` | Created (new on main — absent from origin/main before this commit) | ✅ |
| `reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md` | Created (new on main) | ✅ |
| `reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md` | Created (this file) | ✅ |
| `reports/PROMPT-1845-post-1833-evidence-analyzer-focused-verify.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1858-post-1833-evidence-analyzer-verify-report-backfill.md` | **NOT TOUCHED** | ✅ |

### 3.2 Correction Applied to PROMPT-1831 Report

The 1831 report received the correction notice block from PROMPT 1866:
- HTML comment header marking PASS → CONDITIONAL
- Inline correction notes at the Capture Chain and Checkpoints sections
- Final status line updated to `CONDITIONAL`
- Note added attributing the correction to both PROMPT 1866 (origin) and PROMPT 1871 (re-application)

### 3.3 PROMPT-1866 Reconcile Report

The full 1866 reconcile report is preserved verbatim, with an added note in the
header and Section 8 explaining why 1866 was not main-landed and how 1871 resolves it.

---

## 4. Truth Established (Unchanged from PROMPT 1866)

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

**Confirmed working.** The `desktop_bitblt` fallback chain is active. 10–12
distinct pixel hashes prove live, non-frozen content was captured at each phase
transition. This positive finding from PROMPT 1831 is preserved unchanged.

### 4.4 Automated PASS Gate

NOT satisfied. The PROMPT 1844 blocking acceptance criteria (AC-VPT-01 through
AC-VPT-08) are not landed on main. A clean re-run with window-size lock enforced
is required for an automated PASS.

---

## 5. Validation

### 5.1 git diff --check

```
(no whitespace errors)
```

### 5.2 git diff --stat origin/main..HEAD

```
 reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md         | +N (new file)
 reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md           | +N (new file)
 reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md         | +N (new file)
```

No deletions. PROMPT 1845 and 1858 reports are untouched.

### 5.3 No automated PASS claim

Neither the 1831 report nor this report claims run `20260528-090613-Z` as a
clean automated PASS. The word "PASS" appears only in the correction headers
marking it as superseded/downgraded.

---

## 6. Branch and Commit

- **Worktree:** `D:\tmp\wt-1871-truth-refresh`  
- **Branch:** `wt/1871-truth-refresh`  
- **Base:** `origin/main @ 5c91918d`

---

1871: AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1858: SHIPPED
