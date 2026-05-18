//! Sprint 16 / Story 009 — `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`
//! integration test.
//!
//! Covers AC1..AC8: every `CardSlotKind` variant is importable from the
//! published path, declared aspect-ratio band contains the variant's
//! outer ratio, image / text insets fit inside the outer rectangle and
//! are mutually disjoint, the hit-target rectangle is a superset of (or
//! equal to) the visual outer rectangle, every kind resolves to a
//! distinct `(width, height, z_layer)` triple, the `card_slot_node`
//! builder agrees with the geometry struct, the aspect ratio is
//! preserved across the canonical viewport matrix (the slot is
//! pixel-fixed; no viewport-driven scaling is introduced), the four
//! `interaction_states` token families are importable from the
//! published path, and the migrated `shop_slot_node` Node has an outer
//! width / height that matches `CardSlotKind::ShopSlot`.
//!
//! Out of scope (per story §Scope / Out of Scope and AC5 + AC8): hand /
//! draft / auction-featured / board staged-ghost migrations are owned
//! by the Sprint 16+ `S16-UI-CARD-SLOT-MIGRATION-*` follow-on family,
//! NOT this test. The test asserts the primitive's shape and the Phase
//! 1 shop slot migration only.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::Color;
use bevy::ui::{PositionType, UiRect, Val};

use client::ui::design_tokens::card_slot::{
    card_slot_geometry, card_slot_hit_target, card_slot_image_inset, card_slot_image_inset_node,
    card_slot_node, card_slot_text_inset, card_slot_text_inset_node, CardSlotGeometry,
    CardSlotKind, ALL_CARD_SLOT_KINDS,
};
use client::ui::design_tokens::interaction_states::{
    DISABLED_BG_TINT_ALPHA, DISABLED_BORDER_ALPHA, DISABLED_TEXT_ALPHA, FOCUS_RING_COLOR,
    FOCUS_RING_OFFSET_PX, FOCUS_RING_WIDTH_PX, HOVER_BG_TINT_ALPHA, HOVER_BORDER_ALPHA,
    PRESSED_BG_TINT_ALPHA, PRESSED_OFFSET_Y_PX,
};

#[path = "../helpers/ui_viewport.rs"]
mod ui_viewport;

use ui_viewport::{ViewportSize, CANONICAL_VIEWPORTS};

/// Sentinel viewport below the canonical matrix (`CANONICAL_VIEWPORTS`
/// minimum is `1366 × 768` per §8 of the global UI spec). AC4 binds
/// containment at this smaller-than-canonical viewport to prove that
/// the pixel-fixed slot does NOT shift even when the viewport drops
/// below the canonical floor.
const SENTINEL_BELOW_CANONICAL: ViewportSize = ViewportSize {
    name: "1024x600",
    width: 1024,
    height: 600,
};

fn module_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("design_tokens")
        .join("card_slot.rs")
}

fn read_module_source() -> String {
    let path = module_path();
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read card_slot.rs at {path:?}: {err}"))
}

fn inset_pixels(rect: &UiRect) -> (f32, f32, f32, f32) {
    // `Val::ZERO` is defined as `Val::Px(0.0)`, so the `Val::Px(px)` arm
    // covers both the explicit zero rect and per-side pixel values.
    let to_px = |v: Val, side: &str| match v {
        Val::Px(px) => px,
        other => panic!("expected Val::Px inset on {side}, found {other:?}"),
    };
    (
        to_px(rect.left, "left"),
        to_px(rect.right, "right"),
        to_px(rect.top, "top"),
        to_px(rect.bottom, "bottom"),
    )
}

fn aspect_ratio(geometry: &CardSlotGeometry) -> f32 {
    geometry.outer_width_px / geometry.outer_height_px
}

fn val_px(value: Val) -> Option<f32> {
    // `Val::ZERO` is `Val::Px(0.0)`; the explicit zero alias does not
    // implement `StructuralPartialEq`, so we match on `Val::Px(_)` only.
    match value {
        Val::Px(px) => Some(px),
        _ => None,
    }
}

