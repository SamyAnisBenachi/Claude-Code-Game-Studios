# QA-COND-0001: AU1-b-network FIFO Evidence

| Field | Value |
|---|---|
| ID | QA-COND-0001 |
| Kind | QA Condition |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Open |
| Action State | Needs Evidence |
| Reported | 2026-05-05 |
| Source | Sprint 5 QA sign-off and Production-to-Polish gate check |

## Summary

`AU1-b-network` remains open because the repository does not yet contain ADR-008
Lightyear FIFO integration evidence for the live auction path. Sprint 5 QA
accepted this as a non-blocking condition, but it must remain visible for future
transport validation and gate review.

## Source Evidence

- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records
  `AU1-b-network` as open pending ADR-008 Lightyear FIFO integration evidence.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries the condition forward and recommends resolving or reclassifying it
  with ADR-008 FIFO evidence.

## Expected Closure Evidence

Provide one of the following:

- Passing live or integration evidence that two auction clients preserve
  first-valid-wins FIFO ordering on the intended Lightyear channel.
- A documented reclassification showing why `AU1-b-network` is no longer a
  required gate condition.
- An explicit accepted-risk decision that moves this record to `Accepted Risk`.

## Current Blocker Status

This is not a Sprint 5 close-out blocker. It is a Sprint 6 validation condition
and remains open until evidence, reclassification, or accepted-risk disposition
exists.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit ADR-008, Lightyear code, tests, or sprint status.
- Does not claim live FIFO evidence exists.
