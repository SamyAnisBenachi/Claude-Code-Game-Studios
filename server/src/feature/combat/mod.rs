#![allow(dead_code)]

pub mod modifier_stack;

use std::collections::HashMap;

use bevy::ecs::message::MessageCursor;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use shared::card::{CardData, CardId, CardType, Keyword, SimpleKeyword};
use shared::keyword::KeywordKind;
use shared::protocol::{
    EntityId, GameOverReason, PlacedCard, PlayTarget, S2CPlacementReveal, S2CResolutionEvent,
    TaggedEvent,
};
use shared::session::PlayerId;

use crate::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use crate::core::economy::{api as economy_api, PlayerEconomies};
use crate::core::rsm::{
    advance_phase, BeginResolution, PendingPhaseAdvance, PhaseAdvanceRequest, ResolutionComplete,
    RoundPhase,
};
use crate::core::session::SessionConfig;
use crate::feature::board::{
    detect_objective_presence, BoardCell, BoardConfig, BoardGrid, BoardOccupancy, BoardSystemSet,
    LaneId, PendingPlacements, UnitAtObjective,
};
use crate::feature::keyword::components::UnitKeywordState;
use crate::feature::keyword::effects::apply_stun;
use crate::foundation::config::CardCatalog;

pub const DEFAULT_ITERATION_BUDGET: u32 = 10_000;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CombatSystemSet {
    Resolve,
    PostResolution,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BeginResolution>()
            .add_message::<ResolutionComplete>()
            .init_resource::<BeginResolutionCursor>()
            .init_resource::<PendingResolutionComplete>()
            .init_resource::<CombatIterationBudget>()
            .init_resource::<CombatNetworkOutbox>()
            .init_resource::<CombatResolutionTrace>()
            .init_resource::<AppearanceEffectRegistry>()
            .configure_sets(
                Update,
                (CombatSystemSet::Resolve, CombatSystemSet::PostResolution).chain(),
            )
            .add_systems(
                Update,
                resolve_combat
                    .in_set(CombatSystemSet::Resolve)
                    .after(advance_phase)
                    .after(BoardSystemSet::PlacementClose),
            )
            .add_systems(
                Update,
                drain_pending_resolution_complete.in_set(CombatSystemSet::PostResolution),
            );
    }
}

#[derive(Resource, Default)]
struct BeginResolutionCursor(MessageCursor<BeginResolution>);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingResolutionComplete {
    pending: bool,
}

impl PendingResolutionComplete {
    pub fn mark_pending(&mut self) {
        self.pending = true;
    }

    pub fn take(&mut self) -> bool {
        let was_pending = self.pending;
        self.pending = false;
        was_pending
    }

