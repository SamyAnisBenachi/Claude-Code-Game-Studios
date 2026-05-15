//! Sprint 14 story 016 (`S11-UX-AUCTION-FEATURED-CARD`) — featured-card
//! visual hierarchy integration tests.
//!
//! Asserts AC1-AC4 of `production/epics/shop-auction-ui/
//! story-016-auction-featured-card.md` against stable marker components
//! published by `client::ui::shop_auction`. The tests inspect `Node`
//! intent + `TextFont` values rather than running a Bevy layout pass,
//! mirroring the headless test patterns used by Sprint 14 Tier 0 test
//! bins (`tests/integration/ui_clean_pass/*`).
//!
//! Friend-game scope preserved. `QA-COND-0005`, `QA-COND-0006`,
//! `PAW-TD-*-a`, `S8-QA-001-W1`, and the PROMPT 761 Polish→Release
//! gate-check are NOT advanced by these assertions.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::ClientState;
use client::ui::design_tokens::typography::{BODY, H1, H2};
use client::ui::shop_auction::{
    auction_featured_card_accent_color, AuctionFeaturedCard, AuctionFeaturedCardFrame,
    AuctionFeaturedCardKeyword, AuctionFeaturedCardStats, ShopAuctionCardCatalog,
    ShopAuctionUiEntities, ShopAuctionUiPlugin, AUCTION_FEATURED_CARD_FRAME_THICKNESS_PX,
    AUCTION_FEATURED_CARD_HEIGHT_PX, AUCTION_FEATURED_CARD_WIDTH_PX,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// AC1: featured card width × height are EACH strictly larger than the
/// shop slot well width × height. Story 016 §"Acceptance Criteria"
/// line 186-189; qa-plan §"Tier 1 Layout Composition" §S11-UX-AUCTION-
/// FEATURED-CARD AC1.
///
/// Sources of truth: `Node.width` / `Node.height` on the featured card
/// vs each of the three shop slot wells. Pixel-fixed sizes per spec §8
/// "Per-class scaling rules" — these values are invariant across the
/// canonical viewport matrix (1366×768 / 1920×1080 / 1920×1200 /
/// 1280×960 / 3840×2160 / 2560×1080), so a single ECS read suffices to
/// satisfy the "at both 1920 × 1080 and 1366 × 768" wording of AC1.
#[test]
fn ac1_featured_card_strictly_larger_than_every_shop_slot_well() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    let featured_node = app
        .world()
        .get::<Node>(entities.auction_featured_card)
        .expect("featured card must carry a Node");
    let featured_w = px(featured_node.width);
    let featured_h = px(featured_node.height);

    assert_eq!(
        featured_w, AUCTION_FEATURED_CARD_WIDTH_PX,
        "featured card width must equal the AUCTION_FEATURED_CARD_WIDTH_PX constant"
    );
    assert_eq!(
        featured_h, AUCTION_FEATURED_CARD_HEIGHT_PX,
        "featured card height must equal the AUCTION_FEATURED_CARD_HEIGHT_PX constant"
    );

    for (index, slot) in entities.shop_slots.into_iter().enumerate() {
        let slot_node = app
            .world()
            .get::<Node>(slot)
            .expect("shop slot well must carry a Node");
        let slot_w = px(slot_node.width);
        let slot_h = px(slot_node.height);
        assert!(
            featured_w > slot_w,
            "AC1 width: featured card ({featured_w} px) must be strictly larger than shop slot \
             well {index} ({slot_w} px)"
        );
        assert!(
            featured_h > slot_h,
            "AC1 height: featured card ({featured_h} px) must be strictly larger than shop slot \
             well {index} ({slot_h} px)"
        );
    }
}

/// AC1 (companion): the featured-card pixel footprint is reported
/// against the canonical viewport matrix at the spec's documented
/// values. Because the featured card is pixel-fixed per spec §8, the
/// ECS-level Node intent is identical across every viewport — this
/// test makes that invariance explicit by re-asserting on the constants
/// the worker authored.
#[test]
fn ac1_featured_card_size_constants_are_pixel_fixed_at_every_viewport() {
    test_helpers::init_test_tracing();
    // Constants are `f32`; assert positive + finite + above the shop
    // slot well (136 × 78 px).
    assert!(AUCTION_FEATURED_CARD_WIDTH_PX > 136.0);
    assert!(AUCTION_FEATURED_CARD_HEIGHT_PX > 78.0);
    assert!(AUCTION_FEATURED_CARD_WIDTH_PX.is_finite());
    assert!(AUCTION_FEATURED_CARD_HEIGHT_PX.is_finite());
}

