use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, PeerId, Replicate, Server, ServerMultiMessageSender};
use shared::protocol::{
    CardSource, DraftPhase, ReliableChannel, S2CCardAcquired, S2CPrismRewardDropped,
};
use shared::session::PlayerId;

use crate::core::pool::{PlayerPools, PoolFilter};
use crate::core::rsm::{DraftStarted, GameOverEmitted};
use crate::core::rsm::{RoundPhase, RoundState};
use crate::core::session::{DeferredMessage, PlayerConnectionMap, ReconnectTracker, SessionConfig};
use crate::feature::acquisition::{hand_push, PlayerHands, MAX_HAND_SIZE};
use crate::feature::prism::{
    AuditLog, DiscardLog, PrismAuditEntry, PrismCollected, PrismLaneKey, PrismPresence, PrismState,
    MAX_PLAYERS, PRISM_LANE_COUNT,
};
use crate::foundation::config::CardCatalog;
use crate::foundation::rng::ServerRng;
use shared::card::CardType;

const PRISM_STRIKE_CARD_KEY: &str = "prism_strike";
const PRISM_RESERVE_CARD_KEY: &str = "prism_reserve";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrismCardAcquiredDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CCardAcquired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrismRewardDroppedDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CPrismRewardDropped,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct PrismNetworkOutbox {
    card_acquired: Vec<PrismCardAcquiredDispatch>,
    reward_dropped: Vec<PrismRewardDroppedDispatch>,
}

impl PrismNetworkOutbox {
    pub fn push_card_acquired(&mut self, dispatch: PrismCardAcquiredDispatch) {
        self.card_acquired.push(dispatch);
    }

    pub fn push_reward_dropped(&mut self, dispatch: PrismRewardDroppedDispatch) {
        self.reward_dropped.push(dispatch);
    }

    pub fn card_acquired(&self) -> &[PrismCardAcquiredDispatch] {
        &self.card_acquired
    }

    pub fn reward_dropped(&self) -> &[PrismRewardDroppedDispatch] {
        &self.reward_dropped
    }
}

/// Initializes session-scoped Prism resources and replicated presence entities.
pub fn initialize_prism_session(
    mut commands: Commands,
    mut draft_started: MessageReader<DraftStarted>,
    session: Option<Res<SessionConfig>>,
    existing_prisms: Query<Entity, With<PrismLaneKey>>,
) {
    let should_initialize = draft_started
        .read()
        .any(|message| message.phase == DraftPhase::Initial);

    if !should_initialize {
        return;
    }

    let Some(session) = session else {
        return;
    };

    for entity in existing_prisms.iter() {
        commands.entity(entity).despawn();
    }

    commands.insert_resource(PrismState::default());
    commands.insert_resource(DiscardLog::default());
    commands.insert_resource(AuditLog::default());

    for player in session.players().take(MAX_PLAYERS) {
        for lane in 1..=PRISM_LANE_COUNT as u8 {
            commands.spawn((
                PrismLaneKey { player, lane },
                PrismPresence { collected: false },
                Replicate::to_clients(NetworkTarget::All),
            ));
        }
    }
}

/// Removes all session-scoped Prism state at game over.
pub fn cleanup_prism_session(
    mut commands: Commands,
    mut game_over: MessageReader<GameOverEmitted>,
    prism_entities: Query<Entity, With<PrismLaneKey>>,
) {
    if game_over.read().next().is_none() {
        return;
    }

    for entity in prism_entities.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<PrismState>();
    commands.remove_resource::<DiscardLog>();
    commands.remove_resource::<AuditLog>();
}

