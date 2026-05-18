//! Layout-safety viewport matrix — Sprint 17 UI layout foundation
//! (PROMPT 1181 / `S17-UI-LAYOUT-FOUNDATION-PRIMITIVES`).
//!
//! Every reusable layout primitive that asserts "fits the viewport" reads
//! its (width, height) test grid from [`SAFETY_VIEWPORT_MATRIX`] here
//! instead of inventing an ad-hoc set per call site. PROMPT 1035 §"Test
//! coverage gaps" surfaced that the existing fixtures sit at
//! `1280 × 720` (e.g. `tests/integration/hand-ui/
//! hand_ui_chrome_composition_test.rs:39`) while
//! `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS` names
//! `1366 × 768` as the production minimum — so two parallel viewport
//! grids govern UI safety checks and neither one alone catches both
//! classes of clipping. This module reconciles them as a small
//! 3-viewport "safety matrix" that every modal-panel / CTA-row / scroll
//! region / text-fit test consumes.
//!
//! ## Matrix
//!
//! | Name | Width × Height | Role |
//! |------|----------------|------|
//! | [`SAFETY_VIEWPORT_DEV_FLOOR`]   | `1280 × 720`  | Dev / test-fixture floor (existing chrome-composition tests). A surface that fits here will fit the production minimum below by simple inclusion. |
//! | [`SAFETY_VIEWPORT_PROD_MIN`]    | `1366 × 768`  | Production minimum supported viewport per `docs/ux/global-ui-design-spec.md` §8 ratified row 1. Every surface MUST fit. |
//! | [`SAFETY_VIEWPORT_HD_BASELINE`] | `1920 × 1080` | Baseline reference (HD) per spec §8 row 2. The design-source viewport. |
//!
//! The matrix is deliberately small — the broader 6-viewport set at
//! [`tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`] is
//! the canonical fixture for full-app viewport-invariant suites
//! (story 005 / PROMPT 905-907-909); this 3-row safety matrix is the
//! tight loop that every *primitive* test runs.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: viewport-matrix
//!   constants are read-only presentation primitives. They do not
//!   introduce a new `MessageReceiver` drain or shift system ordering.
//! - **ADR-002 Client-Server Authority**: viewport constants do not
//!   carry game state.
//!
//! ## Scope (PROMPT 1181)
//!
//! - Friend-game scope boundary preserved. `QA-COND-0005` Standard-tier
//!   accessibility (full keyboard navigation, screen-reader hints,
//!   text scaling), `QA-COND-0006` playtest validation, and
//!   `PAW-TD-*-a` placeholder-art accept-risk are **not** advanced by
//!   this module.
//! - No surface migration is attempted by PROMPT 1181 — the matrix is
//!   the foundation that later surface workers consume.

/// One canonical viewport size in the layout-safety matrix. Width and
/// height are integer logical pixels matching bevy's default UI scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SafetyViewport {
    /// Stable human-readable name for assertion failure messages.
    pub name: &'static str,
    pub width_px: u32,
    pub height_px: u32,
}

impl SafetyViewport {
    /// Width as `f32` for use in pixel-budget arithmetic.
    pub const fn width_f32(&self) -> f32 {
        self.width_px as f32
    }

    /// Height as `f32` for use in pixel-budget arithmetic.
    pub const fn height_f32(&self) -> f32 {
        self.height_px as f32
    }
}

/// Dev / test-fixture floor. Equals the
/// `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs` fixture
/// size (`VIEWPORT_WIDTH = 1280.0` / `VIEWPORT_HEIGHT = 720.0`) so legacy
/// hand-UI tests fold into the same matrix without ratchet drift.
pub const SAFETY_VIEWPORT_DEV_FLOOR: SafetyViewport = SafetyViewport {
    name: "1280x720",
    width_px: 1280,
    height_px: 720,
};

/// Production minimum supported viewport per
/// `docs/ux/global-ui-design-spec.md` §8 row 1 (`1366 × 768` 16:9 common
/// laptop default). Every surface MUST fit.
pub const SAFETY_VIEWPORT_PROD_MIN: SafetyViewport = SafetyViewport {
    name: "1366x768",
    width_px: 1366,
    height_px: 768,
};

