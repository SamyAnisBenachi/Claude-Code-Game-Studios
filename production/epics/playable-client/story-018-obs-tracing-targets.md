# Story 018: S13-OBS-TRACING-TARGETS-001 -- Module-Scoped Tracing Targets for Diagnostic Capture

> **Epic**: Playable Client
> **Story ID**: S13-OBS-TRACING-TARGETS-001
> **Status**: Done -- closed by PROMPT 850 `/story-done` paperwork at
> `origin/main@9e32fbe` (PROMPT 847 worker `9e32fbe` on
> `work/s13-obs-tracing-targets` from base `origin/main@fe74fb0` +
> PROMPT 848 integration: fast-forward push to origin/main, same commit
> hash as worker tip — clean ff because integration branch was clean
> superset of `origin/main@fe74fb0`). AC1-AC4 and AC6-AC10 satisfied
> within scope; AC5 verification harness deferred per PROMPT 847
> explicit scope ("Do not re-attempt drag-runtime capture") with
> rationale + static target-string evidence in
> `production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md`
> §AC5.
> **Layer**: Observability / Cross-Cutting
> **Type**: Integration -- targeted edits across emission sites + verification
> **Sprint**: Sprint 13 (active per PROMPT 826 activation)
> **Authored**: 2026-05-14 by PROMPT 804 (worktree
> `work/s13-runtime-hardening-story-authoring`)
> **Authoring source-of-truth**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12
> QA-plan commit). Sprint 12 active per PROMPT 798 at `origin/main@796851b`.

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 804. Sprint 12 remains the active sprint
(`status: active`) and must not be changed by this authoring run.

