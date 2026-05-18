use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use server::core::rsm::{
    AbortAuction, AuctionSettled, BroadcastPhaseChanged, GameOverEmitted, OpponentDisconnectNotice,
    PlayerDisconnected, PlayerHeartbeat, PlayerReconnected, ResolutionComplete, RoundPhase,
    RoundState, RsmPlugin,
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
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(&players));
    app.insert_resource(Time::<()>::default());
    app.init_resource::<DisconnectNoticeLog>();
    // Run the recorder LAST so it observes notices written by tick_disconnect_timers
    // during the same tick.
    app.add_systems(Last, record_disconnect_notices);
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase,
        round_number: 2,
        ..RoundState::new()
    };
    app
}

/// Accumulates every `OpponentDisconnectNotice` ever observed in the test
/// `App`. Necessary because Bevy's `Messages<T>` buffer drops events after
/// two frames, which makes "count notices over many ticks" comparisons
/// flaky if you only inspect the buffer at the end.
#[derive(Resource, Default, Clone)]
struct DisconnectNoticeLog {
    pub seen: Vec<OpponentDisconnectNotice>,
}

fn record_disconnect_notices(
    mut reader: MessageReader<OpponentDisconnectNotice>,
    mut log: ResMut<DisconnectNoticeLog>,
) {
    for notice in reader.read() {
        log.seen.push(*notice);
    }
}

fn notice_log(app: &App) -> Vec<OpponentDisconnectNotice> {
    app.world().resource::<DisconnectNoticeLog>().seen.clone()
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

// PROMPT 1211 -- S18 Opponent Disconnect Broadcast Repair
//
// These tests cover the server-side broadcast hook for
// `S2COpponentDisconnected`. The pre-fix server emitted no
// `OpponentDisconnectNotice` (the rsm-internal message consumed by
// `dispatch_opponent_disconnected`), so the surviving player never saw
// opponent-disconnect grace UI even though the protocol message and the
// client drain both already existed.

#[test]
fn rsm_disconnect_event_emits_first_opponent_disconnect_notice() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);

    let notices = read_messages::<OpponentDisconnectNotice>(&app);
    assert_eq!(
        notices.len(),
        1,
        "exactly one OpponentDisconnectNotice should fire on first PlayerDisconnected event"
    );
    assert_eq!(notices[0].player_id, player(1));
    assert!(
        notices[0].grace_remaining_ms > 0,
        "first notice must carry a positive grace_remaining_ms so the surviving player's UI can render a countdown (got {})",
        notices[0].grace_remaining_ms
    );
    assert!(
        notices[0].grace_remaining_ms <= disconnect_grace_ms(),
        "first notice grace_remaining_ms ({}) must not exceed configured grace_ms ({})",
        notices[0].grace_remaining_ms,
        disconnect_grace_ms()
    );

    // disconnect_notice_state must now be populated so cadence ticks below
    // know to keep emitting until reconnect/heartbeat/breach.
    let rsm = app.world().resource::<RoundState>();
    assert!(
        rsm.disconnect_notice_state.contains_key(&player(1)),
        "RoundState.disconnect_notice_state must track the disconnected player after first notice"
    );
}

#[test]
fn rsm_disconnect_duplicate_event_does_not_double_emit_first_notice() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    // Two PlayerDisconnected events in the same tick (e.g., observer fires
    // twice across runtime quirks) must still produce only one first notice.
    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);

    let notices = read_messages::<OpponentDisconnectNotice>(&app);
    assert_eq!(
        notices.len(),
        1,
        "duplicate PlayerDisconnected events in the same tick must not double-emit first notice"
    );
}

