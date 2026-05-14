//! Canonical production-faithful test app factory.
//!
//! Closes PROMPT 803 §3 DC-7 (fixture parity divergence between MinimalPlugins
//! fixtures and the production `App`) and §3 DC-8 (tests asserting observables
//! without producer verification) by giving every client-side integration test
//! a single call site that mirrors `client::main::main()` plugin composition.
//!
//! Story: `S13-FIXTURE-FACTORY-001`
//! (`production/epics/playable-client/story-016-fixture-factory.md`).
//!
//! The server-side companion factory lives in
//! [`production_server_app_factory.rs`](./production_server_app_factory.rs).
//! It is split from this file because the client test crate (`client`) and the
//! server test crate (`server`) cannot share the same `use` block — neither
//! crate depends on the other, so a single file importing both would fail to
//! compile in either crate.
//!
//! ## Usage
//!
//! Client integration tests include this file via `#[path]` (same pattern as
//! `tests/test_helpers.rs`):
//!
//! ```ignore
//! #[path = "../../helpers/production_app_factory.rs"]
//! mod production_app_factory;
//! use production_app_factory::production_client_app;
//!
//! #[test]
//! fn my_test() {
//!     let mut app = production_client_app();
//!     // ... drive state, inject messages, assert ...
//! }
//! ```
//!
//! ## Plugin parity contract
//!
//! `production_client_app()` registers the same plugin set, in the same order,
//! as `client::main::main()`. Two categories of difference are permitted per
//! S13-FIXTURE-FACTORY-001 AC2:
//!
//! 1. Inline comments that name each plugin (this file documents them).
//! 2. Test-only environment guards (substitution of `DefaultPlugins` with a
//!    headless equivalent; omission of plugins that require external resources
//!    not available under `cargo test`).
//!
//! Each test-only deviation is documented inline at the call site with
//! rationale; the production order itself is preserved verbatim.
//!
//! ## Why a factory and not per-fixture builders
//!
//! Pre-factory, each fixture chose its own subset of production plugins. The
//! cluster B incidents (PROMPT 803 §4 Lane D) all surfaced as green tests
//! that asserted observables (entity counts, message counts) while silently
//! skipping the producer system in the fixture's plugin subset. The factory is
//! the canonical default: opt-in to narrower plugin sets only with an inline
//! rationale comment cross-referencing this story.

#![allow(dead_code)]

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{enter_in_session_via_fixture, AssetWiringPlugin};
use client::presentation::PresentationPlugin;
use client::ui::lobby::LobbyUiPlugin;

/// Builds a Bevy `App` that mirrors `client::main::main()` plugin composition
/// for use in headless integration tests.
///
/// **Production reference**: `client/src/main.rs` `main()` function.
///
/// **Plugin order** (production reference is the authoritative source-of-truth):
///
/// | # | Production plugin     | Factory behaviour                            |
/// |---|-----------------------|----------------------------------------------|
/// | – | `DefaultPlugins`      | Substituted with `MinimalPlugins` +          |
/// |   |                       | `StatesPlugin` + `AssetPlugin` +             |
/// |   |                       | `init_asset::<Image>` (headless subset).     |
/// |   |                       | Rationale: `DefaultPlugins` includes         |
/// |   |                       | `WinitPlugin` and `RenderPlugin`, both of    |
/// |   |                       | which require a window + GPU not available   |
/// |   |                       | under `cargo test` in CI / on the worker.    |
/// |   |                       | AC2 explicitly permits omitting `bevy_winit` |
/// |   |                       | with this rationale.                         |
/// | 1 | `AudioSystemPlugin`   | Omitted. Rationale: spawns `AudioPlayer`     |
/// |   |                       | entities which depend on an audio backend    |
/// |   |                       | not present under `cargo test`.              |
/// | 2 | `ClientNetworkPlugin` | Omitted. Rationale: adds Lightyear           |
/// |   |                       | `ClientPlugins` and runs                     |
/// |   |                       | `connect_websocket_client` at `Startup`,     |
/// |   |                       | which dials `ws://localhost:5000`. Tests     |
/// |   |                       | inject S2C messages directly via             |
/// |   |                       | `world.write_message::<T>()` (the same path  |
/// |   |                       | the drainer reads from in production), so    |
/// |   |                       | Lightyear protocol registration is not       |
/// |   |                       | required for the migrated fixture set.       |
/// | 3 | `PresentationPlugin`  | Added verbatim.                              |
/// | 4 | `LobbyUiPlugin`       | Added verbatim.                              |
/// | 5 | `AssetWiringPlugin`   | Added verbatim.                              |
///
/// **Returned state**: app is in `ClientState::Lobby` (the default initial
/// state from `PresentationPlugin::init_state::<ClientState>()`). Use
/// [`production_client_app_in_session`] to additionally drive the fixture into
/// `ClientState::InSession`, or set `NextState<ClientState>` directly.
pub fn production_client_app() -> App {
    let mut app = App::new();

    // ── Headless test substrate (substitutes for `DefaultPlugins`) ────────
    // See module-level rationale: `DefaultPlugins` is replaced with the
    // GPU/window-free subset that the production plugins actually depend on.
    // This is the AC2-permitted `bevy_winit` omission, generalised to the
    // wider `DefaultPlugins` set.
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();

    // ── Production plugin set (mirrors `client::main::main()` order) ─────
    // The two omissions below are AC2-permitted test-only guards; the
    // remaining three plugins are added verbatim in the production order.
    //
    //   1. AudioSystemPlugin       <-- OMITTED (no audio device in test).
    //   2. ClientNetworkPlugin     <-- OMITTED (no WebSocket server in test;
    //                                 tests inject messages directly).
    //   3. PresentationPlugin
    //   4. LobbyUiPlugin
    //   5. AssetWiringPlugin
    app.add_plugins(PresentationPlugin);
    app.add_plugins(LobbyUiPlugin);
    app.add_plugins(AssetWiringPlugin);

    app
}

/// Convenience wrapper around [`production_client_app`] that immediately drives
/// the app into `ClientState::InSession` via
/// [`client::asset_wiring::enter_in_session_via_fixture`].
///
/// Returns the same `App` produced by `production_client_app()`, then pumps two
/// `update()` ticks to flush the `OnEnter(InSession)` deferred command queue
/// (including `insert_placeholder_assets` and every `spawn_*` system that
/// depends on `PlaceholderAssets`).
///
/// Use this from fixtures that want the factory's full plugin set and an
/// already-entered `InSession` state. Fixtures that need to remain in
/// `ClientState::Lobby` (e.g. the lobby fixture for
/// `native_operator_controls_test.rs`) should call [`production_client_app`]
/// directly.
pub fn production_client_app_in_session() -> App {
    let mut app = production_client_app();
    enter_in_session_via_fixture(&mut app);
    app
}

/// Re-export of `client::asset_wiring::enter_in_session_via_fixture` so
/// fixtures can pull both the factory and the session-entry helper from a
/// single use statement.
///
/// Behaviour is verbatim from `client::asset_wiring::enter_in_session_via_fixture`:
/// inserts `PlaceholderAssets`, drives `NextState<ClientState>` to `InSession`,
/// runs `app.update()` twice to flush deferred commands.
pub fn enter_in_session_via_fixture_helper(app: &mut App) {
    enter_in_session_via_fixture(app);
}
