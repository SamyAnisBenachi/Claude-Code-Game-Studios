//! Named interaction-state visual primitives — Sprint 15 Tier 0
//! Should-priority adjacent row (story 008 /
//! `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`).
//!
//! Every clickable surface in the playable client (lobby Join / Create /
//! Confirm buttons; auction bid buttons; HUD action buttons; shop slot
//! purchase buttons; draft buttons) requests its hover / focus / pressed /
//! disabled visual treatment from the four named token sets defined here
//! instead of authoring per-site ad-hoc alpha / pixel literals. PROMPT 802
//! §3.9 G7 surfaced that the playable client had **no canonical
//! interaction-state primitive set**: hover, focus, pressed, and disabled
//! visual states were either absent or authored per-site, with no shared
//! token. This module is the single source of truth that future Sprint 16+
//! per-surface migration stories consume.
//!
//! ## Token sets
//!
//! | Token set | Purpose | Canonical surfaces |
//! |-----------|---------|---------------------|
//! | [`HOVER_BG_TINT_ALPHA`] / [`HOVER_BORDER_ALPHA`] | Subtle highlight applied when the pointer is over a clickable surface but no mouse button is pressed. White overlay tint + border alpha. | Lobby Join / Create / Confirm buttons; auction bid buttons; HUD action buttons; shop slot purchase buttons. |
//! | [`FOCUS_RING_COLOR`] / [`FOCUS_RING_WIDTH_PX`] / [`FOCUS_RING_OFFSET_PX`] | Keyboard / accessibility focus ring drawn when a clickable surface is the focused element via Tab navigation or equivalent. Ratifies the §7 `ACCENT` palette token; does **not** introduce a fresh RGB triple. | Any clickable surface that participates in Tab focus order. |
//! | [`PRESSED_BG_TINT_ALPHA`] / [`PRESSED_OFFSET_Y_PX`] | Depressed-state visual applied while a mouse button is held down on a clickable surface. Black overlay tint + 1-pixel press-down nudge. | All clickable surfaces above. |
//! | [`DISABLED_BG_TINT_ALPHA`] / [`DISABLED_TEXT_ALPHA`] / [`DISABLED_BORDER_ALPHA`] | Visual state applied when a clickable surface is not interactable in the current game state. Flattens saturation and dims chrome. | Auction bid button when the player already holds the lead; shop slot when the player cannot afford the unit; HUD action button when no valid target exists. |
//!
//! Each alpha token satisfies `0.0 <= alpha <= 1.0`; each pixel token
//! satisfies `0.0 <= px` with a documented upper bound per the story's
//! AC2..AC5 ranges.
//!
//! ## Ratified values (`docs/ux/global-ui-design-spec.md` §11)
//!
//! The numeric values declared here are the **ratified canonical values**
//! published by the global UI design spec amendment (`docs/ux/
//! global-ui-design-spec.md` §11 "Interaction State Primitives"). Consumers
//! reference the named constants symbolically; a future spec revision will
//! edit the values here, not at every clickable-surface call site.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: interaction-state
//!   primitives are read-only presentation tokens. They do not introduce a
//!   new [`lightyear::prelude::MessageReceiver`] drain or shift system
//!   ordering — they are consumed only at clickable-surface spawn time
//!   (and at hover / press / focus / disable state-change time, where the
//!   per-surface migration story owns the state-tracking logic).
//! - **ADR-002 Client-Server Authority**: interaction-state constants do
//!   not carry game state. No optimistic client-side authority is
//!   introduced — `pressed` / `disabled` reflect local input + UI gating
//!   only, never server authority over a game decision.
//!
//! ## Scope (Sprint 15 story 008)
//!
//! - **Friend-game scope boundary preserved.** `QA-COND-0005`
//!   Standard-tier accessibility (WCAG-compliant hover / pressed / focus
//!   contrast ratios; ≥44px hit-target enforcement; full keyboard
//!   navigation focus order; screen-reader hints; colorblind modes; text
//!   scaling), `QA-COND-0006` playtest validation, and `PAW-TD-*-a`
//!   placeholder-art accept-risk are **not** advanced by this module.
//! - **Focus-ring visual presence is friend-game scope only.** The
//!   [`FOCUS_RING_COLOR`] / [`FOCUS_RING_WIDTH_PX`] / [`FOCUS_RING_OFFSET_PX`]
//!   tokens provide a *visual* focus ring but do **not** implement full
//!   keyboard-navigation focus order, screen-reader hints, or
//!   Standard-tier focus conformance per `QA-COND-0005`. Token presence
//!   does **not** flip `QA-COND-0005` to closed.
//! - **No per-surface migration of existing Sprint 14 button surfaces.**
//!   Lobby buttons (`S11-UX-LOBBY-BUTTON-HITTARGETS` DONE), auction bid
//!   buttons (`S11-UX-AUCTION-FEATURED-CARD` DONE), HUD action buttons
//!   (`S11-UX-HUD-TOP-STRIP-LAYOUT` DONE), draft buttons, and shop slot
//!   buttons remain on their existing per-site styling for the duration of
//!   Sprint 15. Per-surface migration is a Sprint 16+ follow-on story
//!   (expected slug family `S16-UI-INTERACTION-STATE-MIGRATION-*`).
//! - **No tween / animation of state transitions.** Static visual states
//!   only. Future per-state easing (e.g. 100 ms fade-in on hover enter)
//!   is a separate scope under the not-yet-authored animation / motion
//!   spec.
//! - **No new color-palette tokens.** The four interaction-state token
//!   sets layer on top of the existing spec §7 palette
//!   (`PRIMARY` / `SURFACE_ELEVATED` / `ACCENT` / `SEMANTIC_*`); they do
//!   **not** introduce new base palette entries. [`FOCUS_RING_COLOR`]
//!   ratifies the §7 `ACCENT` token verbatim — same triple
//!   `Color::srgb(0.949, 0.788, 0.298)` — and is not a fresh RGB choice.

