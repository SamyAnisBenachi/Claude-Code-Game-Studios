# Team-QA Report: Sprint 10 Close-Out

**Date**: 2026-05-11
**Sprint**: Sprint 10
**Stage**: Polish
**Engine**: Bevy 0.18 + Lightyear 0.26
**Scope**: Friend-game (not public release)
**Skill**: `/team-qa sprint` — PROMPT 675
**Commit Under Review**: `5634b8f2bd146e597e2b27eb26fed8923dc12602` (main)
**Mode**: Lean (no `production/review-mode.txt`; QL-TEST-COVERAGE + LP-CODE-REVIEW gates skipped per `feedback_paw_review_flow.md`)
**Source-of-Truth Refs**:

- `production/sprint-status.yaml` (Sprint 10 stories)
- `production/sprints/sprint-10.md` (Sprint plan)
- `production/qa/bugs/QA-COND-000{1..7}.md` (carried conditions)
- `production/session-state/active.md` (Sprint 10 closure trail)

---

## Verdict: APPROVED WITH CONDITIONS

Sprint 10 delivered all 6 Must-Have stories and 2 of 3 Should-Have stories.
No S1 (Critical) or S2 (Major) bugs are open against any delivered story. All
pre-existing test failures surfaced during Sprint 10 (Findings A–D) are
non-regressions and are eligible for carry into Sprint 11 as tech debt. Two
producer-waived S2 conditions (QA-COND-0005 accessibility, QA-COND-0006
playtest) remain explicitly accepted-risk for friend-game scope. One open
manual-route gap (S8-QA-001-W1) remains carried.

Conditions attached to this approval are listed in Section 5.

---

## Phase 0: Gate Audit — PROMPT 674 Smoke-Check

**Status: USER-ASSERTED, UNVERIFIED-FROM-FILE.**

The user-issued PROMPT 675 declares: "Gate: 674 must PASS or PASS WITH WARNINGS
before this fires." However, no `production/qa/smoke-sprint-10-*.md` file
exists on disk. The most recent smoke artifact is
`production/qa/smoke-sprint-8-2026-05-07.md` (PASS WITH WARNINGS, commit
`3cc620c`). Sprint 9 and Sprint 10 have no dedicated smoke files.

This Team-QA report therefore performs an **internal derived smoke-check
verdict** in Phase 2 using automated test evidence already on `main`, in lieu
of the missing PROMPT 674 artifact. The derived verdict is recorded in
Section 2 (`PASS WITH WARNINGS`). Condition C-4 below requires a proper
`/smoke-check sprint` run before any `/gate-check` is claimed.

### Sprint 10 Must-Have + Should-Have Story Status (verified from `sprint-status.yaml`)

| Tier | ID | Title | Status | Closed |
|---|---|---|---|---|
| Must-Have | S10-PAW-001 | PAW-002..006 close-out batch | done | 2026-05-10 |
| Must-Have | S10-TD-001 | Test-fixture cascade-fail repair | done | 2026-05-10 |
| Must-Have | S10-TD-002 | Plugin-registration audit | done | 2026-05-10 |
| Must-Have | S10-CARRY-001 | Sprint 9 carry-over consolidation | done | 2026-05-10 |
| Must-Have | S10-POLISH-001 | HUD visual chrome (timer + figurines + RESOLUTION dim) | done | 2026-05-10 |
| Must-Have | S10-POLISH-002 | Shop/Auction panel chrome wiring | done | 2026-05-10 |
| Should-Have | S10-POLISH-003 | Lobby visual chrome | done | 2026-05-10 |
| Should-Have | ECO-004 | Kill/Objective Awards reward-loop polish | done | 2026-05-10 |
| Should-Have | S10-TD-003 | Doc hygiene tech debt sweep | **ready** (not closed) | — |
| Nice-to-Have | S10-N1 | Sprint 10 evidence index | ready (not closed) | — |
| Nice-to-Have | S10-N2 | Friend-game route readability notes | ready (not closed) | — |

**Must-Have closure**: 6/6 done. **Should-Have closure**: 2/3 done.
S10-TD-003 is doc-only zero-risk work; condition C-5 defers it to Sprint 11.

---

## Phase 1: Story Classification (qa-lead)

