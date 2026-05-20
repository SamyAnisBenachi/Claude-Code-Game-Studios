//! PROMPT 1521 — Resolution replay per-group cadence tests.
//!
//! Verifies that visible board feedback (damage numbers, kill markers)
//! emitted from `S2CResolutionEvent` follows the same
//! `(sub_step, trigger_index)` cadence the server already encodes, rather
//! than burst-firing at script intake.
//!
//! These tests complement the existing intake-frame coverage in
//! `resolution_combat_feedback_test.rs` by exercising multi-sub-step
//! scripts: events in later sub-steps must not fire until their
//! AnimGroup becomes the active playback group.

use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{
    AnimQueue, AnimationTimingConfig, CardAnimationsPlugin, DamageNumberSpawnRequested,
};
use client::presentation::board_rendering::{
    BoardRenderState, BoardRenderingPlugin, BoardUnit, BoardUnitCard, BoardUnitOwner,
    PendingResolutionScript, ResolutionKillMarker, ResolutionReplayProgress, ResolutionRevealWait,
};
use client::presentation::LaneCell;
use client::state::ClientState;
use shared::card::CardId;
use shared::protocol::{EntityId, ResolutionEvent, S2CResolutionEvent, TaggedEvent};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const DEFENDER_ID_1: EntityId = 7001;
const DEFENDER_ID_2: EntityId = 7002;
const ATTACKER_ID: EntityId = 7100;
const DAMAGE_GROUP_1: u8 = 3;
const DAMAGE_GROUP_3: u8 = 5;

#[test]
fn test_replay_emits_damage_for_first_group_only_until_time_advances() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let _d1 = spawn_unit(&mut app, DEFENDER_ID_1, 2, 3, 100.0, 0.0);
    let _d2 = spawn_unit(&mut app, DEFENDER_ID_2, 2, 4, 140.0, 0.0);
    let mut cursor = drained_cursor::<DamageNumberSpawnRequested>(&app);

    let script = S2CResolutionEvent {
        round: 9,
        events: vec![
            tagged(
                1,
                0,
                ResolutionEvent::CombatDamage {
                    attacker_id: ATTACKER_ID,
                    defender_id: DEFENDER_ID_1,
                    damage_amount: DAMAGE_GROUP_1,
                    defender_hp_after: 2,
                    was_blocked_by_shield: false,
                },
            ),
            tagged(
                3,
                0,
                ResolutionEvent::CombatDamage {
                    attacker_id: ATTACKER_ID,
                    defender_id: DEFENDER_ID_2,
                    damage_amount: DAMAGE_GROUP_3,
                    defender_hp_after: 1,
                    was_blocked_by_shield: false,
                },
            ),
        ],
    };
    enter_resolution_with_script(&mut app, script);
    app.update();

    // First frame: only the sub_step==1 group has started; sub_step==3
    // must still be buffered.
    let frame_1 = messages_since(&app, &mut cursor);
    assert_eq!(
        frame_1.len(),
        1,
        "only the first group's CombatDamage should fire on intake"
    );
    assert_eq!(frame_1[0].damage_value, u32::from(DAMAGE_GROUP_1));
    assert_eq!(
        app.world()
            .resource::<ResolutionReplayProgress>()
            .last_emitted_group_index(),
        Some(0),
        "progress should record the first emitted group"
    );

    // Tick partway through group 1's duration: still no group 3 emission.
    run_for(&mut app, Duration::from_millis(599));
    assert!(
        messages_since(&app, &mut cursor).is_empty(),
        "group 3 must not fire mid-playback of group 1"
    );

    // Cross the group boundary (group 1 duration + inter-step pause
    // + at least one frame to land in the next group).
    run_for(&mut app, Duration::from_millis(1));
    run_for(&mut app, Duration::from_millis(150));
    run_for(&mut app, Duration::from_millis(1));

    let frame_after = messages_since(&app, &mut cursor);
    assert_eq!(
        frame_after.len(),
        1,
        "second group's CombatDamage should fire once its group starts"
    );
    assert_eq!(frame_after[0].damage_value, u32::from(DAMAGE_GROUP_3));
}

#[test]
fn test_replay_does_not_double_emit_when_repeated_frames_share_a_group() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let _victim = spawn_unit(&mut app, DEFENDER_ID_1, 1, 2, 0.0, 0.0);

    let script = S2CResolutionEvent {
        round: 9,
        events: vec![tagged(
            2,
            0,
            ResolutionEvent::UnitDied {
                unit_id: DEFENDER_ID_1,
                lane: 1,
                cell: 2,
                killer_id: Some(ATTACKER_ID),
            },
        )],
    };
    enter_resolution_with_script(&mut app, script);

    // Intake frame starts the single group and emits one kill marker.
    app.update();
    assert_eq!(count_kill_markers(&mut app), 1);

    // A second update before the group timer elapses must not re-emit.
    app.update();
    assert_eq!(
        count_kill_markers(&mut app),
        1,
        "replay applier must dedupe within a single active group"
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
    // Use the default 600ms sub-step duration so tests can rely on a
    // stable group boundary.
    app.insert_resource(AnimationTimingConfig::default());
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    app.update();
    // Drop any noise from the initial setup frame.
    let _ = app.world().resource::<AnimQueue>();
    app
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
}

fn spawn_unit(
    app: &mut App,
    unit_id: EntityId,
    lane: u8,
    cell: u8,
    x: f32,
    y: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            BoardUnit { unit_id },
            BoardUnitOwner(PlayerId(2)),
            BoardUnitCard {
                card_id: Some(CardId(11)),
                frame_index: 0,
                used_missing_art_fallback: false,
            },
            LaneCell { lane, cell },
            Transform::from_xyz(x, y, 0.0),
        ))
        .id()
}

fn tagged(sub_step: u8, trigger_index: u32, event: ResolutionEvent) -> TaggedEvent {
    TaggedEvent {
        sub_step,
        trigger_index,
        event,
    }
}

fn count_kill_markers(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<(), With<ResolutionKillMarker>>();
    query.iter(world).count()
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
