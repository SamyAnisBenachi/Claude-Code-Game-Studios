# PROMPT 1565 — Placement Accepted ACK Protocol Integration Refresh

## Summary

Refreshed PROMPT 1546 payload onto current `origin/main@f19ab3ea` via clean
cherry-pick of the three source commits. No conflicts, no manual edits.

## Source

- Source branch: `origin/work/placement-accepted-ack-protocol-1546` @ `44756177`
- Source commits (3) cherry-picked in order:
  - `55fe3396` feat(protocol): add S2CPlacementAccepted message
  - `f7ca750b` feat(server): emit S2CPlacementAccepted on accepted submission
  - `44756177` feat(client): record PlacementSubmitAck lifecycle from S2CPlacementAccepted

## Refreshed Branch

- Branch: `integrate/placement-accepted-ack-protocol-1565`
- HEAD: `1c854cd26c3f0e07cf16038bbea544d8fc43b448`
- Worktree: `D:/tmp/wt-1565`
- Base: `origin/main@f19ab3ea2173e6d88658ea22f86563863e189741`

## Files Changed (vs origin/main)

```
M  client/Cargo.toml
M  client/src/ui/hand/mod.rs
M  server/Cargo.toml
M  server/src/feature/board/mod.rs
M  server/src/feature/board/placement.rs
M  server/src/feature/board/plugin.rs
M  shared/src/protocol.rs
A  tests/integration/board-lane-system/placement_acceptance_dispatch_test.rs
A  tests/integration/hand-ui/hand_ui_placement_accepted_test.rs
```

All files are within the source-payload scope from PROMPT 1546 (protocol message,
server emission hookup including plugin/mod wiring, client lifecycle recording,
focused server + client integration tests, and Cargo manifest updates required
by the new test crates).

## Validation

- `git diff --check origin/main HEAD`: clean (no whitespace errors).
- `git merge-base --is-ancestor origin/main HEAD`: TRUE → FF-ready.
- Path allowlist review: PASS — no touches to `production/**`, `Cargo.lock`,
  CI, or unrelated source areas.
- Broad Cargo verification: deferred to VERIFY lane per policy.
- Source worker (PROMPT 1546) report: 4 server cases + 6 client cases passing.

## Status

`READY_FOR_MAINLAND_ENQUEUE`

1565: PLACEMENT-ACCEPTED-ACK-PROTOCOL-INTEGRATION-REFRESH: READY_FOR_MAINLAND_ENQUEUE
