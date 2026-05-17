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
//!
//! ## Scope discipline
//!
//! These tokens are layout / composition primitives. They do **not**
//! advance Standard-tier accessibility (`QA-COND-0005`), playtest
//! validation (`QA-COND-0006`), or final-art / asset-production
//! (`PAW-TD-*-a`). Friend-game scope boundary preserved.

pub mod interaction_states;
pub mod overlays;
pub mod spacing;
pub mod strips;
pub mod typography;
pub mod z_layers;
