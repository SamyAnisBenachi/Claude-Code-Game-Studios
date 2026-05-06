// Lanes and Lies — WASM Bevy client
// ADR-002: client is a read-only view; no game logic here
// ADR-003: client/ crate — Presentation layer only
// Build: trunk build --release (WASM → Vercel)

use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use client::network::ClientNetworkPlugin;
use client::presentation::PresentationPlugin;
use client::ui::lobby::LobbyUiPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Lanes and Lies".to_string(),
            present_mode: PresentMode::AutoVsync,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(ClientNetworkPlugin);
    app.add_plugins(PresentationPlugin);
    app.add_plugins(LobbyUiPlugin);
    app.run();
}
