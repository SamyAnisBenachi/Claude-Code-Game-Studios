//! PROMPT 1532 — Resolution replay visual mutation follow-up.
//!
//! Verifies that the client replay applier mutates board state for the
//! visible-mutation `ResolutionEvent` variants the protocol already
//! exposes:
//!
//! - `UnitMoved`        : LaneCell + Transform snap to the new cell.
//! - `UnitChangedLane`  : LaneCell.lane + Transform.y snap, cell preserved.
//! - `UnitPlaced`       : deduped against existing BoardUnit (placement
//!                        reveal already spawned the unit); no re-spawn.
//! - `ObjectiveDamage`  : `StandingObjectiveHp.hp_current` follows the
//!                        server-reported `objective_hp_after`.
//! - `ObjectiveDestroyed`: HP zeroed; standing-objective entity despawned.
//!
//! Cadence is already covered by
//! `resolution_replay_per_group_cadence_test.rs`; this file focuses on
//! the per-event visual mutation contract.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{AnimationTimingConfig, CardAnimationsPlugin};
use client::presentation::board_rendering::{
    BoardRenderState, BoardRenderingPlugin, BoardUnit, BoardUnitCard, BoardUnitOwner,
    PendingResolutionScript, ResolutionRevealWait, StandingObjective, StandingObjectiveArt,
    StandingObjectiveHp, ObjectiveArtKind,
};
use client::presentation::LaneCell;
use client::state::ClientState;
use client::ui::shared::BoardLayout;
use shared::card::CardId;
use shared::protocol::{EntityId, ResolutionEvent, S2CResolutionEvent, TaggedEvent};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const UNIT_ID_A: EntityId = 9001;
const OWNER: PlayerId = PlayerId(2);

#[test]
fn test_unit_moved_event_snaps_board_unit_to_new_cell() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let unit = spawn_unit(&mut app, UNIT_ID_A, 2, 3);
    let layout = *app.world().resource::<BoardLayout>();
    let expected_xy = layout.cell_to_world(2, 5);

    let script = S2CResolutionEvent {
        round: 1,
        events: vec![tagged(
            1,
            0,
            ResolutionEvent::UnitMoved {
                unit_id: UNIT_ID_A,
                lane: 2,
                from_cell: 3,
                to_cell: 5,
            },
        )],
    };

    // Act
    enter_resolution_with_script(&mut app, script);
    app.update();

    // Assert
    let world = app.world();
    let lane_cell = world.get::<LaneCell>(unit).unwrap();
    let transform = world.get::<Transform>(unit).unwrap();
    assert_eq!(
        (lane_cell.lane, lane_cell.cell),
        (2, 5),
        "UnitMoved must update LaneCell to destination"
    );
    assert!(
        (transform.translation.x - expected_xy.x).abs() < 0.001,
        "UnitMoved must snap Transform.x to cell_to_world"
    );
    assert!(
        (transform.translation.y - expected_xy.y).abs() < 0.001,
        "UnitMoved must snap Transform.y to cell_to_world"
    );
}

#[test]
fn test_unit_changed_lane_event_updates_lane_preserves_cell() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let unit = spawn_unit(&mut app, UNIT_ID_A, 2, 4);
    let layout = *app.world().resource::<BoardLayout>();
    let expected_xy = layout.cell_to_world(4, 4);

    let script = S2CResolutionEvent {
        round: 1,
        events: vec![tagged(
            1,
            0,
            ResolutionEvent::UnitChangedLane {
                unit_id: UNIT_ID_A,
                from_lane: 2,
                to_lane: 4,
            },
        )],
    };

    // Act
    enter_resolution_with_script(&mut app, script);
    app.update();

    // Assert
    let world = app.world();
    let lane_cell = world.get::<LaneCell>(unit).unwrap();
    let transform = world.get::<Transform>(unit).unwrap();
    assert_eq!(
        (lane_cell.lane, lane_cell.cell),
        (4, 4),
        "UnitChangedLane must update lane and preserve cell"
    );
    assert!(
        (transform.translation.x - expected_xy.x).abs() < 0.001
            && (transform.translation.y - expected_xy.y).abs() < 0.001,
        "UnitChangedLane must snap Transform to (new_lane, preserved_cell)"
    );
}

