use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    compute_fan_slot_layout, FanLayoutMetrics, FanSlotIndex, HandContents, HandFanLayoutConfig,
    HandFanLayoutState, HandFanViewport, HandSubmitInteractionState, HandUiEntities, HandUiPlugin,
    ReserveStripForFanSlot, HAND_FAN_SLOT_COUNT, HAND_FAN_STRIP_HEIGHT_PX,
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
    // PROMPT 671 — coord-space update: `apply_fan_layout_system` writes
    // `transform.translation.y` from `metrics_for_viewport`, which now produces
    // LOCAL-to-fan_root coords (fan_base_y = HAND_FAN_STRIP_HEIGHT_PX -
    // fan_base_margin_px = 260 - 100 = 160 at the inserted 800x600 viewport).
    // The pure-formula tests above (hu_02/hu_02b/hu_03) still use
    // `qa_metrics().fan_base_y = 500` because they feed the metric directly
    // into `compute_fan_slot_layout` — they prove formula math, not screen
    // anchoring. THIS test exercises the system end-to-end, so the asserted
    // y-values must match the new LOCAL base.
    let mut app = app_with_hand_ui_in_session(5);
    app.update();

    let center = slot_transform(&mut app, 2);
    assert_approx(center.translation.x, 400.0);
    assert_approx(center.translation.y, 160.0);
    assert_approx(center.rotation.to_euler(EulerRot::XYZ).2, 0.0);

    let rightmost = slot_transform(&mut app, 4);
    assert_approx(rightmost.translation.x, 680.0);
    assert_approx(rightmost.translation.y, 150.0);
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

#[test]
fn reserve_strip_uses_hand_fan_local_coordinates_above_card() {
    let mut app = app_with_hand_ui_in_session(5);
    app.update();

    let slot = fan_slot_entity(&mut app, 0);
    let slot_node = app
        .world()
        .get::<Node>(slot)
        .expect("fan slot should have a node");
    let slot_top = expect_px(slot_node.top);
    let slot_left = expect_px(slot_node.left);
    let reserve = reserve_strip_entity(&mut app, 0);
    let reserve_node = app
        .world()
        .get::<Node>(reserve)
        .expect("reserve strip should have a node");

    assert_approx(expect_px(reserve_node.left), slot_left);
    assert_approx(
        expect_px(reserve_node.bottom),
        HAND_FAN_STRIP_HEIGHT_PX - slot_top + 10.0,
    );
    assert!(
        expect_px(reserve_node.bottom) < HAND_FAN_STRIP_HEIGHT_PX,
        "reserve strip bottom must stay in HandFanRoot-local space, not viewport space"
    );
}

fn reserve_strip_entity(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &ReserveStripForFanSlot)>();
    query
        .iter(app.world())
        .find_map(|(entity, reserve_slot)| (reserve_slot.0 == index).then_some(entity))
        .expect("reserve strip should exist")
}

fn expect_px(value: Val) -> f32 {
    match value {
        Val::Px(px) => px,
        other => panic!("expected Val::Px, got {other:?}"),
    }
}

/// Card display height in px — mirrors the private constant in hand/mod.rs.
const CARD_H: f32 = 150.0;
/// Right-side badge footprint width in px: 24% of 108px card width ≈ 25.92 px.
const RIGHT_BADGE_W: f32 = 108.0 * 0.24;

