//! QA snapshot overlay — diagnostic tool for human UI testing (PROMPT 1013).
//!
//! Adds a small top-right "Snapshot" button that, on click, captures a
//! structured JSON dump of the current client UI / game state to disk so a
//! human operator can pair it with a manual screenshot for later UI / state
//! analysis. The intent is to make Sprint 15 HUD timer eyeball and future UI
//! audits faster by removing the "what was the timer / phase / hand state
//! when this screenshot was taken?" guessing step.
//!
//! ## Non-product rule
//!
//! - Disabled by default. The plugin is registered unconditionally but the
//!   overlay button is only spawned and the snapshot systems only do work
//!   when [`QASnapshotConfig::enabled`] is `true`.
//! - Activation paths (native only):
//!     - Environment variable `CCGS_QA_SNAPSHOT=1` flips the default config to
//!       enabled at plugin build time.
//!     - Tests / harnesses may insert `QASnapshotConfig { enabled: true, .. }`
//!       directly before adding the plugin; `init_resource` will not overwrite
//!       the pre-inserted resource.
//! - Output directory is `qa-snapshots/` under the current working directory
//!   by default; override with `CCGS_QA_SNAPSHOT_DIR=<path>`.
//! - The plugin does not send any C2S messages, does not touch the lightyear
//!   protocol, does not change gameplay or networking, and does not alter
//!   normal UI layout when disabled. The overlay button paints on the
//!   [`z_layers::DEBUG`] layer so it sits above every production surface
//!   when shown.
//!
//! ## Screenshot capture
//!
//! Screenshot file capture is **intentionally not implemented** by this
//! plugin. Pulling in `bevy::render::view::screenshot::Screenshot` would
//! require enabling a render-pipeline feature that the WASM client build
//! does not currently carry, and the prompt explicitly forbids adding
//! dependencies for this. The structured JSON dump still ships every frame
//! and operators are expected to pair it with an OS-level screenshot (the
//! filename / id matches the directory the JSON is written into).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde::Serialize;

use crate::state::{ClientPhaseView, ClientSessionIdentity, ClientState, CurrentClientPhase};
use crate::ui::design_tokens::{typography, z_layers};

/// Environment variable that, when set to `1`, enables the QA snapshot
/// overlay at plugin build time on native builds.
pub const QA_SNAPSHOT_ENV_VAR: &str = "CCGS_QA_SNAPSHOT";

/// Environment variable that overrides the on-disk output directory for
/// captured snapshots. Defaults to `qa-snapshots/` under the current
/// working directory when unset.
pub const QA_SNAPSHOT_DIR_ENV_VAR: &str = "CCGS_QA_SNAPSHOT_DIR";

/// Default output directory for QA snapshots, resolved relative to the
/// process's current working directory at write time.
pub const DEFAULT_QA_SNAPSHOT_DIR: &str = "qa-snapshots";

/// Constant returned in the JSON `screenshot_status` field — documents that
/// the operator is expected to pair the structured dump with a manual
/// OS-level screenshot. See module doc for the rationale.
pub const SCREENSHOT_STATUS_MANUAL: &str = "manual_capture_required";

/// Plugin entry point. Always-safe to register; does nothing observable
/// when [`QASnapshotConfig::enabled`] is `false`.
pub struct QASnapshotPlugin;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct QASnapshotConfig {
    pub enabled: bool,
    pub output_dir: PathBuf,
}

impl Default for QASnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            output_dir: PathBuf::from(DEFAULT_QA_SNAPSHOT_DIR),
        }
    }
}

impl QASnapshotConfig {
    /// Build a config from environment variables.
    ///
    /// `CCGS_QA_SNAPSHOT=1` enables the overlay; any other value (including
    /// unset) keeps it disabled. `CCGS_QA_SNAPSHOT_DIR=<path>` overrides
    /// the output directory.
    pub fn from_env() -> Self {
        Self::from_env_values(
            std::env::var(QA_SNAPSHOT_ENV_VAR).ok().as_deref(),
            std::env::var(QA_SNAPSHOT_DIR_ENV_VAR).ok().as_deref(),
        )
    }

