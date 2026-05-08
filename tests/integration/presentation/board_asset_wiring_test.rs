use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{
    insert_placeholder_assets, remove_placeholder_assets, PlaceholderAssets, BOARD_UNIT_IOP_ASSET,
};
use client::presentation::board_rendering::{
    resolve_unit_image_handle, BoardRuntimeAssets, UNIT_PLACEHOLDER_ASSET,
};
use client::state::ClientState;
use shared::card::ClassId;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.add_systems(OnEnter(ClientState::InSession), insert_placeholder_assets);
    app.add_systems(OnExit(ClientState::InSession), remove_placeholder_assets);
    app
}

fn enter_session(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
}

fn dummy_board_assets(asset_server: &AssetServer) -> BoardRuntimeAssets {
    BoardRuntimeAssets {
        board_background: asset_server.load("art/board/env_board_background_default.png"),
        cell_idle: asset_server.load("art/board/env_cell_node_idle_board.png"),
        unit_placeholder: asset_server.load(UNIT_PLACEHOLDER_ASSET),
        hp_bar_white_pixel: asset_server.load("art/characters/hp_bar_white_pixel_1x2.png"),
        objective_unknown: asset_server.load("art/board/env_objective_unknown_board.png"),
        objective_real: asset_server.load("art/board/env_objective_real_reveal_board.png"),
        objective_fake: asset_server.load("art/board/env_objective_fake_crack_board.png"),
        board_chrome: asset_server.load("art/board/env_board_chrome_default.png"),
    }
}

// ── PAW-005-d: Class-keyed sprite ─────────────────────────────────────────────

/// PAW-005-d: BoardUnit with BoardUnitSourceClass(ClassId::Iop) and no atlas frame
/// resolves to the Iop-class asset handle, not the placeholder.
#[test]
fn test_board_unit_sprite_class_keyed_iop() {
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let asset_server = world.resource::<AssetServer>();
    let pa = world.resource::<PlaceholderAssets>();
    let ba = dummy_board_assets(asset_server);

    // No atlas frame (UNIT_PLACEHOLDER_FRAME_INDEX path) + source_class = Iop.
    let handle = resolve_unit_image_handle(Some(ClassId::Iop), Some(pa), Some(&ba))
        .expect("handle must be Some");

    let expected = asset_server.load::<Image>(BOARD_UNIT_IOP_ASSET);
    assert_eq!(
        handle.id(),
        expected.id(),
        "Iop class unit should resolve to BOARD_UNIT_IOP_ASSET, not placeholder"
    );
}

/// PAW-005-d supplemental: all 7 class variants resolve to a class-keyed handle,
/// not the unit_placeholder.
#[test]
fn test_board_unit_sprite_all_classes_non_placeholder() {
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let asset_server = world.resource::<AssetServer>();
    let pa = world.resource::<PlaceholderAssets>();
    let ba = dummy_board_assets(asset_server);
    let placeholder_handle = asset_server.load::<Image>(UNIT_PLACEHOLDER_ASSET);

    for class_id in [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ] {
        let handle = resolve_unit_image_handle(Some(class_id), Some(pa), Some(&ba))
            .expect("handle must be Some");
        assert_ne!(
            handle.id(),
            placeholder_handle.id(),
            "{class_id:?} class unit must not fall through to placeholder"
        );
    }
}

// ── PAW-005-e: Fallback to UNIT_PLACEHOLDER_ASSET ────────────────────────────

/// PAW-005-e: BoardUnit with no source class resolves to UNIT_PLACEHOLDER_ASSET.
#[test]
fn test_board_unit_sprite_fallback_no_source_class() {
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let asset_server = world.resource::<AssetServer>();
    let pa = world.resource::<PlaceholderAssets>();
    let ba = dummy_board_assets(asset_server);

    // source_class = None: should fall back to board_assets.unit_placeholder.
    let handle = resolve_unit_image_handle(None, Some(pa), Some(&ba)).expect("handle must be Some");

    let expected = asset_server.load::<Image>(UNIT_PLACEHOLDER_ASSET);
    assert_eq!(
        handle.id(),
        expected.id(),
        "Unit without source class must fall back to UNIT_PLACEHOLDER_ASSET"
    );
}

/// PAW-005-e supplemental: fallback handle is non-empty even when PlaceholderAssets
/// is unavailable (board_assets only path).
#[test]
fn test_board_unit_sprite_fallback_placeholder_assets_absent() {
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let asset_server = world.resource::<AssetServer>();
    let ba = dummy_board_assets(asset_server);

    let handle = resolve_unit_image_handle(None, None, Some(&ba)).expect("handle must be Some");

    let expected = asset_server.load::<Image>(UNIT_PLACEHOLDER_ASSET);
    assert_eq!(
        handle.id(),
        expected.id(),
        "Fallback without PlaceholderAssets must still resolve to UNIT_PLACEHOLDER_ASSET"
    );
}
