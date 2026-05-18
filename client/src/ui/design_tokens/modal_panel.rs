//! Modal / centred-panel content-budget primitive — Sprint 17 UI layout
//! foundation (PROMPT 1181 / `S17-UI-LAYOUT-FOUNDATION-PRIMITIVES`).
//!
//! Every centred modal panel in the playable client (result screen,
//! photosensitivity warning, connection-lost overlay, lobby modal,
//! draft-initial modal, settings shell) has the same shape:
//!
//! 1. An outer rectangle clamped to `(max_width_px, max_height_px)`.
//! 2. Padding inset on all four edges.
//! 3. A title strip at the top.
//! 4. A scrollable body / content region in the middle.
//! 5. A stable CTA row at the bottom that MUST remain visible.
//!
//! PROMPT 1035 §"Structural problems" and the §"Hardcoded-value findings"
//! tables flagged that three different modal RGB triples and three
//! different `row_gap` / `padding` literals exist across `result_screen`,
//! `connection_lost_overlay`, and `photosensitivity_warning`, and that
//! `lobby.rs::LOBBY_PANEL_MAX_HEIGHT_PERCENT = 92.0` + `width: 88%`,
//! `shop_auction/mod.rs::DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT` each
//! re-author their own viewport clamp. This module makes the budget
//! arithmetic a single source of truth: a surface declares its
//! [`ModalPanelKind`] (or its explicit [`ModalPanelBudget`] inputs) and
//! the primitive computes whether the panel actually has room for its
//! content **before** the surface is spawned.
//!
//! ## Why this is foundation, not migration
//!
//! The audit reports above enumerate the surfaces that should consume
//! the primitive. PROMPT 1181 deliberately ships **only** the primitive
//! + tests; surface migration is owned by Sprint 17+ follow-on rows
//! (e.g. `S17-UI-MODAL-PANEL-CHROME-MIGRATION-*`). The viewport-safety
//! invariant is asserted by [`assert_fits_smallest_safety_viewport`]
//! against [`super::viewport_matrix::SAFETY_VIEWPORT_SMALLEST`] so a
//! later surface worker who claims "fits at 1280×720" cannot ship a
//! panel whose padding + title + CTA row + min-body height exceeds the
//! clamp.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: budget arithmetic is
//!   a read-only presentation primitive.
//! - **ADR-002 Client-Server Authority**: no game state.

use bevy::prelude::default;
use bevy::ui::{
    AlignItems, Display, FlexDirection, JustifyContent, Node, Overflow, PositionType, UiRect, Val,
};

use crate::ui::design_tokens::spacing::{SPACING_LG, SPACING_MD};
use crate::ui::design_tokens::viewport_matrix::{SafetyViewport, SAFETY_VIEWPORT_SMALLEST};

/// Default outer-rectangle width clamp expressed as a *percentage* of
/// the viewport width. Matches `lobby.rs::LOBBY_PANEL_WIDTH_PERCENT =
/// 88.0` per PROMPT 933 — the lobby is the canonical centred-modal
/// width clamp on `origin/main`.
pub const MODAL_PANEL_DEFAULT_WIDTH_PERCENT: f32 = 88.0;

/// Default outer-rectangle height clamp expressed as a *percentage* of
/// the viewport height. Matches `lobby.rs::LOBBY_PANEL_MAX_HEIGHT_PERCENT
/// = 92.0` per PROMPT 933.
pub const MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT: f32 = 92.0;

/// Pixel-fixed maximum outer-rectangle width for the `Standard` modal
/// kind (result screen, lobby modal). Mirrors
/// `result_screen.rs:537-543` panel sizes.
pub const MODAL_PANEL_STANDARD_MAX_WIDTH_PX: f32 = 860.0;

/// Pixel-fixed maximum outer-rectangle width for the `Narrow` modal
/// kind (connection-lost, photosensitivity). Mirrors
/// `connection_lost_overlay.rs:230-233` panel `max_width: 520`.
pub const MODAL_PANEL_NARROW_MAX_WIDTH_PX: f32 = 520.0;

