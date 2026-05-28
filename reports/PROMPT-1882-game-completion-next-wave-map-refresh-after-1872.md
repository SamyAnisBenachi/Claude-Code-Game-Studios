# PROMPT 1882 — Game-Completion Next-Wave Map (Refresh After PROMPT 1872)

**Date:** 2026-05-28
**Source-of-truth tip:** `origin/main@2ce3dc6b` (PROMPT 1872)
**Stage:** Polish (locked)
**Active sprint:** Sprint 18
**Supersedes:** `PROMPT-1869-game-completion-next-wave-map-refresh-after-1844-1858.md`
**Preserves:** `PROMPT-1846`, `PROMPT-1859`, `PROMPT-1872` report artifacts (all on main)

---

## §0 — What Changed Since PROMPT 1869

PROMPT 1869 was written when main was at `5c91918d` (PROMPT 1858). Since then one commit
has landed on main:

| Commit | PROMPT | Scope |
|--------|--------|-------|
| `2ce3dc6b` | 1872 | Reports-only: backfill PROMPT 1846 and 1859 analyzer reports onto post-1858 main |

No source code has changed. No AC-VPT branches have been main-landed. The structural
gameplay blockers are unchanged. This refresh updates the source-of-truth tip, records
1872 as landed, notes the two new in-flight dev-launcher branches (1874, 1876), and
corrects PROMPT number predictions in the range 1872–1882 which have diverged from the
1869 map.

**PROMPT number divergences since 1869:**

| 1869 Predicted | Actual Work at That Number |
|---|---|
| 1870 — LANE-B-BATCH-INTEGRATE | Not yet seen on any branch |
| 1871 — KROSMAGA-1853-CARGO-TEST-VERIFY | Not yet seen on any branch |
| 1872 — KROSMAGA-1854-CARGO-TEST-VERIFY | Actual: AUTOPLAY-EVIDENCE-ANALYZER-LATEST-RUN-REFRESH (reports-only, on main) |
| 1873 — AUCTION-WON-CARD-DISPOSITION-INTEGRATION | Not yet seen on any branch |
| 1874 — (no entry in 1869) | Actual: DEV-LAUNCHER-UX-REFRESH (reapply 1837 over post-1858 main; on origin/wt-1874) |
| 1875 — UI-PLAY-AREA-CONTAINER-IMPL | Not yet seen on any branch |
| 1876 — (no entry in 1869) | Actual: DEV-LAUNCHER-UX-REFRESH-V2 (reapply 1837/1874 over post-1872 main; on origin/wt-1876) |
| 1877–1881 — (various) | Not yet seen on any branch |
| 1882 — COMBAT-RESOLUTION-LIVE-VERIFY | Actual: THIS REPORT (game-completion next-wave map refresh) |

Do not use the 1869 map's suggested PROMPT numbers 1870–1882 — all are now taken or
deconflicted by this document. Use the revised suggested numbers in §8 below.

---

## Executive Summary

Since PROMPT 1835 the project has been in a sustained autoplay evidence quality repair
cycle. The root cause — DWM-initiated mid-run window resize causing click coordinates to
mis-target UI elements — was diagnosed in PROMPT 1844. Eight acceptance criteria
(AC-VPT-01 through AC-VPT-08) were defined. As of current main (PROMPT 1872):

- AC-VPT-01 is on branch `integrate/autoplay-window-size-default-1865` (not yet main)
- AC-VPT-02 is on branch `origin/wt-1843-click-viewport-guard` (not yet main)
- AC-VPT-08 is on branch `origin/wt-1844-window-drift-guard` (not yet main)
- AC-VPT-05/07 are on the PROMPT 1850 branch (not yet main)
- No autoplay run has achieved automated PASS; run 090613-Z is conditional/partial best

Two new dev-launcher UX branches (1874, 1876) exist; 1876 is FF-ready over current main.

Structural gameplay blockers unchanged: PROMPT 1472 live two-client QA retest has not
happened. Sprint 18 Must Have rows remain open. M2 release path requires two-client
full-game-loop validation.

---

## Current Blockers

