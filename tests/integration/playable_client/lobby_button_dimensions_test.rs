//! Story 026 -- Lobby button dimensions and hit-target stability.
//!
//! Friend-game scope only. This test does not claim Standard-tier
//! accessibility or advance `QA-COND-0005`: the 30 px button height remains
//! accepted-risk, and no >=44 px hit-target compliance is claimed.

use bevy::asset::AssetPlugin;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::design_tokens::spacing::SPACING_LG;
use client::ui::design_tokens::typography;
use client::ui::lobby::{
    lobby_all_class_ids, lobby_class_options, LobbyClassButton, LobbyClassPickerCell,
    LobbyClassPortrait, LobbyConfirmClassButton, LobbyCreateRoomButton, LobbyInputState,
    LobbyJoinRoomButton, LobbyOpponentSlotPanel, LobbyOwnSlotPanel, LobbyRequestedSlotButton,
    LobbyRoomCodeChip, LobbyRoomCodeField, LobbyUiPlugin, LOBBY_CLASS_PICKER_BUTTON_HEIGHT_PX,
    LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX, LOBBY_CLASS_PICKER_CELL_HEIGHT_PX,
    LOBBY_CLASS_PICKER_CELL_WIDTH_PX, LOBBY_CLASS_PICKER_PORTRAIT_HEIGHT_PX,
    LOBBY_CLASS_PICKER_PORTRAIT_WIDTH_PX, LOBBY_CONFIRM_BUTTON_HEIGHT_PX,
    LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT, LOBBY_CREATE_BUTTON_HEIGHT_PX,
    LOBBY_CREATE_BUTTON_WIDTH_PX, LOBBY_JOIN_BUTTON_HEIGHT_PX, LOBBY_JOIN_BUTTON_WIDTH_PX,
    LOBBY_PANEL_MAX_WIDTH_PX, LOBBY_REQUESTED_SLOT_BUTTON_HEIGHT_PX,
    LOBBY_REQUESTED_SLOT_BUTTON_WIDTH_PX, LOBBY_ROOM_CODE_CHIP_HEIGHT_PX,
    LOBBY_ROOM_CODE_CHIP_WIDTH_PX, LOBBY_ROOM_CODE_FIELD_HEIGHT_PX,
    LOBBY_ROOM_CODE_FIELD_WIDTH_PERCENT, LOBBY_SLOT_PANEL_HEIGHT_PX, LOBBY_SLOT_PANEL_WIDTH_PX,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

const DIMENSION_TOLERANCE_PX: f32 = 1.0;
const BUTTON_HORIZONTAL_PADDING_PX: f32 = 16.0;

#[derive(Debug, Clone, PartialEq)]
struct DimensionSample {
    name: String,
    width: Val,
    height: Val,
}

impl DimensionSample {
    fn new(name: impl Into<String>, width: Val, height: Val) -> Self {
        Self {
            name: name.into(),
            width,
            height,
        }
    }
}

fn spawn_lobby_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(LobbyUiPlugin);

    app.update();
    app.update();

    app
}

fn transition_to(app: &mut App, state: ClientState) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(state);
    app.update();
    app.update();
}

fn push_single<T: Component>(world: &mut World, name: &str, samples: &mut Vec<DimensionSample>) {
    let mut query = world.query_filtered::<&Node, With<T>>();
    let node = query
        .single(world)
        .unwrap_or_else(|err| panic!("{name} must have exactly one Node: {err:?}"));
    samples.push(DimensionSample::new(name, node.width, node.height));
}

