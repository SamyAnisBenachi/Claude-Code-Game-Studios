# QA-COND-0002: Ignored AUC-006 Auction Test

| Field | Value |
|---|---|
| ID | QA-COND-0002 |
| Kind | Test Debt |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Open |
| Action State | Needs Decision |
| Reported | 2026-05-05 |
| Source | Sprint 5 smoke report and QA sign-off |

## Summary

One auction abort test remains intentionally ignored for older AUC-006 settlement
scope. Sprint 5 QA accepted the ignored test as a condition, not a blocking
failure, but the test debt needs an explicit disposition before the register can
close it.

## Source Evidence

- `production/qa/smoke-2026-05-05.md` records the auction/displacement batch as
  passing with one ignored test.
- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records one auction abort
  test intentionally ignored for older AUC-006 settlement scope.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries the ignored auction test forward as an open QA condition.

## Expected Closure Evidence

Provide one of the following:

- The ignored test is updated, unignored, and passes in the relevant auction
  test target.
- The test is retired with a documented reason showing the covered behavior is
  obsolete or superseded.
- A producer or QA decision explicitly accepts the ignored test as a known risk.

## Current Blocker Status

This is not a Sprint 5 close-out blocker. It is Sprint 6 validation test debt
until QA decides whether to remediate, retire, or accept the ignored test.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit auction code or tests.
- Does not change the ignored-test state.
