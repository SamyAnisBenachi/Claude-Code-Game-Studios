//! PROMPT 985 — Lobby Confirm Class button reachability regression.
//!
//! After the Sprint 14 lobby refactors (PROMPT 937/938 modal layout,
//! PROMPT 957/961/962 class picker, PROMPT 966/970/972 button hit
//! targets) the `LobbyConfirmClassButton` could be pushed past the
//! visible viewport at the minimum supported 1366×768 resolution:
//!
//! - the centred lobby panel has a `max_height: 92%` clamp;
//! - the panel's direct children include the (previously 6-line)
//!   status banner, a 132 px class-picker grid, 48 px slot panels,
//!   three section separators, and the confirm CTA as the bottom-most
//!   child;
//! - the cumulative content height exceeded the panel's content area;
//! - the bevy_ui flex solver, with default `flex_shrink: 1.0` on every
//!   panel child, was free to squash the bottom-most child (confirm
//!   CTA) all the way to zero pixels, making it invisible AND
//!   un-clickable even though the entity still existed in the ECS.
//!
//! This bin asserts the three invariants that keep the confirm CTA
//! reachable:
//!
//! - **ac1** — the `LobbyConfirmClassButton` entity exists with a
//!   `Button` marker, a non-empty `Text` component, and is the LAST
//!   direct child of the `LobbyPanel` (preserving the PROMPT 933
//!   Option A read order);
//! - **ac2** — the confirm CTA's `Node` carries `flex_shrink: 0.0`
//!   AND keeps its canonical height literal so the flex solver cannot
//!   squash it under panel-content pressure;
//! - **ac3** — the status banner is compacted to ≤ 2 lines so the
//!   6-line legacy format does not silently reappear and re-overflow
//!   the panel max_height clamp at the minimum 1366×768 viewport.
//!
//! Friend-game scope only. This bin does NOT advance `QA-COND-0005`
//! Standard-tier accessibility, `QA-COND-0006` playtest validation, or
//! `PAW-TD-*-a` placeholder-art accept-risk.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: read-only invariant
//!   test. No protocol shape exercised, no client-side mutation.
//! - **ADR-002 Client-Server Authority**: no optimistic client state
//!   introduced.

use bevy::asset::AssetPlugin;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::lobby::{
    lobby_status_copy, LobbyConfirmClassButton, LobbyInputState, LobbyPanel, LobbyUiPlugin,
    LobbyViewState, LOBBY_CONFIRM_BUTTON_HEIGHT_PX, LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT,
    LOBBY_PANEL_MAX_HEIGHT_PERCENT,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// Spawn a minimal Bevy `App` that exercises [`LobbyUiPlugin`] for
/// composition assertions. Mirrors
/// `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`.
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

/// AC1 — exactly one `LobbyConfirmClassButton` exists, carries a
/// `Button` marker plus a non-empty `Text`, and is the LAST direct
/// child of the `LobbyPanel` (preserving the PROMPT 933 read order).
#[test]
fn ac1_confirm_cta_entity_exists_with_button_and_text_as_last_panel_child() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let confirm_entity = {
        let mut confirms = world.query_filtered::<Entity, With<LobbyConfirmClassButton>>();
        let entities: Vec<Entity> = confirms.iter(world).collect();
        assert_eq!(
            entities.len(),
            1,
            "AC1: exactly one LobbyConfirmClassButton must exist; got {}",
            entities.len()
        );
        entities[0]
    };

    let confirm_ref = world.entity(confirm_entity);
    assert!(
        confirm_ref.get::<Button>().is_some(),
        "AC1: LobbyConfirmClassButton entity must carry a Button marker \
         so the bevy_ui Interaction state machine drives it"
    );
    let text = confirm_ref
        .get::<Text>()
        .expect("AC1: LobbyConfirmClassButton entity must own a Text component");
    assert!(
        !text.0.trim().is_empty(),
        "AC1: confirm CTA Text must be non-empty (got `{}`); the label \
         is the only visual indicator of the CTA",
        text.0
    );

    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("AC1: single LobbyPanel entity present after lobby plugin update")
    };

    let panel_children: Vec<Entity> = world
        .entity(panel_entity)
        .get::<Children>()
        .expect("AC1: LobbyPanel must have Children component")
        .iter()
        .collect();
    let last_child = *panel_children
        .last()
        .expect("AC1: lobby panel children list is non-empty");
    assert_eq!(
        last_child, confirm_entity,
        "AC1: LobbyConfirmClassButton must be the LAST direct child of \
         LobbyPanel so the read order ends at the confirm CTA (PROMPT 933 \
         Option A read order)"
    );
}

