//! PROMPT 1231 — minimal Resolution combat-feedback overlay tests.
//!
//! Verifies that the disjoint resolution-feedback lane (`PROMPT 1201` /
//! `PROMPT 1203`) wires `S2CResolutionEvent::CombatDamage` to the existing
//! `DamageNumberSpawnRequested` lane and that `UnitDied` / `UnitRemoved`
//! events spawn transient `ResolutionKillMarker` entities scoped to
//! the Resolution phase.

use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{
    CardAnimationsPlugin, DamageNumberSpawnRequested, PlacementRevealAnimReady,
};
use client::presentation::board_rendering::{
    BoardRenderState, BoardRenderingPlugin, BoardUnit, BoardUnitCard, BoardUnitOwner,
    PendingResolutionScript, ResolutionKillMarker, ResolutionRevealWait,
    RESOLUTION_KILL_MARKER_TTL_MS,
};
use client::presentation::LaneCell;
use client::state::ClientState;
use shared::card::CardId;
use shared::protocol::{EntityId, ResolutionEvent, S2CResolutionEvent, TaggedEvent};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const DEFENDER_UNIT_ID: EntityId = 4242;
const ATTACKER_UNIT_ID: EntityId = 4141;
const DAMAGE_AMOUNT: u8 = 7;
const TRIGGER_INDEX: u32 = 5;
const UNIT_X: f32 = 120.0;
const UNIT_Y: f32 = -40.0;

#[test]
fn test_combat_damage_event_emits_damage_number_spawn_request() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let defender = spawn_board_unit_with_transform(
        &mut app,
        DEFENDER_UNIT_ID,
        player(2),
        CardId(11),
        3,
        4,
        UNIT_X,
        UNIT_Y,
    );
    // Drop the placement-reveal noise the plugin emits at session startup.
    let mut placement_cursor = drained_cursor::<PlacementRevealAnimReady>(&app);
    let mut damage_cursor = drained_cursor::<DamageNumberSpawnRequested>(&app);

    enter_resolution_with_script(
        &mut app,
        script(vec![tagged(
            1,
            TRIGGER_INDEX,
            ResolutionEvent::CombatDamage {
                attacker_id: ATTACKER_UNIT_ID,
                defender_id: DEFENDER_UNIT_ID,
                damage_amount: DAMAGE_AMOUNT,
                defender_hp_after: 1,
                was_blocked_by_shield: false,
            },
        )]),
    );
    app.update();

    let damage_msgs = messages_since(&app, &mut damage_cursor);
    assert_eq!(damage_msgs.len(), 1, "expected exactly one damage request");
    assert_eq!(damage_msgs[0].target, defender);
    assert_eq!(damage_msgs[0].damage_value, u32::from(DAMAGE_AMOUNT));
    assert_eq!(damage_msgs[0].event_id, TRIGGER_INDEX);

    // No kill markers for damage-only events.
    assert_eq!(count_kill_markers(&mut app), 0);
    // Placement-reveal lane must not be perturbed.
    assert!(messages_since(&app, &mut placement_cursor).is_empty());
}

#[test]
fn test_combat_damage_with_zero_damage_is_skipped() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let _defender = spawn_board_unit_with_transform(
        &mut app,
        DEFENDER_UNIT_ID,
        player(2),
        CardId(11),
        2,
        3,
        UNIT_X,
        UNIT_Y,
    );
    let mut damage_cursor = drained_cursor::<DamageNumberSpawnRequested>(&app);

    enter_resolution_with_script(
        &mut app,
        script(vec![tagged(
            1,
            TRIGGER_INDEX,
            ResolutionEvent::CombatDamage {
                attacker_id: ATTACKER_UNIT_ID,
                defender_id: DEFENDER_UNIT_ID,
                damage_amount: 0,
                defender_hp_after: 5,
                was_blocked_by_shield: true,
            },
        )]),
    );
    app.update();

    assert!(
        messages_since(&app, &mut damage_cursor).is_empty(),
        "0-damage CombatDamage should not produce a damage number"
    );
}

#[test]
fn test_combat_damage_for_unknown_defender_is_silently_skipped() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let mut damage_cursor = drained_cursor::<DamageNumberSpawnRequested>(&app);

    enter_resolution_with_script(
        &mut app,
        script(vec![tagged(
            1,
            TRIGGER_INDEX,
            ResolutionEvent::CombatDamage {
                attacker_id: ATTACKER_UNIT_ID,
                defender_id: 9999,
                damage_amount: DAMAGE_AMOUNT,
                defender_hp_after: 1,
                was_blocked_by_shield: false,
            },
        )]),
    );
    app.update();

    assert!(
        messages_since(&app, &mut damage_cursor).is_empty(),
        "unknown defender entity_id should not produce a damage request"
    );
    assert_eq!(count_kill_markers(&mut app), 0);
}

