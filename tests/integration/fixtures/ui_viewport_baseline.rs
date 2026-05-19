//! Sprint 14 / Story 005 — viewport-invariant baseline fixture.
//!
//! # DEPRECATED — superseded by live-spawn harness
//!
//! **Status**: DEPRECATED as of Sprint 18 / PROMPT 1333 (S18-UI-VIEWPORT-
//! INVARIANT-LIVE-HARNESS-001 AC8 discharge).
//!
//! **Replacement**: `tests/integration/ui_viewport_live_test.rs` — the
//! live-spawn harness landed by PROMPT 1185 builds a production-faithful
//! Bevy `App` with `LobbyUiPlugin` + `bevy::ui::UiPlugin`, drives ≥3
//! layout-convergence ticks, and queries real `(GlobalTransform,
//! ComputedNode)` against the camera viewport across the canonical
//! 7-entry matrix. That harness is the authoritative regression guard.
//!
//! **Why deprecated, not deleted**: PROMPT 1180 §RC-5 audit ("the
//! harness reads a hand-authored `PROVISIONAL_BASELINE` fixture and
//! asserts the baseline against itself - it cannot detect the live
//! overlaps that the 2026-05-18 snapshot batch shows") flagged this
//! fixture as the load-bearing piece of the false-confidence loop. The
//! file is preserved (not deleted) so the legacy
//! `ui_viewport_invariants_test.rs` bin keeps compiling as a fixture-
//! parser sanity check until the `[[test]]` entry in `client/Cargo.toml`
//! can be removed in a follow-on prompt (PROMPT 1333 forbidden list:
//! `client/**`, `Cargo.*`). Direct dependency on these constants from
//! new code is gated by `#[deprecated]` so any fresh `use` statement
//! against `PROVISIONAL_BASELINE` produces a compiler warning that
//! routes the reader to `ui_viewport_live_test.rs`.
//!
//! Records the expected anchor position, kind, z-layer, and per-viewport
//! bounding rectangle for every UI surface tested by
//! `tests/integration/ui_viewport_invariants_test.rs`. The fixture is
//! Rust source (instead of an external `.ron` / `.json` file) so the
//! integration-test crate avoids a new runtime parser dependency; the
//! story's "Likely Files Touched" table explicitly leaves the fixture
//! format `TBD by the worker`.
//!
//! ## Provisional vs ratified values
//!
//! Story 005 §"Dependencies / Sequencing" notes that story 007
//! (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) ratifies the canonical strip
//! heights and surface dimensions. Story 007 has NOT landed on
//! `origin/main` at the time this fixture is authored (Sprint 14 source
//! of truth is `origin/main@36c0b4b` — PROMPT 902 integration of story
//! 002 z-layers only). The values below are therefore **provisional**:
//! they satisfy the four invariant classes (no-overlap, no-clipping,
//! anchor-stability, deterministic strip height) at every viewport in
//! the canonical matrix and so exercise the assertion logic, but the
//! pixel values themselves carry no design authority. When story 007
//! lands, this fixture's constants are ratified against story 007 §4
//! (spacing) / §5 (typography) / §6 (overlay alpha) — any divergence is
//! a one-line constant update.
//!
//! ## Surface taxonomy
//!
//! The fixture lists 9 entries corresponding to the 8 surface families
//! named by story 005 §AC1 (lobby, HUD, hand UI, draft centered modal,
//! shop panel, auction panel, settlement overlay, result screen). The
//! HUD family is decomposed into its two strip primitives — HeaderBar
//! (top-strip; story 015) and FooterBar (bottom-strip; story 016) — so
//! the strip-height invariant covers each independently. HandBar is the
//! hand-UI surface itself, sized for the hand-fan strip (story 004).
//!
//! ## Strip column composition
//!
//! Strips stack from the top of the viewport: HeaderBar (60px) at y=0;
//! center play area; FooterBar (40px) immediately above HandBar
//! (180px) at the bottom. Each strip has a deterministic pixel height
//! that is identical across every viewport in the canonical matrix
//! (story 005 §"Deterministic strip heights" invariant + story 004
//! flex-strip primitive contract).
//!
//! ## ADR alignment
//!
//! - **ADR-002 Client-Server Authority**: the fixture is a read-only
//!   geometry table consumed only by the test bin. No client-side
//!   optimistic authority is introduced.
//! - **ADR-021 Presentation Layer Architecture**: the z-layer column
//!   matches story 002's named [`bevy::ui::GlobalZIndex`] hierarchy;
//!   overlay / modal layers are flagged so the no-overlap rule
//!   excludes them.
//!
//! ## Friend-game scope preserved
//!
//! No accept-risk disposition is advanced. `QA-COND-0005` (Standard-tier
//! accessibility), `QA-COND-0006` (playtest validation), and
//! `PAW-TD-*-a` (placeholder-art) remain unchanged.