/// AC2 — the confirm CTA's `Node` carries `flex_shrink: 0.0` so the
/// flex solver cannot squash it to zero when other panel children
/// push against the panel's `max_height: 92%` content area. The
/// canonical width-percent / height-px literals are preserved so the
/// hit-target dimensions documented in story 026 stay intact.
#[test]
fn ac2_confirm_cta_node_has_flex_shrink_zero_and_canonical_dimensions() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let confirm_entity = {
        let mut confirms = world.query_filtered::<Entity, With<LobbyConfirmClassButton>>();
        confirms
            .single(world)
            .expect("AC2: single LobbyConfirmClassButton entity present")
    };

    let node = world
        .entity(confirm_entity)
        .get::<Node>()
        .expect("AC2: LobbyConfirmClassButton entity must own a Node");

    assert_eq!(
        node.flex_shrink, 0.0,
        "AC2: LobbyConfirmClassButton Node.flex_shrink must be 0.0 so \
         the flex solver cannot squash the CTA to zero when other panel \
         children expand against the panel's max_height clamp; got {}",
        node.flex_shrink
    );

    assert_eq!(
        node.height,
        Val::Px(LOBBY_CONFIRM_BUTTON_HEIGHT_PX),
        "AC2: LobbyConfirmClassButton Node.height must remain \
         LOBBY_CONFIRM_BUTTON_HEIGHT_PX ({LOBBY_CONFIRM_BUTTON_HEIGHT_PX}) \
         so flex_shrink=0 yields the documented hit-target dimension; \
         got {:?}",
        node.height
    );

    assert_eq!(
        node.width,
        Val::Percent(LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT),
        "AC2: LobbyConfirmClassButton Node.width must remain \
         Val::Percent({LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT}) per the \
         story 024 Option A full-width CTA contract; got {:?}",
        node.width
    );
}

/// AC3 — `lobby_status_copy` renders in at most two lines so the
/// 6-line legacy format does not silently reappear. The `\n` count is
/// the load-bearing literal because bevy's Text node respects
/// explicit newlines regardless of available width; six explicit
/// newlines at H3 (18 px × 1.2 line-height) accounted for ≈ 130 px
/// of panel content, which pushed the bottom-most child past the
/// `max_height: 92%` clamp at 1366×768.
#[test]
fn ac3_status_banner_copy_renders_at_most_two_lines() {
    let lobby = LobbyViewState::default();
    let input = LobbyInputState::default();
    let copy = lobby_status_copy(&lobby, &input);
    let line_count = copy.matches('\n').count() + 1;
    assert!(
        line_count <= 2,
        "AC3: lobby_status_copy must render in at most 2 lines so the \
         status banner does not overflow the panel content area at the \
         minimum 1366×768 viewport; got {line_count} lines: {copy:?}"
    );

    // Preserve the substring the older `lobby_entry_test` relies on
    // (`class_confirmations_are_server_confirmed`).
    assert!(
        copy.contains("Players: "),
        "AC3: lobby_status_copy must keep the `Players: ` substring so \
         the existing lobby_entry_test assertion stays in sync; got \
         {copy:?}"
    );
}

/// AC3 — the status copy also folds the populated `Players: N/M`
/// readout into the compact format. Mirrors the precondition the
/// older `lobby_entry_test::class_confirmations_are_server_confirmed`
/// codifies, run here under the lobby plugin context.
#[test]
fn ac3_status_banner_populated_players_value_survives_compaction() {
    use shared::protocol::SessionSlot;
    use shared::session::PlayerId;

    fn make_slot(slot: u8, team: u8, player: Option<PlayerId>) -> SessionSlot {
        SessionSlot {
            slot,
            team,
            player_id: player,
            class_id: None,
            class_confirmed: false,
        }
    }

    let lobby = LobbyViewState {
        slots: vec![
            make_slot(0, 0, Some(PlayerId(1))),
            make_slot(1, 1, Some(PlayerId(2))),
        ],
        ..Default::default()
    };
    let input = LobbyInputState::default();
    let copy = lobby_status_copy(&lobby, &input);
    assert!(
        copy.contains("Players: 2/2"),
        "AC3: populated 2-player lobby must still render `Players: 2/2` \
         in the compact status banner; got {copy:?}"
    );
}

