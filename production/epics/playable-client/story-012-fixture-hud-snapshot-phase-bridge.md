# Story 012: HUD Snapshot Phase Bridge -- Fixture Cleanup + Design Decision

> **Epic**: Playable Client
> **Story ID**: S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001
> **Status**: Done -- closed by PROMPT 814 (`/story-done` paperwork) on
> `origin/main@a3c624e`; integration commit `c1eef10` (PROMPT 806 worker;
> integrated to `main` per PROMPT 809). Path B (relocate assertion to
> dedicated HUD test) chosen and recorded under "Design Decision" below.
> **Layer**: Tech Debt / Test Fixtures (potential design decision -- relocate vs expand)
> **Type**: Integration (test-only by default; production-code path gated on design decision)
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
- Modify the ignored test (`tests/integration/board_rendering/snapshot_spawn_test.rs:39`).
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

**No optimistic client-side authority is introduced or proposed by this story
or by any disposition pathway recorded in "Acceptance Criteria"**. ADR-002
and ADR-009 remain binding for any production code change that may land
under the "expand-fixture-to-include-HudPlugin" path.

---

## Context

Sprint 11 D-5 triage evidence
(`production/qa/evidence/sprint-11-ignored-d5-triage.md`, Cluster B2, row 84)
retained the test
`test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`
at `tests/integration/board_rendering/snapshot_spawn_test.rs:39` with the
PROMPT 750 D-5 owner comment:

> `#[ignore = "PROMPT 750 D-5: assertion expects HudPlugin to bridge
> snapshot.phase -> CurrentClientPhase, but HudPlugin is not in this fixture;
> either expand fixture to include HudPlugin or relocate the assertion to a
> hud test (needs owner decision)"]`

The fixture (`app_in_session()` within `snapshot_spawn_test.rs`) builds a
`BoardRenderingPlugin`-only App. The assertion under test expects the
HUD-side bridge from `S2CGameSnapshot.phase` to the
`Res<CurrentClientPhase>` resource to run, but `HudPlugin` is not registered
in the fixture, so the bridge never fires. The remaining
board-rendering-only assertions in the same test (stale visuals cleared;
units + objectives spawned from the snapshot) are valid against the current
fixture surface; only the phase-bridge assertion is misplaced.

This story is **a design decision plus fixture-cleanup**, not a production
bug. The decision is binary and explicit (see "Design Decision" below).
Neither disposition introduces optimistic client-side authority; the
`HudPlugin` bridge already follows ADR-021 (shared phase sink reads
`S2CPhaseChanged` and snapshot-derived phase through the same single
drainer).

**Primary sources**:

- `production/qa/evidence/sprint-11-ignored-d5-triage.md` (Cluster B2, row 84)
- `production/sprints/sprint-12.md` (Sprint 12 draft Must Have row
  `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`, line 124)
- `tests/integration/board_rendering/snapshot_spawn_test.rs:39` (the test in
  question, with PROMPT 750 D-5 owner comment preserved on
  `origin/main@f72cc60`)
- Pattern reference: `production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`
  (S11-TD-FIXTURE-HAND-UI-ONENTER-001; precedent for fixture-only repair
  with reusable helper)

**GDD, UX, and TR trace**:

- No GDD requirement governs this story directly. The board-rendering
  spawn-from-snapshot path is governed by `design/gdd/board-rendering.md`
  TR-BR-003 / TR-BR-005; this story does NOT modify the board-rendering
  production code path. The HUD phase bridge is governed by
  `design/gdd/hud.md` (TR-HUD-001 phase readout) and ADR-021 (single
  shared phase sink); this story does NOT propose changes to either.
- The misplaced assertion is a test-layer correctness issue, not a
  TR-coverage gap. The HUD phase-bridge invariant is already covered by
  dedicated HUD integration tests (verify at `/story-readiness` time;
  see "Open Questions" below).

**ADR Governing Implementation**:

- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
  -- single shared phase sink drains `S2CPhaseChanged`; `HudPlugin`
  reads the same `Res<CurrentClientPhase>` projected by the shared sink.
  Binding for either disposition.
- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
  -- no optimistic client-side authority. Binding for either disposition.
- [ADR-009: RSM Phase State](../../../docs/architecture/adr-009-rsm-phase-state.md)
  -- phase transitions are server-authoritative; client reads only.

**Engine**: Bevy 0.18 (Rust) | **Risk**: LOW
(test-fixture / assertion-relocation; production code change only under
the explicit "expand fixture" path with a written rationale)

**Engine Notes**: Bevy 0.18 plugin composition is additive -- expanding
the fixture to include `HudPlugin` requires verifying that `HudPlugin`'s
sub-plugin dependencies (typically `PresentationPlugin` set, snapshot
sink, asset wiring) are present in the fixture or are mocked. If they
are not, the expansion would cascade additional fixture work that is
out of scope for this story. The "relocate" path avoids this cascade
entirely by moving the phase-bridge assertion to a HUD-side test where
the relevant plugin set is already wired.

**Control Manifest Rules (2026-05-05)**: Test-fixture changes only by
default. If the producer-recorded decision selects "expand fixture to
include `HudPlugin`", the change is still test-only (additional plugin
registration inside the fixture's `app_in_session()` -- no production
code touched). If diagnosis during implementation surfaces a production
runtime regression in the HUD phase bridge, the disposition is to
author a separate follow-on production-fix story and keep this story's
commit test-only -- mirroring the S11-TD-FIXTURE-HAND-UI-ONENTER-001
(Story 011) discipline.

---

## Story Classification

**Story type**: Fixture cleanup + binary design decision
(test-relocation OR fixture-expansion). Neither disposition is a
production code repair on its face; both are scoped to `tests/` unless
the implementation prompt's diagnosis surfaces a production runtime
regression.

This is **NOT** a:

- Repair story (no broken production runtime is asserted; the test is
  ignored because the fixture is misshaped, not because the production
  HUD bridge is broken).
- Pure decision story (B4 is the pure decision row; this story has
  concrete fixture work to land after the decision is recorded).
- Evidence-only story (B2 has an executable artifact -- the
  un-`#[ignore]`d test -- that lands as part of `/dev-story`).

---

## Scope

### In Scope

- **Producer decision recorded in this story file**: choose exactly one
  of:
  - **Path A -- Expand fixture**: add `HudPlugin` (and any minimal
    sub-plugin dependencies) to the `app_in_session()` fixture in
    `tests/integration/board_rendering/snapshot_spawn_test.rs` so the
    `snapshot.phase -> CurrentClientPhase` bridge runs end-to-end.
    Un-`#[ignore]` the test under this fixture.
  - **Path B -- Relocate assertion**: split the misplaced phase-bridge
    assertion out of
    `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`
    into a new dedicated HUD-side test (e.g. under
    `tests/integration/hud/snapshot_phase_bridge_test.rs` or extending
    an existing HUD test file). Un-`#[ignore]` whatever remains of the
    original test (which retains only the board-rendering-side
    assertions: stale visuals cleared, units + objectives spawned from
    snapshot).
