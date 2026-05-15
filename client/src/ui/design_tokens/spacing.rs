//! Named UI spacing-scale constants — Sprint 14 Tier 0 foundation
//! (story 004 / S11-TD-UI-FLEX-STRIPS).
//!
//! Every `bevy_ui` strip primitive, panel, button, and label spawn site
//! in the playable client requests its child gap, padding, or
//! inter-element margin from one of the five named semantic scales
//! defined here instead of embedding ad-hoc numeric literals at the
//! spawn site. PROMPT 802 §3.9 G2 surfaced that the previous codebase
//! had per-module `_GAP_PX` constants (`HUD_GOLD_ROW_GAP_PX = 48.0`,
//! `HUD_SECONDARY_ROW_GAP_PX = 28.0`, etc.) with no shared scale; this
//! module is the single source of truth those surfaces consume.
//!
//! ## Semantic scale (smallest → largest)
//!
//! | Token       | Pixels | Canonical use |
//! |-------------|--------|---------------|
//! | [`SPACING_XS`] | `4`  | Tightest gap. Adjacent icon + numeric readout, badge padding, intra-cluster spacing. |
//! | [`SPACING_SM`] | `8`  | Default child gap inside a tight cluster (e.g. HUD secondary-row gold-icon + value). |
//! | [`SPACING_MD`] | `16` | Default gap between distinct readouts on the same strip (e.g. HUD gold cluster ↔ HUD mana cluster). Default panel padding. |
//! | [`SPACING_LG`] | `24` | Section separator inside a panel; gap between a strip's left edge and its first child. |
//! | [`SPACING_XL`] | `32` | Largest single step. Headline ↔ body separation, lobby form section separator. |
//!
//! The geometric step is approximately `×2` (4 → 8 → 16 → 24 → 32). The
//! 24-step breaks strict doubling intentionally so consumers have a
//! "between MD and XL" middle option for asymmetric layouts.
//!
//! Each adjacent semantic level is separated by at least
//! [`SPACING_MIN_GAP`] pixels so future intermediate levels can be added
//! without re-ordering existing constants.
//!
//! ## Recomposition for values larger than `SPACING_XL`
//!
//! Per `docs/ux/global-ui-design-spec.md` §4 "Replacement target":
//! values larger than `SPACING_XL = 32` are recomposed as `XL + MD`,
//! `XL + XL`, or as explicit padding on the strip's container. The HUD
//! 48-pixel gold-row gap is the canonical example — it migrates to
//! `SPACING_XL + SPACING_MD` (32 + 16) rather than gaining a new named
//! scale step.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: spacing tokens are
//!   read-only presentation primitives. They do not introduce a new
//!   `MessageReceiver` drain or shift system ordering — they are
//!   consumed only at UI root spawn time.
//! - **ADR-002 Client-Server Authority**: spacing constants do not
//!   carry game state. No optimistic client-side authority is
//!   introduced.
//!
//! ## Scope (Sprint 14 story 004)
//!
//! - Friend-game scope boundary preserved. `QA-COND-0005` Standard-tier
//!   accessibility (≥44px hit-targets, focus order, keyboard
//!   navigation), `QA-COND-0006` playtest validation, and `PAW-TD-*-a`
//!   placeholder-art accept-risk are **not** advanced by this module.
//! - Viewport-driven spacing scaling is out of scope; sizes are
//!   fixed-pixel for friend-game scope.

/// 4 px. Tightest gap. Adjacent icon + numeric readout, badge padding,
/// intra-cluster spacing.
pub const SPACING_XS: f32 = 4.0;

/// 8 px. Default child gap inside a tight cluster — e.g. HUD secondary
/// row gold-icon + value, lobby form label-input adjacency.
pub const SPACING_SM: f32 = 8.0;

/// 16 px. Default gap between distinct readouts on the same strip —
/// e.g. HUD gold cluster ↔ HUD mana cluster — and the default
/// panel padding step.
pub const SPACING_MD: f32 = 16.0;

/// 24 px. Section separator inside a panel; gap between a strip's
/// left edge and its first child; primary-modal padding step.
pub const SPACING_LG: f32 = 24.0;

/// 32 px. Largest single step. Headline ↔ body separation, lobby form
/// section separator. Values above this are recomposed as `XL + MD`,
/// `XL + XL`, or as explicit padding rather than gaining a new named
/// scale step.
pub const SPACING_XL: f32 = 32.0;

