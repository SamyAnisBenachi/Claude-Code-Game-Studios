# Story 022: S11-HU-PHASE-IDEMPOTENCY-001 -- Client `phase_changed=true` 60Hz Idempotency

> **Epic**: Playable Client
> **Story ID**: S11-HU-PHASE-IDEMPOTENCY-001
> **Status**: Done -- closed by PROMPT 844 `/story-done` on
> `origin/main@534d9df` (worker `8810698` PROMPT 836 on
> `work/s13-client-phase-idempotency` + integration `fbcb03a` + `537ea3f`
> rustfmt collapse + merge-origin/main `a9e636c` PROMPT 841 fast-forward
> push). AC1-AC10 all satisfied per worker + integration evidence (see
> Closure Trail below).
> **Layer**: Client gameplay -- shared phase sink / HU phase consumer
> **Type**: Integration -- targeted fix to phase-change consumer + integration
> test
> **Sprint**: Sprint 13 active (Sprint 12 close-out deferral; PROMPT 803
> §3 DC-5 same-class candidate); activated by PROMPT 826
> **Authored**: 2026-05-14 by PROMPT 819
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)
> **Closure source-of-truth**: `origin/main@534d9df` (PROMPT 841
> integration commits `fbcb03a` + `537ea3f` + `a9e636c` on top of
> PROMPT 833 base `4f7ba78`; subsequent PROMPT 843 closure commit
> `534d9df` does not modify story-022 scope)

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 819. Sprint 12 is closed-with-conditions per PROMPT
817 and is not changed by this authoring run.

PROMPT 819 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** The `S2CPhaseChanged` drain remains the single source of phase
truth; this story narrows the consumer's `phase_changed=true` signal so
that it fires only on actual phase transitions. ADR-002 + ADR-009 +
ADR-021 binding.

---

## Source Finding

- Sprint 11 close-out (`S11-HU-PHASE-IDEMPOTENCY-001` row) flagged
  spurious `phase_changed=true` on every frame from a hand-UI phase
  consumer.
- Sprint 12 close-out (PROMPT 817) deferred the row forward to Sprint
  13 planning.
- PROMPT 803 §3 DC-5 ("Client-side phase idempotency drift") names the
  same defect class; the Sprint 13 plan folds the two so this story
  also addresses DC-5.

---

## Problem Class / Prevention Target

**Defect class (DC-5)**: A client-side phase consumer emits
`phase_changed=true` on every frame (60Hz) rather than only on actual
phase transitions. The downstream subscribers correctly read
`Res<CurrentClientPhase>` for phase truth, but the spurious "phase
changed this frame" signal causes:

- Misleading log noise around phase transitions (every frame appears
  to be a transition).
- Risk that a future subscriber treats the signal as authoritative
  and performs one-shot work per frame instead of per transition.
- Confusing diagnostic captures (e.g., the Sprint 12 story 019
  tighter-capture run).

**Prevention target**: Narrow the `phase_changed` signal so that it
is `true` only on the frame where `Res<CurrentClientPhase>` actually
transitions to a new variant; otherwise `false`. The
`S2CPhaseChanged` drain remains the single source of phase truth
(ADR-021); this story does not change the drain itself, only the
consumer's idempotency.

---

## Context

### Existing surface

- **`client/src/state/` or `client/src/presentation/`** (canonical
  location verified by implementing worker): the phase consumer that
  emits the spurious `phase_changed=true` signal.
- **Shared phase sink (ADR-021)**: the single drainer of
  `S2CPhaseChanged`; **not modified** by this story.
- **`Res<CurrentClientPhase>`**: phase truth source; **read-only** in
  this story.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/round-state-machine.md` (phase semantics);
  `design/gdd/hand-ui.md` (HU phase consumer).
- **ADR-002** (Client-Server Authority): no client-side phase
  authority added.
- **ADR-009** (Round State Machine): phase transitions remain
  server-authoritative.
- **ADR-021** (Presentation Layer Architecture): shared phase sink
  remains the single drainer; this story narrows a downstream
  consumer's idempotency.
- **TR registry**: no new TR.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` (any `.rs` edit), and
  `liv-bevy-lightyear` if the implementing worker touches any code
  importing `lightyear`.

