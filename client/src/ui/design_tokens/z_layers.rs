//! Named UI z-layer constants — Sprint 14 Tier 0 foundation
//! (story 002 / S11-TD-UI-ZINDEX-LAYERS).
//!
//! Every UI root, overlay, modal, drag ghost, and toast in the playable
//! client spawns into one of these layers via [`bevy::ui::GlobalZIndex`]
//! instead of relying on spawn-order. Reconnect / snapshot rebuild / late
//! message replay can respawn UI roots in arbitrary order; with explicit
//! layer assignment the effective paint order matches the named hierarchy
//! rather than the implicit spawn-order.
//!
//! ## Layer hierarchy (lowest → highest)
//!
//! | Layer | `GlobalZIndex` | Canonical surfaces |
//! |-------|---------------|--------------------|
//! | [`BACKGROUND`] | `0`   | Background fills (clears, ambient backdrops). |
//! | [`WORLD`]      | `100` | World-space board content (sprite Transform.z reference; not a bevy_ui consumer — see ADR-021 §R2). |
//! | [`UNITS`]      | `200` | Unit / objective sprites above world layer (sprite Transform.z reference; not a bevy_ui consumer). |
//! | [`UI_BASE`]    | `300` | Foundational bevy_ui roots: lobby root, HUD root, hand fan root, shop/auction root, settings root. |
//! | [`UI_OVERLAY`] | `400` | Translucent overlays painted above the UI base: HUD dim, settlement scrim, draft-initial objective overlay, drag ghost, connection-lost overlay. |
//! | [`MODAL`]      | `500` | Centred modal panels that demand player attention: result screen, photosensitivity warning, settings shell. |
//! | [`TOAST`]      | `600` | Transient notifications painted above modals: shop / auction toasts. |
//! | [`DEBUG`]      | `700` | Diagnostic / dev-only overlays (not shipped in release builds). |
//!
//! Each adjacent layer is separated by 100 integer units, leaving headroom
//! for future intermediate layers without re-ordering existing constants.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture** (R2): world-space sprites
//!   render below bevy_ui. The `BACKGROUND` / `WORLD` / `UNITS` constants
//!   document the conceptual order for sprite Transform.z values; they are
//!   not direct bevy_ui consumers. The `PresentationPlugin` composition
//!   order (CardAnimations → BoardRendering → HandUi → Hud → ShopAuctionUi)
//!   remains the authoritative load-order; this module does not change
//!   that.
//! - **ADR-002 Client-Server Authority**: layer constants are read-only
//!   presentation primitives. No optimistic client-side authority is
//!   introduced.
//!
//! ## Scope (Sprint 14 story 002)
//!
//! - Friend-game scope boundary preserved. `QA-COND-0005` Standard-tier
//!   accessibility, `QA-COND-0006` playtest validation, and `PAW-TD-*-a`
//!   placeholder-art accept-risk are **not** advanced by this module.
//! - This module does not redesign the layer hierarchy or change sprite
//!   z-order under `client/src/presentation/board_rendering.rs`.

use bevy::ui::GlobalZIndex;

/// Background fills (clears, ambient backdrops). Reserved for the lowest
/// painted layer; in practice bevy_ui surfaces do not spawn here.
pub const BACKGROUND: GlobalZIndex = GlobalZIndex(0);

/// World-space board content (sprite `Transform.z` reference). Per ADR-021
/// §R2 board sprites render below bevy_ui regardless of this value; the
/// constant exists for documentation and future cross-layer audits.
pub const WORLD: GlobalZIndex = GlobalZIndex(100);

/// Unit / objective sprites painted above the world layer (sprite
/// `Transform.z` reference). Not a bevy_ui consumer.
pub const UNITS: GlobalZIndex = GlobalZIndex(200);

/// Foundational bevy_ui roots: lobby root, HUD root, hand fan root,
/// shop / auction UI root. The default layer for any surface that is part
/// of the steady-state gameplay HUD.
pub const UI_BASE: GlobalZIndex = GlobalZIndex(300);

/// Translucent overlays painted above the UI base — HUD resolution dim,
/// settlement scrim, draft-initial objective overlay, hand drag ghost,
/// proactive connection-lost overlay. Sits below modals so that a modal
/// (e.g. result screen, photosensitivity warning) always wins focus.
pub const UI_OVERLAY: GlobalZIndex = GlobalZIndex(400);

/// Centred modal panels that demand player attention — result screen,
/// photosensitivity warning, accessibility settings shell. Modals paint
/// above translucent overlays.
pub const MODAL: GlobalZIndex = GlobalZIndex(500);

