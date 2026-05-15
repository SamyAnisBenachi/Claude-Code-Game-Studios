//! Sprint 14 / Story 004 — S11-TD-UI-FLEX-STRIPS integration tests.
//!
//! Covers the acceptance criteria that the inline unit tests in
//! `client/src/ui/design_tokens/strips.rs` and
//! `client/src/ui/design_tokens/spacing.rs` cannot reach:
//!
//! - **AC1** mandatory strip primitives (`HeaderBar`, `HandBar`,
//!   `FooterBar`) export `Display::Flex` + documented flex axes (also
//!   covered inline; reproduced here as the surface-level contract).
//! - **AC2** spacing-scale constants strictly ascending (also covered
//!   inline; reproduced here so the integration bin asserts the
//!   recomposition rule for the deleted `_GAP_PX` magic constants).
//! - **AC3** HUD top strip references the `HeaderBar` primitive and the
//!   `spacing` design-token module (replaces `HUD_GOLD_ROW_GAP_PX` and
//!   `HUD_SECONDARY_ROW_GAP_PX` magic offsets).
//! - **AC4** HUD bottom strip references the `FooterBar` primitive and
//!   the `strips`/`spacing` design-token modules (replaces
//!   `hud_margin + 60.0` figurine magic offset).
//! - **AC5** hand UI references the `HandBar` primitive wrapping
//!   `HandFanRoot` (preserves `f190cc7` chrome verbatim).
//! - **AC6** deterministic per-viewport pixel heights across the
//!   canonical 6-viewport matrix.
//! - **AC7** workspace grep-guard: no surviving `_GAP_PX` identifier
//!   in `client/src/ui/hud/mod.rs`.
//! - **AC8** strip primitive unit-style assertions runnable from this
//!   integration test bin (covers the qa-plan §line 205
//!   `cargo test -p client --test ui_clean_pass_strips_test`
//!   verification path).
//!
//! No optimistic client-side authority is introduced or relied upon by
//! these tests. They are read-only checks over the design-token
//! modules and the migrated source.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use client::ui::design_tokens::spacing::{
    self, ALL_SPACINGS_ASCENDING, SPACING_LG, SPACING_MD, SPACING_MIN_GAP, SPACING_SM, SPACING_XL,
    SPACING_XS,
};
use client::ui::design_tokens::strips::{
    self, footer_bar_node, hand_bar_node, header_bar_node, lane_bar_node, ALL_STRIP_CONTRACTS,
    FOOTER_BAR_HEIGHT_PX, HAND_BAR_HEIGHT_PX, HEADER_BAR_HEIGHT_PX, LANE_BAR_HEIGHT_PX,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_client_source(rel: &str) -> String {
    let path = client_src_root().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

// ─── AC1 ────────────────────────────────────────────────────────────────

#[test]
fn ac1_three_required_strip_primitives_exported_with_flex_display() {
    test_helpers::init_test_tracing();

    let mandatory = ["HeaderBar", "HandBar", "FooterBar"];
    for name in mandatory {
        let found = ALL_STRIP_CONTRACTS.iter().any(|c| c.name == name);
        assert!(
            found,
            "AC1: mandatory strip primitive `{name}` must appear in \
             ALL_STRIP_CONTRACTS"
        );
    }

    // Each strip's spawned Node declares Display::Flex.
    let nodes = [
        ("HeaderBar", header_bar_node()),
        ("HandBar", hand_bar_node()),
        ("FooterBar", footer_bar_node()),
        ("LaneBar", lane_bar_node()),
    ];
    for (name, node) in nodes {
        assert_eq!(
            node.display,
            Display::Flex,
            "AC1: `{name}` must declare Display::Flex parent"
        );
    }
}

#[test]
fn ac1_each_strip_documents_flex_direction_justify_align() {
    // The strips module declares `StripContract { flex_direction,
    // justify_content, align_items }` for every primitive. AC1
    // verification: every contract resolves to a non-default canonical
    // value documented in `docs/ux/global-ui-design-spec.md` §9.
    for c in ALL_STRIP_CONTRACTS {
        // Row direction is the spec canonical for every strip.
        assert_eq!(
            c.flex_direction,
            FlexDirection::Row,
            "AC1: `{}` flex_direction must be Row per spec §9",
            c.name
        );
        // The contract resolves to a documented variant (no Default
        // sentinel slipping through).
        assert!(
            matches!(
                c.justify_content,
                JustifyContent::SpaceBetween | JustifyContent::Center
            ),
            "AC1: `{}` justify_content must be SpaceBetween or Center per spec §9; \
             got {:?}",
            c.name,
            c.justify_content
        );
        assert!(
            matches!(
                c.align_items,
                AlignItems::Center | AlignItems::FlexEnd | AlignItems::FlexStart
            ),
            "AC1: `{}` align_items must be Center / FlexEnd / FlexStart per spec §9; \
             got {:?}",
            c.name,
            c.align_items
        );
    }
}

// ─── AC2 ────────────────────────────────────────────────────────────────

#[test]
fn ac2_spacing_scale_strictly_ascending_canonical_values() {
    test_helpers::init_test_tracing();
    // Spec §4 ratifies SPACING_XS=4 / SM=8 / MD=16 / LG=24 / XL=32.
    assert_eq!(spacing::SPACING_XS, 4.0);
    assert_eq!(SPACING_SM, 8.0);
    assert_eq!(SPACING_MD, 16.0);
    assert_eq!(SPACING_LG, 24.0);
    assert_eq!(SPACING_XL, 32.0);

    let values: Vec<f32> = ALL_SPACINGS_ASCENDING.iter().map(|(_, v)| *v).collect();
    for window in values.windows(2) {
        assert!(
            window[0] < window[1],
            "AC2: spacing scale must be strictly ascending: {} < {} failed",
            window[0],
            window[1]
        );
    }

    // Minimum-gap reservation for future intermediate scale steps.
    for window in ALL_SPACINGS_ASCENDING.windows(2) {
        let gap = window[1].1 - window[0].1;
        assert!(
            gap >= SPACING_MIN_GAP,
            "AC2: gap between `{}` and `{}` must be ≥ SPACING_MIN_GAP \
             ({SPACING_MIN_GAP}); got {gap}",
            window[0].0,
            window[1].0
        );
    }
}

#[test]
fn ac2_hud_gold_row_gap_recomposes_through_spacing_tokens() {
    // PROMPT 802 §3.9 G2 enumerated the magic constant
    // `HUD_GOLD_ROW_GAP_PX = 48.0`. Story 004 recomposes it via
    // `SPACING_XL + SPACING_MD` (32 + 16) and AC2 asserts the
    // recomposition still resolves to 48 px so the visual layout is
    // preserved.
    assert_eq!(SPACING_XL + SPACING_MD, 48.0);
}

#[test]
fn ac2_hud_secondary_row_gap_recomposes_through_spacing_tokens() {
    // PROMPT 802 §3.9 G2 enumerated the magic constant
    // `HUD_SECONDARY_ROW_GAP_PX = 28.0`. Story 004 recomposes it via
    // `SPACING_XL - SPACING_XS` (32 - 4) and AC2 asserts the
    // recomposition still resolves to 28 px so the visual layout is
    // preserved.
    assert_eq!(SPACING_XL - SPACING_XS, 28.0);
}

// ─── AC3 — HUD top strip migration spot-check ───────────────────────────

#[test]
fn ac3_hud_module_imports_strips_and_spacing_design_tokens() {
    test_helpers::init_test_tracing();
    let text = read_client_source("ui/hud/mod.rs");
    // The migrated HUD module routes its strip-relative anchors through
    // the design-token modules instead of magic literals.
    assert!(
        text.contains("design_tokens::{spacing, strips, typography, z_layers}")
            || (text.contains("strips") && text.contains("spacing")),
        "AC3: HUD module must import `strips` and `spacing` design-token modules"
    );
}

#[test]
fn ac3_hud_module_spawns_header_bar_primitive() {
    let text = read_client_source("ui/hud/mod.rs");
    assert!(
        text.contains("strips::HeaderBar") && text.contains("strips::header_bar_node()"),
        "AC3: HUD top strip must spawn the canonical `strips::HeaderBar` \
         primitive via `strips::header_bar_node()`"
    );
}

#[test]
fn ac3_hud_gold_row_offset_resolves_through_spacing_tokens() {
    let text = read_client_source("ui/hud/mod.rs");
    assert!(
        text.contains("spacing::SPACING_XL + spacing::SPACING_MD"),
        "AC3: HUD gold-row vertical offset must recompose via \
         `spacing::SPACING_XL + spacing::SPACING_MD` (replaces the \
         deleted `HUD_GOLD_ROW_GAP_PX = 48.0` magic constant)"
    );
}

#[test]
fn ac3_hud_secondary_row_offset_resolves_through_spacing_tokens() {
    let text = read_client_source("ui/hud/mod.rs");
    assert!(
        text.contains("spacing::SPACING_XL - spacing::SPACING_XS"),
        "AC3: HUD secondary-row vertical offset must recompose via \
         `spacing::SPACING_XL - spacing::SPACING_XS` (replaces the \
         deleted `HUD_SECONDARY_ROW_GAP_PX = 28.0` magic constant)"
    );
}

#[test]
fn ac3_hud_timer_bar_anchors_to_header_bar_height_token() {
    let text = read_client_source("ui/hud/mod.rs");
    assert!(
        text.contains("strips::HEADER_BAR_HEIGHT_PX"),
        "AC3: HUD timer bar must anchor to `strips::HEADER_BAR_HEIGHT_PX` \
         (replaces the previous `hud_margin + 48.0` magic offset)"
    );
}

// ─── AC4 — HUD bottom strip migration spot-check ────────────────────────

#[test]
fn ac4_hud_module_spawns_footer_bar_primitive() {
    let text = read_client_source("ui/hud/mod.rs");
    assert!(
        text.contains("strips::FooterBar") && text.contains("strips::footer_bar_node()"),
        "AC4: HUD bottom strip must spawn the canonical `strips::FooterBar` \
         primitive via `strips::footer_bar_node()`"
    );
}

#[test]
fn ac4_hud_figurine_anchors_to_footer_bar_and_spacing_tokens() {
    let text = read_client_source("ui/hud/mod.rs");
    assert!(
        text.contains("strips::FOOTER_BAR_HEIGHT_PX + spacing::SPACING_XL"),
        "AC4: HUD figurine bottom offset must recompose via \
         `strips::FOOTER_BAR_HEIGHT_PX + spacing::SPACING_XL` \
         (replaces the previous `hud_margin + 60.0` magic offset)"
    );
}

// ─── AC5 — hand UI HandBar migration spot-check ─────────────────────────

#[test]
fn ac5_hand_module_imports_strips_and_spawns_hand_bar_primitive() {
    let text = read_client_source("ui/hand/mod.rs");
    assert!(
        text.contains("strips"),
        "AC5: hand UI module must import `strips` design-token module"
    );
    assert!(
        text.contains("strips::HandBar") && text.contains("strips::hand_bar_node()"),
        "AC5: hand UI must spawn the canonical `strips::HandBar` primitive \
         via `strips::hand_bar_node()`"
    );
}

#[test]
fn ac5_hand_fan_root_is_a_child_of_hand_bar() {
    let text = read_client_source("ui/hand/mod.rs");
    // The HandFanRoot is spawned with ChildOf(hand_bar) so the
    // existing f190cc7 fan layout is preserved verbatim inside the
    // canonical strip primitive.
    assert!(
        text.contains("ChildOf(hand_bar)"),
        "AC5: `HandFanRoot` must be parented to the `HandBar` strip \
         primitive via `ChildOf(hand_bar)`"
    );
    // The f190cc7 chrome contract (HAND_FAN_STRIP_HEIGHT_PX local
    // height) is preserved unchanged.
    assert!(
        text.contains("HAND_FAN_STRIP_HEIGHT_PX"),
        "AC5: `f190cc7` card-fan chrome contract preserved — \
         `HAND_FAN_STRIP_HEIGHT_PX` must remain the local height of \
         `HandFanRoot` so the 7 chrome children at 100×100% / 20×20% / \
         15×15% still resolve"
    );
}

// ─── AC6 — deterministic strip heights across the canonical viewport
// matrix ───────────────────────────────────────────────────────────────

#[test]
fn ac6_strip_heights_are_identical_across_every_canonical_viewport() {
    // The canonical 6-viewport matrix from
    // `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`
    // (already shipped by story 005). Strip heights MUST resolve to
    // the same pixel value across every viewport.
    let viewports = [
        ("1366x768", 1366.0_f32, 768.0_f32),
        ("1920x1080", 1920.0, 1080.0),
        ("1920x1200", 1920.0, 1200.0),
        ("1280x960", 1280.0, 960.0),
        ("3840x2160", 3840.0, 2160.0),
        ("2560x1080", 2560.0, 1080.0),
    ];

    for (name, _w, vh) in viewports {
        // The strip Node's height is `Val::Px(<deterministic>)`,
        // not `Val::Percent(_)`. The height value is read directly
        // from the design-token constants.
        assert_eq!(
            HEADER_BAR_HEIGHT_PX, 60.0,
            "AC6 viewport {name}: HEADER_BAR_HEIGHT_PX must be a \
             deterministic 60 px across all viewports"
        );
        assert_eq!(
            FOOTER_BAR_HEIGHT_PX, 40.0,
            "AC6 viewport {name}: FOOTER_BAR_HEIGHT_PX must be a \
             deterministic 40 px across all viewports"
        );
        assert_eq!(
            HAND_BAR_HEIGHT_PX, 180.0,
            "AC6 viewport {name}: HAND_BAR_HEIGHT_PX must be a \
             deterministic 180 px across all viewports"
        );
        assert_eq!(
            LANE_BAR_HEIGHT_PX, 60.0,
            "AC6 viewport {name}: LANE_BAR_HEIGHT_PX must be a \
             deterministic 60 px across all viewports"
        );
        // The bottom-edge strip column reserves
        // HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX = 220 px at the
        // viewport bottom; the HeaderBar reserves HEADER_BAR_HEIGHT_PX
        // = 60 px at the top. Centre play area = vh - 220 - 60. For
        // 1366×768 that is 488 px (positive) so the strip column does
        // not overlap.
        let reserved = HEADER_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX + HAND_BAR_HEIGHT_PX;
        assert!(
            vh > reserved,
            "AC6 viewport {name}: viewport height {vh} must exceed \
             reserved strip footprint {reserved} so the centre play \
             area has positive height"
        );
    }
}

#[test]
fn ac6_strips_span_full_viewport_width() {
    // AC6: each strip is a `Val::Percent(100.0)` width — scales with
    // the viewport.
    for node in [
        header_bar_node(),
        lane_bar_node(),
        hand_bar_node(),
        footer_bar_node(),
    ] {
        assert_eq!(
            node.width,
            Val::Percent(100.0),
            "AC6: every strip must span the full viewport width \
             (Val::Percent(100.0))"
        );
    }
}

// ─── AC7 — grep guard against surviving `_GAP_PX` magic constants ──────

#[test]
fn ac7_no_gap_px_identifier_in_hud_module() {
    test_helpers::init_test_tracing();
    let path = client_src_root().join("ui").join("hud").join("mod.rs");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("AC7 grep guard failed to read {}: {err}", path.display()));
    let mut violations: Vec<String> = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        if !line.contains("_GAP_PX") {
            continue;
        }
        // Allow doc / comment references that explicitly name the
        // deleted constants in the historical reference block (the
        // call-out that the constants WERE removed).
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
            continue;
        }
        violations.push(format!(
            "{}:{}: {}",
            path.display(),
            line_no + 1,
            line.trim_end()
        ));
    }
    assert!(
        violations.is_empty(),
        "AC7 grep guard found surviving `_GAP_PX` identifier in HUD \
         module — migrate to `spacing::SPACING_*` tokens. Violations:\n{}",
        violations.join("\n")
    );
}

