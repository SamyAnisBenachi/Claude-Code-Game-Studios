// Lanes and Lies — WASM Bevy client
// ADR-002: client is a read-only view; no game logic here
// ADR-003: client/ crate — Presentation layer only
// Build: trunk build --release (WASM → Vercel)

mod network;
mod state;
mod ui;

use bevy::prelude::*;
// lightyear::prelude imported in Epic 4 (S1-05 spike) once API is verified against docs.rs

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // TODO(S1-05 Lightyear spike): register_protocol(&mut app) once API is verified.
    // TODO(Epic 4): add ClientPlugins + Lightyear client setup after S1-05 spike
    app.run();
}
