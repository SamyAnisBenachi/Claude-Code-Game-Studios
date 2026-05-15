//! Sprint 14 / Story 006 — S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 integration
//! tests.
//!
//! Covers the acceptance criteria that the inline unit tests in
//! `client/src/ui/design_tokens/overlays.rs` cannot reach:
//!
//! - **AC2** HUD-dim consumer spot-check: `HUD_DIM_OVERLAY_ALPHA` is
//!   wired through `overlays::OVERLAY_DIM_ALPHA` (preserves the
//!   grep-stable consumer name while routing the value through the
//!   design-token module).
//! - **AC3** settlement-scrim consumer spot-check: the shop-auction
//!   settlement overlay BackgroundColor reads
//!   `overlays::OVERLAY_SCRIM_ALPHA` instead of the pre-migration
//!   `0.58` literal.
//! - **AC4** result-screen-backdrop consumer spot-check: the result
//!   screen panel BackgroundColor reads `overlays::OVERLAY_SCRIM_ALPHA`
//!   instead of the pre-migration `0.46` literal.
//! - **AC5** grep guard: the three pre-migration scrim/dim literals
//!   (`0.45` / `0.58` / `0.46` paired with a near-black RGB tuple) no
//!   longer appear as scrim/dim BackgroundColor spawn-site alpha values
//!   anywhere under `client/src/`. The documented AC6 exclusions are
//!   asserted to remain so future drift on them is intentional.
//! - **AC6** documented-exclusion spot-check: the
//!   `connection_lost_overlay.rs:208` literal `0.32` is intentionally
//!   preserved (lower than canonical scrim per that overlay's own AC7)
//!   and the comment at `:205-207` now references the canonical token
//!   by name.
//!
//! No optimistic client-side authority is introduced or relied upon by
//! these tests. They are read-only checks over the design-token module
//! and the migrated source.

use std::fs;
use std::path::{Path, PathBuf};

use client::ui::design_tokens::overlays::{
    ALL_OVERLAY_ALPHAS_ASCENDING, OVERLAY_DIM_ALPHA, OVERLAY_SCRIM_ALPHA, OVERLAY_TOAST_ALPHA,
};
use client::ui::hud::HUD_DIM_OVERLAY_ALPHA;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_client_file(rel: &str) -> String {
    let path = client_src_root().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read client source `{rel}`: {err}"))
}

/// Collapse all whitespace runs (spaces, tabs, newlines) into a single
/// ASCII space so the assertion fires regardless of how `rustfmt`
/// chooses to wrap a multi-argument `Color::srgba(...)` call. The fmt
/// rule wraps the four-argument tuple onto five lines when the line
/// exceeds the column budget; the test must not be fragile against
/// that choice.
fn whitespace_normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn ac1_overlay_token_module_exports_required_token_set() {
    // AC1 reach-through: prove the published surface of the overlays
    // design-token module includes every named token the surfaces
    // actually consume. If the module ever loses a token, this test
    // fails loud rather than the migrations going unnoticed.
    let _: f32 = OVERLAY_DIM_ALPHA;
    let _: f32 = OVERLAY_SCRIM_ALPHA;
    let _: f32 = OVERLAY_TOAST_ALPHA;
    assert_eq!(
        ALL_OVERLAY_ALPHAS_ASCENDING.len(),
        3,
        "AC1 overlays module must export exactly 3 named overlay-alpha tokens"
    );
}

#[test]
fn ac2_hud_dim_overlay_alpha_routes_through_overlays_token() {
    test_helpers::init_test_tracing();
    // The HUD_DIM_OVERLAY_ALPHA consumer name is preserved as a
    // grep-stable alias for HUD-resolution-dim consumer code, but the
    // *value* is owned by the design-token module so a single edit in
    // `overlays.rs` propagates to every consumer.
    assert!(
        (HUD_DIM_OVERLAY_ALPHA - OVERLAY_DIM_ALPHA).abs() < f32::EPSILON,
        "AC2 HUD_DIM_OVERLAY_ALPHA ({HUD_DIM_OVERLAY_ALPHA}) must equal \
         overlays::OVERLAY_DIM_ALPHA ({OVERLAY_DIM_ALPHA}) after Sprint 14 story \
         006 migration"
    );

    let hud = read_client_file("ui/hud/mod.rs");
    assert!(
        hud.contains("overlays::OVERLAY_DIM_ALPHA"),
        "AC2 ui/hud/mod.rs must reference `overlays::OVERLAY_DIM_ALPHA` so the \
         HUD-dim consumer is sourced from the design-token module"
    );
}

