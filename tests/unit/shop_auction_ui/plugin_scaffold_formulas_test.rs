use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PresentationPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    auction_border_color_tier, bid_button_label_texts, local_free_gold, AuctionBorderColorTier,
    ShopAuctionPanelRoot, ShopAuctionUiEntities, ShopAuctionUiEntity, ShopAuctionUiMode,
    ShopAuctionUiPlugin, SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT,
    SHOP_AUCTION_UI_PANEL_ROOT_COUNT, SHOP_AUCTION_UI_SHOP_SLOT_COUNT,
};
use shared::protocol::RoundPhase;

#[test]
fn shop_auction_ui_plugin_registers_in_minimal_client_app_without_panic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(ShopAuctionUiPlugin);

    app.update();
}

#[test]
fn shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes() {
    let mut app = app_with_shop_auction_ui_in_session();
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    assert_eq!(
        count_with::<ShopAuctionPanelRoot>(&mut app),
        SHOP_AUCTION_UI_PANEL_ROOT_COUNT
    );
    assert_eq!(
        count_with::<ShopAuctionUiEntity>(&mut app),
        1 + SHOP_AUCTION_UI_PANEL_ROOT_COUNT * 2
            + SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT * 2
            + 3
            + 4
            + SHOP_AUCTION_UI_SHOP_SLOT_COUNT
            + 4
            + 12
    );

    for panel_root in entities.panel_roots() {
        assert!(
            app.world().get::<Node>(panel_root).is_some(),
            "panel root {panel_root:?} should be a bevy_ui Node"
        );
        assert!(
            app.world().get::<Sprite>(panel_root).is_none(),
            "panel root {panel_root:?} should not be a world-space Sprite"
        );
        assert_eq!(
            app.world().get::<Visibility>(panel_root),
            Some(&Visibility::Hidden)
        );
    }
}

#[test]
fn shop_auction_ui_roots_are_stable_during_session_updates() {
    let mut app = app_with_shop_auction_ui_in_session();
    let initial = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .panel_roots();

    for _ in 0..3 {
        app.update();
    }

    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .panel_roots(),
        initial
    );
}

#[test]
fn shop_auction_ui_plugin_is_registered_fifth_through_presentation_plugin() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(PresentationPlugin);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    assert!(app
        .world()
        .get_resource::<ShopAuctionUiEntities>()
        .is_some());
}

#[test]
fn shop_auction_ui_phase_visibility_reads_current_phase_resource() {
    let mut app = app_with_shop_auction_ui_in_session();
    set_phase(&mut app, RoundPhase::DraftAuction);
    app.update();

    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    assert_eq!(
        app.world().get_resource::<ShopAuctionUiMode>(),
        Some(&ShopAuctionUiMode::Inactive)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.auction_panel),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.shop_footer),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.draft_offering_panel),
        Some(&Visibility::Hidden)
    );

    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();
    assert_eq!(
        app.world().get_resource::<ShopAuctionUiMode>(),
        Some(&ShopAuctionUiMode::Shop)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.shop_panel),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.auction_panel),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn local_free_gold_saturates_reserved_gold_without_underflow() {
    assert_eq!(local_free_gold(10, 3), 7);
    assert_eq!(local_free_gold(3, 10), 0);
}

#[test]
fn bid_labels_render_total_commitment_with_secondary_increment() {
    assert_eq!(
        bid_button_label_texts(7),
        ["8g\n(+1)", "10g\n(+3)", "12g\n(+5)"]
    );
}

#[test]
fn auction_border_tier_maps_gdd_price_ranges() {
    assert_eq!(
        auction_border_color_tier(3),
        AuctionBorderColorTier::PaleInkBlue
    );
    assert_eq!(
        auction_border_color_tier(4),
        AuctionBorderColorTier::AuctionAmber
    );
    assert_eq!(
        auction_border_color_tier(7),
        AuctionBorderColorTier::DeepAmber
    );
    assert_eq!(
        auction_border_color_tier(10),
        AuctionBorderColorTier::CrimsonAmber
    );
}

fn app_with_shop_auction_ui_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}
