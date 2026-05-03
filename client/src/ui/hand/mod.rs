use std::collections::BTreeSet;
use std::time::Duration;

use bevy::ecs::query::QueryFilter;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, Tween, TweenAnim};
use lightyear::prelude::MessageSender;
use shared::card::{CardCatalog, CardId, CardType};
use shared::protocol::{
    C2SActivateCard, C2SPurchaseCard, C2SSubmitPlacement, EntityId, PlacedCard, PlayTarget,
    ReliableChannel, RoundPhase,
};
use shared::session::PlayerId;

use crate::card_animations::{
    cancel_tween_anim_in_place, make_tween_anim, replace_tweenable, HandCard, HandDragSprite,
};
use crate::state::{ClientState, CurrentClientPhase};
use crate::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};

pub const HAND_FAN_SLOT_COUNT: usize = 10;
pub const DRAFT_INITIAL_GRID_SLOT_COUNT: usize = 9;
pub const HAND_UI_ENTITY_COUNT: usize = HAND_FAN_SLOT_COUNT + DRAFT_INITIAL_GRID_SLOT_COUNT + 7;
const HAND_DRAG_SPRITE_SCALE: f32 = 1.10;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct HandFanLayoutConfig {
    pub fan_base_margin_px: f32,
    pub fan_half_spread_px: f32,
    pub arc_height_px: f32,
    pub max_rotation_deg: f32,
}

