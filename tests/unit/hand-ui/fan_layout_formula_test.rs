use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    compute_fan_slot_layout, FanLayoutMetrics, FanSlotIndex, HandContents, HandFanLayoutConfig,
    HandFanLayoutState, HandFanViewport, HandSubmitInteractionState, HandUiEntities, HandUiPlugin,
    HAND_FAN_SLOT_COUNT,
};
use shared::card::CardId;
use shared::protocol::RoundPhase;

const EPSILON: f32 = 0.001;

#[test]
fn hu_02_count_five_positions_center_and_edges() {
    let metrics = qa_metrics();

    let center = layout(2, 5, metrics);
    assert_layout(center, 0.0, 400.0, 500.0, 0.0);

    let rightmost = layout(4, 5, metrics);
    assert_layout(rightmost, 1.0, 680.0, 490.0, 10.0);
    assert_approx(rightmost.bevy_rotation_radians(), -10.0_f32.to_radians());

    let leftmost = layout(0, 5, metrics);
    assert_layout(leftmost, -1.0, 120.0, 490.0, -10.0);
    assert_approx(leftmost.bevy_rotation_radians(), 10.0_f32.to_radians());
}

#[test]
fn hu_02b_count_two_uses_full_normalized_span() {
    let metrics = qa_metrics();

    let left = layout(0, 2, metrics);
    let right = layout(1, 2, metrics);

    assert_layout(left, -1.0, 120.0, 490.0, -10.0);
    assert_layout(right, 1.0, 680.0, 490.0, 10.0);
}

#[test]
fn hu_03_single_card_early_return_centers_without_arc_or_tilt() {
    let metrics = qa_metrics();

    let single = layout(0, 1, metrics);

    assert_layout(single, 0.0, 400.0, 500.0, 0.0);
    assert_approx(single.bevy_rotation_radians(), 0.0);
}

#[test]
fn hu_03b_zero_cards_skips_formula_hides_slots_and_keeps_submit_active() {
    let mut app = app_with_hand_ui_in_session(0);

    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    let submit = app.world().resource::<HandUiEntities>().submit_button;

    for slot in fan_slot_entities(&mut app) {
        assert_eq!(
            app.world().get::<Visibility>(slot),
            Some(&Visibility::Hidden)
        );
    }
    assert_eq!(
        app.world().get::<Visibility>(submit),
        Some(&Visibility::Visible)
    );
    assert_eq!(text(&app, submit), "Submit (0 cards)");
    assert_eq!(
        app.world().get::<HandSubmitInteractionState>(submit),
        Some(&HandSubmitInteractionState::Active)
    );
    assert!(compute_fan_slot_layout(0, 0, qa_metrics()).is_none());
}

#[test]
fn layout_system_applies_formula_to_visible_pooled_slots() {
    let mut app = app_with_hand_ui_in_session(5);
    app.update();

    let center = slot_transform(&mut app, 2);
    assert_approx(center.translation.x, 400.0);
    assert_approx(center.translation.y, 500.0);
    assert_approx(center.rotation.to_euler(EulerRot::XYZ).2, 0.0);

    let rightmost = slot_transform(&mut app, 4);
    assert_approx(rightmost.translation.x, 680.0);
    assert_approx(rightmost.translation.y, 490.0);
    assert_approx(
        rightmost.rotation.to_euler(EulerRot::XYZ).2,
        -10.0_f32.to_radians(),
    );

    for index in 0..HAND_FAN_SLOT_COUNT {
        let entity = fan_slot_entity(&mut app, index);
        let expected = if index < 5 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        assert_eq!(app.world().get::<Visibility>(entity), Some(&expected));
    }
}

fn app_with_hand_ui_in_session(hand_count: usize) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandFanLayoutConfig {
        fan_base_margin_px: 100.0,
        fan_half_spread_px: 280.0,
        arc_height_px: 10.0,
        max_rotation_deg: 10.0,
    });
    app.insert_resource(HandFanViewport {
        width_px: 800.0,
        height_px: 600.0,
    });
    app.insert_resource(HandFanLayoutState { hand_count });
    app.insert_resource(HandContents {
        cards: (0..hand_count).map(|index| CardId(index as u32)).collect(),
    });
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::Placement;
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn qa_metrics() -> FanLayoutMetrics {
    FanLayoutMetrics {
        fan_center_x: 400.0,
        fan_base_y: 500.0,
        fan_half_spread: 280.0,
        arc_height: 10.0,
        max_rotation_deg: 10.0,
    }
}

fn layout(
    index: usize,
    count: usize,
    metrics: FanLayoutMetrics,
) -> client::ui::hand::FanSlotLayout {
    compute_fan_slot_layout(index, count, metrics).expect("layout should exist")
}

fn assert_layout(
    layout: client::ui::hand::FanSlotLayout,
    t: f32,
    card_x: f32,
    card_y: f32,
    card_rotation_deg: f32,
) {
    assert_approx(layout.t, t);
    assert_approx(layout.card_x, card_x);
    assert_approx(layout.card_y, card_y);
    assert_approx(layout.card_rotation_deg, card_rotation_deg);
}

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "expected {actual} to be within {EPSILON} of {expected}"
    );
}

fn fan_slot_entities(app: &mut App) -> Vec<Entity> {
    app.world().resource::<HandUiEntities>().fan_slots.to_vec()
}

fn fan_slot_entity(app: &mut App, index: usize) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 as usize == index).then_some(entity))
        .expect("fan slot should exist")
}

fn slot_transform(app: &mut App, index: usize) -> Transform {
    let entity = fan_slot_entity(app, index);
    *app.world()
        .get::<Transform>(entity)
        .expect("fan slot should have transform")
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("submit button should have Text")
        .0
        .clone()
}
