use std::time::Duration;

use bevy::ecs::query::QueryFilter;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, Tween, TweenAnim};
use shared::card::{CardCatalog, CardId};
use shared::protocol::{C2SActivateCard, C2SPurchaseCard, RoundPhase};

use crate::card_animations::{
    cancel_tween_anim_in_place, make_tween_anim, replace_tweenable, HandCard, HandDragSprite,
};
use crate::state::{ClientState, CurrentClientPhase};

pub const HAND_FAN_SLOT_COUNT: usize = 10;
pub const DRAFT_INITIAL_GRID_SLOT_COUNT: usize = 9;
pub const HAND_UI_ENTITY_COUNT: usize = HAND_FAN_SLOT_COUNT + DRAFT_INITIAL_GRID_SLOT_COUNT + 5;

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
}

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

#[derive(Resource, Debug, Clone, Copy)]
pub struct HandUiEntities {
    pub fan_root: Entity,
    pub fan_slots: [Entity; HAND_FAN_SLOT_COUNT],
    pub grid_slots: [Entity; DRAFT_INITIAL_GRID_SLOT_COUNT],
    pub drag_sprite: Entity,
    pub submit_button: Entity,
    pub timer: Entity,
    pub hand_full_notification: Entity,
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
            .add_message::<HandFanCardClicked>()
            .add_message::<HandGridCardClicked>()
            .add_message::<HandUiDraftOfferingReceived>()
            .add_message::<HandUiCardAcquiredReceived>()
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
                        handle_grid_card_click_system,
                        handle_hand_fan_card_click_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::Input),
                    (
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
    entities: Option<Res<HandUiEntities>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
    mut submit_buttons: Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    mut animators: Query<(Entity, &mut TweenAnim), With<HandUiEntity>>,
) {
    if !current.is_changed() && !hand_contents.is_changed() {
        return;
    }

    let Some(entities) = entities else {
        return;
    };

    let next_mode = HandUiMode::from_phase(current.phase);
    *mode = next_mode;
    layout_state.hand_count = if next_mode.shows_fan_slots() {
        hand_contents.cards.len().min(HAND_FAN_SLOT_COUNT)
    } else {
        0
    };

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
            } else {
                commands.entity(entity).remove::<HandSlotCard>();
            }
        } else {
            set_visibility(entity, Visibility::Hidden, &mut visibility_query);
            commands.entity(entity).remove::<HandSlotCard>();
        }
    }

    if next_mode == HandUiMode::Staging {
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

    let fan_slots = std::array::from_fn(|index| {
        commands
            .spawn((
                Name::new(format!("Hand UI Fan Slot {index}")),
                HandUiEntity,
                HandCard,
                FanSlotIndex(index as u8),
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

    commands.insert_resource(HandUiEntities {
        fan_root,
        fan_slots,
        grid_slots,
        drag_sprite,
        submit_button,
        timer,
        hand_full_notification,
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
