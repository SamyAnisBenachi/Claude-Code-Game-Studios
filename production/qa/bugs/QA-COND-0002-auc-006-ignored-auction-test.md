# QA-COND-0002: Ignored AUC-006 Auction Test

| Field | Value |
|---|---|
| ID | QA-COND-0002 |
| Kind | Test Debt |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Closed |
| Action State | N/A - Closed |
| Reported | 2026-05-05 |
| Source | Sprint 5 smoke report and QA sign-off |

## Summary

QA-COND-0002 is resolved after the AU19-a repair in commit `2bf7078`
(`fix auction abort resolving settlement guard`). The formerly ignored auction
abort settlement guard is now active and passing with no ignored tests in the
auction abort handler target.

## Source Evidence

- `production/qa/smoke-2026-05-05.md` records the auction/displacement batch as
  passing with one ignored test.
- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records one auction abort
  test intentionally ignored for older AUC-006 settlement scope.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries the ignored auction test forward as an open QA condition.
- AU19-a repair commit: `2bf7078` (`fix auction abort resolving settlement
  guard`).

## Expected Closure Evidence

Satisfied by the first closure path: the ignored test was updated, unignored,
and passes in the relevant auction test target.

## Closure Evidence

Captured 2026-05-05 from `D:\_DEV\claude-code-game-studios`.

- AU19-a repair commit: `2bf7078` (`fix auction abort resolving settlement
  guard`).
- `rg -n "#\\[ignore\\]" tests/unit/auction/auction_abort_handler_test.rs`
  returned no matches.
- `cargo test -p server --test auction_abort_handler_test` passed: 4 passed;
  0 failed; 0 ignored; 0 measured; 0 filtered out.

## Current Blocker Status

Closed. QA-COND-0002 is no longer Sprint 6 validation test debt after the
AU19-a repair evidence confirmed no ignored auction abort tests and a passing
auction/displacement regression batch.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit auction code or tests.
- Does not change the ignored-test state.
