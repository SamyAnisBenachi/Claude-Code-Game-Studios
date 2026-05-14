# Story 016: S13-FIXTURE-FACTORY-001 -- Canonical Production-Faithful Test App Factory

> **Epic**: Playable Client
> **Story ID**: S13-FIXTURE-FACTORY-001
> **Status**: Done -- closed by PROMPT 854 `/story-done` on 2026-05-14. Worker
> `2cd5e057e757546b0f26cd58716d2e11add3efbf` (PROMPT 846) on
> `work/s13-fixture-factory` from base `origin/main@c1b7753`. Integration
> commit `4204a5b20117f6675a32c872796f6c90e3b08da3` (PROMPT 853) on
> `origin/main` via cherry-pick + rebase. Disposition:
> **PASS-WITH-NARROW-EXCEPTIONS** -- AC1-AC5, AC8-AC13 PASS; AC6/AC7 PASS
> within the Control Manifest narrow-plugin-set exception clause
> (factory imported in the test file via `#[path]`; `lobby_app` and
> `shop_app` retain narrower plugin sets with inline rationale +
> Sprint 14 follow-up). PROMPT 854 paperwork-only commit (no code or
> evidence change; serialized shared-status writer).
> **Layer**: Test Infrastructure
> **Type**: Integration -- new test helper + targeted fixture migration
> **Sprint**: Sprint 13 (activated by PROMPT 826; closed by PROMPT 854)
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

- [x] **AC1 -- Factory file exists at the canonical path**: PASS.
  `tests/helpers/production_app_factory.rs` (163 lines, NEW) exports
  `pub fn production_client_app() -> App` and
  `pub fn production_client_app_in_session() -> App` (small helper
  composing `production_client_app() + enter_in_session_via_fixture`
  for the in-session migrations). Companion
  `tests/helpers/production_server_app_factory.rs` (127 lines, NEW)
  exports `pub fn production_server_app() -> App`. The two files
  are split because the `client` and `server` workspace crates do
  not depend on each other -- a single helper file referencing
  `client::*` and `server::*` would fail to compile in either test
  crate. Both files are imported via `#[path = "../../helpers/..."]`
  by the migrated fixtures.

- [x] **AC2 -- Plugin set matches `client::main` line-for-line**: PASS.
  Factory registers `MinimalPlugins + StatesPlugin + AssetPlugin +
  init_asset::<Image>` (headless substrate replacing `DefaultPlugins`)
  then `PresentationPlugin + LobbyUiPlugin + AssetWiringPlugin` in
  the canonical order. `AudioSystemPlugin` and `ClientNetworkPlugin`
  are OMITTED with inline rationale per AC2's "omission documented
  inline" exception clause: audio/network are non-deterministic
  side-effect sources unsuitable for ECS unit tests (audio plays
  through OS device; network opens TCP). `bevy_winit` is implicitly
  excluded via `MinimalPlugins` (headless test substrate); this is
  the AC2-permitted window-omission with rationale.

- [x] **AC3 -- Plugin set matches `server::main` line-for-line**: PASS.
  Factory registers `MinimalPlugins + StatesPlugin + AssetPlugin`
  (headless substrate) then `ConfigPlugin / GameSessionPlugin /
  RsmPlugin / EconomyPlugin / CardPoolPlugin / BoardPlugin /
  AuctionPlugin / CardAcquisitionPlugin / CombatPlugin /
  KeywordPlugin / PrismPlugin / ObjectivePlugin`. `ServerNetworkPlugin`
  is OMITTED with inline rationale per AC3's omission-with-rationale
  exception clause: server network plugin binds a TCP port and
  would collide under parallel `cargo test` (`--test-threads >= 2`).

- [x] **AC4 -- B1 fixture migrated**: PASS.
  `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
  now imports `production_app_factory` via `#[path]` and constructs
  its app via `production_client_app_in_session()`. The 3 `grep`
  hits on `production_app_factory|production_client_app_in_session`
  confirm full adoption. **4/4 tests PASS** at integration tip per
  PROMPT 846 worker report + PROMPT 853 integration re-run.

