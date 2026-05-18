//! Sprint 17 PROMPT 1181 — UI layout-foundation primitives integration
//! test (`S17-UI-LAYOUT-FOUNDATION-PRIMITIVES-REPAIR`).
//!
//! Asserts the cross-cutting **viewport-safety** contract: every modal
//! panel surface that declares its CTA row, title strip, and section
//! gaps MUST compute a positive body / scroll-region height at the
//! smallest row of [`viewport_matrix::SAFETY_VIEWPORT_MATRIX`]
//! (`1280×720`). A surface that claims viewport safety without
//! accounting for content height, row gaps, padding, AND CTA height
//! will fail one of the regressions below.
//!
//! These assertions are deliberately authored at the primitive level —
//! per-surface migration is owned by Sprint 17+ follow-on rows
//! (`S17-UI-MODAL-PANEL-CHROME-MIGRATION-*`). The foundation lane
//! ships the primitive + invariant tests so later workers cannot
//! introduce a regression silently.

use bevy::ui::Val;
use client::ui::design_tokens::cta_row::{
    cta_button_node, cta_row_node, CtaRowKind, CTA_BUTTON_MIN_HEIGHT_PX, CTA_BUTTON_MIN_WIDTH_PX,
    CTA_ROW_FLEX_GROW, CTA_ROW_FLEX_SHRINK, CTA_ROW_HEIGHT_PX,
};
use client::ui::design_tokens::modal_panel::{
    assert_fits_smallest_safety_viewport, modal_panel_content_budget, modal_panel_node,
    ContentBudgetError, ModalPanelBudget, ModalPanelKind, MODAL_PANEL_BORDER_WIDTH_PX,
    MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT, MODAL_PANEL_DEFAULT_WIDTH_PERCENT,
    MODAL_PANEL_MIN_BODY_HEIGHT_PX, MODAL_PANEL_NARROW_MAX_WIDTH_PX,
    MODAL_PANEL_STANDARD_MAX_WIDTH_PX, MODAL_PANEL_TITLE_STRIP_HEIGHT_PX,
};
use client::ui::design_tokens::scroll_region::{
    clipped_body_region_node, scroll_region_node, SCROLL_REGION_FLEX_GROW,
    SCROLL_REGION_FLEX_SHRINK, SCROLL_REGION_MIN_HEIGHT_PX,
};
use client::ui::design_tokens::spacing::{SPACING_LG, SPACING_MD};
use client::ui::design_tokens::status_chip::{
    status_chip_node, VisualRole, STATUS_CHIP_HEIGHT_PX, STATUS_CHIP_TEXT_SIZE_PX,
};
use client::ui::design_tokens::text_fit::{
    single_line_centered, text_layout, wrap_body_left, TextFitPolicy,
};
use client::ui::design_tokens::typography::CAPTION;
use client::ui::design_tokens::viewport_matrix::{
    SAFETY_VIEWPORT_DEV_FLOOR, SAFETY_VIEWPORT_HD_BASELINE, SAFETY_VIEWPORT_MATRIX,
    SAFETY_VIEWPORT_PROD_MIN, SAFETY_VIEWPORT_SMALLEST,
};

// ---------------------------------------------------------------------
// viewport_matrix — cross-references the production constants
// ---------------------------------------------------------------------

#[test]
fn ac_viewport_matrix_smallest_alias_points_to_first_row() {
    assert_eq!(SAFETY_VIEWPORT_SMALLEST, SAFETY_VIEWPORT_MATRIX[0]);
    assert_eq!(SAFETY_VIEWPORT_SMALLEST, SAFETY_VIEWPORT_DEV_FLOOR);
}

#[test]
fn ac_viewport_matrix_contains_dev_floor_prod_min_hd_baseline_in_order() {
    assert_eq!(SAFETY_VIEWPORT_MATRIX.len(), 3);
    assert_eq!(SAFETY_VIEWPORT_MATRIX[0], SAFETY_VIEWPORT_DEV_FLOOR);
    assert_eq!(SAFETY_VIEWPORT_MATRIX[1], SAFETY_VIEWPORT_PROD_MIN);
    assert_eq!(SAFETY_VIEWPORT_MATRIX[2], SAFETY_VIEWPORT_HD_BASELINE);
}

// ---------------------------------------------------------------------
// modal_panel — the core viewport-safety invariant
// ---------------------------------------------------------------------

#[test]
fn ac_default_standard_modal_fits_smallest_safety_viewport() {
    // A Standard modal with default title strip + canonical CTA row +
    // default min-body floor must fit at the dev-floor 1280×720
    // viewport. If a future spec revision lowers
    // MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT or raises chrome, this
    // assertion catches the regression.
    let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CTA_ROW_HEIGHT_PX);
    let computed =
        assert_fits_smallest_safety_viewport(budget).expect("default Standard modal must fit");
    assert!(computed.body_height_px >= MODAL_PANEL_MIN_BODY_HEIGHT_PX);
}

