# Story 016: S13-FIXTURE-FACTORY-001 -- Canonical Production-Faithful Test App Factory

> **Epic**: Playable Client
> **Story ID**: S13-FIXTURE-FACTORY-001
> **Status**: Draft -- Sprint 13 candidate; NOT activated; Sprint 12 is the
> active sprint
> **Layer**: Test Infrastructure
> **Type**: Integration -- new test helper + targeted fixture migration
> **Sprint**: Sprint 13 candidate (per PROMPT 803 §6 line 143; NOT activated)
> **Authored**: 2026-05-14 by PROMPT 804 (worktree
> `work/s13-runtime-hardening-story-authoring`)
> **Authoring source-of-truth**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12
> QA-plan commit). Sprint 12 active per PROMPT 798 at `origin/main@796851b`.

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 804. Sprint 12 remains the active sprint
(`status: active` per `production/sprint-status.yaml` at
`origin/main@b5eef0d`) and must not be changed by this authoring run.
Activation of Sprint 13 happens via a separate `/sprint-plan sprint-13`
prompt after Sprint 12 close-out.

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

Sprint 10 disposition (`closed-with-conditions` per PROMPT 763) and
Sprint 11 disposition (`closed-with-conditions` per PROMPT 792) remain
unchanged. PROMPT 761 Polish->Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** The factory is test-only and mirrors the production client's
plugin composition; it does not introduce any new authoritative state
mutation path. ADR-002 binding for any fixture migrated onto the
factory -- the migrated fixture must continue to obey ADR-002 (client
is read-only over authoritative state).

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-7** Fixture parity divergence (MinimalPlugins vs production
  App) (HIGH): Production client = DefaultPlugins + 5 plugins; most
  fixtures = MinimalPlugins + 1-2 plugins. Cluster A (6 tests) was
  fixed; Cluster B (5 retained) still ignored; no canonical
  "production-faithful test app factory" exists.
- **§3 DC-8** Tests asserting observables without producer
  verification (HIGH): B1, B2, B5 assert message-counts / entity-
  counts / resource state without proving the producer system
  actually ran in the fixture.
- **§4 Lane D Fixture parity / Ignored tests**:
  - `app_with_board_rendering()`
    (`tests/integration/board_rendering/ghost_preview_bridge_test.rs:181-195`)
    -- missing `HandUiPlugin`; producer absent (B1).
  - `app_in_session()`
    (`tests/integration/board_rendering/snapshot_spawn_test.rs:420-433`)
    -- missing `HudPlugin` (B2).
  - `lobby_app()`
    (`tests/integration/playable_client/native_operator_controls_test.rs:260-272`)
    -- no `enter_in_session_via_fixture` call; `PlaceholderAssets`
    missing.
  - `shop_app()` (`:274-302`) -- no helper call; entity-count drift
    57 expected / 66 actual (B5).
  - `hand_app()` (`:304-322`) -- *correct* (uses
    `enter_in_session_via_fixture`).
- **§4 Lane D "Canonical helper exists"**:
  `client/src/asset_wiring.rs:420-453`
  (`enter_in_session_via_fixture`). Pattern doc:
  `docs/architecture/test-fixture-patterns.md`. **There is no
  `tests/helpers/production_app_factory.rs`** -- that absence is DC-7.
- **§5 Must row 5 (S13-FIXTURE-FACTORY-001)**: "Canonical
  'production-faithful test app factory' in
  `tests/helpers/production_app_factory.rs` that mirrors
  `client::main` / `server::main` plugin sets; migrate B1, B2,
  lobby_app, shop_app onto it".
- **§6 PROMPT-N+3 dispatch slot**: paperwork-only story-authoring
  for this row; parallel-safe with other Sprint 13 candidate stories.

---

## Problem Class / Prevention Target

**Defect class (DC-7 + DC-8)**: Test fixtures construct ad-hoc Bevy
`App` instances with `MinimalPlugins` + a hand-picked subset of
production plugins. Each fixture diverges from `client::main` (and
`server::main`) in different ways:

- `HandUiPlugin` missing in board-rendering fixture (B1).
- `HudPlugin` missing in board-rendering fixture (B2).
- `enter_in_session_via_fixture` helper not called in lobby_app /
  shop_app fixtures.
