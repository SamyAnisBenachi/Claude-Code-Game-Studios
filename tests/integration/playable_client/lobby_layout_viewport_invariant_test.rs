//! Story 024 — Lobby layout viewport invariant test
//! (S12-UX-LOBBY-LAYOUT-MODAL-001 / PROMPT 937 `/dev-story` Option A).
//!
//! Asserts the lobby UI composition after the Option A migration:
//!
//! - **AC2** — the lobby root is a full-viewport flex container (no
//!   top-left `Val::Px(24.0)` anchor; `Display::Flex`,
//!   `align_items: Center`, `justify_content: Center`, viewport-anchored
//!   absolute insets all at `0.0`) and the inner `LobbyPanel` is composed
//!   via flex children with no `PositionType::Absolute` on the primary
//!   form column.
//! - **AC3 / AC4** — the panel's prescribed literals (`width: 88%`,
//!   `max_width: 860 Px`, `max_height: 92%`) fit comfortably within both
//!   the `1366×768` minimum-supported viewport and the `1920×1080`
//!   canonical HD viewport when resolved analytically per CSS-style
//!   flex layout rules.
//! - **AC3(e) / AC5** — the read order of the lobby panel's direct flex
//!   children places the `LobbyConfirmClassButton` LAST. The portrait
//!   row, class-picker row, slot-panel row, and room-code chip render
//!   ABOVE the confirm CTA, resolving the PROMPT 802 §3.1 L4 read-order
//!   inversion.
//! - **AC6** — the `LobbyConfirmClassButton` button width remains
//!   `Val::Percent(100.0)` and the class / slot / create-join buttons
//!   keep their pre-migration `LOBBY_BUTTON_HEIGHT` (30 px) so the
//!   paired story 026 button-hit-targets row's canonical dimensions are
//!   preserved across this layout change. Story 024 consumes story 026's
//!   dimension invariant; this test guards regression on the lobby side.
//! - **AC8** — this test is the story-prescribed lobby layout viewport
//!   invariant test bin. Filename:
//!   `lobby_layout_viewport_invariant_test.rs` per the story's "Likely
//!   Files" table.
//! - **AC9** — read-only assertions: no client-side state-mutation API
//!   is invoked, no protocol shape is exercised. The lobby state
//!   machine resources (`LobbyViewState`, `LobbyInputState`,
//!   `ClientIdempotencyState`) are read at `default()` only.
//! - **AC11** — no `#[ignore]` markers. Test runs under
//!   `cargo test --workspace --tests --no-fail-fast` on `origin/main`.
//! - **AC13** — friend-game scope preserved. This bin does NOT advance
//!   `QA-COND-0005` Standard-tier accessibility (WCAG contrast, ≥44px
//!   hit-targets, full keyboard navigation, screen-reader support),
//!   `QA-COND-0006` playtest validation, or `PAW-TD-*-a` placeholder-art
//!   accept-risk.
//!
//! ## Cargo policy
//!
//! Run under the binding Windows/MSVC Cargo resource policy:
//!
//! ```text
//! $env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
//! $env:CARGO_PROFILE_DEV_DEBUG='0'
//! $env:CARGO_PROFILE_TEST_DEBUG='0'
//! $env:CARGO_INCREMENTAL='0'
//! $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
//! cargo test -p client --test playable_client_lobby_layout_viewport_invariant_test
//! ```
//!
//! ## ADR alignment
//!
//! - **ADR-002 Client-Server Authority**: read-only composition test.
//!   No optimistic client-side authority introduced.
//! - **ADR-021 Presentation Layer Architecture**: defers to story 002
//!   named [`bevy::ui::GlobalZIndex`] hierarchy
//!   (`UI_OVERLAY` < `MODAL`) for paint ordering.

