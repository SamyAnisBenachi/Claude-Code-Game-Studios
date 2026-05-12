//! S11-SAU-AUCTION-CARD-DROP-ON-PHASE-LAG-001 — regression tests for the
//! handle_auction_card_system buffer-then-defer path. Previously the system
//! silently dropped `S2CAuctionCard` messages that arrived while
//! `CurrentClientPhase` still held a transitional phase (DraftShop /
//! Resolution / GameOver), leaving the auction modal hidden when the
//! deferred DraftAuction phase change eventually drained from
//! `PendingPhaseChange` into `CurrentClientPhase`.
//!
//! Surface C from PROMPT 684 diagnostic (Stage A buffer in
//! `phase_sink_system` ⇒ Stage B drop in `handle_auction_card_system`).

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{
    AnimGroup, AnimQueue, AnimationTimingConfig, CardAnimationsPlugin, PendingPhaseChange,
};
use client::presentation::board_rendering::BoardRenderState;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    ShopAuctionAuctionCardReceived, ShopAuctionAuctionPanelState, ShopAuctionAuctionState,
    ShopAuctionCardCatalog, ShopAuctionUiEntities, ShopAuctionUiMode, ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{RoundPhase, S2CPhaseChanged};

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// AC1 — auction card delivered while `CurrentClientPhase` still holds
/// Resolution must be buffered (not dropped). When the phase later flips to
/// DraftAuction, `shop_auction_ui_phase_transition_system` must promote the
/// modal to Active and make the panel visible.
#[test]
fn sau_card_arrives_during_resolution_then_phase_change_activates_modal() {
    test_helpers::init_test_tracing();
    let mut app = shop_app_in_session();

    // Drive the session through Placement before Resolution so the round
    // counter matches the expected wire ordering and the phase transition
    // system reaches the Resolution branch via `current.is_changed()`.
    set_phase(&mut app, RoundPhase::Placement, 45_000);
    set_phase(&mut app, RoundPhase::Resolution, 0);
    assert_eq!(
        app.world()
            .resource::<ShopAuctionAuctionState>()
            .panel_state,
        ShopAuctionAuctionPanelState::Hidden,
        "Resolution entry should leave the auction panel hidden",
    );

    send_auction_card(&mut app, CardId(2), 5);

    {
        let auction_state = app.world().resource::<ShopAuctionAuctionState>();
        assert_eq!(
            auction_state.panel_state,
            ShopAuctionAuctionPanelState::Preparing,
            "auction card during Resolution must be buffered into Preparing, not dropped",
        );
        assert_eq!(auction_state.card_id, Some(CardId(2)));
        assert_eq!(auction_state.starting_price, 5);
    }
    assert_eq!(
        *app.world().resource::<ShopAuctionUiMode>(),
        ShopAuctionUiMode::AuctionPreparing,
    );

    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Active,
        "DraftAuction phase change must activate the buffered card",
    );
    assert_eq!(auction_state.card_id, Some(CardId(2)));
    assert_eq!(auction_state.timer_duration_ms, 20_000);
    assert_eq!(auction_state.timer_remaining_ms, 20_000);
    assert_eq!(
        *app.world().resource::<ShopAuctionUiMode>(),
        ShopAuctionUiMode::Auction,
    );
    assert_eq!(auction_panel_visibility(&app), Some(&Visibility::Visible));
}

/// AC2 — full Surface A + Surface C drain. `phase_sink_system` defers the
/// DraftAuction `S2CPhaseChanged` because `BoardRenderState` is
/// `ResolutionExecuting`. While the phase change is in `PendingPhaseChange`,
/// `S2CAuctionCard` arrives — `handle_auction_card_system` must buffer it
/// (Surface C fix). When the resolution queue drains, the pending phase
/// change applies, `current.is_changed()` triggers
/// `shop_auction_ui_phase_transition_system`, and the buffered card is
/// promoted to Active with the panel visible.
#[test]
fn sau_pending_phase_change_drains_with_card_then_modal_visible() {
    test_helpers::init_test_tracing();
    let mut app = shop_app_in_session_with_animations();

    // Approach Resolution from Placement so the auction panel is cleared
    // and the round counter is consistent with a mid-game resolution.
    set_phase(&mut app, RoundPhase::Placement, 45_000);
    set_phase(&mut app, RoundPhase::Resolution, 0);

    // Stage A: a DraftAuction phase change is held in PendingPhaseChange
    // while BoardRenderState is ResolutionExecuting.
    app.insert_resource(BoardRenderState::ResolutionExecuting);
    *app.world_mut().resource_mut::<AnimQueue>() =
        AnimQueue::from_groups(vec![AnimGroup::new(1, 200, Vec::new())]);
    {
        let next_round = app.world().resource::<CurrentClientPhase>().round + 1;
        app.world_mut()
            .resource_mut::<PendingPhaseChange>()
            .set(S2CPhaseChanged {
                phase: RoundPhase::DraftAuction,
                round_number: next_round,
                timer_duration_ms: 20_000,
            });
    }

    // Surface C: auction card arrives while CurrentClientPhase is still
    // Resolution. Pre-fix this was dropped via `continue`; post-fix it is
    // buffered into Preparing and waits for the phase transition.
    send_auction_card(&mut app, CardId(3), 6);
    {
        let auction_state = app.world().resource::<ShopAuctionAuctionState>();
        assert_eq!(
            auction_state.panel_state,
            ShopAuctionAuctionPanelState::Preparing,
            "Surface C: auction card during ResolutionExecuting must be buffered",
        );
        assert_eq!(auction_state.card_id, Some(CardId(3)));
    }
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::Resolution,
        "CurrentClientPhase should still be Resolution while drain is gated",
    );

    // Drain the resolution queue. resolution_executing_system applies the
    // buffered PendingPhaseChange when the last group completes.
    run_for(&mut app, Duration::from_millis(250));
    assert!(
        app.world().resource::<PendingPhaseChange>().is_none(),
        "PendingPhaseChange must drain after AnimQueue completion",
    );
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::DraftAuction,
        "phase must apply once the resolution queue drains",
    );

    // The next frame surfaces `current.is_changed()` to
    // `shop_auction_ui_phase_transition_system`, which promotes the
    // buffered Preparing state to Active.
    run_update(&mut app);

    let auction_state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Active,
        "buffered card must promote to Active when phase drains into DraftAuction",
    );
    assert_eq!(auction_state.card_id, Some(CardId(3)));
    assert_eq!(
        *app.world().resource::<ShopAuctionUiMode>(),
        ShopAuctionUiMode::Auction,
    );
    assert_eq!(auction_panel_visibility(&app), Some(&Visibility::Visible));
}

fn shop_app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=5)
            .map(|id| {
                let card = test_card(id, Rarity::Rare, id + 2);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 20,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
}

fn shop_app_in_session_with_animations() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(CardAnimationsPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=5)
            .map(|id| {
                let card = test_card(id, Rarity::Rare, id + 2);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 20,
        initialized: true,
        ..default()
    });
    // Smaller drain budgets keep the test fast while still exercising the
    // empty-queue + final-group code paths in resolution_executing_system.
    app.insert_resource(AnimationTimingConfig {
        pre_animation_pause_ms: 50,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
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

fn send_auction_card(app: &mut App, card_id: CardId, starting_price: u32) {
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id,
            starting_price,
            timer_duration_ms: 20_000,
        });
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
        let step = remaining.min(Duration::from_millis(50));
        *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
            TimeUpdateStrategy::ManualDuration(step);
        app.update();
        remaining = remaining.saturating_sub(step);
    }
}

fn auction_panel_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .auction_panel,
    )
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
