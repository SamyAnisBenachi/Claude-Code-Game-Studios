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
//!
//! ## Overlay-exclude-on-capture (S18-QA-SNAPSHOT-OVERLAY-EXCLUDE-ON-CAPTURE)
//!
//! The Snapshot button doubles as live click feedback (`Snapshot → Capturing…
//! → Saved <id>`). Without intervention the in-flight `Capturing…` chip
//! appears inside every captured PNG and pollutes the visual evidence (see
//! reports/PROMPT-1129-current-ui-visual-quality-deep-audit.md, UI-1129-16).
//!
//! Resolution: the overlay root's [`Visibility`] is now a derived view of
//! [`QASnapshotFeedbackState`] —
//! [`apply_qa_snapshot_overlay_visibility_system`] flips the root to
//! [`Visibility::Hidden`] for the duration of `Capturing` and back to
//! [`Visibility::Inherited`] on every other state (`Idle` / `Saved` /
//! `Failed`). The mirror system runs after every state-mutating system in
//! the QA chain but still inside `Update`, so visibility propagation
//! (`PostUpdate`) lands before the render schedule that consumes the
//! `Screenshot` entity. The captured frame paints the game UI without the
//! QA overlay; the operator still sees the post-capture `Saved <id>` /
//! `Failed <reason>` chip in the very next painted frame.
//!
//! Splitting visibility into its own system (rather than mutating it
//! inside `write_qa_snapshot_system` / the completion / revert systems)
//! keeps those systems' queries disjoint from [`UiCountQueries`], whose
//! per-sub-surface `&Visibility` reads would otherwise trigger a Bevy
//! B0001 conflict against a co-resident `&mut Visibility` query.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::time::Real;
use bevy::ui::{ComputedNode, GlobalZIndex, UiGlobalTransform, UiScale};
use bevy::window::PrimaryWindow;
use serde::Serialize;

use crate::presentation::board_rendering::{
    BoardLocalPlayer, BoardRenderState, BoardUnit, BoardUnitCard, BoardUnitOwner, BoardUnitStats,
    ObjectiveArtKind, StandingObjective, StandingObjectiveArt, StandingObjectiveHp,
};
use crate::presentation::shared::economy_view::{PlayerEconomyView, PlayerEconomyViewUpdateSource};
use crate::state::{
    ClientObjectiveIdentities, ClientPhaseView, ClientSessionIdentity, ClientState,
    CurrentClientPhase, OpponentConnectionView, SessionLifecycleView, SessionSettingsView,
};
use crate::ui::design_tokens::{typography, z_layers};
use crate::ui::hand::{
    ActiveGhostUnstageDrag, ActivePlacementDrag, HandContents, HandUiMode, HandUiOutboundMessages,
    PendingPlacements, PlacementBoardView, PlacementDisclosureState, PlacementDisclosureStep,
    PlacementTargetKind, PlacementTimer,
};
use crate::ui::hud::{HudClassReveal, HudMode, HudPlayerIds, PhaseTimerState};
use crate::ui::shop_auction::{
    AuctionBidKeyboardFocus, AuctionLocallyPassed, ShopAuctionAuctionPanelState,
    ShopAuctionAuctionState, ShopAuctionDraftInitialState, ShopAuctionLocalGoldView,
    ShopAuctionSettlementOutcome, ShopAuctionSettlementState, ShopAuctionShopState,
    ShopAuctionShopTimerState, ShopAuctionToastState, ShopAuctionUiMode,
    ShopAuctionUiOutboundMessages,
};

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
    /// PROMPT 1132 — broader observability bag covering timers, resources,
    /// hand, drag, shop/auction, board, HUD, settings, intents. Every
    /// sub-field is `Option` (or empty by default) so missing resources or
    /// disabled plugins serialise as `null` / `[]` rather than panicking.
    pub extras: ExtrasSnapshot,
    /// PROMPT 1186 (S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001) — layout-debug
    /// fields covering Q-01..Q-10 from the PROMPT 1180 audit. Captures
    /// viewport size, per-surface root bounds (logical px), child counts,
    /// resolved z-layer, content-overflow signal, button affordance states,
    /// and explicit collision helpers for placement_action_panel and
    /// shop_panel vs hand_bar. Sub-fields are `Option`/empty when not
    /// computable from current ECS data — see
    /// [`LayoutSnapshot::limitations`] for the documented gaps.
    pub layout: LayoutSnapshot,
    /// PROMPT 1229 (S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001 /
    /// B-1203-X-03) — top-level placement-phase state lift, projected
    /// from the same `extras.hand` / `extras.drag` / `extras.timers`
    /// resources but surfaced at the top of `snapshot.json` so PROMPT 1203
    /// audits can correlate visual mismatches against the local placement
    /// intent without descending into `extras.*`. Schema is stable across
    /// phases: `available = false` + nested `null`s outside the placement
    /// phase. See [`PlacementStateSnapshot`].
    pub placement_state: PlacementStateSnapshot,
    /// PROMPT 1229 (S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001 /
    /// B-1203-X-03) — top-level auction-phase state lift, projected from
    /// the same `extras.shop_auction.auction` / `extras.resources.*`
    /// resources but surfaced at the top of `snapshot.json` so PROMPT 1203
    /// audits can correlate the QA screenshot against bid / leader /
    /// timer / local-gold state without spelunking. Schema is stable
    /// across phases: `available = false` + nested `null`s outside the
    /// auction phase. See [`AuctionStateSnapshot`].
    pub auction_state: AuctionStateSnapshot,
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
    /// PROMPT 1229 (S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001 /
    /// B-1203-X-03). Round-phase countdown remaining in milliseconds,
    /// projected from [`crate::ui::hud::PhaseTimerState`] via
    /// `duration_ms.saturating_sub(elapsed_ms)`. `None` when the timer
    /// resource is absent (lobby / pre-handshake) — never inferred from
    /// fixtures.
    pub timer_remaining_ms: Option<u32>,
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

/// PROMPT 1229 (S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001 /
/// B-1203-X-03). Top-level placement-phase state lifted out of
/// [`ExtrasSnapshot`] so audits can correlate the QA screenshot against
/// the active local placement intent without source archaeology. Every
/// field is `Option` (or `false` on the boolean availability flag) so a
/// snapshot captured outside the placement phase still produces a stable
/// JSON shape: `available = false`, every nested field `null`. Read-only
/// projection of [`crate::ui::hand::PendingPlacements`],
/// [`crate::ui::hand::PlacementTimer`],
/// [`crate::ui::hand::PlacementDisclosureState`], and
/// [`crate::ui::hand::ActivePlacementDrag`] — never mutates those
/// resources.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PlacementStateSnapshot {
    /// `true` when at least one placement-phase resource was present at
    /// snapshot time (so the other fields carry meaningful nulls vs
    /// values). `false` for lobby / pre-handshake / non-placement-phase
    /// captures.
    pub available: bool,
    /// `PendingPlacements::staged_count()`. `None` when the resource is
    /// absent.
    pub staged_count: Option<usize>,
    /// `staged_count > 0 && !submitted` — the same gate the placement
    /// submit button uses to decide whether the SubmitPlacements C2S can
    /// fire. `None` when the source resources are absent.
    pub can_submit: Option<bool>,
    /// `PlacementTimer::submitted`. `None` when the timer resource is
    /// absent.
    pub submitted: Option<bool>,
    /// `true` when `ActivePlacementDrag::is_active()` (card + target_kind
    /// both set). `None` when the resource is absent.
    pub drag_active: Option<bool>,
    /// `ActivePlacementDrag::card_id` when a drag is active. `None`
    /// otherwise.
    pub drag_card_id: Option<u32>,
    /// `ActivePlacementDrag::target_kind` projected as a stable string
    /// token (matches [`target_kind_name`]). `None` when no drag is
    /// active.
    pub drag_target_kind: Option<String>,
    /// `PlacementDisclosureState::step` projected via
    /// [`disclosure_step_name`]. Surfaces the local "what step is the
    /// staging UI on" signal — `Hidden` / `CardSelection` /
    /// `TargetSelection(<kind>)` / `StagedCard` / `Correction(<err>)` /
    /// `Submitted`. `None` when the resource is absent.
    pub disclosure_step: Option<String>,
}

/// PROMPT 1229 (S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001 /
/// B-1203-X-03). Top-level auction-phase state lifted out of
/// [`ExtrasSnapshot`] so audits can diagnose bid/leader/timer mismatches
/// from the JSON without spelunking through `extras.shop_auction.auction`
/// and `extras.resources.local_gold_view`. Sub-fields are `Option` so a
/// snapshot captured outside the auction phase produces a stable JSON
/// shape (`available = false`, nested fields `null`). Read-only projection
/// of [`crate::ui::shop_auction::ShopAuctionAuctionState`] +
/// [`crate::ui::shop_auction::ShopAuctionLocalGoldView`] +
/// [`crate::presentation::shared::economy_view::PlayerEconomyView`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct AuctionStateSnapshot {
    /// `true` when [`ShopAuctionAuctionState`] was present at snapshot
    /// time (so the other fields carry meaningful nulls vs values).
    pub available: bool,
    /// `ShopAuctionAuctionState::panel_state` projected via
    /// [`auction_panel_state_name`] (`Hidden` / `Preparing` / `Active` /
    /// `Settling` / `ConnectionError`). `None` when unavailable.
    pub panel_state: Option<String>,
    /// `ShopAuctionAuctionState::card_id`. `None` when no card is
    /// featured (pre-Preparing, between cards, or unavailable).
    pub card_id: Option<u32>,
    /// `ShopAuctionAuctionState::starting_price`. `None` when
    /// unavailable.
    pub starting_price: Option<u32>,
    /// `ShopAuctionAuctionState::current_price` — the highest accepted
    /// bid, or `starting_price` if no bid has landed yet. `None` when
    /// unavailable.
    pub current_price: Option<u32>,
    /// `ShopAuctionAuctionState::current_leader` projected as Debug.
    /// `None` when no bid has landed yet OR the auction resource is
    /// absent — disambiguate via [`Self::available`].
    pub current_leader: Option<String>,
    /// `ShopAuctionAuctionState::timer_duration_ms`. `None` when
    /// unavailable.
    pub timer_duration_ms: Option<u32>,
    /// `ShopAuctionAuctionState::timer_remaining_ms`. `None` when
    /// unavailable.
    pub timer_remaining_ms: Option<u32>,
    /// `ShopAuctionAuctionState::in_flight_bid_amount` — the local
    /// player's pending bid amount awaiting server confirmation. `None`
    /// when no in-flight bid is staged or the resource is absent.
    pub local_in_flight_bid_amount: Option<u32>,
    /// `ShopAuctionLocalGoldView::initialized && PlayerEconomyView`
    /// resolved local gold projection. `None` when either source is
    /// absent — the inner [`AuctionLocalGoldSnapshot`] carries the
    /// per-field nulls.
    pub local_gold: Option<AuctionLocalGoldSnapshot>,
}

