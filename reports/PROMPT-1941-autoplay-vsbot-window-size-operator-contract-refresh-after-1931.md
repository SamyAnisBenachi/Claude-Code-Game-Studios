# PROMPT 1941 — AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REFRESH-AFTER-1931

**Type:** Report-only backfill — branch refresh (superseded by PROMPT 1964)
**Date:** 2026-05-28
**Status:** SUPERSEDED — stale branch `origin/report/autoplay-vsbot-window-contract-1941`
  was NOT_FF against current main; replaced by PROMPT 1964 on `origin/main@2bf3960d`
**Backfills:** `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md`
  and `reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md`
**Source branch (1941, stale):** `origin/report/autoplay-vsbot-window-contract-1941` @ `143df7f8`
**Intended refresh base:** `origin/main` @ `79031021` (PROMPT 1931)
**Actual current base (1964):** `origin/main` @ `2bf3960d` (PROMPT 1957)

---

## Purpose

PROMPT 1941 was intended to re-land the 1861 and 1914 report files cleanly onto
`origin/main@79031021` (PROMPT 1931), after the 1914 branch (`b6a65b65`) became
NOT_FF against main due to PROMPTs 1912, 1915, 1920, 1929, and 1931 landing.

The 1941 branch (`143df7f8`) was reported READY_FOR_MAINLAND_ENQUEUE but was never
merged to main. By the time PROMPT 1964 was authored, the 1941 branch was itself
NOT_FF against current main (`2bf3960d`) because PROMPTs 1932, 1937, 1939, 1943,
1950, and 1957 had landed in the interim.

Additionally, a direct merge of `origin/report/autoplay-vsbot-window-contract-1941`
would have:
1. Deleted already-landed reports (all reports between the 1941 branch base and
   current main that were not present in the 1941 branch).
2. Deleted the PROMPT 1957 auction tier-border report/test that landed as
   `2bf3960d`.
3. Introduced drift in client files that existed on the 1941 branch's history
   but were superseded on main.

PROMPT 1964 rebuilds the backfill cleanly from scratch on `origin/main@2bf3960d`.

---

## Why 1941 Was Stale at PROMPT 1964 Time

| PROMPT | Landed after 1941 base (79031021) | Affected files |
|--------|-----------------------------------|----------------|
| 1932 | post-1830-autoplay-tooling-verify report refresh | `reports/` |
| 1937 | qa-snapshot-observability-gap report refresh | `reports/` |
| 1939 | two-client-launcher-stale-binary-rebuild-guard | `reports/` + tooling |
| 1943 | live-two-client-full-flow-retest report refresh | `reports/` |
| 1950 | post-1830-autoplay-tooling-verify backfill refresh | `reports/` |
| 1957 | krosmaga auction tier-border asset binding refresh | `reports/` + `client/` + `assets/` |

A wholesale merge of `origin/report/autoplay-vsbot-window-contract-1941` would have
clobbered all 6 of these PROMPTs' contributions on main.

---

## What 1941 Was Supposed to Contain

The 1941 branch tip (`143df7f8`) contained only two files relative to its parent:
- `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md`
- `reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md`

Notably, the 1941 branch did NOT include a `reports/PROMPT-1941-*.md` tracking
report for the refresh itself. PROMPT 1964 adds this report (the file you are
reading) retroactively to document the refresh chain.

---

## State of Repairs at PROMPT 1941 Intended Base (79031021 / PROMPT 1931)

For historical reference, this was the AC-VPT status as of `origin/main@79031021`:

| AC | Requirement | PROMPT | Status at 79031021 |
|---|---|---|---|
| AC-VPT-01 | Driver aborts if initial window < [1280, 720] | 1880/1894 | **LANDED** |
| AC-VPT-02 | Driver detects mid-run resize > ±10 px | 1880/1894 | **LANDED** |
| AC-VPT-03 | Driver warns/aborts if `cursor_logical=None` | 1880/1894 | **LANDED** |
| AC-VPT-05 | Composite `frozen_all` downgrade | 1850 | Not confirmed |
| AC-VPT-06 | ≥ 3 distinct hashes for PASS | 1850 | Not confirmed |
| AC-VPT-07 | Composite window resize metadata | 1850 | Not confirmed |
| AC-VPT-08 | Launcher + Rust enforce 1280×720 at Startup | 1912 | **LANDED** |
| AC-VPT-04 | Recipe rebuilt after resize | future | Not scheduled |

This status is unchanged at `origin/main@2bf3960d` — no viewport guard or
composite verdict PROMPTs landed between 1931 and 1957.

---

## Disposition

This report documents the SUPERSEDED state of PROMPT 1941. The canonical re-landing
is PROMPT 1964. The 1861, 1914, 1941, and 1964 report files are all present in
branch `report/autoplay-vsbot-window-contract-1964` based on `origin/main@2bf3960d`.

---

1941: AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REFRESH-AFTER-1931: SUPERSEDED (replaced by PROMPT 1964)