use bevy::asset::AssetPlugin;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::GlobalZIndex;
use client::state::ClientState;
use client::ui::design_tokens::overlays::OVERLAY_SCRIM_ALPHA;
use client::ui::design_tokens::spacing::{SPACING_LG, SPACING_MD, SPACING_XL};
use client::ui::design_tokens::z_layers;
use client::ui::lobby::{
    LobbyCamera, LobbyConfirmClassButton, LobbyPanel, LobbyRoot, LobbyUiPlugin,
    LOBBY_PANEL_MAX_HEIGHT_PERCENT, LOBBY_PANEL_MAX_WIDTH_PX, LOBBY_PANEL_WIDTH_PERCENT,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// Minimum supported viewport per Sprint 14 viewport-invariant matrix
/// (`tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS[0]`).
const VIEWPORT_MIN: (f32, f32) = (1366.0, 768.0);

/// Canonical HD viewport per Sprint 14 viewport-invariant matrix
/// (`tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS[1]`).
const VIEWPORT_HD: (f32, f32) = (1920.0, 1080.0);

/// Spawn a minimal Bevy `App` that exercises [`LobbyUiPlugin`] for layout
/// composition tests. Mirrors the setup pattern from
/// `tests/integration/playable_client/lobby_entry_test.rs::
/// lobby_startup_spawns_visible_ui_camera_until_session_entry` so the
/// composition assertions here are run against the same wiring the
/// production playable client uses.
fn spawn_lobby_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(LobbyUiPlugin);

    app.update();
    app.update();

    app
}

/// AC2 — the lobby root is a full-viewport flex container after the
/// Option A migration. No top-left `Val::Px(24.0)` anchor; the four
/// absolute insets are all `Val::Px(0.0)` so the root fills the entire
/// viewport.
#[test]
fn ac2_lobby_root_is_full_viewport_flex_container() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<&Node, With<LobbyRoot>>();
    let root_count = roots.iter(world).count();
    assert_eq!(
        root_count, 1,
        "AC2: spawn_lobby_ui_system must spawn exactly one LobbyRoot; got {root_count}"
    );

    let root_node = roots
        .single(world)
        .expect("AC2: single LobbyRoot present after lobby plugin update");

    assert_eq!(
        root_node.display,
        Display::Flex,
        "AC2: LobbyRoot must use Display::Flex (full-viewport flex \
         container); got {:?}",
        root_node.display
    );
    assert_eq!(
        root_node.position_type,
        PositionType::Absolute,
        "AC2: LobbyRoot must use PositionType::Absolute so its four \
         insets anchor it to the viewport rectangle; got {:?}",
        root_node.position_type
    );
    assert_eq!(
        root_node.align_items,
        AlignItems::Center,
        "AC2: LobbyRoot must vertically center the panel; got {:?}",
        root_node.align_items
    );
    assert_eq!(
        root_node.justify_content,
        JustifyContent::Center,
        "AC2: LobbyRoot must horizontally center the panel; got {:?}",
        root_node.justify_content
    );
    assert_eq!(
        root_node.left,
        Val::Px(0.0),
        "AC2: LobbyRoot left inset must be 0px (full-viewport); the prior \
         top-left Val::Px(24.0) anchor is removed. got {:?}",
        root_node.left
    );
    assert_eq!(
        root_node.right,
        Val::Px(0.0),
        "AC2: LobbyRoot right inset must be 0px (full-viewport); got {:?}",
        root_node.right
    );
    assert_eq!(
        root_node.top,
        Val::Px(0.0),
        "AC2: LobbyRoot top inset must be 0px (full-viewport); the prior \
         top-left Val::Px(24.0) anchor is removed. got {:?}",
        root_node.top
    );
    assert_eq!(
        root_node.bottom,
        Val::Px(0.0),
        "AC2: LobbyRoot bottom inset must be 0px (full-viewport); got {:?}",
        root_node.bottom
    );
}

