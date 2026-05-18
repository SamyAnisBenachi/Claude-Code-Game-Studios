//! QA snapshot overlay — diagnostic tool for human UI testing.
//!
//! Adds a small top-right `Snapshot` button (and an `F9` keyboard shortcut)
//! that, on trigger, writes a full snapshot bundle to disk:
//!
//! - `snapshot.json` — structured dump of client UI / game state plus a
//!   `screenshot` metadata block (relative path, absolute path, format,
//!   `requested_at_ms`, `status`, optional `error`).
//! - `screenshot.png` — primary-window image captured via the Bevy 0.18
//!   screenshot API (`bevy::render::view::screenshot`).
//! - `README.md` — operator-facing description of the bundle.
//!
//! The trigger gives immediate in-game feedback by re-skinning the button
//! through a sequence of states — `Snapshot → Capturing… → Saved <id> →
//! Snapshot` (or `Failed <reason>`) — with a fixed width so the surrounding
//! layout never shifts. The same path runs whether the trigger is a click
//! or `F9`.
//!
//! ## Non-product rule
//!
//! - The plugin is registered unconditionally; the overlay / shortcut /
//!   capture systems only do work when [`QASnapshotConfig::enabled`] is
//!   `true`.
//! - Default activation rule (native, env-driven):
//!     - `CCGS_QA_SNAPSHOT=1` forces enabled.
//!     - `CCGS_QA_SNAPSHOT=0` forces disabled.
//!     - Unset (or empty/whitespace-only): defaults to
//!       `cfg!(debug_assertions)` — enabled in dev/debug builds, disabled in
//!       release builds. This keeps QA snapshot available out of the box for
//!       agents and manual QA without requiring an env var, while keeping
//!       release builds clean.
//!     - Any other value is logged as invalid and treated as disabled.
//! - Tests / harnesses may insert `QASnapshotConfig { enabled: true, .. }`
//!   directly before adding the plugin; `init_resource` will not overwrite
//!   the pre-inserted resource.
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
//! Capture is driven by `bevy::render::view::screenshot::Screenshot::primary_window()`
//! spawned with two observers:
//!
//! 1. `save_to_disk(<dir>/screenshot.png)` — the canonical Bevy helper, which
//!    handles native PNG writes and WASM browser-download saves uniformly.
//! 2. A second observer that emits a [`QASnapshotCaptureCompleted`] message
//!    so the JSON `screenshot.status` field can be flipped from `pending` to
//!    `captured` and the button feedback can advance to `Saved`.
//!
//! In headless tests (e.g. `MinimalPlugins`) the render world that processes
//! `Screenshot` entities is absent, so the second observer never fires, the
//! JSON stays at `screenshot.status == "pending"`, and the feedback state
//! falls back to `Failed("capture timeout")` after
//! [`QA_CAPTURE_TIMEOUT_SECS`] real seconds. This keeps the tool inert under
//! `MinimalPlugins` without any cfg juggling.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::time::Real;
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

/// Filename used for the captured screenshot inside every per-id snapshot
/// directory. Matches the `screenshot.relative_path` field written into
/// `snapshot.json`.
pub const QA_SCREENSHOT_FILENAME: &str = "screenshot.png";

/// Encoded screenshot format. Mirrors the file extension of
/// [`QA_SCREENSHOT_FILENAME`] and is surfaced in the JSON `screenshot.format`
/// field so downstream tooling does not have to parse the path.
pub const QA_SCREENSHOT_FORMAT: &str = "png";

/// Status sentinel emitted at snapshot-write time, before the render world
/// has had a chance to fire the `ScreenshotCaptured` observer.
pub const SCREENSHOT_STATUS_PENDING: &str = "pending";

/// Status sentinel applied once the `ScreenshotCaptured` observer reports a
/// successful PNG write.
pub const SCREENSHOT_STATUS_CAPTURED: &str = "captured";

/// Status sentinel applied when the JSON update path or PNG existence check
/// fails. The companion `error` field carries the human-readable reason.
pub const SCREENSHOT_STATUS_FAILED: &str = "failed";

/// Keyboard shortcut used to trigger a snapshot from either the lobby, an
/// active session, or the result screen. F9 was chosen because Bevy 0.18
/// does not bind it for any built-in dev tool, Windows leaves it free in
/// foreground apps, and it is reachable without modifiers from the standard
/// touch-typing posture used during human QA.
pub const QA_SNAPSHOT_SHORTCUT_KEY: KeyCode = KeyCode::F9;

/// Real-time (wall-clock) seconds the `Saved` / `Failed` button feedback
/// states linger before reverting to `Idle`. Short enough not to cover a
/// subsequent capture, long enough that a human operator can register the
/// transition without staring at the button.
pub const QA_FEEDBACK_REVERT_SECS: f32 = 1.5;