    /// Deterministic constructor used by `from_env` and the unit tests so
    /// the env-parsing rules can be exercised without touching the process
    /// environment.
    pub fn from_env_values(enable_var: Option<&str>, dir_var: Option<&str>) -> Self {
        let enabled = enable_var.map(str::trim).is_some_and(|v| v == "1");
        let output_dir = dir_var
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_QA_SNAPSHOT_DIR));
        Self {
            enabled,
            output_dir,
        }
    }
}

/// Monotonic counter for generating QA snapshot ids within a single
/// process run.
#[derive(Resource, Debug, Default)]
pub struct QASnapshotCounter(AtomicU64);

impl QASnapshotCounter {
    pub fn next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}

/// Message emitted whenever an operator (or test) requests a snapshot.
/// Decoupling the button-press path from the dump path keeps the dump
/// testable from a minimal Bevy app without simulating UI input.
#[derive(Message, Debug, Clone, Copy, Default)]
pub struct QASnapshotRequested;

/// Marker for the overlay root node.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct QASnapshotOverlayRoot;

/// Marker for the clickable Snapshot button.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct QASnapshotButton;

/// Resource holding the spawned overlay entity ids; only present when the
/// overlay is enabled.
#[derive(Resource, Debug, Clone, Copy)]
pub struct QASnapshotOverlayEntities {
    pub root: Entity,
    pub button: Entity,
}

/// Serialised JSON shape written to disk on each snapshot. Public so tests
/// can deserialize and inspect the structure.
#[derive(Debug, Clone, Serialize)]
pub struct QASnapshotData {
    pub snapshot_id: String,
    pub counter: u64,
    pub unix_millis: u128,
    pub screenshot_status: String,
    pub client_state: String,
    pub current_phase: PhaseInfo,
    pub phase_view: PhaseViewInfo,
    pub session_identity: SessionIdentityInfo,
    pub window: WindowInfo,
    pub ui_counts: UiCounts,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseInfo {
    pub phase: Option<String>,
    pub round: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseViewInfo {
    pub phase: Option<String>,
    pub round_number: Option<u32>,
    pub timer_duration_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionIdentityInfo {
    /// Bevy `PlayerId` is server-assigned; we surface its Debug form so a
    /// snapshot can be cross-referenced with server logs without coupling
    /// this module to the shared protocol's accessor surface.
    pub player_id: Option<String>,
    pub session_id: Option<u64>,
    pub has_session_token: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub scale_factor: Option<f32>,
}

/// Counts of bevy_ui entities matching each named UI surface marker.
/// `None` means the corresponding plugin / resource was not present at
/// snapshot time (a recorded warning, not a panic).
#[derive(Debug, Clone, Default, Serialize)]
pub struct UiCounts {
    pub hud_entities: usize,
    pub hud_timer_bars: usize,
    pub hand_ui_entities: usize,
    pub shop_auction_entities: usize,
    pub lobby_root_entities: usize,
    pub result_screen_roots: usize,
    pub connection_lost_overlay_roots: usize,
    pub qa_snapshot_overlay_roots: usize,
}

impl Plugin for QASnapshotPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<QASnapshotConfig>() {
            let config = QASnapshotConfig::from_env();
            tracing::info!(
                target: "client::presentation::qa_snapshot",
                enabled = config.enabled,
                output_dir = %config.output_dir.display(),
                "QASnapshotPlugin: config from env"
            );
            app.insert_resource(config);
        } else {
            tracing::info!(
                target: "client::presentation::qa_snapshot",
                "QASnapshotPlugin: using pre-inserted QASnapshotConfig"
            );
        }

        app.init_resource::<QASnapshotCounter>()
            .add_message::<QASnapshotRequested>()
            .add_systems(Startup, spawn_qa_snapshot_overlay_system)
            .add_systems(
                Update,
                (
                    qa_snapshot_button_click_system,
                    write_qa_snapshot_system.after(qa_snapshot_button_click_system),
                ),
            );
    }
}

/// Spawns the overlay root + button when enabled. No-op when disabled so
/// the production UI tree is untouched.
pub fn spawn_qa_snapshot_overlay_system(
    mut commands: Commands,
    config: Res<QASnapshotConfig>,
    existing: Option<Res<QASnapshotOverlayEntities>>,
) {
    if !config.enabled || existing.is_some() {
        return;
    }

    let root = commands
        .spawn((
            Name::new("QA snapshot overlay root"),
            QASnapshotOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                ..default()
            },
            z_layers::DEBUG,
        ))
        .id();

