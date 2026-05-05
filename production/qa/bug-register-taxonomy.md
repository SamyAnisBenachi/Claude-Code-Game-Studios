# QA Bug Register Taxonomy

**Status**: Active
**Created**: 2026-05-05
**Scope**: `production/qa/bugs/`

This register classifies QA bugs, QA conditions, evidence gaps, test debt,
accessibility gaps, and performance gaps. It records known validation work
without assigning Sprint 6 capacity.

## Taxonomy

### Kind

| Value | Use |
|---|---|
| Bug | Confirmed product defect with expected versus actual behavior. |
| QA Condition | Residual condition attached to a QA sign-off or gate decision. |
| Evidence Gap | Missing proof needed to verify a requirement or close a gate. |
| Test Debt | Missing, stale, ignored, or insufficient automated test coverage. |
| Accessibility Gap | Missing implementation or verification for committed accessibility requirements. |
| Performance Gap | Missing or failing performance measurement against an accepted budget. |

### Severity

| Value | Meaning |
|---|---|
| S1 Critical | Blocks core play, corrupts state, crashes, leaks secrets, or invalidates release readiness. |
| S2 High | Blocks a gate, major feature validation, or high-risk production decision. |
| S3 Medium | Material QA risk, deferred verification, or non-critical regression exposure. |
| S4 Low | Minor quality issue, documentation gap, or accepted advisory follow-up. |

### Priority

| Value | Meaning |
|---|---|
| P1 Sprint 6 gate blocker | Must be resolved, verified, or explicitly reclassified before the Sprint 6 gate can pass. |
| P2 Sprint 6 validation | Should be addressed during Sprint 6 validation, but does not by itself assign Sprint 6 capacity. |
| P3 Backlog | Valid issue for future planning. |
| P4 Accepted risk candidate | Candidate for explicit accepted-risk decision. |

### Status

| Value | Meaning |
|---|---|
| Open | Filed and awaiting action. |
| In Progress | Work or evidence capture has started. |
| Ready for Verification | Fix or evidence exists and needs QA verification. |
| Verified | QA verified the condition has been satisfied. |
| Closed | Closed after verification or approved disposition. |
| Accepted Risk | Explicitly accepted by the user or producer and retained as a known risk. |

### Action State

| Value | Meaning |
|---|---|
| Needs Evidence | Evidence must be captured or linked. |
| Needs Remediation | Implementation, test, or documentation work is required before verification. |
| Needs Decision | Requires a product, QA, or producer decision before action is clear. |
| Deferred Accepted | Deferred by explicit accepted-risk decision. |

## Record Requirements

Each file in `production/qa/bugs/` should include:

- ID and title.
- Kind, severity, priority, status, and action state.
- Source evidence.
- Closure evidence required.
- Current blocker status.
- Non-goals, including any capacity or scope exclusions.

## Initial Register

| ID | Title | Kind | Severity | Priority | Status | Action State | Blocker Status |
|---|---|---|---|---|---|---|---|
| QA-COND-0001 | AU1-b-network FIFO evidence | QA Condition | S3 Medium | P2 Sprint 6 validation | Open | Needs Evidence | Sprint 5 non-blocking condition; Sprint 6 validation condition |
| QA-COND-0002 | Ignored AUC-006 auction test | Test Debt | S3 Medium | P2 Sprint 6 validation | Closed | N/A - Closed | Closed after AU19-a repair evidence confirmed no ignored auction abort tests |
| QA-COND-0003 | OS-18b two-client objective HP visibility | QA Condition | S3 Medium | P2 Sprint 6 validation | Open | Needs Evidence | Sprint 5 non-blocking condition; live transport visibility remains advisory |
| QA-COND-0004 | Browser/WASM board performance capture | Performance Gap | S2 High | P1 Sprint 6 gate blocker | Closed | N/A - Closed | Closed after BOARD-012 browser/WASM capture passed corrected timing budgets |
| QA-COND-0005 | Standard-tier accessibility gaps | Accessibility Gap | S2 High | P1 Sprint 6 gate blocker | Open | Needs Remediation | Production-to-Polish hard blocker |
| QA-COND-0006 | Playtest/fun-hypothesis evidence | Evidence Gap | S2 High | P1 Sprint 6 gate blocker | Open | Needs Evidence | Production-to-Polish hard blocker |
| QA-COND-0007 | Deferred manual visual evidence | Evidence Gap | S3 Medium | P2 Sprint 6 validation | Open | Needs Evidence | Sprint 5 non-blocking condition; Sprint 6 validation condition |

## Scope Guard

This register does not assign Sprint 6 capacity. Capacity must be planned in the
sprint plan, not in QA condition records.