#[test]
fn test_unit_placed_event_dedupes_against_existing_board_unit() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    // Simulate the PlacementReveal having already spawned the unit at (3,2).
    let _existing = spawn_unit(&mut app, UNIT_ID_A, 3, 2);
    let unit_count_before = count_board_units(&mut app);

    let script = S2CResolutionEvent {
        round: 1,
        events: vec![tagged(
            1,
            0,
            ResolutionEvent::UnitPlaced {
                unit_id: UNIT_ID_A,
                player: OWNER,
                lane: 3,
                cell: 2,
            },
        )],
    };

    // Act
    enter_resolution_with_script(&mut app, script);
    app.update();

    // Assert
    let unit_count_after = count_board_units(&mut app);
    assert_eq!(
        unit_count_after, unit_count_before,
        "UnitPlaced must not double-spawn when a BoardUnit already exists"
    );
}

#[test]
fn test_objective_damage_event_updates_standing_objective_hp() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let objective = spawn_objective(&mut app, OWNER, 3, 10);

    let script = S2CResolutionEvent {
        round: 1,
        events: vec![tagged(
            1,
            0,
            ResolutionEvent::ObjectiveDamage {
                attacker_id: None,
                target_player_id: OWNER,
                lane: 3,
                damage_amount: 4,
                objective_hp_after: 6,
            },
        )],
    };

    // Act
    enter_resolution_with_script(&mut app, script);
    app.update();

    // Assert
    let world = app.world();
    let hp = world.get::<StandingObjectiveHp>(objective).unwrap();
    assert_eq!(
        hp.hp_current, 6,
        "ObjectiveDamage must drive StandingObjectiveHp.hp_current to objective_hp_after"
    );
    assert_eq!(hp.hp_max, 10, "ObjectiveDamage must not mutate hp_max");
}

#[test]
fn test_objective_destroyed_event_despawns_standing_objective() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let objective = spawn_objective(&mut app, OWNER, 2, 8);

    let script = S2CResolutionEvent {
        round: 1,
        events: vec![tagged(
            1,
            0,
            ResolutionEvent::ObjectiveDestroyed {
                target_player_id: OWNER,
                lane: 2,
                was_fake: false,
            },
        )],
    };

    // Act
    enter_resolution_with_script(&mut app, script);
    app.update();

    // Assert
    assert!(
        app.world().get_entity(objective).is_err(),
        "ObjectiveDestroyed must despawn the matching StandingObjective entity"
    );
}

// ---------- helpers ----------

fn enter_resolution_with_script(app: &mut App, script: S2CResolutionEvent) {
    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::ResolutionReveal;
    app.world_mut()
        .resource_mut::<ResolutionRevealWait>()
        .start();
    app.world_mut()
        .resource_mut::<PendingResolutionScript>()
        .set(script);
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(CardAnimationsPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);
    app.insert_resource(AnimationTimingConfig::default());
    // BoardRenderingPlugin only inserts BoardLayout when entering a
    // session-rendered state via snapshot; tests inject it directly so the
    // mutation helpers have layout data to project cells with.
    app.insert_resource(BoardLayout::default());
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_millis(0));
    app.update();
    app
}

fn spawn_unit(app: &mut App, unit_id: EntityId, lane: u8, cell: u8) -> Entity {
    let layout = *app.world().resource::<BoardLayout>();
    let xy = layout.cell_to_world(lane, cell);
    app.world_mut()
        .spawn((
            BoardUnit { unit_id },
            BoardUnitOwner(OWNER),
            BoardUnitCard {
                card_id: Some(CardId(11)),
                frame_index: 0,
                used_missing_art_fallback: false,
            },
            LaneCell { lane, cell },
            Transform::from_xyz(xy.x, xy.y, 0.0),
        ))
        .id()
}

fn spawn_objective(app: &mut App, owner: PlayerId, lane: u8, hp_max: u8) -> Entity {
    app.world_mut()
        .spawn((
            StandingObjective { owner_id: owner, lane },
            StandingObjectiveArt {
                kind: ObjectiveArtKind::Real,
                used_runtime_asset: false,
            },
            StandingObjectiveHp {
                hp_current: hp_max,
                hp_max,
            },
            LaneCell { lane, cell: 1 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id()
}

fn count_board_units(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&BoardUnit>();
    query.iter(world).count()
}

fn tagged(sub_step: u8, trigger_index: u32, event: ResolutionEvent) -> TaggedEvent {
    TaggedEvent {
        sub_step,
        trigger_index,
        event,
    }
}