/// Resolves deterministic Prism lane rewards for Lanes 1/2/4/5.
///
/// Lane 3 RNG draws, hand-full network notification, and full-set respawn are
/// implemented by later Prism stories. This resolver still drains all collection
/// input so stale messages cannot leak into a later frame.
pub fn resolve_prism_draws(
    mut prism_state: Option<ResMut<PrismState>>,
    mut hands: Option<ResMut<PlayerHands>>,
    card_catalog: Option<Res<CardCatalog>>,
    mut discard_log: Option<ResMut<DiscardLog>>,
    mut audit_log: Option<ResMut<AuditLog>>,
    mut server_rng: Option<ResMut<ServerRng>>,
    mut player_pools: Option<ResMut<PlayerPools>>,
    round_state: Option<Res<RoundState>>,
    session: Option<Res<SessionConfig>>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut reconnect_tracker: Option<ResMut<ReconnectTracker>>,
    mut network_outbox: Option<ResMut<PrismNetworkOutbox>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut prism_presence: Query<(&PrismLaneKey, &mut PrismPresence)>,
    mut collected: MessageReader<PrismCollected>,
) {
    let mut events = collected.read().copied().collect::<Vec<_>>();
    if events.is_empty() {
        return;
    }

    if !matches!(
        round_state.as_deref().map(|state| state.phase),
        Some(RoundPhase::Resolution)
    ) {
        return;
    }

    let (Some(prism_state), Some(hands), Some(card_catalog), Some(discard_log)) = (
        prism_state.as_deref_mut(),
        hands.as_deref_mut(),
        card_catalog.as_deref(),
        discard_log.as_deref_mut(),
    ) else {
        warn!("PrismCollected messages discarded because Prism resources are not ready");
        return;
    };

    events.sort_by_key(|event| (event.player_id.0, event.lane));

    let server = server.single().ok();
    let mut network = PrismNetworkContext {
        reconnect_tracker: reconnect_tracker.as_deref_mut(),
        outbox: network_outbox.as_deref_mut(),
        connections: connections.as_deref(),
        server,
        sender: sender.as_mut(),
    };

    for event in events {
        let Some(player_index) = player_index(event.player_id, session.as_deref()) else {
            warn!(
                player_id = event.player_id.0,
                lane = event.lane,
                "PrismCollected discarded for unknown player"
            );
            continue;
        };

        let Some(lane_index) = lane_index(event.lane) else {
            warn!(
                player_id = event.player_id.0,
                lane = event.lane,
                "PrismCollected discarded for invalid lane"
            );
            continue;
        };

        if prism_state.collected[player_index][lane_index] {
            warn!(
                player_id = event.player_id.0,
                lane = event.lane,
                "stale PrismCollected discarded"
            );
            discard_log.entries.push((event.player_id, event.lane));
            continue;
        }

        prism_state.collected[player_index][lane_index] = true;
        set_presence_collected(&mut prism_presence, event.player_id, event.lane, true);

        match event.lane {
            1 | 2 | 4 | 5 => {
                resolve_deterministic_lane_reward(hands, card_catalog, event, &mut network);
            }
            3 => {
                let (Some(audit_log), Some(server_rng), Some(player_pools)) = (
                    audit_log.as_deref_mut(),
                    server_rng.as_deref_mut(),
                    player_pools.as_deref_mut(),
                ) else {
                    warn!(
                        player_id = event.player_id.0,
                        lane = event.lane,
                        "Prism Lane 3 draw skipped because RNG or pool resources are not ready"
                    );
                    continue;
                };

                resolve_lane3_rng_reward(
                    hands,
                    card_catalog,
                    audit_log,
                    server_rng,
                    player_pools,
                    event,
                    &mut network,
                );
            }
            _ => {}
        }
    }
}

