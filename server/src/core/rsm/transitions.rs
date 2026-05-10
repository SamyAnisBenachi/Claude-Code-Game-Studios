use super::events::RsmNetworkOutbox;
use super::events::{
    AbortAuction, AuctionPhaseEntered, AuctionSettled, BeginResolution, BroadcastPhaseChanged,
    DraftReadySignal, DraftStarted, GameOverEmitted, LobbyComplete, PlacementPhaseEntered,
    PlacementSubmitted, PlayerDisconnected, PlayerHeartbeat, PlayerReconnected, ResolutionComplete,
    ResolutionPhaseEntered, ShopRefreshTrigger, ShopRefreshTriggered,
};
use super::state::{
    GameOverRequest, PendingPhaseAdvance, PhaseAdvanceRequest, RoundPhase, RoundState,
};
use std::time::Duration;

use crate::core::objective_contract::ObjectiveCounters;
use crate::core::session::{
    LobbyHeartbeats, PlayerConnectionMap, PlayerSessions, SessionConfig, SessionReady,
};
use crate::foundation::rng::ServerRng;
use bevy::prelude::*;
use lightyear::prelude::{Connected, Disconnected, RemoteId};
use shared::protocol::{DraftPhase, GameOverReason, S2CPhaseChanged};
use shared::session::PlayerId;

pub fn is_auction_round(round_number: u32) -> bool {
    debug_assert!(
        round_number >= 1,
        "round_number must be initialized before auction routing"
    );
    round_number % 3 == 0
}

pub fn rsm_input_reader(
    mut rsm: ResMut<RoundState>,
    session: Option<Res<SessionConfig>>,
    objective_counters: Res<ObjectiveCounters>,
    mut pending: ResMut<PendingPhaseAdvance>,
    mut auction_settled: MessageReader<AuctionSettled>,
    mut resolution_complete: MessageReader<ResolutionComplete>,
    mut ready_signals: MessageReader<DraftReadySignal>,
    mut placement_submitted: MessageReader<PlacementSubmitted>,
) {
    for _event in auction_settled.read() {
        if rsm.phase != RoundPhase::DraftAuction {
            continue;
        }
        pending.request(PhaseAdvanceRequest::new(RoundPhase::DraftAuction));
    }

    for _event in resolution_complete.read() {
        if rsm.phase != RoundPhase::Resolution {
            continue;
        }

        if let Some(outcome) = rsm.pending_disconnect_outcome.take() {
            pending.request(PhaseAdvanceRequest::game_over(
                RoundPhase::Resolution,
                outcome.reason,
                outcome.loser,
            ));
            continue;
        }

        pending.request(
            evaluate_objective_win_condition(&objective_counters, session.as_deref())
                .map(|(reason, loser)| {
                    PhaseAdvanceRequest::game_over(RoundPhase::Resolution, reason, loser)
                })
                .unwrap_or_else(|| PhaseAdvanceRequest::new(RoundPhase::Resolution)),
        );
    }

    for signal in ready_signals.read() {
        if !matches!(rsm.phase, RoundPhase::DraftInitial | RoundPhase::DraftShop) {
            continue;
        }
        if !player_is_in_session(signal.player, session.as_deref()) {
            continue;
        }

        if signal.ready {
            rsm.draft_ready_players.insert(signal.player);
        } else {
            rsm.draft_ready_players.remove(&signal.player);
        }

        if all_players_seen(&rsm.draft_ready_players, session.as_deref()) {
            pending.request(PhaseAdvanceRequest::new(rsm.phase));
        }
    }

    for submission in placement_submitted.read() {
        if rsm.phase != RoundPhase::Placement {
            continue;
        }
        if !player_is_in_session(submission.player, session.as_deref()) {
            continue;
        }

        rsm.submissions_received.insert(submission.player);
        if all_players_seen(&rsm.submissions_received, session.as_deref()) {
            pending.request(PhaseAdvanceRequest::new(RoundPhase::Placement));
        }
    }
}

