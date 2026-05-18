use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use server::core::rsm::{
    AuctionSettled, BroadcastPhaseChanged, DraftReadySignal, DraftStarted, PlacementSubmitted,
    ResolutionComplete, ResolutionPhaseEntered, RoundPhase, RoundState, RsmPlugin,
};
use server::core::session::{GameSessionPlugin, SessionConfig, SessionReady};
use server::foundation::config::GameConfig;
use server::foundation::rng::ServerRng;
use shared::card::{CardId, ClassId};
use shared::protocol::{DraftPhase, GameMode};
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

fn app_with_rsm(phase: RoundPhase, round_number: u32) -> App {
    let players = [player(1), player(2)];
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(&players));
    app.insert_resource(ServerRng::new());
    app.insert_resource(Time::<()>::default());
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase,
        round_number,
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

#[test]
fn rsm_timers_session_ready_starts_draft_initial_timer_and_emits_entry() {
    let mut app = app_with_rsm(RoundPhase::Lobby, 0);

    app.world_mut().trigger(SessionReady);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftInitial);
    assert_eq!(rsm.round_number, 1);
    assert!(rsm.draft_initial_timer.is_some());

    let drafts = read_messages::<DraftStarted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].phase, DraftPhase::Initial);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftInitial);
    assert_eq!(broadcasts[0].timer_ms, 45_000);
}

#[test]
fn rsm_timers_draft_initial_timer_transitions_to_placement_at_45s() {
    let mut app = app_with_rsm(RoundPhase::DraftInitial, 1);
    app.world_mut()
        .resource_mut::<RoundState>()
        .draft_initial_timer = Some(Timer::from_seconds(45.0, TimerMode::Once));

    run_for(&mut app, Duration::from_secs(44));
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::DraftInitial
    );

    run_for(&mut app, Duration::from_secs(1));

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert_eq!(rsm.round_number, 1);
    assert!(rsm.placement_timer.is_some());

    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::Placement);
    assert_eq!(broadcasts[0].timer_ms, 10_000);
}

#[test]
fn rsm_timers_draft_shop_timer_transitions_to_placement_at_30s() {
    let mut app = app_with_rsm(RoundPhase::DraftShop, 2);
    app.world_mut()
        .resource_mut::<RoundState>()
        .draft_shop_timer = Some(Timer::from_seconds(30.0, TimerMode::Once));

    run_for(&mut app, Duration::from_secs(30));

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(rsm.draft_shop_timer.is_none());
    assert!(rsm.placement_timer.is_some());

    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::Placement);
}

#[test]
fn rsm_timers_placement_timer_waits_until_expiry_with_partial_submissions() {
    let mut app = app_with_rsm(RoundPhase::Placement, 2);
    app.world_mut().resource_mut::<RoundState>().placement_timer =
        Some(Timer::from_seconds(10.0, TimerMode::Once));

    app.world_mut()
        .write_message(PlacementSubmitted { player: player(1) });
    run_once(&mut app);

    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Placement
    );
    assert!(app
        .world()
        .resource::<RoundState>()
        .submissions_received
        .contains(&player(1)));

    run_for(&mut app, Duration::from_millis(9_999));
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Placement
    );

    // Placement timer fires here; a small server-side grace window starts so
    // late-arriving C2SSubmitPlacement messages can still be buffered before
    // the phase advance. Phase must remain `Placement` for the grace span.
    // See PROMPT 1209 / HUNT-1201-14.
    run_for(&mut app, Duration::from_millis(1));
    {
        let rsm = app.world().resource::<RoundState>();
        assert_eq!(rsm.phase, RoundPhase::Placement);
        assert!(rsm.placement_timer.is_none());
        assert!(rsm.placement_deadline_grace_timer.is_some());
    }

    // After the grace duration the advance request fires and the chain runs.
    run_for(&mut app, Duration::from_millis(250));

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert!(rsm.placement_deadline_grace_timer.is_none());
    assert!(rsm.submissions_received.contains(&player(1)));

    let resolutions = read_messages::<ResolutionPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(resolutions.len(), 1);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::Resolution);
}

