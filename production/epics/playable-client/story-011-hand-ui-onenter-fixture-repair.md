# Story 011: Hand UI `OnEnter(InSession)` Fixture-Cascade Repair

> **Epic**: Playable Client
> **Story ID**: S11-TD-FIXTURE-HAND-UI-ONENTER-001
> **Status**: Draft -- Sprint 11 draft Must Have, NOT activated
> **Layer**: Tech Debt / Test Fixtures
> **Type**: Integration (test-only)
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 11 (draft -- NOT active per PROMPT 764)
> **Authored**: 2026-05-13 by PROMPT 767 (producer + qa-lead, root checkout)
> **Source-of-truth at authoring**: `origin/main@2f9abfb`

## Context

Sprint 10 smoke retry-7 (`production/qa/smoke-sprint-10-2026-05-12-retry-7.md`)
landed `PASS WITH WARNINGS` with 1123/1123 effective passing and 11
`#[ignore]`d D-5 tests preserved for owner review (W1). Six of those
ignored tests carry an owner-named comment that points at the same root
cause: `spawn_hand_ui` is not firing on `OnEnter(InSession)` when the
fixture builds an App from `MinimalPlugins` plus a hand-picked subset of
`HandUiPlugin` and its sub-plugins. The result is that
`HandUiEntities` is never inserted, fan slots are never spawned, and
every entity-presence assertion downstream fails.

PROMPT 762 candidate-backlog capture (recorded in
`production/session-state/codex-orchestrator-state.md`) names this cluster
as the "7x `spawn_hand_ui` OnEnter fixture cascade" -- highest-value
follow-on item out of Sprint 10 close-out. Sprint 11 draft Must Have row
`S11-TD-FIXTURE-HAND-UI-ONENTER-001` in `production/sprints/sprint-11.md`
pulls it forward. Smoke retry-7 enumerates 6 explicitly Hand UI tagged
tests in this cluster; the 7th referenced in the PROMPT 759 closeout may
have shifted disposition or been folded into another bucket between
retry-5 and retry-7. Scope of this story is defined by the cluster as
documented in smoke retry-7, plus any sibling ignored test that
post-retry-7 evidence proves shares the same root cause.

The cascade has a known shape from S10-TD-001 prior art (closed under
`story-009-test-fixture-cascade-fail-repair.md`):

- **Layer 1**: `add_message::<T>()` dedup waves (`200d2d9` + `6f77d4b`)
  removed silent duplicate registrations that had been keeping
  partial-App fixtures alive.
- **Layer 2**: `f5b7a34` removed the inner `init_state::<ClientState>()`
  from `HandUiPlugin`'s sub-plugins; `MinimalPlugins` fixtures had to
  add their own.
- **Layer 3**: `b92aa97` made `spawn_hand_ui` early-return on
  `Option<Res<PlaceholderAssets>>::None`; `MinimalPlugins` fixtures had
  to insert a `PlaceholderAssets` resource via
  `placeholder_assets_for_tests()`.
- **Layer 4 (this story)**: even after Layers 1-3 are satisfied per
  S10-TD-001, the `OnEnter(InSession)` transition that drives
  `spawn_hand_ui` does not fire end-to-end in some `MinimalPlugins`
  fixtures. State transitions are scheduled but the run condition or
  state-update set required to actually apply `NextState<ClientState>` is
  missing, so `spawn_hand_ui` never runs and `HandUiEntities` never
  spawns.

This story is **diagnosis + repair of the fixture/test layer only**.
Scope to production code in `client/`, `server/`, or `shared/` is gated
on source evidence that production runtime is affected -- this story
does not pre-authorise such a change. If diagnosis surfaces a real
production runtime bug, the disposition is to (a) record the runtime
evidence in this story's evidence document, (b) author a separate
follow-on production-fix story, and (c) keep this story's commit
test-only.