### Control Manifest Rules

- Required: `phase_changed=true` fires only on actual phase
  transitions; verified by an integration test.
- Required: `S2CPhaseChanged` drain remains the single source of
  phase truth (ADR-021).
- Required: `Res<CurrentClientPhase>` is the read-only phase source.
- Forbidden: Adding any optimistic client-side phase authority.
- Forbidden: Modifying the shared phase sink itself.
- Forbidden: Adding a second drainer of `S2CPhaseChanged`.

---

## Story Classification

**Story type**: Integration -- targeted fix to a client-side phase
consumer + integration test.

This is **NOT** a:

- Server-side change.
- Protocol change.
- Pure UX-spec story.

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Spurious signal source located**: GIVEN the
  implementation prompt's first read pass, WHEN the phase consumer
  in `client/src/` is located, THEN the exact `phase_changed=true`
  emission point is named with file:line evidence in the evidence
  document.
  **Closure evidence (PROMPT 844)**: evidence doc §AC1 names
  pre-fix emission at `client/src/ui/hand/mod.rs:1082` (function
  `hand_ui_phase_transition_system`) with the root-cause analysis
  that upstream `phase_sink_system` (`client/src/presentation/mod.rs:149-192`)
  trips Bevy 0.18 change-detection at 60Hz via `DerefMut` even when
  no `S2CPhaseChanged` was drained. PASS.