| Blocker | Scope | Unblocked By |
|---------|-------|--------------|
| PROMPT 1472 live two-client QA retest not yet done | `client/src/ui/hand/`, `board_rendering.rs`, `shop_auction/`, `qa_snapshot.rs` | Human two-client session |
| All three autoplay evidence runs are PARTIAL (no PASS) | `tools/autoplay/`, AC-VPT-02/08 | AC-VPT repair chain landing + new evidence run |
| Mid-run DWM window resize bakes stale recipe coordinates | `tools/autoplay/driver.py`, `win_foreground.py` | wt-1843 + wt-1844 mainland |
| AC-VPT-01 guard (initial window size) not yet on main | `client/src/autoplay.rs`, `Run-AutoplaySmoke.ps1` | integrate/autoplay-window-size-default-1865 landing |
| AC-VPT-02 mid-run resize abort not yet on main | `tools/autoplay/driver.py` | wt-1843 landing |
| AC-VPT-08 viewport drift guard not yet on main | `tools/autoplay/driver.py` | wt-1844 landing |
| Composite verdict downgrade not yet on main | `analyze_evidence_run.py`, `validate_composite_run.py` | PROMPT 1850 branch landing |
| Sprint 18 AUCTION-WON-CARD-DISPOSITION-001 open | `server/src/feature/auction/`, `shared/src/protocol.rs` | PROMPT 1472 + integration worker |
| Sprint 18 UI-PLAY-AREA-CONTAINER-001 unimplemented | `client/src/ui/`, `client/src/presentation/` | PROMPT 1472 |
| Polish→Release gate-check FAIL (PROMPT 761, no retry) | release readiness | Sprint 18 closure + retry |
| S11-HUD-TIMER-EYEBALL-VISUAL-001 carry blocked on human sign-off | HUD visual | Human visual session |

---

## In-Flight Branches (Not Yet on Main)

| Branch | PROMPT | Scope | FF-Ready over 2ce3dc6b? |
|--------|--------|-------|-------------------------|
| `origin/wt-1876-dev-launcher-ux-refresh` | 1876 | Dev-launcher evidence UX reapply (1837+1874) | **Yes** — based on 2ce3dc6b |
| `integrate/autoplay-window-size-default-1865` | 1865 | AC-VPT-01 initial size gate (Rust + PS1) | Needs rebase to 2ce3dc6b |
| `origin/wt-1867-qa-obs-gap-refresh` | 1867 | QA snapshot obs gap refresh after 1844 | Needs rebase to 2ce3dc6b |
| `origin/wt-1874-dev-launcher-ux-refresh` | 1874 | Dev-launcher UX (superseded by 1876) | Stale — superseded |
| `origin/wt-1843-click-viewport-guard` | 1843 | AC-VPT-02 click-target guard in driver.py | Stale base (pre-1833) — needs integrate/ |
| `origin/wt-1844-window-drift-guard` | 1844-wt | AC-VPT-08 window-drift abort in driver.py | Stale base (pre-1833) — needs integrate/ |
| PROMPT 1850 branch | 1850 | Composite verdict downgrade (25+62 tests) | Needs integrate/ (unknown base) |
| `integrate/autoplay-placement-reject-recipe-1849` | 1849 | Placement-reject-probe recipe | Needs rebase to 2ce3dc6b |
| `origin/wt-1841-signoff-pack` | 1841 | Autoplay vs-bot 1831 evidence signoff pack | Needs rebase to 2ce3dc6b |
| `origin/wt-1839-qa-obs-gap` | 1839 | QA snapshot observability gap | Needs rebase to 2ce3dc6b |
| `origin/wt-1837-dev-launcher-evidence-ux` | 1837 | Dev launcher evidence UX (superseded by 1876) | Stale — superseded |
| `origin/wt-1853-tier-border-slice-b` | 1853 | Krosmaga auction tier border asset binding | Needs rebase to 2ce3dc6b |
| PROMPT 1854 branch | 1854 | Krosmaga hand fan readability layout | Needs integrate/ (unknown base) |
| `integrate/auction-won-card-disposition-1141` | 1141 | Auction won-card disposition | Gated on PROMPT 1472 |

**Note on 1837/1874:** Both are superseded by 1876 which incorporates their payloads. They
should not be independently landed. 1876 is the authoritative dev-launcher UX branch.

---

## Autoplay Evidence Quality State

### AC-VPT Acceptance Criteria Status

| AC | Description | Status | Owned by |
|----|-------------|--------|----------|
| AC-VPT-01 | Initial window size gate (abort if < 1280×720 at tick 1) | PARTIAL — on integrate/1865, not on main | PROMPT 1865 |
| AC-VPT-02 | Mid-run resize detection + abort | ON BRANCH — not on main | PROMPT 1843 (wt-1843) |
| AC-VPT-03 | Null cursor guard before clicks (advisory) | NOT IMPLEMENTED | Future PROMPT |
| AC-VPT-04 | Post-resize recipe rebuild (advisory) | NOT IMPLEMENTED — architectural change | Future PROMPT |
| AC-VPT-05 | Win32 all-frozen → NEEDS_HUMAN_GUI flag | ON BRANCH (1850) — not on main | PROMPT 1850 |
| AC-VPT-06 | Min screenshot requirements for PASS (blocking) | PARTIAL — driver side not enforced | Future PROMPT |
| AC-VPT-07 | Window size in composite report (advisory) | ON BRANCH (1850) — not on main | PROMPT 1850 |
| AC-VPT-08 | Viewport drift abort (SW_RESTORE shrink protection) | ON BRANCH (wt-1844) — not on main | PROMPT 1844-wt |

