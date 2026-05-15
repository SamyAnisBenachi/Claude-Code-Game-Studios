//! Named overlay alpha tokens — Sprint 14 Tier 0 foundation
//! (story 006 / S12-TD-UI-OVERLAY-ALPHA-TOKEN-001).
//!
//! Every translucent overlay surface in the playable client requests
//! its alpha channel from one of the three named semantic tokens
//! defined here instead of embedding ad-hoc numeric literals at the
//! spawn site. PROMPT 802 §3.2 H4 / §3.6 A6 / §3.9 G4 surfaced that
//! the previous codebase used **three different scrim/dim alpha
//! values** for what should be a single canonical "modal scrim" effect:
//!
//! - HUD combat-focus dim: `0.45` (`ui/hud/mod.rs`)
//! - Settlement overlay scrim: `0.58` (`ui/shop_auction/mod.rs`)
//! - Result screen backdrop: `0.46` (`presentation/result_screen.rs`)
//!
//! Independently authored, no shared rationale. The visual effect was
//! that switching between game states (combat → settlement → result)
//! flickered between three different darkness levels and broke visual
//! continuity. This module is the single source of truth that those
//! surfaces consume.
//!
//! ## Semantic tokens
//!
//! | Token                   | Float  | Canonical consumers |
//! |-------------------------|--------|---------------------|
//! | [`OVERLAY_DIM_ALPHA`]   | `0.45` | HUD combat-focus dim during settlement entry; light dim where gameplay UI must remain partially legible underneath. |
//! | [`OVERLAY_SCRIM_ALPHA`] | `0.55` | Modal scrim — settlement overlay, result screen panel backdrop. Single canonical value so transitions between settlement → result no longer flicker between darkness levels. |
//! | [`OVERLAY_TOAST_ALPHA`] | `0.80` | Toast root background (shop / auction toasts, hand-full banners). Above the modal scrim so toasts read as foreground notifications. |
//!
//! Each token satisfies the invariant `0.0 < alpha < 1.0` — pure-opaque
//! (`1.0`) and fully-transparent (`0.0`) values are not overlays and
//! live elsewhere.
//!
//! ## Ratified values (`docs/ux/global-ui-design-spec.md` §6)
//!
//! The numeric values declared here (`0.45` / `0.55` / `0.80`) are the
//! **ratified canonical values** published by the global UI design
//! spec PROMPT 911 (`docs/ux/global-ui-design-spec.md` §6, integrated
//! to `origin/main` via PROMPT 912 commit `3d99a04`). Consumers
//! reference [`OVERLAY_DIM_ALPHA`] / [`OVERLAY_SCRIM_ALPHA`] /
//! [`OVERLAY_TOAST_ALPHA`] symbolically; a future spec revision will
//! edit the values here, not at every spawn site.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: overlay alpha
//!   tokens are read-only presentation primitives. They do not
//!   introduce a new [`lightyear::prelude::MessageReceiver`] drain or
//!   shift system ordering — they are consumed only at UI root spawn
//!   time (or at reconnect-rebuild time, where the rebuild path
//!   re-uses the same spawn function).
//! - **ADR-002 Client-Server Authority**: overlay alpha constants do
//!   not carry game state. No optimistic client-side authority is
//!   introduced.
//!
//! ## Scope (Sprint 14 story 006)
//!
//! - Friend-game scope boundary preserved. `QA-COND-0005`
//!   Standard-tier accessibility (WCAG-compliant overlay contrast
//!   ratios, user-controllable overlay opacity), `QA-COND-0006`
//!   playtest validation, and `PAW-TD-*-a` placeholder-art accept-risk
//!   are **not** advanced by this module.
//! - Tweened / animated alpha transitions are out of scope; values are
//!   static. Future per-state fade-in animations are a separate scope.
//! - Board ghost preview opacity is out of scope (separate row
//!   `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`, Tier 2 future
//!   candidate).
//! - HUD timer urgency color / alpha cues are out of scope (separate
//!   row `S11-UX-HUD-TIMER-URGENCY-VISUAL-001`, Tier 2 future
//!   candidate).
//! - Connection-lost overlay backdrop preserves its own intentionally
//!   lighter `0.32` value per that overlay's AC7 (see
//!   `presentation/connection_lost_overlay.rs:208`). Only the doc
//!   comment that references the result-screen `0.46` value is updated
//!   to name [`OVERLAY_SCRIM_ALPHA`] symbolically.

