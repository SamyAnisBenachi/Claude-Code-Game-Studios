//! Integration tests for the PROMPT 1013 QA snapshot overlay.
//!
//! Asserts:
//!  - The overlay is hidden / absent when [`QASnapshotConfig::enabled`] is
//!    `false` — the default — so production UI is unaffected.
//!  - The overlay spawns when `enabled` is `true`, on the `DEBUG` z-layer.
//!  - The full system path (button-click message -> file write) produces
//!    a snapshot JSON on disk in a minimal Bevy app, without panicking.
//!  - `build_snapshot` populates every documented field and records
//!    `warnings` instead of panicking when source resources are missing.
//!  - `QASnapshotConfig::from_env_values` parses the documented activation
//!    rule deterministically without touching the process environment.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use client::presentation::qa_snapshot::{
    build_snapshot, write_snapshot_to_dir, QASnapshotButton, QASnapshotConfig, QASnapshotCounter,
    QASnapshotData, QASnapshotOverlayEntities, QASnapshotOverlayRoot, QASnapshotPlugin,
    QASnapshotRequested, UiCounts, DEFAULT_QA_SNAPSHOT_DIR, SCREENSHOT_STATUS_MANUAL,
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
}

#[test]
fn snapshot_request_writes_json_file_without_panic() {
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
    // is the same channel the button click writes into.
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

    let json = fs::read_to_string(&json_path).expect("snapshot.json must be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("snapshot.json must be valid JSON");
    assert_eq!(
        parsed["screenshot_status"], SCREENSHOT_STATUS_MANUAL,
        "screenshot_status must surface the manual-capture sentinel so operators know to pair with an OS screenshot"
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
fn build_snapshot_includes_all_documented_fields_and_records_warnings() {
    // No state, no phase view, no identity, no window -> every source-
    // missing branch should record a warning instead of panicking.
    let snapshot = build_snapshot(
        7,
        1_700_000_000_000,
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
    );

    assert_eq!(snapshot.counter, 7);
    assert_eq!(snapshot.unix_millis, 1_700_000_000_000);
    assert_eq!(snapshot.screenshot_status, SCREENSHOT_STATUS_MANUAL);
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
        screenshot_status: SCREENSHOT_STATUS_MANUAL.to_string(),
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
        "README.md must be written alongside snapshot.json to remind operators about manual screenshots"
    );
    let readme_body = fs::read_to_string(&readme).unwrap();
    assert!(
        readme_body.contains("screenshot"),
        "README.md must mention manual screenshot pairing"
    );
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
        "PresentationPlugin must register QASnapshotPlugin (PROMPT 1013)"
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