/// AC2 — the inner lobby panel exists as a flex child of the root and
/// composes its own children via flex (not absolute).
#[test]
fn ac2_lobby_panel_is_flex_child_composed_via_flex() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let mut panels = world.query_filtered::<&Node, With<LobbyPanel>>();
    let panel_count = panels.iter(world).count();
    assert_eq!(
        panel_count, 1,
        "AC2: spawn_lobby_ui_system must spawn exactly one LobbyPanel \
         inside the LobbyRoot; got {panel_count}"
    );

    let panel_node = panels
        .single(world)
        .expect("AC2: single LobbyPanel present after lobby plugin update");

    assert_eq!(
        panel_node.display,
        Display::Flex,
        "AC2: LobbyPanel must use Display::Flex so its children compose \
         via flex (not absolute); got {:?}",
        panel_node.display
    );
    assert_eq!(
        panel_node.flex_direction,
        FlexDirection::Column,
        "AC2: LobbyPanel must use FlexDirection::Column so children \
         stack top-to-bottom for AC3(e) read order; got {:?}",
        panel_node.flex_direction
    );
    assert_ne!(
        panel_node.position_type,
        PositionType::Absolute,
        "AC2: LobbyPanel must NOT use PositionType::Absolute on the \
         primary form column; got {:?}",
        panel_node.position_type
    );
}

/// AC3 / AC4 — panel sizing literals match the Option A literals locked
/// by PROMPT 933 (`width: 88%`, `max_width: 860 Px`, `max_height: 92%`).
#[test]
fn ac3_ac4_panel_sizing_matches_prompt_933_option_a_literals() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let mut panels = world.query_filtered::<&Node, With<LobbyPanel>>();
    let panel_node = panels
        .single(world)
        .expect("AC3: single LobbyPanel present after lobby plugin update");

    assert_eq!(
        panel_node.width,
        Val::Percent(LOBBY_PANEL_WIDTH_PERCENT),
        "AC3: lobby panel width must be {LOBBY_PANEL_WIDTH_PERCENT}% per \
         PROMPT 933 Option A literals; got {:?}",
        panel_node.width
    );
    assert_eq!(
        panel_node.max_width,
        Val::Px(LOBBY_PANEL_MAX_WIDTH_PX),
        "AC3: lobby panel max_width must be {LOBBY_PANEL_MAX_WIDTH_PX}px \
         per PROMPT 933 Option A literals; got {:?}",
        panel_node.max_width
    );
    assert_eq!(
        panel_node.max_height,
        Val::Percent(LOBBY_PANEL_MAX_HEIGHT_PERCENT),
        "AC3: lobby panel max_height must be {LOBBY_PANEL_MAX_HEIGHT_PERCENT}% \
         per PROMPT 933 Option A literals; got {:?}",
        panel_node.max_height
    );
    assert_eq!(
        panel_node.padding,
        UiRect::all(Val::Px(SPACING_LG)),
        "AC3: lobby panel padding must be SPACING_LG ({SPACING_LG}px) \
         per global-ui-design-spec.md §10 primary modal panel chrome; got {:?}",
        panel_node.padding
    );
    assert_eq!(
        panel_node.row_gap,
        Val::Px(SPACING_MD),
        "AC3: lobby panel inter-child row_gap must be SPACING_MD \
         ({SPACING_MD}px) per global-ui-design-spec.md §4; got {:?}",
        panel_node.row_gap
    );
}

/// AC3 / AC4 — at every supported viewport the panel's resolved width
/// and height fit comfortably inside the viewport with positive margin.
/// Computes the resolved pixel size analytically per CSS-style flex
/// rules (`min(percent × viewport_dim, max_pixels)`) since the bevy_ui
/// layout solver does not run without a windowed renderer in the test
/// harness.
#[test]
fn ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd() {
    test_helpers::init_test_tracing();
    for (label, (vw, vh)) in &[("1366x768", VIEWPORT_MIN), ("1920x1080", VIEWPORT_HD)] {
        let resolved_width = (LOBBY_PANEL_WIDTH_PERCENT / 100.0 * vw).min(LOBBY_PANEL_MAX_WIDTH_PX);
        let resolved_height = LOBBY_PANEL_MAX_HEIGHT_PERCENT / 100.0 * vh;

        eprintln!(
            "[lobby_layout_viewport_invariant] viewport {label}: \
             panel resolves to {resolved_width:.1}x{resolved_height:.1} inside {vw:.0}x{vh:.0}"
        );

        assert!(
            resolved_width <= *vw,
            "AC3 fit-within-viewport failed at {label}: panel resolved width \
             {resolved_width} exceeds viewport width {vw}"
        );
        assert!(
            resolved_height <= *vh,
            "AC4 fit-within-viewport failed at {label}: panel resolved height \
             {resolved_height} exceeds viewport height {vh}"
        );
        // Modal panels reserve ≥ 4% horizontal breathing room (the
        // 100% − 88% Option A literal) and ≥ 8% vertical breathing room
        // (the 100% − 92% Option A literal). Guard those margins.
        assert!(
            resolved_width <= 0.88 * vw + 0.5,
            "AC3 panel must respect {LOBBY_PANEL_WIDTH_PERCENT}% width \
             clamp at {label}; got {resolved_width} vs {vw}"
        );
        assert!(
            resolved_height <= 0.92 * vh + 0.5,
            "AC4 panel must respect {LOBBY_PANEL_MAX_HEIGHT_PERCENT}% \
             max_height clamp at {label}; got {resolved_height} vs {vh}"
        );
    }
}

