// PROMPT 1150 (S17-UI-CLICK-PICKING-BACKEND-001) regression coverage.
//
// Closes the structural test-coverage gap surfaced by PROMPT 1128: every other
// integration test under `tests/integration/shop_auction_ui/` bypasses the
// production `Interaction` → `*Clicked` pipeline by writing the internal
// `ShopAuction*Clicked` events directly via `World::write_message`. As a
// result the entire shop / auction click hand-off — driven by
// `handle_shop_auction_control_interactions_system` (shop slots, refresh,
// ready, draft slots) and `handle_auction_bid_button_interactions_system`
// (bid increments) — has been silently uncovered, and Sprint 16/17 work that
// quietly disabled the picking backend in default builds (PROMPT 1109 added
// `ui_picking` as opt-in) sailed past CI.
//
// These tests assert the production handler emits the expected internal
// `*Clicked` message when an `Interaction` transitions to `Pressed` on a
// real spawned entity. They do not exercise `bevy_picking` itself (the
// `UiPickingPlugin` requires a window / DefaultPlugins stack that is out
// of scope for MinimalPlugins-based integration tests) — they prove that
// once the picking backend hands a `Pressed` interaction to a `Button`
// entity, the click is routed correctly.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    AuctionBidButton, ShopAuctionAuctionCardReceived, ShopAuctionBidButtonClicked,
    ShopAuctionCardCatalog, ShopAuctionDraftReadyButtonClicked, ShopAuctionDraftSlotClicked,
    ShopAuctionShopReadyButtonClicked, ShopAuctionShopRefreshClicked, ShopAuctionShopSlotClicked,
    ShopAuctionShopSlotsReceived, ShopAuctionUiEntities, ShopAuctionUiPlugin,
    SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// Sink for messages emitted by the production Interaction → click pipeline.
/// Each `MessageReader` has an independent cursor in Bevy 0.18, so the
/// production `handle_*` system and this capture system both observe the
/// same messages in the same frame.
#[derive(Resource, Default, Debug)]
struct CapturedClicks {
    shop_slots: Vec<Entity>,
    shop_refresh: Vec<Entity>,
    shop_ready: Vec<Entity>,
    draft_slots: Vec<Entity>,
    draft_ready: Vec<Entity>,
    bid_buttons: Vec<Entity>,
}

fn capture_clicks_system(
    mut captured: ResMut<CapturedClicks>,
    mut shop_slots: MessageReader<ShopAuctionShopSlotClicked>,
    mut shop_refresh: MessageReader<ShopAuctionShopRefreshClicked>,
    mut shop_ready: MessageReader<ShopAuctionShopReadyButtonClicked>,
    mut draft_slots: MessageReader<ShopAuctionDraftSlotClicked>,
    mut draft_ready: MessageReader<ShopAuctionDraftReadyButtonClicked>,
    mut bid_buttons: MessageReader<ShopAuctionBidButtonClicked>,
) {
    for event in shop_slots.read() {
        captured.shop_slots.push(event.slot);
    }
    for event in shop_refresh.read() {
        captured.shop_refresh.push(event.button);
    }
    for event in shop_ready.read() {
        captured.shop_ready.push(event.button);
    }
    for event in draft_slots.read() {
        captured.draft_slots.push(event.slot);
    }
    for event in draft_ready.read() {
        captured.draft_ready.push(event.button);
    }
    for event in bid_buttons.read() {
        captured.bid_buttons.push(event.button);
    }
}

#[test]
fn prompt_1150_shop_slot_pressed_interaction_emits_shop_slot_click() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftShop);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    let slot = app.world().resource::<ShopAuctionUiEntities>().shop_slots[0];
    press(&mut app, slot);
    app.update();

    let captured = app.world().resource::<CapturedClicks>();
    assert_eq!(
        captured.shop_slots,
        vec![slot],
        "Pressed Interaction on a ShopSlotIndex entity must produce \
         exactly one ShopAuctionShopSlotClicked through \
         handle_shop_auction_control_interactions_system"
    );
    assert!(captured.shop_refresh.is_empty());
    assert!(captured.shop_ready.is_empty());
    assert!(captured.draft_slots.is_empty());
}

#[test]
fn prompt_1150_shop_refresh_pressed_interaction_emits_refresh_click() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftShop);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    let refresh = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .shop_refresh_button;
    press(&mut app, refresh);
    app.update();

    let captured = app.world().resource::<CapturedClicks>();
    assert_eq!(captured.shop_refresh, vec![refresh]);
    assert!(captured.shop_slots.is_empty());
}

#[test]
fn prompt_1150_shop_ready_pressed_interaction_emits_ready_click() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftShop);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    let ready = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .shop_ready_button;
    press(&mut app, ready);
    app.update();

    let captured = app.world().resource::<CapturedClicks>();
    assert_eq!(captured.shop_ready, vec![ready]);
    assert!(captured.shop_slots.is_empty());
}

