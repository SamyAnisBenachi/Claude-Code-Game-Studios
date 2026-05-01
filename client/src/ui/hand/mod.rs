use bevy::prelude::*;

use crate::card_animations::{HandCard, HandDragSprite};
use crate::state::ClientState;

pub const HAND_FAN_SLOT_COUNT: usize = 10;
pub const DRAFT_INITIAL_GRID_SLOT_COUNT: usize = 9;
pub const HAND_UI_ENTITY_COUNT: usize = HAND_FAN_SLOT_COUNT + DRAFT_INITIAL_GRID_SLOT_COUNT + 1;

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
pub enum HandSubmitInteractionState {
    Active,
    Inactive,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct HandUiEntities {
    pub fan_slots: [Entity; HAND_FAN_SLOT_COUNT],
    pub grid_slots: [Entity; DRAFT_INITIAL_GRID_SLOT_COUNT],
    pub drag_sprite: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotIndex(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSlotIndex(pub u8);

pub struct HandUiPlugin;

impl Plugin for HandUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .init_resource::<HandFanLayoutConfig>()
            .init_resource::<HandFanViewport>()
            .init_resource::<HandFanLayoutState>()
            .add_systems(OnEnter(ClientState::InSession), spawn_hand_ui)
            .add_systems(OnExit(ClientState::InSession), despawn_hand_ui)
            .add_systems(
                Update,
                apply_fan_layout_system.run_if(in_state(ClientState::InSession)),
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
    mut submit_buttons: Query<
        (&mut Visibility, &mut Text, &mut HandSubmitInteractionState),
        With<HandSubmitButton>,
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

    if hand_count == 0 {
        for (mut visibility, mut text, mut interaction_state) in &mut submit_buttons {
            *visibility = Visibility::Visible;
            text.0.clear();
            text.0.push_str("Submit (0 cards)");
            *interaction_state = HandSubmitInteractionState::Active;
        }
    }
}

fn spawn_hand_ui(mut commands: Commands, existing: Option<Res<HandUiEntities>>) {
    if existing.is_some() {
        return;
    }

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
        ))
        .id();

    commands.insert_resource(HandUiEntities {
        fan_slots,
        grid_slots,
        drag_sprite,
    });
}

fn despawn_hand_ui(mut commands: Commands, entities: Option<Res<HandUiEntities>>) {
    let Some(entities) = entities else {
        return;
    };

    for entity in entities.fan_slots {
        commands.entity(entity).despawn();
    }
    for entity in entities.grid_slots {
        commands.entity(entity).despawn();
    }
    commands.entity(entities.drag_sprite).despawn();
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