### Current Evidence Corpus (unchanged since PROMPT 1869)

Three runs in `production/qa/evidence/autoplay-runs/`:

| Run | Window | Analyzer Verdict | Notes |
|-----|--------|-----------------|-------|
| `20260528-051148-Z` | 1296×759 fixed | PARTIAL — no win32 capture labels | Bevy RPC screenshots only; no pixel_hash data |
| `20260528-063609-Z` | 1296×759 fixed | PARTIAL — frozen renderer (all 15 hashes identical) | All captures same frame; bot likely clicking clipped/offscreen |
| `20260528-090613-Z` | 1296×759 → 1296×1115 | PARTIAL — 11 FROZEN log lines | Mid-run resize; post-resize clicks at wrong fractions; best bitblt evidence |

**No run has achieved PASS verdict.** A clean PASS requires: distinct pixel hashes across
checkpoints, zero FROZEN lines (or bitblt-primary), window at 1280×720 from tick 1, no
mid-run resize.

---

## Revised Lane Map

### LANE A — AC-VPT Repair Chain (Sequential)

**Step A1 — Land AC-VPT-01 (initial size gate)**

| Branch | Scope | Action Needed |
|--------|-------|---------------|
| `integrate/autoplay-window-size-default-1865` | `client/src/autoplay.rs`, `Run-AutoplaySmoke.ps1` | Rebase onto 2ce3dc6b then FF-merge to main |

**Step A2 — Integrate AC-VPT-02 (click-target viewport guard)**

Cherry-pick wt-1843 payload onto post-A1 main into new integrate/ branch.
Files: `tools/autoplay/driver.py`, `tests/tools/autoplay/test_driver_click_viewport_guard.py`

**Step A3 — Integrate AC-VPT-08 (window-drift abort)**

Cherry-pick wt-1844-wt payload onto post-A2 main into new integrate/ branch.
Files: `tools/autoplay/driver.py`, `tests/tools/autoplay/test_driver_window_drift_guard.py`

**Step A4 — Land composite verdict downgrade (PROMPT 1850)**

Integrate PROMPT 1850 branch onto post-A3 main.
Files: `tools/autoplay/analyze_evidence_run.py`, `validate_composite_run.py`, `test_window_resize_verdict.py`

**Step A5 — New clean evidence run**

After A1–A4 on main: run `Start-AutoplayVsBot.ps1` with `CCGS_AUTOPLAY_BOT_ROOM_READY=1`.
Pass requires: distinct pixel hashes, zero FROZEN lines, fixed 1280×720 throughout.

**Ready to start:** A1 (rebase only). A2/A3/A4 blocked on A1.

---

### LANE B — In-Flight Report/Tool Branches (Parallel-Safe, No Conflicts)

| Priority | Branch | PROMPT | Notes |
|----------|--------|--------|-------|
| **HIGH** | `origin/wt-1876-dev-launcher-ux-refresh` | 1876 | FF-ready over 2ce3dc6b — land immediately |
| HIGH | `integrate/autoplay-placement-reject-recipe-1849` | 1849 | Rebase to 2ce3dc6b |
| HIGH | `origin/wt-1867-qa-obs-gap-refresh` | 1867 | Rebase to 2ce3dc6b |
| MED | `origin/wt-1841-signoff-pack` | 1841 | Reports-only; rebase to 2ce3dc6b |
| MED | `origin/wt-1839-qa-obs-gap` | 1839 | Reports-only; rebase to 2ce3dc6b |
| SKIP | `origin/wt-1837-dev-launcher-evidence-ux` | 1837 | Superseded by 1876 |
| SKIP | `origin/wt-1874-dev-launcher-ux-refresh` | 1874 | Superseded by 1876 |

**Suggested prompt:**
- `PROMPT-1883-LANE-B-BATCH-INTEGRATE` — Land 1876 (FF-ready); rebase and integrate 1849, 1867, 1841, 1839

---

### LANE C — Krosmaga UI Slices (Parallel-Safe, Source Code Changes)

