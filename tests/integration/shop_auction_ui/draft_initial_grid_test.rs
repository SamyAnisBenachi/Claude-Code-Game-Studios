use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    DraftInitialSlotCard, DraftInitialSlotState, ShopAuctionCardAcquiredReceived,
    ShopAuctionCardCatalog, ShopAuctionDraftHandView, ShopAuctionDraftOfferingReceived,
    ShopAuctionDraftReadyButtonClicked, ShopAuctionDraftSlotClicked, ShopAuctionUiEntities,
    ShopAuctionUiOutboundMessages, ShopAuctionUiPlugin, SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn sau_002_panel_waits_for_phase_and_offering_before_showing_grid() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(5, true);
    set_phase(&mut app, RoundPhase::DraftInitial);
    run_update(&mut app);

    assert_eq!(draft_panel_visibility(&app), Some(&Visibility::Hidden));
    assert_eq!(visible_slot_count(&app), 0);

    send_offering(&mut app, card_ids(1, 9));
    assert_eq!(draft_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(visible_slot_count(&app), 9);

    let mut app = app_in_session(5, true);
    send_offering(&mut app, card_ids(1, 9));
    assert_eq!(
        draft_panel_visibility(&app),
        Some(&Visibility::Hidden),
        "offering alone must not show the panel"
    );

    set_phase(&mut app, RoundPhase::DraftInitial);
    run_update(&mut app);
    assert_eq!(draft_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(visible_slot_count(&app), 9);
}

#[test]
fn sau_002_grid_sorts_by_rarity_descending_then_cost_descending() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(9, true);
    insert_catalog(
        &mut app,
        &[
            (1, Rarity::Common, 9),
            (2, Rarity::Legendary, 2),
            (3, Rarity::Epic, 7),
            (4, Rarity::Legendary, 8),
            (5, Rarity::Rare, 10),
            (6, Rarity::Uncommon, 6),
            (7, Rarity::Epic, 4),
            (8, Rarity::Rare, 3),
            (9, Rarity::Common, 1),
        ],
    );
    set_phase(&mut app, RoundPhase::DraftInitial);
    send_offering(&mut app, card_ids(1, 9));

    assert_eq!(
        slot_cards(&app),
        vec![
            CardId(4),
            CardId(2),
            CardId(3),
            CardId(7),
            CardId(5),
            CardId(8),
            CardId(6),
            CardId(1),
            CardId(9),
        ]
    );
}

#[test]
fn sau_002_valid_slot_click_sends_exactly_one_purchase_intent() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);
    let slot = draft_slot(&app, 0);
    let card_id = slot_card(&app, slot);

    click_slot(&mut app, slot);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.purchase_cards.len(), 1);
    assert_eq!(outbound.purchase_cards[0].card_id, card_id);
    assert_eq!(
        app.world().get::<DraftInitialSlotState>(slot),
        Some(&DraftInitialSlotState::Pending)
    );
}

#[test]
fn sau_002_insufficient_gold_suppresses_purchase_and_requests_gold_flash() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(0, true);
    let slot = draft_slot(&app, 0);

    click_slot(&mut app, slot);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert!(outbound.purchase_cards.is_empty());
    assert_eq!(outbound.gold_counter_flash_requests, 1);
    assert_eq!(
        app.world().get::<DraftInitialSlotState>(slot),
        Some(&DraftInitialSlotState::Available)
    );
}

#[test]
fn sau_002_hand_size_ten_locks_unowned_slots_and_shows_banner() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(5, true);
    app.world_mut()
        .resource_mut::<ShopAuctionDraftHandView>()
        .hand_size = 10;
    set_phase(&mut app, RoundPhase::DraftInitial);
    send_offering(&mut app, card_ids(1, 9));

    for slot in draft_slots(&app) {
        assert_eq!(
            app.world().get::<DraftInitialSlotState>(slot),
            Some(&DraftInitialSlotState::HandFullLocked)
        );
    }
    assert_eq!(
        app.world().get::<Visibility>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .draft_initial_hand_full_banner
        ),
        Some(&Visibility::Visible)
    );

    let slot = draft_slot(&app, 0);
    click_slot(&mut app, slot);
    assert!(app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .purchase_cards
        .is_empty());
}

#[test]
fn sau_002_card_acquired_plus_gold_update_marks_slot_purchased_with_overlay() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, false);
    let slot = draft_slot(&app, 0);
    let card_id = slot_card(&app, slot);

    click_slot(&mut app, slot);
    app.world_mut()
        .write_message(ShopAuctionCardAcquiredReceived { card_id });
    run_update(&mut app);
    assert_eq!(
        app.world().get::<DraftInitialSlotState>(slot),
        Some(&DraftInitialSlotState::Pending),
        "card acquisition alone must wait for the economy view update"
    );

    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .initialized = true;
    run_update(&mut app);

    assert_eq!(
        app.world().get::<DraftInitialSlotState>(slot),
        Some(&DraftInitialSlotState::Purchased)
    );
    assert_eq!(
        bought_overlay_visibility(&app, 0),
        Some(&Visibility::Visible)
    );
}

