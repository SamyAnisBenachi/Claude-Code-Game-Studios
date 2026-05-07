# Sprint 7 Friend-Game Evidence Index

Date: 2026-05-07
Scope: Sprint 7 internal friend-game playable path only

## Summary

Sprint 7 Must Have evidence is complete for PLAYABLE-001, PLAYABLE-002, and
PLAYABLE-003. This index exists only to make the friend-game evidence easy to
audit from one place.

Verified endpoint: next-loop DRAFT_SHOP after post-auction placement/resolution.

Full observed route:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.

Game-over was not reached and is not claimed.

## Story Evidence

| Story | Status | Evidence |
|---|---|---|
| PLAYABLE-001 Primary Client Bootstrap + Fresh Lobby Entry | Complete | [playable-client-lobby-entry.md](playable-client-lobby-entry.md) |
| PLAYABLE-002 Live Draft/Shop/Hand Bridge | Complete | [playable-client-draft-shop-hand-bridge.md](playable-client-draft-shop-hand-bridge.md) |
| PLAYABLE-003 Real End-to-End Loop Verification | Complete | [playable-client-real-e2e-loop.md](playable-client-real-e2e-loop.md) |

## Capture Directory And Trace Artifacts

Primary capture directory:

- [captures/playable-client-real-e2e-loop/](captures/playable-client-real-e2e-loop/)

Key trace artifacts:

- [prompt-290-room-session-trace.json](captures/playable-client-real-e2e-loop/prompt-290-room-session-trace.json) - fresh hello, host create-room, joiner join-room, class confirm/reveal, and server-confirmed session entry.
- [prompt-296-draft-shop-trace.json](captures/playable-client-real-e2e-loop/prompt-296-draft-shop-trace.json) - DRAFT_INITIAL offering, purchase/acquisition/economy, ready/retract, placement submit, resolution, and first DRAFT_SHOP.
- [prompt-298-auction-placement-resolution-trace.json](captures/playable-client-real-e2e-loop/prompt-298-auction-placement-resolution-trace.json) - DRAFT_SHOP, non-empty placement, resolution, auction card/bid/settlement, AuctionWon acquisition, post-auction placement/resolution, and next-loop DRAFT_SHOP.
- [phase-captures.md](captures/playable-client-real-e2e-loop/phase-captures.md) - reached and unreached phase notes.

PLAYABLE-001 and PLAYABLE-002 do not have separate story-local capture
directories. Their evidence documents record automated evidence, commands, and
message-path coverage; PLAYABLE-003 owns the real friend-game loop capture
package.

## Supporting Planning Links

- Sprint plan: [../../sprints/sprint-7.md](../../sprints/sprint-7.md)
- Sprint status: [../../sprint-status.yaml](../../sprint-status.yaml)
- Sprint 7 QA plan: [../qa-plan-sprint-7-2026-05-06.md](../qa-plan-sprint-7-2026-05-06.md)

## Non-Claims

Forbidden claims: no public release readiness, no broad accessibility completion, no playtest/fun-hypothesis validation, no full playable-client manual QA, no full game completion.

QA-COND-0005 remains accepted risk for friend-game scope only and is not
verified Standard-tier accessibility completion. QA-COND-0006 remains
accepted-risk/deferred and is not playtest evidence or fun-hypothesis
validation.