pub fn tick_disconnect_timers(
    mut rsm: ResMut<RoundState>,
    time: Res<Time>,
    config: Option<Res<crate::foundation::config::GameConfig>>,
    session: Option<Res<SessionConfig>>,
    mut pending: ResMut<PendingPhaseAdvance>,
    mut disconnected: MessageReader<PlayerDisconnected>,
    mut reconnected: MessageReader<PlayerReconnected>,
    mut heartbeats: MessageReader<PlayerHeartbeat>,
    mut abort_auction: MessageWriter<AbortAuction>,
) {
    let session = session.as_deref();
    let grace_ms = disconnect_grace_ms(&config);

    for event in disconnected.read() {
        if player_is_in_session(event.player, session) {
            rsm.disconnect_trackers
                .entry(event.player)
                .or_insert(grace_ms);
        }
    }

    for event in reconnected.read() {
        if player_is_in_session(event.player, session) {
            rsm.disconnect_trackers.insert(event.player, grace_ms);
        }
    }

    for event in heartbeats.read() {
        if player_is_in_session(event.player, session) {
            rsm.disconnect_trackers.insert(event.player, grace_ms);
        }
    }

    if rsm.phase == RoundPhase::GameOver || pending.is_requested() {
        return;
    }

    let delta_ms = elapsed_millis(time.delta());
    let phase = rsm.phase;
    let mut breaching_players = Vec::new();
    for (player, remaining_ms) in rsm.disconnect_trackers.iter_mut() {
        if !player_is_in_session(*player, session) {
            continue;
        }

        let before = *remaining_ms;
        *remaining_ms = remaining_ms.saturating_sub(delta_ms);
        if delta_ms > before {
            tracing::warn!(
                player_id = ?*player,
                remaining_ms_before = before,
                delta_ms = delta_ms,
                phase = ?phase,
                "RSM disconnect timer breach: grace window exceeded"
            );
            breaching_players.push(*player);
        }
    }

    if rsm.pending_disconnect_outcome.is_some() {
        return;
    }

    let Some(outcome) = disconnect_game_over_outcome(&breaching_players) else {
        return;
    };

    if rsm.phase == RoundPhase::Resolution {
        rsm.pending_disconnect_outcome = Some(outcome);
        return;
    }

    if rsm.phase == RoundPhase::DraftAuction {
        abort_auction.write(AbortAuction);
    }

    pending.request(PhaseAdvanceRequest::game_over(
        rsm.phase,
        outcome.reason,
        outcome.loser,
    ));
}

pub fn on_lightyear_connected(
    trigger: On<Add, Connected>,
    clients: Query<&RemoteId>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut reconnected: MessageWriter<PlayerReconnected>,
) {
    let Ok(remote) = clients.get(trigger.entity) else {
        return;
    };
    let Some(connections) = connections else {
        return;
    };
    let Some(player) = connections.0.get(&remote.0).copied() else {
        return;
    };

    tracing::info!(
        peer_id = ?remote.0,
        player_id = ?player,
        "RSM lightyear connected"
    );
    reconnected.write(PlayerReconnected { player });
}

pub fn on_lightyear_disconnected(
    trigger: On<Add, Disconnected>,
    clients: Query<&RemoteId>,
    connections: Option<Res<PlayerConnectionMap>>,
    rsm: Option<Res<RoundState>>,
    mut disconnected: MessageWriter<PlayerDisconnected>,
) {
    let Ok(remote) = clients.get(trigger.entity) else {
        return;
    };
    let Some(connections) = connections else {
        return;
    };
    let Some(player) = connections.0.get(&remote.0).copied() else {
        return;
    };

    tracing::info!(
        peer_id = ?remote.0,
        player_id = ?player,
        phase = ?rsm.as_deref().map(|s| s.phase),
        "RSM lightyear disconnected"
    );
    disconnected.write(PlayerDisconnected { player });
}

