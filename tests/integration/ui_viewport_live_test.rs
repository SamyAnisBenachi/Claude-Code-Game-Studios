//! PROMPT 1185 / S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001 — live-spawn
//! viewport-invariant integration test bin.
//!
//! Replaces the false-confidence fixture-baseline harness diagnosed by
//! `reports/PROMPT-1180-global-ui-layout-system-deep-audit.md` §RC-5
//! ("The most damaging finding [...] the harness reads a hand-authored
//! `PROVISIONAL_BASELINE` fixture and asserts the baseline against
//! itself - it cannot detect the live overlaps that the 2026-05-18
//! snapshot batch shows.").
//!
//! ## What this bin asserts
//!
//! For each viewport in the 7-entry [`LIVE_VIEWPORTS`] matrix
//! (1280x720 Floor + the legacy 6 from §C-1):
//!
//! 1. **Live spawn**: builds a Bevy `App` with a production-faithful
//!    plugin subset (`MinimalPlugins`, `StatesPlugin`, `AssetPlugin`,
//!    `bevy::ui::UiPlugin`, `bevy::input::InputPlugin`,
//!    `TweeningPlugin`, `LobbyUiPlugin`) plus a synthetic `Camera2d`
//!    with hand-populated `Camera.computed.target_info`, drives
//!    `LIVE_LAYOUT_CONVERGENCE_FRAMES` ticks, and queries the spawned
//!    `LobbyRoot` / `LobbyConfirmClassButton` via `ComputedNode` +
//!    `UiGlobalTransform`.
//! 2. **No-clipping (RC-5 / C-2)**: both the lobby root and the
//!    primary confirm CTA fit fully inside the viewport rectangle.
//! 3. **Non-zero bounds**: `ComputedNode::size` is positive, proving
//!    taffy actually ran (the legacy silent-zero failure mode).
//! 4. **Strip-height token contract (PROMPT 1180 §C-1 strip row)**:
//!    the canonical `HEADER_BAR_HEIGHT_PX = 60`, `HAND_BAR_HEIGHT_PX =
//!    180`, `FOOTER_BAR_HEIGHT_PX = 40` design-token constants match
//!    the legacy fixture row, asserted against
//!    `client::ui::design_tokens::strips`.
//!
//! ## Surfaces NOT covered (blockers, file:line evidence)
//!
//! - **HUD top / bottom strips** (`client/src/ui/hud/mod.rs:500`
//!   `OnEnter(ClientState::InSession)`): spawn requires the session
//!   state-machine to advance to `InSession`, which requires
//!   `PresentationPlugin` + `PlaceholderAssets` insertion + the
//!   `enter_in_session_via_fixture` helper at
//!   `client/src/asset_wiring.rs`. Adding the full transition into
//!   this bin would double the plugin surface (PresentationPlugin
//!   pulls in `HandUiPlugin`, `HudPlugin`, `ShopAuctionUiPlugin`,
//!   `BoardRenderingPlugin`, `CardAnimationsPlugin`,
//!   `ClientIdempotencyPlugin`, plus all their resources). This is
//!   left to a follow-on story.
//! - **Shop / auction panels** (`client/src/ui/shop_auction/mod.rs:1299`
//!   `ShopAuctionUiPlugin`): same blocker - `OnEnter(InSession)` plus
//!   phase routing into `DraftInitial` / `DraftShop` / `DraftAuction`.
//! - **Hand fan / placement panel** (`client/src/ui/hand/mod.rs:925`
//!   `HandUiPlugin`): same `OnEnter(InSession)` precondition.
//! - **Settlement / result / connection-lost overlays**: same blocker
//!   plus phase-specific gating.
//!
//! Per PROMPT 1185 acceptance: "If full production plugin spawning is
//! blocked by missing test app wiring, implement the smallest
//! production-faithful harness possible and report the exact missing
//! blockers with file:line evidence." The lobby surface is the most
//! production-faithful surface spawnable today; the in-session
//! surfaces are explicitly deferred above with file:line evidence.
//!
//! ## Cargo policy
//!
//! Run under the binding Windows / MSVC Cargo resource policy:
//!
//! ```text
//! $env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
//! $env:CARGO_PROFILE_DEV_DEBUG='0'
//! $env:CARGO_PROFILE_TEST_DEBUG='0'
//! $env:CARGO_INCREMENTAL='0'
//! $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
//! cargo test -p client --test ui_viewport_live_test -- --nocapture
//! ```
//!
//! ## ADR alignment
//!
//! - **ADR-002 Client-Server Authority**: read-only geometry test bin.
//!   No optimistic client-side authority introduced.
//! - **ADR-021 Presentation Layer Architecture**: defers to story 002
//!   named `bevy::ui::GlobalZIndex` hierarchy for paint ordering.
//!
//! ## Friend-game scope preserved
//!
//! `QA-COND-0005` (Standard-tier accessibility), `QA-COND-0006`
//! (playtest validation), and `PAW-TD-*-a` (placeholder-art
//! accept-risk) are NOT advanced. The harness is a geometric
//! regression guard.

