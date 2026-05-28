# PROMPT 1956 — AUTOPLAY-VSBOT-1841-SIGNOFF-PACK-REPORT-REFRESH-AFTER-1920

**Date:** 2026-05-28
**Branch:** report/autoplay-vsbot-1841-signoff-pack-1956 (from origin/main @ 1c4981a6)
**Refreshes:** PROMPT 1946 (report/autoplay-vsbot-1841-signoff-pack-1946)
**Source chain:** PROMPT 1841 → 1889 → 1911 → 1946 → 1956

> **Scope:** Report-only refresh. No source edits. No QA evidence mutation.
> No Cargo build. Path allowlist: `reports/` only.

---

## Purpose

PROMPT 1946 branch (`origin/report/autoplay-vsbot-1841-signoff-pack-1946`) was
found stale: not strict-FF against current origin/main, showing main drift in
`client/src/ui/card_inspect.rs` and `client/src/ui/hand/inspect.rs`. This prompt
rebuilds the same report payload cleanly on current origin/main@1c4981a6.

No changes to the underlying evidence assessment. All original verdicts, caveats,
and the BLOCKING C0 human-review gate are preserved unchanged.

---

## Rebuild Method

1. Created fresh worktree `D:\tmp\wt-1956-signoff-pack` from
   `origin/main@1c4981a6` on branch `report/autoplay-vsbot-1841-signoff-pack-1956`.
2. Extracted four report files from stale branch commit `2be7c9d1` via
   `git show 2be7c9d1:<path>` — no wholesale branch merge.
3. Authored this PROMPT-1956 report as the refresh record.
4. No files outside the owned `reports/` scope were touched.

---

## Reports Included in This Branch

| Report file | Description | Status |
|-------------|-------------|--------|
| `reports/PROMPT-1841-autoplay-vsbot-1831-evidence-signoff-pack.md` | Original signoff pack — PROMPT 1831 evidence review | Reapplied unchanged |
| `reports/PROMPT-1889-autoplay-vsbot-1841-signoff-pack-refresh-after-1872.md` | First refresh after PROMPT 1872 main | Reapplied unchanged |
| `reports/PROMPT-1911-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1894.md` | Refresh after PROMPT 1894 main | Reapplied unchanged |
| `reports/PROMPT-1946-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1943.md` | Refresh after PROMPT 1943 main | Reapplied unchanged |
| `reports/PROMPT-1956-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1920.md` | This refresh record | New |

---

## Preserved Caveat (PROMPT 1831 evidence)

PROMPT 1841 Caveat C0 is preserved verbatim and remains BLOCKING:

> **C0 — HUMAN OBSERVATION — Window too small / offscreen clicks (BLOCKING):**
> During the autoplay/bot run the game window opened too small. The full UI was
> not visible; the bot moved the mouse and clicked in empty or offscreen space
> instead of real UI controls. Capture PASS and checkpoint PASS are not sufficient
> to declare a run valid unless the operator can confirm the window was large
> enough, no required control was outside the visible area, and observed click
> coordinates align with visible UI elements. If evidence is inconclusive on this
> point, AUTOPLAY-VS-BOT-QA-001 requires a viewport-size repair or a new run
> after the fix before sign-off can proceed.

This caveat may only be cleared by explicit operator evidence confirming no
offscreen-click or viewport-clipping occurred in the reviewed run.

---

## Validation

### Path allowlist
```
reports/PROMPT-1841-autoplay-vsbot-1831-evidence-signoff-pack.md
reports/PROMPT-1889-autoplay-vsbot-1841-signoff-pack-refresh-after-1872.md
reports/PROMPT-1911-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1894.md
reports/PROMPT-1946-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1943.md
reports/PROMPT-1956-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1920.md
```
No files outside `reports/` were modified.

### git diff --check
Run before commit — no trailing whitespace or mixed line endings.

### Strict-FF gate
```
git merge-base --is-ancestor origin/main report/autoplay-vsbot-1841-signoff-pack-1956
```
Branch is a pure forward extension of origin/main@1c4981a6 — strict-FF confirmed.

---

1956: AUTOPLAY-VSBOT-1841-SIGNOFF-PACK-REPORT-REFRESH-AFTER-1920: READY_FOR_MAINLAND_ENQUEUE
