# PROMPT 1869 — Game-Completion Next-Wave Map (Refresh After PROMPTs 1844 + 1858)

**Date:** 2026-05-28
**Source-of-truth tip:** `origin/main@5c91918d` (PROMPT 1858)
**Stage:** Polish (locked)
**Active sprint:** Sprint 18
**Supersedes:** `PROMPT-1835-game-completion-next-wave-map-after-1830.md` (historical;
see §0 for reconciliation note)

---

## §0 — Reconciliation: PROMPT 1835 vs Actuality

PROMPT 1835 was written when main was at `71484998` (PROMPT 1830). It predicted prompt
numbers for future work. Every one of those predictions diverged: the autoplay evidence
crisis (DWM window-resize, frozen PrintWindow, click-target misrouting) dominated
PROMPTs 1833–1867, consuming all planned-for PROMPT numbers in the 1836–1864 range.

**Summary of divergences:**

| 1835 Predicted | Actual Work at That Number |
|---|---|
| 1836 — VSBOT-LIVE-EVIDENCE | Actual runs triggered; three evidence dirs created (no PROMPT-named commit) |
| 1837 — AUTOPLAY-SOAK-RECIPE | 1837 = DEV-LAUNCHER-EVIDENCE-UX-FAST-LANE (shipped on branch) |
| 1838 — AUCTION-WON-CARD-DISPOSITION | 1838 = POST-1830-AUTOPLAY-TOOLING-VERIFY (PASS, on main) |
| 1839 — (no entry) | 1839 = QA-SNAPSHOT-OBSERVABILITY-GAP (on branch) |
| 1841 — VIEWPORT-INVARIANT-STORY-DONE | 1841 = AUTOPLAY-VSBOT-1831-EVIDENCE-SIGNOFF-PACK (on branch) |
| 1842 — SNAPSHOT-LAYOUT-FIELDS-STORY-DONE | 1842 = WINDOW-SIZE-DEFAULT-REPAIR (PARTIAL, work/ branch) |
| 1843 — SETTINGS-PANEL-RELAYOUT-VERIFY | 1843 = CLICK-TARGET-VIEWPORT-GUARD (on branch wt-1843) |
| 1844 — CARD-ART-LABEL-STRIP-IMPL | 1844 = VIEWPORT/CLICK-EVIDENCE-AUDIT (on main) + WINDOW-DRIFT-GUARD (on branch wt-1844) |
| 1845 — ROUND-STATE-MACHINE-AUDIT | 1845 = POST-1833-EVIDENCE-ANALYZER-FOCUSED-VERIFY (PASS, backfilled via 1858) |
| 1846 — ROUND-STATE-MACHINE-GAP-REPAIR | 1846 = EVIDENCE-ANALYZER-LATEST-RUN-APPLICATION (PARTIAL, on main) |
| 1849 — OBJECTIVE-SYSTEM-SANG-MEPRISE | 1849 = PLACEMENT-REJECT-RECIPE-INTEGRATION-REFRESH (on integrate/ branch) |
| 1850 — QA-SNAPSHOT-COVERAGE-AUDIT | 1850 = COMPOSITE-WINDOW-RESIZE-VERDICT-DOWNGRADE (on branch) |
| 1853 — BOT-SOAK-1607-MAIN-LAND-VERIFY | 1853 = KROSMAGA-AUCTION-TIER-BORDER-SLICE-B (on branch wt-1853) |
| 1854 — AUTOPLAY-RECIPE-LIBRARY-DESIGN | 1854 = KROSMAGA-HAND-FAN-READABILITY-STAGE3-SLICE-D (on branch) |
| 1858 — HUD-TIMER-EYEBALL-VISUAL-IMPL | 1858 = POST-1845-REPORT-BACKFILL (on main) |
| 1865 — (not predicted) | 1865 = WINDOW-SIZE-DEFAULT-REPAIR-MAINLAND-REFRESH (on integrate/ branch) |
| 1867 — (not predicted) | 1867 = QA-SNAPSHOT-OBS-GAP-REFRESH-AFTER-1844 (on branch wt-1867) |

All PROMPT numbers predicted in the 1835 map in the range 1838–1864 are now taken by
different work. **Do not use the 1835 map as a source for prompt numbers or lane
assignments.** Use this document instead.

---

## Executive Summary