/// Per-snapshot projection of the local player's gold state at auction
/// time: total gold, gold reserved for in-flight bids, and free
/// (spendable) gold. Sourced from
/// [`crate::ui::shop_auction::ShopAuctionLocalGoldView`] +
/// [`crate::presentation::shared::economy_view::PlayerEconomyView`].
#[derive(Debug, Clone, Serialize)]
pub struct AuctionLocalGoldSnapshot {
    /// `ShopAuctionLocalGoldView::gold` (fallback: `PlayerEconomyView::gold`
    /// when the gold view is not yet initialised).
    pub gold: u32,
    /// `ShopAuctionLocalGoldView::reserved_gold` (0 when the view is not
    /// initialised — matches the same fallback used by
    /// [`ShopAuctionLocalGoldView::free_gold`]).
    pub reserved_gold: u32,
    /// `ShopAuctionLocalGoldView::free_gold(economy)` — the value the
    /// auction UI feeds to bid affordability checks.
    pub free_gold: u32,
    /// `true` when the gold view itself has been initialised by a
    /// `S2CGoldUpdate` (i.e. the broadcast gate is satisfied); `false`
    /// when the snapshot fell back to `PlayerEconomyView::gold`.
    pub view_initialized: bool,
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

// ─────────────────────────────────────────────────────────────────────────
// PROMPT 1132 — ExtrasSnapshot + ExtrasInputs (broader observability bag)
// ─────────────────────────────────────────────────────────────────────────

/// PROMPT 1132 — broader observability bag added to every QA snapshot. Each
/// sub-field is `Option` (or an empty `Vec`) so a missing resource / disabled
/// plugin serialises as `null` / `[]` rather than panicking. The struct is
/// `#[serde(default)]`-friendly via [`Default`] so downstream tooling can
/// deserialize older snapshots that pre-date this field.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ExtrasSnapshot {
    /// `bevy::diagnostic::FrameCount` (frames since app start); `None` when
    /// the diagnostics plugin is absent (e.g. `MinimalPlugins` tests).
    pub frame_count: Option<u64>,
    /// Local + opponent `PlayerId` projected as Debug strings so the value
    /// can be cross-referenced with server logs without coupling this module
    /// to the protocol accessor surface.
    pub players: PlayerIdsSnapshot,
    /// Aggregated timers (phase / placement / auction / shop). Sub-fields
    /// are `None` when the source resource is absent.
    pub timers: TimersSnapshot,
    /// Local player's economy + per-panel free-gold mirror.
    pub resources: Option<PlayerResourcesSnapshot>,
    /// Hand contents, mode, disclosure step, pending placements.
    pub hand: Option<HandSnapshot>,
    /// Active drag state (placement drag + ghost unstage drag).
    pub drag: DragSnapshot,
    /// Shop / draft / auction surfaces.
    pub shop_auction: Option<ShopAuctionExtrasSnapshot>,
    /// HUD-side state (mode, class reveal).
    pub hud: Option<HudExtrasSnapshot>,
    /// Board entities (units + standing objectives), capped to
    /// [`MAX_BOARD_ENTITIES_PER_KIND`] each to keep the payload compact.
    pub board: BoardSnapshot,
    /// `BoardRenderState` enum (Debug name) — observability into the
    /// pre/post-resolution gating.
    pub board_render_state: Option<String>,
    /// Local player's `SessionSettingsView` (placement timer multiplier).
    pub session_settings: Option<SessionSettingsSnapshot>,
    /// Unicast-revealed objective identities (ADR-001).
    pub objective_identities: Vec<ObjectiveIdentitySnapshot>,
    /// Opponent connection indicator (grace remaining when disconnected).
    pub opponent_connection: Option<OpponentConnectionSnapshot>,
    /// Session cancellation reason (Lifecycle view).
    pub session_lifecycle: Option<SessionLifecycleSnapshot>,
    /// Pending outbound message buffer sizes — never the message bodies, so
    /// the payload stays compact regardless of churn.
    pub outbound_intents: OutboundIntentsSnapshot,
}

/// Maximum number of board entities of one kind (units OR objectives)
/// serialised into a snapshot. 60 covers the 15-cell board × 2 owners with
/// headroom for ghosts; the cap keeps the JSON bounded if any future
/// regression spawns extra entities.
pub const MAX_BOARD_ENTITIES_PER_KIND: usize = 60;

