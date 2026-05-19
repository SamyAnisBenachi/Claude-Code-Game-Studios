//! Story 021 / S13-CONN-LOST-UX-001 — proactive Connection Lost / Reconnecting
//! overlay.
//!
//! Surfaces a visible modal between a Lightyear transport drop and reconnect
//! completion (`On<Add, Connected>`) or `RoundPhase::GameOver`. The overlay is
//! a read-only projection over transport state and the authoritative client
//! phase view; it does not synthesise any S2C message.
//!
//! No optimistic client-side authority is introduced by this overlay.
//! ADR-002 + ADR-011 binding.
//!
//! Composition contract (ADR-021): `PresentationPlugin` registers
//! [`ConnectionLostOverlayPlugin`] after `ResultScreenPlugin` so the
//! presentation order matches the visual layering — gameplay UI below, this
//! overlay in the middle, result screen on top. Both layers consume the
//! Sprint 14 z-layer constants from `crate::ui::design_tokens::z_layers`
//! ([`UI_OVERLAY`](crate::ui::design_tokens::z_layers::UI_OVERLAY) for this
//! overlay; [`MODAL`](crate::ui::design_tokens::z_layers::MODAL) for the
//! result screen).

use bevy::prelude::*;
use bevy::ui::Overflow;
use lightyear::prelude::{Connected, Disconnected};
use shared::protocol::RoundPhase;

use crate::state::{ClientState, CurrentClientPhase};
use crate::ui::design_tokens::{typography, z_layers};

/// Z-layer for this overlay. Resolved from the canonical
/// [`UI_OVERLAY`](crate::ui::design_tokens::z_layers::UI_OVERLAY) constant so
/// the overlay sits above gameplay UI but below the result screen modal —
/// the invariant guarded by `ac7_overlay_z_index_is_below_result_screen` in
/// `tests/integration/playable_client/connection_lost_overlay_test.rs`.
pub const CONNECTION_LOST_OVERLAY_Z_INDEX: i32 = z_layers::UI_OVERLAY.0;

/// PROMPT 1349 (Sprint 18 story 026 / Lane J / §5 C-5) — modal-overflow
/// contract: every modal panel must cap its height at `92%` of the
/// viewport and scroll when content exceeds the cap. Shared with
/// `result_screen.rs` (PROMPT 1180 §1.5 O-04 reference template) and
/// the photosensitivity warning + draft-initial modal.
pub const CONNECTION_LOST_PANEL_MAX_HEIGHT_PERCENT: f32 = 92.0;

/// PROMPT 1349 — panel-node builder for the connection-lost overlay
/// modal. Carries the `max_height: 92%` + `Overflow::scroll_y()`
/// contract so the panel never clips against a short viewport and the
/// body can scroll if reconnect copy grows (story 026 AC2).
pub fn connection_lost_overlay_panel_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        width: Val::Percent(60.0),
        max_width: Val::Px(520.0),
        max_height: Val::Percent(CONNECTION_LOST_PANEL_MAX_HEIGHT_PERCENT),
        overflow: Overflow::scroll_y(),
        row_gap: Val::Px(12.0),
        padding: UiRect::all(Val::Px(22.0)),
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
    }
}

pub struct ConnectionLostOverlayPlugin;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionLostOverlayState {
    pub visible: bool,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ConnectionLostOverlayEntities {
    pub root: Entity,
    pub panel: Entity,
    pub headline: Entity,
    pub body: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ConnectionLostOverlayRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct ConnectionLostOverlayPanel;

#[derive(Component, Debug, Clone, Copy)]
pub struct ConnectionLostOverlayHeadline;

#[derive(Component, Debug, Clone, Copy)]
pub struct ConnectionLostOverlayBody;

impl Plugin for ConnectionLostOverlayPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("ConnectionLostOverlayPlugin loaded");
        app.init_resource::<ConnectionLostOverlayState>()
            .add_systems(Startup, spawn_connection_lost_overlay_system)
            .add_systems(
                Update,
                (
                    dismiss_overlay_on_game_over_system,
                    sync_connection_lost_overlay_visibility_system,
                )
                    .chain(),
            )
            .add_observer(on_transport_disconnected)
            .add_observer(on_transport_connected);
    }
}

/// Read-only predicate: should a transport drop raise the overlay given the
/// current client lifecycle? The overlay is in-session only; in `Lobby` the
/// lobby's own status copy already covers the "Connecting" surface.
pub fn should_show_overlay_for_client_state(state: ClientState) -> bool {
    matches!(state, ClientState::InSession)
}

/// Read-only predicate: is the authoritative phase past the gameplay window?
/// Once `GameOver` lands the result screen takes over and the overlay must
/// dismiss within one frame (AC5).
pub fn overlay_dismissed_by_phase(phase: RoundPhase) -> bool {
    matches!(phase, RoundPhase::GameOver)
}