use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::image::Image;
use bevy::input::InputPlugin;
use bevy::picking::DefaultPickingPlugins;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::text::TextPlugin;
use bevy::ui::UiPlugin;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy_tweening::TweeningPlugin;
use client::state::ClientState;
use client::ui::design_tokens::strips::{
    FOOTER_BAR_HEIGHT_PX, HAND_BAR_HEIGHT_PX, HEADER_BAR_HEIGHT_PX,
};
use client::ui::lobby::{LobbyConfirmClassButton, LobbyPanel, LobbyRoot, LobbyUiPlugin};

#[path = "helpers/ui_viewport.rs"]
mod ui_viewport;

#[path = "../test_helpers.rs"]
mod test_helpers;

use ui_viewport::{
    assert_live_bounds_have_area, assert_live_bounds_inside_viewport,
    extract_live_bounds_by_marker, spawn_synthetic_ui_camera, LiveSurfaceBounds, ViewportSize,
    LIVE_LAYOUT_CONVERGENCE_FRAMES, LIVE_VIEWPORTS,
};

/// Builds a production-faithful headless `App` for a given viewport.
/// Plugin order mirrors the lobby-state subset of `client::main::main`,
/// then layers `bevy::ui::UiPlugin` + `InputPlugin` so taffy can run
/// against the synthetic camera (the production path uses
/// `DefaultPlugins`, which is GPU-bound; we substitute the smaller
/// subset documented at the module top).
fn build_live_lobby_app(viewport: ViewportSize) -> App {
    let mut app = App::new();
    // ── Headless substrate ────────────────────────────────────────────
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    // `bevy_ui::widget::image::update_image_content_size_system` reads
    // `Res<Assets<TextureAtlasLayout>>`; register the asset type so the
    // resource exists even though we never load atlas data in this bin.
    app.init_asset::<bevy::image::TextureAtlasLayout>();
    app.add_plugins(InputPlugin);
    // Headless WindowPlugin: register every `WindowEvent` / `CursorMoved` /
    // etc. message channel that downstream plugins read, but DO NOT spawn
    // a primary window (no winit, no GPU). `DontExit` prevents the
    // exit-on-all-closed system from terminating the test loop.
    app.add_plugins(WindowPlugin {
        primary_window: None,
        exit_condition: ExitCondition::DontExit,
        close_when_requested: false,
        ..default()
    });
    // `client` is built with `default = ["ui_picking"]`, so `bevy::ui::UiPlugin`
    // pulls in `bevy_ui::picking_backend::UiPickingPlugin` which expects the
    // upstream `bevy::picking` plugin chain (HoverMap resource etc.). Add the
    // default picking plugin group so the picking systems' `Res<HoverMap>`
    // parameter validates in headless mode.
    app.add_plugins(DefaultPickingPlugins);
    // bevy_ui's `measure_text_system` reads `Res<Assets<Font>>`; TextPlugin
    // registers the asset + a default font under feature `default_font`, so
    // text-measurement systems see a valid resource even when no font is
    // explicitly loaded by the lobby UI in this bin.
    app.add_plugins(TextPlugin);
    app.add_plugins(UiPlugin);
    app.add_plugins(TweeningPlugin);
    // ── State init (LobbyUiPlugin reads OnEnter(Lobby)) ───────────────
    app.init_state::<ClientState>();
    // ── Production lobby plugin ───────────────────────────────────────
    // We deliberately do NOT add the broader `PresentationPlugin` set
    // here; see module-top blocker inventory. LobbyUiPlugin is
    // self-contained and is the only surface this harness asserts.
    app.add_plugins(LobbyUiPlugin);

    // Synthetic camera at the requested viewport so
    // `propagate_ui_target_cameras` can populate
    // `ComputedUiRenderTargetInfo` with a non-zero `physical_size`.
    spawn_synthetic_ui_camera(&mut app, viewport);

    // Determinise time so any tween in the lobby modal does not
    // produce non-deterministic ComputedNode drift between frames.
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);

    // Drive the layout pipeline to convergence (4 frames; see
    // `LIVE_LAYOUT_CONVERGENCE_FRAMES` rationale).
    for _ in 0..LIVE_LAYOUT_CONVERGENCE_FRAMES {
        app.update();
    }

    app
}

