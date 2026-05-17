//! Sprint 15 / Story 008 — S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001
//! integration test.
//!
//! Covers AC8 (integration test asserts primitive module shape) plus the
//! reach-through assertions for AC1..AC5, AC6 (export shape), and AC9
//! (no inline literal regressions on the module's own surface). The
//! inline `#[cfg(test)] mod tests` block in
//! `client/src/ui/design_tokens/interaction_states.rs` already
//! enforces canonical-band ranges and visual-state ordering invariants;
//! this integration bin proves the **public-path import shape** is what
//! downstream Sprint 16+ migration stories will consume, and runs a
//! source-file scan asserting that every published constant carries at
//! least one `///` doc-comment line.
//!
//! Out of scope (per story §Scope / Out of Scope and AC10): per-surface
//! migration of existing Sprint 14 button surfaces. The lobby / auction
//! / HUD button styling at `S11-UX-LOBBY-BUTTON-HITTARGETS`,
//! `S11-UX-AUCTION-FEATURED-CARD`, and `S11-UX-HUD-TOP-STRIP-LAYOUT`
//! call sites is **not** asserted by this test; per-surface migration is
//! a Sprint 16+ follow-on story (`S16-UI-INTERACTION-STATE-MIGRATION-*`).

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::Color;

use client::ui::design_tokens::interaction_states::{
    ALL_INTERACTION_STATE_ALPHAS, ALL_INTERACTION_STATE_PIXELS, DISABLED_BG_TINT_ALPHA,
    DISABLED_BORDER_ALPHA, DISABLED_TEXT_ALPHA, FOCUS_RING_COLOR, FOCUS_RING_OFFSET_PX,
    FOCUS_RING_WIDTH_PX, HOVER_BG_TINT_ALPHA, HOVER_BORDER_ALPHA, PRESSED_BG_TINT_ALPHA,
    PRESSED_OFFSET_Y_PX,
};

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn module_path() -> PathBuf {
    client_src_root()
        .join("ui")
        .join("design_tokens")
        .join("interaction_states.rs")
}

fn read_module_source() -> String {
    let path = module_path();
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read interaction_states.rs at {path:?}: {err}"))
}

#[test]
fn ac1_ac6_module_exports_four_named_token_set_families() {
    // AC1 / AC6 reach-through: prove the published surface of the
    // interaction_states design-token module includes every named token
    // the four families require. If the module ever loses a token, this
    // test fails loud rather than the per-surface migrations going
    // unnoticed when Sprint 16+ lands.
    let _: f32 = HOVER_BG_TINT_ALPHA;
    let _: f32 = HOVER_BORDER_ALPHA;
    let _: Color = FOCUS_RING_COLOR;
    let _: f32 = FOCUS_RING_WIDTH_PX;
    let _: f32 = FOCUS_RING_OFFSET_PX;
    let _: f32 = PRESSED_BG_TINT_ALPHA;
    let _: f32 = PRESSED_OFFSET_Y_PX;
    let _: f32 = DISABLED_BG_TINT_ALPHA;
    let _: f32 = DISABLED_TEXT_ALPHA;
    let _: f32 = DISABLED_BORDER_ALPHA;
}

#[test]
fn ac2_ac4_ac5_audit_array_alphas_in_unit_interval() {
    // AC8 / AC2 / AC4 / AC5: every audited alpha must satisfy
    // `0.0 <= alpha <= 1.0`. Iterates the published audit array so a
    // future addition is automatically covered.
    assert!(
        !ALL_INTERACTION_STATE_ALPHAS.is_empty(),
        "AC8 ALL_INTERACTION_STATE_ALPHAS must enumerate at least one alpha token"
    );
    for (name, value) in ALL_INTERACTION_STATE_ALPHAS {
        assert!(
            (0.0..=1.0).contains(&value),
            "AC8 audit alpha `{name}` resolved to {value}; must be in 0.0..=1.0"
        );
        assert!(
            value.is_finite(),
            "AC8 audit alpha `{name}` resolved to {value}; must be finite"
        );
    }
}