Since PROMPT 1835 (main @ 71484998 / PROMPT 1830), the project has taken a sustained
detour through the autoplay evidence quality crisis. The root cause — DWM-initiated
mid-run window resize causing recipe click coordinates to mis-target UI elements —
was formally diagnosed in the PROMPT 1844 viewport/click evidence audit. That audit
defined eight acceptance criteria (AC-VPT-01 through AC-VPT-08) for making autoplay
evidence mechanically trustworthy. As of PROMPT 1858 (current main), AC-VPT-01 is
partially addressed (initial-size gate, PROMPT 1865 branch) and AC-VPT-02/08
(mid-run resize abort, viewport drift guard) are on branches wt-1843 and wt-1844
awaiting mainland. Two Krosmaga UI slice branches (1853, 1854) also await landing.

The structural gameplay blockers from 1835 remain unchanged:
- PROMPT 1472 POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST has not happened.
- Sprint 18 Must Have rows (AUCTION-WON-CARD-DISPOSITION, UI-PLAY-AREA-CONTAINER) remain open.
- Polish→Release gate-check (PROMPT 761) failed; no retry attempt has been logged.

No gameplay source code has changed since the PROMPT 1471 repair chain main-land.

---

## Current Blockers

| Blocker | Scope | Unblocked By |
|---------|-------|--------------|
| PROMPT 1472 live two-client QA retest not yet done | `client/src/ui/hand/`, `client/src/presentation/board_rendering.rs`, `client/src/ui/shop_auction/`, `qa_snapshot.rs` | Human two-client session |
| All three autoplay evidence runs are PARTIAL (no PASS) | `tools/autoplay/`, AC-VPT-02/08 | AC-VPT repair chain (1843+1844 branches) landing + new evidence run |
| Mid-run DWM window resize bakes stale recipe coordinates | `tools/autoplay/driver.py`, `win_foreground.py` | wt-1843 + wt-1844 mainland |
| AC-VPT-01 guard (initial window size) not yet on main | `client/src/autoplay.rs`, `Run-AutoplaySmoke.ps1` | integrate/autoplay-window-size-default-1865 landing |
| AC-VPT-02 mid-run resize abort not yet on main | `tools/autoplay/driver.py` | wt-1843 landing |
| AC-VPT-08 viewport drift guard not yet on main | `tools/autoplay/driver.py` | wt-1844 landing |
| Composite verdict downgrade for resize/frozen not yet on main | `tools/autoplay/analyze_evidence_run.py`, `validate_composite_run.py` | PROMPT 1850 branch landing |
| Sprint 18 AUCTION-WON-CARD-DISPOSITION-001 open | `server/src/feature/auction/`, `shared/src/protocol.rs` | PROMPT 1472 + integration worker |
| Sprint 18 UI-PLAY-AREA-CONTAINER-001 unimplemented | `client/src/ui/`, `client/src/presentation/` | PROMPT 1472 |
| Polish→Release gate-check FAIL (PROMPT 761, no retry) | release readiness | Sprint 18 closure + retry |
| S11-HUD-TIMER-EYEBALL-VISUAL-001 carry blocked on human sign-off | HUD visual | Human visual session |

---

## In-Flight Branches (Not Yet on Main)

These branches were shipped by workers but have not been fast-forward merged to main.
Each is a prerequisite for its lane.

| Branch | PROMPT | Scope | FF-Ready? |
|--------|--------|-------|-----------|
| `origin/wt-1837-dev-launcher-evidence-ux` | 1837 | Dev launcher evidence UX fast-lane | Yes (addendum present) |
| `origin/wt-1839-qa-obs-gap` | 1839 | QA snapshot observability gap | Yes |
| `origin/wt-1841-signoff-pack` | 1841 | Autoplay vs-bot 1831 evidence signoff pack + C0 BLOCKING caveat | Yes |
| `origin/wt-1843-click-viewport-guard` | 1843 | AC-VPT-02 click-target guard in driver.py (235 tests) | Stale base (pre-1833) — needs integrate/ |
| `origin/wt-1844-window-drift-guard` | 1844-wt | AC-VPT-08 window-drift abort in driver.py (236 tests) | Stale base (pre-1833) — needs integrate/ |
| `integrate/autoplay-window-size-default-1865` | 1865 | AC-VPT-01 initial size gate (Rust + PS1) | Yes (over bb90d7c2 — needs re-base to 5c91918d) |
| `origin/wt-1867-qa-obs-gap-refresh` | 1867 | QA snapshot obs gap refresh after 1844 | Yes (over bb90d7c2) |
| PROMPT 1850 branch | 1850 | Composite verdict downgrade (25+62 tests) | Needs integrate/ (unknown base) |
| `integrate/autoplay-placement-reject-recipe-1849` | 1849 | Placement-reject-probe recipe (121 lines) | Yes (over b856eef4 — 1 commit ahead) |
| `origin/wt-1853-tier-border-slice-b` | 1853 | Krosmaga auction tier border asset binding | Yes |
| PROMPT 1854 branch | 1854 | Krosmaga hand fan readability layout fix | Needs integrate/ |

