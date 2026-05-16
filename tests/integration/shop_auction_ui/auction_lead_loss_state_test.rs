//! Auction featured-card lead/loss state integration tests.
//!
//! Covers Sprint 14 story 018 against the real `ShopAuctionUiPlugin`
//! state-sync path. The visual state is asserted through the stable
//! `AuctionFeaturedCardLeadLossState` marker carried by the Story 016
//! featured-card frame primitive.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    auction_featured_card_accent_color, auction_featured_card_leading_color,
    auction_featured_card_losing_color, AuctionFeaturedCardFrame, AuctionFeaturedCardLeadLossState,
    ShopAuctionAuctionCardReceived, ShopAuctionBidAcceptedReceived, ShopAuctionCardCatalog,
    ShopAuctionLocalGoldView, ShopAuctionUiEntities, ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);

#[test]
fn sau_018_neutral_frame_uses_accent_before_any_bid() {
    test_helpers::init_test_tracing();
    let app = app_in_active_auction(4, 20_000);

    assert_featured_frame_state(
        &app,
        AuctionFeaturedCardLeadLossState::Neutral,
        auction_featured_card_accent_color(),
    );
    assert_eq!(bid_status_text(&app), "");
    assert_eq!(bid_status_visibility(&app), Visibility::Hidden);
}

#[test]
fn sau_018_local_leader_uses_success_frame_and_preserves_text_fallback() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    write_bid_accepted(&mut app, LOCAL_PLAYER, 5, 10_000);

    assert_featured_frame_state(
        &app,
        AuctionFeaturedCardLeadLossState::Leading,
        auction_featured_card_leading_color(),
    );
    assert_eq!(bid_status_text(&app), "YOU ARE LEADING");
    assert_eq!(bid_status_visibility(&app), Visibility::Visible);
}

#[test]
fn sau_018_opponent_leader_uses_error_frame_and_text_fallback() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    write_bid_accepted(&mut app, OPPONENT_PLAYER, 7, 8_000);

    assert_featured_frame_state(
        &app,
        AuctionFeaturedCardLeadLossState::Losing,
        auction_featured_card_losing_color(),
    );
    assert_eq!(bid_status_text(&app), "OPPONENT LEADING");
    assert_eq!(bid_status_visibility(&app), Visibility::Visible);
}

#[test]
fn sau_018_state_transitions_remain_strictly_exclusive() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    assert_eq!(
        featured_frame_state(&app),
        AuctionFeaturedCardLeadLossState::Neutral
    );

    write_bid_accepted(&mut app, LOCAL_PLAYER, 5, 10_000);
    assert_eq!(
        featured_frame_state(&app),
        AuctionFeaturedCardLeadLossState::Leading
    );

    write_bid_accepted(&mut app, OPPONENT_PLAYER, 7, 8_000);
    assert_eq!(
        featured_frame_state(&app),
        AuctionFeaturedCardLeadLossState::Losing
    );
}

fn app_in_active_auction(starting_price: u32, timer_duration_ms: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: HashMap::from([(CardId(1), test_card(1, Rarity::Rare, 4))]),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 12,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold: 12,
        reserved_gold: 0,
        initialized: true,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    set_phase(&mut app, RoundPhase::DraftAuction, timer_duration_ms);
    send_auction_card(&mut app, CardId(1), starting_price);
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

fn send_auction_card(app: &mut App, card_id: CardId, starting_price: u32) {
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id,
            starting_price,
            timer_duration_ms: 20_000,
        });
    run_update(app);
}

fn write_bid_accepted(app: &mut App, bidder: PlayerId, amount: u32, new_timer_ms: u32) {
    app.world_mut()
        .write_message(ShopAuctionBidAcceptedReceived {
            bidder,
            amount,
            new_timer_ms,
        });
    run_update(app);
}

fn assert_featured_frame_state(
    app: &App,
    expected_state: AuctionFeaturedCardLeadLossState,
    expected_color: Color,
) {
    assert_eq!(featured_frame_state(app), expected_state);
    assert_eq!(featured_frame_border_color(app), expected_color);
}

fn featured_frame_state(app: &App) -> AuctionFeaturedCardLeadLossState {
    *app.world()
        .get::<AuctionFeaturedCardLeadLossState>(featured_frame_entity(app))
        .expect("featured frame should carry lead/loss state")
}

fn featured_frame_border_color(app: &App) -> Color {
    app.world()
        .get::<BorderColor>(featured_frame_entity(app))
        .expect("featured frame should carry border color")
        .left
}

fn featured_frame_entity(app: &App) -> Entity {
    let entity = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_featured_card_frame;
    assert!(
        app.world()
            .get::<AuctionFeaturedCardFrame>(entity)
            .is_some(),
        "frame entity should keep the Story 016 marker"
    );
    entity
}

fn bid_status_text(app: &App) -> String {
    app.world()
        .get::<Text>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .auction_bid_status_text,
        )
        .expect("bid status should have text")
        .0
        .clone()
}

fn bid_status_visibility(app: &App) -> Visibility {
    *app.world()
        .get::<Visibility>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .auction_bid_status_text,
        )
        .expect("bid status should carry visibility")
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
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