#[test]
fn test_unit_died_event_spawns_kill_marker_at_unit_position() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let _victim = spawn_board_unit_with_transform(
        &mut app,
        DEFENDER_UNIT_ID,
        player(2),
        CardId(11),
        3,
        4,
        UNIT_X,
        UNIT_Y,
    );

    enter_resolution_with_script(
        &mut app,
        script(vec![tagged(
            2,
            TRIGGER_INDEX,
            ResolutionEvent::UnitDied {
                unit_id: DEFENDER_UNIT_ID,
                lane: 3,
                cell: 4,
                killer_id: Some(ATTACKER_UNIT_ID),
            },
        )]),
    );
    app.update();

    let markers = kill_marker_positions(&mut app);
    assert_eq!(markers.len(), 1, "expected one kill marker for UnitDied");
    let (x, y) = markers[0];
    // Jitter is event_id-deterministic; the marker should be within a small
    // jitter envelope of the unit's center, not at the origin.
    assert!(
        (x - UNIT_X).abs() < 40.0 && (y - UNIT_Y).abs() < 40.0,
        "kill marker at ({x},{y}) too far from unit center ({UNIT_X},{UNIT_Y})"
    );
}

#[test]
fn test_unit_removed_event_spawns_kill_marker() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let _victim = spawn_board_unit_with_transform(
        &mut app,
        DEFENDER_UNIT_ID,
        player(2),
        CardId(11),
        2,
        3,
        UNIT_X,
        UNIT_Y,
    );

    enter_resolution_with_script(
        &mut app,
        script(vec![tagged(
            3,
            TRIGGER_INDEX,
            ResolutionEvent::UnitRemoved {
                unit_id: DEFENDER_UNIT_ID,
                lane: 2,
                cell: 3,
            },
        )]),
    );
    app.update();

    assert_eq!(
        count_kill_markers(&mut app),
        1,
        "expected one kill marker for UnitRemoved"
    );
}

#[test]
fn test_kill_marker_despawns_after_ttl() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let _victim = spawn_board_unit_with_transform(
        &mut app,
        DEFENDER_UNIT_ID,
        player(2),
        CardId(11),
        2,
        3,
        UNIT_X,
        UNIT_Y,
    );
    enter_resolution_with_script(
        &mut app,
        script(vec![tagged(
            2,
            TRIGGER_INDEX,
            ResolutionEvent::UnitDied {
                unit_id: DEFENDER_UNIT_ID,
                lane: 2,
                cell: 3,
                killer_id: None,
            },
        )]),
    );
    // Consume the script with delta=0 so the marker starts at a known
    // timer position (avoids accumulating wall-clock delta into the first
    // tick after spawn).
    run_for(&mut app, Duration::from_millis(0));
    assert_eq!(count_kill_markers(&mut app), 1);

    // Tick well shy of TTL: the marker is still up.
    run_for(
        &mut app,
        Duration::from_millis(RESOLUTION_KILL_MARKER_TTL_MS / 2),
    );
    assert_eq!(count_kill_markers(&mut app), 1);

    // Tick past TTL: feedback must not bleed into a later phase.
    run_for(
        &mut app,
        Duration::from_millis(RESOLUTION_KILL_MARKER_TTL_MS),
    );
    assert_eq!(
        count_kill_markers(&mut app),
        0,
        "kill marker should despawn once TTL elapses"
    );
}

#[test]
fn test_unrelated_resolution_events_emit_no_combat_feedback() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let mut damage_cursor = drained_cursor::<DamageNumberSpawnRequested>(&app);

    enter_resolution_with_script(
        &mut app,
        script(vec![
            tagged(1, 0, ResolutionEvent::SubStepBegin),
            tagged(
                1,
                1,
                ResolutionEvent::SpawnRangeChanged {
                    player_id: player(2),
                    new_spawn_range_cells: 4,
                },
            ),
        ]),
    );
    app.update();

    assert!(
        messages_since(&app, &mut damage_cursor).is_empty(),
        "non-combat resolution events must not emit damage requests"
    );
    assert_eq!(
        count_kill_markers(&mut app),
        0,
        "non-combat resolution events must not spawn kill markers"
    );
}

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
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    app.update();
    app
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
}

fn spawn_board_unit_with_transform(
    app: &mut App,
    unit_id: EntityId,
    owner_id: PlayerId,
    card_id: CardId,
    lane: u8,
    cell: u8,
    x: f32,
    y: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            BoardUnit { unit_id },
            BoardUnitOwner(owner_id),
            BoardUnitCard {
                card_id: Some(card_id),
                frame_index: 0,
                used_missing_art_fallback: false,
            },
            LaneCell { lane, cell },
            Transform::from_xyz(x, y, 0.0),
        ))
        .id()
}

fn script(events: Vec<TaggedEvent>) -> S2CResolutionEvent {
    S2CResolutionEvent { round: 7, events }
}

fn tagged(sub_step: u8, trigger_index: u32, event: ResolutionEvent) -> TaggedEvent {
    TaggedEvent {
        sub_step,
        trigger_index,
        event,
    }
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn count_kill_markers(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<(), With<ResolutionKillMarker>>();
    query.iter(world).count()
}

fn kill_marker_positions(app: &mut App) -> Vec<(f32, f32)> {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Transform, With<ResolutionKillMarker>>();
    query
        .iter(world)
        .map(|transform| (transform.translation.x, transform.translation.y))
        .collect()
}

fn drained_cursor<M: Message + Clone>(app: &App) -> MessageCursor<M> {
    let messages = app.world().resource::<Messages<M>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn messages_since<M: Message + Clone>(app: &App, cursor: &mut MessageCursor<M>) -> Vec<M> {
    let messages = app.world().resource::<Messages<M>>();
    cursor.read(messages).cloned().collect()
}