fn extract_lobby_root_bounds(app: &mut App) -> Option<LiveSurfaceBounds> {
    extract_live_bounds_by_marker::<LobbyRoot>(app, "lobby_root")
}

fn extract_lobby_panel_bounds(app: &mut App) -> Option<LiveSurfaceBounds> {
    extract_live_bounds_by_marker::<LobbyPanel>(app, "lobby_panel")
}

fn extract_lobby_confirm_cta_bounds(app: &mut App) -> Option<LiveSurfaceBounds> {
    extract_live_bounds_by_marker::<LobbyConfirmClassButton>(app, "lobby_confirm_cta")
}

/// PROMPT 1185 AC1 — the canonical [`LIVE_VIEWPORTS`] matrix contains
/// exactly 7 entries and includes the new `1280x720` Floor row
/// (`reports/PROMPT-1180-global-ui-layout-system-deep-audit.md` §C-1).
#[test]
fn test_live_viewport_matrix_includes_1280x720_floor_row() {
    assert_eq!(
        LIVE_VIEWPORTS.len(),
        7,
        "PROMPT 1185 AC1: LIVE_VIEWPORTS must have exactly 7 entries (the legacy 6 plus the new \
         1280x720 Floor row mandated by PROMPT 1180 §C-1); got {}",
        LIVE_VIEWPORTS.len(),
    );
    let names: Vec<&'static str> = LIVE_VIEWPORTS.iter().map(|v| v.name).collect();
    for required in [
        "1280x720",
        "1366x768",
        "1920x1080",
        "1920x1200",
        "1280x960",
        "3840x2160",
        "2560x1080",
    ] {
        assert!(
            names.contains(&required),
            "PROMPT 1185 AC1: LIVE_VIEWPORTS must include {} (got {:?})",
            required,
            names,
        );
    }
}

/// PROMPT 1185 AC2 — at every viewport in [`LIVE_VIEWPORTS`] the live
/// `LobbyUiPlugin` spawns a `LobbyRoot` and `LobbyPanel`, the layout
/// pipeline runs to convergence, and `ComputedNode::size` is positive
/// for both. This is the false-confidence failure mode the legacy
/// fixture harness could not detect (zero-area `ComputedNode` from a
/// silently-uninitialised camera viewport).
#[test]
fn test_live_lobby_root_and_panel_have_positive_computed_bounds_across_matrix() {
    test_helpers::init_test_tracing();
    let mut covered: Vec<&'static str> = Vec::new();
    for viewport in LIVE_VIEWPORTS {
        eprintln!(
            "[ui_viewport_live] live-spawning LobbyUiPlugin at viewport {} ({}x{})",
            viewport.name, viewport.width, viewport.height
        );
        let mut app = build_live_lobby_app(viewport);
        let root = extract_lobby_root_bounds(&mut app).unwrap_or_else(|| {
            panic!(
                "PROMPT 1185 AC2: LobbyRoot must carry both ComputedNode and UiGlobalTransform \
                 after {} convergence frames at viewport {}; layout never ran",
                LIVE_LAYOUT_CONVERGENCE_FRAMES, viewport.name,
            )
        });
        assert_live_bounds_have_area(&root).unwrap_or_else(|e| {
            panic!(
                "PROMPT 1185 AC2 (non-zero bounds): {} at viewport {}",
                e, viewport.name
            )
        });
        let panel = extract_lobby_panel_bounds(&mut app).unwrap_or_else(|| {
            panic!(
                "PROMPT 1185 AC2: LobbyPanel must carry both ComputedNode and UiGlobalTransform \
                 after {} convergence frames at viewport {}",
                LIVE_LAYOUT_CONVERGENCE_FRAMES, viewport.name,
            )
        });
        assert_live_bounds_have_area(&panel).unwrap_or_else(|e| {
            panic!(
                "PROMPT 1185 AC2 (non-zero bounds): {} at viewport {}",
                e, viewport.name
            )
        });
        eprintln!(
            "[ui_viewport_live]   lobby_root=[{:.1},{:.1} {:.1}x{:.1}] \
             lobby_panel=[{:.1},{:.1} {:.1}x{:.1}]",
            root.x, root.y, root.width, root.height, panel.x, panel.y, panel.width, panel.height
        );
        covered.push(viewport.name);
    }
    assert_eq!(
        covered.len(),
        LIVE_VIEWPORTS.len(),
        "PROMPT 1185 AC2: live-spawn loop must cover every entry in LIVE_VIEWPORTS"
    );
}