#![allow(dead_code)]

use super::ui_viewport::{
    DisplayPhase, ProportionalAnchor, SurfaceBaseline, SurfaceKind, ViewportBaseline, ZLayer,
};

/// Deterministic HeaderBar height (story 004 + 015 strip primitive). 60px
/// across every viewport.
pub const HEADER_BAR_HEIGHT_PX: f32 = 60.0;

/// Deterministic FooterBar height (story 004 + 016 strip primitive). 40px
/// across every viewport.
pub const FOOTER_BAR_HEIGHT_PX: f32 = 40.0;

/// Deterministic HandBar height (story 004 hand-fan strip). 180px across
/// every viewport.
pub const HAND_BAR_HEIGHT_PX: f32 = 180.0;

/// Provisional draft centered modal dimensions (story 015 shop-auction).
pub const DRAFT_MODAL_WIDTH_PX: f32 = 800.0;
pub const DRAFT_MODAL_HEIGHT_PX: f32 = 300.0;

/// Provisional shop panel dimensions (story 016 shop-auction).
pub const SHOP_PANEL_WIDTH_PX: f32 = 1000.0;
pub const SHOP_PANEL_HEIGHT_PX: f32 = 300.0;

/// Provisional auction panel dimensions (story 016 shop-auction).
pub const AUCTION_PANEL_WIDTH_PX: f32 = 700.0;
pub const AUCTION_PANEL_HEIGHT_PX: f32 = 300.0;

/// Provisional result screen modal dimensions (presentation layer).
pub const RESULT_SCREEN_WIDTH_PX: f32 = 600.0;
pub const RESULT_SCREEN_HEIGHT_PX: f32 = 400.0;

// ─── Surface 1: lobby_root ─────────────────────────────────────────────
// Full-viewport anchor (0.0, 0.0). Per producer-decision-3 (story 024)
// the lobby may resolve as a centred modal or full-viewport hero; the
// provisional fixture models the full-viewport hero (option B) because
// it satisfies anchor stability without a centred-modal width.

const LOBBY_ROOT_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    ("1366x768", 0.0, 0.0, 1366.0, 768.0),
    ("1920x1080", 0.0, 0.0, 1920.0, 1080.0),
    ("1920x1200", 0.0, 0.0, 1920.0, 1200.0),
    ("1280x960", 0.0, 0.0, 1280.0, 960.0),
    ("3840x2160", 0.0, 0.0, 3840.0, 2160.0),
    ("2560x1080", 0.0, 0.0, 2560.0, 1080.0),
];

// ─── Surface 2: hud_header_bar (story 004 HeaderBar strip) ──────────────
// Top strip; y=0, h=HEADER_BAR_HEIGHT_PX, full viewport width.

const HUD_HEADER_BAR_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    ("1366x768", 0.0, 0.0, 1366.0, HEADER_BAR_HEIGHT_PX),
    ("1920x1080", 0.0, 0.0, 1920.0, HEADER_BAR_HEIGHT_PX),
    ("1920x1200", 0.0, 0.0, 1920.0, HEADER_BAR_HEIGHT_PX),
    ("1280x960", 0.0, 0.0, 1280.0, HEADER_BAR_HEIGHT_PX),
    ("3840x2160", 0.0, 0.0, 3840.0, HEADER_BAR_HEIGHT_PX),
    ("2560x1080", 0.0, 0.0, 2560.0, HEADER_BAR_HEIGHT_PX),
];

