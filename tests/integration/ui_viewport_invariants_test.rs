//! Sprint 14 / Story 005 — automated UI viewport-invariant test bin.
//!
//! Story file: `production/epics/ui-clean-pass/story-005-ui-viewport-invariant-tests.md`
//! Story ID: `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (Sprint 14 Tier 0 rank 4).
//!
//! ## What this bin asserts
//!
//! - **AC1 / AC4** — across the six canonical viewport sizes
//!   (`1366x768`, `1920x1080`, `1920x1200`, `1280x960`, `3840x2160`,
//!   `2560x1080`), spawns the baseline UI surface set and exercises the
//!   four invariant classes (no-overlap, no-clipping, anchor-stability,
//!   strip-height determinism). Each viewport size is printed in the
//!   test output (via `eprintln!`) so AC4 verification (`cargo test
//!   -- --nocapture` shows each viewport named) is satisfied even
//!   when individual assertions succeed silently.
//! - **AC2** — exercises the three named helper functions
//!   (`spawn_with_viewport`, `extract_root_bounds`,
//!   `assert_invariants_against_baseline`).
//! - **AC3** — the baseline fixture file is consumed verbatim from
//!   `tests/integration/fixtures/ui_viewport_baseline.rs`.
//! - **AC5** — `test_synthesized_overlap_is_detected` synthesises a
//!   geometric overlap and asserts the no-overlap rule fails with a
//!   clear named-root error message.
//! - **AC6** — `test_synthesized_clipping_is_detected` synthesises a
//!   surface that extends past the viewport rectangle and asserts the
//!   no-clipping rule fails with a clear named-root + edge message.
//! - **AC7** — `test_synthesized_baseline_drift_is_detected` synthesises
//!   anchor drift and strip-height drift and asserts both rules fail
//!   with clear named-root + expected-vs-actual messages.
//! - **AC8** — the test bin is registered in `client/Cargo.toml` under
//!   `[[test]]` so `cargo test -p client` and `cargo test --workspace`
//!   execute it (does NOT silently skip).
//! - **AC9** — no `#[ignore]` markers anywhere in this bin EXCEPT (none
//!   needed; the negative tests use `Result`-based assertions that pass
//!   normally without `#[ignore]` per the AC9 exception). The
//!   provisional baseline satisfies all four invariant classes by
//!   construction; the worker report enumerates this and documents the
//!   ratify-on-spec follow-on requirement for story 007 numeric values.
//! - **AC10** — friend-game scope preserved. This bin does NOT advance
//!   `QA-COND-0005` Standard-tier accessibility, `QA-COND-0006`
//!   playtest validation, or `PAW-TD-*-a` placeholder-art.
//!
//! ## Cargo policy
//!
//! Run under the binding Windows/MSVC Cargo resource policy
//! (qa-plan §"Cargo Resource Policy"):
//!
//! ```text
//! $env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
//! $env:CARGO_PROFILE_DEV_DEBUG='0'
//! $env:CARGO_PROFILE_TEST_DEBUG='0'
//! $env:CARGO_INCREMENTAL='0'
//! $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
//! cargo test -p client --test ui_viewport_invariants_test -- --nocapture
//! ```
//!
//! ## ADR alignment
//!
//! - **ADR-002 Client-Server Authority**: read-only geometry test bin.
//!   No optimistic client-side authority introduced.
//! - **ADR-021 Presentation Layer Architecture**: defers to story 002
//!   named [`bevy::ui::GlobalZIndex`] hierarchy for paint ordering.

use bevy::prelude::*;

#[path = "helpers/ui_viewport.rs"]
mod ui_viewport;

#[path = "fixtures/ui_viewport_baseline.rs"]
mod ui_viewport_baseline;

#[path = "../test_helpers.rs"]
mod test_helpers;

use ui_viewport::{
    assert_anchor_stability, assert_invariants_against_baseline, assert_no_clipping,
    assert_no_overlap, assert_strip_height_determinism, extract_root_bounds, spawn_with_viewport,
    BaselineViewportTag, DisplayPhase, Invariant, ProportionalAnchor, SurfaceKind, UiRootBounds,
    ZLayer, CANONICAL_VIEWPORTS,
};
use ui_viewport_baseline::{HEADER_BAR_HEIGHT_PX, PROVISIONAL_BASELINE};

