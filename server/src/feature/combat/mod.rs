#![allow(dead_code)]

pub mod modifier_stack;

use std::collections::HashMap;

use bevy::ecs::message::MessageCursor;
use bevy::ecs::system::{RunSystemOnce, SystemState};
use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Server, ServerMultiMessageSender};
use shared::card::{CardData, CardId, CardType, Keyword, SimpleKeyword};
use shared::config::GameConfig as SharedGameConfig;
use shared::keyword::KeywordKind;
use shared::protocol::{
    EntityId, GameOverReason, GoldReason, PlayTarget, ReliableChannel, ResolutionEvent,
    S2CPlacementReveal, S2CResolutionEvent, TaggedEvent,
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
    advance_direction, apply_f1, detect_objective_presence, is_at_objective, AcceptedPlacement,
    BoardCell, BoardConfig, BoardGrid, BoardOccupancy, BoardSystemSet, LaneId, PendingPlacements,
    UnitAtObjective,
};
use crate::feature::keyword::components::UnitKeywordState;
use crate::feature::keyword::effects::{
    apply_stun, can_execute_first_strike, can_execute_standard_attack,
    can_execute_standard_movement, charge_x_cells_for_sub_step, consume_shield_for_sub_step,
};
use crate::feature::keyword::state_eval::{
    eval_injured_bonuses_at_boundary, eval_outnumbered_for_sub_step, snapshot_leader_bonuses,
};
use crate::feature::keyword::{ChainDeathBuffer, UnitDied};
use crate::feature::objective::{
    take_damage as apply_objective_damage, ObjectiveDestroyed, ObjectiveHp, ObjectiveSlot,
    PendingObjectiveEvents,
};
use crate::foundation::config::{CardCatalog, GameConfig as ServerGameConfig};
use crate::foundation::rng::ServerRng;