#[derive(Debug, Clone, Default, Serialize)]
pub struct PlayerIdsSnapshot {
    pub local_player_id: Option<String>,
    pub opponent_player_id: Option<String>,
    /// Source of `opponent_player_id`: `placement_board_view`,
    /// `hud_player_ids`, or `unknown`. Helps disambiguate when one source
    /// disagrees with another.
    pub opponent_source: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TimersSnapshot {
    pub phase_timer: Option<PhaseTimerSnapshot>,
    pub placement_timer: Option<PlacementTimerSnapshot>,
    pub auction_timer: Option<AuctionTimerSnapshot>,
    pub shop_timer: Option<ShopTimerSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseTimerSnapshot {
    pub duration_ms: u32,
    pub elapsed_ms: u32,
    pub remaining_ms: u32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlacementTimerSnapshot {
    pub remaining_ms: u32,
    pub urgency_fired: bool,
    pub in_grace_window: bool,
    pub grace_remaining_ms: u32,
    pub submitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuctionTimerSnapshot {
    pub panel_state: String,
    pub duration_ms: u32,
    pub remaining_ms: u32,
    pub preparing_elapsed_ms: u32,
    pub locally_expired_elapsed_ms: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShopTimerSnapshot {
    pub duration_ms: u32,
    pub remaining_ms: u32,
    pub started: bool,
    pub deferred: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerResourcesSnapshot {
    pub initialized: bool,
    pub last_update_source: Option<String>,
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u8,
    pub local_gold_view: Option<LocalGoldViewSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalGoldViewSnapshot {
    pub initialized: bool,
    pub gold: u32,
    pub reserved_gold: u32,
    pub free_gold: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct HandSnapshot {
    pub mode: Option<String>,
    pub disclosure_step: Option<String>,
    pub hand_count: usize,
    pub cards: Vec<HandCardSnapshot>,
    pub pending_placements: Vec<PendingPlacementSnapshot>,
    pub staged_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HandCardSnapshot {
    pub card_id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PendingPlacementSnapshot {
    pub card_id: u32,
    pub target: String,
    pub current_mana_spend: u32,
    pub reserve_mana_spend: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DragSnapshot {
    pub placement_drag_active: bool,
    pub placement_drag_card_id: Option<u32>,
    pub placement_drag_card_entity: Option<String>,
    pub placement_drag_owner_id: Option<String>,
    pub placement_drag_target_kind: Option<String>,
    pub placement_drag_cursor_world: Option<[f32; 2]>,
    pub ghost_unstage_active: bool,
    pub ghost_unstage_card_id: Option<u32>,
    pub ghost_unstage_cursor_screen: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ShopAuctionExtrasSnapshot {
    pub ui_mode: Option<String>,
    pub draft_initial: Option<DraftInitialPanelSnapshot>,
    pub shop: Option<ShopPanelSnapshot>,
    pub auction: Option<AuctionPanelSnapshot>,
    pub settlement: Option<SettlementPanelSnapshot>,
    pub bid_keyboard_focus: Option<String>,
    pub locally_passed: bool,
    pub toast: Option<ToastSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DraftInitialPanelSnapshot {
    pub offering_loaded: bool,
    pub ready_signalled: bool,
    pub objective_overlay_visible: bool,
    pub objective_overlay_dismissed: bool,
    pub objective_focus_target: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShopPanelSnapshot {
    pub slots_loaded: bool,
    pub ready_signalled: bool,
    pub refresh_count_this_draft: u32,
    pub refresh_in_flight: bool,
    pub footer_slots_loaded: bool,
    /// `None` represents an empty footer slot.
    pub footer_slots: Vec<Option<u32>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuctionPanelSnapshot {
    pub panel_state: String,
    pub card_id: Option<u32>,
    pub starting_price: u32,
    pub current_price: u32,
    pub current_leader: Option<String>,
    pub timer_duration_ms: u32,
    pub timer_remaining_ms: u32,
    pub preparing_elapsed_ms: u32,
    pub locally_expired_elapsed_ms: u32,
    pub in_flight_bid_amount: Option<u32>,
    pub pending_bid_accepted: bool,
    pub pending_gold_broadcast_seen: bool,
    pub opponent_bid_gate_satisfied: bool,
    pub waiting_for_local_gold_after_opponent_bid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettlementPanelSnapshot {
    pub outcome: Option<String>,
    pub winner: Option<String>,
    pub amount: u32,
    pub card_id: Option<u32>,
    pub elapsed_ms: u32,
    pub transition_duration_ms: u32,
    pub transition_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToastSnapshot {
    pub text: String,
    pub elapsed_ms: u32,
    pub active: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct HudExtrasSnapshot {
    pub mode: Option<String>,
    pub local_class: Option<String>,
    pub opponent_class: Option<String>,
    pub hud_player_ids: Option<HudPlayerIdsSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HudPlayerIdsSnapshot {
    pub local_id: String,
    pub opponent_id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BoardSnapshot {
    pub local_player_id: Option<String>,
    pub units: Vec<BoardUnitSnapshot>,
    pub units_truncated: bool,
    pub objectives: Vec<BoardObjectiveSnapshot>,
    pub objectives_truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoardUnitSnapshot {
    pub entity: String,
    pub unit_id: u64,
    pub owner_id: String,
    pub card_id: Option<u32>,
    pub frame_index: Option<usize>,
    pub used_missing_art_fallback: Option<bool>,
    pub hp_current: Option<u8>,
    pub hp_max: Option<u8>,
    pub atk: Option<u8>,
    pub mp: Option<u8>,
    pub ar: Option<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoardObjectiveSnapshot {
    pub entity: String,
    pub owner_id: String,
    pub lane: u8,
    pub hp_current: Option<u8>,
    pub hp_max: Option<u8>,
    pub art_kind: Option<String>,
    pub used_runtime_asset: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionSettingsSnapshot {
    pub placement_timer_multiplier_effective: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectiveIdentitySnapshot {
    pub lane: u8,
    pub is_fake: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpponentConnectionSnapshot {
    pub disconnected_player_id: Option<String>,
    pub grace_remaining_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionLifecycleSnapshot {
    pub cancellation_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OutboundIntentsSnapshot {
    pub hand_activate_cards: usize,
    pub hand_purchase_cards: usize,
    pub hand_submit_placements: usize,
    pub shop_purchase_cards: usize,
    pub shop_refresh_shops: usize,
    pub shop_ready_signals: usize,
    pub shop_place_bids: usize,
    pub shop_gold_counter_flash_requests: u32,
}

/// Bevy's `SystemParam` limit is 16 fields. The extras bag groups its
/// resources into nested SystemParams so [`write_qa_snapshot_system`] can
/// keep everything in a single SystemParam parameter while still respecting
/// the ceiling.
#[derive(SystemParam)]
pub struct ExtrasInputs<'w, 's> {
    pub frame_count: Option<Res<'w, bevy::diagnostic::FrameCount>>,
    pub identity: Option<Res<'w, ClientSessionIdentity>>,
    pub timers: ExtrasTimerInputs<'w>,
    pub hand: ExtrasHandInputs<'w>,
    pub shop_auction: ExtrasShopAuctionInputs<'w>,
    pub hud: ExtrasHudInputs<'w>,
    pub board: ExtrasBoardInputs<'w, 's>,
    pub session: ExtrasSessionInputs<'w>,
    pub economy: Option<Res<'w, PlayerEconomyView>>,
    pub local_gold: Option<Res<'w, ShopAuctionLocalGoldView>>,
    pub board_render_state: Option<Res<'w, BoardRenderState>>,
    pub objective_identities: Option<Res<'w, ClientObjectiveIdentities>>,
    pub hand_outbound: Option<Res<'w, HandUiOutboundMessages>>,
    pub shop_outbound: Option<Res<'w, ShopAuctionUiOutboundMessages>>,
}

#[derive(SystemParam)]
pub struct ExtrasTimerInputs<'w> {
    pub phase_timer: Option<Res<'w, PhaseTimerState>>,
    pub placement_timer: Option<Res<'w, PlacementTimer>>,
    pub auction_state: Option<Res<'w, ShopAuctionAuctionState>>,
    pub shop_timer: Option<Res<'w, ShopAuctionShopTimerState>>,
}

#[derive(SystemParam)]
pub struct ExtrasHandInputs<'w> {
    pub mode: Option<Res<'w, HandUiMode>>,
    pub disclosure: Option<Res<'w, PlacementDisclosureState>>,
    pub contents: Option<Res<'w, HandContents>>,
    pub pending_placements: Option<Res<'w, PendingPlacements>>,
    pub placement_drag: Option<Res<'w, ActivePlacementDrag>>,
    pub ghost_unstage_drag: Option<Res<'w, ActiveGhostUnstageDrag>>,
    pub placement_board_view: Option<Res<'w, PlacementBoardView>>,
}

#[derive(SystemParam)]
pub struct ExtrasShopAuctionInputs<'w> {
    pub ui_mode: Option<Res<'w, ShopAuctionUiMode>>,
    pub draft_initial: Option<Res<'w, ShopAuctionDraftInitialState>>,
    pub shop: Option<Res<'w, ShopAuctionShopState>>,
    pub auction: Option<Res<'w, ShopAuctionAuctionState>>,
    pub settlement: Option<Res<'w, ShopAuctionSettlementState>>,
    pub bid_focus: Option<Res<'w, AuctionBidKeyboardFocus>>,
    pub locally_passed: Option<Res<'w, AuctionLocallyPassed>>,
    pub toast: Option<Res<'w, ShopAuctionToastState>>,
}

#[derive(SystemParam)]
pub struct ExtrasHudInputs<'w> {
    pub mode: Option<Res<'w, HudMode>>,
    pub class_reveal: Option<Res<'w, HudClassReveal>>,
    pub player_ids: Option<Res<'w, HudPlayerIds>>,
}

#[derive(SystemParam)]
pub struct ExtrasBoardInputs<'w, 's> {
    pub local_player: Option<Res<'w, BoardLocalPlayer>>,
    pub units: Query<
        'w,
        's,
        (
            Entity,
            &'static BoardUnit,
            &'static BoardUnitOwner,
            Option<&'static BoardUnitCard>,
            Option<&'static BoardUnitStats>,
        ),
    >,
    pub objectives: Query<
        'w,
        's,
        (
            Entity,
            &'static StandingObjective,
            Option<&'static StandingObjectiveHp>,
            Option<&'static StandingObjectiveArt>,
        ),
    >,
}

#[derive(SystemParam)]
pub struct ExtrasSessionInputs<'w> {
    pub session_settings: Option<Res<'w, SessionSettingsView>>,
    pub opponent_connection: Option<Res<'w, OpponentConnectionView>>,
    pub session_lifecycle: Option<Res<'w, SessionLifecycleView>>,
}

impl<'w, 's> ExtrasInputs<'w, 's> {
    /// Collect every available extra into an [`ExtrasSnapshot`]. Missing
    /// resources produce `None`/empty fields without panicking; warnings
    /// are surfaced through the caller's `Vec<String>` collector via
    /// [`build_extras_snapshot`].
    pub fn snapshot(&self) -> ExtrasSnapshot {
        let mut warnings = Vec::new();
        self.snapshot_with_warnings(&mut warnings)
    }

    /// Same as [`Self::snapshot`] but appends "<resource> missing"
    /// diagnostics into `warnings` for missing top-level resources. The
    /// host system can fold these into [`QASnapshotData::warnings`].
    pub fn snapshot_with_warnings(&self, warnings: &mut Vec<String>) -> ExtrasSnapshot {
        let frame_count = self.frame_count.as_deref().map(|c| c.0 as u64);

        let players = build_player_ids_snapshot(
            self.identity.as_deref(),
            self.hand.placement_board_view.as_deref(),
            self.hud.player_ids.as_deref(),
        );

        let timers = build_timers_snapshot(&self.timers);

        let resources = self
            .economy
            .as_deref()
            .map(|economy| build_resources_snapshot(economy, self.local_gold.as_deref()));
        if resources.is_none() {
            warnings.push("extras: PlayerEconomyView resource missing".to_string());
        }

        let hand = build_hand_snapshot(&self.hand);
        let drag = build_drag_snapshot(&self.hand);
        let shop_auction = build_shop_auction_snapshot(&self.shop_auction);
        let hud = build_hud_snapshot(&self.hud);
        let board = build_board_snapshot(&self.board);

        let board_render_state = self
            .board_render_state
            .as_deref()
            .map(|state| format!("{:?}", state));

        let session_settings =
            self.session
                .session_settings
                .as_deref()
                .map(|s| SessionSettingsSnapshot {
                    placement_timer_multiplier_effective: format!(
                        "{:?}",
                        s.placement_timer_multiplier_effective
                    ),
                });

        let objective_identities = self
            .objective_identities
            .as_deref()
            .map(|res| {
                res.identities
                    .iter()
                    .map(|(lane, is_fake)| ObjectiveIdentitySnapshot {
                        lane: *lane,
                        is_fake: *is_fake,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let opponent_connection =
            self.session
                .opponent_connection
                .as_deref()
                .map(|view| match view.disconnected {
                    Some(ind) => OpponentConnectionSnapshot {
                        disconnected_player_id: Some(format!("{:?}", ind.player_id)),
                        grace_remaining_ms: Some(ind.grace_remaining_ms),
                    },
                    None => OpponentConnectionSnapshot {
                        disconnected_player_id: None,
                        grace_remaining_ms: None,
                    },
                });

        let session_lifecycle =
            self.session
                .session_lifecycle
                .as_deref()
                .map(|view| SessionLifecycleSnapshot {
                    cancellation_reason: view.cancellation.map(|r| format!("{:?}", r)),
                });

        let outbound_intents = build_outbound_intents_snapshot(
            self.hand_outbound.as_deref(),
            self.shop_outbound.as_deref(),
        );

        ExtrasSnapshot {
            frame_count,
            players,
            timers,
            resources,
            hand,
            drag,
            shop_auction,
            hud,
            board,
            board_render_state,
            session_settings,
            objective_identities,
            opponent_connection,
            session_lifecycle,
            outbound_intents,
        }
    }
}

fn build_player_ids_snapshot(
    identity: Option<&ClientSessionIdentity>,
    placement_board_view: Option<&PlacementBoardView>,
    hud_player_ids: Option<&HudPlayerIds>,
) -> PlayerIdsSnapshot {
    let local_player_id = identity
        .and_then(|i| i.player_id)
        .map(|p| format!("{:?}", p));
    // Opponent is not directly stored on `ClientSessionIdentity`; pull from
    // `PlacementBoardView` (placement-phase) or fall back to `HudPlayerIds`.
    let (opponent_player_id, opponent_source) = if let Some(view) = placement_board_view {
        (
            Some(format!("{:?}", view.opponent_player_id)),
            Some("placement_board_view".to_string()),
        )
    } else if let Some(ids) = hud_player_ids {
        (
            Some(format!("{:?}", ids.opponent_id)),
            Some("hud_player_ids".to_string()),
        )
    } else {
        (None, None)
    };

    PlayerIdsSnapshot {
        local_player_id,
        opponent_player_id,
        opponent_source,
    }
}

fn build_timers_snapshot(inputs: &ExtrasTimerInputs<'_>) -> TimersSnapshot {
    let phase_timer = inputs.phase_timer.as_deref().map(|t| {
        let remaining_ms = t.duration_ms.saturating_sub(t.elapsed_ms);
        PhaseTimerSnapshot {
            duration_ms: t.duration_ms,
            elapsed_ms: t.elapsed_ms,
            remaining_ms,
            active: t.active,
        }
    });
    let placement_timer = inputs
        .placement_timer
        .as_deref()
        .map(|t| PlacementTimerSnapshot {
            remaining_ms: t.remaining_ms,
            urgency_fired: t.urgency_fired,
            in_grace_window: t.in_grace_window,
            grace_remaining_ms: t.grace_remaining_ms,
            submitted: t.submitted,
        });
    let auction_timer = inputs
        .auction_state
        .as_deref()
        .map(|s| AuctionTimerSnapshot {
            panel_state: format!("{:?}", s.panel_state),
            duration_ms: s.timer_duration_ms,
            remaining_ms: s.timer_remaining_ms,
            preparing_elapsed_ms: s.preparing_elapsed_ms,
            locally_expired_elapsed_ms: s.locally_expired_elapsed_ms,
        });
    let shop_timer = inputs.shop_timer.as_deref().map(|s| ShopTimerSnapshot {
        duration_ms: s.duration_ms,
        remaining_ms: s.remaining_ms,
        started: s.started,
        deferred: s.deferred,
    });
    TimersSnapshot {
        phase_timer,
        placement_timer,
        auction_timer,
        shop_timer,
    }
}

fn build_resources_snapshot(
    economy: &PlayerEconomyView,
    local_gold: Option<&ShopAuctionLocalGoldView>,
) -> PlayerResourcesSnapshot {
    let last_update_source = economy.last_update_source.map(|src| match src {
        PlayerEconomyViewUpdateSource::GoldUpdate => "GoldUpdate".to_string(),
        PlayerEconomyViewUpdateSource::Snapshot => "Snapshot".to_string(),
    });
    let local_gold_view = local_gold.map(|g| {
        let free = g.free_gold(economy);
        LocalGoldViewSnapshot {
            initialized: g.initialized,
            gold: g.gold,
            reserved_gold: g.reserved_gold,
            free_gold: free,
        }
    });
    PlayerResourcesSnapshot {
        initialized: economy.initialized,
        last_update_source,
        gold: economy.gold,
        current_mana: economy.current_mana,
        reserve_mana: economy.reserve_mana,
        mana_cap: economy.mana_cap,
        local_gold_view,
    }
}

fn build_hand_snapshot(inputs: &ExtrasHandInputs<'_>) -> Option<HandSnapshot> {
    let mode = inputs.mode.as_deref().copied();
    let contents = inputs.contents.as_deref();
    let pending = inputs.pending_placements.as_deref();
    let disclosure = inputs.disclosure.as_deref();

    if mode.is_none() && contents.is_none() && pending.is_none() && disclosure.is_none() {
        return None;
    }

    let cards = contents
        .map(|c| {
            c.cards
                .iter()
                .map(|card_id| HandCardSnapshot { card_id: card_id.0 })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let hand_count = contents.map(|c| c.cards.len()).unwrap_or(0);
    let pending_placements = pending
        .map(|p| {
            p.placements
                .iter()
                .map(|placement| PendingPlacementSnapshot {
                    card_id: placement.card_id.0,
                    target: format!("{:?}", placement.target),
                    current_mana_spend: placement.current_mana_spend,
                    reserve_mana_spend: placement.reserve_mana_spend,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let staged_count = pending.map(|p| p.staged_count()).unwrap_or(0);
    let disclosure_step = disclosure.map(|d| disclosure_step_name(d.step));

    Some(HandSnapshot {
        mode: mode.map(|m| format!("{:?}", m)),
        disclosure_step,
        hand_count,
        cards,
        pending_placements,
        staged_count,
    })
}

fn disclosure_step_name(step: PlacementDisclosureStep) -> String {
    match step {
        PlacementDisclosureStep::Hidden => "Hidden".to_string(),
        PlacementDisclosureStep::CardSelection => "CardSelection".to_string(),
        PlacementDisclosureStep::TargetSelection { target_kind } => {
            format!("TargetSelection({})", target_kind_name(target_kind))
        }
        PlacementDisclosureStep::StagedCard => "StagedCard".to_string(),
        PlacementDisclosureStep::Correction { error } => format!("Correction({:?})", error),
        PlacementDisclosureStep::Submitted => "Submitted".to_string(),
    }
}

fn target_kind_name(kind: PlacementTargetKind) -> &'static str {
    match kind {
        PlacementTargetKind::Minion => "Minion",
        PlacementTargetKind::TargetObj => "TargetObj",
        PlacementTargetKind::LaneWide => "LaneWide",
        PlacementTargetKind::TargetUnit => "TargetUnit",
        PlacementTargetKind::Instant => "Instant",
    }
}

fn build_drag_snapshot(inputs: &ExtrasHandInputs<'_>) -> DragSnapshot {
    let placement_drag = inputs
        .placement_drag
        .as_deref()
        .copied()
        .unwrap_or_default();
    let ghost = inputs
        .ghost_unstage_drag
        .as_deref()
        .copied()
        .unwrap_or_default();

    DragSnapshot {
        placement_drag_active: placement_drag.card.is_some()
            && placement_drag.target_kind.is_some(),
        placement_drag_card_id: placement_drag.card_id.map(|c| c.0),
        placement_drag_card_entity: placement_drag.card.map(|e| format!("{:?}", e)),
        placement_drag_owner_id: placement_drag.owner_id.map(|p| format!("{:?}", p)),
        placement_drag_target_kind: placement_drag
            .target_kind
            .map(|k| target_kind_name(k).to_string()),
        placement_drag_cursor_world: placement_drag.cursor_world_position.map(|v| [v.x, v.y]),
        ghost_unstage_active: ghost.card_id.is_some(),
        ghost_unstage_card_id: ghost.card_id.map(|c| c.0),
        ghost_unstage_cursor_screen: ghost.cursor_screen_position.map(|v| [v.x, v.y]),
    }
}

fn auction_panel_state_name(state: ShopAuctionAuctionPanelState) -> &'static str {
    match state {
        ShopAuctionAuctionPanelState::Hidden => "Hidden",
        ShopAuctionAuctionPanelState::Preparing => "Preparing",
        ShopAuctionAuctionPanelState::Active => "Active",
        ShopAuctionAuctionPanelState::Settling => "Settling",
        ShopAuctionAuctionPanelState::ConnectionError => "ConnectionError",
    }
}

fn build_shop_auction_snapshot(
    inputs: &ExtrasShopAuctionInputs<'_>,
) -> Option<ShopAuctionExtrasSnapshot> {
    let ui_mode = inputs.ui_mode.as_deref().copied();
    let draft_initial_res = inputs.draft_initial.as_deref();
    let shop_res = inputs.shop.as_deref();
    let auction_res = inputs.auction.as_deref();
    let settlement_res = inputs.settlement.as_deref();
    let bid_focus = inputs.bid_focus.as_deref().copied();
    let locally_passed = inputs.locally_passed.as_deref().copied();
    let toast_res = inputs.toast.as_deref();

    if ui_mode.is_none()
        && draft_initial_res.is_none()
        && shop_res.is_none()
        && auction_res.is_none()
        && settlement_res.is_none()
        && bid_focus.is_none()
        && locally_passed.is_none()
        && toast_res.is_none()
    {
        return None;
    }

    let draft_initial = draft_initial_res.map(|d| DraftInitialPanelSnapshot {
        offering_loaded: d.offering_loaded,
        ready_signalled: d.ready_signalled,
        objective_overlay_visible: d.objective_overlay_visible,
        objective_overlay_dismissed: d.objective_overlay_dismissed,
        objective_focus_target: format!("{:?}", d.objective_focus_target),
    });

    let shop = shop_res.map(|s| {
        let footer_slots: Vec<Option<u32>> = s
            .footer_slots()
            .iter()
            .copied()
            .map(|slot| slot.map(|card_id| card_id.0))
            .collect();
        ShopPanelSnapshot {
            slots_loaded: s.slots_loaded,
            ready_signalled: s.ready_signalled,
            refresh_count_this_draft: s.refresh_count_this_draft,
            refresh_in_flight: s.refresh_in_flight,
            footer_slots_loaded: s.footer_slots_loaded,
            footer_slots,
        }
    });

    let auction = auction_res.map(|a| AuctionPanelSnapshot {
        panel_state: auction_panel_state_name(a.panel_state).to_string(),
        card_id: a.card_id.map(|c| c.0),
        starting_price: a.starting_price,
        current_price: a.current_price,
        current_leader: a.current_leader.map(|p| format!("{:?}", p)),
        timer_duration_ms: a.timer_duration_ms,
        timer_remaining_ms: a.timer_remaining_ms,
        preparing_elapsed_ms: a.preparing_elapsed_ms,
        locally_expired_elapsed_ms: a.locally_expired_elapsed_ms,
        in_flight_bid_amount: a.in_flight_bid_amount,
        pending_bid_accepted: a.pending_bid_accepted,
        pending_gold_broadcast_seen: a.pending_gold_broadcast_seen,
        opponent_bid_gate_satisfied: a.opponent_bid_gate_satisfied,
        waiting_for_local_gold_after_opponent_bid: a.waiting_for_local_gold_after_opponent_bid(),
    });

    let settlement = settlement_res.map(|s| SettlementPanelSnapshot {
        outcome: s.outcome.map(settlement_outcome_name).map(String::from),
        winner: s.winner.map(|p| format!("{:?}", p)),
        amount: s.amount,
        card_id: s.card_id.map(|c| c.0),
        elapsed_ms: s.elapsed_ms,
        transition_duration_ms: s.transition_duration_ms,
        transition_active: s.transition_active,
    });

    let bid_keyboard_focus = bid_focus
        .and_then(|f| f.focused_button)
        .map(|e| format!("{:?}", e));

    let locally_passed = locally_passed.map(|p| p.passed).unwrap_or(false);

    let toast = toast_res.map(|t| ToastSnapshot {
        text: t.text.clone(),
        elapsed_ms: t.elapsed_ms,
        active: t.active,
    });

    Some(ShopAuctionExtrasSnapshot {
        ui_mode: ui_mode.map(|m| format!("{:?}", m)),
        draft_initial,
        shop,
        auction,
        settlement,
        bid_keyboard_focus,
        locally_passed,
        toast,
    })
}

fn settlement_outcome_name(outcome: ShopAuctionSettlementOutcome) -> &'static str {
    match outcome {
        ShopAuctionSettlementOutcome::LocalWinner => "LocalWinner",
        ShopAuctionSettlementOutcome::OpponentWinner => "OpponentWinner",
        ShopAuctionSettlementOutcome::NoBid => "NoBid",
    }
}

fn build_hud_snapshot(inputs: &ExtrasHudInputs<'_>) -> Option<HudExtrasSnapshot> {
    let mode = inputs.mode.as_deref().copied();
    let reveal = inputs.class_reveal.as_deref().copied();
    let ids = inputs.player_ids.as_deref().copied();

    if mode.is_none() && reveal.is_none() && ids.is_none() {
        return None;
    }

    Some(HudExtrasSnapshot {
        mode: mode.map(|m| format!("{:?}", m)),
        local_class: reveal.and_then(|r| r.local).map(|c| format!("{:?}", c)),
        opponent_class: reveal.and_then(|r| r.opponent).map(|c| format!("{:?}", c)),
        hud_player_ids: ids.map(|ids| HudPlayerIdsSnapshot {
            local_id: format!("{:?}", ids.local_id),
            opponent_id: format!("{:?}", ids.opponent_id),
        }),
    })
}

fn objective_art_kind_name(kind: ObjectiveArtKind) -> &'static str {
    match kind {
        ObjectiveArtKind::Unknown => "Unknown",
        ObjectiveArtKind::Real => "Real",
        ObjectiveArtKind::Fake => "Fake",
    }
}

fn build_board_snapshot(inputs: &ExtrasBoardInputs<'_, '_>) -> BoardSnapshot {
    let local_player_id = inputs
        .local_player
        .as_deref()
        .and_then(|lp| lp.player_id)
        .map(|p| format!("{:?}", p));

    let mut units = Vec::new();
    let mut units_truncated = false;
    for (entity, board_unit, owner, card, stats) in inputs.units.iter() {
        if units.len() >= MAX_BOARD_ENTITIES_PER_KIND {
            units_truncated = true;
            break;
        }
        units.push(BoardUnitSnapshot {
            entity: format!("{:?}", entity),
            unit_id: board_unit.unit_id,
            owner_id: format!("{:?}", owner.0),
            card_id: card.and_then(|c| c.card_id).map(|c| c.0),
            frame_index: card.map(|c| c.frame_index),
            used_missing_art_fallback: card.map(|c| c.used_missing_art_fallback),
            hp_current: stats.map(|s| s.hp_current),
            hp_max: stats.map(|s| s.hp_max),
            atk: stats.map(|s| s.atk),
            mp: stats.map(|s| s.mp),
            ar: stats.map(|s| s.ar),
        });
    }

    let mut objectives = Vec::new();
    let mut objectives_truncated = false;
    for (entity, standing, hp, art) in inputs.objectives.iter() {
        if objectives.len() >= MAX_BOARD_ENTITIES_PER_KIND {
            objectives_truncated = true;
            break;
        }
        objectives.push(BoardObjectiveSnapshot {
            entity: format!("{:?}", entity),
            owner_id: format!("{:?}", standing.owner_id),
            lane: standing.lane,
            hp_current: hp.map(|h| h.hp_current),
            hp_max: hp.map(|h| h.hp_max),
            art_kind: art.map(|a| objective_art_kind_name(a.kind).to_string()),
            used_runtime_asset: art.map(|a| a.used_runtime_asset),
        });
    }

    BoardSnapshot {
        local_player_id,
        units,
        units_truncated,
        objectives,
        objectives_truncated,
    }
}

fn build_outbound_intents_snapshot(
    hand: Option<&HandUiOutboundMessages>,
    shop: Option<&ShopAuctionUiOutboundMessages>,
) -> OutboundIntentsSnapshot {
    let mut out = OutboundIntentsSnapshot::default();
    if let Some(h) = hand {
        out.hand_activate_cards = h.activate_cards.len();
        out.hand_purchase_cards = h.purchase_cards.len();
        out.hand_submit_placements = h.submit_placements.len();
    }
    if let Some(s) = shop {
        out.shop_purchase_cards = s.purchase_cards.len();
        out.shop_refresh_shops = s.refresh_shops.len();
        out.shop_ready_signals = s.ready_signals.len();
        out.shop_place_bids = s.place_bids.len();
        out.shop_gold_counter_flash_requests = s.gold_counter_flash_requests;
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────
// PROMPT 1186 — LayoutSnapshot + LayoutInputs
// S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 / PROMPT 1180 Lane D / Q-01..Q-10
// ─────────────────────────────────────────────────────────────────────────

/// PROMPT 1186 — layout-debug fields added to every QA snapshot. Covers the
/// Q-01..Q-10 items called out in
/// `reports/PROMPT-1180-global-ui-layout-system-deep-audit.md` §4: viewport
/// dimensions, per-surface root bounds (logical px), child counts, resolved
/// z-layer, content-overflow signal, button affordance states, and explicit
/// collision helpers for `placement_action_panel` and `shop_panel` vs
/// `hand_bar`. Sub-fields are `Option`/empty when the marker is not spawned
/// or when the data requires ECS support outside this module's owned write
/// scope; see [`Self::limitations`] for the documented gaps (Q-05 text fit,
/// Q-06 image aspect).
#[derive(Debug, Clone, Default, Serialize)]
pub struct LayoutSnapshot {
    /// Viewport size resolved from `PrimaryWindow` (Q-01). `None` when the
    /// window resource is absent (e.g. `MinimalPlugins` tests).
    pub viewport: ViewportLayoutSnapshot,
    /// Per-surface bounds + children_count + resolved z + overflow signal
    /// (Q-02..Q-04, Q-08). Entries are emitted in stable declaration order;
    /// surfaces whose marker is not currently spawned are still listed with
    /// `spawned: false` so the JSON shape is stable across phases.
    pub surfaces: Vec<SurfaceLayoutSnapshot>,
    /// Button affordance probes (Q-07). One entry per entity carrying
    /// [`bevy::ui::widget::Button`] with the current
    /// [`bevy::ui::Interaction`] state surfaced as a stable string token
    /// (`default` / `hover` / `pressed`). The `Name` component is surfaced
    /// when present so the probe can be correlated with spawn sites.
    pub button_affordances: Vec<ButtonAffordanceSnapshot>,
    /// Explicit collision helpers (Q-09, Q-10). `placement_action_panel`
    /// overlaps and the `shop_panel` bottom vs `hand_bar` top edge diff,
    /// computed in logical px from the same surface bounds reported above.
    pub collisions: LayoutCollisionsSnapshot,
    /// Documented limitations: fields whose computation requires ECS data
    /// outside this module's owned write scope. Stable strings, one per
    /// limitation, so audit tooling can grep for them without reading the
    /// surrounding doc comments.
    pub limitations: Vec<String>,
}

/// Q-01 — viewport dimensions.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ViewportLayoutSnapshot {
    pub width_px: Option<f32>,
    pub height_px: Option<f32>,
    /// Resolved render scale factor. `1.0` when no override is applied
    /// (default `UiScale`).
    pub ui_scale: Option<f32>,
    /// Window scale factor (DPI). Surfaced so audit tooling can disambiguate
    /// between UiScale tweaks and OS-level DPI scaling.
    pub window_scale_factor: Option<f32>,
}

/// Q-02..Q-04, Q-08 — per-surface bounds, child count, resolved z, overflow.
#[derive(Debug, Clone, Serialize)]
pub struct SurfaceLayoutSnapshot {
    /// Canonical surface name (matches the per-sub-surface visible counts in
    /// [`UiCounts`] — `hud_root`, `hand_bar`, `placement_action_panel`,
    /// `shop_panel`, etc.).
    pub name: String,
    /// `true` when the surface root marker has at least one matching entity
    /// in the world. Lets audit tooling distinguish "panel exists but
    /// hidden" from "panel was never spawned".
    pub spawned: bool,
    /// Mirrors the `*_visible` reading semantic (`Visibility != Hidden`).
    /// `None` when `spawned == false`.
    pub visible: Option<bool>,
    /// Q-02 — logical-pixel bounds, top-left origin. `None` when not spawned
    /// or when the layout system has not yet produced a `ComputedNode`.
    pub bounds: Option<SurfaceBoundsRect>,
    /// Q-04 — direct child count via the `Children` component. `None` when
    /// the marker is not spawned or the entity has no `Children` component
    /// (a leaf or root with no children yet).
    pub children_count: Option<usize>,
    /// Q-08 — resolved global z-layer. `None` when the marker entity has no
    /// `GlobalZIndex` component (inherits from the implicit stacking
    /// context).
    pub z_layer_resolved: Option<i32>,
    /// `ComputedNode::stack_index` — the per-tick resolved stacking order
    /// inside the UI tree. Useful for diagnosing same-z collisions on
    /// surfaces that share `GlobalZIndex` (RC-1).
    pub stack_index: Option<u32>,
    /// Q-03 — content-overflow signal: `true` when
    /// `ComputedNode::content_size` exceeds `ComputedNode::size` on either
    /// axis. This catches RC-2 ("computed children extend past parent's
    /// content_size") regardless of the parent's `Overflow` clip mode; the
    /// audit's "clipped" semantic is conservative — content that overflows
    /// is reported as `true` even when visible overflow is set, so the
    /// next audit can grep for true → investigate.
    pub overflow_clipped: Option<bool>,
}

/// Logical-pixel axis-aligned bounding rectangle, top-left origin.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SurfaceBoundsRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl SurfaceBoundsRect {
    /// Returns the right edge x-coordinate (`x + w`).
    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    /// Returns the bottom edge y-coordinate (`y + h`).
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    /// Returns `true` when this rect intersects `other` on both axes (open
    /// intervals — touching edges do not count as collision).
    #[inline]
    pub fn intersects(&self, other: &SurfaceBoundsRect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// Q-07 — button affordance probe.
#[derive(Debug, Clone, Serialize)]
pub struct ButtonAffordanceSnapshot {
    /// Stringified `Entity` so the probe can be cross-referenced with other
    /// snapshot fields (e.g. `auction.bid_keyboard_focus`).
    pub entity: String,
    /// `Name` component value when present; `None` when the entity was
    /// spawned without one.
    pub name: Option<String>,
    /// Current [`bevy::ui::Interaction`] state surfaced as a stable string
    /// token: `default` (`Interaction::None`), `hover`
    /// (`Interaction::Hovered`), or `pressed` (`Interaction::Pressed`). The
    /// audit's `disabled` variant is not computable from the standard
    /// `Interaction` enum (Bevy 0.18 has no built-in disabled state); see
    /// [`LayoutSnapshot::limitations`].
    pub interaction: String,
}

/// Q-09, Q-10 — explicit collision helpers.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LayoutCollisionsSnapshot {
    /// Q-09 — list of surface names whose bounds intersect with
    /// `placement_action_panel`'s bounds. Empty when the panel is not
    /// spawned/visible or when no other surface overlaps it.
    pub placement_action_panel_overlaps: Vec<String>,
    /// Q-10 — bottom edge y of the `shop_panel` surface (logical px).
    /// `None` when `shop_panel` is not spawned/visible.
    pub shop_panel_bottom_edge_y: Option<f32>,
    /// Q-10 — top edge y of the `hand_bar` surface (logical px). `None`
    /// when `hand_bar` is not spawned/visible.
    pub hand_bar_top_edge_y: Option<f32>,
    /// Q-10 — signed overlap in logical px between `shop_panel`'s bottom
    /// edge and `hand_bar`'s top edge. Positive value means
    /// `shop_panel.bottom_edge_y > hand_bar.top_edge_y` (panels collide on
    /// the y axis). Negative or zero means a gap or touching edges. `None`
    /// when either surface is missing.
    pub shop_panel_vs_hand_bar_overlap_px: Option<f32>,
}

/// Bundles every surface-root layout query into a single [`SystemParam`].
/// Nested sub-groups keep each `#[derive(SystemParam)]` struct under Bevy's
/// 16-field ceiling.
#[derive(SystemParam)]
pub struct LayoutInputs<'w, 's> {
    pub hud: LayoutHudQueries<'w, 's>,
    pub hand: LayoutHandQueries<'w, 's>,
    pub shop_auction: LayoutShopAuctionQueries<'w, 's>,
    pub misc: LayoutMiscQueries<'w, 's>,
    pub ui_scale: Option<Res<'w, UiScale>>,
    pub interactions: Query<
        'w,
        's,
        (Entity, &'static Interaction, Option<&'static Name>),
        With<bevy::ui::widget::Button>,
    >,
}

/// Shared component tuple read for every surface root: bounds, transform,
/// visibility, optional global z, optional child list.
type SurfaceTuple = (
    &'static ComputedNode,
    &'static UiGlobalTransform,
    &'static Visibility,
    Option<&'static GlobalZIndex>,
    Option<&'static Children>,
);

#[derive(SystemParam)]
pub struct LayoutHudQueries<'w, 's> {
    pub hud_root: Query<'w, 's, SurfaceTuple, With<crate::ui::hud::HudRoot>>,
    pub hud_top_strip: Query<'w, 's, SurfaceTuple, With<crate::ui::hud::HudTopStripRoot>>,
    pub hud_bottom_strip: Query<'w, 's, SurfaceTuple, With<crate::ui::hud::HudBottomStripRoot>>,
    pub hud_scoreboard_dot:
        Query<'w, 's, SurfaceTuple, With<crate::ui::hud::HudScoreboardDotRoot>>,
    pub hud_dim_overlay: Query<'w, 's, SurfaceTuple, With<crate::ui::hud::HudDimOverlayRoot>>,
}

#[derive(SystemParam)]
pub struct LayoutHandQueries<'w, 's> {
    pub hand_bar: Query<'w, 's, SurfaceTuple, With<crate::ui::hand::HandBarRoot>>,
    pub hand_fan: Query<'w, 's, SurfaceTuple, With<crate::ui::hand::HandFanRoot>>,
    pub hand_draft_grid_slot:
        Query<'w, 's, SurfaceTuple, With<crate::ui::hand::HandDraftGridSlotRoot>>,
    pub placement_action_panel:
        Query<'w, 's, SurfaceTuple, With<crate::ui::hand::PlacementActionPanelRoot>>,
}

#[derive(SystemParam)]
pub struct LayoutShopAuctionQueries<'w, 's> {
    pub panel_roots: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static Visibility,
            Option<&'static GlobalZIndex>,
            Option<&'static Children>,
            &'static crate::ui::shop_auction::ShopAuctionPanelRoot,
        ),
    >,
}

#[derive(SystemParam)]
pub struct LayoutMiscQueries<'w, 's> {
    pub lobby_root: Query<'w, 's, SurfaceTuple, With<crate::ui::lobby::LobbyRoot>>,
    pub connection_lost_overlay: Query<
        'w,
        's,
        SurfaceTuple,
        With<crate::presentation::connection_lost_overlay::ConnectionLostOverlayRoot>,
    >,
    pub result_screen:
        Query<'w, 's, SurfaceTuple, With<crate::presentation::result_screen::ResultScreenRoot>>,
    pub qa_snapshot_overlay: Query<'w, 's, SurfaceTuple, With<QASnapshotOverlayRoot>>,
}

/// Logical-px conversion: `ComputedNode::size` and
/// `UiGlobalTransform::translation()` are in physical px; multiply by the
/// node's `inverse_scale_factor` to recover logical/CSS px (matching the
/// bid-target harness pattern in `shop_auction_bid_target_focus_harness.rs`).
fn surface_bounds_logical(
    computed: &ComputedNode,
    transform: &UiGlobalTransform,
) -> SurfaceBoundsRect {
    let inv_scale = computed.inverse_scale_factor;
    let center = transform.transform_point2(Vec2::ZERO) * inv_scale;
    let w = computed.size.x * inv_scale;
    let h = computed.size.y * inv_scale;
    SurfaceBoundsRect {
        x: center.x - w * 0.5,
        y: center.y - h * 0.5,
        w,
        h,
    }
}

/// Q-03 — content-overflow signal. Returns `true` when content extends
/// past the node's box on either axis. Uses a small epsilon to swallow
/// sub-pixel layout noise.
fn surface_overflows_content(computed: &ComputedNode) -> bool {
    const EPS: f32 = 0.5;
    computed.content_size.x > computed.size.x + EPS
        || computed.content_size.y > computed.size.y + EPS
}

fn interaction_token(interaction: Interaction) -> &'static str {
    match interaction {
        Interaction::Pressed => "pressed",
        Interaction::Hovered => "hover",
        Interaction::None => "default",
    }
}

/// Best-effort projection of a `SurfaceTuple` into a `SurfaceLayoutSnapshot`.
/// `name` is the canonical surface name. Returns an entry with
/// `spawned: false` when the marker query is empty.
fn build_surface_snapshot_from_query<M: Component>(
    name: &str,
    query: &Query<SurfaceTuple, With<M>>,
) -> SurfaceLayoutSnapshot {
    let mut iter = query.iter();
    match iter.next() {
        Some((computed, transform, visibility, global_z, children)) => {
            SurfaceLayoutSnapshot {
                name: name.to_string(),
                spawned: true,
                visible: Some(is_visibility_visible(visibility)),
                bounds: Some(surface_bounds_logical(computed, transform)),
                children_count: Some(children.map(|c| c.len()).unwrap_or(0)),
                z_layer_resolved: global_z.map(|g| g.0),
                stack_index: Some(computed.stack_index),
                overflow_clipped: Some(surface_overflows_content(computed)),
            }
        }
        None => SurfaceLayoutSnapshot {
            name: name.to_string(),
            spawned: false,
            visible: None,
            bounds: None,
            children_count: None,
            z_layer_resolved: None,
            stack_index: None,
            overflow_clipped: None,
        },
    }
}

impl<'w, 's> LayoutInputs<'w, 's> {
    /// Collect every surface bound, button affordance, and explicit collision
    /// helper into a [`LayoutSnapshot`]. `window` is the optional
    /// `PrimaryWindow` already resolved by the host system — passing it in
    /// keeps the SystemParam shallow and lets unit tests stub the viewport
    /// directly.
    pub fn snapshot(&self, window: Option<&Window>) -> LayoutSnapshot {
        let viewport = ViewportLayoutSnapshot {
            width_px: window.map(|w| w.resolution.width()),
            height_px: window.map(|w| w.resolution.height()),
            window_scale_factor: window.map(|w| w.resolution.scale_factor()),
            ui_scale: self.ui_scale.as_deref().map(|s| s.0),
        };

        let mut surfaces: Vec<SurfaceLayoutSnapshot> = Vec::new();

        // HUD surfaces.
        surfaces.push(build_surface_snapshot_from_query::<crate::ui::hud::HudRoot>(
            "hud_root",
            &self.hud.hud_root,
        ));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hud::HudTopStripRoot,
        >("hud_top_strip", &self.hud.hud_top_strip));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hud::HudBottomStripRoot,
        >("hud_bottom_strip", &self.hud.hud_bottom_strip));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hud::HudScoreboardDotRoot,
        >("hud_scoreboard_dot", &self.hud.hud_scoreboard_dot));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hud::HudDimOverlayRoot,
        >("hud_dim_overlay", &self.hud.hud_dim_overlay));

        // Hand surfaces.
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hand::HandBarRoot,
        >("hand_bar", &self.hand.hand_bar));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hand::HandFanRoot,
        >("hand_fan", &self.hand.hand_fan));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hand::HandDraftGridSlotRoot,
        >("hand_draft_grid_slot", &self.hand.hand_draft_grid_slot));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::hand::PlacementActionPanelRoot,
        >(
            "placement_action_panel", &self.hand.placement_action_panel
        ));

        // Shop / auction surfaces — five canonical variants. The enum-keyed
        // query may produce more than one entry per variant if a panel ever
        // gets duplicated; we keep the first hit (matching the
        // `*_visible` count semantic in `UiCountQueries::snapshot`).
        use crate::ui::shop_auction::ShopAuctionPanelRoot;
        let mut shop_draft_offering: Option<SurfaceLayoutSnapshot> = None;
        let mut shop_panel: Option<SurfaceLayoutSnapshot> = None;
        let mut auction_panel: Option<SurfaceLayoutSnapshot> = None;
        let mut shop_footer: Option<SurfaceLayoutSnapshot> = None;
        let mut auction_toast: Option<SurfaceLayoutSnapshot> = None;
        let mut settlement_overlay: Option<SurfaceLayoutSnapshot> = None;
        for (computed, transform, visibility, global_z, children, variant) in
            &self.shop_auction.panel_roots
        {
            let snap = SurfaceLayoutSnapshot {
                name: shop_auction_variant_name(*variant).to_string(),
                spawned: true,
                visible: Some(is_visibility_visible(visibility)),
                bounds: Some(surface_bounds_logical(computed, transform)),
                children_count: Some(children.map(|c| c.len()).unwrap_or(0)),
                z_layer_resolved: global_z.map(|g| g.0),
                stack_index: Some(computed.stack_index),
                overflow_clipped: Some(surface_overflows_content(computed)),
            };
            match variant {
                ShopAuctionPanelRoot::DraftOffering if shop_draft_offering.is_none() => {
                    shop_draft_offering = Some(snap);
                }
                ShopAuctionPanelRoot::Shop if shop_panel.is_none() => shop_panel = Some(snap),
                ShopAuctionPanelRoot::Auction if auction_panel.is_none() => {
                    auction_panel = Some(snap);
                }
                ShopAuctionPanelRoot::ShopFooter if shop_footer.is_none() => {
                    shop_footer = Some(snap);
                }
                ShopAuctionPanelRoot::Toast if auction_toast.is_none() => {
                    auction_toast = Some(snap);
                }
                ShopAuctionPanelRoot::SettlementOverlay if settlement_overlay.is_none() => {
                    settlement_overlay = Some(snap);
                }
                _ => {}
            }
        }
        for (name, slot) in [
            ("shop_draft_offering", shop_draft_offering),
            ("shop_panel", shop_panel),
            ("auction_panel", auction_panel),
            ("shop_footer", shop_footer),
            ("auction_toast", auction_toast),
            ("settlement_overlay", settlement_overlay),
        ] {
            surfaces.push(slot.unwrap_or_else(|| SurfaceLayoutSnapshot {
                name: name.to_string(),
                spawned: false,
                visible: None,
                bounds: None,
                children_count: None,
                z_layer_resolved: None,
                stack_index: None,
                overflow_clipped: None,
            }));
        }

        // Misc surfaces (lobby + overlays).
        surfaces.push(build_surface_snapshot_from_query::<
            crate::ui::lobby::LobbyRoot,
        >("lobby_root", &self.misc.lobby_root));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::presentation::connection_lost_overlay::ConnectionLostOverlayRoot,
        >(
            "connection_lost_overlay",
            &self.misc.connection_lost_overlay,
        ));
        surfaces.push(build_surface_snapshot_from_query::<
            crate::presentation::result_screen::ResultScreenRoot,
        >("result_screen", &self.misc.result_screen));
        surfaces.push(build_surface_snapshot_from_query::<QASnapshotOverlayRoot>(
            "qa_snapshot_overlay",
            &self.misc.qa_snapshot_overlay,
        ));

        // Q-07 — button affordance probes for every entity carrying the
        // `Button` widget marker. Sorted by stringified entity id so the
        // ordering is stable across captures.
        let mut button_affordances: Vec<ButtonAffordanceSnapshot> = self
            .interactions
            .iter()
            .map(|(entity, interaction, name)| ButtonAffordanceSnapshot {
                entity: format!("{:?}", entity),
                name: name.map(|n| n.as_str().to_string()),
                interaction: interaction_token(*interaction).to_string(),
            })
            .collect();
        button_affordances.sort_by(|a, b| a.entity.cmp(&b.entity));

        // Q-09, Q-10 — explicit collision helpers computed from the bounds
        // collected above.
        let collisions = build_layout_collisions(&surfaces);

        // Documented limitations (Q-05 / Q-06 / Q-07 disabled).
        let limitations = vec![
            "Q-05 text.<marker>.fits / clipped_chars: not computable without \
             per-text-marker components; adding markers requires touching \
             client/src/ui/* (forbidden write scope for this story)."
                .to_string(),
            "Q-06 image.<marker>.aspect_ratio_src / aspect_ratio_rendered: \
             not computable without per-image-marker components and an \
             Assets<Image> read; adding markers requires touching \
             client/src/ui/* (forbidden write scope for this story)."
                .to_string(),
            "Q-07 button.<marker>.affordance_state.disabled: Bevy 0.18 \
             Interaction enum has no Disabled variant; only default / hover \
             / pressed are emitted."
                .to_string(),
        ];

        LayoutSnapshot {
            viewport,
            surfaces,
            button_affordances,
            collisions,
            limitations,
        }
    }
}

fn shop_auction_variant_name(variant: crate::ui::shop_auction::ShopAuctionPanelRoot) -> &'static str {
    use crate::ui::shop_auction::ShopAuctionPanelRoot;
    match variant {
        ShopAuctionPanelRoot::DraftOffering => "shop_draft_offering",
        ShopAuctionPanelRoot::Shop => "shop_panel",
        ShopAuctionPanelRoot::Auction => "auction_panel",
        ShopAuctionPanelRoot::ShopFooter => "shop_footer",
        ShopAuctionPanelRoot::Toast => "auction_toast",
        ShopAuctionPanelRoot::SettlementOverlay => "settlement_overlay",
    }
}

/// Build [`LayoutCollisionsSnapshot`] from the per-surface bounds collected
/// in [`LayoutInputs::snapshot`]. Exposed (`pub`) so integration tests can
/// exercise the collision math without spinning up a real Bevy world.
pub fn build_layout_collisions(surfaces: &[SurfaceLayoutSnapshot]) -> LayoutCollisionsSnapshot {
    let find_bounds = |name: &str| -> Option<SurfaceBoundsRect> {
        surfaces
            .iter()
            .find(|s| s.name == name && s.visible == Some(true))
            .and_then(|s| s.bounds)
    };

    let placement_panel_bounds = find_bounds("placement_action_panel");
    let placement_action_panel_overlaps = match placement_panel_bounds {
        Some(target) => surfaces
            .iter()
            .filter(|s| {
                s.name != "placement_action_panel"
                    && s.visible == Some(true)
                    && s.bounds
                        .as_ref()
                        .map(|b| b.intersects(&target))
                        .unwrap_or(false)
            })
            .map(|s| s.name.clone())
            .collect(),
        None => Vec::new(),
    };

    let shop_panel_bounds = find_bounds("shop_panel");
    let hand_bar_bounds = find_bounds("hand_bar");
    let shop_panel_bottom_edge_y = shop_panel_bounds.map(|b| b.bottom());
    let hand_bar_top_edge_y = hand_bar_bounds.map(|b| b.y);
    let shop_panel_vs_hand_bar_overlap_px = match (shop_panel_bounds, hand_bar_bounds) {
        (Some(shop), Some(hand)) => Some(shop.bottom() - hand.y),
        _ => None,
    };

    LayoutCollisionsSnapshot {
        placement_action_panel_overlaps,
        shop_panel_bottom_edge_y,
        hand_bar_top_edge_y,
        shop_panel_vs_hand_bar_overlap_px,
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
                    // S18-QA-SNAPSHOT-OVERLAY-EXCLUDE-ON-CAPTURE: derives
                    // overlay-root visibility from feedback state. Must
                    // run after every system that mutates feedback so the
                    // captured frame (later this tick, in render) sees
                    // the correct visibility.
                    apply_qa_snapshot_overlay_visibility_system,
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
            Interaction::None,
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
/// the trigger immediately. The actual overlay-root `Visibility` flip
/// (`Inherited → Hidden`) is performed by
/// [`apply_qa_snapshot_overlay_visibility_system`] later in the same chain
/// — splitting it out keeps this system's queries disjoint from
/// [`UiCountQueries`], which holds read-only `&Visibility` queries on the
/// per-sub-surface root markers and would otherwise conflict with a
/// `&mut Visibility` query here (Bevy B0001).
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
    extras_inputs: ExtrasInputs,
    layout_inputs: LayoutInputs,
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

        // Collect the PROMPT 1132 broader observability bag. Missing
        // resources are tolerated and surfaced as warnings on the snapshot.
        let mut extras_warnings = Vec::new();
        let extras = extras_inputs.snapshot_with_warnings(&mut extras_warnings);

        // PROMPT 1186 — collect Q-01..Q-10 layout fields. The layout
        // SystemParam reads `ComputedNode` / `UiGlobalTransform` /
        // `Visibility` / `Children` for every per-sub-surface root marker;
        // surfaces that have not yet spawned are emitted with
        // `spawned: false` so the JSON shape is phase-stable.
        let layout = layout_inputs.snapshot(window);

        let mut snapshot = build_snapshot_with_extras_and_layout(
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
            extras,
            layout,
        );
        snapshot.warnings.extend(extras_warnings);

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
///
/// The overlay-root `Visibility` is restored by
/// [`apply_qa_snapshot_overlay_visibility_system`] on the same chain tick
/// — once feedback leaves `Capturing`, the visibility system flips the
/// root back to `Inherited` so the post-capture chip is visible to the
/// human operator.
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

/// Mirrors [`QASnapshotFeedbackState`] onto the overlay root's
/// [`Visibility`] so the captured render frame omits the QA chrome
/// (S18-QA-SNAPSHOT-OVERLAY-EXCLUDE-ON-CAPTURE).
///
/// Single source of truth: while feedback is `Capturing` the root is
/// [`Visibility::Hidden`]; on every other state (`Idle` / `Saved` /
/// `Failed`) the root is restored to [`Visibility::Inherited`] (the
/// default spawned in [`spawn_qa_snapshot_overlay_system`], which
/// resolves to visible for a parentless UI root).
///
/// Splitting this out of [`write_qa_snapshot_system`] /
/// [`apply_qa_snapshot_capture_completed_system`] /
/// [`revert_qa_snapshot_feedback_state_system`] keeps those systems'
/// queries disjoint from [`UiCountQueries`] (which holds read-only
/// `&Visibility` queries on the per-sub-surface root markers and would
/// otherwise trigger a Bevy B0001 conflict against a `&mut Visibility`
/// query in the same system). The system runs in the same `Update` tick
/// as the trigger, so visibility propagation (`PostUpdate`) lands before
/// the render schedule that consumes the `Screenshot` entity — the
/// captured PNG sees the overlay as `Hidden`.
pub fn apply_qa_snapshot_overlay_visibility_system(
    feedback: Res<QASnapshotFeedbackState>,
    mut overlay_roots: Query<&mut Visibility, With<QASnapshotOverlayRoot>>,
) {
    let next = match *feedback {
        QASnapshotFeedbackState::Capturing { .. } => Visibility::Hidden,
        QASnapshotFeedbackState::Idle
        | QASnapshotFeedbackState::Saved { .. }
        | QASnapshotFeedbackState::Failed { .. } => Visibility::Inherited,
    };
    for mut vis in overlay_roots.iter_mut() {
        if *vis != next {
            *vis = next;
        }
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
///
/// Pre-1132 callers passing 9 arguments still resolve through this entry
/// point — the new PROMPT 1132 `extras` bag defaults to
/// [`ExtrasSnapshot::default`] (every sub-field `None` / empty). For richer
/// captures see [`build_snapshot_with_extras`].
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
    build_snapshot_with_extras(
        counter,
        unix_millis,
        screenshot,
        state,
        current_phase,
        phase_view,
        identity,
        window,
        ui_counts,
        ExtrasSnapshot::default(),
    )
}

/// Same as [`build_snapshot`] but accepts a fully-populated
/// [`ExtrasSnapshot`] (PROMPT 1132). The host system pre-collects extras
/// through [`ExtrasInputs::snapshot_with_warnings`] so this function is
/// pure / non-`SystemParam`-using and remains test-friendly. Pre-1186 callers
/// receive `LayoutSnapshot::default()` (every layout sub-field empty / `None`).
/// For PROMPT 1186 layout fields see [`build_snapshot_with_extras_and_layout`].
#[allow(clippy::too_many_arguments)]
pub fn build_snapshot_with_extras(
    counter: u64,
    unix_millis: u128,
    screenshot: ScreenshotInfo,
    state: Option<ClientState>,
    current_phase: Option<CurrentClientPhase>,
    phase_view: Option<&ClientPhaseView>,
    identity: Option<ClientSessionIdentity>,
    window: Option<&Window>,
    ui_counts: UiCounts,
    extras: ExtrasSnapshot,
) -> QASnapshotData {
    build_snapshot_with_extras_and_layout(
        counter,
        unix_millis,
        screenshot,
        state,
        current_phase,
        phase_view,
        identity,
        window,
        ui_counts,
        extras,
        LayoutSnapshot::default(),
    )
}

/// Same as [`build_snapshot_with_extras`] but also accepts a fully-populated
/// [`LayoutSnapshot`] (PROMPT 1186). The host system pre-collects layout
/// data through [`LayoutInputs::snapshot`].
#[allow(clippy::too_many_arguments)]
pub fn build_snapshot_with_extras_and_layout(
    counter: u64,
    unix_millis: u128,
    screenshot: ScreenshotInfo,
    state: Option<ClientState>,
    current_phase: Option<CurrentClientPhase>,
    phase_view: Option<&ClientPhaseView>,
    identity: Option<ClientSessionIdentity>,
    window: Option<&Window>,
    ui_counts: UiCounts,
    extras: ExtrasSnapshot,
    layout: LayoutSnapshot,
) -> QASnapshotData {
    let mut warnings: Vec<String> = Vec::new();

    let client_state = match state {
        Some(s) => format!("{:?}", s),
        None => {
            warnings.push("ClientState resource missing".to_string());
            "unknown".to_string()
        }
    };

    // PROMPT 1229 — `current_phase.timer_remaining_ms` is lifted from
    // `extras.timers.phase_timer.remaining_ms` (which itself is sourced
    // from `PhaseTimerState`). Surfaces as `null` when the timer resource
    // is absent (lobby / pre-handshake) or the extras bag is the default
    // empty value.
    let phase_timer_remaining_ms = extras.timers.phase_timer.as_ref().map(|t| t.remaining_ms);
    let current_phase_info = match current_phase {
        Some(p) => PhaseInfo {
            phase: Some(format!("{:?}", p.phase)),
            round: Some(p.round),
            timer_remaining_ms: phase_timer_remaining_ms,
        },
        None => {
            warnings.push("CurrentClientPhase resource missing".to_string());
            PhaseInfo {
                phase: None,
                round: None,
                timer_remaining_ms: phase_timer_remaining_ms,
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

    // PROMPT 1229 — derive the placement_state and auction_state top-level
    // lifts from the already-collected `extras` bag. Pure projection: no
    // additional resource reads, no UI mutation. `available` flags are
    // emitted explicitly so a phase-stable JSON shape never gets confused
    // with a missing-resource regression (B-1203-X-03).
    let placement_state = build_placement_state_snapshot(&extras);
    let auction_state = build_auction_state_snapshot(&extras);

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
        extras,
        layout,
        placement_state,
        auction_state,
        warnings,
    }
}

/// PROMPT 1229 — build [`PlacementStateSnapshot`] from the already-collected
/// [`ExtrasSnapshot`]. Pure projection: walks `extras.hand` /
/// `extras.timers.placement_timer` / `extras.drag` and surfaces a stable
/// JSON shape. Exposed (`pub`) so unit tests can verify the projection
/// without spinning up a real Bevy app.
pub fn build_placement_state_snapshot(extras: &ExtrasSnapshot) -> PlacementStateSnapshot {
    let hand = extras.hand.as_ref();
    let placement_timer = extras.timers.placement_timer.as_ref();
    let drag = &extras.drag;

    let available = hand.is_some() || placement_timer.is_some();
    if !available {
        return PlacementStateSnapshot::default();
    }

    let staged_count = hand.map(|h| h.staged_count);
    let submitted = placement_timer.map(|t| t.submitted);
    // `can_submit` requires both the staged-count source and the submitted
    // flag — if either is missing we surface `None` rather than guessing.
    let can_submit = match (staged_count, submitted) {
        (Some(n), Some(s)) => Some(n > 0 && !s),
        _ => None,
    };

    PlacementStateSnapshot {
        available,
        staged_count,
        can_submit,
        submitted,
        drag_active: Some(drag.placement_drag_active),
        drag_card_id: drag.placement_drag_card_id,
        drag_target_kind: drag.placement_drag_target_kind.clone(),
        disclosure_step: hand.and_then(|h| h.disclosure_step.clone()),
    }
}

/// PROMPT 1229 — build [`AuctionStateSnapshot`] from the already-collected
/// [`ExtrasSnapshot`]. Pure projection: walks
/// `extras.shop_auction.auction` and `extras.resources.local_gold_view`
/// and surfaces a stable JSON shape. Exposed (`pub`) so unit tests can
/// verify the projection without spinning up a real Bevy app.
pub fn build_auction_state_snapshot(extras: &ExtrasSnapshot) -> AuctionStateSnapshot {
    let auction = extras
        .shop_auction
        .as_ref()
        .and_then(|s| s.auction.as_ref());
    let Some(a) = auction else {
        // Local gold may still be available even when no auction is
        // active (e.g. during shop phase), but surfacing it under
        // `auction_state` would be misleading. We keep the shape stable
        // (`available = false` + everything null) so audits can grep for
        // the explicit availability flag.
        return AuctionStateSnapshot::default();
    };

    let local_gold = extras
        .resources
        .as_ref()
        .and_then(|r| r.local_gold_view.as_ref())
        .map(|g| AuctionLocalGoldSnapshot {
            gold: g.gold,
            reserved_gold: g.reserved_gold,
            free_gold: g.free_gold,
            view_initialized: g.initialized,
        });

    AuctionStateSnapshot {
        available: true,
        panel_state: Some(a.panel_state.clone()),
        card_id: a.card_id,
        starting_price: Some(a.starting_price),
        current_price: Some(a.current_price),
        current_leader: a.current_leader.clone(),
        timer_duration_ms: Some(a.timer_duration_ms),
        timer_remaining_ms: Some(a.timer_remaining_ms),
        local_in_flight_bid_amount: a.in_flight_bid_amount,
        local_gold,
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
