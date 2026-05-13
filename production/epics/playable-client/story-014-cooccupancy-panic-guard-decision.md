# Story 014: Co-occupancy Panic-Guard -- Binary Design Decision

> **Epic**: Playable Client
> **Story ID**: S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001
> **Status**: In Progress -- Sprint 12 Must Have (Cluster B4); Path B decision recorded by PROMPT 800 (decision-recording commit precedes code-change commit)
> **Layer**: Tech Debt / Production Invariant -- Binary Design Decision
> **Type**: Decision-first (test-only by default; production-code change only
> under explicit Path A with a written design write-up)
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 12 (draft per PROMPT 793 at `origin/main@8a8451e`; NOT yet activated)
> **Authored**: 2026-05-14 by PROMPT 795 (producer + qa-lead, worktree `work/sprint-12-must-story-authoring`)
> **Authoring source-of-truth**: `origin/main@f72cc60` (PROMPT 793 Sprint 12 draft plan + PROMPT 794 story-019 slug correction).

---

## Status / No-Claim Banner

This story is authored as a Sprint 12 draft Must Have. Sprint 12 is **NOT
activated**; activation happens via `/sprint-plan sprint-12` in a separate
prompt. PROMPT 795 (this authoring run) does NOT:

- Activate Sprint 12.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md`.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any session-state file.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify the ignored test
  (`tests/unit/board_rendering/status_icons_test.rs:167`).
- Modify the production `co_occupancy_offset` function.
- Make a decision between Path A and Path B (that is the
  implementation prompt's responsibility, gated on a written design
  write-up before any code change lands).
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 disposition (`closed-with-conditions`) and Sprint 11 disposition
(`closed-with-conditions` per PROMPT 792) remain unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**The `#[should_panic]` invariant on `co_occupancy_offset(2, ..)` may NOT
be silently deleted under any disposition pathway.** A binary decision is
required; both paths require a written rationale captured in this story
file before any code change lands.

---

## Context

Sprint 11 D-5 triage evidence
(`production/qa/evidence/sprint-11-ignored-d5-triage.md`, Cluster B4, row 86)
retained the test
`test_cooccupancy_index_two_panics_with_offending_index` at
`tests/unit/board_rendering/status_icons_test.rs:167` with the PROMPT 750 D-5
owner comment:

> `#[ignore = "PROMPT 750 D-5: production co_occupancy_offset no longer
> panics on index 2 -- needs design decision: restore panic guard or update
> test to assert non-panic behavior"]`

The test is structured as:

```rust
#[ignore = "PROMPT 750 D-5: ..."]
#[test]
#[should_panic(expected = "unit_index=2")]
fn test_cooccupancy_index_two_panics_with_offending_index() {
    let _ = co_occupancy_offset(2, 8.0);
}
```

The `#[should_panic(expected = "unit_index=2")]` invariant captures a
production assertion that `co_occupancy_offset` must panic with a
specific message when called with an out-of-range unit index. The
production code (`client/src/ui/board/` -- exact module verified at
implementation time) no longer panics on index 2; the panic-guard was
either intentionally removed (silently, without an accompanying design
write-up) or accidentally regressed.

This is a **pure decision-first story**. The decision space is binary
and gate-locked:

- **Path A -- Restore panic-guard in production**: re-add the
  `assert!(unit_index < N, "unit_index={unit_index}")` (or equivalent)
  guard in `co_occupancy_offset`; re-arm the test as
  `#[should_panic(expected = "unit_index=2")]` without the `#[ignore]`
  marker. **Path A requires an explicit production-design write-up
  before code change**: the write-up captures why the guard is
  defensive-mandatory (e.g., upstream caller invariant; debug-build
  fail-fast; protection against silent visual offset overflow).
- **Path B -- Test rewritten to assert non-panic behaviour**: rewrite
  the test to assert the current production behaviour of
  `co_occupancy_offset(2, ..)` (whatever it now returns); remove the
  `#[should_panic]` invariant; record the production rationale for
  *why* the panic-guard is no longer needed (e.g., caller bounds-check
  is now performed upstream; the function is now total over the input
  domain). The production rationale must be captured in the story
  file before the test rewrite lands.

