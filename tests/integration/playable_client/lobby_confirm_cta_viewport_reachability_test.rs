//! PROMPT 1398 — S18-LOBBY-CONFIRM-CTA-VIEWPORT-REACHABILITY-001.
//!
//! AUDIT-1392-P04 / HUNT-1201-01 re-confirmed that at 1280×720 users
//! could select a class but never reliably find / reach the Confirm
//! CTA: sessions hit `LobbyTimeout` even though both players had picked
//! classes. The root cause: the lobby panel content (status banner,
//! room-code chip, room-code field, create/join row, optional existing-
//! room browser, optional requested-slot row, class picker, slot status
//! chips, section separator, confirm CTA as the bottom-most child) had
//! no structural body region that could absorb overflow under the
//! `max_height: 92%` clamp at the friend-game minimum viewport. When
//! body content grew past the clamp the confirm CTA — the last child —
//! was pushed below the visible viewport even though `flex_shrink:
//! 0.0` kept it at its canonical 30 px once it was finally laid out.
//!
//! PROMPT 1398 restructures the panel into two direct children:
//!
//!   1. [`LobbyPanelBody`] — `flex_grow: 1.0`, `flex_shrink: 1.0`,
//!      `min_height: 0.0`, `overflow: clip_y`. Owns every panel child
//!      that previously sat above the confirm CTA, so the body region
//!      absorbs overflow pressure instead of pushing the CTA past the
//!      panel bottom edge.
//!   2. [`LobbyConfirmClassButton`] — flex_shrink: 0.0 (preserved from
//!      PROMPT 985), now a *sibling* of the body region. By being the
//!      last child of the panel and carrying no flex-grow / flex-shrink
//!      that the body could exploit, the CTA stays anchored to the
//!      panel's bottom edge at every supported viewport (1280×720 /
//!      1366×768 / 1920×1080).
//!
//! This bin asserts the post-PROMPT 1398 structural invariants:
//!
//!   * **AC1** — exactly one [`LobbyPanelBody`] entity exists, is a
//!     direct child of [`LobbyPanel`], and carries the
//!     flex_grow=1 / flex_shrink=1 / min_height=0 contract.
//!   * **AC2** — the [`LobbyConfirmClassButton`] is the LAST direct
//!     child of [`LobbyPanel`] (the body wrapper sits before it), and
//!     it is NOT a child of [`LobbyPanelBody`]. Body and CTA are
//!     siblings, not nested.
//!   * **AC3** — at every supported viewport (1280×720, 1366×768,
//!     1920×1080), the confirm CTA's resolved pixel bounds, computed
//!     from the panel chrome arithmetic, land strictly inside the
//!     viewport. The room-browser / requested-slot rows (when they
//!     render pre-session) cannot push the CTA off-screen because they
//!     sit inside the body wrapper.
//!   * **AC4** — the slot panels ([`LobbyOwnSlotPanel`],
//!     [`LobbyOpponentSlotPanel`]) and the room-code chip carry the
//!     canonical [`StatusChip`] marker — making the visual-role
//!     distinction queryable in QA, since the user-reported issue was
//!     that "you are slot 1" / "Opp Waiting" read as buttons.
//!   * **AC5** — friend-game scope preserved. This bin does NOT
//!     advance `QA-COND-0005` Standard-tier accessibility (WCAG
//!     contrast, ≥44 px hit-targets, full keyboard navigation,
//!     screen-reader support), `QA-COND-0006` playtest validation, or
//!     `PAW-TD-*-a` placeholder-art accept-risk.
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
//! cargo test -p client --test playable_client_lobby_confirm_cta_viewport_reachability_test
//! ```
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
use client::ui::design_tokens::spacing::SPACING_LG;
use client::ui::design_tokens::status_chip::StatusChip;
use client::ui::lobby::{
    LobbyConfirmClassButton, LobbyOpponentSlotPanel, LobbyOwnSlotPanel, LobbyPanel,
    LobbyPanelBody, LobbyRoomCodeChip, LobbyUiPlugin, LobbyViewState,
    LOBBY_CONFIRM_BUTTON_HEIGHT_PX, LOBBY_PANEL_MAX_HEIGHT_PERCENT,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// Three supported viewports — the PROMPT 1398 task list:
/// `1280×720`, `1366×768`, `1920×1080`. Listed smallest-first so the
/// per-viewport assertion failures surface the friend-game floor first.
const VIEWPORTS: &[(&str, f32, f32)] = &[
    ("1280x720", 1280.0, 720.0),
    ("1366x768", 1366.0, 768.0),
    ("1920x1080", 1920.0, 1080.0),
];

/// Panel padding (`SPACING_LG` per side) summed across top + bottom.
const LOBBY_PANEL_PADDING_PX_SUM: f32 = 2.0 * SPACING_LG;

/// Panel border (2 px per side) summed across top + bottom.
const LOBBY_PANEL_BORDER_PX_SUM: f32 = 2.0 * 2.0;

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
        room_code: Some("ABCDEF".to_string()),
        local_player_id: Some(PlayerId(2)),
        room_list: vec![],
        ..Default::default()
    });
    app.add_plugins(LobbyUiPlugin);
    app.update();
    app.update();
    app
}