/// Real-time seconds the `Capturing…` feedback state can persist before it
/// is forcibly demoted to `Failed("capture timeout")`. The render world
/// usually delivers `ScreenshotCaptured` within one frame; the generous
/// budget here covers headless test environments and stutters.
pub const QA_CAPTURE_TIMEOUT_SECS: f32 = 3.0;

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
    /// Activation rule for `CCGS_QA_SNAPSHOT`:
    /// - `1` forces enabled.
    /// - `0` forces disabled.
    /// - Unset (or empty/whitespace-only): defaults to
    ///   `cfg!(debug_assertions)` — enabled in dev/debug builds, disabled in
    ///   release builds.
    /// - Any other value is logged as invalid and treated as disabled (never
    ///   panics).
    ///
    /// `CCGS_QA_SNAPSHOT_DIR=<path>` overrides the output directory; blank
    /// values fall back to the default.
    pub fn from_env() -> Self {
        Self::from_env_values(
            std::env::var(QA_SNAPSHOT_ENV_VAR).ok().as_deref(),
            std::env::var(QA_SNAPSHOT_DIR_ENV_VAR).ok().as_deref(),
            cfg!(debug_assertions),
        )
    }

    /// Deterministic constructor used by [`from_env`](Self::from_env) and the
    /// unit tests so the env-parsing rules can be exercised without touching
    /// the process environment or relying on the build-mode
    /// `debug_assertions` flag.
    ///
    /// `dev_default_enabled` is the value applied when `enable_var` is unset
    /// (`None`) or empty/whitespace only. Production callers pass
    /// `cfg!(debug_assertions)`.
    pub fn from_env_values(
        enable_var: Option<&str>,
        dir_var: Option<&str>,
        dev_default_enabled: bool,
    ) -> Self {
        let enabled = match enable_var.map(str::trim) {
            None | Some("") => dev_default_enabled,
            Some("1") => true,
            Some("0") => false,
            Some(other) => {
                tracing::warn!(
                    target: "client::presentation::qa_snapshot",
                    value = %other,
                    "CCGS_QA_SNAPSHOT has invalid value; treating as disabled (expected 1, 0, or unset)"
                );
                false
            }
        };
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
/// Decoupling the trigger from the write path keeps the write testable from
/// a minimal Bevy app without simulating UI input.
#[derive(Message, Debug, Clone, Copy, Default)]
pub struct QASnapshotRequested;

/// Message emitted by the screenshot observer once the render world reports
/// a captured frame. Consumed by [`apply_qa_snapshot_capture_completed_system`]
/// to flip the JSON `screenshot.status` to `captured` and the button
/// feedback to `Saved`.
#[derive(Message, Debug, Clone)]
pub struct QASnapshotCaptureCompleted {
    pub snapshot_id: String,
    pub json_path: PathBuf,
    pub png_path: PathBuf,
    pub captured_at_ms: u128,
}

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

/// Visible feedback state surfaced through the Snapshot button. The button's
/// pixel dimensions are pinned in [`spawn_qa_snapshot_overlay_system`] so
/// transitions never shift the surrounding layout.
#[derive(Resource, Debug, Clone)]
pub enum QASnapshotFeedbackState {
    Idle,
    Capturing {
        snapshot_id: String,
        since_real_seconds: f32,
    },
    Saved {
        snapshot_id: String,
        since_real_seconds: f32,
    },
    Failed {
        snapshot_id: String,
        reason: String,
        since_real_seconds: f32,
    },
}

impl Default for QASnapshotFeedbackState {
    fn default() -> Self {
        Self::Idle
    }
}

impl QASnapshotFeedbackState {
    pub fn label(&self) -> String {
        match self {
            Self::Idle => "Snapshot".to_string(),
            Self::Capturing { .. } => "Capturing…".to_string(),
            Self::Saved { snapshot_id, .. } => {
                format!("Saved {}", short_id(snapshot_id))
            }
            Self::Failed { reason, .. } => format!("Failed: {reason}"),
        }
    }

    pub fn bg_color(&self) -> Color {
        match self {
            Self::Idle => Color::srgba(0.08, 0.10, 0.14, 0.92),
            Self::Capturing { .. } => Color::srgba(0.14, 0.20, 0.30, 0.96),
            Self::Saved { .. } => Color::srgba(0.10, 0.32, 0.16, 0.96),
            Self::Failed { .. } => Color::srgba(0.40, 0.12, 0.12, 0.96),
        }
    }

    pub fn border_color(&self) -> Color {
        match self {
            Self::Idle => Color::srgba(0.96, 0.74, 0.30, 0.85),
            Self::Capturing { .. } => Color::srgba(0.78, 0.86, 1.00, 0.95),
            Self::Saved { .. } => Color::srgba(0.62, 0.96, 0.66, 0.95),
            Self::Failed { .. } => Color::srgba(0.98, 0.62, 0.62, 0.95),
        }
    }
}

fn short_id(snapshot_id: &str) -> &str {
    // S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001: format is now either
    // `{session_id}-{counter:06}-{unix_millis}` (post-handshake) or
    // `pre-session-{counter:06}-{unix_millis}`. The counter chunk is the
    // operator-relevant disambiguator (the session_id / pre-session
    // prefix only varies between clients, not between captures inside one
    // client), so we surface the counter token rather than the leading
    // prefix. Falls back to the full id when the format is unexpected.
    let mut parts = snapshot_id.split('-');
    if snapshot_id.starts_with(QA_SNAPSHOT_PRE_SESSION_PREFIX) {
        // `pre-session-<counter>-<ms>` → skip the two literal-prefix tokens.
        let _ = parts.next();
        let _ = parts.next();
    } else {
        // `<session_id>-<counter>-<ms>` → skip the session_id token.
        let _ = parts.next();
    }
    parts.next().unwrap_or(snapshot_id)
}

/// Serialised JSON shape written to disk on each snapshot. Public so tests
/// can deserialize and inspect the structure.
#[derive(Debug, Clone, Serialize)]
pub struct QASnapshotData {
    pub snapshot_id: String,
    pub counter: u64,
    pub unix_millis: u128,
    /// New: structured screenshot metadata. See [`ScreenshotInfo`].
    pub screenshot: ScreenshotInfo,
    pub client_state: String,
    pub current_phase: PhaseInfo,
    pub phase_view: PhaseViewInfo,
    pub session_identity: SessionIdentityInfo,
    pub window: WindowInfo,
    pub ui_counts: UiCounts,
    pub warnings: Vec<String>,
}

/// Metadata describing the screenshot bundled alongside `snapshot.json`.
///
/// `status` starts at [`SCREENSHOT_STATUS_PENDING`] and is flipped to
/// [`SCREENSHOT_STATUS_CAPTURED`] (or [`SCREENSHOT_STATUS_FAILED`]) by
/// [`apply_qa_snapshot_capture_completed_system`] once the render world
/// reports back.
#[derive(Debug, Clone, Serialize)]
pub struct ScreenshotInfo {
    pub relative_path: String,
    pub absolute_path: String,
    pub format: String,
    pub requested_at_ms: u128,
    pub status: String,
    pub captured_at_ms: Option<u128>,
    pub error: Option<String>,
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
///
/// `None` means the corresponding plugin / resource was not present at
/// snapshot time (a recorded warning, not a panic).
///
/// **S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001** (SOURCE-1077-08 + SOURCE-1077-09):
/// the legacy `hud_entities` / `hand_ui_entities` / `shop_auction_entities`
/// / `*_overlay_roots` fields measure the *spawned-tree size* (universal
/// markers, no visibility filter), which left every PROMPT 1022 / 1034 /
/// 1036 capture reporting the same constants regardless of phase or
/// overlay state. The new per-sub-surface `*_visible` fields measure the
/// *currently-visible* sub-surface count (per-sub-surface marker + own
/// `Visibility != Hidden` filter). Legacy fields are preserved as
/// `#[deprecated]` so historical snapshot comparisons (PROMPT 1022 / 1034
/// / 1036) still resolve for one Sprint cycle; future audits should
/// consume the new `*_visible` fields. The
/// `qa_snapshot_overlay_roots` field is intentionally not deprecated —
/// the QA snapshot overlay itself is not a per-phase UI surface; it
/// behaves like a singleton dev affordance and its count remains stable
/// at 1.
#[derive(Debug, Clone, Default, Serialize)]
#[allow(deprecated)]
pub struct UiCounts {
    // ── Legacy universal-marker spawned-tree counts (deprecated) ─────────
    /// Spawned-tree count of [`crate::ui::hud::HudEntity`] tagged entities.
    /// Deprecated: see struct doc.
    #[deprecated(
        since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
        note = "Use the per-sub-surface visible counts: hud_root_visible, \
                hud_top_strip_visible, hud_bottom_strip_visible, \
                hud_scoreboard_dot_visible, hud_dim_overlay_visible."
    )]
    pub hud_entities: usize,
    pub hud_timer_bars: usize,
    /// Spawned-tree count of [`crate::ui::hand::HandUiEntity`] tagged entities.
    /// Deprecated: see struct doc.
    #[deprecated(
        since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
        note = "Use hand_bar_visible, hand_fan_visible, hand_draft_grid_slot_visible, \
                placement_action_panel_visible."
    )]
    pub hand_ui_entities: usize,
    /// Spawned-tree count of [`crate::ui::shop_auction::ShopAuctionUiEntity`] tagged entities.
    /// Deprecated: see struct doc.
    #[deprecated(
        since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
        note = "Use shop_draft_offering_visible, shop_panel_visible, \
                auction_panel_visible, shop_footer_visible, auction_toast_visible, \
                settlement_overlay_visible."
    )]
    pub shop_auction_entities: usize,
    pub lobby_root_entities: usize,
    /// Spawned-marker count of `ResultScreenRoot` (no visibility filter).
    /// Deprecated: see struct doc and use `result_screen_visible` instead.
    #[deprecated(
        since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
        note = "Use result_screen_visible (honours Visibility) instead."
    )]
    pub result_screen_roots: usize,
    /// Spawned-marker count of `ConnectionLostOverlayRoot` (no visibility filter).
    /// Deprecated: see struct doc and use `connection_lost_overlay_visible` instead.
    #[deprecated(
        since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
        note = "Use connection_lost_overlay_visible (honours Visibility) instead."
    )]
    pub connection_lost_overlay_roots: usize,
    pub qa_snapshot_overlay_roots: usize,

    // ── S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 per-sub-surface visible counts ─
    /// Visible (own `Visibility != Hidden`) count of [`crate::ui::hud::HudRoot`].
    pub hud_root_visible: usize,
    /// Visible count of [`crate::ui::hud::HudTopStripRoot`].
    pub hud_top_strip_visible: usize,
    /// Visible count of [`crate::ui::hud::HudBottomStripRoot`].
    pub hud_bottom_strip_visible: usize,
    /// Visible count of [`crate::ui::hud::HudScoreboardDotRoot`].
    pub hud_scoreboard_dot_visible: usize,
    /// Visible count of [`crate::ui::hud::HudDimOverlayRoot`].
    pub hud_dim_overlay_visible: usize,
    /// Visible count of [`crate::ui::hand::HandBarRoot`].
    pub hand_bar_visible: usize,
    /// Visible count of [`crate::ui::hand::HandFanRoot`].
    pub hand_fan_visible: usize,
    /// Visible count of [`crate::ui::hand::HandDraftGridSlotRoot`].
    pub hand_draft_grid_slot_visible: usize,
    /// Visible count of [`crate::ui::hand::PlacementActionPanelRoot`].
    pub placement_action_panel_visible: usize,
    /// Visible count of [`crate::ui::shop_auction::ShopAuctionPanelRoot::DraftOffering`].
    pub shop_draft_offering_visible: usize,
    /// Visible count of [`crate::ui::shop_auction::ShopAuctionPanelRoot::Shop`].
    pub shop_panel_visible: usize,
    /// Visible count of [`crate::ui::shop_auction::ShopAuctionPanelRoot::Auction`].
    pub auction_panel_visible: usize,
    /// Visible count of [`crate::ui::shop_auction::ShopAuctionPanelRoot::ShopFooter`].
    pub shop_footer_visible: usize,
    /// Visible count of [`crate::ui::shop_auction::ShopAuctionPanelRoot::Toast`].
    pub auction_toast_visible: usize,
    /// Visible count of [`crate::ui::shop_auction::ShopAuctionPanelRoot::SettlementOverlay`].
    pub settlement_overlay_visible: usize,
    /// Visible count of [`crate::presentation::connection_lost_overlay::ConnectionLostOverlayRoot`].
    /// Honours `Visibility != Hidden`; replaces the constant-1
    /// `connection_lost_overlay_roots` reading flagged by SOURCE-1077-09.
    pub connection_lost_overlay_visible: usize,
    /// Visible count of [`crate::presentation::result_screen::ResultScreenRoot`].
    /// Honours `Visibility != Hidden`; replaces the constant-1
    /// `result_screen_roots` reading flagged by SOURCE-1077-09.
    pub result_screen_visible: usize,
}

