//! Canonical production-faithful server test app factory.
//!
//! Sibling to [`production_app_factory.rs`](./production_app_factory.rs).
//! Splitting client and server factories across two files is required because
//! the `client` and `server` workspace crates do not depend on each other —
//! a single helper file cannot reference both `client::*` and `server::*`
//! items without failing to compile in one of the two test crates.
//!
//! Story: `S13-FIXTURE-FACTORY-001`
//! (`production/epics/playable-client/story-016-fixture-factory.md`).
//!
//! ## Usage
//!
//! Server integration tests include this file via `#[path]`:
//!
//! ```ignore
//! #[path = "../../helpers/production_server_app_factory.rs"]
//! mod production_server_app_factory;
//! use production_server_app_factory::production_server_app;
//!
//! #[test]
//! fn my_server_test() {
//!     let mut app = production_server_app();
//!     // ... drive state, inject C2S messages, assert ...
//! }
//! ```
//!
//! ## Plugin parity contract
//!
//! [`production_server_app`] registers the same plugin set, in the same order,
//! as `server::main::main()`, modulo:
//!
//! 1. Inline comments naming each plugin.
//! 2. Test-only environment guards — currently the only deviation is the
//!    omission of `ServerNetworkPlugin`, which binds a TCP port and would
//!    cause parallel test runs to collide. Tests inject C2S messages
//!    directly into the ECS world.
//!
//! ## Scope (S13-FIXTURE-FACTORY-001)
//!
//! No fixture in the original migration list (B1, B2, lobby_app, shop_app,
//! hand_app) consumes this factory — all five are client-side tests. This
//! file satisfies AC3 (server-side plugin set match) and is the canonical
//! entry point for any future server-side fixture (notably the Sprint 13
//! two-client harness, `S13-TWO-CLIENT-RUNTIME-HARNESS-001`).

#![allow(dead_code)]

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use server::{core, feature, foundation};

/// Builds a Bevy `App` that mirrors `server::main::main()` plugin composition.
///
/// **Production reference**: `server/src/main.rs` `main()` function.
///
/// **Plugin order** (production reference is the authoritative source-of-truth;
/// see `server/src/main.rs`):
///
/// | # | Production plugin                              | Factory behaviour |
/// |---|------------------------------------------------|-------------------|
/// | – | `MinimalPlugins`                               | Added verbatim.   |
/// | – | `StatesPlugin`                                 | Added verbatim.   |
/// | – | `AssetPlugin` (path = `../assets`)             | Added verbatim.   |
/// | 1 | `foundation::config::ConfigPlugin`             | Added verbatim.   |
/// | 2 | `core::session::GameSessionPlugin`             | Added verbatim.   |
/// | 3 | `core::rsm::RsmPlugin`                         | Added verbatim.   |
/// | 4 | `core::economy::EconomyPlugin`                 | Added verbatim.   |
/// | 5 | `core::pool::CardPoolPlugin`                   | Added verbatim.   |
/// | 6 | `feature::board::BoardPlugin`                  | Added verbatim.   |
/// | 7 | `feature::auction::AuctionPlugin`              | Added verbatim.   |
/// | 8 | `feature::acquisition::CardAcquisitionPlugin`  | Added verbatim.   |
/// | 9 | `feature::combat::CombatPlugin`                | Added verbatim.   |
/// |10 | `feature::keyword::KeywordPlugin`              | Added verbatim.   |
/// |11 | `feature::prism::PrismPlugin`                  | Added verbatim.   |
/// |12 | `network::ServerNetworkPlugin`                 | Omitted (see below). |
/// |13 | `feature::objective::ObjectivePlugin`          | Added verbatim.   |
///
/// **ServerNetworkPlugin omission rationale**: this plugin adds Lightyear
/// `ServerPlugins` and runs `open_websocket_server` at `Startup`, which binds
/// a TCP listen port (default 5000). Parallel test runs would collide on the
/// port. Tests that need C2S message flow inject messages directly into the
/// ECS world via `world.write_message::<C2SFoo>()` — the same pathway the
/// production drainer systems consume.
pub fn production_server_app() -> App {
    let mut app = App::new();

    // ── Headless test substrate (matches `server::main::main()`) ─────────
    // The server already uses `MinimalPlugins` in production, so this section
    // is a direct mirror rather than a substitution.
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(bevy::asset::AssetPlugin {
        file_path: format!("{}/../assets", env!("CARGO_MANIFEST_DIR")),
        ..default()
    });

    // ── Production plugin set (mirrors `server::main::main()` order) ─────
    //
    //   1. foundation::config::ConfigPlugin
    //   2. core::session::GameSessionPlugin
    //   3. core::rsm::RsmPlugin
    //   4. core::economy::EconomyPlugin
    //   5. core::pool::CardPoolPlugin
    //   6. feature::board::BoardPlugin
    //   7. feature::auction::AuctionPlugin
    //   8. feature::acquisition::CardAcquisitionPlugin
    //   9. feature::combat::CombatPlugin
    //  10. feature::keyword::KeywordPlugin
    //  11. feature::prism::PrismPlugin
    //  12. network::ServerNetworkPlugin   <-- OMITTED (no TCP listen in test)
    //  13. feature::objective::ObjectivePlugin
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
    app.add_plugins(feature::objective::ObjectivePlugin);

    app
}
