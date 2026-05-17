//! Integration tests for the QA snapshot overlay (PROMPT 1013 / PROMPT 1019).
//!
//! Asserts:
//!  - The overlay is hidden / absent when [`QASnapshotConfig::enabled`] is
//!    `false` — the default — so production UI is unaffected.
//!  - The overlay spawns when `enabled` is `true`, on the `DEBUG` z-layer.
//!  - The full system path (button-click message -> file write) produces
//!    a snapshot JSON on disk with a populated `screenshot` block.
//!  - `build_snapshot` populates every documented field and records
//!    `warnings` instead of panicking when source resources are missing.
//!  - `QASnapshotConfig::from_env_values` parses the documented activation
//!    rule deterministically without touching the process environment.
//!  - The `F9` keyboard shortcut routes through the same `QASnapshotRequested`
//!    channel as the button click.
//!  - The feedback state machine flips to `Capturing` on trigger, `Saved`
//!    when the capture observer reports back, and reverts to `Idle` after
//!    the wall-clock timeout.
//!  - The capture-completed system rewrites `snapshot.json` to mark the
//!    screenshot `captured` or `failed` depending on whether the PNG was
//!    written.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::input::InputPlugin;
use bevy::prelude::*;
use client::presentation::qa_snapshot::{
    apply_qa_snapshot_capture_completed_system, build_snapshot,
    revert_qa_snapshot_feedback_state_system, update_snapshot_json_status, write_snapshot_to_dir,
    QASnapshotButton, QASnapshotCaptureCompleted, QASnapshotConfig, QASnapshotCounter,
    QASnapshotData, QASnapshotFeedbackState, QASnapshotOverlayEntities, QASnapshotOverlayRoot,
    QASnapshotPlugin, QASnapshotRequested, ScreenshotInfo, UiCounts, DEFAULT_QA_SNAPSHOT_DIR,
    QA_CAPTURE_TIMEOUT_SECS, QA_FEEDBACK_REVERT_SECS, QA_SCREENSHOT_FILENAME, QA_SCREENSHOT_FORMAT,
    QA_SNAPSHOT_SHORTCUT_KEY, SCREENSHOT_STATUS_CAPTURED, SCREENSHOT_STATUS_FAILED,
    SCREENSHOT_STATUS_PENDING,
};
use client::state::{ClientPhaseView, ClientSessionIdentity, ClientState, CurrentClientPhase};
use client::ui::design_tokens::z_layers;
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn unique_tmp_dir(label: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ccgs-qa-snapshot-{label}-{nanos}-{n}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn placeholder_screenshot(requested_at_ms: u128) -> ScreenshotInfo {
    ScreenshotInfo {
        relative_path: QA_SCREENSHOT_FILENAME.to_string(),
        absolute_path: format!("/abs/{QA_SCREENSHOT_FILENAME}"),
        format: QA_SCREENSHOT_FORMAT.to_string(),
        requested_at_ms,
        status: SCREENSHOT_STATUS_PENDING.to_string(),
        captured_at_ms: None,
        error: None,
    }
}

#[test]
fn overlay_is_absent_by_default() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    // Pre-insert a default (disabled) config so the plugin does not read
    // the process environment — tests must be deterministic regardless of
    // CCGS_QA_SNAPSHOT.
    app.insert_resource(QASnapshotConfig::default());
    app.add_plugins(MinimalPlugins);
    app.add_plugins(QASnapshotPlugin);
    app.update();

    let mut overlay_query = app.world_mut().query::<&QASnapshotOverlayRoot>();
    let overlay_count = overlay_query.iter(app.world()).count();
    assert_eq!(
        overlay_count, 0,
        "overlay root must NOT spawn when QASnapshotConfig::enabled is false"
    );
    assert!(
        app.world()
            .get_resource::<QASnapshotOverlayEntities>()
            .is_none(),
        "QASnapshotOverlayEntities resource must be absent when disabled"
    );
    assert_eq!(
        app.world().resource::<QASnapshotConfig>().output_dir,
        PathBuf::from(DEFAULT_QA_SNAPSHOT_DIR),
        "default output_dir must match the documented constant"
    );
}

