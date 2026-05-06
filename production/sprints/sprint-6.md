# Sprint 6 -- 2026-05-06 to 2026-05-19

## Sprint Goal

Clear the failed Production -> Polish gate blockers through remediation,
evidence capture, verification, and final re-gating, without expanding normal
feature scope.

## Planning Notes

- Sprint 5 is officially closed with conditions.
- Production -> Polish gate failed on 2026-05-05, then passed with conditions
  on 2026-05-06. The project is now in Polish.
- Sprint 6 is a remediation/validation sprint, not a normal feature sprint.
- Pull-forward work already integrated is credited below and is not counted as
  unstarted Sprint 6 capacity.
- PR-SPRINT skipped -- Lean mode. `production/review-mode.txt` is not present,
  so the sprint-plan workflow defaults to `lean`.
- Do not use Sprint 6 capacity for unrelated feature expansion until gate
  blockers are resolved or explicitly reclassified.
- Post-gate reconciliation on 2026-05-06 records S6-06 as complete without
  reopening Sprint 6 or running `/story-done`.

## Capacity

- Total workdays: 10
- Buffer (20%): 2 days reserved for remediation surprises, evidence capture
  friction, and re-gate follow-up
- Available: **8 effective planned days**
- Planned Must Have scope: **8.0 estimated days**
- Should Have and Nice To Have scope is conditional and must not displace
  Production -> Polish remediation.

---

## Tasks

### Must Have (Gate Remediation Critical Path)

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S6-01 | Sprint 6 QA plan and QA condition register tracking/closure | qa-lead + orchestrator | 0.75 | `production/qa/bugs/`; Sprint 5 gate and QA sign-off | Sprint 6 QA plan exists; QA-COND statuses are reconciled, including QA-COND-0002 closure and AU1 FIFO evidence review; open P1/P2 conditions have explicit next action |
| S6-02 | Playtest sessions and fun hypothesis decision | producer + qa-lead | 2.00 | `production/playtests/sprint-6-playtest-protocol.md`; playtest templates | Producer accepted deferral for Sprint 6 on 2026-05-05; no playtests are marked completed, no playtest reports are created, and QA-COND-0006 must carry into the final Production -> Polish gate as an explicit risk/condition rather than passed evidence |
| S6-03 | Browser/WASM board performance evidence | performance analyst + client programmer | 1.50 | BOARD-010 narrowed baseline; browser/WASM harness or equivalent evidence path | 1920x1080 browser/WASM capture records a nonblank board, framed lanes, frame timing against `<=16.67 ms`, and ADR-021 steady-state/spike budget observations |
| S6-04 | Standard-tier accessibility remediation and verification | accessibility owner + UI/client programmer | 2.00 | ADR-023; `design/accessibility-requirements.md`; relevant UX specs | Standard-tier accessibility gaps are remediated, verified, explicitly reclassified, or accepted as risk; verification evidence identifies remaining Standard-tier exposure |
| S6-05 | OS-18b two-client objective HP visibility evidence | network/gameplay programmer + qa-tester | 0.75 | `production/epics/objective-system/story-008-os18b-two-client-objective-hp-visibility.md` | Two-client capture or equivalent end-to-end evidence proves objective HP visibility for both clients after resolution-end sync |
| S6-06 | Final re-smoke, QA sign-off, and Production -> Polish gate-check | orchestrator + qa-lead | 1.00 | S6-01, S6-03, S6-04, S6-05, and the accepted-risk S6-02 disposition | Smoke is `PASS` or `PASS WITH WARNINGS`; QA sign-off is complete; Production -> Polish gate-check is rerun and recorded; QA-COND-0006 is carried as an explicit risk/condition, not as passed playtest evidence |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S6-S1 | SAU-007 readiness refresh and implementation after SAU-006 | UI/client programmer | 1.00 | SAU-004, SAU-005, and SAU-006 Complete | SAU-007 blocker text is refreshed against current completion state; implementation proceeds only after readiness is current and gate remediation remains stable |
| S6-S2 | BLS-012 implementation | gameplay programmer | 1.00 | NP-006 Complete; BLS-012 readiness notes | `SpawnRangeState` becomes the authoritative projection source and emits `SpawnRangeChanged` from the accepted live source |
| S6-S3 | BR-011 readiness after BLS-012 | client programmer + technical artist | 0.50 | BLS-012 Complete | Board Rendering spawn-range highlight story is unblocked and ready; no implementation starts before BLS-012 closure |
| S6-S4 | Deferred visual/manual QA evidence | qa-tester + UI/client programmer | 1.00 | Relevant UI states renderable enough for evidence | Placement timer urgency/checkmark, reserve strip affordance, submit validation inline feedback, and resolution replay readability evidence is captured or explicitly reclassified |

