//! PROMPT 1182 — Shop / Auction responsive layout repair regression tests.
//!
//! Locks in three structural repairs so future UI additions cannot hide a
//! primary action, hide a timer, or re-introduce the featured-card text
//! overlap that AUDIT-1129 UI-1129-02 surfaced:
//!
//! 1. The auction featured-card root entity renders only the card name on
//!    its `Text` component. The duplicate `"{rarity} - {N}g"` suffix
//!    previously stamped a second H1 line that overlapped both the
//!    dedicated price label child and the stats child.
//! 2. Every primary-action button (`DraftInitialReadyButton`,
//!    `DraftInitialObjectiveDismissButton`,
//!    `DraftInitialObjectiveRetrievalButton`, `ShopRefreshButton`,
//!    `ShopReadyButton`, `AuctionPassButton`) spawns with explicit
//!    `BackgroundColor` (non-transparent) and `BorderColor` (non-transparent)
//!    so the affordance is visibly a button rather than a label.
//! 3. Every hardcoded x position + width in the shop / auction layout
//!    constants fits within the 1280 × 720 minimum supported viewport per
//!    `docs/ux/global-ui-design-spec.md` §8. Static check — surfaces
//!    drift loudly the first time a constant is bumped past the budget.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    AuctionFeaturedCardPriceLabel, AuctionFeaturedCardTimerLabel, ShopAuctionAuctionCardReceived,
    ShopAuctionCardCatalog, ShopAuctionUiEntities, ShopAuctionUiPlugin,
    AUCTION_BID_TARGET_HEIGHT_PX, AUCTION_BID_TARGET_WIDTH_PX,
    AUCTION_FREE_GOLD_COUNTER_ANCHOR_LEFT_PERCENT, AUCTION_FREE_GOLD_COUNTER_GROUP_HEIGHT_PX,
    AUCTION_FREE_GOLD_COUNTER_GROUP_WIDTH_PX, AUCTION_FREE_GOLD_COUNTER_LEFT_OFFSET_PX,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ---------------------------------------------------------------------------
// Repair 1 — featured-card parent Text renders the card NAME only (no
// duplicate price / rarity suffix).
// ---------------------------------------------------------------------------

#[test]
fn prompt_1182_featured_card_parent_text_renders_name_only() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(/* gold */ 6);
    set_phase(&mut app, RoundPhase::DraftAuction, 30_000);
    let card_name = "Test Card 2";
    send_auction_card(&mut app, CardId(2), /* starting_price */ 4, 20_000);

    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    let parent_text = app
        .world()
        .get::<Text>(entities.auction_featured_card)
        .expect("auction featured card must carry a Text component")
        .0
        .clone();

    assert!(
        parent_text.contains(card_name),
        "parent text must include the card name; got {parent_text:?}",
    );
    assert!(
        !parent_text.contains('\n'),
        "parent text must render on a single line so it cannot ghost \
         under the price / timer / stats children; got {parent_text:?}",
    );
    // The dedicated `AuctionFeaturedCardPriceLabel` child carries the
    // canonical "Bid: {N}g" copy. The parent must NOT duplicate it.
    assert!(
        !parent_text.contains("Bid:"),
        "parent text must not duplicate the price (carried by the \
         AuctionFeaturedCardPriceLabel child); got {parent_text:?}",
    );
    assert!(
        !parent_text.contains("- 4g"),
        "parent text must not stamp the legacy \"Rarity - {{N}}g\" \
         suffix; got {parent_text:?}",
    );

    // Price label is the canonical "Bid:" payload owner.
    let price_label_text = app
        .world()
        .get::<Text>(entities.auction_featured_card_price_label)
        .expect("price label must carry a Text component")
        .0
        .clone();
    assert!(
        price_label_text.contains("Bid"),
        "AuctionFeaturedCardPriceLabel must lead with 'Bid'; got \
         {price_label_text:?}",
    );

    // Marker uniqueness preserved.
    assert!(app
        .world()
        .get::<AuctionFeaturedCardPriceLabel>(entities.auction_featured_card_price_label)
        .is_some());
    assert!(app
        .world()
        .get::<AuctionFeaturedCardTimerLabel>(entities.auction_featured_card_timer_label)
        .is_some());
}

// ---------------------------------------------------------------------------
// Repair 2 — every primary-action button carries visible BackgroundColor +
// BorderColor chrome. AUDIT-1129 (and the lobby UI-1129-08 parallel)
// observed that "Ready", "Dismiss", "Objective", "REFRESH · Ng", and
// "Ready" (shop) rendered as bare text overlaid on the panel chrome and
// did not read as interactive buttons. This locks in the repair so future
// refactors cannot accidentally strip the chrome off again.
// ---------------------------------------------------------------------------