/// Strictly-ascending array of every named spacing-scale constant.
/// Exposed for the ordering unit test and any future audit tooling
/// that needs to iterate the full scale in canonical order.
pub const ALL_SPACINGS_ASCENDING: [(&str, f32); 5] = [
    ("SPACING_XS", SPACING_XS),
    ("SPACING_SM", SPACING_SM),
    ("SPACING_MD", SPACING_MD),
    ("SPACING_LG", SPACING_LG),
    ("SPACING_XL", SPACING_XL),
];

/// Minimum pixel gap reserved between adjacent semantic spacing
/// levels. Used by the ordering unit test to assert that future
/// intermediate levels can be inserted without re-ordering existing
/// constants.
pub const SPACING_MIN_GAP: f32 = 2.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac2_five_named_spacings_strictly_ascending() {
        let values: Vec<f32> = ALL_SPACINGS_ASCENDING.iter().map(|(_, v)| *v).collect();
        assert_eq!(
            values.len(),
            5,
            "spacing module must export at least 5 named scale constants \
             (SPACING_XS, SPACING_SM, SPACING_MD, SPACING_LG, SPACING_XL)"
        );
        for window in values.windows(2) {
            assert!(
                window[0] < window[1],
                "spacing scale must be strictly ascending: {} < {} failed",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn ac2_canonical_scale_ordering_matches_spec_section_4() {
        // global-ui-design-spec.md §4 spec: XS < SM < MD < LG < XL.
        assert!(SPACING_XS < SPACING_SM);
        assert!(SPACING_SM < SPACING_MD);
        assert!(SPACING_MD < SPACING_LG);
        assert!(SPACING_LG < SPACING_XL);
    }

    #[test]
    fn ac2_canonical_values_match_spec_section_4() {
        // global-ui-design-spec.md §4 ratifies: 4 / 8 / 16 / 24 / 32.
        assert_eq!(SPACING_XS, 4.0);
        assert_eq!(SPACING_SM, 8.0);
        assert_eq!(SPACING_MD, 16.0);
        assert_eq!(SPACING_LG, 24.0);
        assert_eq!(SPACING_XL, 32.0);
    }

    #[test]
    fn ac2_each_spacing_resolves_to_positive_finite_f32() {
        for (name, value) in ALL_SPACINGS_ASCENDING {
            assert!(
                value > 0.0,
                "spacing token `{name}` resolved to {value}; must be > 0.0"
            );
            assert!(
                value.is_finite(),
                "spacing token `{name}` resolved to {value}; must be finite"
            );
        }
    }

    #[test]
    fn ac2_spacings_have_minimum_gap_for_future_intermediates() {
        for window in ALL_SPACINGS_ASCENDING.windows(2) {
            let (name_a, value_a) = window[0];
            let (name_b, value_b) = window[1];
            let gap = value_b - value_a;
            assert!(
                gap >= SPACING_MIN_GAP,
                "spacing gap between `{name_a}` ({value_a}) and `{name_b}` \
                 ({value_b}) is {gap}; must be ≥ SPACING_MIN_GAP ({SPACING_MIN_GAP})"
            );
        }
    }

    #[test]
    fn ac2_spacings_are_pairwise_distinct() {
        let mut values: Vec<f32> = ALL_SPACINGS_ASCENDING.iter().map(|(_, v)| *v).collect();
        let len_before = values.len();
        values.sort_by(|a, b| a.partial_cmp(b).expect("spacing must be comparable"));
        values.dedup();
        assert_eq!(
            len_before,
            values.len(),
            "every named spacing token must resolve to a distinct pixel value"
        );
    }

    #[test]
    fn hud_gold_row_gap_recomposes_through_xl_plus_md() {
        // PROMPT 802 §3.9 G2 enumerated the magic constant
        // HUD_GOLD_ROW_GAP_PX = 48.0. Story 004 migrates it to
        // SPACING_XL + SPACING_MD (32 + 16) per the spec recomposition
        // rule for values larger than XL.
        assert_eq!(
            SPACING_XL + SPACING_MD,
            48.0,
            "HUD gold-row gap must recompose as SPACING_XL + SPACING_MD = 48 px"
        );
    }
}
