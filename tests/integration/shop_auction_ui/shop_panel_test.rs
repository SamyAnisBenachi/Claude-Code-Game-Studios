use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{
    CardDisplayArtAsset, CardDisplayArtFallback, CardDisplayArtFallbackReason,
};
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    ShopAuctionCardCatalog, ShopAuctionDraftHandView, ShopAuctionShopCardAcquiredReceived,
    ShopAuctionShopReadyButtonClicked, ShopAuctionShopRefreshClicked, ShopAuctionShopSlotClicked,
    ShopAuctionShopSlotsReceived, ShopAuctionShopState, ShopAuctionUiEntities,
    ShopAuctionUiOutboundMessages, ShopAuctionUiPlugin, ShopRefreshButtonState, ShopSlotCard,
    ShopSlotState, SHOP_AUCTION_UI_SHOP_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn sau_003_non_auction_shop_waits_for_phase_and_slots_before_interactive() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(5, true);

    set_phase(&mut app, RoundPhase::DraftShop);
    // PROMPT 1042 — shop_panel chrome (title + empty-state copy) appears
    // as soon as the phase changes so the player never sees a blank
    // playfield that looks like Placement. The slot wells stay hidden
    // until `S2CShopSlots` arrives.
    assert_eq!(shop_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        shop_phase_title_visibility(&app),
        Some(&Visibility::Visible),
        "phase title must render in DraftShop even without offers"
    );
    assert_eq!(
        shop_empty_state_visibility(&app),
        Some(&Visibility::Visible),
        "empty-state copy must render while waiting for shop slots"
    );
    assert_eq!(visible_shop_slot_count(&app), 0);

    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    assert_eq!(shop_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        shop_empty_state_visibility(&app),
        Some(&Visibility::Hidden),
        "empty-state must hide once slots are loaded"
    );
    assert_eq!(
        visible_shop_slot_count(&app),
        SHOP_AUCTION_UI_SHOP_SLOT_COUNT
    );

    let mut app = app_in_session(5, true);
    set_phase(&mut app, RoundPhase::Resolution);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    assert_eq!(
        shop_panel_visibility(&app),
        Some(&Visibility::Hidden),
        "Resolution phase must not show the DRAFT_SHOP panel even if slots are buffered"
    );

    set_phase(&mut app, RoundPhase::DraftShop);
    assert_eq!(shop_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))]
    );
}

/// PROMPT 1042 — explicit empty-state surface for the DraftShop race
/// window: phase title visible immediately, empty-state copy visible
/// while slots are pending, slot wells hidden, refresh / ready
/// affordances disabled until offers exist.
#[test]
fn prompt_1042_draftshop_renders_empty_state_until_slots_load() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(5, true);
    set_phase(&mut app, RoundPhase::DraftShop);

    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    assert_eq!(
        app.world().get::<Visibility>(entities.shop_panel).copied(),
        Some(Visibility::Visible)
    );
    assert_eq!(
        app.world()
            .get::<Visibility>(entities.shop_phase_title)
            .copied(),
        Some(Visibility::Visible)
    );
    assert_eq!(
        app.world()
            .get::<Visibility>(entities.shop_empty_state)
            .copied(),
        Some(Visibility::Visible)
    );
    assert_eq!(
        app.world()
            .get::<Visibility>(entities.shop_refresh_button)
            .copied(),
        Some(Visibility::Hidden),
        "refresh affordance must stay hidden until shop slots load"
    );
    assert_eq!(
        app.world()
            .get::<Visibility>(entities.shop_ready_button)
            .copied(),
        Some(Visibility::Hidden),
        "ready affordance must stay hidden until shop slots load"
    );
    assert_eq!(
        app.world()
            .get::<Text>(entities.shop_phase_title)
            .map(|t| t.0.as_str()),
        Some("SHOP")
    );
    assert_eq!(
        app.world()
            .get::<Text>(entities.shop_empty_state)
            .map(|t| t.0.as_str()),
        Some("Waiting for shop offers...")
    );
}