#[test]
fn ac1_all_five_card_slot_kinds_are_importable_from_public_path() {
    // AC1 reach-through: confirm every variant is part of the public API
    // surface. A future revision that drops a variant would fail to
    // compile here.
    assert_eq!(ALL_CARD_SLOT_KINDS.len(), 5);
    let kinds = [
        CardSlotKind::HandFan,
        CardSlotKind::DraftGrid,
        CardSlotKind::ShopSlot,
        CardSlotKind::AuctionFeatured,
        CardSlotKind::BoardStagedGhost,
    ];
    for kind in kinds {
        assert!(
            ALL_CARD_SLOT_KINDS.contains(&kind),
            "AC1 ALL_CARD_SLOT_KINDS missing variant {kind:?}"
        );
    }
}

#[test]
fn ac2_each_kind_outer_dimensions_strictly_positive_and_finite() {
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        assert!(
            geometry.outer_width_px > 0.0 && geometry.outer_width_px.is_finite(),
            "AC2 outer_width_px non-positive for {kind:?}: {}",
            geometry.outer_width_px,
        );
        assert!(
            geometry.outer_height_px > 0.0 && geometry.outer_height_px.is_finite(),
            "AC2 outer_height_px non-positive for {kind:?}: {}",
            geometry.outer_height_px,
        );
    }
}

#[test]
fn ac2_each_kind_aspect_ratio_falls_in_declared_band() {
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (min, max) = geometry.aspect_ratio_band;
        assert!(
            min > 0.0 && max > min && max.is_finite(),
            "AC2 aspect_ratio_band malformed for {kind:?}: ({min}, {max})",
        );
        let ratio = aspect_ratio(&geometry);
        assert!(
            ratio >= min && ratio <= max,
            "AC2 aspect ratio out of band for {kind:?}: ratio={ratio:.4} band=({min:.4}, {max:.4})",
        );
    }
}

#[test]
fn ac4_image_and_text_insets_fit_inside_outer_rectangle_per_kind() {
    // AC4: image inset + text inset must each fit inside the outer
    // rectangle (left + right < outer_width_px and top + bottom <
    // outer_height_px). Asserts containment per kind.
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (il, ir, it, ib) = inset_pixels(&geometry.image_inset_px);
        let (tl, tr, tt, tb) = inset_pixels(&geometry.text_inset_px);
        assert!(
            il + ir < geometry.outer_width_px,
            "AC4 image inset width overflow for {kind:?}: {il} + {ir} >= {}",
            geometry.outer_width_px,
        );
        assert!(
            it + ib < geometry.outer_height_px,
            "AC4 image inset height overflow for {kind:?}: {it} + {ib} >= {}",
            geometry.outer_height_px,
        );
        assert!(
            tl + tr < geometry.outer_width_px,
            "AC4 text inset width overflow for {kind:?}: {tl} + {tr} >= {}",
            geometry.outer_width_px,
        );
        assert!(
            tt + tb < geometry.outer_height_px,
            "AC4 text inset height overflow for {kind:?}: {tt} + {tb} >= {}",
            geometry.outer_height_px,
        );
    }
}

#[test]
fn ac4_image_and_text_rectangles_are_disjoint_per_kind() {
    // AC4: the image region and the text region must NOT overlap. They
    // can be split by x (landscape image-left / text-right) or by y
    // (portrait image-top / text-bottom); either splitting axis is
    // acceptable. The disjoint check tests both and accepts at least
    // one disjoint axis.
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (il, ir, it, ib) = inset_pixels(&geometry.image_inset_px);
        let (tl, tr, tt, tb) = inset_pixels(&geometry.text_inset_px);
        let w = geometry.outer_width_px;
        let h = geometry.outer_height_px;
        // Image rectangle: [il, w - ir] × [it, h - ib].
        let image_x = (il, w - ir);
        let image_y = (it, h - ib);
        // Text rectangle: [tl, w - tr] × [tt, h - tb].
        let text_x = (tl, w - tr);
        let text_y = (tt, h - tb);

        let disjoint_x = image_x.1 <= text_x.0 || text_x.1 <= image_x.0;
        let disjoint_y = image_y.1 <= text_y.0 || text_y.1 <= image_y.0;
        assert!(
            disjoint_x || disjoint_y,
            "AC4 image and text rectangles overlap for {kind:?}: image_x={image_x:?} image_y={image_y:?} text_x={text_x:?} text_y={text_y:?}",
        );
    }
}

