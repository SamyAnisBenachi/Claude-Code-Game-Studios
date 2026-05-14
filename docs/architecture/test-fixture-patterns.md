# Test Fixture Patterns

> Canonical patterns for building integration-test fixtures against the
> Lanes-and-Lies client. The default pattern (`production_client_app`) mirrors
> production plugin composition; the narrower `MinimalPlugins`-based pattern
> is reserved as a documented exception.
>
> **Stewardship**: Append new patterns here when you ship a fixture-helper that
> closes a recurring gap. Authored under Sprint 11 story
> `S11-TD-FIXTURE-HAND-UI-ONENTER-001`
> (`production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`),
> extended under Sprint 13 story `S13-FIXTURE-FACTORY-001`
> (`production/epics/playable-client/story-016-fixture-factory.md`) with the
> canonical production-faithful test app factory.

---

## Default pattern: `production_client_app` factory

### When to use it

This is the **canonical default** for any new client-side integration test.
Use the factory unless the test has a documented reason to use a narrower
plugin set (see the exception clause below).

### What goes wrong without it

Pre-factory, each fixture chose its own subset of production plugins. The
cluster B incidents (PROMPT 803 §3 DC-7 / §4 Lane D) all surfaced as green
tests that asserted observables (entity counts, message counts) while
silently skipping the producer system in the fixture's hand-picked plugin
subset. Common symptoms:

- `HandUiPlugin` missing → spawn-hand-ui never fires → tests assert on
  empty queries.
- `HudPlugin` missing → HUD producer systems absent → snapshot/entity
  counts diverge.
- `AssetWiringPlugin` missing → `PlaceholderAssets` never inserted →
  spawn pipelines early-return silently.

### Helper

```rust
// In your integration test:
#[path = "../../helpers/production_app_factory.rs"]
mod production_app_factory;
use production_app_factory::{production_client_app, production_client_app_in_session};
```

Signatures (`tests/helpers/production_app_factory.rs`):

```rust
pub fn production_client_app() -> App;            // app in ClientState::Lobby
pub fn production_client_app_in_session() -> App; // app in ClientState::InSession (factory + enter_in_session_via_fixture)
```

The factory mirrors `client::main::main()` plugin composition with three
documented test-only deviations (see file docs for full rationale):

1. `DefaultPlugins` is replaced with `MinimalPlugins + StatesPlugin +
   AssetPlugin + init_asset::<Image>` because `DefaultPlugins` requires
   `WinitPlugin` and `RenderPlugin` which need a window + GPU.
2. `AudioSystemPlugin` is omitted (no audio device under `cargo test`).
3. `ClientNetworkPlugin` is omitted (no WebSocket server; tests inject S2C
   messages directly into the ECS world via `world.write_message::<T>()`).

The remaining three production plugins (`PresentationPlugin`,
`LobbyUiPlugin`, `AssetWiringPlugin`) are added in the canonical order.

### Server-side companion

For server-side integration tests, use `production_server_app()` from
`tests/helpers/production_server_app_factory.rs`. The server factory mirrors
`server::main::main()` modulo omission of `ServerNetworkPlugin` (TCP listen
port collision under parallel `cargo test`).

### Migration list (S13-FIXTURE-FACTORY-001 wave)

| Fixture | File | Migration outcome |
|---|---|---|
| `app_with_board_rendering` | `tests/integration/board_rendering/ghost_preview_bridge_test.rs` | Migrated to `production_client_app_in_session`. |
| `app_in_session` | `tests/integration/board_rendering/snapshot_spawn_test.rs` | Migrated to `production_client_app_in_session`. Atlas-path test helpers (`install_test_atlas`, `install_distinct_test_atlas`) now remove `BoardRuntimeAssets` so the atlas-only rendering path is exercised; the runtime-asset path retains its dedicated test `test_runtime_board_assets_drive_placeholder_hp_and_objective_images`. |
| `hand_app` | `tests/integration/playable_client/native_operator_controls_test.rs` | Migrated as the no-op sanity check. |
| `lobby_app` | same file | **Narrow-exception**: retains MinimalPlugins + `LobbyUiPlugin` only. See inline rationale at the fixture call site (Sprint 14 follow-up). |
| `shop_app` | same file | **Narrow-exception**: retains MinimalPlugins + `ShopAuctionUiPlugin` only. See inline rationale at the fixture call site (Sprint 14 follow-up). |

---

## Narrow-plugin-set exception clause

Per the S13-FIXTURE-FACTORY-001 Control Manifest: a fixture that genuinely
needs a narrower plugin set (e.g., a unit test for a single plugin's
plumbing) **must** add an inline rationale comment cross-referencing
`production/epics/playable-client/story-016-fixture-factory.md` and explaining
why `production_client_app` is wrong for that case. The comment lives at the
fixture-builder call site, not in the test bodies.

The narrow-exception fixtures retained as of S13-FIXTURE-FACTORY-001
(`lobby_app`, `shop_app` in `native_operator_controls_test.rs`) all carry
such a rationale comment and are tracked for full migration under a Sprint 14
follow-up.