---

## Autoplay Evidence Quality State

### AC-VPT Acceptance Criteria Status

| AC | Description | Status | Owned by |
|----|-------------|--------|----------|
| AC-VPT-01 | Initial window size gate (abort if < 1280×720 at tick 1) | PARTIAL — shipped on integrate/1865, not on main | PROMPT 1865 |
| AC-VPT-02 | Mid-run resize detection + abort | SHIPPED ON BRANCH — not on main | PROMPT 1843 (wt-1843) |
| AC-VPT-03 | Null cursor guard before clicks (advisory) | NOT IMPLEMENTED | PROMPT 1843 or follow-up |
| AC-VPT-04 | Post-resize recipe rebuild (advisory) | NOT IMPLEMENTED — architectural change needed | Future PROMPT |
| AC-VPT-05 | Win32 all-frozen → NEEDS_HUMAN_GUI flag | SHIPPED ON BRANCH (PROMPT 1850) — not on main | PROMPT 1850 |
| AC-VPT-06 | Min screenshot requirements for PASS (blocking) | PARTIAL — driver side not enforced | Future PROMPT |
| AC-VPT-07 | Window size in composite report (advisory) | SHIPPED ON BRANCH (PROMPT 1850) — not on main | PROMPT 1850 |
| AC-VPT-08 | Viewport drift abort (SW_RESTORE shrink protection) | SHIPPED ON BRANCH (wt-1844) — not on main | PROMPT 1844-wt |

### Current Evidence Corpus (as of PROMPT 1858)

Three runs exist in `production/qa/evidence/autoplay-runs/`:

| Run | Window | Analyzer Verdict | Notes |
|-----|--------|-----------------|-------|
| `20260528-051148-Z` | 1296×759 fixed | PARTIAL — no win32 capture labels | Bevy RPC screenshots only; no pixel_hash data |
| `20260528-063609-Z` | 1296×759 fixed | PARTIAL — frozen renderer (all 15 hashes identical) | All captures same frame; bot likely clicking clipped/offscreen |
| `20260528-090613-Z` | 1296×759 → 1296×1115 | PARTIAL — 11 FROZEN log lines | Mid-run resize; post-resize clicks at wrong fractions; best bitblt evidence |

**No run has achieved PASS verdict. A clean PASS requires:** distinct pixel hashes across checkpoints, zero FROZEN lines (or bitblt-primary), window at 1280×720 from tick 1, no mid-run resize.

---

## Revised Lane Map

### LANE A — AC-VPT Repair Chain (Sequential)

The five blocking AC-VPT items must land before a clean autoplay PASS is achievable.
Ordering is driven by merge base cleanliness; each must not conflict with the previous.

**Step A1 — Land AC-VPT-01 (initial size gate)**

| Branch | Scope | Action Needed |
|--------|-------|---------------|
| `integrate/autoplay-window-size-default-1865` | `client/src/autoplay.rs`, `Run-AutoplaySmoke.ps1` | Re-base onto 5c91918d then FF-merge to main |

**Step A2 — Integrate AC-VPT-02 (click-target viewport guard)**

| Work Needed | Source | Target |
|-------------|--------|--------|
| Cherry-pick wt-1843 payload onto post-A1 main | `tools/autoplay/driver.py`, `tests/tools/autoplay/test_driver_click_viewport_guard.py` | New integrate/ branch |

**Step A3 — Integrate AC-VPT-08 (window-drift abort)**