#[test]
fn rsm_disconnect_periodic_notice_cadence_emits_lower_grace_remaining() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);

    // Capture the first-notice grace value, then drain the message buffer so
    // the second tick's read shows only periodic notices.
    let first = read_messages::<OpponentDisconnectNotice>(&app);
    assert_eq!(first.len(), 1);
    let first_grace = first[0].grace_remaining_ms;

    // Advance past the cadence (DISCONNECT_NOTICE_CADENCE_MS == 1000).
    run_for(&mut app, Duration::from_millis(1500));

    let all_notices = read_messages::<OpponentDisconnectNotice>(&app);
    // Filter to periodic notices (after the first one).
    let periodic: Vec<&OpponentDisconnectNotice> = all_notices.iter().skip(1).collect();
    assert!(
        !periodic.is_empty(),
        "at least one periodic OpponentDisconnectNotice should fire after the cadence elapses (got total {})",
        all_notices.len()
    );
    let last = periodic.last().expect("periodic notice present");
    assert_eq!(last.player_id, player(1));
    assert!(
        last.grace_remaining_ms < first_grace,
        "periodic notice ({} ms) must report a lower grace_remaining_ms than the first notice ({} ms)",
        last.grace_remaining_ms,
        first_grace
    );
    assert!(
        last.grace_remaining_ms > 0,
        "periodic notice must still report a positive grace_remaining_ms while in grace (got {})",
        last.grace_remaining_ms
    );
}

#[test]
fn rsm_disconnect_no_periodic_notice_before_cadence() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);

    // Advance only 500 ms — below the 1000 ms cadence.
    run_for(&mut app, Duration::from_millis(500));

    let notices = read_messages::<OpponentDisconnectNotice>(&app);
    assert_eq!(
        notices.len(),
        1,
        "no periodic notice should fire until the cadence threshold is crossed"
    );
}

#[test]
fn rsm_disconnect_reconnect_stops_periodic_notices() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);
    let after_first = notice_log(&app).len();
    assert_eq!(after_first, 1, "first PlayerDisconnected should emit one notice");

    app.world_mut()
        .write_message(PlayerReconnected { player: player(1) });
    run_once(&mut app);
    let baseline = notice_log(&app).len();

    // Advance well past the cadence — no further notices should fire after reconnect.
    run_for(&mut app, Duration::from_millis(2500));

    let after = notice_log(&app).len();
    assert_eq!(
        after, baseline,
        "no further OpponentDisconnectNotice should fire after the player reconnected (baseline {} -> after {})",
        baseline, after
    );

    let rsm = app.world().resource::<RoundState>();
    assert!(
        !rsm.disconnect_notice_state.contains_key(&player(1)),
        "RoundState.disconnect_notice_state must be cleared on PlayerReconnected"
    );
}

#[test]
fn rsm_disconnect_heartbeat_stops_periodic_notices() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);
    let after_first = notice_log(&app).len();
    assert_eq!(after_first, 1, "first PlayerDisconnected should emit one notice");

    app.world_mut()
        .write_message(PlayerHeartbeat { player: player(1) });
    run_once(&mut app);
    let baseline = notice_log(&app).len();

    run_for(&mut app, Duration::from_millis(2500));

    let after = notice_log(&app).len();
    assert_eq!(
        after, baseline,
        "no further OpponentDisconnectNotice should fire after the player heartbeated (baseline {} -> after {})",
        baseline, after
    );
}

#[test]
fn rsm_disconnect_breach_suppresses_periodic_notice() {
    let mut app = app_with_rsm(RoundPhase::Placement);

    app.world_mut()
        .write_message(PlayerDisconnected { player: player(1) });
    run_once(&mut app);

    // Force breach by advancing past full grace window.
    run_for(
        &mut app,
        Duration::from_millis(u64::from(disconnect_grace_ms()) + 1),
    );

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::GameOver);

    // Periodic notices emitted during the breach tick must not be sent for
    // the breaching player — the game-over broadcast supersedes the
    // disconnect-grace UI on the client.
    let notices = read_messages::<OpponentDisconnectNotice>(&app);
    // First-notice (from the PlayerDisconnected event) is allowed; any
    // periodic notices on the *breach* tick are not.
    let periodic_after_breach: Vec<&OpponentDisconnectNotice> = notices
        .iter()
        .skip(1)
        .filter(|n| n.grace_remaining_ms == 0)
        .collect();
    assert!(
        periodic_after_breach.is_empty(),
        "no zero-grace periodic notice should fire on the same tick the breach occurs (got {})",
        periodic_after_breach.len()
    );
}
