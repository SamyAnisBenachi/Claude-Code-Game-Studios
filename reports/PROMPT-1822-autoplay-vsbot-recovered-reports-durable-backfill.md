# PROMPT 1822 — AUTOPLAY-VSBOT-RECOVERED-REPORTS-DURABLE-BACKFILL

**Status:** SHIPPED
**Date:** 2026-05-28
**Worktree:** `tmpwt-1822-recovered-reports-backfill`
**Branch:** `prompt-1822-recovered-reports-backfill`
**HEAD at task start:** `3c3aa6d72819247daade62e3bf3c441317b7090c`

---

## 1. Purpose

PROMPT 1821 found that PROMPT 1820's report existed only in the worktree-local
`reports/` directory of `tmpwt-1820-live-screenshot-verify`, not in the repo-root
`reports/`. PROMPT 1821's own report had the same problem — written to
`tmpwt-1821-report-recovery/reports/` instead of root `reports/`.

This task backfills both reports into durable root `reports/` on branch
`prompt-1822-recovered-reports-backfill`.

---

## 2. Source Paths (Read-Only)

| Report | Source path |
|---|---|
| PROMPT 1820 full report | `tmpwt-1820-live-screenshot-verify/reports/PROMPT-1820-autoplay-vsbot-live-screenshot-verify-after-1818-1819.md` |
| PROMPT 1820 summary txt | `tmpwt-1820-live-screenshot-verify/reports/PROMPT-1820-autoplay-vsbot-live-screenshot-verify-after-1818-1819.summary.txt` |
| PROMPT 1821 full report | `tmpwt-1821-report-recovery/reports/PROMPT-1821-autoplay-vsbot-1820-report-recovery-human-gui-pack.md` |

---

## 3. Destination Paths (Written)

| Report | Destination path |
|---|---|
| PROMPT 1820 full report | `reports/PROMPT-1820-autoplay-vsbot-live-screenshot-verify-after-1818-1819.md` |
| PROMPT 1820 summary txt | `reports/PROMPT-1820-autoplay-vsbot-live-screenshot-verify-after-1818-1819.summary.txt` |
| PROMPT 1821 full report | `reports/PROMPT-1821-autoplay-vsbot-1820-report-recovery-human-gui-pack.md` |
| PROMPT 1822 this report | `reports/PROMPT-1822-autoplay-vsbot-recovered-reports-durable-backfill.md` |

All files copied verbatim from source — no content edits, no encoding fixes needed
(source files were clean UTF-8 with no encoding artifacts).

---

## 4. Content Summary

### PROMPT 1820 — NEEDS_HUMAN_GUI

- Verified PROMPT 1818 code (`_frozen_win32_check`, `win32_printwindow=` labels)
  is present in `tools/autoplay/driver.py`.
- Confirmed both existing runs (`20260528-051148-Z`, `20260528-063609-Z`) predate
  the PROMPT 1818 feat commit (`d8b41463`, 07:52 UTC 2026-05-28).
- Game client was not running when 1820 executed; no post-1818 live evidence exists.
- Verdict: NEEDS_HUMAN_GUI — human must run `Run-AutoplaySmoke.ps1 -Recipe vs-bot`.

### PROMPT 1821 — SHIPPED

- Located and summarised the PROMPT 1820 report.
- Documented root cause: worktree-local `reports/` vs. repo-root `reports/`.
- Produced full human GUI operator pack (build pre-flight, launcher command,
  expected log markers, 8-item pass/fail checklist).
- Drafted follow-up PROMPT for post-run log verification.

---

## 5. Validation

```
git diff --check  →  (no whitespace errors)
```

Files written are within the allowed scope (`reports/PROMPT-18*.md`,
`reports/PROMPT-18*.summary.txt`). No source code, test, sprint, or session-state
files were touched.

---

## 6. Summary

| Item | Result |
|---|---|
| PROMPT 1820 report backfilled to root `reports/` | YES |
| PROMPT 1820 summary.txt backfilled to root `reports/` | YES |
| PROMPT 1821 report backfilled to root `reports/` | YES |
| Content preserved verbatim | YES — no edits made |
| Encoding artifacts fixed | N/A — none found |
| Branch | `prompt-1822-recovered-reports-backfill` |
| No source code touched | CONFIRMED |

---

1822: AUTOPLAY-VSBOT-RECOVERED-REPORTS-DURABLE-BACKFILL: SHIPPED
