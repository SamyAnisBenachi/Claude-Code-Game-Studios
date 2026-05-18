//! Integration tests for PROMPT 1186 / S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001.
//!
//! Locks the Q-01..Q-10 layout-debug fields enumerated in
//! `reports/PROMPT-1180-global-ui-layout-system-deep-audit.md` §4 against the
//! `client::presentation::qa_snapshot::LayoutSnapshot` shape:
//!
//! - Q-01 — `layout.viewport.{width_px,height_px,ui_scale,window_scale_factor}`
//! - Q-02 — `layout.surfaces[].bounds = {x, y, w, h}` per surface root marker
//! - Q-03 — `layout.surfaces[].overflow_clipped` (content > node size signal)
//! - Q-04 — `layout.surfaces[].children_count`
//! - Q-05 — limitation entry documenting non-computability
//! - Q-06 — limitation entry documenting non-computability
//! - Q-07 — `layout.button_affordances[].{entity,name,interaction}`
//! - Q-08 — `layout.surfaces[].z_layer_resolved` (+ `.stack_index`)
//! - Q-09 — `layout.collisions.placement_action_panel_overlaps`
//! - Q-10 — `layout.collisions.{shop_panel_bottom_edge_y,
//!         hand_bar_top_edge_y, shop_panel_vs_hand_bar_overlap_px}`
//!
//! The tests are deliberately schema-focused: they assert every key is present
//! in the JSON (null or value, never missing), exercise the
//! `build_layout_collisions` pure helper with synthetic surface bounds, and
//! lock the canonical surface-name set so renames downstream cause a single
//! deterministic failure rather than silent drift.

use std::path::PathBuf;