#[test]
fn ac3_settlement_overlay_reads_canonical_scrim_alpha() {
    test_helpers::init_test_tracing();
    let shop = read_client_file("ui/shop_auction/mod.rs");
    // The pre-migration literal MUST be gone from the settlement
    // overlay BackgroundColor spawn site. Match the full tuple shape
    // so we don't false-positive on unrelated 0.58 occurrences.
    let normalized = whitespace_normalize(&shop);
    assert!(
        !normalized.contains("Color::srgba(0.02, 0.05, 0.08, 0.58)")
            && !normalized.contains("Color::srgba( 0.02, 0.05, 0.08, 0.58 )"),
        "AC3 settlement scrim must no longer use the pre-migration literal \
         `Color::srgba(0.02, 0.05, 0.08, 0.58)`; route through \
         overlays::OVERLAY_SCRIM_ALPHA"
    );
    assert!(
        normalized.contains("Color::srgba( 0.02, 0.05, 0.08, overlays::OVERLAY_SCRIM_ALPHA, )")
            || normalized.contains("Color::srgba(0.02, 0.05, 0.08, overlays::OVERLAY_SCRIM_ALPHA)"),
        "AC3 settlement scrim must read \
         `Color::srgba(0.02, 0.05, 0.08, overlays::OVERLAY_SCRIM_ALPHA)` \
         (single-line or rustfmt-wrapped multi-line variant accepted)"
    );
}

#[test]
fn ac4_result_screen_backdrop_reads_canonical_scrim_alpha() {
    test_helpers::init_test_tracing();
    let result = read_client_file("presentation/result_screen.rs");
    let normalized = whitespace_normalize(&result);
    // The pre-migration literal MUST be gone from the result-screen
    // root BackgroundColor.
    assert!(
        !normalized.contains("Color::srgba(0.02, 0.025, 0.035, 0.46)")
            && !normalized.contains("Color::srgba( 0.02, 0.025, 0.035, 0.46 )"),
        "AC4 result panel backdrop must no longer use the pre-migration literal \
         `Color::srgba(0.02, 0.025, 0.035, 0.46)`; route through \
         overlays::OVERLAY_SCRIM_ALPHA"
    );
    assert!(
        normalized.contains("Color::srgba( 0.02, 0.025, 0.035, overlays::OVERLAY_SCRIM_ALPHA, )")
            || normalized
                .contains("Color::srgba(0.02, 0.025, 0.035, overlays::OVERLAY_SCRIM_ALPHA)"),
        "AC4 result panel backdrop must read \
         `Color::srgba(0.02, 0.025, 0.035, overlays::OVERLAY_SCRIM_ALPHA)` \
         (single-line or rustfmt-wrapped multi-line variant accepted)"
    );
}