use self::modifier_stack::{apply_combat_modifier_stack, UnitSnapshot};

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
            .init_resource::<CombatKillLog>()
            .init_resource::<ChainDeathBuffer>()
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
        player: PlayerId,
        lane: LaneId,
        cell: u8,
    },
    UnitRemoved {
        unit: Entity,
        lane: LaneId,
        cell: u8,
    },
    UnitMoved {
        unit: Entity,
        lane: LaneId,
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
    CombatDamage {
        attacker: Entity,
        defender: Entity,
        damage_amount: u8,
        hp_after: u8,
        was_blocked_by_shield: bool,
        sub_step: u8,
    },
    UnitChangedLane {
        unit: Entity,
        from_lane: LaneId,
        to_lane: LaneId,
        sub_step: u8,
    },
    GoldAwarded {
        player: PlayerId,
        amount: u32,
        reason: GoldAwardReason,
    },
    ObjectiveDamaged {
        target_player_id: PlayerId,
        lane: LaneId,
        hp_before: u32,
        hp_after: u32,
        attacker_id: Option<Entity>,
    },
    ObjectiveDestroyed {
        target_player_id: PlayerId,
        lane: LaneId,
        was_fake: bool,
    },
    SubStepStarted(u8),
    IterationBudgetExceeded,
    ResolutionEventEnqueued,
    ResolutionCompleteQueued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoldAwardReason {
    Kill,
    ObjectiveReward,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CombatKillLog {
    records: Vec<KillRecord>,
}

impl CombatKillLog {
    pub fn push(&mut self, record: KillRecord) {
        self.records.push(record);
    }

    pub fn records(&self) -> &[KillRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn drain_for_sub_step(&mut self, sub_step: u8) -> Vec<KillRecord> {
        let mut drained = Vec::new();
        self.records.retain(|record| {
            if record.lethal_sub_step == sub_step {
                drained.push(*record);
                false
            } else {
                true
            }
        });
        drained
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KillRecord {
    pub killer: Entity,
    pub victim: Entity,
    pub killer_player_id: PlayerId,
    pub lethal_sub_step: u8,
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

#[derive(Debug, Clone)]
struct CombatUnit {
    entity: Entity,
    snapshot: UnitSnapshot,
    range_max: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CombatAttack {
    attacker: Entity,
    defender: Entity,
    attacker_lane: LaneId,
    attacker_player: PlayerId,
    damage_amount: u8,
    melee_contact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectiveAttack {
    attacker: Entity,
    attacker_player: PlayerId,
    target_player: PlayerId,
    lane: LaneId,
    cell: u8,
    amount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatAttackPhase {
    FirstStrike,
    Standard,
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

    let trace_start = world.resource::<CombatResolutionTrace>().entries().len();
    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::BeginResolutionRead {
            round: begin_resolution.round,
        });
    world.resource_mut::<CombatKillLog>().clear();

    let iteration_limit = world.resource::<CombatIterationBudget>().limit();
    if run_sub_step_scaffold(world, iteration_limit, begin_resolution.round).is_err() {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::IterationBudgetExceeded);
        request_draw_game_over(world);
        return;
    }

    enqueue_resolution_event(world, begin_resolution.round, trace_start);
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

fn enqueue_resolution_event(world: &mut World, round: u32, trace_start: usize) {
    let events = build_resolution_events(
        &world.resource::<CombatResolutionTrace>().entries()[trace_start..],
    );
    let message = S2CResolutionEvent { round, events };

    world
        .resource_mut::<CombatNetworkOutbox>()
        .push_resolution_event(message.clone());
    broadcast_resolution_event(world, &message);
    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::ResolutionEventEnqueued);
}

fn build_resolution_events(trace: &[CombatTraceEntry]) -> Vec<TaggedEvent> {
    let mut events = Vec::new();
    let mut current_sub_step = 0;

    for entry in trace.iter().copied() {
        if let CombatTraceEntry::SubStepStarted(sub_step) = entry {
            current_sub_step = sub_step;
        }

        let Some((sub_step, event)) = resolution_event_from_trace(entry, current_sub_step) else {
            continue;
        };

        events.push(TaggedEvent {
            sub_step,
            trigger_index: events.len() as u32,
            event,
        });
    }

    events
}

fn resolution_event_from_trace(
    entry: CombatTraceEntry,
    current_sub_step: u8,
) -> Option<(u8, ResolutionEvent)> {
    match entry {
        CombatTraceEntry::SubStepStarted(sub_step) => {
            Some((sub_step, ResolutionEvent::SubStepBegin))
        }
        CombatTraceEntry::UnitPlaced {
            entity,
            player,
            lane,
            cell,
        } => Some((
            current_sub_step,
            ResolutionEvent::UnitPlaced {
                unit_id: entity_id(entity),
                player,
                lane,
                cell,
            },
        )),
        CombatTraceEntry::UnitMoved {
            unit,
            lane,
            from_cell,
            to_cell,
            sub_step,
        } => Some((
            sub_step,
            ResolutionEvent::UnitMoved {
                unit_id: entity_id(unit),
                lane,
                from_cell,
                to_cell,
            },
        )),
        CombatTraceEntry::UnitChangedLane {
            unit,
            from_lane,
            to_lane,
            sub_step,
        } => Some((
            sub_step,
            ResolutionEvent::UnitChangedLane {
                unit_id: entity_id(unit),
                from_lane,
                to_lane,
            },
        )),
        CombatTraceEntry::UnitDamaged {
            source,
            target,
            amount,
            hp_after,
            sub_step,
        } => Some((
            sub_step,
            ResolutionEvent::CombatDamage {
                attacker_id: entity_id(source),
                defender_id: entity_id(target),
                damage_amount: amount,
                defender_hp_after: hp_after,
                was_blocked_by_shield: false,
            },
        )),
        CombatTraceEntry::CombatDamage {
            attacker,
            defender,
            damage_amount,
            hp_after,
            was_blocked_by_shield,
            sub_step,
        } => Some((
            sub_step,
            ResolutionEvent::CombatDamage {
                attacker_id: entity_id(attacker),
                defender_id: entity_id(defender),
                damage_amount,
                defender_hp_after: hp_after,
                was_blocked_by_shield,
            },
        )),
        CombatTraceEntry::UnitRemoved { unit, lane, cell } => Some((
            current_sub_step,
            ResolutionEvent::UnitRemoved {
                unit_id: entity_id(unit),
                lane,
                cell,
            },
        )),
        CombatTraceEntry::KeywordTriggered {
            unit,
            keyword,
            sub_step,
        } => Some((
            sub_step,
            ResolutionEvent::KeywordTriggered {
                unit_id: entity_id(unit),
                keyword,
            },
        )),
        CombatTraceEntry::GoldAwarded {
            player,
            amount,
            reason,
        } => Some((
            current_sub_step,
            ResolutionEvent::GoldAwarded {
                player,
                amount,
                reason: gold_reason(reason),
            },
        )),
        CombatTraceEntry::ObjectiveDamaged {
            target_player_id,
            lane,
            hp_before,
            hp_after,
            attacker_id,
        } => Some((
            current_sub_step,
            ResolutionEvent::ObjectiveDamage {
                attacker_id: attacker_id.map(entity_id),
                target_player_id,
                lane,
                damage_amount: hp_before.saturating_sub(hp_after),
                objective_hp_after: hp_after,
            },
        )),
        CombatTraceEntry::ObjectiveDestroyed {
            target_player_id,
            lane,
            was_fake,
        } => Some((
            current_sub_step,
            ResolutionEvent::ObjectiveDestroyed {
                target_player_id,
                lane,
                was_fake,
            },
        )),
        _ => None,
    }
}

fn entity_id(entity: Entity) -> EntityId {
    entity.to_bits() as EntityId
}

const fn gold_reason(reason: GoldAwardReason) -> GoldReason {
    match reason {
        GoldAwardReason::Kill => GoldReason::Kill,
        GoldAwardReason::ObjectiveReward => GoldReason::ObjectiveDestroyed,
    }
}

fn broadcast_placement_reveal(world: &mut World, message: &S2CPlacementReveal) {
    let mut system_state: SystemState<(Query<&Server>, Option<ServerMultiMessageSender>)> =
        SystemState::new(world);
    let (server, mut sender) = system_state.get_mut(world);
    let (Ok(server), Some(sender)) = (server.single(), sender.as_mut()) else {
        return;
    };

    let _ =
        sender.send::<S2CPlacementReveal, ReliableChannel>(message, server, &NetworkTarget::All);
}

fn broadcast_resolution_event(world: &mut World, message: &S2CResolutionEvent) {
    let mut system_state: SystemState<(Query<&Server>, Option<ServerMultiMessageSender>)> =
        SystemState::new(world);
    let (server, mut sender) = system_state.get_mut(world);
    let (Ok(server), Some(sender)) = (server.single(), sender.as_mut()) else {
        return;
    };

    let _ =
        sender.send::<S2CResolutionEvent, ReliableChannel>(message, server, &NetworkTarget::All);
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
            3 => execute_first_strike(world, current_round, &mut budget)?,
            4 => remove_dead(world, &mut budget)?,
            5 => execute_standard_movement(world, current_round, &mut budget)?,
            6 => {
                execute_standard_combat(world, current_round, &mut budget)?;
                execute_objective_damage(world, &mut budget)?;
            }
            _ => {}
        }

        if sub_step == 1 {
            snapshot_leader_state(world, current_round);
        }
        if let Some(next_sub_step) = boundary_sub_step(sub_step) {
            evaluate_persistent_state_boundary(world, current_round, next_sub_step);
        }
    }

    Ok(())
}

fn boundary_sub_step(completed_sub_step: u8) -> Option<u8> {
    match completed_sub_step {
        1 => Some(2),
        2 => Some(3),
        3 => Some(4),
        4 => Some(5),
        5 | 6 => Some(6),
        _ => None,
    }
}

fn snapshot_leader_state(world: &mut World, current_round: u32) {
    for leader in snapshot_leader_bonuses(world, current_round) {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit: leader,
                keyword: KeywordKind::Leader,
                sub_step: 1,
            });
    }
}

fn evaluate_persistent_state_boundary(world: &mut World, current_round: u32, next_sub_step: u8) {
    for unit in eval_outnumbered_for_sub_step(world, next_sub_step) {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit,
                keyword: KeywordKind::Outnumbered,
                sub_step: next_sub_step,
            });
    }

    for unit in eval_injured_bonuses_at_boundary(world, current_round, next_sub_step) {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit,
                keyword: KeywordKind::Injured,
                sub_step: next_sub_step,
            });
    }
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