- `PlaceholderAssets` resource missing (lobby_app).
- Production spawn count (66) diverges from formula constant (57)
  because the fixture's plugin set doesn't actually run the spawn
  pipeline (B5).

Symptoms: tests pass under a non-production plugin set, but the
observable property the test asserts never actually exercises the
production producer system. Green test != production-correct.

**Prevention target**: A canonical helper at
`tests/helpers/production_app_factory.rs` that builds a Bevy `App`
matching the production client (and server) plugin composition exactly:

- `client::production_app_factory() -> App` -- mirrors
  `client::main::main()` plugin composition (DefaultPlugins + 5
  plugins, registered in the canonical order, with the same
  `register_protocol` call, the same `enter_in_session_via_fixture`
  helper invocation, and the same `PlaceholderAssets` resource
  insertion).
- `server::production_app_factory() -> App` -- mirrors
  `server::main::main()` plugin composition.

Fixtures that today diverge (B1, B2, lobby_app, shop_app) are migrated
onto the factory. Each migration:

- Removes the ad-hoc fixture builder.
- Replaces it with a `production_app_factory()` call + any
  test-specific resource overrides (timer scaling, deterministic RNG
  seed, etc.).
- Re-runs the test and confirms it either (a) passes with the same
  semantics as before, or (b) fails in a more honest way that
  reflects the production plugin set's actual behaviour (in which
  case the test's assertion is updated to match production reality,
  with a rationale entry in the evidence doc).

The new helper is the canonical pattern for all future fixture work
in the project. `docs/architecture/test-fixture-patterns.md` is
updated to cite the factory as the default; ad-hoc
`MinimalPlugins`-based fixtures are reserved for narrow unit-test
scope with an inline rationale comment.

---

## Context

### Existing surface

- **Production client plugin set** (`client/src/main.rs` +
  `client/src/lib.rs`): `DefaultPlugins` + 5 plugins
  (`ClientNetworkPlugin`, `PresentationPlugin`, `HandUiPlugin`,
  `HudPlugin`, `LobbyPlugin` or equivalent canonical set --
  implementation prompt confirms the exact list).
- **Production server plugin set** (`server/src/main.rs` +
  `server/src/lib.rs`): `DefaultPlugins` (or
  `MinimalPlugins`-equivalent for headless) + the canonical server
  plugin set (`ServerNetworkPlugin`, RSM plugin, session plugin,
  pool plugin, auction plugin, acquisition plugin, objective plugin,
  combat plugin).
- **Canonical fixture helper**:
  `client/src/asset_wiring.rs:420-453` exports
  `enter_in_session_via_fixture(app: &mut App)` which performs the
  session-entry sequence used by Cluster A fixture migrations.
- **Pattern doc**:
  `docs/architecture/test-fixture-patterns.md` describes the existing
  fixture patterns but does not yet name the production-app-factory
  as the canonical default.

### Fixture migration targets

| Fixture | Location | Current divergence | Migration |
|---------|----------|---------------------|-----------|
| `app_with_board_rendering()` | `tests/integration/board_rendering/ghost_preview_bridge_test.rs:181-195` | Missing `HandUiPlugin`; producer absent (B1) | Replace with `production_app_factory()` + assertion sanity-check; if Sprint 12 Story 015 chose Path B1.a (expand fixture) the migration is trivial; if Path B1.b (relocate assertion), the migration covers the residual board-rendering test only. |
| `app_in_session()` | `tests/integration/board_rendering/snapshot_spawn_test.rs:420-433` | Missing `HudPlugin` (B2) | Replace with `production_app_factory()`; coordinate with Sprint 12 Story 012's Path A vs B decision. |
| `lobby_app()` | `tests/integration/playable_client/native_operator_controls_test.rs:260-272` | No `enter_in_session_via_fixture` call; `PlaceholderAssets` missing | Replace with `production_app_factory()` + lobby-state setup. Coordinate with Sprint 12 Story 013 (B3 lobby ConfirmClass) which lands first. |
| `shop_app()` | `tests/integration/playable_client/native_operator_controls_test.rs:274-302` | Entity-count drift 57 expected / 66 actual (B5) | Replace with `production_app_factory()`; the count drift is expected to resolve because the production spawn pipeline runs end-to-end. Coordinate with Sprint 12 Story 015's Path B5.a (update formula) or Path B5.b (trim spawn) decision. |
| `hand_app()` | `tests/integration/playable_client/native_operator_controls_test.rs:304-322` | Already correct; uses `enter_in_session_via_fixture` | Migrate as a sanity-check (should be a no-op semantic change); validates that `production_app_factory()` is compatible with the existing pattern. |

