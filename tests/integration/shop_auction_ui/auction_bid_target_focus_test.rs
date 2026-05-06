use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hud::HudGoldBroadcastMessage;
use client::ui::shop_auction::{
    AuctionBidButton, AuctionBidButtonState, AuctionBidFocusState, AuctionBidTargetBounds,
    ShopAuctionAuctionCardReceived, ShopAuctionAuctionState, ShopAuctionBidButtonClicked,
    ShopAuctionCardCatalog, ShopAuctionDraftHandView, ShopAuctionLocalGoldView,
    ShopAuctionUiEntities, ShopAuctionUiOutboundMessages, ShopAuctionUiPlugin,
    AUCTION_BID_FOCUS_RING_WIDTH_PX,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{RoundPhase, S2CGoldBroadcast};
use shared::session::PlayerId;

const LOCAL_PLAYER: PlayerId = PlayerId(1);

#[test]
fn sau_011_bid_targets_meet_44px_minimum_and_preserve_split_labels() {
    let app = app_in_active_auction(4, 20_000, 20, 0);

    for bounds in bid_target_bounds(&app) {
        assert!(bounds.meets_minimum_target());
    }
    for (width, height) in bid_node_sizes(&app) {
        assert_px_at_least(width, 44.0);
        assert_px_at_least(height, 44.0);
    }
    assert_eq!(bid_button_texts(&app), ["5g\n(+1)", "7g\n(+3)", "9g\n(+5)"]);
    assert_eq!(bid_focus_order(&app), [(1, 1), (2, 3), (3, 5)]);
}

#[test]
fn sau_011_keyboard_focus_advances_in_bid_order_and_enter_sends_one_bid() {
    let mut app = app_in_active_auction(4, 20_000, 20, 0);

    press_key(&mut app, KeyCode::Tab);
    assert_focused_increment(&app, Some(1));

    press_key(&mut app, KeyCode::Tab);
    assert_focused_increment(&app, Some(3));
    assert_focused_button_exposes_ring(&app, 3);

    press_key(&mut app, KeyCode::Enter);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.place_bids.len(), 1);
    assert_eq!(outbound.place_bids[0].amount, 7);
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::GenericDisabled,
            AuctionBidButtonState::InFlight,
            AuctionBidButtonState::GenericDisabled
        ]
    );
    assert_eq!(bid_button_texts(&app)[1], "BIDDING...");

    press_key(&mut app, KeyCode::Enter);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .place_bids
            .len(),
        1
    );
}

#[test]
fn sau_011_disabled_and_hidden_bid_controls_are_skipped_by_focus() {
    let mut app = app_in_active_auction(0, 20_000, 2, 0);

    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Unaffordable,
            AuctionBidButtonState::Unaffordable
        ]
    );
    assert_eq!(bid_focusability(&app), [(1, true), (3, false), (5, false)]);

    press_key(&mut app, KeyCode::Tab);
    assert_focused_increment(&app, Some(1));
    press_key(&mut app, KeyCode::Tab);
    assert_focused_increment(&app, Some(1));

    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .current_leader = Some(LOCAL_PLAYER);
    run_update(&mut app);

    assert_eq!(
        bid_button_visibilities(&app),
        [Visibility::Hidden, Visibility::Hidden, Visibility::Hidden]
    );
    assert_eq!(bid_focusability(&app), [(1, false), (3, false), (5, false)]);
    assert_focused_increment(&app, None);

    press_key(&mut app, KeyCode::Tab);
    assert_focused_increment(&app, None);
}

#[test]
fn sau_011_pointer_interaction_still_uses_existing_one_send_path() {
    let mut app = app_in_active_auction(4, 20_000, 20, 0);
    press_bid_interaction(&mut app, 2);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.place_bids.len(), 1);
    assert_eq!(outbound.place_bids[0].amount, 9);
    assert_eq!(bid_button_texts(&app)[2], "BIDDING...");

    let button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons[2];
    app.world_mut()
        .write_message(ShopAuctionBidButtonClicked { button });
    run_update(&mut app);

    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .place_bids
            .len(),
        1
    );
}

