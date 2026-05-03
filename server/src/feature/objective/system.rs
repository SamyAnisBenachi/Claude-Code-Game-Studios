use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, PeerId, Replicate, Server, ServerMultiMessageSender};
use shared::protocol::{DraftPhase, ReliableChannel, S2CObjectiveIdentities};
use shared::session::PlayerId;

use crate::core::rsm::DraftStarted;
use crate::core::session::{
    defer_unicast_for_reconnect, DeferredMessage, PlayerConnectionMap, ReconnectTracker,
    SessionConfig,
};
use crate::feature::board::LaneId;
use crate::feature::objective::{
    HiddenObjectives, ObjectiveCounters, ObjectiveDestroyed, ObjectiveHp, ObjectiveSlot,
    PendingObjectiveEvents, OBJECTIVE_LANE_COUNT,
};
use crate::foundation::config::GameConfig;
use crate::foundation::rng::ServerRng;

const LOSS_THRESHOLD: u32 = 2;

/// Local signal emitted after hidden objective identities are ready to send.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct ObjectiveIdentitiesReady {
    pub players: Vec<PlayerId>,
}

#[derive(Clone, Debug)]
pub struct ObjectiveIdentityDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CObjectiveIdentities,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct ObjectiveNetworkOutbox {
    identity_dispatches: Vec<ObjectiveIdentityDispatch>,
}

impl ObjectiveNetworkOutbox {
    pub fn push_identity_dispatch(&mut self, dispatch: ObjectiveIdentityDispatch) {
        self.identity_dispatches.push(dispatch);
    }

    #[allow(dead_code)]
    pub fn identity_dispatches(&self) -> &[ObjectiveIdentityDispatch] {
        &self.identity_dispatches
    }
}