| Branch | PROMPT | Files | Cargo Test Status |
|--------|--------|-------|-------------------|
| `origin/wt-1853-tier-border-slice-b` | 1853 | `client/src/asset_wiring.rs`, `client/src/ui/shop_auction/mod.rs` | Deferred (disk full at ship time) |
| PROMPT 1854 branch | 1854 | `client/src/ui/hand/mod.rs` | Deferred (disk full at ship time) |

**Blocked-on:** Disk space (PDB cleanup freed ~23 GB previously; reverify before running
`cargo test -p client`). No source conflict with Lanes A or B.

**Suggested prompts:**
- `PROMPT-1884-KROSMAGA-1853-CARGO-TEST-VERIFY` — cargo test for wt-1853 tier-border slice
- `PROMPT-1885-KROSMAGA-1854-CARGO-TEST-VERIFY` — cargo test for PROMPT 1854 fan readability

---

### LANE D — Sprint 18 Must Have Completions (Gated on PROMPT 1472)

#### D1. S18-AUCTION-WON-CARD-DISPOSITION-001

| Suggested Prompt | Scope | Dependency |
|---|---|---|
| `PROMPT-1886-AUCTION-WON-CARD-DISPOSITION-INTEGRATION` | Merge/rebase integrate/auction-won-card-disposition-1141; smoke; story-done | PROMPT 1472 done |
| `PROMPT-1887-AUCTION-WON-CARD-DISPOSITION-VERIFY` | Confirm card flows to hand; screenshot evidence | 1886 SHIPPED |

**Conflict risk:** HIGH on `client/src/ui/shop_auction/` — do not parallelize with other shop_auction workers.

#### D2. S18-UI-PLAY-AREA-CONTAINER-001

| Suggested Prompt | Scope | Dependency |
|---|---|---|
| `PROMPT-1888-UI-PLAY-AREA-CONTAINER-IMPL` | Implement play-area container per Sprint 18 story spec; smoke; story-done | PROMPT 1472 done |

**Conflict risk:** HIGH on `client/src/presentation/board_rendering.rs`.

#### D3. PROMPT 1472 — POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST (Primary Unlock)

| Suggested Prompt | Scope | Dependency |
|---|---|---|
| `PROMPT-1476-POST-REPAIR-LIVE-QA-RETEST` | Two-client session; QA snapshot; verify hand/board/auction/placement | Human GUI session |

---

### LANE E — Gameplay Loop Gaps (Blocked Until PROMPT 1472 + Sprint 18 Closes)

| Story Gap | Suggested Prompt | Gate |
|---|---|---|
| Round state machine completeness | `PROMPT-1890-ROUND-STATE-MACHINE-AUDIT` | None (read-only) |
| Round state machine gap repair | `PROMPT-1891-ROUND-STATE-MACHINE-GAP-REPAIR` | 1890 + PROMPT 1472 |
| Combat resolution live verify | `PROMPT-1892-COMBAT-RESOLUTION-LIVE-VERIFY` | PROMPT 1472 + human |
| Combat keyword integration tests | `PROMPT-1893-COMBAT-KEYWORD-INTEGRATION-TESTS` | None (test authoring) |
| Sang Méprise ADR-024 live verify | `PROMPT-1894-SANG-MEPRISE-LIVE-VERIFY` | PROMPT 1472 |
| Win condition verify | `PROMPT-1895-WIN-CONDITION-LIVE-VERIFY` | PROMPT 1472 + human |

None can start until PROMPT 1472 live retest clears.

---

### LANE F — Sprint 18 Should-Have Paperwork (Parallel-Safe Now)

| Row | Suggested Prompt |
|-----|-----------------|
| S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001 | `PROMPT-1900-S18-VIEWPORT-INVARIANT-STORY-DONE` |
| S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 | `PROMPT-1901-S18-SNAPSHOT-LAYOUT-FIELDS-STORY-DONE` |
| S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001 | `PROMPT-1902-S18-SETTINGS-PANEL-RELAYOUT-VERIFY` |
| S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001 | `PROMPT-1903-S18-HAND-MANA-PREVIEW-STORY-DONE` |
| S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001 | `PROMPT-1904-S18-HAND-IDLE-AFFORDANCE-STORY-DONE` |

All five are parallel-safe (no source file edits; sprint tracker + story state only).

---

### LANE G — Sprint 18 Closure + Release Prep (Sequential After D Closes)

