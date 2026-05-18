//! Sprint 17 PROMPT 1181 — UI layout-foundation primitive *shape* unit
//! test (`S17-UI-LAYOUT-FOUNDATION-PRIMITIVES-REPAIR`).
//!
//! Asserts that the published primitives expose the constants surface
//! that downstream surface workers depend on. The richer
//! viewport-safety invariants live in the integration test
//! (`tests/integration/ui_clean_pass/layout_primitives_test.rs`); this
//! unit test is the cheap "module exports survive refactor" guard.

use client::ui::design_tokens::cta_row::{
    CtaRowKind, CTA_BUTTON_MIN_HEIGHT_PX, CTA_BUTTON_MIN_WIDTH_PX, CTA_ROW_COMPACT_HEIGHT_PX,
    CTA_ROW_FLEX_GROW, CTA_ROW_FLEX_SHRINK, CTA_ROW_HEIGHT_PX,
};
use client::ui::design_tokens::modal_panel::{
    ModalPanelBudget, ModalPanelKind, MODAL_PANEL_BORDER_WIDTH_PX,
    MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT, MODAL_PANEL_DEFAULT_WIDTH_PERCENT,
    MODAL_PANEL_MIN_BODY_HEIGHT_PX, MODAL_PANEL_NARROW_MAX_WIDTH_PX, MODAL_PANEL_NARROW_PADDING_PX,
    MODAL_PANEL_STANDARD_MAX_WIDTH_PX, MODAL_PANEL_STANDARD_PADDING_PX,
    MODAL_PANEL_TITLE_STRIP_HEIGHT_PX,
};
use client::ui::design_tokens::scroll_region::{
    SCROLL_REGION_FLEX_GROW, SCROLL_REGION_FLEX_SHRINK, SCROLL_REGION_MIN_HEIGHT_PX,
};
use client::ui::design_tokens::spacing::{SPACING_LG, SPACING_MD};
use client::ui::design_tokens::status_chip::{
    VisualRole, STATUS_CHIP_HEIGHT_PX, STATUS_CHIP_PADDING_X_PX, STATUS_CHIP_PADDING_Y_PX,
    STATUS_CHIP_TEXT_SIZE_PX,
};
use client::ui::design_tokens::text_fit::TextFitPolicy;
use client::ui::design_tokens::viewport_matrix::{
    SAFETY_VIEWPORT_DEV_FLOOR, SAFETY_VIEWPORT_HD_BASELINE, SAFETY_VIEWPORT_MATRIX,
    SAFETY_VIEWPORT_PROD_MIN, SAFETY_VIEWPORT_SMALLEST,
};

#[test]
fn viewport_matrix_constants_match_published_widths_and_heights() {
    assert_eq!(SAFETY_VIEWPORT_DEV_FLOOR.width_px, 1280);
    assert_eq!(SAFETY_VIEWPORT_DEV_FLOOR.height_px, 720);
    assert_eq!(SAFETY_VIEWPORT_PROD_MIN.width_px, 1366);
    assert_eq!(SAFETY_VIEWPORT_PROD_MIN.height_px, 768);
    assert_eq!(SAFETY_VIEWPORT_HD_BASELINE.width_px, 1920);
    assert_eq!(SAFETY_VIEWPORT_HD_BASELINE.height_px, 1080);
    assert_eq!(SAFETY_VIEWPORT_MATRIX.len(), 3);
    assert_eq!(SAFETY_VIEWPORT_SMALLEST, SAFETY_VIEWPORT_DEV_FLOOR);
}

#[test]
fn modal_panel_constants_resolve_to_spec_ratified_values() {
    assert_eq!(MODAL_PANEL_DEFAULT_WIDTH_PERCENT, 88.0);
    assert_eq!(MODAL_PANEL_DEFAULT_MAX_HEIGHT_PERCENT, 92.0);
    assert_eq!(MODAL_PANEL_STANDARD_MAX_WIDTH_PX, 860.0);
    assert_eq!(MODAL_PANEL_NARROW_MAX_WIDTH_PX, 520.0);
    assert_eq!(MODAL_PANEL_STANDARD_PADDING_PX, SPACING_LG);
    assert_eq!(MODAL_PANEL_NARROW_PADDING_PX, SPACING_MD);
    assert!(MODAL_PANEL_TITLE_STRIP_HEIGHT_PX > 0.0);
    assert!(MODAL_PANEL_BORDER_WIDTH_PX > 0.0);
    assert!(MODAL_PANEL_MIN_BODY_HEIGHT_PX > 0.0);
}

