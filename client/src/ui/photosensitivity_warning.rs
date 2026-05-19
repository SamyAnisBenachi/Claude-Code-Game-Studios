use bevy::prelude::*;
use bevy::ui::Overflow;

use crate::state::ClientState;
use crate::ui::design_tokens::{typography, z_layers};

pub const PHOTOSENSITIVITY_WARNING_TITLE: &str = "Photosensitivity Warning";
pub const PHOTOSENSITIVITY_WARNING_COPY: &str = "Lanes and Lies uses brief impact flashes, timer urgency effects, objective-destruction bursts, and phase transitions. Stop playing and consult a medical professional if you feel discomfort, dizziness, eye twitching, or nausea.";
pub const PHOTOSENSITIVITY_WARNING_ACKNOWLEDGE_COPY: &str = "I understand";

/// PROMPT 1349 (Sprint 18 story 026 / Lane J) — modal-overflow contract
/// shared with `result_screen.rs` (PROMPT 1180 §1.5 O-04 template) and
/// every modal in the in-scope set: panels must declare
/// `max_height: Val::Percent(92.0)` + `overflow: Overflow::scroll_y()`
/// per `production/epics/ui-clean-pass/story-026-ui-overlay-panel-overflow-hardening.md`
/// §5 C-5.
pub const PHOTOSENSITIVITY_PANEL_MAX_HEIGHT_PERCENT: f32 = 92.0;
const PHOTOSENSITIVITY_PANEL_PADDING_PX: f32 = 20.0;
const PHOTOSENSITIVITY_FOOTER_HEIGHT_PX: f32 = 44.0;
const PHOTOSENSITIVITY_FOOTER_TOP_GAP_PX: f32 = 14.0;

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

/// PROMPT 1349 — footer slot wrapping the Acknowledge button. Anchored
/// to the panel's bottom edge via `position_type: Absolute` so the
/// Acknowledge stays in view regardless of body length / panel scroll.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotosensitivityWarningFooter;

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
            )
            .add_systems(
                OnEnter(ClientState::InSession),
                hide_photosensitivity_warning_on_session_entry,
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
            BackgroundColor(Color::srgb(0.04, 0.04, 0.08)),
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
            warning_text_font(typography::H2),
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
            warning_text_font(typography::BODY),
            TextColor(Color::srgb(0.92, 0.94, 0.96)),
            warning_body_node(),
            ChildOf(panel),
        ))
        .id();

    // PROMPT 1349 — footer slot anchored to the panel's bottom edge.
    // The Acknowledge button lives inside this absolute footer so it
    // stays in view regardless of body length or panel scroll position
    // (story 026 AC1).
    let footer = commands
        .spawn((
            Name::new("Photosensitivity Warning Footer"),
            PhotosensitivityWarningFooter,
            warning_footer_node(),
            ChildOf(panel),
        ))
        .id();

    let acknowledge = commands
        .spawn((
            Name::new("Photosensitivity Warning Acknowledge"),
            PhotosensitivityWarningAcknowledge,
            Interaction::None,
            Text::new(PHOTOSENSITIVITY_WARNING_ACKNOWLEDGE_COPY),
            warning_text_font(typography::BODY),
            TextColor(Color::srgb(0.08, 0.12, 0.18)),
            warning_acknowledge_node(),
            BackgroundColor(Color::srgb(0.98, 0.78, 0.26)),
            ChildOf(footer),
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

/// Hide the photosensitivity warning when the client enters [`ClientState::InSession`].
///
/// The warning must never paint over active gameplay (PROMPT 1022 visual audit
/// found it appearing over `DraftInitial` and `Placement` UI). Once a session
/// starts, the warning is considered dismissed for this app run regardless of
/// whether the player clicked the acknowledgement button.
pub fn hide_photosensitivity_warning_on_session_entry(
    mut state: ResMut<PhotosensitivityWarningState>,
    entities: Option<Res<PhotosensitivityWarningEntities>>,
    mut roots: Query<&mut Visibility, With<PhotosensitivityWarningRoot>>,
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

/// PROMPT 1349 (Sprint 18 story 026 / Lane J) — panel-node builder for
/// the photosensitivity warning modal. Declares the modal-overflow
/// contract: `max_height: 92%` + `Overflow::scroll_y()` so the
/// panel never clips its own viewport at any supported resolution. The
/// panel is `position_type: Relative` so the absolute `footer` child
/// anchors to the panel's padding-box rather than to the viewport.
pub fn photosensitivity_warning_panel_node() -> Node {
    Node {
        width: Val::Px(560.0),
        max_width: Val::Percent(92.0),
        max_height: Val::Percent(PHOTOSENSITIVITY_PANEL_MAX_HEIGHT_PERCENT),
        overflow: Overflow::scroll_y(),
        position_type: PositionType::Relative,
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(PHOTOSENSITIVITY_FOOTER_TOP_GAP_PX),
        // Reserve bottom padding for the absolute footer so flex-flow
        // children (title + body) never overlap the Acknowledge button.
        padding: UiRect {
            top: Val::Px(PHOTOSENSITIVITY_PANEL_PADDING_PX),
            right: Val::Px(PHOTOSENSITIVITY_PANEL_PADDING_PX),
            bottom: Val::Px(
                PHOTOSENSITIVITY_PANEL_PADDING_PX
                    + PHOTOSENSITIVITY_FOOTER_HEIGHT_PX
                    + PHOTOSENSITIVITY_FOOTER_TOP_GAP_PX,
            ),
            left: Val::Px(PHOTOSENSITIVITY_PANEL_PADDING_PX),
        },
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

/// PROMPT 1349 — footer slot positioned absolutely at the panel's
/// bottom-padding edge so the Acknowledge button is always reachable
/// without scrolling, even if the body copy or panel size grows
/// (story 026 AC1 / §5 C-5 footer slot pattern).
pub fn warning_footer_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(PHOTOSENSITIVITY_PANEL_PADDING_PX),
        right: Val::Px(PHOTOSENSITIVITY_PANEL_PADDING_PX),
        bottom: Val::Px(PHOTOSENSITIVITY_PANEL_PADDING_PX),
        height: Val::Px(PHOTOSENSITIVITY_FOOTER_HEIGHT_PX),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn warning_acknowledge_node() -> Node {
    Node {
        width: Val::Px(144.0),
        height: Val::Px(PHOTOSENSITIVITY_FOOTER_HEIGHT_PX),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(8.0)),
        ..default()
    }
}

fn warning_text_font(font_size: f32) -> TextFont {
    TextFont {
        font_size,
        ..default()
    }
}