// ─── Surface 3: hud_footer_bar (story 004 FooterBar strip) ──────────────
// Bottom strip immediately above HandBar; y = vh - HAND_HEIGHT -
// FOOTER_HEIGHT, h = FOOTER_BAR_HEIGHT_PX, full viewport width.

const HUD_FOOTER_BAR_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    (
        "1366x768",
        0.0,
        768.0 - HAND_BAR_HEIGHT_PX - FOOTER_BAR_HEIGHT_PX,
        1366.0,
        FOOTER_BAR_HEIGHT_PX,
    ),
    (
        "1920x1080",
        0.0,
        1080.0 - HAND_BAR_HEIGHT_PX - FOOTER_BAR_HEIGHT_PX,
        1920.0,
        FOOTER_BAR_HEIGHT_PX,
    ),
    (
        "1920x1200",
        0.0,
        1200.0 - HAND_BAR_HEIGHT_PX - FOOTER_BAR_HEIGHT_PX,
        1920.0,
        FOOTER_BAR_HEIGHT_PX,
    ),
    (
        "1280x960",
        0.0,
        960.0 - HAND_BAR_HEIGHT_PX - FOOTER_BAR_HEIGHT_PX,
        1280.0,
        FOOTER_BAR_HEIGHT_PX,
    ),
    (
        "3840x2160",
        0.0,
        2160.0 - HAND_BAR_HEIGHT_PX - FOOTER_BAR_HEIGHT_PX,
        3840.0,
        FOOTER_BAR_HEIGHT_PX,
    ),
    (
        "2560x1080",
        0.0,
        1080.0 - HAND_BAR_HEIGHT_PX - FOOTER_BAR_HEIGHT_PX,
        2560.0,
        FOOTER_BAR_HEIGHT_PX,
    ),
];

// ─── Surface 4: hand_ui_hand_bar (story 004 HandBar strip) ──────────────
// Bottom strip; y = vh - HAND_HEIGHT, h = HAND_BAR_HEIGHT_PX, full width.

const HAND_UI_HAND_BAR_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    (
        "1366x768",
        0.0,
        768.0 - HAND_BAR_HEIGHT_PX,
        1366.0,
        HAND_BAR_HEIGHT_PX,
    ),
    (
        "1920x1080",
        0.0,
        1080.0 - HAND_BAR_HEIGHT_PX,
        1920.0,
        HAND_BAR_HEIGHT_PX,
    ),
    (
        "1920x1200",
        0.0,
        1200.0 - HAND_BAR_HEIGHT_PX,
        1920.0,
        HAND_BAR_HEIGHT_PX,
    ),
    (
        "1280x960",
        0.0,
        960.0 - HAND_BAR_HEIGHT_PX,
        1280.0,
        HAND_BAR_HEIGHT_PX,
    ),
    (
        "3840x2160",
        0.0,
        2160.0 - HAND_BAR_HEIGHT_PX,
        3840.0,
        HAND_BAR_HEIGHT_PX,
    ),
    (
        "2560x1080",
        0.0,
        1080.0 - HAND_BAR_HEIGHT_PX,
        2560.0,
        HAND_BAR_HEIGHT_PX,
    ),
];

// ─── Surface 5: draft_centered_modal (story 015 shop-auction-ui) ────────
// Centered on viewport. x = (vw - w) / 2; y = (vh - h) / 2.

