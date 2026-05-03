use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitOwner, UnitStats};
use server::core::rsm::{
    BeginResolution, BroadcastPhaseChanged, GameOverEmitted, ResolutionComplete, RoundPhase,
    RoundState, RsmPlugin,
};
use server::core::session::SessionConfig;
use server::feature::board::{BoardPlugin, UnitAtObjective};
use server::feature::combat::{
    CombatIterationBudget, CombatNetworkMessageKind, CombatNetworkOutbox, CombatPlugin,
    CombatResolutionTrace, CombatTraceEntry, PendingResolutionComplete,
};
use shared::card::ClassId;
use shared::protocol::{GameMode, GameOverReason};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config() -> SessionConfig {
    let player_a = player(1);
    let player_b = player(2);

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player_a, 0), (player_b, 1)]),
        class_map: HashMap::from([(player_a, ClassId::Iop), (player_b, ClassId::Cra)]),
    }
}

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

fn app_with_board_and_combat() -> App {
    let mut app = App::new();
    app.add_plugins((BoardPlugin, CombatPlugin));
    app.insert_resource(session_config());
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

fn spawn_unit(app: &mut App, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    app.world_mut()
        .spawn((
            BoardPosition { lane, cell },
            UnitOwner(owner),
            UnitStats::new(1, 1, 0, 0),
        ))
        .id()
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
fn resolve_combat_ss6_emits_unit_at_objective_messages() {
    let mut app = app_with_board_and_combat();
    let player_a_unit = spawn_unit(&mut app, player(1), 1, 8);
    let player_b_unit = spawn_unit(&mut app, player(2), 3, 1);
    spawn_unit(&mut app, player(1), 2, 7);

    app.update();
    assert!(read_messages::<UnitAtObjective>(&app).is_empty());

    send_begin_resolution(&mut app, 8);
    app.update();

    let hits = read_messages::<UnitAtObjective>(&app);
    assert_eq!(hits.len(), 2);
    assert!(hits.contains(&UnitAtObjective {
        unit_id: player_a_unit,
        lane: 1,
    }));
    assert!(hits.contains(&UnitAtObjective {
        unit_id: player_b_unit,
        lane: 3,
    }));

    let trace = app.world().resource::<CombatResolutionTrace>().entries();
    let sub_step_six_index = trace
        .iter()
        .position(|entry| *entry == CombatTraceEntry::SubStepStarted(6))
        .expect("sub-step 6 should be traced");
    let completion_index = trace
        .iter()
        .position(|entry| *entry == CombatTraceEntry::ResolutionCompleteQueued)
        .expect("completion should be traced");
    assert!(sub_step_six_index < completion_index);
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