| Step | Suggested Prompt | Dependencies |
|------|-----------------|--------------|
| Sprint 18 completion tally | `PROMPT-1905-SPRINT-18-COMPLETION-STATUS-REFRESH` | Lane F done, Lane D in progress |
| Sprint 19 planning draft | `PROMPT-1906-SPRINT-19-PLANNING-DRAFT` | Sprint 18 tally |
| Polish→Release gate-check retry preconditions | `PROMPT-1907-POLISH-RELEASE-GATE-RETRY-PRECONDITIONS` | None (read-only audit of PROMPT 761 findings) |
| Gate-check retry | `PROMPT-1908-POLISH-RELEASE-GATE-CHECK-RETRY` | 1907 + Sprint 18 Must Have closed + PROMPT 1472 done |

---

## Parallelizability Matrix

```
LANE A1 (AC-VPT-01 rebase+land 1865)  ──── safe now (no overlap with B/C/F)
LANE B  (1876 FF-land; 1849/1867 rebase-integrate)  ──── safe now
LANE C  (Krosmaga cargo test verify 1853+1854)       ──── safe now
LANE F  (S18 should-have paperwork)                  ──── safe now
LANE G1 (gate-retry preconditions 1907)              ──── safe now (read-only)
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
        A2/A3/A4 (driver.py     Human two-client session
         repair chain)          → PROMPT 1472 clears
              │                     │
              ▼                     ▼
        A5 (clean evidence run)  Lane D (S18 Must Haves)
                         │
                         ▼
                   Lane E (gameplay gaps)
                   Lane G2/G3/G4 (sprint close + release)
```

**Immediate parallel batch (safe right now):**
1. LANE A1: Rebase and land `integrate/autoplay-window-size-default-1865` onto 2ce3dc6b
2. LANE B: Land 1876 (FF-ready); rebase and integrate 1849, 1867, 1841, 1839
3. LANE C: Cargo test verify for 1853 + 1854 (verify disk space first)
4. LANE F: Sprint 18 paperwork (1900–1904)
5. LANE G1: `PROMPT-1907-POLISH-RELEASE-GATE-RETRY-PRECONDITIONS` (read-only audit)

---

## File Ownership / Conflict Risk Map

| File / Directory | Risk Level | Current Branch Owners |
|---|---|---|
| `tools/autoplay/driver.py` | HIGH | wt-1843 + wt-1844-wt (pre-main, sequential integration required) |
| `tools/autoplay/analyze_evidence_run.py` | MED | PROMPT 1850 branch + PROMPT 1865 |
| `tools/autoplay/validate_composite_run.py` | MED | PROMPT 1850 branch |
| `client/src/autoplay.rs` | MED | integrate/autoplay-window-size-default-1865 |
| `tools/autoplay/Run-AutoplaySmoke.ps1` | LOW | integrate/autoplay-window-size-default-1865 |
| `client/src/ui/shop_auction/` | HIGH | D1 (gated on PROMPT 1472) |
| `client/src/presentation/board_rendering.rs` | HIGH | D2 (gated on PROMPT 1472) |
| `client/src/presentation/qa_snapshot.rs` | MED | Lane E (gated) |
| `client/src/ui/hand/mod.rs` | LOW | PROMPT 1854 (Krosmaga) |
| `client/src/asset_wiring.rs` | LOW | PROMPT 1853 (Krosmaga) |
| `server/src/feature/auction/` | MED | D1 (gated) |
| `server/src/core/` | MED | Lane E (gated) |
| `shared/src/protocol.rs` | MED | D1 (gated) |
| `tools/dev-launcher/` | LOW | wt-1876 (FF-ready) |
| `tools/autoplay/recipes/` | LOW | integrate/autoplay-placement-reject-recipe-1849 |
| `production/sprints/` | LOW | Lane F paperwork |
| `reports/` | NONE | Any worker |

---

## Release Gap Summary

For M2 (Playable Game) milestone gate: two players must connect, play a full 1v1 game
through auction → placement → combat → shop → win condition. Minimum path:

1. **PROMPT 1472** POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST (overdue; primary unlock)
2. **S18-AUCTION-WON-CARD-DISPOSITION-001** (Lane D1)
3. **S18-UI-PLAY-AREA-CONTAINER-001** (Lane D2)
4. **Combat resolution live verify** (Lane E)
5. **Win-condition verify** (Lane E)
6. **Sprint 18 Must Have closure** (all 4 rows)
7. **Polish→Release gate-check retry** (PROMPT 1908)

Lanes A–C are sprint-health work, not on the M2 critical path. All can run in parallel
with the M2 path. The QA snapshot button (`CCGS_QA_SNAPSHOT=1`) exists since
`origin/main@8a3744e` and is the primary evidence tool for Lane D/E.

---

1882: GAME-COMPLETION-NEXT-WAVE-MAP-REFRESH-AFTER-1872: DONE