#[test]
fn overlay_spawns_when_enabled() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    let tmp = unique_tmp_dir("enabled");
    app.insert_resource(QASnapshotConfig {
        enabled: true,
        output_dir: tmp,
    });
    app.add_plugins(MinimalPlugins);
    app.add_plugins(QASnapshotPlugin);
    app.update();

    let entities = app
        .world()
        .get_resource::<QASnapshotOverlayEntities>()
        .copied()
        .expect("QASnapshotOverlayEntities must be inserted when enabled");

    assert!(
        app.world()
            .entity(entities.root)
            .contains::<QASnapshotOverlayRoot>(),
        "root entity must carry the QASnapshotOverlayRoot marker"
    );
    assert!(
        app.world()
            .entity(entities.button)
            .contains::<QASnapshotButton>(),
        "button entity must carry the QASnapshotButton marker"
    );

    let z = app
        .world()
        .entity(entities.root)
        .get::<GlobalZIndex>()
        .copied()
        .expect("overlay root must carry a GlobalZIndex");
    assert_eq!(
        z,
        z_layers::DEBUG,
        "overlay must paint on the DEBUG z-layer so it sits above production UI"
    );

    // Feedback state is initialised to Idle so the button reads "Snapshot".
    let feedback = app.world().resource::<QASnapshotFeedbackState>();
    match feedback {
        QASnapshotFeedbackState::Idle => {}
        other => panic!("feedback state must default to Idle, got {other:?}"),
    }
}

#[test]
fn snapshot_request_writes_json_with_screenshot_block() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    let tmp = unique_tmp_dir("request");
    app.insert_resource(QASnapshotConfig {
        enabled: true,
        output_dir: tmp.clone(),
    });
    // Seed the resources the snapshot looks at so the JSON has populated
    // fields (not just warnings). MinimalPlugins must come before
    // init_state so `StatesPlugin` is registered.
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(CurrentClientPhase {
        phase: RoundPhase::Placement,
        round: 2,
    });
    app.insert_resource(ClientPhaseView {
        phase: RoundPhase::Placement,
        round_number: 2,
        timer_duration_ms: 45_000,
    });
    app.insert_resource(ClientSessionIdentity::default());
    app.add_plugins(QASnapshotPlugin);
    // First update spawns the overlay and lets the plugin register.
    app.update();

    // Emit a snapshot request through the public message channel — this
    // is the same channel both the button click and F9 shortcut write into.
    app.world_mut()
        .resource_mut::<Messages<QASnapshotRequested>>()
        .write(QASnapshotRequested);
    app.update();

    let entries: Vec<_> = fs::read_dir(&tmp)
        .expect("output_dir must exist after a snapshot write")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "exactly one snapshot subdirectory must be produced per request, got {}",
        entries.len()
    );
    let snapshot_subdir = entries[0].path();
    let json_path = snapshot_subdir.join("snapshot.json");
    assert!(
        json_path.is_file(),
        "snapshot.json must be written under {}",
        snapshot_subdir.display()
    );
    let readme_path = snapshot_subdir.join("README.md");
    assert!(
        readme_path.is_file(),
        "README.md must be written next to snapshot.json"
    );
    let readme = fs::read_to_string(&readme_path).unwrap();
    assert!(
        readme.contains("Screenshot::primary_window"),
        "README must document the Bevy capture API in use"
    );
    assert!(
        readme.contains("F9"),
        "README must mention the F9 keyboard shortcut"
    );

    let json = fs::read_to_string(&json_path).expect("snapshot.json must be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("snapshot.json must be valid JSON");

    let screenshot = parsed
        .get("screenshot")
        .expect("snapshot.json must include a `screenshot` block");
    assert_eq!(
        screenshot["relative_path"], QA_SCREENSHOT_FILENAME,
        "screenshot.relative_path must match the canonical filename"
    );
    assert!(
        screenshot["absolute_path"].is_string(),
        "screenshot.absolute_path must be a string"
    );
    assert!(
        screenshot["absolute_path"]
            .as_str()
            .unwrap()
            .ends_with(QA_SCREENSHOT_FILENAME),
        "screenshot.absolute_path must end with the canonical filename"
    );
    assert_eq!(
        screenshot["format"], QA_SCREENSHOT_FORMAT,
        "screenshot.format must surface the encoded format"
    );
    assert!(
        screenshot["requested_at_ms"].is_number(),
        "screenshot.requested_at_ms must be numeric"
    );
    assert_eq!(
        screenshot["status"], SCREENSHOT_STATUS_PENDING,
        "screenshot.status starts at pending until the render world reports back"
    );
    assert!(
        screenshot["captured_at_ms"].is_null(),
        "screenshot.captured_at_ms must be null while pending"
    );
    assert!(
        screenshot["error"].is_null(),
        "screenshot.error must be null while pending"
    );

    assert!(
        parsed["client_state"].is_string(),
        "client_state must be serialised as a string"
    );
    assert!(
        parsed["current_phase"]["phase"].is_string(),
        "current_phase.phase must be populated when CurrentClientPhase is present"
    );
    assert_eq!(
        parsed["phase_view"]["timer_duration_ms"], 45_000,
        "phase_view must carry the timer_duration_ms field from ClientPhaseView"
    );
    assert!(
        parsed["ui_counts"]["hud_entities"].is_number(),
        "ui_counts.hud_entities must be a numeric field"
    );

    // Feedback state must have advanced to Capturing immediately so the
    // operator sees the button change without waiting for the render world.
    let feedback = app.world().resource::<QASnapshotFeedbackState>().clone();
    match feedback {
        QASnapshotFeedbackState::Capturing { .. } => {}
        other => panic!("feedback state must be Capturing after request, got {other:?}"),
    }
}