#[test]
fn rsm_timers_placement_timer_grace_accepts_late_submission_before_advance() {
    // Arrange: placement_timer at 10s with no submissions yet.
    let mut app = app_with_rsm(RoundPhase::Placement, 2);
    app.world_mut().resource_mut::<RoundState>().placement_timer =
        Some(Timer::from_seconds(10.0, TimerMode::Once));

    // Act 1: tick to placement_timer expiry. Phase should remain Placement,
    // grace timer should now be running, and no advance should be pending.
    run_for(&mut app, Duration::from_millis(10_000));
    {
        let rsm = app.world().resource::<RoundState>();
        assert_eq!(
            rsm.phase,
            RoundPhase::Placement,
            "Placement phase must persist into the grace window"
        );
        assert!(
            rsm.placement_timer.is_none(),
            "placement_timer should be dropped once grace starts"
        );
        assert!(
            rsm.placement_deadline_grace_timer.is_some(),
            "grace window should be started on placement_timer expiry"
        );
        assert!(rsm.submissions_received.is_empty());
    }

    // Act 2: simulate a late C2SSubmitPlacement arriving inside the grace by
    // writing PlacementSubmitted (the internal signal produced after
    // handle_placement_submission accepts the submission). rsm_input_reader
    // must still see phase=Placement and record the player.
    app.world_mut()
        .write_message(PlacementSubmitted { player: player(1) });
    app.world_mut()
        .write_message(PlacementSubmitted { player: player(2) });
    run_for(&mut app, Duration::from_millis(50));

    // With both submissions recorded the rsm_input_reader requests the
    // advance immediately even while the grace timer is still running.
    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert!(rsm.placement_deadline_grace_timer.is_none());
    assert!(rsm.submissions_received.contains(&player(1)));
    assert!(rsm.submissions_received.contains(&player(2)));

    let resolutions = read_messages::<ResolutionPhaseEntered>(&app);
    assert_eq!(resolutions.len(), 1);
}

#[test]
fn rsm_timers_placement_timer_grace_advances_when_no_submission_arrives() {
    // Arrange: placement_timer at 10s, no submissions at any point.
    let mut app = app_with_rsm(RoundPhase::Placement, 2);
    app.world_mut().resource_mut::<RoundState>().placement_timer =
        Some(Timer::from_seconds(10.0, TimerMode::Once));

    // Act: tick well past placement_timer + grace.
    run_for(&mut app, Duration::from_millis(10_000));
    {
        let rsm = app.world().resource::<RoundState>();
        assert_eq!(rsm.phase, RoundPhase::Placement);
        assert!(rsm.placement_deadline_grace_timer.is_some());
    }
    run_for(&mut app, Duration::from_millis(250));

    // Assert: phase advances after grace expires; pending submissions remain
    // empty so close_placement_phase has nothing to commit. The acceptance
    // criterion "if no pending placements exist, behavior remains unchanged"
    // means: a no-submission timeout still transitions to Resolution.
    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert!(rsm.placement_timer.is_none());
    assert!(rsm.placement_deadline_grace_timer.is_none());
    assert!(rsm.submissions_received.is_empty());

    let resolutions = read_messages::<ResolutionPhaseEntered>(&app);
    assert_eq!(resolutions.len(), 1);
}

#[test]
fn rsm_timers_placement_timer_does_not_double_advance_after_manual_submit() {
    // Arrange: both players submit before the placement_timer expires; this
    // already requests Placement->Resolution and the grace window must NOT
    // kick in afterwards. Monotonicity guarantee from acceptance shape.
    let mut app = app_with_rsm(RoundPhase::Placement, 2);
    app.world_mut().resource_mut::<RoundState>().placement_timer =
        Some(Timer::from_seconds(10.0, TimerMode::Once));

    app.world_mut()
        .write_message(PlacementSubmitted { player: player(1) });
    app.world_mut()
        .write_message(PlacementSubmitted { player: player(2) });
    run_once(&mut app);

    {
        let rsm = app.world().resource::<RoundState>();
        assert_eq!(rsm.phase, RoundPhase::Resolution);
        assert!(rsm.placement_timer.is_none());
        assert!(rsm.placement_deadline_grace_timer.is_none());
    }

    // Act: keep ticking well past where the placement_timer would have
    // expired. There must not be a second Placement->Resolution advance and
    // the grace window must never start in Resolution.
    run_for(&mut app, Duration::from_millis(11_000));

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert!(rsm.placement_deadline_grace_timer.is_none());

    // exactly one ResolutionPhaseEntered emission across the whole run
    let resolutions = read_messages::<ResolutionPhaseEntered>(&app);
    assert_eq!(
        resolutions.len(),
        1,
        "exactly one Placement->Resolution advance should fire"
    );
    let advances_to_resolution = read_messages::<BroadcastPhaseChanged>(&app)
        .into_iter()
        .filter(|m| m.phase == RoundPhase::Resolution)
        .count();
    assert_eq!(advances_to_resolution, 1);
}

