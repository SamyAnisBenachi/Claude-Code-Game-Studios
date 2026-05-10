/// PAW-005 integration test: board rendering asset wiring — unit sprite asset
/// constants are present and map correctly for all classes.
use client::asset_wiring::{
    board_unit_asset, BOARD_CHROME_ASSET, BOARD_UNIT_CRA_ASSET, BOARD_UNIT_ECAFLIP_ASSET,
    BOARD_UNIT_IOP_ASSET, BOARD_UNIT_NEUTRAL_ASSET, BOARD_UNIT_SACRIER_ASSET,
    BOARD_UNIT_SADIDA_ASSET, BOARD_UNIT_XELOR_ASSET,
};
use shared::card::ClassId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn board_chrome_asset_is_non_empty() {
    test_helpers::init_test_tracing();
    assert!(!BOARD_CHROME_ASSET.is_empty());
}

#[test]
fn board_unit_asset_maps_all_classes() {
    test_helpers::init_test_tracing();
    assert_eq!(board_unit_asset(ClassId::Iop), BOARD_UNIT_IOP_ASSET);
    assert_eq!(board_unit_asset(ClassId::Cra), BOARD_UNIT_CRA_ASSET);
    assert_eq!(board_unit_asset(ClassId::Sacrier), BOARD_UNIT_SACRIER_ASSET);
    assert_eq!(board_unit_asset(ClassId::Xelor), BOARD_UNIT_XELOR_ASSET);
    assert_eq!(board_unit_asset(ClassId::Ecaflip), BOARD_UNIT_ECAFLIP_ASSET);
    assert_eq!(board_unit_asset(ClassId::Sadida), BOARD_UNIT_SADIDA_ASSET);
    assert_eq!(board_unit_asset(ClassId::Neutral), BOARD_UNIT_NEUTRAL_ASSET);
}

#[test]
fn board_unit_asset_paths_are_non_empty() {
    test_helpers::init_test_tracing();
    for class in [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ] {
        assert!(
            !board_unit_asset(class).is_empty(),
            "empty path for {class:?}"
        );
    }
}