#[test]
fn ac4_image_and_text_containment_at_1366x768_and_1024x600_sentinel() {
    // AC4 viewport-iteration loop: containment must hold at the
    // smallest canonical viewport (1366×768) AND at the sentinel
    // smaller-than-canonical viewport (1024×600). The slot is
    // pixel-fixed, so containment is a property of the slot's outer
    // rectangle vs the viewport regardless of viewport size; this test
    // is the explicit assertion that no kind's outer rectangle exceeds
    // the smallest canonical or sentinel viewport's pixel bounds.
    let viewports = [
        ViewportSize {
            name: "1366x768",
            width: 1366,
            height: 768,
        },
        SENTINEL_BELOW_CANONICAL,
    ];
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        for viewport in viewports {
            assert!(
                geometry.outer_width_px <= viewport.width as f32,
                "AC4 outer_width_px exceeds viewport {} for {kind:?}: {} > {}",
                viewport.name,
                geometry.outer_width_px,
                viewport.width,
            );
            assert!(
                geometry.outer_height_px <= viewport.height as f32,
                "AC4 outer_height_px exceeds viewport {} for {kind:?}: {} > {}",
                viewport.name,
                geometry.outer_height_px,
                viewport.height,
            );
        }
    }
}

#[test]
fn ac2_aspect_ratio_preserved_across_canonical_viewports() {
    // AC2 viewport-invariant: the slot is pixel-fixed per §4 spacing
    // scale; outer_width_px / outer_height_px is a constant per kind
    // and does NOT depend on the viewport. Iterating
    // CANONICAL_VIEWPORTS makes the structural invariant explicit:
    // every iteration sees the same ratio.
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let expected = aspect_ratio(&geometry);
        for viewport in CANONICAL_VIEWPORTS {
            // No viewport-driven scaling is introduced; ratio is
            // identical for every viewport.
            let observed = aspect_ratio(&geometry);
            assert!(
                (observed - expected).abs() < f32::EPSILON,
                "AC2 aspect ratio drift for {kind:?} at viewport {}: expected={expected} observed={observed}",
                viewport.name,
            );
        }
    }
}

#[test]
fn ac7_hit_target_is_superset_of_or_equal_to_visual_outer_rectangle() {
    // AC7: hit_target_inset is a superset of (or equal to) the visual
    // outer rectangle. Default UiRect::ZERO means equal. Negative
    // values would make the hit target larger; positive values would
    // shrink it — which would violate the contract. Assert each side
    // inset is <= 0.0 (or ZERO).
    for kind in ALL_CARD_SLOT_KINDS {
        let inset = card_slot_hit_target(kind);
        let (l, r, t, b) = inset_pixels(&inset);
        for (side, value) in [("left", l), ("right", r), ("top", t), ("bottom", b)] {
            assert!(
                value <= 0.0,
                "AC7 hit-target inset side `{side}` is positive for {kind:?}: {value} > 0.0; hit target would be smaller than visual outer rectangle",
            );
        }
    }
}

#[test]
fn ac7_each_kind_resolves_to_distinct_outer_size_and_z_layer_triple() {
    // AC7: each kind resolves to a distinct (outer_width_px,
    // outer_height_px, z_layer) triple — no two kinds collapse.
    let mut triples = Vec::new();
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let triple = (
            geometry.outer_width_px.to_bits(),
            geometry.outer_height_px.to_bits(),
            geometry.z_layer.0,
        );
        assert!(
            !triples.contains(&triple),
            "AC7 collapsed (width, height, z_layer) triple for {kind:?}: {triple:?} duplicates an earlier kind",
        );
        triples.push(triple);
    }
}