#[test]
fn sau_003_renders_three_server_supplied_shop_slots_and_empty_state() {
    test_helpers::init_test_tracing();
    let app = active_shop_app(5, true, vec![Some(CardId(1)), None, Some(CardId(2))]);

    assert_eq!(shop_slots(&app).len(), SHOP_AUCTION_UI_SHOP_SLOT_COUNT);
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(1)), None, Some(CardId(2))]
    );
    assert_eq!(
        shop_slot_states(&app),
        vec![
            ShopSlotState::Available,
            ShopSlotState::Empty,
            ShopSlotState::Available
        ]
    );
    assert_eq!(
        visible_shop_slot_count(&app),
        SHOP_AUCTION_UI_SHOP_SLOT_COUNT
    );
}

#[test]
fn prompt_1462_shop_slots_are_visible_button_wells_with_separate_affordance_copy() {
    test_helpers::init_test_tracing();
    let app = active_shop_app(
        5,
        true,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    for (index, slot) in entities.shop_slots.into_iter().enumerate() {
        assert_eq!(
            app.world().get::<Visibility>(slot).copied(),
            Some(Visibility::Visible),
            "shop slot {index} must remain visible after S2CShopSlots"
        );
        assert!(
            app.world().get::<Button>(slot).is_some(),
            "shop slot {index} must be semantically clickable"
        );
        let background = app
            .world()
            .get::<BackgroundColor>(slot)
            .expect("shop slot must carry a readable background");
        assert!(
            background.0.alpha() > 0.80,
            "shop slot {index} needs an opaque well so offers are not lost on panel chrome"
        );

        let affordance = entities.shop_slot_affordance_labels[index];
        assert!(
            app.world().get::<Button>(affordance).is_none(),
            "shop slot {index} affordance copy must remain an info label, not a nested button"
        );
        let affordance_text = app
            .world()
            .get::<Text>(affordance)
            .map(|text| text.0.as_str());
        assert!(
            affordance_text.is_some_and(|text| text.starts_with("BUY")),
            "shop slot {index} affordance must expose the buy cost; got {affordance_text:?}"
        );
    }
}

#[test]
fn sau_asset_loop_shop_slots_resolve_display_art_or_text_fallback() {
    test_helpers::init_test_tracing();
    // CardId(99) is intentionally absent from the test catalog (1..=6); the
    // catalog miss feeds None into apply_card_display_art, producing the
    // MissingDisplayAsset fallback on slot 1.
    let app = active_shop_app(5, true, vec![Some(CardId(1)), Some(CardId(99)), None]);
    let slots = shop_slots(&app);

    assert_eq!(
        app.world().get::<CardDisplayArtAsset>(slots[0]),
        Some(&CardDisplayArtAsset {
            path: "art/cards/display/card_iop_knight_001_art_display.png".to_string()
        })
    );
    assert_eq!(
        app.world().get::<CardDisplayArtFallback>(slots[1]),
        Some(&CardDisplayArtFallback {
            reason: CardDisplayArtFallbackReason::MissingDisplayAsset
        })
    );
    assert!(app.world().get::<CardDisplayArtAsset>(slots[2]).is_none());
}

#[test]
fn sau_003_purchase_clicks_send_only_valid_affordable_non_empty_slots() {
    test_helpers::init_test_tracing();
    let mut app = active_shop_app(
        10,
        true,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    let slots = shop_slots(&app);

    app.world_mut()
        .write_message(ShopAuctionShopSlotClicked { slot: slots[0] });
    app.world_mut()
        .write_message(ShopAuctionShopSlotClicked { slot: slots[1] });
    run_update(&mut app);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.purchase_cards.len(), 2);
    assert_eq!(outbound.purchase_cards[0].card_id, CardId(1));
    assert_eq!(outbound.purchase_cards[1].card_id, CardId(2));
    assert_eq!(
        slot_state(&app, slots[0]),
        Some(&ShopSlotState::PendingPurchase)
    );
    assert_eq!(
        slot_state(&app, slots[1]),
        Some(&ShopSlotState::PendingPurchase)
    );
    assert_eq!(slot_state(&app, slots[2]), Some(&ShopSlotState::Available));

    let mut app = active_shop_app(0, true, vec![Some(CardId(1)), None, Some(CardId(3))]);
    let slots = shop_slots(&app);
    click_shop_slot(&mut app, slots[0]);
    click_shop_slot(&mut app, slots[1]);
    assert!(app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .purchase_cards
        .is_empty());
}

#[test]
fn sau_003_refresh_disables_same_frame_and_counts_only_confirmed_slots() {
    test_helpers::init_test_tracing();
    let mut app = active_shop_app(
        5,
        true,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    let refresh_button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .shop_refresh_button;

    app.world_mut()
        .write_message(ShopAuctionShopRefreshClicked {
            button: refresh_button,
        });
    app.world_mut()
        .write_message(ShopAuctionShopRefreshClicked {
            button: refresh_button,
        });
    run_update(&mut app);

    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .refresh_shops
            .len(),
        1
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionShopState>()
            .refresh_count_this_draft,
        0,
        "send alone must not advance refresh count"
    );
    assert_eq!(
        app.world().get::<ShopRefreshButtonState>(refresh_button),
        Some(&ShopRefreshButtonState { enabled: false })
    );

    run_update(&mut app);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionShopState>()
            .refresh_count_this_draft,
        0,
        "timeout/failure without S2CShopSlots must not advance refresh count"
    );

    send_shop_slots(
        &mut app,
        vec![Some(CardId(4)), Some(CardId(5)), Some(CardId(6))],
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionShopState>()
            .refresh_count_this_draft,
        1
    );
    assert_eq!(
        app.world()
            .get::<Text>(refresh_button)
            .map(|text| text.0.as_str()),
        Some("Refresh (2g)")
    );
}

