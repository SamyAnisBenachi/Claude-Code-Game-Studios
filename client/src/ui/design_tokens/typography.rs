//! Named typography scale tokens — Sprint 14 Tier 0 foundation
//! (story 003 / S11-TD-UI-FONT-CONSTANTS).
//!
//! Every `bevy_ui` Text node and HUD readout in the playable client
//! requests its `font_size` from one of the six named semantic scales
//! defined here instead of embedding ad-hoc numeric literals at the
//! spawn site. PROMPT 802 §3.9 G3 surfaced that the previous codebase
//! had 14 distinct `font_size` literals spread across HUD, lobby, hand,
//! shop / auction, settings, and the result screen — with no shared
//! scale and the lobby §3.1 L6 *inverted* hierarchy where labels were
//! smaller than the data they described. This module is the single
//! source of truth that those surfaces consume.
//!
//! ## Semantic scale (smallest → largest)
//!
//! | Token       | Pixels | Canonical consumers |
//! |-------------|--------|---------------------|
//! | [`CAPTION`] | `13`   | Footnotes, micro-copy, secondary labels. |
//! | [`BODY`]    | `15`   | Default running text, labels, room-code chip, lobby buttons. |
//! | [`H3`]      | `18`   | Subheads, section labels, lobby status banner, return-to-lobby button. |
//! | [`H2`]      | `22`   | Panel titles, HUD secondary readouts (phase / round / mana / reserve). |
//! | [`H1`]      | `30`   | Screen headlines (result screen "RESULT PENDING"), HUD reserved-gold readout, connection-lost overlay headline. |
//! | [`DISPLAY`] | `40`   | HUD primary readouts (gold). |
//!
//! Each adjacent semantic level is separated by at least
//! [`SCALE_MIN_GAP`] pixels so future intermediate levels can be added
//! without re-ordering existing constants.
//!
//! ## Ratify-on-spec note (PROMPT 802 §9 producer-decision-2)
//!
//! The numeric values declared here (13 / 15 / 18 / 22 / 30 / 40) are
//! the **default values** proposed by the story-003 task brief while
//! story-007 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`, `docs/ux/global-ui-design-spec.md`)
//! is still pending authoring on `origin/main`. When that spec lands
//! the numeric values may be revised — but the **named constants** are
//! the stable contract. Consumers reference [`CAPTION`] / [`BODY`] /
//! [`H3`] / [`H2`] / [`H1`] / [`DISPLAY`] symbolically; a future
//! producer-decision-2 ratification will edit the values here, not at
//! every spawn site.
//!
//! ## Font weights
//!
//! Three semantic weight tokens ([`WEIGHT_REGULAR`] / [`WEIGHT_SEMIBOLD`]
//! / [`WEIGHT_BOLD`]) are exported alongside the size tokens. They are
//! CSS-style numeric weights — they do **not** yet rewire the loaded
//! font asset (the playable client currently uses bevy's default font).
//! Per story-003 §Scope / In Scope, mapping the weights to actual font
//! files is deferred to a follow-on story; this module commits to the
//! semantic *contract* so consumer call sites can already express
//! intent.
//!
//! ## Line-height ratio
//!
//! [`LINE_HEIGHT_DEFAULT_RATIO`] is the canonical multiplier applied to
//! a semantic-size constant when explicit line spacing is required.
//! Spawn sites must not embed ad-hoc ratios; instead they read from
//! this constant so a single source of truth governs vertical rhythm.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: typography tokens
//!   are read-only presentation primitives. They do not introduce a
//!   new `MessageReceiver` drain or shift system ordering — they are
//!   consumed only at UI root spawn time.
//! - **ADR-002 Client-Server Authority**: typography constants do not
//!   carry game state. No optimistic client-side authority is
//!   introduced.
//!
//! ## Scope (Sprint 14 story 003)
//!
//! - Friend-game scope boundary preserved. `QA-COND-0005`
//!   Standard-tier accessibility (WCAG-compliant minimum font sizes,
//!   user-controllable text scaling, contrast adjustments,
//!   screen-reader hints), `QA-COND-0006` playtest validation, and
//!   `PAW-TD-*-a` placeholder-art accept-risk are **not** advanced by
//!   this module.
//! - Responsive viewport-driven font scaling is out of scope; sizes
//!   are fixed-pixel for friend-game scope.
//! - Sprite-rendered text under `client/src/card_animations/` (e.g.
//!   floating damage numbers) is intentionally not migrated — only
//!   `bevy_ui` Text nodes are in scope per story-003 §Scope.

