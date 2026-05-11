use std::collections::HashMap;
use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::{prelude::*, time::Virtual};
use bevy_tweening::TweeningPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, HandCardCatalog, HandUiCardAcquiredReceived, HandUiPlugin, HandUiTimingConfig,
    HAND_FAN_STRIP_HEIGHT_PX,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// PROMPT 671 / Finding B v2 V3 — regression for the audit gap left by the
// existing viewport_sync test (only asserts Node.left/top numeric values, never
// checks whether the slot is on-screen). PROMPT 669 verdict A proved that
// `metrics_for_viewport` was producing viewport-coord `fan_base_y` while every
// fan slot is `ChildOf(fan_root)` with the root anchored to viewport bottom
// with height HAND_FAN_STRIP_HEIGHT_PX — so the slot's REAL on-screen position
// was `viewport.height - HAND_FAN_STRIP_HEIGHT_PX + (viewport.height - margin)`,
// which is off-screen at every viewport larger than ~860px tall.
//
// The fix returns LOCAL-to-fan_root coords. This test asserts the absolute
// on-screen position of each occupied fan slot at three representative
// viewports.
//
// NOTE on coord-space derivation:
// - fan_root is `position_type: Absolute, left:0, right:0, bottom:0,
//   height: HAND_FAN_STRIP_HEIGHT_PX`.
// - Therefore strip's top-left in viewport coords =
//   (0, viewport.height - HAND_FAN_STRIP_HEIGHT_PX).
// - Each fan slot's `Node.left/top` is interpreted relative to that origin.
// - On-screen position = (slot.node.left, viewport.height -
//   HAND_FAN_STRIP_HEIGHT_PX + slot.node.top).
//
// This computation matches what `bevy::ui::UiPlugin`'s taffy layout produces
// for an absolute child of an absolute parent. Running the full UiPlugin
// pipeline headless would require pulling in AssetPlugin, WindowPlugin,
// TransformPlugin, ImagePlugin, and a Camera2d — none of which any other
// hand-ui test currently brings in. The Node-level assertions test the exact
// source-level invariant the PROMPT 669 verdict identified, with no fragile
// dependence on a partial render pipeline.

const ACQUIRED_CARD_COUNT: usize = 3;
const FIRST_ACQUIRED_CARD_ID: u32 = 50;
const LAYOUT_CONVERGENCE_FRAMES: usize = 4;

#[test]
fn fan_slots_remain_onscreen_at_default_800x600_viewport() {
    test_helpers::init_test_tracing();
    assert_all_occupied_fan_slots_on_screen(800.0, 600.0);
}

#[test]
fn fan_slots_remain_onscreen_at_720p_viewport() {
    test_helpers::init_test_tracing();
    assert_all_occupied_fan_slots_on_screen(1280.0, 720.0);
}

#[test]
fn fan_slots_remain_onscreen_at_1080p_viewport() {
    test_helpers::init_test_tracing();
    assert_all_occupied_fan_slots_on_screen(1920.0, 1080.0);
}

fn assert_all_occupied_fan_slots_on_screen(viewport_width: f32, viewport_height: f32) {
    let mut app = app_with_hand_ui_at_resolution(viewport_width, viewport_height);

    for offset in 0..ACQUIRED_CARD_COUNT {
        let card_id = CardId(FIRST_ACQUIRED_CARD_ID + offset as u32);
        app.world_mut()
            .write_message(HandUiCardAcquiredReceived { card_id });
        run_update(&mut app);
    }

    set_phase(&mut app, RoundPhase::Placement);
    for _ in 0..LAYOUT_CONVERGENCE_FRAMES {
        run_update(&mut app);
    }

    let strip_top_y = viewport_height - HAND_FAN_STRIP_HEIGHT_PX;

    for slot_index in 0..ACQUIRED_CARD_COUNT as u8 {
        let slot = fan_slot(&mut app, slot_index);
        let node = app
            .world()
            .get::<Node>(slot)
            .unwrap_or_else(|| panic!("fan slot {slot_index} must carry a Node after layout"));

        let local_left = px(node.left);
        let local_top = px(node.top);
        let screen_x = local_left;
        let screen_y = strip_top_y + local_top;

        assert!(
            (0.0..=viewport_width).contains(&screen_x),
            "fan slot {slot_index} screen X must be inside viewport [0, {viewport_width}] at \
             {viewport_width}x{viewport_height}; got screen_x={screen_x} (local left={local_left})",
        );
        assert!(
            (0.0..=viewport_height).contains(&screen_y),
            "fan slot {slot_index} screen Y must be inside viewport [0, {viewport_height}] at \
             {viewport_width}x{viewport_height}; got screen_y={screen_y} (local top={local_top}, \
             strip_top_y={strip_top_y})",
        );

        let within_strip_bounds_local = (0.0..=HAND_FAN_STRIP_HEIGHT_PX).contains(&local_top);
        assert!(
            within_strip_bounds_local,
            "fan slot {slot_index} local top must be inside HandFanRoot strip [0, {}]; got {}. \
             A value outside this range indicates `metrics_for_viewport` produced a viewport-coord \
             instead of a LOCAL-to-fan_root coord (the PROMPT 669 verdict-A regression).",
            HAND_FAN_STRIP_HEIGHT_PX, local_top,
        );
    }
}

fn app_with_hand_ui_at_resolution(width: f32, height: f32) -> App {
    let mut app = base_app();
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(width as u32, height as u32),
            ..default()
        },
        PrimaryWindow,
    ));
    finalize_app(&mut app);
    app
}

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(TweeningPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog {
        cards: test_catalog((FIRST_ACQUIRED_CARD_ID)..(FIRST_ACQUIRED_CARD_ID + 32)),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 5,
        reserve_mana: 0,
        initialized: true,
        ..default()
    });
    app.insert_resource(HandUiTimingConfig {
        card_draw_animation_ms: 280,
        purchase_timeout_ms: 3_000,
        hand_full_notification_duration_ms: 2_000,
    });
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app
}

fn finalize_app(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(app, RoundPhase::DraftInitial);
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn fan_slot(app: &mut App, slot_index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, idx)| (idx.0 == slot_index).then_some(entity))
        .expect("fan slot must exist")
}

fn px(value: Val) -> f32 {
    match value {
        Val::Px(v) => v,
        other => panic!("expected Val::Px, got {other:?}"),
    }
}

fn test_catalog(ids: impl IntoIterator<Item = u32>) -> HashMap<CardId, CardData> {
    ids.into_iter()
        .map(|id| {
            let card = test_card(id);
            (card.id, card)
        })
        .collect()
}

fn test_card(id: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 1,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: None,
    }
}