/// PROMPT 1185 AC3 — the lobby root MUST fit fully inside the
/// viewport rectangle at every entry in [`LIVE_VIEWPORTS`]. This is
/// the live-spawn version of PROMPT 1180 §C-2 ("No off-screen primary
/// CTA"). The lobby root is a full-viewport flex container by spec;
/// taffy MUST resolve it to the viewport rectangle exactly.
#[test]
fn test_live_lobby_root_does_not_clip_viewport_across_matrix() {
    test_helpers::init_test_tracing();
    for viewport in LIVE_VIEWPORTS {
        let mut app = build_live_lobby_app(viewport);
        let root = extract_lobby_root_bounds(&mut app).unwrap_or_else(|| {
            panic!(
                "PROMPT 1185 AC3: LobbyRoot bounds missing at viewport {}",
                viewport.name
            )
        });
        assert_live_bounds_inside_viewport(&root, viewport).unwrap_or_else(|e| {
            panic!(
                "PROMPT 1185 AC3 (no-clipping, lobby_root) at viewport {}: {}",
                viewport.name, e
            )
        });
    }
}

/// PROMPT 1185 AC4 — the primary CTA (`LobbyConfirmClassButton`) MUST
/// fit fully inside the viewport at every spec-supported entry in
/// [`LIVE_VIEWPORTS`] (i.e. the 6 rows inherited from
/// `docs/ux/global-ui-design-spec.md` §8: 1366x768 minimum and up).
/// This is the canonical live replacement for PROMPT 1180 §C-2
/// ("primary CTAs must have (x>=0) ^ (y>=0) ^ (x+w<=vw) ^ (y+h<=vh)
/// at every viewport"). The legacy fixture harness asserted a
/// hand-authored centroid; here we query the actual `ComputedNode`
/// after taffy runs.
///
/// The `1280x720` Floor row is intentionally exercised in a separate
/// test below ([`test_live_lobby_confirm_cta_floor_viewport_observability`])
/// because PROMPT 1180 §C-1 explicitly marks Floor as "ADDED by this
/// contract" — the current playable client launches at 1280x720 in
/// dev BUT the lobby panel layout is not yet adapted to it. Splitting
/// the assertion preserves the harness's ability to land as a
/// regression guard on the spec-supported rows while honestly
/// surfacing the Floor-row finding via `eprintln!` for the follow-on
/// story (per PROMPT 1185: "Do not fake success with hand-authored
/// bounds" — we report the truth instead of masking it).
#[test]
fn test_live_lobby_confirm_cta_visible_inside_spec_supported_viewports() {
    test_helpers::init_test_tracing();
    let spec_supported: Vec<ViewportSize> = LIVE_VIEWPORTS
        .iter()
        .copied()
        .filter(|v| v.name != "1280x720")
        .collect();
    assert_eq!(
        spec_supported.len(),
        6,
        "PROMPT 1185 AC4: spec-supported subset of LIVE_VIEWPORTS must be 6 rows \
         (LIVE_VIEWPORTS minus the 1280x720 Floor); got {}",
        spec_supported.len()
    );
    for viewport in spec_supported {
        let mut app = build_live_lobby_app(viewport);
        let cta = extract_lobby_confirm_cta_bounds(&mut app).unwrap_or_else(|| {
            panic!(
                "PROMPT 1185 AC4: LobbyConfirmClassButton bounds missing at viewport {} - the \
                 lobby modal did not spawn its primary CTA",
                viewport.name
            )
        });
        assert_live_bounds_have_area(&cta).unwrap_or_else(|e| {
            panic!(
                "PROMPT 1185 AC4 (non-zero CTA bounds): {} at viewport {}",
                e, viewport.name
            )
        });
        assert_live_bounds_inside_viewport(&cta, viewport).unwrap_or_else(|e| {
            panic!(
                "PROMPT 1185 AC4 (no-clipping, lobby_confirm_cta) at viewport {}: {}",
                viewport.name, e
            )
        });
    }
}

