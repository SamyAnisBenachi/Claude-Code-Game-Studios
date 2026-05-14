# Sprint 13 — `S11-HU-PHASE-IDEMPOTENCY-001` Evidence

> Story: `production/epics/playable-client/story-022-client-phase-changed-idempotency.md`
> Sprint 13 Should Have row (PROMPT 826 activation).
> Worker prompt: PROMPT 836 (`/dev-story` implementation).
> Worktree: `D:\_DEV\claude-code-game-studios-worktrees\s13-client-phase-idempotency`.
> Branch: `work/s13-client-phase-idempotency`.
> Source-of-truth at worker start: `origin/main@4f7ba78a303244d696028052a8f4d937416df63c`
> (`qa(s13): /story-done S11-SERVER-POOL-INIT-LOG-GUARD-001 (PROMPT 833)`).

---

## No-Claim Restatement (verbatim from story-022 Status / No-Claim Banner)

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 819. Sprint 12 is closed-with-conditions per PROMPT
817 and is not changed by this authoring run. (PROMPT 826 subsequently
activated Sprint 13; the worker prompt 836 lands the story implementation
under that activation.)

PROMPT 836 (this implementation run) does NOT claim and does NOT modify:
- public release readiness, release-candidate readiness, full game
  completion;
- broad / Standard-tier accessibility completion (`QA-COND-0005`);
- playtest / fun-hypothesis validation (`QA-COND-0006`);
- full playable-client manual QA;
- two-client GAME_OVER closure (`S8-QA-001-W1`);
- final-art / asset-production completion;
- Polish->Release gate-check retry (PROMPT 761 `FAIL` evidence preserved);
- `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, or PROMPT 761 gate-check artifact;
- `production/session-state/` files;
- `shared/` or `server/` source.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**No client-side optimistic phase authority is introduced or proposed by
this story.** The `S2CPhaseChanged` drain (`client::presentation::phase_sink_system`)
remains the single source of phase truth; this fix narrows the hand-UI
consumer's `phase_changed=true` signal so that it fires only on actual
`RoundPhase` transitions. ADR-002 + ADR-009 + ADR-021 binding.

---

## AC1 — File:line evidence of the spurious `phase_changed=true` source

**Before the fix** (at `origin/main@4f7ba78`):

`client/src/ui/hand/mod.rs:1082` (function `hand_ui_phase_transition_system`):

```rust
let phase_changed = current.is_changed();
if !phase_changed && !hand_contents.is_changed() {
    return;
}
```

Where `current: Res<CurrentClientPhase>`. The upstream drainer
`client::presentation::phase_sink_system` (`client/src/presentation/mod.rs:149-192`)
takes `ResMut<CurrentClientPhase>` and passes it through to
`apply_phase_changed_messages_with_resolution_gate(messages, &mut current, ...)`.
The coercion from `&mut ResMut<CurrentClientPhase>` to the expected
`&mut CurrentClientPhase` invokes
`<ResMut as DerefMut>::deref_mut`, which trips Bevy's change-detection
flag every Update tick — even when `messages` is empty (no
`S2CPhaseChanged` was drained that frame). The consumer system therefore
observed `current.is_changed() == true` at 60Hz and emitted
`phase_changed=true` on every frame in its `hand_ui_phase_transition`
tracing log, plus re-ran the `if phase_changed { ... }` reset block
(pending placements clear, timer reset, drag clears,
`PlacementDisclosureState` re-insertion, submitted-checkmark hide) every
frame instead of only on actual phase transitions.

---

## AC2 — Idempotency fix

`client/src/ui/hand/mod.rs:hand_ui_phase_transition_system` now compares
the just-observed `RoundPhase` value to a `Local<Option<RoundPhase>>`
that holds the previous frame's observation:

```rust
let observed_phase = current.phase;
let phase_changed = match *last_observed_phase {
    Some(prev) => prev != observed_phase,
    None => true,
};
if !phase_changed && !hand_contents.is_changed() {
    return;
}
// ... system body uses `phase_changed` exactly as before ...
*last_observed_phase = Some(observed_phase);
```

The 17-param-after-add violation of Bevy 0.18's 16-param system-fn limit
is resolved by bundling the three entity-modifying queries
(`submit_buttons`, `animators`, `timer_states`) into a single
`#[derive(SystemParam)] HandUiPhaseTransitionQueries` slot, leaving the
system at 15 top-level `SystemParam` slots. Field access into the
bundled queries is preserved verbatim via destructure-on-entry.

---

## AC3 — Integration test asserts the narrowed signal

New file: `tests/integration/playable_client/phase_changed_idempotency_test.rs`
(registered in `client/Cargo.toml` as
`playable_client_phase_changed_idempotency_test`).

