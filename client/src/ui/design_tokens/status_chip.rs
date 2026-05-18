//! Status-chip vs button visual-role primitive — Sprint 17 UI layout
//! foundation (PROMPT 1181 / `S17-UI-LAYOUT-FOUNDATION-PRIMITIVES`).
//!
//! Two visually similar elements coexist in every modal-surface:
//!
//! - **Status chips** — read-only labels carrying a semantic state
//!   (auction "Winning" / "Tied" / "Losing"; hand-full banner; shop
//!   "Out of stock"; lobby room-code pill). They are NOT interactive —
//!   clicking them does nothing.
//! - **CTA buttons** — interactive affordances (see [`super::cta_row`]).
//!   They MUST be clickable, focus-trappable, and disabled-state aware.
//!
//! PROMPT 1077 §"UI state source consistency" surfaced multiple cases
//! where read-only chips were spawned with the bevy `Button` component
//! plus `Interaction`, advertising themselves as clickable — and where
//! CTA buttons were spawned without a [`super::cta_row::CtaButton`]
//! marker, leaving QA without a stable query. This primitive enforces
//! the boundary at the API level: `status_chip_node` produces a node
//! that explicitly does NOT carry the `Button` marker and pairs with
//! a [`StatusChip`] marker for runtime queries; `cta_button_node` (in
//! [`super::cta_row`]) is the canonical interactive counterpart.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: chip / button role
//!   tokens are read-only presentation primitives.
//! - **ADR-002 Client-Server Authority**: chip state is rendered from
//!   already-replicated authoritative state; the chip itself does not
//!   carry game state.

use bevy::ecs::component::Component;
use bevy::prelude::default;
use bevy::ui::{AlignItems, Display, FlexDirection, JustifyContent, Node, UiRect, Val};

use crate::ui::design_tokens::spacing::{SPACING_MD, SPACING_XS};
use crate::ui::design_tokens::typography::CAPTION;

/// Canonical status-chip pixel height. Sized to host a `CAPTION` glyph
/// (13 px) plus padding × 2 (`SPACING_XS` = 4 px), totalling 21 px —
/// rounded up to 22 px so chips align cleanly to the typography line
/// rhythm and visibly read as smaller than [`super::cta_row::CTA_ROW_HEIGHT_PX`].
pub const STATUS_CHIP_HEIGHT_PX: f32 = 22.0;

/// Horizontal padding inside a status chip — `SPACING_MD` / 2 = 8 px.
pub const STATUS_CHIP_PADDING_X_PX: f32 = SPACING_MD / 2.0;

/// Vertical padding inside a status chip — `SPACING_XS` = 4 px.
pub const STATUS_CHIP_PADDING_Y_PX: f32 = SPACING_XS;

/// Default text size inside a status chip. Always `CAPTION` so the chip
/// reads as a secondary label, not a primary readout. Surfaces that
/// need a larger semantic readout (auction lead state, etc.) must use
/// a typography token directly — they are no longer chip-shaped.
pub const STATUS_CHIP_TEXT_SIZE_PX: f32 = CAPTION;

/// Marker component for the canonical status-chip primitive. Surfaces
/// MUST add this marker when spawning a chip so QA queries can
/// distinguish chips from CTA buttons at runtime.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusChip;

/// Visual-role classification surfaced for documentation and test
/// assertions. The two roles are mutually exclusive — a node is
/// either a status chip (read-only) or a CTA button (interactive).
/// Surfaces must pick one role per node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualRole {
    /// Read-only chip. Carries [`StatusChip`]. Does NOT carry the
    /// bevy `Button` marker, does NOT carry `Interaction`, does NOT
    /// participate in keyboard focus.
    StatusChip,
    /// Interactive CTA button. Carries `Button` + `Interaction` (added
    /// by the surface) and the [`super::cta_row::CtaButton`] marker.
    CtaButton,
}

impl VisualRole {
    /// `true` if a node of this role accepts pointer / keyboard input.
    pub const fn is_interactive(self) -> bool {
        matches!(self, Self::CtaButton)
    }

    /// `true` if a node of this role is purely read-only.
    pub const fn is_read_only(self) -> bool {
        matches!(self, Self::StatusChip)
    }
}

/// Build the canonical status-chip Node. The chip is sized to a small
/// pixel-fixed pill, with the friend-game `CAPTION` typography token
/// inside and minimal padding. The Node does NOT pair with the bevy
/// `Button` component — surfaces that want pointer feedback must use
/// [`super::cta_row::cta_button_node`] instead.
pub fn status_chip_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        height: Val::Px(STATUS_CHIP_HEIGHT_PX),
        min_height: Val::Px(STATUS_CHIP_HEIGHT_PX),
        padding: UiRect::axes(
            Val::Px(STATUS_CHIP_PADDING_X_PX),
            Val::Px(STATUS_CHIP_PADDING_Y_PX),
        ),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::design_tokens::cta_row::CTA_ROW_HEIGHT_PX;

    #[test]
    fn status_chip_is_visually_smaller_than_cta_row() {
        // Visual-role distinction relies on the chip reading as
        // shorter than the click-target row above / beside it.
        assert!(
            STATUS_CHIP_HEIGHT_PX < CTA_ROW_HEIGHT_PX,
            "STATUS_CHIP_HEIGHT_PX ({STATUS_CHIP_HEIGHT_PX}) must be < \
             CTA_ROW_HEIGHT_PX ({CTA_ROW_HEIGHT_PX}) so chips read as \
             secondary labels rather than interactive affordances"
        );
    }

    #[test]
    fn status_chip_text_size_is_caption_not_body() {
        // Drift-guard: a future refactor that boosts chip text to
        // BODY (15 px) would visually merge chips and buttons.
        assert_eq!(STATUS_CHIP_TEXT_SIZE_PX, CAPTION);
    }

    #[test]
    fn visual_role_is_either_interactive_or_read_only_never_both() {
        for role in [VisualRole::StatusChip, VisualRole::CtaButton] {
            assert_ne!(
                role.is_interactive(),
                role.is_read_only(),
                "VisualRole {role:?} must be exactly one of interactive/read-only"
            );
        }
    }

    #[test]
    fn status_chip_role_is_read_only_cta_button_role_is_interactive() {
        assert!(VisualRole::StatusChip.is_read_only());
        assert!(!VisualRole::StatusChip.is_interactive());
        assert!(VisualRole::CtaButton.is_interactive());
        assert!(!VisualRole::CtaButton.is_read_only());
    }

    #[test]
    fn status_chip_node_uses_pixel_fixed_height_not_percent() {
        let node = status_chip_node();
        match node.height {
            Val::Px(px) => assert!((px - STATUS_CHIP_HEIGHT_PX).abs() < f32::EPSILON),
            other => panic!("status chip height must be Val::Px(_); got {other:?}"),
        }
        match node.min_height {
            Val::Px(px) => assert!((px - STATUS_CHIP_HEIGHT_PX).abs() < f32::EPSILON),
            other => panic!("status chip min_height must be Val::Px(_); got {other:?}"),
        }
    }

    #[test]
    fn status_chip_padding_uses_spacing_tokens() {
        assert!((STATUS_CHIP_PADDING_X_PX - SPACING_MD / 2.0).abs() < f32::EPSILON);
        assert!((STATUS_CHIP_PADDING_Y_PX - SPACING_XS).abs() < f32::EPSILON);
    }
}
