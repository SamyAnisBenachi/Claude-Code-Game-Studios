use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{
    AnimGroup, AnimQueue, AnimQueueEvent, AnimationTimingConfig, CardAnimationsPlugin,
    PendingPhaseChange,
};
use client::presentation::apply_phase_changed_messages_with_resolution_gate;
use client::presentation::board_rendering::{
    resolution_anim_groups_from_script, BoardRenderState, BoardRenderingPlugin,
    PendingResolutionScript, ResolutionQueueBuildError, ResolutionRevealWait,
    SnapshotRecoveryReason, SnapshotRecoveryRequested,
};
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use shared::protocol::{
    ResolutionEvent, RoundPhase, S2CPhaseChanged, S2CResolutionEvent, TaggedEvent,
};

#[test]
fn test_resolution_event_groups_by_sub_step_and_sorts_ascending() {
    let timings = AnimationTimingConfig {
        resolution_sub_step_duration_ms: 123,
        ..default()
    };
    let groups = resolution_anim_groups_from_script(
        &script(vec![tagged(3, 0), tagged(1, 2), tagged(1, 1), tagged(5, 0)]),
        timings,
    )
    .expect("valid sub-steps should build groups");

    assert_eq!(
        groups
            .iter()
            .map(|group| group.sub_step)
            .collect::<Vec<_>>(),
        vec![1, 3, 5]
    );
    assert_eq!(
        groups
            .iter()
            .map(|group| group.duration_ms)
            .collect::<Vec<_>>(),
        vec![123, 123, 123]
    );
    assert_eq!(replay_trigger_indexes(&groups[0]), vec![1, 2]);
    assert_eq!(groups[1].events.len(), 1);
    assert_eq!(groups[2].events.len(), 1);
}

#[test]
fn test_out_of_range_resolution_sub_step_rejects_queue_and_requests_one_snapshot() {
    let mut app = app_in_session();
    let mut cursor = drained_cursor::<SnapshotRecoveryRequested>(&app);

    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::ResolutionReveal;
    app.world_mut()
        .resource_mut::<PendingResolutionScript>()
        .set(script(vec![tagged(9, 0), tagged(10, 1)]));
    *app.world_mut().resource_mut::<AnimQueue>() =
        AnimQueue::from_groups(vec![AnimGroup::new(1, 600, Vec::new())]);

    app.update();

    let messages = messages_since(&app, &mut cursor);
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].reason,
        SnapshotRecoveryReason::ResolutionSubStepOutOfRange
    );
    assert!(app.world().resource::<AnimQueue>().groups.is_empty());
    assert!(app
        .world()
        .resource::<PendingResolutionScript>()
        .script()
        .is_none());

    app.update();
    assert!(
        messages_since(&app, &mut cursor).is_empty(),
        "one rejected script should enqueue exactly one recovery request"
    );
}

#[test]
fn test_phase_change_buffers_while_resolution_executing_and_applies_after_queue_drain() {
    let mut app = app_in_session();

    *app.world_mut().resource_mut::<CurrentClientPhase>() = CurrentClientPhase {
        phase: RoundPhase::Resolution,
        round: 8,
    };
    *app.world_mut().resource_mut::<ClientPhaseView>() = ClientPhaseView {
        phase: RoundPhase::Resolution,
        round_number: 8,
        timer_duration_ms: 60_000,
    };
    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::ResolutionExecuting;
    *app.world_mut().resource_mut::<AnimQueue>() =
        AnimQueue::from_groups(vec![AnimGroup::new(1, 600, Vec::new())]);
    app.world_mut()
        .resource_mut::<PendingPhaseChange>()
        .set(phase_changed(RoundPhase::DraftShop, 8));

    run_for(&mut app, Duration::from_millis(599));
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::Resolution
    );
    assert_eq!(
        app.world().resource::<PendingPhaseChange>().phase(),
        Some(RoundPhase::DraftShop)
    );
    assert!(!app.world().resource::<AnimQueue>().groups.is_empty());

    run_for(&mut app, Duration::from_millis(1));
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::DraftShop
    );
    assert_eq!(
        app.world().resource::<ClientPhaseView>().phase,
        RoundPhase::DraftShop
    );
    assert!(app.world().resource::<PendingPhaseChange>().is_none());
    assert!(app.world().resource::<AnimQueue>().groups.is_empty());
    assert_eq!(
        *app.world().resource::<BoardRenderState>(),
        BoardRenderState::DraftShop
    );
}

#[test]
fn test_same_frame_resolution_script_buffers_draft_phase_before_playback() {
    let mut current = CurrentClientPhase {
        phase: RoundPhase::Resolution,
        round: 8,
    };
    let mut phase_view = ClientPhaseView {
        phase: RoundPhase::Resolution,
        round_number: 8,
        timer_duration_ms: 60_000,
    };
    let render_state = BoardRenderState::ResolutionReveal;
    let mut pending_script = PendingResolutionScript::default();
    let mut reveal_wait = ResolutionRevealWait::default();
    let mut pending_phase = PendingPhaseChange::default();

    pending_script.set(script(vec![tagged(1, 0)]));
    reveal_wait.start();

    apply_phase_changed_messages_with_resolution_gate(
        [phase_changed(RoundPhase::DraftShop, 8)],
        &mut current,
        &mut phase_view,
        Some(&render_state),
        Some(&pending_script),
        Some(&reveal_wait),
        Some(&mut pending_phase),
    );

    assert_eq!(current.phase, RoundPhase::Resolution);
    assert_eq!(phase_view.phase, RoundPhase::Resolution);
    assert_eq!(pending_phase.phase(), Some(RoundPhase::DraftShop));
}

#[test]
fn test_out_of_range_group_builder_reports_offending_sub_step() {
    let err = resolution_anim_groups_from_script(&script(vec![tagged(7, 42)]), default())
        .expect_err("sub-step 7 should be rejected");

    assert_eq!(
        err,
        ResolutionQueueBuildError::OutOfRangeSubStep {
            sub_step: 7,
            trigger_index: 42
        }
    );
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(CardAnimationsPlugin);
    app.add_plugins(BoardRenderingPlugin);
    app.init_resource::<ClientPhaseView>();
    // BoardRenderingPlugin no longer initialises ClientState (f5b7a34 removed
    // duplicate init_state calls). Standalone tests must initialise it here.
    app.init_state::<ClientState>();
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

fn script(events: Vec<TaggedEvent>) -> S2CResolutionEvent {
    S2CResolutionEvent { round: 8, events }
}

fn tagged(sub_step: u8, trigger_index: u32) -> TaggedEvent {
    TaggedEvent {
        sub_step,
        trigger_index,
        event: ResolutionEvent::SubStepBegin,
    }
}

fn phase_changed(phase: RoundPhase, round_number: u32) -> S2CPhaseChanged {
    S2CPhaseChanged {
        phase,
        round_number,
        timer_duration_ms: 30_000,
    }
}

fn replay_trigger_indexes(group: &AnimGroup) -> Vec<u32> {
    group
        .events
        .iter()
        .map(|event| match event {
            AnimQueueEvent::ResolutionReplay { event } => event.trigger_index,
            AnimQueueEvent::TransformTween { .. } => {
                panic!("resolution script should build replay events")
            }
        })
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
