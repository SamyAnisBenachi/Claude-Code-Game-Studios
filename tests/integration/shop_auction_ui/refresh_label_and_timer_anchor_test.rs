//! PROMPT 1245 — S18-SHOP-AUCTION-TIMER-COST-CLARITY-001.
//!
//! Two narrow clarity guards for the DraftShop / DraftAuction UI:
//!
//! 1. The shop **Refresh** button label always surfaces the next refresh
//!    cost (`Refresh (Ng)`), increments after each confirmed refresh, and
//!    stays capped at `refresh_base_cost + refresh_cap` once the cap is
//!    reached. Mirrors the server-side cost formula already represented
//!    in `ShopAuctionRefreshConfig` and `ShopAuctionShopState.refresh_count_this_draft`.
//!
//! 2. The auction `timer_remaining_ms` is server-anchored on
//!    `apply_bid_accepted`: when the server's `new_timer_ms` exceeds the
//!    previously known `timer_duration_ms` (an extension), the duration
//!    bar is bumped so the bar can fit, and the visible remaining time
//!    matches the server. After a local-expiry tick to zero, a fresh
//!    server `S2CAuctionBidAccepted` re-anchors the timer so bid buttons
//!    can come back from the locally-expired state — server settlement
//!    remains authoritative, the client is purely a display.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    ShopAuctionAuctionCardReceived, ShopAuctionAuctionPanelState, ShopAuctionAuctionState,
    ShopAuctionBidAcceptedReceived, ShopAuctionCardCatalog, ShopAuctionLocalGoldView,
    ShopAuctionShopRefreshClicked, ShopAuctionShopSlotsReceived, ShopAuctionShopState,
    ShopAuctionUiEntities, ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);

/// PROMPT 1245 (AC1) — The Refresh button label starts at
/// `Refresh (1g)`, increments to `Refresh (2g)` after the first
/// confirmed `S2CShopSlots` refresh, and stays capped at `(2g)` on
/// subsequent refreshes with the default `refresh_base_cost = 1` /
/// `refresh_cap = 1` config. Mirrors GDD SAU-V12 with the prompt's
/// requested parenthesised format.
#[test]
fn prompt_1245_refresh_label_surfaces_cost_and_caps_at_base_plus_cap() {
    test_helpers::init_test_tracing();
    let mut app = active_shop_app(
        /* gold */ 9,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    let refresh_button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .shop_refresh_button;

    assert_eq!(
        refresh_label(&app, refresh_button),
        "Refresh (1g)",
        "initial label must surface the base refresh cost in parens",
    );

    // First confirmed refresh: click → server delivers fresh slots →
    // `refresh_count_this_draft` advances from 0 → 1, so the label moves
    // to (2g).
    click_refresh(&mut app, refresh_button);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(4)), Some(CardId(5)), Some(CardId(6))],
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionShopState>()
            .refresh_count_this_draft,
        1,
    );
    assert_eq!(
        refresh_label(&app, refresh_button),
        "Refresh (2g)",
        "label must increment to the next-refresh cost after a confirmed refresh",
    );

    // Second confirmed refresh: `refresh_count_this_draft` reaches the
    // cap and the displayed cost stays at 2g (refresh_base_cost +
    // refresh_cap, default 1 + 1 = 2).
    click_refresh(&mut app, refresh_button);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionShopState>()
            .refresh_count_this_draft,
        2,
    );
    assert_eq!(
        refresh_label(&app, refresh_button),
        "Refresh (2g)",
        "label must stay capped at the (base + cap) cost on subsequent refreshes",
    );
}

/// PROMPT 1245 (AC2 part a) — When `S2CAuctionBidAccepted` arrives with
/// `new_timer_ms <= timer_duration_ms` (the common case), the remaining
/// time matches `new_timer_ms` exactly and the duration is unchanged.
#[test]
fn prompt_1245_bid_accepted_anchors_remaining_ms_to_server_value_within_duration() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(/* starting_price */ 4, /* duration */ 20_000);

    // Sanity: fresh auction starts with remaining == duration.
    {
        let state = app.world().resource::<ShopAuctionAuctionState>();
        assert_eq!(state.timer_duration_ms, 20_000);
        assert_eq!(state.timer_remaining_ms, 20_000);
    }

    write_bid_accepted(&mut app, OPPONENT_PLAYER, 5, /* new_timer_ms */ 8_000);

    let state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(state.timer_duration_ms, 20_000, "duration must not shrink");
    assert_eq!(
        state.timer_remaining_ms, 8_000,
        "remaining must equal the server's new_timer_ms",
    );
    assert_eq!(state.current_price, 5);
    assert_eq!(state.current_leader, Some(OPPONENT_PLAYER));
}