/// Initializes objective slots on DRAFT_INITIAL entry.
///
/// Hidden fake identity assignment remains server-only in `HiddenObjectives`.
pub fn initialize_objectives_on_draft_initial(
    mut commands: Commands,
    mut draft_started: MessageReader<DraftStarted>,
    mut identities_ready: MessageWriter<ObjectiveIdentitiesReady>,
    session: Option<Res<SessionConfig>>,
    config: Option<Res<GameConfig>>,
    rng: Option<ResMut<ServerRng>>,
    existing_objectives: Query<Entity, With<ObjectiveSlot>>,
    mut hidden_objectives: ResMut<HiddenObjectives>,
    mut counters: ResMut<ObjectiveCounters>,
) {
    let should_initialize = draft_started
        .read()
        .any(|message| message.phase == DraftPhase::Initial);

    if !should_initialize {
        return;
    }

    let (Some(session), Some(config)) = (session, config) else {
        return;
    };

    let Some(mut rng) = rng else {
        error!("Objective initialization refused: ServerRng resource is missing");
        return;
    };

    if let Err(error) = validate_objective_config(&config.0) {
        error!("Objective initialization refused: {error}");
        return;
    }

    for entity in existing_objectives.iter() {
        commands.entity(entity).despawn();
    }

    let players = session.players().collect::<Vec<_>>();
    counters.reset_for_players(players.iter().copied());
    assign_fake_objectives(
        &mut rng,
        &players,
        config.fake_count as usize,
        &mut hidden_objectives,
    );
    identities_ready.write(ObjectiveIdentitiesReady {
        players: players.clone(),
    });

    for player in players {
        for lane in 1..=OBJECTIVE_LANE_COUNT {
            commands.spawn((
                ObjectiveHp {
                    hp: config.objective_hp,
                },
                ObjectiveSlot {
                    lane,
                    player,
                    destroyed: false,
                },
                Replicate::default(),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn deliver_objective_identities_on_ready(
    mut identities_ready: MessageReader<ObjectiveIdentitiesReady>,
    hidden_objectives: Res<HiddenObjectives>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut reconnect_tracker: Option<ResMut<ReconnectTracker>>,
    mut network_outbox: Option<ResMut<ObjectiveNetworkOutbox>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let server = server.single().ok();

    for ready in identities_ready.read() {
        for player_id in ordered_players(&ready.players) {
            let dispatch = prepare_objective_identity_dispatch(
                player_id,
                &hidden_objectives,
                connections.as_deref(),
            );

            if let Some(outbox) = network_outbox.as_deref_mut() {
                outbox.push_identity_dispatch(dispatch.clone());
            }

            if defer_objective_identities(reconnect_tracker.as_deref_mut(), &dispatch) {
                continue;
            }

            if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
                send_objective_identities(sender, server, &dispatch);
            }
        }
    }
}

/// Applies objective damage through the sole Objective System damage entry point.
///
/// `amount` is unsigned by design: objectives do not have a healing interface.
// Scaffold API consumed by downstream combat and spell stories.
#[allow(dead_code)]
pub fn take_damage(world: &mut World, lane: LaneId, attacker_player: PlayerId, amount: u32) {
    if amount == 0 {
        return;
    }

    let Some(defending_player) = defending_player_for_attacker(world, attacker_player) else {
        return;
    };

    let Some((entity, already_destroyed)) = objective_entity(world, defending_player, lane) else {
        return;
    };

    if already_destroyed {
        return;
    }

    let Some((hp_before, hp_after)) = apply_objective_hp_damage(world, entity, amount) else {
        return;
    };

    if hp_after == 0 && hp_before > 0 {
        mark_objective_destroyed(world, entity);
        trigger_consequence_path(world, defending_player, lane);
    }
}

#[allow(dead_code)]
fn defending_player_for_attacker(world: &World, attacker_player: PlayerId) -> Option<PlayerId> {
    let session = world.get_resource::<SessionConfig>()?;
    let attacker_team = session.team_map.get(&attacker_player).copied()?;

    session.players().find(|player| {
        session
            .team_map
            .get(player)
            .copied()
            .is_some_and(|team| team != attacker_team)
    })
}

#[allow(dead_code)]
fn objective_entity(
    world: &mut World,
    defending_player: PlayerId,
    lane: LaneId,
) -> Option<(Entity, bool)> {
    let mut objectives = world.query::<(Entity, &ObjectiveSlot)>();
    objectives.iter(world).find_map(|(entity, slot)| {
        (slot.player == defending_player && slot.lane == lane).then_some((entity, slot.destroyed))
    })
}

#[allow(dead_code)]
fn apply_objective_hp_damage(
    world: &mut World,
    objective: Entity,
    amount: u32,
) -> Option<(u32, u32)> {
    let mut hp = world.get_mut::<ObjectiveHp>(objective)?;
    let hp_before = hp.hp;
    hp.hp = hp_before.saturating_sub(amount);
    Some((hp_before, hp.hp))
}

#[allow(dead_code)]
fn mark_objective_destroyed(world: &mut World, objective: Entity) {
    if let Some(mut slot) = world.get_mut::<ObjectiveSlot>(objective) {
        slot.destroyed = true;
    }
}

#[allow(dead_code)]
fn trigger_consequence_path(world: &mut World, defending_player: PlayerId, lane: LaneId) {
    let was_fake = world
        .get_resource::<HiddenObjectives>()
        .and_then(|hidden| hidden.identities.get(&(defending_player, lane)).copied())
        .map_or(false, |is_fake| is_fake);

    let Some(mut pending_events) = world.get_resource_mut::<PendingObjectiveEvents>() else {
        return;
    };

    pending_events.queue.push(ObjectiveDestroyed {
        target_player_id: defending_player,
        lane,
        was_fake,
    });
}

pub fn prepare_objective_identity_dispatch(
    player_id: PlayerId,
    hidden: &HiddenObjectives,
    connections: Option<&PlayerConnectionMap>,
) -> ObjectiveIdentityDispatch {
    ObjectiveIdentityDispatch {
        player_id,
        peer_id: peer_for_player(connections, player_id),
        message: objective_identities_for_player(hidden, player_id),
    }
}

pub fn objective_identities_for_player(
    hidden: &HiddenObjectives,
    player_id: PlayerId,
) -> S2CObjectiveIdentities {
    let identities = (1..=OBJECTIVE_LANE_COUNT)
        .filter_map(|lane| {
            hidden
                .identities
                .get(&(player_id, lane))
                .copied()
                .map(|is_fake| (lane, is_fake))
        })
        .collect();
    S2CObjectiveIdentities { identities }
}

pub fn defer_objective_identities(
    tracker: Option<&mut ReconnectTracker>,
    dispatch: &ObjectiveIdentityDispatch,
) -> bool {
    defer_unicast_for_reconnect(
        tracker,
        dispatch.player_id,
        DeferredMessage::ObjectiveIdentities(dispatch.message.clone()),
    )
}

fn send_objective_identities(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    dispatch: &ObjectiveIdentityDispatch,
) {
    let Some(peer_id) = dispatch.peer_id else {
        return;
    };

    let _ = sender.send::<S2CObjectiveIdentities, ReliableChannel>(
        &dispatch.message,
        server,
        &NetworkTarget::Single(peer_id),
    );
}

fn peer_for_player(
    connections: Option<&PlayerConnectionMap>,
    player_id: PlayerId,
) -> Option<PeerId> {
    connections.and_then(|connections| {
        connections
            .0
            .iter()
            .find_map(|(peer_id, mapped_player)| (*mapped_player == player_id).then_some(*peer_id))
    })
}

fn ordered_players(players: &[PlayerId]) -> Vec<PlayerId> {
    let mut players = players.to_vec();
    players.sort_by_key(|player| player.0);
    players
}

/// Validates Objective System invariants that must hold before DRAFT_INITIAL.
pub fn validate_objective_config(config: &shared::config::GameConfig) -> Result<(), String> {
    if config.fake_count < 1 {
        return Err(
            "fake_count must be >= 1: D5 would otherwise consume assignment seeds without a valid fake count"
                .to_string(),
        );
    }

    let lane_count = u32::from(OBJECTIVE_LANE_COUNT);
    let max_fake_count = lane_count - LOSS_THRESHOLD;
    if config.fake_count > max_fake_count {
        return Err(format!(
            "fake_count must be <= lane_count - loss_threshold ({max_fake_count}); got {}",
            config.fake_count
        ));
    }

    if config.objective_hp < 1 {
        return Err("objective_hp must be >= 1: objectives cannot spawn destroyed".to_string());
    }

    Ok(())
}

/// Assigns hidden objective identities in ascending player order.
pub fn assign_fake_objectives(
    rng: &mut ServerRng,
    players: &[shared::session::PlayerId],
    fake_count: usize,
    hidden: &mut HiddenObjectives,
) {
    hidden.identities.clear();

    let mut ordered_players = players.to_vec();
    ordered_players.sort_by_key(|player| player.0);

    for player in ordered_players {
        assign_fakes_for_player(rng, player, fake_count, hidden);
    }
}

fn assign_fakes_for_player(
    rng: &mut ServerRng,
    player: shared::session::PlayerId,
    fake_count: usize,
    hidden: &mut HiddenObjectives,
) {
    let mut lanes = (1..=OBJECTIVE_LANE_COUNT).collect::<Vec<_>>();
    let (first_fake, second_fake) = rng.assign_fake_objectives(rng_player_id(player));
    let mut fake_lanes = Vec::with_capacity(fake_count);

    for candidate in [first_fake, second_fake] {
        if fake_lanes.len() >= fake_count {
            break;
        }
        if let Some(position) = lanes.iter().position(|lane| *lane == candidate) {
            fake_lanes.push(lanes.remove(position));
        }
    }

    while fake_lanes.len() < fake_count {
        if lanes.is_empty() {
            break;
        }
        fake_lanes.push(lanes.remove(0));
    }

    for lane in fake_lanes {
        hidden.identities.insert((player, lane), true);
    }

    for lane in lanes {
        hidden.identities.insert((player, lane), false);
    }
}

fn rng_player_id(player: shared::session::PlayerId) -> u32 {
    u32::try_from(player.0).unwrap_or(u32::MAX)
}