fn resolve_deterministic_lane_reward(
    hands: &mut PlayerHands,
    card_catalog: &CardCatalog,
    event: PrismCollected,
    network: &mut PrismNetworkContext<'_, '_, '_>,
) {
    let Some(card_key) = deterministic_lane_card_key(event.lane) else {
        return;
    };
    let Some(card_id) = card_id_for_key(card_catalog, card_key) else {
        warn!(
            player_id = event.player_id.0,
            lane = event.lane,
            card_key,
            "PrismCollected discarded because static reward card is missing from CardCatalog"
        );
        return;
    };

    match hand_push(hands, event.player_id, card_id) {
        Ok(()) => {
            if let Some(source) = prism_card_source(event.lane) {
                network.stage_card_acquired(event.player_id, card_id, source);
            }
        }
        Err(_) => {
            warn!(
                player_id = event.player_id.0,
                lane = event.lane,
                card_id = card_id.0,
                "Prism deterministic reward dropped because player hand is full"
            );
            network.stage_reward_dropped(event.player_id, event.lane);
        }
    }
}

fn resolve_lane3_rng_reward(
    hands: &mut PlayerHands,
    card_catalog: &CardCatalog,
    audit_log: &mut AuditLog,
    server_rng: &mut ServerRng,
    player_pools: &mut PlayerPools,
    event: PrismCollected,
    network: &mut PrismNetworkContext<'_, '_, '_>,
) {
    if hands.hand_len(event.player_id) >= MAX_HAND_SIZE {
        return;
    }

    let Some(pool) = player_pools.pools.get_mut(&event.player_id) else {
        warn!(
            player_id = event.player_id.0,
            lane = event.lane,
            "Prism Lane 3 draw skipped because player pool is missing"
        );
        return;
    };

    let Ok(rng_player_id) = u32::try_from(event.player_id.0) else {
        warn!(
            player_id = event.player_id.0,
            lane = event.lane,
            "Prism Lane 3 draw skipped because player id exceeds current RNG API width"
        );
        return;
    };

    let seed_index = server_rng.current_seed_index();
    let seed = server_rng.resolve_prism(rng_player_id, event.lane);
    let filter = PoolFilter {
        card_types: Some(vec![CardType::Minion, CardType::Spell]),
        ..Default::default()
    };
    let result = pool.draw_random(&card_catalog.cards, &filter, seed);

    audit_log.entries.push(PrismAuditEntry {
        player_id: event.player_id,
        lane: event.lane,
        seed_index,
        result,
    });

    let Some(card_id) = result else {
        return;
    };

    if let Err(error) = pool.distribute(card_id) {
        warn!(
            player_id = event.player_id.0,
            lane = event.lane,
            card_id = card_id.0,
            ?error,
            "Prism Lane 3 draw result could not be distributed"
        );
        return;
    }

    match hand_push(hands, event.player_id, card_id) {
        Ok(()) => {
            if let Some(source) = prism_card_source(event.lane) {
                network.stage_card_acquired(event.player_id, card_id, source);
            }
        }
        Err(_) => {
            warn!(
                player_id = event.player_id.0,
                lane = event.lane,
                card_id = card_id.0,
                "Prism Lane 3 reward dropped after draw because player hand became full"
            );
        }
    }
}

struct PrismNetworkContext<'a, 'w, 's> {
    reconnect_tracker: Option<&'a mut ReconnectTracker>,
    outbox: Option<&'a mut PrismNetworkOutbox>,
    connections: Option<&'a PlayerConnectionMap>,
    server: Option<&'a Server>,
    sender: Option<&'a mut ServerMultiMessageSender<'w, 's>>,
}