/// Pure handler invoked by the `On<Add, Disconnected>` observer. Exposed so
/// integration tests can drive the same flow without constructing lightyear's
/// internal `Disconnected` component.
pub fn handle_transport_disconnected_event(
    lifecycle: ClientState,
    overlay: &mut ConnectionLostOverlayState,
) {
    if should_show_overlay_for_client_state(lifecycle) {
        tracing::info!(
            target: "client::presentation::connection_lost_overlay",
            lifecycle = ?lifecycle,
            "transport_disconnected: overlay -> visible"
        );
        overlay.visible = true;
    } else {
        tracing::debug!(
            target: "client::presentation::connection_lost_overlay",
            lifecycle = ?lifecycle,
            "transport_disconnected: ignored (not InSession)"
        );
    }
}

/// Pure handler invoked by the `On<Add, Connected>` observer. Per ADR-011 the
/// reconnect path does not exit `InSession`, so the lightyear `Connected` Add
/// fires when the transport re-links and the snapshot/session-replay flow
/// resumes. The overlay dismisses unconditionally on connect.
pub fn handle_transport_connected_event(overlay: &mut ConnectionLostOverlayState) {
    if overlay.visible {
        tracing::info!(
            target: "client::presentation::connection_lost_overlay",
            "transport_connected: overlay -> hidden"
        );
    }
    overlay.visible = false;
}

pub fn on_transport_disconnected(
    _trigger: On<Add, Disconnected>,
    state: Option<Res<State<ClientState>>>,
    mut overlay: ResMut<ConnectionLostOverlayState>,
) {
    let lifecycle = state
        .as_deref()
        .map(|s| *s.get())
        .unwrap_or(ClientState::Lobby);
    handle_transport_disconnected_event(lifecycle, &mut overlay);
}

pub fn on_transport_connected(
    _trigger: On<Add, Connected>,
    mut overlay: ResMut<ConnectionLostOverlayState>,
) {
    handle_transport_connected_event(&mut overlay);
}

pub fn dismiss_overlay_on_game_over_system(
    phase: Option<Res<CurrentClientPhase>>,
    mut overlay: ResMut<ConnectionLostOverlayState>,
) {
    let Some(phase) = phase.as_deref() else {
        return;
    };
    if overlay.visible && overlay_dismissed_by_phase(phase.phase) {
        tracing::info!(
            target: "client::presentation::connection_lost_overlay",
            phase = ?phase.phase,
            "dismiss_overlay_on_game_over: overlay -> hidden"
        );
        overlay.visible = false;
    }
}

pub fn sync_connection_lost_overlay_visibility_system(
    overlay: Res<ConnectionLostOverlayState>,
    entities: Option<Res<ConnectionLostOverlayEntities>>,
    mut visibilities: Query<&mut Visibility, With<ConnectionLostOverlayRoot>>,
) {
    let Some(entities) = entities else {
        return;
    };
    let Ok(mut visibility) = visibilities.get_mut(entities.root) else {
        return;
    };
    let desired = if overlay.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *visibility != desired {
        *visibility = desired;
    }
}

fn spawn_connection_lost_overlay_system(mut commands: Commands) {
    let root = commands
        .spawn((
            Name::new("Connection lost overlay root"),
            ConnectionLostOverlayRoot,
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            // Backdrop alpha is intentionally lower than the canonical
            // `overlays::OVERLAY_SCRIM_ALPHA` (0.55) used by the settlement
            // / result-screen modal scrims so the gameplay UI (hand, HUD,
            // board) remains visible underneath while the overlay is
            // shown (AC7). This 0.32 value is preserved as a documented
            // AC6 exclusion by Sprint 14 story 006
            // (`S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`); see
            // `client/src/ui/design_tokens/overlays.rs` module doc and
            // `production/qa/evidence/sprint-14-overlay-alpha-token/`.
            BackgroundColor(Color::srgba(0.02, 0.025, 0.035, 0.32)),
            Visibility::Hidden,
            z_layers::UI_OVERLAY,
        ))
        .id();

    let panel = commands
        .spawn((
            Name::new("Connection lost overlay panel"),
            ConnectionLostOverlayPanel,
            ChildOf(root),
            connection_lost_overlay_panel_node(),
            BackgroundColor(Color::srgba(0.16, 0.10, 0.04, 0.92)),
            BorderColor::all(Color::srgba(0.96, 0.74, 0.30, 0.85)),
        ))
        .id();

    let headline = commands
        .spawn((
            Name::new("Connection lost overlay headline"),
            ConnectionLostOverlayHeadline,
            ChildOf(panel),
            Text::new("Connection Lost"),
            TextFont {
                font_size: typography::H1,
                ..default()
            },
            TextColor(Color::srgb(0.99, 0.96, 0.84)),
        ))
        .id();

    let body = commands
        .spawn((
            Name::new("Connection lost overlay body"),
            ConnectionLostOverlayBody,
            ChildOf(panel),
            Text::new("Reconnecting..."),
            TextFont {
                font_size: typography::H3,
                ..default()
            },
            TextColor(Color::srgb(0.92, 0.90, 0.82)),
        ))
        .id();

    commands.insert_resource(ConnectionLostOverlayEntities {
        root,
        panel,
        headline,
        body,
    });
}
