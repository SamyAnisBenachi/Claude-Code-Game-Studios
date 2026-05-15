//! Named UI flex-strip composition primitives — Sprint 14 Tier 0
//! foundation (story 004 / S11-TD-UI-FLEX-STRIPS).
//!
//! Tier 0 strip-composition primitives that HUD top (story 015), HUD
//! bottom (story 016), and hand UI consume. Each primitive expresses a
//! `Display::Flex` parent with explicit `flex_direction`,
//! `justify_content`, and `align_items`, anchored as a full-viewport-
//! width strip via `PositionType::Absolute`. PROMPT 802 §3.2 H1 / H8 /
//! H9 surfaced that the previous HUD top + bottom strips were composed
//! out of absolute-positioned children with magic `hud_margin + N`
//! offsets and no shared flex parent; this module is the single source
//! of truth those surfaces consume to express their strip parent.
//!
//! ## Strip primitives (top-of-viewport → bottom-of-viewport)
//!
//! | Primitive | Height (px) | Anchor | Canonical consumers |
//! |-----------|-------------|--------|---------------------|
//! | [`header_bar_node`] / [`HeaderBar`] | `60` | `top: 0`, full width | HUD top strip — gold / mana / phase / round / timer (story 015). |
//! | [`lane_bar_node`] / [`LaneBar`]     | `60` | `top: HEADER_BAR_HEIGHT_PX`, full width | Lane indicators / board-chrome strip (Tier 3 board-rendering scope). |
//! | [`hand_bar_node`] / [`HandBar`]     | `180` | `bottom: 0`, full width | Hand UI card-fan row (existing card-fan layout preserved per `f190cc7`). |
//! | [`footer_bar_node`] / [`FooterBar`] | `40`  | `bottom: HAND_BAR_HEIGHT_PX`, full width | HUD bottom strip — figurine area + reserve-strip readouts (story 016). |
//!
//! Strip heights are ratified verbatim by `docs/ux/global-ui-design-spec.md`
//! §9 and the canonical baseline fixture at
//! `tests/integration/fixtures/ui_viewport_baseline.rs::{HEADER_BAR_HEIGHT_PX,
//! FOOTER_BAR_HEIGHT_PX, HAND_BAR_HEIGHT_PX}` (already shipped by story
//! 005 / PROMPT 905-907-909).
//!
//! ## LaneBar worker decision
//!
//! Per `docs/ux/global-ui-design-spec.md` §9 the `LaneBar` primitive is
//! "implement as bevy_ui IFF the lane indicators are bevy_ui rather
//! than world-space sprites; otherwise the LaneBar primitive remains
//! documented but unimplemented". In the current playable client lane
//! indicators are rendered by `client/src/presentation/board_rendering.rs`
//! as world-space sprites (ADR-021 R2). Story 004 therefore ships
//! `LaneBar` as a **documented-only** primitive: the constant
//! [`LANE_BAR_HEIGHT_PX`] and the [`lane_bar_node`] helper are exported
//! so the contract is testable, but no production spawn site consumes
//! it. A future Tier 3 board-rendering story (`S11-UX-BOARD-RENDERING-SPEC`)
//! can promote it to a real consumer.
//!
//! ## HandBar vs. existing HAND_FAN_STRIP_HEIGHT_PX reconciliation
//!
//! `client/src/ui/hand/mod.rs::HAND_FAN_STRIP_HEIGHT_PX = 260.0` is the
//! `HandFanRoot` strip height shipped at `f190cc7`. The spec §9 ratifies
//! `HandBar = 180.0` px. Story 004 reconciles this by **wrapping
//! `HandFanRoot` inside a `HandBar` primitive** (option (b) from PROMPT
//! 913 readiness Concern #2): `HandBar` is a 180 px-tall full-width flex
//! container at `bottom: 0`; `HandFanRoot` is its child and retains its
//! 260 px local height so the existing card-fan chrome layout
//! (`f190cc7` repair: 7 chrome children at 100×100% / 20×20% / 15×15%)
//! is preserved verbatim. The fan extends 80 px above the `HandBar`
//! footprint; the strip primitive is the canonical
//! viewport-edge-anchored layout box that the responsive matrix
//! invariants (story 005 §"Deterministic strip height") read against.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: strip primitives are
//!   read-only presentation primitives. They do not introduce a new
//!   `MessageReceiver` drain or shift system ordering — they are
//!   consumed only at UI root spawn time.
//! - **ADR-002 Client-Server Authority**: strip constants do not carry
//!   game state. No optimistic client-side authority is introduced.
//!
//! ## Scope (Sprint 14 story 004)
//!
//! - Friend-game scope boundary preserved. `QA-COND-0005` Standard-tier
//!   accessibility (≥44px hit-target enforcement, focus order,
//!   keyboard navigation, screen-reader hints, text scaling),
//!   `QA-COND-0006` playtest validation, and `PAW-TD-*-a`
//!   placeholder-art accept-risk are **not** advanced by this module.
//! - Per-strip child order (HUD top-strip child order, lobby form
//!   sequencing) is owned by the per-surface Tier 1 stories
//!   (15 / 16 / 24 / 25 / 26). This module defines the strip parent;
//!   the surface story defines the children.