#[test]
fn rsm_timers_placement_all_submit_exits_before_timer_zero() {
    let mut app = app_with_rsm(RoundPhase::Placement, 2);
    app.world_mut().resource_mut::<RoundState>().placement_timer =
        Some(Timer::from_seconds(10.0, TimerMode::Once));

    app.world_mut()
        .write_message(PlacementSubmitted { player: player(1) });
    run_once(&mut app);
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Placement
    );

    app.world_mut()
        .write_message(PlacementSubmitted { player: player(2) });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert!(rsm.placement_timer.is_none());
    assert_eq!(read_messages::<ResolutionPhaseEntered>(&app).len(), 1);
}

#[test]
fn rsm_timers_draft_shop_all_ready_exits_before_timer_zero() {
    let mut app = app_with_rsm(RoundPhase::DraftShop, 2);
    app.world_mut()
        .resource_mut::<RoundState>()
        .draft_shop_timer = Some(Timer::from_seconds(30.0, TimerMode::Once));

    app.world_mut().write_message(DraftReadySignal {
        player: player(1),
        ready: true,
    });
    run_once(&mut app);
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::DraftShop
    );

    app.world_mut().write_message(DraftReadySignal {
        player: player(2),
        ready: true,
    });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(rsm.draft_shop_timer.is_none());
    assert!(rsm.placement_timer.is_some());
}

#[test]
fn rsm_timers_draft_ready_retraction_prevents_early_exit() {
    let mut app = app_with_rsm(RoundPhase::DraftShop, 2);
    app.world_mut()
        .resource_mut::<RoundState>()
        .draft_shop_timer = Some(Timer::from_seconds(30.0, TimerMode::Once));

    app.world_mut().write_message(DraftReadySignal {
        player: player(1),
        ready: true,
    });
    run_once(&mut app);

    app.world_mut().write_message(DraftReadySignal {
        player: player(1),
        ready: false,
    });
    run_once(&mut app);

    app.world_mut().write_message(DraftReadySignal {
        player: player(2),
        ready: true,
    });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert!(!rsm.draft_ready_players.contains(&player(1)));
    assert!(rsm.draft_ready_players.contains(&player(2)));
}

#[test]
fn rsm_timers_stale_auction_settled_is_discarded() {
    let mut app = app_with_rsm(RoundPhase::Placement, 2);

    app.world_mut().write_message(AuctionSettled {
        winner: None,
        final_price: 0,
        card_id: CardId(1),
    });
    run_once(&mut app);

    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Placement
    );
    assert!(read_messages::<BroadcastPhaseChanged>(&app).is_empty());
}

#[test]
fn rsm_timers_stale_resolution_complete_is_discarded() {
    let mut app = app_with_rsm(RoundPhase::Placement, 2);

    app.world_mut().write_message(ResolutionComplete);
    run_once(&mut app);

    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Placement
    );
    assert!(read_messages::<BroadcastPhaseChanged>(&app).is_empty());
}

#[test]
fn rsm_timers_resolution_complete_advances_to_next_draft() {
    let mut app = app_with_rsm(RoundPhase::Resolution, 1);

    app.world_mut().write_message(ResolutionComplete);
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.round_number, 2);
    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert!(rsm.draft_shop_timer.is_some());

    let drafts = read_messages::<DraftStarted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].round, 2);
    assert_eq!(drafts[0].phase, DraftPhase::Shop);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftShop);
    assert_eq!(broadcasts[0].timer_ms, 30_000);
}