use bevy::prelude::*;
use client::presentation::qa_snapshot::{
    build_layout_collisions, build_snapshot, build_snapshot_with_extras_and_layout,
    write_snapshot_to_dir, ButtonAffordanceSnapshot, ExtrasSnapshot, LayoutCollisionsSnapshot,
    LayoutSnapshot, QASnapshotData, ScreenshotInfo, SurfaceBoundsRect, SurfaceLayoutSnapshot,
    UiCounts, ViewportLayoutSnapshot, QA_SCREENSHOT_FILENAME, QA_SCREENSHOT_FORMAT,
    SCREENSHOT_STATUS_PENDING,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ─────────────────────────────────────────────────────────────────────────
// Canonical surface-name set asserted by every test. The order matches the
// emission order in `LayoutInputs::snapshot` so audit tooling can iterate
// without re-sorting.
// ─────────────────────────────────────────────────────────────────────────

const EXPECTED_SURFACE_NAMES: &[&str] = &[
    "hud_root",
    "hud_top_strip",
    "hud_bottom_strip",
    "hud_scoreboard_dot",
    "hud_dim_overlay",
    "hand_bar",
    "hand_fan",
    "hand_draft_grid_slot",
    "placement_action_panel",
    "shop_draft_offering",
    "shop_panel",
    "auction_panel",
    "shop_footer",
    "auction_toast",
    "settlement_overlay",
    "lobby_root",
    "connection_lost_overlay",
    "result_screen",
    "qa_snapshot_overlay",
];

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

fn make_layout_with_bounds(
    placement_bounds: Option<SurfaceBoundsRect>,
    shop_bounds: Option<SurfaceBoundsRect>,
    hand_bar_bounds: Option<SurfaceBoundsRect>,
) -> LayoutSnapshot {
    let mut surfaces: Vec<SurfaceLayoutSnapshot> = EXPECTED_SURFACE_NAMES
        .iter()
        .map(|name| SurfaceLayoutSnapshot {
            name: (*name).to_string(),
            spawned: false,
            visible: None,
            bounds: None,
            children_count: None,
            z_layer_resolved: None,
            stack_index: None,
            overflow_clipped: None,
        })
        .collect();

    let mut set_surface = |name: &str, bounds: SurfaceBoundsRect| {
        for surface in surfaces.iter_mut() {
            if surface.name == name {
                surface.spawned = true;
                surface.visible = Some(true);
                surface.bounds = Some(bounds);
                surface.children_count = Some(0);
                surface.z_layer_resolved = Some(0);
                surface.stack_index = Some(0);
                surface.overflow_clipped = Some(false);
            }
        }
    };
    if let Some(b) = placement_bounds {
        set_surface("placement_action_panel", b);
    }
    if let Some(b) = shop_bounds {
        set_surface("shop_panel", b);
    }
    if let Some(b) = hand_bar_bounds {
        set_surface("hand_bar", b);
    }
    let collisions = build_layout_collisions(&surfaces);
    LayoutSnapshot {
        viewport: ViewportLayoutSnapshot::default(),
        surfaces,
        button_affordances: Vec::new(),
        collisions,
        limitations: Vec::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Q-01..Q-08 — every documented field is present in the JSON shape.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_build_snapshot_emits_default_layout_block_without_panic() {
    // Arrange — no window, no state, default UiCounts/Extras. Layout block
    // is expected to default through the legacy `build_snapshot` entry.
    let snapshot = build_snapshot(
        0,
        1_700_000_000_000,
        placeholder_screenshot(1_700_000_000_000),
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
    );

    // Act — serialise to JSON so we can grep for every documented field key.
    let json = serde_json::to_value(&snapshot).expect("snapshot serialises");

    // Assert — top-level `layout` object exists with every documented child
    // present (null/empty allowed; missing keys are not).
    let layout = json.get("layout").expect("layout block present");
    assert!(layout.is_object(), "layout must serialise as a JSON object");
    for key in [
        "viewport",
        "surfaces",
        "button_affordances",
        "collisions",
        "limitations",
    ] {
        assert!(
            layout.get(key).is_some(),
            "layout.{key} key must be present in JSON"
        );
    }
    let viewport = layout.get("viewport").unwrap();
    for key in [
        "width_px",
        "height_px",
        "ui_scale",
        "window_scale_factor",
    ] {
        assert!(
            viewport.get(key).is_some(),
            "layout.viewport.{key} key must be present"
        );
    }
    let collisions = layout.get("collisions").unwrap();
    for key in [
        "placement_action_panel_overlaps",
        "shop_panel_bottom_edge_y",
        "hand_bar_top_edge_y",
        "shop_panel_vs_hand_bar_overlap_px",
    ] {
        assert!(
            collisions.get(key).is_some(),
            "layout.collisions.{key} key must be present"
        );
    }
}

#[test]
fn test_build_snapshot_with_extras_and_layout_embeds_supplied_layout() {
    // Arrange — synthetic LayoutSnapshot with one viewport + one placement
    // overlap + explicit shop/hand edge values, exercising the same shape the
    // host system populates.
    let placement = SurfaceBoundsRect {
        x: 100.0,
        y: 200.0,
        w: 300.0,
        h: 50.0,
    };
    let shop = SurfaceBoundsRect {
        x: 150.0,
        y: 210.0,
        w: 200.0,
        h: 100.0,
    };
    let hand = SurfaceBoundsRect {
        x: 0.0,
        y: 280.0,
        w: 1280.0,
        h: 200.0,
    };
    let mut layout = make_layout_with_bounds(Some(placement), Some(shop), Some(hand));
    layout.viewport = ViewportLayoutSnapshot {
        width_px: Some(1280.0),
        height_px: Some(720.0),
        ui_scale: Some(1.0),
        window_scale_factor: Some(1.0),
    };
    layout.button_affordances.push(ButtonAffordanceSnapshot {
        entity: "Entity(42)".to_string(),
        name: Some("Snapshot button".to_string()),
        interaction: "default".to_string(),
    });
    layout
        .limitations
        .push("Q-05 text not computable in this story".to_string());

    // Act — pump the rich layout into the with_extras_and_layout entry.
    let snapshot = build_snapshot_with_extras_and_layout(
        0,
        0,
        placeholder_screenshot(0),
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
        ExtrasSnapshot::default(),
        layout,
    );

    // Assert — every supplied value lands on the snapshot field.
    assert_eq!(snapshot.layout.viewport.width_px, Some(1280.0));
    assert_eq!(snapshot.layout.viewport.height_px, Some(720.0));
    assert_eq!(snapshot.layout.surfaces.len(), EXPECTED_SURFACE_NAMES.len());
    assert_eq!(snapshot.layout.button_affordances.len(), 1);
    assert_eq!(
        snapshot.layout.button_affordances[0].name.as_deref(),
        Some("Snapshot button")
    );
    assert_eq!(snapshot.layout.button_affordances[0].interaction, "default");
    assert!(!snapshot.layout.limitations.is_empty());
}

#[test]
fn test_layout_surfaces_emits_canonical_set_in_stable_order() {
    // Arrange / Act — empty layout: every surface listed but unspawned.
    let layout = make_layout_with_bounds(None, None, None);
    let names: Vec<&str> = layout.surfaces.iter().map(|s| s.name.as_str()).collect();
    // Assert — exact match against the canonical set.
    assert_eq!(names, EXPECTED_SURFACE_NAMES);
    // Every unspawned surface emits Option fields as None so downstream
    // diff tooling sees nulls rather than zeros.
    for surface in &layout.surfaces {
        assert!(!surface.spawned);
        assert!(surface.bounds.is_none());
        assert!(surface.children_count.is_none());
        assert!(surface.z_layer_resolved.is_none());
        assert!(surface.stack_index.is_none());
        assert!(surface.overflow_clipped.is_none());
        assert!(surface.visible.is_none());
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Q-09 — placement_action_panel collisions vs sibling visible surfaces.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_collisions_emit_placement_action_panel_overlap_with_hand_bar_when_rects_intersect() {
    // Arrange — placement_action_panel spans the same y band as hand_bar.
    let placement = SurfaceBoundsRect {
        x: 100.0,
        y: 600.0,
        w: 300.0,
        h: 80.0,
    };
    let hand = SurfaceBoundsRect {
        x: 50.0,
        y: 620.0,
        w: 800.0,
        h: 100.0,
    };
    let layout = make_layout_with_bounds(Some(placement), None, Some(hand));

    // Assert — Q-09 reports hand_bar.
    assert!(
        layout
            .collisions
            .placement_action_panel_overlaps
            .iter()
            .any(|n| n == "hand_bar"),
        "expected hand_bar overlap in {:?}",
        layout.collisions.placement_action_panel_overlaps
    );
}

#[test]
fn test_collisions_omit_placement_panel_overlap_when_bounds_disjoint() {
    // Arrange — placement_action_panel and hand_bar separated vertically.
    let placement = SurfaceBoundsRect {
        x: 0.0,
        y: 0.0,
        w: 100.0,
        h: 50.0,
    };
    let hand = SurfaceBoundsRect {
        x: 0.0,
        y: 200.0,
        w: 100.0,
        h: 50.0,
    };
    let layout = make_layout_with_bounds(Some(placement), None, Some(hand));
    // Assert — no overlap reported.
    assert!(layout.collisions.placement_action_panel_overlaps.is_empty());
}

#[test]
fn test_collisions_empty_when_placement_panel_not_spawned() {
    let hand = SurfaceBoundsRect {
        x: 0.0,
        y: 200.0,
        w: 100.0,
        h: 50.0,
    };
    let layout = make_layout_with_bounds(None, None, Some(hand));
    assert!(layout.collisions.placement_action_panel_overlaps.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────
// Q-10 — shop_panel.bottom_edge_y vs hand_bar.top_edge_y.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_collisions_emit_positive_overlap_when_shop_panel_extends_below_hand_bar_top() {
    // Arrange — shop_panel bottom at y=310, hand_bar top at y=280 → overlap 30px.
    let shop = SurfaceBoundsRect {
        x: 0.0,
        y: 210.0,
        w: 200.0,
        h: 100.0,
    };
    let hand = SurfaceBoundsRect {
        x: 0.0,
        y: 280.0,
        w: 1280.0,
        h: 200.0,
    };
    let layout = make_layout_with_bounds(None, Some(shop), Some(hand));

    // Assert — explicit edge readings + signed overlap.
    assert_eq!(layout.collisions.shop_panel_bottom_edge_y, Some(310.0));
    assert_eq!(layout.collisions.hand_bar_top_edge_y, Some(280.0));
    assert_eq!(
        layout.collisions.shop_panel_vs_hand_bar_overlap_px,
        Some(30.0)
    );
}

#[test]
fn test_collisions_emit_negative_overlap_when_shop_panel_sits_above_hand_bar_top() {
    // Arrange — shop_panel bottom at y=200, hand_bar top at y=280 → gap -80px.
    let shop = SurfaceBoundsRect {
        x: 0.0,
        y: 100.0,
        w: 200.0,
        h: 100.0,
    };
    let hand = SurfaceBoundsRect {
        x: 0.0,
        y: 280.0,
        w: 200.0,
        h: 50.0,
    };
    let layout = make_layout_with_bounds(None, Some(shop), Some(hand));
    assert_eq!(layout.collisions.shop_panel_bottom_edge_y, Some(200.0));
    assert_eq!(layout.collisions.hand_bar_top_edge_y, Some(280.0));
    assert_eq!(
        layout.collisions.shop_panel_vs_hand_bar_overlap_px,
        Some(-80.0)
    );
}

#[test]
fn test_collisions_emit_none_overlap_when_either_surface_missing() {
    let hand = SurfaceBoundsRect {
        x: 0.0,
        y: 280.0,
        w: 200.0,
        h: 50.0,
    };
    let layout = make_layout_with_bounds(None, None, Some(hand));
    assert!(layout.collisions.shop_panel_bottom_edge_y.is_none());
    assert!(layout
        .collisions
        .shop_panel_vs_hand_bar_overlap_px
        .is_none());
    assert_eq!(layout.collisions.hand_bar_top_edge_y, Some(280.0));
}

// ─────────────────────────────────────────────────────────────────────────
// SurfaceBoundsRect intersection — guards the underlying AABB math.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_surface_bounds_rect_intersects_overlapping_rects() {
    let a = SurfaceBoundsRect {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    let b = SurfaceBoundsRect {
        x: 5.0,
        y: 5.0,
        w: 10.0,
        h: 10.0,
    };
    assert!(a.intersects(&b));
    assert!(b.intersects(&a));
}

#[test]
fn test_surface_bounds_rect_does_not_intersect_touching_edges() {
    // Touching edges should not count as a collision — the open-interval
    // semantic keeps adjacent panels from spuriously colliding.
    let a = SurfaceBoundsRect {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    let b = SurfaceBoundsRect {
        x: 10.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };
    assert!(!a.intersects(&b));
}

#[test]
fn test_surface_bounds_rect_edge_helpers_match_x_w_y_h() {
    let r = SurfaceBoundsRect {
        x: 5.0,
        y: 7.0,
        w: 100.0,
        h: 200.0,
    };
    assert_eq!(r.right(), 105.0);
    assert_eq!(r.bottom(), 207.0);
}

// ─────────────────────────────────────────────────────────────────────────
// QASnapshotData round-trip — serialise then deserialise so the layout
// block survives the JSON write/read cycle used by the relay tooling.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_snapshot_json_round_trip_preserves_layout_block_keys_after_write_to_dir() {
    // Arrange — snapshot whose layout block carries one value per Q-axis so
    // we can assert each key survives the writer round-trip.
    let tmp = std::env::temp_dir().join(format!(
        "ccgs-qa-snapshot-layout-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&tmp);

    let layout = LayoutSnapshot {
        viewport: ViewportLayoutSnapshot {
            width_px: Some(1280.0),
            height_px: Some(720.0),
            ui_scale: Some(1.0),
            window_scale_factor: Some(1.0),
        },
        surfaces: vec![SurfaceLayoutSnapshot {
            name: "hud_root".to_string(),
            spawned: true,
            visible: Some(true),
            bounds: Some(SurfaceBoundsRect {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 80.0,
            }),
            children_count: Some(4),
            z_layer_resolved: Some(300),
            stack_index: Some(12),
            overflow_clipped: Some(false),
        }],
        button_affordances: vec![ButtonAffordanceSnapshot {
            entity: "Entity(123)".to_string(),
            name: Some("Confirm".to_string()),
            interaction: "pressed".to_string(),
        }],
        collisions: LayoutCollisionsSnapshot {
            placement_action_panel_overlaps: vec!["hand_bar".to_string()],
            shop_panel_bottom_edge_y: Some(310.0),
            hand_bar_top_edge_y: Some(280.0),
            shop_panel_vs_hand_bar_overlap_px: Some(30.0),
        },
        limitations: vec!["Q-05 text not computable".to_string()],
    };

    let snapshot = QASnapshotData {
        snapshot_id: "test-layout".to_string(),
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
        extras: ExtrasSnapshot::default(),
        layout,
        warnings: vec![],
    };

    // Act — write then read.
    let json_path: PathBuf =
        write_snapshot_to_dir(&tmp, &snapshot).expect("write_snapshot_to_dir succeeds");
    let raw = std::fs::read_to_string(&json_path).expect("snapshot.json readable");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("snapshot.json parses");

    // Assert — every layout key survives. We pin individual values rather
    // than the whole object so adding new layout fields downstream doesn't
    // break this regression guard.
    let layout = value.get("layout").expect("layout key on serialised data");
    assert_eq!(layout["viewport"]["width_px"], 1280.0);
    assert_eq!(layout["viewport"]["height_px"], 720.0);
    assert_eq!(layout["surfaces"][0]["name"], "hud_root");
    assert_eq!(layout["surfaces"][0]["bounds"]["x"], 0.0);
    assert_eq!(layout["surfaces"][0]["bounds"]["w"], 1280.0);
    assert_eq!(layout["surfaces"][0]["children_count"], 4);
    assert_eq!(layout["surfaces"][0]["z_layer_resolved"], 300);
    assert_eq!(layout["surfaces"][0]["stack_index"], 12);
    assert_eq!(layout["surfaces"][0]["overflow_clipped"], false);
    assert_eq!(layout["button_affordances"][0]["name"], "Confirm");
    assert_eq!(layout["button_affordances"][0]["interaction"], "pressed");
    assert_eq!(
        layout["collisions"]["placement_action_panel_overlaps"][0],
        "hand_bar"
    );
    assert_eq!(layout["collisions"]["shop_panel_bottom_edge_y"], 310.0);
    assert_eq!(layout["collisions"]["shop_panel_vs_hand_bar_overlap_px"], 30.0);
    assert_eq!(layout["limitations"][0], "Q-05 text not computable");

    // Cleanup.
    let _ = std::fs::remove_dir_all(&tmp);
}

// ─────────────────────────────────────────────────────────────────────────
// QASnapshotPlugin smoke — write_qa_snapshot_system compiles and runs in a
// MinimalPlugins app with the layout SystemParam wired. We don't assert
// bounds (no UI plugin → no ComputedNode) — just that the layout block is
// emitted with the canonical surface set and the limitations entries.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_qa_snapshot_layout_block_emits_default_surfaces_and_limitations_under_minimal_plugins() {
    use client::presentation::qa_snapshot::{
        QASnapshotConfig, QASnapshotPlugin, QASnapshotRequested,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    test_helpers::init_test_tracing();

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = std::env::temp_dir().join(format!("ccgs-qa-snapshot-layout-plugin-{nanos}-{n}"));
    let _ = std::fs::remove_dir_all(&tmp);

    let mut app = App::new();
    app.insert_resource(QASnapshotConfig {
        enabled: true,
        output_dir: tmp.clone(),
    });
    app.add_plugins(MinimalPlugins);
    app.add_plugins(QASnapshotPlugin);

    // First tick spawns overlay; second tick processes the requested
    // snapshot. We write a request manually so we don't need an Input plugin.
    app.update();
    app.world_mut().write_message(QASnapshotRequested);
    app.update();

    // Locate the emitted snapshot.json — there must be exactly one.
    let mut snapshot_dir: Option<PathBuf> = None;
    for entry in std::fs::read_dir(&tmp).expect("snapshot dir exists") {
        let entry = entry.expect("dir entry readable");
        if entry.file_type().expect("file_type").is_dir() {
            snapshot_dir = Some(entry.path());
            break;
        }
    }
    let snapshot_dir = snapshot_dir.expect("a per-id snapshot dir must exist after one request");
    let raw = std::fs::read_to_string(snapshot_dir.join("snapshot.json"))
        .expect("snapshot.json written");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("snapshot.json parses");

    // Assert layout block shape under MinimalPlugins (no ComputedNode → all
    // surfaces report spawned=false).
    let layout = value.get("layout").expect("layout key present");
    let surfaces = layout["surfaces"].as_array().expect("surfaces array");
    let names: Vec<&str> = surfaces.iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(names, EXPECTED_SURFACE_NAMES);
    // Under `MinimalPlugins` the QA snapshot overlay self-spawns (it's the
    // dev affordance whose plugin we just added). Every other surface
    // depends on its source plugin (HudPlugin, HandUiPlugin, ShopAuction
    // UI, LobbyPlugin, ResultScreen, ConnectionLostOverlay) and must
    // remain unspawned in this minimal world.
    for surface in surfaces {
        let name = surface["name"].as_str().unwrap();
        if name == "qa_snapshot_overlay" {
            continue;
        }
        assert_eq!(
            surface["spawned"], false,
            "MinimalPlugins world: surface {name} must report spawned=false (only \
             qa_snapshot_overlay is self-spawned by QASnapshotPlugin)"
        );
    }

    let limitations = layout["limitations"].as_array().expect("limitations array");
    assert!(
        limitations.iter().any(|s| s.as_str().unwrap().contains("Q-05")),
        "Q-05 limitation must be documented"
    );
    assert!(
        limitations.iter().any(|s| s.as_str().unwrap().contains("Q-06")),
        "Q-06 limitation must be documented"
    );
    assert!(
        limitations.iter().any(|s| s.as_str().unwrap().contains("Q-07")),
        "Q-07 disabled-state limitation must be documented"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