#[test]
fn sau_003_hand_full_locks_slots_but_keeps_affordable_refresh_enabled() {
    test_helpers::init_test_tracing();
    let mut app = active_shop_app(
        1,
        true,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    app.world_mut()
        .resource_mut::<ShopAuctionDraftHandView>()
        .hand_size = 10;
    run_update(&mut app);

    assert_eq!(
        shop_slot_states(&app),
        vec![
            ShopSlotState::HandFullLocked,
            ShopSlotState::HandFullLocked,
            ShopSlotState::HandFullLocked
        ]
    );
    let refresh_button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .shop_refresh_button;
    assert_eq!(
        app.world().get::<ShopRefreshButtonState>(refresh_button),
        Some(&ShopRefreshButtonState { enabled: true })
    );

    let first_slot = shop_slots(&app)[0];
    click_shop_slot(&mut app, first_slot);
    assert!(app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .purchase_cards
        .is_empty());

    click_refresh(&mut app, refresh_button);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .refresh_shops
            .len(),
        1
    );
}

#[test]
fn sau_003_ready_retract_sends_once_per_click_and_shop_stays_interactive() {
    test_helpers::init_test_tracing();
    let mut app = active_shop_app(
        5,
        true,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    click_shop_ready(&mut app, entities.shop_ready_button);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .ready_signals
            .len(),
        1
    );
    assert!(
        !app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .ready_signals[0]
            .retract
    );
    assert_eq!(
        app.world()
            .get::<Text>(entities.shop_ready_button)
            .map(|text| text.0.as_str()),
        Some("Retract Ready")
    );

    click_shop_slot(&mut app, entities.shop_slots[0]);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .purchase_cards
            .len(),
        1,
        "Ready must not disable purchases before PLACEMENT"
    );

    click_shop_ready(&mut app, entities.shop_ready_button);
    let signals = &app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .ready_signals;
    assert_eq!(signals.len(), 2);
    assert!(signals[1].retract);
}

