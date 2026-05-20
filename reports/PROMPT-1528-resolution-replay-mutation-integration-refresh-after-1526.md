# PROMPT-1528 — Resolution Replay Mutation Integration Refresh (after PROMPT-1526)

## Summary

Refreshed the PROMPT-1521 PARTIAL integration (resolution replay per-group
visual cadence) onto current `origin/main@b82a341d3b9d2ef5d777f71d14ab0422bcfe2d00`
(post PROMPT-1526 land). The prior integration branch
`origin/integrate/resolution-replay-mutation-1527` was based on
`origin/main@a51f0ac7` and is no longer fast-forward eligible after PROMPT-1525
and PROMPT-1526 main-landed.

PROMPT-1521/1527 scope (client board rendering, Cargo test registration,
integration test, report files) is disjoint from the PROMPT-1525/1526
bot/server scope, so the cherry-pick is conflict-free.

## Branch

- Base: `origin/main@b82a341d3b9d2ef5d777f71d14ab0422bcfe2d00`
- Branch: `integrate/resolution-replay-mutation-1528`
- Tip commits (oldest first on top of main):
  - `7f98fc58` PROMPT-1521 resolution replay per-group visual cadence (cherry-pick of `8cc69565`)
  - `1e5cd0e7` PROMPT-1527 integration refresh report (cherry-pick of `00d36b26`)
  - this report commit (PROMPT-1528)

## Operations

```
git worktree add -b integrate/resolution-replay-mutation-1528 D:/tmp/wt-1528 b82a341d
git cherry-pick 8cc69565 00d36b26   # clean, no conflicts
```

## Validation

- `git diff --check origin/main...HEAD` -> clean (DIFF-CHECK-OK).
- `git merge-base --is-ancestor origin/main HEAD` -> true (ANCESTOR-OK).
- Path allowlist review: all 5 changed paths are owned scope.
  - `client/Cargo.toml`
  - `client/src/presentation/board_rendering.rs`
  - `reports/PROMPT-1521-resolution-replay-mutation-client-visual-queue.md`
  - `reports/PROMPT-1527-resolution-replay-mutation-integration-refresh-after-1523.md`
  - `tests/integration/board_rendering/resolution_replay_per_group_cadence_test.rs`
- Focused cargo tests: NOT re-run on the refresh (per PROMPT-1528 "defer
  verification" guidance; PROMPT-1521 already recorded 2/2, 7/7, 5/5 PASS
  on identical code, no rebase conflicts altered the diff).

## Path allowlist review

All changed paths are within the PROMPT-1528 owned scope. No protocol,
server, shop/auction, hand UI, bot, sprint/session/QA/stage paths touched.

## Deferred follow-ups (client-only AC remainders carried over from PROMPT-1521 PARTIAL)

The PROMPT-1521 PARTIAL closed the per-group visual cadence applier + tests
only. The following AC items remain deferred and are not in this integration:

- AC2 final-frame VFX gating (kill markers should latch on last frame of the
  killing group, not on first matching frame).
- AC3 cross-group damage-number coalescing (multi-source same-target damage
  numbers stacking across groups, per design intent).
- AC4 replay-restart safety on mid-script S2CResolutionEvent re-arrival
  (idempotency proof under network re-deliver scenarios).
- AC5 group-boundary audio cue alignment (currently still intake-driven for
  audio; visual was the 1521 scope only).

These should be carried as their own PROMPT-* tickets against current main
when the orchestrator queue picks them up.

## Status

Ready for `MAINLAND_ENQUEUE` — branch is FF on current `origin/main`, diff
clean, allowlist clean, ancestry confirmed.

1528: RESOLUTION-REPLAY-MUTATION-INTEGRATION-REFRESH-AFTER-1526: SHIPPED
