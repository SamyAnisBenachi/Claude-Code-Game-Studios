#![allow(dead_code)]

pub mod modifier_stack;

use bevy::ecs::message::MessageCursor;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use shared::protocol::{GameOverReason, S2CPlacementReveal, S2CResolutionEvent, TaggedEvent};

use crate::core::rsm::{
    advance_phase, BeginResolution, PendingPhaseAdvance, PhaseAdvanceRequest, ResolutionComplete,
    RoundPhase,
};
use crate::core::session::SessionConfig;
use crate::feature::board::{
    detect_objective_presence, BoardConfig, BoardSystemSet, UnitAtObjective,
};

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
    BeginResolutionRead { round: u32 },
    PlacementRevealEnqueued,
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

        if sub_step == 6 {
            run_objective_detection_if_ready(world);
        }
    }

    Ok(())
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
