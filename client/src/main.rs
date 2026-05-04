// Lanes and Lies — WASM Bevy client
// ADR-002: client is a read-only view; no game logic here
// ADR-003: client/ crate — Presentation layer only
// Build: trunk build --release (WASM → Vercel)

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::network::ClientNetworkPlugin;
use client::presentation::PresentationPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.add_plugins(ClientNetworkPlugin);
    app.add_plugins(PresentationPlugin);
    app.run();
}