/// AC1 / AC4 positive coverage: at each of the six canonical viewport
/// sizes, the baseline-driven harness exercises the four invariant
/// classes and they all pass. Each viewport name is printed via
/// `eprintln!` so `cargo test -- --nocapture` reveals the matrix
/// coverage (AC4 verification).
#[test]
fn ac1_ac4_viewport_invariant_suite_passes_across_canonical_matrix() {
    test_helpers::init_test_tracing();
    let baseline = &PROVISIONAL_BASELINE;
    let mut covered_viewports: Vec<&'static str> = Vec::new();
    for viewport in CANONICAL_VIEWPORTS {
        eprintln!(
            "[ui_viewport_invariants] running invariant suite at viewport {} ({}x{})",
            viewport.name, viewport.width, viewport.height
        );
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let spawned = spawn_with_viewport(&mut app, viewport, baseline);
        assert!(
            !spawned.is_empty(),
            "AC1 baseline must spawn at least one surface at viewport {}",
            viewport.name,
        );
        let bounds = extract_root_bounds(&mut app, viewport);
        assert_eq!(
            bounds.len(),
            spawned.len(),
            "AC2 extract_root_bounds must return every spawned surface at viewport {}",
            viewport.name,
        );

        assert_no_overlap(&bounds, viewport).unwrap_or_else(|e| {
            panic!("AC1 no-overlap failed at viewport {}: {}", viewport.name, e)
        });
        assert_no_clipping(&bounds, viewport).unwrap_or_else(|e| {
            panic!(
                "AC1 no-clipping failed at viewport {}: {}",
                viewport.name, e
            )
        });
        assert_invariants_against_baseline(&mut app, viewport, baseline).unwrap_or_else(|e| {
            panic!(
                "AC1 composed assert_invariants_against_baseline failed at viewport {}: {}",
                viewport.name, e
            )
        });
        covered_viewports.push(viewport.name);
    }
    assert_eq!(
        covered_viewports.len(),
        6,
        "AC4 invariant suite MUST execute at all six canonical viewport sizes; got {} of 6: {:?}",
        covered_viewports.len(),
        covered_viewports,
    );
}

/// AC1 cross-viewport invariant: anchor stability and strip-height
/// determinism are baseline-wide invariants. Run them once outside the
/// per-viewport loop.
#[test]
fn ac1_anchor_stability_and_strip_height_determinism_pass_baseline_wide() {
    test_helpers::init_test_tracing();
    let baseline = &PROVISIONAL_BASELINE;
    assert_anchor_stability(baseline)
        .unwrap_or_else(|e| panic!("AC1 anchor stability failed across baseline: {}", e));
    assert_strip_height_determinism(baseline)
        .unwrap_or_else(|e| panic!("AC1 strip-height determinism failed across baseline: {}", e));
}

/// AC2 coverage: confirm the three named helper functions are exported,
/// callable, and produce consistent results.
#[test]
fn ac2_helper_module_exposes_three_reusable_functions() {
    let baseline = &PROVISIONAL_BASELINE;
    let viewport = CANONICAL_VIEWPORTS[1]; // 1920x1080 — canonical HD reference.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let spawned = spawn_with_viewport(&mut app, viewport, baseline);
    let bounds = extract_root_bounds(&mut app, viewport);
    let result = assert_invariants_against_baseline(&mut app, viewport, baseline);

    assert!(
        !spawned.is_empty(),
        "AC2 spawn_with_viewport must spawn at least one entity"
    );
    assert!(
        !bounds.is_empty(),
        "AC2 extract_root_bounds must return at least one bounds record"
    );
    assert!(
        result.is_ok(),
        "AC2 composed assert_invariants_against_baseline must return Ok on the canonical \
         baseline at {}; got {:?}",
        viewport.name,
        result,
    );
}

/// AC3 coverage: the baseline fixture is non-empty and records each
/// surface at every viewport in the canonical matrix.
#[test]
fn ac3_baseline_fixture_records_every_surface_at_every_viewport() {
    let baseline = &PROVISIONAL_BASELINE;
    assert!(
        !baseline.surfaces.is_empty(),
        "AC3 baseline fixture must list at least one surface"
    );
    for surface in baseline.surfaces {
        for viewport in CANONICAL_VIEWPORTS {
            assert!(
                surface.rect_for(viewport.name).is_some(),
                "AC3 baseline fixture must record surface {} at viewport {}",
                surface.name,
                viewport.name,
            );
        }
    }
    // Story 005 §AC1 surface list — confirm each named family is
    // represented at least once in the baseline.
    let names: Vec<&'static str> = baseline.surfaces.iter().map(|s| s.name).collect();
    let required = [
        "lobby_root",
        "hud_header_bar",
        "hud_footer_bar",
        "hand_ui_hand_bar",
        "draft_centered_modal",
        "shop_panel",
        "auction_panel",
        "settlement_overlay",
        "result_screen",
    ];
    for needle in required {
        assert!(
            names.contains(&needle),
            "AC3 baseline must include surface {} (story 005 AC1 surface list)",
            needle
        );
    }
}

