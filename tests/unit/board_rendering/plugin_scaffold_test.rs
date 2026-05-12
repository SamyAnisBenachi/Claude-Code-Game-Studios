use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::board_rendering::BoardLocalPlayer;
use client::presentation::{BoardLayout, BoardRenderingPlugin, CardAtlas, PresentationPlugin};
use client::state::{ClientSessionIdentity, ClientState};
use shared::session::PlayerId;

#[test]
fn board_rendering_plugin_registers_in_minimal_client_app_without_panic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);

    app.update();
}

#[test]
fn board_rendering_resources_exist_only_during_session() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);

    assert!(app.world().get_resource::<BoardLayout>().is_none());
    assert!(app.world().get_resource::<CardAtlas>().is_none());

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    assert!(app.world().get_resource::<BoardLayout>().is_some());
    let atlas = app
        .world()
        .get_resource::<CardAtlas>()
        .expect("CardAtlas should be inserted on session entry");
    let _image_handle: Handle<Image> = atlas.image.clone();
    let _layout_handle: Handle<TextureAtlasLayout> = atlas.layout.clone();

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Lobby);
    app.update();

    assert!(app.world().get_resource::<BoardLayout>().is_none());
    assert!(app.world().get_resource::<CardAtlas>().is_none());
}

#[test]
fn board_layout_maps_cells_from_board_origin() {
    let layout = BoardLayout {
        board_origin: Vec2::new(-128.0, 160.0),
        cell_width: 64.0,
        lane_height: 80.0,
    };

    assert_eq!(layout.cell_to_world(1, 1), layout.board_origin);
    assert_eq!(
        layout.cell_to_world(1, 2),
        layout.board_origin + Vec2::new(layout.cell_width, 0.0)
    );
    assert_eq!(
        layout.cell_to_world(2, 1),
        layout.board_origin - Vec2::new(0.0, layout.lane_height)
    );
}

#[test]
#[should_panic(expected = "invalid lane=0")]
fn board_layout_asserts_on_lane_zero() {
    BoardLayout::default().cell_to_world(0, 1);
}

#[test]
#[should_panic(expected = "invalid lane=6")]
fn board_layout_asserts_on_lane_six() {
    BoardLayout::default().cell_to_world(6, 1);
}

#[test]
#[should_panic(expected = "invalid cell=0")]
fn board_layout_asserts_on_cell_zero() {
    BoardLayout::default().cell_to_world(1, 0);
}

#[test]
#[should_panic(expected = "invalid cell=9")]
fn board_layout_asserts_on_cell_nine() {
    BoardLayout::default().cell_to_world(1, 9);
}

#[test]
fn board_local_player_initializes_from_client_session_identity_handshake_only_path() {
    const LOCAL: PlayerId = PlayerId(1);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);

    app.insert_resource(ClientSessionIdentity {
        player_id: Some(LOCAL),
        session_id: None,
        session_token: None,
    });

    assert_eq!(
        app.world().resource::<BoardLocalPlayer>().player_id,
        None,
        "BoardLocalPlayer must start unpopulated before the init system runs"
    );

    app.update();

    assert_eq!(
        app.world().resource::<BoardLocalPlayer>().player_id,
        Some(LOCAL),
        "BoardLocalPlayer.player_id must be populated from ClientSessionIdentity \
         on the handshake-only path (no S2CGameSnapshot received)"
    );
}

#[test]
fn presentation_plugin_registers_board_rendering_resources_in_adr_order_slot() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(PresentationPlugin);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    assert!(app.world().get_resource::<BoardLayout>().is_some());
    assert!(app.world().get_resource::<CardAtlas>().is_some());
}

#[test]
fn board_rendering_does_not_register_phase_receiver() {
    let client_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut matches = Vec::new();
    collect_normalized_source_matches(
        &client_src,
        "MessageReceiver<S2CPhaseChanged>",
        &mut matches,
    );

    assert_eq!(
        matches,
        vec![client_src.join("presentation").join("mod.rs")]
    );

    let board_rendering_source =
        fs::read_to_string(client_src.join("presentation").join("board_rendering.rs"))
            .expect("board rendering source should be readable");
    assert!(
        !normalize_source(&board_rendering_source).contains("MessageReceiver<S2CPhaseChanged>"),
        "Board Rendering must read Res<CurrentClientPhase>; it must not drain MessageReceiver<S2CPhaseChanged>"
    );
}

fn collect_normalized_source_matches(
    path: &Path,
    needle: &str,
    matches: &mut Vec<std::path::PathBuf>,
) {
    let entries = fs::read_dir(path).expect("client source directory should be readable");
    for entry in entries {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            collect_normalized_source_matches(&path, needle, matches);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("Rust source file should be readable");
        let normalized = normalize_source(&contents);
        for _ in normalized.match_indices(needle) {
            matches.push(path.clone());
        }
    }
}

fn normalize_source(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}