/// `0.45` — gameplay-focus dim. Applied as the alpha channel of a
/// near-black BackgroundColor over the gameplay UI when a state
/// transition wants to push the underlying HUD / board / hand into the
/// background without fully obscuring it.
///
/// Canonical consumer: HUD RESOLUTION dim overlay
/// (`client/src/ui/hud/mod.rs`, alias [`HUD_DIM_OVERLAY_ALPHA`]).
///
/// Visual rationale: light enough that the player retains complete
/// spatial awareness of the gameplay layer underneath while a
/// resolution animation plays. NOT a modal blocker — for that, use
/// [`OVERLAY_SCRIM_ALPHA`].
///
/// [`HUD_DIM_OVERLAY_ALPHA`]: ../../hud/constant.HUD_DIM_OVERLAY_ALPHA.html
pub const OVERLAY_DIM_ALPHA: f32 = 0.45;

/// `0.55` — modal scrim. Applied as the alpha channel of a near-black
/// BackgroundColor on root overlays that block player interaction
/// with the underlying gameplay layer (settlement overlay, result
/// screen panel backdrop).
///
/// Canonical consumers:
/// - Shop auction settlement overlay
///   (`client/src/ui/shop_auction/mod.rs`, settlement overlay root).
/// - Result screen panel backdrop
///   (`client/src/presentation/result_screen.rs`, [`ResultScreenRoot`]).
///
/// Visual rationale: a single value that splits the difference between
/// the pre-migration `0.46` (result-screen) and `0.58` (settlement)
/// values so transitions between settlement → result no longer flicker
/// between three darkness levels. Heavy enough to read as a modal
/// blocker; light enough that the player retains spatial awareness of
/// the underlying state.
///
/// Connection-lost overlay (`connection_lost_overlay.rs:208`) keeps its
/// own intentionally lighter `0.32` value per that overlay's AC7. The
/// comment at `:205-207` references this constant by name even though
/// the literal value is preserved as a documented exception.
///
/// [`ResultScreenRoot`]: ../../../presentation/result_screen/struct.ResultScreenRoot.html
pub const OVERLAY_SCRIM_ALPHA: f32 = 0.55;

/// `0.80` — toast / notification scrim. Applied as the alpha channel
/// of a tinted BackgroundColor on toast / banner roots that surface
/// transient notifications above the modal scrim layer.
///
/// Canonical consumers: shop / auction toast root, hand-full banner
/// (`client/src/ui/shop_auction/mod.rs`). The current toast spawn does
/// not yet set a BackgroundColor on the toast root itself (the toast
/// text is the only visible element); this token is reserved for the
/// future toast root background migration that the global-ui spec §6
/// names as the canonical site.
///
/// Visual rationale: sits above [`OVERLAY_SCRIM_ALPHA`] so the toast
/// reads as a foreground notification rather than a modal blocker.
pub const OVERLAY_TOAST_ALPHA: f32 = 0.80;

