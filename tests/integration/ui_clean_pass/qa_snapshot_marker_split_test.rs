//! S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 integration tests.
//!
//! Covers the acceptance criteria for the QA snapshot marker split
//! (PROMPT 1077 SOURCE-1077-08 + SOURCE-1077-09 + SOURCE-1077-16):
//!
//! - **AC1 / AC2** per-sub-surface root markers exist for HUD, hand, and
//!   shop/auction, and [`UiCountQueries`] surfaces them via the new
//!   `*_visible` fields in [`UiCounts`].
//! - **AC3** every per-sub-surface count honours the entity's own
//!   `Visibility != Hidden` filter — spawning a marker entity with
//!   `Visibility::Hidden` contributes 0 to its count.
//! - **AC4** `connection_lost_overlay_visible` flips with the overlay
//!   root's Visibility.
//! - **AC5** `result_screen_visible` flips with the result-screen root's
//!   Visibility.
//! - **AC6** `format_snapshot_id` uses the `pre-session-` prefix when
//!   `ClientSessionIdentity.session_id` is `None`, and `{session_id}-`
//!   when it is `Some(_)`.
//! - **AC7** two distinct `session_id` values produce non-aliasing
//!   snapshot directory names.
//! - **AC8** legacy universal counts (`hud_entities`, `hand_ui_entities`,
//!   `shop_auction_entities`, `connection_lost_overlay_roots`,
//!   `result_screen_roots`) remain populated alongside the new fields so
//!   PROMPT 1022 / 1034 / 1036 historical comparisons still resolve.
//! - **AC9 (a)/(b)** marker + Visibility::Visible contributes; marker +
//!   Visibility::Hidden does not.
//! - **AC10** `QASnapshotConfig::from_env_values` still honours
//!   `CCGS_QA_SNAPSHOT=1` (smoke-level: regression guard against any
//!   accidental contract change).
//!
//! These tests exercise [`UiCountQueries`] from a `MinimalPlugins` Bevy
//! world — visibility propagation (`InheritedVisibility`) is intentionally
//! NOT relied on; the filter is the marker entity's *own* `Visibility`
//! component, which matches the documented `*_visible` semantic.
//!
//! No protocol shape change, no server change, no shared change.
//! See `production/epics/ui-clean-pass/story-019-qa-snapshot-marker-split.md`.

#![allow(deprecated)]

use std::path::PathBuf;

use bevy::ecs::system::SystemState;
use bevy::prelude::*;

use client::presentation::qa_snapshot::{
    build_snapshot, QASnapshotConfig, ScreenshotInfo, UiCountQueries, UiCounts,
    QA_SCREENSHOT_FILENAME, QA_SCREENSHOT_FORMAT, QA_SNAPSHOT_ENV_VAR,
    QA_SNAPSHOT_PRE_SESSION_PREFIX, SCREENSHOT_STATUS_PENDING,
};
use client::state::ClientSessionIdentity;
use client::ui::hand::{HandBarRoot, HandDraftGridSlotRoot, HandFanRoot, PlacementActionPanelRoot};
use client::ui::hud::{
    HudBottomStripRoot, HudDimOverlayRoot, HudRoot, HudScoreboardDotRoot, HudTopStripRoot,
};
use client::ui::shop_auction::ShopAuctionPanelRoot;

fn placeholder_screenshot() -> ScreenshotInfo {
    ScreenshotInfo {
        relative_path: QA_SCREENSHOT_FILENAME.to_string(),
        absolute_path: format!("/abs/{QA_SCREENSHOT_FILENAME}"),
        format: QA_SCREENSHOT_FORMAT.to_string(),
        requested_at_ms: 0,
        status: SCREENSHOT_STATUS_PENDING.to_string(),
        captured_at_ms: None,
        error: None,
    }
}

fn run_ui_count_snapshot(world: &mut World) -> UiCounts {
    let mut state: SystemState<UiCountQueries> = SystemState::new(world);
    let queries = state.get(world);
    queries.snapshot()
}