/// Default title strip height inside a modal panel. Sized to fit
/// typography::H2 (22 px) plus its line-height-default ratio with
/// breathing room.
pub const MODAL_PANEL_TITLE_STRIP_HEIGHT_PX: f32 = 36.0;

/// Default panel padding on every edge for the `Standard` modal kind —
/// `SPACING_LG` per `docs/ux/global-ui-design-spec.md` §10 "Panel
/// chrome" / "primary modals".
pub const MODAL_PANEL_STANDARD_PADDING_PX: f32 = SPACING_LG;

/// Default panel padding on every edge for the `Narrow` modal kind —
/// `SPACING_MD` per `docs/ux/global-ui-design-spec.md` §10 "Panel
/// chrome" / "inline panels".
pub const MODAL_PANEL_NARROW_PADDING_PX: f32 = SPACING_MD;

/// Default 1 px border width drawn around a modal panel for chrome
/// affordance. Surfaces MAY override; the primitive bakes a minimum so
/// budget arithmetic accounts for the border on both edges.
pub const MODAL_PANEL_BORDER_WIDTH_PX: f32 = 1.0;

/// Minimum body / content region height required for the panel to be
/// considered "useful" once the title strip, CTA row, padding, and
/// border are subtracted from the outer rectangle. Below this floor the
/// panel is unreadable and must either drop the title, drop the CTA
/// row, or pick a larger height clamp. 80 px ≈ three lines of `BODY`
/// text at the default line height — the friend-game readable floor.
pub const MODAL_PANEL_MIN_BODY_HEIGHT_PX: f32 = 80.0;

/// Canonical modal-panel kinds. Each kind binds a (width clamp, padding,
/// CTA-row floor) triple so callers do not re-author them per surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModalPanelKind {
    /// Wider primary-modal kind — result screen, lobby modal,
    /// draft-initial modal. `SPACING_LG` padding, `860 px` max width.
    Standard,
    /// Narrower secondary-modal kind — connection-lost,
    /// photosensitivity, settings shell. `SPACING_MD` padding,
    /// `520 px` max width.
    Narrow,
}

impl ModalPanelKind {
    /// Maximum outer-rectangle width in pixels for this kind.
    pub const fn max_width_px(self) -> f32 {
        match self {
            Self::Standard => MODAL_PANEL_STANDARD_MAX_WIDTH_PX,
            Self::Narrow => MODAL_PANEL_NARROW_MAX_WIDTH_PX,
        }
    }

    /// Padding inset on every edge of the panel for this kind.
    pub const fn padding_px(self) -> f32 {
        match self {
            Self::Standard => MODAL_PANEL_STANDARD_PADDING_PX,
            Self::Narrow => MODAL_PANEL_NARROW_PADDING_PX,
        }
    }
}

/// Per-surface inputs to the content-budget calculation.
///
/// A surface declares the heights of its own *content slots* — title
/// strip, CTA row, optional extras — and the primitive computes whether
/// the modal's outer-rectangle clamp at the supplied viewport has room
/// for them plus a readable [`MODAL_PANEL_MIN_BODY_HEIGHT_PX`] body
/// region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalPanelBudget {
    pub kind: ModalPanelKind,
    /// Height of the title strip inside the panel. Use
    /// [`MODAL_PANEL_TITLE_STRIP_HEIGHT_PX`] for the default H2 title.
    pub title_strip_px: f32,
    /// Height of the bottom CTA row. Use
    /// [`super::cta_row::CTA_ROW_HEIGHT_PX`] for the canonical primary
    /// CTA row.
    pub cta_row_px: f32,
    /// Gap between every consecutive section (title ↔ body, body ↔ CTA).
    /// Use [`SPACING_MD`] by default.
    pub section_gap_px: f32,
    /// Required minimum body-region height. Use
    /// [`MODAL_PANEL_MIN_BODY_HEIGHT_PX`] for the friend-game floor.
    pub min_body_px: f32,
}

