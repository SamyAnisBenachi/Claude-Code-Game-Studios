//! Debug-only bot god-mode overlay (PROMPT 1614).
//!
//! Renders the latest [`S2CDebugBotStatePush`] payload (server-side gated by
//! `CCGS_BOT_DEBUG_UI=1` — see `server::feature::bot::debug_push`) inside a
//! corner UI panel that is hidden by default and toggled with the F8 key.
//! Activation here is a **second** independent gate
//! (`CCGS_DEBUG_UI=1`) so QA can capture decision tails without leaking a
//! god-mode panel to other developers who happen to run a server with
//! `CCGS_BOT_DEBUG_UI=1` enabled.
//!
//! ## Activation contract
//!
//! Mirrors [`QASnapshotConfig::from_env_values`](super::qa_snapshot::QASnapshotConfig)
//! byte-for-byte so operators only learn one convention:
//!
//! - `CCGS_DEBUG_UI=1` forces enabled (overlay surfaces even in release).
//! - `CCGS_DEBUG_UI=0` forces disabled.
//! - Unset / empty / whitespace-only: defaults to `cfg!(debug_assertions)`.
//! - Any other value is logged as invalid and treated as disabled.
//!
//! When disabled, the plugin spawns no UI and adds no per-frame work beyond
//! the inert system registrations — same discipline as
//! `QASnapshotPlugin`.
//!
//! ## Composition
//!
//! Registered after [`QASnapshotPlugin`](super::qa_snapshot::QASnapshotPlugin)
//! in [`PresentationPlugin`](super::PresentationPlugin) so the overlay sits
//! above the QA snapshot button (Z_DEBUG = 700, above MODAL = 500).
//! The overlay is non-interactive: `Pickable::should_block_lower = false`,
//! `is_hoverable = false` — mouse clicks pass through to gameplay UI so QA
//! can keep playing while inspecting bot state.

use bevy::prelude::*;
use bevy::ui::Overflow;
use bevy::picking::Pickable;
use lightyear::prelude::MessageReceiver;
use shared::protocol::S2CDebugBotStatePush;

use crate::ui::design_tokens::{typography, z_layers};

/// Env var that gates the client-side debug overlay.
pub const DEBUG_UI_ENV_VAR: &str = "CCGS_DEBUG_UI";

/// Keyboard shortcut that toggles overlay visibility when
/// [`DebugBotOverlayConfig::enabled`] is `true`. F8 is unbound elsewhere in
/// the client (`reports/PROMPT-1604-…md` §2.5).
pub const DEBUG_BOT_OVERLAY_SHORTCUT_KEY: KeyCode = KeyCode::F8;

/// Maximum number of decision-tail rows the overlay paints per bot. Capped
/// here as well as on the server so the panel does not balloon if a future
/// server bumps the wire cap.
pub const DEBUG_BOT_OVERLAY_TAIL_RENDER_CAP: usize = 12;

// ---------------------------------------------------------------------------
// Config + state
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct DebugBotOverlayConfig {
    pub enabled: bool,
}

impl Default for DebugBotOverlayConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl DebugBotOverlayConfig {
    pub fn from_env() -> Self {
        Self::from_env_values(
            std::env::var(DEBUG_UI_ENV_VAR).ok().as_deref(),
            cfg!(debug_assertions),
        )
    }

    pub fn from_env_values(enable_var: Option<&str>, dev_default_enabled: bool) -> Self {
        let enabled = match enable_var.map(str::trim) {
            None | Some("") => dev_default_enabled,
            Some("1") => true,
            Some("0") => false,
            Some(other) => {
                tracing::warn!(
                    target: "client::presentation::debug_bot_overlay",
                    value = %other,
                    "{} has invalid value; treating as disabled (expected 1, 0, or unset)",
                    DEBUG_UI_ENV_VAR,
                );
                false
            }
        };
        Self { enabled }
    }
}

