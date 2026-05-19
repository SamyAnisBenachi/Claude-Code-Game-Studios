//! PROMPT 1404 / `S19-UI-PHASE-CHANGE-BANNER-001` — transient centered banner
//! overlay surfaced on every major `RoundPhase` transition.
//!
//! Closes V-P1-04 / RC-8 from PROMPT 1396: phase and turn transitions had
//! only repainted the HUD phase pill, which is too quiet a signal for a
//! Krosmaga-style card game. This module owns a self-contained banner
//! surface that spawns on phase change, paints a high-contrast centered
//! label, and auto-despawns after [`PHASE_BANNER_LIFETIME`].
//!
//! ## Composition contract (ADR-021)
//!
//! Read-only over `CurrentClientPhase`. Sends no gameplay messages and does
//! not synthesise any server state. The plugin is wired into
//! [`crate::ui::PhaseBannerPlugin`] and registered by `PresentationPlugin`
//! after the gameplay UI roots so the banner paints above the HUD / hand /
//! board layers but below modals — z-layer
//! [`z_layers::UI_OVERLAY`](crate::ui::design_tokens::z_layers::UI_OVERLAY)
//! so a result-screen or photosensitivity modal still wins focus when
//! `GameOver` lands.
//!
//! ## Scope
//!
//! Banner labels are sourced from [`phase_banner_label_for`] for the five
//! major in-session phases (DraftInitial / DraftShop / DraftAuction /
//! Placement / Resolution). Lobby / Handshaking / GameOver return `None`
//! because those phases already own dedicated surfaces (lobby chrome,
//! result screen) that would compete with a transient banner.

use std::time::Duration;

use bevy::prelude::*;
use shared::protocol::RoundPhase;

use crate::state::{ClientState, CurrentClientPhase};
use crate::ui::design_tokens::{spacing, typography, z_layers};

/// Time a single banner remains on-screen before its parent entity is
/// despawned. Sized so the label is readable on Krosmaga's reference
/// `1366×768` viewport without competing with the next phase's UI.
pub const PHASE_BANNER_LIFETIME: Duration = Duration::from_millis(1400);

/// Maximum panel width in pixels at any viewport. Sized so the longest
/// banner label (`"RESOLUTION"`) renders on a single line at
/// [`typography::H1`] without wrapping or clipping on the smallest
/// supported viewport.
pub const PHASE_BANNER_MAX_WIDTH_PX: f32 = 520.0;

/// Maximum panel width as a percentage of viewport width. Caps the panel
/// at 80 % so the banner never reaches the viewport edges on ultra-wide
/// displays.
pub const PHASE_BANNER_MAX_WIDTH_PERCENT: f32 = 80.0;
pub const PHASE_BANNER_MIN_HEIGHT_PX: f32 = 82.0;
pub const PHASE_BANNER_BACKGROUND_COLOR: Color = Color::srgba(0.045, 0.055, 0.070, 0.95);
pub const PHASE_BANNER_BORDER_COLOR: Color = Color::srgba(1.0, 0.78, 0.30, 1.0);
pub const PHASE_BANNER_TEXT_COLOR: Color = Color::srgb(1.0, 0.95, 0.78);

/// Marker + lifetime carrier for the active phase-banner root entity.
#[derive(Component, Debug)]
pub struct PhaseBannerRoot {
    /// Ticks down each frame; on `finished()` the banner is despawned.
    pub remaining: Timer,
    /// The phase whose transition raised this banner. Diagnostic — read
    /// by tests to assert the banner reflects the intended transition.
    pub phase: RoundPhase,
}

/// Marker for the centered panel node carrying the banner's visual
/// chrome (background, border, padding). Used by tests to assert the
/// `max_width` ceiling.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseBannerPanel;

/// Marker for the banner's centered text label entity. Used by tests to
/// assert the rendered phase label.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseBannerLabel;

pub struct PhaseBannerPlugin;

impl Plugin for PhaseBannerPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("PhaseBannerPlugin loaded");
        app.init_resource::<CurrentClientPhase>()
            .add_systems(
                Update,
                (
                    spawn_phase_banner_on_phase_change,
                    tick_phase_banner_lifetime,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .add_systems(
                OnExit(ClientState::InSession),
                despawn_all_phase_banners_on_session_exit,
            );
    }
}

