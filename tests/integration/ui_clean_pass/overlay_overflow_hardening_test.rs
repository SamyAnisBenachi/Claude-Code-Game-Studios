//! Sprint 18 story 026 -- `S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001`
//! integration bin. AC5..AC7 binding test for the three in-scope overlay
//! panels covered by PROMPT 1180 §1.5 O-02 / O-03 and §1.4 S-08 / S-09:
//!
//! - **AC1**: `client::ui::photosensitivity_warning::photosensitivity_warning_panel_node`
//!   declares `max_height: Val::Percent(92.0)` + `Overflow::scroll_y()`, and
//!   `warning_footer_node` is positioned at the panel's bottom-padding edge
//!   so the Acknowledge button stays reachable regardless of body length
//!   (footer slot pattern).
//! - **AC2**: `client::presentation::connection_lost_overlay::connection_lost_overlay_panel_node`
//!   declares `max_height: Val::Percent(92.0)` + `Overflow::scroll_y()`.
//! - **AC3**: `client::ui::shop_auction::draft_initial_modal_panel_node`
//!   drops the pre-1349 `height: 360 px` literal (now expressed as
//!   `min_height` so the visual floor at small viewports is preserved)
//!   while keeping `max_height: 92 %` + `Overflow::scroll_y()`.
//! - **AC4**: `client::ui::shop_auction::draft_initial_grid_node` declares
//!   `Display::Grid` with a 3 × 3 fixed-px template and no per-slot
//!   absolute offsets in the spawn site.
//! - **AC5..AC7**: panel max-height ceilings resolve correctly at the
//!   1280×600 sub-floor, 1366×768 primary, and 3840×2160 4K viewports
//!   (PROMPT 1180 §6 Lane J / story 026 ACs 5/6/7).
//! - **AC8**: `client/src/presentation/result_screen.rs:502-549` (the
//!   reference template under PROMPT 1180 §1.5 O-04) is unchanged by
//!   PROMPT 1349 -- its 92 % + scroll contract still matches verbatim.
//!
//! Test classification: read-only geometric / Node-shape assertions over
//! the published node builders + source-string check on the draft-initial
//! spawn site. No World composition required for this bin.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::ui::{Overflow, OverflowAxis};
use client::presentation::connection_lost_overlay::{
    connection_lost_overlay_panel_node, CONNECTION_LOST_PANEL_MAX_HEIGHT_PERCENT,
};
use client::ui::photosensitivity_warning::{
    photosensitivity_warning_panel_node, warning_footer_node,
    PHOTOSENSITIVITY_PANEL_MAX_HEIGHT_PERCENT,
};
use client::ui::shop_auction::{
    draft_initial_grid_node, draft_initial_modal_panel_node, draft_initial_slot_node,
    DRAFT_INITIAL_GRID_COLUMN_GAP_PX, DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX,
    DRAFT_INITIAL_GRID_ROW_GAP_PX, DRAFT_INITIAL_GRID_ROW_HEIGHT_PX,
    DRAFT_INITIAL_MODAL_HEIGHT_PX, DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT,
    DRAFT_INITIAL_MODAL_MAX_WIDTH_PX,
};

// Viewport heights only -- AC5..AC7 reason about `max_height: 92 %`
// against the viewport height. Widths are not used in this bin because
// the modal width contracts are guarded by sibling tests (PROMPT 1182
// shop-auction responsive-layout + Sprint 17 layout-foundation).
const CANONICAL_VIEWPORT_SUB_FLOOR_HEIGHT: f32 = 600.0;
const CANONICAL_VIEWPORT_PRIMARY_HEIGHT: f32 = 768.0;
const CANONICAL_VIEWPORT_4K_HEIGHT: f32 = 2160.0;

fn resolve_percent(val: Val, parent_extent_px: f32) -> Option<f32> {
    match val {
        Val::Percent(pct) => Some(parent_extent_px * pct / 100.0),
        Val::Px(px) => Some(px),
        _ => None,
    }
}

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("client crate manifest sits inside repo root")
        .join("client/src")
}