/// Bundles the per-surface entity-count queries into a single
/// [`SystemParam`] so [`write_qa_snapshot_system`] stays under Bevy's
/// 16-param ceiling without losing any of the counts surfaced in
/// [`UiCounts`].
///
/// **S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001**: each query for a per-sub-surface
/// root marker reads `&Visibility` so the snapshot reports the number of
/// entities whose own `Visibility` is *not* `Visibility::Hidden`. The
/// legacy universal queries (`hud_entities` / `hand_ui_entities` /
/// `shop_auction_entities`) keep the original no-filter semantic so
/// historical snapshot comparisons (PROMPT 1022 / 1034 / 1036) continue
/// to resolve.
#[derive(SystemParam)]
#[allow(deprecated)]
pub struct UiCountQueries<'w, 's> {
    // Legacy universal queries — no visibility filter, preserved for backwards
    // compatibility with PROMPT 1022 / 1034 / 1036 captures.
    pub hud_entities: Query<'w, 's, (), With<crate::ui::hud::HudEntity>>,
    pub hud_timer_bars: Query<'w, 's, (), With<crate::ui::hud::HudTimerBar>>,
    pub hand_ui_entities: Query<'w, 's, (), With<crate::ui::hand::HandUiEntity>>,
    pub shop_auction_entities:
        Query<'w, 's, (), With<crate::ui::shop_auction::ShopAuctionUiEntity>>,
    pub lobby_roots: Query<'w, 's, (), With<crate::ui::lobby::LobbyRoot>>,
    pub qa_snapshot_overlay_roots: Query<'w, 's, (), With<QASnapshotOverlayRoot>>,
    // Per-sub-surface root queries — read `&Visibility` so `snapshot()` can
    // apply the "own Visibility != Hidden" filter (AC3).
    pub hud_root_visibility: Query<'w, 's, &'static Visibility, With<crate::ui::hud::HudRoot>>,
    pub hud_top_strip_visibility:
        Query<'w, 's, &'static Visibility, With<crate::ui::hud::HudTopStripRoot>>,
    pub hud_bottom_strip_visibility:
        Query<'w, 's, &'static Visibility, With<crate::ui::hud::HudBottomStripRoot>>,
    pub hud_scoreboard_dot_visibility:
        Query<'w, 's, &'static Visibility, With<crate::ui::hud::HudScoreboardDotRoot>>,
    pub hud_dim_overlay_visibility:
        Query<'w, 's, &'static Visibility, With<crate::ui::hud::HudDimOverlayRoot>>,
    pub hand_visibility: HandVisibilityQueries<'w, 's>,
    pub shop_auction_visibility: ShopAuctionVisibilityQueries<'w, 's>,
    pub connection_lost_overlay_visibility: Query<
        'w,
        's,
        &'static Visibility,
        With<crate::presentation::connection_lost_overlay::ConnectionLostOverlayRoot>,
    >,
    pub result_screen_visibility: Query<
        'w,
        's,
        &'static Visibility,
        With<crate::presentation::result_screen::ResultScreenRoot>,
    >,
}

