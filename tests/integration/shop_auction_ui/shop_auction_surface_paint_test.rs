//! PROMPT 1085 — DraftShop and DraftAuction surface-paint regression.
//!
//! Audit `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
//! AUDIT-1076-04 + AUDIT-1076-13:
//!
//! * Shop tiles rendered as an empty black slab even after `S2CShopSlots`
//!   arrived — no card name, no cost, no buy affordance.
//! * Auction featured card rendered as the same black slab — no current
//!   price, no time-left, no bid controls.
//! * Shop clicks gave no visible "you cannot buy this" feedback when gold
//!   was insufficient or the hand was full.
//!
//! The repair adds three sub-nodes that the player can never miss:
//!
//! * `ShopSlotAffordanceLabel` — child of each shop slot. Always carries
//!   `"BUY · Ng"` when the slot is purchasable and the player can afford
//!   it, and the human-readable disabled reason (`"LOCKED · Need Ng"`,
//!   `"LOCKED · Hand full"`, `"PENDING..."`) otherwise.
//! * `AuctionFeaturedCardPriceLabel` — child of the featured auction
//!   card. Always carries `"Bid: Ng"` while the card is live or settling.
//! * `AuctionFeaturedCardTimerLabel` — sibling of the price label.
//!   Always carries `"{N}s left"` (or the panel-state status copy) while
//!   the card is live or settling.
//!
//! This test drives `S2CShopSlots(slots_len=3)` and `S2CAuctionCard`,
//! then asserts each of those labels is `Visibility::Visible` and carries
//! non-empty copy. It also drives the click intent paths that previously
//! produced no feedback and asserts the disabled-reason copy survives.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    AuctionFeaturedCardPriceLabel, AuctionFeaturedCardTimerLabel, ShopAuctionAuctionCardReceived,
    ShopAuctionCardCatalog, ShopAuctionDraftHandView, ShopAuctionShopSlotClicked,
    ShopAuctionShopSlotsReceived, ShopAuctionUiEntities, ShopAuctionUiOutboundMessages,
    ShopAuctionUiPlugin, ShopSlotAffordanceLabel, SHOP_AUCTION_UI_SHOP_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// AUDIT-1076-04 part 1 — three shop tiles must paint non-empty content
/// when `S2CShopSlots(slots_len=3)` arrives. The slot wells themselves
/// must be `Visibility::Visible`, and each must carry a child
/// `ShopSlotAffordanceLabel` with the buy / locked-reason copy.
#[test]
fn prompt_1085_shop_slots_paint_visible_tiles_with_affordance_copy() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(/* gold */ 5);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    for (index, slot_entity) in entities.shop_slots.into_iter().enumerate() {
        assert_eq!(
            app.world().get::<Visibility>(slot_entity).copied(),
            Some(Visibility::Visible),
            "shop slot {index} root must be Visible after S2CShopSlots; \
             AUDIT-1076-04 observed an empty black slab even when slots \
             arrived",
        );
    }

    let affordance_texts = collect_shop_slot_affordance_texts(&app);
    assert_eq!(affordance_texts.len(), SHOP_AUCTION_UI_SHOP_SLOT_COUNT);
    for (index, copy) in affordance_texts.iter().enumerate() {
        assert!(
            !copy.is_empty(),
            "shop slot {index} affordance label must be non-empty; \
             AUDIT-1076-04 + AUDIT-1076-13 required a visible buy / \
             disabled-reason tag so the player can see purchase intent. \
             Got: {copy:?}",
        );
        assert!(
            copy.contains("BUY"),
            "shop slot {index} affordance must read as BUY when the \
             player has 5g and the card costs ≤ 3g; got {copy:?}",
        );
    }
}

/// AUDIT-1076-13 — when gold is insufficient, clicking the slot must NOT
/// send `C2SPurchaseCard` and the affordance label must surface the
/// `"LOCKED · Need Ng"` disabled reason so the player can read why the
/// click did not buy.
#[test]
fn prompt_1085_unaffordable_shop_slot_surfaces_locked_need_n_gold_copy() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(/* gold */ 0);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    // Card cost = 3g per test_card(id, _, id + 2) — gold = 0 makes the
    // tile unaffordable.
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );

    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    app.world_mut().write_message(ShopAuctionShopSlotClicked {
        slot: entities.shop_slots[0],
    });
    run_update(&mut app);

    assert!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .purchase_cards
            .is_empty(),
        "unaffordable click must NOT dispatch C2SPurchaseCard (the \
         existing flash signal is preserved, but the slot must not \
         enter PendingPurchase)",
    );

    let copy = shop_slot_affordance_text(&app, 0);
    assert!(
        copy.contains("LOCKED"),
        "unaffordable slot affordance must say LOCKED; got {copy:?}",
    );
    assert!(
        copy.contains("Need"),
        "unaffordable slot affordance must include the 'Need Ng' \
         disabled-reason copy; got {copy:?}",
    );
}

/// AUDIT-1076-13 — when the hand is full, the slot transitions to
/// `HandFullLocked` and the affordance label must read as such so the
/// player sees the disabled reason without consulting the HUD banner.
#[test]
fn prompt_1085_hand_full_shop_slot_surfaces_hand_full_copy() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(/* gold */ 5);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    app.world_mut()
        .resource_mut::<ShopAuctionDraftHandView>()
        .hand_size = 10;
    run_update(&mut app);

    let copy = shop_slot_affordance_text(&app, 0);
    assert!(
        copy.contains("LOCKED"),
        "hand-full slot affordance must say LOCKED; got {copy:?}",
    );
    assert!(
        copy.contains("Hand full"),
        "hand-full slot affordance must spell out the disabled reason; \
         got {copy:?}",
    );
}

