# PROMPT 1521 — Resolution Replay Mutation Client Visual Queue

**Status**: PARTIAL — smallest-safe client-side slice landed; remainder is a
client-only scope follow-up (no protocol gap).

**Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/resolution-replay-mutation-1521`
**Branch**: `work/resolution-replay-mutation-1521` (from `origin/main@5d46b9a9`)

---

## What landed

PROMPT 1521 ships the **cadence pivot** the story is really about: visible
resolution feedback (damage numbers, kill markers) now fires *per AnimGroup*
as the group becomes the active playback group, instead of burst-firing the
moment `S2CResolutionEvent` is consumed.

Concretely:

1. **New resource `ResolutionReplayProgress`**
   (`client/src/presentation/board_rendering.rs`). Tracks
   `last_emitted_group_index: Option<usize>`. Resets when the queue drains
   or when `consume_pending_resolution_script_system` loads a new script.

2. **New system `apply_resolution_replay_group_system`**, scheduled in
   `PresentationSet::StateSync` `.after(resolution_executing_system)` and
   `.before(sync_resolution_queue_drain_state_system)`. Walks the active
   `AnimGroup`'s `ResolutionReplay { event }` items and applies the existing
   visible feedback:
   - `CombatDamage` → `DamageNumberSpawnRequested` (existing damage-number
     lane, including the zero-damage skip semantics).
   - `UnitDied` / `UnitRemoved` → `ResolutionKillMarker` text2d at the
     unit's last known transform (existing TTL + board-rebuild cleanup).
   The applier is idempotent per group: progress index advances after
   emission, so subsequent frames in the same group are no-ops.

3. **Intake path simplified** in `consume_pending_resolution_script_system`:
   the prior intake-time call to `emit_resolution_combat_feedback` has been
   removed and replaced by `replay_progress.reset()`. The unused
   `damage_writer` / `commands` parameters were dropped from the consume
   signature. `emit_resolution_combat_feedback` remains as a public helper
   for callers that still want the synchronous semantics, but no live
   caller exists.

4. **Focused integration test**
   `tests/integration/board_rendering/resolution_replay_per_group_cadence_test.rs`
   (new `[[test]]` entry in `client/Cargo.toml`):
   - `test_replay_emits_damage_for_first_group_only_until_time_advances` —
     mixed sub-step-1/sub-step-3 script; intake frame fires only group 1,
     mid-playback yields no group-3 emission, group boundary crossing
     yields the group-3 emission.
   - `test_replay_does_not_double_emit_when_repeated_frames_share_a_group`
     — repeated frames within the same active group never re-emit feedback.

## Validation

- `cargo check -p client --test board_rendering_resolution_replay_per_group_cadence_test` — clean.
- `cargo test -p client --test board_rendering_resolution_replay_per_group_cadence_test` — 2/2 PASS.
- `cargo test -p client --test board_rendering_resolution_combat_feedback_test` — 7/7 PASS (no regressions).
- `cargo test -p client --test board_rendering_resolution_anim_queue_test` — 5/5 PASS (no regressions).
- `git diff --check` — clean.
- Path allowlist: only `client/src/presentation/board_rendering.rs`,
  `client/Cargo.toml`, and `tests/integration/board_rendering/<new>` were
  touched. No edits to `shared/src/protocol.rs`, shop/auction, hand UI, bot,
  sprint/session/QA paperwork.

Pre-existing root-checkout warnings/errors (`ScoreboardDotState.known`
field on `hud_phase_transitions_test`, the universal-marker deprecation
warnings) are unrelated to this PROMPT and were not modified.

## Protocol gap report

**There is no protocol gap.** The existing `S2CResolutionEvent` /
`TaggedEvent { sub_step, trigger_index, event }` already encodes the
ordered replay clock; the existing `AnimQueueEvent::ResolutionReplay`
carries each event into the queued group. The only thing missing was a
client-side applier that drains those entries at group-start cadence.

The remainder of Story 015's AC surface (AC2 unit movement, AC3 lane
change, AC4 placed-unit display dedupe, AC7 objective HP/destruction,
AC8 gold awarded HUD fanout, AC9/AC10 spawn-range + phase-handoff
ordering coverage, AC12 author notes) is **purely client-side** and can
ship as follow-up PROMPTs against this same cadence harness — they
extend `apply_replay_event_visual_feedback` and add per-AC tests, but
need no protocol or server changes.

## Authority / no-claim

The client still receives `S2CResolutionEvent` from the
server-authoritative resolver and only mutates **presentation** state.
No client-side combat recomputation. No new C2S game-logic messages.
No release-readiness, final-art, broad combat redesign, or sprint
closeout claim.

## Files changed

- `client/Cargo.toml` — register new `[[test]]` target.
- `client/src/presentation/board_rendering.rs` —
  `ResolutionReplayProgress` resource, `apply_resolution_replay_group_system`,
  `apply_replay_event_visual_feedback` helper, consume-system signature
  trim + `replay_progress.reset()` wiring, plugin registration.
- `tests/integration/board_rendering/resolution_replay_per_group_cadence_test.rs` — new.
- `reports/PROMPT-1521-resolution-replay-mutation-client-visual-queue.md` — this file.

1521: RESOLUTION-REPLAY-MUTATION-CLIENT-VISUAL-QUEUE: PARTIAL