/// Transient notifications painted above modals — shop / auction toasts,
/// hand-full banners. Toasts are short-lived and should never be occluded
/// by a modal.
pub const TOAST: GlobalZIndex = GlobalZIndex(600);

/// Diagnostic / dev-only overlays. Reserved for the highest painted layer
/// so dev tooling sits above all production UI; not shipped in release
/// builds.
pub const DEBUG: GlobalZIndex = GlobalZIndex(700);

/// Strictly-ascending list of every named layer, lowest → highest.
/// Exposed for layer-ordering tests and any future audit tooling that
/// needs to iterate every named layer in canonical order.
pub const ALL_LAYERS_ASCENDING: [(&str, GlobalZIndex); 8] = [
    ("Background", BACKGROUND),
    ("World", WORLD),
    ("Units", UNITS),
    ("UiBase", UI_BASE),
    ("UiOverlay", UI_OVERLAY),
    ("Modal", MODAL),
    ("Toast", TOAST),
    ("Debug", DEBUG),
];

/// Minimum integer gap reserved between adjacent layers. Used by the
/// ordering tests to assert that future intermediate layers can be
/// inserted without re-ordering existing constants.
pub const LAYER_MIN_GAP: i32 = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac1_layer_constants_are_strictly_ascending() {
        let values: Vec<i32> = ALL_LAYERS_ASCENDING
            .iter()
            .map(|(_, layer)| layer.0)
            .collect();
        for window in values.windows(2) {
            assert!(
                window[0] < window[1],
                "z-layer constants must be strictly ascending: {:?} not < {:?}",
                window[0],
                window[1],
            );
        }
    }

    #[test]
    fn ac1_layer_constants_have_minimum_gap_for_future_intermediates() {
        for window in ALL_LAYERS_ASCENDING.windows(2) {
            let (name_a, layer_a) = window[0];
            let (name_b, layer_b) = window[1];
            let gap = layer_b.0 - layer_a.0;
            assert!(
                gap >= LAYER_MIN_GAP,
                "z-layer gap between {name_a} ({}) and {name_b} ({}) is {gap}; \
                 must be ≥ LAYER_MIN_GAP ({LAYER_MIN_GAP}) to allow future intermediate layers",
                layer_a.0,
                layer_b.0,
            );
        }
    }

    #[test]
    fn ac1_layer_constants_are_pairwise_distinct() {
        let mut values: Vec<i32> = ALL_LAYERS_ASCENDING
            .iter()
            .map(|(_, layer)| layer.0)
            .collect();
        let len_before = values.len();
        values.sort_unstable();
        values.dedup();
        assert_eq!(
            len_before,
            values.len(),
            "every named z-layer must resolve to a distinct integer"
        );
    }

    #[test]
    fn ac1_named_set_covers_at_least_eight_canonical_layers() {
        assert!(
            ALL_LAYERS_ASCENDING.len() >= 8,
            "z-layer module must export at least 8 named layers (Background, World, \
             Units, UiBase, UiOverlay, Modal, Toast, Debug)"
        );
    }

    #[test]
    fn ac1_canonical_layer_ordering_matches_story_spec() {
        // Story 002 AC1 spec: Background < World < Units < UiBase <
        // UiOverlay < Modal < Toast < Debug.
        assert!(BACKGROUND.0 < WORLD.0);
        assert!(WORLD.0 < UNITS.0);
        assert!(UNITS.0 < UI_BASE.0);
        assert!(UI_BASE.0 < UI_OVERLAY.0);
        assert!(UI_OVERLAY.0 < MODAL.0);
        assert!(MODAL.0 < TOAST.0);
        assert!(TOAST.0 < DEBUG.0);
    }

    #[test]
    fn ac4_modal_is_above_ui_overlay_so_result_screen_wins_over_conn_lost() {
        // Pre-migration invariant: the result screen (`GlobalZIndex(100)`)
        // sat above the connection-lost overlay (`GlobalZIndex(90)`). Post-
        // migration the result screen uses MODAL and the connection-lost
        // overlay uses UI_OVERLAY — the same relative order must hold so
        // the visual stack is preserved.
        assert!(
            MODAL.0 > UI_OVERLAY.0,
            "MODAL ({}) must paint above UI_OVERLAY ({}) so the result screen \
             continues to win over the connection-lost overlay on GameOver",
            MODAL.0,
            UI_OVERLAY.0,
        );
    }
}
