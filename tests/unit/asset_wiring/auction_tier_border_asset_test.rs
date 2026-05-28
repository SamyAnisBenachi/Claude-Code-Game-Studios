// PROMPT 1853 SLICE-B — unit coverage for `auction_tier_border_asset` selector
// and path allowlist for all tier-border + gem constants.
//
// Verified:
// - Each tier (1–4) resolves to the expected disk path.
// - Out-of-range tier values clamp to tier 1.
// - All 10 auction-scope constants (4 borders + 6 gems) point under
//   `art/ui/shop_auction/`.

use client::asset_wiring::{
    auction_tier_border_asset, AUCTION_GEM_EPIC_24_ASSET, AUCTION_GEM_EPIC_32_ASSET,
    AUCTION_GEM_LEGENDARY_24_ASSET, AUCTION_GEM_LEGENDARY_32_ASSET, AUCTION_GEM_RARE_24_ASSET,
    AUCTION_GEM_RARE_32_ASSET, AUCTION_TIER_BORDER_1_ASSET, AUCTION_TIER_BORDER_2_ASSET,
    AUCTION_TIER_BORDER_3_ASSET, AUCTION_TIER_BORDER_4_ASSET,
};

#[test]
fn test_auction_tier_border_asset_tier1_returns_tier1_path() {
    assert_eq!(
        auction_tier_border_asset(1),
        "art/ui/shop_auction/ui_auction_border_tier1_hud.png"
    );
}

#[test]
fn test_auction_tier_border_asset_tier2_returns_tier2_path() {
    assert_eq!(
        auction_tier_border_asset(2),
        "art/ui/shop_auction/ui_auction_border_tier2_hud.png"
    );
}

#[test]
fn test_auction_tier_border_asset_tier3_returns_tier3_path() {
    assert_eq!(
        auction_tier_border_asset(3),
        "art/ui/shop_auction/ui_auction_border_tier3_hud.png"
    );
}

#[test]
fn test_auction_tier_border_asset_tier4_returns_tier4_path() {
    assert_eq!(
        auction_tier_border_asset(4),
        "art/ui/shop_auction/ui_auction_border_tier4_hud.png"
    );
}

#[test]
fn test_auction_tier_border_asset_out_of_range_clamps_to_tier1() {
    // Tier 0 and tier 5+ both clamp to tier 1.
    assert_eq!(auction_tier_border_asset(0), auction_tier_border_asset(1));
    assert_eq!(auction_tier_border_asset(5), auction_tier_border_asset(1));
    assert_eq!(auction_tier_border_asset(255), auction_tier_border_asset(1));
}

#[test]
fn test_auction_tier_border_constants_all_under_shop_auction_prefix() {
    let tier_border_paths = [
        AUCTION_TIER_BORDER_1_ASSET,
        AUCTION_TIER_BORDER_2_ASSET,
        AUCTION_TIER_BORDER_3_ASSET,
        AUCTION_TIER_BORDER_4_ASSET,
    ];
    for path in &tier_border_paths {
        assert!(
            path.starts_with("art/ui/shop_auction/"),
            "tier-border path not under art/ui/shop_auction/: {path}"
        );
    }
}

#[test]
fn test_auction_gem_constants_all_under_shop_auction_prefix() {
    let gem_paths = [
        AUCTION_GEM_RARE_24_ASSET,
        AUCTION_GEM_RARE_32_ASSET,
        AUCTION_GEM_EPIC_24_ASSET,
        AUCTION_GEM_EPIC_32_ASSET,
        AUCTION_GEM_LEGENDARY_24_ASSET,
        AUCTION_GEM_LEGENDARY_32_ASSET,
    ];
    for path in &gem_paths {
        assert!(
            path.starts_with("art/ui/shop_auction/"),
            "gem path not under art/ui/shop_auction/: {path}"
        );
    }
}

#[test]
fn test_auction_tier_border_selector_covers_all_four_tiers_distinct() {
    let paths: Vec<_> = (1u8..=4).map(auction_tier_border_asset).collect();
    // All four paths must be distinct — each tier has its own texture.
    let unique: std::collections::HashSet<_> = paths.iter().collect();
    assert_eq!(unique.len(), 4, "tier-border paths must all be distinct");
}
