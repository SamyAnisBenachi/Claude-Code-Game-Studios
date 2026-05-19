use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{
    AuctionPanelTransitionRequested, CardAcquiredAnimReady, CardAnimationsPlugin,
    SettlementOverlayRequested,
};
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::settings::AccessibilityPreferences;
use client::ui::shop_auction::{
    AuctionBidButtonState, AuctionTimerTargetFill, ShopAuctionAuctionCardReceived,
    ShopAuctionAuctionState, ShopAuctionBidAcceptedReceived, ShopAuctionBidRejectedReceived,
    ShopAuctionCardCatalog, ShopAuctionDraftHandView, ShopAuctionLocalGoldView,
    ShopAuctionSettledReceived, ShopAuctionSettlementOutcome, ShopAuctionSettlementState,
    ShopAuctionShopSlotsReceived, ShopAuctionShopState, ShopAuctionShopTimerState,
    ShopAuctionToastState, ShopAuctionUiEntities, ShopAuctionUiMode, ShopAuctionUiPlugin,
    ShopSlotCard, AUCTION_SETTLEMENT_TRANSITION_MS,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{BidRejectedReason, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);

#[test]
fn sau_007_local_winner_enters_settling_and_requests_card_feedback() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .in_flight_bid_amount = Some(7);
    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .pending_bid_accepted = true;

    write_settled(&mut app, Some(LOCAL_PLAYER), 7);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(auction_state.in_flight_bid_amount, None);
    assert!(!auction_state.pending_bid_accepted);
    assert!(!auction_state.pending_gold_broadcast_seen);
    assert!(!auction_state.opponent_bid_gate_satisfied);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::AuctionSettling
    );

    let settlement = app.world().resource::<ShopAuctionSettlementState>();
    assert_eq!(
        settlement.outcome,
        Some(ShopAuctionSettlementOutcome::LocalWinner)
    );
    assert_eq!(settlement.winner, Some(LOCAL_PLAYER));
    assert_eq!(settlement.amount, 7);
    assert_eq!(settlement.card_id, Some(CardId(1)));
    assert_eq!(settlement.local_card_feedback_requests, 1);
    assert_eq!(settlement.overlay_requests, 1);
    assert_eq!(settlement.panel_transition_requests, 1);
    assert_eq!(
        app.world().resource::<ShopAuctionDraftHandView>().hand_size,
        1
    );
    assert_eq!(read_messages::<CardAcquiredAnimReady>(&app).len(), 1);
    assert_eq!(read_messages::<SettlementOverlayRequested>(&app).len(), 1);
    assert_eq!(
        read_messages::<AuctionPanelTransitionRequested>(&app).len(),
        1
    );
    assert_eq!(
        bid_button_states(&app),
        [AuctionBidButtonState::GenericDisabled; 3]
    );
    assert_eq!(settlement_overlay_visibility(&app), Visibility::Visible);
    assert_eq!(
        settlement_overlay_text(&app),
        "Auction won - card moving to hand"
    );
}

