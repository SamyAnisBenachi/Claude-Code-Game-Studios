//! Scroll-region primitive for long modal bodies — Sprint 17 UI layout
//! foundation (PROMPT 1181 / `S17-UI-LAYOUT-FOUNDATION-PRIMITIVES`).
//!
//! When the body region of a modal panel ([`super::modal_panel`]) grows
//! past the available content budget, the surface must either
//! (a) clip the overflow off-screen — which silently hides game state —
//! or (b) scroll the body. Today every modal in the playable client
//! does (a): there is no shared scroll-region builder, and the
//! `result_screen.rs:537-543`, `connection_lost_overlay.rs:230-233`, and
//! `lobby.rs:1161` panels all rely on `max_height: Val::Percent(...)`
//! plus implicit clipping. This module ships the foundation for (b):
//! a deterministic `flex_grow: 1.0`, `flex_shrink: 1.0`,
//! `min_height: 0`, `overflow: Overflow::scroll_y()` body container
//! that grows to fill the budget computed by
//! [`super::modal_panel::modal_panel_content_budget`] and scrolls
//! vertically when its children exceed the available room.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: scroll-region nodes
//!   are read-only presentation primitives.
//! - **ADR-002 Client-Server Authority**: no game state.

use bevy::ecs::component::Component;
use bevy::prelude::default;
use bevy::ui::{AlignItems, Display, FlexDirection, JustifyContent, Node, Overflow, Val};

use crate::ui::design_tokens::spacing::SPACING_SM;

/// Marker component for a canonical scroll body region. Allows tests
/// to query for the body node at runtime without depending on a
/// per-surface marker.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollRegion;

/// Flex-grow value the scroll region MUST carry so it expands to fill
/// the available content budget after the title / CTA strips are sized.
pub const SCROLL_REGION_FLEX_GROW: f32 = 1.0;

/// Flex-shrink value the scroll region MUST carry so it absorbs body
/// overflow rather than pushing the CTA row off-screen.
pub const SCROLL_REGION_FLEX_SHRINK: f32 = 1.0;

/// `min_height` of a scroll region. MUST be `0 px` so the region can
/// shrink under flex pressure — a non-zero min-height would propagate
/// pressure back to the CTA row and break the [`super::cta_row`]
/// invariant.
pub const SCROLL_REGION_MIN_HEIGHT_PX: f32 = 0.0;

/// Build the canonical scroll-region body Node. The region grows
/// vertically to fill the available content budget, scrolls its
/// children on the Y axis, and never imposes a min-height floor that
/// could push neighbouring rows off-screen.
pub fn scroll_region_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Stretch,
        width: Val::Percent(100.0),
        flex_grow: SCROLL_REGION_FLEX_GROW,
        flex_shrink: SCROLL_REGION_FLEX_SHRINK,
        min_height: Val::Px(SCROLL_REGION_MIN_HEIGHT_PX),
        row_gap: Val::Px(SPACING_SM),
        overflow: Overflow::scroll_y(),
        ..default()
    }
}

/// Build a body region that clips overflow without scrolling. Used by
/// surfaces that prefer "drop content past the budget" rather than
/// "scroll to reveal it". The flex contract is identical to
/// [`scroll_region_node`] — only the overflow axis differs.
pub fn clipped_body_region_node() -> Node {
    let mut node = scroll_region_node();
    node.overflow = Overflow::clip_y();
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_region_grows_and_shrinks_under_flex_pressure() {
        let node = scroll_region_node();
        assert_eq!(node.flex_grow, SCROLL_REGION_FLEX_GROW);
        assert_eq!(node.flex_shrink, SCROLL_REGION_FLEX_SHRINK);
        assert!(node.flex_grow > 0.0);
        assert!(node.flex_shrink > 0.0);
    }

    #[test]
    fn scroll_region_min_height_is_zero_to_protect_cta_row() {
        let node = scroll_region_node();
        match node.min_height {
            Val::Px(px) => assert!((px - SCROLL_REGION_MIN_HEIGHT_PX).abs() < f32::EPSILON),
            other => panic!("scroll region min_height must be Val::Px(0.0); got {other:?}"),
        }
    }

    #[test]
    fn scroll_region_overflow_is_scroll_y_not_clip() {
        let node = scroll_region_node();
        // `scroll_y` differs from `clip_y` in the Y-axis OverflowAxis
        // variant — `Scroll` vs `Clip`. We assert the shape via the
        // visible / invisible API instead of re-comparing the
        // OverflowAxis enum.
        assert!(!node.overflow.is_visible());
        // The scroll variant must NOT equal a clip-y body.
        assert_ne!(node.overflow, Overflow::clip_y());
        assert_eq!(node.overflow, Overflow::scroll_y());
    }

    #[test]
    fn clipped_body_region_shares_flex_contract_but_clips_instead_of_scrolling() {
        let scroll = scroll_region_node();
        let clipped = clipped_body_region_node();
        assert_eq!(scroll.flex_grow, clipped.flex_grow);
        assert_eq!(scroll.flex_shrink, clipped.flex_shrink);
        assert_eq!(scroll.min_height, clipped.min_height);
        assert_eq!(clipped.overflow, Overflow::clip_y());
        assert_ne!(scroll.overflow, clipped.overflow);
    }

    #[test]
    fn scroll_region_width_is_full_panel() {
        let node = scroll_region_node();
        assert_eq!(node.width, Val::Percent(100.0));
    }
}