/// Per-sub-surface hand-UI visibility queries, grouped so the parent
/// [`UiCountQueries`] stays under Bevy's 16-field [`SystemParam`] ceiling.
#[derive(SystemParam)]
pub struct HandVisibilityQueries<'w, 's> {
    pub hand_bar: Query<'w, 's, &'static Visibility, With<crate::ui::hand::HandBarRoot>>,
    pub hand_fan: Query<'w, 's, &'static Visibility, With<crate::ui::hand::HandFanRoot>>,
    pub hand_draft_grid_slot:
        Query<'w, 's, &'static Visibility, With<crate::ui::hand::HandDraftGridSlotRoot>>,
    pub placement_action_panel:
        Query<'w, 's, &'static Visibility, With<crate::ui::hand::PlacementActionPanelRoot>>,
}

/// Per-sub-surface shop/auction visibility queries, grouped so the parent
/// [`UiCountQueries`] stays under Bevy's 16-field [`SystemParam`] ceiling.
/// `ShopAuctionPanelRoot` is an enum carried on every panel-root entity;
/// we read it alongside `&Visibility` and discriminate by variant in
/// [`UiCountQueries::snapshot`].
#[derive(SystemParam)]
pub struct ShopAuctionVisibilityQueries<'w, 's> {
    pub panel_roots: Query<
        'w,
        's,
        (
            &'static Visibility,
            &'static crate::ui::shop_auction::ShopAuctionPanelRoot,
        ),
    >,
}

