use std::time::Duration;

use bevy::prelude::*;
use server::core::rsm::{
    advance_phase, BroadcastPhaseChanged, GameOverEmitted, PendingPhaseAdvance,
    PhaseAdvanceRequest, RoundPhase, RoundState, RsmNetworkOutbox, RsmPlugin,
};
use server::foundation::config::GameConfig;
use server::network::rsm_dispatch::dispatch_phase_changed;
use shared::protocol::{GameOverReason, RoundPhase as ProtocolRoundPhase};

fn app_with_rsm_dispatch() -> App {
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.insert_resource(Time::<()>::default());
    app.insert_resource(GameConfig(shared::config::GameConfig {
        resolution_max_duration_seconds: 1,
        ..shared::config::GameConfig::default()
    }));
    app.add_systems(Update, dispatch_phase_changed.after(advance_phase));
    app
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn run_for(app: &mut App, duration: Duration) {
    app.world_mut().resource_mut::<Time>().advance_by(duration);
    app.update();
}

#[test]
fn rsm_network_dispatch_sends_one_phase_change_per_broadcast() {
    let mut app = app_with_rsm_dispatch();

    app.update();
    assert!(
        app.world()
            .resource::<RsmNetworkOutbox>()
            .phase_changed()
            .is_empty(),
        "no S2CPhaseChanged should be dispatched without BroadcastPhaseChanged"
    );

    app.world_mut().write_message(BroadcastPhaseChanged {
        phase: RoundPhase::Placement,
        round: 4,
        timer_ms: 10_000,
    });
    app.update();

    let outbox = app.world().resource::<RsmNetworkOutbox>().phase_changed();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].phase, ProtocolRoundPhase::Placement);
    assert_eq!(outbox[0].round_number, 4);
    assert_eq!(outbox[0].timer_duration_ms, 10_000);
}

#[test]
fn rsm_network_dispatch_preserves_each_broadcast_payload_once() {
    let mut app = app_with_rsm_dispatch();

    app.world_mut().write_message(BroadcastPhaseChanged {
        phase: RoundPhase::DraftShop,
        round: 2,
        timer_ms: 30_000,
    });
    app.world_mut().write_message(BroadcastPhaseChanged {
        phase: RoundPhase::Resolution,
        round: 2,
        timer_ms: 60_000,
    });
    app.update();

    let outbox = app.world().resource::<RsmNetworkOutbox>().phase_changed();
    assert_eq!(outbox.len(), 2);
    assert_eq!(outbox[0].phase, ProtocolRoundPhase::DraftShop);
    assert_eq!(outbox[0].round_number, 2);
    assert_eq!(outbox[0].timer_duration_ms, 30_000);
    assert_eq!(outbox[1].phase, ProtocolRoundPhase::Resolution);
    assert_eq!(outbox[1].round_number, 2);
    assert_eq!(outbox[1].timer_duration_ms, 60_000);
}

#[test]
fn rsm_resolution_safety_timeout_transitions_to_game_over() {
    let mut app = app_with_rsm_dispatch();
    {
        let mut rsm = app.world_mut().resource_mut::<RoundState>();
        rsm.phase = RoundPhase::Placement;
        rsm.round_number = 5;
    }
    app.world_mut()
        .resource_mut::<PendingPhaseAdvance>()
        .request(PhaseAdvanceRequest::new(RoundPhase::Placement));

    app.update();
    {
        let rsm = app.world().resource::<RoundState>();
        assert_eq!(rsm.phase, RoundPhase::Resolution);
        assert!(rsm.resolution_safety_timer.is_some());
    }

    run_for(&mut app, Duration::from_millis(1_001));

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);
    let phase_changes = app.world().resource::<RsmNetworkOutbox>().phase_changed();

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert!(rsm.resolution_safety_timer.is_none());
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::ResolutionTimeout);
    assert_eq!(game_over[0].loser, None);
    assert_eq!(
        phase_changes
            .last()
            .expect("game-over phase dispatch")
            .phase,
        ProtocolRoundPhase::GameOver
    );
}
