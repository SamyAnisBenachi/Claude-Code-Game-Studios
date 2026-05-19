//! Sprint 18 story-022 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001`) AC9
//! integration test — featured auction card spawn produces:
//!
//!   * a `CardSlotArtImage` child sized to the
//!     `CardSlotKind::AuctionFeatured` image inset, carrying an
//!     `ImageNode` with `NodeImageMode::Auto` (the Bevy 0.18
//!     justified mapping for AC2 `Fit`),
//!   * a `CardSlotLabelStrip` child sized to the
//!     `CardSlotKind::AuctionFeatured` text inset, carrying an
//!     opaque `BackgroundColor` (alpha ≥ 0.85), and
//!   * the four featured-card text children (stats / keyword /
//!     price / timer) re-parent into the strip.
//!
//! Friend-game scope preserved. `QA-COND-0005`, `QA-COND-0006`,
//! `PAW-TD-*-a`, and the PROMPT 761 Polish → Release gate-check are
//! NOT advanced by this test (story-022 §"Status / No-Claim Banner").

use std::collections::HashMap;
use std::time::Duration;

use bevy::color::Alpha;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::ui::widget::NodeImageMode;
use client::presentation::PlayerEconomyView;
use client::state::ClientState;
use client::ui::design_tokens::card_slot::{
    card_slot_geometry, CardSlotArtImage, CardSlotKind, CardSlotLabelStrip,
};
use client::ui::shop_auction::{
    AuctionFeaturedCardKeyword, AuctionFeaturedCardPriceLabel, AuctionFeaturedCardStats,
    AuctionFeaturedCardTimerLabel, ShopAuctionCardCatalog, ShopAuctionUiEntities,
    ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// AC1 / AC9: the spawned featured-card art child carries the
/// `CardSlotArtImage` marker and is published as
/// `ShopAuctionUiEntities::auction_featured_card_art`.
#[test]
fn ac9_featured_card_art_child_carries_card_slot_art_image_marker() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    let art_entity = entities.auction_featured_card_art;
    assert!(
        app.world().get::<CardSlotArtImage>(art_entity).is_some(),
        "AC9 auction_featured_card_art must carry CardSlotArtImage",
    );
    assert!(
        app.world().get::<Node>(art_entity).is_some(),
        "AC9 auction_featured_card_art must carry a Node",
    );
    assert!(
        app.world().get::<ImageNode>(art_entity).is_some(),
        "AC9 auction_featured_card_art must carry an ImageNode at spawn time",
    );
}

/// AC2 / AC9: the spawned art-child ImageNode carries
/// `NodeImageMode::Auto`. Stretch is forbidden (UI-1129-05).
#[test]
fn ac9_featured_card_art_image_node_carries_image_mode_auto() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    let image_node = app
        .world()
        .get::<ImageNode>(entities.auction_featured_card_art)
        .expect("AC9 featured card art must carry ImageNode");
    assert!(
        matches!(image_node.image_mode, NodeImageMode::Auto),
        "AC9 featured-card art ImageNode must carry NodeImageMode::Auto; got {:?}",
        image_node.image_mode,
    );
    assert!(
        !matches!(image_node.image_mode, NodeImageMode::Stretch),
        "AC9 featured-card art ImageNode must NOT carry NodeImageMode::Stretch (UI-1129-05 banner-stretch)",
    );
}

/// AC8 reach-through (AC9): the art child's Node per-side
/// absolute-position edges match the canonical
/// `CardSlotKind::AuctionFeatured` image-inset rectangle exactly.
#[test]
fn ac9_featured_card_art_node_matches_geometry_image_inset() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();
    let geometry = card_slot_geometry(CardSlotKind::AuctionFeatured);

    let node = app
        .world()
        .get::<Node>(entities.auction_featured_card_art)
        .expect("AC9 featured card art must carry Node");
    assert_eq!(
        node.position_type,
        PositionType::Absolute,
        "AC9 art child must be PositionType::Absolute",
    );
    assert_eq!(node.left, geometry.image_inset_px.left);
    assert_eq!(node.right, geometry.image_inset_px.right);
    assert_eq!(node.top, geometry.image_inset_px.top);
    assert_eq!(node.bottom, geometry.image_inset_px.bottom);
}

/// AC3 / AC9: the spawned label strip carries the
/// `CardSlotLabelStrip` marker, an opaque `BackgroundColor` with
/// alpha ≥ 0.85, and is published as
/// `ShopAuctionUiEntities::auction_featured_card_label_strip`.
#[test]
fn ac9_featured_card_label_strip_carries_marker_and_opaque_background() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    let strip_entity = entities.auction_featured_card_label_strip;
    assert!(
        app.world().get::<CardSlotLabelStrip>(strip_entity).is_some(),
        "AC9 auction_featured_card_label_strip must carry CardSlotLabelStrip",
    );
    let strip_bg = app
        .world()
        .get::<BackgroundColor>(strip_entity)
        .expect("AC9 label strip must carry BackgroundColor");
    let alpha = strip_bg.0.alpha();
    assert!(
        alpha >= 0.85,
        "AC9 label-strip BackgroundColor alpha must be >= 0.85 (opaque); got {alpha}",
    );
}