pub fn tick_rsm_timers(
    mut rsm: ResMut<RoundState>,
    time: Res<Time>,
    mut pending: ResMut<PendingPhaseAdvance>,
) {
    if pending.is_requested() {
        return;
    }

    let elapsed = time.delta();
    let finished = match rsm.phase {
        RoundPhase::DraftInitial => tick_timer(rsm.draft_initial_timer.as_mut(), elapsed),
        RoundPhase::DraftShop => tick_timer(rsm.draft_shop_timer.as_mut(), elapsed),
        RoundPhase::Placement => tick_timer(rsm.placement_timer.as_mut(), elapsed),
        RoundPhase::Resolution => tick_timer(rsm.resolution_safety_timer.as_mut(), elapsed),
        RoundPhase::Lobby | RoundPhase::DraftAuction | RoundPhase::GameOver => false,
    };

    if finished {
        let timer_name = match rsm.phase {
            RoundPhase::DraftInitial => "draft_initial",
            RoundPhase::DraftShop => "draft_shop",
            RoundPhase::Placement => "placement",
            RoundPhase::Resolution => "resolution_safety",
            RoundPhase::Lobby | RoundPhase::DraftAuction | RoundPhase::GameOver => "",
        };
        tracing::info!(
            phase = ?rsm.phase,
            round = rsm.round_number,
            timer = timer_name,
            "RSM phase timer finished"
        );
        if rsm.phase == RoundPhase::Resolution {
            pending.request(PhaseAdvanceRequest::game_over(
                RoundPhase::Resolution,
                GameOverReason::ResolutionTimeout,
                None,
            ));
        } else {
            pending.request(PhaseAdvanceRequest::new(rsm.phase));
        }
    }
}

pub fn on_session_ready(
    _trigger: On<SessionReady>,
    mut commands: Commands,
    mut rsm: ResMut<RoundState>,
    session: Res<SessionConfig>,
    _rng: Res<ServerRng>,
    config: Option<Res<crate::foundation::config::GameConfig>>,
    lobby_heartbeats: Option<Res<LobbyHeartbeats>>,
    mut sessions: Option<ResMut<PlayerSessions>>,
    mut network_outbox: Option<ResMut<RsmNetworkOutbox>>,
    mut lobby_complete: MessageWriter<LobbyComplete>,
    mut draft_started: MessageWriter<DraftStarted>,
    mut shop_refresh: MessageWriter<ShopRefreshTriggered>,
    mut auction_entered: MessageWriter<AuctionPhaseEntered>,
    mut broadcast: MessageWriter<BroadcastPhaseChanged>,
) {
    if rsm.phase != RoundPhase::Lobby {
        return;
    }

    tracing::info!(
        player_count = session.player_count,
        round_number = rsm.round_number,
        "RSM on_session_ready: entering DRAFT_INITIAL"
    );

    let session = Some(session);
    enter_draft_initial(
        &mut rsm,
        &session,
        &config,
        sessions.as_deref_mut(),
        &mut lobby_complete,
        &mut draft_started,
        &mut shop_refresh,
        &mut auction_entered,
        &mut broadcast,
    );

    if lobby_heartbeats.is_some() {
        commands.remove_resource::<LobbyHeartbeats>();
    }

    if let Some(outbox) = network_outbox.as_deref_mut() {
        outbox.push_phase_changed(S2CPhaseChanged {
            phase: protocol_round_phase(rsm.phase),
            round_number: rsm.round_number,
            timer_duration_ms: draft_timer_ms(DraftPhase::Initial, &config),
        });
    }
}