use bevy::prelude::*;

/// Deterministic HeaderBar height. 60 px across every viewport in the
/// canonical matrix (`docs/ux/global-ui-design-spec.md` §9; verbatim
/// from `tests/integration/fixtures/ui_viewport_baseline.rs`).
pub const HEADER_BAR_HEIGHT_PX: f32 = 60.0;

/// Deterministic LaneBar height. 60 px. Documented-only — no
/// production spawn site consumes this primitive at story 004 because
/// lane indicators are world-space sprites under
/// `client/src/presentation/board_rendering.rs` (ADR-021 R2).
pub const LANE_BAR_HEIGHT_PX: f32 = 60.0;

/// Deterministic HandBar height. 180 px across every viewport
/// (`docs/ux/global-ui-design-spec.md` §9). The existing
/// `HAND_FAN_STRIP_HEIGHT_PX = 260.0` from `f190cc7` is preserved as
/// the `HandFanRoot` child's local height; the spec strip footprint is
/// 180 px and the fan chrome extends 80 px above it.
pub const HAND_BAR_HEIGHT_PX: f32 = 180.0;

/// Deterministic FooterBar height. 40 px across every viewport
/// (`docs/ux/global-ui-design-spec.md` §9; verbatim from
/// `tests/integration/fixtures/ui_viewport_baseline.rs`).
pub const FOOTER_BAR_HEIGHT_PX: f32 = 40.0;

/// Marker component for the canonical HeaderBar strip primitive.
/// Spawned at most once per HUD root.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderBar;

/// Marker component for the canonical LaneBar strip primitive.
/// Documented but unimplemented at story 004; reserved for a future
/// Tier 3 board-rendering consumer.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneBar;

/// Marker component for the canonical HandBar strip primitive.
/// Spawned at most once per hand-UI root. Wraps the existing
/// `HandFanRoot` so the spec-ratified 180 px strip footprint is the
/// viewport-edge anchor while the f190cc7 card-fan chrome layout is
/// preserved verbatim inside the strip.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandBar;

/// Marker component for the canonical FooterBar strip primitive.
/// Spawned at most once per HUD root.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterBar;

/// Description of a single strip primitive's flex contract. The
/// integration test bin uses this to assert AC1 (`Display::Flex` +
/// documented `flex_direction` / `justify_content` / `align_items`)
/// without re-deriving the spec values at the test site.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StripContract {
    pub name: &'static str,
    pub height_px: f32,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
}

/// HeaderBar flex contract per `docs/ux/global-ui-design-spec.md` §9:
/// row direction, space-between justification, centre-aligned children.
pub const HEADER_BAR_CONTRACT: StripContract = StripContract {
    name: "HeaderBar",
    height_px: HEADER_BAR_HEIGHT_PX,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
};

/// LaneBar flex contract per `docs/ux/global-ui-design-spec.md` §9: row
/// direction, centred justification, centred children.
pub const LANE_BAR_CONTRACT: StripContract = StripContract {
    name: "LaneBar",
    height_px: LANE_BAR_HEIGHT_PX,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
};

/// HandBar flex contract per `docs/ux/global-ui-design-spec.md` §9: row
/// direction, centred justification, end-aligned children so the fan
/// reads as anchored to the viewport's bottom edge.
pub const HAND_BAR_CONTRACT: StripContract = StripContract {
    name: "HandBar",
    height_px: HAND_BAR_HEIGHT_PX,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::FlexEnd,
};

