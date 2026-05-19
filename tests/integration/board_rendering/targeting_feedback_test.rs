//! Integration tests for the PROMPT 1390 targeting feedback overlay
//! (`S19-BR-PLAYAREA-HIERARCHY-TARGETING-FEEDBACK-001`).
//!
//! Each test exercises one or more story acceptance criteria:
//!
//! - **AC3 — Targeting dim**: `targeting_active_spawns_dim_wash_and_clears_on_idle`
//! - **AC4 — Valid path/rings**: `valid_rings_spawn_for_local_spawn_range_cells`
//! - **AC4 + AC5 colour distinction**: `valid_ring_tint_distinct_from_invalid_marker_tint`
//! - **AC5 — Invalid target**: `out_of_range_board_cell_target_spawns_invalid_marker`
//! - **AC6 — Source-card link**: `board_cell_target_spawns_source_card_link`
//! - **AC7 — Authority boundary**: `targeting_overlay_never_writes_placement_board_view`
//! - **AC8 — Z-order**: `overlay_z_order_below_objectives_and_units`
//! - **AC9 — QA snapshot fields**: `qa_snapshot_exposes_board_envelope_and_targeting_state`
//!
//! The fixture is the canonical production app factory in
//! `tests/helpers/production_app_factory.rs` so every test sees the same
//! plugin composition that ships with the client binary.

use bevy::color::Alpha;
use bevy::prelude::*;
use client::presentation::{
    board_rendering::{
        rendering_constants, BoardEnvelope, SourceCardLink, TargetingDimWash,
        TargetingEndpointRing, TargetingInvalidMarker, TargetingOverlayState, TargetingValidRing,
        TARGETING_INVALID_COLOR, TARGETING_VALID_RING_COLOR,
    },
    qa_snapshot::{
        build_snapshot_with_extras_and_layout, BoardTargetingSnapshot, ExtrasSnapshot,
        LayoutSnapshot, ScreenshotInfo, UiCounts, SCREENSHOT_STATUS_PENDING,
    },
};
use client::ui::hand::{BoardSpawnEdge, GhostPlacementChanged, PlacementBoardView};
use shared::{card::CardId, protocol::PlayTarget, session::PlayerId};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[path = "../../helpers/production_app_factory.rs"]
mod production_app_factory;

fn fixture_app(spawn_range_cells: u8) -> App {
    test_helpers::init_test_tracing();
    let mut app = production_app_factory::production_client_app_in_session();
    // Pin the local player's spawn range so the valid-cell mirror is
    // deterministic across tests; production wires this from
    // `S2CGameSnapshot` / `SpawnRangeChanged`.
    app.world_mut().insert_resource(PlacementBoardView {
        local_player_id: PlayerId(1),
        opponent_player_id: PlayerId(2),
        spawn_edge: BoardSpawnEdge::LowCells,
        spawn_range_cells,
    });
    app.update();
    app
}

fn stage_ghost(app: &mut App, card_id: CardId, target: Option<PlayTarget>) {
    app.world_mut().write_message(GhostPlacementChanged {
        target,
        card_id: Some(card_id),
    });
    app.update();
}

fn count<C: Component>(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&C>();
    q.iter(app.world()).count()
}

#[test]
fn targeting_active_spawns_dim_wash_and_clears_on_idle() {
    // AC3 — When a card actively targets a board cell, a single
    // TargetingDimWash sprite spawns; when targeting clears, it
    // despawns.
    let mut app = fixture_app(2);
    stage_ghost(
        &mut app,
        CardId(10),
        Some(PlayTarget::BoardCell { lane: 1, cell: 1 }),
    );
    assert_eq!(
        count::<TargetingDimWash>(&mut app),
        1,
        "active targeting must spawn exactly one dim wash"
    );

    stage_ghost(&mut app, CardId(10), None);
    assert_eq!(
        count::<TargetingDimWash>(&mut app),
        0,
        "clearing targeting must despawn the dim wash"
    );
}