use bevy::prelude::Color;

// ---------------------------------------------------------------------------
// HOVER_*
// ---------------------------------------------------------------------------

/// `0.08` — hover background tint alpha (white overlay).
///
/// Applied as the alpha channel of a white `BackgroundColor` overlay
/// painted on top of the clickable surface's base palette token (`PRIMARY`,
/// `SURFACE_ELEVATED`, …) when the pointer is over the surface but no
/// mouse button is pressed. Subtle by design so the hover state reads as a
/// pointer-feedback affordance rather than a full state change.
///
/// Canonical consumers: lobby Join / Create / Confirm buttons; auction bid
/// buttons; HUD action buttons; shop slot purchase buttons. (Per-surface
/// migration deferred to Sprint 16+ — `S16-UI-INTERACTION-STATE-MIGRATION-*`.)
///
/// AC2 range: `0.0 <= alpha <= 1.0`; canonical band `0.04..=0.16`.
pub const HOVER_BG_TINT_ALPHA: f32 = 0.08;

/// `0.40` — hover border alpha.
///
/// Applied as the alpha channel of a `BorderColor` outline drawn around
/// the clickable surface when the pointer is over it. Heavier than
/// [`HOVER_BG_TINT_ALPHA`] so the border reads as a clear pointer-feedback
/// edge without overwhelming the surface fill.
///
/// Canonical consumers: same surfaces as [`HOVER_BG_TINT_ALPHA`]. Surfaces
/// that do not draw a base border may omit the hover border treatment.
///
/// AC2 range: `0.0 <= alpha <= 1.0`; canonical band `0.20..=0.60`.
pub const HOVER_BORDER_ALPHA: f32 = 0.40;

// ---------------------------------------------------------------------------
// FOCUS_*
// ---------------------------------------------------------------------------

