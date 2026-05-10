/// PAW-003 integration test: shop/auction asset wiring — bid button chrome
/// and shop slot well asset constants are present and non-empty.
use client::asset_wiring::{
    bid_button_asset, BidButtonChromeState, BID_BUTTON_DISABLED_ASSET, BID_BUTTON_HOVER_ASSET,
    BID_BUTTON_NORMAL_ASSET, SHOP_PANEL_CHROME_ASSET, SHOP_SLOT_WELL_IDLE_ASSET,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn shop_panel_chrome_asset_is_non_empty() {
    test_helpers::init_test_tracing();
    assert!(!SHOP_PANEL_CHROME_ASSET.is_empty());
}

#[test]
fn shop_slot_well_idle_asset_is_non_empty() {
    test_helpers::init_test_tracing();
    assert!(!SHOP_SLOT_WELL_IDLE_ASSET.is_empty());
}

#[test]
fn bid_button_asset_normal() {
    test_helpers::init_test_tracing();
    assert_eq!(
        bid_button_asset(BidButtonChromeState::Normal),
        BID_BUTTON_NORMAL_ASSET
    );
}

#[test]
fn bid_button_asset_hover() {
    test_helpers::init_test_tracing();
    assert_eq!(
        bid_button_asset(BidButtonChromeState::Hover),
        BID_BUTTON_HOVER_ASSET
    );
}

#[test]
fn bid_button_asset_disabled() {
    test_helpers::init_test_tracing();
    assert_eq!(
        bid_button_asset(BidButtonChromeState::Disabled),
        BID_BUTTON_DISABLED_ASSET
    );
}