### Nice To Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S6-N1 | Evidence index/backlink cleanup | orchestrator | 0.25 | Must Have evidence complete | Gate evidence paths are easy to audit from QA, smoke, sign-off, and gate-check reports |
| S6-N2 | SAU-008 readiness only if SAU-007 lands early | UI/client programmer | 0.50 | SAU-007 Complete; Must Have gate remediation stable | SAU-008 readiness is refreshed without implementation unless gate blockers are already resolved |

---

## Post-Gate Status

Sprint 6 final gate reconciliation is complete as of 2026-05-06.

| Item | Final status | Evidence |
|------|--------------|----------|
| Stage | Polish | `production/stage.txt` |
| S6-06 | Done | Smoke `PASS WITH WARNINGS`, QA sign-off `APPROVED WITH CONDITIONS`, and gate `PASS WITH CONDITIONS` |
| Gate report | PASS WITH CONDITIONS | `production/gate-checks/gate-production-polish-sprint-6-2026-05-06.md` |
| Gate commit | Recorded | `f622605872856ea29014601db5a9996c8b3d1122` |

Current condition state after post-gate QA closure reconciliation:

- QA-COND-0005 is friend-game-only accepted risk. This is not verified
  Standard-tier accessibility completion and must be revisited before any
  public, external, commercial, or broader release candidate.
- QA-COND-0006 is accepted-risk/deferred. This is not playtest evidence and is
  not fun-hypothesis validation.
- QA-COND-0001 is Closed / N/A - Closed via integrated two-client FIFO evidence
  at `8fc30de33526d2f44e53d7e1fa82673ee4f26ccf`.
- QA-COND-0007 is Closed / N/A - Closed via Hand UI visual evidence and
  resolution replay readability evidence.
- Full manual playable-client QA is not claimed by the smoke report, QA
  sign-off, gate report, or this reconciliation.

S6-04 is not an active Sprint 6 blocker after the producer's friend-game-only
accepted-risk decision for QA-COND-0005.

## Pull-forward Credit Already Integrated

The following work is already integrated or documented before Sprint 6 planning.
It is credited as remediation progress and must not be double-counted as
unstarted Sprint 6 capacity:

- SAU-003
- SAU-004
- SAU-005
- SAU-006
- BOARD-005
- BOARD-009 narrowed status/co-occupancy
- BOARD-010 narrowed baseline
- NP-006
- BLS-012 readiness
- BOARD-006 readiness
- OS-18b Story 008 docs
- Spawn range docs repair
- Playtest package and fun hypothesis decision docs
- QA condition register
- AU19-a repair / QA-COND-0002 closure
- AU1 FIFO harness
- ADR-023 timer accessibility authority

## Blocked / Deferred

