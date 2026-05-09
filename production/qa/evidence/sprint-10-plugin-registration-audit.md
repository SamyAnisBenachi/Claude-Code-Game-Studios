# Sprint 10 Plugin-Registration Audit (Server + Client)

> **Status**: Pre-stage audit (PROMPT 563). Sprint 10 is **not** activated by
> this document. S10-TD-002 is **not** marked done. No sprint condition is
> closed by this audit. Source-of-truth audit-only artefact.
>
> **Author**: Codex implementation worker (PROMPT 563).
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\plugin-registration-audit-pre-stage`
> **Branch**: `work/plugin-registration-audit-pre-stage`
> **Audit base commit**: `710d305` (`origin/main` HEAD at audit time).
> **Load-bearing breakthrough commit**: `d7211f1` (PROMPT 545 — wired
> `CardPoolPlugin` and `KeywordPlugin`); verified present on `main` and verified
> to add the two `add_plugins(...)` calls in `server/src/main.rs`.

## Summary

The DRAFT_INITIAL silent failure was caused by `CardPoolPlugin` +
`KeywordPlugin` being defined in the server source tree but never
`.add_plugins(...)`-registered in `server/src/main.rs`. Since `cargo check`
and per-system unit tests cannot detect that gap, this audit enumerates every
`pub struct *Plugin` in the server and client crates and diffs them against
the actual `add_plugins(...)` registration tree (including transitive
registrations from parent plugins like `PresentationPlugin` /
`ServerNetworkPlugin`).

**Server**: 14 plugins defined / 14 registered. **Zero defined-but-not-
registered plugins remain in the server binary.** The PROMPT 545 fix appears
to be exhaustive for the server side.

**Client**: 15 plugins defined / 13 registered in the main game client +
1 registered in the `board_rendering_perf_harness` binary. **Two client
plugins are defined but never reached by the production main game client
binary** — both are flagged for user decision rather than auto-registered or
auto-deleted, because each would change runtime behaviour or lose
intentional code:

1. `AssetWiringPlugin` — **behaviour-change-flagged**. Registering it would
   start inserting the `PlaceholderAssets` resource on
   `OnEnter(ClientState::InSession)`, which board / hud / hand renderers
   currently fall back from when the resource is absent. This is the
   PAW-002..PAW-006 wiring that Sprint 10 S10-PAW-001 is scoped to close out.
2. `BoardWasmPerfHarnessPlugin` — **deletion-candidate-flagged**. Verbatim
   duplicate of `BoardRenderingPerfHarnessPlugin` (calls the same
   `add_board_rendering_perf_harness(app)` body) but never referenced by any
   binary or test. No `[[bin]]` entry in `client/Cargo.toml` consumes it.

This audit also surfaces an **E2E test gap** (Phase 4) which is the structural
reason DRAFT_INITIAL stayed silent for months: there is no test in the
workspace that compiles the production registration path of `server/src/main.rs`
or `client/src/main.rs`. Every test that builds a server App hand-rolls a
custom plugin tuple, so divergence between the production binary's
`add_plugins(...)` list and what tests exercise is invisible. Recommended
follow-up tech-debt item is at the end of this document.

This audit does **not**:

- mark S10-TD-002 done
- activate Sprint 10
- close any Sprint 9, Sprint 10, or carry-over condition
- consume Sprint 10 capacity
- modify production runtime behaviour (no behaviour-changing registrations
  were applied; no plugin was deleted)

## Audit method

Per `production/session-state/codex-orchestrator-state.md` ("Critical
sanity-check pattern — plugin registration audit", 2026-05-09):

```
grep -rn "pub struct .*Plugin"  server/src/
grep -n  "add_plugins\|\.add_plugins"  server/src/main.rs
grep -rn "pub struct .*Plugin"  client/src/
grep -n  "add_plugins\|\.add_plugins"  client/src/main.rs
```

Plus a transitive sweep: each parent plugin's `build()` was inspected for
nested `add_plugins(...)` calls (e.g. `PresentationPlugin` registers eight
sub-plugins; `ServerNetworkPlugin` nests `EconomyNetworkPlugin`).

A plugin is treated as **registered** if any `add_plugins(...)` call in any
production binary path (transitively) reaches it. Standalone harness binaries
(e.g. `board_rendering_perf_harness`) count as production registration for
their target plugin, since each is a real `[[bin]]` entry in `Cargo.toml`.

## Phase 2 — Server enumeration

Source: `server/src/main.rs` `fn main()` lines 79–154 (audit base
`710d305`); plus the transitive `add_plugins(...)` inside
`server/src/network/mod.rs:23,29`.

| # | Plugin | Source file | Registered? | How registered | Classification |
|---|---|---|---|---|---|
| 1 | `ConfigPlugin` | `server/src/foundation/config.rs:532` | yes | `main.rs:108` direct | n/a |
| 2 | `GameSessionPlugin` | `server/src/core/session/plugin.rs:19` | yes | `main.rs:111` direct | n/a |
| 3 | `RsmPlugin` | `server/src/core/rsm/plugin.rs:21` | yes | `main.rs:112` direct | n/a |
| 4 | `EconomyPlugin` | `server/src/core/economy/plugin.rs:14` | yes | `main.rs:113` direct | n/a |
| 5 | `CardPoolPlugin` | `server/src/core/pool/plugin.rs:20` | yes | `main.rs:120` direct (PROMPT 545 fix) | n/a |
| 6 | `BoardPlugin` | `server/src/feature/board/plugin.rs:26` | yes | `main.rs:121` direct | n/a |
| 7 | `AuctionPlugin` | `server/src/feature/auction/plugin.rs:18` | yes | `main.rs:122` direct | n/a |
| 8 | `CardAcquisitionPlugin` | `server/src/feature/acquisition/plugin.rs:13` | yes | `main.rs:123` direct | n/a |
| 9 | `CombatPlugin` | `server/src/feature/combat/mod.rs:59` | yes | `main.rs:124` direct | n/a |
| 10 | `KeywordPlugin` | `server/src/feature/keyword/mod.rs:26` | yes | `main.rs:129` direct (PROMPT 545 fix) | n/a |
| 11 | `PrismPlugin` | `server/src/feature/prism/plugin.rs:19` | yes | `main.rs:130` direct | n/a |
| 12 | `ServerNetworkPlugin` | `server/src/network/mod.rs:19` | yes | `main.rs:133` direct | n/a |
| 13 | `ObjectivePlugin` | `server/src/feature/objective/plugin.rs:26` | yes | `main.rs:136` direct | n/a |
| 14 | `EconomyNetworkPlugin` | `server/src/network/economy_dispatch.rs:19` | yes | `network/mod.rs:29` transitive (inside `ServerNetworkPlugin::build`) | n/a |

**Server result**: 14 / 14 plugins registered. **Zero silent dead-plugin
paths in the server binary.** PROMPT 545 (`d7211f1`) closed the breach.

## Phase 3 — Client enumeration

Source: `client/src/main.rs` lines 14–37 (game client binary) plus the
transitive `add_plugins(...)` calls inside `PresentationPlugin::build`
(`client/src/presentation/mod.rs:68–75`). Standalone harness binaries are
listed as a separate column.

| # | Plugin | Source file | Registered in main client? | How registered | Classification |
|---|---|---|---|---|---|
| 1 | `AudioSystemPlugin` | `client/src/audio/mod.rs:10` | yes | `main.rs:33` direct | n/a |
| 2 | `ClientNetworkPlugin` | `client/src/network/mod.rs:12` | yes | `main.rs:34` direct | n/a |
| 3 | `PresentationPlugin` | `client/src/presentation/mod.rs:46` | yes | `main.rs:35` direct | n/a |
| 4 | `LobbyUiPlugin` | `client/src/ui/lobby.rs:18` | yes | `main.rs:36` direct | n/a |
| 5 | `CardAnimationsPlugin` | `client/src/card_animations/mod.rs:43` | yes | `presentation/mod.rs:68` transitive | n/a |
| 6 | `BoardRenderingPlugin` | `client/src/presentation/board_rendering.rs:863` | yes | `presentation/mod.rs:69` transitive | n/a |
| 7 | `HandUiPlugin` | `client/src/ui/hand/mod.rs:772` | yes | `presentation/mod.rs:70` transitive | n/a |
| 8 | `HudPlugin` | `client/src/ui/hud/mod.rs:266` | yes | `presentation/mod.rs:71` transitive | n/a |
| 9 | `ShopAuctionUiPlugin` | `client/src/ui/shop_auction/mod.rs:1047` | yes | `presentation/mod.rs:72` transitive | n/a |
| 10 | `ResultScreenPlugin` | `client/src/presentation/result_screen.rs:17` | yes | `presentation/mod.rs:73` transitive | n/a |
| 11 | `SettingsAccessibilityPlugin` | `client/src/ui/settings/mod.rs:256` | yes | `presentation/mod.rs:74` transitive | n/a |
| 12 | `PhotosensitivityWarningPlugin` | `client/src/ui/photosensitivity_warning.rs:7` | yes | `presentation/mod.rs:75` transitive | n/a |
| 13 | `BoardRenderingPerfHarnessPlugin` | `client/src/presentation/board_rendering/perf_harness.rs:43` | n/a (not a main-game plugin) | `board_rendering_perf_harness.rs:23` direct (in the `board_rendering_perf_harness` `[[bin]]` entry) | registered in its dedicated harness binary |
| 14 | **`AssetWiringPlugin`** | `client/src/asset_wiring.rs:332` | **NO** | not referenced by any `add_plugins(...)` call in any binary | **behaviour-change — flag for user decision** |
| 15 | **`BoardWasmPerfHarnessPlugin`** | `client/src/presentation/board_rendering/perf_harness.rs:51` | **NO** | not referenced by any `add_plugins(...)` call in any binary | **deletion-candidate — flag for user decision** |

**Client result**: 13 / 15 plugins registered in the main game client.
1 plugin (`BoardRenderingPerfHarnessPlugin`) is registered in its own
dedicated harness binary, which is the correct location for it. **Two
plugins are silently dead in every client binary** — see Phase 5 for the
classification details.

## Phase 4 — Existing-tests audit

### 4.1 Tests that build the **production** server registration path

**None.** There is no test under `tests/integration/` (or anywhere else in
the workspace) that compiles `server/src/main.rs::main()`'s plugin set.
`server/src/main.rs` exposes no `pub fn build_app(...)` that tests could call;
it is purely a binary entry point. Every server-side integration test that
needs an `App` hand-rolls its own plugin tuple. The most thorough example,
`tests/integration/playable_client/real_e2e_loop_test.rs`, defines its own
`build_server_app(...)` (line 1014) and a separate harder build at line 1086,
each adding a different subset of `RsmPlugin / GameSessionPlugin /
EconomyPlugin / CardPoolPlugin / CardAcquisitionPlugin / AuctionPlugin /
BoardPlugin / CombatPlugin / EconomyNetworkPlugin` directly — never via
`server::main`.

**Implication**: even the most thorough server-side E2E integration test in
the workspace would **not** have caught the missing `CardPoolPlugin` /
`KeywordPlugin` registration in `server/src/main.rs` at any point before
PROMPT 545. The test would have continued to pass while the production
binary silently lost an entire system family. This matches the PROMPT 545
finding written into `codex-orchestrator-state.md` (line 1847): *"this
category of bug is invisible to type checks and invisible to per-system tests
… only an E2E integration test that spawns the real App would catch it — and
we don't have one for the server boot path."*

### 4.2 Tests that build the **production** client registration path

**None.** `client/src/main.rs` is a binary entry point with no exported
`build_app(...)`. The harness binaries
(`hud_text_size_contrast_harness.rs`, `hand_ui_placement_staged_disclosure_harness.rs`,
`board_rendering_qa_cond_0007_replay_harness.rs`,
`shop_auction_bid_target_focus_harness.rs`,
`shop_auction_draft_initial_objective_overlay_harness.rs`, etc.) each
construct a custom plugin set tuned to one feature area; none mirror the
production game-client `main.rs` registration list.

### 4.3 Would existing tests catch each defined-but-not-registered plugin if it were registered?

| Plugin | Existing tests that would catch a regression in this plugin's runtime effect |
|---|---|
| `AssetWiringPlugin` | `tests/integration/presentation/asset_wiring_foundation_test.rs`, `hand_ui_asset_wiring_test.rs`, `shop_auction_asset_wiring_test.rs`, `board_asset_wiring_test.rs`, `hud_asset_wiring_test.rs`, `lobby_asset_wiring_test.rs` — six dedicated tests reference `PlaceholderAssets`. They build mini-Apps that include `AssetWiringPlugin` directly. They do **not** inspect `client/src/main.rs`'s registration list, so they would still pass even if `main.rs` continued to omit `AssetWiringPlugin`. They will pass once the plugin is registered in `main.rs` (visual evidence is the PAW-002..PAW-006 manual screenshot route).|
| `BoardWasmPerfHarnessPlugin` | Zero. The plugin is never referenced by any test or binary, so there is nothing to break. |

### 4.4 Recommended follow-up tech-debt item

**Recommended** (do not adopt in this audit; flagged for separate-prompt
review): expose `build_app(app: &mut App)` from `server/src/main.rs` and
`client/src/main.rs` as a library function (or an integration entry-point
crate path), then write a single E2E boot test that asserts every
`pub struct *Plugin` declared under `server/src/feature/*` /
`server/src/core/*` (and the equivalent client paths) is present in the App's
plugin registry after `build_app` runs. That test would have caught
DRAFT_INITIAL silently and would catch any future
`AssetWiringPlugin`-style regression. This belongs as its own story under
the playable-client epic and is not in scope for S10-TD-002.

## Phase 5 — Classification

### Server

No defined-but-not-registered plugins. No edits required.

### Client

#### `AssetWiringPlugin` — behaviour-change-flagged

- **Defined at**: `client/src/asset_wiring.rs:332`.
- **Plugin body** (`asset_wiring.rs:334-339`):
  - `add_systems(OnEnter(ClientState::InSession), insert_placeholder_assets)`
  - `add_systems(OnExit(ClientState::InSession), remove_placeholder_assets)`
  - The two systems insert / remove the `PlaceholderAssets` resource.
- **Consumers** (would change behaviour upon registration):
  - `client/src/presentation/board_rendering.rs:1468, 1710, 1742, 2283, 2292, 2309` — board unit / class sprite handle resolution falls back when `PlaceholderAssets` is `None`.
  - `client/src/ui/hud/mod.rs:440-447` — HUD figurine / objective dot asset resolution falls back when `PlaceholderAssets` is `None`. The fall-back is documented in code as "test contexts".
  - `client/src/ui/hand/mod.rs:1181, 1223, 1242, 1258, 2353` — hand UI card frame / portrait resolution similarly falls back when the resource is absent.
- **Impact of registering**: every `Option<Res<PlaceholderAssets>>` query that
  currently sees `None` in production would start returning `Some` for the
  duration of `ClientState::InSession`. Renderers would start binding the
  PAW-002..PAW-006 wired sprites instead of the placeholder fall-backs.
  This is exactly the behaviour the Sprint 10 S10-PAW-001 close-out batch
  expects to land. It is **not** a behaviour-neutral registration sweep.
- **Decision per Phase 5 rules**:
  > "If the plugin would change runtime behavior on registration (adds a
  > system that mutates state, registers a `MessageReceiver`, etc.), do
  > NOT auto-register — flag for user decision in the audit doc. Behavior
  > changes need an explicit story, not a silent registration sweep."
- **Action**: **flag for user decision**. Recommended path is the same
  shape as the PROMPT 545 fix: a single thin commit that adds
  `app.add_plugins(AssetWiringPlugin)` to `client/src/main.rs` (probably
  immediately after `LobbyUiPlugin` so the plugin is live before
  `OnEnter(ClientState::InSession)` fires), wired as part of the Sprint 10
  S10-PAW-001 PAW-002..PAW-006 close-out. The plugin is **not** a candidate
  for `#[allow(dead_code)]` because it is intended to be live; it is
  category-(a)-shaped, but registration is behaviour-changing and so the
  prompt requires user decision instead of an automatic Phase 6 edit.

#### `BoardWasmPerfHarnessPlugin` — deletion-candidate-flagged

- **Defined at**: `client/src/presentation/board_rendering/perf_harness.rs:51`.
- **Plugin body** (`perf_harness.rs:53-57`): identical body to
  `BoardRenderingPerfHarnessPlugin` (line 43) — both call
  `add_board_rendering_perf_harness(app)`.
- **References**: zero. The struct is declared (line 51), an
  `impl Plugin for ...` is provided (line 53), and that is the only contact
  the rest of the workspace has with it. There is no `[[bin]]` entry in
  `client/Cargo.toml` for a `board_wasm_perf_harness` binary; only
  `board_rendering_perf_harness` exists, and it uses
  `BoardRenderingPerfHarnessPlugin`.
- **Impact of registering in any current binary**: registering it alongside
  `BoardRenderingPerfHarnessPlugin` would call
  `add_board_rendering_perf_harness(app)` twice in one App, which would
  duplicate every `add_systems(...)` and `init_resource::<...>()` call
  inside that helper — that is a behaviour change and a likely panic at
  startup.
- **Decision per Phase 5 rules**:
  > "(c) Genuinely unreachable — no current path uses it; deletion candidate.
  > Do NOT delete. Flag for user decision in the audit doc."
- **Action**: **flag for user decision**. Two reasonable resolutions, both
  out of scope for this audit:
  1. Remove the duplicate (delete `BoardWasmPerfHarnessPlugin` and let
     `BoardRenderingPerfHarnessPlugin` cover both native and WASM perf
     harness use). Loses no behaviour because the body is identical.
  2. Add a `board_wasm_perf_harness` `[[bin]]` entry in `client/Cargo.toml`
     pointing at a new `client/src/board_wasm_perf_harness.rs` and
     register `BoardWasmPerfHarnessPlugin` there — if a separate WASM
     entrypoint is genuinely planned (BOARD-012 mentioned in
     `perf_harness.rs:11`, but the existing `board_rendering_perf_harness`
     binary already uses
     `fit_canvas_to_parent / canvas: Some("#bevy")` so it's already the
     WASM target).
  - I lean toward (1), but the prompt scope forbids deletion in this audit.

## Phase 6 — Audit doc + safe edits

| Bucket | Count | Source edits applied? |
|---|---|---|
| Server defined plugins | 14 | n/a |
| Server registered (direct + transitive) | 14 | n/a |
| Server safe-registered this pass (category a) | 0 | n/a |
| Server intentional-dead annotated this pass (category b) | 0 | n/a |
| Server unreachable / deletion-candidate (category c, flagged) | 0 | n/a |
| Server behaviour-change-flagged | 0 | n/a |
| Client defined plugins | 15 | n/a |
| Client registered in main game client (direct + transitive) | 13 | n/a |
| Client registered in dedicated harness binary | 1 | n/a |
| Client safe-registered this pass (category a) | 0 | none |
| Client intentional-dead annotated this pass (category b) | 0 | none |
| Client unreachable / deletion-candidate (category c, flagged) | 1 | none — `BoardWasmPerfHarnessPlugin` flagged for user decision |
| Client behaviour-change-flagged | 1 | none — `AssetWiringPlugin` flagged for user decision |

**Source edits applied by this audit**: **none**. Both client findings fall
into "flag for user decision" categories per Phase 5 rules, so Phase 6
applies no `add_plugins(...)` registration and no `#[allow(dead_code)]`
annotation. The audit doc itself is the only artefact. This keeps the
audit pre-stage genuinely docs-only and preserves the PROMPT 563 invariant
that no Sprint 10 capacity is consumed and no runtime behaviour is changed.

## Phase 7 — Local verification

Because no source edits were applied, only documentation-level checks were
required. See the worker final report for the exact commands and their
results.

## Non-claims (explicit)

This audit does not, will not, and cannot:

- mark S10-TD-002 done in `production/sprint-status.yaml`
- activate Sprint 10 in `production/sprint-status.yaml` or any sprint file
- close S9-QA-001, S8-QA-001-W1, QA-COND-0005, or QA-COND-0006
- change any sprint, story, or epic-level status
- change any runtime behaviour in `server/` or `client/`
- delete any plugin, even `BoardWasmPerfHarnessPlugin`
- register any plugin that would change runtime behaviour, including
  `AssetWiringPlugin`
- substitute for the Sprint 10 S10-TD-002 closure work, which remains owned
  by the Sprint 10 producer and is gated on Sprint 10 activation.

S10-TD-002 close-out is expected to be a thin wrapper around this audit
doc, plus the user-decision outcomes for `AssetWiringPlugin` and
`BoardWasmPerfHarnessPlugin` (likely landed via PAW-002..PAW-006 close-out
for the former and a small dedicated commit for the latter).