#[test]
fn sau_007_opponent_and_no_bid_settlement_skip_local_card_feedback() {
    test_helpers::init_test_tracing();
    let mut opponent_app = app_in_active_auction(4, 20_000);
    write_settled(&mut opponent_app, Some(OPPONENT_PLAYER), 8);

    let settlement = opponent_app
        .world()
        .resource::<ShopAuctionSettlementState>();
    assert_eq!(
        settlement.outcome,
        Some(ShopAuctionSettlementOutcome::OpponentWinner)
    );
    assert_eq!(settlement.local_card_feedback_requests, 0);
    assert_eq!(
        opponent_app
            .world()
            .resource::<ShopAuctionDraftHandView>()
            .hand_size,
        0
    );
    assert!(read_messages::<CardAcquiredAnimReady>(&opponent_app).is_empty());
    // PROMPT 1347 / AC7 — loser-side toast now names the price. The
    // pre-PROMPT-1347 copy was "Opponent won the auction"; the new copy
    // surfaces the bid commitment so the loser sees what the opponent
    // paid. Static helper `ShopAuctionSettlementState::overlay_text()`
    // still returns the legacy copy; `dynamic_overlay_text()` is the new
    // source-of-truth and is what `sync_settlement_overlay_system` renders.
    assert_eq!(
        settlement_overlay_text(&opponent_app),
        "Opponent won for 8g"
    );

    let mut no_bid_app = app_in_active_auction(4, 20_000);
    write_settled(&mut no_bid_app, None, 0);

    let settlement = no_bid_app.world().resource::<ShopAuctionSettlementState>();
    assert_eq!(
        settlement.outcome,
        Some(ShopAuctionSettlementOutcome::NoBid)
    );
    assert_eq!(settlement.local_card_feedback_requests, 0);
    assert_eq!(
        no_bid_app
            .world()
            .resource::<ShopAuctionDraftHandView>()
            .hand_size,
        0
    );
    assert!(read_messages::<CardAcquiredAnimReady>(&no_bid_app).is_empty());
    assert_eq!(
        settlement_overlay_text(&no_bid_app),
        "No bids - card returned"
    );
}

#[test]
fn sau_007_settlement_suppresses_stale_bid_accepted_and_rejected_messages() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    write_settled(&mut app, Some(LOCAL_PLAYER), 7);
    app.world_mut()
        .write_message(ShopAuctionBidAcceptedReceived {
            bidder: OPPONENT_PLAYER,
            amount: 9,
            new_timer_ms: 15_000,
        });
    app.world_mut()
        .write_message(ShopAuctionBidRejectedReceived {
            reason: BidRejectedReason::InsufficientGold,
        });
    run_update(&mut app);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(auction_state.current_price, 7);
    assert_eq!(auction_state.current_leader, None);
    assert_eq!(auction_state.in_flight_bid_amount, None);
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill::default()
    );
    assert!(!app.world().resource::<ShopAuctionToastState>().active);
    assert_eq!(
        bid_button_states(&app),
        [AuctionBidButtonState::GenericDisabled; 3]
    );
}

#[test]
fn sau_007_same_update_accepted_then_settled_renders_only_terminal_state() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    app.world_mut()
        .write_message(ShopAuctionBidAcceptedReceived {
            bidder: LOCAL_PLAYER,
            amount: 7,
            new_timer_ms: 12_000,
        });
    app.world_mut().write_message(ShopAuctionSettledReceived {
        winner: Some(LOCAL_PLAYER),
        amount: 7,
    });
    run_update(&mut app);

    let settlement = app.world().resource::<ShopAuctionSettlementState>();
    assert_eq!(
        settlement.outcome,
        Some(ShopAuctionSettlementOutcome::LocalWinner)
    );
    assert_eq!(settlement.amount, 7);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::AuctionSettling
    );
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill::default()
    );
    assert_eq!(
        settlement_overlay_text(&app),
        "Auction won - card moving to hand"
    );
}

#[test]
fn sau_007_draft_shop_timer_defers_until_transition_completes() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(2)), Some(CardId(3)), Some(CardId(4))],
    );
    write_settled(&mut app, Some(OPPONENT_PLAYER), 8);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::AuctionSettling
    );
    assert!(app.world().resource::<ShopAuctionShopState>().slots_loaded);
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(2)), Some(CardId(3)), Some(CardId(4))]
    );
    assert_eq!(
        app.world().resource::<ShopAuctionShopTimerState>(),
        &ShopAuctionShopTimerState {
            duration_ms: 30_000,
            remaining_ms: 30_000,
            started: false,
            deferred: true,
        }
    );
    assert_eq!(shop_panel_visibility(&app), Visibility::Hidden);

    run_for(
        &mut app,
        Duration::from_millis((AUCTION_SETTLEMENT_TRANSITION_MS - 1).into()),
    );
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::AuctionSettling
    );
    assert!(!app.world().resource::<ShopAuctionShopTimerState>().started);

    run_for(&mut app, Duration::from_millis(1));
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Shop
    );
    assert_eq!(
        app.world().resource::<ShopAuctionShopTimerState>(),
        &ShopAuctionShopTimerState {
            duration_ms: 30_000,
            remaining_ms: 30_000,
            started: true,
            deferred: false,
        }
    );
    assert_eq!(shop_panel_visibility(&app), Visibility::Visible);
    assert_eq!(settlement_overlay_visibility(&app), Visibility::Hidden);
}

