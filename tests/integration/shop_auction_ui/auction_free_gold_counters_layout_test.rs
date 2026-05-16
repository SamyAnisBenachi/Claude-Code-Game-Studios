//! Sprint 14 story 017 (`S11-UX-AUCTION-FREE-GOLD-COUNTERS`) integration tests.
//!
//! These assertions inspect Bevy UI intent and stable marker components in a
//! headless app. They do not add protocol drains or mutate authoritative
//! economy state; the counter values are read through the existing
//! `ShopAuctionLocalGoldView` / `PlayerEconomyView` path.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::design_tokens::{spacing, typography};
use client::ui::hud::HudGoldBroadcastMessage;
use client::ui::shop_auction::{
    AuctionBidButtonState, AuctionFreeGoldCounter, AuctionFreeGoldCounterGroup,
    AuctionFreeGoldCounterLabel, AuctionFreeGoldCounterValue, ShopAuctionAuctionCardReceived,
    ShopAuctionCardCatalog, ShopAuctionLocalGoldView, ShopAuctionUiEntities, ShopAuctionUiPlugin,
    AUCTION_BID_TARGET_HEIGHT_PX, AUCTION_BID_TARGET_WIDTH_PX, AUCTION_FEATURED_CARD_HEIGHT_PX,
    AUCTION_FEATURED_CARD_WIDTH_PX, AUCTION_FREE_GOLD_COUNTER_ANCHOR_LEFT_PERCENT,
    AUCTION_FREE_GOLD_COUNTER_BOTTOM_PX, AUCTION_FREE_GOLD_COUNTER_COUNT,
    AUCTION_FREE_GOLD_COUNTER_GROUP_HEIGHT_PX, AUCTION_FREE_GOLD_COUNTER_GROUP_WIDTH_PX,
    AUCTION_FREE_GOLD_COUNTER_KINDS, AUCTION_FREE_GOLD_COUNTER_LABEL_FONT_PX,
    AUCTION_FREE_GOLD_COUNTER_LEFT_GAP_PX, AUCTION_FREE_GOLD_COUNTER_LEFT_OFFSET_PX,
    AUCTION_FREE_GOLD_COUNTER_PADDING_PX, AUCTION_FREE_GOLD_COUNTER_VALUE_FONT_PX,
    AUCTION_FREE_GOLD_COUNTER_WIDTH_PX,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{RoundPhase, S2CGoldBroadcast};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const VIEWPORT_1366: (f32, f32) = (1366.0, 768.0);
const VIEWPORT_1920: (f32, f32) = (1920.0, 1080.0);
const ADJACENCY_TOLERANCE_PX: f32 = 0.01;

#[test]
fn ac1_counter_group_is_single_shared_container_with_two_sibling_counters() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(0, 20_000, 10, 0);
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    assert_eq!(count_with::<AuctionFreeGoldCounterGroup>(&mut app), 1);
    assert_eq!(count_with::<AuctionFreeGoldCounter>(&mut app), 2);
    assert_eq!(count_with::<AuctionFreeGoldCounterLabel>(&mut app), 2);
    assert_eq!(count_with::<AuctionFreeGoldCounterValue>(&mut app), 2);

    let group_node = app
        .world()
        .get::<Node>(entities.auction_free_gold_counter_group)
        .expect("free-gold group must carry a Node");
    assert_eq!(group_node.display, Display::Flex);
    assert_eq!(group_node.flex_direction, FlexDirection::Row);
    assert_eq!(
        group_node.column_gap,
        Val::Px(AUCTION_FREE_GOLD_COUNTER_LEFT_GAP_PX)
    );
    assert_eq!(
        group_node.bottom,
        Val::Px(AUCTION_FREE_GOLD_COUNTER_BOTTOM_PX)
    );
    assert_eq!(
        group_node.padding.left,
        Val::Px(AUCTION_FREE_GOLD_COUNTER_PADDING_PX),
        "group padding must use the story-017 spacing token composition"
    );

    let mut observed_kinds = Vec::new();
    for index in 0..AUCTION_FREE_GOLD_COUNTER_COUNT {
        let counter = entities.auction_free_gold_counters[index];
        let label = entities.auction_free_gold_counter_labels[index];
        let value = entities.auction_free_gold_counter_values[index];

        assert_eq!(
            parent_of(&app, counter),
            entities.auction_free_gold_counter_group,
            "counter {index} must be a direct child of the shared group"
        );
        assert_eq!(parent_of(&app, label), counter);
        assert_eq!(parent_of(&app, value), counter);

        let counter_node = app
            .world()
            .get::<Node>(counter)
            .expect("counter must carry Node");
        assert_eq!(
            counter_node.width,
            Val::Px(AUCTION_FREE_GOLD_COUNTER_WIDTH_PX)
        );

        let counter_marker = app
            .world()
            .get::<AuctionFreeGoldCounter>(counter)
            .expect("counter must carry marker");
        let label_marker = app
            .world()
            .get::<AuctionFreeGoldCounterLabel>(label)
            .expect("label must carry marker");
        let value_marker = app
            .world()
            .get::<AuctionFreeGoldCounterValue>(value)
            .expect("value must carry marker");
        assert_eq!(counter_marker.kind, label_marker.kind);
        assert_eq!(counter_marker.kind, value_marker.kind);
        observed_kinds.push(counter_marker.kind);
    }
    assert_eq!(observed_kinds, AUCTION_FREE_GOLD_COUNTER_KINDS);
}