fn sample_lobby_dimensions(app: &mut App) -> Vec<DimensionSample> {
    let world = app.world_mut();
    let mut samples = Vec::new();

    push_single::<LobbyRoomCodeField>(world, "room_code_field", &mut samples);
    push_single::<LobbyCreateRoomButton>(world, "create_button", &mut samples);
    push_single::<LobbyJoinRoomButton>(world, "join_button", &mut samples);
    push_single::<LobbyConfirmClassButton>(world, "confirm_button", &mut samples);
    push_single::<LobbyRoomCodeChip>(world, "room_code_chip", &mut samples);
    push_single::<LobbyOwnSlotPanel>(world, "own_slot_panel", &mut samples);
    push_single::<LobbyOpponentSlotPanel>(world, "opponent_slot_panel", &mut samples);

    let mut slot_query = world.query::<(&LobbyRequestedSlotButton, &Node)>();
    for (button, node) in slot_query.iter(world) {
        samples.push(DimensionSample::new(
            format!("slot_button_{}", button.slot),
            node.width,
            node.height,
        ));
    }

    let mut class_button_query = world.query::<(&LobbyClassButton, &Node)>();
    for (button, node) in class_button_query.iter(world) {
        samples.push(DimensionSample::new(
            format!("class_button_{:?}", button.class_id),
            node.width,
            node.height,
        ));
    }

    let mut cell_query = world.query::<(&LobbyClassPickerCell, &Node)>();
    for (cell, node) in cell_query.iter(world) {
        samples.push(DimensionSample::new(
            format!("class_cell_{:?}", cell.class_id),
            node.width,
            node.height,
        ));
    }

    let mut portrait_query = world.query::<(&LobbyClassPortrait, &Node)>();
    for (portrait, node) in portrait_query.iter(world) {
        samples.push(DimensionSample::new(
            format!("portrait_{:?}", portrait.class_id),
            node.width,
            node.height,
        ));
    }

    samples.sort_by(|left, right| left.name.cmp(&right.name));
    samples
}

fn expected_lobby_dimensions() -> Vec<DimensionSample> {
    let mut expected = vec![
        DimensionSample::new(
            "room_code_field",
            Val::Percent(LOBBY_ROOM_CODE_FIELD_WIDTH_PERCENT),
            Val::Px(LOBBY_ROOM_CODE_FIELD_HEIGHT_PX),
        ),
        DimensionSample::new(
            "create_button",
            Val::Px(LOBBY_CREATE_BUTTON_WIDTH_PX),
            Val::Px(LOBBY_CREATE_BUTTON_HEIGHT_PX),
        ),
        DimensionSample::new(
            "join_button",
            Val::Px(LOBBY_JOIN_BUTTON_WIDTH_PX),
            Val::Px(LOBBY_JOIN_BUTTON_HEIGHT_PX),
        ),
        DimensionSample::new(
            "confirm_button",
            Val::Percent(LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT),
            Val::Px(LOBBY_CONFIRM_BUTTON_HEIGHT_PX),
        ),
        DimensionSample::new(
            "room_code_chip",
            Val::Px(LOBBY_ROOM_CODE_CHIP_WIDTH_PX),
            Val::Px(LOBBY_ROOM_CODE_CHIP_HEIGHT_PX),
        ),
        DimensionSample::new(
            "own_slot_panel",
            Val::Px(LOBBY_SLOT_PANEL_WIDTH_PX),
            Val::Px(LOBBY_SLOT_PANEL_HEIGHT_PX),
        ),
        DimensionSample::new(
            "opponent_slot_panel",
            Val::Px(LOBBY_SLOT_PANEL_WIDTH_PX),
            Val::Px(LOBBY_SLOT_PANEL_HEIGHT_PX),
        ),
    ];

    for slot in 0..=3 {
        expected.push(DimensionSample::new(
            format!("slot_button_{slot}"),
            Val::Px(LOBBY_REQUESTED_SLOT_BUTTON_WIDTH_PX),
            Val::Px(LOBBY_REQUESTED_SLOT_BUTTON_HEIGHT_PX),
        ));
    }

    for class_id in lobby_class_options() {
        expected.push(DimensionSample::new(
            format!("class_button_{class_id:?}"),
            Val::Px(LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX),
            Val::Px(LOBBY_CLASS_PICKER_BUTTON_HEIGHT_PX),
        ));
    }

    for class_id in lobby_all_class_ids() {
        expected.push(DimensionSample::new(
            format!("class_cell_{class_id:?}"),
            Val::Px(LOBBY_CLASS_PICKER_CELL_WIDTH_PX),
            Val::Px(LOBBY_CLASS_PICKER_CELL_HEIGHT_PX),
        ));
        expected.push(DimensionSample::new(
            format!("portrait_{class_id:?}"),
            Val::Px(LOBBY_CLASS_PICKER_PORTRAIT_WIDTH_PX),
            Val::Px(LOBBY_CLASS_PICKER_PORTRAIT_HEIGHT_PX),
        ));
    }

    expected.sort_by(|left, right| left.name.cmp(&right.name));
    expected
}