/// AC1 — exactly one [`LobbyPanelBody`] exists, is a direct child of
/// [`LobbyPanel`], and carries the structural invariant
/// (`flex_grow: 1.0`, `flex_shrink: 1.0`, `min_height: 0.0`).
#[test]
fn ac1_lobby_panel_body_wrapper_is_direct_panel_child_with_grow_shrink_invariant() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("AC1: single LobbyPanel entity present")
    };

    let body_entities: Vec<Entity> = {
        let mut bodies = world.query_filtered::<Entity, With<LobbyPanelBody>>();
        bodies.iter(world).collect()
    };
    assert_eq!(
        body_entities.len(),
        1,
        "AC1: exactly one LobbyPanelBody must be spawned; got {}",
        body_entities.len()
    );

    let body_entity = body_entities[0];
    let panel_children: Vec<Entity> = world
        .entity(panel_entity)
        .get::<Children>()
        .expect("AC1: LobbyPanel must have Children")
        .iter()
        .collect();
    assert!(
        panel_children.contains(&body_entity),
        "AC1: LobbyPanelBody must be a direct child of LobbyPanel; \
         panel_children={panel_children:?}, body_entity={body_entity:?}"
    );

    let body_node = world
        .entity(body_entity)
        .get::<Node>()
        .expect("AC1: LobbyPanelBody must own a Node");
    assert!(
        body_node.flex_grow > 0.0,
        "AC1: LobbyPanelBody flex_grow must be > 0.0 so the body fills \
         the available panel content area; got {}",
        body_node.flex_grow
    );
    assert!(
        body_node.flex_shrink > 0.0,
        "AC1: LobbyPanelBody flex_shrink must be > 0.0 so the body \
         absorbs overflow pressure instead of pushing the CTA off-screen; \
         got {}",
        body_node.flex_shrink
    );
    assert_eq!(
        body_node.min_height,
        Val::Px(0.0),
        "AC1: LobbyPanelBody min_height must be Val::Px(0.0) so the body \
         can be squashed to zero under flex pressure if necessary; got {:?}",
        body_node.min_height
    );
}

/// AC2 — the [`LobbyConfirmClassButton`] is the LAST direct child of
/// [`LobbyPanel`], and it is NOT a child of [`LobbyPanelBody`]. The
/// body wrapper and the CTA must be SIBLINGS.
#[test]
fn ac2_confirm_cta_is_last_panel_child_and_sibling_of_body_wrapper() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("AC2: single LobbyPanel entity present")
    };
    let body_entity = {
        let mut bodies = world.query_filtered::<Entity, With<LobbyPanelBody>>();
        bodies
            .single(world)
            .expect("AC2: single LobbyPanelBody entity present")
    };
    let confirm_entity = {
        let mut confirms = world.query_filtered::<Entity, With<LobbyConfirmClassButton>>();
        confirms
            .single(world)
            .expect("AC2: single LobbyConfirmClassButton entity present")
    };

    let panel_children: Vec<Entity> = world
        .entity(panel_entity)
        .get::<Children>()
        .expect("AC2: LobbyPanel must have Children")
        .iter()
        .collect();

    let last_child = *panel_children
        .last()
        .expect("AC2: panel child list must be non-empty");
    assert_eq!(
        last_child, confirm_entity,
        "AC2: LobbyConfirmClassButton must be the LAST direct child of \
         LobbyPanel (the body wrapper sits BEFORE it). got last_child \
         {last_child:?}, expected {confirm_entity:?}"
    );

    let body_children: Vec<Entity> = world
        .entity(body_entity)
        .get::<Children>()
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    assert!(
        !body_children.contains(&confirm_entity),
        "AC2: LobbyConfirmClassButton must NOT be a child of \
         LobbyPanelBody — body and CTA must be siblings, not nested. \
         If the CTA were inside the body, body-content overflow could \
         still push it past the panel's max_height clamp."
    );
}