/// True when the entity's own [`Visibility`] is not [`Visibility::Hidden`].
/// Matches the semantic surfaced by `UiCounts::*_visible` fields under
/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 AC3. The check is intentionally
/// scoped to the marker entity's *own* `Visibility` component (rather than
/// the propagated `InheritedVisibility`) so the filter is observable from
/// a `MinimalPlugins` test world that does not register the
/// `VisibilityPlugin` propagation system.
#[inline]
fn is_visibility_visible(visibility: &Visibility) -> bool {
    !matches!(visibility, Visibility::Hidden)
}

impl<'w, 's> UiCountQueries<'w, 's> {
    #[allow(deprecated)]
    pub fn snapshot(&self) -> UiCounts {
        let hud_root_visible = self
            .hud_root_visibility
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let hud_top_strip_visible = self
            .hud_top_strip_visibility
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let hud_bottom_strip_visible = self
            .hud_bottom_strip_visibility
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let hud_scoreboard_dot_visible = self
            .hud_scoreboard_dot_visibility
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let hud_dim_overlay_visible = self
            .hud_dim_overlay_visibility
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let hand_bar_visible = self
            .hand_visibility
            .hand_bar
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let hand_fan_visible = self
            .hand_visibility
            .hand_fan
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let hand_draft_grid_slot_visible = self
            .hand_visibility
            .hand_draft_grid_slot
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let placement_action_panel_visible = self
            .hand_visibility
            .placement_action_panel
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();

        use crate::ui::shop_auction::ShopAuctionPanelRoot;
        let mut shop_draft_offering_visible = 0usize;
        let mut shop_panel_visible = 0usize;
        let mut auction_panel_visible = 0usize;
        let mut shop_footer_visible = 0usize;
        let mut auction_toast_visible = 0usize;
        let mut settlement_overlay_visible = 0usize;
        for (visibility, variant) in &self.shop_auction_visibility.panel_roots {
            if !is_visibility_visible(visibility) {
                continue;
            }
            match variant {
                ShopAuctionPanelRoot::DraftOffering => shop_draft_offering_visible += 1,
                ShopAuctionPanelRoot::Shop => shop_panel_visible += 1,
                ShopAuctionPanelRoot::Auction => auction_panel_visible += 1,
                ShopAuctionPanelRoot::ShopFooter => shop_footer_visible += 1,
                ShopAuctionPanelRoot::Toast => auction_toast_visible += 1,
                ShopAuctionPanelRoot::SettlementOverlay => settlement_overlay_visible += 1,
            }
        }

        let connection_lost_overlay_visible = self
            .connection_lost_overlay_visibility
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();
        let result_screen_visible = self
            .result_screen_visibility
            .iter()
            .filter(|v| is_visibility_visible(v))
            .count();

        UiCounts {
            hud_entities: self.hud_entities.iter().count(),
            hud_timer_bars: self.hud_timer_bars.iter().count(),
            hand_ui_entities: self.hand_ui_entities.iter().count(),
            shop_auction_entities: self.shop_auction_entities.iter().count(),
            lobby_root_entities: self.lobby_roots.iter().count(),
            // Legacy marker counts: report the spawned-tree size of each
            // overlay marker so PROMPT 1022 / 1034 / 1036 historical
            // comparisons still resolve. The visibility-aware reading is
            // exposed via the `*_visible` fields below.
            result_screen_roots: self.result_screen_visibility.iter().count(),
            connection_lost_overlay_roots: self.connection_lost_overlay_visibility.iter().count(),
            qa_snapshot_overlay_roots: self.qa_snapshot_overlay_roots.iter().count(),
            hud_root_visible,
            hud_top_strip_visible,
            hud_bottom_strip_visible,
            hud_scoreboard_dot_visible,
            hud_dim_overlay_visible,
            hand_bar_visible,
            hand_fan_visible,
            hand_draft_grid_slot_visible,
            placement_action_panel_visible,
            shop_draft_offering_visible,
            shop_panel_visible,
            auction_panel_visible,
            shop_footer_visible,
            auction_toast_visible,
            settlement_overlay_visible,
            connection_lost_overlay_visible,
            result_screen_visible,
        }
    }
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
            .init_resource::<QASnapshotFeedbackState>()
            .add_message::<QASnapshotRequested>()
            .add_message::<QASnapshotCaptureCompleted>()
            .add_systems(Startup, spawn_qa_snapshot_overlay_system)
            .add_systems(
                Update,
                (
                    qa_snapshot_button_click_system,
                    qa_snapshot_keyboard_shortcut_system,
                    write_qa_snapshot_system,
                    apply_qa_snapshot_capture_completed_system,
                    revert_qa_snapshot_feedback_state_system,
                    update_qa_snapshot_button_visuals_system,
                )
                    .chain(),
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
                width: Val::Px(160.0),
                height: Val::Px(32.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.5)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(QASnapshotFeedbackState::Idle.bg_color()),
            BorderColor::all(QASnapshotFeedbackState::Idle.border_color()),
            Text::new(QASnapshotFeedbackState::Idle.label()),
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
            tracing::info!(
                target: "client::presentation::qa_snapshot",
                trigger = "button",
                "QA snapshot trigger"
            );
            writer.write(QASnapshotRequested);
        }
    }
}

