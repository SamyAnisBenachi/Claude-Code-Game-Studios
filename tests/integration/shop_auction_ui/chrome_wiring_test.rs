use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::ClientState;
use client::ui::shop_auction::{
    ShopAuctionCardCatalog, ShopAuctionUiEntities, ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn shop_auction_panel_root_carries_non_default_image_node_after_on_enter_in_session() {
    test_helpers::init_test_tracing();
    // Arrange: build the canonical partial-App fixture and transition to
    // InSession so OnEnter spawn systems run and wire ImageNode chrome.
    let app = app_in_session();

    // Act: read each chrome-bearing panel root entity.
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    // Assert: shop panel root carries a non-default ImageNode.image handle.
    let shop_image = app
        .world()
        .get::<ImageNode>(entities.shop_panel)
        .expect("shop panel root must have ImageNode after OnEnter(InSession)");
    assert_ne!(
        shop_image.image,
        Handle::<Image>::default(),
        "shop panel ImageNode.image must be non-default (sourced from asset_wiring SHOP_PANEL_CHROME_ASSET)"
    );
}

#[test]
fn shop_auction_auction_panel_root_carries_non_default_image_node_after_on_enter_in_session() {
    test_helpers::init_test_tracing();
    // Arrange.
    let app = app_in_session();

    // Act.
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    // Assert: auction panel root carries a non-default ImageNode.image handle.
    let auction_image = app
        .world()
        .get::<ImageNode>(entities.auction_panel)
        .expect("auction panel root must have ImageNode after OnEnter(InSession)");
    assert_ne!(
        auction_image.image,
        Handle::<Image>::default(),
        "auction panel ImageNode.image must be non-default (sourced from asset_wiring constant)"
    );
}

#[test]
fn shop_auction_bid_buttons_carry_non_default_image_node_after_on_enter_in_session() {
    test_helpers::init_test_tracing();
    // Arrange.
    let app = app_in_session();

    // Act.
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    // Assert: each of the three bid buttons carries a non-default ImageNode.image handle.
    for (index, button) in entities.auction_bid_buttons.into_iter().enumerate() {
        let image = app.world().get::<ImageNode>(button).unwrap_or_else(|| {
            panic!("bid button {index} must have ImageNode after OnEnter(InSession)")
        });
        assert_ne!(
            image.image,
            Handle::<Image>::default(),
            "bid button {index} ImageNode.image must be non-default (sourced from asset_wiring bid_button_asset)"
        );
    }
}

#[test]
fn shop_auction_shop_slots_carry_non_default_image_node_after_on_enter_in_session() {
    test_helpers::init_test_tracing();
    // Arrange.
    let app = app_in_session();

    // Act.
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    // Assert: each shop slot well carries a non-default ImageNode.image handle.
    for (index, slot) in entities.shop_slots.into_iter().enumerate() {
        let image = app.world().get::<ImageNode>(slot).unwrap_or_else(|| {
            panic!("shop slot {index} must have ImageNode after OnEnter(InSession)")
        });
        assert_ne!(
            image.image,
            Handle::<Image>::default(),
            "shop slot {index} ImageNode.image must be non-default (sourced from asset_wiring SHOP_SLOT_WELL_IDLE_ASSET)"
        );
    }
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    insert_catalog(&mut app);
    app.insert_resource(PlayerEconomyView {
        gold: 5,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn insert_catalog(app: &mut App) {
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=6)
            .map(|id| {
                let card = test_card(id);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
}

fn test_card(id: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: id.min(3),
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: None,
    }
}