    pub fn is_pending(&self) -> bool {
        self.pending
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatIterationBudget {
    limit: u32,
}

impl CombatIterationBudget {
    pub const fn with_limit(limit: u32) -> Self {
        Self { limit }
    }

    pub const fn limit(self) -> u32 {
        self.limit
    }
}

impl Default for CombatIterationBudget {
    fn default() -> Self {
        Self {
            limit: DEFAULT_ITERATION_BUDGET,
        }
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CombatNetworkOutbox {
    messages: Vec<CombatNetworkMessage>,
}

impl CombatNetworkOutbox {
    pub fn push_placement_reveal(&mut self, message: S2CPlacementReveal) {
        self.messages
            .push(CombatNetworkMessage::PlacementReveal(message));
    }

    pub fn push_resolution_event(&mut self, message: S2CResolutionEvent) {
        self.messages
            .push(CombatNetworkMessage::ResolutionEvent(message));
    }

    pub fn messages(&self) -> &[CombatNetworkMessage] {
        &self.messages
    }

    pub fn message_kinds(&self) -> Vec<CombatNetworkMessageKind> {
        self.messages
            .iter()
            .map(CombatNetworkMessage::kind)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum CombatNetworkMessage {
    PlacementReveal(S2CPlacementReveal),
    ResolutionEvent(S2CResolutionEvent),
}

impl CombatNetworkMessage {
    pub const fn kind(&self) -> CombatNetworkMessageKind {
        match self {
            Self::PlacementReveal(_) => CombatNetworkMessageKind::PlacementReveal,
            Self::ResolutionEvent(_) => CombatNetworkMessageKind::ResolutionEvent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatNetworkMessageKind {
    PlacementReveal,
    ResolutionEvent,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CombatResolutionTrace {
    entries: Vec<CombatTraceEntry>,
}

impl CombatResolutionTrace {
    pub fn push(&mut self, entry: CombatTraceEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[CombatTraceEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatTraceEntry {
    BeginResolutionRead {
        round: u32,
    },
    PlacementRevealEnqueued,
    UnitPlaced {
        entity: Entity,
        lane: LaneId,
        cell: u8,
    },
    KeywordTriggered {
        unit: Entity,
        keyword: KeywordKind,
        sub_step: u8,
    },
    UnitDamaged {
        source: Entity,
        target: Entity,
        amount: u8,
        hp_after: u8,
        sub_step: u8,
    },
    UnitChangedLane {
        unit: Entity,
        from_lane: LaneId,
        to_lane: LaneId,
        sub_step: u8,
    },
    SubStepStarted(u8),
    IterationBudgetExceeded,
    ResolutionEventEnqueued,
    ResolutionCompleteQueued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatAbort {
    IterationBudgetExceeded,
}

struct IterationBudget {
    count: u32,
    limit: u32,
}

impl IterationBudget {
    const fn new(limit: u32) -> Self {
        Self { count: 0, limit }
    }

    fn tick(&mut self) -> Result<(), CombatAbort> {
        self.count = self.count.saturating_add(1);
        if self.count > self.limit {
            Err(CombatAbort::IterationBudgetExceeded)
        } else {
            Ok(())
        }
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct AppearanceEffectRegistry {
    pub effects: HashMap<CardId, Vec<AppearanceEffect>>,
}

impl AppearanceEffectRegistry {
    pub fn insert(&mut self, card_id: CardId, effects: Vec<AppearanceEffect>) {
        self.effects.insert(card_id, effects);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearanceEffect {
    Damage {
        target: AppearanceTarget,
        amount: u8,
    },
    Stun {
        target: AppearanceTarget,
    },
    ChangeLane {
        delta: i8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearanceTarget {
    SelfUnit,
    FirstEnemyInLane,
    UnitAt { lane: LaneId, cell: u8 },
}

pub fn resolve_combat(world: &mut World) {
    let Some(begin_resolution) = read_begin_resolution(world) else {
        return;
    };

    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::BeginResolutionRead {
            round: begin_resolution.round,
        });

    let iteration_limit = world.resource::<CombatIterationBudget>().limit();
    if run_sub_step_scaffold(world, iteration_limit).is_err() {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::IterationBudgetExceeded);
        request_draw_game_over(world);
        return;
    }

    enqueue_resolution_event(world);
    world
        .resource_mut::<PendingResolutionComplete>()
        .mark_pending();
    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::ResolutionCompleteQueued);
}

pub fn drain_pending_resolution_complete(
    mut pending: ResMut<PendingResolutionComplete>,
    mut writer: MessageWriter<ResolutionComplete>,
) {
    if pending.take() {
        writer.write(ResolutionComplete);
    }
}

fn read_begin_resolution(world: &mut World) -> Option<BeginResolution> {
    world.resource_scope(
        |world, mut cursor: Mut<BeginResolutionCursor>| -> Option<BeginResolution> {
            let messages = world.resource::<Messages<BeginResolution>>();
            cursor.0.read(messages).last().copied()
        },
    )
}

fn enqueue_resolution_event(world: &mut World) {
    world
        .resource_mut::<CombatNetworkOutbox>()
        .push_resolution_event(S2CResolutionEvent {
            events: Vec::<TaggedEvent>::new(),
        });
    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::ResolutionEventEnqueued);
}

fn run_sub_step_scaffold(world: &mut World, iteration_limit: u32) -> Result<(), CombatAbort> {
    let mut budget = IterationBudget::new(iteration_limit);

    for sub_step in 1..=6 {
        budget.tick()?;
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::SubStepStarted(sub_step));

        match sub_step {
            1 => apply_placements(world, &mut budget)?,
            6 => run_objective_detection_if_ready(world),
            _ => {}
        }
    }

    Ok(())
}

fn apply_placements(world: &mut World, budget: &mut IterationBudget) -> Result<(), CombatAbort> {
    if !world.contains_resource::<PendingPlacements>() {
        return Ok(());
    }

    let placements = collect_pending_placements(world);
    deduct_committed_mana(world, &placements);
    enqueue_placement_reveal(world, &placements);

    let spawned = spawn_committed_placements(world, &placements);
    let mut defeated_by_appearance = Vec::new();
    let mut queued_lane_changes = Vec::new();

    for unit in appearance_units_in_order(world, &spawned) {
        budget.tick()?;
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit,
                keyword: KeywordKind::Appearance,
                sub_step: 1,
            });

        for effect in appearance_effects_for(unit, world) {
            apply_appearance_effect(
                world,
                unit,
                effect,
                &mut defeated_by_appearance,
                &mut queued_lane_changes,
            );
        }
    }

    fire_deferred_death_triggers(world, budget, &mut defeated_by_appearance)?;
    apply_queued_lane_changes(world, queued_lane_changes);
    clear_pending_placements(world);

    Ok(())
}

fn collect_pending_placements(world: &World) -> Vec<PlacedCard> {
    let pending = world.resource::<PendingPlacements>();
    let mut players = pending.submissions.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);

    players
        .into_iter()
        .filter_map(|player| pending.submissions.get(&player))
        .flat_map(|submission| submission.placements.iter().cloned())
        .collect()
}

fn deduct_committed_mana(world: &mut World, placements: &[PlacedCard]) {
    if placements.is_empty()
        || !world.contains_resource::<CardCatalog>()
        || !world.contains_resource::<PlayerEconomies>()
    {
        return;
    }

    let costs_by_player = {
        let catalog = world.resource::<CardCatalog>();
        let mut costs = HashMap::<PlayerId, u32>::new();
        for placement in placements {
            let Some(card) = catalog.cards.get(&placement.card_id) else {
                continue;
            };
            costs
                .entry(placement.owner_id)
                .and_modify(|cost| *cost = cost.saturating_add(card.cost))
                .or_insert(card.cost);
        }
        costs
    };

    let mut players = costs_by_player.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);

    let mut economies = world.resource_mut::<PlayerEconomies>();
    for player in players {
        let Some(economy) = economies.0.get_mut(&player) else {
            continue;
        };
        let cost = costs_by_player[&player];
        if economy_api::validate_spend(economy, cost, false).is_ok() {
            economy_api::apply_spend(economy, cost, false);
        }
    }
}

fn enqueue_placement_reveal(world: &mut World, placements: &[PlacedCard]) {
    world
        .resource_mut::<CombatNetworkOutbox>()
        .push_placement_reveal(S2CPlacementReveal {
            placements: placements.to_vec(),
        });
    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::PlacementRevealEnqueued);
}

fn spawn_committed_placements(world: &mut World, placements: &[PlacedCard]) -> Vec<Entity> {
    let mut spawned = Vec::new();

    for placement in placements {
        let Some(card) = card_for(placement.card_id, world).cloned() else {
            continue;
        };
        let Some((lane, cell)) = board_cell_target(placement) else {
            continue;
        };

        let entity = world
            .spawn((
                UnitCardRef(placement.card_id),
                UnitOwner(placement.owner_id),
                UnitStats::new(card.hp, card.atk, card.mp, card.ar),
                UnitKeywordState::default(),
                BoardPosition { lane, cell },
            ))
            .id();
        spawned.push(entity);
        update_board_for_spawn(
            world,
            placement.owner_id,
            lane,
            cell,
            card.card_type,
            entity,
        );
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::UnitPlaced { entity, lane, cell });
    }

    spawned
}

fn board_cell_target(placement: &PlacedCard) -> Option<(LaneId, u8)> {
    match placement.target {
        PlayTarget::BoardCell { lane, cell } => Some((lane, cell)),
        PlayTarget::TargetUnit { .. }
        | PlayTarget::TargetObj { .. }
        | PlayTarget::LaneWide { .. }
        | PlayTarget::Instant => None,
    }
}

fn update_board_for_spawn(
    world: &mut World,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
    card_type: CardType,
    entity: Entity,
) {
    if world.contains_resource::<BoardGrid>() {
        let mut grid = world.resource_mut::<BoardGrid>();
        if let Some((lane_index, cell_index)) = grid_indices(lane, cell) {
            grid.lanes[lane_index][cell_index] = Some(BoardCell::new(entity));
        }
    }

    if world.contains_resource::<BoardOccupancy>() {
        let mut occupancy = world.resource_mut::<BoardOccupancy>();
        match card_type {
            CardType::Minion => {
                occupancy.minion_slots.insert((player, lane), entity);
            }
            CardType::Trap => {
                occupancy.traps.insert((player, lane, cell), entity);
            }
            CardType::Structure => {
                occupancy.structures.insert((player, lane, cell), entity);
            }
            CardType::Field => {
                occupancy.fields.insert((player, lane), entity);
            }
            CardType::Spell | CardType::Order | CardType::DoubleFace => {}
        }
    }
}

fn appearance_units_in_order(world: &World, spawned: &[Entity]) -> Vec<Entity> {
    let mut units = spawned
        .iter()
        .copied()
        .filter(|unit| unit_has_simple_keyword(*unit, SimpleKeyword::Appearance, world))
        .collect::<Vec<_>>();

    units.sort_by_key(|unit| {
        let position = world.get::<BoardPosition>(*unit).copied();
        (
            position.map_or(u8::MAX, |pos| pos.lane),
            position.map_or(u8::MAX, |pos| pos.cell),
            unit.index(),
        )
    });
    units
}

fn appearance_effects_for(unit: Entity, world: &World) -> Vec<AppearanceEffect> {
    let Some(card_id) = world.get::<UnitCardRef>(unit).map(|card| card.0) else {
        return Vec::new();
    };

    world
        .get_resource::<AppearanceEffectRegistry>()
        .and_then(|registry| registry.effects.get(&card_id))
        .cloned()
        .unwrap_or_default()
}

fn apply_appearance_effect(
    world: &mut World,
    source: Entity,
    effect: AppearanceEffect,
    defeated_by_appearance: &mut Vec<Entity>,
    queued_lane_changes: &mut Vec<(Entity, i8)>,
) {
    match effect {
        AppearanceEffect::Damage { target, amount } => {
            let Some(target) = resolve_appearance_target(world, source, target) else {
                return;
            };
            let Some(mut stats) = world.get_mut::<UnitStats>(target) else {
                return;
            };
            stats.hp = stats.hp.saturating_sub(amount);
            let hp_after = stats.hp;
            drop(stats);

            world
                .resource_mut::<CombatResolutionTrace>()
                .push(CombatTraceEntry::UnitDamaged {
                    source,
                    target,
                    amount,
                    hp_after,
                    sub_step: 1,
                });

            if hp_after == 0 && !defeated_by_appearance.contains(&target) {
                defeated_by_appearance.push(target);
            }
        }
        AppearanceEffect::Stun { target } => {
            if let Some(target) = resolve_appearance_target(world, source, target) {
                let source_id = Some(source.to_bits() as EntityId);
                apply_stun(target, source_id, 1, world);
            }
        }
        AppearanceEffect::ChangeLane { delta } => queued_lane_changes.push((source, delta)),
    }
}

fn fire_deferred_death_triggers(
    world: &mut World,
    budget: &mut IterationBudget,
    defeated_by_appearance: &mut Vec<Entity>,
) -> Result<(), CombatAbort> {
    defeated_by_appearance.sort_by_key(|unit| {
        let position = world.get::<BoardPosition>(*unit).copied();
        (
            position.map_or(u8::MAX, |pos| pos.lane),
            position.map_or(u8::MAX, |pos| pos.cell),
            unit.index(),
        )
    });
    defeated_by_appearance.dedup();

    for unit in defeated_by_appearance.iter().copied() {
        if !unit_has_simple_keyword(unit, SimpleKeyword::Death, world) {
            continue;
        }
        budget.tick()?;
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit,
                keyword: KeywordKind::Death,
                sub_step: 1,
            });
    }

    Ok(())
}

fn apply_queued_lane_changes(world: &mut World, mut queued_lane_changes: Vec<(Entity, i8)>) {
    queued_lane_changes.sort_by_key(|(unit, _)| {
        let position = world.get::<BoardPosition>(*unit).copied();
        (
            position.map_or(u8::MAX, |pos| pos.lane),
            position.map_or(u8::MAX, |pos| pos.cell),
            unit.index(),
        )
    });

    for (unit, delta) in queued_lane_changes {
        let Some(position) = world.get::<BoardPosition>(unit).copied() else {
            continue;
        };
        let to_lane = shifted_lane(position.lane, delta, board_lane_count(world));
        if to_lane == position.lane {
            continue;
        }

        if let Some(mut board_position) = world.get_mut::<BoardPosition>(unit) {
            board_position.lane = to_lane;
        }
        update_board_for_lane_change(world, unit, position.lane, to_lane, position.cell);
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::UnitChangedLane {
                unit,
                from_lane: position.lane,
                to_lane,
                sub_step: 1,
            });
    }
}

fn update_board_for_lane_change(
    world: &mut World,
    unit: Entity,
    from_lane: LaneId,
    to_lane: LaneId,
    cell: u8,
) {
    if world.contains_resource::<BoardGrid>() {
        let mut grid = world.resource_mut::<BoardGrid>();
        if let Some((from_lane_index, cell_index)) = grid_indices(from_lane, cell) {
            if grid.lanes[from_lane_index][cell_index].map(|cell| cell.entity) == Some(unit) {
                grid.lanes[from_lane_index][cell_index] = None;
            }
        }
        if let Some((to_lane_index, cell_index)) = grid_indices(to_lane, cell) {
            grid.lanes[to_lane_index][cell_index] = Some(BoardCell::new(unit));
        }
    }

    if world.contains_resource::<BoardOccupancy>() {
        let owner = world.get::<UnitOwner>(unit).map(|owner| owner.0);
        if let Some(owner) = owner {
            let mut occupancy = world.resource_mut::<BoardOccupancy>();
            if occupancy.minion_slots.remove(&(owner, from_lane)) == Some(unit) {
                occupancy.minion_slots.insert((owner, to_lane), unit);
            }
        }
    }
}

fn resolve_appearance_target(
    world: &mut World,
    source: Entity,
    target: AppearanceTarget,
) -> Option<Entity> {
    match target {
        AppearanceTarget::SelfUnit => Some(source),
        AppearanceTarget::FirstEnemyInLane => first_enemy_in_lane(world, source),
        AppearanceTarget::UnitAt { lane, cell } => unit_at(world, lane, cell),
    }
}

fn first_enemy_in_lane(world: &mut World, source: Entity) -> Option<Entity> {
    let source_owner = world.get::<UnitOwner>(source)?.0;
    let source_lane = world.get::<BoardPosition>(source)?.lane;
    let mut query = world.query::<(Entity, &BoardPosition, &UnitOwner)>();
    let mut candidates = query
        .iter(world)
        .filter(|(entity, position, owner)| {
            *entity != source && position.lane == source_lane && owner.0 != source_owner
        })
        .map(|(entity, position, _)| (entity, position.cell))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(entity, cell)| (*cell, entity.index()));
    candidates.first().map(|(entity, _)| *entity)
}

fn unit_at(world: &mut World, lane: LaneId, cell: u8) -> Option<Entity> {
    let mut query = world.query::<(Entity, &BoardPosition)>();
    query
        .iter(world)
        .filter(|(_, position)| position.lane == lane && position.cell == cell)
        .map(|(entity, _)| entity)
        .min_by_key(|entity| entity.index())
}

fn card_for(card_id: CardId, world: &World) -> Option<&CardData> {
    world.get_resource::<CardCatalog>()?.cards.get(&card_id)
}

fn unit_has_simple_keyword(unit: Entity, keyword: SimpleKeyword, world: &World) -> bool {
    let Some(card_id) = world.get::<UnitCardRef>(unit).map(|card| card.0) else {
        return false;
    };

    card_for(card_id, world).is_some_and(|card| {
        card.keywords
            .iter()
            .any(|candidate| matches!(candidate, Keyword::Simple(simple) if *simple == keyword))
    })
}

fn shifted_lane(lane: LaneId, delta: i8, lane_count: LaneId) -> LaneId {
    (lane as i16 + delta as i16).clamp(1, lane_count as i16) as LaneId
}

fn board_lane_count(world: &World) -> LaneId {
    world
        .get_resource::<BoardConfig>()
        .map_or(BoardConfig::default().lane_count, |config| {
            config.lane_count
        })
}

fn grid_indices(lane: LaneId, cell: u8) -> Option<(usize, usize)> {
    let config = BoardConfig::default();
    if lane < 1 || lane > config.lane_count || cell < config.cell_min || cell > config.cell_max {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}

fn clear_pending_placements(world: &mut World) {
    world
        .resource_mut::<PendingPlacements>()
        .submissions
        .clear();
}

fn run_objective_detection_if_ready(world: &mut World) {
    if !world.contains_resource::<BoardConfig>()
        || !world.contains_resource::<SessionConfig>()
        || !world.contains_resource::<Messages<UnitAtObjective>>()
    {
        return;
    }

    world
        .run_system_once(detect_objective_presence)
        .expect("objective detection should run during combat sub-step 6");
}

fn request_draw_game_over(world: &mut World) {
    let request =
        PhaseAdvanceRequest::game_over(RoundPhase::Resolution, GameOverReason::Draw, None);

    if world.contains_resource::<PendingPhaseAdvance>() {
        world.resource_mut::<PendingPhaseAdvance>().request(request);
    } else {
        world.insert_resource(request);
    }
}