impl ModalPanelBudget {
    /// Convenience constructor with [`MODAL_PANEL_TITLE_STRIP_HEIGHT_PX`],
    /// [`SPACING_MD`] section gap, and
    /// [`MODAL_PANEL_MIN_BODY_HEIGHT_PX`] body floor.
    pub const fn with_defaults(kind: ModalPanelKind, cta_row_px: f32) -> Self {
        Self {
            kind,
            title_strip_px: MODAL_PANEL_TITLE_STRIP_HEIGHT_PX,
            cta_row_px,
            section_gap_px: SPACING_MD,
            min_body_px: MODAL_PANEL_MIN_BODY_HEIGHT_PX,
        }
    }

    /// Vertical chrome height — the sum of every fixed-height slot that
    /// is NOT the body region. Padding × 2 (top + bottom), border × 2,
    /// title strip, CTA row, two section gaps.
    pub fn chrome_height_px(&self) -> f32 {
        let padding = self.kind.padding_px();
        2.0 * padding
            + 2.0 * MODAL_PANEL_BORDER_WIDTH_PX
            + self.title_strip_px
            + self.cta_row_px
            + 2.0 * self.section_gap_px
    }

    /// Total minimum outer height the panel needs to satisfy
    /// `min_body_px` once the chrome is subtracted.
    pub fn min_outer_height_px(&self) -> f32 {
        self.chrome_height_px() + self.min_body_px
    }
}

/// Errors surfaced by [`modal_panel_content_budget`] when the requested
/// budget cannot fit at the supplied viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContentBudgetError {
    /// The outer-height clamp at this viewport is too small to host the
    /// declared chrome + min-body slots. `available` is the outer
    /// rectangle height after the percentage clamp; `required` is
    /// [`ModalPanelBudget::min_outer_height_px`].
    OuterClampBelowMinimum {
        viewport: SafetyViewport,
        available_px: f32,
        required_px: f32,
    },
    /// The chrome alone consumes more pixels than the outer-height
    /// clamp — even a 0 px body would not fit. Strongest signal: drop
    /// a slot (title strip OR CTA row) before re-trying.
    ChromeExceedsOuterClamp {
        viewport: SafetyViewport,
        chrome_px: f32,
        outer_clamp_px: f32,
    },
}

/// Computed budget breakdown — returned by
/// [`modal_panel_content_budget`] on success.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalPanelContentBudget {
    pub viewport: SafetyViewport,
    /// Outer-rectangle height clamp at this viewport
    /// (`viewport.height_px * MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT /
    /// 100`).
    pub outer_height_px: f32,
    pub chrome_height_px: f32,
    /// Body / scroll region height = `outer_height_px - chrome_height_px`.
    /// Guaranteed `>= budget.min_body_px` on the `Ok` return.
    pub body_height_px: f32,
}

/// Computes the content-region height budget for a modal panel at the
/// supplied viewport.
///
/// Returns `Err` if the requested budget cannot fit — the panel is
/// guaranteed to clip its CTA row or its body off-screen if a surface
/// proceeds to spawn anyway. Callers MUST check the result rather than
/// shipping a panel whose contents do not fit at the
/// [`SAFETY_VIEWPORT_SMALLEST`] floor.
pub fn modal_panel_content_budget(
    viewport: SafetyViewport,
    budget: ModalPanelBudget,
) -> Result<ModalPanelContentBudget, ContentBudgetError> {
    let outer_height_px = viewport.height_f32() * (MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT / 100.0);
    let chrome_height_px = budget.chrome_height_px();
    if chrome_height_px > outer_height_px {
        return Err(ContentBudgetError::ChromeExceedsOuterClamp {
            viewport,
            chrome_px: chrome_height_px,
            outer_clamp_px: outer_height_px,
        });
    }
    let body_height_px = outer_height_px - chrome_height_px;
    if body_height_px < budget.min_body_px - f32::EPSILON {
        return Err(ContentBudgetError::OuterClampBelowMinimum {
            viewport,
            available_px: body_height_px,
            required_px: budget.min_body_px,
        });
    }
    Ok(ModalPanelContentBudget {
        viewport,
        outer_height_px,
        chrome_height_px,
        body_height_px,
    })
}

