use std::time::Duration;

use bevy::prelude::*;
use server::core::rsm::{
    BeginResolution, BroadcastPhaseChanged, GameOverEmitted, ResolutionComplete, RoundPhase,
    RoundState, RsmPlugin,
};
use server::feature::combat::{
    CombatIterationBudget, CombatNetworkMessageKind, CombatNetworkOutbox, CombatPlugin,
    CombatResolutionTrace, CombatTraceEntry, PendingResolutionComplete,
};
use shared::protocol::GameOverReason;

fn app_with_combat() -> App {
    let mut app = App::new();
    app.add_plugins(CombatPlugin);
    app
}

fn app_with_rsm_and_combat() -> App {
    let mut app = App::new();
    app.add_plugins((RsmPlugin, CombatPlugin));
    app.insert_resource(Time::<()>::default());
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase: RoundPhase::Resolution,
        round_number: 4,
        ..RoundState::new()
    };
    app
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn send_begin_resolution(app: &mut App, round: u32) {
    app.world_mut().write_message(BeginResolution { round });
}

#[test]
fn resolve_combat_idle_without_begin_resolution_touches_no_story_state() {
    let mut app = app_with_combat();
    app.update();

    assert!(app
        .world()
        .resource::<CombatNetworkOutbox>()
        .messages()
        .is_empty());
    assert!(app
        .world()
        .resource::<CombatResolutionTrace>()
        .entries()
        .is_empty());
    assert!(!app
        .world()
        .resource::<PendingResolutionComplete>()
        .is_pending());
    assert!(read_messages::<ResolutionComplete>(&app).is_empty());
}

#[test]
fn resolve_combat_runs_substeps_and_completion_after_resolution_event() {
    let mut app = app_with_combat();
    send_begin_resolution(&mut app, 7);

    app.update();

    let outbox = app.world().resource::<CombatNetworkOutbox>();
    assert_eq!(
        outbox.message_kinds(),
        vec![CombatNetworkMessageKind::ResolutionEvent]
    );

    let trace = app.world().resource::<CombatResolutionTrace>().entries();
    let begin_index = trace
        .iter()
        .position(|entry| *entry == CombatTraceEntry::BeginResolutionRead { round: 7 })
        .expect("begin resolution should be traced");
    let sub_step_one_index = trace
        .iter()
        .position(|entry| *entry == CombatTraceEntry::SubStepStarted(1))
        .expect("sub-step 1 should be traced");
    let resolution_event_index = trace
        .iter()
        .position(|entry| *entry == CombatTraceEntry::ResolutionEventEnqueued)
        .expect("resolution event should be traced");
    let completion_index = trace
        .iter()
        .position(|entry| *entry == CombatTraceEntry::ResolutionCompleteQueued)
        .expect("completion should be traced");

    assert!(begin_index < sub_step_one_index);
    assert!(resolution_event_index < completion_index);
    assert_eq!(read_messages::<ResolutionComplete>(&app).len(), 1);
}

#[test]
fn resolve_combat_iteration_budget_overflow_requests_draw_without_completion() {
    let mut app = app_with_rsm_and_combat();
    app.insert_resource(CombatIterationBudget::with_limit(0));
    send_begin_resolution(&mut app, 4);

    app.update();

    assert!(read_messages::<ResolutionComplete>(&app).is_empty());
    assert!(app
        .world()
        .resource::<CombatResolutionTrace>()
        .entries()
        .contains(&CombatTraceEntry::IterationBudgetExceeded));

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);
    app.update();

    let game_over = read_messages::<GameOverEmitted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    let rsm = app.world().resource::<RoundState>();

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Draw);
    assert_eq!(game_over[0].loser, None);
    assert_eq!(
        broadcasts.last().map(|msg| msg.phase),
        Some(RoundPhase::GameOver)
    );
}