#[test]
fn snapshot_request_is_inert_when_disabled() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    let tmp = unique_tmp_dir("inert");
    // enabled: false should be a no-op even if a request is issued — the
    // request channel is registered (so message::write does not panic) but
    // no file is written and no overlay is spawned.
    app.insert_resource(QASnapshotConfig {
        enabled: false,
        output_dir: tmp.clone(),
    });
    app.add_plugins(MinimalPlugins);
    app.add_plugins(QASnapshotPlugin);
    app.update();

    app.world_mut()
        .resource_mut::<Messages<QASnapshotRequested>>()
        .write(QASnapshotRequested);
    app.update();

    assert!(
        !tmp.exists() || fs::read_dir(&tmp).map(|d| d.count()).unwrap_or(0) == 0,
        "no snapshot must be written when QASnapshotConfig::enabled is false (got {:?})",
        tmp
    );
}

#[test]
fn f9_shortcut_writes_snapshot_request_through_same_channel() {
    use bevy::ecs::system::RunSystemOnce;
    use client::presentation::qa_snapshot::{
        qa_snapshot_keyboard_shortcut_system, write_qa_snapshot_system,
    };

    test_helpers::init_test_tracing();
    let mut app = App::new();
    let tmp = unique_tmp_dir("shortcut");
    app.insert_resource(QASnapshotConfig {
        enabled: true,
        output_dir: tmp.clone(),
    });
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(CurrentClientPhase {
        phase: RoundPhase::Placement,
        round: 1,
    });
    app.insert_resource(ClientPhaseView::default());
    app.insert_resource(ClientSessionIdentity::default());
    app.add_plugins(QASnapshotPlugin);
    // First update lets the plugin register its systems and run startup.
    app.update();

    // Stage F9 as just-pressed. Driving the system directly via
    // `run_system_once` bypasses the InputPlugin frame-start `clear()` that
    // wipes `just_pressed` at the top of every PreUpdate — under
    // MinimalPlugins + InputPlugin we cannot synthesise a real
    // `KeyboardInput` event without a Window entity, so we exercise the
    // shortcut system in isolation.
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(QA_SNAPSHOT_SHORTCUT_KEY);
    }
    app.world_mut()
        .run_system_once(qa_snapshot_keyboard_shortcut_system)
        .expect("shortcut system must run");
    app.world_mut()
        .run_system_once(write_qa_snapshot_system)
        .expect("write system must run");

    let entries: Vec<_> = fs::read_dir(&tmp)
        .expect("output_dir must exist after the shortcut fires")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "F9 must route through the same snapshot channel as the button click"
    );
    let subdir = entries[0].path();
    assert!(subdir.join("snapshot.json").is_file());
    assert!(subdir.join("README.md").is_file());
}