#[test]
fn ac_default_narrow_modal_fits_smallest_safety_viewport() {
    let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Narrow, CTA_ROW_HEIGHT_PX);
    let computed =
        assert_fits_smallest_safety_viewport(budget).expect("default Narrow modal must fit");
    assert!(computed.body_height_px >= MODAL_PANEL_MIN_BODY_HEIGHT_PX);
}

#[test]
fn ac_modal_panel_fails_when_oversized_cta_row_consumes_body_budget() {
    // The viewport-safety contract: a surface that claims fit must
    // account for its CTA-row height. Pass a CTA row taller than the
    // available outer-height clamp and assert the primitive surfaces
    // the failure rather than silently clipping.
    let oversized_cta_px = 800.0;
    let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Narrow, oversized_cta_px);
    let err =
        assert_fits_smallest_safety_viewport(budget).expect_err("oversized CTA must fail budget");
    match err {
        ContentBudgetError::ChromeExceedsOuterClamp { chrome_px, .. } => {
            assert!(chrome_px >= oversized_cta_px);
        }
        ContentBudgetError::OuterClampBelowMinimum { .. } => {}
    }
}

#[test]
fn ac_modal_panel_fails_when_oversized_title_strip_consumes_body_budget() {
    let mut budget = ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CTA_ROW_HEIGHT_PX);
    budget.title_strip_px = 700.0;
    let err = assert_fits_smallest_safety_viewport(budget)
        .expect_err("oversized title strip must fail budget");
    match err {
        ContentBudgetError::ChromeExceedsOuterClamp { .. }
        | ContentBudgetError::OuterClampBelowMinimum { .. } => {}
    }
}

#[test]
fn ac_modal_panel_fails_when_oversized_section_gaps_consume_body_budget() {
    let mut budget = ModalPanelBudget::with_defaults(ModalPanelKind::Narrow, CTA_ROW_HEIGHT_PX);
    // Two gaps × 500 px = 1000 px chrome contribution alone — bigger
    // than 720 × 92% = 662.4 px outer clamp.
    budget.section_gap_px = 500.0;
    let err = assert_fits_smallest_safety_viewport(budget)
        .expect_err("oversized section gaps must fail budget");
    match err {
        ContentBudgetError::ChromeExceedsOuterClamp {
            chrome_px,
            outer_clamp_px,
            ..
        } => {
            assert!(chrome_px > outer_clamp_px);
        }
        ContentBudgetError::OuterClampBelowMinimum { .. } => {}
    }
}

#[test]
fn ac_modal_panel_fails_when_min_body_floor_exceeds_outer_clamp() {
    let mut budget = ModalPanelBudget::with_defaults(ModalPanelKind::Narrow, CTA_ROW_HEIGHT_PX);
    // Demand a 700 px min-body — bigger than 720 × 92% - chrome.
    budget.min_body_px = 700.0;
    let err = assert_fits_smallest_safety_viewport(budget)
        .expect_err("oversized min_body must fail budget");
    match err {
        ContentBudgetError::OuterClampBelowMinimum {
            available_px,
            required_px,
            ..
        } => {
            assert!(required_px > available_px);
        }
        ContentBudgetError::ChromeExceedsOuterClamp { .. } => {
            // Equally acceptable — chrome dominated; we still failed.
        }
    }
}

#[test]
fn ac_fits_smallest_implies_fits_every_larger_safety_viewport_row() {
    let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CTA_ROW_HEIGHT_PX);
    assert_fits_smallest_safety_viewport(budget).expect("smallest fits");
    for v in SAFETY_VIEWPORT_MATRIX {
        modal_panel_content_budget(v, budget)
            .unwrap_or_else(|err| panic!("viewport {} must fit but failed: {err:?}", v.name));
    }
}

#[test]
fn ac_modal_panel_node_uses_percent_anchor_and_clip_overflow() {
    let node = modal_panel_node(ModalPanelKind::Standard);
    assert_eq!(node.width, Val::Percent(MODAL_PANEL_DEFAULT_WIDTH_PERCENT));
    assert_eq!(
        node.max_height,
        Val::Percent(MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT)
    );
    assert_eq!(node.max_width, Val::Px(MODAL_PANEL_STANDARD_MAX_WIDTH_PX));
    assert!(!node.overflow.is_visible());
}

#[test]
fn ac_modal_panel_kind_max_widths_differ_so_narrow_is_narrower() {
    assert_eq!(
        ModalPanelKind::Narrow.max_width_px(),
        MODAL_PANEL_NARROW_MAX_WIDTH_PX
    );
    assert!(MODAL_PANEL_STANDARD_MAX_WIDTH_PX > MODAL_PANEL_NARROW_MAX_WIDTH_PX);
}

#[test]
fn ac_modal_panel_chrome_uses_spacing_tokens_not_magic_literals() {
    // PROMPT 1035 §"Hardcoded-value findings" surfaced three separate
    // panel-chrome row_gap / padding literals. The primitive must read
    // from SPACING_LG / SPACING_MD to consolidate them.
    let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CTA_ROW_HEIGHT_PX);
    let expected_padding_x2 = 2.0 * SPACING_LG;
    let expected_section_gaps_x2 = 2.0 * SPACING_MD;
    let chrome = budget.chrome_height_px();
    assert!(chrome > expected_padding_x2);
    assert!(chrome > expected_section_gaps_x2);
    assert!(MODAL_PANEL_TITLE_STRIP_HEIGHT_PX > 0.0);
    assert!(MODAL_PANEL_BORDER_WIDTH_PX > 0.0);
}

