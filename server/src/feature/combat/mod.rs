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
    advance_direction, apply_f1, detect_objective_presence, BoardCell, BoardConfig, BoardGrid,
    BoardOccupancy, BoardSystemSet, LaneId, PendingPlacements, UnitAtObjective,
};
use crate::feature::keyword::components::UnitKeywordState;
use crate::feature::keyword::effects::{
    apply_stun, can_execute_standard_movement, charge_x_cells_for_sub_step,
};
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
    UnitMoved {
        unit: Entity,
        from_cell: u8,
        to_cell: u8,
        sub_step: u8,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementUnit {
    entity: Entity,
    team: u8,
    lane: LaneId,
    original_cell: u8,
    cell: u8,
    destination: u8,
    direction: i16,
    halted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementProposal {
    unit_index: usize,
    from_cell: u8,
    to_cell: u8,
    wall_halt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoardUnitInfo {
    entity: Entity,
    team: Option<u8>,
    lane: LaneId,
    cell: u8,
    is_wall: bool,
    range_max: Option<u8>,
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
    if run_sub_step_scaffold(world, iteration_limit, begin_resolution.round).is_err() {
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

fn run_sub_step_scaffold(
    world: &mut World,
    iteration_limit: u32,
    current_round: u32,
) -> Result<(), CombatAbort> {
    let mut budget = IterationBudget::new(iteration_limit);

    for sub_step in 1..=6 {
        budget.tick()?;
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::SubStepStarted(sub_step));

        match sub_step {
            1 => apply_placements(world, &mut budget)?,
            2 => execute_charge_x(world, current_round, &mut budget)?,
            5 => execute_standard_movement(world, current_round, &mut budget)?,
            6 => run_objective_detection_if_ready(world),
            _ => {}
        }
    }

    Ok(())
}

fn execute_charge_x(
    world: &mut World,
    current_round: u32,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    execute_movement(world, current_round, true, budget)
}

fn execute_standard_movement(
    world: &mut World,
    current_round: u32,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    execute_movement(world, current_round, false, budget)
}

fn execute_movement(
    world: &mut World,
    current_round: u32,
    charge_x_mode: bool,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    let board_units = collect_board_units(world);
    let mut units = collect_movement_units(world, &board_units, current_round, charge_x_mode);

    loop {
        let proposals = movement_proposals(&board_units, &units);
        if proposals.is_empty() {
            break;
        }

        for _ in &proposals {
            budget.tick()?;
        }

        let crossing_halts = path_crossing_halts(&units, &proposals);
        let same_cell_halts = same_cell_landing_halts(&units, &proposals);
        let mut any_moved = false;

        for proposal in proposals {
            let unit = &mut units[proposal.unit_index];
            if crossing_halts.contains(&proposal.unit_index) {
                unit.halted = true;
                continue;
            }

            unit.cell = proposal.to_cell;
            unit.halted = proposal.wall_halt || same_cell_halts.contains(&proposal.unit_index);
            any_moved |= proposal.from_cell != proposal.to_cell;
        }

        if !any_moved {
            break;
        }
    }

    commit_movement_results(world, &units, movement_sub_step(charge_x_mode));

    Ok(())
}

fn collect_movement_units(
    world: &mut World,
    board_units: &[BoardUnitInfo],
    current_round: u32,
    charge_x_mode: bool,
) -> Vec<MovementUnit> {
    let Some(board_config) = world.get_resource::<BoardConfig>().copied() else {
        return Vec::new();
    };
    let Some(session_config) = world.get_resource::<SessionConfig>().cloned() else {
        return Vec::new();
    };

    let raw_units = {
        let mut query = world.query::<(Entity, &BoardPosition, &UnitStats, &UnitOwner)>();
        query
            .iter(world)
            .map(|(entity, position, stats, owner)| (entity, *position, *stats, *owner))
            .collect::<Vec<_>>()
    };

    let mut units = raw_units
        .into_iter()
        .filter_map(|(entity, position, stats, owner)| {
            let move_value = if charge_x_mode {
                charge_x_cells_for_sub_step(entity, current_round, world)?
            } else {
                if !can_execute_standard_movement(entity, current_round, world) || stats.mp == 0 {
                    return None;
                }
                stats.mp
            };

            if move_value == 0
                || (!charge_x_mode
                    && range_wall_target_in_range(
                        entity,
                        owner.0,
                        position.lane,
                        position.cell,
                        board_units,
                        &session_config,
                        &board_config,
                    ))
            {
                return None;
            }

            let direction = advance_direction(owner.0, &session_config, &board_config)?;
            let team = session_config.team_map.get(&owner.0).copied()?;
            let destination = apply_f1(
                position.cell,
                direction,
                move_value,
                board_config.cell_min,
                board_config.cell_max,
            );

            (destination != position.cell).then_some(MovementUnit {
                entity,
                team,
                lane: position.lane,
                original_cell: position.cell,
                cell: position.cell,
                destination,
                direction,
                halted: false,
            })
        })
        .collect::<Vec<_>>();

    units.sort_by_key(|unit| (unit.lane, unit.original_cell, unit.entity.index()));
    units
}

fn collect_board_units(world: &mut World) -> Vec<BoardUnitInfo> {
    let teams = world
        .get_resource::<SessionConfig>()
        .map(|session| session.team_map.clone())
        .unwrap_or_default();
    let raw_units = {
        let mut query = world.query::<(Entity, &BoardPosition, &UnitOwner)>();
        query
            .iter(world)
            .map(|(entity, position, owner)| (entity, *position, *owner))
            .collect::<Vec<_>>()
    };

    raw_units
        .into_iter()
        .map(|(entity, position, owner)| BoardUnitInfo {
            entity,
            team: teams.get(&owner.0).copied(),
            lane: position.lane,
            cell: position.cell,
            is_wall: unit_has_simple_keyword(entity, SimpleKeyword::Wall, world),
            range_max: range_max(entity, world),
        })
        .collect()
}

fn movement_proposals(
    board_units: &[BoardUnitInfo],
    units: &[MovementUnit],
) -> Vec<MovementProposal> {
    units
        .iter()
        .enumerate()
        .filter_map(|(unit_index, unit)| {
            if unit.halted || unit.cell == unit.destination {
                return None;
            }

            let to_cell = (unit.cell as i16 + unit.direction).clamp(1, 8) as u8;
            Some(MovementProposal {
                unit_index,
                from_cell: unit.cell,
                to_cell,
                wall_halt: enemy_wall_at(board_units, unit.team, unit.lane, to_cell),
            })
        })
        .collect()
}

fn path_crossing_halts(units: &[MovementUnit], proposals: &[MovementProposal]) -> Vec<usize> {
    let mut halted = Vec::new();

    for first in proposals {
        for second in proposals {
            if first.unit_index >= second.unit_index {
                continue;
            }

            let first_unit = units[first.unit_index];
            let second_unit = units[second.unit_index];
            if first_unit.lane != second_unit.lane
                || !teams_are_enemies(first_unit.team, second_unit.team)
            {
                continue;
            }

            if first.to_cell == second.from_cell && second.to_cell == first.from_cell {
                halted.push(first.unit_index);
                halted.push(second.unit_index);
            }
        }
    }

    halted.sort_unstable();
    halted.dedup();
    halted
}

fn same_cell_landing_halts(units: &[MovementUnit], proposals: &[MovementProposal]) -> Vec<usize> {
    let mut halted = Vec::new();

    for first in proposals {
        for second in proposals {
            if first.unit_index >= second.unit_index || first.to_cell != second.to_cell {
                continue;
            }

            let first_unit = units[first.unit_index];
            let second_unit = units[second.unit_index];
            if first_unit.lane == second_unit.lane
                && teams_are_enemies(first_unit.team, second_unit.team)
            {
                halted.push(first.unit_index);
                halted.push(second.unit_index);
            }
        }
    }

    halted.sort_unstable();
    halted.dedup();
    halted
}

fn commit_movement_results(world: &mut World, units: &[MovementUnit], sub_step: u8) {
    for unit in units {
        if unit.cell == unit.original_cell {
            continue;
        }

        if let Some(mut position) = world.get_mut::<BoardPosition>(unit.entity) {
            position.cell = unit.cell;
        }

        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::UnitMoved {
                unit: unit.entity,
                from_cell: unit.original_cell,
                to_cell: unit.cell,
                sub_step,
            });
    }

    rebuild_board_grid(world);
}

fn rebuild_board_grid(world: &mut World) {
    if !world.contains_resource::<BoardGrid>() {
        return;
    }

    let mut query = world.query::<(Entity, &BoardPosition)>();
    let positions = query
        .iter(world)
        .map(|(entity, position)| (entity, *position))
        .collect::<Vec<_>>();

    let mut grid = world.resource_mut::<BoardGrid>();
    *grid = BoardGrid::default();
    for (entity, position) in positions {
        if let Some((lane_index, cell_index)) = grid_indices(position.lane, position.cell) {
            grid.lanes[lane_index][cell_index] = Some(BoardCell::new(entity));
        }
    }
}

fn movement_sub_step(charge_x_mode: bool) -> u8 {
    if charge_x_mode {
        2
    } else {
        5
    }
}

fn range_wall_target_in_range(
    unit: Entity,
    owner: PlayerId,
    lane: LaneId,
    cell: u8,
    board_units: &[BoardUnitInfo],
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> bool {
    let Some(max_range) = board_units
        .iter()
        .find(|candidate| candidate.entity == unit)
        .and_then(|candidate| candidate.range_max)
    else {
        return false;
    };
    let Some(direction) = advance_direction(owner, session_config, board_config) else {
        return false;
    };
    let Some(team) = session_config.team_map.get(&owner).copied() else {
        return false;
    };

    board_units.iter().any(|candidate| {
        candidate.entity != unit
            && candidate.lane == lane
            && candidate
                .team
                .is_some_and(|candidate_team| teams_are_enemies(team, candidate_team))
            && candidate.is_wall
            && forward_distance(cell, candidate.cell, direction)
                .is_some_and(|distance| distance <= max_range)
    })
}

fn enemy_wall_at(board_units: &[BoardUnitInfo], team: u8, lane: LaneId, cell: u8) -> bool {
    board_units.iter().any(|candidate| {
        candidate.lane == lane
            && candidate.cell == cell
            && candidate
                .team
                .is_some_and(|candidate_team| teams_are_enemies(team, candidate_team))
            && candidate.is_wall
    })
}

fn teams_are_enemies(first: u8, second: u8) -> bool {
    first != second
}

fn range_max(unit: Entity, world: &World) -> Option<u8> {
    let card_id = world.get::<UnitCardRef>(unit).map(|card| card.0)?;
    card_for(card_id, world)?
        .keywords
        .iter()
        .find_map(|keyword| {
            if let Keyword::RangeX { max_range } = keyword {
                Some(*max_range)
            } else {
                None
            }
        })
}

fn forward_distance(from_cell: u8, target_cell: u8, direction: i16) -> Option<u8> {
    let distance = (target_cell as i16 - from_cell as i16) * direction.signum();
    (distance > 0).then_some(distance as u8)
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