This story does **not** activate Sprint 11, change `production/stage.txt`,
modify Sprint 10 disposition, close any S8 / Sprint 9 carried condition,
modify `production/sprint-status.yaml`, modify `production/sprints/sprint-11.md`,
run `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
or claim public release readiness.

**Primary sources**:

- `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` (the 11 ignored
  tests table, lines 60-74; the 6 Hand UI tagged entries in that table
  define this story's primary scope)
- `production/sprints/sprint-11.md` (Sprint 11 draft Must Have row
  `S11-TD-FIXTURE-HAND-UI-ONENTER-001`, line 92)
- `production/session-state/codex-orchestrator-state.md` (PROMPT 762
  candidate #2 capture, "7x `spawn_hand_ui` not firing on
  `OnEnter(InSession)` in `MinimalPlugins` fixtures"; Sprint 11 draft
  top-5 Must Have rationale)
- Pattern reference: `production/epics/playable-client/story-009-test-fixture-cascade-fail-repair.md`
  (S10-TD-001 prior art -- Layers 1-3 of the same cascade)

**GDD, UX, and TR trace**:

- No GDD requirement. This is a test-fixture tech-debt repair story --
  there is no TR-ID in `docs/architecture/tr-registry.yaml` for fixture
  hygiene.
- The repair protects the existing TR-HU / TR-PRES surface area by
  ensuring `cargo test -p client` partial-App fixtures can drive
  `OnEnter(InSession)` end-to-end through `spawn_hand_ui` without
  silently skipping the entity-spawn step that downstream assertions
  depend on.

**ADR Governing Implementation**:

No ADR governs this story directly. ADR-021 (presentation boundaries)
constrains what each fixture is allowed to assert about; this story
conforms each fixture to ADR-021 by ensuring the `MinimalPlugins`
fixture actually drives the same `OnEnter(InSession)` -> `spawn_hand_ui`
chain that the production App composition drives. No protocol or
architecture decision is changed.

**Engine**: Bevy 0.18 (Rust) | **Risk**: MEDIUM (silent failure class --
fixture skips entity spawn, downstream assertions fail with cryptic
"entity not found" messages, ignored-marker accretion masks production
regressions)

**Engine Notes**: Bevy 0.18 splits state transitions into a scheduled
set (`StateTransition`) that runs between `PreUpdate` and `Update`.
`OnEnter(S::Variant)` schedules are only run when the
`StateTransition` schedule runs and `NextState<S>` is observed. A
fixture that builds from `MinimalPlugins` and inserts state via
`init_state::<ClientState>()` still needs at least one frame of
`app.update()` with `NextState::<ClientState>::Pending(InSession)`
applied between the seed and the assertion for the `OnEnter` system
chain to fire. Additionally, some `OnEnter(InSession)` systems use
`OnEnter(ClientState::InSession)` as a `SystemSet`-style trigger;
omission of any prerequisite system, run condition, or required
resource will silently skip the spawn without panicking. The repair
must close this gap by (a) ensuring the right `OnEnter` schedule runs
in fixture context, (b) ensuring all prerequisite resources exist
before the schedule runs, and (c) advancing the App with
`app.update()` until the spawn has actually happened.

**Control Manifest Rules (2026-05-05)**: Not applicable in the
production-code sense at story-authoring time. Scope is `tests/` plus
(optionally) a `tests/helpers/` (or existing `client/src/asset_wiring.rs`
`#[cfg(test)]` helper) test-only helper. No presentation, networking, or
gameplay production code path is altered unless diagnosis proves a
production runtime regression and a separate follow-on production-fix
story is authored.

---

## Scope

### In Scope

- **Diagnosis**: capture a minimal reproducible fixture pattern that
  matches the ignored-test cluster; identify whether the gap is
  (a) missing `StateTransition` advancement, (b) missing run-condition
  resource for `spawn_hand_ui`, (c) missing parent-plugin registration
  whose schedule registers the relevant `OnEnter(InSession)` system,
  (d) an off-by-one `app.update()` between `NextState::set` and the
  assertion, or (e) something else surfaced by tracing.
- **Repair (fixture-only)**: for each affected fixture, add the minimum
  viable lines to drive `OnEnter(InSession)` end-to-end through
  `spawn_hand_ui` and `HandUiEntities` insertion. Permissible repairs
  include: explicit `NextState<ClientState>::Pending(InSession)` +
  `app.update()` cycles; explicit insertion of a missing run-condition
  resource (e.g., a snapshot or session token marker); adding a parent
  sub-plugin that registers the missing schedule; or factoring a shared
  `enter_in_session_via_fixture(&mut App)` helper.