/// Keyboard / accessibility focus-ring color — ratified `ACCENT` palette
/// token from `docs/ux/global-ui-design-spec.md` §7
/// (`Color::srgb(0.949, 0.788, 0.298)`, hex `#F2C94C`).
///
/// **Friend-game scope only.** Visual focus-ring presence does **not**
/// advance `QA-COND-0005` Standard-tier focus-order conformance, keyboard
/// navigation completeness, screen-reader hints, or any other
/// Standard-tier accessibility requirement. Token presence is a
/// *visual* primitive; it does not by itself implement Tab focus order.
///
/// This constant ratifies the §7 `ACCENT` palette triple verbatim — it is
/// **not** a fresh RGB choice. A future "colorization pass" story may
/// migrate it to read from a `client/src/ui/design_tokens/colors.rs`
/// `ACCENT` constant; that migration is downstream and out of scope for
/// this story.
///
/// AC3 verification: the triple matches `Color::srgb(0.949, 0.788,
/// 0.298)` from spec §7. The integration test enforces this.
pub const FOCUS_RING_COLOR: Color = Color::srgb(0.949, 0.788, 0.298);

/// `2.0` px — focus-ring stroke width.
///
/// Pixel width of the ring outline drawn around the focused clickable
/// surface. Wide enough to read as a deliberate focus indicator at every
/// canonical viewport (§8 6-viewport matrix); narrow enough not to
/// distort the surface's perceived size.
///
/// **Friend-game scope only.** See [`FOCUS_RING_COLOR`] for the
/// `QA-COND-0005` accept-risk boundary statement.
///
/// AC3 range: `0.0 < px <= 8.0`; canonical band `1.0..=3.0`.
pub const FOCUS_RING_WIDTH_PX: f32 = 2.0;

/// `2.0` px — focus-ring outset from the surface edge.
///
/// Pixel offset between the clickable surface's outer edge and the inner
/// edge of the focus ring. Non-zero so the ring reads as visually distinct
/// from a base border; small enough that the ring still hugs the surface.
///
/// **Friend-game scope only.** See [`FOCUS_RING_COLOR`] for the
/// `QA-COND-0005` accept-risk boundary statement.
///
/// AC3 range: `0.0 <= px <= 8.0`; canonical band `0.0..=4.0`.
pub const FOCUS_RING_OFFSET_PX: f32 = 2.0;

// ---------------------------------------------------------------------------
// PRESSED_*
// ---------------------------------------------------------------------------

/// `0.16` — pressed background tint alpha (black overlay).
///
/// Applied as the alpha channel of a black `BackgroundColor` overlay
/// painted on top of the clickable surface's base palette token while a
/// mouse button is held down on it. Twice the magnitude of
/// [`HOVER_BG_TINT_ALPHA`] so the pressed state reads as a clearly
/// distinct visual state from hover.
///
/// Canonical consumers: same clickable surfaces as [`HOVER_BG_TINT_ALPHA`].
///
/// AC4 range: `0.0 <= alpha <= 1.0`; canonical band `0.08..=0.24`.
pub const PRESSED_BG_TINT_ALPHA: f32 = 0.16;

/// `1.0` px — pressed downward nudge.
///
/// Vertical pixel offset applied to the clickable surface's content while
/// the surface is in the pressed state — a one-pixel press-down nudge so
/// the surface reads as physically depressed. Subtle by design; large
/// offsets would visibly shift the surface's bounding box and disrupt
/// neighbouring layout.
///
/// Canonical consumers: same clickable surfaces as
/// [`PRESSED_BG_TINT_ALPHA`].
///
/// AC4 range: `0.0 <= px <= 4.0`; canonical band `0.0..=2.0`.
pub const PRESSED_OFFSET_Y_PX: f32 = 1.0;

// ---------------------------------------------------------------------------
// DISABLED_*
// ---------------------------------------------------------------------------