    let button = commands
        .spawn((
            Name::new("QA snapshot button"),
            QASnapshotButton,
            ChildOf(root),
            Button,
            Node {
                width: Val::Px(96.0),
                height: Val::Px(32.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.5)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.92)),
            BorderColor::all(Color::srgba(0.96, 0.74, 0.30, 0.85)),
            Text::new("Snapshot"),
            TextFont {
                font_size: typography::BODY,
                ..default()
            },
            TextColor(Color::srgb(0.98, 0.96, 0.86)),
        ))
        .id();

    commands.insert_resource(QASnapshotOverlayEntities { root, button });

    tracing::info!(
        target: "client::presentation::qa_snapshot",
        root = ?root,
        button = ?button,
        "QA snapshot overlay spawned"
    );
}

/// Translates a Pressed Interaction transition on the Snapshot button into
/// a `QASnapshotRequested` message. Inert when the overlay is disabled
/// (no button entity exists, the query is empty).
pub fn qa_snapshot_button_click_system(
    config: Res<QASnapshotConfig>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<QASnapshotButton>)>,
    mut writer: MessageWriter<QASnapshotRequested>,
) {
    if !config.enabled {
        return;
    }
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            writer.write(QASnapshotRequested);
        }
    }
}

/// Drains pending `QASnapshotRequested` messages and writes one JSON file
/// per request. Captures everything readable at the time the system runs.
#[allow(clippy::too_many_arguments)]
pub fn write_qa_snapshot_system(
    config: Res<QASnapshotConfig>,
    counter: Res<QASnapshotCounter>,
    mut requests: MessageReader<QASnapshotRequested>,
    state: Option<Res<State<ClientState>>>,
    current_phase: Option<Res<CurrentClientPhase>>,
    phase_view: Option<Res<ClientPhaseView>>,
    identity: Option<Res<ClientSessionIdentity>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    hud_entities: Query<(), With<crate::ui::hud::HudEntity>>,
    hud_timer_bars: Query<(), With<crate::ui::hud::HudTimerBar>>,
    hand_ui_entities: Query<(), With<crate::ui::hand::HandUiEntity>>,
    shop_auction_entities: Query<(), With<crate::ui::shop_auction::ShopAuctionUiEntity>>,
    lobby_roots: Query<(), With<crate::ui::lobby::LobbyRoot>>,
    result_screen_roots: Query<(), With<crate::presentation::result_screen::ResultScreenRoot>>,
    connection_lost_overlay_roots: Query<
        (),
        With<crate::presentation::connection_lost_overlay::ConnectionLostOverlayRoot>,
    >,
    qa_snapshot_overlay_roots: Query<(), With<QASnapshotOverlayRoot>>,
) {
    if requests.is_empty() {
        return;
    }
    let request_count = requests.read().count();
    if !config.enabled {
        tracing::debug!(
            target: "client::presentation::qa_snapshot",
            request_count,
            "QA snapshot requested while disabled — skipping write"
        );
        return;
    }
    for _ in 0..request_count {
        let ui_counts = UiCounts {
            hud_entities: hud_entities.iter().count(),
            hud_timer_bars: hud_timer_bars.iter().count(),
            hand_ui_entities: hand_ui_entities.iter().count(),
            shop_auction_entities: shop_auction_entities.iter().count(),
            lobby_root_entities: lobby_roots.iter().count(),
            result_screen_roots: result_screen_roots.iter().count(),
            connection_lost_overlay_roots: connection_lost_overlay_roots.iter().count(),
            qa_snapshot_overlay_roots: qa_snapshot_overlay_roots.iter().count(),
        };

        let window = windows.iter().next();
        let snapshot = build_snapshot(
            counter.next(),
            current_unix_millis(),
            state.as_deref().map(|s| *s.get()),
            current_phase.as_deref().copied(),
            phase_view.as_deref(),
            identity.as_deref().copied(),
            window,
            ui_counts,
        );

        match write_snapshot_to_dir(&config.output_dir, &snapshot) {
            Ok(path) => {
                tracing::info!(
                    target: "client::presentation::qa_snapshot",
                    snapshot_id = %snapshot.snapshot_id,
                    output = %path.display(),
                    "QA snapshot written"
                );
            }
            Err(err) => {
                tracing::warn!(
                    target: "client::presentation::qa_snapshot",
                    snapshot_id = %snapshot.snapshot_id,
                    error = %err,
                    "QA snapshot write failed"
                );
            }
        }
    }
}