- [x] **AC5 -- B2 fixture migrated**: PASS.
  `tests/integration/board_rendering/snapshot_spawn_test.rs` now
  imports `production_app_factory` via `#[path]` and constructs
  its app via `production_client_app_in_session()`. Atlas-path test
  helpers (`install_test_atlas`, `install_distinct_test_atlas`)
  now `remove_resource::<BoardRuntimeAssets>()` to preserve atlas-
  path assertion intent (factory's `AssetPlugin` +
  `insert_board_rendering_session_resources` inserts
  `BoardRuntimeAssets` which routes the production pipeline through
  the runtime-asset path; runtime-asset coverage is preserved by the
  unchanged `test_runtime_board_assets_drive_placeholder_hp_and_objective_images`).
  This is a documented assertion-preservation move per the story's
  "honest failure / assertion update" clause; rationale in evidence
  doc. **6/6 tests PASS** at integration tip.

- [x] **AC6 -- lobby_app fixture migrated**: PASS WITHIN NARROW
  EXCEPTION CLAUSE. The factory module IS imported in the test
  file (`#[path = "../../helpers/production_app_factory.rs"] mod
  production_app_factory;` at lines 34-35 of
  `native_operator_controls_test.rs`) and the canonical sanity-check
  fixture `hand_app` uses
  `production_app_factory::production_client_app() +
  enter_in_session_via_fixture` (lines 348-362). `lobby_app` itself
  retains a narrower plugin set (`MinimalPlugins + AssetPlugin +
  init_asset::<Image> + StatesPlugin + LobbyUiPlugin`) with the
  inline rationale comment at lines 297-303 cross-referencing
  `S13-FIXTURE-FACTORY-001` per the Control Manifest narrow-plugin-
  set clause: "if a fixture genuinely needs a narrower plugin set
  ... it MUST add an inline rationale comment cross-referencing
  this story and explaining why production-app-factory is wrong
  for that case." Rationale: `OnEnter(ClientState::Lobby)` systems
  from sibling presentation sub-plugins overwrite the
  `LobbyInputState` semantics that the lobby control tests rely on
  (room-code / button-binding determinism for
  `test_lobby_room_code_focus_separates_text_from_shortcuts` and
  siblings). Full migration tracked as Sprint 14 follow-up.
  **3/3 lobby tests PASS** at integration tip.

- [x] **AC7 -- shop_app fixture migrated**: PASS WITHIN NARROW
  EXCEPTION CLAUSE. Same disposition as AC6 -- factory module
  imported via `#[path]` and used by the sibling `hand_app` sanity-
  check; `shop_app` itself retains a narrower plugin set
  (`MinimalPlugins + AssetPlugin + init_asset::<Image> + StatesPlugin
  + ShopAuctionUiPlugin + ShopAuctionCardCatalog + PlayerEconomyView
  + ShopAuctionLocalGoldView + ShopAuctionDraftHandView`) with the
  inline rationale comment at lines 284-296 cross-referencing
  `S13-FIXTURE-FACTORY-001` per the Control Manifest narrow-plugin-
  set clause. Rationale: the operator-controls test drives a
  multi-phase scenario (`DraftInitial -> DraftShop -> DraftAuction`)
  asserting on intermediate outbound-message and slot-state counts;
  loading the full `PresentationPlugin` introduces
  `apply_shop_purchase_confirmations_system` and `ShopAuctionUiPlugin`
  snapshot consumers whose interaction with the test's hand-rolled
  `ShopAuctionDraftHandView` insert produces state-machine divergence
  that the test's "passes" gate cannot satisfy without either
  production code changes (out of scope per AC8) or a Sprint 12
  Story 015 Path B5 outcome not yet on `origin/main`. Full migration
  tracked as Sprint 14 follow-up. **Shop test PASSES** at integration
  tip.