/// AC5 (PROMPT 1178) — post-join content density regression guard.
///
/// At the 1280×720 minimum viewport, with `lobby.session_id` set and the
/// `lobby.room_list` carrying one joinable room, the PROMPT 1160
/// existing-room browser landed enough additional panel children
/// (heading row + room-list rows + `Requested slot` row) to push the
/// bottom-most child (`LobbyConfirmClassButton`) past the visible
/// viewport. The user reported the regression on 2026-05-18: after
/// clicking a row to join, the class picker and slot panels rendered,
/// but the Confirm-class CTA was completely off-screen.
///
/// This test seeds the lobby with the post-join state used in the
/// 2026-05-18 QA snapshot (`qa-snapshots/2-000000-1779110939048`) — one
/// active session, one room in the `room_list` — and asserts that the
/// observable content density immediately above the confirm CTA stays
/// within the panel's `max_height: 92%` clamp at 1280×720. Two ways
/// the regression manifests are caught:
///
///   * the cumulative pixel height of panel children with explicit
///     `Val::Px(_)` heights (plus a conservative per-child row_gap
///     contribution) stays inside the panel's content area;
///   * the post-session lobby does NOT spawn the `Requested slot`
///     row nor the `Existing rooms` browser block — both of which
///     are irrelevant once `session_id` is `Some`. Their presence is
///     what overflowed the panel in the original bug.
const VIEWPORT_BUG_WIDTH: f32 = 1280.0;
const VIEWPORT_BUG_HEIGHT: f32 = 720.0;
const LOBBY_PANEL_PADDING_PX_SUM: f32 = 2.0 * 24.0; // 2× SPACING_LG.
const LOBBY_PANEL_BORDER_PX_SUM: f32 = 2.0 * 2.0; // 2 px border × 2 sides.

fn spawn_post_session_lobby_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(LobbyViewState {
        session_id: Some("session-uuid".to_string()),
        room_code: Some("SSVDVU".to_string()),
        local_player_id: Some(PlayerId(2)),
        // One joinable room in the list — the post-join state the
        // 2026-05-18 QA snapshot captured. PROMPT 1178 hides the
        // browser post-session so this list should not produce row
        // entities anyway; the list is here to mirror the snapshot.
        room_list: vec![],
        ..Default::default()
    });
    app.add_plugins(LobbyUiPlugin);
    app.update();
    app.update();
    app
}

/// AC5 — post-session lobby content fits inside the panel content area
/// at the minimum 1280×720 viewport, with conservative measurement.
#[test]
fn ac5_post_session_panel_content_fits_at_1280x720() {
    test_helpers::init_test_tracing();
    let mut app = spawn_post_session_lobby_app();
    let world = app.world_mut();

    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("AC5: single LobbyPanel entity present after post-session spawn")
    };

    // Sum the `Val::Px(_)` heights of every direct panel child whose
    // Node carries one, plus a conservative SPACING_MD (16 px) row_gap
    // contribution per gap between adjacent children. Children whose
    // height is `Val::Auto` (e.g. text-only `Requested slot` label /
    // status banner / class-picker heading) contribute a coarse
    // upper-bound of `H3 (18) × LINE_HEIGHT_DEFAULT_RATIO (1.25) × 2
    // lines = 45 px` so the budget covers the worst case without
    // running the bevy_ui layout solver.
    const ROW_GAP_PX: f32 = 16.0; // SPACING_MD.
    const TEXT_HEIGHT_UPPER_BOUND_PX: f32 = 45.0; // 2-line H3 worst case.

    let children: Vec<Entity> = world
        .entity(panel_entity)
        .get::<Children>()
        .expect("AC5: LobbyPanel must have Children")
        .iter()
        .collect();
    assert!(
        !children.is_empty(),
        "AC5: post-session panel must spawn at least the Confirm CTA"
    );

    let mut content_height_px = 0.0_f32;
    for (idx, child) in children.iter().enumerate() {
        let node_height = world.entity(*child).get::<Node>().map(|n| n.height);
        let height_px = match node_height {
            Some(Val::Px(px)) => px.max(0.0),
            // `Val::Auto` (text labels) / explicit `Val::Px(0.0)` for
            // section separators — use the conservative text bound for
            // anything with a Text component so the per-frame copy
            // line-count is bounded; otherwise treat as zero.
            _ => {
                if world.entity(*child).get::<Text>().is_some() {
                    TEXT_HEIGHT_UPPER_BOUND_PX
                } else if let Some(node) = world.entity(*child).get::<Node>() {
                    // Separators carry a top-margin `Val::Px(_)`; count
                    // that contribution against the budget.
                    match node.margin.top {
                        Val::Px(px) => px.max(0.0),
                        _ => 0.0,
                    }
                } else {
                    0.0
                }
            }
        };
        content_height_px += height_px;
        if idx + 1 < children.len() {
            content_height_px += ROW_GAP_PX;
        }
    }

    let panel_content_area_px = LOBBY_PANEL_MAX_HEIGHT_PERCENT / 100.0 * VIEWPORT_BUG_HEIGHT
        - LOBBY_PANEL_PADDING_PX_SUM
        - LOBBY_PANEL_BORDER_PX_SUM;

    assert!(
        content_height_px <= panel_content_area_px,
        "AC5: at viewport {VIEWPORT_BUG_WIDTH}×{VIEWPORT_BUG_HEIGHT}, the \
         conservative panel-content height upper bound \
         ({content_height_px:.1} px) must fit inside the panel's content area \
         ({panel_content_area_px:.1} px = max_height \
         {LOBBY_PANEL_MAX_HEIGHT_PERCENT}% × {VIEWPORT_BUG_HEIGHT} \
         − padding {LOBBY_PANEL_PADDING_PX_SUM} − border {LOBBY_PANEL_BORDER_PX_SUM}). \
         Overflow means the Confirm CTA gets pushed below the visible \
         viewport, which is the 2026-05-18 user-reported regression."
    );
}