impl Default for HandFanLayoutConfig {
    fn default() -> Self {
        Self {
            fan_base_margin_px: 100.0,
            fan_half_spread_px: 280.0,
            arc_height_px: 10.0,
            max_rotation_deg: 10.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct HandFanViewport {
    pub width_px: f32,
    pub height_px: f32,
}

impl Default for HandFanViewport {
    fn default() -> Self {
        Self {
            width_px: 800.0,
            height_px: 600.0,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFanLayoutState {
    pub hand_count: usize,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct HandCardCatalog {
    pub cards: CardCatalog,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiTimingConfig {
    pub card_draw_animation_ms: u64,
    pub purchase_timeout_ms: u64,
    pub hand_full_notification_duration_ms: u64,
}

impl Default for HandUiTimingConfig {
    fn default() -> Self {
        Self {
            card_draw_animation_ms: 280,
            purchase_timeout_ms: 3_000,
            hand_full_notification_duration_ms: 2_000,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementTimerConfig {
    pub placement_duration_ms: u32,
    pub urgency_threshold_ms: u32,
    pub grace_window_ms: u32,
}

impl Default for PlacementTimerConfig {
    fn default() -> Self {
        Self {
            placement_duration_ms: 10_000,
            urgency_threshold_ms: 5_000,
            grace_window_ms: 200,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiEconomyView {
    pub gold: u32,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct HandContents {
    pub cards: Vec<CardId>,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandUiMode {
    #[default]
    Hidden,
    Grid,
    Passive,
    PassiveLocked,
    Staging,
}

impl HandUiMode {
    pub fn from_phase(phase: RoundPhase) -> Self {
        match phase {
            RoundPhase::DraftInitial => Self::Grid,
            RoundPhase::DraftShop => Self::Passive,
            RoundPhase::DraftAuction => Self::PassiveLocked,
            RoundPhase::Placement => Self::Staging,
            RoundPhase::Lobby
            | RoundPhase::Resolution
            | RoundPhase::GameOver
            | RoundPhase::Handshaking => Self::Hidden,
        }
    }

    fn shows_fan_root(self) -> bool {
        matches!(
            self,
            Self::Grid | Self::Passive | Self::PassiveLocked | Self::Staging
        )
    }

    fn shows_fan_slots(self) -> bool {
        self.shows_fan_root()
    }

    fn allows_activation(self) -> bool {
        self == Self::Passive
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct HandUiOutboundMessages {
    pub activate_cards: Vec<C2SActivateCard>,
    pub purchase_cards: Vec<C2SPurchaseCard>,
    pub submit_placements: Vec<C2SSubmitPlacement>,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PendingPlacements {
    pub placements: Vec<PlacedCard>,
}

impl PendingPlacements {
    pub fn staged_count(&self) -> usize {
        self.placements.len()
    }

    fn clear(&mut self) {
        self.placements.clear();
    }

    fn stage_or_update(&mut self, placement: PlacedCard) {
        if let Some(existing) = self
            .placements
            .iter_mut()
            .find(|existing| existing.card_id == placement.card_id)
        {
            *existing = placement;
            return;
        }

        self.placements.push(placement);
    }

    fn target_for(&self, card_id: CardId) -> Option<&PlayTarget> {
        self.placements
            .iter()
            .find(|placement| placement.card_id == card_id)
            .map(|placement| &placement.target)
    }

    fn remove_staged(&mut self, card_id: CardId) -> Option<PlacedCard> {
        let index = self
            .placements
            .iter()
            .position(|placement| placement.card_id == card_id)?;
        Some(self.placements.remove(index))
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementTimer {
    pub remaining_ms: u32,
    pub urgency_fired: bool,
    pub in_grace_window: bool,
    pub grace_remaining_ms: u32,
    pub submitted: bool,
}

impl Default for PlacementTimer {
    fn default() -> Self {
        Self {
            remaining_ms: 0,
            urgency_fired: false,
            in_grace_window: false,
            grace_remaining_ms: 0,
            submitted: false,
        }
    }
}

impl PlacementTimer {
    fn reset_for_placement(&mut self, duration_ms: u32) {
        self.remaining_ms = duration_ms;
        self.urgency_fired = false;
        self.in_grace_window = false;
        self.grace_remaining_ms = 0;
        self.submitted = false;
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementBoardView {
    pub local_player_id: PlayerId,
    pub opponent_player_id: PlayerId,
    pub spawn_edge: BoardSpawnEdge,
    pub spawn_range_cells: u8,
}

impl Default for PlacementBoardView {
    fn default() -> Self {
        Self {
            local_player_id: PlayerId(1),
            opponent_player_id: PlayerId(2),
            spawn_edge: BoardSpawnEdge::LowCells,
            spawn_range_cells: 1,
        }
    }
}

impl PlacementBoardView {
    fn is_spawn_cell(self, lane: u8, cell: u8) -> bool {
        if !(1..=BOARD_LANE_COUNT).contains(&lane) {
            return false;
        }

        let range = self.spawn_range_cells.clamp(1, BOARD_CELL_COUNT);
        match self.spawn_edge {
            BoardSpawnEdge::LowCells => (1..=range).contains(&cell),
            BoardSpawnEdge::HighCells => {
                let first_cell = BOARD_CELL_COUNT - range + 1;
                (first_cell..=BOARD_CELL_COUNT).contains(&cell)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardSpawnEdge {
    LowCells,
    HighCells,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct ActivePlacementDrag {
    pub card: Option<Entity>,
    pub card_id: Option<CardId>,
    pub owner_id: Option<PlayerId>,
    pub target_kind: Option<PlacementTargetKind>,
    pub cursor_world_position: Option<Vec2>,
}

impl ActivePlacementDrag {
    fn start(
        &mut self,
        card: Entity,
        card_id: CardId,
        owner_id: PlayerId,
        target_kind: PlacementTargetKind,
    ) {
        self.card = Some(card);
        self.card_id = Some(card_id);
        self.owner_id = Some(owner_id);
        self.target_kind = Some(target_kind);
        self.cursor_world_position = None;
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn is_active(self) -> bool {
        self.card.is_some() && self.target_kind.is_some()
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct ActiveGhostUnstageDrag {
    pub card_id: Option<CardId>,
    pub cursor_screen_position: Option<Vec2>,
}

impl ActiveGhostUnstageDrag {
    fn start(&mut self, card_id: CardId) {
        self.card_id = Some(card_id);
        self.cursor_screen_position = None;
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn is_active(self) -> bool {
        self.card_id.is_some()
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct FanZoneBounds {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl Default for FanZoneBounds {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            x_max: 800.0,
            y_min: 340.0,
            y_max: 600.0,
        }
    }
}

impl FanZoneBounds {
    fn contains(self, position: Vec2) -> bool {
        let x_min = self.x_min.min(self.x_max);
        let x_max = self.x_min.max(self.x_max);
        let y_min = self.y_min.min(self.y_max);
        let y_max = self.y_min.max(self.y_max);

        (x_min..=x_max).contains(&position.x) && (y_min..=y_max).contains(&position.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementTargetKind {
    Minion,
    TargetObj,
    LaneWide,
    TargetUnit,
    Instant,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandPlacementTargetKind(pub PlacementTargetKind);

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiPlacementDragStarted {
    pub card: Entity,
    pub owner_id: PlayerId,
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct HandUiPlacementCursorMoved {
    pub world_position: Option<Vec2>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiPlacementDragEnded;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFanCardClicked {
    pub card: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandGridCardClicked {
    pub card: Entity,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct HandUiDraftOfferingReceived {
    pub card_ids: Vec<CardId>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiCardAcquiredReceived {
    pub card_id: CardId,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct HandUiPlacementDropResolved {
    pub card: Entity,
    pub owner_id: PlayerId,
    pub target: Option<PlayTarget>,
}

#[derive(Message, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimerUrgencyAudio;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSubmitButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct GhostPlacementChanged {
    pub target: Option<PlayTarget>,
    pub card_id: Option<CardId>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostClickedEvent {
    pub card_id: CardId,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostDragStartEvent {
    pub card_id: CardId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FanLayoutMetrics {
    pub fan_center_x: f32,
    pub fan_base_y: f32,
    pub fan_half_spread: f32,
    pub arc_height: f32,
    pub max_rotation_deg: f32,
}

impl HandFanLayoutConfig {
    pub fn metrics_for_viewport(&self, viewport: HandFanViewport) -> FanLayoutMetrics {
        FanLayoutMetrics {
            fan_center_x: viewport.width_px / 2.0,
            fan_base_y: viewport.height_px - self.fan_base_margin_px,
            fan_half_spread: self.fan_half_spread_px,
            arc_height: self.arc_height_px,
            max_rotation_deg: self.max_rotation_deg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FanSlotLayout {
    pub t: f32,
    pub card_x: f32,
    pub card_y: f32,
    pub card_rotation_deg: f32,
}

impl FanSlotLayout {
    pub fn bevy_rotation_radians(&self) -> f32 {
        -self.card_rotation_deg.to_radians()
    }

    pub fn bevy_rotation(&self) -> Quat {
        Quat::from_rotation_z(self.bevy_rotation_radians())
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSubmitButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmationModal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFanRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandTimer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Normal,
    Urgent,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerSubmittedCheckmark;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFullNotification;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSubmitInteractionState {
    Active,
    Inactive,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanSlotState {
    Active,
    Ghost,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCellHighlighted;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanPlateDropZone;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanPlateHighlighted;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCellOccupied;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveCell {
    pub player_id: PlayerId,
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveAlive;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementTargetUnit {
    pub owner_id: PlayerId,
    pub unit_id: EntityId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetUnitHover;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoValidTargetsOverlay;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveStripForFanSlot(pub u8);

#[derive(Resource, Debug, Clone, Copy)]
pub struct HandUiEntities {
    pub fan_root: Entity,
    pub fan_slots: [Entity; HAND_FAN_SLOT_COUNT],
    pub grid_slots: [Entity; DRAFT_INITIAL_GRID_SLOT_COUNT],
    pub drag_sprite: Entity,
    pub submit_button: Entity,
    pub timer: Entity,
    pub submitted_checkmark: Entity,
    pub hand_full_notification: Entity,
    pub no_valid_targets_overlay: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotIndex(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSlotIndex(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSlotCard(pub CardId);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSlotCard(pub CardId);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct GridSlotCardName(pub String);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSlotManaCost(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridSlotState {
    Available,
    Pending,
    HandFullLocked,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingPurchaseTimer {
    pub remaining_ms: u64,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationTimer {
    pub remaining_ms: u64,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandUiSystemSet {
    PhaseTransition,
    MessageDrain,
    Input,
    StateSync,
}

pub struct HandUiPlugin;

impl Plugin for HandUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .init_resource::<CurrentClientPhase>()
            .init_resource::<HandFanLayoutConfig>()
            .init_resource::<HandFanViewport>()
            .init_resource::<HandFanLayoutState>()
            .init_resource::<HandCardCatalog>()
            .init_resource::<HandUiTimingConfig>()
            .init_resource::<PlacementTimerConfig>()
            .init_resource::<HandUiEconomyView>()
            .init_resource::<HandContents>()
            .init_resource::<HandUiMode>()
            .init_resource::<HandUiOutboundMessages>()
            .init_resource::<PendingPlacements>()
            .init_resource::<PlacementTimer>()
            .init_resource::<PlacementBoardView>()
            .init_resource::<ActivePlacementDrag>()
            .init_resource::<ActiveGhostUnstageDrag>()
            .init_resource::<FanZoneBounds>()
            .add_message::<HandFanCardClicked>()
            .add_message::<HandGridCardClicked>()
            .add_message::<HandUiDraftOfferingReceived>()
            .add_message::<HandUiCardAcquiredReceived>()
            .add_message::<HandUiPlacementDragStarted>()
            .add_message::<HandUiPlacementCursorMoved>()
            .add_message::<HandUiPlacementDragEnded>()
            .add_message::<HandUiPlacementDropResolved>()
            .add_message::<HandSubmitButtonClicked>()
            .add_message::<TimerUrgencyAudio>()
            .add_message::<GhostPlacementChanged>()
            .add_message::<GhostClickedEvent>()
            .add_message::<GhostDragStartEvent>()
            .configure_sets(
                Update,
                (
                    HandUiSystemSet::PhaseTransition,
                    HandUiSystemSet::MessageDrain,
                    HandUiSystemSet::Input,
                    HandUiSystemSet::StateSync,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .add_systems(OnEnter(ClientState::InSession), spawn_hand_ui)
            .add_systems(OnExit(ClientState::InSession), despawn_hand_ui)
            .add_systems(
                Update,
                (
                    hand_ui_phase_transition_system.in_set(HandUiSystemSet::PhaseTransition),
                    (
                        handle_draft_offering_system,
                        handle_card_acquired_system,
                        handle_ghost_clicked_unstage_system,
                        handle_ghost_drag_started_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::MessageDrain),
                    (
                        handle_placement_drag_started_system,
                        handle_placement_cursor_moved_system,
                        handle_placement_drag_ended_system,
                        handle_ghost_drag_ended_system,
                        handle_grid_card_click_system,
                        handle_hand_fan_card_click_system,
                        handle_placement_drop_resolved_system,
                        handle_submit_button_click_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::Input),
                    (
                        tick_placement_timer_system,
                        apply_placement_drag_highlights_system,
                        tick_pending_purchase_timeouts_system,
                        apply_fan_layout_system,
                        tick_hand_full_notification_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::StateSync),
                ),
            );
    }
}

pub fn compute_fan_slot_layout(
    index: usize,
    count: usize,
    metrics: FanLayoutMetrics,
) -> Option<FanSlotLayout> {
    if count == 0 || count > HAND_FAN_SLOT_COUNT || index >= count {
        return None;
    }

    let t = if count == 1 {
        0.0
    } else {
        let half_span = (count - 1) as f32 / 2.0;
        (index as f32 - half_span) / half_span
    };

    Some(FanSlotLayout {
        t,
        card_x: metrics.fan_center_x + t * metrics.fan_half_spread,
        card_y: metrics.fan_base_y - metrics.arc_height * t * t,
        card_rotation_deg: metrics.max_rotation_deg * t,
    })
}

pub fn apply_fan_layout_system(
    layout_state: Res<HandFanLayoutState>,
    config: Res<HandFanLayoutConfig>,
    viewport: Res<HandFanViewport>,
    mut fan_slots: Query<
        (&FanSlotIndex, &mut Visibility, &mut Transform, &mut Node),
        Without<HandSubmitButton>,
    >,
) {
    let hand_count = layout_state.hand_count.min(HAND_FAN_SLOT_COUNT);
    let metrics = config.metrics_for_viewport(*viewport);

    for (slot_index, mut visibility, mut transform, mut node) in &mut fan_slots {
        let Some(layout) = compute_fan_slot_layout(slot_index.0 as usize, hand_count, metrics)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        *visibility = Visibility::Visible;
        transform.translation.x = layout.card_x;
        transform.translation.y = layout.card_y;
        transform.rotation = layout.bevy_rotation();
        node.left = Val::Px(layout.card_x);
        node.top = Val::Px(layout.card_y);
    }
}

pub fn hand_ui_phase_transition_system(
    current: Res<CurrentClientPhase>,
    hand_contents: Res<HandContents>,
    timer_config: Res<PlacementTimerConfig>,
    mut mode: ResMut<HandUiMode>,
    mut layout_state: ResMut<HandFanLayoutState>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
    entities: Option<Res<HandUiEntities>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
    mut submit_buttons: Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    mut animators: Query<(Entity, &mut TweenAnim), With<HandUiEntity>>,
    mut timer_states: Query<&mut TimerState, With<HandTimer>>,
) {
    let phase_changed = current.is_changed();
    if !phase_changed && !hand_contents.is_changed() {
        return;
    }

    let Some(entities) = entities else {
        return;
    };

    let next_mode = HandUiMode::from_phase(current.phase);
    let entering_staging = phase_changed && next_mode == HandUiMode::Staging;
    *mode = next_mode;
    layout_state.hand_count = if next_mode.shows_fan_slots() {
        hand_contents.cards.len().min(HAND_FAN_SLOT_COUNT)
    } else {
        0
    };

    if phase_changed {
        pending_placements.clear();
        placement_timer.in_grace_window = false;
        placement_timer.grace_remaining_ms = 0;
        placement_timer.submitted = false;
        active_drag.clear();
        active_ghost_drag.clear();
        commands
            .entity(entities.fan_root)
            .remove::<FanPlateHighlighted>();
    }

    if next_mode == HandUiMode::Hidden {
        cancel_hand_ui_tweens(&mut commands, &mut animators);
    }

    set_visibility(
        entities.fan_root,
        visibility_for(next_mode.shows_fan_root()),
        &mut visibility_query,
    );
    set_visibility(
        entities.submit_button,
        visibility_for(next_mode == HandUiMode::Staging),
        &mut visibility_query,
    );
    set_visibility(
        entities.timer,
        visibility_for(next_mode == HandUiMode::Staging),
        &mut visibility_query,
    );
    if phase_changed {
        set_visibility(
            entities.submitted_checkmark,
            Visibility::Hidden,
            &mut visibility_query,
        );
    }
    set_visibility(
        entities.drag_sprite,
        Visibility::Hidden,
        &mut visibility_query,
    );
    set_visibility(
        entities.no_valid_targets_overlay,
        Visibility::Hidden,
        &mut visibility_query,
    );

    for entity in entities.grid_slots.iter().copied() {
        if next_mode != HandUiMode::Grid {
            set_visibility(entity, Visibility::Hidden, &mut visibility_query);
            clear_grid_slot(&mut commands, entity);
        }
    }

    if next_mode != HandUiMode::Grid {
        set_visibility(
            entities.hand_full_notification,
            Visibility::Hidden,
            &mut visibility_query,
        );
        commands
            .entity(entities.hand_full_notification)
            .remove::<NotificationTimer>();
    }

    for (index, entity) in entities.fan_slots.iter().copied().enumerate() {
        if next_mode.shows_fan_slots() {
            if let Some(card_id) = hand_contents.cards.get(index).copied() {
                commands.entity(entity).insert(HandSlotCard(card_id));
                if entering_staging {
                    commands.entity(entity).insert(FanSlotState::Active);
                }
            } else {
                commands
                    .entity(entity)
                    .remove::<(HandSlotCard, FanSlotState)>();
            }
        } else {
            set_visibility(entity, Visibility::Hidden, &mut visibility_query);
            commands
                .entity(entity)
                .remove::<(HandSlotCard, FanSlotState)>();
        }
    }

    if entering_staging {
        placement_timer.reset_for_placement(timer_config.placement_duration_ms);
        if let Ok(mut timer_state) = timer_states.get_mut(entities.timer) {
            *timer_state = TimerState::Normal;
        }

        for (mut text, mut interaction_state) in &mut submit_buttons {
            text.0.clear();
            text.0.push_str("Submit (0 cards)");
            *interaction_state = HandSubmitInteractionState::Active;
        }
    }
}

pub fn tick_placement_timer_system(
    mode: Res<HandUiMode>,
    time: Res<Time<Virtual>>,
    timer_config: Res<PlacementTimerConfig>,
    entities: Option<Res<HandUiEntities>>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    pending_placements: Res<PendingPlacements>,
    mut urgency_writer: MessageWriter<TimerUrgencyAudio>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<HandTimer>>,
        Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    )>,
    mut timer_states: Query<&mut TimerState, With<HandTimer>>,
    mut submit_senders: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        return;
    };

    if *mode != HandUiMode::Staging {
        return;
    }

    let mut elapsed_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
    let previous_remaining_ms = placement_timer.remaining_ms;

    if placement_timer.remaining_ms > 0 {
        let consumed_ms = placement_timer.remaining_ms.min(elapsed_ms);
        placement_timer.remaining_ms -= consumed_ms;
        elapsed_ms -= consumed_ms;
    }

    if previous_remaining_ms > timer_config.urgency_threshold_ms
        && placement_timer.remaining_ms <= timer_config.urgency_threshold_ms
        && !placement_timer.urgency_fired
    {
        placement_timer.urgency_fired = true;
        if let Ok(mut timer_state) = timer_states.get_mut(entities.timer) {
            *timer_state = TimerState::Urgent;
        }
        urgency_writer.write(TimerUrgencyAudio);
    }

    if placement_timer.remaining_ms == 0
        && !placement_timer.submitted
        && !placement_timer.in_grace_window
    {
        if active_drag.is_active() {
            placement_timer.in_grace_window = true;
            placement_timer.grace_remaining_ms = timer_config.grace_window_ms;
        } else {
            let mut submit_buttons = text_sets.p1();
            submit_pending_placements(
                &pending_placements,
                entities.submit_button,
                entities.submitted_checkmark,
                &mut submit_buttons,
                &mut submit_senders,
                &mut outbound,
                &mut placement_timer,
                &mut visibility_query,
            );
        }
    }

    if placement_timer.in_grace_window && !placement_timer.submitted {
        placement_timer.grace_remaining_ms = placement_timer
            .grace_remaining_ms
            .saturating_sub(elapsed_ms);

        if placement_timer.grace_remaining_ms == 0 {
            cancel_active_placement_drag(
                &mut active_drag,
                &mut commands,
                entities.drag_sprite,
                entities.fan_root,
                &mut visibility_query,
            );

            let mut submit_buttons = text_sets.p1();
            submit_pending_placements(
                &pending_placements,
                entities.submit_button,
                entities.submitted_checkmark,
                &mut submit_buttons,
                &mut submit_senders,
                &mut outbound,
                &mut placement_timer,
                &mut visibility_query,
            );
        }
    }

    {
        let mut timer_texts = text_sets.p0();
        set_timer_text(&mut timer_texts, placement_timer.remaining_ms);
    }
}

pub fn handle_draft_offering_system(
    mode: Res<HandUiMode>,
    catalog: Res<HandCardCatalog>,
    entities: Option<Res<HandUiEntities>>,
    mut offerings: MessageReader<HandUiDraftOfferingReceived>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Some(entities) = entities else {
        for _offering in offerings.read() {}
        return;
    };

    for offering in offerings.read() {
        for (index, entity) in entities.grid_slots.iter().copied().enumerate() {
            let Some(card_id) = offering.card_ids.get(index).copied() else {
                set_visibility(entity, Visibility::Hidden, &mut visibility_query);
                clear_grid_slot(&mut commands, entity);
                continue;
            };

            let Some(card) = catalog.cards.get(&card_id) else {
                warn!("Draft offering referenced unknown card id {card_id:?}");
                set_visibility(entity, Visibility::Hidden, &mut visibility_query);
                clear_grid_slot(&mut commands, entity);
                continue;
            };

            commands.entity(entity).insert((
                GridSlotCard(card_id),
                GridSlotCardName(card.name_en.clone()),
                GridSlotManaCost(card.cost),
                GridSlotState::Available,
            ));
            commands.entity(entity).remove::<PendingPurchaseTimer>();
            set_visibility(
                entity,
                visibility_for(*mode == HandUiMode::Grid),
                &mut visibility_query,
            );
        }
    }
}

pub fn handle_card_acquired_system(
    mode: Res<HandUiMode>,
    config: Res<HandFanLayoutConfig>,
    viewport: Res<HandFanViewport>,
    timing: Res<HandUiTimingConfig>,
    entities: Option<Res<HandUiEntities>>,
    mut acquisitions: MessageReader<HandUiCardAcquiredReceived>,
    mut hand_contents: ResMut<HandContents>,
    mut layout_state: ResMut<HandFanLayoutState>,
    mut commands: Commands,
    mut grid_slots: ParamSet<(
        Query<(Entity, &GridSlotCard, &mut Visibility), With<GridSlotIndex>>,
        Query<(Entity, &Visibility, Option<&GridSlotCard>), With<GridSlotIndex>>,
    )>,
    mut fan_slots: Query<
        (
            &mut Visibility,
            &mut Transform,
            &mut Node,
            Option<&mut TweenAnim>,
        ),
        (With<FanSlotIndex>, Without<GridSlotIndex>),
    >,
    mut notification: Query<
        &mut Visibility,
        (
            With<HandFullNotification>,
            Without<GridSlotIndex>,
            Without<FanSlotIndex>,
        ),
    >,
) {
    let Some(entities) = entities else {
        for _acquisition in acquisitions.read() {}
        return;
    };

    for acquisition in acquisitions.read() {
        hide_acquired_grid_slot(&mut commands, &mut grid_slots.p0(), acquisition.card_id);

        if hand_contents.cards.len() < HAND_FAN_SLOT_COUNT {
            hand_contents.cards.push(acquisition.card_id);
        }

        let hand_count = hand_contents.cards.len().min(HAND_FAN_SLOT_COUNT);
        layout_state.hand_count = if mode.shows_fan_slots() {
            hand_count
        } else {
            0
        };

        if hand_count > 0 {
            let fan_index = hand_count - 1;
            let fan_entity = entities.fan_slots[fan_index];
            if let Ok((mut visibility, mut transform, mut node, animator)) =
                fan_slots.get_mut(fan_entity)
            {
                let metrics = config.metrics_for_viewport(*viewport);
                if let Some(layout) = compute_fan_slot_layout(fan_index, hand_count, metrics) {
                    *visibility = Visibility::Visible;
                    transform.rotation = layout.bevy_rotation();
                    node.left = Val::Px(layout.card_x);
                    node.top = Val::Px(layout.card_y);
                    commands
                        .entity(fan_entity)
                        .insert(HandSlotCard(acquisition.card_id));
                    install_card_draw_animation(
                        &mut commands,
                        fan_entity,
                        animator,
                        transform.translation,
                        Vec3::new(layout.card_x, layout.card_y, transform.translation.z),
                        timing.card_draw_animation_ms,
                    );
                }
            }
        }

        if hand_contents.cards.len() >= HAND_FAN_SLOT_COUNT {
            lock_visible_grid_slots(&mut commands, &mut grid_slots.p1());
            activate_hand_full_notification(
                &mut commands,
                entities.hand_full_notification,
                &mut notification,
                timing.hand_full_notification_duration_ms,
            );
        }
    }
}

pub fn handle_ghost_clicked_unstage_system(
    mode: Res<HandUiMode>,
    mut clicks: MessageReader<GhostClickedEvent>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    fan_slots: Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    mut submit_buttons: Query<&mut Text, With<HandSubmitButton>>,
) {
    for click in clicks.read() {
        if *mode != HandUiMode::Staging {
            continue;
        }

        let should_unstage = pending_placements
            .target_for(click.card_id)
            .map(is_board_ghost_target)
            .unwrap_or(false);

        if should_unstage {
            unstage_card(
                click.card_id,
                &mut pending_placements,
                &mut ghost_writer,
                &mut commands,
                &fan_slots,
                &mut reserve_strips,
                &mut submit_buttons,
            );
        }
    }
}

pub fn handle_ghost_drag_started_system(
    mode: Res<HandUiMode>,
    mut starts: MessageReader<GhostDragStartEvent>,
    pending_placements: Res<PendingPlacements>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
) {
    for start in starts.read() {
        let should_track = *mode == HandUiMode::Staging
            && pending_placements
                .target_for(start.card_id)
                .map(is_board_ghost_target)
                .unwrap_or(false);

        if should_track {
            active_drag.clear();
            active_ghost_drag.start(start.card_id);
        } else {
            active_ghost_drag.clear();
        }
    }
}

pub fn handle_grid_card_click_system(
    mode: Res<HandUiMode>,
    timing: Res<HandUiTimingConfig>,
    mut clicks: MessageReader<HandGridCardClicked>,
    mut grid_cards: Query<(&GridSlotCard, Option<&GridSlotState>), With<GridSlotIndex>>,
    mut commands: Commands,
    mut outbound: ResMut<HandUiOutboundMessages>,
) {
    for click in clicks.read() {
        if *mode != HandUiMode::Grid {
            continue;
        }

        let Ok((card, state)) = grid_cards.get_mut(click.card) else {
            continue;
        };

        if state != Some(&GridSlotState::Available) {
            continue;
        }

        outbound
            .purchase_cards
            .push(C2SPurchaseCard { card_id: card.0 });
        commands.entity(click.card).insert((
            GridSlotState::Pending,
            PendingPurchaseTimer {
                remaining_ms: timing.purchase_timeout_ms,
            },
        ));
    }
}

pub fn handle_hand_fan_card_click_system(
    mode: Res<HandUiMode>,
    mut clicks: MessageReader<HandFanCardClicked>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    hand_cards: Query<(Entity, &FanSlotIndex, &HandSlotCard, Option<&FanSlotState>)>,
    fan_slots: Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    mut submit_buttons: Query<&mut Text, With<HandSubmitButton>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
) {
    for click in clicks.read() {
        let Ok((_entity, _slot_index, card, slot_state)) = hand_cards.get(click.card) else {
            continue;
        };

        if mode.allows_activation() {
            outbound
                .activate_cards
                .push(C2SActivateCard { card_id: card.0 });
            continue;
        }

        if *mode != HandUiMode::Staging || slot_state != Some(&FanSlotState::Ghost) {
            continue;
        }

        let is_instant_ghost = pending_placements
            .target_for(card.0)
            .map(|target| matches!(target, PlayTarget::Instant))
            .unwrap_or(false);

        if is_instant_ghost {
            unstage_card(
                card.0,
                &mut pending_placements,
                &mut ghost_writer,
                &mut commands,
                &fan_slots,
                &mut reserve_strips,
                &mut submit_buttons,
            );
        }
    }
}

pub fn handle_placement_drag_started_system(
    mode: Res<HandUiMode>,
    catalog: Res<HandCardCatalog>,
    entities: Option<Res<HandUiEntities>>,
    mut starts: MessageReader<HandUiPlacementDragStarted>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    hand_cards: Query<(&HandSlotCard, Option<&HandPlacementTargetKind>), With<FanSlotIndex>>,
    mut visibility_query: Query<&mut Visibility>,
) {
    for start in starts.read() {
        if *mode != HandUiMode::Staging {
            active_drag.clear();
            continue;
        }

        let Some(entities) = &entities else {
            active_drag.clear();
            continue;
        };

        let Ok((card, target_kind)) = hand_cards.get(start.card) else {
            active_drag.clear();
            continue;
        };

        let Some(target_kind) = resolve_placement_target_kind(card.0, target_kind, &catalog) else {
            active_drag.clear();
            continue;
        };

        active_drag.start(start.card, card.0, start.owner_id, target_kind);
        set_visibility(
            entities.drag_sprite,
            Visibility::Visible,
            &mut visibility_query,
        );
    }
}

pub fn handle_placement_cursor_moved_system(
    mut moves: MessageReader<HandUiPlacementCursorMoved>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
) {
    for cursor_move in moves.read() {
        if active_drag.is_active() {
            active_drag.cursor_world_position = cursor_move.world_position;
        }

        if active_ghost_drag.is_active() {
            active_ghost_drag.cursor_screen_position = cursor_move.world_position;
        }
    }
}

pub fn handle_placement_drag_ended_system(
    mode: Res<HandUiMode>,
    viewport: Res<HandFanViewport>,
    entities: Option<Res<HandUiEntities>>,
    mut ends: MessageReader<HandUiPlacementDragEnded>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    fan_plates: Query<&Node, With<FanPlateDropZone>>,
    mut drops: MessageWriter<HandUiPlacementDropResolved>,
) {
    for _end in ends.read() {
        if *mode == HandUiMode::Staging
            && active_drag.target_kind == Some(PlacementTargetKind::Instant)
        {
            let target = entities
                .as_ref()
                .and_then(|entities| fan_plates.get(entities.fan_root).ok())
                .and_then(|node| {
                    active_drag
                        .cursor_world_position
                        .filter(|cursor| cursor_over_fan_plate(*cursor, node, *viewport))
                })
                .map(|_cursor| PlayTarget::Instant);

            if let (Some(card), Some(owner_id)) = (active_drag.card, active_drag.owner_id) {
                drops.write(HandUiPlacementDropResolved {
                    card,
                    owner_id,
                    target,
                });
            }
        }

        active_drag.clear();
    }
}

pub fn handle_ghost_drag_ended_system(
    mode: Res<HandUiMode>,
    fan_zone_bounds: Res<FanZoneBounds>,
    mut ends: MessageReader<HandUiPlacementDragEnded>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    fan_slots: Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    mut submit_buttons: Query<&mut Text, With<HandSubmitButton>>,
) {
    for _end in ends.read() {
        let should_unstage = *mode == HandUiMode::Staging
            && active_ghost_drag
                .cursor_screen_position
                .map(|position| fan_zone_bounds.contains(position))
                .unwrap_or(false);

        if let (true, Some(card_id)) = (should_unstage, active_ghost_drag.card_id) {
            unstage_card(
                card_id,
                &mut pending_placements,
                &mut ghost_writer,
                &mut commands,
                &fan_slots,
                &mut reserve_strips,
                &mut submit_buttons,
            );
        }

        active_ghost_drag.clear();
    }
}

pub fn handle_placement_drop_resolved_system(
    mode: Res<HandUiMode>,
    entities: Option<Res<HandUiEntities>>,
    mut drops: MessageReader<HandUiPlacementDropResolved>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility>,
        Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    )>,
    fan_slots: Query<(&FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut submit_button_sets: ParamSet<(
        Query<&mut Text, With<HandSubmitButton>>,
        Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    )>,
    mut submit_senders: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        for _drop in drops.read() {}
        return;
    };

    for drop in drops.read() {
        if *mode != HandUiMode::Staging {
            continue;
        }

        active_drag.clear();
        set_visibility(
            entities.drag_sprite,
            Visibility::Hidden,
            &mut visibility_sets.p0(),
        );
        commands
            .entity(entities.fan_root)
            .remove::<FanPlateHighlighted>();

        let Ok((slot_index, card)) = fan_slots.get(drop.card) else {
            continue;
        };

        let Some(target) = drop.target.clone() else {
            commands.entity(drop.card).insert(FanSlotState::Active);
            continue;
        };

        let placement = PlacedCard {
            card_id: card.0,
            owner_id: drop.owner_id,
            target: target.clone(),
        };
        pending_placements.stage_or_update(placement);
        ghost_writer.write(GhostPlacementChanged {
            target: Some(target),
            card_id: Some(card.0),
        });
        commands.entity(drop.card).insert(FanSlotState::Ghost);
        {
            let mut submit_texts = submit_button_sets.p0();
            set_submit_count_text(&mut submit_texts, pending_placements.staged_count());
        }
        set_reserve_strip_visibility(&mut visibility_sets.p1(), slot_index.0, Visibility::Visible);

        if placement_timer.in_grace_window && !placement_timer.submitted {
            let mut submit_buttons = submit_button_sets.p1();
            submit_pending_placements(
                &pending_placements,
                entities.submit_button,
                entities.submitted_checkmark,
                &mut submit_buttons,
                &mut submit_senders,
                &mut outbound,
                &mut placement_timer,
                &mut visibility_sets.p0(),
            );
        }
    }
}

pub fn handle_submit_button_click_system(
    mode: Res<HandUiMode>,
    entities: Option<Res<HandUiEntities>>,
    mut clicks: MessageReader<HandSubmitButtonClicked>,
    pending_placements: Res<PendingPlacements>,
    mut submit_buttons: Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    mut submit_senders: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if *mode != HandUiMode::Staging || click.button != entities.submit_button {
            continue;
        }

        let submit_is_active = {
            let Ok((_text, interaction_state)) = submit_buttons.get_mut(entities.submit_button)
            else {
                continue;
            };
            *interaction_state == HandSubmitInteractionState::Active
        };

        if !submit_is_active {
            continue;
        }

        submit_pending_placements(
            &pending_placements,
            entities.submit_button,
            entities.submitted_checkmark,
            &mut submit_buttons,
            &mut submit_senders,
            &mut outbound,
            &mut placement_timer,
            &mut visibility_query,
        );
    }
}

pub fn apply_placement_drag_highlights_system(
    mode: Res<HandUiMode>,
    board_layout: Option<Res<BoardLayout>>,
    board_view: Res<PlacementBoardView>,
    catalog: Res<HandCardCatalog>,
    active_drag: Res<ActivePlacementDrag>,
    pending_placements: Res<PendingPlacements>,
    mut commands: Commands,
    board_cells: Query<(
        Entity,
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
    highlighted_cells: Query<Entity, With<BoardCellHighlighted>>,
    highlighted_fan_plates: Query<Entity, With<FanPlateHighlighted>>,
    objectives: Query<(Entity, &ObjectiveCell, Option<&ObjectiveAlive>)>,
    target_units: Query<(Entity, &PlacementTargetUnit, &GlobalTransform)>,
    hovered_units: Query<Entity, With<TargetUnitHover>>,
    mut overlays: Query<&mut Visibility, With<NoValidTargetsOverlay>>,
    entities: Option<Res<HandUiEntities>>,
) {
    if *mode != HandUiMode::Staging || !active_drag.is_active() {
        cleanup_placement_highlights(
            &mut commands,
            &highlighted_cells,
            &highlighted_fan_plates,
            &hovered_units,
            &mut overlays,
        );
        return;
    }

    if active_drag.target_kind == Some(PlacementTargetKind::Instant) {
        set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
        sync_target_unit_hover(&mut commands, &hovered_units, None);
        sync_board_cell_highlights(&mut commands, &highlighted_cells, &BTreeSet::new());
        sync_fan_plate_highlight(
            &mut commands,
            &highlighted_fan_plates,
            entities.as_ref().map(|entities| entities.fan_root),
        );
        return;
    }

    let Some(board_layout) = board_layout else {
        cleanup_placement_highlights(
            &mut commands,
            &highlighted_cells,
            &highlighted_fan_plates,
            &hovered_units,
            &mut overlays,
        );
        warn!("BoardLayout missing; placement drag highlights skipped");
        return;
    };

    let desired_highlights = match active_drag.target_kind {
        Some(PlacementTargetKind::Minion) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
            sync_target_unit_hover(&mut commands, &hovered_units, None);
            minion_highlight_cells(
                &board_layout,
                *board_view,
                &catalog,
                &pending_placements,
                &board_cells,
            )
        }
        Some(PlacementTargetKind::TargetObj) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
            sync_target_unit_hover(&mut commands, &hovered_units, None);
            target_objective_highlight_cells(*board_view, &objectives)
        }
        Some(PlacementTargetKind::LaneWide) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
            sync_target_unit_hover(&mut commands, &hovered_units, None);
            lane_wide_highlight_cells(&board_layout, &board_cells)
        }
        Some(PlacementTargetKind::TargetUnit) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            sync_board_cell_highlights(&mut commands, &highlighted_cells, &BTreeSet::new());
            sync_target_unit_highlights(
                &mut commands,
                &board_layout,
                active_drag.cursor_world_position,
                &target_units,
                &hovered_units,
                &mut overlays,
            );
            return;
        }
        Some(PlacementTargetKind::Instant) => BTreeSet::new(),
        None => BTreeSet::new(),
    };

    sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
    sync_board_cell_highlights(&mut commands, &highlighted_cells, &desired_highlights);
}

pub fn tick_pending_purchase_timeouts_system(
    mode: Res<HandUiMode>,
    time: Res<Time<Virtual>>,
    mut pending_slots: Query<(Entity, &mut GridSlotState, &mut PendingPurchaseTimer)>,
    mut commands: Commands,
) {
    if *mode != HandUiMode::Grid {
        return;
    }

    let delta_ms = elapsed_ms(time.delta());
    if delta_ms == 0 {
        return;
    }

    for (entity, mut state, mut timer) in &mut pending_slots {
        if *state != GridSlotState::Pending {
            commands.entity(entity).remove::<PendingPurchaseTimer>();
            continue;
        }

        timer.remaining_ms = timer.remaining_ms.saturating_sub(delta_ms);
        if timer.remaining_ms == 0 {
            *state = GridSlotState::Available;
            commands.entity(entity).remove::<PendingPurchaseTimer>();
        }
    }
}

pub fn tick_hand_full_notification_system(
    time: Res<Time<Virtual>>,
    mut notifications: Query<
        (Entity, &mut Visibility, &mut NotificationTimer),
        With<HandFullNotification>,
    >,
    mut commands: Commands,
) {
    let delta_ms = elapsed_ms(time.delta());
    if delta_ms == 0 {
        return;
    }

    for (entity, mut visibility, mut timer) in &mut notifications {
        timer.remaining_ms = timer.remaining_ms.saturating_sub(delta_ms);
        if timer.remaining_ms == 0 {
            *visibility = Visibility::Hidden;
            commands.entity(entity).remove::<NotificationTimer>();
        }
    }
}

fn spawn_hand_ui(mut commands: Commands, existing: Option<Res<HandUiEntities>>) {
    if existing.is_some() {
        return;
    }

    let fan_root = commands
        .spawn((
            Name::new("Hand UI Fan Root"),
            HandUiEntity,
            HandFanRoot,
            FanPlateDropZone,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                height: Val::Px(260.0),
                ..default()
            },
            Visibility::Hidden,
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands
        .entity(fan_root)
        .insert(bevy::picking::Pickable::IGNORE);

    let fan_slots = std::array::from_fn(|index| {
        commands
            .spawn((
                Name::new(format!("Hand UI Fan Slot {index}")),
                HandUiEntity,
                HandCard,
                FanSlotIndex(index as u8),
                FanSlotState::Active,
                hidden_slot_node(),
                Transform::default(),
                Visibility::Hidden,
                ChildOf(fan_root),
            ))
            .id()
    });

    let grid_slots = std::array::from_fn(|index| {
        commands
            .spawn((
                Name::new(format!("Hand UI Draft Grid Slot {index}")),
                HandUiEntity,
                GridSlotIndex(index as u8),
                hidden_slot_node(),
                Visibility::Hidden,
                ChildOf(fan_root),
            ))
            .id()
    });

    let drag_sprite = commands
        .spawn((
            Name::new("Hand UI Drag Sprite"),
            HandUiEntity,
            HandDragSprite,
            hidden_slot_node(),
            Transform::from_scale(Vec3::splat(HAND_DRAG_SPRITE_SCALE)),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    let submit_button = commands
        .spawn((
            Name::new("Hand UI Submit Button"),
            HandUiEntity,
            HandSubmitButton,
            HandSubmitInteractionState::Inactive,
            Text::new("Submit (0 cards)"),
            hidden_control_node(96.0, 28.0, 88.0),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    let timer = commands
        .spawn((
            Name::new("Hand UI Placement Timer"),
            HandUiEntity,
            HandTimer,
            TimerState::Normal,
            Text::new(""),
            hidden_control_node(64.0, 28.0, 128.0),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    let submitted_checkmark = commands
        .spawn((
            Name::new("Hand UI Timer Submitted Checkmark"),
            HandUiEntity,
            TimerSubmittedCheckmark,
            Text::new("OK"),
            hidden_control_node(24.0, 28.0, 128.0),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    let hand_full_notification = commands
        .spawn((
            Name::new("Hand UI Hand Full Notification"),
            HandUiEntity,
            HandFullNotification,
            Text::new("Hand full"),
            hidden_control_node(120.0, 28.0, 168.0),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    let no_valid_targets_overlay = commands
        .spawn((
            Name::new("Hand UI No Valid Targets Overlay"),
            HandUiEntity,
            NoValidTargetsOverlay,
            hidden_control_node(180.0, 28.0, 208.0),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    commands.insert_resource(HandUiEntities {
        fan_root,
        fan_slots,
        grid_slots,
        drag_sprite,
        submit_button,
        timer,
        submitted_checkmark,
        hand_full_notification,
        no_valid_targets_overlay,
    });
}

fn despawn_hand_ui(mut commands: Commands, entities: Option<Res<HandUiEntities>>) {
    let Some(entities) = entities else {
        return;
    };

    commands.entity(entities.fan_root).despawn();
    commands.remove_resource::<HandUiEntities>();
}

fn hidden_slot_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Px(0.0),
        height: Val::Px(0.0),
        ..default()
    }
}

fn hidden_control_node(width_px: f32, height_px: f32, bottom_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Px(width_px),
        height: Val::Px(height_px),
        bottom: Val::Px(bottom_px),
        ..default()
    }
}

fn visibility_for(visible: bool) -> Visibility {
    if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

fn set_visibility(
    entity: Entity,
    visibility: Visibility,
    visibility_query: &mut Query<&mut Visibility>,
) {
    if let Ok(mut current) = visibility_query.get_mut(entity) {
        *current = visibility;
    }
}

fn set_submit_count_text(
    submit_buttons: &mut Query<&mut Text, With<HandSubmitButton>>,
    count: usize,
) {
    for mut text in submit_buttons.iter_mut() {
        text.0.clear();
        text.0.push_str(&format!("Submit ({count} cards)"));
    }
}

fn set_timer_text(timer_texts: &mut Query<&mut Text, With<HandTimer>>, remaining_ms: u32) {
    let seconds = if remaining_ms == 0 {
        0
    } else {
        remaining_ms.saturating_add(999) / 1000
    };

    for mut text in timer_texts.iter_mut() {
        text.0.clear();
        text.0.push_str(&seconds.to_string());
    }
}

fn submit_pending_placements(
    pending_placements: &PendingPlacements,
    submit_button: Entity,
    submitted_checkmark: Entity,
    submit_buttons: &mut Query<
        (&mut Text, &mut HandSubmitInteractionState),
        With<HandSubmitButton>,
    >,
    submit_senders: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
    outbound: &mut HandUiOutboundMessages,
    placement_timer: &mut PlacementTimer,
    visibility_query: &mut Query<&mut Visibility>,
) -> bool {
    if placement_timer.submitted {
        return false;
    }

    let Ok((mut text, mut interaction_state)) = submit_buttons.get_mut(submit_button) else {
        return false;
    };

    if *interaction_state != HandSubmitInteractionState::Active {
        return false;
    }

    let msg = C2SSubmitPlacement {
        placements: pending_placements.placements.clone(),
    };
    if let Ok(mut sender) = submit_senders.single_mut() {
        sender.send::<ReliableChannel>(msg.clone());
    }
    outbound.submit_placements.push(msg);
    placement_timer.submitted = true;
    placement_timer.in_grace_window = false;
    placement_timer.grace_remaining_ms = 0;
    *interaction_state = HandSubmitInteractionState::Inactive;
    text.0.clear();
    text.0.push_str("Submitted");
    set_visibility(submitted_checkmark, Visibility::Visible, visibility_query);
    true
}

fn cancel_active_placement_drag(
    active_drag: &mut ActivePlacementDrag,
    commands: &mut Commands,
    drag_sprite: Entity,
    fan_root: Entity,
    visibility_query: &mut Query<&mut Visibility>,
) {
    if let Some(card) = active_drag.card {
        commands.entity(card).insert(FanSlotState::Active);
    }

    active_drag.clear();
    set_visibility(drag_sprite, Visibility::Hidden, visibility_query);
    commands.entity(fan_root).remove::<FanPlateHighlighted>();
}

fn set_reserve_strip_visibility(
    reserve_strips: &mut Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    slot_index: u8,
    visibility: Visibility,
) {
    for (reserve_slot, mut reserve_visibility) in reserve_strips.iter_mut() {
        if reserve_slot.0 == slot_index {
            *reserve_visibility = visibility;
        }
    }
}

fn unstage_card(
    card_id: CardId,
    pending_placements: &mut PendingPlacements,
    ghost_writer: &mut MessageWriter<GhostPlacementChanged>,
    commands: &mut Commands,
    fan_slots: &Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    reserve_strips: &mut Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    submit_buttons: &mut Query<&mut Text, With<HandSubmitButton>>,
) -> bool {
    let Some((slot_entity, slot_index)) = fan_slot_for_card(fan_slots, card_id) else {
        return false;
    };

    if pending_placements.remove_staged(card_id).is_none() {
        return false;
    }

    ghost_writer.write(GhostPlacementChanged {
        target: None,
        card_id: Some(card_id),
    });
    commands.entity(slot_entity).insert(FanSlotState::Active);
    set_submit_count_text(submit_buttons, pending_placements.staged_count());
    set_reserve_strip_visibility(reserve_strips, slot_index, Visibility::Hidden);
    true
}

fn fan_slot_for_card(
    fan_slots: &Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    card_id: CardId,
) -> Option<(Entity, u8)> {
    fan_slots.iter().find_map(|(entity, slot_index, card)| {
        (card.0 == card_id).then_some((entity, slot_index.0))
    })
}

fn is_board_ghost_target(target: &PlayTarget) -> bool {
    matches!(
        target,
        PlayTarget::BoardCell { .. }
            | PlayTarget::TargetUnit { .. }
            | PlayTarget::TargetObj { .. }
            | PlayTarget::LaneWide { .. }
    )
}

fn resolve_placement_target_kind(
    card_id: CardId,
    target_kind: Option<&HandPlacementTargetKind>,
    catalog: &HandCardCatalog,
) -> Option<PlacementTargetKind> {
    if let Some(target_kind) = target_kind {
        return Some(target_kind.0);
    }

    let card = catalog.cards.get(&card_id)?;
    match card.card_type {
        CardType::Minion => Some(PlacementTargetKind::Minion),
        CardType::Field => Some(PlacementTargetKind::LaneWide),
        CardType::Order => Some(PlacementTargetKind::Instant),
        CardType::Spell | CardType::Trap | CardType::Structure | CardType::DoubleFace => None,
    }
}

fn minion_highlight_cells(
    board_layout: &BoardLayout,
    board_view: PlacementBoardView,
    catalog: &HandCardCatalog,
    pending_placements: &PendingPlacements,
    board_cells: &Query<(
        Entity,
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
) -> BTreeSet<Entity> {
    let staged_minion_cells = staged_minion_cells(catalog, pending_placements);
    board_cells
        .iter()
        .filter_map(|(entity, lane_cell, occupied, objective)| {
            let valid_cell = board_layout
                .cell_to_world(lane_cell.lane, lane_cell.cell)
                .is_some()
                && board_view.is_spawn_cell(lane_cell.lane, lane_cell.cell)
                && occupied.is_none()
                && objective.is_none()
                && !staged_minion_cells.contains(&(lane_cell.lane, lane_cell.cell));

            valid_cell.then_some(entity)
        })
        .collect()
}

fn target_objective_highlight_cells(
    board_view: PlacementBoardView,
    objectives: &Query<(Entity, &ObjectiveCell, Option<&ObjectiveAlive>)>,
) -> BTreeSet<Entity> {
    objectives
        .iter()
        .filter_map(|(entity, objective, alive)| {
            (objective.player_id == board_view.opponent_player_id && alive.is_some())
                .then_some(entity)
        })
        .collect()
}

fn lane_wide_highlight_cells(
    board_layout: &BoardLayout,
    board_cells: &Query<(
        Entity,
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
) -> BTreeSet<Entity> {
    board_cells
        .iter()
        .filter_map(|(entity, lane_cell, _occupied, objective)| {
            let valid_cell = board_layout
                .cell_to_world(lane_cell.lane, lane_cell.cell)
                .is_some()
                && objective.is_none();

            valid_cell.then_some(entity)
        })
        .collect()
}

fn staged_minion_cells(
    catalog: &HandCardCatalog,
    pending_placements: &PendingPlacements,
) -> BTreeSet<(u8, u8)> {
    pending_placements
        .placements
        .iter()
        .filter_map(|placement| {
            let card = catalog.cards.get(&placement.card_id)?;
            let PlayTarget::BoardCell { lane, cell } = &placement.target else {
                return None;
            };

            (card.card_type == CardType::Minion).then_some((*lane, *cell))
        })
        .collect()
}

fn cleanup_placement_highlights(
    commands: &mut Commands,
    highlighted_cells: &Query<Entity, With<BoardCellHighlighted>>,
    highlighted_fan_plates: &Query<Entity, With<FanPlateHighlighted>>,
    hovered_units: &Query<Entity, With<TargetUnitHover>>,
    overlays: &mut Query<&mut Visibility, With<NoValidTargetsOverlay>>,
) {
    sync_board_cell_highlights(commands, highlighted_cells, &BTreeSet::new());
    sync_fan_plate_highlight(commands, highlighted_fan_plates, None);
    sync_target_unit_hover(commands, hovered_units, None);
    set_no_valid_targets_overlay(overlays, Visibility::Hidden);
}

fn sync_board_cell_highlights(
    commands: &mut Commands,
    highlighted_cells: &Query<Entity, With<BoardCellHighlighted>>,
    desired: &BTreeSet<Entity>,
) {
    let current = highlighted_cells.iter().collect::<BTreeSet<_>>();

    for entity in current.difference(desired) {
        commands.entity(*entity).remove::<BoardCellHighlighted>();
    }

    for entity in desired.difference(&current) {
        commands.entity(*entity).insert(BoardCellHighlighted);
    }
}

fn sync_fan_plate_highlight(
    commands: &mut Commands,
    highlighted_fan_plates: &Query<Entity, With<FanPlateHighlighted>>,
    desired: Option<Entity>,
) {
    for entity in highlighted_fan_plates.iter() {
        if Some(entity) != desired {
            commands.entity(entity).remove::<FanPlateHighlighted>();
        }
    }

    if let Some(entity) = desired {
        commands.entity(entity).insert(FanPlateHighlighted);
    }
}

fn cursor_over_fan_plate(
    cursor_screen_position: Vec2,
    node: &Node,
    viewport: HandFanViewport,
) -> bool {
    let Some(left) = val_px(node.left) else {
        return false;
    };
    let Some(right) = val_px(node.right) else {
        return false;
    };
    let Some(bottom) = val_px(node.bottom) else {
        return false;
    };
    let Some(height) = val_px(node.height) else {
        return false;
    };

    if viewport.width_px <= left + right || viewport.height_px <= bottom || height <= 0.0 {
        return false;
    }

    let min_x = left;
    let max_x = viewport.width_px - right;
    let max_y = viewport.height_px - bottom;
    let min_y = (max_y - height).max(0.0);

    (min_x..=max_x).contains(&cursor_screen_position.x)
        && (min_y..=max_y).contains(&cursor_screen_position.y)
}

fn val_px(value: Val) -> Option<f32> {
    match value {
        Val::Px(px) => Some(px),
        _ => None,
    }
}

fn sync_target_unit_highlights(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    cursor_world_position: Option<Vec2>,
    target_units: &Query<(Entity, &PlacementTargetUnit, &GlobalTransform)>,
    hovered_units: &Query<Entity, With<TargetUnitHover>>,
    overlays: &mut Query<&mut Visibility, With<NoValidTargetsOverlay>>,
) {
    let mut hovered = None;
    let mut valid_count = 0;

    for (entity, _target_unit, transform) in target_units.iter() {
        valid_count += 1;
        if hovered.is_none()
            && cursor_world_position
                .map(|cursor| cursor_over_unit(cursor, transform, board_layout))
                .unwrap_or(false)
        {
            hovered = Some(entity);
        }
    }

    if valid_count == 0 {
        sync_target_unit_hover(commands, hovered_units, None);
        set_no_valid_targets_overlay(overlays, Visibility::Visible);
        return;
    }

    set_no_valid_targets_overlay(overlays, Visibility::Hidden);
    sync_target_unit_hover(commands, hovered_units, hovered);
}

fn cursor_over_unit(
    cursor_world_position: Vec2,
    transform: &GlobalTransform,
    board_layout: &BoardLayout,
) -> bool {
    let unit_position = transform.translation().truncate();
    let half_width = board_layout.cell_width * 0.5;
    let half_height = board_layout.lane_height * 0.5;

    (cursor_world_position.x - unit_position.x).abs() <= half_width
        && (cursor_world_position.y - unit_position.y).abs() <= half_height
}

fn sync_target_unit_hover(
    commands: &mut Commands,
    hovered_units: &Query<Entity, With<TargetUnitHover>>,
    desired: Option<Entity>,
) {
    for entity in hovered_units.iter() {
        if Some(entity) != desired {
            commands.entity(entity).remove::<TargetUnitHover>();
        }
    }

    if let Some(entity) = desired {
        commands.entity(entity).insert(TargetUnitHover);
    }
}

fn set_no_valid_targets_overlay(
    overlays: &mut Query<&mut Visibility, With<NoValidTargetsOverlay>>,
    visibility: Visibility,
) {
    for mut overlay_visibility in overlays.iter_mut() {
        *overlay_visibility = visibility;
    }
}

fn clear_grid_slot(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<(
        GridSlotCard,
        GridSlotCardName,
        GridSlotManaCost,
        GridSlotState,
        PendingPurchaseTimer,
    )>();
}

fn hide_acquired_grid_slot(
    commands: &mut Commands,
    grid_slots: &mut Query<(Entity, &GridSlotCard, &mut Visibility), With<GridSlotIndex>>,
    card_id: CardId,
) {
    for (entity, card, mut visibility) in grid_slots.iter_mut() {
        if card.0 != card_id {
            continue;
        }

        *visibility = Visibility::Hidden;
        clear_grid_slot(commands, entity);
        break;
    }
}

fn install_card_draw_animation(
    commands: &mut Commands,
    entity: Entity,
    animator: Option<Mut<TweenAnim>>,
    start: Vec3,
    end: Vec3,
    duration_ms: u64,
) {
    let tween = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(duration_ms),
        TransformPositionLens { start, end },
    );

    if let Some(mut animator) = animator {
        if let Err(error) = replace_tweenable(&mut animator, tween) {
            warn!("Failed to replace Hand UI card draw tween on entity {entity:?}: {error}");
        }
    } else {
        commands.entity(entity).insert(make_tween_anim(tween));
    }
}

fn lock_visible_grid_slots(
    commands: &mut Commands,
    grid_slots: &mut Query<(Entity, &Visibility, Option<&GridSlotCard>), With<GridSlotIndex>>,
) {
    for (entity, visibility, card) in grid_slots.iter_mut() {
        if *visibility != Visibility::Visible || card.is_none() {
            continue;
        }

        commands
            .entity(entity)
            .insert(GridSlotState::HandFullLocked);
        commands.entity(entity).remove::<PendingPurchaseTimer>();
    }
}

fn activate_hand_full_notification<F: QueryFilter>(
    commands: &mut Commands,
    entity: Entity,
    notifications: &mut Query<&mut Visibility, F>,
    duration_ms: u64,
) {
    if let Ok(mut visibility) = notifications.get_mut(entity) {
        *visibility = Visibility::Visible;
    }
    commands.entity(entity).insert(NotificationTimer {
        remaining_ms: duration_ms,
    });
}

fn elapsed_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn cancel_hand_ui_tweens(
    commands: &mut Commands,
    animators: &mut Query<(Entity, &mut TweenAnim), With<HandUiEntity>>,
) {
    for (entity, mut animator) in animators.iter_mut() {
        if let Err(error) = cancel_tween_anim_in_place(&mut animator) {
            warn!("Failed to cancel Hand UI tween on entity {entity:?}: {error}");
        }
        commands.entity(entity).remove::<TweenAnim>();
    }
}
