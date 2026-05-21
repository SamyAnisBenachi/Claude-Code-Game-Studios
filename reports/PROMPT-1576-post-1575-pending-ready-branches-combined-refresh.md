# PROMPT 1576 -- Post-1575 Pending Ready Branches Combined Refresh

## Summary

Created a single combined main-ready refresh branch that preserves all three
pending payloads from PROMPT 1572, 1573, and 1574 on top of the current
source-of-truth (`origin/main@2bdff922`). This avoids serial stale-refresh
churn where landing one branch makes the other two non-FF.

## Source-of-truth

- `origin/main` = `2bdff922fab4cee2f8666d410c03c4bd0cecaf2a`
  (`PROMPT-1575 update-latestmain play-main worktree fallback`).

## Refreshed branch

- Branch: `integrate/post-1575-pending-ready-branches-1576`
- Head: `80240df9d55119bba303c6b9f12fec96b7a87c6a`
- Worktree: `D:/tmp/wt-1576`

## Source branches refreshed

| Source branch | Pre-refresh head | Base before refresh |
|---|---|---|
| `origin/integrate/placement-accepted-ack-protocol-1572` | `c75ffdf8` | `origin/main@e8e4651b` (stale) |
| `origin/integrate/qa-snapshot-observability-fields-1573` | `71eaaf0f` | `origin/main@e8e4651b` (stale) |
| `origin/integrate/krosmaga-dev-proxy-stage2-1574` | `f861a274` | `origin/main@e8e4651b` (stale) |

Each was NOT_FF_READY against current `origin/main@2bdff922`. This combined
refresh restages the semantic payloads onto current main as a single FF-ready
branch.

## Commits in this refresh (oldest first, on top of 2bdff922)

### Placement accepted ACK protocol (PROMPT 1572 payload)

- `d90cbeea` feat(protocol): add S2CPlacementAccepted message (PROMPT 1546)
- `072f6ae9` feat(server): emit S2CPlacementAccepted on accepted submission (PROMPT 1546)
- `dcb434d1` feat(client): record PlacementSubmitAck lifecycle from S2CPlacementAccepted (PROMPT 1546)
- `820b5368` PROMPT-1565 integration refresh report
- `950e302f` PROMPT-1568 main-ready refresh report after 1564
- `1a58d7a6` PROMPT-1572 main-ready refresh report after state commit

### QA snapshot observability fields (PROMPT 1573 payload)

- `03342873` PROMPT-1533 qa_snapshot: ACK lifecycle + hover provenance + label roles
- `e9c1af95` PROMPT-1569 main-ready refresh report after 1564
- `69089431` PROMPT-1573 main-ready refresh report after state commit

### Krosmaga dev-proxy Stage 2 (PROMPT 1574 payload)

- `95c67366` PROMPT-1570 main-ready refresh report after 1564 (includes Stage 2 tooling payload)
- `80240df9` PROMPT-1574 main-ready refresh report after state commit

## Payloads included

- Placement accepted ACK protocol: `shared/src/protocol.rs`,
  `server/src/feature/board/{mod.rs,placement.rs,plugin.rs}`, `server/Cargo.toml`,
  `client/src/ui/hand/mod.rs`, `client/Cargo.toml`,
  `tests/integration/board-lane-system/placement_acceptance_dispatch_test.rs`,
  `tests/integration/hand-ui/hand_ui_placement_accepted_test.rs`, plus the
  three carried reports (PROMPT-1565/1568/1572).
- QA snapshot observability fields: `client/src/presentation/qa_snapshot.rs`,
  `tests/integration/qa_snapshot/placement_auction_state_field_coverage_test.rs`,
  plus carried reports (PROMPT-1533/1569/1573).
- Krosmaga dev-proxy Stage 2 tooling: `tools/asset-provenance/README.md`,
  `tools/asset-provenance/validate_dev_proxy_pack.py`,
  `tools/asset-provenance/test_validate_dev_proxy_pack.py`,
  `tools/asset-provenance/fixtures/dev-proxy-pack-{clean,bad-logical-id,stage2-candidate}.json`,
  plus carried reports (PROMPT-1534/1539/1545/1559/1563/1567/1570/1574).

## Payloads skipped

None. None of the three payloads is already present on current main; all three
needed restage.

## Conflict notes

No conflicts. The three payloads are orthogonal at the file level:
- 1572 touches protocol/server-board/client-hand-ui + dedicated integration tests
- 1573 touches `client/src/presentation/qa_snapshot.rs` + dedicated test
- 1574 touches `tools/asset-provenance/**` only

All cherry-picks applied cleanly onto `2bdff922`. The stale `Update-LatestMain.ps1`
and `PROMPT-1575-*` reports that appeared in the raw branch diffs (artifacts of
the older `e8e4651b` base) are absent here because we cherry-picked only the
payload commits, not the stale base divergence.

## Path allowlist review

All paths fall within the documented owned scope (union of the three source
payload scopes plus this PROMPT 1576 report). No forbidden paths touched
(`production/sprint-status.yaml`, `production/session-state/**`,
`production/sprints/**`, `production/qa/**`, `production/stage.txt`,
unrelated Cargo/CI/source modules).

## Validation

- `git diff --check 2bdff922..HEAD`: clean (no whitespace errors).
- `git merge-base --is-ancestor origin/main HEAD`: TRUE (FF-ready against
  `origin/main@2bdff922`).
- `python -m unittest test_validate_dev_proxy_pack` in
  `tools/asset-provenance/`: 25 tests PASS in 0.27s.
- Broad Cargo verification deferred to a later VERIFY lane per policy.

## Verdict

READY_FOR_MAINLAND_ENQUEUE -- FF-ready against `origin/main@2bdff922`.

1576: POST-1575-PENDING-READY-BRANCHES-COMBINED-REFRESH: READY_FOR_MAINLAND_ENQUEUE