fn execute_first_strike(
    world: &mut World,
    current_round: u32,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    let mut attacks = collect_first_strike_attacks(world, current_round);
    apply_combat_attacks(world, &mut attacks, 3, budget)
}

fn execute_standard_combat(
    world: &mut World,
    current_round: u32,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    let mut attacks = collect_standard_combat_attacks(world, current_round);
    apply_combat_attacks(world, &mut attacks, 6, budget)?;
    remove_dead_for_sub_step(world, budget, 6, 6)
}

fn collect_first_strike_attacks(world: &mut World, current_round: u32) -> Vec<CombatAttack> {
    collect_combat_attacks(world, CombatAttackPhase::FirstStrike, |unit, world| {
        unit.snapshot.hp > 0 && can_execute_first_strike(unit.entity, current_round, world)
    })
}

fn collect_standard_combat_attacks(world: &mut World, current_round: u32) -> Vec<CombatAttack> {
    collect_combat_attacks(world, CombatAttackPhase::Standard, |unit, world| {
        unit.snapshot.hp > 0 && can_execute_standard_attack(unit.entity, current_round, world)
    })
}

fn collect_combat_attacks(
    world: &mut World,
    phase: CombatAttackPhase,
    can_attack: impl Fn(&CombatUnit, &World) -> bool,
) -> Vec<CombatAttack> {
    let Some(session_config) = world.get_resource::<SessionConfig>().cloned() else {
        return Vec::new();
    };
    let Some(board_config) = world.get_resource::<BoardConfig>().copied() else {
        return Vec::new();
    };
    let config = combat_config(world);
    let units = collect_combat_units(world);
    let mut attackers = units
        .iter()
        .filter(|unit| can_attack(unit, world))
        .collect::<Vec<_>>();
    attackers.sort_by_key(|unit| {
        (
            unit.snapshot.player.0,
            unit.snapshot.lane,
            unit.snapshot.cell,
            unit.snapshot.unit_id,
        )
    });

    let mut rng = world.get_resource_mut::<ServerRng>();
    attackers
        .into_iter()
        .filter_map(|attacker| {
            let defender = select_combat_target(
                attacker,
                &units,
                &session_config,
                &board_config,
                phase,
                rng.as_mut().map(|rng| &mut **rng),
            )?;
            let result =
                apply_combat_modifier_stack(&attacker.snapshot, &defender.snapshot, &config);
            Some(CombatAttack {
                attacker: attacker.entity,
                defender: defender.entity,
                attacker_lane: attacker.snapshot.lane,
                attacker_player: attacker.snapshot.player,
                damage_amount: result.net_damage,
                melee_contact: attacker.range_max.is_none()
                    && attacker.snapshot.lane == defender.snapshot.lane
                    && attacker.snapshot.cell.abs_diff(defender.snapshot.cell) <= 1,
            })
        })
        .collect()
}

