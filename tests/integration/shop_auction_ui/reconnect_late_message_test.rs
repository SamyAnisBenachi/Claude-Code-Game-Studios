use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    AuctionBidButtonState, AuctionTimerTargetFill, ShopAuctionAuctionCardReceived,
    ShopAuctionAuctionPanelState, ShopAuctionAuctionState, ShopAuctionBidAcceptedReceived,
    ShopAuctionBidRejectedReceived, ShopAuctionCardCatalog, ShopAuctionDraftHandView,
    ShopAuctionDraftInitialState, ShopAuctionDraftOfferingReceived, ShopAuctionDraftSlotClicked,
    ShopAuctionLocalGoldView, ShopAuctionSettledReceived, ShopAuctionSettlementOutcome,
    ShopAuctionSettlementState, ShopAuctionShopCardAcquiredReceived, ShopAuctionShopSlotsReceived,
    ShopAuctionShopState, ShopAuctionShopTimerState, ShopAuctionToastState, ShopAuctionUiEntities,
    ShopAuctionUiMode, ShopAuctionUiOutboundMessages, ShopAuctionUiPlugin, ShopSlotCard,
    ShopSlotState, SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{
    AuctionSnapshot, BidRejectedReason, BoardSnapshot, PlacementTimerMultiplier, PlayerSnapshot,
    RoundPhase, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);

#[test]
fn sau_008_draft_auction_snapshot_rebuilds_without_auction_card_and_clears_transients() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(8);
    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);
    seed_stale_auction_transients(&mut app);

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::DraftAuction,
            Some(12_000),
            player_snapshot(
                LOCAL_PLAYER,
                8,
                5,
                vec![CardId(10), CardId(11)],
                vec![Some(CardId(1)), None, Some(CardId(2))],
            ),
            player_snapshot(OPPONENT_PLAYER, 20, 0, Vec::new(), Vec::new()),
            Some(AuctionSnapshot {
                card_id: CardId(2),
                starting_price: 4,
                last_accepted_bid: 6,
                current_leader: Some(OPPONENT_PLAYER),
                timer_remaining_ms: 12_000,
            }),
        ),
    );

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Active
    );
    assert_eq!(auction_state.card_id, Some(CardId(2)));
    assert_eq!(auction_state.starting_price, 4);
    assert_eq!(auction_state.current_price, 6);
    assert_eq!(auction_state.current_leader, Some(OPPONENT_PLAYER));
    assert_eq!(auction_state.timer_duration_ms, 20_000);
    assert_eq!(auction_state.timer_remaining_ms, 12_000);
    assert_eq!(auction_state.in_flight_bid_amount, None);
    assert!(!auction_state.pending_bid_accepted);
    assert!(!auction_state.pending_gold_broadcast_seen);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Auction
    );
    assert_eq!(
        app.world().resource::<ShopAuctionSettlementState>().outcome,
        None
    );
    assert!(!app.world().resource::<ShopAuctionToastState>().active);
    assert_eq!(
        app.world().resource::<AuctionTimerTargetFill>(),
        &AuctionTimerTargetFill::default()
    );
    assert_eq!(
        app.world().resource::<ShopAuctionDraftHandView>().hand_size,
        2
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionLocalGoldView>()
            .free_gold(app.world().resource::<PlayerEconomyView>()),
        3
    );
}

#[test]
fn sau_008_auction_snapshot_no_bid_uses_starting_price_and_saturating_free_gold() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(99);
    set_phase(&mut app, RoundPhase::DraftAuction, 18_000);

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::DraftAuction,
            Some(9_000),
            player_snapshot(LOCAL_PLAYER, 3, 5, Vec::new(), Vec::new()),
            player_snapshot(OPPONENT_PLAYER, 20, 0, Vec::new(), Vec::new()),
            Some(AuctionSnapshot {
                card_id: CardId(3),
                starting_price: 4,
                last_accepted_bid: 0,
                current_leader: None,
                timer_remaining_ms: 9_000,
            }),
        ),
    );

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(auction_state.current_price, 4);
    assert_eq!(auction_state.current_leader, None);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionLocalGoldView>()
            .free_gold(app.world().resource::<PlayerEconomyView>()),
        0
    );
    assert_eq!(
        bid_button_states(&app),
        [AuctionBidButtonState::Unaffordable; 3]
    );
}