### Sprint 12 coordination

Sprint 12 Must Have rows already touch B1 (Story 015), B2 (Story 012),
B3 (Story 013), B4 (Story 014), B5 (Story 015). Sprint 12 must close
before this Sprint 13 candidate lands. The factory migration is a
**second-pass refactor** over the Sprint 12 outcomes: Sprint 12 chose
per-cluster paths (fixture expansion, assertion relocation, formula
update, spawn trim); Sprint 13's factory work generalises whatever
remains as the canonical pattern.

If Sprint 12 Story 015's umbrella-vs-split decision lands as "split",
B1 and B5 land under separate stories; this story's migration list
remains the same (4 fixtures + 1 sanity-check) because the factory
target is independent of the Cluster B closure path.

### GDD / ADR / TR trace

- **No GDD change**: this is test infrastructure.
- **ADR-002** (Client-Server Authority): the factory must not
  introduce any client-side optimism path; the factory mirrors the
  production client's read-only-over-S2C behaviour.
- **ADR-008** (Lightyear Channel Config): the factory calls
  `register_protocol(app)` exactly once, same as production.
- **ADR-021** (Presentation Layer Architecture): the factory
  registers `PresentationPlugin` and its sub-plugins in the canonical
  order documented in ADR-021.
- **TR registry**: no new TR. The factory references existing TRs
  (TR-PRES-001 for `PlayerEconomyView`, TR-NP-* for protocol).

### Engine

- **Engine**: Bevy 0.18 (Rust). The factory builds a Bevy `App` using
  `App::new()` + `App::add_plugins(DefaultPlugins)` + the canonical
  plugin set.
- **Lightyear**: 0.26 (Bevy 0.18 compatible). `register_protocol(app)`
  call is the same Lightyear 0.26 API used in production.

### Mandatory skills

- **`liv-bevy-018`** -- mandatory for the factory file and all
  migrated fixture files (Bevy `.rs` code).
- **`liv-bevy-lightyear`** -- mandatory for the `register_protocol`
  call within the factory and for any fixture that exercises a
  drain or sender.

### Control Manifest Rules (Test Infra scope)

- Required: The factory's plugin set is enumerated by name in an
  inline comment list that matches `client::main` / `server::main`
  line-for-line. The list is verifiable by grep.
- Required: The factory is deterministic; it does not depend on
  random seeds, system time, or external resources. Tests that need
  randomness inject a seeded `rand_chacha::ChaCha8Rng` after the
  factory call.