- **Helper authoring**: add or extend a single test-only helper (under
  `tests/helpers/` if that path exists in the workspace, or as a
  `#[cfg(test)]` fn in an existing client production-source helper file
  mirroring the `placeholder_assets_for_tests()` precedent from
  S10-TD-001) so the affected fixtures share one insertion path instead
  of duplicating the in-session entry sequence.
- **Pattern documentation**: append the canonical fixture pattern to an
  existing test-pattern doc (or create
  `docs/architecture/test-fixture-patterns.md` if no comparable doc
  exists) so future fixture authors are not forced to rediscover the
  cascade. Scope of the doc is one page: pre-conditions, helper
  signature, the `OnEnter(InSession)` driving sequence, and a link back
  to this story for evidence.
- **Per-test disposition**: for each of the 6 explicitly Hand UI tagged
  ignored tests from smoke retry-7 (and any additional sibling test
  that diagnosis proves shares the same root cause), record one of:
  (a) un-`#[ignore]` after fixture-only repair lands; (b) un-`#[ignore]`
  after a separate production-fix story lands and is referenced here;
  (c) formally redesign + retain `#[ignore]` with a documented reason
  and a follow-on story id; (d) delete with rationale recorded in the
  triage doc. Tests covered by `S11-TD-IGNORED-D5-TRIAGE-001` are
  linked, not re-resolved here.
- **Evidence document** (slot reserved): one evidence file at
  `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
  recording, per affected test: file path, repair lines added, pre/post
  pass count, sibling tests confirmed in-cluster, and any production
  runtime evidence surfaced during diagnosis. Authoring of this evidence
  file is **deferred to a separate prompt** (story-doc authoring is
  paperwork-first per the friend-game-lite practice that closed
  S10-TD-001).

### Out of Scope

- **No production-code change in `client/`, `server/`, or `shared/`
  beyond a single test-only helper fn**, unless diagnosis evidence
  proves a production runtime regression. In that case, a separate
  follow-on production-fix story is authored and referenced from this
  story; the production code change does NOT land under this story id.
- No expansion to plugin-registration audit (S10-TD-002 family) or
  broader fixture-cascade sweep beyond the cluster scoped above.
- No Sprint 11 activation. No `production/stage.txt` modification.
  No `production/sprint-status.yaml` modification. No
  `production/sprints/sprint-11.md` modification under this story.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, QA sign-off, or close-out under this story.
- No closure of S8-QA-001-W1, QA-COND-0005, QA-COND-0006, or any other
  carried-forward condition from Sprint 10 close-out.
- No claim of public release readiness, release-candidate readiness,
  full playable-client manual QA, full game completion, broad
  Standard-tier accessibility completion, playtest / fun-hypothesis
  validation, or final-art / asset-production completion.
- No new automated E2E test that boots the production App and asserts
  every fixture builds successfully (this remains the natural follow-up
  to S10-TD-002's plugin-registration audit and stays out of scope
  here).

---

## Affected Ignored Tests (smoke retry-7 W1)

Source: `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` lines 60-74.

| # | Test name | Smoke retry-7 owner-comment |
|---|-----------|----------------------------|
| 1 | `test_hand_pointer_controls_stage_unstage_and_submit_placement` | D-5 follow-on: `spawn_hand_ui` not firing on `OnEnter(InSession)` in `MinimalPlugins` fixture |
| 2 | `test_one_card_acquired_fanout_updates_hand_and_draft_pending_purchase` | D-5 follow-on: `HandUiEntities` never spawned in `MinimalPlugins` fixture |
| 3 | `test_one_draft_offering_fanout_updates_hand_grid_and_shop_grid` | D-5 follow-on: `HandUiEntities` never spawned in `MinimalPlugins` fixture |
| 4 | `test_placement_exit_clears_stale_hand_timer_submit_and_pending_state` | D-5 follow-on: `HandUiEntities` missing after fixture transitions to `InSession` |
| 5 | `test_reserve_strip_input_does_not_mutate_player_economy_view` | D-5 follow-on: fan slots never spawned |
| 6 | `test_shop_purchase_reconciles_hand_size_slots_and_shared_economy` | D-5 follow-on: `HandUiEntities` never spawned |

**Cluster count note**: Sprint 11 draft AC targets `7x` per the PROMPT 762
candidate-backlog capture (which references the PROMPT 759 closeout
worker comment). The smoke retry-7 table enumerates 6 explicitly Hand UI
tagged tests. The 7th may have shifted disposition between retry-5 and
retry-7 (a different `#[ignore]` cause was assigned, or the test was
deleted under the PROMPT 759 sweep). Diagnosis must confirm whether a
7th sibling test exists in the cluster; if so, list it under
"Additional sibling tests" in the evidence document. If no 7th exists,
record that fact explicitly in the evidence document and update the
Sprint 11 draft acceptance language at activation time.