#[test]
fn build_snapshot_includes_all_documented_fields_and_records_warnings() {
    // No state, no phase view, no identity, no window -> every source-
    // missing branch should record a warning instead of panicking.
    let snapshot = build_snapshot(
        7,
        1_700_000_000_000,
        placeholder_screenshot(1_700_000_000_000),
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
    );

    assert_eq!(snapshot.counter, 7);
    assert_eq!(snapshot.unix_millis, 1_700_000_000_000);
    assert_eq!(snapshot.screenshot.relative_path, QA_SCREENSHOT_FILENAME);
    assert_eq!(snapshot.screenshot.format, QA_SCREENSHOT_FORMAT);
    assert_eq!(snapshot.screenshot.status, SCREENSHOT_STATUS_PENDING);
    assert!(snapshot.screenshot.error.is_none());
    assert_eq!(snapshot.client_state, "unknown");
    assert!(snapshot.current_phase.phase.is_none());
    assert!(snapshot.phase_view.phase.is_none());
    assert!(snapshot.session_identity.player_id.is_none());
    assert!(snapshot.window.width.is_none());
    // Every missing resource must appear in the warnings list.
    for needle in [
        "ClientState resource missing",
        "CurrentClientPhase resource missing",
        "ClientPhaseView resource missing",
        "ClientSessionIdentity resource missing",
        "PrimaryWindow not found",
    ] {
        assert!(
            snapshot.warnings.iter().any(|w| w == needle),
            "expected warning {needle:?} in {:?}",
            snapshot.warnings
        );
    }
}

#[test]
fn build_snapshot_serialises_present_resources_without_warnings() {
    let phase_view = ClientPhaseView {
        phase: RoundPhase::DraftShop,
        round_number: 3,
        timer_duration_ms: 30_000,
    };
    let identity = ClientSessionIdentity::default();
    let snapshot = build_snapshot(
        0,
        0,
        placeholder_screenshot(0),
        Some(ClientState::InSession),
        Some(CurrentClientPhase {
            phase: RoundPhase::DraftShop,
            round: 3,
        }),
        Some(&phase_view),
        Some(identity),
        None,
        UiCounts {
            hud_entities: 23,
            hud_timer_bars: 1,
            hand_ui_entities: 10,
            shop_auction_entities: 0,
            lobby_root_entities: 0,
            result_screen_roots: 0,
            connection_lost_overlay_roots: 1,
            qa_snapshot_overlay_roots: 1,
        },
    );

    assert_eq!(snapshot.client_state, "InSession");
    assert_eq!(
        snapshot.current_phase.phase.as_deref(),
        Some("DraftShop"),
        "phase must be serialised as the Debug name"
    );
    assert_eq!(snapshot.phase_view.round_number, Some(3));
    assert_eq!(snapshot.ui_counts.hud_entities, 23);
    // The PrimaryWindow missing warning is expected here; no other warning should fire.
    assert!(
        snapshot
            .warnings
            .iter()
            .all(|w| w == "PrimaryWindow not found"),
        "unexpected warnings: {:?}",
        snapshot.warnings
    );
}

#[test]
fn write_snapshot_to_dir_creates_per_id_subdirectory() {
    let tmp = unique_tmp_dir("write");
    let snapshot = QASnapshotData {
        snapshot_id: "test-1".to_string(),
        counter: 1,
        unix_millis: 0,
        screenshot: placeholder_screenshot(0),
        client_state: "Lobby".to_string(),
        current_phase: client::presentation::qa_snapshot::PhaseInfo {
            phase: None,
            round: None,
        },
        phase_view: client::presentation::qa_snapshot::PhaseViewInfo {
            phase: None,
            round_number: None,
            timer_duration_ms: None,
        },
        session_identity: client::presentation::qa_snapshot::SessionIdentityInfo {
            player_id: None,
            session_id: None,
            has_session_token: false,
        },
        window: client::presentation::qa_snapshot::WindowInfo {
            width: None,
            height: None,
            scale_factor: None,
        },
        ui_counts: UiCounts::default(),
        warnings: vec![],
    };

    let json_path =
        write_snapshot_to_dir(&tmp, &snapshot).expect("write_snapshot_to_dir must succeed");
    assert!(json_path.is_file());
    assert!(json_path.starts_with(&tmp));
    assert!(json_path.ends_with("snapshot.json"));
    let readme = json_path.with_file_name("README.md");
    assert!(
        readme.is_file(),
        "README.md must be written alongside snapshot.json"
    );
    let readme_body = fs::read_to_string(&readme).unwrap();
    assert!(
        readme_body.contains("screenshot"),
        "README.md must mention the screenshot bundle"
    );
}

