//! Sprint 18 story 020 — `S18-UI-PLAY-AREA-CONTAINER-001` integration
//! bin. AC7 binding test for the `PlayArea` flex container + strip-budget
//! contract introduced by PROMPT 1180 §6 Lane A.
//!
//! Covers:
//!
//! - **AC1**: `client::ui::design_tokens::play_area::play_area_node()`
//!   returns the canonical `Node { position_type: Absolute,
//!   top: HEADER_BAR_HEIGHT_PX, left: 0, right: 0,
//!   bottom: HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX, display: Flex,
//!   flex_direction: Column }`.
//! - **AC2..AC6**: each of the five migrated consumer node builders
//!   (`bottom_panel_node`, `auction_panel_node`, `footer_node`,
//!   `toast_node`, `placement_action_panel_node`) drops every
//!   viewport-anchored literal (60 / 80 / 100 / 140 / 220) introduced
//!   pre-Sprint 18 and now expresses its anchor relative to `PlayArea`.
//!   AC6 additionally requires `placement_action_panel_node` to declare
//!   `max_height` + `Overflow::scroll_y()`.
//! - **AC7**: the budget edges hold at the canonical 1280×720 /
//!   1366×768 / 1920×1080 viewport matrix — `PlayArea` always sits
//!   strictly inside the viewport interior, and each consumer's
//!   geometric rectangle is fully contained inside the `PlayArea`
//!   rectangle at every viewport in the matrix.
//! - **AC8**: the four canonical strip primitives (`HeaderBar`,
//!   `LaneBar`, `HandBar`, `FooterBar`) keep their viewport-edge anchors
//!   — i.e. their existing `top: 0` / `bottom: 0` / `bottom:
//!   HAND_BAR_HEIGHT_PX` anchors are unchanged, and they remain
//!   siblings of `PlayArea` rather than children of it.
//!
//! No client-side optimistic authority is introduced. The test is a
//! read-only geometric assertion over the published `Node` shapes plus
//! a source-string check that the migrated consumer spawn sites parent
//! into the `PlayAreaRoot` resource.
//!
//! Test naming follows the `test_[system]_[scenario]_[expected_result]`
//! convention documented in `.claude/rules/test-standards.md`.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::ui::Overflow;
use client::ui::design_tokens::play_area::{
    play_area_node, PlayArea, PlayAreaRoot, PlayAreaSpawnSet, PLAY_AREA_BOTTOM_RESERVE_PX,
};
use client::ui::design_tokens::strips::{
    footer_bar_node, hand_bar_node, header_bar_node, FOOTER_BAR_HEIGHT_PX, HAND_BAR_HEIGHT_PX,
    HEADER_BAR_HEIGHT_PX,
};
use client::ui::hand::placement_action_panel_node;
use client::ui::shop_auction::{auction_panel_node, bottom_panel_node, footer_node, toast_node};

// ─── Test fixtures ──────────────────────────────────────────────────────

/// Canonical viewport matrix for the strip-budget contract. AC7 names
/// 1280×720 / 1366×768 / 1920×1080 verbatim.
const CANONICAL_VIEWPORT_MATRIX: &[(&str, f32, f32)] = &[
    ("1280x720", 1280.0, 720.0),
    ("1366x768", 1366.0, 768.0),
    ("1920x1080", 1920.0, 1080.0),
];