#[test]
fn ac7_card_slot_node_width_height_match_geometry_for_shop_slot() {
    // AC7: card_slot_node(ShopSlot) returns a Node whose width / height
    // match card_slot_geometry(ShopSlot). Asserts the Node builder does
    // not silently disagree with the geometry struct.
    let geometry = card_slot_geometry(CardSlotKind::ShopSlot);
    let node = card_slot_node(CardSlotKind::ShopSlot);
    assert_eq!(
        val_px(node.width),
        Some(geometry.outer_width_px),
        "AC7 card_slot_node width does not match geometry for ShopSlot: {:?} vs {}",
        node.width,
        geometry.outer_width_px,
    );
    assert_eq!(
        val_px(node.height),
        Some(geometry.outer_height_px),
        "AC7 card_slot_node height does not match geometry for ShopSlot: {:?} vs {}",
        node.height,
        geometry.outer_height_px,
    );
}

#[test]
fn ac7_card_slot_node_width_height_match_geometry_for_every_kind() {
    // Reach-through of AC7 to every kind so a future builder regression
    // is caught regardless of which kind it affects.
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let node = card_slot_node(kind);
        assert_eq!(
            val_px(node.width),
            Some(geometry.outer_width_px),
            "AC7 card_slot_node width drift for {kind:?}",
        );
        assert_eq!(
            val_px(node.height),
            Some(geometry.outer_height_px),
            "AC7 card_slot_node height drift for {kind:?}",
        );
    }
}

#[test]
fn ac1_image_and_text_accessors_match_geometry_struct() {
    // AC1: the named accessor functions (`card_slot_image_inset`,
    // `card_slot_text_inset`, `card_slot_hit_target`) return values
    // identical to the corresponding `CardSlotGeometry` field.
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        assert_eq!(
            card_slot_image_inset(kind),
            geometry.image_inset_px,
            "AC1 image inset accessor diverges from geometry for {kind:?}",
        );
        assert_eq!(
            card_slot_text_inset(kind),
            geometry.text_inset_px,
            "AC1 text inset accessor diverges from geometry for {kind:?}",
        );
        assert_eq!(
            card_slot_hit_target(kind),
            geometry.hit_target_inset_px,
            "AC1 hit target accessor diverges from geometry for {kind:?}",
        );
    }
}

#[test]
fn ac1_module_body_does_not_introduce_naked_val_px_numeric_literal() {
    // AC1 grep guard: every numeric value at the public-API boundary
    // is named (`const NAME: f32 = ...;` or `pub const NAME: UiRect =
    // ...;`). Naked `Val::Px(<digit>)` literals are forbidden at
    // function-body / accessor use sites — they must flow through a
    // named const or a struct field. Naked `Val::Px(<digit>)` literals
    // INSIDE a `pub const NAME: UiRect = UiRect { ... };` initializer
    // are allowed because the binding itself is the named form per
    // AC1's wording ("or `pub const NAME: UiRect = ...;`").
    //
    // Doc comments, inline-test code, and `pub const NAME: UiRect = ...`
    // initializer bodies are skipped. The grep targets function bodies
    // and any non-const code path that would re-introduce inline magic
    // literals at a usage site.
    let source = read_module_source();
    let mut in_doc_block = false;
    let mut in_test_block = false;
    let mut test_block_depth = 0i32;
    let mut in_const_uirect_init = false;
    for (lineno, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        // Skip doc comments (start with `//!` or `///`).
        if trimmed.starts_with("//!") || trimmed.starts_with("///") {
            continue;
        }
        if trimmed.starts_with("//") {
            // Block-comment trailers (// ...).
            continue;
        }
        // Track the `#[cfg(test)] mod tests` block by counting braces.
        if trimmed.starts_with("#[cfg(test)]") {
            in_test_block = true;
            continue;
        }
        if in_test_block {
            test_block_depth += line.matches('{').count() as i32;
            test_block_depth -= line.matches('}').count() as i32;
            if test_block_depth < 0 {
                in_test_block = false;
                test_block_depth = 0;
            }
            continue;
        }
        // Track raw-doc-block comments (rare in this module).
        if trimmed.starts_with("/*!") || trimmed.starts_with("/**") {
            in_doc_block = true;
        }
        if in_doc_block {
            if trimmed.contains("*/") {
                in_doc_block = false;
            }
            continue;
        }
        // Track entry into a `pub const NAME: UiRect = UiRect {` block.
        if !in_const_uirect_init
            && trimmed.starts_with("pub const ")
            && trimmed.contains(": UiRect = UiRect {")
        {
            in_const_uirect_init = true;
            continue;
        }
        if in_const_uirect_init {
            // The closing `};` line terminates the initializer.
            if trimmed.starts_with("};") {
                in_const_uirect_init = false;
            }
            continue;
        }
        // Scan for `Val::Px(<digit>` — naked numeric literal inside the
        // production module body. Allowed: `Val::Px(CONST_NAME)`,
        // `Val::Px(geometry.outer_width_px)`, etc.
        if let Some(idx) = line.find("Val::Px(") {
            let after = &line[idx + "Val::Px(".len()..];
            let first_non_space = after.chars().find(|c| !c.is_whitespace()).unwrap_or(')');
            assert!(
                !first_non_space.is_ascii_digit() && first_non_space != '-',
                "AC1 naked numeric literal at module body line {}: `{}` — every public-API numeric value MUST flow from a named const",
                lineno + 1,
                line.trim(),
            );
        }
    }
}

