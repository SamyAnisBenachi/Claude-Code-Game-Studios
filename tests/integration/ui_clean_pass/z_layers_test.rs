//! Sprint 14 / Story 002 — S11-TD-UI-ZINDEX-LAYERS integration tests.
//!
//! Covers the acceptance criteria that the inline unit tests in
//! `client/src/ui/design_tokens/z_layers.rs` cannot reach:
//!
//! - **AC5** workspace grep-guard: no inline `ZIndex(N)` / `GlobalZIndex(N)`
//!   literals remain anywhere under `client/src/` except the design-token
//!   module itself.
//! - **AC6** reconnect / snapshot-rebuild paint-order invariant: spawning
//!   layered UI roots in arbitrary order still yields a strictly-ascending
//!   `GlobalZIndex` order matching the named layer hierarchy, not the spawn
//!   order.
//! - **AC7** spot-checks of the production migration sites (lobby root, HUD
//!   root, hand fan root, shop-auction root, settings root, photosensitivity
//!   root, result screen root, connection-lost overlay root) reference the
//!   design-token constants — surface-level grep against the source.
//! - **AC8** ADR-021 alignment: the module's doc names ADR-021 and the
//!   `PresentationPlugin` composition order as the authoritative load-order
//!   contract; the named layers do not reorder presentation plugin
//!   registration.
//!
//! No optimistic client-side authority is introduced or relied upon by these
//! tests. They are read-only checks over the design-token module and the
//! migrated source.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use client::ui::design_tokens::z_layers::{
    self, ALL_LAYERS_ASCENDING, BACKGROUND, DEBUG, LAYER_MIN_GAP, MODAL, TOAST, UI_BASE,
    UI_OVERLAY, UNITS, WORLD,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Walk `client/src/` and return every `*.rs` file path, skipping the
/// design-token module that is explicitly allowed to declare the
/// `GlobalZIndex(N)` literals.
fn collect_client_rs_files_outside_design_tokens() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root = client_src_root();
    let design_tokens_dir = root.join("ui").join("design_tokens");
    walk_dir(&root, &design_tokens_dir, &mut out);
    out
}