/// FooterBar flex contract per `docs/ux/global-ui-design-spec.md` §9:
/// row direction, space-between justification, centre-aligned children.
pub const FOOTER_BAR_CONTRACT: StripContract = StripContract {
    name: "FooterBar",
    height_px: FOOTER_BAR_HEIGHT_PX,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
};

/// All four canonical strip contracts in top-to-bottom viewport order.
pub const ALL_STRIP_CONTRACTS: [StripContract; 4] = [
    HEADER_BAR_CONTRACT,
    LANE_BAR_CONTRACT,
    HAND_BAR_CONTRACT,
    FOOTER_BAR_CONTRACT,
];

/// Build the canonical [`HeaderBar`] node — full-viewport-width flex
/// container anchored to the top edge.
pub fn header_bar_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(0.0),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Px(HEADER_BAR_HEIGHT_PX),
        display: Display::Flex,
        flex_direction: HEADER_BAR_CONTRACT.flex_direction,
        justify_content: HEADER_BAR_CONTRACT.justify_content,
        align_items: HEADER_BAR_CONTRACT.align_items,
        ..default()
    }
}

/// Build the canonical [`LaneBar`] node — full-viewport-width flex
/// container anchored immediately below [`HeaderBar`].
///
/// Documented but unimplemented at story 004 (see module doc); kept as
/// a callable helper so the integration test bin can assert the
/// primitive's flex contract without spawning a production consumer.
pub fn lane_bar_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(HEADER_BAR_HEIGHT_PX),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Px(LANE_BAR_HEIGHT_PX),
        display: Display::Flex,
        flex_direction: LANE_BAR_CONTRACT.flex_direction,
        justify_content: LANE_BAR_CONTRACT.justify_content,
        align_items: LANE_BAR_CONTRACT.align_items,
        ..default()
    }
}

/// Build the canonical [`HandBar`] node — full-viewport-width flex
/// container anchored to the bottom edge.
pub fn hand_bar_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(0.0),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Px(HAND_BAR_HEIGHT_PX),
        display: Display::Flex,
        flex_direction: HAND_BAR_CONTRACT.flex_direction,
        justify_content: HAND_BAR_CONTRACT.justify_content,
        align_items: HAND_BAR_CONTRACT.align_items,
        overflow: Overflow::visible(),
        ..default()
    }
}