#[test]
fn ac3_ac4_audit_array_pixels_non_negative_and_bounded() {
    // AC8 / AC3 / AC4: every audited pixel must satisfy
    // `0.0 <= px <= 8.0` (the looser of the per-family upper bounds in
    // AC3 / AC4 — FOCUS_RING_WIDTH_PX/OFFSET_PX up to 8.0,
    // PRESSED_OFFSET_Y_PX up to 4.0, all within 8.0).
    assert!(
        !ALL_INTERACTION_STATE_PIXELS.is_empty(),
        "AC8 ALL_INTERACTION_STATE_PIXELS must enumerate at least one pixel token"
    );
    for (name, value) in ALL_INTERACTION_STATE_PIXELS {
        assert!(
            value >= 0.0,
            "AC8 audit pixel `{name}` resolved to {value}; must be >= 0.0"
        );
        assert!(
            value <= 8.0,
            "AC8 audit pixel `{name}` resolved to {value}; outside upper bound 8.0"
        );
        assert!(
            value.is_finite(),
            "AC8 audit pixel `{name}` resolved to {value}; must be finite"
        );
    }
}

#[test]
fn ac3_focus_ring_color_ratifies_spec_section_seven_accent_palette_triple() {
    // AC8 / AC3: FOCUS_RING_COLOR must equal the spec §7 ACCENT triple
    // `Color::srgb(0.949, 0.788, 0.298)` (hex #F2C94C) — not a fresh RGB
    // choice. This is the integration-bin reach-through that pairs with
    // the inline unit test inside the module.
    let expected = Color::srgb(0.949, 0.788, 0.298);
    assert_eq!(
        FOCUS_RING_COLOR, expected,
        "AC3 / AC8 FOCUS_RING_COLOR must equal the spec §7 ACCENT triple \
         Color::srgb(0.949, 0.788, 0.298); not a fresh RGB triple"
    );
}