/// Translates a just-pressed [`QA_SNAPSHOT_SHORTCUT_KEY`] into a
/// `QASnapshotRequested` message. `ButtonInput<KeyCode>` is `Option`-wrapped
/// because test environments using `MinimalPlugins` do not register the
/// input plugin.
pub fn qa_snapshot_keyboard_shortcut_system(
    config: Res<QASnapshotConfig>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut writer: MessageWriter<QASnapshotRequested>,
) {
    if !config.enabled {
        return;
    }
    let Some(keys) = keys else {
        return;
    };
    if keys.just_pressed(QA_SNAPSHOT_SHORTCUT_KEY) {
        tracing::info!(
            target: "client::presentation::qa_snapshot",
            trigger = "shortcut",
            key = ?QA_SNAPSHOT_SHORTCUT_KEY,
            "QA snapshot trigger"
        );
        writer.write(QASnapshotRequested);
    }
}

/// Drains pending `QASnapshotRequested` messages and writes one bundle per
/// request: the snapshot JSON, the README, and a `Screenshot::primary_window`
/// entity whose observers save the PNG and report completion. The system also
/// flips [`QASnapshotFeedbackState`] to `Capturing` so the button reflects
/// the trigger immediately.
#[allow(clippy::too_many_arguments)]
pub fn write_qa_snapshot_system(
    mut commands: Commands,
    config: Res<QASnapshotConfig>,
    counter: Res<QASnapshotCounter>,
    mut requests: MessageReader<QASnapshotRequested>,
    mut feedback: ResMut<QASnapshotFeedbackState>,
    time: Res<Time<Real>>,
    state: Option<Res<State<ClientState>>>,
    current_phase: Option<Res<CurrentClientPhase>>,
    phase_view: Option<Res<ClientPhaseView>>,
    identity: Option<Res<ClientSessionIdentity>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_count_queries: UiCountQueries,
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
        let ui_counts = ui_count_queries.snapshot();

        let window = windows.iter().next();
        let counter_value = counter.next();
        let requested_at_ms = current_unix_millis();
        let identity_snapshot = identity.as_deref().copied();
        let snapshot_id = format_snapshot_id(
            counter_value,
            requested_at_ms,
            identity_snapshot.and_then(|id| id.session_id),
        );
        let snapshot_dir = config.output_dir.join(&snapshot_id);
        let png_path = snapshot_dir.join(QA_SCREENSHOT_FILENAME);
        let absolute_png_path = absolute_path(&png_path);
        let json_path = snapshot_dir.join("snapshot.json");

        let snapshot = build_snapshot(
            counter_value,
            requested_at_ms,
            ScreenshotInfo {
                relative_path: QA_SCREENSHOT_FILENAME.to_string(),
                absolute_path: absolute_png_path.to_string_lossy().into_owned(),
                format: QA_SCREENSHOT_FORMAT.to_string(),
                requested_at_ms,
                status: SCREENSHOT_STATUS_PENDING.to_string(),
                captured_at_ms: None,
                error: None,
            },
            state.as_deref().map(|s| *s.get()),
            current_phase.as_deref().copied(),
            phase_view.as_deref(),
            identity.as_deref().copied(),
            window,
            ui_counts,
        );

        match write_snapshot_to_dir(&config.output_dir, &snapshot) {
            Ok(json_path_actual) => {
                tracing::info!(
                    target: "client::presentation::qa_snapshot",
                    snapshot_id = %snapshot.snapshot_id,
                    output = %json_path_actual.display(),
                    "QA snapshot JSON + README written; awaiting screenshot capture"
                );
            }
            Err(err) => {
                tracing::warn!(
                    target: "client::presentation::qa_snapshot",
                    snapshot_id = %snapshot.snapshot_id,
                    error = %err,
                    "QA snapshot write failed"
                );
                *feedback = QASnapshotFeedbackState::Failed {
                    snapshot_id: snapshot.snapshot_id.clone(),
                    reason: format!("write json: {err}"),
                    since_real_seconds: time.elapsed_secs(),
                };
                continue;
            }
        }

        *feedback = QASnapshotFeedbackState::Capturing {
            snapshot_id: snapshot.snapshot_id.clone(),
            since_real_seconds: time.elapsed_secs(),
        };

        let observer_id = snapshot.snapshot_id.clone();
        let observer_png_path = png_path.clone();
        let observer_json_path = json_path.clone();

        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(png_path.clone()))
            .observe(
                move |_trigger: On<ScreenshotCaptured>,
                      mut writer: MessageWriter<QASnapshotCaptureCompleted>| {
                    writer.write(QASnapshotCaptureCompleted {
                        snapshot_id: observer_id.clone(),
                        json_path: observer_json_path.clone(),
                        png_path: observer_png_path.clone(),
                        captured_at_ms: current_unix_millis(),
                    });
                },
            );
    }
}