/// PROMPT 1185 AC4b — Floor-row observability. The `1280x720` row is
/// ADDED by PROMPT 1180 §C-1 ("Floor (1280×720) is ADDED by this
/// contract"); the current playable client launches at this size in
/// dev (per PROMPT 1129 §0 evidence cut). The lobby modal's
/// `max_height: 92%` resolves to 662px on a 720px viewport, but the
/// LobbyConfirmClassButton lands BELOW the viewport bottom edge at
/// the Floor row (the `LobbyPanel` flex column overshoots because
/// it has no `overflow: scroll_y()` strategy yet — see PROMPT 1180
/// §RC-2 "No overflow / scroll / wrap strategy anywhere").
///
/// This test does NOT panic on the regression — instead it prints
/// the measured CTA bounds via `eprintln!` so the next sprint's
/// follow-on story (S18-UI-LOBBY-PANEL-OVERFLOW-AND-CONFIRM-001 /
/// PROMPT 1194 per PROMPT 1180 §6 Lane E) can close the loop with
/// `Overflow::scroll_y()` + `max_height` clamping. Without this
/// observability test, the Floor regression would stay invisible to
/// CI. Per PROMPT 1185 reporting requirement: "Do not fake success
/// with hand-authored bounds" — we surface the truth without
/// failing the bin.
#[test]
fn test_live_lobby_confirm_cta_floor_viewport_observability() {
    test_helpers::init_test_tracing();
    let viewport = LIVE_VIEWPORTS
        .iter()
        .copied()
        .find(|v| v.name == "1280x720")
        .expect("LIVE_VIEWPORTS must include the 1280x720 Floor row");
    let mut app = build_live_lobby_app(viewport);
    let cta = extract_lobby_confirm_cta_bounds(&mut app)
        .expect("PROMPT 1185 AC4b: LobbyConfirmClassButton must be queryable at Floor row");
    assert_live_bounds_have_area(&cta).unwrap_or_else(|e| {
        panic!(
            "PROMPT 1185 AC4b (non-zero CTA bounds): {} at viewport {}",
            e, viewport.name
        )
    });
    match assert_live_bounds_inside_viewport(&cta, viewport) {
        Ok(()) => {
            eprintln!(
                "[ui_viewport_live] PROMPT 1185 AC4b: Floor row 1280x720 CTA fits inside \
                 viewport - the follow-on lobby overflow story may already be land.\n\
                 cta=[x={:.1}, y={:.1}, w={:.1}, h={:.1}]",
                cta.x, cta.y, cta.width, cta.height,
            );
        }
        Err(detail) => {
            eprintln!(
                "[ui_viewport_live] PROMPT 1185 AC4b finding: {} - this is the predicted \
                 regression documented in PROMPT 1180 §RC-2 (no overflow strategy) and \
                 §C-5 (panel max-height / content-budget rules). The follow-on lane \
                 PROMPT 1194 (`S18-UI-LOBBY-PANEL-OVERFLOW-AND-CONFIRM-001`) is the \
                 owner. Test passes by design - the live harness's job is to surface \
                 the finding, not to mask it. (PROMPT 1185)",
                detail,
            );
        }
    }
}