const DRAFT_CENTERED_MODAL_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    (
        "1366x768",
        (1366.0 - DRAFT_MODAL_WIDTH_PX) / 2.0,
        (768.0 - DRAFT_MODAL_HEIGHT_PX) / 2.0,
        DRAFT_MODAL_WIDTH_PX,
        DRAFT_MODAL_HEIGHT_PX,
    ),
    (
        "1920x1080",
        (1920.0 - DRAFT_MODAL_WIDTH_PX) / 2.0,
        (1080.0 - DRAFT_MODAL_HEIGHT_PX) / 2.0,
        DRAFT_MODAL_WIDTH_PX,
        DRAFT_MODAL_HEIGHT_PX,
    ),
    (
        "1920x1200",
        (1920.0 - DRAFT_MODAL_WIDTH_PX) / 2.0,
        (1200.0 - DRAFT_MODAL_HEIGHT_PX) / 2.0,
        DRAFT_MODAL_WIDTH_PX,
        DRAFT_MODAL_HEIGHT_PX,
    ),
    (
        "1280x960",
        (1280.0 - DRAFT_MODAL_WIDTH_PX) / 2.0,
        (960.0 - DRAFT_MODAL_HEIGHT_PX) / 2.0,
        DRAFT_MODAL_WIDTH_PX,
        DRAFT_MODAL_HEIGHT_PX,
    ),
    (
        "3840x2160",
        (3840.0 - DRAFT_MODAL_WIDTH_PX) / 2.0,
        (2160.0 - DRAFT_MODAL_HEIGHT_PX) / 2.0,
        DRAFT_MODAL_WIDTH_PX,
        DRAFT_MODAL_HEIGHT_PX,
    ),
    (
        "2560x1080",
        (2560.0 - DRAFT_MODAL_WIDTH_PX) / 2.0,
        (1080.0 - DRAFT_MODAL_HEIGHT_PX) / 2.0,
        DRAFT_MODAL_WIDTH_PX,
        DRAFT_MODAL_HEIGHT_PX,
    ),
];

// ─── Surface 6: shop_panel (story 016 shop-auction-ui) ──────────────────

const SHOP_PANEL_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    (
        "1366x768",
        (1366.0 - SHOP_PANEL_WIDTH_PX) / 2.0,
        (768.0 - SHOP_PANEL_HEIGHT_PX) / 2.0,
        SHOP_PANEL_WIDTH_PX,
        SHOP_PANEL_HEIGHT_PX,
    ),
    (
        "1920x1080",
        (1920.0 - SHOP_PANEL_WIDTH_PX) / 2.0,
        (1080.0 - SHOP_PANEL_HEIGHT_PX) / 2.0,
        SHOP_PANEL_WIDTH_PX,
        SHOP_PANEL_HEIGHT_PX,
    ),
    (
        "1920x1200",
        (1920.0 - SHOP_PANEL_WIDTH_PX) / 2.0,
        (1200.0 - SHOP_PANEL_HEIGHT_PX) / 2.0,
        SHOP_PANEL_WIDTH_PX,
        SHOP_PANEL_HEIGHT_PX,
    ),
    (
        "1280x960",
        (1280.0 - SHOP_PANEL_WIDTH_PX) / 2.0,
        (960.0 - SHOP_PANEL_HEIGHT_PX) / 2.0,
        SHOP_PANEL_WIDTH_PX,
        SHOP_PANEL_HEIGHT_PX,
    ),
    (
        "3840x2160",
        (3840.0 - SHOP_PANEL_WIDTH_PX) / 2.0,
        (2160.0 - SHOP_PANEL_HEIGHT_PX) / 2.0,
        SHOP_PANEL_WIDTH_PX,
        SHOP_PANEL_HEIGHT_PX,
    ),
    (
        "2560x1080",
        (2560.0 - SHOP_PANEL_WIDTH_PX) / 2.0,
        (1080.0 - SHOP_PANEL_HEIGHT_PX) / 2.0,
        SHOP_PANEL_WIDTH_PX,
        SHOP_PANEL_HEIGHT_PX,
    ),
];

// ─── Surface 7: auction_panel (story 016 shop-auction-ui) ───────────────