#[test]
fn update_snapshot_json_status_flips_to_captured_when_png_present() {
    let tmp = unique_tmp_dir("status-captured");
    let snapshot = QASnapshotData {
        snapshot_id: "status-1".to_string(),
        counter: 1,
        unix_millis: 1_700_000_000_000,
        screenshot: placeholder_screenshot(1_700_000_000_000),
        client_state: "Lobby".to_string(),
        current_phase: client::presentation::qa_snapshot::PhaseInfo {
            phase: None,
            round: None,
        },
        phase_view: client::presentation::qa_snapshot::PhaseViewInfo {
            phase: None,
            round_number: None,
            timer_duration_ms: None,
        },
        session_identity: client::presentation::qa_snapshot::SessionIdentityInfo {
            player_id: None,
            session_id: None,
            has_session_token: false,
        },
        window: client::presentation::qa_snapshot::WindowInfo {
            width: None,
            height: None,
            scale_factor: None,
        },
        ui_counts: UiCounts::default(),
        warnings: vec![],
    };
    let json_path = write_snapshot_to_dir(&tmp, &snapshot).unwrap();
    let png_path = json_path.with_file_name(QA_SCREENSHOT_FILENAME);
    fs::write(&png_path, b"\x89PNG-fake").expect("write fake png");

    update_snapshot_json_status(&json_path, &png_path, 1_700_000_001_000, true)
        .expect("update must succeed");

    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert_eq!(
        parsed["screenshot"]["status"], SCREENSHOT_STATUS_CAPTURED,
        "status must flip to captured once the PNG is on disk"
    );
    assert_eq!(
        parsed["screenshot"]["captured_at_ms"], 1_700_000_001_000_u64,
        "captured_at_ms must be filled in"
    );
    assert!(
        parsed["screenshot"]["error"].is_null(),
        "error must be null on success"
    );
}

#[test]
fn update_snapshot_json_status_marks_failed_when_png_missing() {
    let tmp = unique_tmp_dir("status-failed");
    let snapshot = QASnapshotData {
        snapshot_id: "status-2".to_string(),
        counter: 2,
        unix_millis: 1_700_000_000_000,
        screenshot: placeholder_screenshot(1_700_000_000_000),
        client_state: "Lobby".to_string(),
        current_phase: client::presentation::qa_snapshot::PhaseInfo {
            phase: None,
            round: None,
        },
        phase_view: client::presentation::qa_snapshot::PhaseViewInfo {
            phase: None,
            round_number: None,
            timer_duration_ms: None,
        },
        session_identity: client::presentation::qa_snapshot::SessionIdentityInfo {
            player_id: None,
            session_id: None,
            has_session_token: false,
        },
        window: client::presentation::qa_snapshot::WindowInfo {
            width: None,
            height: None,
            scale_factor: None,
        },
        ui_counts: UiCounts::default(),
        warnings: vec![],
    };
    let json_path = write_snapshot_to_dir(&tmp, &snapshot).unwrap();
    let png_path = json_path.with_file_name(QA_SCREENSHOT_FILENAME);
    // Note: png file NOT created.

    update_snapshot_json_status(&json_path, &png_path, 1_700_000_001_000, false)
        .expect("update must succeed even when capture failed");

    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert_eq!(
        parsed["screenshot"]["status"], SCREENSHOT_STATUS_FAILED,
        "status must mark as failed when png is missing"
    );
    assert!(
        parsed["screenshot"]["error"].is_string(),
        "error field must carry a human-readable reason on failure"
    );
}