## Why this doc exists

`MinimalPlugins`-based partial-App fixtures are the cheapest way to write
ECS-level tests against `client/` UI systems — no window, no renderer, no asset
server. But these fixtures must explicitly satisfy every prerequisite that
production-side plugin composition would normally hand to the system under
test. When a prerequisite is missing, the system typically *silently skips*
rather than panicking, and the failure surfaces downstream as a cryptic
"entity not found" / "resource not found" panic in the assertion code.

This doc lists each recurring gap and the canonical helper that closes it, so
future fixture authors don't have to rediscover the cascade.

---

## Pattern: drive `OnEnter(ClientState::InSession)` end-to-end

### When you need it

Any `MinimalPlugins` fixture that:

- adds `HandUiPlugin` (or another plugin that registers a `spawn_*` system on
  `OnEnter(ClientState::InSession)`), AND
- needs the spawn to *actually* fire (e.g., the test asserts on
  `HandUiEntities` / `FanSlotIndex` / `HandCardFrame` / chrome children).

### What goes wrong without the helper

`spawn_hand_ui` (`client/src/ui/hand/mod.rs`) early-returns on
`Option<Res<PlaceholderAssets>>::None`. In production, `AssetWiringPlugin`'s
`insert_placeholder_assets` system runs on `OnEnter(InSession)` and inserts the
resource via `AssetServer`. `HandUiPlugin` schedules
`spawn_hand_ui.after(insert_placeholder_assets)` so the ordering is correct.

A `MinimalPlugins` fixture that omits `AssetPlugin` / `AssetServer` (and does
not insert `PlaceholderAssets` directly) sees `insert_placeholder_assets`
silently skip on missing `Res<AssetServer>` and `spawn_hand_ui` early-return.
The state transition completes "cleanly" and the test fails downstream when
the spawn-dependent queries find nothing.

### Helper

```rust
use client::asset_wiring::enter_in_session_via_fixture;
```

Signature:

```rust
pub fn enter_in_session_via_fixture(app: &mut bevy::prelude::App);
```

Behavior:

1. Inserts `placeholder_assets_for_tests()` into the world if absent.
2. Sets `NextState::<ClientState>::Pending(ClientState::InSession)`.
3. Runs `app.update()` twice — the first cycle applies the state transition
   and runs `OnEnter(InSession)` systems (which queue spawn commands); the
   second cycle flushes those deferred commands so downstream queries
   resolve in the same tick as the assertions.

### Pre-conditions the caller must satisfy

| Plugin / Resource | Why |
|---|---|
| `MinimalPlugins` | Time, Hierarchy, schedule infrastructure |
| `StatesPlugin` + `init_state::<ClientState>()` | `OnEnter(InSession)` schedule registration |
| `HandUiPlugin` (or the plugin that registers `spawn_hand_ui`) | The actual `OnEnter(InSession)` system under test |

The helper does *not* add these plugins itself — fixtures vary in which
adjacent plugins they bundle (e.g., `ShopAuctionUiPlugin`, `TweeningPlugin`)
and we don't want to force a one-size-fits-all bundle.

### Minimal example

```rust
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::enter_in_session_via_fixture;
use client::state::ClientState;
use client::ui::hand::{HandUiEntities, HandUiPlugin};

fn fixture() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(HandUiPlugin);
    enter_in_session_via_fixture(&mut app);
    app
}

#[test]
fn test_hand_ui_entities_exist_after_in_session_transition() {
    let app = fixture();
    let _entities = app.world().resource::<HandUiEntities>();
}
```

### Side effects and limits

- The helper flips `ClientState::Lobby -> ClientState::InSession`. It does not
  set a `RoundPhase`; callers that need the hand UI in
  `RoundPhase::Placement` (or any other phase) must set
  `CurrentClientPhase`/`ClientPhaseView` themselves and call `app.update()`
  once more so phase-transition systems observe the new phase.
- The helper inserts `placeholder_assets_for_tests()` — every `Handle<Image>`
  is `Handle::default()`. Tests that assert on a *non-default* image handle
  (PAW-002-e style assertions) need the
  `AssetPlugin + insert_placeholder_assets` pattern instead (see
  `tests/integration/presentation/hand_ui_asset_wiring_test.rs`).

### Related precedent

Mirrors the `placeholder_assets_for_tests()` precedent that closed Layer 3 of
the S10-TD-001 cascade (see
`production/epics/playable-client/story-009-test-fixture-cascade-fail-repair.md`).
This helper closes the residual gap that smoke retry-7 surfaced after Layers
1-3 landed.

---

## Pattern history

| Date | Story | Pattern |
|------|-------|---------|
| 2026-05-13 | `S11-TD-FIXTURE-HAND-UI-ONENTER-001` | `enter_in_session_via_fixture` |
| 2026-05-14 | `S13-FIXTURE-FACTORY-001` | `production_client_app` / `production_server_app` factories at `tests/helpers/` |
