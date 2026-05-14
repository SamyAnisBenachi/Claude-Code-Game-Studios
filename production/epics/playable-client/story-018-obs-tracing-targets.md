# Story 018: S13-OBS-TRACING-TARGETS-001 -- Module-Scoped Tracing Targets for Diagnostic Capture

> **Epic**: Playable Client
> **Story ID**: S13-OBS-TRACING-TARGETS-001
> **Status**: Draft -- Sprint 13 candidate; NOT activated; Sprint 12 is the
> active sprint
> **Layer**: Observability / Cross-Cutting
> **Type**: Integration -- targeted edits across emission sites + verification
> **Sprint**: Sprint 13 candidate (per PROMPT 803 §6 line 145; NOT activated)
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

- [ ] **AC1 -- `client::ui::hand` target landed at every relevant
  emission site**: GIVEN the diff under `client/src/ui/hand/`, WHEN
  every `tracing::*!()` macro invocation in the module is
  inspected, THEN each invocation that is part of the S1-S5
  instrumentation (per PROMPT 706 / 709 / `7e0c663`) carries
  `target: "client::ui::hand"` (or an explicitly-narrower target
  like `target: "client::ui::hand::drag"` with rationale).

- [ ] **AC2 -- `client::presentation::board_rendering` target
  landed**: same for `client/src/presentation/board_rendering.rs`.

- [ ] **AC3 -- `client::card_animations::input_gating` target
  landed**: same for `client/src/card_animations/input_gating.rs`.

- [ ] **AC4 -- `server::game` target landed at all relevant
  server feature emission sites**: GIVEN the diff under
  `server/src/feature/`, WHEN every `tracing::*!()` invocation
  relevant to gameplay state changes (auction resolution,
  acquisition tick, objective destroyed, combat resolution) is
  inspected, THEN each carries `target: "server::game"` (or a
  narrower target like `target: "server::game::auction"` with
  rationale). At minimum, the
  `server/src/network/rsm_dispatch.rs` `S2C*` broadcast emission
  sites (per PROMPT 803 §4 Lane A) are tagged.

- [ ] **AC5 -- Verification harness pass**: GIVEN the
  implementation commit, WHEN the Story 019 invocation
  `RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=debug,server::game=debug`
  is run against either (a) a smoke test that exercises the
  drag-runtime route or (b) the
  `S13-TWO-CLIENT-RUNTIME-HARNESS-001` harness if available,
  THEN per-target output is non-empty for each named target. The
  evidence doc records sample lines from each target.

- [ ] **AC6 -- Behaviour unchanged**: GIVEN `cargo test
  --workspace --tests --no-fail-fast` at the implementation
  commit, WHEN compared to the pre-implementation baseline,
  THEN no test regressions are observed (same pass/fail/ignored
  counts modulo Sprint 12 close-out deltas).

- [ ] **AC7 -- No new emission sites added**: GIVEN the
  implementation diff, WHEN searched for new `tracing::*!()`
  macro invocations, THEN zero new sites are introduced (the
  diff is purely "add `target:` arg to existing sites" + reformat
  surrounding lines as needed).

- [ ] **AC8 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed for any
  client-side mutation of authoritative state outside the
  shared phase sink, snapshot drainers, and S2C consumers,
  THEN no such mutation is present. ADR-002 binding.
  *Evidence*: text search for "no optimistic" in the evidence
  document.

- [ ] **AC9 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`,
  and `production/qa/qa-plan-sprint-12.md` are diffed, THEN
  none of them are modified under this story.

- [ ] **AC10 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md`
  (NEW). Records pre/post grep counts of `target: "..."`
  occurrences, AC5 harness output sample lines, no-claim
  restatement, cross-link to PROMPT 803 §3 DC-11.

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

## Authoring Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for Module-Scoped Tracing Targets per PROMPT 803 §3
  DC-11 / §5 Must row 7. Sprint 12 is `active` (PROMPT 798) and is
  not modified by this authoring run. No code changes, no smoke /
  gate / QA / `/dev-story` / `/story-done` / `/story-readiness` /
  `/qa-plan` run. Source-of-truth at authoring: `origin/main@b5eef0d`.
  Worker branch: `work/s13-runtime-hardening-story-authoring`.
  Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.