/// Strictly-ascending array of every named overlay-alpha constant.
/// Exposed for the AC1 ordering / range unit test and any future audit
/// tooling that needs to iterate the full overlay-alpha scale in
/// canonical order.
pub const ALL_OVERLAY_ALPHAS_ASCENDING: [(&str, f32); 3] = [
    ("OverlayDim", OVERLAY_DIM_ALPHA),
    ("OverlayScrim", OVERLAY_SCRIM_ALPHA),
    ("OverlayToast", OVERLAY_TOAST_ALPHA),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac1_three_named_overlay_alphas_present() {
        assert_eq!(
            ALL_OVERLAY_ALPHAS_ASCENDING.len(),
            3,
            "overlays module must export at least 3 named overlay-alpha constants \
             (OverlayDim, OverlayScrim, OverlayToast)"
        );
    }

    #[test]
    fn ac1_every_overlay_alpha_is_strictly_between_zero_and_one() {
        for (name, value) in ALL_OVERLAY_ALPHAS_ASCENDING {
            assert!(
                value > 0.0,
                "overlay alpha `{name}` resolved to {value}; must be > 0.0 \
                 (pure-transparent values are not overlays)"
            );
            assert!(
                value < 1.0,
                "overlay alpha `{name}` resolved to {value}; must be < 1.0 \
                 (pure-opaque values are not overlays)"
            );
            assert!(
                value.is_finite(),
                "overlay alpha `{name}` resolved to {value}; must be finite"
            );
        }
    }

    #[test]
    fn ac1_overlay_alphas_strictly_ascending_dim_lt_scrim_lt_toast() {
        // Story-006 visual hierarchy: dim < scrim < toast so the toast
        // reads above the scrim and the scrim reads above the focus dim.
        let values: Vec<f32> = ALL_OVERLAY_ALPHAS_ASCENDING
            .iter()
            .map(|(_, v)| *v)
            .collect();
        for window in values.windows(2) {
            assert!(
                window[0] < window[1],
                "overlay alphas must be strictly ascending dim < scrim < toast: \
                 {} < {} failed",
                window[0],
                window[1],
            );
        }
    }

    #[test]
    fn ac1_overlay_alphas_pairwise_distinct() {
        let mut values: Vec<f32> = ALL_OVERLAY_ALPHAS_ASCENDING
            .iter()
            .map(|(_, v)| *v)
            .collect();
        let len_before = values.len();
        values.sort_by(|a, b| a.partial_cmp(b).expect("overlay alphas must be comparable"));
        values.dedup();
        assert_eq!(
            len_before,
            values.len(),
            "every named overlay alpha must resolve to a distinct value"
        );
    }

    #[test]
    fn ac1_overlay_dim_alpha_matches_spec_ratified_value() {
        // Ratified at docs/ux/global-ui-design-spec.md §6 (PROMPT 911,
        // integrated at PROMPT 912 commit 3d99a04). Preserves the
        // pre-migration HUD dim alpha so the HUD RESOLUTION dim
        // overlay's visual intent is unchanged.
        assert!(
            (OVERLAY_DIM_ALPHA - 0.45).abs() < f32::EPSILON,
            "OVERLAY_DIM_ALPHA ({OVERLAY_DIM_ALPHA}) must equal the spec-ratified \
             0.45 to preserve HUD dim visual intent"
        );
    }

    #[test]
    fn ac1_overlay_scrim_alpha_matches_spec_ratified_value() {
        // Ratified at docs/ux/global-ui-design-spec.md §6. Splits the
        // difference between the pre-migration 0.46 (result) and 0.58
        // (settlement) values.
        assert!(
            (OVERLAY_SCRIM_ALPHA - 0.55).abs() < f32::EPSILON,
            "OVERLAY_SCRIM_ALPHA ({OVERLAY_SCRIM_ALPHA}) must equal the \
             spec-ratified 0.55"
        );
    }

    #[test]
    fn ac1_overlay_toast_alpha_matches_spec_ratified_value() {
        // Ratified at docs/ux/global-ui-design-spec.md §6.
        assert!(
            (OVERLAY_TOAST_ALPHA - 0.80).abs() < f32::EPSILON,
            "OVERLAY_TOAST_ALPHA ({OVERLAY_TOAST_ALPHA}) must equal the \
             spec-ratified 0.80"
        );
    }

    #[test]
    fn ac7_scrim_is_heavier_than_dim_for_visual_modal_blocker() {
        // The visual contract: the modal scrim must read as "blocking"
        // relative to the focus dim. If a future spec revision ever
        // lowers OVERLAY_SCRIM_ALPHA below OVERLAY_DIM_ALPHA, the
        // settlement / result modal overlays would stop reading as
        // modal blockers. Guard against that.
        assert!(
            OVERLAY_SCRIM_ALPHA > OVERLAY_DIM_ALPHA,
            "OVERLAY_SCRIM_ALPHA ({OVERLAY_SCRIM_ALPHA}) must be > \
             OVERLAY_DIM_ALPHA ({OVERLAY_DIM_ALPHA}) so the modal scrim reads \
             heavier than the gameplay focus dim"
        );
    }

    #[test]
    fn ac7_toast_is_heavier_than_scrim_for_foreground_notification() {
        // The visual contract: a toast above the scrim layer must read
        // as foreground notification, not as another modal blocker.
        assert!(
            OVERLAY_TOAST_ALPHA > OVERLAY_SCRIM_ALPHA,
            "OVERLAY_TOAST_ALPHA ({OVERLAY_TOAST_ALPHA}) must be > \
             OVERLAY_SCRIM_ALPHA ({OVERLAY_SCRIM_ALPHA}) so toasts read above \
             the modal scrim layer"
        );
    }
}