fn app_in_active_auction(
    starting_price: u32,
    timer_duration_ms: u32,
    gold: u32,
    reserved_gold: u32,
) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: HashMap::from([(CardId(1), test_card(1, Rarity::Rare, 4))]),
    });
    app.insert_resource(PlayerEconomyView {
        gold,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold,
        reserved_gold,
        initialized: true,
    });
    app.insert_resource(ShopAuctionDraftHandView { hand_size: 0 });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    set_phase(&mut app, RoundPhase::DraftAuction, timer_duration_ms);
    send_auction_card(&mut app, CardId(1), starting_price);
    write_local_gold_broadcast(&mut app, gold, reserved_gold);
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

fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    run_update(app);
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(key);
        input.clear();
    }
    run_update(app);
}

fn press_bid_interaction(app: &mut App, index: usize) {
    let button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons[index];
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("bid button should have Interaction") = Interaction::Pressed;
    run_update(app);
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("bid button should have Interaction") = Interaction::None;
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn bid_target_bounds(app: &App) -> [AuctionBidTargetBounds; 3] {
    bid_button_entities(app).map(|button| {
        *app.world()
            .get::<AuctionBidTargetBounds>(button)
            .expect("bid button should expose target bounds")
    })
}

fn bid_node_sizes(app: &App) -> [(Val, Val); 3] {
    bid_button_entities(app).map(|button| {
        let node = app
            .world()
            .get::<Node>(button)
            .expect("bid button should have a node");
        (node.width, node.height)
    })
}

fn bid_button_texts(app: &App) -> [String; 3] {
    bid_button_entities(app).map(|button| {
        app.world()
            .get::<Text>(button)
            .expect("bid button should have text")
            .0
            .clone()
    })
}

fn bid_button_states(app: &App) -> [AuctionBidButtonState; 3] {
    bid_button_entities(app).map(|button| {
        *app.world()
            .get::<AuctionBidButtonState>(button)
            .expect("bid button should have state")
    })
}

fn bid_button_visibilities(app: &App) -> [Visibility; 3] {
    bid_button_entities(app).map(|button| {
        *app.world()
            .get::<Visibility>(button)
            .expect("bid button should have visibility")
    })
}

fn bid_focus_order(app: &App) -> [(u8, u32); 3] {
    bid_button_entities(app).map(|button| {
        let bid_button = app
            .world()
            .get::<AuctionBidButton>(button)
            .expect("bid button should have increment");
        let focus_state = app
            .world()
            .get::<AuctionBidFocusState>(button)
            .expect("bid button should have focus state");
        (focus_state.order, bid_button.increment)
    })
}

fn bid_focusability(app: &App) -> [(u32, bool); 3] {
    bid_button_entities(app).map(|button| {
        let bid_button = app
            .world()
            .get::<AuctionBidButton>(button)
            .expect("bid button should have increment");
        let focus_state = app
            .world()
            .get::<AuctionBidFocusState>(button)
            .expect("bid button should have focus state");
        (bid_button.increment, focus_state.focusable)
    })
}

fn assert_focused_increment(app: &App, expected: Option<u32>) {
    let focused = bid_button_entities(app).iter().find_map(|button| {
        let bid_button = app.world().get::<AuctionBidButton>(*button)?;
        let focus_state = app.world().get::<AuctionBidFocusState>(*button)?;
        focus_state.focused.then_some(bid_button.increment)
    });
    assert_eq!(focused, expected);
}

fn assert_focused_button_exposes_ring(app: &App, increment: u32) {
    let button = bid_button_entities(app)
        .into_iter()
        .find(|button| {
            app.world()
                .get::<AuctionBidButton>(*button)
                .is_some_and(|bid_button| bid_button.increment == increment)
        })
        .expect("focused increment should exist");
    let focus_state = app
        .world()
        .get::<AuctionBidFocusState>(button)
        .expect("bid button should have focus state");
    assert!(focus_state.focused);
    assert!(focus_state.focus_ring_visible);
    assert_eq!(
        focus_state.focus_ring_width_px,
        AUCTION_BID_FOCUS_RING_WIDTH_PX
    );

    let node = app
        .world()
        .get::<Node>(button)
        .expect("bid button should have node");
    assert_eq!(node.border.left, Val::Px(AUCTION_BID_FOCUS_RING_WIDTH_PX));
    assert_eq!(node.border.right, Val::Px(AUCTION_BID_FOCUS_RING_WIDTH_PX));
}

fn bid_button_entities(app: &App) -> [Entity; 3] {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons
}

fn assert_px_at_least(value: Val, minimum: f32) {
    let Val::Px(px) = value else {
        panic!("expected px value, got {value:?}");
    };
    assert!(px >= minimum, "expected {px} >= {minimum}");
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