/// AC1 + AC9(a): each per-sub-surface root marker spawned with
/// `Visibility::Visible` contributes `1` to the corresponding visible
/// count.
#[test]
fn each_per_sub_surface_marker_with_visible_visibility_contributes_to_count() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let world = app.world_mut();

    world.spawn((HudRoot, Visibility::Visible));
    world.spawn((HudTopStripRoot, Visibility::Visible));
    world.spawn((HudBottomStripRoot, Visibility::Visible));
    world.spawn((HudScoreboardDotRoot, Visibility::Visible));
    world.spawn((HudDimOverlayRoot, Visibility::Visible));
    world.spawn((HandBarRoot, Visibility::Visible));
    world.spawn((HandFanRoot, Visibility::Visible));
    world.spawn((HandDraftGridSlotRoot, Visibility::Visible));
    world.spawn((PlacementActionPanelRoot, Visibility::Visible));
    world.spawn((ShopAuctionPanelRoot::DraftOffering, Visibility::Visible));
    world.spawn((ShopAuctionPanelRoot::Shop, Visibility::Visible));
    world.spawn((ShopAuctionPanelRoot::Auction, Visibility::Visible));
    world.spawn((ShopAuctionPanelRoot::ShopFooter, Visibility::Visible));
    world.spawn((ShopAuctionPanelRoot::Toast, Visibility::Visible));
    world.spawn((ShopAuctionPanelRoot::SettlementOverlay, Visibility::Visible));

    let counts = run_ui_count_snapshot(world);

    assert_eq!(counts.hud_root_visible, 1, "hud_root visible should be 1");
    assert_eq!(counts.hud_top_strip_visible, 1);
    assert_eq!(counts.hud_bottom_strip_visible, 1);
    assert_eq!(counts.hud_scoreboard_dot_visible, 1);
    assert_eq!(counts.hud_dim_overlay_visible, 1);
    assert_eq!(counts.hand_bar_visible, 1);
    assert_eq!(counts.hand_fan_visible, 1);
    assert_eq!(counts.hand_draft_grid_slot_visible, 1);
    assert_eq!(counts.placement_action_panel_visible, 1);
    assert_eq!(counts.shop_draft_offering_visible, 1);
    assert_eq!(counts.shop_panel_visible, 1);
    assert_eq!(counts.auction_panel_visible, 1);
    assert_eq!(counts.shop_footer_visible, 1);
    assert_eq!(counts.auction_toast_visible, 1);
    assert_eq!(counts.settlement_overlay_visible, 1);
}

/// AC1 + AC9(a): repeated marker spawns add up — three visible draft-grid
/// slots should appear as `hand_draft_grid_slot_visible == 3`.
#[test]
fn repeated_marker_spawns_accumulate_into_visible_count() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let world = app.world_mut();

    for _ in 0..3 {
        world.spawn((HandDraftGridSlotRoot, Visibility::Visible));
    }
    world.spawn((HandDraftGridSlotRoot, Visibility::Hidden));

    let counts = run_ui_count_snapshot(world);

    assert_eq!(
        counts.hand_draft_grid_slot_visible, 3,
        "three Visible draft-grid slots should count; the Hidden one must not"
    );
}

/// AC3 + AC9(b): marker entities spawned with `Visibility::Hidden` are
/// excluded from every per-sub-surface visible count.
#[test]
fn hidden_visibility_excludes_marker_from_per_sub_surface_counts() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let world = app.world_mut();

    world.spawn((HudRoot, Visibility::Hidden));
    world.spawn((HudTopStripRoot, Visibility::Hidden));
    world.spawn((HudBottomStripRoot, Visibility::Hidden));
    world.spawn((HudScoreboardDotRoot, Visibility::Hidden));
    world.spawn((HudDimOverlayRoot, Visibility::Hidden));
    world.spawn((HandBarRoot, Visibility::Hidden));
    world.spawn((HandFanRoot, Visibility::Hidden));
    world.spawn((HandDraftGridSlotRoot, Visibility::Hidden));
    world.spawn((PlacementActionPanelRoot, Visibility::Hidden));
    world.spawn((ShopAuctionPanelRoot::DraftOffering, Visibility::Hidden));
    world.spawn((ShopAuctionPanelRoot::Shop, Visibility::Hidden));
    world.spawn((ShopAuctionPanelRoot::Auction, Visibility::Hidden));
    world.spawn((ShopAuctionPanelRoot::ShopFooter, Visibility::Hidden));
    world.spawn((ShopAuctionPanelRoot::Toast, Visibility::Hidden));
    world.spawn((ShopAuctionPanelRoot::SettlementOverlay, Visibility::Hidden));

    let counts = run_ui_count_snapshot(world);

    assert_eq!(counts.hud_root_visible, 0);
    assert_eq!(counts.hud_top_strip_visible, 0);
    assert_eq!(counts.hud_bottom_strip_visible, 0);
    assert_eq!(counts.hud_scoreboard_dot_visible, 0);
    assert_eq!(counts.hud_dim_overlay_visible, 0);
    assert_eq!(counts.hand_bar_visible, 0);
    assert_eq!(counts.hand_fan_visible, 0);
    assert_eq!(counts.hand_draft_grid_slot_visible, 0);
    assert_eq!(counts.placement_action_panel_visible, 0);
    assert_eq!(counts.shop_draft_offering_visible, 0);
    assert_eq!(counts.shop_panel_visible, 0);
    assert_eq!(counts.auction_panel_visible, 0);
    assert_eq!(counts.shop_footer_visible, 0);
    assert_eq!(counts.auction_toast_visible, 0);
    assert_eq!(counts.settlement_overlay_visible, 0);
}