/// AC3 / AC9: the label-strip Node is sized to the canonical
/// `CardSlotKind::AuctionFeatured` text inset rectangle and carries a
/// `min_width` clamp + `overflow.clip_x()` policy. The
/// `card_slot_label_strip_node` builder is the single source of
/// these structural guarantees.
#[test]
fn ac9_featured_card_label_strip_node_matches_geometry_and_clips_overflow_x() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();
    let geometry = card_slot_geometry(CardSlotKind::AuctionFeatured);

    let node = app
        .world()
        .get::<Node>(entities.auction_featured_card_label_strip)
        .expect("AC9 label strip must carry Node");

    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.left, geometry.text_inset_px.left);
    assert_eq!(node.right, geometry.text_inset_px.right);
    assert_eq!(node.top, geometry.text_inset_px.top);
    assert_eq!(node.bottom, geometry.text_inset_px.bottom);
    assert!(
        matches!(node.min_width, Val::Px(v) if v > 0.0),
        "AC9 label strip min_width must be a positive Val::Px; got {:?}",
        node.min_width,
    );
    assert_eq!(
        node.overflow.x,
        bevy::ui::OverflowAxis::Clip,
        "AC9 label strip must clip horizontal overflow",
    );
}

/// AC9: each of the four featured-card text children
/// (stats / keyword / price / timer) carries `ChildOf(label_strip)`
/// — the strip is the canonical parent of every per-card readout.
#[test]
fn ac9_four_featured_card_text_children_are_parented_into_label_strip() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();
    let strip = entities.auction_featured_card_label_strip;

    let text_children = [
        ("stats", entities.auction_featured_card_stats),
        ("keyword", entities.auction_featured_card_keyword),
        ("price", entities.auction_featured_card_price_label),
        ("timer", entities.auction_featured_card_timer_label),
    ];
    for (label, child) in text_children {
        let child_of = app
            .world()
            .get::<ChildOf>(child)
            .unwrap_or_else(|| panic!("AC9 featured-card {label} child must carry ChildOf"));
        assert_eq!(
            child_of.parent(),
            strip,
            "AC9 featured-card {label} child must be parented into label strip; \
             got parent {:?}, expected {:?}",
            child_of.parent(),
            strip,
        );
    }
}

/// AC9 — the strip hosts exactly the four canonical text markers
/// the story names. A future revision that introduces a fifth text
/// readout under the strip would either need a new AC, or live
/// outside the strip; this assertion makes the boundary explicit.
#[test]
fn ac9_label_strip_hosts_each_canonical_text_marker_exactly_once() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();

    let mut stats_q = app.world_mut().query::<&AuctionFeaturedCardStats>();
    assert_eq!(
        stats_q.iter(app.world()).count(),
        1,
        "AC9 exactly one entity must carry AuctionFeaturedCardStats",
    );
    let mut keyword_q = app.world_mut().query::<&AuctionFeaturedCardKeyword>();
    assert_eq!(
        keyword_q.iter(app.world()).count(),
        1,
        "AC9 exactly one entity must carry AuctionFeaturedCardKeyword",
    );
    let mut price_q = app.world_mut().query::<&AuctionFeaturedCardPriceLabel>();
    assert_eq!(
        price_q.iter(app.world()).count(),
        1,
        "AC9 exactly one entity must carry AuctionFeaturedCardPriceLabel",
    );
    let mut timer_q = app.world_mut().query::<&AuctionFeaturedCardTimerLabel>();
    assert_eq!(
        timer_q.iter(app.world()).count(),
        1,
        "AC9 exactly one entity must carry AuctionFeaturedCardTimerLabel",
    );
}

/// AC9 — the art child and label strip are distinct entities, both
/// parented into the featured card root. Catches a regression where
/// the two markers collapse onto the same entity.
#[test]
fn ac9_art_child_and_label_strip_are_distinct_children_of_featured_card() {
    test_helpers::init_test_tracing();
    let app = app_in_session();
    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();

    assert_ne!(
        entities.auction_featured_card_art, entities.auction_featured_card_label_strip,
        "AC9 art child and label strip must be distinct entities",
    );
    let art_parent = app
        .world()
        .get::<ChildOf>(entities.auction_featured_card_art)
        .expect("AC9 art child must carry ChildOf");
    let strip_parent = app
        .world()
        .get::<ChildOf>(entities.auction_featured_card_label_strip)
        .expect("AC9 label strip must carry ChildOf");
    assert_eq!(
        art_parent.parent(),
        entities.auction_featured_card,
        "AC9 art child must be parented into the featured-card root",
    );
    assert_eq!(
        strip_parent.parent(),
        entities.auction_featured_card,
        "AC9 label strip must be parented into the featured-card root",
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
