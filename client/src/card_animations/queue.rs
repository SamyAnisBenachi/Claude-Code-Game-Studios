use std::{collections::VecDeque, time::Duration};

use bevy::{math::curve::EaseFunction, prelude::*};
use bevy_tweening::{lens::TransformPositionLens, Tween};
use shared::protocol::{RoundPhase, S2CPhaseChanged};

use super::{events::GroupDrainedSignal, make_tween_anim};

const DEFAULT_PRE_ANIMATION_PAUSE_MS: u64 = 400;
const DEFAULT_UNIT_REVEAL_TWEEN_DURATION_MS: u64 = 250;
const MIN_UNIT_REVEAL_TWEEN_DURATION_MS: u64 = 150;
const MAX_UNIT_REVEAL_TWEEN_DURATION_MS: u64 = 400;
const DEFAULT_INTER_STEP_PAUSE_MS: u64 = 150;
const DEFAULT_RESOLUTION_SUB_STEP_DURATION_MS: u64 = 600;
const DEFAULT_OBJECTIVE_REVEAL_MS: u64 = 400;
const DEFAULT_STAGGER_CADENCE_MS: u64 = 100;
const DEFAULT_DAMAGE_NUMBER_FLOAT_TWEEN_MS: u64 = 500;
const DEFAULT_DAMAGE_NUMBER_FADE_TWEEN_MS: u64 = 500;
const DAMAGE_NUMBER_BUDGET_BUFFER_MS: u64 = 50;

#[derive(Resource, Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnimationTimingConfig {
    pub pre_animation_pause_ms: u64,
    pub unit_reveal_tween_duration_ms: u64,
    pub inter_step_pause_ms: u64,
    pub resolution_sub_step_duration_ms: u64,
    pub objective_reveal_ms: u64,
    pub stagger_cadence_ms: u64,
    pub damage_number_float_tween_ms: u64,
    pub damage_number_fade_tween_ms: u64,
}

impl Default for AnimationTimingConfig {
    fn default() -> Self {
        Self {
            pre_animation_pause_ms: DEFAULT_PRE_ANIMATION_PAUSE_MS,
            unit_reveal_tween_duration_ms: DEFAULT_UNIT_REVEAL_TWEEN_DURATION_MS,
            inter_step_pause_ms: DEFAULT_INTER_STEP_PAUSE_MS,
            resolution_sub_step_duration_ms: DEFAULT_RESOLUTION_SUB_STEP_DURATION_MS,
            objective_reveal_ms: DEFAULT_OBJECTIVE_REVEAL_MS,
            stagger_cadence_ms: DEFAULT_STAGGER_CADENCE_MS,
            damage_number_float_tween_ms: DEFAULT_DAMAGE_NUMBER_FLOAT_TWEEN_MS,
            damage_number_fade_tween_ms: DEFAULT_DAMAGE_NUMBER_FADE_TWEEN_MS,
        }
    }
}

impl AnimationTimingConfig {
    pub fn assert_unit_reveal_tween_budget(self) {
        assert!(
            (MIN_UNIT_REVEAL_TWEEN_DURATION_MS..=MAX_UNIT_REVEAL_TWEEN_DURATION_MS)
                .contains(&self.unit_reveal_tween_duration_ms),
            "unit_reveal_tween_duration_ms={} outside allowed range {}..={}",
            self.unit_reveal_tween_duration_ms,
            MIN_UNIT_REVEAL_TWEEN_DURATION_MS,
            MAX_UNIT_REVEAL_TWEEN_DURATION_MS
        );
    }

    pub fn assert_damage_number_budget(self) {
        let despawn_delay_ms = self.damage_number_despawn_delay_ms();
        assert!(
            despawn_delay_ms + DAMAGE_NUMBER_BUDGET_BUFFER_MS
                < self.resolution_sub_step_duration_ms,
            "damage number lifecycle budget invalid: max(float_tween_duration_ms={}, fade_tween_duration_ms={}) + {}ms must be strictly less than resolution_sub_step_duration_ms={}",
            self.damage_number_float_tween_ms,
            self.damage_number_fade_tween_ms,
            DAMAGE_NUMBER_BUDGET_BUFFER_MS,
            self.resolution_sub_step_duration_ms
        );
    }

    pub fn damage_number_float_duration(self) -> Duration {
        Duration::from_millis(self.damage_number_float_tween_ms)
    }