/// `0.50` — disabled background tint alpha (black overlay).
///
/// Applied as the alpha channel of a black `BackgroundColor` overlay
/// painted on top of the clickable surface's base palette token when the
/// surface is not interactable in the current game state. Heavy enough to
/// flatten the surface's perceived saturation so the disabled state is
/// unambiguously distinct from hover / pressed / default.
///
/// Canonical consumers: auction bid button when the player already holds
/// the lead; shop slot when the player cannot afford the unit; HUD action
/// button when no valid target exists; lobby Confirm button when class
/// selection is incomplete. (Per-surface migration deferred to Sprint 16+.)
///
/// AC5 range: `0.0 <= alpha <= 1.0`; canonical band `0.30..=0.70`.
pub const DISABLED_BG_TINT_ALPHA: f32 = 0.50;

/// `0.40` — disabled text alpha multiplier.
///
/// Applied as the alpha channel of the clickable surface's label
/// `TextColor` when the surface is disabled. Sits below the
/// [`DISABLED_BG_TINT_ALPHA`] band so the label reads as faded relative to
/// an enabled surface's label without becoming unreadable.
///
/// Canonical consumers: same disabled-state surfaces as
/// [`DISABLED_BG_TINT_ALPHA`].
///
/// AC5 range: `0.0 <= alpha <= 1.0`; canonical band `0.20..=0.60`.
pub const DISABLED_TEXT_ALPHA: f32 = 0.40;

/// `0.20` — disabled border alpha.
///
/// Applied as the alpha channel of the clickable surface's `BorderColor`
/// outline when the surface is disabled. Lower than [`HOVER_BORDER_ALPHA`]
/// so the border recedes alongside the surface's flattened fill.
///
/// Canonical consumers: same disabled-state surfaces as
/// [`DISABLED_BG_TINT_ALPHA`]. Surfaces that do not draw a base border may
/// omit the disabled border treatment.
///
/// AC5 range: `0.0 <= alpha <= 1.0`; canonical band `0.10..=0.40`.
pub const DISABLED_BORDER_ALPHA: f32 = 0.20;

// ---------------------------------------------------------------------------
// Audit arrays
// ---------------------------------------------------------------------------

/// Every named alpha token exported by this module, paired with its
/// declared identifier. Exposed for AC2 / AC4 / AC5 range tests and any
/// future audit tooling that needs to iterate the full alpha set in
/// canonical order. The `FOCUS_*` family contributes no alpha entry —
/// the focus ring is opaque `ACCENT` and its visibility is controlled by
/// stroke width / offset (see [`ALL_INTERACTION_STATE_PIXELS`]).
pub const ALL_INTERACTION_STATE_ALPHAS: [(&str, f32); 6] = [
    ("HoverBgTintAlpha", HOVER_BG_TINT_ALPHA),
    ("HoverBorderAlpha", HOVER_BORDER_ALPHA),
    ("PressedBgTintAlpha", PRESSED_BG_TINT_ALPHA),
    ("DisabledBgTintAlpha", DISABLED_BG_TINT_ALPHA),
    ("DisabledTextAlpha", DISABLED_TEXT_ALPHA),
    ("DisabledBorderAlpha", DISABLED_BORDER_ALPHA),
];

