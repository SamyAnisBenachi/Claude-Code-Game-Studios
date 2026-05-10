use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{
    lobby_portrait_asset, LOBBY_PLAYER_SLOT_PANEL_ASSET, LOBBY_ROOM_CODE_CHIP_ASSET,
};
use client::state::ClientState;
use client::ui::lobby::{
    LobbyClassPortrait, LobbyOpponentSlotPanel, LobbyOwnSlotPanel, LobbyRoomCodeChip, LobbyUiPlugin,
};
use shared::card::ClassId;

// Canonical partial-App fixture for S10-POLISH-003 lobby chrome MVP.
//
// Mirrors the S10-POLISH-002 pattern at
// `tests/integration/shop_auction_ui/chrome_wiring_test.rs` — drives
// through `OnEnter(ClientState::Lobby)` via the actual `LobbyUiPlugin`
// (not a `spawn_lobby_chrome` direct copy as in
// `tests/integration/presentation/lobby_asset_wiring_test.rs`) and
// asserts that PAW-006 wiring still resolves to non-default
// `ImageNode.image` handles after the plugin's spawn system runs.
//
// `ClientState::default()` is `Lobby`, so the `OnEnter(Lobby)` schedule
// fires on the first `app.update()` call without an explicit
// `NextState::set(Lobby)` transition.
fn lobby_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    // `lobby_keyboard_input_system` requires `Res<ButtonInput<KeyCode>>`,
    // which `MinimalPlugins` does not insert (Bevy 0.18 gates input behind
    // its own plugin/feature). Inject a default empty `ButtonInput` so the
    // run-condition systems schedule cleanly without requiring `InputPlugin`
    // (which would also pull in winit infrastructure unnecessary here).
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.add_plugins(LobbyUiPlugin);
    app.update();
    app
}

const ALL_CLASS_IDS: [ClassId; 7] = [
    ClassId::Iop,
    ClassId::Cra,
    ClassId::Sacrier,
    ClassId::Xelor,
    ClassId::Ecaflip,
    ClassId::Sadida,
    ClassId::Neutral,
];

// AC-3: every lobby chrome entity carries a non-default ImageNode.image
// handle after OnEnter(ClientState::Lobby) via LobbyUiPlugin.

#[test]
fn test_lobby_class_portraits_carry_non_default_image_node_after_on_enter_lobby() {
    // Arrange: build the partial-App fixture and drive through OnEnter(Lobby).
    let mut app = lobby_app();

    // Act: query LobbyClassPortrait + ImageNode entities.
    let world = app.world_mut();
    let mut query = world.query::<(&LobbyClassPortrait, &ImageNode)>();
    let portraits: Vec<(ClassId, Handle<Image>)> = query
        .iter(world)
        .map(|(p, img)| (p.class_id, img.image.clone()))
        .collect();

    // Assert: 7 portrait entities, each with non-default ImageNode.image.
    assert_eq!(
        portraits.len(),
        7,
        "Expected 7 LobbyClassPortrait entities after OnEnter(Lobby), got {}",
        portraits.len()
    );

    for (class_id, handle) in &portraits {
        assert_ne!(
            *handle,
            Handle::<Image>::default(),
            "LobbyClassPortrait {:?} ImageNode.image must be non-default \
             (sourced from asset_wiring lobby_portrait_asset)",
            class_id
        );
    }
}

#[test]
fn test_lobby_own_slot_panel_carries_non_default_image_node_after_on_enter_lobby() {
    // Arrange.
    let mut app = lobby_app();

    // Act.
    let world = app.world_mut();
    let mut query = world.query_filtered::<&ImageNode, With<LobbyOwnSlotPanel>>();
    let images: Vec<Handle<Image>> = query.iter(world).map(|img| img.image.clone()).collect();

    // Assert.
    assert_eq!(
        images.len(),
        1,
        "Expected 1 LobbyOwnSlotPanel after OnEnter(Lobby), got {}",
        images.len()
    );
    assert_ne!(
        images[0],
        Handle::<Image>::default(),
        "LobbyOwnSlotPanel ImageNode.image must be non-default \
         (sourced from asset_wiring LOBBY_PLAYER_SLOT_PANEL_ASSET)"
    );
}