    pub fn damage_number_fade_duration(self) -> Duration {
        Duration::from_millis(self.damage_number_fade_tween_ms)
    }

    pub fn damage_number_despawn_delay(self) -> Duration {
        Duration::from_millis(self.damage_number_despawn_delay_ms())
    }

    pub fn damage_number_despawn_delay_ms(self) -> u64 {
        self.damage_number_float_tween_ms
            .max(self.damage_number_fade_tween_ms)
    }

    pub fn unit_reveal_tween_duration(self) -> Duration {
        Duration::from_millis(self.unit_reveal_tween_duration_ms)
    }

    fn pre_animation_pause(self) -> Duration {
        Duration::from_millis(self.pre_animation_pause_ms)
    }

    fn inter_step_pause(self) -> Duration {
        Duration::from_millis(self.inter_step_pause_ms)
    }

    fn objective_reveal_duration(self) -> Duration {
        Duration::from_millis(self.objective_reveal_ms)
    }
}

#[derive(Clone, Debug)]
pub enum AnimQueueEvent {
    TransformTween {
        target: Entity,
        start: Vec3,
        end: Vec3,
        duration_ms: u64,
    },
}

impl AnimQueueEvent {
    pub fn transform_tween(target: Entity, start: Vec3, end: Vec3, duration_ms: u64) -> Self {
        Self::TransformTween {
            target,
            start,
            end,
            duration_ms,
        }
    }

