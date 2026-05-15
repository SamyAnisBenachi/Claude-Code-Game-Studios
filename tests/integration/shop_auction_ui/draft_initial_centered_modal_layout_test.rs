use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::GlobalZIndex;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::design_tokens::z_layers;
use client::ui::shop_auction::{
    DraftInitialGrid, DraftInitialModalPanel, DraftInitialSlotCard, ShopAuctionCardCatalog,
    ShopAuctionDraftOfferingReceived, ShopAuctionShopSlotsReceived, ShopAuctionUiEntities,
    ShopAuctionUiPlugin, DRAFT_INITIAL_GRID_COLUMN_GAP_PX, DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX,
    DRAFT_INITIAL_GRID_HEIGHT_PX, DRAFT_INITIAL_GRID_LEFT_PX, DRAFT_INITIAL_GRID_ROW_GAP_PX,
    DRAFT_INITIAL_GRID_ROW_HEIGHT_PX, DRAFT_INITIAL_GRID_TOP_PX, DRAFT_INITIAL_GRID_WIDTH_PX,
    DRAFT_INITIAL_MODAL_HEIGHT_PX, DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT,
    DRAFT_INITIAL_MODAL_MAX_WIDTH_PX, DRAFT_INITIAL_MODAL_PADDING_PX,
    DRAFT_INITIAL_MODAL_WIDTH_PERCENT, SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn sau_015_draft_initial_uses_centered_modal_root_and_panel_constants() {
    test_helpers::init_test_tracing();
    let app = active_draft_app();
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    let root_node = node(&app, entities.draft_offering_panel);
    assert_eq!(root_node.display, Display::Flex);
    assert_eq!(root_node.align_items, AlignItems::Center);
    assert_eq!(root_node.justify_content, JustifyContent::Center);
    assert_eq!(root_node.left, Val::Px(0.0));
    assert_eq!(root_node.right, Val::Px(0.0));
    assert_eq!(root_node.top, Val::Px(0.0));
    assert_eq!(root_node.bottom, Val::Px(0.0));
    assert_eq!(
        app.world()
            .get::<GlobalZIndex>(entities.draft_offering_panel),
        Some(&z_layers::MODAL)
    );

    let modal_node = node(&app, entities.draft_initial_modal_panel);
    assert!(app
        .world()
        .get::<DraftInitialModalPanel>(entities.draft_initial_modal_panel)
        .is_some());
    assert_eq!(
        parent_of(&app, entities.draft_initial_modal_panel),
        entities.draft_offering_panel
    );
    assert_eq!(modal_node.display, Display::Flex);
    assert_eq!(
        modal_node.width,
        Val::Percent(DRAFT_INITIAL_MODAL_WIDTH_PERCENT)
    );
    assert_eq!(
        modal_node.max_width,
        Val::Px(DRAFT_INITIAL_MODAL_MAX_WIDTH_PX)
    );
    assert_eq!(modal_node.height, Val::Px(DRAFT_INITIAL_MODAL_HEIGHT_PX));
    assert_eq!(
        modal_node.max_height,
        Val::Percent(DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT)
    );
    assert_eq!(
        modal_node.padding,
        UiRect::all(Val::Px(DRAFT_INITIAL_MODAL_PADDING_PX))
    );
}

#[test]
fn sau_015_draft_initial_and_shop_roots_are_siblings_and_mutually_exclusive() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app();
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    assert_eq!(
        parent_of(&app, entities.draft_offering_panel),
        entities.root
    );
    assert_eq!(parent_of(&app, entities.shop_panel), entities.root);
    assert_ne!(entities.draft_offering_panel, entities.shop_panel);
    assert_eq!(
        app.world().get::<Visibility>(entities.draft_offering_panel),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.shop_panel),
        Some(&Visibility::Hidden)
    );

    set_phase(&mut app, RoundPhase::DraftShop);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    assert_eq!(
        app.world().get::<Visibility>(entities.draft_offering_panel),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.shop_panel),
        Some(&Visibility::Visible)
    );
}