| Story | Type | Auto Test Evidence | Manual Evidence | Result | Notes |
|---|---|---|---|---|---|
| S10-PAW-001 (PAW-002..006 batch) | Visual/UI (asset wiring) | `tests/integration/presentation/hand_ui_asset_wiring_test.rs`, `shop_auction_asset_wiring_test.rs`, `hud_asset_wiring_test.rs`, `board_asset_wiring_test.rs`, `lobby_asset_wiring_test.rs` — 5 test files present | Evidence docs deferred per friend-game-lite | PASS WITH ADVISORY | `hud_asset_wiring_test.rs` 0/6 (Finding A); `lobby_asset_wiring_test.rs` 12× E0596 compile errors (Finding D). Both pre-existing, both ADVISORY for visual story type. |
| S10-TD-001 | Integration (test infra) | `cargo test -p server` fixture suite — 4/4 ACs per PROMPT 611; scope expanded 14→~57 fixtures; PROMPT 606 verification recorded | AC4 evidence doc deferred | PASS WITH ADVISORY | Test-only changes per spec. Test-helper `placeholder_assets_for_tests` in production source under `#[cfg(test)]` recorded as deviation. |
| S10-TD-002 | Config/Data (audit) | Audit doc at `production/qa/evidence/sprint-10-plugin-registration-audit.md` (0648deb) | Audit doc IS the evidence | PASS | 14/14 server + 14/14 client = 0 silent dead plugins. Commits 0648deb, bbdb91e, 8932d8c, f06271a all on main. |
| S10-CARRY-001 | Config/Data (paperwork) | sprint-status.yaml diff | sprint-status.yaml is the evidence | PASS | 3/3 ACs. Carried conditions (S8-QA-001-W1, QA-COND-0005, QA-COND-0006) explicitly preserved. |
| S10-POLISH-001 | Integration + Visual | `tests/integration/hud/hud_resolution_dim_test.rs` 8/8 PASS (covers AC1/3/4/5/6/7/8); integration commit `b780f0e` | `production/qa/evidence/sprint-10-hud-chrome-evidence.md` (manual capture deferred) | PASS | Dim-overlay logic is `S2CPhaseChanged`-driven, automated coverage met. |
| S10-POLISH-002 | Visual/UI (asset wiring) | `tests/integration/shop_auction_ui/chrome_wiring_test.rs` 4/4 PASS; SAU-007 7/7 + SAU-008 6/6 regression intact | `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md` (AC-3/AC-7 screenshot deferred) | PASS WITH ADVISORY | Auction border ramp out of scope; SHOP_PANEL_CHROME_ASSET reuse on auction panel = PAW-TD-003-a accept-risk. |
| S10-POLISH-003 | Visual/UI (asset wiring) | `tests/integration/session/lobby_chrome_wiring_test.rs` 5/5 PASS; integration commit `084129c` | `production/qa/evidence/sprint-10-lobby-chrome-evidence.md` | PASS WITH ADVISORY | 6/7 ACs pass; AC-5 ADVISORY — `lobby_asset_wiring_test.rs` PAW-006 compile breakage is pre-existing (Finding D), not introduced. |
| ECO-004 | Logic (economy formula) | 12/12 ACs claimed per PROMPT 640+650; integration commit `9fb8e60` | None required for Logic type | PASS WITH NOTE | Test file path **claimed but not file-verified by qa-lead from PROMPT 675 data**. Condition C-2. |

**Test Evidence Audit (qa-tester verification)**: All ten listed test/evidence
files in scope **exist on disk** and are structurally correct for their stated
scope. No file was missing. The two pre-existing test failures (Findings A
and D) are confirmed non-regressions: production lobby code is independently
validated by `lobby_chrome_wiring_test.rs` passing against `LobbyUiPlugin`.

---

## Phase 2: Smoke Check Verdict (derived in lieu of PROMPT 674 artifact)

### Verdict: PASS WITH WARNINGS

**Automated evidence available on `main` (derived from story closures)**:

- `cargo test -p server` fixture suite substantially repaired (S10-TD-001; ~57 fixtures no longer panic-cascade on `Messages<T>`).
- Plugin registration clean: 0 silent dead plugins in server and client (S10-TD-002).
- HUD dim overlay integration: 8/8 PASS (S10-POLISH-001).
- Shop/Auction chrome wiring: 4/4 + SAU-007 7/7 + SAU-008 6/6 (S10-POLISH-002).
- Lobby chrome wiring: 5/5 PASS (S10-POLISH-003).
- ECO-004: 12/12 ACs claimed (not file-verified — see Condition C-2).
- `cargo check -p server` and `cargo check -p client` passing per story closure notes.

**Known pre-existing failures on `main` (NOT regressions from Sprint 10)**:

| Finding | Description | Status | Disposition |
|---|---|---|---|
| A | `hud_asset_wiring_test.rs` 0/6 (PAW-004 timer-bar `Name` break) | Pre-existing | Carry to S11 as tech debt |
| B | `hud_plugin_scaffold_test` 3/4 (PAW-004 timer-bar `Name` break) | Pre-existing | Carry to S11 as tech debt |
| C | Broken `*_harness.rs` binaries (Bevy 0.18 Input feature reorg) | Pre-existing | Carry to S11 as tech debt |
| D | `lobby_asset_wiring_test.rs` 12× E0596 compile errors (PAW-006 Bevy 0.18 `world()` API mismatch) | Pre-existing | Carry to S11 — candidate `S11-TD-PAW-006-COMPILE-001` (already proposed at PROMPT 649) |