| Item | Status | Blocker / Deferral |
|------|--------|--------------------|
| BR-011 | Blocked | Blocked until BLS-012 is complete and the authoritative spawn range projection exists |
| SAU-009 | Blocked | Blocked until panel states from prerequisites are renderable and evidence prerequisites are satisfied |
| Final board visual/browser evidence | Closed for Sprint 6 P1 gate | BOARD-012 browser/WASM capture exists with nonblank 1920x1080 board evidence and corrected timing verdict PASS |
| QA-COND-0004 | Closed | BOARD-012 browser/WASM capture passed corrected timing budgets |
| QA-COND-0005 | Accepted-risk / producer waived | Producer accepted remaining Standard-tier accessibility exposure as friend-game-only risk: Standard-tier accessibility waived by producer; no public release, no obligation. Not verified accessibility completion. |
| QA-COND-0006 | Accepted-risk / deferred | Producer accepted deferral for Sprint 6 on 2026-05-05; required playtest sessions and aggregate fun decision remain future closure evidence, and the final Production -> Polish gate carried this as an explicit risk/condition rather than passed evidence |
| QA-COND-0001 | Closed / N/A - Closed | Closed via integrated two-client FIFO evidence at `8fc30de33526d2f44e53d7e1fa82673ee4f26ccf` |
| QA-COND-0003 | Closed | OS-18b two-client ObjectiveHp visibility evidence verified final-only observations for both clients |
| QA-COND-0007 | Closed / N/A - Closed | Closed via Hand UI visual evidence and resolution replay readability evidence |

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Standard-tier accessibility scope may exceed one remediation sprint | HIGH | HIGH | Verify, remediate, or explicitly reclassify gaps; do not leave broad `Not Started` Standard-tier rows unexplained |
| Browser/WASM performance capture may need harness work before measurement | HIGH | HIGH | Treat harness/evidence capture as part of S6-03; preserve BOARD-010 native baseline as supporting evidence only |
| Deferred playtest evidence may leave the fun hypothesis unvalidated | MEDIUM | HIGH | Use the Sprint 6 fun-hypothesis decision file as the authoritative accepted-risk disposition; do not mark playtests complete, and carry QA-COND-0006 into the final Production -> Polish gate as an explicit risk/condition |
| QA condition taxonomy and condition files may drift | MEDIUM | MEDIUM | Reconcile condition files and taxonomy during S6-01 before final QA sign-off |
| Should Have implementation may compete with gate remediation | MEDIUM | MEDIUM | Start Should Have only after Must Have work is stable or blocked on external evidence timing |

## Dependencies on External Factors

- Testers and session time are not assumed for Sprint 6 because S6-02 /
  QA-COND-0006 has a producer accepted-risk deferral.
- Browser/WASM local or CI environment can run the board baseline capture.
- QA can verify accessibility and visual/manual evidence against the browser
  target.
- Network/two-client evidence environment can run OS-18b visibility checks.

## QA Plan

Sprint 6 QA plan exists at
`production/qa/qa-plan-sprint-6-2026-05-05.md`. Use it as the current source for
condition reconciliation, remediation validation, smoke scope, QA sign-off, and
Production -> Polish gate-check prerequisites.

## Definition of Done for this Sprint

- [x] All Must Have tasks completed, closed, or explicitly accepted as risk for
      the Sprint 6 Production -> Polish gate.
- [x] Sprint 6 QA plan exists
- [x] P1 QA conditions are closed, verified, explicitly reclassified, or accepted as risk
- [x] P2 QA validation conditions have evidence, reclassification, or accepted-risk disposition
- [x] S6-02 / QA-COND-0006 accepted-risk deferral is recorded without marking
      playtests completed or creating playtest reports
- [x] Production -> Polish gate-check carries QA-COND-0006 as an explicit
      risk/condition, not as passed playtest evidence
- [x] Browser/WASM board performance evidence is captured and compared against budgets
- [x] Standard-tier accessibility remediation/verification is documented as
      friend-game-only accepted risk, not verified Standard-tier completion
- [x] OS-18b two-client objective HP visibility evidence is captured or explicitly reclassified
- [x] Smoke check passed as `PASS` or `PASS WITH WARNINGS`
- [x] QA sign-off report is complete
- [x] Production -> Polish gate-check rerun is recorded
- [x] Pull-forward credit remains separated from unstarted Sprint 6 capacity

---

**Scope check:** Sprint 6 is a remediation/validation sprint. If any normal
feature work is proposed beyond the listed Should Have or Nice To Have items,
run `/scope-check` before implementation begins and confirm it does not displace
gate remediation.