pub fn advance_phase(
    mut rsm: ResMut<RoundState>,
    request: Option<Res<PhaseAdvanceRequest>>,
    mut pending: Option<ResMut<PendingPhaseAdvance>>,
    session: Option<Res<SessionConfig>>,
    config: Option<Res<crate::foundation::config::GameConfig>>,
    mut sessions: Option<ResMut<PlayerSessions>>,
    mut lobby_complete: MessageWriter<LobbyComplete>,
    mut draft_started: MessageWriter<DraftStarted>,
    mut shop_refresh: MessageWriter<ShopRefreshTriggered>,
    mut auction_entered: MessageWriter<AuctionPhaseEntered>,
    mut placement_entered: MessageWriter<PlacementPhaseEntered>,
    mut resolution_entered: MessageWriter<ResolutionPhaseEntered>,
    mut begin_resolution: MessageWriter<BeginResolution>,
    mut game_over_emitted: MessageWriter<GameOverEmitted>,
    mut broadcast: MessageWriter<BroadcastPhaseChanged>,
) {
    let request = pending
        .as_deref_mut()
        .and_then(PendingPhaseAdvance::take)
        .or_else(|| request.map(|request| request.as_ref().clone()));

    let Some(request) = request else {
        return;
    };

    if rsm.phase != request.expected_source {
        return;
    }

    let from_phase = rsm.phase;

    if let Some(game_over) = &request.game_over {
        rsm.phase = RoundPhase::GameOver;
        rsm.placement_timer = None;
        rsm.draft_shop_timer = None;
        rsm.draft_initial_timer = None;
        rsm.resolution_safety_timer = None;
        rsm.auction_safety_timer = None;
        tracing::info!(
            from = ?from_phase,
            to = ?RoundPhase::GameOver,
            round = rsm.round_number,
            game_over = true,
            "RSM advance_phase: game over"
        );
        game_over_emitted.write(GameOverEmitted {
            reason: game_over.reason,
            loser: game_over.loser,
            round: rsm.round_number,
        });
        tracing::info!(
            reason = ?game_over.reason,
            loser = ?game_over.loser,
            round = rsm.round_number,
            "RSM GameOverEmitted dispatched"
        );
        broadcast.write(BroadcastPhaseChanged {
            phase: RoundPhase::GameOver,
            round: rsm.round_number,
            timer_ms: 0,
        });
        return;
    }

    match rsm.phase {
        RoundPhase::Lobby => {
            enter_draft_initial(
                &mut rsm,
                &session,
                &config,
                sessions.as_deref_mut(),
                &mut lobby_complete,
                &mut draft_started,
                &mut shop_refresh,
                &mut auction_entered,
                &mut broadcast,
            );
            tracing::info!(
                from = ?from_phase,
                to = ?rsm.phase,
                round = rsm.round_number,
                game_over = false,
                "RSM advance_phase: Lobby -> DraftInitial"
            );
        }
        RoundPhase::DraftInitial => {
            rsm.phase = RoundPhase::Placement;
            rsm.draft_initial_timer = None;
            rsm.draft_shop_timer = None;
            rsm.draft_ready_players.clear();
            rsm.submissions_received.clear();
            let timer_ms = placement_timer_ms(session.as_deref(), &config, false);
            rsm.placement_timer = (timer_ms > 0).then(|| once_timer_ms(timer_ms));
            placement_entered.write(PlacementPhaseEntered {
                round: rsm.round_number,
            });
            broadcast.write(BroadcastPhaseChanged {
                phase: RoundPhase::Placement,
                round: rsm.round_number,
                timer_ms,
            });
            tracing::info!(
                from = ?from_phase,
                to = ?rsm.phase,
                round = rsm.round_number,
                game_over = false,
                "RSM advance_phase: DraftInitial -> Placement"
            );
        }
        RoundPhase::DraftAuction => {
            rsm.phase = RoundPhase::DraftShop;
            rsm.draft_ready_players.clear();
            rsm.draft_shop_timer = config
                .as_ref()
                .map(|config| once_timer(config.draft_shop_timer_seconds));
            emit_draft_entry(
                &mut rsm,
                &session,
                &config,
                DraftPhase::Shop,
                &mut draft_started,
                &mut shop_refresh,
                ShopRefreshTrigger::ShopUnlock,
                None,
                &mut auction_entered,
                &mut broadcast,
            );
            tracing::info!(
                from = ?from_phase,
                to = ?rsm.phase,
                round = rsm.round_number,
                game_over = false,
                "RSM advance_phase: DraftAuction -> DraftShop"
            );
        }
        RoundPhase::DraftShop => {
            rsm.phase = RoundPhase::Placement;
            rsm.draft_shop_timer = None;
            rsm.draft_ready_players.clear();
            rsm.submissions_received.clear();
            let timer_ms = placement_timer_ms(
                session.as_deref(),
                &config,
                is_auction_round(rsm.round_number),
            );
            rsm.placement_timer = (timer_ms > 0).then(|| once_timer_ms(timer_ms));
            placement_entered.write(PlacementPhaseEntered {
                round: rsm.round_number,
            });
            broadcast.write(BroadcastPhaseChanged {
                phase: RoundPhase::Placement,
                round: rsm.round_number,
                timer_ms,
            });
            tracing::info!(
                from = ?from_phase,
                to = ?rsm.phase,
                round = rsm.round_number,
                game_over = false,
                "RSM advance_phase: DraftShop -> Placement"
            );
        }
        RoundPhase::Placement => {
            rsm.phase = RoundPhase::Resolution;
            rsm.placement_timer = None;
            rsm.resolution_safety_timer = config
                .as_ref()
                .map(|config| once_timer(config.resolution_max_duration_seconds));
            resolution_entered.write(ResolutionPhaseEntered {
                round: rsm.round_number,
            });
            begin_resolution.write(BeginResolution {
                round: rsm.round_number,
            });
            broadcast.write(BroadcastPhaseChanged {
                phase: RoundPhase::Resolution,
                round: rsm.round_number,
                timer_ms: 0,
            });
            tracing::info!(
                from = ?from_phase,
                to = ?rsm.phase,
                round = rsm.round_number,
                game_over = false,
                "RSM advance_phase: Placement -> Resolution"
            );
        }
        RoundPhase::Resolution => {
            rsm.resolution_safety_timer = None;
            rsm.round_number += 1;
            debug_assert!(
                rsm.round_number >= 1,
                "round_number was not initialized before resolution exit"
            );
            let enters_auction = is_auction_round(rsm.round_number);
            let next_round = rsm.round_number;
            let draft_phase = if enters_auction {
                rsm.phase = RoundPhase::DraftAuction;
                rsm.draft_shop_timer = None;
                DraftPhase::Auction
            } else {
                rsm.phase = RoundPhase::DraftShop;
                rsm.draft_ready_players.clear();
                rsm.draft_shop_timer = config
                    .as_ref()
                    .map(|config| once_timer(config.draft_shop_timer_seconds));
                DraftPhase::Shop
            };
            emit_draft_entry(
                &mut rsm,
                &session,
                &config,
                draft_phase,
                &mut draft_started,
                &mut shop_refresh,
                if enters_auction {
                    ShopRefreshTrigger::AuctionLock
                } else {
                    ShopRefreshTrigger::ShopOpen
                },
                enters_auction.then_some(next_round),
                &mut auction_entered,
                &mut broadcast,
            );
            tracing::info!(
                from = ?from_phase,
                to = ?rsm.phase,
                round = rsm.round_number,
                game_over = false,
                "RSM advance_phase: Resolution -> next draft"
            );
        }
        RoundPhase::GameOver => {
            tracing::info!(
                from = ?from_phase,
                to = ?rsm.phase,
                round = rsm.round_number,
                game_over = false,
                "RSM advance_phase: no-op (already GameOver)"
            );
            return;
        }
    }
}