#[test]
fn modal_panel_kind_dispatches_to_expected_constants() {
    assert_eq!(
        ModalPanelKind::Standard.max_width_px(),
        MODAL_PANEL_STANDARD_MAX_WIDTH_PX
    );
    assert_eq!(
        ModalPanelKind::Narrow.max_width_px(),
        MODAL_PANEL_NARROW_MAX_WIDTH_PX
    );
    assert_eq!(
        ModalPanelKind::Standard.padding_px(),
        MODAL_PANEL_STANDARD_PADDING_PX
    );
    assert_eq!(
        ModalPanelKind::Narrow.padding_px(),
        MODAL_PANEL_NARROW_PADDING_PX
    );
}

#[test]
fn modal_panel_budget_with_defaults_uses_published_defaults() {
    let budget = ModalPanelBudget::with_defaults(ModalPanelKind::Standard, CTA_ROW_HEIGHT_PX);
    assert_eq!(budget.kind, ModalPanelKind::Standard);
    assert_eq!(budget.title_strip_px, MODAL_PANEL_TITLE_STRIP_HEIGHT_PX);
    assert_eq!(budget.cta_row_px, CTA_ROW_HEIGHT_PX);
    assert_eq!(budget.section_gap_px, SPACING_MD);
    assert_eq!(budget.min_body_px, MODAL_PANEL_MIN_BODY_HEIGHT_PX);
}

#[test]
fn cta_row_constants_resolve_to_friend_game_floors() {
    assert!(CTA_ROW_HEIGHT_PX >= 44.0);
    assert!(CTA_ROW_COMPACT_HEIGHT_PX < CTA_ROW_HEIGHT_PX);
    assert_eq!(CtaRowKind::Primary.height_px(), CTA_ROW_HEIGHT_PX);
    assert_eq!(CtaRowKind::Compact.height_px(), CTA_ROW_COMPACT_HEIGHT_PX);
    assert_eq!(CTA_ROW_FLEX_GROW, 0.0);
    assert_eq!(CTA_ROW_FLEX_SHRINK, 0.0);
    assert!(CTA_BUTTON_MIN_WIDTH_PX > 0.0);
    assert_eq!(CTA_BUTTON_MIN_HEIGHT_PX, CTA_ROW_HEIGHT_PX);
}

#[test]
fn scroll_region_constants_protect_cta_row_via_zero_min_height() {
    assert_eq!(SCROLL_REGION_FLEX_GROW, 1.0);
    assert_eq!(SCROLL_REGION_FLEX_SHRINK, 1.0);
    assert_eq!(SCROLL_REGION_MIN_HEIGHT_PX, 0.0);
}

#[test]
fn status_chip_constants_separate_chip_from_button() {
    assert!(STATUS_CHIP_HEIGHT_PX < CTA_ROW_HEIGHT_PX);
    assert!(STATUS_CHIP_TEXT_SIZE_PX > 0.0);
    assert!(STATUS_CHIP_PADDING_X_PX > 0.0);
    assert!(STATUS_CHIP_PADDING_Y_PX > 0.0);
    assert!(VisualRole::StatusChip.is_read_only());
    assert!(VisualRole::CtaButton.is_interactive());
}

#[test]
fn text_fit_policies_classify_wrap_vs_no_wrap_correctly() {
    assert!(!TextFitPolicy::SingleLineNoWrap.allows_soft_wrap());
    assert!(TextFitPolicy::WrapWordBoundary.allows_soft_wrap());
    assert!(TextFitPolicy::WrapWordOrCharacter.allows_soft_wrap());
}
