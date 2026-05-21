pub mod asset_wiring;
pub mod audio;
pub mod card_animations;
pub mod network;
pub mod presentation;
pub mod state;
pub mod ui;

// PROMPT 1595 -- autoplay/automation harness (dev-only, low-level input).
// Gated by the `autoplay-remote` Cargo feature AND the `CCGS_AUTOPLAY=1` env
// var at runtime. See `docs/autoplay.md`.
#[cfg(feature = "autoplay-remote")]
pub mod autoplay;