/// AC3: `Visibility::Inherited` is treated as "not Hidden" → the marker
/// entity's own component value is the only signal we consult. Documents
/// the intentional choice to NOT depend on `InheritedVisibility`
/// propagation (which `MinimalPlugins` does not register).
#[test]
fn inherited_visibility_counts_as_visible() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let world = app.world_mut();

    world.spawn((HudTopStripRoot, Visibility::Inherited));
    world.spawn((HandFanRoot, Visibility::Inherited));

    let counts = run_ui_count_snapshot(world);

    assert_eq!(
        counts.hud_top_strip_visible, 1,
        "Visibility::Inherited counts as visible — see is_visibility_visible"
    );
    assert_eq!(counts.hand_fan_visible, 1);
}

/// AC4: the connection-lost overlay's `_visible` flag flips with its own
/// `Visibility` component — replacing the constant-1 reading flagged by
/// SOURCE-1077-09.
#[test]
fn connection_lost_overlay_visible_honours_own_visibility() {
    use client::presentation::connection_lost_overlay::ConnectionLostOverlayRoot;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let world = app.world_mut();

    let root = world
        .spawn((ConnectionLostOverlayRoot, Visibility::Hidden))
        .id();

    let counts_hidden = run_ui_count_snapshot(world);
    assert_eq!(
        counts_hidden.connection_lost_overlay_visible, 0,
        "hidden overlay must NOT count under the new visible-aware field"
    );
    // Legacy `connection_lost_overlay_roots` keeps reading 1 — that's the
    // back-compat reading PROMPT 1022 historical comparisons rely on.
    assert_eq!(counts_hidden.connection_lost_overlay_roots, 1);

    *world.entity_mut(root).get_mut::<Visibility>().unwrap() = Visibility::Visible;
    let counts_visible = run_ui_count_snapshot(world);
    assert_eq!(counts_visible.connection_lost_overlay_visible, 1);
    assert_eq!(counts_visible.connection_lost_overlay_roots, 1);
}

/// AC5: the result-screen overlay's `_visible` flag flips with its own
/// `Visibility` component (same pattern as AC4 for SOURCE-1077-09).
#[test]
fn result_screen_visible_honours_own_visibility() {
    use client::presentation::result_screen::ResultScreenRoot;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let world = app.world_mut();

    let root = world.spawn((ResultScreenRoot, Visibility::Hidden)).id();

    let counts_hidden = run_ui_count_snapshot(world);
    assert_eq!(counts_hidden.result_screen_visible, 0);
    assert_eq!(counts_hidden.result_screen_roots, 1);

    *world.entity_mut(root).get_mut::<Visibility>().unwrap() = Visibility::Visible;
    let counts_visible = run_ui_count_snapshot(world);
    assert_eq!(counts_visible.result_screen_visible, 1);
    assert_eq!(counts_visible.result_screen_roots, 1);
}

/// AC6 + AC9(c): no `session_id` known → `pre-session-` prefix on
/// `snapshot.snapshot_id` and on the captured directory name.
#[test]
fn pre_session_prefix_used_when_session_id_is_none() {
    let identity = ClientSessionIdentity::default();
    assert!(identity.session_id.is_none());

    let snapshot = build_snapshot(
        42,
        1_700_000_000_000,
        placeholder_screenshot(),
        None,
        None,
        None,
        Some(identity),
        None,
        UiCounts::default(),
    );

    let expected = format!(
        "{QA_SNAPSHOT_PRE_SESSION_PREFIX}-{counter:06}-{unix_ms}",
        counter = 42u64,
        unix_ms = 1_700_000_000_000u128,
    );
    assert_eq!(snapshot.snapshot_id, expected);
}