#[test]
fn valid_rings_spawn_for_local_spawn_range_cells() {
    // AC4 — Each in-range cell renders a distinct ring sprite. With
    // `spawn_range_cells = 2`, the 5-lane × 2-cell prefix yields 10
    // valid rings.
    let mut app = fixture_app(2);
    stage_ghost(
        &mut app,
        CardId(11),
        Some(PlayTarget::BoardCell { lane: 1, cell: 1 }),
    );
    assert_eq!(
        count::<TargetingValidRing>(&mut app),
        10,
        "valid range rings must cover every in-range spawn cell"
    );

    // Confirm rings sit on the expected lanes/cells.
    let mut q = app.world_mut().query::<&TargetingValidRing>();
    let cells: Vec<(u8, u8)> = q
        .iter(app.world())
        .map(|r| (r.lane, r.cell))
        .collect();
    for lane in 1..=5 {
        for cell in [1u8, 2u8] {
            assert!(
                cells.contains(&(lane, cell)),
                "expected valid ring on lane {lane} cell {cell}"
            );
        }
    }
}

#[test]
fn valid_ring_tint_distinct_from_invalid_marker_tint() {
    // AC4 + AC5 — Valid rings (cyan) and invalid markers (red) must use
    // visibly distinct tints. Compare with a 0.2 channel-delta tolerance
    // to keep the test resilient to minor palette tuning.
    let valid = TARGETING_VALID_RING_COLOR.to_srgba();
    let invalid = TARGETING_INVALID_COLOR.to_srgba();
    let delta_r = (valid.red - invalid.red).abs();
    let delta_g = (valid.green - invalid.green).abs();
    let delta_b = (valid.blue - invalid.blue).abs();
    assert!(
        delta_r + delta_g + delta_b > 0.6,
        "valid/invalid tints must be visually distinct; got delta=({delta_r}, {delta_g}, {delta_b})"
    );
}

#[test]
fn out_of_range_board_cell_target_spawns_invalid_marker() {
    // AC5 — Targeting a cell outside the local spawn range surfaces an
    // invalid marker. With `spawn_range_cells = 1` on the LowCells edge,
    // only `cell = 1` is valid; targeting `cell = 8` must mark the cell
    // as invalid.
    let mut app = fixture_app(1);
    stage_ghost(
        &mut app,
        CardId(12),
        Some(PlayTarget::BoardCell { lane: 3, cell: 8 }),
    );
    assert_eq!(
        count::<TargetingInvalidMarker>(&mut app),
        1,
        "out-of-range board cell must spawn exactly one invalid marker"
    );
    assert_eq!(
        count::<TargetingEndpointRing>(&mut app),
        1,
        "active target cell must always carry an endpoint ring"
    );
    // The invalid marker tint reads as a warning red — assert alpha > 0
    // so a future refactor cannot accidentally make it invisible.
    let mut q = app.world_mut().query::<(&TargetingInvalidMarker, &Sprite)>();
    let (_marker, sprite) = q.iter(app.world()).next().expect("invalid marker present");
    assert!(
        sprite.color.alpha() > 0.5,
        "invalid marker must remain visible"
    );
}

#[test]
fn board_cell_target_spawns_source_card_link() {
    // AC6 — Targeting begins from a hand card; the board feedback
    // visually links to that source via the SourceCardLink sprite. Its
    // sprite size + rotation derive from the BoardEnvelope bottom
    // anchor + target cell, so we just assert presence + non-zero
    // dimensions here.
    let mut app = fixture_app(3);
    stage_ghost(
        &mut app,
        CardId(13),
        Some(PlayTarget::BoardCell { lane: 5, cell: 2 }),
    );
    assert_eq!(
        count::<SourceCardLink>(&mut app),
        1,
        "active board-cell targeting must spawn one source-card link"
    );

    let mut q = app.world_mut().query::<(&SourceCardLink, &Sprite)>();
    let (_link, sprite) = q.iter(app.world()).next().expect("source link sprite");
    let size = sprite.custom_size.expect("source link has a custom size");
    assert!(size.x > 0.0 && size.y > 0.0, "source link size must be positive");
}

