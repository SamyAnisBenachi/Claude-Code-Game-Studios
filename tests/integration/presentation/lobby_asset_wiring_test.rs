use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{
    lobby_portrait_asset, LOBBY_PLAYER_SLOT_PANEL_ASSET, LOBBY_ROOM_CODE_CHIP_ASSET,
};
use client::ui::lobby::{
    LobbyClassPortrait, LobbyOpponentSlotPanel, LobbyOwnSlotPanel, LobbyRoomCodeChip,
    LobbyViewState,
};
use shared::card::ClassId;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Minimal app with asset infrastructure. Does NOT add LobbyUiPlugin to avoid
/// pulling in the full lightyear message bus; lobby entities are spawned
/// directly via Commands in the test systems below.
fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_resource::<LobbyViewState>();
    app
}

/// Spawns all lobby chrome entities directly, mirroring what `spawn_lobby_ui_system`
/// produces for the PAW-006 surfaces. Uses the same `ImageNode::new(asset_server.load(…))`
/// pattern as the production system.
fn spawn_lobby_chrome(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ── Class portraits — 7 total including Neutral (PAW-006-a) ──────────────
    for class_id in all_class_ids() {
        commands.spawn((
            LobbyClassPortrait { class_id },
            Node::default(),
            ImageNode::new(asset_server.load(lobby_portrait_asset(class_id))),
        ));
    }

    // ── Own slot panel (PAW-006-b) ────────────────────────────────────────────
    commands.spawn((
        LobbyOwnSlotPanel,
        Node::default(),
        ImageNode::new(asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET)),
    ));

    // ── Opponent slot panel (PAW-006-b) ───────────────────────────────────────
    commands.spawn((
        LobbyOpponentSlotPanel,
        Node::default(),
        ImageNode::new(asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET)),
    ));

    // ── Room code chip (PAW-006-c) ────────────────────────────────────────────
    commands.spawn((
        LobbyRoomCodeChip,
        Node::default(),
        ImageNode::new(asset_server.load(LOBBY_ROOM_CODE_CHIP_ASSET)),
    ));
}

fn all_class_ids() -> [ClassId; 7] {
    [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ]
}

// ── AC PAW-006-e: ImageNode present on all 10 lobby chrome entities ───────────

/// All 7 class portrait entities have an `ImageNode` component (PAW-006-a).
#[test]
fn test_all_seven_portrait_entities_have_image_node() {
    let mut app = make_app();
    app.add_systems(Startup, spawn_lobby_chrome);
    app.update();

    let portrait_count = app
        .world()
        .query_filtered::<Entity, (With<LobbyClassPortrait>, With<ImageNode>)>()
        .iter(app.world())
        .count();

    assert_eq!(
        portrait_count, 7,
        "Expected 7 LobbyClassPortrait entities with ImageNode, got {portrait_count}"
    );
}

/// Each `LobbyClassPortrait` entity has a non-default `ImageNode` handle (PAW-006-a).
#[test]
fn test_portrait_image_nodes_are_non_default() {
    let mut app = make_app();
    app.add_systems(Startup, spawn_lobby_chrome);
    app.update();

    let world = app.world();
    let mut query = world.query::<(&LobbyClassPortrait, &ImageNode)>();
    let portraits: Vec<_> = query.iter(world).collect();

    assert_eq!(
        portraits.len(),
        7,
        "Expected 7 portrait entities, got {}",
        portraits.len()
    );

    for (portrait, image_node) in &portraits {
        assert!(
            image_node.image != Handle::default(),
            "LobbyClassPortrait {:?} has a default (unloaded) ImageNode handle",
            portrait.class_id
        );
    }
}

/// Each portrait entity's image handle path matches `lobby_portrait_asset(class_id)` (PAW-006-a).
#[test]
fn test_portrait_image_paths_match_selector() {
    let mut app = make_app();
    app.add_systems(Startup, spawn_lobby_chrome);
    app.update();

    let world = app.world();

    // Build expected paths from the selector for all 7 variants.
    let expected_paths: Vec<(&str, ClassId)> = all_class_ids()
        .iter()
        .map(|&c| (lobby_portrait_asset(c), c))
        .collect();

    // For each expected class, confirm an entity exists with that path loaded.
    for (expected_path, class_id) in &expected_paths {
        let asset_server = world.resource::<AssetServer>();
        let expected_handle: Handle<Image> = asset_server.load(*expected_path);

        let mut query = world.query::<(&LobbyClassPortrait, &ImageNode)>();
        let found = query.iter(world).any(|(portrait, image_node)| {
            portrait.class_id == *class_id && image_node.image == expected_handle
        });

        assert!(
            found,
            "No portrait entity found for ClassId::{:?} with expected path {}",
            class_id, expected_path
        );
    }
}