// ─── AC8 — strip primitive unit-style assertions runnable from the
// integration test bin ─────────────────────────────────────────────────

#[test]
fn ac8_each_strip_node_resolves_to_documented_flex_axis_set() {
    // AC8 (`cargo test -p client --test ui_clean_pass_strips_test`):
    // every strip's Node style fields match the contract.
    let cases = [
        (
            "HeaderBar",
            header_bar_node(),
            FlexDirection::Row,
            JustifyContent::SpaceBetween,
            AlignItems::Center,
            HEADER_BAR_HEIGHT_PX,
        ),
        (
            "LaneBar",
            lane_bar_node(),
            FlexDirection::Row,
            JustifyContent::Center,
            AlignItems::Center,
            LANE_BAR_HEIGHT_PX,
        ),
        (
            "HandBar",
            hand_bar_node(),
            FlexDirection::Row,
            JustifyContent::Center,
            AlignItems::FlexEnd,
            HAND_BAR_HEIGHT_PX,
        ),
        (
            "FooterBar",
            footer_bar_node(),
            FlexDirection::Row,
            JustifyContent::SpaceBetween,
            AlignItems::Center,
            FOOTER_BAR_HEIGHT_PX,
        ),
    ];
    for (name, node, fdir, justify, align, height) in cases {
        assert_eq!(node.display, Display::Flex, "AC8 {name} display");
        assert_eq!(node.flex_direction, fdir, "AC8 {name} flex_direction");
        assert_eq!(node.justify_content, justify, "AC8 {name} justify_content");
        assert_eq!(node.align_items, align, "AC8 {name} align_items");
        assert_eq!(
            node.height,
            Val::Px(height),
            "AC8 {name} height must be Val::Px({height})"
        );
        assert_eq!(
            node.width,
            Val::Percent(100.0),
            "AC8 {name} width must be Val::Percent(100.0)"
        );
        assert_eq!(
            node.position_type,
            PositionType::Absolute,
            "AC8 {name} position_type must be Absolute"
        );
    }
}