#[test]
fn sau_007_reduced_motion_completes_transition_without_reordering_timer_start() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);
    app.insert_resource(AccessibilityPreferences {
        reduced_motion: true,
        ..default()
    });
    send_shop_slots(
        &mut app,
        vec![Some(CardId(2)), Some(CardId(3)), Some(CardId(4))],
    );
    write_settled(&mut app, Some(OPPONENT_PLAYER), 8);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Shop
    );
    let settlement = app.world().resource::<ShopAuctionSettlementState>();
    assert_eq!(
        settlement.outcome,
        Some(ShopAuctionSettlementOutcome::OpponentWinner)
    );
    assert!(!settlement.transition_active);
    assert_eq!(
        app.world().resource::<ShopAuctionShopTimerState>(),
        &ShopAuctionShopTimerState {
            duration_ms: 30_000,
            remaining_ms: 30_000,
            started: true,
            deferred: false,
        }
    );
}

#[test]
fn sau_007_placement_phase_interrupt_cancels_settlement_immediately() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(4, 20_000);

    write_settled(&mut app, Some(OPPONENT_PLAYER), 8);
    run_for(&mut app, Duration::from_millis(100));
    set_phase(&mut app, RoundPhase::Placement, 45_000);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Inactive
    );
    assert_eq!(
        app.world().resource::<ShopAuctionSettlementState>().outcome,
        None
    );
    assert_eq!(
        app.world().resource::<ShopAuctionAuctionState>().card_id,
        None
    );
    assert_eq!(
        app.world().resource::<ShopAuctionShopTimerState>(),
        &ShopAuctionShopTimerState::default()
    );
    assert_eq!(settlement_overlay_visibility(&app), Visibility::Hidden);

    run_for(
        &mut app,
        Duration::from_millis(AUCTION_SETTLEMENT_TRANSITION_MS.into()),
    );
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::Placement
    );
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Inactive
    );
}

fn app_in_active_auction(starting_price: u32, timer_duration_ms: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(CardAnimationsPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=4)
            .map(|id| {
                let card = test_card(id, Rarity::Rare, id + 3);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 20,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold: 20,
        reserved_gold: 0,
        initialized: true,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    set_phase(&mut app, RoundPhase::DraftAuction, timer_duration_ms);
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id: CardId(1),
            starting_price,
            timer_duration_ms,
        });
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

fn write_settled(app: &mut App, winner: Option<PlayerId>, amount: u32) {
    app.world_mut()
        .write_message(ShopAuctionSettledReceived { winner, amount });
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

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn bid_button_states(app: &App) -> [AuctionBidButtonState; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        *app.world()
            .get::<AuctionBidButtonState>(button)
            .expect("bid button should have state")
    })
}

fn settlement_overlay_visibility(app: &App) -> Visibility {
    *app.world()
        .get::<Visibility>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .settlement_overlay,
        )
        .expect("settlement overlay should have visibility")
}

fn settlement_overlay_text(app: &App) -> String {
    app.world()
        .get::<Text>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .settlement_overlay_text,
        )
        .expect("settlement overlay text should exist")
        .0
        .clone()
}

fn shop_panel_visibility(app: &App) -> Visibility {
    *app.world()
        .get::<Visibility>(app.world().resource::<ShopAuctionUiEntities>().shop_panel)
        .expect("shop panel should have visibility")
}

fn shop_slot_cards(app: &App) -> Vec<Option<CardId>> {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .shop_slots
        .iter()
        .map(|slot| app.world().get::<ShopSlotCard>(*slot).map(|card| card.0))
        .collect()
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