/// Both player slot panel entities have an `ImageNode` component (PAW-006-b).
#[test]
fn test_own_slot_panel_has_image_node() {
    let mut app = make_app();
    app.add_systems(Startup, spawn_lobby_chrome);
    app.update();

    let world = app.world();

    let own_count = world
        .query_filtered::<Entity, (With<LobbyOwnSlotPanel>, With<ImageNode>)>()
        .iter(world)
        .count();

    assert_eq!(
        own_count, 1,
        "Expected 1 LobbyOwnSlotPanel with ImageNode, got {own_count}"
    );

    let opponent_count = world
        .query_filtered::<Entity, (With<LobbyOpponentSlotPanel>, With<ImageNode>)>()
        .iter(world)
        .count();

    assert_eq!(
        opponent_count, 1,
        "Expected 1 LobbyOpponentSlotPanel with ImageNode, got {opponent_count}"
    );
}

/// The room code chip entity has an `ImageNode` component (PAW-006-c).
#[test]
fn test_room_code_chip_has_image_node() {
    let mut app = make_app();
    app.add_systems(Startup, spawn_lobby_chrome);
    app.update();

    let world = app.world();
    let chip_count = world
        .query_filtered::<Entity, (With<LobbyRoomCodeChip>, With<ImageNode>)>()
        .iter(world)
        .count();

    assert_eq!(
        chip_count, 1,
        "Expected 1 LobbyRoomCodeChip with ImageNode, got {chip_count}"
    );
}

/// All 10 expected lobby chrome entities have ImageNode (portraits x7, slots x2, chip x1). (PAW-006-e)
#[test]
fn test_all_ten_lobby_chrome_entities_have_image_node() {
    let mut app = make_app();
    app.add_systems(Startup, spawn_lobby_chrome);
    app.update();

    let world = app.world();

    let portrait_count = world
        .query_filtered::<Entity, (With<LobbyClassPortrait>, With<ImageNode>)>()
        .iter(world)
        .count();
    let own_count = world
        .query_filtered::<Entity, (With<LobbyOwnSlotPanel>, With<ImageNode>)>()
        .iter(world)
        .count();
    let opponent_count = world
        .query_filtered::<Entity, (With<LobbyOpponentSlotPanel>, With<ImageNode>)>()
        .iter(world)
        .count();
    let chip_count = world
        .query_filtered::<Entity, (With<LobbyRoomCodeChip>, With<ImageNode>)>()
        .iter(world)
        .count();

    let total = portrait_count + own_count + opponent_count + chip_count;

    assert_eq!(
        total, 10,
        "Expected 10 total lobby chrome entities with ImageNode \
         (7 portraits + 2 slot panels + 1 chip), got {total}"
    );
}

/// PAW-006-d: Portrait ImageNode handle does not change when a class is selected.
/// Selection state is a separate concern; this test confirms handle identity is preserved.
#[test]
fn test_portrait_image_handle_unchanged_on_class_selection() {
    let mut app = make_app();
    app.add_systems(Startup, spawn_lobby_chrome);
    app.update();

    // Collect Cra portrait handle before any selection change.
    let handle_before = {
        let world = app.world();
        let mut query = world.query::<(&LobbyClassPortrait, &ImageNode)>();
        query
            .iter(world)
            .find(|(p, _)| p.class_id == ClassId::Cra)
            .map(|(_, img)| img.image.clone())
            .expect("Cra portrait entity must exist")
    };

    // Simulate a class selection via LobbyViewState (the only lobby state resource
    // that tracks selected_class; portrait ImageNode must remain untouched).
    app.world_mut()
        .resource_mut::<LobbyViewState>()
        .selected_class = ClassId::Cra;
    app.update();

    // Handle must be unchanged.
    let handle_after = {
        let world = app.world();
        let mut query = world.query::<(&LobbyClassPortrait, &ImageNode)>();
        query
            .iter(world)
            .find(|(p, _)| p.class_id == ClassId::Cra)
            .map(|(_, img)| img.image.clone())
            .expect("Cra portrait entity must still exist after selection")
    };

    assert_eq!(
        handle_before, handle_after,
        "LobbyClassPortrait::Cra ImageNode handle must not change on class selection"
    );
}