/// AC6 + AC9(d): with a handshake-assigned `session_id`, the snapshot id
/// is prefixed by that id token so concurrent-client captures sort by
/// client first.
#[test]
fn session_id_prefix_used_when_session_id_is_some() {
    let identity = ClientSessionIdentity {
        player_id: None,
        session_id: Some(0xDEAD_BEEFu64),
        session_token: None,
    };

    let snapshot = build_snapshot(
        7,
        1_700_000_000_005,
        placeholder_screenshot(),
        None,
        None,
        None,
        Some(identity),
        None,
        UiCounts::default(),
    );

    let expected = format!(
        "{session_id}-{counter:06}-{unix_ms}",
        session_id = 0xDEAD_BEEFu64,
        counter = 7u64,
        unix_ms = 1_700_000_000_005u128,
    );
    assert_eq!(snapshot.snapshot_id, expected);
}

/// AC7: two distinct `session_id` values produce non-aliasing snapshot
/// directory names — even when counter and unix_millis collide (the
/// SOURCE-1077-16 collision condition before the prefix change).
#[test]
fn two_clients_with_distinct_session_ids_do_not_alias() {
    let identity_a = ClientSessionIdentity {
        player_id: None,
        session_id: Some(11),
        session_token: None,
    };
    let identity_b = ClientSessionIdentity {
        player_id: None,
        session_id: Some(22),
        session_token: None,
    };

    // Same counter, same unix_millis — pre-fix this collision is exactly
    // the SOURCE-1077-16 symptom.
    let a = build_snapshot(
        0,
        1_700_000_000_000,
        placeholder_screenshot(),
        None,
        None,
        None,
        Some(identity_a),
        None,
        UiCounts::default(),
    );
    let b = build_snapshot(
        0,
        1_700_000_000_000,
        placeholder_screenshot(),
        None,
        None,
        None,
        Some(identity_b),
        None,
        UiCounts::default(),
    );
    assert_ne!(a.snapshot_id, b.snapshot_id);
    assert!(a.snapshot_id.starts_with("11-"));
    assert!(b.snapshot_id.starts_with("22-"));

    // Confirm the same id pair, when used as a directory name, also
    // differs — i.e. the on-disk layout AC7 expects (worker-injected
    // fixture identity in lieu of a real two-client harness).
    let dir_a = PathBuf::from("qa-snapshots").join(&a.snapshot_id);
    let dir_b = PathBuf::from("qa-snapshots").join(&b.snapshot_id);
    assert_ne!(dir_a, dir_b);
}

/// AC8: legacy universal counts remain populated alongside the new
/// `*_visible` fields so historical PROMPT 1022 / 1034 / 1036 snapshot
/// comparisons still resolve. Asserts via the
/// `qa_snapshot_overlay_roots` field (still non-deprecated) AND
/// `hud_entities` (deprecated but populated).
#[test]
fn legacy_universal_counts_remain_populated_alongside_new_fields() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let world = app.world_mut();

    // Universal-marker entities — these contribute to the legacy
    // spawned-tree counts only.
    #[allow(deprecated)]
    {
        use client::ui::hand::HandUiEntity;
        use client::ui::hud::HudEntity;
        use client::ui::shop_auction::ShopAuctionUiEntity;
        world.spawn(HudEntity);
        world.spawn(HudEntity);
        world.spawn(HandUiEntity);
        world.spawn(HandUiEntity);
        world.spawn(HandUiEntity);
        world.spawn(ShopAuctionUiEntity);
    }

    // Also spawn one per-sub-surface marker so the visible count is
    // independently populated.
    world.spawn((HudTopStripRoot, Visibility::Visible));

    let counts = run_ui_count_snapshot(world);

    assert_eq!(
        counts.hud_entities, 2,
        "legacy hud_entities must reflect spawned-tree size"
    );
    assert_eq!(counts.hand_ui_entities, 3);
    assert_eq!(counts.shop_auction_entities, 1);
    assert_eq!(counts.hud_top_strip_visible, 1);
}

/// AC10: the `CCGS_QA_SNAPSHOT` env-var name and `=1` activation rule
/// are unchanged by this refactor.
#[test]
fn ccgs_qa_snapshot_env_contract_preserved() {
    assert_eq!(QA_SNAPSHOT_ENV_VAR, "CCGS_QA_SNAPSHOT");

    // `dev_default_enabled` is the unset/empty fallback; pick `false` so
    // the explicit `=1` / `=0` paths are observable independently.
    let enabled = QASnapshotConfig::from_env_values(Some("1"), None, false);
    assert!(
        enabled.enabled,
        "CCGS_QA_SNAPSHOT=1 must continue to force-enable the overlay"
    );
    let disabled = QASnapshotConfig::from_env_values(Some("0"), None, true);
    assert!(
        !disabled.enabled,
        "CCGS_QA_SNAPSHOT=0 must continue to force-disable the overlay even when the dev default is enabled"
    );
}