#[test]
fn ac2_counter_group_is_adjacent_to_bid_cluster_with_documented_gap() {
    test_helpers::init_test_tracing();
    let app = app_in_active_auction(0, 20_000, 10, 0);

    for viewport in [VIEWPORT_1366, VIEWPORT_1920] {
        let layout = auction_layout_rects(&app, viewport);
        let expected_gap = spacing::SPACING_MD;
        let actual_gap = layout.free_gold_group.left - layout.bid_cluster.right;
        assert_close(
            actual_gap,
            expected_gap,
            ADJACENCY_TOLERANCE_PX,
            "free-gold group must sit one SPACING_MD token to the right of the bid cluster",
        );
        assert_close(
            layout.free_gold_group.center_y(),
            layout.bid_cluster.center_y(),
            ADJACENCY_TOLERANCE_PX,
            "free-gold group and bid cluster should share the same vertical decision row",
        );

        let expected_left = viewport.0 * (AUCTION_FREE_GOLD_COUNTER_ANCHOR_LEFT_PERCENT / 100.0)
            + AUCTION_FREE_GOLD_COUNTER_LEFT_OFFSET_PX;
        assert_close(
            layout.free_gold_group.left,
            expected_left,
            ADJACENCY_TOLERANCE_PX,
            "panel-relative group x-offset drifted from the story-017 anchor",
        );
    }
}

#[test]
fn ac3_counter_value_typography_is_larger_than_labels() {
    test_helpers::init_test_tracing();
    let app = app_in_active_auction(0, 20_000, 10, 0);
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    for index in 0..AUCTION_FREE_GOLD_COUNTER_COUNT {
        let label_font = app
            .world()
            .get::<TextFont>(entities.auction_free_gold_counter_labels[index])
            .expect("label must carry TextFont");
        let value_font = app
            .world()
            .get::<TextFont>(entities.auction_free_gold_counter_values[index])
            .expect("value must carry TextFont");

        assert_eq!(
            label_font.font_size,
            AUCTION_FREE_GOLD_COUNTER_LABEL_FONT_PX
        );
        assert_eq!(
            value_font.font_size,
            AUCTION_FREE_GOLD_COUNTER_VALUE_FONT_PX
        );
        assert_eq!(label_font.font_size, typography::CAPTION);
        assert_eq!(value_font.font_size, typography::H2);
        assert!(
            value_font.font_size > label_font.font_size,
            "counter numeric value font must be larger than label font"
        );
    }
}