// ---------------------------------------------------------------------
// cta_row — stable rows that cannot be squashed off-screen
// ---------------------------------------------------------------------

#[test]
fn ac_cta_row_has_flex_grow_and_shrink_zero_so_it_cannot_be_squashed() {
    for kind in [CtaRowKind::Primary, CtaRowKind::Compact] {
        let node = cta_row_node(kind);
        assert_eq!(node.flex_grow, CTA_ROW_FLEX_GROW);
        assert_eq!(node.flex_shrink, CTA_ROW_FLEX_SHRINK);
        assert_eq!(node.flex_grow, 0.0);
        assert_eq!(node.flex_shrink, 0.0);
    }
}

#[test]
fn ac_cta_row_height_is_pixel_fixed_never_percent() {
    let node = cta_row_node(CtaRowKind::Primary);
    match node.height {
        Val::Px(_) => {}
        other => panic!("CTA row height must be Val::Px(_); got {other:?}"),
    }
}

#[test]
fn ac_cta_button_min_width_and_min_height_are_pixel_fixed_floors() {
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

// ---------------------------------------------------------------------
// scroll_region — body region grows + scrolls + protects CTA row
// ---------------------------------------------------------------------

#[test]
fn ac_scroll_region_grows_shrinks_and_carries_zero_min_height() {
    let node = scroll_region_node();
    assert_eq!(node.flex_grow, SCROLL_REGION_FLEX_GROW);
    assert_eq!(node.flex_shrink, SCROLL_REGION_FLEX_SHRINK);
    assert!(node.flex_grow > 0.0);
    assert!(node.flex_shrink > 0.0);
    match node.min_height {
        Val::Px(px) => assert!((px - SCROLL_REGION_MIN_HEIGHT_PX).abs() < f32::EPSILON),
        other => panic!("scroll region min_height must be Val::Px(0.0); got {other:?}"),
    }
}

#[test]
fn ac_clipped_body_region_shares_flex_contract_with_scroll_region() {
    let scroll = scroll_region_node();
    let clipped = clipped_body_region_node();
    assert_eq!(scroll.flex_grow, clipped.flex_grow);
    assert_eq!(scroll.flex_shrink, clipped.flex_shrink);
    assert_eq!(scroll.min_height, clipped.min_height);
}

// ---------------------------------------------------------------------
// status_chip — read-only vs interactive role boundary
// ---------------------------------------------------------------------

#[test]
fn ac_status_chip_is_visually_distinct_from_cta_row() {
    assert!(STATUS_CHIP_HEIGHT_PX < CTA_ROW_HEIGHT_PX);
    assert_eq!(STATUS_CHIP_TEXT_SIZE_PX, CAPTION);
}

#[test]
fn ac_status_chip_role_is_read_only_cta_button_role_is_interactive() {
    assert!(VisualRole::StatusChip.is_read_only());
    assert!(!VisualRole::StatusChip.is_interactive());
    assert!(VisualRole::CtaButton.is_interactive());
    assert!(!VisualRole::CtaButton.is_read_only());
}

#[test]
fn ac_status_chip_node_has_pixel_fixed_height() {
    let node = status_chip_node();
    match node.height {
        Val::Px(px) => assert!((px - STATUS_CHIP_HEIGHT_PX).abs() < f32::EPSILON),
        other => panic!("status chip height must be Val::Px(_); got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// text_fit — single-line vs wrap policy
// ---------------------------------------------------------------------

#[test]
fn ac_text_fit_policies_resolve_to_distinct_line_break_modes() {
    let no_wrap = TextFitPolicy::SingleLineNoWrap.line_break();
    let word_boundary = TextFitPolicy::WrapWordBoundary.line_break();
    let word_or_char = TextFitPolicy::WrapWordOrCharacter.line_break();
    assert_ne!(no_wrap, word_boundary);
    assert_ne!(no_wrap, word_or_char);
    assert_ne!(word_boundary, word_or_char);
}

#[test]
fn ac_text_fit_single_line_does_not_allow_soft_wrap() {
    assert!(!TextFitPolicy::SingleLineNoWrap.allows_soft_wrap());
    assert!(TextFitPolicy::WrapWordBoundary.allows_soft_wrap());
    assert!(TextFitPolicy::WrapWordOrCharacter.allows_soft_wrap());
}

#[test]
fn ac_text_layout_factories_pair_line_break_with_default_left_justify() {
    use bevy::text::Justify;
    let layout = text_layout(TextFitPolicy::WrapWordBoundary);
    assert_eq!(layout.justify, Justify::Left);
    let centered = single_line_centered();
    assert_eq!(centered.justify, Justify::Center);
    let wrap = wrap_body_left();
    assert_eq!(wrap.justify, Justify::Left);
}