/// Asserts the supplied budget fits at every row of
/// [`super::viewport_matrix::SAFETY_VIEWPORT_MATRIX`]. Returns the first
/// failing row — by construction the smallest viewport is the
/// hardest, so a passing return guarantees fit at every larger row.
pub fn assert_fits_smallest_safety_viewport(
    budget: ModalPanelBudget,
) -> Result<ModalPanelContentBudget, ContentBudgetError> {
    modal_panel_content_budget(SAFETY_VIEWPORT_SMALLEST, budget)
}

/// Build the canonical centred-modal outer Node for the given kind.
/// Width is a percentage of the viewport, clamped to
/// [`ModalPanelKind::max_width_px`]; height is a percentage of the
/// viewport, clamped by [`MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT`].
/// The panel composes top-to-bottom as a flex column: title strip,
/// body / scroll region, CTA row.
pub fn modal_panel_node(kind: ModalPanelKind) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0 - MODAL_PANEL_DEFAULT_WIDTH_PERCENT / 2.0),
        top: Val::Percent(50.0 - MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT / 2.0),
        width: Val::Percent(MODAL_PANEL_DEFAULT_WIDTH_PERCENT),
        max_width: Val::Px(kind.max_width_px()),
        max_height: Val::Percent(MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Stretch,
        padding: UiRect::all(Val::Px(kind.padding_px())),
        border: UiRect::all(Val::Px(MODAL_PANEL_BORDER_WIDTH_PX)),
        row_gap: Val::Px(SPACING_MD),
        overflow: Overflow::clip(),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::design_tokens::viewport_matrix::SAFETY_VIEWPORT_MATRIX;

    const CANONICAL_CTA_ROW_PX: f32 = 44.0;

    #[test]
    fn standard_kind_width_clamps_match_published_constants() {
        assert_eq!(
            ModalPanelKind::Standard.max_width_px(),
            MODAL_PANEL_STANDARD_MAX_WIDTH_PX
        );
        assert_eq!(
            ModalPanelKind::Narrow.max_width_px(),
            MODAL_PANEL_NARROW_MAX_WIDTH_PX
        );
        assert!(ModalPanelKind::Standard.max_width_px() > ModalPanelKind::Narrow.max_width_px());
    }

    #[test]
    fn standard_kind_padding_is_spacing_lg_narrow_is_spacing_md() {
        assert_eq!(ModalPanelKind::Standard.padding_px(), SPACING_LG);
        assert_eq!(ModalPanelKind::Narrow.padding_px(), SPACING_MD);
    }

    #[test]
    fn chrome_height_sums_padding_border_title_cta_gaps() {
        let budget =
            ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CANONICAL_CTA_ROW_PX);
        let expected = 2.0 * MODAL_PANEL_STANDARD_PADDING_PX
            + 2.0 * MODAL_PANEL_BORDER_WIDTH_PX
            + MODAL_PANEL_TITLE_STRIP_HEIGHT_PX
            + CANONICAL_CTA_ROW_PX
            + 2.0 * SPACING_MD;
        assert!((budget.chrome_height_px() - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn min_outer_height_includes_min_body_floor() {
        let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Narrow, CANONICAL_CTA_ROW_PX);
        let outer = budget.min_outer_height_px();
        assert!(outer >= budget.chrome_height_px() + MODAL_PANEL_MIN_BODY_HEIGHT_PX - f32::EPSILON);
    }

    #[test]
    fn budget_returns_chrome_exceeds_clamp_when_required() {
        let mut huge =
            ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CANONICAL_CTA_ROW_PX);
        huge.title_strip_px = 1_000.0;
        huge.cta_row_px = 1_000.0;
        let err = modal_panel_content_budget(SAFETY_VIEWPORT_SMALLEST, huge)
            .expect_err("oversized chrome must fail the budget");
        match err {
            ContentBudgetError::ChromeExceedsOuterClamp {
                viewport,
                chrome_px,
                outer_clamp_px,
            } => {
                assert_eq!(viewport, SAFETY_VIEWPORT_SMALLEST);
                assert!(chrome_px > outer_clamp_px);
            }
            other => panic!("expected ChromeExceedsOuterClamp; got {other:?}"),
        }
    }

    #[test]
    fn budget_returns_outer_clamp_below_minimum_when_body_too_small() {
        let mut tight =
            ModalPanelBudget::with_defaults(ModalPanelKind::Narrow, CANONICAL_CTA_ROW_PX);
        // Demand a 600 px body — bigger than (720 px * 92%) - chrome.
        tight.min_body_px = 600.0;
        let err = modal_panel_content_budget(SAFETY_VIEWPORT_SMALLEST, tight)
            .expect_err("oversized min_body must fail the budget");
        match err {
            ContentBudgetError::OuterClampBelowMinimum {
                viewport,
                available_px,
                required_px,
            } => {
                assert_eq!(viewport, SAFETY_VIEWPORT_SMALLEST);
                assert!(required_px > available_px);
                assert!((required_px - 600.0).abs() < f32::EPSILON);
            }
            other => panic!("expected OuterClampBelowMinimum; got {other:?}"),
        }
    }

    #[test]
    fn default_standard_budget_fits_dev_floor_viewport() {
        // Regression guard: the friend-game default modal — Standard
        // padding, canonical CTA row, default title strip, default
        // min-body floor — fits the dev-floor 1280×720 viewport.
        let budget =
            ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CANONICAL_CTA_ROW_PX);
        let computed = assert_fits_smallest_safety_viewport(budget)
            .expect("default Standard modal must fit at 1280x720");
        assert!(computed.body_height_px >= MODAL_PANEL_MIN_BODY_HEIGHT_PX);
        // outer_height is 720 * 0.92 = 662.4 px.
        assert!((computed.outer_height_px - 662.4).abs() < 0.05);
    }

    #[test]
    fn default_narrow_budget_fits_dev_floor_viewport() {
        let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Narrow, CANONICAL_CTA_ROW_PX);
        let computed = assert_fits_smallest_safety_viewport(budget)
            .expect("default Narrow modal must fit at 1280x720");
        assert!(computed.body_height_px >= MODAL_PANEL_MIN_BODY_HEIGHT_PX);
    }

    #[test]
    fn fits_smallest_implies_fits_every_larger_row() {
        let budget =
            ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CANONICAL_CTA_ROW_PX);
        assert_fits_smallest_safety_viewport(budget).expect("ok");
        for v in SAFETY_VIEWPORT_MATRIX {
            modal_panel_content_budget(v, budget).expect("larger viewport must also fit");
        }
    }

    #[test]
    fn modal_panel_node_uses_flex_column_with_clip_overflow() {
        let node = modal_panel_node(ModalPanelKind::Standard);
        assert_eq!(node.display, Display::Flex);
        assert_eq!(node.flex_direction, FlexDirection::Column);
        // Overflow must clip so a runaway child does not paint past the
        // panel chrome (and past the viewport clamp).
        assert!(!node.overflow.is_visible());
    }

    #[test]
    fn modal_panel_node_width_and_height_are_percentage_clamped() {
        let node = modal_panel_node(ModalPanelKind::Standard);
        assert_eq!(node.width, Val::Percent(MODAL_PANEL_DEFAULT_WIDTH_PERCENT));
        assert_eq!(
            node.max_height,
            Val::Percent(MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT)
        );
        // max_width is the kind's pixel-fixed cap.
        assert_eq!(node.max_width, Val::Px(MODAL_PANEL_STANDARD_MAX_WIDTH_PX));
    }

    #[test]
    fn modal_panel_node_is_centered_via_anchor_math() {
        let node = modal_panel_node(ModalPanelKind::Standard);
        assert_eq!(node.position_type, PositionType::Absolute);
        // 50% - half-width-percent: 50 - 44 = 6 percent.
        assert_eq!(
            node.left,
            Val::Percent(50.0 - MODAL_PANEL_DEFAULT_WIDTH_PERCENT / 2.0)
        );
        assert_eq!(
            node.top,
            Val::Percent(50.0 - MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT / 2.0)
        );
    }
}
