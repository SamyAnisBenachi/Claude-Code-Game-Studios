// Lanes and Lies — WASM Bevy client
// ADR-002: client is a read-only view; no game logic here
// ADR-003: client/ crate — Presentation layer only
// Build: trunk build --release (WASM → Vercel)

mod network;
mod state;
mod ui;

use bevy::prelude::*;
use lightyear::prelude::*;

/// Register protocol channels and messages on the client side.
/// ADR-003 fallback: lives here (not shared/) because lightyear has no `shared` feature.
/// ADR-008: ReliableChannel for all game state; UnreliableChannel for heartbeat + auction timer.
/// Lightyear 0.26 API verified via liv-bevy-lightyear skill (api_patterns.md).
/// Must be called AFTER ClientPlugins are added (Epic 4).
fn register_protocol(app: &mut App) {
    app.add_channel::<shared::protocol::ReliableChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        ..default()
    });
    app.add_channel::<shared::protocol::UnreliableChannel>(ChannelSettings {
        mode: ChannelMode::UnorderedUnreliable,
        ..default()
    });
    app.register_message::<shared::protocol::S2CHeartbeat>()
        .add_direction(NetworkDirection::ServerToClient);
}

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    register_protocol(&mut app);
    // TODO(Epic 4): add ClientPlugins + Lightyear client setup after S1-05 spike
    app.run();
}