/// Build the canonical [`FooterBar`] node — full-viewport-width flex
/// container anchored immediately above [`HandBar`].
pub fn footer_bar_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(HAND_BAR_HEIGHT_PX),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Px(FOOTER_BAR_HEIGHT_PX),
        display: Display::Flex,
        flex_direction: FOOTER_BAR_CONTRACT.flex_direction,
        justify_content: FOOTER_BAR_CONTRACT.justify_content,
        align_items: FOOTER_BAR_CONTRACT.align_items,
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac1_three_required_strip_primitives_are_exported() {
        // AC1 requires ≥3 named strip primitives. HeaderBar / HandBar /
        // FooterBar are MANDATORY; LaneBar is documented (worker
        // discretion per spec §9). Verify the mandatory three appear in
        // the canonical contract list.
        let names: Vec<&str> = ALL_STRIP_CONTRACTS.iter().map(|c| c.name).collect();
        assert!(names.contains(&"HeaderBar"), "AC1: HeaderBar required");
        assert!(names.contains(&"HandBar"), "AC1: HandBar required");
        assert!(names.contains(&"FooterBar"), "AC1: FooterBar required");
    }

    #[test]
    fn ac1_each_strip_node_declares_display_flex_and_documented_axes() {
        // AC1: each strip primitive expresses Display::Flex with
        // documented flex_direction / justify_content / align_items.
        let cases = [
            ("HeaderBar", header_bar_node(), HEADER_BAR_CONTRACT),
            ("LaneBar", lane_bar_node(), LANE_BAR_CONTRACT),
            ("HandBar", hand_bar_node(), HAND_BAR_CONTRACT),
            ("FooterBar", footer_bar_node(), FOOTER_BAR_CONTRACT),
        ];
        for (name, node, contract) in cases {
            assert_eq!(
                node.display,
                Display::Flex,
                "AC1: {name} must declare Display::Flex"
            );
            assert_eq!(
                node.flex_direction, contract.flex_direction,
                "AC1: {name} flex_direction must match contract"
            );
            assert_eq!(
                node.justify_content, contract.justify_content,
                "AC1: {name} justify_content must match contract"
            );
            assert_eq!(
                node.align_items, contract.align_items,
                "AC1: {name} align_items must match contract"
            );
        }
    }

    #[test]
    fn ac1_each_strip_is_full_viewport_width_at_absolute_position() {
        // AC1 / AC6: every strip is a full-viewport-width
        // PositionType::Absolute anchor so its width scales with the
        // viewport (Val::Percent(100.0)) while the height is
        // deterministic (Val::Px).
        let cases = [
            ("HeaderBar", header_bar_node()),
            ("LaneBar", lane_bar_node()),
            ("HandBar", hand_bar_node()),
            ("FooterBar", footer_bar_node()),
        ];
        for (name, node) in cases {
            assert_eq!(
                node.position_type,
                PositionType::Absolute,
                "AC1: {name} must be PositionType::Absolute"
            );
            assert_eq!(
                node.width,
                Val::Percent(100.0),
                "AC1: {name} must span the full viewport width"
            );
            match node.height {
                Val::Px(_) => {}
                other => panic!("AC1: {name} height must be Val::Px(_); got {other:?}"),
            }
        }
    }

    #[test]
    fn ac1_canonical_strip_heights_match_spec_section_9() {
        // global-ui-design-spec.md §9 ratifies 60 / 60 / 180 / 40.
        assert_eq!(HEADER_BAR_HEIGHT_PX, 60.0);
        assert_eq!(LANE_BAR_HEIGHT_PX, 60.0);
        assert_eq!(HAND_BAR_HEIGHT_PX, 180.0);
        assert_eq!(FOOTER_BAR_HEIGHT_PX, 40.0);
    }

    #[test]
    fn ac6_strip_heights_are_deterministic_pixel_values() {
        // AC6 / §8: pixel-fixed strip heights are identical across
        // every viewport. Asserted as a finite-positive guard so a
        // future regression cannot replace these with viewport-scaled
        // values without tripping this assertion.
        for c in ALL_STRIP_CONTRACTS {
            assert!(
                c.height_px > 0.0 && c.height_px.is_finite(),
                "{} height_px must be a positive finite pixel value",
                c.name
            );
        }
    }

    #[test]
    fn ac6_top_strip_does_not_overlap_bottom_strips_in_canonical_viewport() {
        // For the smallest supported viewport (1366×768) and the
        // canonical 16:9 baseline (1920×1080), the top strip
        // (HeaderBar) and the bottom strip column (FooterBar +
        // HandBar) must not overlap. FooterBar sits at
        // `bottom: HAND_BAR_HEIGHT_PX` and HandBar at `bottom: 0`, so
        // the bottom strip column reserves `HAND_BAR_HEIGHT_PX +
        // FOOTER_BAR_HEIGHT_PX` pixels at the bottom of the viewport.
        for &vh in &[768.0_f32, 1080.0, 1200.0, 960.0, 2160.0] {
            let reserved_bottom = HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX;
            let header_bottom_edge_y = HEADER_BAR_HEIGHT_PX;
            let footer_top_edge_y = vh - reserved_bottom;
            assert!(
                header_bottom_edge_y < footer_top_edge_y,
                "HeaderBar bottom edge ({header_bottom_edge_y}) must sit above \
                 the FooterBar top edge ({footer_top_edge_y}) for viewport \
                 height {vh}"
            );
        }
    }

    #[test]
    fn ac6_strip_anchor_offsets_match_spec_section_9_column_composition() {
        // §9 column composition:
        //   1. HeaderBar at top: 0
        //   2. FooterBar at bottom: HAND_BAR_HEIGHT_PX
        //   3. HandBar at bottom: 0
        let header = header_bar_node();
        assert_eq!(header.top, Val::Px(0.0), "HeaderBar must anchor at top: 0");
        let footer = footer_bar_node();
        assert_eq!(
            footer.bottom,
            Val::Px(HAND_BAR_HEIGHT_PX),
            "FooterBar must anchor at bottom: HAND_BAR_HEIGHT_PX"
        );
        let hand = hand_bar_node();
        assert_eq!(
            hand.bottom,
            Val::Px(0.0),
            "HandBar must anchor at bottom: 0"
        );
    }
}
