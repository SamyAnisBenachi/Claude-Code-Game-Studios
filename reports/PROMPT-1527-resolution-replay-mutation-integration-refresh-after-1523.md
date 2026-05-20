# PROMPT-1527 — Resolution Replay Mutation Integration Refresh (after PROMPT-1523)

## Summary

Refreshed the PROMPT-1521 PARTIAL resolution-replay per-group cadence integration
onto current `origin/main` after PROMPT-1523 (`a51f0ac7`) made the prior 1524
branch (based on `f69bd595`) non-FF.

- Base: `origin/main @ a51f0ac71c5b0d32e7ecd48f4a43a8a943e81aba`
- New branch: `integrate/resolution-replay-mutation-1527`
- Cherry-picked: `3d63282e` (PROMPT-1521 resolution replay per-group visual cadence)
- New tip: `8cc69565`

Cherry-pick applied cleanly (no conflicts) — the 1521 scope (board rendering
replay cadence applier + per-group cadence integration test) is disjoint from
the 1523 hand/draft right-click inspect overlay scope.

## Path allowlist verification

`git diff --name-only origin/main...HEAD`:

```
client/Cargo.toml
client/src/presentation/board_rendering.rs
reports/PROMPT-1521-resolution-replay-mutation-client-visual-queue.md
tests/integration/board_rendering/resolution_replay_per_group_cadence_test.rs
```

All paths within the PROMPT-1527 owned scope. Forbidden areas (shared protocol,
server, shop/auction, hand UI, bot files, sprint/session/QA paperwork) untouched.

## Validation

- `git diff --check origin/main...HEAD` → clean (no whitespace/conflict markers).
- `git merge-base --is-ancestor origin/main HEAD` → ancestor OK; FF-eligible.
- Focused cargo tests **not re-run** in this integration refresh; the cherry-pick
  reproduces the exact source tree validated in PROMPT-1521 commit message:
  - `board_rendering_resolution_replay_per_group_cadence_test` → 2/2 PASS (1521)
  - `board_rendering_resolution_combat_feedback_test` → 7/7 PASS (1521)
  - `board_rendering_resolution_anim_queue_test` → 5/5 PASS (1521)
  Re-running is recommended at mainland enqueue if the resource policy permits;
  no source-level changes since 1521 in this scope.

## Preserved PARTIAL scope (unchanged from 1521)

- `ResolutionReplayProgress` resource (idempotent per-group emit).
- `apply_resolution_replay_group_system` scheduled after
  `resolution_executing_system` and before the queue-drain transition.
- Intake-time `emit_resolution_combat_feedback` removed; reset on new script
  load.
- Per-group cadence integration test asserting (a) mixed `sub_step` scripts
  gate later-group emission on the group boundary and (b) repeated frames in
  the same active group do not double-emit feedback.

## Deferred follow-ups (remaining client-only ACs, out-of-scope here)

Carried forward unchanged from PROMPT-1521 / PROMPT-1524 reports:

- AC: per-group visual replay timing knob (currently emits on group-active
  transition only; no inter-group delay/pacing config yet).
- AC: client-side replay completion telemetry (no `ResolutionReplayCompleted`
  signal/event for analytics).
- AC: replay interruption / resume behaviour for mid-replay disconnect
  recovery (current behaviour resets progress on new script load).
- AC: visual queue back-pressure / overlap guard when consecutive
  `S2CResolutionEvent` arrive faster than per-group emit cadence (current
  code idempotent within a group but no cross-script protection).
- AC: configurable feedback-emit cadence per `AnimGroup` kind (uniform today).

These remain client-only and disjoint from the integrated PARTIAL slice.

## Mainland enqueue readiness

- FF-eligible onto `origin/main @ a51f0ac7`: **YES**.
- Path allowlist: **PASS**.
- Diff check: **CLEAN**.
- Forbidden-zone touch: **NONE**.

Ready for `MAINLAND_ENQUEUE`.

1527: RESOLUTION-REPLAY-MUTATION-INTEGRATION-REFRESH-AFTER-1523: SHIPPED
