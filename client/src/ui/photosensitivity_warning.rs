use bevy::prelude::*;

use crate::ui::design_tokens::z_layers;

pub const PHOTOSENSITIVITY_WARNING_TITLE: &str = "Photosensitivity Warning";
pub const PHOTOSENSITIVITY_WARNING_COPY: &str = "Lanes and Lies uses brief impact flashes, timer urgency effects, objective-destruction bursts, and phase transitions. Stop playing and consult a medical professional if you feel discomfort, dizziness, eye twitching, or nausea.";
pub const PHOTOSENSITIVITY_WARNING_ACKNOWLEDGE_COPY: &str = "I understand";

pub struct PhotosensitivityWarningPlugin;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhotosensitivityWarningState {
    acknowledged: bool,
}

impl PhotosensitivityWarningState {
    pub fn is_acknowledged(&self) -> bool {
        self.acknowledged
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotosensitivityWarningEntities {
    pub root: Entity,
    pub title: Entity,
    pub body: Entity,
    pub acknowledge: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotosensitivityWarningAcknowledged;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotosensitivityWarningRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotosensitivityWarningTitle;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotosensitivityWarningBody;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotosensitivityWarningAcknowledge;

impl Plugin for PhotosensitivityWarningPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("PhotosensitivityWarningPlugin loaded");
        app.init_resource::<PhotosensitivityWarningState>()
            .add_message::<PhotosensitivityWarningAcknowledged>()
            .add_systems(Startup, spawn_photosensitivity_warning)
            .add_systems(
                Update,
                (
                    acknowledge_photosensitivity_warning_from_message,
                    acknowledge_photosensitivity_warning_from_interaction,
                ),
            );
    }
}

pub fn spawn_photosensitivity_warning(
    mut commands: Commands,
    state: Res<PhotosensitivityWarningState>,
) {
    if state.is_acknowledged() {
        return;
    }

    let root = commands
        .spawn((
            Name::new("Photosensitivity Warning Root"),
            PhotosensitivityWarningRoot,
            photosensitivity_warning_root_node(),
            BackgroundColor(Color::srgba(0.04, 0.04, 0.08, 0.94)),
            Visibility::Visible,
            z_layers::MODAL,
        ))
        .id();

    let panel = commands
        .spawn((
            Name::new("Photosensitivity Warning Panel"),
            photosensitivity_warning_panel_node(),
            BackgroundColor(Color::srgb(0.08, 0.12, 0.18)),
            BorderColor::all(Color::srgb(0.92, 0.94, 0.96)),
            ChildOf(root),
        ))
        .id();

    let title = commands
        .spawn((
            Name::new("Photosensitivity Warning Title"),
            PhotosensitivityWarningTitle,
            Text::new(PHOTOSENSITIVITY_WARNING_TITLE),
            warning_text_font(24.0),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
            warning_text_node(),
            ChildOf(panel),
        ))
        .id();

    let body = commands
        .spawn((
            Name::new("Photosensitivity Warning Body"),
            PhotosensitivityWarningBody,
            Text::new(PHOTOSENSITIVITY_WARNING_COPY),
            warning_text_font(15.0),
            TextColor(Color::srgb(0.92, 0.94, 0.96)),
            warning_body_node(),
            ChildOf(panel),
        ))
        .id();

    let acknowledge = commands
        .spawn((
            Name::new("Photosensitivity Warning Acknowledge"),
            PhotosensitivityWarningAcknowledge,
            Interaction::None,
            Text::new(PHOTOSENSITIVITY_WARNING_ACKNOWLEDGE_COPY),
            warning_text_font(16.0),
            TextColor(Color::srgb(0.08, 0.12, 0.18)),
            warning_acknowledge_node(),
            BackgroundColor(Color::srgb(0.98, 0.78, 0.26)),
            ChildOf(panel),
        ))
        .id();

    commands.insert_resource(PhotosensitivityWarningEntities {
        root,
        title,
        body,
        acknowledge,
    });
}

pub fn acknowledge_photosensitivity_warning_from_message(
    mut acknowledgements: MessageReader<PhotosensitivityWarningAcknowledged>,
    mut state: ResMut<PhotosensitivityWarningState>,
    entities: Option<Res<PhotosensitivityWarningEntities>>,
    mut roots: Query<&mut Visibility, With<PhotosensitivityWarningRoot>>,
) {
    let acknowledged = acknowledgements.read().next().is_some();
    if acknowledged {
        acknowledge_photosensitivity_warning(&mut state, entities.as_deref(), &mut roots);
    }
}

pub fn acknowledge_photosensitivity_warning_from_interaction(
    mut state: ResMut<PhotosensitivityWarningState>,
    entities: Option<Res<PhotosensitivityWarningEntities>>,
    interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<PhotosensitivityWarningAcknowledge>,
        ),
    >,
    mut roots: Query<&mut Visibility, With<PhotosensitivityWarningRoot>>,
) {
    if interactions
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        acknowledge_photosensitivity_warning(&mut state, entities.as_deref(), &mut roots);
    }
}

fn acknowledge_photosensitivity_warning(
    state: &mut PhotosensitivityWarningState,
    entities: Option<&PhotosensitivityWarningEntities>,
    roots: &mut Query<&mut Visibility, With<PhotosensitivityWarningRoot>>,
) {
    state.acknowledged = true;

    let Some(entities) = entities else {
        return;
    };

    if let Ok(mut visibility) = roots.get_mut(entities.root) {
        *visibility = Visibility::Hidden;
    }
}

fn photosensitivity_warning_root_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(24.0)),
        ..default()
    }
}

fn photosensitivity_warning_panel_node() -> Node {
    Node {
        width: Val::Px(560.0),
        max_width: Val::Percent(92.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(14.0),
        padding: UiRect::all(Val::Px(20.0)),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn warning_text_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        ..default()
    }
}

fn warning_body_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        ..default()
    }
}

fn warning_acknowledge_node() -> Node {
    Node {
        width: Val::Px(144.0),
        height: Val::Px(44.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(8.0)),
        margin: UiRect::top(Val::Px(4.0)),
        ..default()
    }
}

fn warning_text_font(font_size: f32) -> TextFont {
    TextFont {
        font_size,
        ..default()
    }
}
