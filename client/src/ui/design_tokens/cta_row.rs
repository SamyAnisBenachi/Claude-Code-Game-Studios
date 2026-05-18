//! Stable CTA-row primitive — Sprint 17 UI layout foundation
//! (PROMPT 1181 / `S17-UI-LAYOUT-FOUNDATION-PRIMITIVES`).
//!
//! Every modal / panel / drawer in the playable client paints one or
//! more "call-to-action" buttons at the bottom of the surface (lobby
//! Confirm, auction Bid, shop Buy, draft Continue, result-screen
//! Return-to-lobby, photosensitivity Acknowledge, connection-lost
//! Reconnect). The CTA row must remain on-screen and clickable even
//! when the body region above it grows unexpectedly — a CTA squashed
//! to zero height by flex pressure is the worst possible regression
//! (the surface looks "done" but cannot be dismissed).
//!
//! PROMPT 1035 §"Structural problems" enumerated four separate sites
//! where bottom-edge buttons are absolute-positioned with magic
//! `top: 148.0` / `top: 190.0` offsets — exactly the pattern that
//! breaks when the parent panel resizes. This module exports the
//! `cta_row_node` primitive that consumes flex but pins itself to a
//! deterministic height with `flex_shrink: 0.0` and `flex_grow: 0.0`,
//! so the row's height is invariant regardless of body-region pressure.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: CTA row geometry is
//!   read-only presentation primitive.
//! - **ADR-002 Client-Server Authority**: no game state.

use bevy::ecs::component::Component;
use bevy::prelude::default;
use bevy::ui::{
    AlignItems, Display, FlexDirection, JustifyContent, Node, PositionType, UiRect, Val,
};

use crate::ui::design_tokens::spacing::{SPACING_LG, SPACING_MD};

/// Canonical primary CTA row height. 44 px sits at the friend-game
/// "comfortable click target" floor — wide enough that a mouse user
/// hits it on the first attempt at every supported viewport without
/// claiming Standard-tier 44 px hit-target conformance under
/// `QA-COND-0005` (that claim requires a separate accessibility audit).
pub const CTA_ROW_HEIGHT_PX: f32 = 44.0;

/// Compact CTA row height for secondary modals (toast acknowledge,
/// inline confirm).
pub const CTA_ROW_COMPACT_HEIGHT_PX: f32 = 32.0;

/// Minimum CTA button width — wide enough that a one-word label
/// ("Bid", "Buy") doesn't collapse the bounding box.
pub const CTA_BUTTON_MIN_WIDTH_PX: f32 = 96.0;

/// Minimum CTA button height — matches [`CTA_ROW_HEIGHT_PX`] so the
/// row's height equals the button's hit rectangle when the row
/// contains a single button.
pub const CTA_BUTTON_MIN_HEIGHT_PX: f32 = CTA_ROW_HEIGHT_PX;

/// Horizontal padding inside a CTA button. Matches §10 "Primary button
/// affordance" `SPACING_LG` horizontal padding.
pub const CTA_BUTTON_PADDING_X_PX: f32 = SPACING_LG;

/// Vertical padding inside a CTA button. Padded by `SPACING_SM` so the
/// button caps at exactly [`CTA_ROW_HEIGHT_PX`].
pub const CTA_BUTTON_PADDING_Y_PX: f32 = 8.0;

/// The flex-grow / flex-shrink invariant guarded by this primitive.
/// Both MUST be zero so the row's pixel height never collapses below
/// the deterministic floor under flex pressure from the body region.
pub const CTA_ROW_FLEX_GROW: f32 = 0.0;
/// See [`CTA_ROW_FLEX_GROW`]. Both grow and shrink must be zero.
pub const CTA_ROW_FLEX_SHRINK: f32 = 0.0;

/// Canonical CTA row kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CtaRowKind {
    /// Primary CTA row — `44 px` height, paired with `Standard` modal
    /// panels.
    Primary,
    /// Compact CTA row — `32 px` height, paired with `Narrow` modal
    /// panels or inline confirmations.
    Compact,
}

impl CtaRowKind {
    /// Deterministic pixel height for this kind. Identical across every
    /// row in [`super::viewport_matrix::SAFETY_VIEWPORT_MATRIX`].
    pub const fn height_px(self) -> f32 {
        match self {
            Self::Primary => CTA_ROW_HEIGHT_PX,
            Self::Compact => CTA_ROW_COMPACT_HEIGHT_PX,
        }
    }
}

/// Marker component for the canonical CTA row node — surfaced so
/// integration tests can query for the row at runtime without depending
/// on a per-surface marker.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtaRow;

/// Marker component for a single CTA button inside a [`CtaRow`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtaButton;