const AUCTION_PANEL_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    (
        "1366x768",
        (1366.0 - AUCTION_PANEL_WIDTH_PX) / 2.0,
        (768.0 - AUCTION_PANEL_HEIGHT_PX) / 2.0,
        AUCTION_PANEL_WIDTH_PX,
        AUCTION_PANEL_HEIGHT_PX,
    ),
    (
        "1920x1080",
        (1920.0 - AUCTION_PANEL_WIDTH_PX) / 2.0,
        (1080.0 - AUCTION_PANEL_HEIGHT_PX) / 2.0,
        AUCTION_PANEL_WIDTH_PX,
        AUCTION_PANEL_HEIGHT_PX,
    ),
    (
        "1920x1200",
        (1920.0 - AUCTION_PANEL_WIDTH_PX) / 2.0,
        (1200.0 - AUCTION_PANEL_HEIGHT_PX) / 2.0,
        AUCTION_PANEL_WIDTH_PX,
        AUCTION_PANEL_HEIGHT_PX,
    ),
    (
        "1280x960",
        (1280.0 - AUCTION_PANEL_WIDTH_PX) / 2.0,
        (960.0 - AUCTION_PANEL_HEIGHT_PX) / 2.0,
        AUCTION_PANEL_WIDTH_PX,
        AUCTION_PANEL_HEIGHT_PX,
    ),
    (
        "3840x2160",
        (3840.0 - AUCTION_PANEL_WIDTH_PX) / 2.0,
        (2160.0 - AUCTION_PANEL_HEIGHT_PX) / 2.0,
        AUCTION_PANEL_WIDTH_PX,
        AUCTION_PANEL_HEIGHT_PX,
    ),
    (
        "2560x1080",
        (2560.0 - AUCTION_PANEL_WIDTH_PX) / 2.0,
        (1080.0 - AUCTION_PANEL_HEIGHT_PX) / 2.0,
        AUCTION_PANEL_WIDTH_PX,
        AUCTION_PANEL_HEIGHT_PX,
    ),
];

// ─── Surface 8: settlement_overlay (UI_OVERLAY z-layer; excluded from
// geometric no-overlap; full viewport) ──────────────────────────────────

const SETTLEMENT_OVERLAY_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    ("1366x768", 0.0, 0.0, 1366.0, 768.0),
    ("1920x1080", 0.0, 0.0, 1920.0, 1080.0),
    ("1920x1200", 0.0, 0.0, 1920.0, 1200.0),
    ("1280x960", 0.0, 0.0, 1280.0, 960.0),
    ("3840x2160", 0.0, 0.0, 3840.0, 2160.0),
    ("2560x1080", 0.0, 0.0, 2560.0, 1080.0),
];

// ─── Surface 9: result_screen (MODAL z-layer; excluded from geometric
// no-overlap; centered) ─────────────────────────────────────────────────

const RESULT_SCREEN_RECTS: &[(&str, f32, f32, f32, f32)] = &[
    (
        "1366x768",
        (1366.0 - RESULT_SCREEN_WIDTH_PX) / 2.0,
        (768.0 - RESULT_SCREEN_HEIGHT_PX) / 2.0,
        RESULT_SCREEN_WIDTH_PX,
        RESULT_SCREEN_HEIGHT_PX,
    ),
    (
        "1920x1080",
        (1920.0 - RESULT_SCREEN_WIDTH_PX) / 2.0,
        (1080.0 - RESULT_SCREEN_HEIGHT_PX) / 2.0,
        RESULT_SCREEN_WIDTH_PX,
        RESULT_SCREEN_HEIGHT_PX,
    ),
    (
        "1920x1200",
        (1920.0 - RESULT_SCREEN_WIDTH_PX) / 2.0,
        (1200.0 - RESULT_SCREEN_HEIGHT_PX) / 2.0,
        RESULT_SCREEN_WIDTH_PX,
        RESULT_SCREEN_HEIGHT_PX,
    ),
    (
        "1280x960",
        (1280.0 - RESULT_SCREEN_WIDTH_PX) / 2.0,
        (960.0 - RESULT_SCREEN_HEIGHT_PX) / 2.0,
        RESULT_SCREEN_WIDTH_PX,
        RESULT_SCREEN_HEIGHT_PX,
    ),
    (
        "3840x2160",
        (3840.0 - RESULT_SCREEN_WIDTH_PX) / 2.0,
        (2160.0 - RESULT_SCREEN_HEIGHT_PX) / 2.0,
        RESULT_SCREEN_WIDTH_PX,
        RESULT_SCREEN_HEIGHT_PX,
    ),
    (
        "2560x1080",
        (2560.0 - RESULT_SCREEN_WIDTH_PX) / 2.0,
        (1080.0 - RESULT_SCREEN_HEIGHT_PX) / 2.0,
        RESULT_SCREEN_WIDTH_PX,
        RESULT_SCREEN_HEIGHT_PX,
    ),
];