/// Every named pixel token exported by this module, paired with its
/// declared identifier. Exposed for AC3 / AC4 range tests and any future
/// audit tooling.
pub const ALL_INTERACTION_STATE_PIXELS: [(&str, f32); 3] = [
    ("FocusRingWidthPx", FOCUS_RING_WIDTH_PX),
    ("FocusRingOffsetPx", FOCUS_RING_OFFSET_PX),
    ("PressedOffsetYPx", PRESSED_OFFSET_Y_PX),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac2_hover_tokens_in_documented_alpha_range() {
        for (name, value) in [
            ("HOVER_BG_TINT_ALPHA", HOVER_BG_TINT_ALPHA),
            ("HOVER_BORDER_ALPHA", HOVER_BORDER_ALPHA),
        ] {
            assert!(
                (0.0..=1.0).contains(&value),
                "AC2 hover alpha `{name}` resolved to {value}; must be in 0.0..=1.0"
            );
            assert!(
                value.is_finite(),
                "AC2 hover alpha `{name}` resolved to {value}; must be finite"
            );
        }
        // Band guard: HOVER_BG_TINT_ALPHA in 0.04..=0.16 keeps the tint
        // subtle; values outside this band would either disappear or
        // overwhelm the surface fill.
        assert!(
            (0.04..=0.16).contains(&HOVER_BG_TINT_ALPHA),
            "AC2 HOVER_BG_TINT_ALPHA ({HOVER_BG_TINT_ALPHA}) outside canonical band 0.04..=0.16"
        );
        assert!(
            (0.20..=0.60).contains(&HOVER_BORDER_ALPHA),
            "AC2 HOVER_BORDER_ALPHA ({HOVER_BORDER_ALPHA}) outside canonical band 0.20..=0.60"
        );
    }

    #[test]
    fn ac3_focus_ring_color_ratifies_spec_accent_palette_triple() {
        // The §7 `ACCENT` palette token is `Color::srgb(0.949, 0.788,
        // 0.298)` (hex #F2C94C). Ratifies the spec; not a fresh RGB.
        let expected = Color::srgb(0.949, 0.788, 0.298);
        assert_eq!(
            FOCUS_RING_COLOR, expected,
            "AC3 FOCUS_RING_COLOR must equal the spec §7 ACCENT triple \
             Color::srgb(0.949, 0.788, 0.298); not a fresh RGB choice"
        );
    }

    #[test]
    fn ac3_focus_ring_pixel_tokens_in_documented_range() {
        assert!(
            FOCUS_RING_WIDTH_PX > 0.0 && FOCUS_RING_WIDTH_PX <= 8.0,
            "AC3 FOCUS_RING_WIDTH_PX ({FOCUS_RING_WIDTH_PX}) must be in 0.0 < px <= 8.0"
        );
        assert!(
            (1.0..=3.0).contains(&FOCUS_RING_WIDTH_PX),
            "AC3 FOCUS_RING_WIDTH_PX ({FOCUS_RING_WIDTH_PX}) outside canonical band 1.0..=3.0"
        );
        assert!(
            (0.0..=8.0).contains(&FOCUS_RING_OFFSET_PX),
            "AC3 FOCUS_RING_OFFSET_PX ({FOCUS_RING_OFFSET_PX}) must be in 0.0..=8.0"
        );
        assert!(
            (0.0..=4.0).contains(&FOCUS_RING_OFFSET_PX),
            "AC3 FOCUS_RING_OFFSET_PX ({FOCUS_RING_OFFSET_PX}) outside canonical band 0.0..=4.0"
        );
    }

    #[test]
    fn ac4_pressed_tokens_in_documented_range() {
        assert!(
            (0.0..=1.0).contains(&PRESSED_BG_TINT_ALPHA),
            "AC4 PRESSED_BG_TINT_ALPHA ({PRESSED_BG_TINT_ALPHA}) must be in 0.0..=1.0"
        );
        assert!(
            (0.08..=0.24).contains(&PRESSED_BG_TINT_ALPHA),
            "AC4 PRESSED_BG_TINT_ALPHA ({PRESSED_BG_TINT_ALPHA}) outside band 0.08..=0.24"
        );
        assert!(
            (0.0..=4.0).contains(&PRESSED_OFFSET_Y_PX),
            "AC4 PRESSED_OFFSET_Y_PX ({PRESSED_OFFSET_Y_PX}) must be in 0.0..=4.0"
        );
        assert!(
            (0.0..=2.0).contains(&PRESSED_OFFSET_Y_PX),
            "AC4 PRESSED_OFFSET_Y_PX ({PRESSED_OFFSET_Y_PX}) outside canonical band 0.0..=2.0"
        );
    }

    #[test]
    fn ac5_disabled_tokens_in_documented_range() {
        for (name, value, lo, hi) in [
            ("DISABLED_BG_TINT_ALPHA", DISABLED_BG_TINT_ALPHA, 0.30, 0.70),
            ("DISABLED_TEXT_ALPHA", DISABLED_TEXT_ALPHA, 0.20, 0.60),
            ("DISABLED_BORDER_ALPHA", DISABLED_BORDER_ALPHA, 0.10, 0.40),
        ] {
            assert!(
                (0.0..=1.0).contains(&value),
                "AC5 disabled alpha `{name}` resolved to {value}; must be in 0.0..=1.0"
            );
            assert!(
                (lo..=hi).contains(&value),
                "AC5 disabled alpha `{name}` ({value}) outside canonical band {lo}..={hi}"
            );
        }
    }

    #[test]
    fn ac4_pressed_distinct_from_hover_for_visual_state_disambiguation() {
        // The visual contract: pressed reads heavier than hover so the
        // player perceives a clear state change between hover-enter and
        // mouse-down. If a future revision ever sets
        // PRESSED_BG_TINT_ALPHA <= HOVER_BG_TINT_ALPHA, the pressed state
        // would stop reading as distinct.
        assert!(
            PRESSED_BG_TINT_ALPHA > HOVER_BG_TINT_ALPHA,
            "AC4 PRESSED_BG_TINT_ALPHA ({PRESSED_BG_TINT_ALPHA}) must be > \
             HOVER_BG_TINT_ALPHA ({HOVER_BG_TINT_ALPHA}) so pressed reads heavier than hover"
        );
    }

    #[test]
    fn ac5_disabled_bg_is_heaviest_to_flatten_saturation() {
        // The visual contract: disabled reads heavier than both hover
        // and pressed so the disabled state is unambiguously
        // distinguishable from interactive states.
        assert!(
            DISABLED_BG_TINT_ALPHA > PRESSED_BG_TINT_ALPHA,
            "AC5 DISABLED_BG_TINT_ALPHA ({DISABLED_BG_TINT_ALPHA}) must be > \
             PRESSED_BG_TINT_ALPHA ({PRESSED_BG_TINT_ALPHA}) so disabled reads heavier than pressed"
        );
    }

    #[test]
    fn ac9_audit_arrays_match_published_token_counts() {
        // Audit arrays must enumerate every alpha / pixel token the
        // module publishes so future additions cannot silently bypass
        // range tests.
        assert_eq!(
            ALL_INTERACTION_STATE_ALPHAS.len(),
            6,
            "AC9 ALL_INTERACTION_STATE_ALPHAS must cover the 6 alpha tokens \
             (HOVER_BG_TINT_ALPHA, HOVER_BORDER_ALPHA, PRESSED_BG_TINT_ALPHA, \
             DISABLED_BG_TINT_ALPHA, DISABLED_TEXT_ALPHA, DISABLED_BORDER_ALPHA)"
        );
        assert_eq!(
            ALL_INTERACTION_STATE_PIXELS.len(),
            3,
            "AC9 ALL_INTERACTION_STATE_PIXELS must cover the 3 pixel tokens \
             (FOCUS_RING_WIDTH_PX, FOCUS_RING_OFFSET_PX, PRESSED_OFFSET_Y_PX)"
        );
    }

    #[test]
    fn ac2_ac4_ac5_every_audited_alpha_within_unit_interval() {
        for (name, value) in ALL_INTERACTION_STATE_ALPHAS {
            assert!(
                (0.0..=1.0).contains(&value),
                "audit alpha `{name}` resolved to {value}; must be in 0.0..=1.0"
            );
            assert!(
                value.is_finite(),
                "audit alpha `{name}` resolved to {value}; must be finite"
            );
        }
    }

    #[test]
    fn ac3_ac4_every_audited_pixel_non_negative_and_finite() {
        for (name, value) in ALL_INTERACTION_STATE_PIXELS {
            assert!(
                value >= 0.0,
                "audit pixel `{name}` resolved to {value}; must be >= 0.0"
            );
            assert!(
                value.is_finite(),
                "audit pixel `{name}` resolved to {value}; must be finite"
            );
        }
    }
}