**Sibling ignored tests not in primary scope** (recorded for cross-link
only -- handled by adjacent Sprint 11 candidates, not by this story):

- `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui`
  (board-rendering side; `GhostDragStartEvent` producer fixture gap --
  Should Have row `S11-TD-FIXTURE-D-RESIDUALS-001`).
- `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
  (`ShopAuctionUiEntity` count drift 57->66 -- Should Have row
  `S11-TD-FIXTURE-D-RESIDUALS-001`).
- `test_cooccupancy_index_two_panics_with_offending_index` (cooccupancy
  panic-guard drift -- separate Sprint 11 candidate not folded into
  this story).
- `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
  (`ConfirmClass` intent chain after `SelectClass` -- separate Sprint 11
  candidate not folded into this story).
- `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`
  (`HudPlugin` snapshot.phase bridge fixture gap -- separate Sprint 11
  candidate not folded into this story).

If diagnosis surfaces that one or more of these sibling tests does in
fact share the `spawn_hand_ui` / `OnEnter(InSession)` root cause, record
the evidence and either (a) fold the test into this story's scope with
a written rationale, or (b) leave it under its current Sprint 11
candidate and cross-link.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-11.md:92` `S11-TD-FIXTURE-HAND-UI-ONENTER-001`
draft AC. ACs below are draft and become binding at Sprint 11 activation.)

- [ ] **AC1 -- Per-test disposition**: GIVEN the 6 ignored tests listed
      under "Affected Ignored Tests" above (plus any sibling test that
      diagnosis evidence proves shares the same root cause), WHEN the
      repair commit set is read, THEN each test is dispositioned as one
      of: (a) un-`#[ignore]` + passes locally under the corrected
      fixture pattern; (b) un-`#[ignore]` after a referenced separate
      production-fix story lands; (c) formally redesigned + retained
      `#[ignore]` with a documented reason and a follow-on story id;
      (d) deleted with rationale. *Evidence*: per-test row in the
      evidence document.

- [ ] **AC2 -- Ignored count reduction OR explicit owner disposition**:
      GIVEN the workspace-level ignored count at smoke retry-7 baseline
      (11 ignored tests), WHEN `cargo test --workspace --tests
      --no-fail-fast` is re-run at the repair commit, THEN either
      (a) the workspace ignored count drops by N where N equals the
      number of cluster tests un-`#[ignore]`d under AC1; OR (b) every
      test that remains `#[ignore]` carries an owner-named disposition
      comment that points at the resolving story id (this story id,
      or the referenced production-fix follow-on story id, or
      `S11-TD-IGNORED-D5-TRIAGE-001` if folded into the triage
      disposition). No silent `#[ignore]` retention without
      disposition.

- [ ] **AC3 -- Reusable fixture helper authored**: GIVEN the scope of
      the repair commit, WHEN the helper's signature and location are
      read, THEN a single test-only helper (in `tests/helpers/` if that
      path exists in the workspace, or as a `#[cfg(test)]` fn in an
      existing client production-source helper file mirroring
      `placeholder_assets_for_tests()`) drives the `OnEnter(InSession)`
      entry sequence end-to-end. The helper is called from every
      repaired fixture. No duplicated in-session entry boilerplate
      across the cluster.