/// Baseline reference (HD) per `docs/ux/global-ui-design-spec.md` §8
/// row 2 (`1920 × 1080` 16:9). The design-source viewport — every
/// baseline screenshot captures here.
pub const SAFETY_VIEWPORT_HD_BASELINE: SafetyViewport = SafetyViewport {
    name: "1920x1080",
    width_px: 1920,
    height_px: 1080,
};

/// The full 3-viewport layout-safety matrix in ascending-width order.
/// Every primitive's viewport-safety test iterates this array and
/// asserts containment / no-clipping at every row.
pub const SAFETY_VIEWPORT_MATRIX: [SafetyViewport; 3] = [
    SAFETY_VIEWPORT_DEV_FLOOR,
    SAFETY_VIEWPORT_PROD_MIN,
    SAFETY_VIEWPORT_HD_BASELINE,
];

/// Smallest viewport in the safety matrix. A surface that fits this
/// rectangle fits every larger row in the matrix by inclusion.
pub const SAFETY_VIEWPORT_SMALLEST: SafetyViewport = SAFETY_VIEWPORT_DEV_FLOOR;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_contains_three_canonical_rows() {
        assert_eq!(SAFETY_VIEWPORT_MATRIX.len(), 3);
        assert_eq!(SAFETY_VIEWPORT_MATRIX[0], SAFETY_VIEWPORT_DEV_FLOOR);
        assert_eq!(SAFETY_VIEWPORT_MATRIX[1], SAFETY_VIEWPORT_PROD_MIN);
        assert_eq!(SAFETY_VIEWPORT_MATRIX[2], SAFETY_VIEWPORT_HD_BASELINE);
    }

    #[test]
    fn matrix_is_strictly_ascending_in_width_and_height() {
        for window in SAFETY_VIEWPORT_MATRIX.windows(2) {
            assert!(
                window[0].width_px < window[1].width_px,
                "safety matrix width must be strictly ascending: {} < {} failed",
                window[0].name,
                window[1].name,
            );
            assert!(
                window[0].height_px < window[1].height_px,
                "safety matrix height must be strictly ascending: {} < {} failed",
                window[0].name,
                window[1].name,
            );
        }
    }

    #[test]
    fn smallest_alias_points_at_first_matrix_row() {
        // The "fits smallest -> fits all" inclusion rule depends on the
        // alias resolving to the matrix's first row.
        assert_eq!(SAFETY_VIEWPORT_SMALLEST, SAFETY_VIEWPORT_MATRIX[0]);
    }

    #[test]
    fn prod_min_matches_global_spec_section_8_row_1() {
        // Cross-checks docs/ux/global-ui-design-spec.md §8 row 1
        // "Minimum supported viewport — 1366 × 768".
        assert_eq!(SAFETY_VIEWPORT_PROD_MIN.width_px, 1366);
        assert_eq!(SAFETY_VIEWPORT_PROD_MIN.height_px, 768);
    }

    #[test]
    fn hd_baseline_matches_global_spec_section_8_row_2() {
        assert_eq!(SAFETY_VIEWPORT_HD_BASELINE.width_px, 1920);
        assert_eq!(SAFETY_VIEWPORT_HD_BASELINE.height_px, 1080);
    }

    #[test]
    fn dev_floor_matches_legacy_chrome_composition_fixture() {
        // Cross-checks `tests/integration/hand-ui/
        // hand_ui_chrome_composition_test.rs::VIEWPORT_*`. The PROMPT
        // 1035 audit flagged the dev-fixture vs prod-minimum split;
        // this module owns the reconciliation.
        assert_eq!(SAFETY_VIEWPORT_DEV_FLOOR.width_px, 1280);
        assert_eq!(SAFETY_VIEWPORT_DEV_FLOOR.height_px, 720);
    }

    #[test]
    fn width_and_height_f32_accessors_match_underlying_u32() {
        for v in SAFETY_VIEWPORT_MATRIX {
            assert!((v.width_f32() - v.width_px as f32).abs() < f32::EPSILON);
            assert!((v.height_f32() - v.height_px as f32).abs() < f32::EPSILON);
        }
    }
}