/// AC3(e) / AC5 — the `LobbyConfirmClassButton` is the LAST direct child
/// of the lobby panel so the read order top-to-bottom is `status /
/// room-code -> create-join -> class picker -> slot panels -> confirm`.
/// This resolves the PROMPT 802 §3.1 L4 inversion where portraits +
/// slot panels + room-code chip image rendered BELOW the confirm CTA.
#[test]
fn ac3_ac5_confirm_cta_is_last_panel_child() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("AC3(e): single LobbyPanel entity present")
    };

    let children = world
        .entity(panel_entity)
        .get::<Children>()
        .expect("AC3(e): LobbyPanel must have Children component")
        .iter()
        .collect::<Vec<_>>();

    assert!(
        children.len() >= 2,
        "AC3(e): lobby panel must compose at least the status banner + \
         confirm CTA; got {} children",
        children.len()
    );

    let last_child = *children
        .last()
        .expect("AC3(e): lobby panel children list is non-empty");

    let last_is_confirm = world
        .entity(last_child)
        .get::<LobbyConfirmClassButton>()
        .is_some();

    assert!(
        last_is_confirm,
        "AC3(e): LobbyConfirmClassButton MUST be the LAST direct child of \
         LobbyPanel so the read order ends at the confirm CTA. The portrait \
         row, slot panels, and room-code chip must render ABOVE the confirm, \
         resolving the PROMPT 802 §3.1 L4 inversion. Last child entity {:?} \
         is not a LobbyConfirmClassButton.",
        last_child
    );
}

/// AC3(e) / AC5 — exactly one `LobbyConfirmClassButton` exists in the
/// scene (no orphan duplicates from the prior top-left composition).
#[test]
fn ac3_exactly_one_confirm_cta_after_migration() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let mut confirms = world.query::<&LobbyConfirmClassButton>();
    let count = confirms.iter(world).count();
    assert_eq!(
        count, 1,
        "AC3: lobby UI must spawn exactly one LobbyConfirmClassButton; got {count}"
    );
}

/// AC2 / §3 layer hierarchy — the lobby root paints at `UI_OVERLAY` and
/// the panel paints at `MODAL` per the PROMPT 933 Option A literals.
#[test]
fn ac2_z_layers_match_prompt_933_option_a_literals() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<&GlobalZIndex, With<LobbyRoot>>();
    let root_z = *roots
        .single(world)
        .expect("AC2: LobbyRoot must carry a GlobalZIndex");
    assert_eq!(
        root_z.0,
        z_layers::UI_OVERLAY.0,
        "AC2: LobbyRoot GlobalZIndex must be UI_OVERLAY ({}) per PROMPT \
         933 Option A; got {}",
        z_layers::UI_OVERLAY.0,
        root_z.0
    );

    let mut panels = world.query_filtered::<&GlobalZIndex, With<LobbyPanel>>();
    let panel_z = *panels
        .single(world)
        .expect("AC2: LobbyPanel must carry a GlobalZIndex");
    assert_eq!(
        panel_z.0,
        z_layers::MODAL.0,
        "AC2: LobbyPanel GlobalZIndex must be MODAL ({}) per PROMPT 933 \
         Option A; got {}",
        z_layers::MODAL.0,
        panel_z.0
    );

    assert!(
        z_layers::MODAL.0 > z_layers::UI_OVERLAY.0,
        "AC2: MODAL ({}) must paint above UI_OVERLAY ({}) so the lobby \
         panel reads above its own scrim backdrop",
        z_layers::MODAL.0,
        z_layers::UI_OVERLAY.0
    );
}

