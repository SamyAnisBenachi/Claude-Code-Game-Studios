# PROMPT 1524 — Resolution Replay Mutation Integration Refresh

**Status**: SHIPPED — PROMPT 1521 worker output integrated onto current
`origin/main` with no scope drift.

**Worktree**: `D:/tmp/wt-1524`
**Branch**: `integrate/resolution-replay-mutation-1524`
**Branched from**: `origin/main@f69bd595`
**Source commit**: `origin/work/resolution-replay-mutation-1521@7f6980a2`
**Integration commit**: `3d63282e` (clean cherry-pick, no conflicts)

---

## What integrated

A single clean cherry-pick of `7f6980a2` from
`origin/work/resolution-replay-mutation-1521` onto current `origin/main`
(`f69bd595`). The source branch was based on `origin/main@5d46b9a9` and
was not FF after PROMPT 1518 main-land; cherry-pick was the correct
strategy and applied without conflict.

PARTIAL scope from 1521 preserved exactly:

- `client/Cargo.toml` — new `[[test]]` entry for the per-group cadence
  integration test.
- `client/src/presentation/board_rendering.rs` —
  `ResolutionReplayProgress` resource, `apply_resolution_replay_group_system`
  applier, `apply_replay_event_visual_feedback` helper, consume-system
  signature trim + `replay_progress.reset()` wiring, plugin registration.
- `tests/integration/board_rendering/resolution_replay_per_group_cadence_test.rs`
  — new per-group cadence integration test (2 cases).
- `reports/PROMPT-1521-resolution-replay-mutation-client-visual-queue.md`
  — original 1521 worker report.

## Validation

- `git diff --check origin/main...HEAD` — clean (no whitespace errors).
- Path allowlist review — `git diff --name-only origin/main...HEAD`
  returns exactly the four files above. No edits to shared protocol,
  server, shop/auction, hand UI, bot, sprint/session/QA paperwork.
- `git merge-base --is-ancestor origin/main HEAD` — PASS;
  `origin/main@f69bd595` is an ancestor of integration HEAD.
- Focused Cargo tests deferred per the broad-Cargo guidance in the
  PROMPT body. The 1521 source worker already reported the three
  recommended `board_rendering_*` focused suites green at `7f6980a2`;
  cherry-pick is a content-identical replay onto a newer main with no
  conflicts, so the test deltas are unchanged.

## Protocol gap report

None. 1521 was a client-only PARTIAL slice and this is a content-identical
re-base; no protocol surface touched.

## Follow-up list (deferred, client-only)

Tracked here so a future PROMPT can pick them up against the same
cadence harness without re-deriving scope. All client-only; no protocol
or server changes required.

- AC2 — unit movement visualisation at group cadence.
- AC3 — lane change visualisation at group cadence.
- AC4 — placed-unit display dedupe (no double-render across the
  intake → replay handoff).
- AC7 — objective HP delta + objective destruction feedback.
- AC8 — gold-awarded HUD fanout from `ResolutionReplay` events.
- AC9 / AC10 — spawn-range and phase-handoff ordering coverage.
- AC12 — author notes / debug overlay for the replay clock.

Each item extends `apply_replay_event_visual_feedback` and adds a
focused integration test under
`tests/integration/board_rendering/`.

## Authority / no-claim

Integration-only PROMPT. No new game logic, no protocol mutations, no
sprint closeout, no release-readiness claim. Worker branch ready for
`MAINLAND_ENQUEUE`; main push intentionally not performed by this
worker.

1524: RESOLUTION-REPLAY-MUTATION-INTEGRATION-REFRESH: SHIPPED