/// PROMPT 1854 (STAGE3-D) — 10-card readability invariants at 1280×720.
///
/// Two structural guarantees on `HandFanLayoutConfig::default()`:
///
/// 1. **No bottom clip**: every card's local-strip bottom (card_y + CARD_H) stays
///    within `HAND_FAN_STRIP_HEIGHT_PX` so ATK/HP badges are never cut off at the
///    viewport bottom edge.
///
/// 2. **Right-badge visible**: spacing between adjacent card left edges exceeds the
///    right-badge width, so the AR/HP badges (rightmost 24% of the card) are not
///    occluded by the neighbour card.
#[test]
fn default_config_10_cards_at_1280x720_readability_invariants() {
    let config = HandFanLayoutConfig::default();
    let viewport = HandFanViewport {
        width_px: 1280.0,
        height_px: 720.0,
    };
    let metrics = config.metrics_for_viewport(viewport);

    let layouts: Vec<_> = (0..HAND_FAN_SLOT_COUNT)
        .map(|i| {
            compute_fan_slot_layout(i, HAND_FAN_SLOT_COUNT, metrics)
                .expect("all 10 slots must produce a layout")
        })
        .collect();

    // Invariant 1: no card bottom clips below the strip.
    for (i, layout) in layouts.iter().enumerate() {
        let card_bottom = layout.card_y + CARD_H;
        assert!(
            card_bottom <= HAND_FAN_STRIP_HEIGHT_PX + EPSILON,
            "slot {i}: card bottom {card_bottom:.1} exceeds strip height {HAND_FAN_STRIP_HEIGHT_PX} — ATK/HP badges off-screen",
        );
    }

    // Invariant 2: adjacent card spacing > right-badge width so AR/HP badges are
    // not hidden behind the next card.
    for i in 0..HAND_FAN_SLOT_COUNT - 1 {
        let spacing = layouts[i + 1].card_x - layouts[i].card_x;
        assert!(
            spacing > RIGHT_BADGE_W,
            "slot {i}→{}: spacing {spacing:.1} px is less than right-badge width {RIGHT_BADGE_W:.1} — AR/HP badges hidden",
            i + 1,
        );
    }
}

/// PROMPT 2037 — small hands must cluster around `fan_center_x` instead of
/// stretching to the full `fan_half_spread`. With the default
/// `max_card_pitch_px = 78` the effective half-spread is `(count - 1) * 78 / 2`
/// until it hits `fan_half_spread_px = 380`. At 3 cards the half-spread caps
/// at 78 (centres at 562, 640, 718 in a 1280-wide viewport), not the legacy
/// ±380.
#[test]
fn default_config_3_cards_at_1280x720_cluster_around_center() {
    let config = HandFanLayoutConfig::default();
    let viewport = HandFanViewport {
        width_px: 1280.0,
        height_px: 720.0,
    };
    let metrics = config.metrics_for_viewport(viewport);

    let left = compute_fan_slot_layout(0, 3, metrics).expect("slot 0");
    let middle = compute_fan_slot_layout(1, 3, metrics).expect("slot 1");
    let right = compute_fan_slot_layout(2, 3, metrics).expect("slot 2");

    assert_approx(middle.card_x, 640.0);
    let pitch_left = middle.card_x - left.card_x;
    let pitch_right = right.card_x - middle.card_x;
    assert_approx(pitch_left, 78.0);
    assert_approx(pitch_right, 78.0);
    assert!(
        pitch_left < 100.0,
        "3-card pitch {pitch_left:.1} must stay clustered (< 100 px) rather than spreading the \
         whole 760 px half_spread; small hands were spreading to the viewport edges before \
         PROMPT 2037",
    );
}

/// PROMPT 2037 — at 10 cards the pitch cap (`9 * 78 / 2 = 351`) is below the
/// configured `fan_half_spread_px = 380`, so the cap engages and per-card
/// spacing is 78 px — well above the 30%-wide right-badge so AR/HP stay
/// readable, and below the 108 px card width so the fan visually overlaps
/// instead of leaving gaps.
#[test]
fn default_config_10_cards_at_1280x720_pitch_is_clamped_to_max_card_pitch() {
    let config = HandFanLayoutConfig::default();
    let viewport = HandFanViewport {
        width_px: 1280.0,
        height_px: 720.0,
    };
    let metrics = config.metrics_for_viewport(viewport);

    let layouts: Vec<_> = (0..HAND_FAN_SLOT_COUNT)
        .map(|i| compute_fan_slot_layout(i, HAND_FAN_SLOT_COUNT, metrics).expect("all 10 slots"))
        .collect();

    for i in 0..HAND_FAN_SLOT_COUNT - 1 {
        let spacing = layouts[i + 1].card_x - layouts[i].card_x;
        assert_approx(spacing, 78.0);
    }
    let fan_center_x = viewport.width_px / 2.0;
    let span_centre = (layouts.first().unwrap().card_x + layouts.last().unwrap().card_x) / 2.0;
    assert_approx(span_centre, fan_center_x);
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
        // PROMPT 2037 — pre-clustering tests assert the historical pure-spread
        // formula. Setting INFINITY disables the pitch cap so the existing
        // expectations (`card_x = 680` at t=1 for 5 cards) still hold.
        max_card_pitch_px: f32::INFINITY,
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
        // PROMPT 2037 — disable the pitch cap so pre-clustering formula tests
        // keep their historical expectations.
        max_card_pitch: f32::INFINITY,
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
