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
    FanSlotIndex, HandCardCatalog, HandContents, HandFanLayoutConfig, HandFanViewport,
    HandUiCardAcquiredReceived, HandUiPlugin, HandUiTimingConfig,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

// Verdict 3 — Suspect 1 PROVEN: HandFanViewport had no writer system, so
// metrics_for_viewport always anchored the fan to the 800x600 default. This
// test spawns a (Window, PrimaryWindow) at 1920x1080, runs HandUiPlugin, and
// asserts:
//   1. sync_hand_fan_viewport_from_window_system overwrites the default with
//      the live primary-window dimensions.
//   2. apply_fan_layout_system, downstream of the viewport sync, places the
//      pre-pooled fan slots at the formula positions for 1920x1080 - not the
//      stale 800x600 base. AC-HU-02 / AC-HU-02b / AC-HU-03b assume a
//      runtime-screen-anchored fan; without the writer this assumption was
//      silently violated whenever the actual window size differed from 800x600.
const ACQUIRED_CARD_COUNT: usize = 3;
const FIRST_ACQUIRED_CARD_ID: u32 = 50;
const VIEWPORT_WIDTH_PX: f32 = 1920.0;
const VIEWPORT_HEIGHT_PX: f32 = 1080.0;
const POSITION_EPSILON_PX: f32 = 0.5;

#[test]
fn viewport_sync_anchors_fan_layout_to_primary_window_at_1920_1080() {
    let mut app = app_with_hand_ui_at_resolution(VIEWPORT_WIDTH_PX, VIEWPORT_HEIGHT_PX);

    for offset in 0..ACQUIRED_CARD_COUNT {
        let card_id = CardId(FIRST_ACQUIRED_CARD_ID + offset as u32);
        app.world_mut()
            .write_message(HandUiCardAcquiredReceived { card_id });
        run_update(&mut app);
    }

    assert_eq!(
        app.world().resource::<HandContents>().cards.len(),
        ACQUIRED_CARD_COUNT,
        "HandContents must hold the {ACQUIRED_CARD_COUNT} acquired cards before PLACEMENT entry",
    );

    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);

    let viewport = *app.world().resource::<HandFanViewport>();
    assert_eq!(
        viewport.width_px, VIEWPORT_WIDTH_PX,
        "HandFanViewport.width_px must be synced from the primary window (expected {VIEWPORT_WIDTH_PX}, got {})",
        viewport.width_px,
    );
    assert_eq!(
        viewport.height_px, VIEWPORT_HEIGHT_PX,
        "HandFanViewport.height_px must be synced from the primary window (expected {VIEWPORT_HEIGHT_PX}, got {})",
        viewport.height_px,
    );

    let config = *app.world().resource::<HandFanLayoutConfig>();
    let fan_center_x = VIEWPORT_WIDTH_PX / 2.0;
    let fan_base_y = VIEWPORT_HEIGHT_PX - config.fan_base_margin_px;

    // count=3 -> t values are -1, 0, +1
    let expectations: [(u8, f32, f32); ACQUIRED_CARD_COUNT] = [
        (
            0,
            fan_center_x - config.fan_half_spread_px,
            fan_base_y - config.arc_height_px,
        ),
        (1, fan_center_x, fan_base_y),
        (
            2,
            fan_center_x + config.fan_half_spread_px,
            fan_base_y - config.arc_height_px,
        ),
    ];

    for (slot_index, expected_left, expected_top) in expectations {
        let slot = fan_slot(&mut app, slot_index);
        let node = app
            .world()
            .get::<Node>(slot)
            .expect("fan slot entity must carry a Node component after layout");
        let left = px(node.left);
        let top = px(node.top);
        assert!(
            (left - expected_left).abs() <= POSITION_EPSILON_PX,
            "fan slot {slot_index} left expected {expected_left}px, got {left}px (1920x1080 viewport)",
        );
        assert!(
            (top - expected_top).abs() <= POSITION_EPSILON_PX,
            "fan slot {slot_index} top expected {expected_top}px, got {top}px (1920x1080 viewport)",
        );
    }
}

#[test]
fn default_viewport_persists_when_no_primary_window_is_present() {
    // Sanity: tests that ran before Verdict 3 worked because they injected
    // HandFanViewport (800x600) directly. The new sync system must NOT clobber
    // the default when no Window/PrimaryWindow entity exists in the world.
    let mut app = app_with_hand_ui_no_window();

    for offset in 0..ACQUIRED_CARD_COUNT {
        let card_id = CardId(FIRST_ACQUIRED_CARD_ID + offset as u32);
        app.world_mut()
            .write_message(HandUiCardAcquiredReceived { card_id });
        run_update(&mut app);
    }

    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);

    let viewport = *app.world().resource::<HandFanViewport>();
    assert_eq!(
        viewport,
        HandFanViewport::default(),
        "without a primary window the sync system must leave HandFanViewport at its default (800x600)",
    );
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

fn app_with_hand_ui_no_window() -> App {
    let mut app = base_app();
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