- Required: Fixtures migrated to the factory remove their ad-hoc
  plugin-set builder; if a fixture genuinely needs a narrower plugin
  set (e.g., a unit test for a single plugin's plumbing), it MUST
  add an inline rationale comment cross-referencing this story and
  explaining why production-app-factory is wrong for that case.
- Required: Each migration's test pass/fail behaviour is recorded in
  the evidence doc with pre/post `cargo test` output.
- Required: `docs/architecture/test-fixture-patterns.md` is updated
  to cite the factory as the default.
- Forbidden: Adding optimistic client-side authority to any
  migrated fixture.
- Forbidden: Modifying production code (`client/`, `server/`,
  `shared/`) beyond test-helper exports. The factory may export
  small helpers from `client/src/lib.rs` or `server/src/lib.rs`
  (scope-capped per AC8).
- Forbidden: Editing Sprint 12 story files' decisions or evidence
  paths. Sprint 12 outcomes are inputs to this story.

---

## Story Classification

**Story type**: Integration -- new test helper + targeted fixture
migration with semantic validation.

This is **NOT** a:

- Pure refactor story (semantic validation per fixture is required;
  some migrations may surface honest failures).
- Pure documentation story (real code lands).
- Sprint 12 expansion (this story is paperwork-only at authoring;
  implementation is Sprint 13 candidate).

---

## Acceptance Criteria

All criteria are independently checkable. Most are GIVEN/WHEN/THEN.

- [ ] **AC1 -- Factory file exists at the canonical path**:
  `tests/helpers/production_app_factory.rs` exists on the
  implementation branch. The file exports:
  - `pub fn production_client_app() -> App` (or equivalent canonical
    name).
  - `pub fn production_server_app() -> App` (or equivalent canonical
    name).
  - Optional small helpers for common setup (deterministic-RNG
    injection, fixture session entry).

- [ ] **AC2 -- Plugin set matches `client::main` line-for-line**:
  GIVEN `production_client_app()`, WHEN its plugin registration
  block is compared to `client::main::main()`'s plugin block, THEN
  the two are line-for-line identical modulo: (a) the inline
  comment list naming each plugin, (b) any test-only environment
  guards (e.g., `if cfg!(test) { ... }`). The factory is allowed to
  omit window-creation plugins (`bevy_winit`) IF and ONLY IF the
  omission is documented inline with a rationale.

- [ ] **AC3 -- Plugin set matches `server::main` line-for-line**:
  Same as AC2 for the server side.

- [ ] **AC4 -- B1 fixture migrated**:
  `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
  uses `production_client_app()` (post-Sprint-12 Story 015 outcome)
  and the test passes. The original `app_with_board_rendering()`
  helper is deleted OR is reduced to a thin wrapper over the factory.

- [ ] **AC5 -- B2 fixture migrated**:
  `tests/integration/board_rendering/snapshot_spawn_test.rs` uses
  `production_client_app()` (post-Sprint-12 Story 012 outcome) and
  the test passes. The original `app_in_session()` helper is deleted
  OR is reduced to a thin wrapper over the factory.

- [ ] **AC6 -- lobby_app fixture migrated**:
  `tests/integration/playable_client/native_operator_controls_test.rs`
  `lobby_app()` is replaced with `production_client_app()` +
  `enter_in_session_via_fixture()` + any lobby-state setup. The
  test passes (post-Sprint-12 Story 013 outcome).

- [ ] **AC7 -- shop_app fixture migrated**:
  `tests/integration/playable_client/native_operator_controls_test.rs`
  `shop_app()` is replaced with `production_client_app()` +
  shop-state setup. The test passes; the entity-count assertion
  matches production reality (post-Sprint-12 Story 015 Path B5
  outcome).

- [ ] **AC8 -- Production code touched minimally**: GIVEN the diff
  in `client/src/`, `server/src/`, `shared/src/`, WHEN inspected,
  THEN: (a) any new `pub` exports are scope-capped to test-helper
  visibility (e.g., a `pub fn build_production_app()` extracted from
  `client::main::main()` so the factory can call it); (b) no
  functional behaviour change lands; (c) each export carries an
  inline rationale comment cross-referencing this story.

- [ ] **AC9 -- `docs/architecture/test-fixture-patterns.md`
  updated**: The doc cites `tests/helpers/production_app_factory.rs`
  as the canonical default; ad-hoc `MinimalPlugins`-based fixtures
  are documented as a narrow exception with required inline rationale
  comments.

- [ ] **AC10 -- Workspace test pass + ignored count behave
  predictably**: GIVEN `cargo test --workspace --tests --no-fail-fast`
  at the implementation commit, WHEN compared to the post-Sprint-12
  baseline, THEN no new `#[ignore]` markers are introduced; the
  migrated tests pass; any previously-passing test continues to pass.

- [ ] **AC11 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed for any client-side
  mutation of authoritative state outside the shared phase sink,
  snapshot drainers, and S2C consumers, THEN no such mutation is
  present. ADR-002 binding. *Evidence*: text search for "no
  optimistic" in the evidence document.

- [ ] **AC12 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`, and
  `production/qa/qa-plan-sprint-12.md` are diffed, THEN none of them
  are modified under this story. Sprint 11 / Sprint 10 dispositions
  unchanged.

- [ ] **AC13 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-fixture-factory-evidence.md`
  (NEW; populated by the implementation prompt). Records pre/post
  test output per migration, factory plugin-list grep evidence,
  no-claim restatement, cross-link to PROMPT 803 §3 DC-7 + §4 Lane D
  + Sprint 12 Story 012/013/015 outcomes.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `tests/helpers/production_app_factory.rs` | NEW. Canonical factory exporting `production_client_app()` and `production_server_app()`. |
| `tests/helpers/mod.rs` | Updated to register the new module. |
| `tests/integration/board_rendering/ghost_preview_bridge_test.rs` | Migrated to use factory. |
| `tests/integration/board_rendering/snapshot_spawn_test.rs` | Migrated to use factory. |
| `tests/integration/playable_client/native_operator_controls_test.rs` | `lobby_app`, `shop_app`, `hand_app` migrated. |
| `client/src/lib.rs` (or `client/src/main.rs` extraction) | OPTIONAL: extract the plugin-composition block into a reusable `pub fn build_production_client_app() -> App` callable by both `main()` and the factory. Scope-capped to test-helper export. |
| `server/src/lib.rs` (or equivalent) | OPTIONAL: same for server. |
| `docs/architecture/test-fixture-patterns.md` | Updated to cite the factory as canonical default. |
| `production/qa/evidence/sprint-13-fixture-factory-evidence.md` | NEW evidence document per AC13. |
| This story file | Status updates per /story-readiness or /story-done if/when Sprint 13 activates. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code touched
  (factory + fixture migrations).
- **`liv-bevy-lightyear`** -- mandatory for the `register_protocol`
  call inside the factory and for any fixture that exercises drains.

---

## Evidence Path

`production/qa/evidence/sprint-13-fixture-factory-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Factory plugin-list verbatim transcript (output of `cargo expand`
  or `grep`-based comparison of `production_client_app()` vs
  `client::main::main()`).
- Per-migration pre/post test pass/fail output.
- Sprint 12 outcome cross-link per migration (Story 012/013/015 path
  chosen + impact on this migration).
- Diff summary (file paths + line counts).
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no optimistic client-side authority").
- Cross-link to PROMPT 803 §3 DC-7 + DC-8 + §4 Lane D.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test ghost_preview_bridge -- --nocapture`
- `cargo test -p client --test snapshot_spawn -- --nocapture`
- `cargo test -p client --test native_operator_controls -- --nocapture`
- `cargo test --workspace --tests --no-fail-fast 2>&1 | grep -E "ignored|FAILED|passed"`
- `git diff <pre-impl-sha>..<impl-sha> -- 'tests/**' 'client/src/**' 'server/src/**' 'docs/architecture/test-fixture-patterns.md'`
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- **Editing Sprint 12 Must Have story files' decisions or evidence
  paths**. Sprint 12 outcomes are inputs to this story.
- **Migrating fixtures beyond the 5 named** (B1, B2, lobby_app,
  shop_app, hand_app sanity-check). Other ad-hoc fixtures in the
  workspace are scoped to a Sprint 14 follow-on.
- **Adding new test cases** beyond the migrated tests' existing
  assertions. The factory migration is structural, not behavioural;
  new tests for new behaviour land in their own stories.
- **Changing channel bindings or protocol shapes**. ADR-008 binding.
- **Adding optimistic client-side authority**. ADR-002 binding.
- **Sprint 13 activation**. No `production/sprint-status.yaml` /
  `production/stage.txt` / `production/sprints/sprint-12.md` /
  `production/sprints/sprint-13.md` modification under this story.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  or any carried Sprint 10 / Sprint 11 / Sprint 12 condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Depends on Sprint 12 close-out**: This story's migration list
  references Sprint 12 Must Have outcomes (Story 012 Path A vs B,
  Story 013 production fix, Story 014 Path A vs B, Story 015
  umbrella vs split + per-sub-disposition). The implementation
  prompt MUST run after Sprint 12 close-out lands; running in
  parallel risks colliding with Sprint 12's fixture/file edits.
- **Touches the same fixture files as Sprint 12 Must Have rows**:
  - `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
    -- Sprint 12 Story 015 (B1) touches this file. Conflict mitigated
    by sequencing (Sprint 12 closes first).
  - `tests/integration/board_rendering/snapshot_spawn_test.rs` --
    Sprint 12 Story 012 (B2) touches this file. Conflict mitigated
    by sequencing.
  - `tests/integration/playable_client/native_operator_controls_test.rs`
    -- Sprint 12 Story 013 (B3) touches this file. Conflict mitigated
    by sequencing.
  - `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs` --
    Sprint 12 Story 015 (B5) touches this file. Conflict mitigated
    by sequencing.
- **No Sprint 12 invasion**: this story's implementation MUST NOT
  land before Sprint 12 close-out unless the producer explicitly
  authorises a pull-forward via a separate prompt.
- **Coordinate with `S13-TWO-CLIENT-RUNTIME-HARNESS-001` (Story 017
  in this epic)**: the two-client harness may benefit from the
  factory for its server-app construction; if both Sprint 13
  candidate stories land in the same wave, the harness uses
  `production_server_app()` for setup.
- **No shared-status writer overlap**: `production/sprint-status.yaml`
  is not touched by this story.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. Sprint 12 reaches close-out (separate prompt).
2. Sprint 13 is planned via `/sprint-plan sprint-13` (separate prompt).
3. This story passes `/story-readiness` (separate prompt).
4. Sprint 13 `/qa-plan sprint` is authored (separate prompt).
5. `/dev-story story-016-fixture-factory.md` is dispatched (separate
   prompt).

Expected implementation flow:

1. **Wave 1 -- Factory authored**: Implement
   `tests/helpers/production_app_factory.rs` with plugin sets
   matching `client::main` and `server::main` line-for-line.
2. **Wave 2 -- Sanity-check migration (`hand_app`)**: migrate the
   already-correct `hand_app` fixture as a no-op semantic check;
   confirms factory compatibility.
3. **Wave 3 -- Per-fixture migration**: migrate B1, B2, lobby_app,
   shop_app one at a time; each migration commit records pre/post
   test output and rationale for any assertion update.
4. **Wave 4 -- Docs update**:
   `docs/architecture/test-fixture-patterns.md` updated.
5. **Wave 5 -- Evidence**: populate evidence file.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Factory's plugin set drifts from production over time | Medium | Medium | AC2 + AC3 enforce line-for-line match; grep-based audit in evidence doc; consider Sprint 14 invariant test that asserts plugin-set parity. |
| Migrated fixtures fail honestly under production plugin set (e.g., new producer system emits unexpected entities) | High | Low-Medium | Expected; assertion updates land per fixture with rationale entry in evidence doc. |
| `DefaultPlugins` brings up a window in headless CI | Medium | High | Factory omits `bevy_winit` or uses `MinimalPlugins` for the window subsystem with an inline rationale comment. AC2 documents the exception. |
| Lobby.rs collision with Sprint 12 Story 013 | High | High | Sequence: Sprint 12 closes first. This story's implementation prompt is gated on Sprint 12 close-out. |
| Sprint 12 Story 015's umbrella-vs-split outcome changes the migration list | Medium | Low | The factory target is independent of the closure path; the migration list remains 4 fixtures + 1 sanity-check. |
| Sprint 13 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate; this story stays `Draft` until activation. |

---

## Verification (orchestrator-side, before worker dispatch)

- `production/sprint-status.yaml` `sprint:` field reads `13` after
  Sprint 13 activation; Sprint 12 close-out has landed.
- All Sprint 12 Must Have rows are `done`.
- `production/stage.txt` reads `Polish` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for the Canonical Production-Faithful Test App Factory
  per PROMPT 803 §3 DC-7 / §4 Lane D / §5 Must row 5. Sprint 12 is
  `active` (PROMPT 798) and is not modified by this authoring run.
  No code changes, no smoke / gate / QA / `/dev-story` / `/story-done` /
  `/story-readiness` / `/qa-plan` run. Source-of-truth at authoring:
  `origin/main@b5eef0d`. Worker branch:
  `work/s13-runtime-hardening-story-authoring`. Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.