#[test]
fn test_lobby_opponent_slot_panel_carries_non_default_image_node_after_on_enter_lobby() {
    // Arrange.
    let mut app = lobby_app();

    // Act.
    let world = app.world_mut();
    let mut query = world.query_filtered::<&ImageNode, With<LobbyOpponentSlotPanel>>();
    let images: Vec<Handle<Image>> = query.iter(world).map(|img| img.image.clone()).collect();

    // Assert.
    assert_eq!(
        images.len(),
        1,
        "Expected 1 LobbyOpponentSlotPanel after OnEnter(Lobby), got {}",
        images.len()
    );
    assert_ne!(
        images[0],
        Handle::<Image>::default(),
        "LobbyOpponentSlotPanel ImageNode.image must be non-default \
         (sourced from asset_wiring LOBBY_PLAYER_SLOT_PANEL_ASSET)"
    );
}

#[test]
fn test_lobby_room_code_chip_carries_non_default_image_node_after_on_enter_lobby() {
    // Arrange.
    let mut app = lobby_app();

    // Act.
    let world = app.world_mut();
    let mut query = world.query_filtered::<&ImageNode, With<LobbyRoomCodeChip>>();
    let images: Vec<Handle<Image>> = query.iter(world).map(|img| img.image.clone()).collect();

    // Assert.
    assert_eq!(
        images.len(),
        1,
        "Expected 1 LobbyRoomCodeChip after OnEnter(Lobby), got {}",
        images.len()
    );
    assert_ne!(
        images[0],
        Handle::<Image>::default(),
        "LobbyRoomCodeChip ImageNode.image must be non-default \
         (sourced from asset_wiring LOBBY_ROOM_CODE_CHIP_ASSET)"
    );
}

// AC-4: per-class portrait path matches lobby_portrait_asset(class_id).
//
// Asserts that each spawned `LobbyClassPortrait` entity's `ImageNode.image`
// is the handle the AssetServer returns for `lobby_portrait_asset(class_id)`.
// This guards against a regression where every portrait would share the
// same fallback handle (e.g. hard-coded to `LOBBY_PORTRAIT_NEUTRAL_ASSET`).
#[test]
fn test_lobby_portrait_per_class_path_matches_asset_wiring_selector() {
    // Arrange.
    let mut app = lobby_app();

    let world = app.world();
    let asset_server = world.resource::<AssetServer>().clone();

    // Pre-compute the expected handles for each class via the same
    // `asset_server.load(lobby_portrait_asset(class_id))` shape the
    // production system uses.
    let expected: Vec<(ClassId, Handle<Image>)> = ALL_CLASS_IDS
        .iter()
        .map(|class_id| {
            (
                *class_id,
                asset_server.load(lobby_portrait_asset(*class_id)),
            )
        })
        .collect();

    // Act.
    let world = app.world_mut();
    let mut query = world.query::<(&LobbyClassPortrait, &ImageNode)>();
    let actual: Vec<(ClassId, Handle<Image>)> = query
        .iter(world)
        .map(|(p, img)| (p.class_id, img.image.clone()))
        .collect();

    // Assert: for every expected (class_id, handle), an entity exists
    // whose `class_id` matches and whose `ImageNode.image` equals the
    // expected handle.
    for (class_id, expected_handle) in &expected {
        let found = actual
            .iter()
            .any(|(cid, h)| cid == class_id && h == expected_handle);
        assert!(
            found,
            "LobbyClassPortrait::{:?} must carry the handle returned by \
             asset_server.load(lobby_portrait_asset(ClassId::{:?}))",
            class_id, class_id
        );
    }
}

// AC-1 / AC-2 audit-style assertions are intentionally NOT mirrored as
// runtime tests — they are static-source guarantees verified by grep
// (recorded in the story's QA Test Cases section). The runtime tests
// above cover AC-3 + AC-4. AC-5 is preserved by leaving the existing
// `lobby_asset_wiring_test.rs` untouched. AC-6 is preserved by leaving
// `client/src/ui/lobby.rs` untouched (no edits to the C2S send or S2C
// drain paths in this story's diff).
//
// The constants below are referenced only to keep the import surface
// honest: if PAW-006 ever drops any of these from `asset_wiring`, this
// file fails to compile and surfaces the regression.
const _PAW_006_LOBBY_ASSET_TOUCHSTONES: &[&str] =
    &[LOBBY_PLAYER_SLOT_PANEL_ASSET, LOBBY_ROOM_CODE_CHIP_ASSET];