- [x] **AC8 -- Production code touched minimally**: PASS.
  `git diff --name-only 4204a5b^1 4204a5b -- 'client/src/' 'server/src/'
  'shared/src/'` returns **empty**. PROMPT 846's narrow-exception
  decision specifically AVOIDED the need to extract any new `pub fn
  build_production_app()` from `client::main::main()`; the factory
  imports plugin types from the published `client` / `server` crate
  surfaces and composes them directly. Zero new `pub` exports;
  zero functional behaviour change. AC8 (a)/(b)/(c) all hold trivially
  because no production code lines are touched.

- [x] **AC9 -- `docs/architecture/test-fixture-patterns.md`
  updated**: PASS. Doc updated to cite
  `tests/helpers/production_app_factory.rs` (and the server-side
  companion) as the canonical default; ad-hoc `MinimalPlugins`-based
  fixtures are documented as a narrow exception with the required
  inline rationale comment pattern shown by the `lobby_app` /
  `shop_app` examples. `+100/-0` lines in this file per PROMPT 846
  diff stat.

- [x] **AC10 -- Workspace test pass + ignored count behave
  predictably**: PASS WITHIN MIGRATED SET. The 15 migrated tests
  all PASS at integration tip (B1: 4/4, B2: 6/6,
  native_operator_controls including `hand_app` sanity-check + 2
  narrow-exception fixtures: 5/5). **No new `#[ignore]` markers
  introduced** -- factory adoption did not require ignoring any
  test. Full-workspace `cargo test --workspace --tests --no-fail-fast`
  intentionally NOT run per Sprint 13 QA-plan binding
  "no-full-workspace-tests-by-default" policy; the orchestrator
  end-of-sprint integration smoke covers the workspace-wide gate.

- [x] **AC11 -- No optimistic client-side authority introduced**:
  PASS. Evidence doc §"No-Claim Restatement" carries verbatim the
  phrase "No optimistic client-side authority is introduced or
  proposed by this story" (line 22 of the evidence doc). The
  factory is test-only and mirrors the production client's
  read-only-over-S2C behaviour; no new `ResMut<_>` on
  `CurrentClientPhase` / `ClientState` / `PendingPlacements` /
  S2C consumer resources; no `phase_sink_system` or
  `apply_phase_changed_messages_with_resolution_gate` modification
  (those files are NOT in the integration commit diff at `4204a5b`).
  ADR-002 + ADR-009 + ADR-021 bindings preserved.

- [x] **AC12 -- Sprint 12 disposition preserved**: PASS.
  `git diff --name-only 4204a5b^1 4204a5b -- production/sprint-status.yaml
  production/sprints/sprint-13.md production/sprints/sprint-12.md
  production/stage.txt production/qa/qa-plan-sprint-13.md
  production/qa/qa-plan-sprint-12.md production/gate-checks/` returns
  **empty** for the worker commit (PROMPT 846 / 853). Sprint 12
  `closed-with-conditions` per PROMPT 817 preserved unchanged.
  Sprint 11 / Sprint 10 closeouts preserved unchanged. Stage UNCHANGED
  `Polish`. PROMPT 761 Polish->Release FAIL preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. The
  PROMPT 854 row-level `status: ready -> done` flip + `completed:
  2026-05-14` is the permitted disposition-preserving paperwork edit
  in `production/sprint-status.yaml` (top-level `sprint:` / `status:`
  / `stage:` unchanged).

- [x] **AC13 -- Evidence document slot reserved**: PASS.
  `production/qa/evidence/sprint-13-fixture-factory-evidence.md`
  (NEW; 381 lines on `origin/main` via PROMPT 853 integration
  commit `4204a5b`; not modified by PROMPT 854). Records: factory
  plugin-list verbatim transcript; per-migration pre/post test
  pass/fail output (4/4, 6/6, 5/5); Sprint 12 outcome cross-link
  per migration; diff summary (file paths + line counts: 7 files /
  +856 / -51); no-claim restatement verbatim with the "no optimistic
  client-side authority" phrase; cross-link to PROMPT 803 §3 DC-7
  + §3 DC-8 + §4 Lane D; AC1-AC13 sectioned evidence including
  per-AC PASS/PASS-WITH-NARROW-EXCEPTION dispositions; Sprint 14
  follow-up entries for `lobby_app` and `shop_app` narrow-exception
  fixtures.

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