- [ ] **AC4 -- Pattern documentation**: GIVEN the helper, WHEN the
      project docs are searched, THEN the canonical fixture pattern is
      documented at `docs/architecture/test-fixture-patterns.md` (new
      file, ~one page) or appended to an existing test-pattern doc.
      The doc names the helper, lists pre-conditions (state, asset,
      resource), shows a minimal example, and links back to this story.

- [ ] **AC5 -- `cargo test -p client` passes for repaired set**:
      GIVEN the repair commit, WHEN
      `cargo test -p client --no-fail-fast` is run, THEN no fixture in
      the repaired cluster panics with `Resource not found` or
      `HandUiEntities not found`; every test that was un-`#[ignore]`d
      under AC1 passes. Pre/post pass counts recorded in the evidence
      document.

- [ ] **AC6 -- No production code modified**: GIVEN the diff of the
      repair commit set, WHEN the diff is filtered to `server/src/`,
      `client/src/` (excluding `#[cfg(test)]`-gated helper fns of the
      `placeholder_assets_for_tests()` precedent), and `shared/src/`,
      THEN zero production-code changes are present. If diagnosis
      surfaces a production runtime regression, the disposition is to
      author a separate follow-on production-fix story and reference it
      from this story's evidence document -- the production code change
      does NOT land under this story id. *Evidence*: `git show` of
      every commit in this story's trail filtered to non-test paths.

- [ ] **AC7 -- Sprint 11 disposition preserved**: GIVEN the repair
      commit, WHEN `production/sprint-status.yaml`,
      `production/sprints/sprint-11.md`, and `production/stage.txt` are
      diffed, THEN none of them are modified under this story.
      Sprint 11 remains `draft / not_active`. Stage remains `Polish`.
      Sprint 10 disposition (`closed-with-conditions`) is unchanged.
      No release claim, no `Polish->Release` retry, no manual-QA
      sign-off, no accessibility completion claim, no playtest
      validation claim.

- [ ] **AC8 -- Evidence document slot reserved**: GIVEN this story
      file, WHEN the evidence-doc path is checked, THEN a slot is
      reserved at
      `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
      for population by the implementation prompt(s). Authoring of the
      evidence file itself is deferred to the implementation prompt
      per the S10-TD-001 paperwork-first precedent.

---

## Implementation Notes

This story is **draft scope** at authoring time -- substantive work has
not yet landed on `main`. Activation requires (in this order):

1. `/sprint-plan sprint-11` activates Sprint 11 (separate prompt).
2. This story passes `/story-readiness` (separate prompt).
3. Sprint 11 `/qa-plan sprint` is authored (separate prompt).
4. `/dev-story story-011-hand-ui-onenter-fixture-repair.md` is
   dispatched (separate prompt).

The expected implementation flow, learned from the S10-TD-001 cascade:

1. **Wave 1 -- Diagnosis**: build a minimal failing fixture from one of
   the 6 affected tests; trace `OnEnter(InSession)` schedule with
   `bevy::log::Level::TRACE` and identify the missing prerequisite
   (run-condition resource, schedule registration, or update cycle).
   Record findings in
   `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`.
2. **Wave 2 -- Helper authoring**: factor the corrected fixture pattern
   into a reusable helper. The helper signature mirrors the
   `placeholder_assets_for_tests()` precedent and is callable from any
   partial-App fixture.
3. **Wave 3 -- Per-test repair**: apply the helper across the 6
   affected fixtures; un-`#[ignore]` each one that passes.
4. **Wave 4 -- Disposition record**: for any test that does NOT pass
   under the corrected helper (because diagnosis surfaced a deeper
   production-runtime gap), author a follow-on production-fix story
   id and reference it from the evidence document. Retain the
   `#[ignore]` with the corrected owner-named disposition comment.
5. **Wave 5 -- Pattern doc**: append the canonical pattern to
   `docs/architecture/test-fixture-patterns.md` (or equivalent).

The diagnosis wave must distinguish between two failure shapes:

- **Fixture-layer**: `OnEnter(InSession)` chain has all production
  prerequisites but the fixture is not driving them (missing
  `app.update()` cycle, missing state seed). This is the expected
  shape for the cluster.
- **Production-runtime layer**: `OnEnter(InSession)` is reachable in
  production but a missing run-condition or resource in
  `HandUiPlugin`'s sub-plugin composition makes the spawn silently
  skip under fixtures *and* under certain production paths. If
  diagnosis evidence points here, the disposition is to author a
  separate follow-on production-fix story and keep this story's
  commit test-only.

## Performance Budget

N/A -- test-fixture changes only; one test-only helper fn under
`#[cfg(test)]` paths or `tests/helpers/`. No hot-path code changed.

---

## QA Test Cases

(Draft -- becomes binding at Sprint 11 activation. Sprint 11 QA plan
authored via `/qa-plan sprint` will pull from this set.)

- **Cluster fixture cargo test pass**
  - Given: `main` at the post-repair commit set.
  - When: `cargo test -p client --no-fail-fast` is run.
  - Then: no test in the affected cluster panics with `Resource not
    found` or `HandUiEntities not found`; every test un-`#[ignore]`d
    under AC1 passes.

- **Workspace ignored-count regression check**
  - Given: smoke retry-7 baseline (11 ignored, 1123 passing).
  - When: `cargo test --workspace --tests --no-fail-fast` is run at
    the repair commit.
  - Then: workspace ignored count is `11 - N` where N is the number of
    tests un-`#[ignore]`d under AC1. No ignored count increase. Every
    remaining ignored test carries an owner-named disposition comment.

- **Production source diff audit**
  - Given: the union diff of every commit in this story's trail.
  - When: paths under `server/src/`, `client/src/` (excluding
    `#[cfg(test)]`-gated test-helper fns), and `shared/src/` are
    filtered.
  - Then: zero production-code changes outside the test-helper
    exception are present. If a production-runtime regression was
    surfaced during diagnosis, the follow-on production-fix story id
    is referenced from this story's evidence document.

- **Sprint 11 disposition preservation audit**
  - Given: the repair commit.
  - When: `production/sprint-status.yaml`,
    `production/sprints/sprint-11.md`, and `production/stage.txt` are
    diffed.
  - Then: none of them are modified under this story id.

---

## Test Evidence

**Story Type**: Integration (test-fixture / test-only)

**Required evidence document** (deferred -- populated by implementation
prompt):

- `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
  -- per-test disposition table; pre/post pass counts; helper
  signature + location; pattern-doc location; any production-runtime
  evidence surfaced during diagnosis; cross-links to follow-on
  production-fix story ids if any; cross-links to
  `S11-TD-IGNORED-D5-TRIAGE-001` per-test triage rows.

**Required source evidence before this story can close**:

- A repair commit (or commit set) on `main` that satisfies AC1-AC8.
- Pre/post `cargo test -p client` pass counts captured in the evidence
  document.
- Pre/post workspace ignored-count captured.
- Pattern doc at `docs/architecture/test-fixture-patterns.md` (or
  equivalent appended location) committed.

**Required verification commands**:

- `cargo test -p client --no-fail-fast`
- `cargo test --workspace --tests --no-fail-fast`
- `git show <repair-sha> --stat`
- `git diff <pre-repair-sha>..<repair-sha> -- 'server/src/**' 'client/src/**' 'shared/src/**'`
  (expected: empty or `#[cfg(test)]`-gated helper additions only)

**Status**: DRAFT -- substantive work has not started. Activation is
gated by Sprint 11 activation in a separate prompt.

---

## Files Anticipated To Be Modified (planning estimate, NOT binding)