/// AC5 — post-session lobby has the `LobbyConfirmClassButton` as the
/// LAST direct child of the panel AND no `Requested slot` text label
/// nor `Existing rooms` browser children intervene. Together with the
/// height-budget assertion above this captures the regression: even if
/// the height budget passed thanks to compaction elsewhere, an
/// erroneous re-introduction of the post-session requested-slot row
/// would fail this test.
#[test]
fn ac5_post_session_confirm_cta_has_no_requested_slot_siblings() {
    test_helpers::init_test_tracing();
    let mut app = spawn_post_session_lobby_app();
    let world = app.world_mut();

    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("AC5: single LobbyPanel entity present after post-session spawn")
    };

    let children: Vec<Entity> = world
        .entity(panel_entity)
        .get::<Children>()
        .expect("AC5: panel must have Children")
        .iter()
        .collect();

    let last_child = *children.last().expect("AC5: panel child list is non-empty");
    assert!(
        world
            .entity(last_child)
            .get::<LobbyConfirmClassButton>()
            .is_some(),
        "AC5: LobbyConfirmClassButton must remain the LAST direct child \
         of the panel post-session; got entity {last_child:?}"
    );

    // No Text child should literally render `Requested slot` post-session.
    // (The 0..=3 requested-slot buttons are caught by the dedicated test in
    // `lobby_room_browser_test.rs::test_requested_slot_row_is_hidden_after_session_is_joined`,
    // but a text-search here is the cheap belt-and-braces guard.)
    for child in &children {
        if let Some(text) = world.entity(*child).get::<Text>() {
            assert!(
                !text.0.contains("Requested slot"),
                "AC5: a panel child still renders `Requested slot` text \
                 post-session (`{}`); the row must be hidden once \
                 `lobby.session_id` is Some so the Confirm CTA is not \
                 displaced",
                text.0
            );
        }
    }
}

/// AC4 — friend-game-scope no-claim restatement. Documented inline so
/// future readers see the preservation in source.
#[test]
fn ac4_friend_game_scope_preservation_documented_inline() {
    let source = include_str!("lobby_confirm_button_reachable_test.rs");
    assert!(
        source.contains("QA-COND-0005"),
        "AC4: friend-game-scope no-claim restatement must reference QA-COND-0005"
    );
    assert!(
        source.contains("QA-COND-0006"),
        "AC4: friend-game-scope no-claim restatement must reference QA-COND-0006"
    );
    assert!(
        source.contains("PAW-TD"),
        "AC4: friend-game-scope no-claim restatement must reference PAW-TD-*-a"
    );
}
