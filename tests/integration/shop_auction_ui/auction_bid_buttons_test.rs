use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hud::HudGoldBroadcastMessage;
use client::ui::shop_auction::{
    AuctionBidButtonState, ShopAuctionAuctionCardReceived, ShopAuctionAuctionState,
    ShopAuctionBidButtonClicked, ShopAuctionCardCatalog, ShopAuctionDraftHandView,
    ShopAuctionLocalGoldView, ShopAuctionUiEntities, ShopAuctionUiOutboundMessages,
    ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{RoundPhase, S2CGoldBroadcast};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);

#[test]
fn sau_005_bid_buttons_use_local_free_gold_and_split_labels() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(0, 20_000);

    write_local_gold_broadcast(&mut app, 5, 3);

    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Unaffordable,
            AuctionBidButtonState::Unaffordable
        ]
    );
    assert_eq!(bid_button_texts(&app), ["1g\n(+1)", "3g\n(+3)", "5g\n(+5)"]);
}

#[test]
fn sau_005_hand_size_ten_disables_bids_and_shows_hand_full_message() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(2, 20_000);

    app.world_mut()
        .resource_mut::<ShopAuctionDraftHandView>()
        .hand_size = 10;
    run_update(&mut app);

    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::HandFullLocked,
            AuctionBidButtonState::HandFullLocked,
            AuctionBidButtonState::HandFullLocked
        ]
    );
    assert_eq!(
        bid_status_text(&app),
        "Hand full - no bids possible this auction"
    );

    click_bid_button(&mut app, 0);
    assert!(app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .place_bids
        .is_empty());
}

#[test]
fn sau_005_local_leader_hides_bid_buttons_and_shows_badge() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .current_leader = Some(LOCAL_PLAYER);
    run_update(&mut app);

    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::HiddenLeading,
            AuctionBidButtonState::HiddenLeading,
            AuctionBidButtonState::HiddenLeading
        ]
    );
    assert_eq!(
        bid_button_visibilities(&app),
        [Visibility::Hidden, Visibility::Hidden, Visibility::Hidden]
    );
    assert_eq!(bid_status_text(&app), "YOU ARE LEADING");
}

#[test]
fn sau_005_enabled_click_sends_one_bid_and_locks_in_flight_buttons() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(2, 20_000);

    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    app.world_mut()
        .write_message(ShopAuctionBidButtonClicked { button: buttons[1] });
    app.world_mut()
        .write_message(ShopAuctionBidButtonClicked { button: buttons[0] });
    run_update(&mut app);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.place_bids.len(), 1);
    assert_eq!(outbound.place_bids[0].amount, 5);
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::GenericDisabled,
            AuctionBidButtonState::InFlight,
            AuctionBidButtonState::GenericDisabled
        ]
    );
    assert_eq!(bid_button_texts(&app)[1], "BIDDING...");

    click_bid_button(&mut app, 2);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .place_bids
            .len(),
        1
    );
}

#[test]
fn sau_005_locally_expired_timer_disables_bids_and_escalates_status() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(1, 1_000);

    run_for(&mut app, Duration::from_millis(1_000));

    assert_eq!(auction_status_text(&app), "Auction ending...");
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::LocallyExpired,
            AuctionBidButtonState::LocallyExpired,
            AuctionBidButtonState::LocallyExpired
        ]
    );

    click_bid_button(&mut app, 0);
    assert!(app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .place_bids
        .is_empty());

    run_for(&mut app, Duration::from_millis(1_500));
    assert_eq!(auction_status_text(&app), "Awaiting server...");
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
        gold: 10,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold: 10,
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
        });
    run_update(app);
}

fn write_local_gold_broadcast(app: &mut App, gold: u32, reserved_gold: u32) {
    app.world_mut()
        .write_message(HudGoldBroadcastMessage(S2CGoldBroadcast {
            player_id: LOCAL_PLAYER,
            gold,
            reserved_gold,
        }));
    run_update(app);
}

fn click_bid_button(app: &mut App, index: usize) {
    let button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons[index];
    app.world_mut()
        .write_message(ShopAuctionBidButtonClicked { button });
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

fn bid_button_states(app: &App) -> [AuctionBidButtonState; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        *app.world()
            .get::<AuctionBidButtonState>(button)
            .expect("bid button should have a state")
    })
}

fn bid_button_texts(app: &App) -> [String; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        app.world()
            .get::<Text>(button)
            .expect("bid button should have text")
            .0
            .clone()
    })
}

fn bid_button_visibilities(app: &App) -> [Visibility; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        *app.world()
            .get::<Visibility>(button)
            .expect("bid button should have visibility")
    })
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

fn auction_status_text(app: &App) -> String {
    app.world()
        .get::<Text>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .auction_status_text,
        )
        .expect("auction status should have text")
        .0
        .clone()
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