#[test]
fn targeting_overlay_never_writes_placement_board_view() {
    // AC7 — Targeting visuals are derived from existing mirrors; the
    // overlay must never decide legality or mutate the authoritative
    // local placement view.
    let mut app = fixture_app(2);
    let before = *app.world().resource::<PlacementBoardView>();
    stage_ghost(
        &mut app,
        CardId(14),
        Some(PlayTarget::BoardCell { lane: 2, cell: 7 }),
    );
    let after = *app.world().resource::<PlacementBoardView>();
    assert_eq!(
        before, after,
        "targeting overlay must never modify PlacementBoardView"
    );
}

#[test]
fn overlay_z_order_below_objectives_and_units() {
    // AC8 — Every overlay must sit below Z_TRAPS_STRUCTURES (and
    // therefore below objectives/units/hover cards), per ADR-021's
    // immutable z-order contract.
    let mut app = fixture_app(2);
    stage_ghost(
        &mut app,
        CardId(15),
        Some(PlayTarget::BoardCell { lane: 1, cell: 2 }),
    );
    let upper_bound = rendering_constants::Z_TRAPS_STRUCTURES;

    {
        let mut q = app.world_mut().query::<(&TargetingDimWash, &Transform)>();
        for (_marker, transform) in q.iter(app.world()) {
            assert!(
                transform.translation.z < upper_bound,
                "dim wash z={} must be below Z_TRAPS_STRUCTURES={upper_bound}",
                transform.translation.z
            );
        }
    }
    {
        let mut q = app.world_mut().query::<(&TargetingValidRing, &Transform)>();
        for (_marker, transform) in q.iter(app.world()) {
            assert!(
                transform.translation.z < upper_bound,
                "valid ring z={} must be below Z_TRAPS_STRUCTURES={upper_bound}",
                transform.translation.z
            );
        }
    }
    {
        let mut q = app
            .world_mut()
            .query::<(&TargetingEndpointRing, &Transform)>();
        for (_marker, transform) in q.iter(app.world()) {
            assert!(
                transform.translation.z < upper_bound,
                "endpoint ring z={} must be below Z_TRAPS_STRUCTURES={upper_bound}",
                transform.translation.z
            );
        }
    }
    // Source-card link is allowed to sit between units and ghosts per
    // the AC6 "visually links" requirement, so it has its own upper
    // bound (Z_GHOST_UNIT). Validate that separately.
    let ghost_upper_bound = rendering_constants::Z_GHOST_UNIT;
    {
        let mut q = app.world_mut().query::<(&SourceCardLink, &Transform)>();
        for (_marker, transform) in q.iter(app.world()) {
            assert!(
                transform.translation.z < ghost_upper_bound,
                "source-card link z={} must remain below the ghost layer {ghost_upper_bound}",
                transform.translation.z
            );
        }
    }
}