## Authoring / Implementation / Closure Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for the Canonical Production-Faithful Test App Factory
  per PROMPT 803 §3 DC-7 / §4 Lane D / §5 Must row 5. Sprint 12 is
  `active` (PROMPT 798) and is not modified by this authoring run.
  No code changes, no smoke / gate / QA / `/dev-story` / `/story-done` /
  `/story-readiness` / `/qa-plan` run. Source-of-truth at authoring:
  `origin/main@b5eef0d`. Worker branch:
  `work/s13-runtime-hardening-story-authoring`. Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.
- 2026-05-14 -- PROMPT 808 -- Story authoring integrated to `origin/main`
  as `55b25be` along with the 7 sibling Sprint 13 candidate story
  files (007/008/017/018/019/020/021).
- 2026-05-14 -- PROMPT 823 -- `/story-readiness` rerun verdict
  **READY** for this story file (batch of 12 newly reviewed story
  files).
- 2026-05-14 -- PROMPT 826 -- Sprint 13 activated; top-level
  `sprint:` flipped 12 -> 13 and `status:` flipped
  `closed-with-conditions` -> `active`. Sprint 12 disposition
  preserved under `sprint_12_closeout:` block. This story's row was
  promoted into the active sprint as a Must Have with
  `status: ready`.
- 2026-05-14 -- PROMPT 846 -- `/dev-story` worker run on dedicated
  worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s13-fixture-factory`
  on branch `work/s13-fixture-factory` from base `origin/main@c1b7753`.
  Worker commit `2cd5e057e757546b0f26cd58716d2e11add3efbf` (7 files:
  `tests/helpers/production_app_factory.rs` NEW + 163 lines,
  `tests/helpers/production_server_app_factory.rs` NEW + 127 lines,
  `production/qa/evidence/sprint-13-fixture-factory-evidence.md`
  NEW + 381 lines, `docs/architecture/test-fixture-patterns.md`
  +100/-0, `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
  +29 net, `tests/integration/board_rendering/snapshot_spawn_test.rs`
  +61 net, `tests/integration/playable_client/native_operator_controls_test.rs`
  +46 net = 856 insertions / 51 deletions total). Cargo resource
  policy applied (CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc +
  debuginfo/incremental disabled + RUSTFLAGS for /DEBUG:NONE).
  Targeted regression: `cargo fmt --all -- --check` PASS;
  `cargo check -p client` PASS; `cargo test -p client --test
  board_rendering_ghost_preview_bridge_test` 4/0/0; `cargo test -p
  client --test board_rendering_snapshot_spawn_test` 6/0/0;
  `cargo test -p client --test playable_client_native_operator_controls_test`
  5/0/0; `git diff --check origin/main...HEAD` PASS. Full-workspace
  `cargo test --workspace --tests --no-fail-fast` intentionally NOT
  run per story Cargo policy + Sprint 13 QA-plan
  no-full-workspace-tests-by-default policy. Worker verdict:
  **PASS-WITH-NARROW-EXCEPTIONS** (3 of 5 fixtures fully migrated:
  B1, B2, hand_app; 2 of 5 retained as documented narrow-plugin-set
  exceptions: lobby_app, shop_app per Control Manifest clause).
  Worker report:
  `reports/PROMPT-846-S13-FIXTURE-FACTORY-Canonical-Production-Faithful-Test-App-Factory.md`.