| Work Needed | Source | Target |
|-------------|--------|--------|
| Cherry-pick wt-1844-wt payload onto post-A2 main | `tools/autoplay/driver.py`, `tests/tools/autoplay/test_driver_window_drift_guard.py` | New integrate/ branch |

**Step A4 — Land composite verdict downgrade (PROMPT 1850)**

| Work Needed | Source | Target |
|-------------|--------|--------|
| Integrate PROMPT 1850 branch | `tools/autoplay/analyze_evidence_run.py`, `validate_composite_run.py`, `test_window_resize_verdict.py` | New integrate/ branch over post-A3 main |

**Step A5 — New clean evidence run**

After A1–A4 are on main: run `Start-AutoplayVsBot.ps1` with `CCGS_AUTOPLAY_BOT_ROOM_READY=1`.
A passing run would produce `smoke_exit=0` + distinct pixel hashes + no FROZEN lines +
fixed 1280×720 window throughout.

**Ready to start:** A1 (re-base only). A2/A3/A4 blocked on A1.

---

### LANE B — In-Flight Report/Tool Branches (Parallel-Safe, No Conflicts)

These branches have no source code conflicts with Lane A or each other. They can be
mainlined in parallel with the AC-VPT work.

| Priority | Branch | PROMPT | Notes |
|----------|--------|--------|-------|
| HIGH | `integrate/autoplay-placement-reject-recipe-1849` | 1849 | FF-ready over b856eef4; rebase to 5c91918d |
| HIGH | `origin/wt-1867-qa-obs-gap-refresh` | 1867 | Over bb90d7c2; rebase to 5c91918d |
| MED | `origin/wt-1841-signoff-pack` | 1841 | Reports-only; FF over 71484998 — rebase to 5c91918d |
| MED | `origin/wt-1839-qa-obs-gap` | 1839 | Reports-only; FF over 71484998 |
| LOW | `origin/wt-1837-dev-launcher-evidence-ux` | 1837 | dev-launcher tools; over 71484998 |

**Suggested PROMPT titles:**

- `PROMPT-1870-LANE-B-BATCH-INTEGRATE` — Integrate the five LANE B branches above (re-base each to 5c91918d; confirm no deletions of 1833/1844/1858 artifacts; commit + push each)

---

### LANE C — Krosmaga UI Slices (Parallel-Safe, Source Code Changes)

Two Krosmaga visual improvement branches shipped but await main-land + cargo test verification.

| Branch | PROMPT | Files | Cargo Test Status |
|--------|--------|-------|-------------------|
| `origin/wt-1853-tier-border-slice-b` | 1853 | `client/src/asset_wiring.rs`, `client/src/ui/shop_auction/mod.rs`, `tests/unit/asset_wiring/` | Deferred (disk full at ship time) |
| PROMPT 1854 branch | 1854 | `client/src/ui/hand/mod.rs`, `tests/integration/hand-ui/`, `tests/unit/hand-ui/` | Deferred (disk full at ship time) |

**Blocked-on:** Disk space (D: drive, PDB cleanup freed ~23 GB). Must run `cargo test -p client` before these can be gate-checked.

**No conflict** between 1853 and 1854 (different source directories). Both safe to parallelize. Both safe to run in parallel with Lanes A and B (no `tools/autoplay/` or report overlap).

**Suggested PROMPT titles:**

- `PROMPT-1871-KROSMAGA-1853-CARGO-TEST-VERIFY` — Run cargo test for wt-1853 tier-border slice; confirm 7 unit tests pass; produce verify report
- `PROMPT-1872-KROSMAGA-1854-CARGO-TEST-VERIFY` — Run cargo test for PROMPT 1854 fan readability; confirm `default_config_10_cards_at_1280x720_readability_invariants` passes; produce verify report

---

### LANE D — Sprint 18 Must Have Completions (Gated on PROMPT 1472)

These remain structurally unchanged from PROMPT 1835. Nothing has unblocked them.

#### D1. S18-AUCTION-WON-CARD-DISPOSITION-001 (0.75d)

Branch `integrate/auction-won-card-disposition-1141` is outstanding.

| Suggested Prompt | Scope | Dependency |
|---|---|---|
| `PROMPT-1873-AUCTION-WON-CARD-DISPOSITION-INTEGRATION` | Merge/rebase integration branch; smoke; story-done S18 row | PROMPT 1472 retest done |
| `PROMPT-1874-AUCTION-WON-CARD-DISPOSITION-VERIFY` | Confirm card flows to hand; screenshot evidence | 1873 SHIP |

