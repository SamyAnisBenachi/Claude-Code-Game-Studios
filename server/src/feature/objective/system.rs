use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, PeerId, Replicate, Server, ServerMultiMessageSender};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use shared::card::CardId;
use shared::protocol::{
    DraftPhase, ReliableChannel, ResolutionEvent, S2CObjectiveIdentities, S2CResolutionEvent,
    TaggedEvent,
};
use shared::session::PlayerId;

use crate::core::economy::{AwardGold, ManaCapIncreased};
use crate::core::pool::{PlayerPools, PoolFilter};
use crate::core::rsm::{DraftStarted, ResolutionComplete, ResolutionPhaseEntered};
use crate::core::session::{
    defer_unicast_for_reconnect, DeferredMessage, PlayerConnectionMap, ReconnectTracker,
    SessionConfig,
};
use crate::feature::acquisition::{hand_push, PlayerHands, MAX_HAND_SIZE};
use crate::feature::board::{FakeObjectiveDestroyed, LaneId};
use crate::feature::objective::{
    HiddenObjectives, ObjectiveCounters, ObjectiveDestroyed, ObjectiveHp, ObjectiveSlot,
    PendingObjectiveEvents, OBJECTIVE_LANE_COUNT,
};
use crate::foundation::config::{CardCatalog, GameConfig};
use crate::foundation::rng::ServerRng;

const LOSS_THRESHOLD: u32 = 2;
const DEFAULT_OBJECTIVE_GOLD_REWARD: u32 = 3;
const FAKE_OBJECTIVE_HAND_FULL_GOLD_REWARD: u32 = 1;
const OBJECTIVE_DESTROYED_REVEAL_SUB_STEP: u8 = 6;

/// Unfiltered pool draw used by fake objective FreeCardPick rewards.
pub const FAKE_REWARD_POOL_FILTER: PoolFilter = PoolFilter {
    card_type: None,
    card_types: None,
    class: None,
    rarity: None,
    max_cost: None,
};

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
    destroyed_broadcasts: Vec<ObjectiveDestroyed>,
    resolution_batches: Vec<S2CResolutionEvent>,
}

impl ObjectiveNetworkOutbox {
    pub fn push_identity_dispatch(&mut self, dispatch: ObjectiveIdentityDispatch) {
        self.identity_dispatches.push(dispatch);
    }

    pub fn push_destroyed_broadcasts(
        &mut self,
        events: impl IntoIterator<Item = ObjectiveDestroyed>,
    ) {
        self.destroyed_broadcasts.extend(events);
    }

    pub fn push_resolution_batch(&mut self, message: S2CResolutionEvent) {
        self.resolution_batches.push(message);
    }

    #[allow(dead_code)]
    pub fn identity_dispatches(&self) -> &[ObjectiveIdentityDispatch] {
        &self.identity_dispatches
    }

    #[allow(dead_code)]
    pub fn destroyed_broadcasts(&self) -> &[ObjectiveDestroyed] {
        &self.destroyed_broadcasts
    }

    #[allow(dead_code)]
    pub fn resolution_batches(&self) -> &[S2CResolutionEvent] {
        &self.resolution_batches
    }
}

/// Tracks whether the Objective System has observed RESOLUTION entry.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub struct ObjectiveResolutionState {
    current_round: Option<u32>,
    entries_seen: u32,
}

impl ObjectiveResolutionState {
    #[allow(dead_code)]
    pub const fn current_round(&self) -> Option<u32> {
        self.current_round
    }

    #[allow(dead_code)]
    pub const fn entries_seen(&self) -> u32 {
        self.entries_seen
    }

    fn mark_entered(&mut self, round: u32) {
        self.current_round = Some(round);
        self.entries_seen = self.entries_seen.saturating_add(1);
    }