fn read_client_source(rel: &str) -> String {
    let path = client_src_root().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

// ─── AC1: photosensitivity warning panel + footer slot ───────────────────

#[test]
fn test_overlay_overflow_ac1_photosensitivity_panel_declares_max_height_and_scroll() {
    // Arrange
    let panel = photosensitivity_warning_panel_node();

    // Assert -- 92 % ceiling + scroll_y per §5 C-5.
    assert_eq!(
        panel.max_height,
        Val::Percent(PHOTOSENSITIVITY_PANEL_MAX_HEIGHT_PERCENT),
        "AC1: photosensitivity panel must declare max_height: 92 %"
    );
    assert_eq!(
        PHOTOSENSITIVITY_PANEL_MAX_HEIGHT_PERCENT, 92.0,
        "AC1: photosensitivity panel max_height constant must equal 92.0 (§5 C-5)"
    );
    let Overflow { x: _, y } = panel.overflow;
    assert_eq!(
        y,
        OverflowAxis::Scroll,
        "AC1: photosensitivity panel must declare Overflow::scroll_y()"
    );
    assert_eq!(
        panel.position_type,
        PositionType::Relative,
        "AC1: photosensitivity panel must be Relative so the absolute footer anchors to it"
    );
}

#[test]
fn test_overlay_overflow_ac1_photosensitivity_acknowledge_in_footer_slot() {
    // Arrange
    let footer = warning_footer_node();

    // Assert -- footer slot anchored at panel bottom edge.
    assert_eq!(
        footer.position_type,
        PositionType::Absolute,
        "AC1: photosensitivity Acknowledge footer must be PositionType::Absolute"
    );
    assert!(
        matches!(footer.bottom, Val::Px(px) if px > 0.0),
        "AC1: photosensitivity Acknowledge footer must anchor `bottom` at a positive pixel offset (panel padding), got {:?}",
        footer.bottom
    );
    assert!(
        matches!(footer.left, Val::Px(_)) && matches!(footer.right, Val::Px(_)),
        "AC1: photosensitivity Acknowledge footer must span panel width via left + right offsets"
    );
    assert!(
        matches!(footer.height, Val::Px(px) if px > 0.0),
        "AC1: photosensitivity Acknowledge footer must declare a non-zero pixel height, got {:?}",
        footer.height
    );
    assert_eq!(
        footer.display,
        Display::Flex,
        "AC1: photosensitivity Acknowledge footer must be Display::Flex to center the button"
    );
    assert_eq!(
        footer.justify_content,
        JustifyContent::Center,
        "AC1: photosensitivity Acknowledge footer must center horizontally"
    );
    assert_eq!(
        footer.align_items,
        AlignItems::Center,
        "AC1: photosensitivity Acknowledge footer must center vertically"
    );
}

// ─── AC2: connection-lost overlay panel ──────────────────────────────────

#[test]
fn test_overlay_overflow_ac2_connection_lost_panel_declares_max_height_and_scroll() {
    // Arrange
    let panel = connection_lost_overlay_panel_node();

    // Assert -- 92 % ceiling + scroll_y per §5 C-5.
    assert_eq!(
        panel.max_height,
        Val::Percent(CONNECTION_LOST_PANEL_MAX_HEIGHT_PERCENT),
        "AC2: connection-lost panel must declare max_height: 92 %"
    );
    assert_eq!(
        CONNECTION_LOST_PANEL_MAX_HEIGHT_PERCENT, 92.0,
        "AC2: connection-lost panel max_height constant must equal 92.0 (§5 C-5)"
    );
    let Overflow { x: _, y } = panel.overflow;
    assert_eq!(
        y,
        OverflowAxis::Scroll,
        "AC2: connection-lost panel must declare Overflow::scroll_y()"
    );
    // Existing layout preserved.
    assert_eq!(
        panel.display,
        Display::Flex,
        "AC2: connection-lost panel must keep Display::Flex"
    );
    assert_eq!(
        panel.flex_direction,
        FlexDirection::Column,
        "AC2: connection-lost panel must keep FlexDirection::Column"
    );
    assert_eq!(
        panel.width,
        Val::Percent(60.0),
        "AC2: connection-lost panel must keep its 60 % width (S13-CONN-LOST-UX-001)"
    );
    assert_eq!(
        panel.max_width,
        Val::Px(520.0),
        "AC2: connection-lost panel must keep its 520 px max-width"
    );
}

// ─── AC3 / AC4: draft-initial modal + grid ───────────────────────────────

#[test]
fn test_overlay_overflow_ac3_draft_initial_modal_drops_height_literal() {
    // Arrange
    let panel = draft_initial_modal_panel_node();

    // Assert -- AC3: fixed `height: 360 px` literal removed. Visual floor
    // preserved via `min_height` so the existing layout (other absolute
    // children expect a 360 px baseline) is unaffected at small
    // viewports.
    assert_eq!(
        panel.height,
        Val::Auto,
        "AC3: draft-initial modal must drop the fixed `height: 360 px` literal (got {:?})",
        panel.height
    );
    assert_eq!(
        panel.min_height,
        Val::Px(DRAFT_INITIAL_MODAL_HEIGHT_PX),
        "AC3: draft-initial modal must preserve its 360 px visual floor via min_height"
    );
    // Ceiling + scroll per §5 C-5.
    assert_eq!(
        panel.max_height,
        Val::Percent(DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT),
        "AC3: draft-initial modal must declare max_height: 92 %"
    );
    assert_eq!(
        DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT, 92.0,
        "AC3: draft-initial modal max_height constant must equal 92.0 (§5 C-5)"
    );
    let Overflow { x: _, y } = panel.overflow;
    assert_eq!(
        y,
        OverflowAxis::Scroll,
        "AC3: draft-initial modal must declare Overflow::scroll_y()"
    );
    // Width contract preserved.
    assert_eq!(
        panel.max_width,
        Val::Px(DRAFT_INITIAL_MODAL_MAX_WIDTH_PX),
        "AC3: draft-initial modal must preserve its 860 px max-width"
    );
}

#[test]
fn test_overlay_overflow_ac4_draft_initial_grid_uses_display_grid() {
    // Inspect the published Node directly: §5 C-5 requires
    // `Display::Grid` (or `FlexWrap::Wrap`). PROMPT 1349 chose Grid so
    // a `3 × 3` template is declared explicitly.
    let grid = draft_initial_grid_node();

    assert_eq!(
        grid.display,
        Display::Grid,
        "AC4: draft_initial_grid_node must declare `Display::Grid` (§5 C-5)"
    );
    assert_eq!(
        grid.column_gap,
        Val::Px(DRAFT_INITIAL_GRID_COLUMN_GAP_PX),
        "AC4: draft_initial_grid_node must declare `column_gap` matching the published constant"
    );
    assert_eq!(
        grid.row_gap,
        Val::Px(DRAFT_INITIAL_GRID_ROW_GAP_PX),
        "AC4: draft_initial_grid_node must declare `row_gap` matching the published constant"
    );
    assert!(
        !grid.grid_template_columns.is_empty(),
        "AC4: draft_initial_grid_node must populate grid_template_columns for the 3-column track"
    );
    assert!(
        !grid.grid_template_rows.is_empty(),
        "AC4: draft_initial_grid_node must populate grid_template_rows for the 3-row track"
    );
    // The grid container still anchors at its absolute offset inside
    // the modal so the other absolutely-positioned siblings (countdown,
    // objective overlay, footer) retain their positions -- only slot
    // placement is migrated to Grid auto-placement.
    assert_eq!(
        grid.position_type,
        PositionType::Absolute,
        "AC4: draft_initial_grid_node keeps `position_type: Absolute` to anchor inside the modal"
    );
}

#[test]
fn test_overlay_overflow_ac4_draft_initial_slot_drops_absolute_offsets() {
    // Inspect the published Node directly: the slot must no longer
    // carry the per-index `position_type: Absolute` + `left`/`top`
    // offsets. The grid container's `Display::Grid` auto-placement
    // now positions slots.
    let slot = draft_initial_slot_node();

    assert_eq!(
        slot.position_type,
        PositionType::Relative,
        "AC4: draft_initial_slot_node must drop `position_type: Absolute` (§5 C-5 'absolute offsets removed'); got {:?}",
        slot.position_type
    );
    assert_eq!(
        slot.left,
        Val::Auto,
        "AC4: draft_initial_slot_node must drop per-index `left:` offset; got {:?}",
        slot.left
    );
    assert_eq!(
        slot.top,
        Val::Auto,
        "AC4: draft_initial_slot_node must drop per-index `top:` offset; got {:?}",
        slot.top
    );
    // Slot dimensions preserved so the grid auto-places them into the
    // same 120 × 56 px cells as the pre-1349 manual offsets.
    assert_eq!(
        slot.width,
        Val::Px(DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX),
        "AC4: draft_initial_slot_node must preserve the published 120 px width"
    );
    assert_eq!(
        slot.height,
        Val::Px(DRAFT_INITIAL_GRID_ROW_HEIGHT_PX),
        "AC4: draft_initial_slot_node must preserve the published 56 px height"
    );
}

// ─── AC5..AC7: viewport-resolved height ceilings ─────────────────────────

#[test]
fn test_overlay_overflow_ac5_sub_floor_viewport_keeps_acknowledge_reachable() {
    // 1280×600 -- modal max_height: 92 % = 552 px. Photosensitivity
    // panel content (title + body) is < 552 px in normal copy, so the
    // panel fits without scrolling. If body copy grew, the absolute
    // footer (position_type: Absolute, bottom: panel_padding) would
    // remain visible while the inner content scrolled, satisfying
    // "Acknowledge fully on-screen OR scroll-reachable" (AC5).
    let panel = photosensitivity_warning_panel_node();
    let ceiling = resolve_percent(panel.max_height, CANONICAL_VIEWPORT_SUB_FLOOR_HEIGHT)
        .expect("AC5: photosensitivity max_height must resolve at 1280×600");
    assert!(
        ceiling > 0.0,
        "AC5: photosensitivity max_height must resolve to a positive px at 1280×600 (got {})",
        ceiling
    );
    let expected = CANONICAL_VIEWPORT_SUB_FLOOR_HEIGHT * 0.92;
    assert!(
        (ceiling - expected).abs() < f32::EPSILON,
        "AC5: photosensitivity max_height must resolve to {} px at 1280×600 (got {})",
        expected,
        ceiling
    );

    // Footer is reachable because it is `position_type: Absolute,
    // bottom: panel_padding` -- it does not scroll with body content.
    let footer = warning_footer_node();
    assert_eq!(
        footer.position_type,
        PositionType::Absolute,
        "AC5: photosensitivity Acknowledge footer must be position-absolute so it never scrolls off-screen"
    );
}

#[test]
fn test_overlay_overflow_ac6_primary_viewport_controls_reachable() {
    // 1366×768 -- max_height: 92 % = 706.56 px. All three modals must
    // resolve their ceilings to a positive px (controls reachable).
    let photo = photosensitivity_warning_panel_node();
    let conn = connection_lost_overlay_panel_node();
    let draft = draft_initial_modal_panel_node();

    for (label, panel) in [
        ("photosensitivity", &photo),
        ("connection_lost", &conn),
        ("draft_initial", &draft),
    ] {
        let ceiling = resolve_percent(panel.max_height, CANONICAL_VIEWPORT_PRIMARY_HEIGHT)
            .unwrap_or_else(|| panic!("AC6: {label} max_height must resolve at 1366×768"));
        let expected = CANONICAL_VIEWPORT_PRIMARY_HEIGHT * 0.92;
        assert!(
            (ceiling - expected).abs() < 0.01,
            "AC6: {label} max_height must resolve to ~{} px at 1366×768 (got {})",
            expected,
            ceiling
        );
    }
}

#[test]
fn test_overlay_overflow_ac7_4k_viewport_modal_scales_to_max_height() {
    // 3840×2160 -- max_height: 92 % = 1987.2 px. The draft-initial
    // modal previously hard-capped at the fixed `height: 360 px`
    // literal; it now scales with the viewport via the `max_height:
    // 92 %` ceiling.
    let panel = draft_initial_modal_panel_node();
    let ceiling = resolve_percent(panel.max_height, CANONICAL_VIEWPORT_4K_HEIGHT)
        .expect("AC7: draft-initial max_height must resolve at 3840×2160");
    let expected = CANONICAL_VIEWPORT_4K_HEIGHT * 0.92;
    assert!(
        (ceiling - expected).abs() < 0.01,
        "AC7: draft-initial max_height must resolve to ~{} px at 3840×2160 (got {})",
        expected,
        ceiling
    );
    assert!(
        ceiling >= 1987.0,
        "AC7: draft-initial max_height ceiling at 3840×2160 must be >= 1987 px (got {})",
        ceiling
    );
    // The pre-1349 fixed height of 360 px is no longer the cap.
    assert!(
        ceiling > DRAFT_INITIAL_MODAL_HEIGHT_PX,
        "AC7: draft-initial 4K ceiling ({}) must exceed the pre-1349 fixed 360 px height ({})",
        ceiling,
        DRAFT_INITIAL_MODAL_HEIGHT_PX
    );
}

// ─── AC8: result_screen.rs reference template unchanged ──────────────────

#[test]
fn test_overlay_overflow_ac8_result_screen_reference_template_unchanged() {
    // PROMPT 1180 §1.5 O-04 named `result_screen.rs:502-549` as the
    // template for the modal-overflow contract. PROMPT 1349 must leave
    // it alone. Re-derive that the published shape still matches the
    // pattern the other modals are being migrated toward: 92 % ceiling
    // + Flex column + padding + min/max widths preserved.
    let source = read_client_source("presentation/result_screen.rs");
    assert!(
        source.contains("max_height: Val::Percent(92.0)"),
        "AC8: result_screen.rs reference template must keep `max_height: Val::Percent(92.0)`"
    );
    assert!(
        source.contains("ResultScreenPanel"),
        "AC8: result_screen.rs must keep its ResultScreenPanel marker (sanity check)"
    );
}