#[test]
fn ac8_ac9_every_named_constant_carries_at_least_one_doc_comment_line() {
    // AC8 / AC9 doc-comment sanity scan: every `pub const NAME: TYPE =
    // ...;` in the module must be immediately preceded by at least one
    // `///` doc-comment line. Walks the module source line by line so the
    // assertion fires regardless of how `rustfmt` chooses to break the
    // declaration across lines.
    let source = read_module_source();
    let mut violations = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("pub const ") {
            continue;
        }
        // Walk backwards from the declaration line over blank lines and
        // `#[...]` attributes; require the immediately-preceding
        // non-blank, non-attribute line to be a `///` doc comment.
        let mut cursor = idx;
        let has_doc = loop {
            if cursor == 0 {
                break false;
            }
            cursor -= 1;
            let prev = lines[cursor].trim_start();
            if prev.is_empty() || prev.starts_with("#[") {
                continue;
            }
            break prev.starts_with("///");
        };
        if !has_doc {
            violations.push(format!(
                "line {}: `{}` is published without an immediately-preceding `///` doc comment",
                idx + 1,
                line.trim()
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "AC8 / AC9 doc-comment scan: every `pub const` in \
         client/src/ui/design_tokens/interaction_states.rs must carry at least \
         one `///` doc-comment line. Violations:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn ac9_module_publishes_every_required_token_family_prefix() {
    // AC9 (narrow grep): assert the module source declares the four
    // named token-set families. If a future refactor renames a token or
    // drops a family, this grep guard fires loud rather than silently
    // leaving consumer call sites without a source of truth.
    let source = read_module_source();
    for needle in [
        "pub const HOVER_BG_TINT_ALPHA: f32",
        "pub const HOVER_BORDER_ALPHA: f32",
        "pub const FOCUS_RING_COLOR: Color",
        "pub const FOCUS_RING_WIDTH_PX: f32",
        "pub const FOCUS_RING_OFFSET_PX: f32",
        "pub const PRESSED_BG_TINT_ALPHA: f32",
        "pub const PRESSED_OFFSET_Y_PX: f32",
        "pub const DISABLED_BG_TINT_ALPHA: f32",
        "pub const DISABLED_TEXT_ALPHA: f32",
        "pub const DISABLED_BORDER_ALPHA: f32",
    ] {
        assert!(
            source.contains(needle),
            "AC9 module-shape scan: \
             client/src/ui/design_tokens/interaction_states.rs must declare \
             `{needle}` so the four named token-set families remain published"
        );
    }
}

#[test]
fn ac4_ac5_pressed_disabled_visual_state_ordering_holds() {
    // Reach-through duplication of the inline unit-test invariants for
    // defence-in-depth: pressed reads heavier than hover so the player
    // perceives a clear state change between hover-enter and mouse-down;
    // disabled reads heaviest so the disabled state is unambiguously
    // distinguishable from any interactive state. Same invariants as
    // the inline `ac4_pressed_distinct_from_hover_*` /
    // `ac5_disabled_bg_is_heaviest_*` tests.
    assert!(
        PRESSED_BG_TINT_ALPHA > HOVER_BG_TINT_ALPHA,
        "AC4 PRESSED_BG_TINT_ALPHA ({PRESSED_BG_TINT_ALPHA}) must be > \
         HOVER_BG_TINT_ALPHA ({HOVER_BG_TINT_ALPHA}) so pressed reads heavier than hover"
    );
    assert!(
        DISABLED_BG_TINT_ALPHA > PRESSED_BG_TINT_ALPHA,
        "AC5 DISABLED_BG_TINT_ALPHA ({DISABLED_BG_TINT_ALPHA}) must be > \
         PRESSED_BG_TINT_ALPHA ({PRESSED_BG_TINT_ALPHA}) so disabled reads heaviest"
    );
}

#[test]
fn ac7_spec_amendment_anchors_present_in_global_ui_design_spec() {
    // AC7 reach-through: the global UI design spec amendment authored
    // by this story must include the new §11 "Interaction State
    // Primitives" section, must flip the §10 "Primary button affordance"
    // / "Secondary button affordance" deferral notes to forward
    // references to §11, and must amend the Spec Adoption Matrix row
    // for `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` to cite §11 as
    // the source of truth. The friend-game-vs-Standard-tier scope
    // boundary must be preserved verbatim.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // CARGO_MANIFEST_DIR resolves to the `client/` crate; the spec lives
    // at the workspace root `docs/ux/global-ui-design-spec.md`.
    let spec_path = Path::new(manifest_dir)
        .join("..")
        .join("docs")
        .join("ux")
        .join("global-ui-design-spec.md");
    let spec = fs::read_to_string(&spec_path)
        .unwrap_or_else(|err| panic!("failed to read spec at {spec_path:?}: {err}"));

    // §11 heading present.
    assert!(
        spec.contains("## §11 Interaction State Primitives"),
        "AC7 spec must contain the new `## §11 Interaction State Primitives` heading"
    );
    // §11 names the canonical module by file path.
    assert!(
        spec.contains("client/src/ui/design_tokens/interaction_states.rs"),
        "AC7 §11 must cite the new module file path `client/src/ui/design_tokens/interaction_states.rs`"
    );
    // §10 deferral notes flipped to forward references to §11.
    assert!(
        spec.contains("see §11 \"Interaction State"),
        "AC7 §10 button-affordance subsections must forward-reference §11"
    );
    // Spec Adoption Matrix row updated to cite §11.
    assert!(
        spec.contains("§11 interaction state primitives"),
        "AC7 Spec Adoption Matrix row for `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` \
         must cite §11 alongside §7 and §10"
    );
    // Friend-game scope boundary preserved verbatim within §11.
    assert!(
        spec.contains("does **not** advance `QA-COND-0005`"),
        "AC7 §11 must preserve the friend-game scope boundary verbatim — \
         focus-ring visual presence does not advance `QA-COND-0005`"
    );
}