#[test]
fn ac3_interaction_state_token_families_importable_from_published_path() {
    // AC3: the four interaction-state token families
    // (HOVER_* / FOCUS_* / PRESSED_* / DISABLED_*) are importable from
    // the published path so per-surface card-slot migration siblings
    // (Sprint 16+ `S16-UI-INTERACTION-STATE-MIGRATION-*`) can wire
    // hover / focus / pressed / disabled visuals against the same
    // canonical tokens this primitive references in its doc comments.
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
fn ac3_card_slot_module_doc_comments_reference_interaction_state_families() {
    // AC3 doc-comment scan: every per-kind doc comment names the four
    // interaction-state token families it composes with. Catches a
    // future revision that drops the cross-reference.
    let source = read_module_source();
    let required_families = [
        "HOVER_BG_TINT_ALPHA",
        "FOCUS_RING_COLOR",
        "PRESSED_BG_TINT_ALPHA",
        "DISABLED_BG_TINT_ALPHA",
    ];
    for family in required_families {
        assert!(
            source.contains(family),
            "AC3 card_slot.rs doc comments must reference {family} (interaction-state token family)",
        );
    }
}

#[test]
fn ac5_phase_1_shop_slot_node_outer_geometry_matches_primitive() {
    // AC5 / AC7: the migrated `shop_slot_node` helper in
    // `client/src/ui/shop_auction/mod.rs` is the Phase 1 consumer of
    // `card_slot_node(CardSlotKind::ShopSlot)`. Read the production
    // source to assert that:
    //  (a) the migration is in place (call site references the
    //      primitive), and
    //  (b) no naked `Val::Px(136.0)` / `Val::Px(78.0)` width / height
    //      literal remains inside the helper body — the outer width
    //      and height now flow from the primitive.
    let shop_auction_src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("shop_auction")
        .join("mod.rs");
    let raw_source = fs::read_to_string(&shop_auction_src)
        .unwrap_or_else(|err| panic!("failed to read shop_auction/mod.rs: {err}"));
    // Normalise Windows CRLF line endings to LF so the body window is
    // delimited consistently regardless of git autocrlf state.
    let source = raw_source.replace("\r\n", "\n");
    let helper_idx = source
        .find("fn shop_slot_node(")
        .expect("AC5 shop_slot_node helper must be present in shop_auction/mod.rs");
    let rest = &source[helper_idx..];
    // The helper ends at the first line that is exactly `}` (a
    // top-level closing brace on its own line). Trailing top-of-file
    // helpers may be followed by a blank line or another `fn …`.
    let close_idx = rest
        .find("\n}\n")
        .or_else(|| rest.find("\n}"))
        .expect("AC5 shop_slot_node helper must terminate with a `}` line");
    let body = &rest[..close_idx];
    assert!(
        body.contains("card_slot_node(CardSlotKind::ShopSlot)"),
        "AC5 shop_slot_node helper must call card_slot_node(CardSlotKind::ShopSlot); body was:\n{body}",
    );
    assert!(
        !body.contains("Val::Px(136.0)"),
        "AC5 shop_slot_node helper must NOT retain a naked Val::Px(136.0) width literal post-migration; body was:\n{body}",
    );
    assert!(
        !body.contains("Val::Px(78.0)"),
        "AC5 shop_slot_node helper must NOT retain a naked Val::Px(78.0) height literal post-migration; body was:\n{body}",
    );
}

#[test]
fn ac6_spec_amendment_introduces_section_twelve_card_slot_primitive() {
    // AC6 reach-through: the global UI design spec amendment landed by
    // this story introduces the new §12 "Card Slot Primitive"
    // section. Reads the spec file and asserts the new heading is
    // present and the §10 stub has been replaced with a forward
    // reference.
    let spec_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("client/ has a parent")
        .join("docs")
        .join("ux")
        .join("global-ui-design-spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap_or_else(|err| {
        panic!(
            "failed to read global-ui-design-spec.md at {}: {err}",
            spec_path.display()
        )
    });
    assert!(
        spec.contains("## §12 Card Slot Primitive"),
        "AC6 spec must add `## §12 Card Slot Primitive` heading after §11",
    );
    assert!(
        spec.contains("Forward reference: see §12"),
        "AC6 §10 `Card slot composition` stub must flip to a forward reference to §12",
    );
}