**Warnings recorded**:

- W1 — No dedicated Sprint 10 smoke file was authored as a discrete `/smoke-check sprint` artifact. PROMPT 674 gate is user-asserted; this report's smoke verdict is derived from story-closure evidence.
- W2 — ECO-004 test evidence file path not independently file-verified.
- W3 — `hud_asset_wiring_test.rs` and `lobby_asset_wiring_test.rs` remain failing on main (Findings A and D) and must be carried to S11.

---

## Phase 3: Bug Triage & Open Conditions Disposition

### Open Bugs Register

| ID | Severity | Status | Sprint 10 Disposition | Friend-Game Rationale |
|---|---|---|---|---|
| QA-COND-0001 (AU1-b FIFO evidence) | S3 | **Closed** (in file) | No action | Two-client FIFO harness passed 2026-05-06. Not carried. |
| QA-COND-0002 (auc-006 ignored auction test) | — | **Closed** (in file) | No action | AU19-a repair `2bf7078` confirmed 0 ignored tests. Not carried. |
| QA-COND-0003 (OS-18b two-client objective HP) | — | **Closed** (in file) | No action | OS-008 harness verified two-client visibility. Not carried. |
| QA-COND-0004 (browser/WASM board performance) | — | **Closed** (in file) | No action | BOARD-012 browser/WASM capture passed all three budgets. Not carried. |
| QA-COND-0005 (Standard-tier accessibility) | S2 | Accepted Risk (producer waived 2026-05-06) | **ACCEPT-RISK**, carry to S11 | Producer waiver on file. Friend-game-only; does not apply to any future public/external release. |
| QA-COND-0006 (playtest/fun-hypothesis evidence) | S2 | Accepted Risk / Deferred (producer 2026-05-05) | **ACCEPT-RISK**, carry to S11 | Producer deferral on file. Friend-game internal evidence only. Underlying gap remains real for any public release gate. |
| QA-COND-0007 (deferred manual visual evidence) | — | **Closed** (in file) | No action | All listed deferred paths evidenced 2026-05-06. Not carried. |
| S8-QA-001-W1 (manual/browser two-client GAME_OVER gap) | S3 | Open — tracked in `sprint-status.yaml` `carried_conditions` (no standalone bug file) | **DEFER-TO-S11** | Automated GAME_OVER route covered via in-process Lightyear harnesses; gap is the absence of two-window browser run, not a broken code path. Friend-game scope does NOT auto-waive — this is a gameplay-path reachability gap, not an accessibility gap. |
| Findings A–D (pre-existing test failures) | S3 (test debt) | Open — surfaced during Sprint 10 | **DEFER-TO-S11** | Test/test-infra debt only; no production-path impact. Candidate `S11-TD-PAW-006-COMPILE-001` for Finding D already proposed. |

### Bug Triage Summary

- **S1 Critical bugs open**: **0**
- **S2 High bugs open against delivered features**: **0**
- **S2 Producer-waived / accepted-risk**: 2 (QA-COND-0005, QA-COND-0006)
- **S3 Medium open (deferred to S11)**: S8-QA-001-W1 + Findings A–D
- **New bugs filed this cycle**: 0 (qa-tester audit surfaced no new S1/S2 issues)

### Friend-Game Scope Rule Application

Per the user-specified rule for PROMPT 675: "accept-risk on accessibility-tier
ONLY (QA-COND-0005); other quality items must be addressed or explicitly
accept-risked with rationale". This report applies the rule as follows:

- **QA-COND-0005** — auto-waivable accessibility tier. ACCEPT-RISK.
- **QA-COND-0006** — playtest/fun-hypothesis. NOT auto-waivable under the
  rule. Explicit accept-risk rationale recorded: producer reclassified out of
  active remediation 2026-05-05; this is a release-gate-blocking condition for
  any future public release and must not be silently dropped.
- **S8-QA-001-W1** — manual two-window route gap. NOT auto-waivable.
  Explicit defer rationale recorded: automated coverage is sufficient for
  friend-game close-out; manual route remains carried into S11.
- **Findings A–D** — test-debt only. NOT auto-waivable. Explicit defer
  rationale recorded: pre-existing, not introduced by S10 stories, no
  production-path impact, candidate stories already scoped for S11.