fn apply_combat_attacks(
    world: &mut World,
    attacks: &mut [CombatAttack],
    sub_step: u8,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    attacks.sort_by_key(|attack| {
        (
            attack.defender.index(),
            attack.attacker_lane,
            attack.attacker.index(),
        )
    });

    let mut index = 0;
    while index < attacks.len() {
        let defender = attacks[index].defender;
        let start = index;
        while index < attacks.len() && attacks[index].defender == defender {
            budget.tick()?;
            index += 1;
        }

        apply_combat_attack_group(world, &attacks[start..index], sub_step, budget)?;
    }

    Ok(())
}

fn apply_combat_attack_group(
    world: &mut World,
    attacks: &[CombatAttack],
    sub_step: u8,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    let Some(first_attack) = attacks.first().copied() else {
        return Ok(());
    };

    if consume_shield_for_sub_step(first_attack.defender, sub_step, world) {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit: first_attack.defender,
                keyword: KeywordKind::Shield,
                sub_step,
            });
        let hp_after = world
            .get::<UnitStats>(first_attack.defender)
            .map_or(0, |stats| stats.hp);
        for attack in attacks.iter().copied() {
            world
                .resource_mut::<CombatResolutionTrace>()
                .push(CombatTraceEntry::CombatDamage {
                    attacker: attack.attacker,
                    defender: attack.defender,
                    damage_amount: 0,
                    hp_after,
                    was_blocked_by_shield: true,
                    sub_step,
                });
        }
    } else {
        for attack in attacks.iter().copied() {
            apply_combat_attack(world, attack, sub_step);
        }
    }

    apply_counterattacks(world, first_attack.defender, attacks, sub_step, budget)
}

fn apply_combat_attack(world: &mut World, attack: CombatAttack, sub_step: u8) {
    apply_damage_to_defender(
        world,
        attack.attacker,
        attack.defender,
        attack.attacker_player,
        attack.damage_amount,
        sub_step,
        false,
    );
}

fn apply_damage_to_defender(
    world: &mut World,
    attacker: Entity,
    defender: Entity,
    attacker_player: PlayerId,
    damage_amount: u8,
    sub_step: u8,
    was_blocked_by_shield: bool,
) {
    let Some(mut stats) = world.get_mut::<UnitStats>(defender) else {
        return;
    };
    let hp_before = stats.hp;
    let hp_after = if was_blocked_by_shield {
        hp_before
    } else {
        hp_before.saturating_sub(damage_amount)
    };
    stats.hp = hp_after;
    drop(stats);

    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::CombatDamage {
            attacker,
            defender,
            damage_amount,
            hp_after,
            was_blocked_by_shield,
            sub_step,
        });

    if hp_before > 0 && hp_after == 0 {
        world.resource_mut::<CombatKillLog>().push(KillRecord {
            killer: attacker,
            victim: defender,
            killer_player_id: attacker_player,
            lethal_sub_step: sub_step,
        });

        if unit_has_simple_keyword(attacker, SimpleKeyword::FinalBlow, world) {
            world.resource_mut::<CombatResolutionTrace>().push(
                CombatTraceEntry::KeywordTriggered {
                    unit: attacker,
                    keyword: KeywordKind::FinalBlow,
                    sub_step,
                },
            );
        }
    }
}

