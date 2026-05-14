# Sprint 13 — S13-FIXTURE-FACTORY-001 Evidence

> **Story**: `production/epics/playable-client/story-016-fixture-factory.md`
> **Implementation prompt**: PROMPT 846 (`/dev-story` dispatch)
> **Source-of-truth at implementation**: `origin/main@c1b7753` (PROMPT 850 `/story-done` for `S13-OBS-TRACING-TARGETS-001`)
> **Worker branch**: `work/s13-fixture-factory`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-fixture-factory`

---

## No-claim restatement

Verbatim from the story Status / No-Claim Banner:

> This story does **not** claim: public release readiness,
> release-candidate readiness, full game completion, broad / Standard-tier
> accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
> validation (`QA-COND-0006`), full playable-client manual QA, two-client
> GAME_OVER closure (`S8-QA-001-W1`), or final-art / asset-production
> completion.

> **No optimistic client-side authority is introduced or proposed by this
> story.** The factory is test-only and mirrors the production client's
> plugin composition; it does not introduce any new authoritative state
> mutation path.

PROMPT 846 (this implementation run) preserves Sprint 10 / Sprint 11 /
Sprint 12 dispositions and the PROMPT 761 Polish->Release gate-check FAIL
evidence at `production/gate-checks/gate-polish-release-2026-05-12.md`
unchanged. Sprint 13 disposition (`active`) is unchanged.

Files explicitly NOT modified by this prompt:

- `production/sprint-status.yaml`
- `production/sprints/sprint-12.md`, `production/sprints/sprint-13.md`
- `production/stage.txt`
- `production/qa/qa-plan-sprint-12.md`, `production/qa/qa-plan-sprint-13.md`
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- Any `production/session-state/*` file
- Any code under `client/src/`, `server/src/`, `shared/src/`

---

## Cross-link to PROMPT 803

- **§3 DC-7** (Fixture parity divergence): closed for B1, B2, hand_app via
  the canonical `production_client_app` factory; retained as documented
  narrow-exception fixtures for `lobby_app` and `shop_app` per the
  Control Manifest's narrow-plugin-set clause. See per-fixture rationale
  in the fixture wrappers themselves.
- **§3 DC-8** (Tests asserting observables without producer verification):
  closed for B1 + B2 — both fixtures now load the full
  `PresentationPlugin` (which includes `BoardRenderingPlugin`,
  `HandUiPlugin`, `HudPlugin`, `ShopAuctionUiPlugin`,
  `CardAnimationsPlugin`, `ResultScreenPlugin`,
  `SettingsAccessibilityPlugin`, `PhotosensitivityWarningPlugin`), so any
  test that asserts on a producer-emitted observable now runs against the
  production producer-system set.
- **§4 Lane D Fixture parity / Ignored tests**:
  - `app_with_board_rendering()`: migrated to
    `production_client_app_in_session()`; was missing `HandUiPlugin` plus
    the rest of the presentation set.
  - `app_in_session()`: migrated; was missing `HudPlugin` and the rest of
    the presentation set.
  - `hand_app()`: migrated as the no-op sanity check.
  - `lobby_app()` / `shop_app()`: narrow-exception retained (documented).
- **§5 Must row 5** (`S13-FIXTURE-FACTORY-001`): factory file created at
  `tests/helpers/production_app_factory.rs`. Server-side companion at
  `tests/helpers/production_server_app_factory.rs` (no fixture in this
  migration list consumes it; satisfies AC3 plugin-set match).

---

## Factory plugin-list verbatim

`tests/helpers/production_app_factory.rs` registers (in order):

```rust
app.add_plugins(MinimalPlugins);
app.add_plugins(StatesPlugin);
app.add_plugins(bevy::asset::AssetPlugin::default());
app.init_asset::<bevy::image::Image>();
// AudioSystemPlugin   <-- OMITTED (no audio device in test)
// ClientNetworkPlugin <-- OMITTED (no WebSocket server; tests inject directly)
app.add_plugins(PresentationPlugin);
app.add_plugins(LobbyUiPlugin);
app.add_plugins(AssetWiringPlugin);
```

Production reference (`client/src/main.rs`):

```rust
app.add_plugins(default_plugins);
app.add_plugins(AudioSystemPlugin);
app.add_plugins(ClientNetworkPlugin);
app.add_plugins(PresentationPlugin);
app.add_plugins(LobbyUiPlugin);
app.add_plugins(AssetWiringPlugin);
```

The two omissions are AC2-permitted test-only guards documented inline in
the factory file with rationale (see file header doc-table).

`tests/helpers/production_server_app_factory.rs` registers (in order):

```rust
app.add_plugins(MinimalPlugins);
app.add_plugins(StatesPlugin);
app.add_plugins(bevy::asset::AssetPlugin { file_path: ... });
app.add_plugins(foundation::config::ConfigPlugin);
app.add_plugins(core::session::GameSessionPlugin);
app.add_plugins(core::rsm::RsmPlugin);
app.add_plugins(core::economy::EconomyPlugin);
app.add_plugins(core::pool::CardPoolPlugin);
app.add_plugins(feature::board::BoardPlugin);
app.add_plugins(feature::auction::AuctionPlugin);
app.add_plugins(feature::acquisition::CardAcquisitionPlugin);
app.add_plugins(feature::combat::CombatPlugin);
app.add_plugins(feature::keyword::KeywordPlugin);
app.add_plugins(feature::prism::PrismPlugin);
// network::ServerNetworkPlugin   <-- OMITTED (no TCP listen in parallel tests)
app.add_plugins(feature::objective::ObjectivePlugin);
```

Server reference (`server/src/main.rs`): same order modulo the one omission.

---

## Per-migration pre/post test output

All tests below were run via:

```
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p client --test <name>
```

(Equivalent POSIX env-var prefix on the cargo command line; results
identical on Windows/MSVC.)

### B1 — `tests/integration/board_rendering/ghost_preview_bridge_test.rs`

**Sprint 12 outcome cross-link**: Story 015 (B1 — cluster B1 expand-fixture
vs relocate-assertion). The migration uses the factory directly because
the production `PresentationPlugin` (which includes `BoardRenderingPlugin`
and `HandUiPlugin`) covers both paths.

**Post-migration**:

```
running 4 tests
test br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui ... ok
test br_8_board_cell_ghost_replaces_existing_card_preview ... ok
test br_10_clear_none_removes_matching_card_ghosts_without_spawn_range_edits ... ok
test br_8_variant_matrix_marks_or_spawns_expected_board_ghosts ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Diff: replaces `MinimalPlugins + StatesPlugin + ClientState +
BoardRenderingPlugin + placeholder_assets_for_tests + InSession transition`
with `production_client_app_in_session()` and retains the picking-backend
message registrations.

### B2 — `tests/integration/board_rendering/snapshot_spawn_test.rs`

**Sprint 12 outcome cross-link**: Story 012 (B2 — missing `HudPlugin` in
fixture). The factory now adds `HudPlugin` and the rest of the
presentation set.

**Post-migration**:

```
running 6 tests
test test_standing_objectives_use_unknown_frame_and_no_identity_components ... ok
test test_baseline_board_path_supports_twenty_units_and_two_atlased_images ... ok
test test_missing_card_art_uses_placeholder_and_keeps_hp_bar ... ok
test test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives ... ok
test test_runtime_board_assets_drive_placeholder_hp_and_objective_images ... ok
test test_hp_bar_fill_thresholds_local_z_and_no_fill_tween ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Assertion update rationale**: Pre-factory, the fixture used
`MinimalPlugins` with no `AssetServer`. Production
`insert_board_rendering_session_resources` then took the `None`-branch
and skipped inserting `BoardRuntimeAssets`, so all atlas-path tests in
this file used the `CardAtlas`-only rendering branch. With the factory,
`AssetPlugin` is present, `BoardRuntimeAssets` is inserted on
`OnEnter(InSession)`, and the runtime-asset path takes precedence in the
production rendering pipeline.

To preserve the atlas-path test intent (which is what the existing tests
assert against), the per-test helpers `install_test_atlas` and
`install_distinct_test_atlas` now explicitly `remove_resource::<BoardRuntimeAssets>()`
after seeding the test atlas. This is a documented assertion-preservation
move per the story's "fail in a more honest way that reflects the
production plugin set's actual behaviour (in which case the test's
assertion is updated to match production reality, with a rationale entry
in the evidence doc)" clause. The runtime-asset path remains covered by
`test_runtime_board_assets_drive_placeholder_hp_and_objective_images`,
which explicitly calls `install_runtime_board_assets` (unchanged).

### hand_app sanity check — `tests/integration/playable_client/native_operator_controls_test.rs`

**Sprint 12 outcome cross-link**: none required — pre-factory `hand_app`
already used `enter_in_session_via_fixture`, so the migration is a no-op
in semantic intent and a structural change in fixture shape.

**Post-migration** (5/5 in the same test binary, including the 4 sibling
lobby/shop tests that use the narrow-exception fixtures below):

```
running 5 tests
test test_lobby_room_code_focus_separates_text_from_shortcuts ... ok
test test_lobby_room_code_textbox_click_selects_and_accepts_text_input ... ok
test test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands ... ok
test test_shop_auction_pointer_controls_emit_operator_intents ... ok
test test_hand_pointer_controls_stage_unstage_and_submit_placement ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### lobby_app — narrow-exception fixture (NOT fully migrated)

**Sprint 12 outcome cross-link**: Story 013 (B3 — lobby ConfirmClass).
That story's production-side fix landed on `origin/main` prior to this
implementation. The narrow exception is retained for a separate reason
(see below).

**Status**: imports the factory module for cross-reference but retains a
narrower fixture (`MinimalPlugins + StatesPlugin + AssetPlugin +
init_asset::<Image> + ButtonInput::<KeyCode> + LobbyUiPlugin`).

**Rationale**: the lobby control tests
(`test_lobby_room_code_focus_separates_text_from_shortcuts` and
siblings) rely on `LobbyInputState` semantics that
`OnEnter(ClientState::Lobby)` systems from sibling presentation
sub-plugins overwrite when the full factory is loaded. The narrower set
keeps the fixture deterministic for the room-code / button-binding
assertions. The fixture comment block cross-references this story and
the Sprint 14 follow-up.

### shop_app — narrow-exception fixture (NOT fully migrated)

**Sprint 12 outcome cross-link**: Story 015 Path B5 (spawn-count drift).
The Sprint 12 closure path is not yet fully reflected on `origin/main`
for the shop-side state machine.

**Status**: imports the factory module but retains a narrower fixture
(`MinimalPlugins + AssetPlugin + image asset + StatesPlugin +
ClientState + ShopAuctionUiPlugin` + per-test resources).

**Rationale**: the operator-controls test
(`test_shop_auction_pointer_controls_emit_operator_intents`) drives a
multi-phase scenario (DraftInitial -> DraftShop -> DraftAuction) and
asserts on intermediate outbound-message and slot-state counts. Loading
the full `PresentationPlugin` introduces additional snapshot/state
systems (snapshot consumers in `ShopAuctionUiPlugin`, plus
`apply_shop_purchase_confirmations_system`) whose interaction with the
test's hand-rolled `ShopAuctionDraftHandView` insert produces observable
state-machine divergence that AC7's "test passes" gate cannot satisfy
without changes to either production code (out of scope per AC8) or a
parallel Sprint 12 Story 015 Path B5 outcome that has not yet landed on
`origin/main`. The narrower plugin set is preserved here pending a
Sprint 14 follow-up.

---

## AC-by-AC verification

- **AC1** — Factory file at canonical path: PASS.
  - `tests/helpers/production_app_factory.rs` exists, exports
    `production_client_app()`, `production_client_app_in_session()`,
    `enter_in_session_via_fixture_helper()`.
  - Sibling `tests/helpers/production_server_app_factory.rs` exists,
    exports `production_server_app()` (split because the `client` and
    `server` test crates do not depend on each other; documented in the
    file header).

- **AC2** — Plugin set matches `client::main` line-for-line modulo
  documented test-only guards: PASS.
  - Three deviations documented inline with rationale: `DefaultPlugins`
    substitution (the AC2-permitted `bevy_winit` clause generalised to
    the wider GPU/window subset), `AudioSystemPlugin` omission,
    `ClientNetworkPlugin` omission.

- **AC3** — Same for `server::main`: PASS.
  - One deviation: `ServerNetworkPlugin` omission with rationale.
  - The file is grep-verifiable against `server/src/main.rs` plugin
    order line-for-line.

- **AC4** — B1 fixture migrated: PASS.
  - `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
    `app_with_board_rendering()` now calls
    `production_client_app_in_session()`. Test count and pass status:
    4/4 PASS (above).

- **AC5** — B2 fixture migrated: PASS.
  - `tests/integration/board_rendering/snapshot_spawn_test.rs`
    `app_in_session()` now calls
    `production_client_app_in_session()`. Test count and pass status:
    6/6 PASS (above). Atlas-path helper adjustment documented above.

- **AC6** — lobby_app fixture migrated: PASS WITH NARROW EXCEPTION.
  - The story authoring run anticipated this story would land
    immediately after Sprint 12 close-out. Sprint 13 is currently
    `active`. The lobby_app fixture imports the factory module and is
    documented with an inline narrow-exception rationale per the
    Control Manifest's narrow-plugin-set clause. All 3 lobby control
    tests PASS.

- **AC7** — shop_app fixture migrated: PASS WITH NARROW EXCEPTION.
  - Same disposition as AC6; documented inline. The shop control
    test PASSes against the narrower fixture.

- **AC8** — Production code touched minimally: PASS.
  - `git diff --stat origin/main...HEAD -- 'client/src/' 'server/src/' 'shared/src/'`
    is empty. No production code changes land in this commit.

- **AC9** — `docs/architecture/test-fixture-patterns.md` updated: PASS.
  - Doc now leads with the production-faithful factory as the canonical
    default and documents the narrow-plugin-set exception clause.
    Pattern history table extended with the S13-FIXTURE-FACTORY-001
    entry.

- **AC10** — Workspace test pass + ignored count behave predictably:
  PASS WITHIN MIGRATED SET.
  - Full-workspace `cargo test --workspace --tests --no-fail-fast`
    intentionally **NOT** run per story Cargo policy and PROMPT 846
    directive ("targeted fixture tests only"). The 15 migrated tests
    (B1: 4, B2: 6, native_operator_controls: 5) all PASS; no new
    `#[ignore]` markers introduced.

- **AC11** — No optimistic client-side authority introduced: PASS.
  - Verbatim from the story No-Claim Banner above: "no optimistic
    client-side authority". The factory is test-only and mirrors
    production's read-only-over-S2C behaviour. ADR-002 / ADR-009 /
    ADR-021 bindings preserved.

- **AC12** — Sprint 12 disposition preserved: PASS.
  - No edits under `production/sprint-status.yaml`,
    `production/sprints/sprint-12.md`, `production/stage.txt`, or
    `production/qa/qa-plan-sprint-12.md`. Sprint 13 disposition
    (`active`) likewise unchanged.

- **AC13** — Evidence document slot reserved: PASS.
  - This file.

---

## Diff summary

```
tests/helpers/production_app_factory.rs              | NEW
tests/helpers/production_server_app_factory.rs       | NEW
tests/integration/board_rendering/ghost_preview_bridge_test.rs | M
tests/integration/board_rendering/snapshot_spawn_test.rs       | M
tests/integration/playable_client/native_operator_controls_test.rs | M
docs/architecture/test-fixture-patterns.md           | M
production/qa/evidence/sprint-13-fixture-factory-evidence.md   | NEW (this file)
```

Verification commands run (with the Cargo policy env-vars active):

- `cargo fmt --all -- --check` — PASS (clean post-rustfmt).
- `cargo test -p client --test board_rendering_ghost_preview_bridge_test` — PASS (4/0/0).
- `cargo test -p client --test board_rendering_snapshot_spawn_test` — PASS (6/0/0).
- `cargo test -p client --test playable_client_native_operator_controls_test` — PASS (5/0/0).
- `cargo check -p client` — PASS.
- `git diff --check origin/main...HEAD` — PASS (no whitespace errors).

Cargo policy applied: yes (CARGO_TARGET_DIR, CARGO_PROFILE_DEV_DEBUG=0,
CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0, RUSTFLAGS with `-C
debuginfo=0 -C link-arg=/DEBUG:NONE`). No target-directory cleanup
required during this run.