- **S10-TD-003, S10-N1, S10-N2** — zero-risk doc/index work not closed.
  Explicit defer rationale recorded: deferring to S11 carries zero technical
  risk; alternative is dropping them with rationale (not recommended).

---

## Phase 4: Verdict Conditions

The APPROVED WITH CONDITIONS verdict carries the following five conditions.
Each must be satisfied or formally accepted-risked before `/gate-check
Polish→Release` (PROMPT 676) is run.

### Condition C-1 — S8-QA-001-W1 explicit S11 carry

S8-QA-001-W1 (manual/browser two-client GAME_OVER gap) must be carried
explicitly into Sprint 11 `carried_conditions` in `sprint-status.yaml` and
must NOT be silently dropped. Friend-game scope does not waive a gameplay
reachability gap.

### Condition C-2 — ECO-004 test-evidence file verification

ECO-004's test file path must be verified against disk at S11 sprint start.
Any missing test evidence for a Logic-type story must be treated as a
BLOCKING gate failure before ECO-004 is claimed done in any future release
scope. This Team-QA cycle did not file-verify the ECO-004 unit test path.

### Condition C-3 — Findings A, B, C, D carried to S11 as tech debt

The four pre-existing test failures must be tracked as Sprint 11 candidate
tech-debt stories before any build is promoted toward public/external
release. Finding D candidate (`S11-TD-PAW-006-COMPILE-001`) is already
proposed.

### Condition C-4 — Proper `/smoke-check sprint` artifact

No discrete Sprint 10 smoke-check file exists. Before `/gate-check
Polish→Release` (PROMPT 676), a proper `/smoke-check sprint` run must
produce `production/qa/smoke-sprint-10-2026-05-11.md` (or equivalent dated
file) so the gate-check has a file-verifiable smoke artifact.

### Condition C-5 — S10-TD-003 / S10-N1 / S10-N2 disposition

S10-TD-003 (doc hygiene), S10-N1 (evidence index), S10-N2 (route readability
notes) are formally deferred to Sprint 11. They must appear in the S11
sprint plan or be explicitly dropped with written rationale in
`sprint-status.yaml`.

---

## Phase 5: Verdict Rule Reconciliation

Verdict rules from `/team-qa` skill (lines 186–188):

- **APPROVED**: All stories PASS or PASS WITH NOTES; no S1/S2 bugs open.
- **APPROVED WITH CONDITIONS**: S3/S4 bugs open, or PASS WITH NOTES issues documented; no S1/S2 bugs.
- **NOT APPROVED**: Any S1/S2 bugs open; or stories FAIL without documented workaround.

**Applied to Sprint 10**:

- All 8 delivered stories PASS or PASS WITH ADVISORY/NOTE. None FAIL.
- 0 S1 bugs open. 0 S2 bugs open against delivered features. 2 S2 conditions
  remain explicitly producer-waived (QA-COND-0005, QA-COND-0006) — these
  are not "open bugs against delivered features" but standing
  release-scope-dependent conditions.
- Findings A–D are S3 test-debt items, all pre-existing, all carried to S11.
- S8-QA-001-W1 is S3 manual-route gap, carried to S11.

→ **APPROVED WITH CONDITIONS** satisfies the rule.

---

## Phase 6: Non-Claims

This Team-QA report does NOT claim:

- Public release readiness.
- Release-candidate readiness.
- Store readiness or deployment readiness.
- Full playable-client manual QA.
- Two-window manual or browser GAME_OVER route completion.
- Broad Standard-tier accessibility completion.
- Closure of QA-COND-0005.
- Playtest validation or fun-hypothesis validation.
- Closure of QA-COND-0006.
- Full game completion.
- Asset production approval.
- Final visual polish completion.
- `/gate-check Polish→Release` PASS.

This report is internal QA close-out paperwork for friend-game scope only.

---

## Phase 7: Next Step Guidance

Run `/gate-check Polish→Release` as PROMPT 676 **only after**:

1. Condition C-4 is satisfied (a proper `/smoke-check sprint` produces a file at `production/qa/smoke-sprint-10-*.md`), and
2. Condition C-1 is recorded (S8-QA-001-W1 carried explicitly in Sprint 11 activation block in `sprint-status.yaml`).

At gate-check time, carry **QA-COND-0005** and **QA-COND-0006** as explicitly
accepted-risk conditions in the gate-check report (not silent omissions), so
the gate accurately reflects the project's real risk posture.

---

## Phase 8: Changed Files

- `production/qa/team-qa-sprint-10-2026-05-11.md` (this report)

No source code, `production/sprint-status.yaml`, `/story-done` records,
QA sign-off, `/gate-check` reports, smoke-check files, Sprint 10 close-out
files, asset approval, or bug-register entries are changed by this report.