/// Geometric rectangle in viewport-space pixels (origin top-left).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    fn from_viewport(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    fn right(&self) -> f32 {
        self.x + self.width
    }

    fn bottom(&self) -> f32 {
        self.y + self.height
    }

    fn contains_rect(&self, inner: Rect) -> bool {
        inner.x >= self.x
            && inner.y >= self.y
            && inner.right() <= self.right() + f32::EPSILON
            && inner.bottom() <= self.bottom() + f32::EPSILON
    }

    fn intersects(&self, other: Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// Resolve a `Val` into an absolute pixel offset within a parent
/// dimension. Only the variants used by the migrated consumer node
/// builders are handled (Px / Percent / Auto). Test-side helper — not
/// a layout solver substitute.
fn resolve_px(val: Val, parent_extent_px: f32) -> Option<f32> {
    match val {
        Val::Px(px) => Some(px),
        Val::Percent(pct) => Some(parent_extent_px * pct / 100.0),
        Val::Auto => None,
        _ => None,
    }
}

/// Compute the absolute viewport rectangle for a `Node` that is
/// `PositionType::Absolute` inside a parent rectangle. Honors the four
/// edge anchors and either `width` / `height` or "stretch" (both
/// left + right OR top + bottom resolved).
fn compute_absolute_rect(node: &Node, parent: Rect) -> Rect {
    let left = resolve_px(node.left, parent.width);
    let right = resolve_px(node.right, parent.width);
    let top = resolve_px(node.top, parent.height);
    let bottom = resolve_px(node.bottom, parent.height);
    let width = resolve_px(node.width, parent.width);
    let height = resolve_px(node.height, parent.height);

    let (x, w) = match (left, right, width) {
        (Some(l), Some(r), _) => (parent.x + l, parent.width - l - r),
        (Some(l), None, Some(w)) => (parent.x + l, w),
        (None, Some(r), Some(w)) => (parent.right() - r - w, w),
        (Some(l), None, None) => (parent.x + l, parent.width - l),
        (None, Some(r), None) => (parent.x, parent.width - r),
        (None, None, Some(w)) => (parent.x, w),
        (None, None, None) => (parent.x, parent.width),
    };
    let (y, h) = match (top, bottom, height) {
        (Some(t), Some(b), _) => (parent.y + t, parent.height - t - b),
        (Some(t), None, Some(h)) => (parent.y + t, h),
        (None, Some(b), Some(h)) => (parent.bottom() - b - h, h),
        (Some(t), None, None) => (parent.y + t, parent.height - t),
        (None, Some(b), None) => (parent.y, parent.height - b),
        (None, None, Some(h)) => (parent.y, h),
        (None, None, None) => (parent.y, parent.height),
    };

    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_client_source(rel: &str) -> String {
    let path = client_src_root().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

// ─── AC1: PlayArea module exists with documented Node shape ────────────

#[test]
fn test_play_area_ac1_node_builder_returns_documented_shape() {
    // Arrange
    let node = play_area_node();

    // Assert — Node shape matches the Control Manifest verbatim.
    assert_eq!(
        node.position_type,
        PositionType::Absolute,
        "AC1: PlayArea must be PositionType::Absolute"
    );
    assert_eq!(
        node.top,
        Val::Px(HEADER_BAR_HEIGHT_PX),
        "AC1: PlayArea must anchor `top` at HEADER_BAR_HEIGHT_PX"
    );
    assert_eq!(
        node.left,
        Val::Px(0.0),
        "AC1: PlayArea must anchor `left: 0`"
    );
    assert_eq!(
        node.right,
        Val::Px(0.0),
        "AC1: PlayArea must anchor `right: 0`"
    );
    assert_eq!(
        node.bottom,
        Val::Px(HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX),
        "AC1: PlayArea must anchor `bottom: HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX`"
    );
    assert_eq!(
        node.display,
        Display::Flex,
        "AC1: PlayArea must declare Display::Flex"
    );
    assert_eq!(
        node.flex_direction,
        FlexDirection::Column,
        "AC1: PlayArea must declare FlexDirection::Column"
    );
}

#[test]
fn test_play_area_ac1_bottom_reserve_matches_strip_constants() {
    // Arrange / Act / Assert — the published `PLAY_AREA_BOTTOM_RESERVE_PX`
    // constant collapses to the sum of the two bottom strips so callers
    // (Lane J / story 026) consume it without re-deriving the sum.
    assert_eq!(
        PLAY_AREA_BOTTOM_RESERVE_PX,
        HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX,
        "AC1: PLAY_AREA_BOTTOM_RESERVE_PX must equal HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX"
    );
}

#[test]
fn test_play_area_ac1_marker_and_resource_are_exported() {
    // Type-level smoke check — the marker, resource, and system-set
    // names are all addressable from the integration crate. Compilation
    // success is the contract; the function bodies are no-ops that
    // exercise the public surface so a removed export trips a test
    // build failure rather than a silent regression.
    fn _check_marker(_: PlayArea) {}
    fn _check_root(_: PlayAreaRoot) {}
    fn _check_set(_: PlayAreaSpawnSet) {}
}

// ─── AC7: PlayArea fits the viewport at the canonical matrix ────────────

#[test]
fn test_play_area_ac7_fits_inside_every_canonical_viewport() {
    for (label, vw, vh) in CANONICAL_VIEWPORT_MATRIX {
        // Arrange
        let viewport = Rect::from_viewport(*vw, *vh);
        let pa_node = play_area_node();

        // Act
        let pa_rect = compute_absolute_rect(&pa_node, viewport);

        // Assert — PlayArea sits strictly inside the viewport with the
        // canonical header / bottom-strip reserve.
        assert!(
            viewport.contains_rect(pa_rect),
            "AC7: PlayArea must fit inside viewport {label} (vw={vw}, vh={vh}); \
             pa_rect={pa_rect:?}, viewport={viewport:?}"
        );
        assert_eq!(
            pa_rect.x, 0.0,
            "AC7 @ {label}: PlayArea must start at x=0 (left:0)"
        );
        assert_eq!(
            pa_rect.y, HEADER_BAR_HEIGHT_PX,
            "AC7 @ {label}: PlayArea must start at y=HEADER_BAR_HEIGHT_PX"
        );
        assert_eq!(
            pa_rect.width, *vw,
            "AC7 @ {label}: PlayArea must span the full viewport width"
        );
        assert_eq!(
            pa_rect.height,
            vh - HEADER_BAR_HEIGHT_PX - HAND_BAR_HEIGHT_PX - FOOTER_BAR_HEIGHT_PX,
            "AC7 @ {label}: PlayArea height must equal the residual middle band"
        );
    }
}

// ─── AC2..AC6 + AC7: consumer migrations fit inside PlayArea ────────────

#[test]
fn test_play_area_ac2_shop_panel_fits_inside_play_area_at_every_viewport() {
    for (label, vw, vh) in CANONICAL_VIEWPORT_MATRIX {
        // Arrange
        let viewport = Rect::from_viewport(*vw, *vh);
        let pa = compute_absolute_rect(&play_area_node(), viewport);

        // Act
        let shop = compute_absolute_rect(&bottom_panel_node(), pa);

        // Assert — shop panel sits fully inside PlayArea (AC2 + AC7).
        assert!(
            pa.contains_rect(shop),
            "AC2/AC7 @ {label}: shop panel must fit inside PlayArea; shop={shop:?}, pa={pa:?}"
        );
        // Migrated `bottom_panel_node` removed the literal `height: 260`
        // viewport-anchored offset and now fills PlayArea
        // (top:0/left:0/right:0/bottom:0).
        assert_eq!(
            shop, pa,
            "AC2 @ {label}: migrated shop panel must fill PlayArea"
        );
    }
}

#[test]
fn test_play_area_ac3_auction_panel_fits_inside_play_area_at_every_viewport() {
    for (label, vw, vh) in CANONICAL_VIEWPORT_MATRIX {
        // Arrange
        let viewport = Rect::from_viewport(*vw, *vh);
        let pa = compute_absolute_rect(&play_area_node(), viewport);

        // Act
        let auction = compute_absolute_rect(&auction_panel_node(), pa);

        // Assert — auction panel sits fully inside PlayArea (AC3 + AC7).
        assert!(
            pa.contains_rect(auction),
            "AC3/AC7 @ {label}: auction panel must fit inside PlayArea; \
             auction={auction:?}, pa={pa:?}"
        );
        // Migrated `auction_panel_node` removed the `top: 80, bottom: 140`
        // viewport-anchored literals; the new shape fills PlayArea.
        assert_eq!(
            auction, pa,
            "AC3 @ {label}: migrated auction panel must fill PlayArea"
        );
    }
}

#[test]
fn test_play_area_ac4_shop_footer_fits_inside_play_area_at_every_viewport() {
    for (label, vw, vh) in CANONICAL_VIEWPORT_MATRIX {
        // Arrange
        let viewport = Rect::from_viewport(*vw, *vh);
        let pa = compute_absolute_rect(&play_area_node(), viewport);

        // Act
        let footer = compute_absolute_rect(&footer_node(), pa);

        // Assert — shop footer sits fully inside PlayArea (AC4 + AC7).
        assert!(
            pa.contains_rect(footer),
            "AC4/AC7 @ {label}: shop footer must fit inside PlayArea; \
             footer={footer:?}, pa={pa:?}"
        );
        // Migrated `footer_node` removed the `bottom: 100` viewport-anchored
        // offset; the 96 px band now sits flush at PlayArea's bottom edge.
        assert_eq!(
            footer.bottom(),
            pa.bottom(),
            "AC4 @ {label}: migrated shop footer must anchor at PlayArea bottom"
        );
        assert_eq!(
            footer.height, 96.0,
            "AC4 @ {label}: shop footer height must stay at 96 px"
        );
    }
}

#[test]
fn test_play_area_ac5_toast_fits_inside_play_area_at_every_viewport() {
    for (label, vw, vh) in CANONICAL_VIEWPORT_MATRIX {
        // Arrange
        let viewport = Rect::from_viewport(*vw, *vh);
        let pa = compute_absolute_rect(&play_area_node(), viewport);

        // Act
        let toast = compute_absolute_rect(&toast_node(), pa);

        // Assert — toast sits fully inside PlayArea (AC5 + AC7).
        assert!(
            pa.contains_rect(toast),
            "AC5/AC7 @ {label}: toast must fit inside PlayArea; toast={toast:?}, pa={pa:?}"
        );
        // AC5 explicitly allows the toast to remain absolute within
        // PlayArea; the bottom: 220 viewport literal is replaced by an
        // inset relative to PlayArea's bottom edge.
        assert!(
            toast.bottom() <= pa.bottom() + f32::EPSILON,
            "AC5 @ {label}: toast must not overflow PlayArea bottom"
        );
        assert_eq!(
            toast.width, 260.0,
            "AC5 @ {label}: toast width must stay at 260 px"
        );
    }
}

#[test]
fn test_play_area_ac6_placement_action_panel_fits_inside_play_area_with_scroll_y() {
    // Assert — placement action panel declares the AC6-required
    // `max_height` + `Overflow::scroll_y()` policy. This guards against
    // a future regression that re-introduces an unbounded height which
    // would overflow PlayArea at small viewports.
    let panel = placement_action_panel_node();
    assert!(
        matches!(panel.max_height, Val::Percent(p) if p == 100.0),
        "AC6: placement action panel must declare `max_height: 100%`; got {:?}",
        panel.max_height
    );
    assert_eq!(
        panel.overflow,
        Overflow::scroll_y(),
        "AC6: placement action panel must declare `Overflow::scroll_y()`"
    );

    for (label, vw, vh) in CANONICAL_VIEWPORT_MATRIX {
        // Arrange
        let viewport = Rect::from_viewport(*vw, *vh);
        let pa = compute_absolute_rect(&play_area_node(), viewport);

        // Act — solve the panel rect inside PlayArea using its
        // `right` / `bottom` / `width` anchors. Height is bounded by
        // `max_height: 100%` of PlayArea.
        let panel_rect = compute_absolute_rect(&panel, pa);

        // Assert — fits horizontally inside PlayArea (AC6 + AC7).
        assert!(
            panel_rect.x >= pa.x,
            "AC6/AC7 @ {label}: placement action panel must not overflow PlayArea left"
        );
        assert!(
            panel_rect.right() <= pa.right() + f32::EPSILON,
            "AC6/AC7 @ {label}: placement action panel must not overflow PlayArea right"
        );
        assert!(
            panel_rect.bottom() <= pa.bottom() + f32::EPSILON,
            "AC6/AC7 @ {label}: placement action panel must not overflow PlayArea bottom"
        );
    }
}

// ─── AC2..AC6: spawn-site parent migration is wired through PlayArea ────

#[test]
fn test_play_area_ac2_to_ac5_shop_auction_spawn_sites_parent_into_play_area() {
    let src = read_client_source("ui/shop_auction/mod.rs");

    // Assert — `spawn_shop_auction_ui` reads the `PlayAreaRoot`
    // resource and the four migrated panels parent into the resolved
    // `play_area_parent` entity instead of the historical
    // `ShopAuctionUiRoot`.
    assert!(
        src.contains("play_area_root: Option<Res<crate::ui::PlayAreaRoot>>"),
        "AC2..AC5: spawn_shop_auction_ui must accept `Option<Res<PlayAreaRoot>>`"
    );
    assert!(
        src.contains("play_area_root.as_ref().map(|p| p.0).unwrap_or(root)"),
        "AC2..AC5: spawn_shop_auction_ui must fall back to local root when PlayAreaRoot absent"
    );

    for panel in [
        "ShopAuctionPanelRoot::Shop",
        "ShopAuctionPanelRoot::Auction",
        "ShopAuctionPanelRoot::ShopFooter",
        "ShopAuctionPanelRoot::Toast",
    ] {
        let block_start = src.find(panel).unwrap_or_else(|| {
            panic!("AC2..AC5: marker {panel} missing from spawn_shop_auction_ui")
        });
        // Look back ≤200 chars to the preceding `spawn_panel_root` call
        // and verify the parent argument is `play_area_parent`.
        let window_start = block_start.saturating_sub(200);
        let window = &src[window_start..block_start];
        assert!(
            window.contains("play_area_parent"),
            "AC2..AC5: panel {panel} must spawn under `play_area_parent`; window=\n{window}"
        );
    }
}

#[test]
fn test_play_area_ac6_placement_action_panel_spawn_parents_into_play_area() {
    let src = read_client_source("ui/hand/mod.rs");

    // Assert — `spawn_hand_ui` resolves `placement_action_panel_parent`
    // from `PlayAreaRoot` and parents the panel into it with a
    // `fan_root` fallback for harness apps.
    assert!(
        src.contains("play_area_root: Option<Res<crate::ui::PlayAreaRoot>>"),
        "AC6: spawn_hand_ui must accept `Option<Res<PlayAreaRoot>>`"
    );
    assert!(
        src.contains("let placement_action_panel_parent = play_area_root.as_ref().map(|p| p.0).unwrap_or(fan_root);"),
        "AC6: spawn_hand_ui must resolve placement_action_panel_parent with `unwrap_or(fan_root)`"
    );
    assert!(
        src.contains("ChildOf(placement_action_panel_parent),"),
        "AC6: placement action panel must `ChildOf(placement_action_panel_parent)`"
    );
}

// ─── AC8: strip primitives unchanged ────────────────────────────────────

#[test]
fn test_play_area_ac8_strip_primitives_are_unchanged_viewport_edge_siblings() {
    // Assert — HeaderBar still anchors at viewport `top: 0`.
    let header = header_bar_node();
    assert_eq!(
        header.position_type,
        PositionType::Absolute,
        "AC8: HeaderBar must remain PositionType::Absolute"
    );
    assert_eq!(
        header.top,
        Val::Px(0.0),
        "AC8: HeaderBar must remain anchored at `top: 0` (viewport-edge sibling of PlayArea)"
    );
    assert_eq!(
        header.height,
        Val::Px(HEADER_BAR_HEIGHT_PX),
        "AC8: HeaderBar height must stay at HEADER_BAR_HEIGHT_PX"
    );

    // Assert — HandBar still anchors at viewport `bottom: 0`.
    let hand = hand_bar_node();
    assert_eq!(
        hand.position_type,
        PositionType::Absolute,
        "AC8: HandBar must remain PositionType::Absolute"
    );
    assert_eq!(
        hand.bottom,
        Val::Px(0.0),
        "AC8: HandBar must remain anchored at `bottom: 0` (viewport-edge sibling of PlayArea)"
    );
    assert_eq!(
        hand.height,
        Val::Px(HAND_BAR_HEIGHT_PX),
        "AC8: HandBar height must stay at HAND_BAR_HEIGHT_PX"
    );

    // Assert — FooterBar still anchors at viewport `bottom: HAND_BAR_HEIGHT_PX`.
    let footer = footer_bar_node();
    assert_eq!(
        footer.position_type,
        PositionType::Absolute,
        "AC8: FooterBar must remain PositionType::Absolute"
    );
    assert_eq!(
        footer.bottom,
        Val::Px(HAND_BAR_HEIGHT_PX),
        "AC8: FooterBar must remain anchored at `bottom: HAND_BAR_HEIGHT_PX` (sibling of PlayArea)"
    );
    assert_eq!(
        footer.height,
        Val::Px(FOOTER_BAR_HEIGHT_PX),
        "AC8: FooterBar height must stay at FOOTER_BAR_HEIGHT_PX"
    );
}

#[test]
fn test_play_area_ac8_strips_do_not_overlap_play_area_at_canonical_matrix() {
    // Strip primitives are viewport-edge anchors that frame PlayArea.
    // At every viewport in the canonical matrix the three mandatory
    // strips and the PlayArea rectangle tile the viewport without
    // overlap — this is the "strip-budget contract" PROMPT 1180 Lane A
    // introduces.
    for (label, vw, vh) in CANONICAL_VIEWPORT_MATRIX {
        let viewport = Rect::from_viewport(*vw, *vh);
        let header = compute_absolute_rect(&header_bar_node(), viewport);
        let footer = compute_absolute_rect(&footer_bar_node(), viewport);
        let hand = compute_absolute_rect(&hand_bar_node(), viewport);
        let pa = compute_absolute_rect(&play_area_node(), viewport);

        for (name, a) in [
            ("HeaderBar vs PlayArea", header),
            ("FooterBar vs PlayArea", footer),
            ("HandBar vs PlayArea", hand),
        ] {
            assert!(
                !a.intersects(pa),
                "AC8 @ {label}: {name} must not overlap PlayArea; a={a:?}, pa={pa:?}"
            );
        }

        // The four rectangles also tile the viewport with no gaps and
        // no overlap (the canonical strip-budget tiling).
        let total_height = header.height + pa.height + footer.height + hand.height;
        assert!(
            (total_height - vh).abs() <= f32::EPSILON,
            "AC8 @ {label}: header + play_area + footer + hand_bar must tile the viewport \
             height ({vh}); actual sum {total_height}"
        );
    }
}