/// Pure construction of a [`QASnapshotData`] from the world projections
/// gathered by [`write_qa_snapshot_system`]. Exposed so unit / integration
/// tests can exercise the serialization shape and warning behaviour
/// without driving a full Bevy app.
pub fn build_snapshot(
    counter: u64,
    unix_millis: u128,
    state: Option<ClientState>,
    current_phase: Option<CurrentClientPhase>,
    phase_view: Option<&ClientPhaseView>,
    identity: Option<ClientSessionIdentity>,
    window: Option<&Window>,
    ui_counts: UiCounts,
) -> QASnapshotData {
    let mut warnings: Vec<String> = Vec::new();

    let client_state = match state {
        Some(s) => format!("{:?}", s),
        None => {
            warnings.push("ClientState resource missing".to_string());
            "unknown".to_string()
        }
    };

    let current_phase_info = match current_phase {
        Some(p) => PhaseInfo {
            phase: Some(format!("{:?}", p.phase)),
            round: Some(p.round),
        },
        None => {
            warnings.push("CurrentClientPhase resource missing".to_string());
            PhaseInfo {
                phase: None,
                round: None,
            }
        }
    };

    let phase_view_info = match phase_view {
        Some(p) => PhaseViewInfo {
            phase: Some(format!("{:?}", p.phase)),
            round_number: Some(p.round_number),
            timer_duration_ms: Some(p.timer_duration_ms),
        },
        None => {
            warnings.push("ClientPhaseView resource missing".to_string());
            PhaseViewInfo {
                phase: None,
                round_number: None,
                timer_duration_ms: None,
            }
        }
    };

    let session_identity_info = match identity {
        Some(id) => SessionIdentityInfo {
            player_id: id.player_id.map(|p| format!("{:?}", p)),
            session_id: id.session_id,
            has_session_token: id.session_token.is_some(),
        },
        None => {
            warnings.push("ClientSessionIdentity resource missing".to_string());
            SessionIdentityInfo {
                player_id: None,
                session_id: None,
                has_session_token: false,
            }
        }
    };

    let window_info = match window {
        Some(w) => WindowInfo {
            width: Some(w.resolution.width()),
            height: Some(w.resolution.height()),
            scale_factor: Some(w.resolution.scale_factor()),
        },
        None => {
            warnings.push("PrimaryWindow not found".to_string());
            WindowInfo {
                width: None,
                height: None,
                scale_factor: None,
            }
        }
    };

    let snapshot_id = format!("{counter:06}-{unix_millis}");

    QASnapshotData {
        snapshot_id,
        counter,
        unix_millis,
        screenshot_status: SCREENSHOT_STATUS_MANUAL.to_string(),
        client_state,
        current_phase: current_phase_info,
        phase_view: phase_view_info,
        session_identity: session_identity_info,
        window: window_info,
        ui_counts,
        warnings,
    }
}

/// Writes the snapshot JSON into `<output_dir>/<snapshot_id>/snapshot.json`,
/// creating directories as needed. Returns the absolute path of the JSON
/// file on success.
pub fn write_snapshot_to_dir(
    output_dir: &Path,
    snapshot: &QASnapshotData,
) -> Result<PathBuf, std::io::Error> {
    let dir = output_dir.join(&snapshot.snapshot_id);
    fs::create_dir_all(&dir)?;
    let json_path = dir.join("snapshot.json");
    let json = serde_json::to_string_pretty(snapshot).map_err(std::io::Error::other)?;
    fs::write(&json_path, json)?;
    let notes_path = dir.join("README.md");
    let notes = format!(
        "# QA snapshot {snapshot_id}\n\n\
         Pair `snapshot.json` with a manual OS-level screenshot saved as \
         `screenshot.png` in this directory. `screenshot_status` in the \
         JSON documents that automated capture is intentionally not \
         performed (see `client/src/presentation/qa_snapshot.rs` module \
         doc).\n",
        snapshot_id = snapshot.snapshot_id
    );
    fs::write(notes_path, notes)?;
    Ok(json_path)
}

fn current_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