fn enter_draft_initial(
    rsm: &mut RoundState,
    session: &Option<Res<SessionConfig>>,
    config: &Option<Res<crate::foundation::config::GameConfig>>,
    sessions: Option<&mut PlayerSessions>,
    lobby_complete: &mut MessageWriter<LobbyComplete>,
    draft_started: &mut MessageWriter<DraftStarted>,
    shop_refresh: &mut MessageWriter<ShopRefreshTriggered>,
    auction_entered: &mut MessageWriter<AuctionPhaseEntered>,
    broadcast: &mut MessageWriter<BroadcastPhaseChanged>,
) {
    if let Some(sessions) = sessions {
        if !sessions.all_classes_chosen() {
            return;
        }
        sessions.lock_all_classes();
    }

    rsm.phase = RoundPhase::DraftInitial;
    rsm.round_number = 1;
    rsm.draft_ready_players.clear();
    rsm.submissions_received.clear();
    rsm.draft_shop_timer = None;
    rsm.placement_timer = None;
    rsm.draft_initial_timer = config
        .as_ref()
        .map(|config| once_timer(config.draft_initial_timer_seconds));
    reset_disconnect_trackers_for_session(rsm, session, config);

    tracing::info!(
        round = rsm.round_number,
        auction_round = is_auction_round(rsm.round_number),
        "RSM enter_draft_initial"
    );
    lobby_complete.write(LobbyComplete);
    emit_draft_entry(
        rsm,
        session,
        config,
        DraftPhase::Initial,
        draft_started,
        shop_refresh,
        ShopRefreshTrigger::DraftInitial,
        None,
        auction_entered,
        broadcast,
    );
}

