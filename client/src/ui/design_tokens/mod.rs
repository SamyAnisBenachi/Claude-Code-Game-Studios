//! Shared UI design tokens (z-layers, typography, spacing, alpha) consumed
//! across `client/src/ui/` and the presentation overlay surfaces.
//!
//! Tier 0 foundational modules introduced by Sprint 14 UI clean-pass. Each
//! token module exports named constants with stable values so that surface
//! code never embeds bare layer integers, font sizes, spacing pixels, or
//! overlay alphas inline.
//!
//! ## Modules
//!
//! - [`z_layers`] — named [`bevy::ui::GlobalZIndex`] constants for every
//!   layered surface (Sprint 14 story 002 / S11-TD-UI-ZINDEX-LAYERS).
//! - [`typography`] — named typography scale (Caption / Body / H3 / H2 /
//!   H1 / Display), font weights, and line-height ratio
//!   (Sprint 14 story 003 / S11-TD-UI-FONT-CONSTANTS).
//! - [`spacing`] — named spacing scale (`SPACING_XS` / `SM` / `MD` /
//!   `LG` / `XL`) for child gaps, padding, and inter-element margins
//!   (Sprint 14 story 004 / S11-TD-UI-FLEX-STRIPS).
//! - [`strips`] — named flex-strip composition primitives
//!   (`HeaderBar` / `LaneBar` / `HandBar` / `FooterBar`) with
//!   deterministic pixel heights and documented flex axes
//!   (Sprint 14 story 004 / S11-TD-UI-FLEX-STRIPS).
//! - [`overlays`] — named overlay alpha tokens (Dim / Scrim / Toast)
//!   for translucent UI overlays (Sprint 14 story 006 /
//!   S12-TD-UI-OVERLAY-ALPHA-TOKEN-001).
//! - [`interaction_states`] — named hover / focus / pressed / disabled
//!   visual-state tokens for clickable surfaces (Sprint 15 story 008 /
//!   S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001). Visual primitives only;
//!   per-surface migration of existing Sprint 14 button surfaces is
//!   deferred to Sprint 16+.
//! - [`card_slot`] — named card-slot primitive (kind enum, geometry,
//!   `Node` builder, image / text / hit-target inset accessors) shared
//!   across every card-painting surface (hand fan, draft initial grid,
//!   shop slot, auction featured card, board staged-ghost preview)
//!   (Sprint 16 story 009 / S12-TD-UI-CARD-SLOT-PRIMITIVE-001). Phase 1
//!   migrates only the shop slot call site; per-surface migration of the
//!   remaining four surfaces is owned by the Sprint 16+
//!   `S16-UI-CARD-SLOT-MIGRATION-*` follow-on family.
//! - [`viewport_matrix`] — layout-safety viewport matrix
//!   (`1280×720` / `1366×768` / `1920×1080`) consumed by every modal /
//!   CTA / scroll / text-fit primitive's viewport-safety test
//!   (Sprint 17 PROMPT 1181 layout-foundation row).
//! - [`modal_panel`] — modal / centred-panel content-budget primitive
//!   that computes `outer_height – chrome → body` and fails closed when
//!   the body region would clip the CTA off-screen at the smallest
//!   safety viewport (Sprint 17 PROMPT 1181).
//! - [`cta_row`] — stable CTA-row primitive that pins
//!   `flex_grow: 0 / flex_shrink: 0` so the row's pixel height is
//!   invariant under body-region flex pressure (Sprint 17 PROMPT 1181).
//! - [`scroll_region`] — body / scroll-region primitive for long
//!   modal-panel content with `flex_grow: 1 / flex_shrink: 1 /
//!   min_height: 0` and `Overflow::scroll_y()` (Sprint 17 PROMPT 1181).
//! - [`status_chip`] — status-chip vs CTA-button visual-role
//!   distinction (read-only chip ≠ interactive button)
//!   (Sprint 17 PROMPT 1181).
//! - [`text_fit`] — text-fitting / wrap-policy primitive that names the
//!   three canonical `LineBreak` modes (single-line never-wrap,
//!   word-boundary wrap, word-or-character wrap) (Sprint 17 PROMPT 1181).
//!
//! ## Scope discipline
//!
//! These tokens are layout / composition primitives. They do **not**
//! advance Standard-tier accessibility (`QA-COND-0005`), playtest
//! validation (`QA-COND-0006`), or final-art / asset-production
//! (`PAW-TD-*-a`). Friend-game scope boundary preserved.

pub mod card_slot;
pub mod cta_row;
pub mod interaction_states;
pub mod modal_panel;
pub mod overlays;
pub mod scroll_region;
pub mod spacing;
pub mod status_chip;
pub mod strips;
pub mod text_fit;
pub mod typography;
pub mod viewport_matrix;
pub mod z_layers;
