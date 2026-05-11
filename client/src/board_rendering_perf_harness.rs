use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use client::card_animations::CardAnimationsPlugin;
use client::presentation::board_rendering::perf_harness::BoardRenderingPerfHarnessPlugin;
use client::presentation::BoardRenderingPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "BOARD-012 Browser/WASM Board Performance Harness".to_string(),
            resolution: (1920, 1080).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins((
        CardAnimationsPlugin,
        BoardRenderingPlugin,
        BoardRenderingPerfHarnessPlugin,
    ));
    app.init_state::<client::state::ClientState>();
    app.run();
}
