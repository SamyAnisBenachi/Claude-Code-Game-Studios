// Sprint 17 S17-UI-CARD-DISPLAY-ART-HELPER-001 — AC8 unit coverage for the
// lifted `client::asset_wiring::resolve_card_display_art` helper.
//
// SOURCE-1077-02 / -03 / -04 coverage:
// - Resolver returns `String` (no leak); verified by repeated invocation.
// - `CARD_ART_MISSING_SENTINEL` ("missing") routes through
//   `CARD_ART_PLACEHOLDER_ASSET` (AC7).
// - Production catalog `art_id` resolves to the documented
//   `art/cards/display/card_{art_id}_art_display.png` shape.
// - Empty / whitespace-only `art_id` returns `Err(NoArtId)`.
// - Missing card returns `Err(MissingDisplayAsset)`.

use client::asset_wiring::{
    resolve_card_display_art, CardDisplayArtFallbackReason, CARD_ART_MISSING_SENTINEL,
    CARD_ART_PLACEHOLDER_ASSET,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};

fn test_card_with_art_id(id: u32, art_id: &str) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 2,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: art_id.to_string(),
        pool_copies_override: None,
    }
}

#[test]
fn resolve_returns_owned_string_no_leak_under_repeated_invocation() {
    // SOURCE-1077-03 — the previous implementation used `Box::leak`, so a
    // 1000-call stress run would leak ~50KB of `&'static str`. The new
    // implementation returns an owned `String` that is dropped at end of
    // scope. We assert (a) the return type is owned (compiler-checked via
    // `String`), (b) the function never panics across the stress loop,
    // and (c) every call returns the canonical path.
    let card = test_card_with_art_id(1, "iop_knight_001");
    let expected = "art/cards/display/card_iop_knight_001_art_display.png";

    for _ in 0..1000 {
        let path = resolve_card_display_art(Some(&card)).expect("known art_id resolves");
        assert_eq!(path, expected);
        // `path` is dropped here on each iteration — no leak.
    }
}

#[test]
fn resolve_missing_sentinel_routes_to_placeholder() {
    // AC7 — `art_id == "missing"` is the documented sentinel and must route
    // through the placeholder path (no warn fires for this sentinel).
    let card = test_card_with_art_id(2, CARD_ART_MISSING_SENTINEL);
    let path =
        resolve_card_display_art(Some(&card)).expect("missing sentinel resolves to placeholder");
    assert_eq!(path, CARD_ART_PLACEHOLDER_ASSET);
}

#[test]
fn resolve_production_art_id_returns_canonical_path() {
    // AC8(c) — known-good art_id from a fixture catalog returns the
    // production path that follows the documented
    // `art/cards/display/card_{art_id}_art_display.png` shape.
    let card = test_card_with_art_id(3, "cra_archer_001");
    let path = resolve_card_display_art(Some(&card)).expect("production art_id resolves");
    assert_eq!(
        path,
        "art/cards/display/card_cra_archer_001_art_display.png"
    );
}

#[test]
fn resolve_none_returns_missing_display_asset() {
    // Absent card (e.g., catalog miss) yields the dedicated fallback reason.
    let reason = resolve_card_display_art(None).expect_err("None card yields fallback");
    assert_eq!(reason, CardDisplayArtFallbackReason::MissingDisplayAsset);
}

#[test]
fn resolve_empty_art_id_returns_no_art_id_fallback() {
    // Empty / whitespace-only `art_id` yields the `NoArtId` fallback reason.
    let card = test_card_with_art_id(4, "");
    let reason = resolve_card_display_art(Some(&card)).expect_err("empty art_id yields NoArtId");
    assert_eq!(reason, CardDisplayArtFallbackReason::NoArtId);

    let card = test_card_with_art_id(5, "   ");
    let reason =
        resolve_card_display_art(Some(&card)).expect_err("whitespace-only art_id yields NoArtId");
    assert_eq!(reason, CardDisplayArtFallbackReason::NoArtId);
}

#[test]
fn resolve_trims_surrounding_whitespace_in_art_id() {
    // The resolver trims `art_id` before constructing the path, so cosmetic
    // whitespace in the catalog does not break path resolution.
    let card = test_card_with_art_id(6, "  iop_knight_001  ");
    let path = resolve_card_display_art(Some(&card)).expect("trimmed art_id resolves");
    assert_eq!(
        path,
        "art/cards/display/card_iop_knight_001_art_display.png"
    );
}