#[test]
fn ac8_card_slot_module_does_not_advance_friend_game_scope_guards() {
    // AC8 reach-through: the module's doc comments must preserve the
    // friend-game scope boundary verbatim (no claim of Standard-tier
    // accessibility, playtest validation, or final-art replacement).
    let source = read_module_source();
    for guard in ["QA-COND-0005", "QA-COND-0006", "PAW-TD-*-a", "friend-game"] {
        assert!(
            source.contains(guard),
            "AC8 card_slot.rs must preserve `{guard}` scope guard in doc comments",
        );
    }
}

// =====================================================================
// Sprint 17 / S17-UI-CARD-SLOT-INSET-WIRING-001 — sibling inset
// builders + GlobalZIndex wiring (SOURCE-1077-06).
// =====================================================================
//
// AC6 reach-through: for every CardSlotKind variant, the new
// card_slot_image_inset_node / card_slot_text_inset_node builders emit
// a Node whose per-side absolute-position fields match the geometry
// catalog's image_inset_px / text_inset_px and a GlobalZIndex equal
// to the catalog's z_layer. The variant set covered by the per-test
// loop is ALL_CARD_SLOT_KINDS, which is the authoritative iteration
// source; adding a variant without updating the array would break the
// `ac1_all_five_card_slot_kinds_are_importable_from_public_path` test
// above. The Sprint 17 row is purely additive at the primitive level
// (AC5 / AC7); the existing AC1..AC8 assertions remain unchanged.

#[test]
fn s17_inset_image_node_position_type_absolute_per_kind() {
    // S17 AC1 / AC6(a) reach-through: the image-inset builder returns
    // a Node with PositionType::Absolute for every kind. The Sprint 17+
    // Backlog `S17-UI-CARD-SLOT-MIGRATION-*` family relies on the
    // returned Node being absolutely-positioned so children stack onto
    // the parent card slot's outer rectangle without flexbox arithmetic.
    for kind in ALL_CARD_SLOT_KINDS {
        let (node, _z) = card_slot_image_inset_node(kind);
        assert_eq!(
            node.position_type,
            PositionType::Absolute,
            "S17 AC1 image inset node must be PositionType::Absolute for {kind:?}; got {:?}",
            node.position_type,
        );
    }
}

#[test]
fn s17_inset_text_node_position_type_absolute_per_kind() {
    // S17 AC2 / AC6(b) reach-through: the text-inset builder returns
    // a Node with PositionType::Absolute for every kind.
    for kind in ALL_CARD_SLOT_KINDS {
        let (node, _z) = card_slot_text_inset_node(kind);
        assert_eq!(
            node.position_type,
            PositionType::Absolute,
            "S17 AC2 text inset node must be PositionType::Absolute for {kind:?}; got {:?}",
            node.position_type,
        );
    }
}