/// PROMPT 1245 (AC2 part b) — When `S2CAuctionBidAccepted` arrives with
/// `new_timer_ms > timer_duration_ms` (a server-side extension on a late
/// bid), the client honours the server: it bumps `timer_duration_ms` to
/// fit the new remaining time and sets `timer_remaining_ms` to the
/// server's exact value. Prevents the prior `.min(timer_duration_ms)`
/// clamp from understating remaining seconds and leaving bid buttons in
/// a phantom-disabled `LocallyExpired` state after the clamped value
/// ticks to zero even though the server still considers the auction
/// live.
#[test]
fn prompt_1245_bid_accepted_extends_duration_when_server_new_timer_exceeds_prior_duration() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(/* starting_price */ 1, /* duration */ 5_000);

    write_bid_accepted(&mut app, OPPONENT_PLAYER, 2, /* new_timer_ms */ 12_000);

    let state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(
        state.timer_duration_ms, 12_000,
        "duration must grow to fit the extended remaining time",
    );
    assert_eq!(
        state.timer_remaining_ms, 12_000,
        "remaining must equal the server's exact new_timer_ms, never clamped",
    );
    assert_eq!(state.current_price, 2);
}

/// PROMPT 1245 (AC2 part c) — After the local countdown reaches expiry
/// (`timer_remaining_ms == 0`), the bid pipeline is parked: panel state
/// is `locally_expired`, bid buttons read as `LocallyExpired`, and clicks
/// are rejected by `handle_auction_bid_button_click_system`. A
/// subsequent server `S2CAuctionBidAccepted` re-anchors the timer to the
/// server's `new_timer_ms` and clears `locally_expired_elapsed_ms`, so
/// the panel can come back from the expired state — without permitting
/// a "phantom click" in the window between local-zero and the next
/// server message.
#[test]
fn prompt_1245_bid_accepted_after_local_expiry_restores_timer_from_server() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(/* starting_price */ 1, /* duration */ 5_000);

    // Force local expiry by zeroing remaining and parking the
    // expired-elapsed counter at a non-zero value (matching what the
    // production tick system writes after several frames at zero).
    {
        let mut state = app.world_mut().resource_mut::<ShopAuctionAuctionState>();
        state.timer_remaining_ms = 0;
        state.locally_expired_elapsed_ms = 250;
    }
    {
        let state = app.world().resource::<ShopAuctionAuctionState>();
        assert!(
            state.locally_expired(),
            "panel must report locally_expired = true with remaining == 0 and panel Active",
        );
    }

    // Server extends the auction with a late accepted bid.
    write_bid_accepted(&mut app, OPPONENT_PLAYER, 4, /* new_timer_ms */ 6_000);

    let state = app.world().resource::<ShopAuctionAuctionState>();
    assert_eq!(state.panel_state, ShopAuctionAuctionPanelState::Active);
    assert_eq!(
        state.timer_duration_ms, 6_000,
        "duration must extend to match server when prior duration was smaller",
    );
    assert_eq!(
        state.timer_remaining_ms, 6_000,
        "remaining must equal the server's new_timer_ms — no .min() clamp",
    );
    assert_eq!(
        state.locally_expired_elapsed_ms, 0,
        "locally_expired_elapsed_ms must reset once the server re-anchors the timer",
    );
    assert!(
        !state.locally_expired(),
        "after server re-anchor the panel must leave the locally_expired state",
    );
}

// --- test fixture plumbing -------------------------------------------------

fn active_shop_app(gold: u32, slots: Vec<Option<CardId>>) -> App {
    let mut app = build_app(gold);
    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    send_shop_slots(&mut app, slots);
    app
}

fn app_in_active_auction(starting_price: u32, timer_duration_ms: u32) -> App {
    let mut app = build_app(/* gold */ 20);
    set_phase(&mut app, RoundPhase::DraftAuction, timer_duration_ms);
    send_auction_card(&mut app, CardId(1), starting_price, timer_duration_ms);
    app
}

fn build_app(gold: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=6)
            .map(|id| {
                let card = test_card(id, Rarity::Common, id.min(3));
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

fn send_shop_slots(app: &mut App, slots: Vec<Option<CardId>>) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotsReceived { slots });
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

fn write_bid_accepted(app: &mut App, bidder: PlayerId, amount: u32, new_timer_ms: u32) {
    app.world_mut()
        .write_message(ShopAuctionBidAcceptedReceived {
            bidder,
            amount,
            new_timer_ms,
        });
    run_update(app);
}

fn click_refresh(app: &mut App, button: Entity) {
    app.world_mut()
        .write_message(ShopAuctionShopRefreshClicked { button });
    run_update(app);
}

fn run_update(app: &mut App) {
    app.update();
}

fn refresh_label(app: &App, button: Entity) -> String {
    app.world()
        .get::<Text>(button)
        .expect("refresh button must carry a Text component")
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
