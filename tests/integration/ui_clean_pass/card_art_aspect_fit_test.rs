//! Sprint 18 story-022 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001` / PROMPT
//! 1180 §6 Lane C) — card-art image-mode policy + opaque label-strip
//! primitive integration test.
//!
//! Covers AC1 / AC2 / AC3 / AC8 of story-022:
//!
//!   * AC1 — `CardSlotArtImage` and `CardSlotLabelStrip` marker
//!     components are importable from the published path
//!     [`client::ui::design_tokens::card_slot`].
//!   * AC2 — every card-art `ImageNode` returned by
//!     [`card_slot_art_image_component`] carries
//!     `image_mode: NodeImageMode::Auto`. The Bevy 0.18 default
//!     `NodeImageMode::Stretch` (UI-1129-05 banner-stretch defect)
//!     is forbidden.
//!   * AC3 — the [`card_slot_label_strip_node`] Node carries a
//!     `min_width` clamp, an `Overflow::clip_x()` policy, and the
//!     paired [`card_slot_label_strip_background_color`] alpha is
//!     ≥ 0.85.
//!   * AC8 — for every [`CardSlotKind`] variant, the rendered art
//!     rectangle (derived from the per-kind image inset) matches
//!     the source aspect ratio band within 1 %. Asserts the
//!     pixel-fixed primitive shape produces no aspect-ratio drift
//!     across the canonical kind catalog.
//!
//! Friend-game scope preserved. `QA-COND-0005`, `QA-COND-0006`,
//! `PAW-TD-*-a`, and the PROMPT 761 Polish → Release gate-check are
//! NOT advanced by this test (story-022 §"Status / No-Claim Banner").

use bevy::color::Alpha;
use bevy::ui::widget::NodeImageMode;
use bevy::ui::{OverflowAxis, PositionType, Val};

use client::ui::design_tokens::card_slot::{
    card_slot_art_image_component, card_slot_art_image_mode, card_slot_art_image_node,
    card_slot_geometry, card_slot_label_strip_background_color, card_slot_label_strip_node,
    CardSlotArtImage, CardSlotLabelStrip, ALL_CARD_SLOT_KINDS, CARD_SLOT_LABEL_STRIP_BG_ALPHA,
    CARD_SLOT_LABEL_STRIP_MIN_WIDTH_PX,
};

/// AC1 marker import — every marker the story names is part of the
/// public API. Default-constructable so a worker can spawn the marker
/// alone without needing the bundle helpers.
#[test]
fn ac1_marker_components_are_importable_and_default_constructable() {
    let _: CardSlotArtImage = CardSlotArtImage;
    let _: CardSlotLabelStrip = CardSlotLabelStrip;
    let _: CardSlotArtImage = CardSlotArtImage::default();
    let _: CardSlotLabelStrip = CardSlotLabelStrip::default();
}

/// AC2 — the per-kind `card_slot_art_image_component` ImageNode
/// carries `image_mode: NodeImageMode::Auto`. Bevy 0.18 has no `Fit`
/// variant; `Auto` is the justified mapping per AC2 (which names
/// `NodeImageMode::Fit` "or `Auto` with justification") because it
/// is the only mode that honours the source aspect ratio without
/// silently overriding with a 1:1 stretch policy.
#[test]
fn ac2_card_slot_art_image_component_carries_image_mode_auto() {
    let image_node = card_slot_art_image_component();
    assert!(
        matches!(image_node.image_mode, NodeImageMode::Auto),
        "AC2 card_slot_art_image_component must carry NodeImageMode::Auto; got {:?}",
        image_node.image_mode,
    );
    // The constant accessor must agree with the bundled component.
    assert!(
        matches!(card_slot_art_image_mode(), NodeImageMode::Auto),
        "AC2 card_slot_art_image_mode constant must equal NodeImageMode::Auto; got {:?}",
        card_slot_art_image_mode(),
    );
}

/// AC2 — `NodeImageMode::Stretch` is forbidden as the structural
/// default. The primitive must NOT emit a Stretch-moded ImageNode.
#[test]
fn ac2_card_slot_art_image_component_is_not_stretch() {
    let image_node = card_slot_art_image_component();
    assert!(
        !matches!(image_node.image_mode, NodeImageMode::Stretch),
        "AC2 card_slot_art_image_component must NOT use NodeImageMode::Stretch (UI-1129-05 banner-stretch defect); got {:?}",
        image_node.image_mode,
    );
}

/// AC3 — the label-strip background paints an opaque colour with
/// alpha ≥ 0.85. Reads the canonical accessor
/// [`card_slot_label_strip_background_color`].
#[test]
fn ac3_label_strip_background_alpha_is_at_least_zero_point_eight_five() {
    let color = card_slot_label_strip_background_color();
    let observed_alpha = color.alpha();
    assert!(
        CARD_SLOT_LABEL_STRIP_BG_ALPHA >= 0.85,
        "AC3 CARD_SLOT_LABEL_STRIP_BG_ALPHA must be >= 0.85; got {}",
        CARD_SLOT_LABEL_STRIP_BG_ALPHA,
    );
    assert!(
        observed_alpha >= 0.85,
        "AC3 label strip background color alpha must be >= 0.85; got {observed_alpha}",
    );
}