/// 13 px. Footnotes, micro-copy, secondary labels. Smallest semantic
/// level in the scale.
pub const CAPTION: f32 = 13.0;

/// 15 px. Default running text, labels, room-code chip, lobby
/// buttons. Reference baseline for body copy.
pub const BODY: f32 = 15.0;

/// 18 px. Subheads, section labels, lobby status banner,
/// return-to-lobby button. Default for any text that should read as a
/// minor heading.
pub const H3: f32 = 18.0;

/// 22 px. Panel titles, HUD secondary readouts (phase label, round
/// counter, current mana, reserve mana). Sits comfortably above the
/// accessibility floor `HUD_RESOURCE_TEXT_MIN_SIZE_PX = 20.0`.
pub const H2: f32 = 22.0;

/// 30 px. Screen headlines (result screen "RESULT PENDING"), HUD
/// reserved-gold readout, connection-lost overlay headline.
pub const H1: f32 = 30.0;

/// 40 px. HUD primary readouts (own gold, opponent gold). Equals the
/// accessibility floor `HUD_GOLD_TEXT_MIN_SIZE_PX = 40.0` exactly.
pub const DISPLAY: f32 = 40.0;

/// Strictly-ascending array of every named semantic-size constant.
/// Exposed for the AC1 ordering unit test and any future audit
/// tooling that needs to iterate the full scale in canonical order.
pub const ALL_SCALES_ASCENDING: [(&str, f32); 6] = [
    ("Caption", CAPTION),
    ("Body", BODY),
    ("H3", H3),
    ("H2", H2),
    ("H1", H1),
    ("Display", DISPLAY),
];

/// Minimum pixel gap reserved between adjacent semantic levels. Used
/// by the AC1 ordering unit test to assert that future intermediate
/// levels can be inserted without re-ordering existing constants.
pub const SCALE_MIN_GAP: f32 = 2.0;

/// Canonical line-height ratio applied to running text. Multiply by a
/// semantic-size constant to obtain a `Val::Px(...)` line height when
/// explicit vertical rhythm is required. Single source of truth so
/// spawn sites never embed ad-hoc ratios.
pub const LINE_HEIGHT_DEFAULT_RATIO: f32 = 1.25;

/// Regular weight (CSS-style numeric 400). Default text weight for
/// body copy and most labels.
pub const WEIGHT_REGULAR: u16 = 400;

/// SemiBold weight (CSS-style numeric 600). Subheads, emphasised
/// labels, and primary CTAs that should read heavier than body text
/// without dominating the screen.
pub const WEIGHT_SEMIBOLD: u16 = 600;

/// Bold weight (CSS-style numeric 700). Screen headlines and the HUD
/// primary readouts that must dominate visual hierarchy.
pub const WEIGHT_BOLD: u16 = 700;