- **Decision rationale written into this story file** (see "Design
  Decision" section below for the template). The rationale captures:
  fixture-expansion cascade risk, HUD coverage already present
  elsewhere, and reviewer cost.
- **Test un-`#[ignore]`d** under the chosen path. Pre/post pass count
  recorded in the evidence document.
- **Original PROMPT 750 D-5 owner comment removed** only after the test
  (or its replacement assertion) passes locally under the chosen path.
- **Evidence document slot reserved** at
  `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md`
  (NEW; populated by the implementation prompt -- this story authoring
  prompt does NOT populate the evidence file).

### Out of Scope

- **No production-code change in `client/`, `server/`, or `shared/`
  beyond what the chosen path explicitly requires** (Path A is
  test-only; Path B is test-only). If diagnosis during implementation
  surfaces a production runtime regression in the HUD phase bridge, the
  disposition is to author a separate follow-on production-fix story
  and keep this story's commit test-only.
- No expansion to other Cluster B residuals (B1/B3/B4/B5) -- each is
  scoped to its own Sprint 12 Must Have story.
- No expansion of the board-rendering snapshot test's coverage beyond
  the original assertion set.
- No Sprint 12 activation. No `production/stage.txt` modification.
  No `production/sprint-status.yaml` modification. No
  `production/sprints/sprint-12.md` modification under this story.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, QA sign-off, or close-out under this story authoring
  prompt.
- No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or any
  carried Sprint 11 / Sprint 10 condition.
- No claim of public release readiness, release-candidate readiness,
  full playable-client manual QA, full game completion, broad
  Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.

---

## Design Decision (to be recorded by the implementation prompt)

The implementation prompt MUST record exactly one of:

- [ ] **Path A -- Expand fixture to include `HudPlugin`**. NOT chosen.
- [x] **Path B -- Relocate the `snapshot.phase -> CurrentClientPhase`
      assertion to a HUD-side test**. CHOSEN by PROMPT 806
      (`/dev-story`) on `origin/main@d8d0196` in worktree
      `work/s11-fixture-hud-snapshot-phase-bridge`. Rationale recorded
      below before any test code modification:

      1. **Authority root-cause**: `BoardRenderingPlugin` does not
         own the `snapshot.phase -> CurrentClientPhase` write. The
         write happens in
         `client::ui::hud::handle_game_snapshot_system`
         (`client/src/ui/hud/mod.rs:884-941`, lines 940-941:
         `current.phase = snapshot.phase; current.round =
         snapshot.round_number;`), which is a `HudPlugin` system
         consuming `PresentationGameSnapshotMessage`. The board
         rendering plugin consumes a different message
         (`ClientGameSnapshotMessage`) and writes only board-side
         state (`BoardRenderState`, unit/objective entities,
         `AnimQueue`, `PendingPhaseChange`, etc.) -- it never writes
         `CurrentClientPhase`. Putting the assertion in a
         `BoardRenderingPlugin`-only fixture is therefore a
         test-layout defect, not a fixture-cascade problem.
      2. **HUD coverage already exists**: the HUD bridge invariant is
         already exercised by
         `tests/integration/hud/reconnect_snapshot_rebuild_test.rs::full_snapshot_rebuild_populates_all_hud_zones_without_respawning_entities`
         at lines 65-69, which asserts
         `app.world().resource::<CurrentClientPhase>().phase ==
         RoundPhase::Placement` and
         `app.world().resource::<CurrentClientPhase>().round == 7`
         after a `PresentationGameSnapshotMessage` is written into
         an App with `HudPlugin`. Path B does NOT create a coverage
         gap. To make the invariant trace explicit (vs buried inside
         a 9-zone rebuild test), PROMPT 806 adds a small dedicated
         HUD-side test
         `tests/integration/hud/snapshot_phase_bridge_test.rs`
         whose single responsibility is the
         `snapshot.phase + snapshot.round_number ->
         CurrentClientPhase` bridge under `HudPlugin`.
      3. **Path A cost (rejected)**: registering `HudPlugin` into
         `app_in_session()` cascades into HUD asset-wiring fixture
         work (HUD UI nodes, text spans, asset placeholders for
         class figurines, scoreboard dot textures, gold/mana label
         entities), all of which is already wired in
         `tests/integration/hud/*` fixtures. Doubling that wiring in
         the board-rendering fixture inflates fixture surface for
         every other test in `snapshot_spawn_test.rs` (the other
         five tests in that file are pure board-rendering and do not
         need a HUD plugin) and increases reviewer cost without
         improving invariant coverage.
      4. **AC5 / ADR conformance**: Path B is test-only. No
         production code in `client/`, `server/`, or `shared/` is
         touched. ADR-002 (no optimistic client-side authority),
         ADR-009 (server-authoritative phase transitions), and
         ADR-021 (single shared phase sink) remain binding and
         unchanged. The HUD bridge already conforms to ADR-021 by
         reading from the shared `Res<CurrentClientPhase>`.

The decision is binary; both paths are acceptable. Whichever path is
chosen, the rationale is captured in **this story file** (the
implementation prompt edits this section) before any test code is
modified.

**Default producer recommendation (advisory only; not binding)**: Path B
(relocate) -- mirrors the precedent that the misplaced assertion is the
defect, not the fixture composition. If the HUD-side coverage gap turns
out to be real (i.e., no existing HUD test covers the snapshot.phase
bridge), Path A becomes binding because relocation would create the
gap.

**Coverage gap verification (PROMPT 806)**: The "Open Questions" item
"Is there an existing HUD-side integration test that already covers the
`snapshot.phase -> CurrentClientPhase` bridge invariant?" resolved
**YES** at implementation time (see rationale item 2 above). The
producer recommendation (Path B) is therefore binding; no coverage gap
is created.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-12.md:124` Sprint 12 draft Must Have
row. ACs below are draft and become binding at Sprint 12 activation.)

- [x] **AC1 -- Design decision recorded in this story file**: GIVEN
      this story file, WHEN the "Design Decision" section is read at
      the implementation commit, THEN exactly one of {Path A, Path B}
      is checked, the unchecked path is explicitly marked NOT chosen,
      and a written rationale is present under the chosen path.
      *Evidence*: this story file's "Design Decision" section.

- [x] **AC2 -- Test un-`#[ignore]`d and passes under chosen path**:
      GIVEN the chosen path, WHEN
      `cargo test -p client --test snapshot_spawn` (or the equivalent
      `cargo test` invocation that exercises whichever file the
      assertion lives in after the decision) is run at the
      implementation commit, THEN
      `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`
      (Path A) **or** the relocated HUD-side test plus the residual
      board-rendering test (Path B) pass without `#[ignore]` tagging.
      *Evidence*: pre/post pass count captured in the evidence
      document.

- [x] **AC3 -- Workspace ignored count drops by 1**: GIVEN Sprint 11
      close-out baseline of 5 retained Cluster B `#[ignore]` tests on
      `origin/main`, WHEN
      `cargo test --workspace --tests --no-fail-fast` is run at the
      implementation commit, THEN the workspace ignored count drops by
      1 (relative to the baseline) and no new undocumented `#[ignore]`
      marker is introduced.

- [x] **AC4 -- Original PROMPT 750 D-5 owner comment removed only
      after the test passes**: GIVEN the implementation commit, WHEN
      `tests/integration/board_rendering/snapshot_spawn_test.rs` and
      any new HUD-side test file are read, THEN no PROMPT 750 D-5
      owner comment for this test remains. If Path B was chosen, the
      relocated assertion in the HUD-side test does NOT carry a
      PROMPT 750 D-5 owner comment (this is a new test, not a
      `#[ignore]`d test).

- [x] **AC5 -- No production code modified (default path)**: GIVEN
      the diff of the implementation commit set, WHEN the diff is
      filtered to `server/src/`, `client/src/`, and `shared/src/`,
      THEN zero production-code changes are present **unless** the
      chosen path's rationale explicitly requires a production-code
      change (e.g., a discovered HUD bridge bug surfaced during
      diagnosis). In that case, the production change is scoped
      narrowly to the surfaced bug, has its own follow-on story file
      authored, and is NOT bundled into this story's commit.
      *Evidence*: `git show` of every commit in this story's trail
      filtered to non-test paths.

- [x] **AC6 -- No optimistic client-side authority introduced**: GIVEN
      the implementation commit, WHEN the diff is reviewed for any
      client-side mutation of phase state outside the shared phase
      sink, THEN no such mutation is present. ADR-002 and ADR-009
      remain binding.

- [x] **AC7 -- Sprint 12 disposition preserved**: GIVEN the
      implementation commit, WHEN `production/sprint-status.yaml`,
      `production/sprints/sprint-12.md`, and `production/stage.txt`
      are diffed, THEN none of them are modified under this story.
      Sprint 12 activation disposition (set by `/sprint-plan
      sprint-12` in a separate prompt) is preserved. Stage remains
      `Polish`. Sprint 11 disposition (`closed-with-conditions`) is
      unchanged.

- [x] **AC8 -- Evidence document slot reserved**: GIVEN this story
      file, WHEN the evidence-doc path is checked, THEN a slot is
      reserved at
      `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md`
      for population by the implementation prompt. Authoring of the
      evidence file itself is deferred to the implementation prompt
      per the S10-TD-001 / S11-TD-FIXTURE-HAND-UI-ONENTER-001
      paperwork-first precedent.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. `/sprint-plan sprint-12` activates Sprint 12 (separate prompt).
2. This story passes `/story-readiness` (separate prompt).
3. Sprint 12 `/qa-plan sprint` is authored (separate prompt).
4. `/dev-story story-012-fixture-hud-snapshot-phase-bridge.md` is
   dispatched (separate prompt).

Expected implementation flow:

1. **Wave 1 -- Decision**: the implementation prompt reads the
   board-rendering and HUD test layout, identifies whether the HUD
   phase-bridge invariant already has dedicated coverage in
   `tests/integration/hud/*` (default Path B trigger if yes), and
   records the decision in this story file's "Design Decision"
   section.
2. **Wave 2 -- Test work**: apply the chosen path to the test code.
   Path A: expand `app_in_session()` in `snapshot_spawn_test.rs` to
   register `HudPlugin` (plus any minimal sub-plugin dependencies);
   un-`#[ignore]` the test. Path B: extract the
   `snapshot.phase -> CurrentClientPhase` assertion into a new
   HUD-side test (or append to an existing HUD test file);
   un-`#[ignore]` the residual board-rendering test (which retains
   the stale-visuals and snapshot-spawn assertions).
3. **Wave 3 -- Validation**: run `cargo test -p client --no-fail-fast`
   and `cargo test --workspace --tests --no-fail-fast`; capture
   pre/post pass + ignored counts in the evidence document.
4. **Wave 4 -- Evidence doc**: populate
   `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md`
   with the decision rationale, fixture-or-relocation diff summary,
   pre/post counts, and any production-runtime evidence (none
   expected; if surfaced, follow-on story authored separately).

---

## Performance Budget

N/A -- test-fixture changes (and at most an assertion relocation) only.
No hot-path code changed.

---

## QA Test Cases

(Draft -- becomes binding at Sprint 12 activation. Sprint 12 QA plan
authored via `/qa-plan sprint` will pull from this set.)

- **HUD snapshot phase bridge invariant**
  - Given: implementation commit set on `main` for this story.
  - When: the chosen path's test (Path A: original test re-armed; Path
    B: relocated assertion in HUD-side test) is run via `cargo test`.
  - Then: assertion `snapshot.phase -> CurrentClientPhase` passes;
    `HudPlugin` is in the App graph at assertion time.