#[test]
fn qa_snapshot_exposes_board_envelope_and_targeting_state() {
    // AC9 — `BoardTargetingSnapshot` shape contains every field listed
    // in the story: envelope, active targeting state, valid count,
    // invalid count, path segment count, endpoint ring count, overlap
    // booleans. The pure-projection `build_snapshot_with_extras_and_layout`
    // constructor is exercised here so the JSON shape is locked in
    // without a real Bevy world drive.
    let envelope_snapshot = BoardEnvelope::from_layout(&Default::default());

    let board_targeting = BoardTargetingSnapshot {
        available: true,
        envelope: Some(client::presentation::qa_snapshot::BoardEnvelopeSnapshot {
            world_center: [
                envelope_snapshot.world_center.x,
                envelope_snapshot.world_center.y,
            ],
            world_width: envelope_snapshot.world_width(),
            world_height: envelope_snapshot.world_height(),
            cell_width: envelope_snapshot.cell_size.x,
            lane_height: envelope_snapshot.cell_size.y,
            lane_count: envelope_snapshot.lane_count,
            cell_count: envelope_snapshot.cell_count,
        }),
        active_targeting: Some(client::presentation::qa_snapshot::ActiveTargetingSnapshot {
            card_id: 16,
            target_kind: "board_cell".into(),
            endpoint_cell: Some([3, 2]),
            endpoint_invalid: false,
            valid_cell_count: 10,
        }),
        valid_target_count: 10,
        invalid_target_count: 0,
        path_segment_count: 1,
        endpoint_ring_count: 1,
        dim_wash_count: 1,
        overlaps_hand_bar: false,
        overlaps_top_chrome: false,
        overlay_idle: false,
        placement_action_panel_overlaps_any: false,
    };

    let snapshot = build_snapshot_with_extras_and_layout(
        0,
        0,
        ScreenshotInfo {
            relative_path: "screenshot.png".into(),
            absolute_path: "/tmp/screenshot.png".into(),
            format: "png".into(),
            requested_at_ms: 0,
            status: SCREENSHOT_STATUS_PENDING.into(),
            captured_at_ms: None,
            error: None,
        },
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
        ExtrasSnapshot::default(),
        LayoutSnapshot::default(),
        board_targeting.clone(),
    );

    let json = serde_json::to_value(&snapshot).expect("snapshot serialises");
    let bt = &json["board_targeting"];
    assert_eq!(bt["available"], serde_json::Value::Bool(true));
    assert!(bt["envelope"].is_object(), "envelope must be present");
    assert!(
        bt["envelope"]["world_width"].as_f64().unwrap() > 0.0,
        "world_width must be positive"
    );
    assert_eq!(bt["valid_target_count"], serde_json::Value::from(10u64));
    assert_eq!(bt["invalid_target_count"], serde_json::Value::from(0u64));
    assert_eq!(bt["path_segment_count"], serde_json::Value::from(1u64));
    assert_eq!(bt["endpoint_ring_count"], serde_json::Value::from(1u64));
    assert_eq!(bt["dim_wash_count"], serde_json::Value::from(1u64));
    assert_eq!(bt["overlaps_hand_bar"], serde_json::Value::Bool(false));
    assert_eq!(bt["overlaps_top_chrome"], serde_json::Value::Bool(false));
    assert_eq!(bt["overlay_idle"], serde_json::Value::Bool(false));
    assert_eq!(
        bt["placement_action_panel_overlaps_any"],
        serde_json::Value::Bool(false)
    );
    assert_eq!(
        bt["active_targeting"]["target_kind"],
        serde_json::Value::String("board_cell".into())
    );
}

#[test]
fn targeting_overlay_idempotent_under_repeated_updates() {
    // Driver invariant: targeting state must be reconciled idempotently.
    // Running `app.update()` repeatedly while the same target is active
    // must not produce duplicate overlay entities.
    let mut app = fixture_app(2);
    stage_ghost(
        &mut app,
        CardId(17),
        Some(PlayTarget::BoardCell { lane: 2, cell: 1 }),
    );
    let before = (
        count::<TargetingDimWash>(&mut app),
        count::<TargetingValidRing>(&mut app),
        count::<TargetingEndpointRing>(&mut app),
        count::<SourceCardLink>(&mut app),
    );

    for _ in 0..3 {
        app.update();
    }
    let after = (
        count::<TargetingDimWash>(&mut app),
        count::<TargetingValidRing>(&mut app),
        count::<TargetingEndpointRing>(&mut app),
        count::<SourceCardLink>(&mut app),
    );
    assert_eq!(before, after, "overlay counts must remain stable under repeated updates");
}

#[test]
fn targeting_overlay_state_resource_clears_on_session_exit() {
    // Lifecycle invariant: leaving the session resets the overlay state
    // so a re-entry starts from a clean slate (the
    // `remove_board_rendering_session_resources` system writes
    // `TargetingOverlayState::default()` back).
    let mut app = fixture_app(2);
    stage_ghost(
        &mut app,
        CardId(18),
        Some(PlayTarget::BoardCell { lane: 1, cell: 1 }),
    );
    assert!(app.world().resource::<TargetingOverlayState>().is_active());

    let mut next_state = app
        .world_mut()
        .resource_mut::<NextState<client::state::ClientState>>();
    *next_state = NextState::Pending(client::state::ClientState::Lobby);
    app.update();
    app.update();

    assert!(
        !app.world().resource::<TargetingOverlayState>().is_active(),
        "leaving InSession must clear the targeting overlay state"
    );
}
