use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hand::{
    ActivePlacementDrag, HandUiEntities, HandUiMode, HandUiPlugin, PendingPlacements,
    PlacementDisclosureState, PlacementDisclosureStep, PlacementTimer, SubmitValidationError,
    TimerState,
};
use client::ui::shop_auction::{
    AuctionBidButtonState, AuctionTimerTargetFill, ShopAuctionAuctionCardReceived,
    ShopAuctionAuctionState, ShopAuctionBidAcceptedReceived, ShopAuctionBidRejectedReceived,
    ShopAuctionCardCatalog, ShopAuctionLocalGoldView, ShopAuctionSettledReceived,
    ShopAuctionSettlementState, ShopAuctionShopReadyButtonClicked, ShopAuctionShopSlotsReceived,
    ShopAuctionShopState, ShopAuctionShopTimerState, ShopAuctionToastState, ShopAuctionUiEntities,
    ShopAuctionUiMode, ShopAuctionUiOutboundMessages, ShopAuctionUiPlugin, ShopSlotCard,
    AUCTION_SETTLEMENT_TRANSITION_MS,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{BidRejectedReason, PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);

#[test]
fn test_phase_boundaries_clear_stale_auction_feedback_before_next_loop_auction() {
    test_helpers::init_test_tracing();
    let mut app = shop_app_in_active_auction(4, 20_000);

    write_bid_accepted(&mut app, OPPONENT_PLAYER, 7, 8_000);
    write_bid_rejected(&mut app, BidRejectedReason::InsufficientGold);

    assert!(app.world().resource::<ShopAuctionToastState>().active);
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill {
            fill_pct: 0.4,
            new_timer_ms: 8_000,
            duration_ms: 20_000,
            updated: true,
        }
    );

    set_phase(&mut app, RoundPhase::Placement, 45_000);

    assert_eq!(
        app.world().resource::<ShopAuctionToastState>(),
        &ShopAuctionToastState::default()
    );
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill::default()
    );
    assert_eq!(
        app.world().resource::<ShopAuctionAuctionState>().card_id,
        None
    );
    assert_eq!(
        bid_button_states(&app),
        [AuctionBidButtonState::GenericDisabled; 3]
    );

    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);
    send_auction_card(&mut app, CardId(2), 6);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(auction_state.card_id, Some(CardId(2)));
    assert_eq!(auction_state.current_price, 6);
    assert_eq!(auction_state.current_leader, None);
    assert_eq!(auction_state.in_flight_bid_amount, None);
    assert_eq!(
        app.world().resource::<ShopAuctionToastState>(),
        &ShopAuctionToastState::default()
    );
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill::default()
    );
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled,
        ]
    );
}

#[test]
fn test_repeated_draft_shop_phase_message_resets_ready_and_waits_for_new_slots() {
    test_helpers::init_test_tracing();
    let mut app = shop_app();
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    click_shop_ready(&mut app);

    assert!(
        app.world()
            .resource::<ShopAuctionShopState>()
            .ready_signalled
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .ready_signals
            .len(),
        1
    );

    set_phase(&mut app, RoundPhase::DraftShop, 45_000);

    let shop_state = app.world().resource::<ShopAuctionShopState>();
    assert!(!shop_state.ready_signalled);
    assert!(!shop_state.slots_loaded);
    assert_eq!(
        app.world().resource::<ShopAuctionShopTimerState>(),
        &ShopAuctionShopTimerState {
            duration_ms: 45_000,
            remaining_ms: 45_000,
            started: true,
            deferred: false,
        }
    );
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Shop
    );

    send_shop_slots(&mut app, vec![Some(CardId(4)), None, Some(CardId(5))]);
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(4)), None, Some(CardId(5))]
    );
}

#[test]
fn test_late_settlement_after_shop_convergence_does_not_resurrect_auction_ui() {
    test_helpers::init_test_tracing();
    let mut app = shop_app_in_active_auction(4, 20_000);
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

    run_for(
        &mut app,
        Duration::from_millis((AUCTION_SETTLEMENT_TRANSITION_MS + 250).into()),
    );

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Shop
    );
    assert_eq!(
        app.world().resource::<ShopAuctionAuctionState>().card_id,
        None
    );
    assert!(
        !app.world()
            .resource::<ShopAuctionSettlementState>()
            .transition_active
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
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(2)), Some(CardId(3)), Some(CardId(4))]
    );

    let converged_settlement = app.world().resource::<ShopAuctionSettlementState>().clone();
    write_settled(&mut app, Some(LOCAL_PLAYER), 99);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Shop
    );
    assert_eq!(
        app.world().resource::<ShopAuctionSettlementState>(),
        &converged_settlement
    );
    assert_eq!(
        app.world().resource::<ShopAuctionAuctionState>().card_id,
        None
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
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(2)), Some(CardId(3)), Some(CardId(4))]
    );
    assert_eq!(shop_panel_visibility(&app), Visibility::Visible);
    assert_eq!(settlement_overlay_visibility(&app), Visibility::Hidden);
}