#[test]
fn prompt_1150_draft_initial_slot_pressed_interaction_emits_draft_slot_click() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftInitial);

    let slot = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_slots[0];
    press(&mut app, slot);
    app.update();

    let captured = app.world().resource::<CapturedClicks>();
    assert_eq!(
        captured.draft_slots,
        vec![slot],
        "Pressed Interaction on a DraftInitialSlotIndex entity must \
         produce a ShopAuctionDraftSlotClicked"
    );
}

#[test]
fn prompt_1150_draft_initial_ready_pressed_interaction_emits_draft_ready_click() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftInitial);

    let ready = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_ready_button;
    press(&mut app, ready);
    app.update();

    let captured = app.world().resource::<CapturedClicks>();
    assert_eq!(captured.draft_ready, vec![ready]);
}

#[test]
fn prompt_1150_auction_bid_button_pressed_interaction_emits_bid_click() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftAuction);
    // `handle_auction_bid_button_interactions_system` filters on
    // `With<AuctionBidButton>` and `Changed<Interaction>` regardless of
    // whether an auction card is actively bidding — the click hand-off is
    // strictly a UI plumbing concern. The card-id condition is enforced
    // by the downstream `handle_auction_bid_button_click_system` (covered
    // by `auction_bid_buttons_test.rs`).
    let bid_buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    let bid_target = bid_buttons[1];
    assert!(
        app.world().get::<AuctionBidButton>(bid_target).is_some(),
        "bid button entity must carry the AuctionBidButton marker"
    );

    press(&mut app, bid_target);
    app.update();

    let captured = app.world().resource::<CapturedClicks>();
    assert_eq!(
        captured.bid_buttons,
        vec![bid_target],
        "Pressed Interaction on an AuctionBidButton entity must produce \
         exactly one ShopAuctionBidButtonClicked through \
         handle_auction_bid_button_interactions_system"
    );
}

#[test]
fn prompt_1150_only_pressed_interaction_emits_click_hovered_is_ignored() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftShop);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    let slot = app.world().resource::<ShopAuctionUiEntities>().shop_slots[0];

    set_interaction(&mut app, slot, Interaction::Hovered);
    app.update();
    assert!(
        app.world()
            .resource::<CapturedClicks>()
            .shop_slots
            .is_empty(),
        "Hovered must not produce a click — only Pressed counts"
    );

    set_interaction(&mut app, slot, Interaction::Pressed);
    app.update();
    assert_eq!(
        app.world().resource::<CapturedClicks>().shop_slots,
        vec![slot],
        "Pressed after Hovered must produce one click"
    );

    set_interaction(&mut app, slot, Interaction::None);
    app.update();
    assert_eq!(
        app.world().resource::<CapturedClicks>().shop_slots,
        vec![slot],
        "Releasing back to None must not produce a second click"
    );
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.init_resource::<CapturedClicks>();
    // Run after the production handlers so we observe the messages they
    // produced this frame. `MessageReader` cursors are per-system, so this
    // does not steal messages from the production drainers.
    app.add_systems(Last, capture_clicks_system);
    insert_catalog(&mut app);
    app.insert_resource(PlayerEconomyView {
        gold: 100,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
    app.update();
    // Pre-arm an active auction-card path so AuctionBidButton entities get
    // wired during DraftAuction. The handler-under-test doesn't actually
    // require an active auction (it's pure UI plumbing) — this is here so
    // the bid-button bevy_ui Button entities exist and carry the marker.
    if phase == RoundPhase::DraftAuction {
        app.world_mut()
            .write_message(ShopAuctionAuctionCardReceived {
                card_id: CardId(1),
                starting_price: 1,
                timer_duration_ms: 20_000,
            });
        app.update();
    }
}

fn send_shop_slots(app: &mut App, slots: Vec<Option<CardId>>) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotsReceived { slots });
    app.update();
}

fn press(app: &mut App, entity: Entity) {
    set_interaction(app, entity, Interaction::Pressed);
}

fn set_interaction(app: &mut App, entity: Entity, value: Interaction) {
    let mut interaction = app
        .world_mut()
        .get_mut::<Interaction>(entity)
        .expect("button entity must carry Interaction (Required Components on Button)");
    *interaction = value;
}

fn insert_catalog(app: &mut App) {
    let mut cards = HashMap::new();
    for id in 1..=9u32 {
        let card = test_card(id, Rarity::Common, id.min(3));
        cards.insert(card.id, card);
    }
    app.insert_resource(ShopAuctionCardCatalog { cards });
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

// `SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT` is re-exported but currently
// unused in this file. Keep the import to document the slot fanout for
// future test extension.
const _: usize = SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT;