/// AC2: featured card carries an explicit visual frame observable via
/// the `AuctionFeaturedCardFrame` stable marker; no shop slot well
/// carries the same marker. Story 016 §"Acceptance Criteria"
/// line 192-194.
#[test]
fn ac2_featured_card_carries_unique_frame_marker() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    let frame_entity = entities.auction_featured_card_frame;
    assert!(
        app.world()
            .get::<AuctionFeaturedCardFrame>(frame_entity)
            .is_some(),
        "frame entity must carry the AuctionFeaturedCardFrame marker"
    );

    // Frame must carry a border (the visible primitive) and its color
    // must be the ACCENT token per spec §7 (#F2C94C).
    let frame_node = app
        .world()
        .get::<Node>(frame_entity)
        .expect("frame entity must carry a Node");
    assert_eq!(
        frame_node.border.left,
        Val::Px(AUCTION_FEATURED_CARD_FRAME_THICKNESS_PX),
        "frame border thickness must equal AUCTION_FEATURED_CARD_FRAME_THICKNESS_PX",
    );
    let frame_border_color = app
        .world()
        .get::<BorderColor>(frame_entity)
        .expect("frame entity must carry a BorderColor");
    let expected_accent = auction_featured_card_accent_color();
    let actual_left = frame_border_color.left;
    assert_eq!(
        actual_left, expected_accent,
        "frame border color must equal the ACCENT token (#F2C94C) per global-ui-design-spec.md §7",
    );

    // No shop slot well carries the AuctionFeaturedCardFrame marker.
    let mut marker_q = app.world_mut().query::<&AuctionFeaturedCardFrame>();
    let frame_count = marker_q.iter(app.world()).count();
    assert_eq!(
        frame_count, 1,
        "exactly one entity may carry AuctionFeaturedCardFrame; found {frame_count}",
    );
    for slot in entities.shop_slots {
        assert!(
            app.world().get::<AuctionFeaturedCardFrame>(slot).is_none(),
            "shop slot well {slot:?} must NOT carry AuctionFeaturedCardFrame",
        );
    }
    // Footer slots also must not carry the marker.
    for footer_slot in entities.shop_footer_slots {
        assert!(
            app.world()
                .get::<AuctionFeaturedCardFrame>(footer_slot)
                .is_none(),
            "shop footer slot {footer_slot:?} must NOT carry AuctionFeaturedCardFrame",
        );
    }
}

/// AC3: featured card center anchored at the center of the auction
/// panel. Asserts the canonical bevy_ui centering trick — absolute
/// positioning with `left: 50%, top: 50%` and a negative margin equal
/// to half the card's pixel size. This places the card's geometric
/// center exactly at the panel's geometric center for any panel size
/// resolved at layout time. Story 016 §"Acceptance Criteria"
/// line 194-197.
#[test]
fn ac3_featured_card_centered_on_panel_via_percent_anchor() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    let featured_node = app
        .world()
        .get::<Node>(entities.auction_featured_card)
        .expect("featured card must carry a Node");

    assert_eq!(featured_node.position_type, PositionType::Absolute);
    assert_eq!(
        featured_node.left,
        Val::Percent(50.0),
        "featured card left must anchor at 50% of panel width",
    );
    assert_eq!(
        featured_node.top,
        Val::Percent(50.0),
        "featured card top must anchor at 50% of panel height",
    );
    assert_eq!(
        featured_node.margin.left,
        Val::Px(-AUCTION_FEATURED_CARD_WIDTH_PX / 2.0),
        "featured card margin.left must cancel half its width so the geometric center sits at 50%",
    );
    assert_eq!(
        featured_node.margin.top,
        Val::Px(-AUCTION_FEATURED_CARD_HEIGHT_PX / 2.0),
        "featured card margin.top must cancel half its height so the geometric center sits at 50%",
    );
}