Five tests, all asserting observable side-effect of `phase_changed=true`
via the `placement_timer.submitted` sentinel (cleared inside the
system's `if phase_changed { ... }` block):

```text
running 5 tests
test ac4_phase_changed_fires_on_first_observation ... ok
test ac2_phase_changed_fires_on_actual_transition ... ok
test ac2_repeated_same_phase_assignments_do_not_register_as_transitions ... ok
test ac3_at_most_one_phase_changed_across_multi_frame_run_with_one_transition ... ok
test ac3_phase_changed_does_not_fire_on_frames_without_transition ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Key cases:
- `ac3_phase_changed_does_not_fire_on_frames_without_transition` holds
  `RoundPhase::Placement` across 10 Update ticks; sentinel survives all
  10 (no spurious fire).
- `ac3_at_most_one_phase_changed_across_multi_frame_run_with_one_transition`
  drives 10 ticks containing exactly one `Placement -> DraftShop`
  transition at frame 5; asserts the sentinel was cleared **exactly
  once** (1 fire, not 10).
- `ac2_repeated_same_phase_assignments_do_not_register_as_transitions`
  re-assigns the same `RoundPhase::Placement` value across 5 frames
  (each `ResMut` write trips Bevy's change flag, but the phase value is
  unchanged); the consumer correctly compares values, not mutable
  accesses, and does NOT register a transition.

---

## AC4 — Existing phase-driven UI unaffected

Adjacent phase-consumer regression set re-run at the implementation tip
(narrowest justified set per Sprint 13 QA plan no-full-workspace-tests
policy):

```text
hand_ui_phase_state_machine_test    : 4 passed; 0 failed; 0 ignored
hand_ui_placement_unstaging_test    : 4 passed; 0 failed; 0 ignored
hand_ui_placement_timer_test        : 5 passed; 0 failed; 0 ignored
hud_phase_label_round_counter_test  : 6 passed; 0 failed; 0 ignored
hud_phase_transitions_test          : 5 passed; 0 failed; 0 ignored
playable_client_active_loop_ui_state_test : 4 passed; 0 failed; 0 ignored
```

All previously-passing phase-driven tests continue to pass. No
`#[ignore]` markers introduced.

---

## AC5 — No client-side optimistic phase authority introduced

The fix is read-only over `Res<CurrentClientPhase>`. The `Local<Option<RoundPhase>>`
is the consumer's private memory of the previous frame's observation; it
does NOT participate in phase truth. The drainer
`phase_sink_system` remains the single mutator of `CurrentClientPhase`,
and is unmodified by this story. ADR-002 + ADR-009 + ADR-021 binding.

---

## AC6 — No protocol or server-side change

```text
git diff --stat origin/main -- 'shared/' 'server/' 'production/sprint-status.yaml' \
  'production/sprints/sprint-13.md' 'production/stage.txt' 'production/gate-checks/'
# (empty)
```

Zero functional change under `shared/` or `server/`.

---

## AC7 — Shared phase sink unchanged

`client/src/presentation/mod.rs` (which hosts
`phase_sink_system` + `apply_phase_changed_messages_with_resolution_gate`)
is NOT in the diff:

```text
git diff --stat origin/main...HEAD
 client/Cargo.toml                                                      |  4 +
 client/src/ui/hand/mod.rs                                              | 53 +++++++++++++++++++--
 tests/integration/playable_client/phase_changed_idempotency_test.rs    | 188 ++++++++++++++++++++++++ (new)
```

---

## AC8 — Sprint 13 disposition / Polish->Release artifacts preserved

PROMPT 836 does NOT modify:
- `production/sprint-status.yaml`
- `production/sprints/sprint-13.md`
- `production/stage.txt`
- `production/gate-checks/gate-polish-release-2026-05-12.md`

`/story-done` paperwork (status flip, AC checkboxes, sprint-status.yaml
row flip) is the next prompt's responsibility, NOT this implementation
prompt.

---

## AC9 — Workspace test pass

Per Sprint 13 QA plan no-full-workspace-tests-by-default policy, the
worker ran the narrowest BLOCKING + adjacent regression set
(commands above). No `#[ignore]` markers were introduced. The new test
binary `playable_client_phase_changed_idempotency_test` passes with 5/5
cases. Full `cargo test --workspace` is intentionally deferred to a
later integration / `/team-qa` prompt per the QA plan.

---

## Regression Commands Executed

```text
cargo fmt -p client -- --check                                # EXIT=0
cargo check -p client                                         # EXIT=0
cargo test -p client --test playable_client_phase_changed_idempotency_test
                                                              # EXIT=0, 5/5 pass
cargo test -p client --test hand_ui_phase_state_machine_test \
   --test hand_ui_placement_unstaging_test \
   --test hand_ui_placement_timer_test \
   --test hud_phase_transitions_test \
   --test hud_phase_label_round_counter_test \
   --test playable_client_active_loop_ui_state_test           # EXIT=0, all pass
git diff --check origin/main...HEAD                           # EXIT=0
```

Cargo resource policy was applied for every Cargo command:
```text
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

No disk cleanup was required (target directory size remained well below
disk-pressure threshold).

---

## Cross-links

- PROMPT 803 §3 DC-5 ("Client-side phase idempotency drift"): this story
  folds with that audit row and lands the canonical fix for the
  hand-UI consumer. Same defect class affects `client/src/ui/hud/mod.rs`
  (lines 390, 1247) and `client/src/ui/shop_auction/mod.rs` (line 1248);
  those consumers early-return on `current.is_changed()` without
  emitting a spurious `phase_changed=true` tracing log, so they are
  performance-impact-only and out of this story's narrow scope. A
  follow-on story may extend the same `Local<Option<RoundPhase>>`
  pattern to those consumers under DC-5; that is **not** undertaken
  here.
- Sprint 12 close-out deferral row
  `sprint_12_closeout.deferred_into_sprint_13_planning.S11-HU-PHASE-IDEMPOTENCY-001`
  (PROMPT 817).
- ADR-002 (Client-Server Authority), ADR-009 (Round State Machine),
  ADR-021 (Presentation Layer Architecture): binding; no deviation.