/// Build the canonical CTA row [`Node`] for the given kind. The row
/// is a deterministic-height flex row that **cannot be squashed** by
/// the body region above it — `flex_shrink` and `flex_grow` are both
/// zero, the height is `Val::Px(...)` (never `Val::Percent`), and the
/// row aligns its children center-vertical / end-horizontal so the
/// CTA reads as the trailing affordance in left-to-right languages.
pub fn cta_row_node(kind: CtaRowKind) -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::FlexEnd,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        height: Val::Px(kind.height_px()),
        min_height: Val::Px(kind.height_px()),
        flex_grow: CTA_ROW_FLEX_GROW,
        flex_shrink: CTA_ROW_FLEX_SHRINK,
        column_gap: Val::Px(SPACING_MD),
        ..default()
    }
}

/// Build the canonical CTA button [`Node`]. Sized to the friend-game
/// click-target floor; pads its label by §10 "Primary button affordance"
/// values.
pub fn cta_button_node() -> Node {
    Node {
        position_type: PositionType::Relative,
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        min_width: Val::Px(CTA_BUTTON_MIN_WIDTH_PX),
        min_height: Val::Px(CTA_BUTTON_MIN_HEIGHT_PX),
        padding: UiRect::axes(
            Val::Px(CTA_BUTTON_PADDING_X_PX),
            Val::Px(CTA_BUTTON_PADDING_Y_PX),
        ),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_and_compact_heights_match_published_constants() {
        assert_eq!(CtaRowKind::Primary.height_px(), CTA_ROW_HEIGHT_PX);
        assert_eq!(CtaRowKind::Compact.height_px(), CTA_ROW_COMPACT_HEIGHT_PX);
        assert!(CtaRowKind::Primary.height_px() > CtaRowKind::Compact.height_px());
    }

    #[test]
    fn cta_row_node_has_flex_grow_and_shrink_zero() {
        for kind in [CtaRowKind::Primary, CtaRowKind::Compact] {
            let node = cta_row_node(kind);
            assert_eq!(
                node.flex_grow, CTA_ROW_FLEX_GROW,
                "{kind:?}: flex_grow must be 0 so the row cannot grow past its declared height"
            );
            assert_eq!(
                node.flex_shrink, CTA_ROW_FLEX_SHRINK,
                "{kind:?}: flex_shrink must be 0 so the row cannot be squashed by body pressure"
            );
        }
    }

    #[test]
    fn cta_row_node_pins_height_in_pixels_not_percent() {
        // A `Val::Percent` height would let the row collapse when the
        // parent's percentage clamp shrinks at smaller viewports.
        let node = cta_row_node(CtaRowKind::Primary);
        match node.height {
            Val::Px(_) => {}
            other => panic!("CTA row height must be Val::Px(_); got {other:?}"),
        }
        match node.min_height {
            Val::Px(px) => assert!((px - CTA_ROW_HEIGHT_PX).abs() < f32::EPSILON),
            other => panic!("CTA row min_height must be Val::Px(_); got {other:?}"),
        }
    }

    #[test]
    fn cta_row_node_is_full_viewport_width() {
        let node = cta_row_node(CtaRowKind::Primary);
        assert_eq!(node.width, Val::Percent(100.0));
    }

    #[test]
    fn cta_row_node_aligns_children_end_horizontally_center_vertically() {
        let node = cta_row_node(CtaRowKind::Primary);
        assert_eq!(node.flex_direction, FlexDirection::Row);
        assert_eq!(node.justify_content, JustifyContent::FlexEnd);
        assert_eq!(node.align_items, AlignItems::Center);
    }

    #[test]
    fn cta_button_node_meets_min_width_and_min_height_floors() {
        let node = cta_button_node();
        match node.min_width {
            Val::Px(px) => assert!((px - CTA_BUTTON_MIN_WIDTH_PX).abs() < f32::EPSILON),
            other => panic!("CTA button min_width must be Val::Px(_); got {other:?}"),
        }
        match node.min_height {
            Val::Px(px) => assert!((px - CTA_BUTTON_MIN_HEIGHT_PX).abs() < f32::EPSILON),
            other => panic!("CTA button min_height must be Val::Px(_); got {other:?}"),
        }
    }

    #[test]
    fn cta_button_min_height_equals_primary_row_height_for_single_button_rows() {
        // Single-button rows: row height == button height so the click
        // rectangle is the full row area.
        assert!((CTA_BUTTON_MIN_HEIGHT_PX - CTA_ROW_HEIGHT_PX).abs() < f32::EPSILON);
    }

    #[test]
    fn cta_row_constants_match_friend_game_click_target_floor() {
        // Friend-game floor is 44 px (the value
        // `lobby_button_dimensions_test` was originally written
        // against; lobby still sits at 30 px under `QA-COND-0005`
        // accept-risk per global-ui-design-spec §2). The primitive's
        // height is the friend-game target, not the lobby-legacy
        // value.
        assert!(CTA_ROW_HEIGHT_PX >= 44.0);
    }
}