    fn clear(&mut self) {
        self.current_round = None;
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

/// Subscribes to RESOLUTION entry without polling `RoundState`.
pub fn objective_resolution_ready(
    mut resolution_entered: MessageReader<ResolutionPhaseEntered>,
    mut resolution_state: ResMut<ObjectiveResolutionState>,
) {
    for event in resolution_entered.read() {
        resolution_state.mark_entered(event.round);
    }
}

/// Drains queued objective destructions at the RESOLUTION-end sync point.
///
/// `take_damage` only queues during sub-steps; this system is the sole network
/// and internal-message broadcast point for authoritative destruction reveals.
pub fn broadcast_objective_events(
    mut resolution_complete: MessageReader<ResolutionComplete>,
    mut pending: ResMut<PendingObjectiveEvents>,
    mut objective_destroyed: MessageWriter<ObjectiveDestroyed>,
    mut network_outbox: Option<ResMut<ObjectiveNetworkOutbox>>,
    mut resolution_state: Option<ResMut<ObjectiveResolutionState>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let server = server.single().ok();

    for _event in resolution_complete.read() {
        if let Some(state) = resolution_state.as_deref_mut() {
            state.clear();
        }

        let events = drain_sorted_pending_objective_events(&mut pending);
        if events.is_empty() {
            continue;
        }

        for event in &events {
            objective_destroyed.write(*event);
        }

        let message = objective_destroyed_resolution_batch(&events);

        if let Some(outbox) = network_outbox.as_deref_mut() {
            outbox.push_destroyed_broadcasts(events.iter().copied());
            outbox.push_resolution_batch(message.clone());
        }

        if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
            let _ = sender.send::<S2CResolutionEvent, ReliableChannel>(
                &message,
                server,
                &NetworkTarget::All,
            );
        }
    }
}

fn drain_sorted_pending_objective_events(
    pending: &mut PendingObjectiveEvents,
) -> Vec<ObjectiveDestroyed> {
    pending
        .queue
        .sort_by_key(|event| (event.lane, event.target_player_id.0));
    pending.queue.drain(..).collect()
}

fn objective_destroyed_resolution_batch(events: &[ObjectiveDestroyed]) -> S2CResolutionEvent {
    S2CResolutionEvent {
        events: events
            .iter()
            .map(|event| TaggedEvent {
                sub_step: OBJECTIVE_DESTROYED_REVEAL_SUB_STEP,
                event: ResolutionEvent::ObjectiveDestroyed {
                    target_player_id: event.target_player_id,
                    lane: event.lane,
                    was_fake: event.was_fake,
                },
            })
            .collect(),
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
        apply_consequence_path(world, lane, attacker_player, defending_player);
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
pub fn apply_consequence_path(
    world: &mut World,
    lane: LaneId,
    attacker_player: PlayerId,
    defending_player: PlayerId,
) {
    if let Some((entity, already_destroyed)) = objective_entity(world, defending_player, lane) {
        if already_destroyed {
            return;
        }
        mark_objective_destroyed(world, entity);
    }

    let was_fake = objective_was_fake(world, defending_player, lane);

    queue_objective_destroyed(world, defending_player, lane, was_fake);

    if attacker_player != defending_player {
        emit_award_gold(world, attacker_player);

        if was_fake {
            emit_fake_objective_destroyed(world, attacker_player);
            increment_fake_destroyed(world, attacker_player);
            draw_fake_reward(world, lane, attacker_player);
        }
    }

    if !was_fake {
        increment_real_destroyed(world, defending_player);
    }
}

fn objective_was_fake(world: &World, defending_player: PlayerId, lane: LaneId) -> bool {
    world
        .get_resource::<HiddenObjectives>()
        .and_then(|hidden| hidden.identities.get(&(defending_player, lane)).copied())
        .unwrap_or(false)
}

fn queue_objective_destroyed(
    world: &mut World,
    defending_player: PlayerId,
    lane: LaneId,
    was_fake: bool,
) {
    if let Some(mut pending_events) = world.get_resource_mut::<PendingObjectiveEvents>() {
        pending_events.queue.push(ObjectiveDestroyed {
            target_player_id: defending_player,
            lane,
            was_fake,
        });
    }
}

fn emit_award_gold(world: &mut World, player: PlayerId) {
    let amount = world
        .get_resource::<GameConfig>()
        .map_or(DEFAULT_OBJECTIVE_GOLD_REWARD, |config| {
            config.objective_gold_reward
        });

    emit_award_gold_amount(world, player, amount);
}

fn emit_award_gold_amount(world: &mut World, player: PlayerId, amount: u32) {
    if let Some(mut messages) = world.get_resource_mut::<Messages<AwardGold>>() {
        messages.write(AwardGold { player, amount });
    }
}

fn emit_mana_cap_increased(world: &mut World, player: PlayerId) {
    if let Some(mut messages) = world.get_resource_mut::<Messages<ManaCapIncreased>>() {
        messages.write(ManaCapIncreased { player, amount: 1 });
    }
}

fn emit_fake_objective_destroyed(world: &mut World, player: PlayerId) {
    if let Some(mut messages) = world.get_resource_mut::<Messages<FakeObjectiveDestroyed>>() {
        messages.write(FakeObjectiveDestroyed {
            destroyed_by: player,
        });
    }
}

fn increment_fake_destroyed(world: &mut World, player: PlayerId) {
    if let Some(mut counters) = world.get_resource_mut::<ObjectiveCounters>() {
        counters
            .fake_destroyed
            .entry(player)
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }
}

fn increment_real_destroyed(world: &mut World, player: PlayerId) {
    if let Some(mut counters) = world.get_resource_mut::<ObjectiveCounters>() {
        counters
            .real_destroyed
            .entry(player)
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }
}

/// Draws and applies the D4 fake objective reward for a destroyed fake.
///
/// The caller owns the consequence-path predicate: fake objective destroyed by
/// the opponent. This function owns only the reward roll and application.
#[allow(dead_code)]
pub fn draw_fake_reward(world: &mut World, lane: LaneId, attacker_player: PlayerId) {
    let Ok(rng_player_id) = u32::try_from(attacker_player.0) else {
        warn!(
            player_id = attacker_player.0,
            lane, "fake objective reward skipped because player id exceeds current RNG API width"
        );
        return;
    };

    let Some(reward_seed) = consume_fake_reward_seed(world, rng_player_id, lane) else {
        return;
    };

    match fake_reward_outcome(reward_seed) {
        FakeRewardOutcome::ManaCapIncreased => emit_mana_cap_increased(world, attacker_player),
        FakeRewardOutcome::FreeCardPick => {
            resolve_fake_reward_free_card_pick(world, lane, attacker_player, rng_player_id);
        }
    }
}

fn consume_fake_reward_seed(world: &mut World, rng_player_id: u32, lane: LaneId) -> Option<u64> {
    world
        .get_resource_mut::<ServerRng>()
        .map(|mut rng| rng.award_fake_objective_reward(rng_player_id, lane))
}

fn consume_draw_free_card_seed(world: &mut World, rng_player_id: u32) -> Option<u64> {
    world
        .get_resource_mut::<ServerRng>()
        .map(|mut rng| rng.draw_free_card(rng_player_id))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FakeRewardOutcome {
    ManaCapIncreased,
    FreeCardPick,
}

fn fake_reward_outcome(seed: u64) -> FakeRewardOutcome {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    match rng.gen_range(0u32..2) {
        0 => FakeRewardOutcome::ManaCapIncreased,
        1 => FakeRewardOutcome::FreeCardPick,
        _ => unreachable!("gen_range(0..2) returns only 0 or 1"),
    }
}

fn resolve_fake_reward_free_card_pick(
    world: &mut World,
    lane: LaneId,
    attacker_player: PlayerId,
    rng_player_id: u32,
) {
    let Some(hands) = world.get_resource::<PlayerHands>() else {
        return;
    };

    if hands.hand_len(attacker_player) >= MAX_HAND_SIZE {
        emit_award_gold_amount(world, attacker_player, FAKE_OBJECTIVE_HAND_FULL_GOLD_REWARD);
        return;
    }

    if !world.contains_resource::<CardCatalog>() || !player_has_pool(world, attacker_player) {
        return;
    }

    let Some(draw_seed) = consume_draw_free_card_seed(world, rng_player_id) else {
        return;
    };

    let Some(card_id) = draw_and_distribute_fake_reward_card(world, attacker_player, draw_seed)
    else {
        return;
    };

    if let Some(mut hands) = world.get_resource_mut::<PlayerHands>() {
        if hand_push(&mut hands, attacker_player, card_id).is_err() {
            warn!(
                player_id = attacker_player.0,
                lane,
                card_id = card_id.0,
                "fake objective free card reward dropped after draw because hand became full"
            );
        }
    }
}

fn player_has_pool(world: &World, player: PlayerId) -> bool {
    world
        .get_resource::<PlayerPools>()
        .is_some_and(|pools| pools.pools.contains_key(&player))
}

fn draw_and_distribute_fake_reward_card(
    world: &mut World,
    player: PlayerId,
    draw_seed: u64,
) -> Option<CardId> {
    world.resource_scope(|world, card_catalog: Mut<CardCatalog>| -> Option<CardId> {
        let mut player_pools = world.get_resource_mut::<PlayerPools>()?;
        let pool = player_pools.pools.get_mut(&player)?;
        let card_id = pool.draw_random(&card_catalog.cards, &FAKE_REWARD_POOL_FILTER, draw_seed)?;
        if let Err(error) = pool.distribute(card_id) {
            warn!(
                player_id = player.0,
                card_id = card_id.0,
                ?error,
                "fake objective free card reward could not distribute drawn card"
            );
            return None;
        }
        Some(card_id)
    })
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
