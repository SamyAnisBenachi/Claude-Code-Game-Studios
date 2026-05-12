use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hud::HudGoldBroadcastMessage;
use client::ui::shop_auction::{
    AuctionBidButtonState, AuctionTimerTargetFill, AuctionToastText,
    ShopAuctionAuctionCardReceived, ShopAuctionAuctionState, ShopAuctionBidAcceptedReceived,
    ShopAuctionBidRejectedReceived, ShopAuctionCardCatalog, ShopAuctionLocalGoldView,
    ShopAuctionToastState, ShopAuctionUiEntities, ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{BidRejectedReason, RoundPhase, S2CGoldBroadcast};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);

#[test]
fn sau_006_accepted_local_bid_updates_leader_hides_buttons_and_writes_timer_target() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .in_flight_bid_amount = Some(5);
    write_bid_accepted(&mut app, LOCAL_PLAYER, 5, 10_000);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(auction_state.current_price, 5);
    assert_eq!(auction_state.current_leader, Some(LOCAL_PLAYER));
    assert_eq!(auction_state.in_flight_bid_amount, None);
    assert!(!auction_state.pending_bid_accepted);
    assert_eq!(auction_state.timer_remaining_ms, 10_000);
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill {
            fill_pct: 0.5,
            new_timer_ms: 10_000,
            duration_ms: 20_000,
            updated: true,
        }
    );
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::HiddenLeading,
            AuctionBidButtonState::HiddenLeading,
            AuctionBidButtonState::HiddenLeading
        ]
    );
    assert_eq!(bid_status_text(&app), "YOU ARE LEADING");
}

#[test]
fn sau_006_opponent_accepted_waits_for_local_gold_broadcast_gate() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .in_flight_bid_amount = Some(5);
    write_bid_accepted(&mut app, OPPONENT_PLAYER, 7, 8_000);

    {
        let auction_state = app.world().resource::<ShopAuctionAuctionState>();
        assert_eq!(auction_state.current_price, 7);
        assert_eq!(auction_state.current_leader, Some(OPPONENT_PLAYER));
        assert_eq!(auction_state.in_flight_bid_amount, None);
        assert!(auction_state.pending_bid_accepted);
        assert!(!auction_state.pending_gold_broadcast_seen);
        assert!(!auction_state.opponent_bid_gate_satisfied);
    }
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill {
            fill_pct: 0.4,
            new_timer_ms: 8_000,
            duration_ms: 20_000,
            updated: true,
        }
    );
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::GenericDisabled,
            AuctionBidButtonState::GenericDisabled,
            AuctionBidButtonState::GenericDisabled
        ]
    );

    write_gold_broadcast(&mut app, OPPONENT_PLAYER, 20, 7);
    assert!(
        !app.world()
            .resource::<ShopAuctionAuctionState>()
            .opponent_bid_gate_satisfied
    );
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::GenericDisabled,
            AuctionBidButtonState::GenericDisabled,
            AuctionBidButtonState::GenericDisabled
        ]
    );

    write_gold_broadcast(&mut app, LOCAL_PLAYER, 12, 0);
    {
        let auction_state = app.world().resource::<ShopAuctionAuctionState>();
        assert!(auction_state.pending_bid_accepted);
        assert!(auction_state.pending_gold_broadcast_seen);
        assert!(auction_state.opponent_bid_gate_satisfied);
    }
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled
        ]
    );
}

#[test]
fn sau_006_gold_before_opponent_accepted_satisfies_gate_on_accepted_arrival() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    write_gold_broadcast(&mut app, LOCAL_PLAYER, 12, 0);
    write_bid_accepted(&mut app, OPPONENT_PLAYER, 7, 8_000);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert!(auction_state.pending_bid_accepted);
    assert!(auction_state.pending_gold_broadcast_seen);
    assert!(auction_state.opponent_bid_gate_satisfied);
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled
        ]
    );
}