fn apply_counterattacks(
    world: &mut World,
    defender: Entity,
    attacks: &[CombatAttack],
    sub_step: u8,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    if !unit_has_simple_keyword(defender, SimpleKeyword::Counterattack, world) {
        return Ok(());
    }

    let melee_attacks = attacks
        .iter()
        .copied()
        .filter(|attack| attack.melee_contact)
        .collect::<Vec<_>>();
    if melee_attacks.is_empty() {
        return Ok(());
    }

    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::KeywordTriggered {
            unit: defender,
            keyword: KeywordKind::Counterattack,
            sub_step,
        });

    for attack in melee_attacks.iter().copied() {
        budget.tick()?;
        apply_counterattack_damage(world, defender, attack.attacker, sub_step);
    }

    for attack in melee_attacks.iter().copied() {
        if !unit_has_simple_keyword(attack.attacker, SimpleKeyword::Counterattack, world) {
            continue;
        }
        budget.tick()?;
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit: attack.attacker,
                keyword: KeywordKind::Counterattack,
                sub_step,
            });
        apply_counterattack_damage(world, attack.attacker, defender, sub_step);
    }

    Ok(())
}

fn apply_counterattack_damage(world: &mut World, attacker: Entity, defender: Entity, sub_step: u8) {
    let Some(attacker_snapshot) = snapshot_for_combat_entity(attacker, world) else {
        return;
    };
    let Some(defender_snapshot) = snapshot_for_combat_entity(defender, world) else {
        return;
    };
    let attacker_player = attacker_snapshot.player;

    if consume_shield_for_sub_step(defender, sub_step, world) {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit: defender,
                keyword: KeywordKind::Shield,
                sub_step,
            });
        apply_damage_to_defender(
            world,
            attacker,
            defender,
            attacker_player,
            0,
            sub_step,
            true,
        );
        return;
    }

    let damage_amount = apply_combat_modifier_stack(
        &attacker_snapshot,
        &defender_snapshot,
        &combat_config(world),
    )
    .net_damage;
    apply_damage_to_defender(
        world,
        attacker,
        defender,
        attacker_player,
        damage_amount,
        sub_step,
        false,
    );
}

fn remove_dead(world: &mut World, budget: &mut IterationBudget) -> Result<(), CombatAbort> {
    remove_dead_for_sub_step(world, budget, 4, 3)
}

fn remove_dead_for_sub_step(
    world: &mut World,
    budget: &mut IterationBudget,
    death_sub_step: u8,
    gold_sub_step: u8,
) -> Result<(), CombatAbort> {
    seed_chain_death_buffer(world);

    loop {
        let next = world.resource_mut::<ChainDeathBuffer>().0.pop_front();
        let Some((unit, attacker)) = next else {
            break;
        };

        if let Err(err) = budget.tick() {
            world.resource_mut::<ChainDeathBuffer>().0.clear();
            return Err(err);
        }

        remove_dead_unit(world, unit, attacker, death_sub_step);
    }

    world.resource_mut::<ChainDeathBuffer>().0.clear();
    drain_kill_gold(world, gold_sub_step);

    Ok(())
}

fn seed_chain_death_buffer(world: &mut World) {
    let initial_deaths = collect_dead_units_lane_ordered(world);
    let mut chain_death_buffer = world.resource_mut::<ChainDeathBuffer>();
    chain_death_buffer.0.clear();
    chain_death_buffer.0.extend(initial_deaths);
}

fn collect_dead_units_lane_ordered(world: &mut World) -> Vec<(Entity, Option<Entity>)> {
    let killers_by_victim = world
        .get_resource::<CombatKillLog>()
        .map(|kill_log| {
            kill_log
                .records()
                .iter()
                .map(|record| (record.victim, record.killer))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();

    let mut query = world.query::<(Entity, &BoardPosition, &UnitStats)>();
    let mut deaths = query
        .iter(world)
        .filter(|(_, _, stats)| stats.hp == 0)
        .map(|(unit, position, _)| {
            (
                unit,
                position.lane,
                position.cell,
                killers_by_victim.get(&unit).copied(),
            )
        })
        .collect::<Vec<_>>();

    deaths.sort_by_key(|(unit, lane, cell, _)| (*lane, *cell, unit.index()));
    deaths
        .into_iter()
        .map(|(unit, _, _, attacker)| (unit, attacker))
        .collect()
}

fn remove_dead_unit(world: &mut World, unit: Entity, attacker: Option<Entity>, sub_step: u8) {
    let Some(position) = world.get::<BoardPosition>(unit).copied() else {
        return;
    };
    let Some(stats) = world.get::<UnitStats>(unit).copied() else {
        return;
    };
    if stats.hp > 0 {
        return;
    }

    let triggers_death = unit_has_simple_keyword(unit, SimpleKeyword::Death, world);
    remove_unit_from_board_state(world, unit, position);

    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::UnitRemoved {
            unit,
            lane: position.lane,
            cell: position.cell,
        });

    if triggers_death {
        world.trigger(UnitDied {
            entity: unit,
            attacker,
        });
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::KeywordTriggered {
                unit,
                keyword: KeywordKind::Death,
                sub_step,
            });
    }

    let _ = world.despawn(unit);
}

