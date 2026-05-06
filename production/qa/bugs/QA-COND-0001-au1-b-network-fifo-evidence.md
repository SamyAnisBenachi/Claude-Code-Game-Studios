# QA-COND-0001: AU1-b-network FIFO Evidence

| Field | Value |
|---|---|
| ID | QA-COND-0001 |
| Kind | QA Condition |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Closed |
| Action State | N/A - Closed |
| Reported | 2026-05-05 |
| Source | Sprint 5 QA sign-off and Production-to-Polish gate check |

## Summary

`AU1-b-network` is resolved by the expanded ADR-008 Lightyear FIFO integration
harness. The original single-client S2C ordering evidence remains accepted as
partial evidence, and the 2026-05-06 repair adds the missing live two-client
first-valid-wins FIFO evidence for same-window competing auction bids.

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

This section is retained as the partial-evidence review that kept the condition
open before the two-client repair below.

## 2026-05-06 QA Disposition Review

Historical disposition before the two-client repair below.

Disposition: Keep `Open / Needs Evidence`.

Reviewed evidence:

- Sprint 6 smoke report: `production/qa/smoke-2026-05-06.md` records
  `auction_fifo_ordering_test` passing during the Sprint 6 smoke run.
- Focused rerun on 2026-05-06:
  `cargo test -p server --test auction_fifo_ordering_test -- --nocapture`.
- Focused rerun result: PASS, 1 passed, 0 failed, 0 ignored.
- Harness reviewed:
  `tests/integration/network/auction_fifo_ordering_test.rs`.

Evidence accepted:

- The harness is acceptable partial evidence that one live Lightyear
  WebSocket client receives `S2CAuctionCard` before
  `S2CPhaseChanged(DraftAuction)` on `ReliableChannel`.
- The harness asserts payload fidelity for the auction card and phase change,
  verifies both messages were observed, and verifies the auction card message id
  and observed receive order precede the draft-auction phase change.

Remaining closure gap:

- The current closure path still asks for passing live or integration evidence
  that two auction clients preserve first-valid-wins FIFO ordering on the
  intended Lightyear channel.
- The current harness uses one live client. It does not prove the ordering
  across two auction clients and does not validate first-valid-wins behavior
  between multiple auction participants.
- No QA closure, reclassification, or accepted-risk disposition has been
  recorded for this condition.

Required next evidence:

- Either add or expand live integration evidence so two auction clients preserve
  the required FIFO/first-valid-wins behavior on the intended Lightyear channel,
  then record QA acceptance; or
- Reclassify this condition with an explicit rationale that narrows closure to
  the single-client cross-message `ReliableChannel` ordering already covered by
  the current harness; or
- Record an explicit accepted-risk disposition.

## 2026-05-06 Two-Client FIFO Evidence Repair

Disposition: Close `Closed / N/A - Closed`.

Evidence added:

- Harness: `tests/integration/network/auction_fifo_ordering_test.rs`.
- New test:
  `two_clients_same_window_duplicate_bid_first_valid_wins_on_reliable_channel`.
- Existing partial S2C FIFO test retained:
  `auction_card_precedes_draft_auction_phase_on_reliable_channel`.
- Focused verification command:
  `cargo test -p server --test auction_fifo_ordering_test -- --nocapture`.
- Focused verification result on 2026-05-06: PASS, 2 passed, 0 failed,
  0 ignored.
- Adjacent auction/network regression command:
  `cargo test -p server --test e2e_websocket_test --test auction_bid_validation_gate_test --test accepted_bid_reservation_test`.
- Adjacent regression result on 2026-05-06: PASS, 15 passed, 0 failed,
  0 ignored across the three targets.
- Server check: `cargo check -p server`: PASS.

Closure rationale:

- The new harness starts one live Lightyear WebSocket server and two live
  Lightyear WebSocket clients.
- Client A is connected and mapped first; Client B is connected and mapped
  second.
- After both clients are connected, each client queues one
  `C2SPlaceBid { amount: 5 }` on `ReliableChannel` in the same armed auction
  window.
- The server collects both client inputs before one authoritative
  `process_bid_batch` call. Before that batch, the auction leader is `None` and
  the current price is the starting price, `4`.
- The server records the observed FIFO bid order, accepts exactly the first
  observed valid bid at amount `5`, rejects the later same-amount conflicting
  bid with `BidRejectedReason::AmountTooLow`, and leaves final auction state
  with only the first bidder as leader.
- Economy assertions prove only the first bidder reserves `5` gold and the
  later conflicting bidder reserves `0`.

This satisfies the requested two-client / first-valid-wins FIFO closure path for
QA-COND-0001. No reclassification or accepted-risk disposition is needed.

## Expected Closure Evidence

Provide one of the following:

- Passing live or integration evidence that two auction clients preserve
  first-valid-wins FIFO ordering on the intended Lightyear channel.
- A documented reclassification showing why `AU1-b-network` is no longer a
  required gate condition.
- An explicit accepted-risk decision that moves this record to `Accepted Risk`.

## Current Blocker Status

Closed. QA-COND-0001 is no longer Sprint 6 validation debt after the expanded
AU1 FIFO harness verified live two-client first-valid-wins behavior for
same-window competing auction bids.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit ADR-008, Lightyear production code, sprint status, or
  session-state records.
- Does not alter QA-COND-0005, QA-COND-0006, or QA-COND-0007.
- Does not claim playable-client manual QA.
