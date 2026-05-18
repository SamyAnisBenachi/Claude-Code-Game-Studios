use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::asset_wiring::{
    CardDisplayArtAsset, CardDisplayArtFallback, CardDisplayArtFallbackReason,
};
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    AuctionTimerBarState, ShopAuctionAuctionCardReceived, ShopAuctionAuctionPanelState,
    ShopAuctionAuctionState, ShopAuctionCardCatalog, ShopAuctionShopRefreshClicked,
    ShopAuctionShopSlotClicked, ShopAuctionShopSlotsReceived, ShopAuctionUiEntities,
    ShopAuctionUiMode, ShopAuctionUiOutboundMessages, ShopAuctionUiPlugin, ShopFooterSlotCard,
    ShopFooterSlotState,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn sau_004_card_before_phase_enters_preparing_without_countdown() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    send_auction_card(&mut app, CardId(1), 4, 20_000);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Preparing
    );
    assert_eq!(auction_state.card_id, Some(CardId(1)));
    assert_eq!(auction_state.current_price, 4);
    assert_eq!(auction_state.timer_remaining_ms, 0);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::AuctionPreparing
    );
    assert_eq!(auction_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        timer_bar_state(&app),
        Some(&AuctionTimerBarState {
            greyed: true,
            countdown_active: false,
            connection_error: false,
        })
    );
    assert!(auction_card_text(&app).contains("Card 1"));
    // PROMPT 1182 — the price "{N}g" now lives on the dedicated
    // `AuctionFeaturedCardPriceLabel` child entity (previously the
    // parent's `Text` carried "name\nrarity - {N}g" which ghosted
    // under the price / timer band children — AUDIT-1129 UI-1129-02).
    assert!(auction_price_label_text(&app).contains("4g"));
}

#[test]
fn sau_004_phase_before_card_waits_then_activates_countdown() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    // S2CPhaseChanged carries timer 0 for the auction phase per server
    // `draft_timer_ms(DraftPhase::Auction)`. The countdown duration is now
    // sourced from S2CAuctionCard.timer_duration_ms.
    set_phase(&mut app, RoundPhase::DraftAuction, 0);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Inactive
    );
    assert_eq!(auction_panel_visibility(&app), Some(&Visibility::Hidden));

    send_auction_card(&mut app, CardId(2), 5, 20_000);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Active
    );
    assert_eq!(auction_state.timer_duration_ms, 20_000);
    assert_eq!(auction_state.timer_remaining_ms, 20_000);
    assert_eq!(
        timer_bar_state(&app),
        Some(&AuctionTimerBarState {
            greyed: false,
            countdown_active: true,
            connection_error: false,
        })
    );

    run_for(&mut app, Duration::from_secs(1));
    assert_eq!(
        app.world()
            .resource::<ShopAuctionAuctionState>()
            .timer_remaining_ms,
        19_000
    );
}

#[test]
fn sau_asset_loop_featured_auction_card_resolves_display_art_or_fallback() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    send_auction_card(&mut app, CardId(1), 4, 20_000);
    let featured_card = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_featured_card;
    assert_eq!(
        app.world().get::<CardDisplayArtAsset>(featured_card),
        Some(&CardDisplayArtAsset {
            path: "art/cards/display/card_iop_knight_001_art_display.png".to_string()
        })
    );

    // CardId(99) is intentionally absent from the test catalog (1..=3); the
    // catalog miss feeds None into apply_card_display_art, producing the
    // MissingDisplayAsset fallback on the featured auction card.
    send_auction_card(&mut app, CardId(99), 5, 20_000);
    assert_eq!(
        app.world().get::<CardDisplayArtFallback>(featured_card),
        Some(&CardDisplayArtFallback {
            reason: CardDisplayArtFallbackReason::MissingDisplayAsset
        })
    );
}

#[test]
fn sau_zero_phase_timer_with_valid_card_timer_activates_countdown() {
    // Regression for S11-PROTO-AUCTION-TIMER-DURATION-001 (Surface B):
    // server `draft_timer_ms(DraftPhase::Auction)` returns 0, so
    // S2CPhaseChanged.timer_duration_ms is 0 for the auction phase. The
    // live-bidding countdown duration must come from
    // S2CAuctionCard.timer_duration_ms instead — the client must enter
    // ::Active with the card's timer ticking, even with phase timer 0.
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    set_phase(&mut app, RoundPhase::DraftAuction, 0);
    send_auction_card(&mut app, CardId(2), 5, 20_000);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Active
    );
    assert_eq!(auction_state.timer_duration_ms, 20_000);
    assert_eq!(auction_state.timer_remaining_ms, 20_000);
    assert_eq!(
        timer_bar_state(&app),
        Some(&AuctionTimerBarState {
            greyed: false,
            countdown_active: true,
            connection_error: false,
        })
    );

    run_for(&mut app, Duration::from_secs(1));
    let after_tick = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(after_tick.timer_remaining_ms, 19_000);
    assert_eq!(after_tick.timer_duration_ms, 20_000);
}