/// AC5: synthesised geometric overlap MUST be detected by the
/// no-overlap rule with a clear named-root error message.
#[test]
fn test_synthesized_overlap_is_detected() {
    test_helpers::init_test_tracing();
    let viewport = CANONICAL_VIEWPORTS[1];
    let bounds = vec![
        UiRootBounds {
            name: "synth_alpha",
            phase: DisplayPhase::InSessionBase,
            kind: SurfaceKind::Surface,
            z_layer: ZLayer::UiBase,
            anchor: ProportionalAnchor::TOP_LEFT,
            x: 100.0,
            y: 100.0,
            width: 400.0,
            height: 400.0,
        },
        UiRootBounds {
            name: "synth_beta",
            phase: DisplayPhase::InSessionBase,
            kind: SurfaceKind::Surface,
            z_layer: ZLayer::UiBase,
            anchor: ProportionalAnchor::TOP_LEFT,
            x: 300.0,
            y: 300.0,
            width: 400.0,
            height: 400.0,
        },
    ];
    let result = assert_no_overlap(&bounds, viewport);
    let Err(failure) = result else {
        panic!("AC5 no-overlap rule MUST fail when two UI roots geometrically overlap");
    };
    assert_eq!(failure.invariant, Invariant::NoOverlap);
    assert_eq!(failure.surface, "synth_alpha");
    assert_eq!(failure.other_surface, Some("synth_beta"));
    let rendered = failure.to_string();
    assert!(
        rendered.contains("synth_alpha") && rendered.contains("synth_beta"),
        "AC5 failure message must name both overlapping roots; got {}",
        rendered,
    );
    assert!(
        rendered.contains("overlap"),
        "AC5 failure message must mention overlap; got {}",
        rendered,
    );
}