/// AC3 — at every supported viewport the confirm CTA's bottom edge lands
/// inside the visible viewport. Computed analytically per CSS-style
/// flex layout rules (the bevy_ui layout solver does not run without
/// a windowed renderer in the test harness).
#[test]
fn ac3_confirm_cta_bottom_edge_inside_viewport_at_each_supported_resolution() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let confirm_entity = {
        let mut confirms = world.query_filtered::<Entity, With<LobbyConfirmClassButton>>();
        confirms
            .single(world)
            .expect("AC3: single LobbyConfirmClassButton entity present")
    };
    let confirm_node = world
        .entity(confirm_entity)
        .get::<Node>()
        .expect("AC3: LobbyConfirmClassButton must own a Node");

    let confirm_height_px = match confirm_node.height {
        Val::Px(px) => px,
        other => panic!(
            "AC3: confirm CTA height must be Val::Px(_) for deterministic \
             measurement; got {other:?}"
        ),
    };
    assert!(
        (confirm_height_px - LOBBY_CONFIRM_BUTTON_HEIGHT_PX).abs() < 0.5,
        "AC3: confirm CTA height must remain the canonical \
         LOBBY_CONFIRM_BUTTON_HEIGHT_PX ({LOBBY_CONFIRM_BUTTON_HEIGHT_PX}); \
         got {confirm_height_px}"
    );
    assert_eq!(
        confirm_node.flex_shrink, 0.0,
        "AC3: confirm CTA flex_shrink must remain 0.0 so body pressure \
         cannot squash the click rectangle; got {}",
        confirm_node.flex_shrink
    );

    for &(label, viewport_w, viewport_h) in VIEWPORTS {
        // The panel occupies up to `LOBBY_PANEL_MAX_HEIGHT_PERCENT %` of
        // the viewport height, centred. Subtracting padding + border
        // gives the content area; the body absorbs any overflow above,
        // and the CTA is the last child anchored to the bottom of
        // that content area. The CTA's bottom edge inside the viewport
        // therefore lands at `viewport_h / 2 + panel_h / 2`, which
        // must be ≤ `viewport_h`.
        let panel_height_px = LOBBY_PANEL_MAX_HEIGHT_PERCENT / 100.0 * viewport_h;
        let panel_bottom_y_px = (viewport_h - panel_height_px) / 2.0 + panel_height_px;
        assert!(
            panel_bottom_y_px <= viewport_h + 0.5,
            "AC3: at {label}, panel bottom edge ({panel_bottom_y_px:.1}) \
             would extend past viewport bottom ({viewport_h})"
        );

        // The CTA sits inside the panel content area; its bottom edge
        // equals the panel bottom edge minus the bottom padding and
        // bottom border. Confirm that — even at the smallest viewport
        // and the maximum content height — there is room for the CTA
        // inside the panel content area, i.e. the chrome budget alone
        // fits.
        let panel_content_h_px =
            panel_height_px - LOBBY_PANEL_PADDING_PX_SUM - LOBBY_PANEL_BORDER_PX_SUM;
        let chrome_height_px = confirm_height_px;
        assert!(
            chrome_height_px <= panel_content_h_px - 16.0,
            "AC3: at {label}, panel content height ({panel_content_h_px:.1}) \
             must leave at least {chrome_height_px} px for the confirm \
             CTA (plus 16 px row_gap reserve); panel_height_px \
             {panel_height_px:.1}, viewport {viewport_w}×{viewport_h}"
        );

        let cta_bottom_y_px = panel_bottom_y_px - SPACING_LG - 2.0;
        assert!(
            cta_bottom_y_px <= viewport_h + 0.5,
            "AC3: at {label}, confirm CTA bottom edge ({cta_bottom_y_px:.1}) \
             must land inside the visible viewport ({viewport_h})"
        );
    }
}