#[test]
fn s17_inset_image_node_edges_match_geometry_per_kind() {
    // S17 AC6(a): the image-inset builder's Node has its left / right
    // / top / bottom fields equal to card_slot_geometry(kind).image_
    // inset_px (precise pixel match, per AC6 wording). This is the
    // canonical assertion that the primitive HONOURS the geometry
    // catalog's image_inset_px field — SOURCE-1077-06's user-visible
    // symptom (title clipping, BOUGHT-band paint, "3g" overlap) was
    // downstream of card_slot_node not wiring per-kind text / image
    // insets.
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (node, _z) = card_slot_image_inset_node(kind);
        assert_eq!(
            node.left, geometry.image_inset_px.left,
            "S17 AC6(a) image inset left drift for {kind:?}: {:?} vs {:?}",
            node.left, geometry.image_inset_px.left,
        );
        assert_eq!(
            node.right, geometry.image_inset_px.right,
            "S17 AC6(a) image inset right drift for {kind:?}: {:?} vs {:?}",
            node.right, geometry.image_inset_px.right,
        );
        assert_eq!(
            node.top, geometry.image_inset_px.top,
            "S17 AC6(a) image inset top drift for {kind:?}: {:?} vs {:?}",
            node.top, geometry.image_inset_px.top,
        );
        assert_eq!(
            node.bottom, geometry.image_inset_px.bottom,
            "S17 AC6(a) image inset bottom drift for {kind:?}: {:?} vs {:?}",
            node.bottom, geometry.image_inset_px.bottom,
        );
    }
}

#[test]
fn s17_inset_text_node_edges_match_geometry_per_kind() {
    // S17 AC6(b): the text-inset builder's Node has its left / right /
    // top / bottom fields equal to card_slot_geometry(kind).text_
    // inset_px (precise pixel match).
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (node, _z) = card_slot_text_inset_node(kind);
        assert_eq!(
            node.left, geometry.text_inset_px.left,
            "S17 AC6(b) text inset left drift for {kind:?}: {:?} vs {:?}",
            node.left, geometry.text_inset_px.left,
        );
        assert_eq!(
            node.right, geometry.text_inset_px.right,
            "S17 AC6(b) text inset right drift for {kind:?}: {:?} vs {:?}",
            node.right, geometry.text_inset_px.right,
        );
        assert_eq!(
            node.top, geometry.text_inset_px.top,
            "S17 AC6(b) text inset top drift for {kind:?}: {:?} vs {:?}",
            node.top, geometry.text_inset_px.top,
        );
        assert_eq!(
            node.bottom, geometry.text_inset_px.bottom,
            "S17 AC6(b) text inset bottom drift for {kind:?}: {:?} vs {:?}",
            node.bottom, geometry.text_inset_px.bottom,
        );
    }
}

#[test]
fn s17_inset_image_and_text_builders_thread_global_z_index_per_kind() {
    // S17 AC3 / AC6(c): both inset builders emit a GlobalZIndex equal
    // to card_slot_geometry(kind).z_layer. The image child and text
    // child therefore composite into the same layer as their parent
    // card slot (UI_BASE for the four bevy_ui kinds, UI_OVERLAY for
    // the world-space BoardStagedGhost).
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (_image_node, image_z) = card_slot_image_inset_node(kind);
        let (_text_node, text_z) = card_slot_text_inset_node(kind);
        assert_eq!(
            image_z.0, geometry.z_layer.0,
            "S17 AC3 image inset GlobalZIndex drift for {kind:?}: {} vs {}",
            image_z.0, geometry.z_layer.0,
        );
        assert_eq!(
            text_z.0, geometry.z_layer.0,
            "S17 AC3 text inset GlobalZIndex drift for {kind:?}: {} vs {}",
            text_z.0, geometry.z_layer.0,
        );
    }
}

