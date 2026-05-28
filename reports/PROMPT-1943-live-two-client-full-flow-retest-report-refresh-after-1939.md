# PROMPT 1943 — Live Two-Client Full-Flow Retest Report Refresh After PROMPT 1939

**Date:** 2026-05-28
**Worker branch:** `report/live-two-client-full-flow-retest-1943`
**Base commit (origin/main):** `be40e0c6b1349267480c0b18d6144b881dbc170e`

---

## Purpose

PROMPT 1903 shipped on branch `origin/report/live-two-client-full-flow-retest-1903`
(commit `cf98ffa9e9a8e941c593ad2a6c38f4f93e4cd287`) but that branch is NOT
fast-forward against current `origin/main@be40e0c6`. Landing it wholesale would:

- Delete 11 reports already on main (PROMPT 1831/1855/1866/1871/1877/1879/1880/
  1890/1892/1893/1894/1912/1915/1929/1931/1939)
- Modify `client/src/autoplay.rs`, `tools/autoplay/**`, `tools/dev-launcher/**`
- Delete `tests/tools/autoplay/test_driver_click_viewport_guard.py`

This PROMPT recreates the report-only payload cleanly on current `origin/main`,
preserving all existing reports.

---

## Source Branch Analysis

| Field | Value |
|-------|-------|
| Source branch | `origin/report/live-two-client-full-flow-retest-1903` |
| Source commit | `cf98ffa9e9a8e941c593ad2a6c38f4f93e4cd287` |
| Intended payload | PROMPT-1883 + PROMPT-1903 report files only |
| Conflict risk | Deletes current reports + modifies source/tools/tests |
| Action taken | Copied report content verbatim; no tool/src/test files touched |

---

## Branch Construction

1. Fetched `origin` — confirmed `origin/main` at `be40e0c6`.
2. Created worktree at `D:\_DEV\Work\tmpwt-1943-report-refresh`
   on branch `report/live-two-client-full-flow-retest-1943` tracking `origin/main`.
3. Copied `reports/PROMPT-1883-live-two-client-full-flow-retest-after-1872.md`
   verbatim from `git show origin/report/live-two-client-full-flow-retest-1903:reports/...`.
4. Copied `reports/PROMPT-1903-live-two-client-full-flow-retest-report-refresh.md`
   verbatim from `git show origin/report/live-two-client-full-flow-retest-1903:reports/...`.
5. Wrote `reports/PROMPT-1943-live-two-client-full-flow-retest-report-refresh-after-1939.md`
   (this file).
6. Added all three with `git add -f` (reports/ is gitignored).
7. Committed as a single docs commit.

---

## Validation

### Path allowlist check

All three files are under `reports/` — the only directory owned by this PROMPT.
No files under `tools/`, `client/`, `server/`, `production/`, `tests/`, or
`Cargo.*` were touched.

### git diff --name-status origin/main..HEAD

```
A       reports/PROMPT-1883-live-two-client-full-flow-retest-after-1872.md
A       reports/PROMPT-1903-live-two-client-full-flow-retest-report-refresh.md
A       reports/PROMPT-1943-live-two-client-full-flow-retest-report-refresh-after-1939.md
```

Three files added. Zero files modified. Zero files deleted.

### FF status

Branch is a strict fast-forward from `be40e0c6` (origin/main).

### git diff --check

No whitespace errors.

---

## PROMPT 1883 Finding Summary (from source branch cf98ffa9)

Retest blocked before lobby UI. Root cause: `client.exe` (built 2026-05-21
from commit `3a4603af`, branch `play-main`) is 7 days and 20+ Rust commits
stale vs `server.exe` (built 2026-05-28). Lightyear protocol hash mismatch
causes both clients to panic at handshake. Server startup is clean (PASS).
Rebuild required before GUI phases can be verified.

See `reports/PROMPT-1883-live-two-client-full-flow-retest-after-1872.md` for
full panic text, server log timeline, phase coverage table, and rebuild steps.

---

## Final State

| Field | Value |
|-------|-------|
| Branch | `report/live-two-client-full-flow-retest-1943` |
| Base (origin/main) | `be40e0c6b1349267480c0b18d6144b881dbc170e` |
| Source commit (1903 branch) | `cf98ffa9e9a8e941c593ad2a6c38f4f93e4cd287` |
| FF-ready | YES |
| Files added | 3 |
| Files deleted | 0 |
| Tools/src touched | NO |

---

1943: LIVE-TWO-CLIENT-FULL-FLOW-RETEST-REPORT-REFRESH-AFTER-1939: READY_FOR_MAINLAND_ENQUEUE