/// AC4: typography hierarchy. Name (`H1 = 30 px`) > stats (`H2 = 22 px`)
/// > keyword (`BODY = 15 px`), asserted numerically via stable marker
/// queries. Story 016 §"Acceptance Criteria" line 198-200; spec §5.
#[test]
fn ac4_typography_hierarchy_name_gt_stats_gt_keyword() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    // Name → carried by the AuctionFeaturedCard entity itself
    // (`shop_auction_text_font(typography::H1)` at spawn).
    let name_font = app
        .world()
        .get::<TextFont>(entities.auction_featured_card)
        .expect("featured card must carry a TextFont (acting as the name node)");
    let stats_font = app
        .world()
        .get::<TextFont>(entities.auction_featured_card_stats)
        .expect("stats sub-node must carry a TextFont");
    let keyword_font = app
        .world()
        .get::<TextFont>(entities.auction_featured_card_keyword)
        .expect("keyword sub-node must carry a TextFont");

    assert_eq!(
        name_font.font_size, H1,
        "name font size must equal typography::H1 ({H1} px)",
    );
    assert_eq!(
        stats_font.font_size, H2,
        "stats font size must equal typography::H2 ({H2} px)",
    );
    assert_eq!(
        keyword_font.font_size, BODY,
        "keyword font size must equal typography::BODY ({BODY} px)",
    );

    assert!(
        name_font.font_size > stats_font.font_size,
        "AC4 name > stats: {} px must be > {} px",
        name_font.font_size,
        stats_font.font_size,
    );
    assert!(
        stats_font.font_size > keyword_font.font_size,
        "AC4 stats > keyword: {} px must be > {} px",
        stats_font.font_size,
        keyword_font.font_size,
    );
}

/// AC4 (companion): the markers used to query typography sizes are
/// `AuctionFeaturedCard` / `AuctionFeaturedCardStats` /
/// `AuctionFeaturedCardKeyword`. Each marker must be carried by
/// exactly one entity inside the spawned UI — defends against future
/// drift where a refactor accidentally duplicates a sub-node.
#[test]
fn ac4_typography_marker_uniqueness() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    let mut name_q = app.world_mut().query::<&AuctionFeaturedCard>();
    assert_eq!(
        name_q.iter(app.world()).count(),
        1,
        "exactly one entity may carry AuctionFeaturedCard",
    );
    let mut stats_q = app.world_mut().query::<&AuctionFeaturedCardStats>();
    assert_eq!(
        stats_q.iter(app.world()).count(),
        1,
        "exactly one entity may carry AuctionFeaturedCardStats",
    );
    let mut keyword_q = app.world_mut().query::<&AuctionFeaturedCardKeyword>();
    assert_eq!(
        keyword_q.iter(app.world()).count(),
        1,
        "exactly one entity may carry AuctionFeaturedCardKeyword",
    );
}

/// AC5 / Story 011 (companion): featured-card visual change does not
/// regress the bid target 44 × 44 CSS-px contract. Bid-button width /
/// height + focus ring width constants stay at their Story 011 values.
#[test]
fn ac5_bid_target_size_constants_unchanged_by_featured_card_story() {
    test_helpers::init_test_tracing();
    use client::ui::shop_auction::{
        AUCTION_BID_FOCUS_RING_WIDTH_PX, AUCTION_BID_TARGET_HEIGHT_PX, AUCTION_BID_TARGET_WIDTH_PX,
    };
    assert_eq!(AUCTION_BID_TARGET_WIDTH_PX, 108.0);
    assert_eq!(AUCTION_BID_TARGET_HEIGHT_PX, 44.0);
    assert_eq!(AUCTION_BID_FOCUS_RING_WIDTH_PX, 2.0);
}

/// Returns the f32 px value carried by a `Val::Px(_)`; panics otherwise
/// so a future migration to `Val::Percent` is loudly surfaced.
fn px(value: Val) -> f32 {
    match value {
        Val::Px(v) => v,
        other => panic!("expected Val::Px, got {other:?}"),
    }
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
        cards: (1..=3)
            .map(|id| {
                let card = test_card(id, Rarity::Rare, id + 2);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 10,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
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