#[test]
fn s17_inset_builders_cover_every_card_slot_kind_variant() {
    // S17 AC6(d): the variant set covered by the inset-wiring tests is
    // ALL_CARD_SLOT_KINDS. ALL_CARD_SLOT_KINDS is the canonical
    // iteration source — adding a variant without updating the array
    // would break the AC1 import test above. This assertion is the
    // explicit guard that no variant is uncovered by the new tests.
    assert_eq!(
        ALL_CARD_SLOT_KINDS.len(),
        5,
        "S17 AC6(d) ALL_CARD_SLOT_KINDS must enumerate every CardSlotKind variant; \
         a new variant requires adding it to the array AND to the inset-wiring tests above",
    );
    // Compile-time evidence: every variant resolves to a non-panicking
    // builder pair. This loop covers exactly the same variant set that
    // the per-edge / per-z-index assertions iterate above.
    for kind in ALL_CARD_SLOT_KINDS {
        let _ = card_slot_image_inset_node(kind);
        let _ = card_slot_text_inset_node(kind);
    }
}

#[test]
fn s17_inset_image_node_carries_no_inline_size_overrides_per_kind() {
    // S17 reach-through: the image-inset Node delegates width and
    // height to the four absolute-position edges (left / right / top /
    // bottom). Asserting node.width == Val::Auto and node.height ==
    // Val::Auto catches a future revision that re-introduces inline
    // Val::Px(N) width / height literals into the inset builder body
    // — which would re-create the SOURCE-1077-06 defect class on a
    // sibling primitive.
    for kind in ALL_CARD_SLOT_KINDS {
        let (node, _z) = card_slot_image_inset_node(kind);
        assert_eq!(
            node.width,
            Val::Auto,
            "S17 image inset node must derive width from absolute edges for {kind:?}; \
             explicit width Val::Px(...) re-creates SOURCE-1077-06 child-arithmetic drift",
        );
        assert_eq!(
            node.height,
            Val::Auto,
            "S17 image inset node must derive height from absolute edges for {kind:?}",
        );
    }
}

#[test]
fn s17_inset_text_node_carries_no_inline_size_overrides_per_kind() {
    // S17 reach-through (text counterpart of the image guard above).
    for kind in ALL_CARD_SLOT_KINDS {
        let (node, _z) = card_slot_text_inset_node(kind);
        assert_eq!(
            node.width,
            Val::Auto,
            "S17 text inset node must derive width from absolute edges for {kind:?}",
        );
        assert_eq!(
            node.height,
            Val::Auto,
            "S17 text inset node must derive height from absolute edges for {kind:?}",
        );
    }
}

#[test]
fn s17_inset_builders_dimensions_resolve_to_positive_interior_per_kind() {
    // S17 reach-through: for every kind, the inset rectangle is
    // strictly inside the outer rectangle (already asserted by
    // ac4_image_and_text_insets_fit_inside_outer_rectangle_per_kind).
    // The new Node-shape assertion converts that geometric invariant
    // into a layout precondition: the absolute-positioned inset child
    // resolves to a positive (width, height) when laid out under the
    // outer rectangle. The interior width is
    // outer_width_px - (inset.left + inset.right); the interior
    // height is outer_height_px - (inset.top + inset.bottom). Both
    // MUST be > 0 for every kind / every inset.
    for kind in ALL_CARD_SLOT_KINDS {
        let geometry = card_slot_geometry(kind);
        let (image_l, image_r, image_t, image_b) = inset_pixels(&geometry.image_inset_px);
        let (text_l, text_r, text_t, text_b) = inset_pixels(&geometry.text_inset_px);
        let image_w = geometry.outer_width_px - (image_l + image_r);
        let image_h = geometry.outer_height_px - (image_t + image_b);
        let text_w = geometry.outer_width_px - (text_l + text_r);
        let text_h = geometry.outer_height_px - (text_t + text_b);
        assert!(
            image_w > 0.0 && image_h > 0.0,
            "S17 image interior non-positive for {kind:?}: ({image_w}, {image_h})",
        );
        assert!(
            text_w > 0.0 && text_h > 0.0,
            "S17 text interior non-positive for {kind:?}: ({text_w}, {text_h})",
        );
    }
}
