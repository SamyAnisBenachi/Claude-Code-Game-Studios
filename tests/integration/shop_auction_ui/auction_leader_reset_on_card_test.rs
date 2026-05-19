//! PROMPT 1397 / S18-AUCTION-LEADER-RESET-ON-CARD-001 (AUDIT-1392-P03).
//!
//! Regression coverage for the auction-leader-stickiness defect surfaced by
//! the 2026-05-18 capture audit: across rounds R3 → R6 the previous winner's
//! `PlayerId` carried over into the next auction's empty-bid window because
//! `S2CAuctionCard` carries only `{ card_id, starting_price, timer_duration_ms }`
//! and the client `ShopAuctionAuctionState` was not explicitly reset by the
//! drain → handle path.
//!
//! This test bin seeds `ShopAuctionAuctionState` with stale leader / price /
//! timer / lead-loss state, then drains a fresh `S2CAuctionCard` through
//! `ShopAuctionAuctionCardReceived` and asserts that every audit-flagged
//! field is reset before `sync_auction_panel_system` reads the resource.
//!
//! The system-set order
//! (`MessageDrain` → `Input` → `StateSync`, configured in
//! `ShopAuctionUiPlugin`) guarantees the reset lands in the same frame as
//! the card arrival; the test asserts both the resource fields *and* the
//! derived `AuctionFeaturedCardLeadLossState` marker so a future code shift
//! that decoupled the marker from `current_leader` would also be caught.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    auction_featured_card_accent_color, AuctionFeaturedCardFrame, AuctionFeaturedCardLeadLossState,
    ShopAuctionAuctionCardReceived, ShopAuctionAuctionPanelState, ShopAuctionAuctionState,
    ShopAuctionCardCatalog, ShopAuctionLocalGoldView, ShopAuctionUiEntities, ShopAuctionUiMode,
    ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);