#[test]
fn ac4_counter_group_fits_canonical_viewports_without_overlap() {
    test_helpers::init_test_tracing();
    let app = app_in_active_auction(0, 20_000, 10, 0);

    for viewport in [VIEWPORT_1366, VIEWPORT_1920] {
        let layout = auction_layout_rects(&app, viewport);

        assert!(layout.free_gold_group.left >= 0.0);
        assert!(layout.free_gold_group.top >= 0.0);
        assert!(layout.free_gold_group.right <= viewport.0);
        assert!(layout.free_gold_group.bottom <= layout.panel_height);

        assert_no_overlap(layout.free_gold_group, layout.bid_cluster, "bid cluster");
        assert_no_overlap(
            layout.free_gold_group,
            layout.featured_card,
            "featured card",
        );
        assert_no_overlap(layout.free_gold_group, layout.timer_bar, "timer bar");
        assert_no_overlap(layout.free_gold_group, layout.bid_status, "bid status");
        assert_no_overlap(
            layout.free_gold_group,
            layout.settlement_text,
            "settlement text",
        );
    }
}

#[test]
fn ac5_counter_values_follow_existing_local_free_gold_path_every_frame() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(0, 20_000, 8, 5);

    assert_counter_values(&app, 3);
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Unaffordable
        ],
        "Story 005 affordability must continue reading local_free_gold"
    );

    write_local_gold_broadcast(&mut app, 11, 0);
    assert_counter_values(&app, 11);
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Enabled
        ]
    );

    write_local_gold_broadcast(&mut app, 2, 6);
    assert_counter_values(&app, 0);
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Unaffordable,
            AuctionBidButtonState::Unaffordable,
            AuctionBidButtonState::Unaffordable
        ],
        "server-invariant violations must continue to saturate free gold at zero"
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
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
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
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    set_phase(&mut app, RoundPhase::DraftAuction, timer_duration_ms);
    send_auction_card(&mut app, CardId(1), starting_price, timer_duration_ms);
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