fn emit_draft_entry(
    rsm: &mut RoundState,
    session: &Option<Res<SessionConfig>>,
    config: &Option<Res<crate::foundation::config::GameConfig>>,
    draft_phase: DraftPhase,
    draft_started: &mut MessageWriter<DraftStarted>,
    shop_refresh: &mut MessageWriter<ShopRefreshTriggered>,
    refresh_trigger: ShopRefreshTrigger,
    auction_round: Option<u32>,
    auction_entered: &mut MessageWriter<AuctionPhaseEntered>,
    broadcast: &mut MessageWriter<BroadcastPhaseChanged>,
) {
    draft_started.write(DraftStarted {
        round: rsm.round_number,
        phase: draft_phase,
    });

    if let Some(session) = session {
        for player in session.players() {
            shop_refresh.write(ShopRefreshTriggered {
                player_id: player,
                trigger: refresh_trigger,
            });
        }
    }

    if let Some(round) = auction_round {
        auction_entered.write(AuctionPhaseEntered { round });
    }

    broadcast.write(BroadcastPhaseChanged {
        phase: rsm.phase,
        round: rsm.round_number,
        timer_ms: draft_timer_ms(draft_phase, config),
    });
}

fn draft_timer_ms(
    draft_phase: DraftPhase,
    config: &Option<Res<crate::foundation::config::GameConfig>>,
) -> u32 {
    let Some(config) = config else {
        return 0;
    };
    match draft_phase {
        DraftPhase::Initial => seconds_to_ms(config.draft_initial_timer_seconds),
        DraftPhase::Auction => 0,
        DraftPhase::Shop => seconds_to_ms(config.draft_shop_timer_seconds),
    }
}

fn seconds_to_ms(seconds: u32) -> u32 {
    seconds.saturating_mul(1000)
}

fn placement_timer_ms(
    session: Option<&SessionConfig>,
    config: &Option<Res<crate::foundation::config::GameConfig>>,
    auction_followup: bool,
) -> u32 {
    let Some(config) = config else {
        return 0;
    };

    let base_seconds = if auction_followup {
        config.auction_followup_placement_timer_seconds
    } else {
        config.placement_timer_seconds
    };
    let base_ms = seconds_to_ms(base_seconds);
    let multiplier = session
        .map(|session| session.placement_timer_multiplier_effective)
        .unwrap_or_default();
    multiplier.apply_to_ms(base_ms)
}