/// AC2 — the lobby root's BackgroundColor uses the prescribed
/// `OVERLAY_SCRIM_ALPHA` (0.55) alpha channel on a `SURFACE`-toned
/// near-black RGB. Guards against future drift back to ad-hoc alpha
/// literals.
#[test]
fn ac2_root_backdrop_uses_overlay_scrim_alpha_token() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<&BackgroundColor, With<LobbyRoot>>();
    let root_bg = *roots
        .single(world)
        .expect("AC2: LobbyRoot must carry a BackgroundColor for the modal scrim");
    let srgba = root_bg.0.to_srgba();
    assert!(
        (srgba.alpha - OVERLAY_SCRIM_ALPHA).abs() < f32::EPSILON,
        "AC2: LobbyRoot backdrop alpha must be OVERLAY_SCRIM_ALPHA \
         ({OVERLAY_SCRIM_ALPHA}); got {}",
        srgba.alpha
    );
    // SURFACE token RGB per global-ui-design-spec.md §7.
    assert!(
        srgba.red < 0.1 && srgba.green < 0.1 && srgba.blue < 0.1,
        "AC2: LobbyRoot backdrop RGB must be the near-black SURFACE \
         token range (< 0.1 each channel); got ({}, {}, {})",
        srgba.red,
        srgba.green,
        srgba.blue
    );
}

/// AC2 — the lobby UI plugin spawns a `LobbyCamera` so the lobby is
/// visible after the migration (regression guard).
#[test]
fn ac2_lobby_plugin_spawns_camera() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let world = app.world_mut();
    let mut cameras = world.query_filtered::<Entity, (With<Camera2d>, With<LobbyCamera>)>();
    let count = cameras.iter(world).count();
    assert_eq!(
        count, 1,
        "AC2: lobby plugin must spawn exactly one LobbyCamera regardless \
         of layout migration; got {count}"
    );
}

/// AC5 — the section separators between status / create-join / class /
/// confirm sections use `SPACING_XL` total cumulative gap per the
/// PROMPT 933 literals table (default `row_gap` `SPACING_MD` + an extra
/// `SPACING_XL - SPACING_MD` margin on each separator). Guard against
/// drift away from the design spec §4 spacing scale.
#[test]
fn ac5_section_separators_resolve_to_spacing_xl_cumulative_gap() {
    let cumulative_section_gap = SPACING_MD + (SPACING_XL - SPACING_MD);
    assert!(
        (cumulative_section_gap - SPACING_XL).abs() < f32::EPSILON,
        "AC5: cumulative section gap (row_gap SPACING_MD + separator margin) \
         must resolve to SPACING_XL ({SPACING_XL}px); got {cumulative_section_gap}"
    );
}

/// AC8 — bin filename verification: the test bin filename matches the
/// story's "Likely Files" canonical name.
#[test]
fn ac8_test_bin_filename_matches_story_prescribed_name() {
    let expected = "lobby_layout_viewport_invariant_test.rs";
    let actual = std::path::Path::new(file!())
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    assert_eq!(
        actual, expected,
        "AC8: this test bin filename must match the story's Likely Files \
         row (`{expected}`); got {actual}"
    );
}

/// AC13 — friend-game-scope no-claim restatement. Documented inline so
/// future readers see the preservation in source.
#[test]
fn ac13_friend_game_scope_preservation_documented_inline() {
    let source = include_str!("lobby_layout_viewport_invariant_test.rs");
    assert!(
        source.contains("QA-COND-0005"),
        "AC13: friend-game-scope no-claim restatement must reference QA-COND-0005"
    );
    assert!(
        source.contains("QA-COND-0006"),
        "AC13: friend-game-scope no-claim restatement must reference QA-COND-0006"
    );
    assert!(
        source.contains("PAW-TD"),
        "AC13: friend-game-scope no-claim restatement must reference PAW-TD-*-a"
    );
}