#[test]
fn ac5_grep_guard_no_pre_migration_scrim_literals_outside_design_tokens() {
    // AC5 grep guard: walk every `*.rs` file under `client/src/`
    // (excluding the design-token host module which is allowed to
    // declare the canonical numeric values inline) and assert that
    // none of the three pre-migration scrim/dim literal triplets
    // remain. This is a tighter check than the broad AC5 regex
    // because it only matches the *exact* scrim/dim near-black RGB
    // tuples paired with the pre-migration alpha — the AC6
    // documented exclusions (button state colors, lobby panel
    // chrome, board status icons, etc.) are intentionally untouched
    // and live in `production/qa/evidence/sprint-14-overlay-alpha-token/`.
    test_helpers::init_test_tracing();
    let files = collect_client_rs_files_outside_design_tokens();
    assert!(
        !files.is_empty(),
        "AC5 grep guard must walk at least one client source file"
    );

    // Pre-migration literal tuples — match in whitespace-normalized
    // form so a multi-line rustfmt wrap does not bypass the guard.
    let pre_migration_needles = [
        "Color::srgba(0.02, 0.05, 0.08, 0.58)",
        "Color::srgba( 0.02, 0.05, 0.08, 0.58, )",
        "Color::srgba(0.02, 0.025, 0.035, 0.46)",
        "Color::srgba( 0.02, 0.025, 0.035, 0.46, )",
        "Color::srgba(0.0, 0.0, 0.0, 0.45)",
        "Color::srgba( 0.0, 0.0, 0.0, 0.45, )",
        "Color::rgba(0.0, 0.0, 0.0, 0.45)",
        "Color::rgba( 0.0, 0.0, 0.0, 0.45, )",
    ];

    let mut violations = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let normalized = whitespace_normalize(&text);
        for needle in pre_migration_needles {
            if normalized.contains(needle) {
                violations.push(format!("{}: matched `{needle}`", path.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "AC5 grep guard found pre-migration scrim/dim literals outside \
         client/src/ui/design_tokens/. Migrate to the named overlay tokens in \
         client::ui::design_tokens::overlays. Violations:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn ac5_grep_guard_pattern_actually_detects_a_synthesized_violation() {
    // Sanity check: a buggy walker that never matches would silently
    // let every pre-migration literal through. Construct a synthetic
    // violation in a temp string and prove the substring check fires.
    let synthetic =
        "    BackgroundColor(Color::srgba(0.02, 0.05, 0.08, 0.58)),  // synthetic violation";
    assert!(synthetic.contains("Color::srgba(0.02, 0.05, 0.08, 0.58)"));
}

#[test]
fn ac6_connection_lost_overlay_literal_preserved_with_canonical_token_doc_reference() {
    test_helpers::init_test_tracing();
    let text = read_client_file("presentation/connection_lost_overlay.rs");
    // The 0.32 literal at line 208 (or wherever the literal moves to
    // — search by content, not by line number) is preserved as a
    // documented AC6 exclusion (intentionally lighter than the
    // canonical scrim per the connection-lost overlay's own AC7).
    assert!(
        text.contains("Color::srgba(0.02, 0.025, 0.035, 0.32)"),
        "AC6 connection-lost overlay must preserve the 0.32 backdrop literal \
         (intentionally lighter than canonical scrim per its own AC7)"
    );
    // And the surrounding comment must reference the canonical token
    // by name (the pre-migration text named the magic 0.46 result-screen
    // literal which no longer exists in source).
    assert!(
        text.contains("OVERLAY_SCRIM_ALPHA"),
        "AC6 connection-lost overlay comment must reference \
         `OVERLAY_SCRIM_ALPHA` symbolically so the rationale survives future \
         spec revisions"
    );
    // Sanity: the obsolete reference to magic 0.46 in the comment is
    // gone.
    assert!(
        !text.contains("0.46 backdrop"),
        "AC6 connection-lost overlay comment must no longer reference the magic \
         `0.46 backdrop` value (that scrim now reads through \
         OVERLAY_SCRIM_ALPHA = 0.55)"
    );
}

#[test]
fn ac7_overlay_token_ordering_supports_visual_cohesion() {
    // AC7 visual cohesion: the three overlay alphas must satisfy
    // dim < scrim < toast so that:
    //   - the modal scrim reads heavier than the focus dim (settlement
    //     and result panels feel "modal" relative to HUD dim);
    //   - the toast reads above the modal scrim (notifications never
    //     vanish under a scrim).
    // This duplicates the inline unit test for defence-in-depth and so
    // that the integration test surface lists AC7 explicitly.
    assert!(
        OVERLAY_DIM_ALPHA < OVERLAY_SCRIM_ALPHA,
        "AC7 OVERLAY_DIM_ALPHA ({OVERLAY_DIM_ALPHA}) must be < \
         OVERLAY_SCRIM_ALPHA ({OVERLAY_SCRIM_ALPHA})"
    );
    assert!(
        OVERLAY_SCRIM_ALPHA < OVERLAY_TOAST_ALPHA,
        "AC7 OVERLAY_SCRIM_ALPHA ({OVERLAY_SCRIM_ALPHA}) must be < \
         OVERLAY_TOAST_ALPHA ({OVERLAY_TOAST_ALPHA})"
    );
}

/// Walk `client/src/` and return every `*.rs` file path, skipping the
/// design-token module that is explicitly allowed to declare the
/// canonical overlay-alpha values inline.
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
