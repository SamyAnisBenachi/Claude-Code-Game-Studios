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

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::Overflow;
use lightyear::prelude::{Connected, Disconnected};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

use crate::state::{ClientState, CurrentClientPhase, OpponentConnectionView};
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
    pub cause: ConnectionLostOverlayCause,
    pub disconnected_player_id: Option<PlayerId>,
    pub grace_remaining_ms: Option<u32>,
    pub input_blocking: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionLostOverlayCause {
    #[default]
    None,
    LocalTransportDisconnected,
    OpponentDisconnected,
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
                    sync_connection_lost_overlay_from_opponent_connection_system,
                    sync_connection_lost_overlay_copy_system,
                    sync_connection_lost_overlay_layout_system,
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
            cause = ?ConnectionLostOverlayCause::LocalTransportDisconnected,
            input_blocking = true,
            "transport_disconnected: overlay -> visible"
        );
        overlay.visible = true;
        overlay.cause = ConnectionLostOverlayCause::LocalTransportDisconnected;
        overlay.disconnected_player_id = None;
        overlay.grace_remaining_ms = None;
        overlay.input_blocking = true;
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
            cause = ?overlay.cause,
            "transport_connected: overlay -> hidden"
        );
    }
    overlay.clear();
}

impl ConnectionLostOverlayState {
    fn clear(&mut self) {
        self.visible = false;
        self.cause = ConnectionLostOverlayCause::None;
        self.disconnected_player_id = None;
        self.grace_remaining_ms = None;
        self.input_blocking = false;
    }

    fn set_opponent_disconnected(&mut self, player_id: PlayerId, grace_remaining_ms: u32) {
        self.visible = true;
        self.cause = ConnectionLostOverlayCause::OpponentDisconnected;
        self.disconnected_player_id = Some(player_id);
        self.grace_remaining_ms = Some(grace_remaining_ms);
        self.input_blocking = false;
    }

    pub fn cause_label(&self) -> &'static str {
        match self.cause {
            ConnectionLostOverlayCause::None => "none",
            ConnectionLostOverlayCause::LocalTransportDisconnected => {
                "local_transport_disconnected"
            }
            ConnectionLostOverlayCause::OpponentDisconnected => "opponent_disconnected",
        }
    }
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
            cause = ?overlay.cause,
            "dismiss_overlay_on_game_over: overlay -> hidden"
        );
        overlay.clear();
    }
}

pub fn sync_connection_lost_overlay_from_opponent_connection_system(
    opponent: Option<Res<OpponentConnectionView>>,
    mut overlay: ResMut<ConnectionLostOverlayState>,
) {
    if matches!(
        overlay.cause,
        ConnectionLostOverlayCause::LocalTransportDisconnected
    ) {
        return;
    }

    let Some(opponent) = opponent.as_deref() else {
        return;
    };
    match opponent.disconnected {
        Some(indicator) => {
            let changed = overlay.cause != ConnectionLostOverlayCause::OpponentDisconnected
                || overlay.disconnected_player_id != Some(indicator.player_id)
                || overlay.grace_remaining_ms != Some(indicator.grace_remaining_ms)
                || overlay.input_blocking;
            if changed {
                tracing::info!(
                    target: "client::presentation::connection_lost_overlay",
                    cause = ?ConnectionLostOverlayCause::OpponentDisconnected,
                    player_id = ?indicator.player_id,
                    grace_remaining_ms = indicator.grace_remaining_ms,
                    input_blocking = false,
                    "opponent_disconnected: overlay -> non_blocking_visible"
                );
            }
            overlay.set_opponent_disconnected(indicator.player_id, indicator.grace_remaining_ms);
        }
        None if overlay.cause == ConnectionLostOverlayCause::OpponentDisconnected => {
            tracing::info!(
                target: "client::presentation::connection_lost_overlay",
                cause = ?overlay.cause,
                "opponent_reconnected: overlay -> hidden"
            );
            overlay.clear();
        }
        None => {}
    }
}

pub fn connection_lost_overlay_copy(
    overlay: &ConnectionLostOverlayState,
) -> (&'static str, String) {
    match overlay.cause {
        ConnectionLostOverlayCause::LocalTransportDisconnected => (
            "Connection Interrupted",
            "Your client is reconnecting. Game input is blocked until the transport reconnects; auction and shop state remains visible behind this notice.".to_string(),
        ),
        ConnectionLostOverlayCause::OpponentDisconnected => {
            let player = overlay
                .disconnected_player_id
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|| "opponent".to_string());
            let grace = overlay
                .grace_remaining_ms
                .map(|ms| format!(" Reconnect grace remaining: {}s.", (ms + 999) / 1000))
                .unwrap_or_default();
            (
                "Opponent Reconnecting",
                format!(
                    "{player} disconnected. Your local input is not blocked.{grace} Auction and shop controls stay visible."
                ),
            )
        }
        ConnectionLostOverlayCause::None => ("Connection Status", String::new()),
    }
}

pub fn sync_connection_lost_overlay_copy_system(
    overlay: Res<ConnectionLostOverlayState>,
    entities: Option<Res<ConnectionLostOverlayEntities>>,
    mut text_query: Query<&mut Text>,
) {
    if !overlay.is_changed() {
        return;
    }
    let Some(entities) = entities else {
        return;
    };
    let (headline, body) = connection_lost_overlay_copy(&overlay);
    if let Ok(mut text) = text_query.get_mut(entities.headline) {
        text.0 = headline.to_string();
    }
    if let Ok(mut text) = text_query.get_mut(entities.body) {
        text.0 = body;
    }
}

pub fn sync_connection_lost_overlay_layout_system(
    overlay: Res<ConnectionLostOverlayState>,
    entities: Option<Res<ConnectionLostOverlayEntities>>,
    mut roots: Query<
        (&mut Node, &mut BackgroundColor, &mut Pickable),
        With<ConnectionLostOverlayRoot>,
    >,
    mut panels: Query<
        &mut Node,
        (
            With<ConnectionLostOverlayPanel>,
            Without<ConnectionLostOverlayRoot>,
        ),
    >,
) {
    if !overlay.is_changed() {
        return;
    }
    let Some(entities) = entities else {
        return;
    };
    let Ok((mut root_node, mut root_bg, mut pickable)) = roots.get_mut(entities.root) else {
        return;
    };
    let Ok(mut panel_node) = panels.get_mut(entities.panel) else {
        return;
    };

    if overlay.input_blocking {
        root_node.align_items = AlignItems::Center;
        root_node.justify_content = JustifyContent::Center;
        root_node.padding = UiRect::all(Val::Px(24.0));
        panel_node.width = Val::Percent(60.0);
        panel_node.max_width = Val::Px(520.0);
        root_bg.0 = Color::srgba(0.02, 0.025, 0.035, 0.32);
    } else {
        root_node.align_items = AlignItems::FlexStart;
        root_node.justify_content = JustifyContent::FlexEnd;
        root_node.padding = UiRect {
            left: Val::Px(24.0),
            right: Val::Px(24.0),
            top: Val::Px(84.0),
            bottom: Val::Px(24.0),
        };
        panel_node.width = Val::Px(380.0);
        panel_node.max_width = Val::Px(380.0);
        root_bg.0 = Color::NONE;
    }

    pickable.should_block_lower = overlay.input_blocking;
    pickable.is_hoverable = overlay.input_blocking;
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
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
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