fn remove_unit_from_board_state(world: &mut World, unit: Entity, position: BoardPosition) {
    if world.contains_resource::<BoardGrid>() {
        let mut grid = world.resource_mut::<BoardGrid>();
        if let Some((lane_index, cell_index)) = grid_indices(position.lane, position.cell) {
            if grid.lanes[lane_index][cell_index].map(|cell| cell.entity) == Some(unit) {
                grid.lanes[lane_index][cell_index] = None;
            }
        }
    }

    if world.contains_resource::<BoardOccupancy>() {
        let mut occupancy = world.resource_mut::<BoardOccupancy>();
        occupancy
            .minion_slots
            .retain(|_, occupant| *occupant != unit);
        occupancy.traps.retain(|_, occupant| *occupant != unit);
        occupancy.structures.retain(|_, occupant| *occupant != unit);
        occupancy.fields.retain(|_, occupant| *occupant != unit);
    }
}

fn drain_kill_gold(world: &mut World, sub_step: u8) {
    let records = world
        .get_resource_mut::<CombatKillLog>()
        .map(|mut kill_log| kill_log.drain_for_sub_step(sub_step))
        .unwrap_or_default();

    for record in records {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::GoldAwarded {
                player: record.killer_player_id,
                amount: 1,
                reason: GoldAwardReason::Kill,
            });
        award_kill_gold(world, record.killer_player_id);
    }
}

fn drain_ss3_kill_gold(world: &mut World) {
    drain_kill_gold(world, 3);
}

fn award_kill_gold(world: &mut World, player: PlayerId) {
    let Some(mut economies) = world.get_resource_mut::<PlayerEconomies>() else {
        return;
    };
    let Some(economy) = economies.0.get_mut(&player) else {
        return;
    };
    economy_api::apply_gold_award(economy, 1);
}

fn collect_combat_units(world: &mut World) -> Vec<CombatUnit> {
    let raw_units = {
        let mut query =
            world.query::<(Entity, &BoardPosition, &UnitStats, &UnitOwner, &UnitCardRef)>();
        query
            .iter(world)
            .map(|(entity, position, stats, owner, card_ref)| {
                (entity, *position, *stats, *owner, *card_ref)
            })
            .collect::<Vec<_>>()
    };

    raw_units
        .into_iter()
        .filter_map(|(entity, position, stats, owner, card_ref)| {
            let card = card_for(card_ref.0, world)?;
            let leader_atk_bonus = world
                .get::<UnitKeywordState>(entity)
                .map_or(0, |state| state.leader_bonus_atk);
            Some(CombatUnit {
                entity,
                snapshot: UnitSnapshot {
                    unit_id: entity.to_bits() as EntityId,
                    player: owner.0,
                    lane: position.lane,
                    cell: position.cell,
                    atk: stats.atk,
                    hp: stats.hp,
                    ar: stats.ar,
                    mp: stats.mp,
                    unit_type: card.unit_type,
                    keywords: card.keywords.clone(),
                    leader_atk_bonus,
                },
                range_max: card.keywords.iter().find_map(|keyword| {
                    if let Keyword::RangeX { max_range } = keyword {
                        Some(*max_range)
                    } else {
                        None
                    }
                }),
            })
        })
        .collect()
}

fn snapshot_for_combat_entity(entity: Entity, world: &World) -> Option<UnitSnapshot> {
    let position = world.get::<BoardPosition>(entity).copied()?;
    let stats = world.get::<UnitStats>(entity).copied()?;
    let owner = world.get::<UnitOwner>(entity).copied()?;
    let card_ref = world.get::<UnitCardRef>(entity).copied()?;
    let card = card_for(card_ref.0, world)?;
    let leader_atk_bonus = world
        .get::<UnitKeywordState>(entity)
        .map_or(0, |state| state.leader_bonus_atk);

    Some(UnitSnapshot {
        unit_id: entity.to_bits() as EntityId,
        player: owner.0,
        lane: position.lane,
        cell: position.cell,
        atk: stats.atk,
        hp: stats.hp,
        ar: stats.ar,
        mp: stats.mp,
        unit_type: card.unit_type,
        keywords: card.keywords.clone(),
        leader_atk_bonus,
    })
}

fn select_combat_target<'a>(
    attacker: &CombatUnit,
    units: &'a [CombatUnit],
    session_config: &SessionConfig,
    board_config: &BoardConfig,
    phase: CombatAttackPhase,
    rng: Option<&mut ServerRng>,
) -> Option<&'a CombatUnit> {
    if attacker.range_max.is_some() {
        select_range_target(attacker, units, session_config, board_config, rng)
    } else {
        select_melee_target(
            attacker,
            units,
            session_config,
            phase == CombatAttackPhase::Standard,
        )
    }
}