fn assert_dimension_sets_close(
    actual: &[DimensionSample],
    expected: &[DimensionSample],
    context: &str,
) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{context}: dimension sample count changed"
    );

    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(
            actual.name, expected.name,
            "{context}: sample identity drifted"
        );
        assert_val_close(
            actual.width,
            expected.width,
            &format!("{context}: {} width", actual.name),
        );
        assert_val_close(
            actual.height,
            expected.height,
            &format!("{context}: {} height", actual.name),
        );
    }
}

fn assert_val_close(actual: Val, expected: Val, label: &str) {
    match (actual, expected) {
        (Val::Px(actual), Val::Px(expected)) => assert!(
            (actual - expected).abs() <= DIMENSION_TOLERANCE_PX,
            "{label} drifted: expected {expected}px, got {actual}px"
        ),
        (Val::Percent(actual), Val::Percent(expected)) => assert!(
            (actual - expected).abs() <= f32::EPSILON,
            "{label} drifted: expected {expected}%, got {actual}%"
        ),
        _ => assert_eq!(actual, expected, "{label} changed unit or value"),
    }
}

fn estimated_text_width_px(label: &str) -> f32 {
    label.chars().count() as f32 * typography::BODY * 0.52
}

#[test]
fn ac1_named_constants_match_spawned_lobby_dimensions() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let actual = sample_lobby_dimensions(&mut app);
    let expected = expected_lobby_dimensions();
    assert_dimension_sets_close(&actual, &expected, "AC1");
}

#[test]
fn ac2_repeat_lobby_spawns_preserve_button_dimensions_within_one_pixel() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let first_spawn = sample_lobby_dimensions(&mut app);
    transition_to(&mut app, ClientState::InSession);
    transition_to(&mut app, ClientState::Lobby);
    let second_spawn = sample_lobby_dimensions(&mut app);

    assert_dimension_sets_close(&second_spawn, &first_spawn, "AC2 repeat spawn");
}

#[test]
fn ac3_button_labels_fit_analytically_at_canonical_dimensions() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    {
        let mut input = app.world_mut().resource_mut::<LobbyInputState>();
        input.join_room_code = "ABCDEFGH".to_string();
        input.requested_slot = 3;
    }
    app.update();

    let minimum_panel_inner_width = LOBBY_PANEL_MAX_WIDTH_PX - (2.0 * SPACING_LG);
    let percent_button_inner_width = minimum_panel_inner_width - BUTTON_HORIZONTAL_PADDING_PX;
    let cases = [
        (
            "Create Room",
            LOBBY_CREATE_BUTTON_WIDTH_PX - BUTTON_HORIZONTAL_PADDING_PX,
        ),
        (
            "Join ABCDEFGH",
            LOBBY_JOIN_BUTTON_WIDTH_PX - BUTTON_HORIZONTAL_PADDING_PX,
        ),
        (
            "Slot 3 *",
            LOBBY_REQUESTED_SLOT_BUTTON_WIDTH_PX - BUTTON_HORIZONTAL_PADDING_PX,
        ),
        (
            "Ecaflip *",
            LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX - BUTTON_HORIZONTAL_PADDING_PX,
        ),
        ("Room code: ABCDEFGH (idle)", percent_button_inner_width),
        ("Confirm your class to continue", percent_button_inner_width),
    ];

    for (label, inner_width) in cases {
        let estimate = estimated_text_width_px(label);
        assert!(
            estimate <= inner_width,
            "AC3: `{label}` estimated width {estimate:.1}px exceeds \
             canonical inner button width {inner_width:.1}px"
        );
    }
}

#[test]
fn ac5_friend_game_scope_no_claim_documented_inline() {
    let source = include_str!("lobby_button_dimensions_test.rs");
    assert!(source.contains("QA-COND-0005"));
    assert!(source.contains("accepted-risk"));
    assert!(source.contains("no >=44 px hit-target compliance is claimed"));
}