#[test]
fn sau_008_draft_shop_snapshot_uses_local_slots_and_beats_same_frame_late_slots() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(10);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    {
        let mut shop_state = app.world_mut().resource_mut::<ShopAuctionShopState>();
        shop_state.refresh_in_flight = true;
        shop_state.refresh_count_this_draft = 2;
        shop_state.ready_signalled = true;
    }

    app.world_mut().write_message(ShopAuctionShopSlotsReceived {
        slots: vec![Some(CardId(7)), Some(CardId(8)), Some(CardId(9))],
    });
    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(snapshot(
            RoundPhase::DraftShop,
            Some(24_000),
            player_snapshot(
                LOCAL_PLAYER,
                10,
                0,
                vec![CardId(4)],
                vec![Some(CardId(1)), None, Some(CardId(3))],
            ),
            player_snapshot(
                OPPONENT_PLAYER,
                20,
                0,
                Vec::new(),
                vec![Some(CardId(4)), Some(CardId(5)), Some(CardId(6))],
            ),
            None,
        )));
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Shop
    );
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(1)), None, Some(CardId(3))]
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
        app.world().resource::<ShopAuctionShopTimerState>(),
        &ShopAuctionShopTimerState {
            duration_ms: 30_000,
            remaining_ms: 24_000,
            started: true,
            deferred: false,
        }
    );
    let shop_state = app.world().resource::<ShopAuctionShopState>();
    assert!(shop_state.slots_loaded);
    assert!(!shop_state.refresh_in_flight);
    assert_eq!(shop_state.refresh_count_this_draft, 0);
    assert!(!shop_state.ready_signalled);
}

#[test]
fn sau_008_late_auction_accepted_rejected_and_card_are_ignored_after_phase_exit() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(20);
    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);
    send_auction_card(&mut app, CardId(1), 4);
    app.world_mut().write_message(ShopAuctionSettledReceived {
        winner: Some(LOCAL_PLAYER),
        amount: 7,
        card_id: CardId(1),
    });
    run_update(&mut app);
    assert_eq!(
        app.world().resource::<ShopAuctionSettlementState>().outcome,
        Some(ShopAuctionSettlementOutcome::LocalWinner)
    );

    set_phase(&mut app, RoundPhase::Placement, 45_000);
    app.world_mut()
        .write_message(ShopAuctionBidAcceptedReceived {
            bidder: OPPONENT_PLAYER,
            amount: 99,
            new_timer_ms: 15_000,
        });
    app.world_mut()
        .write_message(ShopAuctionBidRejectedReceived {
            reason: BidRejectedReason::InsufficientGold,
        });
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id: CardId(3),
            starting_price: 12,
            timer_duration_ms: 20_000,
        });
    run_update(&mut app);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Hidden
    );
    assert_eq!(auction_state.card_id, None);
    assert_eq!(auction_state.current_price, 0);
    assert_eq!(auction_state.current_leader, None);
    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Inactive
    );
    assert!(!app.world().resource::<ShopAuctionToastState>().active);
}

#[test]
fn sau_008_late_shop_confirmations_are_ignored_after_phase_exit() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(10);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    app.world_mut().write_message(ShopAuctionShopSlotsReceived {
        slots: vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    });
    run_update(&mut app);

    set_phase(&mut app, RoundPhase::Placement, 45_000);
    app.world_mut().write_message(ShopAuctionShopSlotsReceived {
        slots: vec![Some(CardId(4)), Some(CardId(5)), Some(CardId(6))],
    });
    app.world_mut()
        .write_message(ShopAuctionShopCardAcquiredReceived { card_id: CardId(4) });
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Inactive
    );
    assert!(!app.world().resource::<ShopAuctionShopState>().slots_loaded);
    assert_eq!(shop_panel_visibility(&app), Visibility::Hidden);
    assert_eq!(app.world().resource::<PlayerEconomyView>().gold, 10);
}

#[test]
fn shop_slots_buffered_during_placement_apply_on_next_draft_shop() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(10);
    set_phase(&mut app, RoundPhase::Placement, 10_000);

    app.world_mut().write_message(ShopAuctionShopSlotsReceived {
        slots: vec![Some(CardId(4)), Some(CardId(5)), Some(CardId(6))],
    });
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Inactive
    );
    assert!(!app.world().resource::<ShopAuctionShopState>().slots_loaded);
    assert_eq!(shop_panel_visibility(&app), Visibility::Hidden);

    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::Shop
    );
    assert!(app.world().resource::<ShopAuctionShopState>().slots_loaded);
    assert_eq!(
        shop_slot_cards(&app),
        vec![Some(CardId(4)), Some(CardId(5)), Some(CardId(6))]
    );
    assert_eq!(
        shop_slot_states(&app),
        vec![
            ShopSlotState::Available,
            ShopSlotState::Available,
            ShopSlotState::Available
        ]
    );
    assert_eq!(shop_panel_visibility(&app), Visibility::Visible);
    assert_eq!(visible_shop_slot_count(&app), 3);
}