**Either path is acceptable**; neither path may silently delete the
`#[should_panic]` invariant. The producer-recorded decision lives in
this story file (the implementation prompt updates the "Binary Design
Decision" section before any code change is staged).

**Primary sources**:

- `production/qa/evidence/sprint-11-ignored-d5-triage.md` (Cluster B4, row 86)
- `production/sprints/sprint-12.md` (Sprint 12 draft Must Have row
  `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`, line 126)
- `tests/unit/board_rendering/status_icons_test.rs:167` (the test in
  question, with PROMPT 750 D-5 owner comment and `#[should_panic]`
  invariant preserved on `origin/main@f72cc60`)
- Production source: `co_occupancy_offset` in
  `client/src/ui/board/` (exact module verified at
  `/story-readiness` time)

**GDD, UX, and TR trace**:

- `design/gdd/board-rendering.md` -- TR-BR-006 (status indicators) and
  TR-BR-007 (co-occupancy visual offsets) cover the production
  behaviour. The panic-guard is a defensive assertion on the unit
  index domain; not a TR-coverage requirement itself, but a
  development-time fail-fast guard.
- No new TR is added by this story. Either path leaves the TR
  coverage unchanged.

**ADR Governing Implementation**:

- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
  -- co-occupancy visual offsets are presentation-side; no
  authoritative state is changed under either disposition.
- No protocol or networking ADR is touched.

**Engine**: Bevy 0.18 (Rust) | **Risk**: LOW
(production-code change is at most one function body under Path A; test
rewrite is at most one function body under Path B. Both are local and
reversible.)

**Engine Notes**: Bevy 0.18 has no runtime difference between
`assert!`-based panic-guards and `Result`-returning bounds checks
beyond hot-path cost. `co_occupancy_offset` is presentation-side and
called at status-icon spawn time (not per-frame in the hot path);
panic-guard cost is negligible. The decision is purely about API
contract shape, not performance.

**Mandatory skills**:
- `liv-bevy-018` -- any read/review/edit of Bevy `.rs` code touched.

**Control Manifest Rules (2026-05-05)**:
- Required: Either path documents the decision in this story file
  before any code change is staged.
- Required: Path A includes a written production-design write-up
  before code change.
- Required: Path B documents *why* the panic-guard is no longer
  needed before the test rewrite lands.
- Forbidden: Silently deleting the `#[should_panic]` invariant
  without a written rationale.

---

## Story Classification

**Story type**: Pure binary design decision (decision-first). Both
paths land code changes after the decision is recorded; the story is
NOT evidence-only.

This is **NOT** a:

- Repair story (no broken runtime is asserted; the decision space is
  binary).
- Fixture-cleanup story (the test fixture is correct; the production
  code or test assertion must change, depending on the decision).
- Evidence-only story (executable code change lands after decision).

---

## Binary Design Decision (to be recorded by the implementation prompt)

The implementation prompt MUST record exactly one of the following
before any code change is staged. **Both paths require a written
rationale in this story file.**

- [ ] **Path A -- Restore panic-guard in production**. **NOT CHOSEN.**
  - Production rationale (mandatory, before code change):
    _N/A -- Path A explicitly NOT chosen. Restoring the panic would
    crash the production renderer when 3+ allied units stack on the
    same board cell, which is a recoverable visual condition rather
    than a programmer-error invariant. The presentation-layer
    contract (ADR-021) prefers non-fatal degradation over hard
    panics in the snapshot-rendering hot path._
  - Production change shape:
    _N/A -- no production code change under Path A. (Reference
    shape, for the record: `client/src/presentation/board_rendering.rs`
    -- re-add `assert!(unit_index < 2, "unit_index={unit_index}")` at
    the entry of `co_occupancy_offset`; re-arm test with
    `#[should_panic(expected = "unit_index=2")]` and remove
    `#[ignore]`.)_

- [x] **Path B -- Test rewritten to assert non-panic behaviour**. **CHOSEN.**
  - Production rationale (mandatory, before test rewrite -- recorded
    at PROMPT 800 decision-recording commit, before the test rewrite
    commit):
    - **(a) Where the upstream bounds-check now lives.** The sole
      caller of `co_occupancy_offset` is
      `snapshot_co_occupancy_offsets` at
      `client/src/presentation/board_rendering.rs:1888-1927`. That
      caller already enforces an upstream invariant:
      `assert!(index <= u8::MAX as usize, "F3 co-occupancy:
      unit_index={} > 255 - invalid co-occupancy state", index)` at
      `client/src/presentation/board_rendering.rs:1917-1921`. This
      upstream assertion bounds the input domain of
      `co_occupancy_offset` to fit in `u8` (i.e. the parameter
      type's own invariant is preserved). Beyond that, the function
      is intentionally defined as **total over its `u8` input
      domain**: every `u8` value maps to a defined `f32` offset.
    - **(b) What `co_occupancy_offset(2, ..)` now returns.** The
      function `client/src/presentation/board_rendering.rs:1929-1938`
      reads:
      ```rust
      pub fn co_occupancy_offset(unit_index: u8, side_offset: f32) -> f32 {
          if unit_index > 1 {
              warn!(
                  "co_occupancy_offset: unit_index {} out of range, clamping to 1",
                  unit_index
              );
          }
          let index = unit_index.min(1);
          (f32::from(index) - 0.5) * side_offset
      }
      ```
      For `unit_index = 2` (and any `unit_index >= 2`) the function
      emits a `warn!` diagnostic and clamps to `1`, returning
      `(1.0 - 0.5) * side_offset = 0.5 * side_offset`. For
      `side_offset = 8.0` (the test's call site), the return value
      is exactly `4.0`.
    - **(c) Why silent overflow is no longer a risk.** Two layers
      defend against silent visual mis-alignment:
      1. **Diagnostic visibility.** The `warn!` macro is wired
         through Bevy's `bevy::log` subscriber (env-filter
         `client=info` and above in dev / `server::game=debug` in
         test harnesses). When 3+ allied units co-occupy a cell,
         the warning surfaces in the log stream and is observable
         in CI logs and developer consoles. This is the same
         visibility profile that the previous panic provided in
         debug builds, minus the crash.
      2. **Clamp semantics.** The clamp deliberately overlaps the
         third+ unit with the second-unit visual offset. This is
         the **safe visual fallback** for an inherently 2-slot
         layout: an extra unit at the same cell renders on top of
         an existing slot rather than at an undefined offset
         outside the cell. The cell-layout invariant in
         `design/gdd/board-rendering.md` (TR-BR-007 co-occupancy
         visual offsets) does NOT specify behaviour for >2
         co-occupants, so overlap is the correct degradation.
    - **(d) Why this is the right shape for the presentation
      layer.** ADR-021 (Presentation Layer Architecture)
      authoritative state lives on the server snapshot; the
      presentation layer renders a derived view. A panic in the
      snapshot-rendering path would crash the client on a *visual*
      anomaly that does not affect game state. The current
      warn+clamp shape is the ADR-021-aligned degradation: non-fatal
      visual rendering with a diagnostic trail.
    - **(e) Historical disposition.** The panic-guard was
      intentionally replaced with warn+clamp in commit
      `ac9305b07764038611f4a62e79c018e072d41002` on 2026-05-08
      (`fix(board_rendering): observer refactor + Pointer<Click>/Press
      to On<> + co_occupancy clamp + BoardRenderingConfig threading +
      ADR-021 PresentationSet::MessageDrain`). That commit's
      production change shape (warn+clamp) is the established
      disposition; PROMPT 750 D-5 ignored the test pending the
      written design write-up; PROMPT 800 records the write-up here
      and updates the test to match the disposition.
  - Test rewrite shape:
    `tests/unit/board_rendering/status_icons_test.rs` -- rewrite
    `test_cooccupancy_index_two_panics_with_offending_index` to
    `test_cooccupancy_index_two_clamps_to_second_slot_offset` (or
    equivalent canonical name); remove `#[should_panic(expected =
    "unit_index=2")]`; remove `#[ignore = "PROMPT 750 D-5: ..."]`.
    Assert that `co_occupancy_offset(2, 8.0)` returns `4.0`
    (i.e. `0.5 * side_offset`), matching the production clamp
    behaviour. Also assert that `co_occupancy_offset(0, 8.0)` and
    `co_occupancy_offset(1, 8.0)` return `-4.0` and `4.0`
    respectively, to lock the 2-slot canonical layout alongside the
    >=2 clamp.

The decision is binary; both paths are acceptable IFF the rationale is
captured. The unchecked path is explicitly marked NOT chosen.

**The `#[should_panic]` invariant may NOT be silently deleted without
this written rationale.** This is the hard constraint of Cluster B4
disposition. Under Path B (chosen), the rationale above is committed
*before* the test rewrite lands.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-12.md:126` Sprint 12 draft Must Have
row. ACs below are draft and become binding at Sprint 12 activation.)

- [ ] **AC1 -- Binary decision recorded in this story file before
      code change**: GIVEN this story file, WHEN the "Binary Design
      Decision" section is read at the implementation commit, THEN
      exactly one of {Path A, Path B} is checked, the unchecked path
      is explicitly marked NOT chosen, and a written rationale is
      present under the chosen path. **The decision recording commit
      MUST precede any code change commit in the story's commit
      trail.** *Evidence*: `git log --oneline` of the story's commit
      trail shows the decision-recording commit before the code-
      change commit.

- [ ] **AC2 -- Production-design write-up under Path A**: GIVEN that
      Path A is the chosen path, WHEN the evidence document and/or
      this story file is read, THEN a production-design write-up is
      present that captures: (a) upstream caller invariant; (b) why
      the guard is defensive-mandatory; (c) why this is not a hot-
      path cost concern. The write-up is at most ~one page; it may
      live in this story file's rationale section or in the evidence
      document. The write-up is committed *before* the production
      code change.

- [ ] **AC3 -- Production rationale under Path B**: GIVEN that Path B
      is the chosen path, WHEN the evidence document and/or this
      story file is read, THEN a production rationale is present
      that captures: (a) where the upstream bounds-check now lives;
      (b) what `co_occupancy_offset(2, ..)` now returns; (c) why
      silent overflow is no longer a risk. The rationale is at most
      ~one page; it is committed *before* the test rewrite lands.

- [ ] **AC4 -- Test un-`#[ignore]`d and passes under chosen path**:
      GIVEN the chosen path's implementation commit, WHEN
      `cargo test -p client --test status_icons` (or the equivalent
      `cargo test` invocation) is run, THEN whichever test variant
      the chosen path produces (Path A: re-armed
      `test_cooccupancy_index_two_panics_with_offending_index` with
      `#[should_panic]`; Path B: rewritten
      `test_cooccupancy_index_two_returns_expected_offset` with no
      `#[should_panic]` and no `#[ignore]`) passes locally. Pre/post
      pass count recorded in the evidence document.

- [ ] **AC5 -- Workspace ignored count drops by 1**: GIVEN Sprint 11
      close-out baseline of 5 retained Cluster B `#[ignore]` tests on
      `origin/main`, WHEN
      `cargo test --workspace --tests --no-fail-fast` is run at the
      implementation commit, THEN the workspace ignored count drops
      by 1 (relative to the baseline) and no new undocumented
      `#[ignore]` marker is introduced.

- [ ] **AC6 -- Original PROMPT 750 D-5 owner comment removed only
      after the test passes**: GIVEN the implementation commit, WHEN
      `tests/unit/board_rendering/status_icons_test.rs` is read,
      THEN no PROMPT 750 D-5 owner comment for this test remains.

- [ ] **AC7 -- `#[should_panic]` invariant NOT silently deleted**:
      GIVEN the diff of the implementation commit set, WHEN the
      `#[should_panic]` attribute removal is reviewed (Path B only;
      Path A keeps the attribute), THEN the rationale in AC3 is
      committed *before* the attribute removal commit. The diff
      review verifies that no other `#[should_panic]` attribute was
      silently dropped under either disposition. *Evidence*:
      `git log --oneline -- tests/unit/board_rendering/status_icons_test.rs`
      shows the rationale commit before the test rewrite commit.

- [ ] **AC8 -- Sprint 12 disposition preserved**: GIVEN the
      implementation commit, WHEN `production/sprint-status.yaml`,
      `production/sprints/sprint-12.md`, and `production/stage.txt`
      are diffed, THEN none of them are modified under this story.
      Sprint 12 activation disposition is preserved. Stage remains
      `Polish`. Sprint 11 disposition (`closed-with-conditions`) is
      unchanged.

- [ ] **AC9 -- Evidence document slot reserved**: GIVEN this story
      file, WHEN the evidence-doc path is checked, THEN a slot is
      reserved at
      `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md`
      for population by the implementation prompt. Authoring of the
      evidence file itself is deferred to the implementation prompt.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. `/sprint-plan sprint-12` activates Sprint 12 (separate prompt).
2. This story passes `/story-readiness` (separate prompt).
3. Sprint 12 `/qa-plan sprint` is authored (separate prompt).
4. `/dev-story story-014-cooccupancy-panic-guard-decision.md` is
   dispatched (separate prompt).

Expected implementation flow:

1. **Wave 1 -- Decision + rationale**: the implementation prompt
   reads the production `co_occupancy_offset` function, identifies
   the post-guard-removal behaviour, surveys upstream callers for
   bounds-check responsibility, and records the binary decision plus
   rationale in this story file's "Binary Design Decision" section.
   **This commit precedes any code change.**
2. **Wave 2 -- Code change**: applies the chosen path's code change.
   Path A: re-add panic-guard in production + re-arm test. Path B:
   rewrite test + remove `#[should_panic]`.
3. **Wave 3 -- Validation**: run `cargo test -p client --no-fail-fast`
   and `cargo test --workspace --tests --no-fail-fast`; capture
   pre/post pass + ignored counts in the evidence document.
4. **Wave 4 -- Evidence doc**: populate
   `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md`
   with the decision rationale (transcribed), code-change diff
   summary, pre/post counts, no-claim restatement, and cross-link
   to Cluster B4 row 86 in the triage doc.

---

## Performance Budget

N/A -- `co_occupancy_offset` is called at status-icon spawn time, not
per-frame in the hot path. Panic-guard cost (under Path A) or upstream
bounds-check cost (under Path B) is negligible.

---

## QA Test Cases

(Draft -- becomes binding at Sprint 12 activation. Sprint 12 QA plan
authored via `/qa-plan sprint` will pull from this set.)

- **Decision-first ordering audit**
  - Given: the story's commit trail on `main`.
  - When: `git log --oneline -- production/epics/playable-client/story-014-cooccupancy-panic-guard-decision.md tests/unit/board_rendering/status_icons_test.rs client/src/ui/board/*.rs`
    is run.
  - Then: the decision-recording commit precedes any code-change
    commit.

- **Test passes under chosen path**
  - Given: implementation commit set on `main` for this story.
  - When: `cargo test -p client --test status_icons` (or equivalent)
    is run.
  - Then: the chosen path's test variant passes; no `#[ignore]`
    tagging remains on the row.

- **Workspace ignored-count regression check**
  - Given: Sprint 11 close-out baseline (1129 passing / 5 ignored on
    `origin/main@8a8451e`).
  - When: `cargo test --workspace --tests --no-fail-fast` is run at
    the implementation commit.
  - Then: workspace ignored count is at most `5 - 1 = 4` and no new
    undocumented `#[ignore]` marker is introduced.

- **No-silent-deletion audit**
  - Given: the diff of the implementation commit set.
  - When: `git log -p -- tests/unit/board_rendering/status_icons_test.rs`
    is reviewed.
  - Then: under Path B, the `#[should_panic]` attribute removal is
    preceded by a rationale commit. Under Path A, the
    `#[should_panic]` attribute is retained.

- **Sprint 12 disposition preservation audit**
  - Given: the implementation commit.
  - When: `production/sprint-status.yaml`,
    `production/sprints/sprint-12.md`, and `production/stage.txt` are
    diffed.
  - Then: none of them are modified under this story id.

---

## Test Evidence

**Story Type**: Decision-first (decision-recording commit precedes
code-change commit; pre/post test counts capture the executable
artefact).

**Evidence path**: `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Path A or Path B decision + rationale (transcribed from this story
  file's "Binary Design Decision" section).
- Production-design write-up (under Path A) or production rationale
  (under Path B), ~one page.
- Code-change diff summary (Path A: production guard re-added; Path
  B: test rewritten).
- Pre/post `cargo test -p client` pass + ignored counts.
- Pre/post `cargo test --workspace --tests --no-fail-fast` pass +
  ignored counts.
- No-silent-deletion audit: `git log --oneline` showing
  rationale-commit-before-code-change-commit ordering.
- No-claim restatement (verbatim from this story file's "Status /
  No-Claim Banner" section).
- Cross-link back to this story file and to Cluster B4 row 86 in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`.

**Required verification commands** (for the implementation prompt):

- `cargo test -p client --no-fail-fast`
- `cargo test --workspace --tests --no-fail-fast`
- `git log --oneline -- production/epics/playable-client/story-014-cooccupancy-panic-guard-decision.md tests/unit/board_rendering/status_icons_test.rs client/src/ui/board/`
- `git diff <pre-impl-sha>..<impl-sha> -- 'client/src/ui/board/**' 'tests/unit/board_rendering/**'`
- `git diff --check` and `git diff --cached --check` before commit

**Status**: [ ] Captured and locked

---

## Owner / Classification

- **Owner**: board-rendering owner (production `co_occupancy_offset`)
  + qa-lead (decision arbitration) -- per Cluster B4 row 86 in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`.
- **Estimated days**: 0.50 (per Sprint 12 draft row).
- **Story classification**: pure binary design decision; code change
  follows decision-recording commit.

## Dependencies

- **Depends on**: Sprint 12 activation via `/sprint-plan sprint-12`
  (separate prompt). This story remains `Draft` until then.
- **Depends on**: Sprint 11 D-5 triage doc
  (`production/qa/evidence/sprint-11-ignored-d5-triage.md`, Cluster B4,
  row 86) for owner / disposition / decision-gate language.
- **Depends on**: Sprint 12 QA plan authored via `/qa-plan sprint`
  (separate prompt) before `/dev-story` runs.
- **Coordinated with**: `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`
  (Story 012 -- Cluster B2 board-rendering-tagged decision; both
  stories touch the board-rendering review surface, but disjoint
  files).
- **Coordinated with**: `S11-TD-FIXTURE-D-RESIDUALS-001`
  (Story 015 -- Cluster B1+B5 umbrella; sibling fixture residuals
  from Wave 12 D-3 / D-4 / D-5 sweeps; no shared file scope with
  this story).
- **Not coordinated with**: `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`
  (Story 013 -- different module; no shared file scope).
- **Not coordinated with**: `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  (story 019 in hand-ui epic -- different module; no shared file
  scope).

## Readiness Notes

**Implementation readiness verdict**: Draft -- substantive work has not
started. The live downstream gates are Sprint 12 activation, Sprint 12
QA plan authorship, and `/story-readiness` PASS on this file.

Pre-conditions for `/story-readiness` PASS:

- Sprint 12 is activated (`sprint:` field in
  `production/sprint-status.yaml` bumped + active row written) --
  **Pending separate `/sprint-plan sprint-12` prompt.**
- Sprint 12 QA plan exists at `production/qa/qa-plan-sprint-12.md` --
  **Pending separate `/qa-plan sprint` prompt.**
- This story file is referenced from Sprint 12's active row in
  `production/sprint-status.yaml` after activation -- **Pending
  separate `/sprint-plan sprint-12` prompt.**

Open questions to resolve at `/story-readiness` time:

- Which upstream callers of `co_occupancy_offset` exist in
  `client/src/ui/board/`? (Diagnosis at implementation time
  enumerates them and informs the bounds-check responsibility
  decision.)
- Is there an existing post-D-1 commit that intentionally removed
  the panic-guard? `git log -p -- client/src/ui/board/*.rs` review
  surfaces the disposition history.
- Has the `co_occupancy_offset` API contract been documented anywhere
  beyond the test's `#[should_panic]` invariant? (If yes, the
  documentation site is updated under the chosen path.)

---

## Files Anticipated To Be Modified (planning estimate, NOT binding)

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/board/<module>.rs` (exact module verified at implementation; Path A only) | Path A: re-add `assert!(unit_index < N, "unit_index={unit_index}")` (or equivalent) at the entry of `co_occupancy_offset`. |
| `tests/unit/board_rendering/status_icons_test.rs` | Path A: un-`#[ignore]` + keep `#[should_panic]`. Path B: rewrite test body + remove `#[should_panic]` + remove `#[ignore]`. |
| This story file (decision-recording commit) | "Binary Design Decision" section updated with chosen path + rationale. **Commit precedes code change.** |
| `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md` (NEW) | evidence document per AC9 |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Implementation prompt lands the code change before recording the decision | Medium | High | AC1 hard constraint: decision-recording commit must precede code-change commit. Verified by `git log --oneline`. |
| Path B rationale is hand-waved ("we don't need the guard") without identifying the upstream bounds-check site | Medium | Medium | AC3 requires the rationale to identify (a) upstream bounds-check site, (b) current return value, (c) overflow-safety argument. |
| `#[should_panic]` invariant is silently deleted | Low | High | AC7 hard constraint + no-silent-deletion audit (`git log -p` review). |
| Diagnosis surfaces a deeper API contract divergence (e.g., `co_occupancy_offset` is called from a different module than the test expects) | Low | Medium | Diagnosis wave records the upstream caller layout; if a deeper divergence surfaces, the story scope expands narrowly OR a separate follow-on story is authored. |
| Sprint 12 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate; this story stays `Draft` until activation. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator emitting the implementation
prompt, not for the worker:

- `production/sprint-status.yaml` `sprint:` field reads `sprint 12` and
  this story is referenced from the active row, OR the row is held with
  a written blocker.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-12.md` status block reads `active` (after
  separate `/sprint-plan sprint-12` activation prompt).
- The PROMPT 761 Polish->Release gate-check FAIL evidence at
  `production/gate-checks/gate-polish-release-2026-05-12.md` is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 795 -- Story file authored as a Sprint 12 draft
  Must Have for Cluster B4. Sprint 12 is `draft` (PROMPT 793) and not
  yet activated -- this story is **not yet activated** into the
  Sprint 12 active scope. Activation is a separate prompt
  (`/sprint-plan sprint-12`). No code changes, no smoke / gate / QA /
  `/dev-story` / `/story-done` / `/story-readiness` / `/qa-plan` run.
  Source-of-truth at authoring: `origin/main@f72cc60`.
- 2026-05-14 -- PROMPT 800 -- `/dev-story` Wave 1 (decision +
  rationale). Path B chosen and rationale recorded above before any
  code change. This commit lands the decision + rationale only;
  the test-rewrite commit follows in Wave 2. Source-of-truth at
  decision time: `origin/main@b5eef0d` (PROMPT 799 Sprint 12 QA plan
  authoring commit). Worker branch `work/s11-cooccupancy-panic-guard-decision`.
  No production code under `client/`, `server/`, `shared/` is
  modified by this commit. `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`, and
  all `production/session-state/*` files are NOT modified.