fn walk_dir(dir: &Path, design_tokens_dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read dir {}: {err}", dir.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.starts_with(design_tokens_dir) {
            continue;
        }
        if path.is_dir() {
            walk_dir(&path, design_tokens_dir, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn ac5_grep_guard_no_inline_global_z_index_literals_outside_design_tokens() {
    test_helpers::init_test_tracing();
    let files = collect_client_rs_files_outside_design_tokens();
    assert!(
        !files.is_empty(),
        "AC5 grep guard must walk at least one client source file"
    );
    let mut violations = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        for (line_no, line) in text.lines().enumerate() {
            if line.contains("ZIndex(") || line.contains("GlobalZIndex(") {
                violations.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    line_no + 1,
                    line.trim_end()
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "AC5 grep guard found inline ZIndex(N) / GlobalZIndex(N) literals \
         outside client/src/ui/design_tokens/. Migrate to the named layer \
         constants in client::ui::design_tokens::z_layers. Violations:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn ac6_paint_order_matches_named_layers_under_out_of_order_spawn() {
    // Spawns layered UI roots in REVERSE order (Debug first, Background last)
    // — the equivalent of a reconnect / snapshot rebuild that respawns roots
    // out of their initial order — then queries the `GlobalZIndex` values
    // and asserts the effective paint order matches the named layer
    // hierarchy, not the spawn order.
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    #[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
    struct LayerMarker(&'static str);

    // Build a reversed-order spawn list so the spawn iteration order is the
    // opposite of the canonical layer order.
    let spawn_in_reverse: Vec<(&'static str, GlobalZIndex)> = ALL_LAYERS_ASCENDING
        .iter()
        .rev()
        .map(|(name, layer)| (*name, *layer))
        .collect();
    for (name, layer) in &spawn_in_reverse {
        app.world_mut()
            .spawn((LayerMarker(name), Node::default(), *layer));
    }

    // Query in the order the entities were spawned and assert the layer
    // names came out in reverse-canonical order (sanity: world iteration
    // returns spawn order).
    let mut spawn_order: Vec<&'static str> = Vec::new();
    let mut layer_values: Vec<(&'static str, i32)> = Vec::new();
    {
        let mut query = app.world_mut().query::<(&LayerMarker, &GlobalZIndex)>();
        for (marker, z) in query.iter(app.world()) {
            spawn_order.push(marker.0);
            layer_values.push((marker.0, z.0));
        }
    }
    assert_eq!(
        spawn_order.len(),
        ALL_LAYERS_ASCENDING.len(),
        "AC6 must spawn one entity per named layer"
    );

    // Sort by the GlobalZIndex value — that is the bevy_ui paint order.
    let mut paint_order = layer_values.clone();
    paint_order.sort_by_key(|(_, z)| *z);
    let paint_names: Vec<&'static str> = paint_order.iter().map(|(n, _)| *n).collect();

    let canonical_names: Vec<&'static str> = ALL_LAYERS_ASCENDING.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        paint_names, canonical_names,
        "AC6 effective paint order must match the named layer hierarchy \
         regardless of spawn order. Spawn order was {:?}; paint order should \
         be {:?}",
        spawn_order, canonical_names,
    );

    // Spot-assert: the spawn order really was reversed (so we're not just
    // asserting against an accidentally-canonical spawn).
    let reversed_canonical: Vec<&'static str> = canonical_names.iter().rev().copied().collect();
    assert_eq!(
        spawn_order, reversed_canonical,
        "AC6 test setup precondition: entities must be spawned in reverse \
         canonical order to prove the invariant survives a non-canonical spawn"
    );
}

#[test]
fn ac6_layer_constants_survive_pairwise_distinctness_under_arbitrary_permutation() {
    // A second AC6 angle: even if the spawn order is an arbitrary
    // permutation (not just reversed), the GlobalZIndex values resolved from
    // the named constants remain pairwise-distinct and the canonical
    // hierarchy still emerges when sorted.
    let permutation: [(&str, GlobalZIndex); 8] = [
        ("Modal", MODAL),
        ("Background", BACKGROUND),
        ("Toast", TOAST),
        ("UiBase", UI_BASE),
        ("Debug", DEBUG),
        ("UiOverlay", UI_OVERLAY),
        ("Units", UNITS),
        ("World", WORLD),
    ];
    let mut sorted: Vec<(&str, i32)> = permutation.iter().map(|(n, l)| (*n, l.0)).collect();
    sorted.sort_by_key(|(_, z)| *z);
    let sorted_names: Vec<&str> = sorted.iter().map(|(n, _)| *n).collect();
    let canonical: Vec<&str> = ALL_LAYERS_ASCENDING.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        sorted_names, canonical,
        "AC6 sorted permutation must equal canonical layer order"
    );
}

#[test]
fn ac7_production_migration_sites_reference_design_tokens() {
    test_helpers::init_test_tracing();
    let cases = [
        ("ui/lobby.rs", "z_layers::UI_OVERLAY"),
        ("ui/lobby.rs", "z_layers::MODAL"),
        ("ui/hud/mod.rs", "z_layers::UI_BASE"),
        ("ui/hud/mod.rs", "z_layers::UI_OVERLAY"),
        ("ui/hand/mod.rs", "z_layers::UI_BASE"),
        ("ui/hand/mod.rs", "z_layers::UI_OVERLAY"),
        ("ui/shop_auction/mod.rs", "z_layers::UI_BASE"),
        ("ui/shop_auction/mod.rs", "z_layers::UI_OVERLAY"),
        ("ui/shop_auction/mod.rs", "z_layers::TOAST"),
        ("ui/settings/mod.rs", "z_layers::MODAL"),
        ("ui/photosensitivity_warning.rs", "z_layers::MODAL"),
        ("presentation/result_screen.rs", "z_layers::MODAL"),
        (
            "presentation/connection_lost_overlay.rs",
            "z_layers::UI_OVERLAY",
        ),
    ];
    for (rel, needle) in cases {
        let path = client_src_root().join(rel);
        let text = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("AC7 spot-check failed to read {}: {err}", path.display())
        });
        assert!(
            text.contains(needle),
            "AC7 production migration site {rel} must reference design-token \
             constant `{needle}` (search returned no match)",
        );
    }
}

#[test]
fn ac8_module_doc_names_adr_021_and_presentation_plugin_load_order() {
    test_helpers::init_test_tracing();
    let module = client_src_root()
        .join("ui")
        .join("design_tokens")
        .join("z_layers.rs");
    let text = fs::read_to_string(&module)
        .unwrap_or_else(|err| panic!("AC8 module doc read failed for {}: {err}", module.display()));
    assert!(
        text.contains("ADR-021"),
        "AC8 z-layer module doc must name ADR-021 (Presentation Layer Architecture)"
    );
    assert!(
        text.contains("PresentationPlugin"),
        "AC8 z-layer module doc must reference PresentationPlugin composition \
         order so future readers can reconcile the named layers against the \
         canonical load-order"
    );
    assert!(
        text.contains("ADR-002"),
        "AC8 z-layer module doc must affirm ADR-002 (Client-Server Authority) \
         binding — no optimistic client-side authority introduced"
    );
}

#[test]
fn module_exports_minimum_gap_constant_for_future_intermediate_layers() {
    // LAYER_MIN_GAP exists and is positive so the AC1 inline unit test that
    // asserts a non-zero gap between adjacent layers is meaningful.
    assert!(
        LAYER_MIN_GAP > 0,
        "LAYER_MIN_GAP must be positive so adjacent-layer gap assertions are meaningful"
    );
    // Also surface that the canonical 100-unit gap is well above the
    // minimum, so removing or shrinking the gap in a future story is a
    // deliberate decision rather than an accident.
    assert!(
        z_layers::WORLD.0 - z_layers::BACKGROUND.0 >= LAYER_MIN_GAP,
        "canonical gap between BACKGROUND and WORLD must satisfy LAYER_MIN_GAP"
    );
}