/// Reads completion messages emitted by the screenshot observer and:
///
/// - rewrites `snapshot.json` so the `screenshot.status` field flips from
///   `pending` to either `captured` or `failed` (with the corresponding
///   `captured_at_ms` / `error` fields),
/// - advances [`QASnapshotFeedbackState`] to `Saved` / `Failed`.
pub fn apply_qa_snapshot_capture_completed_system(
    mut reader: MessageReader<QASnapshotCaptureCompleted>,
    mut feedback: ResMut<QASnapshotFeedbackState>,
    time: Res<Time<Real>>,
) {
    for ev in reader.read() {
        let png_exists = ev.png_path.is_file();
        let result =
            update_snapshot_json_status(&ev.json_path, &ev.png_path, ev.captured_at_ms, png_exists);
        match result {
            Ok(()) if png_exists => {
                tracing::info!(
                    target: "client::presentation::qa_snapshot",
                    snapshot_id = %ev.snapshot_id,
                    png = %ev.png_path.display(),
                    "QA snapshot screenshot captured"
                );
                *feedback = QASnapshotFeedbackState::Saved {
                    snapshot_id: ev.snapshot_id.clone(),
                    since_real_seconds: time.elapsed_secs(),
                };
            }
            Ok(()) => {
                let reason = format!("png missing at {}", ev.png_path.display());
                tracing::warn!(
                    target: "client::presentation::qa_snapshot",
                    snapshot_id = %ev.snapshot_id,
                    reason = %reason,
                    "QA snapshot capture reported but png file is absent"
                );
                *feedback = QASnapshotFeedbackState::Failed {
                    snapshot_id: ev.snapshot_id.clone(),
                    reason,
                    since_real_seconds: time.elapsed_secs(),
                };
            }
            Err(err) => {
                let reason = format!("update json: {err}");
                tracing::warn!(
                    target: "client::presentation::qa_snapshot",
                    snapshot_id = %ev.snapshot_id,
                    error = %err,
                    "QA snapshot JSON status update failed"
                );
                *feedback = QASnapshotFeedbackState::Failed {
                    snapshot_id: ev.snapshot_id.clone(),
                    reason,
                    since_real_seconds: time.elapsed_secs(),
                };
            }
        }
    }
}

/// Demotes `Capturing` states past [`QA_CAPTURE_TIMEOUT_SECS`] to `Failed`
/// and clears `Saved` / `Failed` states once they have lingered past
/// [`QA_FEEDBACK_REVERT_SECS`]. Time uses `Time<Real>` so the timeout is
/// wall-clock rather than affected by gameplay pause / scaling.
pub fn revert_qa_snapshot_feedback_state_system(
    mut feedback: ResMut<QASnapshotFeedbackState>,
    time: Res<Time<Real>>,
) {
    let now = time.elapsed_secs();
    let next = match &*feedback {
        QASnapshotFeedbackState::Idle => None,
        QASnapshotFeedbackState::Capturing {
            snapshot_id,
            since_real_seconds,
        } if now - *since_real_seconds >= QA_CAPTURE_TIMEOUT_SECS => {
            Some(QASnapshotFeedbackState::Failed {
                snapshot_id: snapshot_id.clone(),
                reason: "capture timeout".to_string(),
                since_real_seconds: now,
            })
        }
        QASnapshotFeedbackState::Capturing { .. } => None,
        QASnapshotFeedbackState::Saved {
            since_real_seconds, ..
        }
        | QASnapshotFeedbackState::Failed {
            since_real_seconds, ..
        } if now - *since_real_seconds >= QA_FEEDBACK_REVERT_SECS => {
            Some(QASnapshotFeedbackState::Idle)
        }
        QASnapshotFeedbackState::Saved { .. } | QASnapshotFeedbackState::Failed { .. } => None,
    };
    if let Some(next) = next {
        *feedback = next;
    }
}

