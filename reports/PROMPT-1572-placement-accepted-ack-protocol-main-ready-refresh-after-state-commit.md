# PROMPT 1572 — Placement Accepted ACK Protocol — Main-Ready Refresh After State Commit

## Summary

Refreshed the PROMPT 1568 placement-accepted ACK protocol payload onto the
current `origin/main@e8e4651b` (post state-snapshot push). New branch is
strict-FF-ready over `origin/main`, preserving payload commits from PROMPT
1546 and carried reports from PROMPT 1565/1568. Added this PROMPT 1572
refresh report.

## Source-of-truth

- `origin/main` = `e8e4651ba9b97bbab1a004815513c5cff1948a0c`
  (`state: record post-1564 orchestration pause`).
- Previous payload branch:
  `origin/integrate/placement-accepted-ack-protocol-1568 @ 6546c9bd`
  (FF over `5be95a9b` but no longer FF over current main).
- Diff between `5be95a9b` and `origin/main@e8e4651b` is one orchestrator
  state file (`production/session-state/codex-orchestrator-state.md`,
  63 insertions) — no payload overlap.

## Refreshed branch

- Branch: `integrate/placement-accepted-ack-protocol-1572`
- Worktree: `D:/Tmp/wt-1572`
- HEAD: `e57bce010e63ef9d3a60cd092aa4220402555fff`
- Base: `origin/main@e8e4651b` (FF parent).

## Replayed commits (cherry-pick order)

1. `51ec857f` feat(protocol): add S2CPlacementAccepted message (PROMPT 1546)
2. `5910a0aa` feat(server): emit S2CPlacementAccepted on accepted submission (PROMPT 1546)
3. `5f6238a2` feat(client): record PlacementSubmitAck lifecycle from S2CPlacementAccepted (PROMPT 1546)
4. `1bcecee8` PROMPT-1565 integration refresh report
5. `e57bce01` PROMPT-1568 main-ready refresh report
   (this PROMPT 1572 report committed on top after report file is added.)

## Owned-scope verification

`git diff --name-only origin/main..HEAD`:

- `shared/src/protocol.rs`
- `server/src/feature/board/mod.rs`
- `server/src/feature/board/placement.rs`
- `server/src/feature/board/plugin.rs`
- `server/Cargo.toml`
- `client/src/ui/hand/mod.rs`
- `client/Cargo.toml`
- `tests/integration/board-lane-system/placement_acceptance_dispatch_test.rs`
- `tests/integration/hand-ui/hand_ui_placement_accepted_test.rs`
- `reports/PROMPT-1565-placement-accepted-ack-protocol-integration-refresh.md`
- `reports/PROMPT-1568-placement-accepted-ack-protocol-main-ready-refresh-after-1564.md`
- `reports/PROMPT-1572-placement-accepted-ack-protocol-main-ready-refresh-after-state-commit.md`

All within the owned scope declared in PROMPT 1572. No edits to
`production/sprint-status.yaml`, `production/session-state/**`,
`production/sprints/**`, `production/qa/**`, `production/stage.txt`, or
unrelated source/CI modules.

## Checks performed

- `git diff --check origin/main..HEAD` — clean (no whitespace errors).
- `git merge-base --is-ancestor origin/main HEAD` — `FF_READY`.
- Path allowlist review — pass.
- Broad Cargo suites intentionally skipped per PROMPT policy; defer to
  VERIFY lane.

## Status

`READY_FOR_MAINLAND_ENQUEUE`

1572: PLACEMENT-ACCEPTED-ACK-PROTOCOL-MAIN-READY-REFRESH-AFTER-STATE-COMMIT: READY_FOR_MAINLAND_ENQUEUE