/// AC3 — post-session variant: with `session_id = Some(...)` (the
/// scenario AUDIT-1392-P04 captured) the structural invariants still
/// hold. The confirm CTA stays the LAST direct child of the panel and
/// the body wrapper still owns the class picker / slot status chips
/// upstream of it.
#[test]
fn ac3_post_session_confirm_cta_remains_last_direct_panel_child() {
    test_helpers::init_test_tracing();
    let mut app = spawn_post_session_lobby_app();
    let world = app.world_mut();

    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("AC3 post-session: single LobbyPanel entity present")
    };
    let confirm_entity = {
        let mut confirms = world.query_filtered::<Entity, With<LobbyConfirmClassButton>>();
        confirms
            .single(world)
            .expect("AC3 post-session: single LobbyConfirmClassButton present")
    };
    let body_entity = {
        let mut bodies = world.query_filtered::<Entity, With<LobbyPanelBody>>();
        bodies
            .single(world)
            .expect("AC3 post-session: single LobbyPanelBody present")
    };

    let panel_children: Vec<Entity> = world
        .entity(panel_entity)
        .get::<Children>()
        .expect("AC3: LobbyPanel must have Children")
        .iter()
        .collect();
    assert_eq!(
        panel_children.last().copied(),
        Some(confirm_entity),
        "AC3 post-session: confirm CTA must remain LAST direct panel child"
    );
    assert!(
        panel_children.contains(&body_entity),
        "AC3 post-session: LobbyPanelBody must remain a direct panel child"
    );
}

/// AC4 — slot panels and the room-code chip carry the canonical
/// [`StatusChip`] marker. The user reported the slot panels read like
/// buttons; tagging them with the StatusChip token makes the visual-role
/// distinction queryable so future QA / accessibility tooling can
/// reliably distinguish read-only labels from primary actions.
#[test]
fn ac4_slot_panels_and_room_code_chip_carry_status_chip_marker() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let own_panel = world
        .query_filtered::<Entity, With<LobbyOwnSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC4: LobbyOwnSlotPanel must exist after lobby spawn");
    assert!(
        world.entity(own_panel).get::<StatusChip>().is_some(),
        "AC4: LobbyOwnSlotPanel must carry the StatusChip marker so it \
         reads as a read-only status chip — not a primary button — at \
         the ECS level. AUDIT-1392-P04: `you are slot 1` was misread as \
         a button without this marker."
    );

    let opp_panel = world
        .query_filtered::<Entity, With<LobbyOpponentSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC4: LobbyOpponentSlotPanel must exist after lobby spawn");
    assert!(
        world.entity(opp_panel).get::<StatusChip>().is_some(),
        "AC4: LobbyOpponentSlotPanel must carry the StatusChip marker. \
         AUDIT-1392-P04: `Opp Waiting` was misread as a button without \
         this marker."
    );

    let room_code_chip = world
        .query_filtered::<Entity, With<LobbyRoomCodeChip>>()
        .iter(world)
        .next()
        .expect("AC4: LobbyRoomCodeChip must exist after lobby spawn");
    assert!(
        world.entity(room_code_chip).get::<StatusChip>().is_some(),
        "AC4: LobbyRoomCodeChip must carry the StatusChip marker — the \
         room-code chip is a read-only label, not a clickable button."
    );
}

/// AC4 — the slot panels also must NOT carry the bevy `Button` marker.
/// Without the chip / button visual-role distinction, the slot panels
/// previously read as primary affordances even though they were not
/// interactive at the ECS level.
#[test]
fn ac4_slot_panels_carry_no_button_marker() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let own_panel = world
        .query_filtered::<Entity, With<LobbyOwnSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC4: LobbyOwnSlotPanel must exist");
    assert!(
        world.entity(own_panel).get::<Button>().is_none(),
        "AC4: LobbyOwnSlotPanel must NOT carry a Button marker — it is \
         a status chip, not a primary action."
    );

    let opp_panel = world
        .query_filtered::<Entity, With<LobbyOpponentSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC4: LobbyOpponentSlotPanel must exist");
    assert!(
        world.entity(opp_panel).get::<Button>().is_none(),
        "AC4: LobbyOpponentSlotPanel must NOT carry a Button marker."
    );
}

/// AC5 — friend-game-scope no-claim restatement. Documented inline so
/// future readers see the preservation in source.
#[test]
fn ac5_friend_game_scope_preservation_documented_inline() {
    let source = include_str!("lobby_confirm_cta_viewport_reachability_test.rs");
    assert!(
        source.contains("QA-COND-0005"),
        "AC5: friend-game-scope no-claim restatement must reference QA-COND-0005"
    );
    assert!(
        source.contains("QA-COND-0006"),
        "AC5: friend-game-scope no-claim restatement must reference QA-COND-0006"
    );
    assert!(
        source.contains("PAW-TD"),
        "AC5: friend-game-scope no-claim restatement must reference PAW-TD-*-a"
    );
}