- [x] **AC2 -- Idempotency fix lands**: GIVEN the located source,
  WHEN the fix lands, THEN `phase_changed=true` is set only on the
  frame where `Res<CurrentClientPhase>` actually transitions (i.e.,
  the consumer compares the just-observed phase value to the prior
  frame's value and emits `true` only on inequality).
  **Closure evidence (PROMPT 844)**: worker commit `8810698`
  replaces `current.is_changed()` with a `Local<Option<RoundPhase>>`
  comparison against the previous frame's observed phase value;
  `phase_changed = match *last_observed_phase { Some(prev) => prev != observed_phase, None => true }`.
  Bevy 0.18 16-param system-fn limit honoured by bundling three
  entity-modifying queries into a `#[derive(SystemParam)]
  HandUiPhaseTransitionQueries` slot. PASS.

- [x] **AC3 -- Integration test asserts narrowed signal**: GIVEN a
  new or extended integration test (e.g.,
  `tests/integration/playable_client/phase_changed_idempotency_test.rs`),
  WHEN the test drives a multi-frame run with at most one phase
  transition, THEN it asserts `phase_changed=true` fires at most
  once across all frames.
  **Closure evidence (PROMPT 844)**: new test file
  `tests/integration/playable_client/phase_changed_idempotency_test.rs`
  (NEW; registered in `client/Cargo.toml` as
  `playable_client_phase_changed_idempotency_test`) contains 5 cases
  including `ac3_phase_changed_does_not_fire_on_frames_without_transition`
  (10 ticks, sentinel survives) and
  `ac3_at_most_one_phase_changed_across_multi_frame_run_with_one_transition`
  (10 ticks with one Placement->DraftShop transition; sentinel
  cleared exactly once). 5/5 pass at worker tip per evidence doc
  §AC3. PASS.

- [x] **AC4 -- Existing phase-driven UI unaffected**: GIVEN the
  Sprint 12 phase-driven UI tests (HUD phase label, hand-UI phase
  transitions, shop-auction-UI phase transitions, board-rendering
  phase transitions), WHEN the workspace test suite runs at the
  implementation commit, THEN all previously-passing phase tests
  still pass.
  **Closure evidence (PROMPT 844)**: per evidence doc §AC4, the
  adjacent phase-consumer regression set (`hand_ui_phase_state_machine`,
  `placement_unstaging`, `placement_timer`,
  `hud_phase_label_round_counter`, `hud_phase_transitions`,
  `playable_client_active_loop_ui_state`) all pass at the
  implementation tip (4+4+5+6+5+4 = 28 passed, 0 failed, 0 ignored).
  No new `#[ignore]` markers introduced. Full-workspace `cargo test
  --workspace --tests --no-fail-fast` intentionally deferred to
  Sprint 13 end-of-sprint integration smoke per QA-plan-sprint-13
  binding no-full-workspace-tests-by-default policy. PASS within
  worker scope.

- [x] **AC5 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed, THEN no client-side
  mutation of authoritative state outside the shared phase sink is
  present. ADR-002 binding.
  **Closure evidence (PROMPT 844)**: the `Local<Option<RoundPhase>>`
  is consumer-private memory; it does NOT participate in phase
  truth. `Res<CurrentClientPhase>` remains read-only in
  `hand_ui_phase_transition_system`. Evidence doc §AC5 explicitly
  states "no optimistic client-side phase authority added"
  (verbatim phrase preserved). ADR-002 + ADR-009 + ADR-021 binding
  preserved. PASS.

- [x] **AC6 -- No protocol or server-side change**: GIVEN the diff
  in `shared/src/protocol.rs` and `server/`, WHEN inspected, THEN
  no functional change lands.
  **Closure evidence (PROMPT 844)**: `git diff --name-only
  fbcb03a^1 fbcb03a` returns exactly `client/Cargo.toml`,
  `client/src/ui/hand/mod.rs`,
  `production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md`,
  `tests/integration/playable_client/phase_changed_idempotency_test.rs`
  -- zero file under `shared/` or `server/`. Evidence doc §AC6
  confirms `git diff --stat origin/main -- 'shared/' 'server/'`
  empty at worker tip. PASS.

- [x] **AC7 -- Shared phase sink unchanged**: GIVEN the diff, WHEN
  the canonical shared phase sink file is reviewed, THEN it is not
  modified by this story.
  **Closure evidence (PROMPT 844)**: `client/src/presentation/mod.rs`
  (host of `phase_sink_system` +
  `apply_phase_changed_messages_with_resolution_gate`) is NOT in
  the integration commit diff. Only `client/src/ui/hand/mod.rs`,
  `client/Cargo.toml`, the integration test, and the evidence doc
  were modified. PASS.

- [x] **AC8 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, and PROMPT 761 gate-check artifact are
  diffed, THEN none of them are modified by this story.
  **Closure evidence (PROMPT 844)**: `git diff --name-only
  fbcb03a^1 fbcb03a` did NOT include any of
  `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, or
  `production/gate-checks/gate-polish-release-2026-05-12.md`. The
  PROMPT 844 `/story-done` paperwork makes the row-level flip
  (`status: ready -> done` + `completed: 2026-05-14`) which is the
  permitted disposition-preserving paperwork edit. PASS.

- [x] **AC9 -- Workspace test pass**: GIVEN `cargo test --workspace
  --tests --no-fail-fast` at the implementation commit, WHEN
  compared to the post-Sprint-12 baseline, THEN no new `#[ignore]`
  markers are introduced; the new test passes; previously-passing
  tests continue to pass.
  **Closure evidence (PROMPT 844)**: per evidence doc §AC9, the
  worker ran the narrowest BLOCKING + adjacent regression set per
  Sprint 13 QA plan no-full-workspace-tests-by-default policy
  (full-workspace `cargo test --workspace` intentionally deferred to
  end-of-sprint integration / `/team-qa`). The new test binary
  `playable_client_phase_changed_idempotency_test` passes with 5/5
  cases at worker tip; adjacent regression set 28/0/0. No
  `#[ignore]` markers introduced. PROMPT 841 integration applied a
  pure rustfmt collapse (`537ea3f`) to the new test file because
  rustfmt 1.9.0-stable (2026-04-14) on the integration host
  produced a 1-line format differing from the worker's 3-line
  chained format; zero semantic change; test still passes. PASS
  within worker scope.

- [x] **AC10 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md`
  (NEW). Records the file:line evidence, the diff summary, the
  integration-test pass output, no-claim restatement (including
  "no client-side optimistic phase authority added"), cross-link to
  PROMPT 803 §3 DC-5.
  **Closure evidence (PROMPT 844)**: evidence document exists NEW
  on `origin/main` via PROMPT 841 integration commit `fbcb03a` (270
  lines). Contents: no-claim restatement (verbatim from story
  banner with the "no client-side optimistic phase authority added"
  phrase preserved at §"No-Claim Restatement"), AC1-AC10 sectioned
  evidence, regression commands executed with Cargo resource policy
  recorded, cross-links to PROMPT 803 §3 DC-5 + Sprint 12 close-out
  deferral row + ADR-002 / ADR-009 / ADR-021. PASS.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/state/` or `client/src/presentation/` (canonical path verified by implementing worker) | Edited to narrow the `phase_changed=true` emission to actual transitions. |
| `tests/integration/playable_client/phase_changed_idempotency_test.rs` | NEW integration test asserting AC3. |
| `production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md` | NEW evidence document per AC10. |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for the `.rs` edit and for the
  integration test.
- **`liv-bevy-lightyear`** -- mandatory only if any touched file
  imports `lightyear`.

---

## Evidence Path

`production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- File:line evidence of the spurious `phase_changed=true` emission
  source.
- Diff summary for the targeted fix.
- New integration-test pass output (with explicit single-transition
  assertion).
- Confirmation that the workspace test suite still passes with no
  new `#[ignore]` markers.
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no client-side optimistic phase authority added").
- Cross-link to PROMPT 803 §3 DC-5 and to the Sprint 12 close-out
  deferral row.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test phase_changed_idempotency -- --nocapture`
  (or the new test name)
- `git diff <pre-impl-sha>..<impl-sha> -- 'shared/src/**' 'server/src/**'`
  (verifies AC6: zero protocol-shape change; zero server-side
  change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- Server-side phase authority changes (ADR-009 binding).
- Protocol changes to `S2CPhaseChanged`.
- Adding a second drainer of `S2CPhaseChanged`.
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Touches client gameplay code. Should sequence after Sprint 13 Must
  Have row `S13-OBS-TRACING-TARGETS-001` (story 018) if both touch
  the same files (`client/src/ui/hand/`, `client/src/presentation/`)
  -- worker checks for file-scope collision at activation HEAD.
- Folds with PROMPT 803 §3 DC-5; landing here resolves both
  `S11-HU-PHASE-IDEMPOTENCY-001` and DC-5.
- Wider backlog row `S13-PHASE-IDEMPOTENCY-CLIENT-001` is the same
  defect class; activator should treat this story as the canonical
  fix and not double-pull `S13-PHASE-IDEMPOTENCY-CLIENT-001`.

---

## Authoring / Implementation / Closure Trail

- **PROMPT 819 (2026-05-14)** -- Story file authored from
  `production/sprints/sprint-13.md` Sprint 13 Should Have row;
  authoring source-of-truth `origin/main@be69f5c` (PROMPT 818
  `/sprint-plan sprint-13` DRAFT). Status set Draft.
- **PROMPT 823 (2026-05-14)** -- `/story-readiness` rerun verdict
  **READY** for this story file (batch of 12 newly authored
  Sprint 13 story files reviewed).
- **PROMPT 826 (2026-05-14)** -- Sprint 13 activated (flipped
  top-level `sprint: 12 -> 13` and `status:
  closed-with-conditions -> active`). This story row promoted from
  Sprint 12 deferred Should Have to Sprint 13 active Should Have at
  status `ready`.
- **PROMPT 836 (2026-05-14)** -- `/dev-story` worker
  implementation on `work/s13-client-phase-idempotency` from base
  `origin/main@4f7ba78` (PROMPT 833 base). Worker commit
  `88106986da59263e44870ce75a032d76e1fd783e`:
  - `client/src/ui/hand/mod.rs`: narrowed `phase_changed=true` via
    `Local<Option<RoundPhase>>` comparison; bundled three
    entity-modifying queries into `HandUiPhaseTransitionQueries`
    `SystemParam` slot (preserves Bevy 0.18 16-param limit).
  - `client/Cargo.toml`: registered new integration test
    `playable_client_phase_changed_idempotency_test`.
  - `tests/integration/playable_client/phase_changed_idempotency_test.rs`
    (NEW; 181 lines worker / 179 lines post-rustfmt): 5 cases all
    pass, asserting at-most-one phase_changed fire across
    multi-frame runs with at most one transition.
  - `production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md`
    (NEW; 270 lines): AC1-AC10 closure evidence with verbatim
    no-claim restatement and PROMPT 803 §3 DC-5 cross-link.
  - Targeted regression: `cargo fmt -p client -- --check` (EXIT=0);
    `cargo check -p client` (EXIT=0); `cargo test -p client --test
    playable_client_phase_changed_idempotency_test` (5/5 pass);
    adjacent regression `cargo test -p client --test
    hand_ui_phase_state_machine_test --test
    hand_ui_placement_unstaging_test --test
    hand_ui_placement_timer_test --test hud_phase_transitions_test
    --test hud_phase_label_round_counter_test --test
    playable_client_active_loop_ui_state_test` (28/0/0 pass);
    `git diff --check origin/main...HEAD` (EXIT=0). Cargo resource
    policy applied (CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc,
    debuginfo/incremental disabled).
- **PROMPT 841 (2026-05-14)** -- Integration to `origin/main`.
  Merge commit `fbcb03ae88b6d0aaab80ac449495c89982d74579` merges
  `work/s13-client-phase-idempotency` (worker tip `8810698`) into
  integration branch built from `origin/main@0d59ba3`. Follow-up
  rustfmt-collapse commit
  `537ea3f68e6aaee49417d1f957a8c178c92144d0` collapsed a 3-line
  chained method call into a single line in the new test file
  (rustfmt 1.9.0-stable 2026-04-14 drift; pure format change; zero
  semantic change; worker `cargo fmt -p client -- --check` was
  clean on the worker host). Subsequent merge commit
  `a9e636c0dafd75f0b133c5ab90d76823da19fd3e` merged `origin/main`
  (PROMPT 837 + 840 closures) into the integration branch; fast-
  forward pushed to `origin/main`. Integration scope at `fbcb03a`:
  4 files (`client/Cargo.toml`, `client/src/ui/hand/mod.rs`,
  `production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md`,
  `tests/integration/playable_client/phase_changed_idempotency_test.rs`)
  +492/-4; AC6 (`shared/`, `server/`) and AC7
  (`client/src/presentation/mod.rs`) zero-touch verified.
- **PROMPT 843 (2026-05-14)** -- Sibling `/story-done` for the
  parallel story 019 (`S13-OBS-WALLCLOCK-TIMESTAMPS-001`); commit
  `534d9df6c7c284491e3ec36e63915abc7c4fd7e1` on `origin/main`.
  Preserves the PROMPT 836 / 841 work for story 022 unchanged on
  `origin/main`.
- **PROMPT 844 (2026-05-14)** -- `/story-done` paperwork closure
  at root checkout against `origin/main@534d9df` (serialized
  shared-status writer per 2026-05-13 override; matches PROMPT 843
  paperwork pattern). AC1-AC10 all verified against integrated
  evidence on `origin/main`. Files modified:
  - This story file (Status flipped Draft -> Done with PROMPT 844
    closure context; AC1-AC10 checkboxes `[ ]` -> `[x]` with per-AC
    closure-evidence annotations; Authoring / Implementation /
    Closure Trail appended).
  - `production/sprint-status.yaml` (Sprint 13 Should Have row
    `S11-HU-PHASE-IDEMPOTENCY-001` flipped `status: ready -> done`
    with `completed: 2026-05-14`, `worker_prompt: 836`,
    `worker_commit: 8810698`, `integration_prompt: 841`,
    `integration_commit: a9e636c0dafd75f0b133c5ab90d76823da19fd3e`,
    `story_done_prompt: 844`,
    `test_evidence: tests/integration/playable_client/phase_changed_idempotency_test.rs`,
    `acceptance_evidence: production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md`;
    top-level `updated:` annotation refreshed for PROMPT 844;
    `sprint_13_story_done:` block extended with PROMPT 844 entry
    as a sibling to the prior PROMPT 833 + 840 + 843 entries).
  - `production/session-state/active.md` (PROMPT 844 banner
    prepended above PROMPT 843 banner).
  - `production/session-state/codex-orchestrator-state.md`
    (PROMPT 844 section prepended above PROMPT 843 section).
- **Cargo policy**: N/A for PROMPT 844 itself (paperwork-only
  closure; no cargo command invoked). Worker (PROMPT 836)
  regression run applied the binding Windows/MSVC Cargo resource
  policy at its targeted regression checkpoints.

## Conditions carried forward unchanged

- S8-QA-001-W1 manual/browser two-client GAME_OVER gap remains
  OPEN. Story 017 AC12 forbid-auto-closure preserved.
- QA-COND-0005 Standard-tier accessibility accepted-risk
  (friend-game scope only).
- QA-COND-0006 playtest / fun-hypothesis validation accepted-risk
  / deferred.
- PAW-TD-*-a placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved (folded
  into Sprint 12 story 019 cannot-reproduce closure; third
  same-scope retest NOT authorised per TQ-S12-C2). PROMPT 844 does
  NOT re-attempt the Sprint 12 capture.
- PROMPT 761 Polish->Release gate-check FAIL preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no
  retry in PROMPT 844 scope.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed (closed cannot-reproduce, NOT bug-fixed).
- TQ-S12-C1..C7 (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817
  preserved unchanged.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- Prior `/story-done` closures preserved unchanged on
  `origin/main`: PROMPT 833 (`S11-SERVER-POOL-INIT-LOG-GUARD-001`),
  PROMPT 835 (`S11-LOBBY-UX-CONFIRM-STATE-001`), PROMPT 840
  (`S13-UI-AUDIT-ROADMAP-PREP-001`), PROMPT 843
  (`S13-OBS-WALLCLOCK-TIMESTAMPS-001`).
- DC-5 follow-on application of the same
  `Local<Option<RoundPhase>>` pattern to
  `client/src/ui/hud/mod.rs` (lines 390, 1247) and
  `client/src/ui/shop_auction/mod.rs` (line 1248) -- those
  consumers early-return on `current.is_changed()` without
  emitting a spurious `phase_changed=true` tracing log, so they
  are performance-impact-only and were intentionally OUT of this
  story's narrow scope. A follow-on story may extend the pattern
  to those consumers; that is NOT undertaken here.

## Explicitly NOT claimed by PROMPT 844

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- Stage advance from Polish to Release
- underlying drag-runtime bug fix (Sprint 12 story 019 closed
  cannot-reproduce, NOT bug-fixed)
- full UI clean-pass repair
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`
- closure of `S13-PHASE-IDEMPOTENCY-CLIENT-001` (same-class
  defect for HUD + shop-auction consumers; out of this story's
  narrow scope)
- Sprint 13 close-out (Sprint 13 remains active; 5 of 19 rows
  closed after PROMPT 844 -- 1 of 6 Must Have, 3 of 6 Should
  Have, 1 of 7 Nice to Have)
- full-workspace `cargo test --workspace --tests --no-fail-fast`
  result claim (deferred to orchestrator end-of-sprint
  integration gate per QA-plan-sprint-13 no-full-workspace-tests-
  by-default policy)