#[test]
fn sau_008_draft_initial_snapshot_does_not_restore_grid_from_shop_slots() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(10);
    set_phase(&mut app, RoundPhase::DraftInitial, 45_000);
    app.world_mut()
        .write_message(ShopAuctionDraftOfferingReceived {
            card_ids: (1..=SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT as u32)
                .map(CardId)
                .collect(),
        });
    run_update(&mut app);
    assert_eq!(
        visible_draft_slot_count(&app),
        SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT
    );

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::DraftInitial,
            Some(30_000),
            player_snapshot(
                LOCAL_PLAYER,
                10,
                0,
                Vec::new(),
                vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
            ),
            player_snapshot(OPPONENT_PLAYER, 20, 0, Vec::new(), Vec::new()),
            None,
        ),
    );

    assert_eq!(
        app.world().resource::<ShopAuctionUiMode>(),
        &ShopAuctionUiMode::DraftOffering
    );
    assert!(
        !app.world()
            .resource::<ShopAuctionDraftInitialState>()
            .offering_loaded
    );
    assert_eq!(visible_draft_slot_count(&app), 0);

    let first_slot = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_slots[0];
    app.world_mut()
        .write_message(ShopAuctionDraftSlotClicked { slot: first_slot });
    run_update(&mut app);
    assert!(app
        .world()
        .resource::<ShopAuctionUiOutboundMessages>()
        .purchase_cards
        .is_empty());
}

fn app_in_session(gold: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=12)
            .map(|id| {
                let card = test_card(id, id.min(5));
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
    app.insert_resource(PlayerEconomyView {
        gold,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold,
        reserved_gold: 0,
        initialized: true,
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

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(snapshot));
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

fn seed_stale_auction_transients(app: &mut App) {
    {
        let mut auction_state = app.world_mut().resource_mut::<ShopAuctionAuctionState>();
        auction_state.card_id = Some(CardId(9));
        auction_state.panel_state = ShopAuctionAuctionPanelState::Settling;
        auction_state.current_price = 99;
        auction_state.current_leader = Some(LOCAL_PLAYER);
        auction_state.in_flight_bid_amount = Some(100);
        auction_state.pending_bid_accepted = true;
        auction_state.pending_gold_broadcast_seen = true;
        auction_state.opponent_bid_gate_satisfied = true;
        auction_state.locally_expired_elapsed_ms = 1_000;
    }
    app.world_mut()
        .resource_mut::<ShopAuctionSettlementState>()
        .begin_for_test();
    app.world_mut()
        .resource_mut::<ShopAuctionToastState>()
        .show("stale rejection");
    *app.world_mut().resource_mut::<AuctionTimerTargetFill>() = AuctionTimerTargetFill {
        fill_pct: 0.25,
        new_timer_ms: 5_000,
        duration_ms: 20_000,
        updated: true,
    };
}

trait SettlementTestSeed {
    fn begin_for_test(&mut self);
}

impl SettlementTestSeed for ShopAuctionSettlementState {
    fn begin_for_test(&mut self) {
        self.outcome = Some(ShopAuctionSettlementOutcome::OpponentWinner);
        self.winner = Some(OPPONENT_PLAYER);
        self.amount = 99;
        self.card_id = Some(CardId(9));
        self.elapsed_ms = 1;
        self.transition_active = true;
    }
}

fn snapshot(
    phase: RoundPhase,
    timer_remaining_ms: Option<u32>,
    local_player: PlayerSnapshot,
    opponent_player: PlayerSnapshot,
    auction_state: Option<AuctionSnapshot>,
) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: LOCAL_PLAYER,
        round_number: 4,
        phase,
        timer_remaining_ms,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![local_player, opponent_player],
        board: BoardSnapshot::default(),
        auction_state,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(
    player_id: PlayerId,
    gold: u32,
    reserved_gold: u32,
    hand: Vec<CardId>,
    shop_slots: Vec<Option<CardId>>,
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold,
        reserved_gold,
        current_mana: 0,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand,
        shop_slots,
        pool_snapshot: Vec::new(),
        objectives: Vec::new(),
        opponent_objectives: Vec::new(),
    }
}

fn run_update(app: &mut App) {
    app.update();
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

fn shop_slot_states(app: &App) -> Vec<ShopSlotState> {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .shop_slots
        .iter()
        .map(|slot| {
            *app.world()
                .get::<ShopSlotState>(*slot)
                .expect("shop slot should have state")
        })
        .collect()
}

fn visible_draft_slot_count(app: &App) -> usize {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_slots
        .iter()
        .filter(|slot| app.world().get::<Visibility>(**slot) == Some(&Visibility::Visible))
        .count()
}

fn visible_shop_slot_count(app: &App) -> usize {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .shop_slots
        .iter()
        .filter(|slot| app.world().get::<Visibility>(**slot) == Some(&Visibility::Visible))
        .count()
}

fn test_card(id: u32, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Rare,
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