/// Provisional surfaces table consumed by [`PROVISIONAL_BASELINE`]. Static
/// array so the test bin can iterate without runtime allocation.
const SURFACES: [SurfaceBaseline; 9] = [
    SurfaceBaseline {
        name: "lobby_root",
        phase: DisplayPhase::Lobby,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::TOP_LEFT,
        per_viewport: LOBBY_ROOT_RECTS,
        strip_height_px: None,
    },
    SurfaceBaseline {
        name: "hud_header_bar",
        phase: DisplayPhase::InSessionBase,
        kind: SurfaceKind::Strip,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::TOP_LEFT,
        per_viewport: HUD_HEADER_BAR_RECTS,
        strip_height_px: Some(HEADER_BAR_HEIGHT_PX),
    },
    SurfaceBaseline {
        name: "hud_footer_bar",
        phase: DisplayPhase::InSessionBase,
        kind: SurfaceKind::Strip,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::BOTTOM_LEFT,
        per_viewport: HUD_FOOTER_BAR_RECTS,
        strip_height_px: Some(FOOTER_BAR_HEIGHT_PX),
    },
    SurfaceBaseline {
        name: "hand_ui_hand_bar",
        phase: DisplayPhase::InSessionBase,
        kind: SurfaceKind::Strip,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::BOTTOM_LEFT,
        per_viewport: HAND_UI_HAND_BAR_RECTS,
        strip_height_px: Some(HAND_BAR_HEIGHT_PX),
    },
    SurfaceBaseline {
        name: "draft_centered_modal",
        phase: DisplayPhase::DraftInitial,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::CENTER,
        per_viewport: DRAFT_CENTERED_MODAL_RECTS,
        strip_height_px: None,
    },
    SurfaceBaseline {
        name: "shop_panel",
        phase: DisplayPhase::DraftShop,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::CENTER,
        per_viewport: SHOP_PANEL_RECTS,
        strip_height_px: None,
    },
    SurfaceBaseline {
        name: "auction_panel",
        phase: DisplayPhase::DraftAuction,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::CENTER,
        per_viewport: AUCTION_PANEL_RECTS,
        strip_height_px: None,
    },
    SurfaceBaseline {
        name: "settlement_overlay",
        phase: DisplayPhase::Settlement,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::UiOverlay,
        anchor: ProportionalAnchor::TOP_LEFT,
        per_viewport: SETTLEMENT_OVERLAY_RECTS,
        strip_height_px: None,
    },
    SurfaceBaseline {
        name: "result_screen",
        phase: DisplayPhase::GameOver,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::Modal,
        anchor: ProportionalAnchor::CENTER,
        per_viewport: RESULT_SCREEN_RECTS,
        strip_height_px: None,
    },
];

/// The canonical baseline consumed by the integration test bin. Holds
/// every surface entry referenced by story 005's AC1 surface list.
///
/// **DEPRECATED**: hand-authored fixture superseded by the live-spawn
/// harness at `tests/integration/ui_viewport_live_test.rs`. See module
/// docstring for the AC8 discharge rationale (PROMPT 1180 §RC-5 audit).
#[deprecated(
    since = "Sprint 18",
    note = "Hand-authored fixture is asserted against itself (PROMPT 1180 §RC-5). \
            Use `tests/integration/ui_viewport_live_test.rs` for live-spawn \
            (GlobalTransform, ComputedNode) viewport-invariant coverage."
)]
pub const PROVISIONAL_BASELINE: ViewportBaseline = ViewportBaseline {
    surfaces: &SURFACES,
};