**Conflict risk:** HIGH on `client/src/ui/shop_auction/` — do not parallelize with any other shop_auction workers.

#### D2. S18-UI-PLAY-AREA-CONTAINER-001 (0.75d)

| Suggested Prompt | Scope | Dependency |
|---|---|---|
| `PROMPT-1875-UI-PLAY-AREA-CONTAINER-IMPL` | Implement play-area container per Sprint 18 story spec; smoke; story-done | PROMPT 1472 retest done |

**Conflict risk:** HIGH on `client/src/presentation/board_rendering.rs`.

#### D3. PROMPT 1472 — POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST

This is the primary unlock for all of Lane D. No new evidence has been generated since
the PROMPT 1471 repair chain landed. A human two-client session is required.

| Suggested Prompt | Scope | Dependency |
|---|---|---|
| `PROMPT-1476-POST-REPAIR-LIVE-QA-RETEST` | Run two-client session; capture QA snapshot; verify hand/board/auction/placement per PROMPT 1472 scope | Human GUI session (two browser windows) |

---

### LANE E — Gameplay Loop Gaps (Blocked Until PROMPT 1472 + Sprint 18 Closes)

These were in LANE 4 of the 1835 map. They remain unstarted. Renumbered to avoid
collision with taken PROMPT numbers.

| Story Gap | Suggested Prompt | Scope | Gate |
|---|---|---|---|
| Round state machine completeness | `PROMPT-1880-ROUND-STATE-MACHINE-AUDIT` | Read-only compare GDD vs `server/src/core/` | None (read-only) |
| Round state machine gap repair | `PROMPT-1881-ROUND-STATE-MACHINE-GAP-REPAIR` | Implement GDD gaps | 1880 + PROMPT 1472 |
| Combat resolution live verify | `PROMPT-1882-COMBAT-RESOLUTION-LIVE-VERIFY` | Two-client combat session; capture log; verify formula | PROMPT 1472 + human |
| Combat keyword integration tests | `PROMPT-1883-COMBAT-KEYWORD-INTEGRATION-TESTS` | Write tests for Speed/Bulwark/Ecaflip interactions | None (test authoring) |
| Sang Méprise ADR-024 live verify | `PROMPT-1884-SANG-MEPRISE-LIVE-VERIFY` | Trigger reveal in live session; confirm unicast delivery | PROMPT 1472 |
| Win condition verify | `PROMPT-1885-WIN-CONDITION-LIVE-VERIFY` | Confirm win/loss propagates to both clients | PROMPT 1472 + human |

**None of these can start until PROMPT 1472 live retest clears.** The QA snapshot
button (`CCGS_QA_SNAPSHOT=1`) exists since `origin/main@8a3744e` and is the
primary evidence tool for this lane.

---

### LANE F — Sprint 18 Should-Have Paperwork (Parallel-Safe Now)

These should-have rows need story-done paperwork but no new source edits. Originally
mapped in 1835 LANE 3 but the predicted prompt numbers are taken. Renumbered.

| Row | Status | Suggested Prompt |
|-----|--------|-----------------|
| S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001 | Implemented; needs story-done | `PROMPT-1890-S18-VIEWPORT-INVARIANT-STORY-DONE` |
| S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 | Implemented; needs story-done | `PROMPT-1891-S18-SNAPSHOT-LAYOUT-FIELDS-STORY-DONE` |
| S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001 | Verify-only candidate | `PROMPT-1892-S18-SETTINGS-PANEL-RELAYOUT-VERIFY` |
| S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001 | Landed via PROMPT 1228; paperwork-only | `PROMPT-1893-S18-HAND-MANA-PREVIEW-STORY-DONE` |
| S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001 | Landed via PROMPT 1239/1243; paperwork-only | `PROMPT-1894-S18-HAND-IDLE-AFFORDANCE-STORY-DONE` |

All five prompts are parallel-safe (no source file edits; sprint tracker + story state only).

---

### LANE G — Sprint 18 Closure + Release Prep (Sequential After D Closes)