| Path | Anticipated change |
|------|--------------------|
| `tests/integration/hand-ui/placement_*.rs` (and sibling files in the cluster) | un-`#[ignore]` + replace ad-hoc fixture-init with helper call |
| `tests/integration/hand-ui/draft_initial_*.rs` (if in cluster) | un-`#[ignore]` + replace ad-hoc fixture-init with helper call |
| `tests/integration/hand-ui/shop_purchase_*.rs` (if in cluster) | un-`#[ignore]` + replace ad-hoc fixture-init with helper call |
| `tests/unit/hand-ui/reserve_*.rs` (if in cluster) | un-`#[ignore]` + replace ad-hoc fixture-init with helper call |
| `tests/helpers/` (path TBD by implementation) OR `client/src/asset_wiring.rs` (mirroring `placeholder_assets_for_tests()`) | new `#[cfg(test)]` helper fn `enter_in_session_via_fixture(&mut App)` (or equivalent name) |
| `docs/architecture/test-fixture-patterns.md` (new) | canonical pattern doc (~one page) |
| `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md` (new) | evidence document per AC8 |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set. The S10-TD-001 cascade taught that
the realised file count can exceed initial estimates as deeper layers
surface -- AC6's "no production code modified" guard remains the hard
constraint regardless of file count.

---

## Dependencies

- **Depends on**: Sprint 11 activation via `/sprint-plan sprint-11`
  (separate prompt). This story remains `Draft` until then.
- **Depends on**: S10-TD-001 cascade Layers 1-3 already landed on
  `main` (`200d2d9`, `6f77d4b`, `7075da7`, `4b0c456`, `c11d1b6`,
  `bb51463`, `7c8f400`) -- this story is Layer 4 of the same cascade.
- **Depends on**: Smoke retry-7 `PASS WITH WARNINGS` baseline at
  `bc96700` (per `production/qa/smoke-sprint-10-2026-05-12-retry-7.md`)
  for the 1123/11 pre-repair ignored-count baseline.
- **Coordinated with**: `S11-TD-IGNORED-D5-TRIAGE-001` -- the 11
  ignored tests from smoke retry-7 W1 are triaged per-test by that
  story. Cluster tests covered by this story are linked from the
  triage doc, not re-resolved.
- **Coordinated with**: `S11-TD-FIXTURE-D-RESIDUALS-001` (Should Have)
  -- sibling fixture residuals from Wave 12 D-3 / D-4 / D-5 sweeps
  that do NOT share the `spawn_hand_ui` root cause are handled there.
- **Not coordinated with**: `S11-DRAG-RUNTIME-RETEST-001` -- separate
  runtime-trace story; no shared file scope.

## Readiness Notes

**Implementation readiness verdict**: NOT READY (Sprint 11 not active).

`/story-readiness` is the next step **after** Sprint 11 activation.
Until then, this story exists as a draft Must Have row reference for
Sprint 11 planning.

Pre-conditions for `/story-readiness` PASS:

- Sprint 11 is activated (`sprint:` field in
  `production/sprint-status.yaml` bumped + active row written).
- Sprint 11 QA plan exists at
  `production/qa/qa-plan-sprint-11.md`.
- This story file is referenced from Sprint 11's active row in
  `production/sprint-status.yaml`.

Open questions to resolve at `/story-readiness` time:

- Is there a 7th sibling Hand UI ignored test that diagnosis must
  pick up? (See "Cluster count note" above.)
- Is the helper destination `tests/helpers/` or
  `client/src/asset_wiring.rs` style co-location? (Either is
  acceptable; implementation prompt decides per existing workspace
  layout.)
- Does the pattern doc live at
  `docs/architecture/test-fixture-patterns.md` (new) or as an append
  to an existing doc? (Either is acceptable; implementation prompt
  decides.)

---

## Definition of Done

This story is **draft** at authoring time. Definition of Done at
implementation time:

- All AC1-AC8 satisfied.
- Repair commit(s) on `main`.
- Evidence document populated at
  `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`.
- Pattern doc committed.
- `/story-done` re-fires with this story file and flips
  `production/sprint-status.yaml`
  `S11-TD-FIXTURE-HAND-UI-ONENTER-001` -> `done` (separate prompt;
  NOT this prompt).
- If diagnosis surfaced a production-runtime regression, the follow-on
  production-fix story id is filed and referenced from the evidence
  document. The production code change does NOT land under this story.

This story does NOT claim public release readiness, release-candidate
readiness, full playable-client manual QA, full game completion, broad
Standard-tier accessibility completion, playtest / fun-hypothesis
validation, or final-art / asset-production completion.