impl PrismNetworkContext<'_, '_, '_> {
    fn stage_card_acquired(
        &mut self,
        player_id: PlayerId,
        card_id: shared::card::CardId,
        source: CardSource,
    ) {
        let deferred = DeferredMessage::CardAcquired { card_id, source };
        if self.defer_if_snapshot_pending(player_id, deferred) {
            return;
        }

        let message = S2CCardAcquired { card_id, source };
        let peer_id = self.peer_for_player(player_id);
        if let Some(outbox) = self.outbox.as_deref_mut() {
            outbox.push_card_acquired(PrismCardAcquiredDispatch {
                player_id,
                peer_id,
                message: message.clone(),
            });
        }

        let (Some(server), Some(sender), Some(peer_id)) =
            (self.server, self.sender.as_deref_mut(), peer_id)
        else {
            return;
        };

        let _ = sender.send::<S2CCardAcquired, ReliableChannel>(
            &message,
            server,
            &NetworkTarget::Single(peer_id),
        );
    }

    fn stage_reward_dropped(&mut self, player_id: PlayerId, lane: u8) {
        let deferred = DeferredMessage::PrismRewardDropped { player_id, lane };
        if self.defer_if_snapshot_pending(player_id, deferred) {
            return;
        }

        let message = S2CPrismRewardDropped { player_id, lane };
        let peer_id = self.peer_for_player(player_id);
        if let Some(outbox) = self.outbox.as_deref_mut() {
            outbox.push_reward_dropped(PrismRewardDroppedDispatch {
                player_id,
                peer_id,
                message: message.clone(),
            });
        }

        let (Some(server), Some(sender), Some(peer_id)) =
            (self.server, self.sender.as_deref_mut(), peer_id)
        else {
            return;
        };

        let _ = sender.send::<S2CPrismRewardDropped, ReliableChannel>(
            &message,
            server,
            &NetworkTarget::Single(peer_id),
        );
    }

    fn defer_if_snapshot_pending(&mut self, player_id: PlayerId, message: DeferredMessage) -> bool {
        let Some(tracker) = self.reconnect_tracker.as_deref_mut() else {
            return false;
        };

        if tracker
            .snapshot_sent
            .get(&player_id)
            .copied()
            .unwrap_or(false)
        {
            return false;
        }

        tracker
            .deferred_queue
            .entry(player_id)
            .or_default()
            .push(message);
        true
    }

    fn peer_for_player(&self, player_id: PlayerId) -> Option<PeerId> {
        self.connections.and_then(|connections| {
            connections.0.iter().find_map(|(peer_id, mapped_player)| {
                (*mapped_player == player_id).then_some(*peer_id)
            })
        })
    }
}

fn deterministic_lane_card_key(lane: u8) -> Option<&'static str> {
    match lane {
        1 | 5 => Some(PRISM_STRIKE_CARD_KEY),
        2 | 4 => Some(PRISM_RESERVE_CARD_KEY),
        3 => None,
        _ => None,
    }
}

fn prism_card_source(lane: u8) -> Option<CardSource> {
    match lane {
        1 => Some(CardSource::PrismLane1),
        2 => Some(CardSource::PrismLane2),
        3 => Some(CardSource::PrismLane3),
        4 => Some(CardSource::PrismLane4),
        5 => Some(CardSource::PrismLane5),
        _ => None,
    }
}

fn card_id_for_key(card_catalog: &CardCatalog, card_key: &str) -> Option<shared::card::CardId> {
    let mut matches = card_catalog
        .cards
        .iter()
        .filter_map(|(card_id, card)| (card.art_id == card_key).then_some(*card_id))
        .collect::<Vec<_>>();
    matches.sort_by_key(|card_id| card_id.0);
    matches.first().copied()
}

fn lane_index(lane: u8) -> Option<usize> {
    let index = usize::from(lane.checked_sub(1)?);
    (index < PRISM_LANE_COUNT).then_some(index)
}

fn player_index(player: PlayerId, session: Option<&SessionConfig>) -> Option<usize> {
    if let Some(session) = session {
        return session
            .players()
            .take(MAX_PLAYERS)
            .position(|candidate| candidate == player);
    }

    let index = usize::try_from(player.0.checked_sub(1)?).ok()?;
    (index < MAX_PLAYERS).then_some(index)
}

fn set_presence_collected(
    prism_presence: &mut Query<(&PrismLaneKey, &mut PrismPresence)>,
    player: PlayerId,
    lane: u8,
    collected: bool,
) {
    for (key, mut presence) in prism_presence.iter_mut() {
        if key.player == player && key.lane == lane {
            presence.collected = collected;
        }
    }
}