/// Latest cached payload + visibility state. Always initialised even when
/// the overlay is disabled so test scaffolds can pre-insert a payload and
/// then enable the overlay in-place.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct DebugBotOverlayState {
    pub visible: bool,
    pub latest: Option<S2CDebugBotStatePush>,
    /// Monotonic counter incremented on every successful receive — used by
    /// tests to assert the drain system advances independently of payload
    /// contents.
    pub receive_count: u64,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct DebugBotOverlayEntities {
    pub root: Entity,
    pub panel: Entity,
    pub headline: Entity,
    pub body: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DebugBotOverlayRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct DebugBotOverlayPanel;

#[derive(Component, Debug, Clone, Copy)]
pub struct DebugBotOverlayHeadline;

#[derive(Component, Debug, Clone, Copy)]
pub struct DebugBotOverlayBody;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DebugBotOverlayPlugin;

impl Plugin for DebugBotOverlayPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<DebugBotOverlayConfig>() {
            app.insert_resource(DebugBotOverlayConfig::from_env());
        }
        app.init_resource::<DebugBotOverlayState>();

        // Only spawn UI / register the keyboard + receive systems when the
        // overlay is enabled. When disabled, the plugin is otherwise inert
        // and pays zero per-frame cost.
        let enabled = app
            .world()
            .resource::<DebugBotOverlayConfig>()
            .enabled;
        if enabled {
            app.add_systems(Startup, spawn_debug_bot_overlay_system);
            app.add_systems(
                Update,
                (
                    drain_debug_bot_state_receiver_system,
                    debug_bot_overlay_keyboard_shortcut_system,
                    sync_debug_bot_overlay_text_system,
                    sync_debug_bot_overlay_visibility_system,
                )
                    .chain(),
            );
        } else {
            tracing::debug!(
                target: "client::presentation::debug_bot_overlay",
                "DebugBotOverlayPlugin loaded but inert ({}=0)",
                DEBUG_UI_ENV_VAR,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Drain receiver system: copies the latest [`S2CDebugBotStatePush`] into
/// [`DebugBotOverlayState`]. Uses `Option<…>` for the query so test apps
/// without a Lightyear `MessageReceiver` registered still compile.
pub fn drain_debug_bot_state_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CDebugBotStatePush>>,
    mut overlay: ResMut<DebugBotOverlayState>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::debug!(
                target: "client::presentation::debug_bot_overlay",
                bots = message.bots.len(),
                decision_log_total = message.decision_log_total,
                assembled_at_ms = message.assembled_at_ms,
                msg_type = "S2CDebugBotStatePush",
                "debug_bot_overlay: recv"
            );
            overlay.latest = Some(message);
            overlay.receive_count = overlay.receive_count.saturating_add(1);
        }
    }
}

/// Pure helper: maps the keyboard event into the next overlay visibility.
/// Exposed so tests can drive the toggle without constructing a Bevy
/// `ButtonInput<KeyCode>`.
pub fn apply_overlay_toggle(state: &mut DebugBotOverlayState) {
    state.visible = !state.visible;
}

pub fn debug_bot_overlay_keyboard_shortcut_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut overlay: ResMut<DebugBotOverlayState>,
) {
    let Some(keyboard) = keyboard else {
        return;
    };
    if keyboard.just_pressed(DEBUG_BOT_OVERLAY_SHORTCUT_KEY) {
        apply_overlay_toggle(&mut overlay);
        tracing::info!(
            target: "client::presentation::debug_bot_overlay",
            visible = overlay.visible,
            "debug_bot_overlay: F8 toggled visibility"
        );
    }
}

/// Pure helper: render the overlay body text from a payload. Stable enough
/// to assert in unit tests; kept short so the panel does not eat the
/// gameplay viewport.
pub fn render_overlay_body(payload: &S2CDebugBotStatePush) -> String {
    if payload.bots.is_empty() {
        return format!(
            "No bots in session.\nAssembled @ {} ms\nDecisions logged: {}",
            payload.assembled_at_ms, payload.decision_log_total
        );
    }
    let mut out = String::new();
    out.push_str(&format!(
        "Bots: {}  |  Decisions: {}  |  @ {} ms\n",
        payload.bots.len(),
        payload.decision_log_total,
        payload.assembled_at_ms,
    ));
    for bot in &payload.bots {
        let class = bot
            .class_id
            .map(|c| format!("{:?}", c))
            .unwrap_or_else(|| "?".to_string());
        out.push_str(&format!(
            "\n[{:?}] class={class} gold={} mana={}/{} submitted={} hand={}",
            bot.player_id,
            bot.gold,
            bot.current_mana,
            bot.mana_cap,
            bot.submitted,
            bot.hand.len(),
        ));
        if let Some(val) = bot.last_bid_valuation {
            out.push_str(&format!(" last_bid_val={val}"));
        }
        let tail_render = bot
            .decision_tail
            .iter()
            .rev()
            .take(DEBUG_BOT_OVERLAY_TAIL_RENDER_CAP)
            .collect::<Vec<_>>();
        for entry in tail_render.iter().rev() {
            let detail = entry
                .detail
                .clone()
                .map(|d| format!(" {d}"))
                .unwrap_or_default();
            out.push_str(&format!(
                "\n   r{}.{:?} {} {}{}",
                entry.round_number,
                entry.phase,
                entry.timestamp_ms,
                entry.kind_label,
                detail,
            ));
        }
    }
    out
}

pub fn sync_debug_bot_overlay_text_system(
    overlay: Res<DebugBotOverlayState>,
    entities: Option<Res<DebugBotOverlayEntities>>,
    mut text_query: Query<&mut Text>,
) {
    if !overlay.is_changed() {
        return;
    }
    let Some(entities) = entities else {
        return;
    };
    let body = match &overlay.latest {
        Some(payload) => render_overlay_body(payload),
        None => "Waiting for first S2CDebugBotStatePush…".to_string(),
    };
    if let Ok(mut text) = text_query.get_mut(entities.body) {
        text.0 = body;
    }
}