    fn spawn(&self, commands: &mut Commands) {
        match self {
            Self::TransformTween {
                target,
                start,
                end,
                duration_ms,
            } => {
                let tween = Tween::new(
                    EaseFunction::Linear,
                    Duration::from_millis(*duration_ms),
                    TransformPositionLens {
                        start: *start,
                        end: *end,
                    },
                );

                if let Ok(mut entity) = commands.get_entity(*target) {
                    entity.insert(make_tween_anim(tween));
                } else {
                    warn!("AnimQueue target entity {target:?} no longer exists");
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimGroup {
    pub sub_step: u8,
    pub events: Vec<AnimQueueEvent>,
    pub duration_ms: u64,
}

impl AnimGroup {
    pub fn new(sub_step: u8, duration_ms: u64, events: Vec<AnimQueueEvent>) -> Self {
        Self {
            sub_step,
            events,
            duration_ms,
        }
    }
}

#[derive(Resource, Debug)]
pub struct AnimQueue {
    pub groups: Vec<AnimGroup>,
    pub current_index: usize,
    pub group_timer: Timer,
    pub inter_step_timer: Timer,
    empty_queue_elapsed: Duration,
    current_group_started: bool,
    inter_step_active: bool,
    empty_queue_active: bool,
}

impl Default for AnimQueue {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            current_index: 0,
            group_timer: Timer::new(Duration::ZERO, TimerMode::Once),
            inter_step_timer: Timer::new(Duration::ZERO, TimerMode::Once),
            empty_queue_elapsed: Duration::ZERO,
            current_group_started: false,
            inter_step_active: false,
            empty_queue_active: false,
        }
    }
}

impl AnimQueue {
    pub fn from_groups(groups: Vec<AnimGroup>) -> Self {
        let mut queue = Self::default();
        queue.load_groups(groups);
        queue
    }

    pub fn load_groups(&mut self, groups: Vec<AnimGroup>) {
        self.groups = groups;
        self.current_index = 0;
        self.current_group_started = false;
        self.inter_step_active = false;
        self.empty_queue_active = false;
        self.reset_current_group_timer();
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn reset_current_group_timer(&mut self) {
        let duration = self
            .groups
            .get(self.current_index)
            .map(|group| Duration::from_millis(group.duration_ms))
            .unwrap_or(Duration::ZERO);
        self.group_timer = Timer::new(duration, TimerMode::Once);
    }

    fn start_current_group(&mut self, commands: &mut Commands) {
        if let Some(group) = self.groups.get(self.current_index) {
            for event in &group.events {
                event.spawn(commands);
            }
        }
        self.current_group_started = true;
    }

    fn clear_after_drain(&mut self) {
        self.groups.clear();
        self.current_index = 0;
        self.current_group_started = false;
        self.inter_step_active = false;
        self.empty_queue_active = false;
        self.reset_current_group_timer();
    }
}

#[derive(Resource, Default, Debug)]
pub struct PendingPhaseChange {
    phase_change: Option<S2CPhaseChanged>,
}

impl PendingPhaseChange {
    pub fn set(&mut self, phase_change: S2CPhaseChanged) {
        self.phase_change = Some(phase_change);
    }

    pub fn set_phase(&mut self, phase: RoundPhase) {
        self.set(S2CPhaseChanged {
            phase,
            round_number: 0,
            timer_duration_ms: 0,
        });
    }

    pub fn phase(&self) -> Option<RoundPhase> {
        self.phase_change
            .as_ref()
            .map(|phase_change| phase_change.phase)
    }

    pub fn is_none(&self) -> bool {
        self.phase_change.is_none()
    }

    pub fn clear(&mut self) {
        self.phase_change = None;
    }

    fn take(&mut self) -> Option<S2CPhaseChanged> {
        self.phase_change.take()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingObjectiveDestroyedEvent {
    pub lane: u8,
    pub entity: Entity,
}

#[derive(Resource, Default, Debug)]
pub struct PendingObjectiveDestroyedEvents {
    events: Vec<PendingObjectiveDestroyedEvent>,
}

impl PendingObjectiveDestroyedEvents {
    pub fn push(&mut self, lane: u8, entity: Entity) {
        self.events
            .push(PendingObjectiveDestroyedEvent { lane, entity });
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    fn drain_sorted_by_lane(&mut self) -> Vec<PendingObjectiveDestroyedEvent> {
        let mut events = std::mem::take(&mut self.events);
        events.sort_by_key(|event| event.lane);
        events
    }
}

#[derive(Debug)]
pub struct StagedObjectiveReveal {
    pub lane: u8,
    pub entity: Option<Entity>,
    timer: Timer,
    ready_immediately: bool,
    staged_this_frame: bool,
}

impl StagedObjectiveReveal {
    pub fn new(lane: u8, entity: Entity, timer: Timer) -> Self {
        Self {
            lane,
            entity: Some(entity),
            timer,
            ready_immediately: false,
            staged_this_frame: false,
        }
    }

    pub fn immediate(lane: u8, entity: Entity) -> Self {
        Self {
            lane,
            entity: Some(entity),
            timer: Timer::new(Duration::ZERO, TimerMode::Once),
            ready_immediately: true,
            staged_this_frame: false,
        }
    }

    pub fn without_entity(lane: u8, timer: Timer) -> Self {
        Self {
            lane,
            entity: None,
            timer,
            ready_immediately: false,
            staged_this_frame: false,
        }
    }

    fn staged(lane: u8, entity: Entity, delay: Duration) -> Self {
        if delay.is_zero() {
            return Self::immediate(lane, entity);
        }

        Self {
            lane,
            entity: Some(entity),
            timer: Timer::new(delay, TimerMode::Once),
            ready_immediately: false,
            staged_this_frame: true,
        }
    }

    fn tick(&mut self, delta: Duration) {
        if self.ready_immediately {
            return;
        }

        if self.staged_this_frame {
            self.staged_this_frame = false;
            return;
        }

        self.timer.tick(delta);
    }

    fn is_ready(&self) -> bool {
        self.ready_immediately || self.timer.is_finished()
    }
}

#[derive(Resource, Default, Debug)]
pub struct StagedObjectiveRevealQueue {
    reveals: VecDeque<StagedObjectiveReveal>,
}

impl StagedObjectiveRevealQueue {
    pub fn push(&mut self, lane: u8, timer: Timer) {
        self.reveals
            .push_back(StagedObjectiveReveal::without_entity(lane, timer));
    }

    pub fn push_reveal(&mut self, reveal: StagedObjectiveReveal) {
        self.reveals.push_back(reveal);
    }

    pub fn pop_front(&mut self) -> Option<StagedObjectiveReveal> {
        self.reveals.pop_front()
    }

    pub fn clear(&mut self) {
        self.reveals.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.reveals.is_empty()
    }

    pub fn len(&self) -> usize {
        self.reveals.len()
    }

    fn pop_ready(&mut self, delta: Duration) -> Vec<StagedObjectiveReveal> {
        let mut ready = Vec::new();

        for reveal in &mut self.reveals {
            reveal.tick(delta);
        }

        while self
            .reveals
            .front()
            .is_some_and(StagedObjectiveReveal::is_ready)
        {
            if let Some(reveal) = self.reveals.pop_front() {
                ready.push(reveal);
            }
        }

        ready
    }
}

pub fn resolution_executing_system(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    timings: Res<AnimationTimingConfig>,
    mut queue: ResMut<AnimQueue>,
    mut pending_phase: ResMut<PendingPhaseChange>,
    mut pending_objectives: ResMut<PendingObjectiveDestroyedEvents>,
    mut staged_objectives: ResMut<StagedObjectiveRevealQueue>,
    mut drained_signals: MessageWriter<GroupDrainedSignal>,
) {
    let delta = time.delta();

    if queue.groups.is_empty() {
        drain_empty_queue(
            delta,
            timings.pre_animation_pause(),
            &mut queue,
            &mut pending_phase,
        );
        return;
    }

    if !queue.current_group_started {
        queue.start_current_group(&mut commands);
    }

    queue.group_timer.tick(delta);
    if !queue.group_timer.is_finished() {
        return;
    }

    if pending_phase.phase() == Some(RoundPhase::GameOver) {
        stage_objective_reveals(&mut pending_objectives, &mut staged_objectives, &timings);
        drain_pending_phase(&mut pending_phase);
        drained_signals.write(GroupDrainedSignal);
        queue.clear_after_drain();
        return;
    }

    if queue.current_index + 1 >= queue.groups.len() {
        stage_objective_reveals(&mut pending_objectives, &mut staged_objectives, &timings);
        drain_pending_phase(&mut pending_phase);
        queue.clear_after_drain();
        return;
    }

    if !queue.inter_step_active {
        queue.inter_step_timer = Timer::new(timings.inter_step_pause(), TimerMode::Once);
        queue.inter_step_active = true;
        return;
    }

    queue.inter_step_timer.tick(delta);
    if !queue.inter_step_timer.is_finished() {
        return;
    }

    queue.current_index += 1;
    queue.inter_step_active = false;
    queue.current_group_started = false;
    queue.reset_current_group_timer();
    queue.start_current_group(&mut commands);
}

pub fn resolution_objective_reveal_system(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    timings: Res<AnimationTimingConfig>,
    mut staged_objectives: ResMut<StagedObjectiveRevealQueue>,
) {
    for reveal in staged_objectives.pop_ready(time.delta()) {
        let Some(entity) = reveal.entity else {
            warn!(
                "Objective reveal for lane {} has no target entity; skipping",
                reveal.lane
            );
            continue;
        };

        let tween = Tween::new(
            EaseFunction::Linear,
            timings.objective_reveal_duration(),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::new(0.0, 8.0, 0.0),
            },
        );

        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(make_tween_anim(tween));
        } else {
            warn!(
                "Objective reveal target for lane {} no longer exists",
                reveal.lane
            );
        }
    }
}

fn drain_empty_queue(
    delta: Duration,
    pre_animation_pause: Duration,
    queue: &mut AnimQueue,
    pending_phase: &mut PendingPhaseChange,
) {
    if pending_phase.is_none() {
        queue.empty_queue_active = false;
        return;
    }

    if !queue.empty_queue_active {
        queue.empty_queue_elapsed = Duration::ZERO;
        queue.empty_queue_active = true;
    }

    queue.empty_queue_elapsed += delta;
    if queue.empty_queue_elapsed >= pre_animation_pause {
        drain_pending_phase(pending_phase);
        queue.empty_queue_active = false;
        queue.empty_queue_elapsed = Duration::ZERO;
    }
}

fn stage_objective_reveals(
    pending_objectives: &mut PendingObjectiveDestroyedEvents,
    staged_objectives: &mut StagedObjectiveRevealQueue,
    timings: &AnimationTimingConfig,
) {
    for (index, event) in pending_objectives
        .drain_sorted_by_lane()
        .into_iter()
        .enumerate()
    {
        let delay_ms = index as u64 * timings.stagger_cadence_ms;
        staged_objectives.push_reveal(StagedObjectiveReveal::staged(
            event.lane,
            event.entity,
            Duration::from_millis(delay_ms),
        ));
    }
}

fn drain_pending_phase(pending_phase: &mut PendingPhaseChange) {
    let _ = pending_phase.take();
}
