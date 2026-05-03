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
pub const HAND_UI_ENTITY_COUNT: usize = HAND_FAN_SLOT_COUNT + DRAFT_INITIAL_GRID_SLOT_COUNT + 6;
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

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSubmitButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct GhostPlacementChanged {
    pub target: Option<PlayTarget>,
    pub card_id: Option<CardId>,
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
            .init_resource::<HandUiEconomyView>()
            .init_resource::<HandContents>()
            .init_resource::<HandUiMode>()
            .init_resource::<HandUiOutboundMessages>()
            .init_resource::<PendingPlacements>()
            .init_resource::<PlacementBoardView>()
            .init_resource::<ActivePlacementDrag>()
            .add_message::<HandFanCardClicked>()
            .add_message::<HandGridCardClicked>()
            .add_message::<HandUiDraftOfferingReceived>()
            .add_message::<HandUiCardAcquiredReceived>()
            .add_message::<HandUiPlacementDragStarted>()
            .add_message::<HandUiPlacementCursorMoved>()
            .add_message::<HandUiPlacementDragEnded>()
            .add_message::<HandUiPlacementDropResolved>()
            .add_message::<HandSubmitButtonClicked>()
            .add_message::<GhostPlacementChanged>()
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
                    (handle_draft_offering_system, handle_card_acquired_system)
                        .chain()
                        .in_set(HandUiSystemSet::MessageDrain),
                    (
                        handle_placement_drag_started_system,
                        handle_placement_cursor_moved_system,
                        handle_placement_drag_ended_system,
                        handle_grid_card_click_system,
                        handle_hand_fan_card_click_system,
                        handle_placement_drop_resolved_system,
                        handle_submit_button_click_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::Input),
                    (
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
    mut mode: ResMut<HandUiMode>,
    mut layout_state: ResMut<HandFanLayoutState>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    entities: Option<Res<HandUiEntities>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
    mut submit_buttons: Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    mut animators: Query<(Entity, &mut TweenAnim), With<HandUiEntity>>,
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
        active_drag.clear();
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
        for (mut text, mut interaction_state) in &mut submit_buttons {
            text.0.clear();
            text.0.push_str("Submit (0 cards)");
            *interaction_state = HandSubmitInteractionState::Active;
        }
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
    hand_cards: Query<&HandSlotCard, With<FanSlotIndex>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
) {
    for click in clicks.read() {
        if !mode.allows_activation() {
            continue;
        }

        let Ok(card) = hand_cards.get(click.card) else {
            continue;
        };

        outbound
            .activate_cards
            .push(C2SActivateCard { card_id: card.0 });
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
) {
    for cursor_move in moves.read() {
        if active_drag.is_active() {
            active_drag.cursor_world_position = cursor_move.world_position;
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

pub fn handle_placement_drop_resolved_system(
    mode: Res<HandUiMode>,
    entities: Option<Res<HandUiEntities>>,
    mut drops: MessageReader<HandUiPlacementDropResolved>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility>,
        Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    )>,
    fan_slots: Query<(&FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut submit_buttons: Query<&mut Text, With<HandSubmitButton>>,
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
        set_submit_count_text(&mut submit_buttons, pending_placements.staged_count());
        set_reserve_strip_visibility(&mut visibility_sets.p1(), slot_index.0, Visibility::Visible);
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
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if *mode != HandUiMode::Staging || click.button != entities.submit_button {
            continue;
        }

        let Ok((mut text, mut interaction_state)) = submit_buttons.get_mut(entities.submit_button)
        else {
            continue;
        };

        if *interaction_state != HandSubmitInteractionState::Active {
            continue;
        }

        let msg = C2SSubmitPlacement {
            placements: pending_placements.placements.clone(),
        };
        if let Ok(mut sender) = submit_senders.single_mut() {
            sender.send::<ReliableChannel>(msg.clone());
        }
        outbound.submit_placements.push(msg);
        *interaction_state = HandSubmitInteractionState::Inactive;
        text.0.clear();
        text.0.push_str("Submitted");
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
            Text::new(""),
            hidden_control_node(64.0, 28.0, 128.0),
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