fn write_local_gold_broadcast(app: &mut App, gold: u32, reserved_gold: u32) {
    app.world_mut()
        .write_message(HudGoldBroadcastMessage(S2CGoldBroadcast {
            player_id: LOCAL_PLAYER,
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

fn parent_of(app: &App, entity: Entity) -> Entity {
    app.world()
        .get::<ChildOf>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have a ChildOf parent"))
        .parent()
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn assert_counter_values(app: &App, expected: u32) {
    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    for entity in entities.auction_free_gold_counter_values {
        let value = app
            .world()
            .get::<AuctionFreeGoldCounterValue>(entity)
            .expect("counter value must carry marker");
        let text = app
            .world()
            .get::<Text>(entity)
            .expect("counter value must carry Text");
        assert_eq!(value.amount, expected);
        assert_eq!(text.0, format!("{expected}g"));
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

#[derive(Debug, Clone, Copy)]
struct Rect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Rect {
    fn center_y(self) -> f32 {
        (self.top + self.bottom) / 2.0
    }

    fn intersects(self, other: Self) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top < other.bottom
            && self.bottom > other.top
    }
}

#[derive(Debug, Clone, Copy)]
struct AuctionLayoutRects {
    panel_height: f32,
    free_gold_group: Rect,
    bid_cluster: Rect,
    featured_card: Rect,
    timer_bar: Rect,
    bid_status: Rect,
    settlement_text: Rect,
}

fn auction_layout_rects(app: &App, viewport: (f32, f32)) -> AuctionLayoutRects {
    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    let panel = app
        .world()
        .get::<Node>(entities.auction_panel)
        .expect("auction panel must carry Node");
    let panel_height = viewport.1 - px(panel.top) - px(panel.bottom);

    let free_gold_group = rect_from_node(
        app.world()
            .get::<Node>(entities.auction_free_gold_counter_group)
            .expect("free-gold group must carry Node"),
        viewport.0,
        panel_height,
    );
    assert_close(
        free_gold_group.right - free_gold_group.left,
        AUCTION_FREE_GOLD_COUNTER_GROUP_WIDTH_PX,
        ADJACENCY_TOLERANCE_PX,
        "free-gold group width",
    );
    assert_close(
        free_gold_group.bottom - free_gold_group.top,
        AUCTION_FREE_GOLD_COUNTER_GROUP_HEIGHT_PX,
        ADJACENCY_TOLERANCE_PX,
        "free-gold group height",
    );

    let bid_rects = entities.auction_bid_buttons.map(|button| {
        rect_from_node(
            app.world().get::<Node>(button).unwrap(),
            viewport.0,
            panel_height,
        )
    });
    let bid_cluster = union_rects(&bid_rects);
    assert_close(
        bid_rects[2].right - bid_rects[2].left,
        AUCTION_BID_TARGET_WIDTH_PX,
        ADJACENCY_TOLERANCE_PX,
        "bid target width",
    );
    assert_close(
        bid_rects[2].bottom - bid_rects[2].top,
        AUCTION_BID_TARGET_HEIGHT_PX,
        ADJACENCY_TOLERANCE_PX,
        "bid target height",
    );

    let featured_card = rect_from_node(
        app.world()
            .get::<Node>(entities.auction_featured_card)
            .expect("featured card must carry Node"),
        viewport.0,
        panel_height,
    );
    assert_close(
        featured_card.right - featured_card.left,
        AUCTION_FEATURED_CARD_WIDTH_PX,
        ADJACENCY_TOLERANCE_PX,
        "featured card width",
    );
    assert_close(
        featured_card.bottom - featured_card.top,
        AUCTION_FEATURED_CARD_HEIGHT_PX,
        ADJACENCY_TOLERANCE_PX,
        "featured card height",
    );

    AuctionLayoutRects {
        panel_height,
        free_gold_group,
        bid_cluster,
        featured_card,
        timer_bar: rect_from_node(
            app.world()
                .get::<Node>(entities.auction_timer_bar)
                .expect("timer must carry Node"),
            viewport.0,
            panel_height,
        ),
        bid_status: rect_from_node(
            app.world()
                .get::<Node>(entities.auction_bid_status_text)
                .expect("bid status must carry Node"),
            viewport.0,
            panel_height,
        ),
        settlement_text: rect_from_node(
            app.world()
                .get::<Node>(entities.settlement_overlay_text)
                .expect("settlement text must carry Node"),
            viewport.0,
            panel_height,
        ),
    }
}

fn rect_from_node(node: &Node, parent_width: f32, parent_height: f32) -> Rect {
    let width = size_px(node.width, parent_width);
    let height = size_px(node.height, parent_height);
    let left = position_px(node.left, parent_width) + margin_px(node.margin.left);
    let top = match node.top {
        Val::Px(_) | Val::Percent(_) => {
            position_px(node.top, parent_height) + margin_px(node.margin.top)
        }
        _ => {
            parent_height
                - position_px(node.bottom, parent_height)
                - height
                - margin_px(node.margin.bottom)
        }
    };

    Rect {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

fn union_rects(rects: &[Rect; 3]) -> Rect {
    Rect {
        left: rects
            .iter()
            .map(|rect| rect.left)
            .fold(f32::INFINITY, f32::min),
        top: rects
            .iter()
            .map(|rect| rect.top)
            .fold(f32::INFINITY, f32::min),
        right: rects
            .iter()
            .map(|rect| rect.right)
            .fold(f32::NEG_INFINITY, f32::max),
        bottom: rects
            .iter()
            .map(|rect| rect.bottom)
            .fold(f32::NEG_INFINITY, f32::max),
    }
}

fn size_px(value: Val, parent: f32) -> f32 {
    match value {
        Val::Px(v) => v,
        Val::Percent(p) => parent * p / 100.0,
        other => panic!("expected size Val::Px/Percent, got {other:?}"),
    }
}

fn position_px(value: Val, parent: f32) -> f32 {
    match value {
        Val::Px(v) => v,
        Val::Percent(p) => parent * p / 100.0,
        Val::Auto => 0.0,
        other => panic!("unexpected position value {other:?}"),
    }
}

fn margin_px(value: Val) -> f32 {
    match value {
        Val::Px(v) => v,
        Val::Auto => 0.0,
        other => panic!("expected margin Val::Px/Auto, got {other:?}"),
    }
}

fn px(value: Val) -> f32 {
    match value {
        Val::Px(v) => v,
        other => panic!("expected Val::Px, got {other:?}"),
    }
}

fn assert_no_overlap(left: Rect, right: Rect, label: &str) {
    assert!(
        !left.intersects(right),
        "free-gold counter group must not overlap {label}: left={left:?}, right={right:?}"
    );
}

fn assert_close(actual: f32, expected: f32, tolerance: f32, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tolerance,
        "{label}: expected {expected}, got {actual} (diff {diff}, tolerance {tolerance})"
    );
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