- **Workspace ignored-count regression check**
  - Given: Sprint 11 close-out baseline (1129 passing / 5 ignored on
    `origin/main@8a8451e`).
  - When: `cargo test --workspace --tests --no-fail-fast` is run at the
    implementation commit.
  - Then: workspace ignored count is at most `5 - 1 = 4` and no new
    undocumented `#[ignore]` marker is introduced.

- **Production source diff audit**
  - Given: the union diff of every commit in this story's trail.
  - When: paths under `server/src/`, `client/src/`, and `shared/src/`
    are filtered.
  - Then: zero production-code changes are present (default path). If
    a production change is present, it has its own follow-on story
    file authored and is NOT bundled into this story's commit.

- **Sprint 12 disposition preservation audit**
  - Given: the implementation commit.
  - When: `production/sprint-status.yaml`,
    `production/sprints/sprint-12.md`, and `production/stage.txt` are
    diffed.
  - Then: none of them are modified under this story id.

---

## Test Evidence

**Story Type**: Integration (test-fixture / test-relocation)

**Evidence path**: `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Path A or Path B decision + rationale (transcribed from this story
  file's "Design Decision" section).
- Pre/post `cargo test -p client` pass + ignored counts.
- Pre/post `cargo test --workspace --tests --no-fail-fast` pass +
  ignored counts.
- Helper or fixture diff summary (Path A: list of plugins added to
  `app_in_session()`; Path B: location of relocated assertion + name of
  new test).
- No-claim restatement (verbatim from this story file's "Status /
  No-Claim Banner" section).
- Cross-link back to this story file and to Cluster B2 row 84 in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`.