fn select_melee_target<'a>(
    attacker: &CombatUnit,
    units: &'a [CombatUnit],
    session_config: &SessionConfig,
    allow_adjacent_contact: bool,
) -> Option<&'a CombatUnit> {
    units
        .iter()
        .filter(|candidate| {
            candidate.entity != attacker.entity
                && candidate.snapshot.hp > 0
                && candidate.snapshot.lane == attacker.snapshot.lane
                && (candidate.snapshot.cell == attacker.snapshot.cell
                    || (allow_adjacent_contact
                        && candidate.snapshot.cell.abs_diff(attacker.snapshot.cell) == 1))
                && snapshots_are_enemies(&attacker.snapshot, &candidate.snapshot, session_config)
        })
        .min_by_key(|candidate| candidate.entity.index())
}

fn select_range_target<'a>(
    attacker: &CombatUnit,
    units: &'a [CombatUnit],
    session_config: &SessionConfig,
    board_config: &BoardConfig,
    rng: Option<&mut ServerRng>,
) -> Option<&'a CombatUnit> {
    let max_range = attacker.range_max?;
    let direction = advance_direction(attacker.snapshot.player, session_config, board_config)?;
    let mut candidates = units
        .iter()
        .filter_map(|candidate| {
            if candidate.entity == attacker.entity
                || candidate.snapshot.hp == 0
                || !snapshots_are_enemies(&attacker.snapshot, &candidate.snapshot, session_config)
            {
                return None;
            }

            let distance =
                forward_distance(attacker.snapshot.cell, candidate.snapshot.cell, direction)?;
            (distance <= max_range).then_some((candidate, distance))
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|(candidate, distance)| {
        (
            *distance,
            candidate.snapshot.cell,
            candidate.snapshot.unit_id,
        )
    });

    let nearest_distance = candidates.first().map(|(_, distance)| *distance)?;
    let nearest = candidates
        .into_iter()
        .filter(|(_, distance)| *distance == nearest_distance)
        .map(|(candidate, _)| candidate)
        .collect::<Vec<_>>();

    if nearest.len() == 1 {
        return nearest.first().copied();
    }

    let seed = rng?.range_equidistant_select(
        rng_player_id(attacker.snapshot.player),
        attacker.snapshot.lane,
    );
    nearest.get(seed as usize % nearest.len()).copied()
}

fn rng_player_id(player: PlayerId) -> u32 {
    u32::try_from(player.0).unwrap_or(u32::MAX)
}

fn snapshots_are_enemies(
    first: &UnitSnapshot,
    second: &UnitSnapshot,
    session_config: &SessionConfig,
) -> bool {
    let Some(first_team) = session_config.team_map.get(&first.player).copied() else {
        return first.player != second.player;
    };
    let Some(second_team) = session_config.team_map.get(&second.player).copied() else {
        return first.player != second.player;
    };

    teams_are_enemies(first_team, second_team)
}

fn combat_config(world: &World) -> SharedGameConfig {
    world
        .get_resource::<ServerGameConfig>()
        .map(|config| config.0.clone())
        .unwrap_or_default()
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
                lane: unit.lane,
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

fn collect_pending_placements(world: &World) -> Vec<AcceptedPlacement> {
    let pending = world.resource::<PendingPlacements>();
    let mut players = pending.submissions.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);

    players
        .into_iter()
        .filter_map(|player| pending.submissions.get(&player))
        .flat_map(|submission| submission.placements.iter().cloned())
        .collect()
}

fn deduct_committed_mana(world: &mut World, placements: &[AcceptedPlacement]) {
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

fn enqueue_placement_reveal(world: &mut World, placements: &[AcceptedPlacement]) {
    let message = S2CPlacementReveal {
        placements: placements.iter().map(AcceptedPlacement::reveal).collect(),
    };

    world
        .resource_mut::<CombatNetworkOutbox>()
        .push_placement_reveal(message.clone());
    broadcast_placement_reveal(world, &message);
    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::PlacementRevealEnqueued);
}

fn spawn_committed_placements(world: &mut World, placements: &[AcceptedPlacement]) -> Vec<Entity> {
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
            .push(CombatTraceEntry::UnitPlaced {
                entity,
                player: placement.owner_id,
                lane,
                cell,
            });
    }

    spawned
}