#[test]
fn sau_003_placement_dismisses_panel_blocks_sends_and_ignores_late_confirmation() {
    test_helpers::init_test_tracing();
    let mut app = active_shop_app(
        5,
        true,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    click_shop_slot(&mut app, entities.shop_slots[0]);
    assert_eq!(
        slot_state(&app, entities.shop_slots[0]),
        Some(&ShopSlotState::PendingPurchase)
    );

    set_phase(&mut app, RoundPhase::Placement);
    assert_eq!(shop_panel_visibility(&app), Some(&Visibility::Hidden));
    assert_eq!(
        slot_state(&app, entities.shop_slots[0]),
        Some(&ShopSlotState::Available),
        "phase transition restores pre-click visual state"
    );
    assert_eq!(app.world().resource::<PlayerEconomyView>().gold, 5);

    click_shop_slot(&mut app, entities.shop_slots[1]);
    click_refresh(&mut app, entities.shop_refresh_button);
    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.purchase_cards.len(), 1);
    assert!(outbound.refresh_shops.is_empty());

    app.world_mut()
        .write_message(ShopAuctionShopCardAcquiredReceived { card_id: CardId(1) });
    run_update(&mut app);
    assert_eq!(
        slot_state(&app, entities.shop_slots[0]),
        Some(&ShopSlotState::Available)
    );
    assert_eq!(app.world().resource::<PlayerEconomyView>().gold, 5);
}

#[test]
fn sau_003_auction_shop_slots_buffer_until_draft_shop() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(5, true);
    set_phase(&mut app, RoundPhase::DraftAuction);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    assert_eq!(shop_panel_visibility(&app), Some(&Visibility::Hidden));
    assert_eq!(shop_slot_cards(&app), vec![None, None, None]);

    set_phase(&mut app, RoundPhase::DraftShop);
    assert_eq!(shop_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))]
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionShopState>()
            .refresh_count_this_draft,
        0,
        "auction-buffered auto slots must not count as a manual refresh"
    );
}

fn active_shop_app(gold: u32, economy_initialized: bool, slots: Vec<Option<CardId>>) -> App {
    let mut app = app_in_session(gold, economy_initialized);
    set_phase(&mut app, RoundPhase::DraftShop);
    send_shop_slots(&mut app, slots);
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
    insert_catalog(&mut app);
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

fn insert_catalog(app: &mut App) {
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=6)
            .map(|id| {
                let card = test_card(id, Rarity::Common, id.min(3));
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
        art_id: if id == 1 {
            "iop_knight_001".to_string()
        } else {
            format!("test_{id}")
        },
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
    run_update(app);
}

fn send_shop_slots(app: &mut App, slots: Vec<Option<CardId>>) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotsReceived { slots });
    run_update(app);
}

fn click_shop_slot(app: &mut App, slot: Entity) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotClicked { slot });
    run_update(app);
}

fn click_refresh(app: &mut App, button: Entity) {
    app.world_mut()
        .write_message(ShopAuctionShopRefreshClicked { button });
    run_update(app);
}

fn click_shop_ready(app: &mut App, button: Entity) {
    app.world_mut()
        .write_message(ShopAuctionShopReadyButtonClicked { button });
    run_update(app);
}

fn run_update(app: &mut App) {
    app.update();
}

fn shop_slots(app: &App) -> [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT] {
    app.world().resource::<ShopAuctionUiEntities>().shop_slots
}

fn shop_panel_visibility(app: &App) -> Option<&Visibility> {
    app.world()
        .get::<Visibility>(app.world().resource::<ShopAuctionUiEntities>().shop_panel)
}

fn shop_phase_title_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .shop_phase_title,
    )
}

fn shop_empty_state_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .shop_empty_state,
    )
}

fn visible_shop_slot_count(app: &App) -> usize {
    shop_slots(app)
        .iter()
        .filter(|slot| app.world().get::<Visibility>(**slot) == Some(&Visibility::Visible))
        .count()
}

fn shop_slot_cards(app: &App) -> Vec<Option<CardId>> {
    shop_slots(app)
        .iter()
        .map(|slot| app.world().get::<ShopSlotCard>(*slot).map(|card| card.0))
        .collect()
}

fn shop_slot_states(app: &App) -> Vec<ShopSlotState> {
    shop_slots(app)
        .iter()
        .map(|slot| {
            *app.world()
                .get::<ShopSlotState>(*slot)
                .expect("shop slot should have a state")
        })
        .collect()
}

fn slot_state(app: &App, slot: Entity) -> Option<&ShopSlotState> {
    app.world().get::<ShopSlotState>(slot)
}