/// Return the centered banner label for major phase transitions, or
/// `None` for phases that intentionally don't raise a banner.
///
/// Lobby / Handshaking / GameOver return `None`: those phases either
/// have no in-session UI (Lobby / Handshaking) or own a full-screen
/// modal (result screen on GameOver) that would clash with a transient
/// centered banner.
pub fn phase_banner_label_for(phase: RoundPhase) -> Option<&'static str> {
    match phase {
        RoundPhase::DraftInitial => Some("DRAFT"),
        RoundPhase::DraftShop => Some("SHOP"),
        RoundPhase::DraftAuction => Some("AUCTION"),
        RoundPhase::Placement => Some("PLACEMENT"),
        RoundPhase::Resolution => Some("RESOLUTION"),
        RoundPhase::Lobby | RoundPhase::Handshaking | RoundPhase::GameOver => None,
    }
}

pub fn spawn_phase_banner_on_phase_change(
    mut last_phase: Local<Option<RoundPhase>>,
    mut commands: Commands,
    current: Res<CurrentClientPhase>,
    existing: Query<Entity, With<PhaseBannerRoot>>,
) {
    if *last_phase == Some(current.phase) {
        return;
    }
    *last_phase = Some(current.phase);

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Some(label) = phase_banner_label_for(current.phase) else {
        return;
    };

    tracing::info!(
        target: "client::ui::phase_banner",
        phase = ?current.phase,
        round = current.round,
        label,
        "phase_banner_spawn",
    );

    let root = commands
        .spawn((
            Name::new("Phase Banner Root"),
            PhaseBannerRoot {
                remaining: Timer::new(PHASE_BANNER_LIFETIME, TimerMode::Once),
                phase: current.phase,
            },
            phase_banner_root_node(),
            Visibility::Visible,
            z_layers::UI_OVERLAY,
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands
        .entity(root)
        .insert(bevy::picking::Pickable::IGNORE);

    let panel = commands
        .spawn((
            Name::new("Phase Banner Panel"),
            PhaseBannerPanel,
            phase_banner_panel_node(),
            BackgroundColor(PHASE_BANNER_BACKGROUND_COLOR),
            BorderColor::all(PHASE_BANNER_BORDER_COLOR),
            ChildOf(root),
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands
        .entity(panel)
        .insert(bevy::picking::Pickable::IGNORE);

    commands.spawn((
        Name::new("Phase Banner Label"),
        PhaseBannerLabel,
        Text::new(label),
        TextFont {
            font_size: typography::H1,
            ..default()
        },
        TextColor(PHASE_BANNER_TEXT_COLOR),
        ChildOf(panel),
    ));
}

pub fn tick_phase_banner_lifetime(
    time: Res<Time>,
    mut commands: Commands,
    mut banners: Query<(Entity, &mut PhaseBannerRoot)>,
) {
    let delta = time.delta();
    for (entity, mut banner) in &mut banners {
        banner.remaining.tick(delta);
        if banner.remaining.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn despawn_all_phase_banners_on_session_exit(
    mut commands: Commands,
    banners: Query<Entity, With<PhaseBannerRoot>>,
) {
    for entity in &banners {
        commands.entity(entity).despawn();
    }
}

/// Root node that fills the viewport and centers the inner panel both
/// horizontally and vertically. `position_type: Absolute` keeps the
/// banner detached from the flex flow of the underlying gameplay UI.
pub fn phase_banner_root_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

/// Panel node carrying the banner's visual chrome. Width is bounded so
/// the longest banner label never reaches the viewport edges; padding
/// uses the canonical [`spacing`] tokens so the banner reads as a
/// proper card-game phase callout rather than ad-hoc text.
pub fn phase_banner_panel_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        max_width: Val::Percent(PHASE_BANNER_MAX_WIDTH_PERCENT),
        width: Val::Px(PHASE_BANNER_MAX_WIDTH_PX),
        min_height: Val::Px(PHASE_BANNER_MIN_HEIGHT_PX),
        padding: UiRect::axes(Val::Px(spacing::SPACING_XL), Val::Px(spacing::SPACING_LG)),
        border: UiRect::all(Val::Px(3.0)),
        border_radius: BorderRadius::all(Val::Px(spacing::SPACING_SM)),
        ..default()
    }
}
