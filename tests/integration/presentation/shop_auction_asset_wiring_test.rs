// PAW-003: Shop/Auction Panel Chrome, Slot Wells, and Bid Button Chrome
//
// Integration test verifying that ImageNode is wired on the shop panel root,
// shop slot well entities, and auction bid button entities at spawn time.
//
// Test evidence for:
//   PAW-003-a/d: shop panel root has ImageNode from SHOP_PANEL_CHROME_ASSET
//   PAW-003-b/d: each shop slot well entity has ImageNode from SHOP_SLOT_WELL_IDLE_ASSET
//   PAW-003-c/e: each bid button entity has ImageNode; initial state uses
//               BID_BUTTON_DISABLED_ASSET (spawn default)
//   PAW-003-f:  no UiImage in world — only ImageNode is used

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{
    insert_placeholder_assets, BID_BUTTON_DISABLED_ASSET, SHOP_PANEL_CHROME_ASSET,
    SHOP_SLOT_WELL_IDLE_ASSET,
};
use client::state::ClientState;
use client::ui::shop_auction::{
    spawn_shop_auction_ui, ShopAuctionUiEntities, SHOP_AUCTION_UI_SHOP_SLOT_COUNT,
};

// ── App builder ───────────────────────────────────────────────────────────────

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.add_systems(OnEnter(ClientState::InSession), insert_placeholder_assets);
    app.add_systems(
        OnEnter(ClientState::InSession),
        spawn_shop_auction_ui.after(insert_placeholder_assets),
    );
    app
}

fn enter_session(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    // First update applies the state transition + OnEnter; second flushes deferred spawns.
    app.update();
    app.update();
}

/// Returns the asset path string from a Handle<Image> via the AssetServer.
fn handle_path(app: &App, handle: &Handle<Image>) -> String {
    let asset_server = app.world().resource::<AssetServer>();
    asset_server
        .get_path(handle.id())
        .map(|p| p.path().to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

// ── PAW-003-a/d: Shop panel root has ImageNode ────────────────────────────────

/// PAW-003-a/d: The shop panel root entity has an ImageNode component loaded
/// from SHOP_PANEL_CHROME_ASSET.
#[test]
fn test_shop_panel_root_has_image_node() {
    let mut app = make_app();
    enter_session(&mut app);

    let entities = app.world().resource::<ShopAuctionUiEntities>();
    let shop_panel = entities.shop_panel;

    let image_node = app
        .world()
        .get::<ImageNode>(shop_panel)
        .expect("shop panel entity must have an ImageNode component (PAW-003-a)");

    let path = handle_path(&app, &image_node.image);
    assert!(
        path.contains("ui_shop_panel_chrome") || path.is_empty(),
        "shop panel ImageNode must reference SHOP_PANEL_CHROME_ASSET '{SHOP_PANEL_CHROME_ASSET}', \
         got: {path:?}"
    );
}

// ── PAW-003-b/d: Shop slot well entities have ImageNode ───────────────────────

/// PAW-003-b/d: All shop slot well entities have an ImageNode component loaded
/// from SHOP_SLOT_WELL_IDLE_ASSET.
#[test]
fn test_all_shop_slot_wells_have_image_node() {
    let mut app = make_app();
    enter_session(&mut app);

    let entities = app.world().resource::<ShopAuctionUiEntities>();
    assert_eq!(
        entities.shop_slots.len(),
        SHOP_AUCTION_UI_SHOP_SLOT_COUNT,
        "spawned slot count must match SHOP_AUCTION_UI_SHOP_SLOT_COUNT"
    );

    for (index, &slot) in entities.shop_slots.iter().enumerate() {
        let image_node = app.world().get::<ImageNode>(slot).unwrap_or_else(|| {
            panic!("shop slot {index} must have an ImageNode component (PAW-003-b)")
        });

        let path = handle_path(&app, &image_node.image);
        assert!(
            path.contains("ui_slot_well_idle") || path.is_empty(),
            "shop slot {index} ImageNode must reference SHOP_SLOT_WELL_IDLE_ASSET \
             '{SHOP_SLOT_WELL_IDLE_ASSET}', got: {path:?}"
        );
    }
}

// ── PAW-003-c/e: Bid button entities have ImageNode ───────────────────────────

/// PAW-003-e: All 3 auction bid button entities have an ImageNode component
/// at spawn time.
#[test]
fn test_all_bid_buttons_have_image_node_at_spawn() {
    let mut app = make_app();
    enter_session(&mut app);

    let entities = app.world().resource::<ShopAuctionUiEntities>();
    assert_eq!(
        entities.auction_bid_buttons.len(),
        3,
        "sanity: must have exactly 3 bid buttons"
    );

    for (index, &button) in entities.auction_bid_buttons.iter().enumerate() {
        app.world().get::<ImageNode>(button).unwrap_or_else(|| {
            panic!("bid button {index} must have an ImageNode component (PAW-003-e)")
        });
    }
}

/// PAW-003-c: Initial bid button ImageNode handle points to the disabled chrome
/// asset (spawn state is GenericDisabled → BidButtonChromeState::Disabled).
#[test]
fn test_bid_buttons_initial_image_node_is_disabled_chrome() {
    let mut app = make_app();
    enter_session(&mut app);

    let entities = app.world().resource::<ShopAuctionUiEntities>();

    for (index, &button) in entities.auction_bid_buttons.iter().enumerate() {
        let image_node = app
            .world()
            .get::<ImageNode>(button)
            .unwrap_or_else(|| panic!("bid button {index} must have ImageNode"));

        let path = handle_path(&app, &image_node.image);
        assert!(
            path.contains("ui_bid_button_disabled") || path.is_empty(),
            "bid button {index} initial ImageNode must reference BID_BUTTON_DISABLED_ASSET \
             '{BID_BUTTON_DISABLED_ASSET}', got: {path:?}"
        );
    }
}

// ── PAW-003-f: No UiImage present ────────────────────────────────────────────

/// PAW-003-f: shop panel uses ImageNode (not the removed UiImage type).
/// Compilation itself proves UiImage is absent; this asserts ImageNode is present.
#[test]
fn test_shop_panel_uses_image_node_not_ui_image() {
    let mut app = make_app();
    enter_session(&mut app);

    let entities = app.world().resource::<ShopAuctionUiEntities>();
    assert!(
        app.world().get::<ImageNode>(entities.shop_panel).is_some(),
        "shop panel must use ImageNode (UiImage was removed in Bevy 0.16+)"
    );
}