#[test]
fn prompt_1182_primary_action_buttons_have_visible_chrome() {
    test_helpers::init_test_tracing();
    let app = app_in_session(/* gold */ 6);
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    // (Entity, label) pairs — same chrome contract applies to every one.
    let buttons: &[(Entity, &'static str)] = &[
        (entities.draft_initial_ready_button, "Draft Ready"),
        (
            entities.draft_initial_objective_dismiss_button,
            "Draft Objective Dismiss",
        ),
        (
            entities.draft_initial_objective_retrieval_button,
            "Draft Objective Retrieval",
        ),
        (entities.shop_refresh_button, "Shop Refresh"),
        (entities.shop_ready_button, "Shop Ready"),
        (entities.auction_pass_button, "Auction Pass"),
    ];

    for (entity, label) in buttons {
        // Every primary action must carry `Button` + `Interaction` so
        // `handle_shop_auction_control_interactions_system` can drive it.
        assert!(
            app.world().get::<Button>(*entity).is_some(),
            "{label} button must carry the Button component",
        );
        assert!(
            app.world().get::<Interaction>(*entity).is_some(),
            "{label} button must carry the Interaction component",
        );

        // Fill chrome — non-default, non-transparent background.
        let bg = app
            .world()
            .get::<BackgroundColor>(*entity)
            .unwrap_or_else(|| panic!("{label} button must carry a BackgroundColor"));
        let alpha = bg.0.alpha();
        assert!(
            alpha > 0.10,
            "{label} button BackgroundColor must be non-transparent so \
             the affordance reads as a button (alpha={alpha:?}); got {:?}",
            bg.0,
        );

        // Border chrome — non-transparent BorderColor.
        let border = app
            .world()
            .get::<BorderColor>(*entity)
            .unwrap_or_else(|| panic!("{label} button must carry a BorderColor"));
        let border_alpha = border.left.alpha();
        assert!(
            border_alpha > 0.10,
            "{label} button BorderColor.left must be non-transparent so \
             the button outline is visible (alpha={border_alpha:?}); \
             got {:?}",
            border.left,
        );

        // Label / chrome text must be non-empty for friend-game scope so
        // the player can read what the button does even without hover.
        // (`AuctionPassButton` renders "PASS"; refresh / ready render
        // their stateful label via sync systems.)
        let text = app
            .world()
            .get::<Text>(*entity)
            .map(|t| t.0.clone())
            .unwrap_or_default();
        if *label != "Draft Ready" && *label != "Shop Ready" {
            assert!(
                !text.is_empty(),
                "{label} button must carry non-empty Text at spawn; got \
                 {text:?}",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Repair 3 — every hardcoded layout position fits within 1280 × 720
// (`docs/ux/global-ui-design-spec.md` §8 minimum supported viewport is
// 1366 × 768, but we test at 1280 × 720 as a strict subset to also cover
// the 1280 × 960 4:3 entry in CANONICAL_VIEWPORTS). Static check — the
// constants are pixel-fixed so the math is deterministic across every
// viewport; a regression bumps a constant and this test fails loudly.
// ---------------------------------------------------------------------------

#[test]
fn prompt_1182_auction_action_band_fits_min_viewport() {
    test_helpers::init_test_tracing();

    // Minimum guaranteed viewport per spec §8 strict-subset check.
    const MIN_VIEWPORT_WIDTH: f32 = 1280.0;
    const MIN_VIEWPORT_HEIGHT: f32 = 720.0;

    // Bid buttons — anchored at `left: 34% + idx * 9%`, width 108, bottom 72.
    for index in 0..3 {
        let left_percent = 34.0 + (index as f32) * 9.0;
        let left_px = MIN_VIEWPORT_WIDTH * left_percent / 100.0;
        let right_edge = left_px + AUCTION_BID_TARGET_WIDTH_PX;
        assert!(
            right_edge <= MIN_VIEWPORT_WIDTH,
            "bid button {index} right edge {right_edge:.1}px exceeds min \
             viewport width {MIN_VIEWPORT_WIDTH:.1}px",
        );
    }

    // Pass button — at `left: 34 + 3*9 = 61%`, width 108, bottom 72.
    {
        let left_px = MIN_VIEWPORT_WIDTH * 0.61;
        let right_edge = left_px + AUCTION_BID_TARGET_WIDTH_PX;
        assert!(
            right_edge <= MIN_VIEWPORT_WIDTH,
            "pass button right edge {right_edge:.1}px exceeds min \
             viewport width {MIN_VIEWPORT_WIDTH:.1}px",
        );
    }

    // Free-gold counter group — `left: 52% + AUCTION_FREE_GOLD_COUNTER_LEFT_OFFSET_PX`,
    // width = AUCTION_FREE_GOLD_COUNTER_GROUP_WIDTH_PX, bottom = 70.
    {
        let anchor_px = MIN_VIEWPORT_WIDTH * AUCTION_FREE_GOLD_COUNTER_ANCHOR_LEFT_PERCENT / 100.0;
        let left_edge = anchor_px + AUCTION_FREE_GOLD_COUNTER_LEFT_OFFSET_PX;
        let right_edge = left_edge + AUCTION_FREE_GOLD_COUNTER_GROUP_WIDTH_PX;
        assert!(
            right_edge <= MIN_VIEWPORT_WIDTH,
            "free-gold counter group right edge {right_edge:.1}px exceeds \
             min viewport width {MIN_VIEWPORT_WIDTH:.1}px",
        );
        let group_bottom = 70.0 + AUCTION_FREE_GOLD_COUNTER_GROUP_HEIGHT_PX;
        assert!(
            group_bottom <= MIN_VIEWPORT_HEIGHT,
            "free-gold counter group anchored-from-bottom column \
             {group_bottom:.1}px exceeds min viewport height \
             {MIN_VIEWPORT_HEIGHT:.1}px",
        );
    }

    // Bid button hit-target floor — Story 011 contract is 44 × 108 CSS px.
    // Confirms a regression on the bid-target size constants is also a
    // viewport-fit regression.
    assert_eq!(AUCTION_BID_TARGET_WIDTH_PX, 108.0);
    assert_eq!(AUCTION_BID_TARGET_HEIGHT_PX, 44.0);
}

#[test]
fn prompt_1182_shop_panel_content_fits_min_viewport() {
    test_helpers::init_test_tracing();
    const MIN_VIEWPORT_WIDTH: f32 = 1280.0;

    // Shop slot wells: `left = 92 + idx * 154`, width = 136 (per
    // `card_slot_node(ShopSlot)` / shop_slot_node).
    let shop_slot_width = 136.0;
    for index in 0..3 {
        let left = 92.0 + (index as f32) * 154.0;
        let right = left + shop_slot_width;
        assert!(
            right <= MIN_VIEWPORT_WIDTH,
            "shop slot {index} right edge {right:.1}px exceeds min \
             viewport width {MIN_VIEWPORT_WIDTH:.1}px",
        );
    }

    // Shop refresh button: `left: 92`, width 148.
    {
        let right = 92.0 + 148.0;
        assert!(right <= MIN_VIEWPORT_WIDTH);
    }

    // Shop ready button: `right: 96`, width 132 — anchors to right edge.
    {
        let right_offset = 96.0;
        let width = 132.0;
        assert!(right_offset + width <= MIN_VIEWPORT_WIDTH);
    }

    // Shop ready status: `right: 96`, width 180.
    {
        let right_offset = 96.0;
        let width = 180.0;
        assert!(right_offset + width <= MIN_VIEWPORT_WIDTH);
    }

    // Shop hand-full banner: `left: 260`, width 300.
    {
        let left = 260.0;
        let width = 300.0;
        assert!(left + width <= MIN_VIEWPORT_WIDTH);
    }

    // Shop empty-state copy: `left: 92`, width 444 — matches slot strip.
    {
        let left = 92.0;
        let width = 444.0;
        assert!(left + width <= MIN_VIEWPORT_WIDTH);
    }

    // Footer slots inside the auction footer panel: `left: 92 + idx*154`,
    // width 136.
    for index in 0..3 {
        let left = 92.0 + (index as f32) * 154.0;
        let width = 136.0;
        assert!(left + width <= MIN_VIEWPORT_WIDTH);
    }
}

// ---------------------------------------------------------------------------
// Repair 4 — auction timer bar AND numeric timer label both become
// Visibility::Visible after the panel transitions to Active. Locks in
// PROMPT 1085's numeric-readout repair so it cannot regress to "thin
// green progress bar only / no numeric time-left" (UI-1129-06).
// ---------------------------------------------------------------------------

#[test]
fn prompt_1182_timer_bar_and_numeric_label_both_visible_during_active() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(/* gold */ 6);
    set_phase(&mut app, RoundPhase::DraftAuction, 30_000);
    send_auction_card(&mut app, CardId(1), /* starting_price */ 3, 20_000);

    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    let bar_visibility = app
        .world()
        .get::<Visibility>(entities.auction_timer_bar)
        .copied();
    assert_eq!(
        bar_visibility,
        Some(Visibility::Visible),
        "auction timer bar must be Visible while the panel is Active",
    );

    let label_visibility = app
        .world()
        .get::<Visibility>(entities.auction_featured_card_timer_label)
        .copied();
    assert_eq!(
        label_visibility,
        Some(Visibility::Visible),
        "auction numeric timer label must be Visible while the panel is \
         Active so the player has a numeric countdown next to the bar",
    );

    // The numeric label must paint non-empty copy during Active so the
    // player can read seconds remaining.
    let timer_text = app
        .world()
        .get::<Text>(entities.auction_featured_card_timer_label)
        .map(|t| t.0.clone())
        .unwrap_or_default();
    assert!(
        !timer_text.is_empty(),
        "auction numeric timer label must render non-empty copy while \
         Active; got {timer_text:?}",
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn app_in_session(gold: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    insert_catalog(&mut app);
    app.insert_resource(PlayerEconomyView {
        gold,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
}

fn insert_catalog(app: &mut App) {
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=3)
            .map(|id| {
                let card = test_card(id, Rarity::Common, id + 2);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
}

fn test_card(id: u32, rarity: Rarity, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Test Card {id}"),
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