/// AC1 — the buffer_card chokepoint resets every audit-flagged field even
/// when seeded with stale leader / price / timer / in-flight-bid state from
/// a previous auction. Asserts both the `ShopAuctionAuctionState` resource
/// shape and the derived `AuctionFeaturedCardLeadLossState` marker.
#[test]
fn sau_1397_drain_new_card_clears_stale_leader_and_price_and_timer() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    // Enter DraftAuction so the new card path runs the active branch
    // (buffer_card → enter_active), matching the production R6 wire flow.
    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);

    // Seed a stale auction state as if the previous auction had ended with
    // OPPONENT_PLAYER as leader at a high bid and a near-empty timer. The
    // production lifecycle would normally clear this via the settlement
    // transition tick, but capturing the contract independently of that
    // path is the whole point of this regression.
    {
        let mut state = app.world_mut().resource_mut::<ShopAuctionAuctionState>();
        state.panel_state = ShopAuctionAuctionPanelState::Active;
        state.card_id = Some(CardId(7));
        state.starting_price = 3;
        state.current_price = 14;
        state.current_leader = Some(OPPONENT_PLAYER);
        state.timer_duration_ms = 20_000;
        state.timer_remaining_ms = 1_200;
        state.locally_expired_elapsed_ms = 4_500;
        state.in_flight_bid_amount = Some(17);
        state.pending_bid_accepted = true;
        state.pending_gold_broadcast_seen = true;
        state.opponent_bid_gate_satisfied = true;
    }

    // Drain a fresh card. New auction, new card_id, fresh starting price,
    // server-issued timer budget — and crucially nothing about a leader.
    let new_card_id = CardId(1);
    let new_starting_price = 4;
    let new_timer_duration_ms = 15_000;
    send_auction_card(&mut app, new_card_id, new_starting_price, new_timer_duration_ms);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.card_id,
        Some(new_card_id),
        "new card_id should bind on drain",
    );
    assert_eq!(
        auction_state.starting_price, new_starting_price,
        "starting_price should follow the new wire message",
    );
    assert_eq!(
        auction_state.current_price, new_starting_price,
        "current_price should snap back to the new starting_price",
    );
    assert_eq!(
        auction_state.current_leader, None,
        "AUDIT-1392-P03 — current_leader MUST be cleared on every new card",
    );
    assert_eq!(
        auction_state.timer_duration_ms, new_timer_duration_ms,
        "timer_duration_ms should bind to the new wire budget",
    );
    assert_eq!(
        auction_state.timer_remaining_ms, new_timer_duration_ms,
        "active-entry path should seed timer_remaining_ms = timer_duration_ms",
    );
    assert_eq!(
        auction_state.locally_expired_elapsed_ms, 0,
        "stale local-expiry count must not bleed into the new auction",
    );
    assert_eq!(
        auction_state.in_flight_bid_amount, None,
        "stale in-flight bid amount must be cleared (clear_bid_resolution_state contract)",
    );
    assert!(
        !auction_state.pending_bid_accepted,
        "stale pending-bid-accepted flag must be cleared",
    );
    assert!(
        !auction_state.opponent_bid_gate_satisfied,
        "stale opponent-bid-gate flag must be cleared",
    );
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Active,
        "DraftAuction entry should activate the panel",
    );

    // The derived featured-frame marker must agree with the cleared leader.
    let frame_entity = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_featured_card_frame;
    assert!(
        app.world()
            .get::<AuctionFeaturedCardFrame>(frame_entity)
            .is_some(),
        "featured frame entity should keep its story-016 marker",
    );
    let lead_loss_state = *app
        .world()
        .get::<AuctionFeaturedCardLeadLossState>(frame_entity)
        .expect("featured frame should carry lead/loss state");
    assert_eq!(
        lead_loss_state,
        AuctionFeaturedCardLeadLossState::Neutral,
        "lead-loss state must reset to Neutral before any bid lands",
    );
    let border_color = app
        .world()
        .get::<BorderColor>(frame_entity)
        .expect("featured frame should carry border color")
        .left;
    assert_eq!(
        border_color,
        auction_featured_card_accent_color(),
        "border color must reset to the accent (Neutral) palette",
    );
    assert_eq!(
        *app.world().resource::<ShopAuctionUiMode>(),
        ShopAuctionUiMode::Auction,
        "DraftAuction phase + buffered card should land in Auction mode",
    );
}

/// AC2 — same contract, but exercises the Preparing path (card arrives while
/// the phase is still Resolution / DraftShop / GameOver). The leader clear
/// is independent of which post-buffer entry state is chosen.
#[test]
fn sau_1397_drain_new_card_clears_stale_leader_in_preparing_path() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    // Stay in a transitional phase so the handler takes the buffer-and-
    // enter-Preparing branch (Surface C from PROMPT 684).
    set_phase(&mut app, RoundPhase::Resolution, 0);

    {
        let mut state = app.world_mut().resource_mut::<ShopAuctionAuctionState>();
        state.panel_state = ShopAuctionAuctionPanelState::Hidden;
        state.card_id = None;
        state.current_leader = Some(LOCAL_PLAYER);
        state.current_price = 9;
        state.in_flight_bid_amount = Some(12);
    }

    send_auction_card(&mut app, CardId(2), 5, 20_000);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Preparing,
        "transitional-phase card should buffer into Preparing, not Active",
    );
    assert_eq!(
        auction_state.current_leader, None,
        "current_leader must clear on every new card, Preparing path included",
    );
    assert_eq!(auction_state.current_price, 5);
    assert_eq!(auction_state.starting_price, 5);
    assert_eq!(
        auction_state.in_flight_bid_amount, None,
        "stale in-flight bid amount must be cleared",
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
    app.insert_resource(ShopAuctionCardCatalog {
        cards: HashMap::from([
            (CardId(1), test_card(1, Rarity::Rare, 4)),
            (CardId(2), test_card(2, Rarity::Common, 2)),
        ]),
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