/// AUDIT-1076-04 part 2 — auction featured card must paint with a
/// visible price label and a visible time-left label as soon as
/// `S2CAuctionCard` arrives and the panel goes Active.
#[test]
fn prompt_1085_auction_featured_card_paints_price_and_timer_labels() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(/* gold */ 6);
    set_phase(&mut app, RoundPhase::DraftAuction, 30_000);
    send_auction_card(&mut app, CardId(2), /* starting_price */ 4, 20_000);

    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    assert_eq!(
        app.world()
            .get::<Visibility>(entities.auction_featured_card)
            .copied(),
        Some(Visibility::Visible),
        "auction featured card root must be Visible while the panel is \
         Active",
    );
    assert_eq!(
        app.world()
            .get::<Visibility>(entities.auction_featured_card_price_label)
            .copied(),
        Some(Visibility::Visible),
        "auction featured card price label must be Visible",
    );
    assert_eq!(
        app.world()
            .get::<Visibility>(entities.auction_featured_card_timer_label)
            .copied(),
        Some(Visibility::Visible),
        "auction featured card timer label must be Visible",
    );

    let price_text = auction_price_label_text(&app);
    assert!(
        price_text.contains("Bid"),
        "auction featured price label must lead with 'Bid:'; got {price_text:?}",
    );
    assert!(
        price_text.contains("4g"),
        "auction featured price label must include the current price \
         (4g for the test card); got {price_text:?}",
    );

    let timer_text = auction_timer_label_text(&app);
    assert!(
        !timer_text.is_empty(),
        "auction featured timer label must be non-empty while panel is \
         Active; got {timer_text:?}",
    );
    assert!(
        timer_text.contains("left"),
        "auction featured timer label must read '{{N}}s left' while \
         counting down; got {timer_text:?}",
    );
}

/// Sanity — after the auction panel hides (e.g., phase changes to a
/// non-auction state), the price + timer labels must stop advertising
/// the stale auction so the player does not see ghost copy.
#[test]
fn prompt_1085_auction_featured_card_labels_clear_when_panel_hides() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(/* gold */ 6);
    set_phase(&mut app, RoundPhase::DraftAuction, 30_000);
    send_auction_card(&mut app, CardId(1), 3, 20_000);

    // Sanity: while active, both labels carry copy.
    assert!(!auction_price_label_text(&app).is_empty());
    assert!(!auction_timer_label_text(&app).is_empty());

    set_phase(&mut app, RoundPhase::Placement, 0);
    assert_eq!(auction_price_label_text(&app), "");
    assert_eq!(auction_timer_label_text(&app), "");
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn app_in_session(gold: u32) -> App {
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
        cards: (1..=3)
            .map(|id| {
                let card = test_card(id, Rarity::Common, id + 2);
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

fn set_phase(app: &mut App, phase: RoundPhase, timer_duration_ms: u32) {
    let round = app.world().resource::<CurrentClientPhase>().round + 1;
    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        current.phase = phase;
        current.round = round;
    }
    {
        let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
        phase_view.phase = phase;
        phase_view.round_number = round;
        phase_view.timer_duration_ms = timer_duration_ms;
    }
    run_update(app);
}

fn send_shop_slots(app: &mut App, slots: Vec<Option<CardId>>) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotsReceived { slots });
    run_update(app);
}

fn send_auction_card(app: &mut App, card_id: CardId, starting_price: u32, timer_duration_ms: u32) {
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id,
            starting_price,
            timer_duration_ms,
        });
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn collect_shop_slot_affordance_texts(app: &App) -> Vec<String> {
    let entities = app.world().resource::<ShopAuctionUiEntities>();
    entities
        .shop_slot_affordance_labels
        .iter()
        .map(|entity| {
            app.world()
                .get::<Text>(*entity)
                .map(|text| text.0.clone())
                .unwrap_or_default()
        })
        .collect()
}

fn shop_slot_affordance_text(app: &App, index: usize) -> String {
    let entity = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .shop_slot_affordance_labels[index];
    // Sanity: the entity must carry the marker so future refactors don't
    // accidentally repurpose this slot.
    assert!(
        app.world().get::<ShopSlotAffordanceLabel>(entity).is_some(),
        "shop slot affordance entity must keep the marker component",
    );
    app.world()
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .unwrap_or_default()
}

fn auction_price_label_text(app: &App) -> String {
    let entity = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_featured_card_price_label;
    assert!(
        app.world()
            .get::<AuctionFeaturedCardPriceLabel>(entity)
            .is_some(),
        "auction featured price label entity must keep the marker",
    );
    app.world()
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .unwrap_or_default()
}

fn auction_timer_label_text(app: &App) -> String {
    let entity = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_featured_card_timer_label;
    assert!(
        app.world()
            .get::<AuctionFeaturedCardTimerLabel>(entity)
            .is_some(),
        "auction featured timer label entity must keep the marker",
    );
    app.world()
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .unwrap_or_default()
}