pub fn sync_debug_bot_overlay_visibility_system(
    overlay: Res<DebugBotOverlayState>,
    entities: Option<Res<DebugBotOverlayEntities>>,
    mut visibilities: Query<&mut Visibility, With<DebugBotOverlayRoot>>,
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

fn spawn_debug_bot_overlay_system(mut commands: Commands) {
    let root = commands
        .spawn((
            Name::new("Debug bot overlay root"),
            DebugBotOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(72.0),
                right: Val::Px(16.0),
                width: Val::Px(360.0),
                max_height: Val::Percent(70.0),
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.86)),
            BorderColor::all(Color::srgba(0.30, 0.85, 0.55, 0.70)),
            Visibility::Hidden,
            z_layers::DEBUG,
            // Non-blocking: gameplay input keeps working underneath the
            // overlay so QA can interact while inspecting the bot state.
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .id();

    let panel = commands
        .spawn((
            Name::new("Debug bot overlay panel"),
            DebugBotOverlayPanel,
            ChildOf(root),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .id();

    let headline = commands
        .spawn((
            Name::new("Debug bot overlay headline"),
            DebugBotOverlayHeadline,
            ChildOf(panel),
            Text::new("Bot Debug (F8)"),
            TextFont {
                font_size: typography::H3,
                ..default()
            },
            TextColor(Color::srgb(0.78, 0.96, 0.84)),
        ))
        .id();

    let body = commands
        .spawn((
            Name::new("Debug bot overlay body"),
            DebugBotOverlayBody,
            ChildOf(panel),
            Text::new("Waiting for first S2CDebugBotStatePush…"),
            TextFont {
                font_size: typography::BODY,
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.92, 0.86)),
        ))
        .id();

    commands.insert_resource(DebugBotOverlayEntities {
        root,
        panel,
        headline,
        body,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::card::{CardId, ClassId};
    use shared::protocol::{
        DebugBotDecisionEntry, DebugBotStateEntry, RoundPhase, S2CDebugBotStatePush,
    };
    use shared::session::PlayerId;

    fn make_payload_with_one_bot() -> S2CDebugBotStatePush {
        S2CDebugBotStatePush {
            bots: vec![DebugBotStateEntry {
                player_id: PlayerId(1),
                class_id: Some(ClassId::Cra),
                gold: 5,
                current_mana: 3,
                mana_cap: 4,
                submitted: false,
                hand: vec![CardId(7), CardId(8)],
                decision_tail: vec![DebugBotDecisionEntry {
                    round_number: 1,
                    phase: RoundPhase::DraftAuction,
                    timestamp_ms: 1234,
                    kind_label: "auction_bid".to_string(),
                    detail: Some("card=7 amt=2 val=3".to_string()),
                }],
                last_bid_valuation: Some(3),
            }],
            decision_log_total: 1,
            assembled_at_ms: 9999,
        }
    }

    #[test]
    fn from_env_values_respects_explicit_enable() {
        assert!(DebugBotOverlayConfig::from_env_values(Some("1"), false).enabled);
        assert!(!DebugBotOverlayConfig::from_env_values(Some("0"), true).enabled);
    }

    #[test]
    fn from_env_values_uses_dev_default_when_unset() {
        assert!(DebugBotOverlayConfig::from_env_values(None, true).enabled);
        assert!(DebugBotOverlayConfig::from_env_values(Some(""), true).enabled);
        assert!(DebugBotOverlayConfig::from_env_values(Some("   "), true).enabled);
        assert!(!DebugBotOverlayConfig::from_env_values(None, false).enabled);
    }

    #[test]
    fn from_env_values_invalid_is_disabled() {
        assert!(!DebugBotOverlayConfig::from_env_values(Some("on"), true).enabled);
        assert!(!DebugBotOverlayConfig::from_env_values(Some("yes"), true).enabled);
    }

    #[test]
    fn apply_overlay_toggle_flips_visibility() {
        let mut state = DebugBotOverlayState::default();
        assert!(!state.visible);
        apply_overlay_toggle(&mut state);
        assert!(state.visible);
        apply_overlay_toggle(&mut state);
        assert!(!state.visible);
    }

    #[test]
    fn render_overlay_body_with_no_bots_includes_assembled_at() {
        let payload = S2CDebugBotStatePush {
            bots: vec![],
            decision_log_total: 0,
            assembled_at_ms: 555,
        };
        let body = render_overlay_body(&payload);
        assert!(body.contains("No bots in session"));
        assert!(body.contains("555"));
    }

    #[test]
    fn render_overlay_body_includes_class_gold_mana_submitted_hand_and_decision() {
        let payload = make_payload_with_one_bot();
        let body = render_overlay_body(&payload);
        assert!(body.contains("Cra"));
        assert!(body.contains("gold=5"));
        assert!(body.contains("mana=3/4"));
        assert!(body.contains("submitted=false"));
        assert!(body.contains("hand=2"));
        assert!(body.contains("last_bid_val=3"));
        assert!(body.contains("auction_bid"));
        assert!(body.contains("card=7 amt=2 val=3"));
    }

    #[test]
    fn debug_bot_overlay_state_default_is_hidden_and_empty() {
        let state = DebugBotOverlayState::default();
        assert!(!state.visible);
        assert!(state.latest.is_none());
        assert_eq!(state.receive_count, 0);
    }
}