#[test]
fn ac8_strip_anchors_match_spec_column_composition() {
    // §9 column composition: HeaderBar at top:0, FooterBar at
    // bottom:HAND_BAR_HEIGHT_PX, HandBar at bottom:0.
    assert_eq!(header_bar_node().top, Val::Px(0.0));
    assert_eq!(
        footer_bar_node().bottom,
        Val::Px(HAND_BAR_HEIGHT_PX),
        "FooterBar must anchor immediately above HandBar"
    );
    assert_eq!(hand_bar_node().bottom, Val::Px(0.0));
    // LaneBar is documented but unimplemented — its helper anchors at
    // `top: HEADER_BAR_HEIGHT_PX` for the integration bin.
    assert_eq!(lane_bar_node().top, Val::Px(HEADER_BAR_HEIGHT_PX));
}

#[test]
fn ac8_strip_marker_components_are_distinct_zero_sized_components() {
    // Each strip primitive exports a unique marker component so
    // production spawn sites can query the strip parent without
    // resorting to entity-name string matching.
    use std::any::TypeId;
    let ids = [
        TypeId::of::<strips::HeaderBar>(),
        TypeId::of::<strips::LaneBar>(),
        TypeId::of::<strips::HandBar>(),
        TypeId::of::<strips::FooterBar>(),
    ];
    let mut sorted = ids.to_vec();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        4,
        "AC8: every strip primitive must export a distinct marker component"
    );
}