fn board_cell_target(placement: &AcceptedPlacement) -> Option<(LaneId, u8)> {
    match &placement.target {
        PlayTarget::BoardCell { lane, cell } => Some((*lane, *cell)),
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

fn execute_objective_damage(
    world: &mut World,
    budget: &mut IterationBudget,
) -> Result<(), CombatAbort> {
    run_objective_detection_if_ready(world);

    for attack in collect_objective_attacks(world) {
        budget.tick()?;
        apply_single_objective_attack(world, attack);
    }

    Ok(())
}

fn collect_objective_attacks(world: &mut World) -> Vec<ObjectiveAttack> {
    let Some(board_config) = world.get_resource::<BoardConfig>().copied() else {
        return Vec::new();
    };
    let Some(session_config) = world.get_resource::<SessionConfig>().cloned() else {
        return Vec::new();
    };

    let raw_units = {
        let mut query = world.query::<(Entity, &BoardPosition, &UnitOwner, &UnitStats)>();
        query
            .iter(world)
            .map(|(entity, position, owner, stats)| (entity, *position, *owner, *stats))
            .collect::<Vec<_>>()
    };

    let mut attacks = raw_units
        .into_iter()
        .filter_map(|(entity, position, owner, stats)| {
            if stats.hp == 0
                || !is_at_objective(owner.0, position.cell, &session_config, &board_config)
            {
                return None;
            }

            let target_player = objective_target_player(&session_config, owner.0)?;
            let leader_bonus = world
                .get::<UnitKeywordState>(entity)
                .map_or(0, |state| state.leader_bonus_atk);
            let amount = u32::from(stats.atk).saturating_add(u32::from(leader_bonus));
            (amount > 0).then_some(ObjectiveAttack {
                attacker: entity,
                attacker_player: owner.0,
                target_player,
                lane: position.lane,
                cell: position.cell,
                amount,
            })
        })
        .collect::<Vec<_>>();

    attacks.sort_by_key(|attack| {
        (
            attack.lane,
            attack.cell,
            attack.attacker_player.0,
            attack.attacker.index(),
        )
    });
    attacks
}

fn objective_target_player(
    session_config: &SessionConfig,
    attacker_player: PlayerId,
) -> Option<PlayerId> {
    let attacker_team = session_config.team_map.get(&attacker_player).copied()?;
    session_config.players().find(|player| {
        session_config
            .team_map
            .get(player)
            .copied()
            .is_some_and(|team| teams_are_enemies(attacker_team, team))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectiveStateSnapshot {
    hp: u32,
    destroyed: bool,
}

fn objective_state_snapshot(
    world: &mut World,
    target_player: PlayerId,
    lane: LaneId,
) -> Option<ObjectiveStateSnapshot> {
    let mut query = world.query::<(&ObjectiveSlot, &ObjectiveHp)>();
    query.iter(world).find_map(|(slot, hp)| {
        (slot.player == target_player && slot.lane == lane).then_some(ObjectiveStateSnapshot {
            hp: hp.hp,
            destroyed: slot.destroyed,
        })
    })
}

fn apply_single_objective_attack(world: &mut World, attack: ObjectiveAttack) {
    let Some(before) = objective_state_snapshot(world, attack.target_player, attack.lane) else {
        return;
    };
    if before.destroyed || before.hp == 0 {
        return;
    }

    let pending_start = pending_objective_event_count(world);
    apply_objective_damage(world, attack.lane, attack.attacker_player, attack.amount);

    let Some(after) = objective_state_snapshot(world, attack.target_player, attack.lane) else {
        return;
    };

    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::ObjectiveDamaged {
            target_player_id: attack.target_player,
            lane: attack.lane,
            hp_before: before.hp,
            hp_after: after.hp,
            attacker_id: Some(attack.attacker),
        });

    if let Some(event) =
        objective_destroyed_event_since(world, attack.target_player, attack.lane, pending_start)
    {
        world
            .resource_mut::<CombatResolutionTrace>()
            .push(CombatTraceEntry::ObjectiveDestroyed {
                target_player_id: event.target_player_id,
                lane: event.lane,
                was_fake: event.was_fake,
            });

        award_objective_gold(world, attack.attacker_player);
    }
}

fn pending_objective_event_count(world: &World) -> usize {
    world
        .get_resource::<PendingObjectiveEvents>()
        .map_or(0, |pending| pending.queue.len())
}

fn objective_destroyed_event_since(
    world: &World,
    target_player: PlayerId,
    lane: LaneId,
    start_index: usize,
) -> Option<ObjectiveDestroyed> {
    world
        .get_resource::<PendingObjectiveEvents>()?
        .queue
        .iter()
        .skip(start_index)
        .find(|event| event.target_player_id == target_player && event.lane == lane)
        .copied()
}

fn award_objective_gold(world: &mut World, player: PlayerId) {
    let amount = world.get_resource::<ServerGameConfig>().map_or(
        SharedGameConfig::default().objective_gold_reward,
        |config| config.objective_gold_reward,
    );

    world
        .resource_mut::<CombatResolutionTrace>()
        .push(CombatTraceEntry::GoldAwarded {
            player,
            amount,
            reason: GoldAwardReason::ObjectiveReward,
        });

    let Some(mut economies) = world.get_resource_mut::<PlayerEconomies>() else {
        return;
    };
    let Some(economy) = economies.0.get_mut(&player) else {
        return;
    };
    economy_api::apply_gold_award(economy, amount);
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