#[test]
fn test_placement_exit_clears_stale_hand_timer_submit_and_pending_state() {
    test_helpers::init_test_tracing();
    let mut app = hand_app_in_placement(30_000);
    let entities = *app.world().resource::<HandUiEntities>();

    *app.world_mut().resource_mut::<PendingPlacements>() = PendingPlacements {
        placements: vec![PlacedCardSubmit {
            card_id: CardId(1),
            target: PlayTarget::BoardCell { lane: 1, cell: 1 },
            current_mana_spend: 1,
            reserve_mana_spend: 0,
        }],
    };
    *app.world_mut().resource_mut::<PlacementTimer>() = PlacementTimer {
        remaining_ms: 12_000,
        urgency_fired: true,
        in_grace_window: true,
        grace_remaining_ms: 100,
        submitted: true,
    };
    app.world_mut().resource_mut::<ActivePlacementDrag>().card = Some(entities.submit_button);
    app.world_mut()
        .entity_mut(entities.submit_button)
        .insert(SubmitValidationError::ManaOverdrawn);

    set_phase(&mut app, RoundPhase::Resolution, 0);
    run_for(&mut app, Duration::from_millis(500));

    assert_eq!(app.world().resource::<HandUiMode>(), &HandUiMode::Hidden);
    assert!(app
        .world()
        .resource::<PendingPlacements>()
        .placements
        .is_empty());
    assert_eq!(
        app.world().resource::<PlacementTimer>(),
        &PlacementTimer::default()
    );
    assert_eq!(
        app.world().resource::<PlacementDisclosureState>().step,
        PlacementDisclosureStep::Hidden
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.timer),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<TimerState>(entities.timer),
        Some(&TimerState::Normal)
    );
    assert!(app
        .world()
        .get::<SubmitValidationError>(entities.submit_button)
        .is_none());
}

fn shop_app_in_active_auction(starting_price: u32, timer_duration_ms: u32) -> App {
    let mut app = shop_app();
    set_phase(&mut app, RoundPhase::DraftAuction, timer_duration_ms);
    send_auction_card(&mut app, CardId(1), starting_price);
    app
}

fn shop_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: test_catalog(1..=8),
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
    app
}

fn hand_app_in_placement(timer_duration_ms: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(PlayerEconomyView {
        current_mana: 5,
        mana_cap: 10,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    set_phase(&mut app, RoundPhase::Placement, timer_duration_ms);
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

fn click_shop_ready(app: &mut App) {
    let button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .shop_ready_button;
    app.world_mut()
        .write_message(ShopAuctionShopReadyButtonClicked { button });
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
            .expect("bid button should have state")
    })
}

fn shop_slot_cards(app: &App) -> Vec<Option<CardId>> {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .shop_slots
        .iter()
        .map(|slot| app.world().get::<ShopSlotCard>(*slot).map(|card| card.0))
        .collect()
}

fn shop_panel_visibility(app: &App) -> Visibility {
    *app.world()
        .get::<Visibility>(app.world().resource::<ShopAuctionUiEntities>().shop_panel)
        .expect("shop panel should have visibility")
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

fn test_catalog(ids: impl IntoIterator<Item = u32>) -> HashMap<CardId, CardData> {
    ids.into_iter()
        .map(|id| {
            let card = CardData {
                id: CardId(id),
                name_fr: format!("Carte {id}"),
                name_en: format!("Card {id}"),
                class: ClassId::Iop,
                family: Some("Test".to_string()),
                rarity: Rarity::Rare,
                card_type: CardType::Minion,
                unit_type: UnitType::Blade,
                cost: 2,
                atk: 1,
                hp: 2,
                mp: 1,
                ar: 0,
                keywords: Vec::new(),
                effect_text: String::new(),
                art_id: format!("test_{id}"),
                pool_copies_override: None,
            };
            (card.id, card)
        })
        .collect()
}
