# PROMPT 1568 -- Placement Accepted ACK Protocol Main-Ready Refresh After 1564

## Summary

Refreshed the PROMPT 1565 placement accepted ACK protocol payload onto current
`origin/main@5be95a9b` (PROMPT 1564 bot participant action loop main-land) so
the result is strict-FF eligible for MAINLAND_ENQUEUE. Payload preserved
verbatim from the PROMPT 1546 implementation chain plus the carried PROMPT 1565
integration-refresh report; no code modifications.

## Branch / commit

- Refreshed branch: `integrate/placement-accepted-ack-protocol-1568`
- HEAD: `2ac97c61`
- Worktree: `D:/Tmp/wt-1568`
- Base: `origin/main @ 5be95a9b` (PROMPT 1564)
- Source branch: `origin/integrate/placement-accepted-ack-protocol-1565 @ 285c5b61`
  (NOT strict-FF over current `origin/main`; FF-ready only over `f19ab3ea`)

## Replay chain (origin/main..HEAD)

```
58b22db1 feat(protocol): add S2CPlacementAccepted message (PROMPT 1546)
214b480a feat(server): emit S2CPlacementAccepted on accepted submission (PROMPT 1546)
37c6670a feat(client): record PlacementSubmitAck lifecycle from S2CPlacementAccepted (PROMPT 1546)
2ac97c61 PROMPT-1565 integration refresh report for placement accepted ACK protocol (1546 cherry-picked onto origin/main f19ab3ea)
```

Cherry-picks applied cleanly with no merge conflicts.

## Path allowlist review (`git diff --name-only origin/main HEAD`)

```
client/Cargo.toml
client/src/ui/hand/mod.rs
reports/PROMPT-1565-placement-accepted-ack-protocol-integration-refresh.md
server/Cargo.toml
server/src/feature/board/mod.rs
server/src/feature/board/placement.rs
server/src/feature/board/plugin.rs
shared/src/protocol.rs
tests/integration/board-lane-system/placement_acceptance_dispatch_test.rs
tests/integration/hand-ui/hand_ui_placement_accepted_test.rs
```

All paths are within the owned scope (shared protocol ACK additions, server
placement dispatch/registration, client hand UI ACK consumer/snapshot wiring,
focused integration tests, and carried PROMPT 1565 report). No forbidden
paths touched (no `production/**`, no unrelated Cargo/CI/source modules).

Plus this report itself:
`reports/PROMPT-1568-placement-accepted-ack-protocol-main-ready-refresh-after-1564.md`.

## Validation checks

- `git diff --check origin/main HEAD` -> clean (no whitespace/conflict markers)
- `git merge-base --is-ancestor origin/main HEAD` -> exit 0 (strict-FF)
- Path allowlist -> PASS (owned scope only)
- Broad Cargo verification deferred to a VERIFY lane per worker policy.

## Status

`READY_FOR_MAINLAND_ENQUEUE`

1568: PLACEMENT-ACCEPTED-ACK-PROTOCOL-MAIN-READY-REFRESH-AFTER-1564: READY_FOR_MAINLAND_ENQUEUE