#[test]
fn capture_completed_message_updates_feedback_to_saved() {
    test_helpers::init_test_tracing();
    let tmp = unique_tmp_dir("capture-completed");
    let snapshot = QASnapshotData {
        snapshot_id: "ack-1".to_string(),
        counter: 1,
        unix_millis: 1_700_000_000_000,
        screenshot: placeholder_screenshot(1_700_000_000_000),
        client_state: "Lobby".to_string(),
        current_phase: client::presentation::qa_snapshot::PhaseInfo {
            phase: None,
            round: None,
        },
        phase_view: client::presentation::qa_snapshot::PhaseViewInfo {
            phase: None,
            round_number: None,
            timer_duration_ms: None,
        },
        session_identity: client::presentation::qa_snapshot::SessionIdentityInfo {
            player_id: None,
            session_id: None,
            has_session_token: false,
        },
        window: client::presentation::qa_snapshot::WindowInfo {
            width: None,
            height: None,
            scale_factor: None,
        },
        ui_counts: UiCounts::default(),
        warnings: vec![],
    };
    let json_path = write_snapshot_to_dir(&tmp, &snapshot).unwrap();
    let png_path = json_path.with_file_name(QA_SCREENSHOT_FILENAME);
    fs::write(&png_path, b"\x89PNG-fake").unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<QASnapshotFeedbackState>();
    app.add_message::<QASnapshotCaptureCompleted>();
    app.add_systems(Update, apply_qa_snapshot_capture_completed_system);

    // Seed feedback with Capturing so we can observe the transition.
    *app.world_mut().resource_mut::<QASnapshotFeedbackState>() =
        QASnapshotFeedbackState::Capturing {
            snapshot_id: "ack-1".to_string(),
            since_real_seconds: 0.0,
        };
    app.world_mut()
        .resource_mut::<Messages<QASnapshotCaptureCompleted>>()
        .write(QASnapshotCaptureCompleted {
            snapshot_id: "ack-1".to_string(),
            json_path: json_path.clone(),
            png_path: png_path.clone(),
            captured_at_ms: 1_700_000_002_000,
        });
    app.update();

    let feedback = app.world().resource::<QASnapshotFeedbackState>().clone();
    match feedback {
        QASnapshotFeedbackState::Saved { snapshot_id, .. } => {
            assert_eq!(snapshot_id, "ack-1");
        }
        other => panic!("feedback must advance to Saved after capture, got {other:?}"),
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert_eq!(parsed["screenshot"]["status"], SCREENSHOT_STATUS_CAPTURED);
}

#[test]
fn capture_completed_with_missing_png_demotes_to_failed() {
    test_helpers::init_test_tracing();
    let tmp = unique_tmp_dir("capture-failed");
    let snapshot = QASnapshotData {
        snapshot_id: "ack-2".to_string(),
        counter: 2,
        unix_millis: 1_700_000_000_000,
        screenshot: placeholder_screenshot(1_700_000_000_000),
        client_state: "Lobby".to_string(),
        current_phase: client::presentation::qa_snapshot::PhaseInfo {
            phase: None,
            round: None,
        },
        phase_view: client::presentation::qa_snapshot::PhaseViewInfo {
            phase: None,
            round_number: None,
            timer_duration_ms: None,
        },
        session_identity: client::presentation::qa_snapshot::SessionIdentityInfo {
            player_id: None,
            session_id: None,
            has_session_token: false,
        },
        window: client::presentation::qa_snapshot::WindowInfo {
            width: None,
            height: None,
            scale_factor: None,
        },
        ui_counts: UiCounts::default(),
        warnings: vec![],
    };
    let json_path = write_snapshot_to_dir(&tmp, &snapshot).unwrap();
    let png_path = json_path.with_file_name(QA_SCREENSHOT_FILENAME);
    // Intentionally do NOT write png.

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<QASnapshotFeedbackState>();
    app.add_message::<QASnapshotCaptureCompleted>();
    app.add_systems(Update, apply_qa_snapshot_capture_completed_system);

    app.world_mut()
        .resource_mut::<Messages<QASnapshotCaptureCompleted>>()
        .write(QASnapshotCaptureCompleted {
            snapshot_id: "ack-2".to_string(),
            json_path: json_path.clone(),
            png_path: png_path.clone(),
            captured_at_ms: 1_700_000_002_000,
        });
    app.update();

    let feedback = app.world().resource::<QASnapshotFeedbackState>().clone();
    match feedback {
        QASnapshotFeedbackState::Failed { reason, .. } => {
            assert!(
                reason.contains("png missing"),
                "failure reason must explain the missing png file: {reason:?}"
            );
        }
        other => panic!("feedback must advance to Failed when png missing, got {other:?}"),
    }
}

#[test]
fn feedback_state_capturing_times_out_to_failed() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<QASnapshotFeedbackState>();
    app.add_systems(Update, revert_qa_snapshot_feedback_state_system);
    app.update();

    *app.world_mut().resource_mut::<QASnapshotFeedbackState>() =
        QASnapshotFeedbackState::Capturing {
            snapshot_id: "timeout-1".to_string(),
            since_real_seconds: 0.0,
        };
    // Sleep past the timeout budget so Time<Real>::elapsed_secs() crosses
    // the threshold on the next update. Sleep is bounded so the test
    // remains fast even in CI.
    sleep(Duration::from_millis(
        ((QA_CAPTURE_TIMEOUT_SECS + 0.1) * 1_000.0) as u64,
    ));
    app.update();

    let feedback = app.world().resource::<QASnapshotFeedbackState>().clone();
    match feedback {
        QASnapshotFeedbackState::Failed { reason, .. } => {
            assert!(
                reason.contains("timeout"),
                "timeout reason must mention timeout, got {reason:?}"
            );
        }
        other => panic!("feedback must time out to Failed, got {other:?}"),
    }
}