| Step | Suggested Prompt | Dependencies |
|------|-----------------|--------------|
| Sprint 18 completion tally | `PROMPT-1895-SPRINT-18-COMPLETION-STATUS-REFRESH` | Lane F done, Lane D in progress |
| Sprint 19 planning draft | `PROMPT-1896-SPRINT-19-PLANNING-DRAFT` | Sprint 18 tally |
| Polish→Release gate-check retry preconditions | `PROMPT-1897-POLISH-RELEASE-GATE-RETRY-PRECONDITIONS` | None (read-only audit of PROMPT 761 findings) |
| Gate-check retry | `PROMPT-1898-POLISH-RELEASE-GATE-CHECK-RETRY` | 1897 + Sprint 18 Must Have closed + PROMPT 1472 done |

---

## Parallelizability Matrix

```
LANE A1 (AC-VPT-01 land 1865)         ──── safe now (no source overlap with B/C/F)
LANE B  (in-flight reports/tools)      ──── safe now (reports + dev-tools only)
LANE C  (Krosmaga cargo test verify)   ──── safe now (client/src/ui/hand, asset_wiring)
LANE F  (S18 should-have paperwork)    ──── safe now (sprint tracker only)
LANE G1 (gate-retry preconditions)     ──── safe now (read-only)
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
1. LANE A1: Re-base and land `integrate/autoplay-window-size-default-1865`
2. LANE B: Integrate branches 1849, 1867, 1841, 1839, 1837
3. LANE C: Cargo test verify for 1853 + 1854 (if disk space available)
4. LANE F: Sprint 18 paperwork (1890–1894)
5. LANE G1: `PROMPT-1897-POLISH-RELEASE-GATE-RETRY-PRECONDITIONS` (read-only audit)

---

## File Ownership / Conflict Risk Map (Current)

| File / Directory | Risk Level | Current Branch Owners |
|---|---|---|
| `tools/autoplay/driver.py` | HIGH | wt-1843 + wt-1844-wt (both pre-main) — must integrate sequentially |
| `tools/autoplay/analyze_evidence_run.py` | MED | PROMPT 1850 branch + PROMPT 1865 (extends it) |
| `tools/autoplay/validate_composite_run.py` | MED | PROMPT 1850 branch |
| `client/src/autoplay.rs` | MED | integrate/autoplay-window-size-default-1865 |
| `tools/autoplay/Run-AutoplaySmoke.ps1` | LOW | integrate/autoplay-window-size-default-1865 |
| `client/src/ui/shop_auction/` | HIGH | D1 (gated) |
| `client/src/presentation/board_rendering.rs` | HIGH | D2 (gated) |
| `client/src/presentation/qa_snapshot.rs` | MED | Lane E (gated) |
| `client/src/ui/hand/mod.rs` | LOW | PROMPT 1854 (Krosmaga) |
| `client/src/asset_wiring.rs` | LOW | PROMPT 1853 (Krosmaga) |
| `server/src/feature/auction/` | MED | D1 (gated) |
| `server/src/core/` | MED | Lane E (gated) |
| `shared/src/protocol.rs` | MED | D1 (gated) |
| `tools/dev-launcher/` | LOW | wt-1837 |
| `tools/autoplay/recipes/` | LOW | integrate/autoplay-placement-reject-recipe-1849 |
| `production/sprints/` | LOW | Lane F paperwork |
| `reports/` | NONE | Any worker |

---

## Release Gap Summary

For M2 (Playable Game) milestone gate: two players must connect, play a full 1v1 game
through auction → placement → combat → shop → win condition. The minimum path:

1. **PROMPT 1472** POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST (overdue; primary unlock)
2. **S18-AUCTION-WON-CARD-DISPOSITION-001** (Lane D1)
3. **S18-UI-PLAY-AREA-CONTAINER-001** (Lane D2)
4. **Combat resolution live verify** (Lane E)
5. **Win-condition verify** (Lane E)
6. **Sprint 18 Must Have closure** (all 4 rows)
7. **Polish→Release gate-check retry** (PROMPT 1898)

Nothing in Lanes A–C is on the M2 critical path; autoplay evidence quality is a
sprint-health concern, not a release gate. Lanes A–C can run in parallel to the M2 path.

---

1869: GAME-COMPLETION-NEXT-WAVE-MAP-REFRESH-AFTER-1844-1858: DONE