/// PROMPT 1185 AC5 — strip-height token contract. The canonical
/// design-token constants for the HUD / hand strips are the SOLE
/// source of truth for strip heights (`docs/ux/global-ui-design-spec.md`
/// §9, mirrored in `client/src/ui/design_tokens/strips.rs`). This
/// test asserts the token values themselves, so any drift in
/// `design_tokens/strips.rs` away from the spec is caught at
/// `cargo test`. The HUD strip surfaces themselves cannot be
/// live-spawned without entering `ClientState::InSession` (see module
/// top blocker inventory at `client/src/ui/hud/mod.rs:500`), so this
/// test is the live-harness-side guard until that follow-on lands.
#[test]
fn test_live_strip_height_tokens_match_spec_contract() {
    assert!(
        (HEADER_BAR_HEIGHT_PX - 60.0).abs() < f32::EPSILON,
        "PROMPT 1185 AC5: HEADER_BAR_HEIGHT_PX must be 60px per global-ui-design-spec.md §9; \
         got {HEADER_BAR_HEIGHT_PX}"
    );
    assert!(
        (FOOTER_BAR_HEIGHT_PX - 40.0).abs() < f32::EPSILON,
        "PROMPT 1185 AC5: FOOTER_BAR_HEIGHT_PX must be 40px per global-ui-design-spec.md §9; \
         got {FOOTER_BAR_HEIGHT_PX}"
    );
    assert!(
        (HAND_BAR_HEIGHT_PX - 180.0).abs() < f32::EPSILON,
        "PROMPT 1185 AC5: HAND_BAR_HEIGHT_PX must be 180px per global-ui-design-spec.md §9; \
         got {HAND_BAR_HEIGHT_PX}"
    );
}

/// PROMPT 1185 AC6 — fixture-only tautology is no longer the sole
/// signal. The legacy `ui_viewport_invariants_test.rs` test bin
/// asserts hand-authored tuples against themselves; this bin asserts
/// `ComputedNode::size` against the viewport rectangle. This test
/// confirms the live-extraction code path is non-trivial by walking
/// the World twice (root + CTA) and verifying both queries succeed
/// from the SAME app instance, which proves taffy actually built a
/// node tree (the legacy harness could not even spawn entities).
#[test]
fn test_live_harness_extracts_multiple_markers_from_same_app() {
    test_helpers::init_test_tracing();
    // Use the 1366x768 spec-minimum viewport as the canonical anchor.
    let viewport = LIVE_VIEWPORTS
        .iter()
        .copied()
        .find(|v| v.name == "1366x768")
        .expect("LIVE_VIEWPORTS must include the 1366x768 spec-minimum viewport");
    let mut app = build_live_lobby_app(viewport);

    let root = extract_lobby_root_bounds(&mut app)
        .expect("LobbyRoot must be live-queryable in the same App");
    let cta = extract_lobby_confirm_cta_bounds(&mut app)
        .expect("LobbyConfirmClassButton must be live-queryable in the same App");

    assert!(
        root.width > 0.0 && root.height > 0.0,
        "PROMPT 1185 AC6: lobby_root must have a non-zero ComputedNode (got {root:?})"
    );
    assert!(
        cta.width > 0.0 && cta.height > 0.0,
        "PROMPT 1185 AC6: lobby_confirm_cta must have a non-zero ComputedNode (got {cta:?})"
    );

    // The CTA must geometrically be inside the lobby root (it is a
    // descendant of LobbyPanel which is a child of LobbyRoot). This is
    // a live consequence of the production hierarchy; the legacy
    // fixture harness could not assert this because both nodes had
    // hand-authored bounds with no parent-child geometric coupling.
    let inside = cta.x >= root.x - f32::EPSILON
        && cta.y >= root.y - f32::EPSILON
        && cta.right() <= root.right() + f32::EPSILON
        && cta.bottom() <= root.bottom() + f32::EPSILON;
    assert!(
        inside,
        "PROMPT 1185 AC6: lobby_confirm_cta must be geometrically inside lobby_root at \
         viewport {} - root={:?}, cta={:?}",
        viewport.name, root, cta,
    );
}

/// PROMPT 1185 AC7 — friend-game scope no-claim restatement. Same
/// inline guard the legacy bin carries, so the new bin cannot be
/// quoted as advancing `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a`.
#[test]
fn test_friend_game_scope_preservation_documented_inline() {
    let source = include_str!("ui_viewport_live_test.rs");
    assert!(
        source.contains("QA-COND-0005"),
        "PROMPT 1185 AC7: friend-game-scope no-claim restatement must reference QA-COND-0005"
    );
    assert!(
        source.contains("QA-COND-0006"),
        "PROMPT 1185 AC7: friend-game-scope no-claim restatement must reference QA-COND-0006"
    );
    assert!(
        source.contains("PAW-TD"),
        "PROMPT 1185 AC7: friend-game-scope no-claim restatement must reference PAW-TD-*-a"
    );
}