#[test]
fn sau_006_rejected_bid_clears_inflight_reenables_and_maps_toasts() {
    test_helpers::init_test_tracing();
    let cases = [
        (BidRejectedReason::InsufficientGold, "Not enough gold"),
        (BidRejectedReason::AmountTooLow, "Bid must be at least 5g"),
        (BidRejectedReason::AlreadyLeader, "You are already leading"),
        (
            BidRejectedReason::HandFull,
            "Hand full — no bids possible this auction",
        ),
        (BidRejectedReason::AuctionExpired, "Auction has ended"),
    ];

    for (reason, expected_toast) in cases {
        let mut app = app_in_active_auction(4, 20_000);
        app.world_mut()
            .resource_mut::<ShopAuctionAuctionState>()
            .in_flight_bid_amount = Some(5);

        write_bid_rejected(&mut app, reason);

        assert_eq!(
            app.world()
                .resource::<ShopAuctionAuctionState>()
                .in_flight_bid_amount,
            None
        );
        assert_eq!(
            bid_button_states(&app),
            [
                AuctionBidButtonState::Enabled,
                AuctionBidButtonState::Enabled,
                AuctionBidButtonState::Enabled
            ]
        );
        assert_eq!(toast_text(&app), expected_toast);
        assert_eq!(toast_visibility(&app), Visibility::Visible);
    }
}

#[test]
fn sau_006_toast_replacement_resets_timer_without_stacking() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    write_bid_rejected(&mut app, BidRejectedReason::InsufficientGold);
    run_for(&mut app, Duration::from_millis(1_000));
    assert!(app.world().resource::<ShopAuctionToastState>().elapsed_ms > 0);

    write_bid_rejected(&mut app, BidRejectedReason::AuctionExpired);

    let toast_state = app.world().resource::<ShopAuctionToastState>();
    assert_eq!(toast_state.text, "Auction has ended");
    assert_eq!(toast_state.elapsed_ms, 0);
    assert!(toast_state.active);
    assert_eq!(count_toast_text_entities(&mut app), 1);
}

#[test]
fn sau_006_phase_exit_clears_pending_gate_and_late_rejection_does_not_reenable() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .in_flight_bid_amount = Some(5);
    write_bid_accepted(&mut app, OPPONENT_PLAYER, 7, 8_000);
    assert!(
        app.world()
            .resource::<ShopAuctionAuctionState>()
            .pending_bid_accepted
    );

    set_phase(&mut app, RoundPhase::DraftShop, 30_000);

    {
        let auction_state = app.world().resource::<ShopAuctionAuctionState>();
        assert_eq!(auction_state.in_flight_bid_amount, None);
        assert!(!auction_state.pending_bid_accepted);
        assert!(!auction_state.pending_gold_broadcast_seen);
        assert!(!auction_state.opponent_bid_gate_satisfied);
    }
    assert_eq!(
        bid_button_visibility(&app),
        [Visibility::Hidden, Visibility::Hidden, Visibility::Hidden]
    );

    write_bid_rejected(&mut app, BidRejectedReason::InsufficientGold);

    assert!(!app.world().resource::<ShopAuctionToastState>().active);
    assert_eq!(
        bid_button_visibility(&app),
        [Visibility::Hidden, Visibility::Hidden, Visibility::Hidden]
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

fn write_bid_rejected(app: &mut App, reason: BidRejectedReason) {
    app.world_mut()
        .write_message(ShopAuctionBidRejectedReceived { reason });
    run_update(app);
}

fn write_gold_broadcast(app: &mut App, player_id: PlayerId, gold: u32, reserved_gold: u32) {
    app.world_mut()
        .write_message(HudGoldBroadcastMessage(S2CGoldBroadcast {
            player_id,
            gold,
            reserved_gold,
        }));
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

fn bid_button_visibility(app: &App) -> [Visibility; 3] {
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

fn toast_text(app: &App) -> String {
    app.world()
        .get::<Text>(app.world().resource::<ShopAuctionUiEntities>().toast_text)
        .expect("toast should have text")
        .0
        .clone()
}

fn toast_visibility(app: &App) -> Visibility {
    *app.world()
        .get::<Visibility>(app.world().resource::<ShopAuctionUiEntities>().toast_root)
        .expect("toast root should have visibility")
}

fn count_toast_text_entities(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<AuctionToastText>>();
    query.iter(app.world()).count()
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