#[test]
fn sau_015_draft_initial_grid_has_stable_rows_columns_and_spacing() {
    test_helpers::init_test_tracing();
    let app = active_draft_app();
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    assert!(app
        .world()
        .get::<DraftInitialGrid>(entities.draft_initial_grid)
        .is_some());
    assert_eq!(
        parent_of(&app, entities.draft_initial_grid),
        entities.draft_initial_modal_panel
    );

    let grid_node = node(&app, entities.draft_initial_grid);
    assert_eq!(grid_node.position_type, PositionType::Absolute);
    assert_eq!(grid_node.left, Val::Px(DRAFT_INITIAL_GRID_LEFT_PX));
    assert_eq!(grid_node.top, Val::Px(DRAFT_INITIAL_GRID_TOP_PX));
    assert_eq!(grid_node.width, Val::Px(DRAFT_INITIAL_GRID_WIDTH_PX));
    assert_eq!(grid_node.height, Val::Px(DRAFT_INITIAL_GRID_HEIGHT_PX));

    for (index, slot) in entities.draft_initial_slots.iter().enumerate() {
        assert_eq!(parent_of(&app, *slot), entities.draft_initial_grid);
        assert!(app.world().get::<DraftInitialSlotCard>(*slot).is_some());
        assert_eq!(
            app.world().get::<Visibility>(*slot),
            Some(&Visibility::Visible)
        );

        let column = index % 3;
        let row = index / 3;
        let slot_node = node(&app, *slot);
        assert_eq!(
            slot_node.left,
            Val::Px(
                column as f32
                    * (DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX + DRAFT_INITIAL_GRID_COLUMN_GAP_PX)
            )
        );
        assert_eq!(
            slot_node.top,
            Val::Px(
                row as f32 * (DRAFT_INITIAL_GRID_ROW_HEIGHT_PX + DRAFT_INITIAL_GRID_ROW_GAP_PX)
            )
        );
        assert_eq!(slot_node.width, Val::Px(DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX));
        assert_eq!(slot_node.height, Val::Px(DRAFT_INITIAL_GRID_ROW_HEIGHT_PX));
    }

    assert_eq!(
        entities.draft_initial_slots.len(),
        SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT
    );
    assert!(DRAFT_INITIAL_GRID_COLUMN_GAP_PX > 0.0);
    assert!(DRAFT_INITIAL_GRID_ROW_GAP_PX > 0.0);
}

#[test]
fn sau_015_objective_and_ready_controls_do_not_overlap_the_grid_band() {
    test_helpers::init_test_tracing();
    let app = active_draft_app();
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    let overlay = node(&app, entities.draft_initial_objective_overlay);
    let overlay_bottom = px(overlay.top) + px(overlay.height);
    assert!(
        overlay_bottom <= DRAFT_INITIAL_GRID_TOP_PX,
        "objective overlay bottom {overlay_bottom} must stay above grid top {}",
        DRAFT_INITIAL_GRID_TOP_PX
    );

    let retrieval = node(&app, entities.draft_initial_objective_retrieval_button);
    let retrieval_bottom = px(retrieval.top) + px(retrieval.height);
    assert!(
        retrieval_bottom <= DRAFT_INITIAL_GRID_TOP_PX,
        "retrieval affordance bottom {retrieval_bottom} must stay above grid top {}",
        DRAFT_INITIAL_GRID_TOP_PX
    );

    let ready = node(&app, entities.draft_initial_ready_button);
    let ready_left_at_max_width =
        DRAFT_INITIAL_MODAL_MAX_WIDTH_PX - px(ready.right) - px(ready.width);
    let grid_right = DRAFT_INITIAL_GRID_LEFT_PX + DRAFT_INITIAL_GRID_WIDTH_PX;
    assert!(
        grid_right < ready_left_at_max_width,
        "grid right {grid_right} must stay left of ready button left {ready_left_at_max_width}"
    );
}

fn active_draft_app() -> App {
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftInitial);
    send_offering(&mut app, card_ids(1, 9));
    app
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
    run_update(&mut app);
    app
}

fn insert_catalog(app: &mut App) {
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=9)
            .map(|id| {
                let card = test_card(id, Rarity::Common, (id - 1) % 5 + 1);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
}

fn test_card(id: u32, rarity: Rarity, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
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

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
    run_update(app);
}

fn send_offering(app: &mut App, card_ids: Vec<CardId>) {
    app.world_mut()
        .write_message(ShopAuctionDraftOfferingReceived { card_ids });
    run_update(app);
}

fn send_shop_slots(app: &mut App, slots: Vec<Option<CardId>>) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotsReceived { slots });
    run_update(app);
}

fn run_update(app: &mut App) {
    app.update();
}

fn card_ids(start: u32, count: u32) -> Vec<CardId> {
    (start..start + count).map(CardId).collect()
}

fn node(app: &App, entity: Entity) -> &Node {
    app.world()
        .get::<Node>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have a Node"))
}

fn parent_of(app: &App, entity: Entity) -> Entity {
    app.world()
        .get::<ChildOf>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have a ChildOf parent"))
        .parent()
}

fn px(value: Val) -> f32 {
    match value {
        Val::Px(value) => value,
        other => panic!("expected Val::Px, got {other:?}"),
    }
}