#[test]
fn sau_004_card_first_then_phase_activates_countdown() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    // Card carries the live-bidding countdown (18_000); the subsequent
    // phase-changed message reports timer 0 per server `draft_timer_ms`.
    send_auction_card(&mut app, CardId(3), 6, 18_000);
    set_phase(&mut app, RoundPhase::DraftAuction, 0);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Active
    );
    assert_eq!(auction_state.card_id, Some(CardId(3)));
    assert_eq!(auction_state.timer_duration_ms, 18_000);
    assert_eq!(auction_state.timer_remaining_ms, 18_000);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Auction
    );
}

#[test]
fn sau_004_preparing_timeout_shows_connection_error() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    send_auction_card(&mut app, CardId(1), 4, 20_000);
    run_for(&mut app, Duration::from_secs(10));

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::ConnectionError
    );
    assert_eq!(auction_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        timer_bar_state(&app),
        Some(&AuctionTimerBarState {
            greyed: true,
            countdown_active: false,
            connection_error: true,
        })
    );
    assert_eq!(
        auction_status_text(&app),
        "Connection error - awaiting server..."
    );
}

#[test]
fn sau_004_non_auction_phase_during_preparing_clears_buffer_and_dismisses() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    send_auction_card(&mut app, CardId(1), 4, 20_000);
    set_phase(&mut app, RoundPhase::GameOver, 0);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Hidden
    );
    assert_eq!(auction_state.card_id, None);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Inactive
    );
    assert_eq!(auction_panel_visibility(&app), Some(&Visibility::Hidden));
}

#[test]
fn sau_004_draft_auction_footer_is_locked_and_does_not_send_shop_messages() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    set_phase(&mut app, RoundPhase::DraftAuction, 0);
    send_shop_slots(&mut app, vec![Some(CardId(1)), None, Some(CardId(2))]);
    send_auction_card(&mut app, CardId(3), 6, 20_000);

    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    assert_eq!(shop_footer_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(refresh_button_visibility(&app), Some(&Visibility::Hidden));
    assert_eq!(
        footer_slot_states(&app),
        vec![
            ShopFooterSlotState::Locked,
            ShopFooterSlotState::EmptyLocked,
            ShopFooterSlotState::Locked
        ]
    );
    assert_eq!(
        footer_slot_cards(&app),
        vec![Some(CardId(1)), None, Some(CardId(2))]
    );

    app.world_mut().write_message(ShopAuctionShopSlotClicked {
        slot: entities.shop_slots[0],
    });
    app.world_mut().write_message(ShopAuctionShopSlotClicked {
        slot: entities.shop_footer_slots[0],
    });
    app.world_mut()
        .write_message(ShopAuctionShopRefreshClicked {
            button: entities.shop_refresh_button,
        });
    run_update(&mut app);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert!(outbound.purchase_cards.is_empty());
    assert!(outbound.refresh_shops.is_empty());
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=3)
            .map(|id| {
                let card = test_card(id, Rarity::Rare, id + 2);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 10,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
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

fn send_auction_card(app: &mut App, card_id: CardId, starting_price: u32, timer_duration_ms: u32) {
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id,
            starting_price,
            timer_duration_ms,
        });
    run_update(app);
}

fn send_shop_slots(app: &mut App, slots: Vec<Option<CardId>>) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotsReceived { slots });
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn run_for(app: &mut App, duration: Duration) {
    let mut remaining = duration;
    while remaining > Duration::ZERO {
        let step = remaining.min(Duration::from_millis(250));
        *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
            TimeUpdateStrategy::ManualDuration(step);
        app.update();
        remaining = remaining.saturating_sub(step);
    }
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

fn auction_panel_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .auction_panel,
    )
}

fn shop_footer_visibility(app: &App) -> Option<&Visibility> {
    app.world()
        .get::<Visibility>(app.world().resource::<ShopAuctionUiEntities>().shop_footer)
}

fn refresh_button_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .shop_refresh_button,
    )
}

fn timer_bar_state(app: &App) -> Option<&AuctionTimerBarState> {
    app.world().get::<AuctionTimerBarState>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .auction_timer_bar,
    )
}

fn auction_card_text(app: &App) -> String {
    app.world()
        .get::<Text>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .auction_featured_card,
        )
        .map(|text| text.0.clone())
        .unwrap_or_default()
}

/// PROMPT 1182 — the auction price line lives on the dedicated
/// `AuctionFeaturedCardPriceLabel` child entity (`"Bid: {N}g"`). The
/// parent `auction_featured_card` only renders the card name now.
fn auction_price_label_text(app: &App) -> String {
    app.world()
        .get::<Text>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .auction_featured_card_price_label,
        )
        .map(|text| text.0.clone())
        .unwrap_or_default()
}

fn auction_status_text(app: &App) -> String {
    app.world()
        .get::<Text>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .auction_status_text,
        )
        .map(|text| text.0.clone())
        .unwrap_or_default()
}

fn footer_slot_states(app: &App) -> Vec<ShopFooterSlotState> {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .shop_footer_slots
        .iter()
        .map(|slot| {
            *app.world()
                .get::<ShopFooterSlotState>(*slot)
                .expect("footer slot should have a locked state")
        })
        .collect()
}

fn footer_slot_cards(app: &App) -> Vec<Option<CardId>> {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .shop_footer_slots
        .iter()
        .map(|slot| {
            app.world()
                .get::<ShopFooterSlotCard>(*slot)
                .map(|card| card.0)
        })
        .collect()
}