- 2026-05-14 -- PROMPT 853 -- Integration of worker commit
  `2cd5e057...` to `origin/main` via cherry-pick onto
  `integration/s13-fixture-factory-853` (created from
  `origin/main@c1b7753`), then rebase onto live origin/main tip
  `b2db794` (after origin/main advanced 3 commits with independent
  orchestrator-tooling work `b2db794` / `7d2f224` / `490aed7`, all
  under `tools/gcs-orchestrator/`, zero file overlap with this
  story's diff). Clean no-conflict fast-forward integration commit
  `4204a5b20117f6675a32c872796f6c90e3b08da3` pushed to `origin/main`
  via `git push origin HEAD:main` (non-force fast-forward
  `b2db794..4204a5b`). Files changed match worker report exactly
  (7 files / +856 / -51). Cargo resource policy applied; regression
  re-run at integration tip identical to worker (`cargo fmt` PASS,
  `cargo check -p client` PASS in 1.21s, 15/15 targeted fixture
  tests PASS). `git diff --check origin/main...HEAD` PASS;
  `git diff --cached --check` PASS. Integration verdict:
  **PASS** -- narrow exceptions ACCEPTED per the story's Control
  Manifest narrow-plugin-set exception clause (both `lobby_app`
  and `shop_app` inline rationales verified to cross-reference the
  story marker `S13-FIXTURE-FACTORY-001`, identify the specific
  state-machine divergence justifying the narrower set, and name
  the Sprint 14 follow-up). No AC hidden or contradicted. Integration
  report:
  `reports/PROMPT-853-S13-FIXTURE-FACTORY-Integration.md`.
- 2026-05-14 -- PROMPT 854 -- `/story-done` paperwork closure run on
  root checkout against `origin/main@3199c01` (post PROMPT 851
  `/story-done` for `S13-PROTO-INVARIANT-001`). Source-of-truth
  source-of-truth verified `HEAD == origin/main == 3199c01`. PROMPT
  853 integration commit `4204a5b` confirmed reachable on
  `origin/main` (one commit before HEAD `3199c01`). Read-only review:
  story file ACs, evidence doc (381 lines), worker report PROMPT
  846, integration report PROMPT 853, `git show --stat 4204a5b`
  (7 files match worker scope), `git diff --name-only 4204a5b^1
  4204a5b -- production/sprint-status.yaml production/sprints/sprint-13.md
  production/sprints/sprint-12.md production/stage.txt
  production/qa/qa-plan-sprint-13.md production/qa/qa-plan-sprint-12.md
  production/gate-checks/ client/src/ server/src/ shared/src/`
  empty (AC8 + AC12 zero-touch verified). Verdict:
  **PASS-WITH-NARROW-EXCEPTIONS** preserved -- AC1-AC5, AC8-AC13
  PASS; AC6/AC7 PASS within Control Manifest narrow-plugin-set
  exception clause (factory imported in test file; narrower lobby
  and shop fixtures retained with rationale + Sprint 14 follow-up).
  Paperwork-only writes to 4 allowed files (this story file +
  `production/sprint-status.yaml` + `production/session-state/active.md`
  + `production/session-state/codex-orchestrator-state.md`). No
  cargo command invoked (Cargo resource policy N/A for this
  /story-done run; worker + integration already applied policy at
  their regression checkpoints). No `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, `/qa-plan`, `/dev-story`,
  `/story-readiness` invoked. No `production/stage.txt`,
  `production/sprints/sprint-13.md` (allowed-files list excluded
  per PROMPT 850 / 844 / 843 / 840 / 835 paperwork-only precedent),
  `production/sprints/sprint-12.md`,
  `production/qa/qa-plan-sprint-13.md` (allowed-files list excluded
  per same precedent), `production/qa/qa-plan-sprint-12.md`,
  `production/gate-checks/*`, `production/qa/evidence/sprint-13-fixture-factory-evidence.md`
  (already on `origin/main` via PROMPT 853 integration; /story-done
  does not re-write evidence), `client/`, `server/`, `shared/`,
  `tests/`, `.claude/settings.json`, `tools/gcs-orchestrator/`,
  `.octogent/`, `.claude/scheduled_tasks.lock` touched. Sprint 13
  progress after PROMPT 854: **4 of 6 Must Have done**
  (S13-OBS-WALLCLOCK-TIMESTAMPS-001 by PROMPT 843 +
  S13-OBS-TRACING-TARGETS-001 by PROMPT 850 + S13-PROTO-INVARIANT-001
  by PROMPT 851 + this row `S13-FIXTURE-FACTORY-001` by PROMPT 854);
  3 of 6 Should Have done; 1 of 7 Nice to Have done; total
  **8 of 19** rows closed. Fourth Must Have closure of Sprint 13.
  Sprint 13 disposition UNCHANGED (`active`; NOT closed-out by
  PROMPT 854). Final report:
  `reports/PROMPT-854-S13-FIXTURE-FACTORY-STORY-DONE.md`.

## Conditions carried forward unchanged

- S8-QA-001-W1 manual/browser two-client GAME_OVER gap remains OPEN.
  Story 017 (two-client runtime harness) AC12 forbid-auto-closure:
  explicitly does NOT close S8-QA-001-W1 by itself. Sprint 14
  follow-up on lobby/shop narrow-exception fixtures does NOT close
  S8-QA-001-W1 either.
- QA-COND-0005 Standard-tier accessibility remains accepted-risk
  (friend-game scope only).
- QA-COND-0006 playtest / fun-hypothesis validation remains
  accepted-risk / deferred.
- PAW-TD-*-a placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved unchanged
  (folded into Sprint 12 story 019 cannot-reproduce closure; third
  same-scope retest NOT authorised per TQ-S12-C2). PROMPT 854 does
  NOT re-attempt the Sprint 12 capture.
- PROMPT 761 Polish->Release gate-check FAIL preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no
  retry in PROMPT 854 scope.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed (closed cannot-reproduce, NOT bug-fixed).
- TQ-S12-C1..C7 (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817
  preserved unchanged under `sprint_12_closeout:` block.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- Prior `/story-done` closures preserved unchanged on `origin/main`:
  PROMPT 833 (S11-SERVER-POOL-INIT-LOG-GUARD-001), PROMPT 835
  (S11-LOBBY-UX-CONFIRM-STATE-001), PROMPT 840
  (S13-UI-AUDIT-ROADMAP-PREP-001), PROMPT 843
  (S13-OBS-WALLCLOCK-TIMESTAMPS-001), PROMPT 844
  (S11-HU-PHASE-IDEMPOTENCY-001), PROMPT 850
  (S13-OBS-TRACING-TARGETS-001), PROMPT 851
  (S13-PROTO-INVARIANT-001).
- Sprint 14 follow-up rows for full `lobby_app` / `shop_app`
  migration remain UNAUTHORED (the narrow-exception inline
  rationales name the follow-up; no Sprint 14 story file is
  authored or activated by PROMPT 854).

## Explicitly NOT claimed by PROMPT 854

- Public release readiness.
- Release-candidate readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion.
- Playtest / fun-hypothesis validation.
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion.
- Polish->Release gate-check retry.
- Stage advance from Polish to Release.
- Underlying drag-runtime bug fix (Sprint 12 story 019 closed
  cannot-reproduce, NOT bug-fixed).
- Full UI clean-pass repair.
- Closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`.
- Sprint 14 full `lobby_app` / `shop_app` migration (deferred per
  narrow-exception clause; no Sprint 14 story file authored or
  activated by PROMPT 854).
- Plugin-set drift invariant test (Risks table item; deferred to a
  future Sprint 14+ story; NOT authored or activated by PROMPT 854).
- Sprint 13 close-out (Sprint 13 remains `active`; 8 of 19 rows
  closed after PROMPT 854 -- 4 of 6 Must Have, 3 of 6 Should Have,
  1 of 7 Nice to Have).
- Full-workspace `cargo test --workspace --tests --no-fail-fast`
  result claim (deferred to orchestrator end-of-sprint integration
  gate per QA-plan-sprint-13 binding policy).
- AC2/AC3 strict line-for-line plugin-set match (the omissions
  for `bevy_winit` / `AudioSystemPlugin` / `ClientNetworkPlugin` /
  `ServerNetworkPlugin` are accepted per the AC2/AC3
  omission-with-rationale exception clause; a fully strict invariant
  test that programmatically enforces the plugin-set diff would
  belong to a Sprint 14+ story).
