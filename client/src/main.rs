// Lanes and Lies — WASM Bevy client
// ADR-002: client is a read-only view; no game logic here
// ADR-003: client/ crate — Presentation layer only
// Build: trunk build --release (WASM → Vercel)

use bevy::asset::AssetPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use client::asset_wiring::AssetWiringPlugin;
use client::audio::AudioSystemPlugin;
use client::network::ClientNetworkPlugin;
use client::presentation::PresentationPlugin;
use client::ui::lobby::LobbyUiPlugin;

fn main() {
    // Initialise tracing-subscriber so app-level `tracing::*` output reaches
    // stdout (S11-TD-CLIENT-LOG-001 / PROMPT 647). Required because this crate
    // builds bevy with `default-features = false` and no `bevy_log` feature,
    // so DefaultPlugins does NOT install a LogPlugin / tracing dispatcher.
    // Without this block every `tracing::info!` etc. call from client code is
    // a silent no-op — see production/qa/evidence/client-tracing-init-fix-evidence.md.
    //
    // Pattern matches server/src/main.rs but adds an EnvFilter to suppress
    // wgpu / naga / bevy_ecs noise so app spans are visible at INFO. Override
    // with `RUST_LOG=...` (e.g. `RUST_LOG=client=debug,wgpu=warn`).
    //
    // wasm32 has no usable stdout for tracing-subscriber; browser logging is
    // out of scope for this prompt and handled by the trunk build separately.
    //
    // S13-OBS-WALLCLOCK-TIMESTAMPS-001 (PROMPT 837): wall-clock UTC ISO-8601
    // (RFC 3339) timer so multi-process logs from server + client + tests
    // align at sub-second precision. Default fmt timer emits relative seconds
    // since process start, which is useless for cross-process correlation.
    #[cfg(not(target_arch = "wasm32"))]
    {
        use tracing_subscriber::fmt::time::UtcTime;
        use tracing_subscriber::EnvFilter;
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,wgpu=warn,wgpu_hal=warn,naga=warn,bevy_ecs=info")
        });
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_timer(UtcTime::rfc_3339())
            .init();
    }

    let mut app = App::new();

    // `LogPlugin` is pulled in transitively by the `2d` feature collection.
    // It would attempt to install its own `tracing-subscriber` on startup and
    // collide with the one set just above ("Could not set global logger ...
    // Consider disabling LogPlugin"). Disabling it makes our subscriber
    // authoritative and removes the spurious ERROR line from stdout.
    let default_plugins = DefaultPlugins
        .build()
        .disable::<LogPlugin>()
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lanes and Lies".to_string(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        });

    #[cfg(not(target_arch = "wasm32"))]
    let default_plugins = default_plugins.set(AssetPlugin {
        file_path: format!("{}/../assets", env!("CARGO_MANIFEST_DIR")),
        ..default()
    });

    app.add_plugins(default_plugins);
    app.add_plugins(AudioSystemPlugin);
    app.add_plugins(ClientNetworkPlugin);
    app.add_plugins(PresentationPlugin);
    app.add_plugins(LobbyUiPlugin);
    app.add_plugins(AssetWiringPlugin);
    app.run();
}
