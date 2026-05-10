use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{
    board_unit_asset, class_type_icon_asset, hud_figurine_asset, hud_objective_dot_asset,
    insert_placeholder_assets, lobby_portrait_asset, remove_placeholder_assets,
    BidButtonChromeState, ObjectiveDotState, PlaceholderAssets, BOARD_UNIT_IOP_ASSET,
    HUD_FIGURINE_IOP_ASSET, LOBBY_PORTRAIT_IOP_ASSET, PLACEHOLDER_FALLBACK_ASSET,
};
use client::state::ClientState;
use shared::card::ClassId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.add_systems(OnEnter(ClientState::InSession), insert_placeholder_assets);
    app.add_systems(OnExit(ClientState::InSession), remove_placeholder_assets);
    app
}

// ── Resource lifecycle ────────────────────────────────────────────────────────

/// PAW-001-c/d: PlaceholderAssets must be present after InSession entry.
#[test]
fn test_placeholder_assets_resource_inserted_on_session_entry() {
    test_helpers::init_test_tracing();
    let mut app = make_app();

    // Default state is Lobby — resource must not exist yet.
    app.update();
    assert!(
        app.world().get_resource::<PlaceholderAssets>().is_none(),
        "PlaceholderAssets must not exist before InSession"
    );

    // Transition to InSession.
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    assert!(
        app.world().get_resource::<PlaceholderAssets>().is_some(),
        "PlaceholderAssets must be inserted on OnEnter(ClientState::InSession)"
    );
}

/// PAW-001-c: PlaceholderAssets must be removed on InSession exit.
#[test]
fn test_placeholder_assets_resource_removed_on_session_exit() {
    test_helpers::init_test_tracing();
    let mut app = make_app();

    // Enter InSession.
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    assert!(
        app.world().get_resource::<PlaceholderAssets>().is_some(),
        "PlaceholderAssets must exist after entering InSession"
    );

    // Exit InSession back to Lobby.
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Lobby);
    app.update();

    assert!(
        app.world().get_resource::<PlaceholderAssets>().is_none(),
        "PlaceholderAssets must be removed on OnExit(ClientState::InSession)"
    );
}

// ── Path constant shape ───────────────────────────────────────────────────────

/// PAW-001-b: A representative sample of path constants must be non-empty,
/// start with "art/", end with ".png", and contain no whitespace.
#[test]
fn test_path_constants_all_non_empty_and_well_formed() {
    test_helpers::init_test_tracing();
    let representative = [
        PLACEHOLDER_FALLBACK_ASSET,
        HUD_FIGURINE_IOP_ASSET,
        LOBBY_PORTRAIT_IOP_ASSET,
        BOARD_UNIT_IOP_ASSET,
        client::asset_wiring::CARD_FRAME_COMMON_HAND_ASSET,
        client::asset_wiring::CARD_FRAME_RARE_HAND_ASSET,
        client::asset_wiring::CARD_FRAME_EPIC_HAND_ASSET,
        client::asset_wiring::CARD_FRAME_LEGENDARY_HAND_ASSET,
        client::asset_wiring::HUD_OBJECTIVE_DOT_ALIVE_ASSET,
        client::asset_wiring::HUD_OBJECTIVE_DOT_DESTROYED_ASSET,
    ];

    for path in representative {
        assert!(
            !path.is_empty(),
            "Path constant must not be empty: {path:?}"
        );
        assert!(
            path.starts_with("art/"),
            "Path constant must start with 'art/': {path:?}"
        );
        assert!(
            path.ends_with(".png"),
            "Path constant must end with '.png': {path:?}"
        );
        assert!(
            !path.contains(char::is_whitespace),
            "Path constant must contain no whitespace: {path:?}"
        );
    }
}

// ── Selector function coverage ────────────────────────────────────────────────

/// PAW-001-b: hud_figurine_asset must return a distinct non-empty path for
/// every ClassId variant.
#[test]
fn test_selector_functions_cover_all_class_variants() {
    test_helpers::init_test_tracing();
    let all_variants = [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ];

    // Collect all figurine paths — every one must be non-empty and distinct.
    let paths: Vec<&str> = all_variants
        .iter()
        .map(|&c| hud_figurine_asset(c))
        .collect();

    for &path in &paths {
        assert!(!path.is_empty(), "hud_figurine_asset returned empty string");
        assert!(
            path.starts_with("art/"),
            "hud_figurine_asset path must start with 'art/': {path:?}"
        );
        assert!(
            path.ends_with(".png"),
            "hud_figurine_asset path must end with '.png': {path:?}"
        );
    }

    // All paths must be distinct (no two ClassId variants share the same asset).
    let mut deduped = paths.clone();
    deduped.sort_unstable();
    deduped.dedup();
    assert_eq!(
        paths.len(),
        deduped.len(),
        "hud_figurine_asset must return a distinct path per ClassId variant"
    );
}

/// Selector sanity: class_type_icon_asset covers all ClassId variants distinctly.
#[test]
fn test_class_type_icon_selector_covers_all_variants() {
    test_helpers::init_test_tracing();
    let all_variants = [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ];

    let mut paths: Vec<&str> = all_variants
        .iter()
        .map(|&c| class_type_icon_asset(c))
        .collect();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(
        paths.len(),
        all_variants.len(),
        "class_type_icon_asset must return a distinct path per ClassId variant"
    );
}

/// Selector sanity: board_unit_asset covers all ClassId variants distinctly.
#[test]
fn test_board_unit_selector_covers_all_variants() {
    test_helpers::init_test_tracing();
    let all_variants = [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ];

    let mut paths: Vec<&str> = all_variants.iter().map(|&c| board_unit_asset(c)).collect();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(
        paths.len(),
        all_variants.len(),
        "board_unit_asset must return a distinct path per ClassId variant"
    );
}

/// Selector sanity: lobby_portrait_asset covers all ClassId variants distinctly.
#[test]
fn test_lobby_portrait_selector_covers_all_variants() {
    test_helpers::init_test_tracing();
    let all_variants = [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ];

    let mut paths: Vec<&str> = all_variants
        .iter()
        .map(|&c| lobby_portrait_asset(c))
        .collect();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(
        paths.len(),
        all_variants.len(),
        "lobby_portrait_asset must return a distinct path per ClassId variant"
    );
}

/// Selector sanity: hud_objective_dot_asset covers all ObjectiveDotState variants.
#[test]
fn test_objective_dot_selector_covers_all_variants() {
    test_helpers::init_test_tracing();
    let all_variants = [
        ObjectiveDotState::Alive,
        ObjectiveDotState::Destroyed,
        ObjectiveDotState::Unknown,
        ObjectiveDotState::Fake,
    ];

    let mut paths: Vec<&str> = all_variants
        .iter()
        .map(|&s| hud_objective_dot_asset(s))
        .collect();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(
        paths.len(),
        all_variants.len(),
        "hud_objective_dot_asset must return a distinct path per ObjectiveDotState variant"
    );
}

/// Selector sanity: bid_button_asset covers all BidButtonChromeState variants.
#[test]
fn test_bid_button_selector_covers_all_variants() {
    test_helpers::init_test_tracing();
    use client::asset_wiring::bid_button_asset;

    let all_variants = [
        BidButtonChromeState::Normal,
        BidButtonChromeState::Hover,
        BidButtonChromeState::Disabled,
    ];

    let mut paths: Vec<&str> = all_variants.iter().map(|&s| bid_button_asset(s)).collect();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(
        paths.len(),
        all_variants.len(),
        "bid_button_asset must return a distinct path per BidButtonChromeState variant"
    );
}