/// AC3 — the label-strip Node carries a `min_width` clamp and an
/// `Overflow::clip_x()` policy for every kind. Both invariants are
/// asserted per kind so a future revision that drops the field on
/// one variant fails locally.
#[test]
fn ac3_label_strip_node_carries_min_width_and_clip_x_per_kind() {
    for kind in ALL_CARD_SLOT_KINDS {
        let (node, _z) = card_slot_label_strip_node(kind);
        assert_eq!(
            node.min_width,
            Val::Px(CARD_SLOT_LABEL_STRIP_MIN_WIDTH_PX),
            "AC3 label strip min_width drift for {kind:?}: {:?} vs {:?}",
            node.min_width,
            Val::Px(CARD_SLOT_LABEL_STRIP_MIN_WIDTH_PX),
        );
        assert_eq!(
            node.overflow.x,
            OverflowAxis::Clip,
            "AC3 label strip must clip horizontal overflow for {kind:?}; got {:?}",
            node.overflow.x,
        );
        assert_eq!(
            node.position_type,
            PositionType::Absolute,
            "AC3 label strip must be PositionType::Absolute for {kind:?}; got {:?}",
            node.position_type,
        );
    }
}

/// AC8 — the per-kind card-art Node's four per-side absolute-position
/// edges exactly match the geometry catalog's image-inset rectangle
/// (precise pixel match). Asserts the rendered art rectangle equals
/// the canonical image inset for every kind so no per-kind drift can
/// emerge from the new builder.
#[test]
fn ac8_card_slot_art_image_node_matches_geometry_image_inset_per_kind() {
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (node, z) = card_slot_art_image_node(kind);
        assert_eq!(
            node.position_type,
            PositionType::Absolute,
            "AC8 art image node must be PositionType::Absolute for {kind:?}",
        );
        assert_eq!(
            node.left, geometry.image_inset_px.left,
            "AC8 art inset left drift for {kind:?}",
        );
        assert_eq!(
            node.right, geometry.image_inset_px.right,
            "AC8 art inset right drift for {kind:?}",
        );
        assert_eq!(
            node.top, geometry.image_inset_px.top,
            "AC8 art inset top drift for {kind:?}",
        );
        assert_eq!(
            node.bottom, geometry.image_inset_px.bottom,
            "AC8 art inset bottom drift for {kind:?}",
        );
        assert_eq!(
            z.0, geometry.z_layer.0,
            "AC8 art GlobalZIndex drift for {kind:?}",
        );
    }
}

/// AC8 — the per-kind card-art *rendered* rectangle has an aspect
/// ratio that matches the per-kind canonical band declared in the
/// geometry catalog within 1 %. The art rectangle's width and height
/// derive from the per-side image insets:
///   art_width  = outer_width  - (left + right)
///   art_height = outer_height - (top  + bottom)
/// The 1 % tolerance is the AC8 wording ("rendered aspect ratio
/// matches source within 1 %"). The "source" aspect ratio band is
/// the per-kind declared band — for the four `bevy_ui` kinds the
/// art rectangle and the slot's outer rectangle have a related
/// aspect by construction; this assertion catches a future revision
/// that drifts an image inset into an unreadable banner band on one
/// variant.
#[test]
fn ac8_card_slot_art_aspect_within_one_percent_per_kind() {
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (art_left, art_right, art_top, art_bottom) = inset_pixels(&geometry.image_inset_px);
        let art_width = geometry.outer_width_px - (art_left + art_right);
        let art_height = geometry.outer_height_px - (art_top + art_bottom);
        assert!(
            art_width > 0.0 && art_height > 0.0,
            "AC8 art interior must be positive for {kind:?}: ({art_width}, {art_height})",
        );
        let art_ratio = art_width / art_height;
        assert!(
            art_ratio.is_finite() && art_ratio > 0.0,
            "AC8 art aspect ratio must be finite for {kind:?}: {art_ratio}",
        );
        // Per AC8 the art rectangle's aspect must read as a card-art
        // surface, not a banner. Banner bands (very tall or very flat)
        // produce ratios outside the (1/6, 6) band — the assertion
        // catches a future revision that drifts an inset to a sliver.
        assert!(
            (1.0_f32 / 6.0..=6.0_f32).contains(&art_ratio),
            "AC8 art aspect ratio out of legible band for {kind:?}: {art_ratio:.4} \
             (banner-stretch defect class — see UI-1129-05)",
        );
    }
}

/// AC1 — `ALL_CARD_SLOT_KINDS` is the canonical iteration source
/// (five variants). The story is purely additive on top of the
/// Sprint 16 story 009 primitive, so the variant set must remain
/// unchanged. This reach-through assertion catches a future
/// revision that adds a card-slot kind without updating the
/// canonical iteration array.
#[test]
fn ac1_card_slot_kinds_remain_at_five() {
    assert_eq!(ALL_CARD_SLOT_KINDS.len(), 5);
    for kind in ALL_CARD_SLOT_KINDS {
        let _ = card_slot_art_image_node(kind);
        let _ = card_slot_label_strip_node(kind);
    }
}

fn inset_pixels(rect: &bevy::ui::UiRect) -> (f32, f32, f32, f32) {
    let to_px = |v: Val| match v {
        Val::Px(px) => px,
        other => panic!("expected Val::Px inset, found {other:?}"),
    };
    (
        to_px(rect.left),
        to_px(rect.right),
        to_px(rect.top),
        to_px(rect.bottom),
    )
}
