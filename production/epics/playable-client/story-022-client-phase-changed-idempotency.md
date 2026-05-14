# Story 022: S11-HU-PHASE-IDEMPOTENCY-001 -- Client `phase_changed=true` 60Hz Idempotency

> **Epic**: Playable Client
> **Story ID**: S11-HU-PHASE-IDEMPOTENCY-001
> **Status**: Draft -- Sprint 13 candidate (Should Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: Client gameplay -- shared phase sink / HU phase consumer
> **Type**: Integration -- targeted fix to phase-change consumer + integration
> test
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; PROMPT 803
> §3 DC-5 same-class candidate); NOT activated
> **Authored**: 2026-05-14 by PROMPT 819
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)

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

- [ ] **AC1 -- Spurious signal source located**: GIVEN the
  implementation prompt's first read pass, WHEN the phase consumer
  in `client/src/` is located, THEN the exact `phase_changed=true`
  emission point is named with file:line evidence in the evidence
  document.

- [ ] **AC2 -- Idempotency fix lands**: GIVEN the located source,
  WHEN the fix lands, THEN `phase_changed=true` is set only on the
  frame where `Res<CurrentClientPhase>` actually transitions (i.e.,
  the consumer compares the just-observed phase value to the prior
  frame's value and emits `true` only on inequality).

- [ ] **AC3 -- Integration test asserts narrowed signal**: GIVEN a
  new or extended integration test (e.g.,
  `tests/integration/playable_client/phase_changed_idempotency_test.rs`),
  WHEN the test drives a multi-frame run with at most one phase
  transition, THEN it asserts `phase_changed=true` fires at most
  once across all frames.

- [ ] **AC4 -- Existing phase-driven UI unaffected**: GIVEN the
  Sprint 12 phase-driven UI tests (HUD phase label, hand-UI phase
  transitions, shop-auction-UI phase transitions, board-rendering
  phase transitions), WHEN the workspace test suite runs at the
  implementation commit, THEN all previously-passing phase tests
  still pass.

- [ ] **AC5 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed, THEN no client-side
  mutation of authoritative state outside the shared phase sink is
  present. ADR-002 binding.

- [ ] **AC6 -- No protocol or server-side change**: GIVEN the diff
  in `shared/src/protocol.rs` and `server/`, WHEN inspected, THEN
  no functional change lands.

- [ ] **AC7 -- Shared phase sink unchanged**: GIVEN the diff, WHEN
  the canonical shared phase sink file is reviewed, THEN it is not
  modified by this story.

- [ ] **AC8 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, and PROMPT 761 gate-check artifact are
  diffed, THEN none of them are modified by this story.

- [ ] **AC9 -- Workspace test pass**: GIVEN `cargo test --workspace
  --tests --no-fail-fast` at the implementation commit, WHEN
  compared to the post-Sprint-12 baseline, THEN no new `#[ignore]`
  markers are introduced; the new test passes; previously-passing
  tests continue to pass.

- [ ] **AC10 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-hu-phase-idempotency-evidence.md`
  (NEW). Records the file:line evidence, the diff summary, the
  integration-test pass output, no-claim restatement (including
  "no client-side optimistic phase authority added"), cross-link to
  PROMPT 803 §3 DC-5.

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
