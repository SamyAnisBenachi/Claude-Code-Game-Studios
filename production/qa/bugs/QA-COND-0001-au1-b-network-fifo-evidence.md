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

`AU1-b-network` remains open pending QA disposition of the ADR-008 Lightyear
FIFO integration harness added after Sprint 5 close-out. Sprint 5 QA accepted
this as a non-blocking condition, but it must remain visible until the harness
is run, recorded, and accepted as closure evidence or the condition is otherwise
reclassified.

## Source Evidence

- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records
  `AU1-b-network` as open pending ADR-008 Lightyear FIFO integration evidence.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries the condition forward and recommends resolving or reclassifying it
  with ADR-008 FIFO evidence.
- Commit `bf3ef3dd734bf8c8e0bab0b9094282c7680ab4fb` adds the AU1 FIFO harness
  and registers it as a server integration test target.

## AU1 FIFO Evidence Review

Evidence now available for QA review:

- Commit: `bf3ef3dd734bf8c8e0bab0b9094282c7680ab4fb`
  (`test: add auction FIFO ordering harness`).
- Harness: `tests/integration/network/auction_fifo_ordering_test.rs`.
- Cargo target registration: `server/Cargo.toml`, test target
  `auction_fifo_ordering_test`.
- Expected verification command:
  `cargo test -p server --test auction_fifo_ordering_test`.

The harness exercises a live Lightyear WebSocket server/client pair and sends
`S2CAuctionCard` followed by `S2CPhaseChanged(DraftAuction)` on
`ReliableChannel`. The client records message ids and observed receive order to
prove the auction card arrives before the phase change on the intended ordered
reliable channel.

QA disposition is still required. This condition should remain `Open / Needs
Evidence` until the command result is captured in QA evidence and the QA lead or
orchestrator explicitly accepts it as closure evidence, reclassifies the
condition, or records accepted risk.

## Expected Closure Evidence

Provide one of the following:

- Passing live or integration evidence that two auction clients preserve
  first-valid-wins FIFO ordering on the intended Lightyear channel.
- A documented reclassification showing why `AU1-b-network` is no longer a
  required gate condition.
- An explicit accepted-risk decision that moves this record to `Accepted Risk`.

## Current Blocker Status

This is not a Sprint 5 close-out blocker. It is a Sprint 6 validation condition
and remains open until the AU1 FIFO harness result is captured and accepted as
closure evidence, or until reclassification or accepted-risk disposition exists.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit ADR-008, Lightyear code, tests, or sprint status.
- Does not claim QA has already verified or accepted the AU1 FIFO harness.