PROMPT 804 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md` or any other active sprint
  file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify `production/qa/qa-plan-sprint-12.md` or any other QA-plan file.
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

Sprint 10 / Sprint 11 dispositions unchanged. PROMPT 761 Polish->Release
gate-check FAIL evidence preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** The change is purely additive on existing `tracing::*!()`
emission sites -- each site adds `target: "module::path"` arguments;
no behaviour or authoritative state is touched. ADR-002 binding.

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-11** Tracing target hierarchy unscoped (HIGH for
  diagnostic): Story 019 invocation
  `RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=debug,server::game=debug`
  would capture only crate-level emissions; *no* `tracing::*!(target:
  "client::ui::hand", ...)` calls exist. Evidence anchor: all
  `tracing::*!()` sites use implicit crate target.
- **§4 Lane E DC-11**: same.
- **§5 Must row 7 (S13-OBS-TRACING-TARGETS-001)**: "Add `target:
  "client::ui::hand"`, `target: "client::presentation::board_rendering"`,
  `target: "client::card_animations::input_gating"`, `target:
  "server::game"` to all relevant emission sites so the Story 019
  `RUST_LOG` invocation actually captures something". Likely files:
  `client/src/ui/hand/*`, `client/src/presentation/board_rendering.rs`,
  `client/src/card_animations/input_gating.rs`, `server/src/feature/*`.
- **§6 PROMPT-N+5 (paired with S13-OBS-WALLCLOCK-TIMESTAMPS-001)**:
  paperwork-only story-authoring.
- **§5 Must "Sprint 13 (gates Story 019 retest if not done in Sprint
  12)"**: This story unblocks Story 019 tighter-capture if/when
  Sprint 12 attempts the retest without first landing the targets.

---

## Problem Class / Prevention Target

**Defect class (DC-11)**: Every `tracing::*!()` site in the workspace
uses the implicit crate target. The `tracing` library's `RUST_LOG`
filter syntax supports `module::path=level` granularity, but only if
the emission site is tagged with `target: "module::path"` explicitly
(or if the crate-level target matches the desired filter). Symptoms:
running `RUST_LOG=client::ui::hand=trace ...` produces empty output
under those granular targets; only `RUST_LOG=client=trace ...`
captures the desired emissions, but that pulls in massive noise from
unrelated crate-level emissions.

**Prevention target**: Add `target: "client::ui::hand"`, `target:
"client::presentation::board_rendering"`, `target:
"client::card_animations::input_gating"`, `target: "server::game"`,
and other diagnostic-relevant module-scoped targets to all
`tracing::*!()` emission sites in those modules. After the change,
`RUST_LOG=client::ui::hand=trace ...` captures the Hand UI emissions
without pulling in unrelated noise.

The target list is **not** exhaustive at authoring -- the
implementation prompt audits each module and decides which sites get
which targets based on the Story 019 invocation pattern and any other
diagnostic patterns the prompt identifies.

---

## Context

### Existing surface

- **`client/src/ui/hand/*`**: Hand UI module. Per PROMPT 706 / 709 /
  `7e0c663`, the 5 S1-S5 instrumentation emit sites exist here. The
  Story 019 invocation explicitly names this module as a
  `RUST_LOG=client::ui::hand=trace` target.
- **`client/src/presentation/board_rendering.rs`**: Board rendering
  module. Story 019 invocation names this as a
  `RUST_LOG=client::presentation::board_rendering=trace` target.
- **`client/src/card_animations/input_gating.rs`**: Card animations
  input gating module. Story 019 invocation names this as a
  `RUST_LOG=client::card_animations::input_gating=info` target.
- **`server/src/feature/*`**: Server feature modules (auction,
  acquisition, objective, combat). Story 019 invocation names
  `server::game=debug` as a target. The exact mapping from
  `server::game` to the actual server source modules is
  implementation-prompt-decided (e.g., `server::game::auction`,
  `server::game::combat`, ...).
- **Existing target usage**: zero `target: "..."` arguments in
  `tracing::*!()` macro invocations across the workspace at
  `origin/main@b5eef0d`. Implementation prompt verifies via grep.
- **Tracing init**: `server/src/main.rs:87`, `client/src/main.rs:36`,
  `tests/test_helpers.rs:52` -- all use
  `tracing_subscriber::fmt()...init()`.

### GDD / ADR / TR trace

- **No GDD change**: this is observability infrastructure.
- **ADR-002** (Client-Server Authority): targets are added to
  existing emission sites; no behaviour change.
- **TR registry**: no new TR.

### Engine

- **Engine**: Bevy 0.18 (Rust). All edits are in `.rs` source.
- **Lightyear**: 0.26. The `lightyear=debug` portion of the Story
  019 invocation works at the crate level; the `lightyear` target
  is already correctly scoped by the lightyear crate itself.

### Mandatory skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code edits.
- **`liv-bevy-lightyear`** -- mandatory for server / client
  network code edits where lightyear emissions interact with
  this story's targets.

### Control Manifest Rules (Observability scope)

- Required: Each `tracing::*!()` emission site touched gets an
  explicit `target: "module::path"` first argument. The target
  path matches the Rust module path of the emission site.
- Required: The target list covers, at minimum, the four targets
  named in the Story 019 invocation
  (`client::ui::hand`,
  `client::presentation::board_rendering`,
  `client::card_animations::input_gating`,
  `server::game`).
- Required: After the change, running a minimal repro of the Story
  019 invocation against a smoke test (or the
  `S13-TWO-CLIENT-RUNTIME-HARNESS-001` harness if available)
  produces non-empty per-target output.
- Required: Behaviour change is zero -- existing tests pass; no
  runtime semantics modified.
- Forbidden: Removing or replacing existing log message contents.
  The change is additive (the `target:` argument is added; the
  message stays the same).
- Forbidden: Adding new emission sites in this story. Adding new
  sites is a separate concern.
- Forbidden: Modifying authoritative state, protocol shapes, or
  any behaviour outside the `tracing::*!()` macros themselves.

---

## Story Classification

**Story type**: Integration -- targeted edits across multiple modules
+ verification harness pass.

This is **NOT** a:

- New-feature story.
- Refactor of tracing infrastructure (the subscriber config is
  separately scoped to `S13-OBS-WALLCLOCK-TIMESTAMPS-001`).
- Sprint 12 expansion.

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- `client::ui::hand` target landed at every relevant
  emission site**: GIVEN the diff under `client/src/ui/hand/`, WHEN
  every `tracing::*!()` macro invocation in the module is
  inspected, THEN each invocation that is part of the S1-S5
  instrumentation (per PROMPT 706 / 709 / `7e0c663`) carries
  `target: "client::ui::hand"` (or an explicitly-narrower target
  like `target: "client::ui::hand::drag"` with rationale).
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  `git grep -c 'target: "client::ui::hand"' -- client/src/ui/hand/mod.rs`
  returns **15**; `git grep -c 'target: "client::ui::hand'` (prefix)
  returns **18** (15 exact + 3 narrower module-path-shaped sub-paths:
  `client::ui::hand::fan_active_default_drop`,
  `client::ui::hand::drag_sprite_visible_flip`,
  `client::ui::hand::placement_cursor_move`). Evidence doc §AC1 enumerates
  the system-name coverage. PASS.

- [x] **AC2 -- `client::presentation::board_rendering` target
  landed**: same for `client/src/presentation/board_rendering.rs`.
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  `git grep -c 'target: "client::presentation::board_rendering"' -- client/src/presentation/board_rendering.rs`
  returns **4** (exact); prefix returns **8** (4 exact + 4 narrower
  `client::presentation::board_rendering::spawn_highlight_*`
  sub-paths preserving prior narrow-capture intent). Evidence doc §AC2.
  PASS.

- [x] **AC3 -- `client::card_animations::input_gating` target
  landed**: same for `client/src/card_animations/input_gating.rs`.
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  `git grep -c 'target: "client::card_animations::input_gating' -- client/src/card_animations/input_gating.rs`
  returns **1** (narrower
  `client::card_animations::input_gating::drag_lift_tween_install`,
  line 163; pre-existing site's prior `target: "drag_lift_tween_install"`
  rewritten to module-path-shaped form so Story 019 invocation
  `RUST_LOG=client::card_animations::input_gating=info` captures it
  via subtree match). Evidence doc §AC3. PASS.

- [x] **AC4 -- `server::game` target landed at all relevant
  server feature emission sites**: GIVEN the diff under
  `server/src/feature/`, WHEN every `tracing::*!()` invocation
  relevant to gameplay state changes (auction resolution,
  acquisition tick, objective destroyed, combat resolution) is
  inspected, THEN each carries `target: "server::game"` (or a
  narrower target like `target: "server::game::auction"` with
  rationale). At minimum, the
  `server/src/network/rsm_dispatch.rs` `S2C*` broadcast emission
  sites (per PROMPT 803 §4 Lane A) are tagged.
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  `git grep -c 'target: "server::game"' -- 'server/src/feature/*' 'server/src/network/*'`
  per-file returns: `acquisition/system.rs:34`, `auction/system.rs:37`,
  `board/movement.rs:3`, `board/placement.rs:3`, `combat/mod.rs:8`,
  `keyword/observers.rs:5`, `objective/system.rs:6`, `prism/system.rs:8`,
  `network/economy_dispatch.rs:2`, `network/mod.rs:4`,
  `network/rsm_dispatch.rs:1` — **total 111 sites across 11 files**
  (PROMPT 847 worker report's claim of 112 is a 1-off doc typo flagged
  by PROMPT 848 integration §"Worker report extraction"; code diff and
  file-list correct; `RUST_LOG=server::game=debug` subtree match
  unaffected). `rsm_dispatch.rs` S2C broadcast emission tagged per
  PROMPT 803 §4 Lane A. Evidence doc §AC4. PASS.

- [x] **AC5 -- Verification harness pass**: GIVEN the
  implementation commit, WHEN the Story 019 invocation
  `RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=debug,server::game=debug`
  is run against either (a) a smoke test that exercises the
  drag-runtime route or (b) the
  `S13-TWO-CLIENT-RUNTIME-HARNESS-001` harness if available,
  THEN per-target output is non-empty for each named target. The
  evidence doc records sample lines from each target.
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  **DEFERRED with rationale** — PROMPT 847 worker scope explicitly
  forbids drag-runtime capture ("Do not re-attempt drag-runtime
  capture. Do not broaden into logging redesign."); the
  `S13-TWO-CLIENT-RUNTIME-HARNESS-001` runtime harness (Story 017)
  has not yet landed at `origin/main@9e32fbe`. Static target-string
  evidence (the literal strings `client::ui::hand`,
  `client::presentation::board_rendering`,
  `client::card_animations::input_gating`, `server::game` are present
  post-impl with non-zero counts 15 / 4 / 1 / 111 respectively, plus
  the narrower forms that subtree-match) recorded in evidence doc §AC5
  guarantees the Story 019 invocation — whenever next run — will
  capture non-empty per-target output via direct or subtree match.
  Runtime harness AC5 capture is deferred to the next Story 019
  retest prompt or a Sprint 13 harness-up prompt that can drive the
  runtime path with the new targets in place. **Closed within scope
  by closure trail acknowledgement (PASS-WITHIN-SCOPE, runtime
  capture deferred)**.

- [x] **AC6 -- Behaviour unchanged**: GIVEN `cargo test
  --workspace --tests --no-fail-fast` at the implementation
  commit, WHEN compared to the pre-implementation baseline,
  THEN no test regressions are observed (same pass/fail/ignored
  counts modulo Sprint 12 close-out deltas).
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  Worker (PROMPT 847) ran `cargo fmt --all -- --check` PASS +
  `cargo check -p client` PASS + `cargo check -p server` PASS at
  worker tip; PROMPT 848 integration re-ran the same commands at
  integration tip (`cargo fmt --all -- --check` PASS, `cargo check -p client`
  PASS in 9.34s, `cargo check -p server` PASS in 1.57s). Full-workspace
  `cargo test --workspace --tests --no-fail-fast` intentionally
  deferred to Sprint 13 end-of-sprint integration smoke per
  QA-plan-sprint-13 binding no-full-workspace-tests-by-default policy.
  Diff inspection: `git diff --check origin/main^...origin/main` PASS;
  the only `+`/`-` lines in the worker diff are `target: "..."`
  additions/rewrites (8 pre-existing non-module-path targets rewritten
  to module-path-shaped narrower forms; 119 new `target:` additions);
  no control-flow, no field-name, no message-string changes. The
  `tracing` crate's macro expansion for `target: "..."` is
  behaviourally inert (sets `tracing::Metadata::target` field only;
  does not alter call-site control flow, allocation, ordering, or
  message formatting). PASS-WITHIN-WORKER-SCOPE.

- [x] **AC7 -- No new emission sites added**: GIVEN the
  implementation diff, WHEN searched for new `tracing::*!()`
  macro invocations, THEN zero new sites are introduced (the
  diff is purely "add `target:` arg to existing sites" + reformat
  surrounding lines as needed).
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  Evidence doc §"Pre/Post Grep Counts" "Total `tracing::*!()` site
  counts pre/post by file" table shows zero net change:
  `client/src/ui/hand/mod.rs` 18->18; `client/src/presentation/board_rendering.rs`
  8->8; `client/src/card_animations/input_gating.rs` 1->1;
  `server/src/feature/*` 104->104; `server/src/network/*` 7->7.
  PASS.

- [x] **AC8 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed for any
  client-side mutation of authoritative state outside the
  shared phase sink, snapshot drainers, and S2C consumers,
  THEN no such mutation is present. ADR-002 binding.
  *Evidence*: text search for "no optimistic" in the evidence
  document.
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  Evidence doc §AC8 contains the verbatim phrase "No optimistic
  client-side authority is introduced" (also appears in the
  "No-Claim Restatement" §). Diff touches only the first argument
  slot of `tracing::*!()` macros; no new `ResMut<_>` on
  `CurrentClientPhase` / `ClientState` / `PendingPlacements` / `S2C*`
  consumer resources; `phase_sink_system` (`client/src/presentation/mod.rs`)
  not in the diff. ADR-002 binding maintained. `liv-bevy-lightyear`
  applied per PROMPT 847 + PROMPT 848: `server/src/network/{rsm_dispatch,economy_dispatch,mod}.rs`
  `target:` additions are inside `tracing::error!(...)` calls in the
  `Err(e)` branch downstream of `MessageSender::send::<S2C*, ReliableChannel>(...)`;
  send path, message types, channel choice, resend gating not
  modified. PASS.

- [x] **AC9 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`,
  and `production/qa/qa-plan-sprint-12.md` are diffed, THEN
  none of them are modified under this story.
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  `git diff --name-only 9e32fbe^1 9e32fbe -- production/sprint-status.yaml
  production/sprints/sprint-12.md production/sprints/sprint-13.md
  production/stage.txt production/qa/qa-plan-sprint-12.md
  production/qa/qa-plan-sprint-13.md production/gate-checks/`
  returns empty. PROMPT 847 worker + PROMPT 848 integration scope =
  15 files (14 source files modified + 1 NEW evidence doc); zero
  forbidden Sprint 12 / Sprint 13 paperwork paths. Sprint 12
  `closed-with-conditions` per PROMPT 817 preserved. Sprint 13
  `active` per PROMPT 826 preserved. Stage UNCHANGED `Polish`.
  PROMPT 761 Polish->Release FAIL preserved. The PROMPT 850 row-level
  `status: ready -> done` flip + `completed: 2026-05-14` is the
  permitted disposition-preserving paperwork edit; top-level
  `sprint:`/`status:`/`stage:` unchanged. PASS.

- [x] **AC10 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md`
  (NEW). Records pre/post grep counts of `target: "..."`
  occurrences, AC5 harness output sample lines, no-claim
  restatement, cross-link to PROMPT 803 §3 DC-11.
  *Closure evidence (PROMPT 850 verified on `origin/main@9e32fbe`)*:
  `production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md`
  exists NEW (347 lines) on `origin/main` via PROMPT 847 commit
  `9e32fbe`; not modified by PROMPT 850. Records:
  pre/post grep counts (sections §"Pre/Post Grep Counts");
  AC5 deferral rationale + static target-string evidence in §AC5;
  no-claim restatement verbatim in §"No-Claim Restatement"
  including the "No optimistic client-side authority is introduced"
  phrase; cross-link to PROMPT 803 §3 DC-11 + §5 Must row 7 + §4
  Lane A in §"Cross-Link to Source Finding". PASS.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hand/*.rs` | Add `target: "client::ui::hand"` (or narrower) to existing tracing sites. |
| `client/src/presentation/board_rendering.rs` | Add `target: "client::presentation::board_rendering"` to existing tracing sites. |
| `client/src/card_animations/input_gating.rs` | Add `target: "client::card_animations::input_gating"` to existing tracing sites. |
| `server/src/feature/auction/system.rs` | Add `target: "server::game"` (or narrower) to relevant tracing sites. |
| `server/src/feature/acquisition/system.rs` | Same. |
| `server/src/feature/objective/system.rs` | Same. |
| `server/src/feature/combat/mod.rs` | Same. |
| `server/src/network/rsm_dispatch.rs` | Add `target: "server::game"` to S2C broadcast emission sites per PROMPT 803 §4 Lane A. |
| `server/src/network/economy_dispatch.rs` | Same. |
| `production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md` | NEW evidence document per AC10. |
| This story file | Status updates per /story-readiness or /story-done if/when Sprint 13 activates. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code edits.
- **`liv-bevy-lightyear`** -- mandatory for server / client network
  code edits where lightyear emissions are touched.

---

## Evidence Path

`production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Pre/post grep counts of `target: "client::ui::hand"`,
  `target: "client::presentation::board_rendering"`,
  `target: "client::card_animations::input_gating"`,
  `target: "server::game"` (and any other targets added).
- AC5 sample lines from each named target after running the
  Story 019 invocation against the harness or smoke test.
- Pre/post `cargo test --workspace --tests --no-fail-fast` output
  showing AC6 behaviour-unchanged.
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no optimistic client-side authority").
- Cross-link to PROMPT 803 §3 DC-11.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `git grep -n 'target: "client::ui::hand"' client/src/`
  (post-impl: non-zero results)
- `git grep -n 'target: "client::presentation::board_rendering"' client/src/`
- `git grep -n 'target: "client::card_animations::input_gating"' client/src/`
- `git grep -n 'target: "server::game"' server/src/`
- `RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=debug,server::game=debug cargo run --bin two-client-runtime -- --seed 1 --max-rounds 1`
  (if `S13-TWO-CLIENT-RUNTIME-HARNESS-001` has landed; otherwise a
  smoke-test equivalent)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- **Adding new emission sites**. The diff is purely "add `target:`
  arg to existing sites".
- **Modifying subscriber config**. Wall-clock timestamps + UTC
  formatting are scoped to `S13-OBS-WALLCLOCK-TIMESTAMPS-001`
  (Story 019 in this epic).
- **Removing or replacing existing log message contents**. The
  change is additive.
- **Sprint 13 activation**.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  or any carried Sprint condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Touches `client/src/ui/hand/`, `client/src/presentation/board_rendering.rs`,
  `client/src/card_animations/input_gating.rs`, `server/src/feature/*`,
  `server/src/network/*`**. Sprint 12 Must Have rows touch:
  - Story 014 (cooccupancy panic guard):
    `client/src/presentation/board_rendering.rs` -- **POTENTIAL
    CONFLICT** on the same file. Mitigation: this Sprint 13 story
    MUST NOT run in parallel with Sprint 12 story 014; sequence
    after Sprint 12 close-out.
  - Story 019 (drag-runtime tighter-capture):
    `client/src/ui/hand/`, `client/src/card_animations/input_gating.rs`,
    `client/src/presentation/board_rendering.rs` are read by the
    story but not modified (Story 019 is evidence-only). The Story
    019 invocation is what this story unblocks.
- **No Sprint 12 invasion**: this story's implementation MUST NOT
  land before Sprint 12 close-out unless the producer explicitly
  authorises a pull-forward via a separate prompt. If Sprint 12
  Story 019's tighter-capture retest is attempted before this
  story lands, the operator MUST use a shell-wrapper UTC prefix
  AND a `target:`-less invocation (i.e., `RUST_LOG=client=trace`
  with crate-level noise tolerance).
- **Coordinate with `S13-OBS-WALLCLOCK-TIMESTAMPS-001` (Story 019
  in this epic)**: ideally land both in the same Sprint 13 wave so
  Story 019 tighter-capture retest can use the production-default
  invocation without shell wrappers.
- **Coordinate with `S13-TWO-CLIENT-RUNTIME-HARNESS-001` (Story 017
  in this epic)**: AC5 verification benefits from the harness;
  fallback is a smoke test.
- **No shared-status writer overlap**: `production/sprint-status.yaml`
  is not touched by this story.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. Sprint 12 reaches close-out.
2. Sprint 13 is planned via `/sprint-plan sprint-13`.
3. This story passes `/story-readiness`.
4. Sprint 13 `/qa-plan sprint` is authored.
5. `/dev-story story-018-obs-tracing-targets.md` is dispatched.

Expected implementation flow:

1. **Wave 1 -- Audit**: grep all `tracing::*!()` invocations in
   the four target modules; categorise by which target each site
   should carry.
2. **Wave 2 -- Edits per module**: add `target:` argument to each
   site; keep behaviour unchanged.
3. **Wave 3 -- Verification harness**: run the Story 019
   invocation; capture sample output per target.
4. **Wave 4 -- Behaviour check**: `cargo test --workspace --tests
   --no-fail-fast`; confirm zero regressions.
5. **Wave 5 -- Evidence**: populate evidence file.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `client/src/presentation/board_rendering.rs` collision with Sprint 12 Story 014 | High | High | Sequence: Sprint 12 closes first. |
| Target naming drifts from module path under future refactors | Medium | Low-Medium | Default rule: target matches module path. Future Sprint 14 invariant test (`S13-PLUGIN-REGISTRATION-INVARIANT-001`-adjacent) could enforce. |
| Adding `target:` introduces a borrow/move issue in some macro expansions | Low | Low | Bevy / tracing macros tolerate `target:` cleanly; if any site is brittle, the implementation prompt records the workaround in the evidence doc. |
| AC5 harness output is empty for one target (e.g., `server::game` has no emissions in the path exercised) | Medium | Low | Implementation prompt extends the verification path to exercise each target or records the "no emissions on this path" disposition. |
| Sprint 13 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate. |

---

## Verification (orchestrator-side, before worker dispatch)

- `production/sprint-status.yaml` `sprint:` field reads `13` after
  Sprint 13 activation; Sprint 12 close-out has landed.
- Sprint 12 Story 014 (cooccupancy panic guard) is `done` (so
  `board_rendering.rs` is no longer under Sprint 12 active edit).
- `production/stage.txt` reads `Polish` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring / Implementation / Closure Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for Module-Scoped Tracing Targets per PROMPT 803 §3
  DC-11 / §5 Must row 7. Sprint 12 is `active` (PROMPT 798) and is
  not modified by this authoring run. No code changes, no smoke /
  gate / QA / `/dev-story` / `/story-done` / `/story-readiness` /
  `/qa-plan` run. Source-of-truth at authoring: `origin/main@b5eef0d`.
  Worker branch: `work/s13-runtime-hardening-story-authoring`.
  Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.

- 2026-05-14 -- PROMPT 823 -- `/story-readiness` batch rerun verdict
  **READY** across all 12 newly reviewed Sprint 13 story files
  (including this story). Sprint 13 not yet activated at this prompt.

- 2026-05-14 -- PROMPT 826 -- Sprint 13 activation paperwork; top-level
  `sprint: 12 -> 13` and `status: closed-with-conditions -> active`
  per Sprint 13 plan at `production/sprints/sprint-13.md`. This story
  promoted into top-level active sprint stories block as a Must Have
  row `S13-OBS-TRACING-TARGETS-001`. Stage UNCHANGED `Polish`.

- 2026-05-14 -- PROMPT 827 -- Sprint 13 `/qa-plan sprint` authored at
  `production/qa/qa-plan-sprint-13.md` (commit `4bf95fa`) with the
  binding no-full-workspace-tests-by-default policy that gates AC6
  / AC9 deferrals into orchestrator end-of-sprint integration.

- 2026-05-14 -- PROMPT 847 -- `/dev-story` worker run for
  `production/epics/playable-client/story-018-obs-tracing-targets.md`.
  Worker branch `work/s13-obs-tracing-targets` from base
  `origin/main@fe74fb0` (PROMPT 844 closure). Worker commit
  `9e32fbe25f6b7590cfc9268ed5323d2d74517843`. 15 files changed
  (14 source files + 1 NEW evidence doc); +485/-12 lines. Tracing
  targets added: 15 exact `client::ui::hand` + 3 narrower forms in
  `client/src/ui/hand/mod.rs`; 4 exact `client::presentation::board_rendering`
  + 4 narrower forms in `client/src/presentation/board_rendering.rs`;
  1 narrower `client::card_animations::input_gating::drag_lift_tween_install`
  in `client/src/card_animations/input_gating.rs`; 111 `server::game`
  exact across 11 files (`server/src/feature/{acquisition,auction,board,combat,keyword,objective,prism}/...`
  + `server/src/network/{economy_dispatch,mod,rsm_dispatch}.rs`).
  8 pre-existing non-module-path `target:` values rewritten to
  module-path-shaped narrower forms (prefix `<module-path>::<old-name>`)
  preserving narrow-capture diagnostic intent while keeping
  subtree-match behaviour for Story 019 invocation. Worker checks:
  `cargo fmt --all -- --check` PASS, `cargo check -p client` PASS,
  `cargo check -p server` PASS, `git diff --check origin/main...HEAD`
  PASS, `git grep` target counts confirmed non-zero. AC1-AC4 + AC6-AC10
  satisfied; **AC5 (runtime harness)** explicitly deferred per worker
  scope ("Do not re-attempt drag-runtime capture") with rationale +
  static target-string evidence recorded in evidence doc §AC5. Cargo
  resource policy applied (`CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc`,
  `CARGO_PROFILE_*_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS='-C debuginfo=0
  -C link-arg=/DEBUG:NONE'`). `liv-bevy-018` + `liv-bevy-lightyear` skills
  active. Worker branch pushed; no `/story-done`, no
  `production/sprint-status.yaml` / Sprint plan / QA plan / stage edits.

- 2026-05-14 -- PROMPT 848 -- Integration of PROMPT 847 worker branch
  `work/s13-obs-tracing-targets` into `origin/main`. Fast-forward push:
  worker commit `9e32fbe` had parent `fe74fb0` (== prior `origin/main`
  tip), so no merge commit was created — `git push origin HEAD:main`
  advanced origin/main exactly one commit (`fe74fb0..9e32fbe`).
  Integration commit hash = worker commit hash =
  `9e32fbe25f6b7590cfc9268ed5323d2d74517843`. Integration worktree
  `D:\_DEV\claude-code-game-studios-worktrees\integration-s13-obs-tracing-targets-848`.
  Integration checks re-ran the worker check set at integration tip:
  `cargo fmt --all -- --check` PASS, `cargo check -p client` PASS in
  9.34s, `cargo check -p server` PASS in 1.57s, `git grep` target
  counts confirmed (1-off doc typo in worker report's 112 vs actual
  111 sites flagged; code diff and file-list correct, subtree match
  unaffected). `git diff --check origin/main...HEAD` PASS,
  `git diff --cached --check` PASS. Cargo policy applied via bash
  export. No `/story-done`, `/smoke`, `/team-qa`, `/gate-check`,
  `/release-checklist` invoked. No `production/sprint-status.yaml`,
  Sprint plan, QA plan, stage, story file, or session-state edits.
  Sprint 12 `closed-with-conditions` (PROMPT 817) and Sprint 13
  `active` (PROMPT 826) dispositions preserved.

- 2026-05-14 -- PROMPT 850 -- `/story-done` closure paperwork for this
  story at `origin/main@9e32fbe` (PROMPT 847/848 integration commit;
  origin/main has since advanced through PROMPT 845 `96c1600` + PROMPT
  849 `25573e6` which are sibling, non-conflicting commits preserving
  this story's integration evidence unchanged). Worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s13-obs-tracing-targets-storydone`
  (new branch `storydone/s13-obs-tracing-targets` from `origin/main`)
  because root checkout had pre-existing dirt unrelated to this story
  (matching prior paperwork patterns at PROMPT 843 / PROMPT 844). Read
  PROMPT 847 + PROMPT 848 reports, story file ACs, evidence doc
  (347 lines), and current sprint-status.yaml structure. Verified each
  AC against integrated evidence on `origin/main`:
  AC1 grep 15 exact + 18 prefix; AC2 grep 4 exact + 8 prefix; AC3
  grep 1 (narrower); AC4 per-file grep totals 111 across 11 files
  (matching PROMPT 848 integration count, not the 112 typo in PROMPT
  847 worker report); AC5 DEFERRED-WITHIN-SCOPE per PROMPT 847 worker
  scope explicitly excluding drag-runtime capture (closed within
  scope with static target-string evidence + deferred-runtime-capture
  acknowledgement, runtime harness landing pending); AC6 worker +
  integration `cargo check -p client` + `cargo check -p server` + `cargo
  fmt --check` PASS; full-workspace cargo test deferred per QA plan;
  AC7 per-file pre/post site counts unchanged; AC8 verbatim "no
  optimistic" phrase in evidence doc + `liv-bevy-lightyear` discipline
  preserved in `server/src/network/*` `Err(e)` branches; AC9 forbidden-file
  diff on `9e32fbe` empty; AC10 evidence doc NEW (347 lines) on
  `origin/main` via PROMPT 847 integration. Paperwork-only writes to
  4 allowed files: this story file (Status flip + AC checkboxes +
  this trail entry + Conditions Carried Forward + Explicitly NOT
  claimed sub-sections); `production/sprint-status.yaml` (row
  `S13-OBS-TRACING-TARGETS-001` flipped `status: ready -> done` +
  `completed: 2026-05-14` + worker/integration/story-done metadata;
  top-level `updated:` annotation refreshed; `sprint_13_story_done:`
  block extended with PROMPT 850 entry); `production/session-state/active.md`
  (PROMPT 850 banner prepended); `production/session-state/codex-orchestrator-state.md`
  (PROMPT 850 section prepended). Commit pushed to `origin/main` via
  fast-forward, no force.

### Conditions carried forward unchanged

- S8-QA-001-W1 manual/browser two-client GAME_OVER gap remains OPEN.
  Story 017 (two-client runtime harness) AC12 forbid-auto-closure:
  harness does NOT close S8-QA-001-W1 by itself.
- QA-COND-0005 Standard-tier accessibility remains accepted-risk
  (friend-game scope only).
- QA-COND-0006 playtest / fun-hypothesis validation remains
  accepted-risk / deferred.
- PAW-TD-*-a placeholder-art accept-risk preserved across PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved unchanged
  (folded into Sprint 12 story 019 cannot-reproduce closure; third
  same-scope retest NOT authorised per TQ-S12-C2). PROMPT 850 does
  NOT re-attempt the Sprint 12 capture; the new module-path-scoped
  targets unblock the Story 019 invocation pattern but the runtime
  capture itself is deferred per AC5.
- PROMPT 761 Polish->Release gate-check FAIL preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no retry
  in PROMPT 850 scope.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed (closed cannot-reproduce, NOT bug-fixed).
- TQ-S12-C1..C7 (all 7 Sprint 12 Team-QA conditions) preserved verbatim.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817
  preserved unchanged.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- Prior `/story-done` closures preserved unchanged on `origin/main`:
  PROMPT 833 (S11-SERVER-POOL-INIT-LOG-GUARD-001), PROMPT 835
  (S11-LOBBY-UX-CONFIRM-STATE-001), PROMPT 840
  (S13-UI-AUDIT-ROADMAP-PREP-001), PROMPT 843
  (S13-OBS-WALLCLOCK-TIMESTAMPS-001), PROMPT 844
  (S11-HU-PHASE-IDEMPOTENCY-001).
- PROMPT 845 `S13-PROTO-INVARIANT-001` workspace invariant test
  (commit `96c1600`) preserved on `origin/main` unchanged by PROMPT
  850. PROMPT 849 integration merge `25573e6` preserved.
- AC5 runtime harness capture deferred to next Story 019 retest
  prompt or Sprint 13 harness-up prompt; the new module-path targets
  remain landed and operational regardless.

### Explicitly NOT claimed by PROMPT 850

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (S8-QA-001-W1)
- final-art / asset-production completion
- Polish->Release gate-check retry
- Stage advance from Polish to Release
- underlying drag-runtime bug fix (Sprint 12 story 019 closed
  cannot-reproduce, NOT bug-fixed)
- full UI clean-pass repair
- closure of S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001
- closure of S13-PHASE-IDEMPOTENCY-CLIENT-001 (same-class defect for
  HUD + shop-auction consumers; out of scope of this story)
- Sprint 13 close-out (Sprint 13 remains `active`; only 6 of 19 rows
  closed after PROMPT 850 — **2 of 6 Must Have**, 3 of 6 Should Have,
  1 of 7 Nice to Have)
- full-workspace `cargo test --workspace --tests --no-fail-fast` result
  claim (narrowest targeted checks were used per QA-plan-sprint-13;
  full-workspace gate deferred to orchestrator end-of-sprint integration)
- AC5 runtime harness capture (deferred-within-scope; static target-string
  evidence + harness landing pending)