/// AC5 negative test: overlay z-layer surfaces are EXCLUDED from the
/// geometric no-overlap rule. Even when their rectangles overlap, the
/// rule must NOT fire — story 002's named z-layer hierarchy guarantees
/// paint order.
#[test]
fn ac5_overlay_z_layer_geometric_overlap_is_excluded_from_rule() {
    let viewport = CANONICAL_VIEWPORTS[1];
    let bounds = vec![
        UiRootBounds {
            name: "ui_base_surface",
            phase: DisplayPhase::InSessionBase,
            kind: SurfaceKind::Surface,
            z_layer: ZLayer::UiBase,
            anchor: ProportionalAnchor::TOP_LEFT,
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
        UiRootBounds {
            name: "overlay_surface",
            phase: DisplayPhase::Settlement,
            kind: SurfaceKind::Surface,
            z_layer: ZLayer::UiOverlay,
            anchor: ProportionalAnchor::TOP_LEFT,
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        },
    ];
    let result = assert_no_overlap(&bounds, viewport);
    assert!(
        result.is_ok(),
        "AC5 overlay z-layer surfaces must be excluded from the geometric overlap rule \
         (story 002 z-layer hierarchy guarantees paint order); got {:?}",
        result,
    );
}

/// AC6: synthesised clipping MUST be detected by the no-clipping rule
/// with a clear named-root + edge message.
#[test]
fn test_synthesized_clipping_is_detected() {
    test_helpers::init_test_tracing();
    let viewport = CANONICAL_VIEWPORTS[0]; // 1366x768 — minimum viewport.
    let bounds = vec![UiRootBounds {
        name: "synth_clipped",
        phase: DisplayPhase::InSessionBase,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::TOP_LEFT,
        x: 1300.0,
        y: 100.0,
        width: 400.0, // 1300 + 400 = 1700 > 1366 viewport width
        height: 200.0,
    }];
    let result = assert_no_clipping(&bounds, viewport);
    let Err(failure) = result else {
        panic!("AC6 no-clipping rule MUST fail when a surface extends past the viewport rectangle");
    };
    assert_eq!(failure.invariant, Invariant::NoClipping);
    assert_eq!(failure.surface, "synth_clipped");
    let rendered = failure.to_string();
    assert!(
        rendered.contains("synth_clipped"),
        "AC6 failure message must name the clipped surface; got {}",
        rendered,
    );
    assert!(
        rendered.contains("right")
            || rendered.contains("left")
            || rendered.contains("top")
            || rendered.contains("bottom"),
        "AC6 failure message must name a viewport edge; got {}",
        rendered,
    );
}

/// AC7 — anchor drift: synthesise a baseline where a surface's per-viewport
/// rectangle does NOT match its declared proportional anchor and confirm
/// the anchor-stability rule fails with a clear named-root + expected-vs-actual
/// message.
#[test]
fn test_synthesized_baseline_drift_is_detected() {
    use ui_viewport::{SurfaceBaseline, ViewportBaseline};
    test_helpers::init_test_tracing();

    // Surface declares anchor=CENTER but the per-viewport rect places it
    // off-center by 100px in x — anchor stability MUST flag it.
    static DRIFTED_RECT: [(&str, f32, f32, f32, f32); 6] = [
        (
            "1366x768",
            (1366.0 - 800.0) / 2.0 + 100.0,
            (768.0 - 300.0) / 2.0,
            800.0,
            300.0,
        ),
        (
            "1920x1080",
            (1920.0 - 800.0) / 2.0,
            (1080.0 - 300.0) / 2.0,
            800.0,
            300.0,
        ),
        (
            "1920x1200",
            (1920.0 - 800.0) / 2.0,
            (1200.0 - 300.0) / 2.0,
            800.0,
            300.0,
        ),
        (
            "1280x960",
            (1280.0 - 800.0) / 2.0,
            (960.0 - 300.0) / 2.0,
            800.0,
            300.0,
        ),
        (
            "3840x2160",
            (3840.0 - 800.0) / 2.0,
            (2160.0 - 300.0) / 2.0,
            800.0,
            300.0,
        ),
        (
            "2560x1080",
            (2560.0 - 800.0) / 2.0,
            (1080.0 - 300.0) / 2.0,
            800.0,
            300.0,
        ),
    ];
    static DRIFTED_SURFACES: [SurfaceBaseline; 1] = [SurfaceBaseline {
        name: "drifted_modal",
        phase: DisplayPhase::DraftInitial,
        kind: SurfaceKind::Surface,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::CENTER,
        per_viewport: &DRIFTED_RECT,
        strip_height_px: None,
    }];
    const DRIFTED_BASELINE: ViewportBaseline = ViewportBaseline {
        surfaces: &DRIFTED_SURFACES,
    };

    let result = assert_anchor_stability(&DRIFTED_BASELINE);
    let Err(failure) = result else {
        panic!("AC7 anchor-stability rule MUST fail when a per-viewport rect drifts from its declared anchor");
    };
    assert_eq!(failure.invariant, Invariant::AnchorStability);
    assert_eq!(failure.surface, "drifted_modal");
    let rendered = failure.to_string();
    assert!(
        rendered.contains("drifted_modal") && rendered.contains("anchor"),
        "AC7 failure message must name the drifted root and mention anchor; got {}",
        rendered,
    );
    assert!(
        rendered.contains("dx=") || rendered.contains("dy="),
        "AC7 failure message must expose the dx/dy delta so the reader can diagnose; got {}",
        rendered,
    );
}

/// AC7 — strip-height drift: synthesise a baseline where a strip's
/// height varies across viewports (or differs from its declared
/// strip_height_px) and confirm the strip-height-determinism rule fails
/// with a clear message.
#[test]
fn test_synthesized_strip_height_drift_is_detected() {
    use ui_viewport::{SurfaceBaseline, ViewportBaseline};
    test_helpers::init_test_tracing();

    // Strip declares strip_height_px=60.0 but the third viewport rect
    // has a height of 72.0 — strip-height determinism MUST flag it.
    static DRIFTED_STRIP_RECTS: [(&str, f32, f32, f32, f32); 6] = [
        ("1366x768", 0.0, 0.0, 1366.0, HEADER_BAR_HEIGHT_PX),
        ("1920x1080", 0.0, 0.0, 1920.0, HEADER_BAR_HEIGHT_PX),
        ("1920x1200", 0.0, 0.0, 1920.0, 72.0), // drift!
        ("1280x960", 0.0, 0.0, 1280.0, HEADER_BAR_HEIGHT_PX),
        ("3840x2160", 0.0, 0.0, 3840.0, HEADER_BAR_HEIGHT_PX),
        ("2560x1080", 0.0, 0.0, 2560.0, HEADER_BAR_HEIGHT_PX),
    ];
    static DRIFTED_STRIP_SURFACES: [SurfaceBaseline; 1] = [SurfaceBaseline {
        name: "drifted_header_bar",
        phase: DisplayPhase::InSessionBase,
        kind: SurfaceKind::Strip,
        z_layer: ZLayer::UiBase,
        anchor: ProportionalAnchor::TOP_LEFT,
        per_viewport: &DRIFTED_STRIP_RECTS,
        strip_height_px: Some(HEADER_BAR_HEIGHT_PX),
    }];
    const DRIFTED_STRIP_BASELINE: ViewportBaseline = ViewportBaseline {
        surfaces: &DRIFTED_STRIP_SURFACES,
    };

    let result = assert_strip_height_determinism(&DRIFTED_STRIP_BASELINE);
    let Err(failure) = result else {
        panic!(
            "AC7 strip-height-determinism rule MUST fail when a strip's pixel height drifts \
             from its declared strip_height_px"
        );
    };
    assert_eq!(failure.invariant, Invariant::StripHeight);
    assert_eq!(failure.surface, "drifted_header_bar");
    let rendered = failure.to_string();
    assert!(
        rendered.contains("drifted_header_bar") && rendered.contains("60.0"),
        "AC7 strip-height failure must name the drifted strip and show expected height; got {}",
        rendered,
    );
}

/// Surface-extraction sanity check: each spawned [`UiRootBounds`] entity
/// carries a [`BaselineViewportTag`] that pairs it with the viewport it
/// was resolved against. Avoids cross-talk between successive
/// spawn_with_viewport calls in the same App.
#[test]
fn extracted_bounds_carry_viewport_tag_for_filtering() {
    let baseline = &PROVISIONAL_BASELINE;
    let viewport_a = CANONICAL_VIEWPORTS[0];
    let viewport_b = CANONICAL_VIEWPORTS[1];
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    spawn_with_viewport(&mut app, viewport_a, baseline);
    spawn_with_viewport(&mut app, viewport_b, baseline);

    let bounds_a = extract_root_bounds(&mut app, viewport_a);
    let bounds_b = extract_root_bounds(&mut app, viewport_b);

    assert_eq!(
        bounds_a.len(),
        baseline.surfaces.len(),
        "extract_root_bounds(viewport_a) must filter to viewport_a's tag"
    );
    assert_eq!(
        bounds_b.len(),
        baseline.surfaces.len(),
        "extract_root_bounds(viewport_b) must filter to viewport_b's tag"
    );
    let total_in_world = {
        let mut q = app.world_mut().query::<&BaselineViewportTag>();
        q.iter(app.world()).count()
    };
    assert_eq!(
        total_in_world,
        bounds_a.len() + bounds_b.len(),
        "spawn_with_viewport for two distinct viewports must spawn 2 * baseline.surfaces entities"
    );
}

/// Canonical-matrix size sanity: AC1 requires "at least 6 canonical
/// viewport sizes". The fixture provides exactly the 6 required.
#[test]
fn canonical_viewport_matrix_covers_required_six_sizes() {
    let names: Vec<&'static str> = CANONICAL_VIEWPORTS.iter().map(|v| v.name).collect();
    for required in [
        "1366x768",
        "1920x1080",
        "1920x1200",
        "1280x960",
        "3840x2160",
        "2560x1080",
    ] {
        assert!(
            names.contains(&required),
            "AC1 canonical viewport matrix must include {} (got {:?})",
            required,
            names,
        );
    }
    assert_eq!(
        CANONICAL_VIEWPORTS.len(),
        6,
        "AC1 canonical viewport matrix size must be exactly 6"
    );
}

/// AC10 friend-game scope preservation: the helper module + fixture
/// MUST NOT advance Standard-tier accessibility (`QA-COND-0005`),
/// playtest validation (`QA-COND-0006`), or placeholder-art accept-risk
/// (`PAW-TD-*-a`). Documented inline so future readers see the
/// preservation in source.
#[test]
fn ac10_friend_game_scope_preservation_is_documented_inline() {
    use std::fs;
    use std::path::{Path, PathBuf};
    fn tests_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("client manifest dir must have a parent (workspace root)")
            .join("tests")
            .join("integration")
    }
    let helper = tests_root().join("helpers").join("ui_viewport.rs");
    let fixture = tests_root()
        .join("fixtures")
        .join("ui_viewport_baseline.rs");
    let test_bin = tests_root().join("ui_viewport_invariants_test.rs");
    for path in [&helper, &fixture, &test_bin] {
        let text = fs::read_to_string(path).unwrap_or_else(|e| {
            panic!(
                "AC10 docstring inspection failed: read {} err {e}",
                path.display()
            )
        });
        assert!(
            text.contains("QA-COND-0005"),
            "AC10 friend-game scope preservation must reference QA-COND-0005 in {}",
            path.display(),
        );
        assert!(
            text.contains("QA-COND-0006"),
            "AC10 friend-game scope preservation must reference QA-COND-0006 in {}",
            path.display(),
        );
        assert!(
            text.contains("PAW-TD"),
            "AC10 friend-game scope preservation must reference PAW-TD-*-a in {}",
            path.display(),
        );
    }
}