#[test]
fn sau_002_ready_and_retract_send_expected_signal_values() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);
    let ready = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_ready_button;

    click_ready(&mut app, ready);
    assert!(
        !app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .ready_signals[0]
            .retract
    );
    assert_eq!(
        app.world().get::<Text>(ready).map(|text| text.0.as_str()),
        Some("Retract Ready")
    );

    click_ready(&mut app, ready);
    let signals = &app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .ready_signals;
    assert_eq!(signals.len(), 2);
    assert_eq!(signals[1].retract, true);
    assert_eq!(
        app.world().get::<Text>(ready).map(|text| text.0.as_str()),
        Some("Ready")
    );
}

#[test]
fn sau_002_grid_remains_interactive_after_ready_until_phase_changes() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);
    let ready = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_ready_button;
    click_ready(&mut app, ready);

    let slot = draft_slot(&app, 0);
    click_slot(&mut app, slot);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .purchase_cards
            .len(),
        1
    );
}

#[test]
fn sau_002_placement_phase_dismisses_panel_and_blocks_purchase_sends() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);
    let slot = draft_slot(&app, 0);

    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);
    assert_eq!(draft_panel_visibility(&app), Some(&Visibility::Hidden));

    click_slot(&mut app, slot);
    assert!(app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .purchase_cards
        .is_empty());
}

#[test]
fn sau_002_pending_slot_visual_differs_from_available_within_one_tick() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);
    let baseline_slot = draft_slot(&app, 1);
    let pending_slot = draft_slot(&app, 0);

    let baseline_color = slot_background_color(&app, baseline_slot);

    click_slot(&mut app, pending_slot);

    assert_eq!(
        app.world().get::<DraftInitialSlotState>(pending_slot),
        Some(&DraftInitialSlotState::Pending),
        "click must transition slot to Pending state"
    );
    assert_eq!(
        app.world().get::<DraftInitialSlotState>(baseline_slot),
        Some(&DraftInitialSlotState::Available),
        "untouched slot must remain Available"
    );

    let pending_color = slot_background_color(&app, pending_slot);
    let post_baseline_color = slot_background_color(&app, baseline_slot);

    assert_eq!(
        post_baseline_color, baseline_color,
        "untouched Available slot must keep its baseline BackgroundColor"
    );
    assert_ne!(
        pending_color, baseline_color,
        "Pending slot BackgroundColor must differ from Available baseline within one tick of click"
    );
}

fn active_draft_app(gold: u32, economy_initialized: bool) -> App {
    let mut app = app_in_session(gold, economy_initialized);
    set_phase(&mut app, RoundPhase::DraftInitial);
    send_offering(&mut app, card_ids(1, 9));
    app
}

fn app_in_session(gold: u32, economy_initialized: bool) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    insert_catalog(
        &mut app,
        &[
            (1, Rarity::Common, 1),
            (2, Rarity::Common, 2),
            (3, Rarity::Common, 3),
            (4, Rarity::Common, 4),
            (5, Rarity::Common, 5),
            (6, Rarity::Common, 1),
            (7, Rarity::Common, 2),
            (8, Rarity::Common, 3),
            (9, Rarity::Common, 4),
        ],
    );
    app.insert_resource(PlayerEconomyView {
        gold,
        initialized: economy_initialized,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
}

fn insert_catalog(app: &mut App, cards: &[(u32, Rarity, u32)]) {
    app.insert_resource(ShopAuctionCardCatalog {
        cards: cards
            .iter()
            .map(|(id, rarity, cost)| {
                let card = test_card(*id, *rarity, *cost);
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
}

fn send_offering(app: &mut App, card_ids: Vec<CardId>) {
    app.world_mut()
        .write_message(ShopAuctionDraftOfferingReceived { card_ids });
    run_update(app);
}

fn click_slot(app: &mut App, slot: Entity) {
    app.world_mut()
        .write_message(ShopAuctionDraftSlotClicked { slot });
    run_update(app);
}

fn click_ready(app: &mut App, button: Entity) {
    app.world_mut()
        .write_message(ShopAuctionDraftReadyButtonClicked { button });
    run_update(app);
}

fn run_update(app: &mut App) {
    app.update();
}

fn card_ids(start: u32, count: u32) -> Vec<CardId> {
    (start..start + count).map(CardId).collect()
}

fn draft_panel_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .draft_offering_panel,
    )
}

fn draft_slots(app: &App) -> [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT] {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_slots
}

fn draft_slot(app: &App, index: usize) -> Entity {
    draft_slots(app)[index]
}

fn visible_slot_count(app: &App) -> usize {
    draft_slots(app)
        .iter()
        .filter(|slot| app.world().get::<Visibility>(**slot) == Some(&Visibility::Visible))
        .count()
}

fn slot_cards(app: &App) -> Vec<CardId> {
    draft_slots(app)
        .iter()
        .map(|slot| {
            app.world()
                .get::<DraftInitialSlotCard>(*slot)
                .expect("slot should have a card")
                .0
        })
        .collect()
}

fn slot_card(app: &App, slot: Entity) -> CardId {
    app.world()
        .get::<DraftInitialSlotCard>(slot)
        .expect("slot should have a card")
        .0
}

fn bought_overlay_visibility(app: &App, slot_index: u8) -> Option<&Visibility> {
    let overlay = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_bought_overlays[slot_index as usize];
    app.world().get::<Visibility>(overlay)
}

fn slot_background_color(app: &App, slot: Entity) -> Color {
    app.world()
        .get::<BackgroundColor>(slot)
        .expect("slot should have a BackgroundColor component")
        .0
}