#[test]
fn feedback_state_saved_reverts_to_idle_after_wall_clock_window() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<QASnapshotFeedbackState>();
    app.add_systems(Update, revert_qa_snapshot_feedback_state_system);
    app.update();

    *app.world_mut().resource_mut::<QASnapshotFeedbackState>() = QASnapshotFeedbackState::Saved {
        snapshot_id: "revert-1".to_string(),
        since_real_seconds: 0.0,
    };
    sleep(Duration::from_millis(
        ((QA_FEEDBACK_REVERT_SECS + 0.1) * 1_000.0) as u64,
    ));
    app.update();

    let feedback = app.world().resource::<QASnapshotFeedbackState>().clone();
    matches!(feedback, QASnapshotFeedbackState::Idle)
        .then_some(())
        .unwrap_or_else(|| panic!("feedback must revert to Idle, got {feedback:?}"));
}

#[test]
fn config_from_env_values_parses_documented_rule() {
    // Default: nothing set -> disabled, default dir.
    let cfg = QASnapshotConfig::from_env_values(None, None);
    assert!(!cfg.enabled);
    assert_eq!(cfg.output_dir, PathBuf::from(DEFAULT_QA_SNAPSHOT_DIR));

    // Any value other than literal "1" must NOT enable (avoids accidental
    // activation from `true`, `yes`, `0`, leading whitespace mistakes).
    for not_one in ["", "0", "true", "yes", "1.0", "01"] {
        let cfg = QASnapshotConfig::from_env_values(Some(not_one), None);
        assert!(
            !cfg.enabled,
            "value {not_one:?} must not enable QA snapshot overlay"
        );
    }

    // Exactly "1" enables, with or without surrounding whitespace.
    for one in ["1", " 1 "] {
        let cfg = QASnapshotConfig::from_env_values(Some(one), None);
        assert!(cfg.enabled, "value {one:?} must enable QA snapshot overlay");
    }

    // Output dir override is honoured.
    let cfg = QASnapshotConfig::from_env_values(Some("1"), Some("/tmp/elsewhere"));
    assert_eq!(cfg.output_dir, PathBuf::from("/tmp/elsewhere"));

    // Empty / whitespace-only dir override falls back to the default.
    for blank in ["", "   "] {
        let cfg = QASnapshotConfig::from_env_values(Some("1"), Some(blank));
        assert_eq!(
            cfg.output_dir,
            PathBuf::from(DEFAULT_QA_SNAPSHOT_DIR),
            "blank CCGS_QA_SNAPSHOT_DIR ({blank:?}) must fall back to the default"
        );
    }
}

#[test]
fn presentation_plugin_registers_qa_snapshot_plugin() {
    // The QASnapshotPlugin is bundled into PresentationPlugin. Asserting
    // its registration here means we catch regressions where the plugin
    // line is accidentally removed from `presentation/mod.rs`.
    // CARGO_MANIFEST_DIR is the `client/` crate root because that's where the
    // [[test]] entry is registered.
    let mod_rs = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("presentation")
        .join("mod.rs");
    let source = fs::read_to_string(&mod_rs).unwrap_or_else(|err| {
        panic!(
            "presentation/mod.rs must be readable at {}: {err}",
            mod_rs.display()
        )
    });
    assert!(
        source.contains("QASnapshotPlugin"),
        "PresentationPlugin must register QASnapshotPlugin"
    );
    assert!(
        source.contains("app.add_plugins(QASnapshotPlugin)"),
        "QASnapshotPlugin must be added via app.add_plugins"
    );
}

#[test]
fn counter_is_monotonic() {
    let counter = QASnapshotCounter::default();
    let a = counter.next();
    let b = counter.next();
    let c = counter.next();
    assert!(
        a < b && b < c,
        "counter must produce strictly increasing ids"
    );
}