/// Mirrors [`QASnapshotFeedbackState`] onto the visible Snapshot button so
/// every trigger produces an immediate label / color change.
pub fn update_qa_snapshot_button_visuals_system(
    overlay: Option<Res<QASnapshotOverlayEntities>>,
    feedback: Res<QASnapshotFeedbackState>,
    mut texts: Query<&mut Text, With<QASnapshotButton>>,
    mut backgrounds: Query<&mut BackgroundColor, With<QASnapshotButton>>,
    mut borders: Query<&mut BorderColor, With<QASnapshotButton>>,
) {
    if overlay.is_none() {
        return;
    }
    let label = feedback.label();
    let bg = feedback.bg_color();
    let border = feedback.border_color();
    for mut text in &mut texts {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
    for mut background in &mut backgrounds {
        background.0 = bg;
    }
    for mut border_color in &mut borders {
        *border_color = BorderColor::all(border);
    }
}

/// Pure construction of a [`QASnapshotData`] from the world projections
/// gathered by [`write_qa_snapshot_system`]. Exposed so unit / integration
/// tests can exercise the serialization shape and warning behaviour
/// without driving a full Bevy app.
#[allow(clippy::too_many_arguments)]
pub fn build_snapshot(
    counter: u64,
    unix_millis: u128,
    screenshot: ScreenshotInfo,
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

    let snapshot_id = format_snapshot_id(counter, unix_millis, identity.and_then(|i| i.session_id));

    QASnapshotData {
        snapshot_id,
        counter,
        unix_millis,
        screenshot,
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
         This directory holds a single QA snapshot bundle:\n\n\
         - `snapshot.json` — structured dump of client state at trigger time.\n\
         - `{filename}` — primary-window screenshot captured via the Bevy 0.18 \
         `Screenshot::primary_window()` API. The `screenshot.status` field in \
         `snapshot.json` reflects whether the capture completed (`captured`), \
         is still in flight (`pending`), or failed (`failed`).\n\n\
         Triggered by clicking the in-game `Snapshot` button or pressing \
         `F9`. See `client/src/presentation/qa_snapshot.rs` for the source \
         of truth.\n",
        snapshot_id = snapshot.snapshot_id,
        filename = QA_SCREENSHOT_FILENAME,
    );
    fs::write(notes_path, notes)?;
    Ok(json_path)
}

/// Loads the snapshot JSON from `json_path`, updates the `screenshot`
/// block to either `captured` (with `captured_at_ms`) or `failed` (with
/// `error`), and writes it back. Exposed so integration tests can exercise
/// the update without a real Bevy app.
pub fn update_snapshot_json_status(
    json_path: &Path,
    png_path: &Path,
    captured_at_ms: u128,
    png_exists: bool,
) -> Result<(), String> {
    let raw = fs::read_to_string(json_path).map_err(|e| e.to_string())?;
    let mut value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let screenshot = value
        .get_mut("screenshot")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| "snapshot json missing `screenshot` object".to_string())?;
    if png_exists {
        screenshot.insert(
            "status".to_string(),
            serde_json::Value::String(SCREENSHOT_STATUS_CAPTURED.to_string()),
        );
        screenshot.insert(
            "captured_at_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(captured_at_ms as u64)),
        );
        screenshot.insert("error".to_string(), serde_json::Value::Null);
    } else {
        screenshot.insert(
            "status".to_string(),
            serde_json::Value::String(SCREENSHOT_STATUS_FAILED.to_string()),
        );
        screenshot.insert(
            "error".to_string(),
            serde_json::Value::String(format!("png missing at {}", png_path.display())),
        );
    }
    let updated = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    fs::write(json_path, updated).map_err(|e| e.to_string())?;
    Ok(())
}

fn current_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Prefix applied to snapshot ids captured before the handshake assigns a
/// [`ClientSessionIdentity::session_id`]. See
/// [`format_snapshot_id`].
pub const QA_SNAPSHOT_PRE_SESSION_PREFIX: &str = "pre-session";

/// Build the per-snapshot directory id.
///
/// **S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 (SOURCE-1077-16)**: the previous
/// `{counter:06}-{unix_millis}` format aliased across concurrent clients
/// — two clients running in parallel produced colliding `000000-*`
/// prefixes that differed only by millisecond-precision wall clock. The
/// new format inserts the handshake-assigned `session_id` as the leading
/// component so concurrent-client captures sort by client first:
///
/// - `{session_id}-{counter:06}-{unix_millis}` when the handshake has
///   landed.
/// - `pre-session-{counter:06}-{unix_millis}` when no `session_id` is
///   yet known (lobby / pre-handshake captures).
fn format_snapshot_id(counter: u64, unix_millis: u128, session_id: Option<u64>) -> String {
    match session_id {
        Some(id) => format!("{id}-{counter:06}-{unix_millis}"),
        None => format!("{QA_SNAPSHOT_PRE_SESSION_PREFIX}-{counter:06}-{unix_millis}"),
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(path),
        Err(_) => path.to_path_buf(),
    }
}