/// Strictly-ascending array of the three named weight tokens.
pub const ALL_WEIGHTS_ASCENDING: [(&str, u16); 3] = [
    ("Regular", WEIGHT_REGULAR),
    ("SemiBold", WEIGHT_SEMIBOLD),
    ("Bold", WEIGHT_BOLD),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac1_six_named_semantic_sizes_strictly_ascending() {
        let values: Vec<f32> = ALL_SCALES_ASCENDING.iter().map(|(_, v)| *v).collect();
        assert_eq!(
            values.len(),
            6,
            "typography module must export at least 6 named semantic-size constants \
             (Caption, Body, H3, H2, H1, Display)"
        );
        for window in values.windows(2) {
            assert!(
                window[0] < window[1],
                "typography sizes must be strictly ascending: {} < {} failed",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn ac1_canonical_scale_ordering_matches_story_spec() {
        // Story 003 AC1 spec: Caption < Body < H3 < H2 < H1 < Display.
        assert!(CAPTION < BODY);
        assert!(BODY < H3);
        assert!(H3 < H2);
        assert!(H2 < H1);
        assert!(H1 < DISPLAY);
    }

    #[test]
    fn ac1_each_scale_resolves_to_positive_finite_f32() {
        for (name, value) in ALL_SCALES_ASCENDING {
            assert!(
                value > 0.0,
                "typography scale `{name}` resolved to {value}; must be > 0.0"
            );
            assert!(
                value.is_finite(),
                "typography scale `{name}` resolved to {value}; must be finite"
            );
        }
    }

    #[test]
    fn ac1_scale_constants_have_minimum_gap_for_future_intermediates() {
        for window in ALL_SCALES_ASCENDING.windows(2) {
            let (name_a, value_a) = window[0];
            let (name_b, value_b) = window[1];
            let gap = value_b - value_a;
            assert!(
                gap >= SCALE_MIN_GAP,
                "typography gap between `{name_a}` ({value_a}) and `{name_b}` \
                 ({value_b}) is {gap}; must be ≥ SCALE_MIN_GAP ({SCALE_MIN_GAP})"
            );
        }
    }

    #[test]
    fn ac1_scale_constants_are_pairwise_distinct() {
        let mut values: Vec<f32> = ALL_SCALES_ASCENDING.iter().map(|(_, v)| *v).collect();
        let len_before = values.len();
        values.sort_by(|a, b| {
            a.partial_cmp(b)
                .expect("typography sizes must be comparable")
        });
        values.dedup();
        assert_eq!(
            len_before,
            values.len(),
            "every named typography scale must resolve to a distinct pixel value"
        );
    }

    #[test]
    fn ac1_three_named_weights_strictly_ascending() {
        let values: Vec<u16> = ALL_WEIGHTS_ASCENDING.iter().map(|(_, v)| *v).collect();
        assert_eq!(
            values.len(),
            3,
            "typography module must export at least 3 named weight constants"
        );
        for window in values.windows(2) {
            assert!(
                window[0] < window[1],
                "typography weights must be strictly ascending: {} < {} failed",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn ac2_line_height_ratio_is_positive_finite_and_at_least_one() {
        assert!(LINE_HEIGHT_DEFAULT_RATIO > 0.0);
        assert!(LINE_HEIGHT_DEFAULT_RATIO.is_finite());
        assert!(
            LINE_HEIGHT_DEFAULT_RATIO >= 1.0,
            "LINE_HEIGHT_DEFAULT_RATIO must be ≥ 1.0 so line height is never \
             tighter than the glyph height"
        );
    }

    #[test]
    fn display_size_meets_hud_gold_accessibility_floor() {
        // Regression guard: the HUD gold readout migration aliases
        // HUD_GOLD_FONT_SIZE_PX through typography::DISPLAY. The
        // accessibility test
        // `tests/integration/hud/text_size_contrast_accessibility_test.rs`
        // asserts the rendered gold font ≥ HUD_GOLD_TEXT_MIN_SIZE_PX
        // (40.0). DISPLAY must therefore be ≥ 40.0 at all times.
        const HUD_GOLD_TEXT_MIN_SIZE_PX: f32 = 40.0;
        assert!(
            DISPLAY >= HUD_GOLD_TEXT_MIN_SIZE_PX,
            "DISPLAY ({DISPLAY}) must be ≥ HUD_GOLD_TEXT_MIN_SIZE_PX \
             ({HUD_GOLD_TEXT_MIN_SIZE_PX}) so the HUD gold readout continues to \
             satisfy the accessibility floor"
        );
    }

    #[test]
    fn h2_meets_hud_resource_accessibility_floor() {
        // Regression guard: the HUD phase / round / mana / reserve
        // readouts migrate from HUD_SECONDARY_FONT_SIZE_PX through
        // typography::H2. The accessibility test asserts the rendered
        // resource font ≥ HUD_RESOURCE_TEXT_MIN_SIZE_PX (20.0). H2
        // must therefore be ≥ 20.0 at all times.
        const HUD_RESOURCE_TEXT_MIN_SIZE_PX: f32 = 20.0;
        assert!(
            H2 >= HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            "H2 ({H2}) must be ≥ HUD_RESOURCE_TEXT_MIN_SIZE_PX \
             ({HUD_RESOURCE_TEXT_MIN_SIZE_PX}) so the HUD secondary readouts \
             continue to satisfy the accessibility floor"
        );
    }
}
