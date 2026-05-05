use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use server::core::rsm::{
    AbortAuction, BroadcastPhaseChanged, GameOverEmitted, PlayerDisconnected, PlayerHeartbeat,
    PlayerReconnected, ResolutionComplete, RoundPhase, RoundState, RsmPlugin,
};
use server::core::session::SessionConfig;
use server::foundation::config::GameConfig;
use shared::card::ClassId;
use shared::protocol::{GameMode, GameOverReason};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config(players: &[PlayerId]) -> SessionConfig {
    let mut team_map = HashMap::new();
    let mut class_map = HashMap::new();

    for (index, player) in players.iter().copied().enumerate() {
        team_map.insert(player, index as u8);
        class_map.insert(player, ClassId::Iop);
    }

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map,
        class_map,
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn app_with_rsm(phase: RoundPhase) -> App {
    let players = [player(1), player(2)];
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(&players));
    app.insert_resource(Time::<()>::default());
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase,
        round_number: 2,
        ..RoundState::new()
    };
    app
}

fn run_for(app: &mut App, duration: Duration) {
    app.world_mut().resource_mut::<Time>().advance_by(duration);
    app.update();
}

fn run_once(app: &mut App) {
    run_for(app, Duration::ZERO);
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn disconnect_grace_ms() -> u32 {
    shared::config::GameConfig::default()
        .disconnect_grace_seconds
        .saturating_mul(1000)
}

fn set_tracker(app: &mut App, player: PlayerId, remaining_ms: u32) {
    app.world_mut()
        .resource_mut::<RoundState>()
        .disconnect_trackers
        .insert(player, remaining_ms);
}

#[test]
fn rsm_disconnect_single_disconnect_exceeds_grace_game_over() {
    let mut app = app_with_rsm(RoundPhase::Placement);
    set_tracker(&mut app, player(1), disconnect_grace_ms());

    run_for(
        &mut app,
        Duration::from_millis(u64::from(disconnect_grace_ms()) + 1),
    );

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Disconnect);
    assert_eq!(game_over[0].loser, Some(player(1)));
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::GameOver);
}

#[test]
fn rsm_disconnect_boundary_equal_to_grace_survives() {
    let mut app = app_with_rsm(RoundPhase::Placement);
    set_tracker(&mut app, player(1), disconnect_grace_ms());

    run_for(
        &mut app,
        Duration::from_millis(u64::from(disconnect_grace_ms())),
    );

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert_eq!(rsm.disconnect_trackers.get(&player(1)).copied(), Some(0));
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_disconnect_boundary_below_grace_survives() {
    let mut app = app_with_rsm(RoundPhase::Placement);
    set_tracker(&mut app, player(1), disconnect_grace_ms());

    run_for(
        &mut app,
        Duration::from_millis(u64::from(disconnect_grace_ms() - 1)),
    );

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert_eq!(rsm.disconnect_trackers.get(&player(1)).copied(), Some(1));
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_disconnect_reconnect_within_grace_resets_tracker() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_for(&mut app, Duration::from_secs(15));
    assert_eq!(
        app.world()
            .resource::<RoundState>()
            .disconnect_trackers
            .get(&player(1))
            .copied(),
        Some(disconnect_grace_ms() - 15_000)
    );

    app.world_mut()
        .write_message(PlayerReconnected { player: player(1) });
    run_for(&mut app, Duration::from_secs(20));

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert_eq!(
        rsm.disconnect_trackers.get(&player(1)).copied(),
        Some(disconnect_grace_ms() - 20_000)
    );
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_disconnect_re_disconnect_starts_fresh_tracker() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_for(&mut app, Duration::from_secs(15));
    app.world_mut()
        .write_message(PlayerReconnected { player: player(1) });
    run_once(&mut app);
    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(
        rsm.disconnect_trackers.get(&player(1)).copied(),
        Some(disconnect_grace_ms())
    );
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_disconnect_heartbeat_resets_tracker() {
    let mut app = app_with_rsm(RoundPhase::Placement);
    set_tracker(&mut app, player(1), 1);

    app.world_mut()
        .write_message(PlayerHeartbeat { player: player(1) });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(
        rsm.disconnect_trackers.get(&player(1)).copied(),
        Some(disconnect_grace_ms())
    );
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_disconnect_mutual_disconnect_emits_single_draw() {
    let mut app = app_with_rsm(RoundPhase::Placement);
    set_tracker(&mut app, player(1), 1);
    set_tracker(&mut app, player(2), 1);

    run_for(&mut app, Duration::from_millis(2));

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Draw);
    assert_eq!(game_over[0].loser, None);
}

#[test]
fn rsm_disconnect_one_breaching_player_is_not_draw() {
    let mut app = app_with_rsm(RoundPhase::Placement);
    set_tracker(&mut app, player(1), 1);
    set_tracker(&mut app, player(2), 2);

    run_for(&mut app, Duration::from_millis(2));

    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Disconnect);
    assert_eq!(game_over[0].loser, Some(player(1)));
}

#[test]
fn rsm_disconnect_mid_resolution_defers_until_resolution_complete() {
    let mut app = app_with_rsm(RoundPhase::Resolution);
    set_tracker(&mut app, player(1), 1);

    run_for(&mut app, Duration::from_millis(2));

    {
        let rsm = app.world().resource::<RoundState>();
        assert_eq!(rsm.phase, RoundPhase::Resolution);
        let pending = rsm
            .pending_disconnect_outcome
            .as_ref()
            .expect("disconnect outcome should be deferred");
        assert_eq!(pending.reason, GameOverReason::Disconnect);
        assert_eq!(pending.loser, Some(player(1)));
    }
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());

    app.world_mut().write_message(ResolutionComplete);
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert!(rsm.pending_disconnect_outcome.is_none());
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Disconnect);
    assert_eq!(game_over[0].loser, Some(player(1)));
}

#[test]
fn rsm_disconnect_mid_resolution_mutual_disconnect_defers_draw() {
    let mut app = app_with_rsm(RoundPhase::Resolution);
    set_tracker(&mut app, player(1), 1);
    set_tracker(&mut app, player(2), 1);

    run_for(&mut app, Duration::from_millis(2));

    let rsm = app.world().resource::<RoundState>();
    let pending = rsm
        .pending_disconnect_outcome
        .as_ref()
        .expect("draw outcome should be deferred");

    assert_eq!(pending.reason, GameOverReason::Draw);
    assert_eq!(pending.loser, None);
    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_disconnect_draft_auction_aborts_before_game_over() {
    let mut app = app_with_rsm(RoundPhase::DraftAuction);
    app.world_mut().resource_mut::<RoundState>().round_number = 3;
    set_tracker(&mut app, player(1), 1);

    run_for(&mut app, Duration::from_millis(2));

    let rsm = app.world().resource::<RoundState>();
    let aborts = read_messages::<AbortAuction>(&app);
    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert_eq!(aborts.len(), 1);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Disconnect);
    assert_eq!(game_over[0].loser, Some(player(1)));
}
