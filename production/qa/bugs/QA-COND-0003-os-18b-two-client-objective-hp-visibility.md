# QA-COND-0003: OS-18b Two-Client Objective HP Visibility

| Field | Value |
|---|---|
| ID | QA-COND-0003 |
| Kind | QA Condition |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Open |
| Action State | Needs Evidence |
| Reported | 2026-05-05 |
| Source | Sprint 5 QA sign-off and Production-to-Polish gate check |

## Summary

OS-18b objective HP replication is covered by automated objective-resolution
logic, but live two-client visibility remains advisory. The repository needs
evidence that two live clients both see objective HP updates correctly after
resolution-end sync.

## Source Evidence

- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records OS-18b live
  two-client objective HP replication visibility as advisory.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries OS-18b live two-client visibility forward as an open QA condition.

## Expected Closure Evidence

Provide one of the following:

- A live two-client capture showing objective HP visibility after
  resolution-end sync for both clients.
- An automated end-to-end transport test that proves the same visibility
  contract.
- A documented reclassification explaining why live two-client visibility is no
  longer required for this gate.

## Current Blocker Status

This is not a Sprint 5 close-out blocker. It is a Sprint 6 validation condition
for live transport visibility.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit objective, HUD, board rendering, or networking code.
- Does not claim live two-client evidence exists.