**Required verification commands** (for the implementation prompt):

- `cargo test -p client --no-fail-fast`
- `cargo test --workspace --tests --no-fail-fast`
- `git diff <pre-impl-sha>..<impl-sha> -- 'server/src/**' 'client/src/**' 'shared/src/**'`
  (expected: empty)
- `git diff --check` and `git diff --cached --check` before commit
- Story file's "Design Decision" section read at implementation commit
  (exactly one path checked, rationale present)

**Status**: [ ] Captured and locked

---

## Owner / Classification

- **Owner**: board-rendering test-infra owner + HUD plugin owner +
  qa-lead (per Cluster B2 row 84 in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`).
- **Estimated days**: 0.75 (per Sprint 12 draft row).
- **Story classification**: fixture cleanup + binary design decision.

## Dependencies

- **Depends on**: Sprint 12 activation via `/sprint-plan sprint-12`
  (separate prompt). This story remains `Draft` until then.
- **Depends on**: Sprint 11 D-5 triage doc
  (`production/qa/evidence/sprint-11-ignored-d5-triage.md`, Cluster B2,
  row 84) for owner / disposition / decision-gate language.
- **Depends on**: Sprint 12 QA plan authored via `/qa-plan sprint`
  (separate prompt) before `/dev-story` runs.
- **Coordinated with**: `S11-TD-FIXTURE-D-RESIDUALS-001`
  (Story 015 -- Cluster B1+B5 umbrella; sibling fixture residuals
  from Wave 12 D-3 / D-4 / D-5 sweeps that do NOT share the
  `snapshot.phase` root cause are handled there).
- **Coordinated with**: `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`
  (Story 014 -- Cluster B4 binary decision; both stories are
  board-rendering-tagged but touch disjoint files).
- **Coordinated with**: `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`
  (Story 013 -- Cluster B3 lobby intent chain; no shared file scope).
- **Not coordinated with**: `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  (story 019 in hand-ui epic -- separate runtime-trace story; no
  shared file scope).

## Readiness Notes

**Implementation readiness verdict**: Draft -- substantive work has not
started. The live downstream gate is Sprint 12 activation, Sprint 12 QA
plan authorship, and `/story-readiness` PASS on this file.

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

- Is there an existing HUD-side integration test that already covers
  the `snapshot.phase -> CurrentClientPhase` bridge invariant? If yes,
  Path B (relocate) becomes the strong default. If no, Path A (expand
  fixture) becomes binding to avoid creating a coverage gap.
- If Path A is selected, what is the minimal `HudPlugin` sub-plugin
  set required for the bridge to fire end-to-end? (Determined by
  diagnosis at implementation time.)
- If Path B is selected, what is the destination test file? (e.g.,
  `tests/integration/hud/snapshot_phase_bridge_test.rs` NEW, or
  appended to an existing HUD test file.)

---

## Files Anticipated To Be Modified (planning estimate, NOT binding)

| Path | Anticipated change |
|------|--------------------|
| `tests/integration/board_rendering/snapshot_spawn_test.rs` | Path A: expand `app_in_session()` fixture to include `HudPlugin`; un-`#[ignore]` test. Path B: split phase-bridge assertion out; un-`#[ignore]` residual test. |
| `tests/integration/hud/snapshot_phase_bridge_test.rs` (NEW; Path B only) | new test asserting `snapshot.phase -> CurrentClientPhase` bridge with `HudPlugin` already wired. (May instead be an append to an existing HUD test file.) |
| `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md` (NEW) | evidence document per AC8 |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set. AC5's "no production code modified"
guard remains the hard constraint regardless of file count.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Path A fixture expansion cascades into asset-wiring + presentation-plugin setup that doubles the story scope | Medium | Medium | Scope-cap at the named test + minimum viable sub-plugin set. If the cascade exceeds 0.75 days, escalate to Path B. |
| Path B relocation creates a coverage gap because no existing HUD test asserts the phase bridge | Medium | Medium | `/story-readiness` open question forces verification; if no coverage exists, Path A becomes binding. |
| Diagnosis surfaces an actual HUD bridge bug | Low | Medium | Author a separate follow-on production-fix story; keep this story's commit test-only. |
| Implementation prompt bundles a production code change into this story | Low | High | AC5 hard constraint; verified by `git show` filtered to non-test paths. |
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
  Must Have for Cluster B2. Sprint 12 is `draft` (PROMPT 793) and not
  yet activated -- this story is **not yet activated** into the
  Sprint 12 active scope. Activation is a separate prompt
  (`/sprint-plan sprint-12`). No code changes, no smoke / gate / QA /
  `/dev-story` / `/story-done` / `/story-readiness` / `/qa-plan` run.
  Source-of-truth at authoring: `origin/main@f72cc60`.

- 2026-05-14 -- PROMPT 806 -- `/dev-story` worker landed Path B on
  worktree `work/s11-fixture-hud-snapshot-phase-bridge`: relocated the
  `snapshot.phase -> CurrentClientPhase` assertion into the new
  dedicated HUD-side test
  `tests/integration/hud/snapshot_phase_bridge_test.rs` and un-`#[ignore]`d
  the residual board-rendering test in
  `tests/integration/board_rendering/snapshot_spawn_test.rs`. Original
  PROMPT 750 D-5 owner comment removed. Evidence captured at
  `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md`.
  No production code under `client/src/`, `server/src/`, `shared/src/`
  touched (test-only diff). ADR-002, ADR-009, ADR-021 preserved.

- 2026-05-14 -- PROMPT 809 -- Integration verification:
  PROMPT 806 commit was already on `origin/main` as `c1eef10`
  (`dev(s12-b2): Path B relocate snapshot.phase HUD bridge assertion (PROMPT 806)`).
  Workspace test suite at HEAD reported **1133 pass / 0 fail / 2 ignored**.
  No additional push required.

- 2026-05-14 -- PROMPT 814 -- `/story-done` paperwork: this Status
  field flipped Draft -> Done; AC checkboxes resolved against
  `origin/main@a3c624e` evidence; `production/sprint-status.yaml`
  Sprint 12 Must Have row `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`
  flipped `status: ready -> done` with `completed: 2026-05-14`. Sprint 12
  is NOT closed-out by PROMPT 814 (4 of 5 Must Have rows remain pending
  ahead of this batch; this batch only closes the 5 Must Have rows
  whose worker + integration evidence is on `origin/main`). No
  `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, no
  Sprint 12 close-out, no stage advance, no S8-QA-001-W1 closure, no
  release-readiness claim. Carry conditions and non-claims preserved
  verbatim from this story file's "Status / No-Claim Banner".