fn disconnect_grace_ms(config: &Option<Res<crate::foundation::config::GameConfig>>) -> u32 {
    seconds_to_ms(
        config
            .as_ref()
            .map(|config| config.disconnect_grace_seconds)
            .unwrap_or_else(|| shared::config::GameConfig::default().disconnect_grace_seconds),
    )
}

fn elapsed_millis(duration: std::time::Duration) -> u32 {
    u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)
}

fn reset_disconnect_trackers_for_session(
    rsm: &mut RoundState,
    session: &Option<Res<SessionConfig>>,
    config: &Option<Res<crate::foundation::config::GameConfig>>,
) {
    let Some(session) = session else {
        return;
    };

    let grace_ms = disconnect_grace_ms(config);
    rsm.disconnect_trackers.clear();
    for player in session.players() {
        rsm.disconnect_trackers.insert(player, grace_ms);
    }
}

fn protocol_round_phase(phase: RoundPhase) -> shared::protocol::RoundPhase {
    match phase {
        RoundPhase::Lobby => shared::protocol::RoundPhase::Lobby,
        RoundPhase::DraftInitial => shared::protocol::RoundPhase::DraftInitial,
        RoundPhase::DraftAuction => shared::protocol::RoundPhase::DraftAuction,
        RoundPhase::DraftShop => shared::protocol::RoundPhase::DraftShop,
        RoundPhase::Placement => shared::protocol::RoundPhase::Placement,
        RoundPhase::Resolution => shared::protocol::RoundPhase::Resolution,
        RoundPhase::GameOver => shared::protocol::RoundPhase::GameOver,
    }
}

fn once_timer(seconds: u32) -> Timer {
    Timer::from_seconds(seconds as f32, TimerMode::Once)
}

fn once_timer_ms(milliseconds: u32) -> Timer {
    Timer::new(
        Duration::from_millis(u64::from(milliseconds)),
        TimerMode::Once,
    )
}

fn tick_timer(timer: Option<&mut Timer>, elapsed: std::time::Duration) -> bool {
    let Some(timer) = timer else {
        return false;
    };

    timer.tick(elapsed);
    timer.just_finished()
}

fn all_players_seen(
    players_seen: &std::collections::HashSet<shared::session::PlayerId>,
    session: Option<&SessionConfig>,
) -> bool {
    let Some(session) = session else {
        return false;
    };
    let expected = usize::from(session.player_count);
    expected > 0 && players_seen.len() >= expected
}

fn player_is_in_session(
    player: shared::session::PlayerId,
    session: Option<&SessionConfig>,
) -> bool {
    session
        .map(|session| session.team_map.contains_key(&player))
        .unwrap_or(false)
}

fn disconnect_game_over_outcome(breaching_players: &[PlayerId]) -> Option<GameOverRequest> {
    match breaching_players {
        [] => None,
        [loser] => Some(GameOverRequest {
            reason: GameOverReason::Disconnect,
            loser: Some(*loser),
        }),
        _ => Some(GameOverRequest {
            reason: GameOverReason::Draw,
            loser: None,
        }),
    }
}

fn evaluate_objective_win_condition(
    objective_counters: &ObjectiveCounters,
    session: Option<&SessionConfig>,
) -> Option<(GameOverReason, Option<PlayerId>)> {
    let session = session?;
    let qualifying_players = session
        .players()
        .filter(|player| objective_counters.real_objectives_destroyed(*player) >= 2)
        .collect::<Vec<_>>();

    match qualifying_players.as_slice() {
        [] => None,
        [loser] => Some((GameOverReason::ObjectivesDestroyed, Some(*loser))),
        _ => Some((GameOverReason::Draw, None)),
    }
}
