use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::{
    presentation::PlayerEconomyView,
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::{
        hud::{
            HudEntities, HudEntity, HudGoldBroadcastMessage, HudMode, HudPlayerIds, HudPlugin,
            HUD_ENTITY_COUNT, HUD_GOLD_TEXT_MIN_SIZE_PX, HUD_RESOURCE_TEXT_MIN_SIZE_PX,
        },
        shared::BoardLayout,
    },
};
use shared::{
    protocol::{RoundPhase, S2CGoldBroadcast, S2CGoldUpdate, S2CPhaseChanged},
    session::PlayerId,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

const HUD_CONTRAST_MIN_RATIO: f32 = 4.5;
const VIEWPORTS: [(u32, u32); 2] = [(1366, 768), (1920, 1080)];

#[test]
fn test_hud_accessibility_fixture_meets_text_size_floors_and_contrast() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    apply_accessibility_fixture(&mut app);

    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.phase_label), "AUCTION");
    assert_eq!(text(&app, entities.round_counter), "R9");
    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, entities.own_gold_span), " (4r)");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, entities.opponent_gold_span), " (3r)");
    assert_eq!(text(&app, entities.mana_label), "6 / 10");
    assert_eq!(text(&app, entities.reserve_label), "+2 reserve");

    for viewport in VIEWPORTS {
        assert_font_floor(
            &app,
            "own gold",
            entities.own_gold_parent,
            HUD_GOLD_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "opponent gold",
            entities.opponent_gold_parent,
            HUD_GOLD_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "own reserved-gold suffix",
            entities.own_gold_span,
            HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "opponent reserved-gold suffix",
            entities.opponent_gold_span,
            HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "current mana",
            entities.mana_label,
            HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "reserve mana",
            entities.reserve_label,
            HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "phase label",
            entities.phase_label,
            HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "round counter",
            entities.round_counter,
            HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            viewport,
        );
    }

    assert_text_contrast(
        &app,
        "own gold",
        entities.own_gold_parent,
        entities.own_gold_parent,
    );
    assert_text_contrast(
        &app,
        "opponent gold",
        entities.opponent_gold_parent,
        entities.opponent_gold_parent,
    );
    assert_text_contrast(
        &app,
        "own reserved-gold suffix",
        entities.own_gold_span,
        entities.own_gold_parent,
    );
    assert_text_contrast(
        &app,
        "opponent reserved-gold suffix",
        entities.opponent_gold_span,
        entities.opponent_gold_parent,
    );
    assert_text_contrast(
        &app,
        "current mana",
        entities.mana_label,
        entities.mana_label,
    );
    assert_text_contrast(
        &app,
        "reserve mana",
        entities.reserve_label,
        entities.reserve_container,
    );
    assert_text_contrast(
        &app,
        "phase label",
        entities.phase_label,
        entities.phase_label,
    );
    assert_text_contrast(
        &app,
        "round counter",
        entities.round_counter,
        entities.round_counter,
    );
}

#[test]
fn test_cold_start_placeholders_meet_matching_accessibility_floors() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(text(&app, entities.own_gold_parent), "--g");
    assert_eq!(text(&app, entities.opponent_gold_parent), "--g");
    assert_eq!(text(&app, entities.mana_label), "-- / --");

    for viewport in VIEWPORTS {
        assert_font_floor(
            &app,
            "own gold placeholder",
            entities.own_gold_parent,
            HUD_GOLD_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "opponent gold placeholder",
            entities.opponent_gold_parent,
            HUD_GOLD_TEXT_MIN_SIZE_PX,
            viewport,
        );
        assert_font_floor(
            &app,
            "mana placeholder",
            entities.mana_label,
            HUD_RESOURCE_TEXT_MIN_SIZE_PX,
            viewport,
        );
    }

    assert_text_contrast(
        &app,
        "own gold placeholder",
        entities.own_gold_parent,
        entities.own_gold_parent,
    );
    assert_text_contrast(
        &app,
        "opponent gold placeholder",
        entities.opponent_gold_parent,
        entities.opponent_gold_parent,
    );
    assert_text_contrast(
        &app,
        "mana placeholder",
        entities.mana_label,
        entities.mana_label,
    );
}

#[test]
fn test_resolution_hud_contrast_has_no_phase_dimming_regression() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    apply_accessibility_fixture(&mut app);

    set_phase(&mut app, RoundPhase::Resolution, 9);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(text(&app, entities.phase_label), "RESOLUTION");
    assert_eq!(text(&app, entities.round_counter), "R9");
    assert_eq!(text(&app, entities.own_gold_span), "");
    assert_eq!(text(&app, entities.opponent_gold_span), "");

    for (name, text_entity, background_entity) in [
        (
            "own gold",
            entities.own_gold_parent,
            entities.own_gold_parent,
        ),
        (
            "opponent gold",
            entities.opponent_gold_parent,
            entities.opponent_gold_parent,
        ),
        ("current mana", entities.mana_label, entities.mana_label),
        (
            "reserve mana",
            entities.reserve_label,
            entities.reserve_container,
        ),
        ("phase label", entities.phase_label, entities.phase_label),
        (
            "round counter",
            entities.round_counter,
            entities.round_counter,
        ),
    ] {
        assert_text_contrast(&app, name, text_entity, background_entity);
    }
}

#[test]
fn test_hud_accessibility_changes_preserve_visibility_and_entity_pool() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let initial_entities = hud_entity_ids(&mut app);
    apply_accessibility_fixture(&mut app);

    let entities = hud_entities(&app);
    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
    assert_eq!(hud_entity_ids(&mut app), initial_entities);

    apply_own_gold_update(&mut app, gold_update(12, 6, 10, 0));
    run_for(&mut app, Duration::ZERO);
    run_for(&mut app, Duration::from_millis(300));
    assert_eq!(text(&app, entities.reserve_label), "");
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&Visibility::Hidden)
    );

    apply_own_gold_update(&mut app, gold_update(13, 7, 10, 1));
    apply_own_gold_update(&mut app, gold_update(14, 8, 10, 2));
    apply_gold_broadcast(&mut app, gold_broadcast(player(2), 9, 4));
    app.update();

    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
    assert_eq!(hud_entity_ids(&mut app), initial_entities);
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.insert_resource(HudPlayerIds {
        local_id: player(1),
        opponent_id: player(2),
    });
    app.insert_resource(BoardLayout {
        board_origin: Vec2::new(100.0, 20.0),
        cell_width: 64.0,
        lane_height: 44.0,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn apply_accessibility_fixture(app: &mut App) {
    set_phase(app, RoundPhase::DraftAuction, 9);
    app.update();

    apply_own_gold_update(app, gold_update(11, 6, 10, 2));
    apply_gold_broadcast(app, gold_broadcast(player(1), 11, 4));
    apply_gold_broadcast(app, gold_broadcast(player(2), 8, 3));
    app.update();
}

fn set_phase(app: &mut App, phase: RoundPhase, round_number: u32) {
    let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
    apply_phase_changed_message(
        S2CPhaseChanged {
            phase,
            round_number,
            timer_duration_ms: 60_000,
        },
        &mut current,
    );
}

fn apply_own_gold_update(app: &mut App, message: S2CGoldUpdate) {
    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .apply_gold_update(&message);
}

fn apply_gold_broadcast(app: &mut App, message: S2CGoldBroadcast) {
    app.world_mut()
        .resource_mut::<Messages<HudGoldBroadcastMessage>>()
        .write(HudGoldBroadcastMessage(message));
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
}

fn gold_update(gold: u32, current_mana: u32, mana_cap: u8, reserve_mana: u32) -> S2CGoldUpdate {
    S2CGoldUpdate {
        gold,
        current_mana,
        reserve_mana,
        mana_cap,
    }
}

fn gold_broadcast(player_id: PlayerId, gold: u32, reserved_gold: u32) -> S2CGoldBroadcast {
    S2CGoldBroadcast {
        player_id,
        gold,
        reserved_gold,
    }
}

fn assert_font_floor(app: &App, label: &str, entity: Entity, floor_px: f32, viewport: (u32, u32)) {
    let measured_px = app
        .world()
        .get::<TextFont>(entity)
        .expect("HUD text should have TextFont")
        .font_size;
    assert!(
        measured_px >= floor_px,
        "{label} measured {measured_px}px below {floor_px}px at {}x{}",
        viewport.0,
        viewport.1
    );
}

fn assert_text_contrast(app: &App, label: &str, text_entity: Entity, background_entity: Entity) {
    let foreground = app
        .world()
        .get::<TextColor>(text_entity)
        .expect("HUD text should have TextColor")
        .0;
    let background = app
        .world()
        .get::<BackgroundColor>(background_entity)
        .expect("HUD text background should be explicit")
        .0;
    let ratio = contrast_ratio(foreground, background);
    assert!(
        ratio >= HUD_CONTRAST_MIN_RATIO,
        "{label} contrast {ratio:.2}:1 below {HUD_CONTRAST_MIN_RATIO}:1"
    );
}

fn contrast_ratio(foreground: Color, background: Color) -> f32 {
    let foreground = foreground.to_srgba();
    let background = background.to_srgba();
    let alpha = foreground.alpha.clamp(0.0, 1.0);
    let composited = [
        foreground.red * alpha + background.red * (1.0 - alpha),
        foreground.green * alpha + background.green * (1.0 - alpha),
        foreground.blue * alpha + background.blue * (1.0 - alpha),
    ];
    let background = [background.red, background.green, background.blue];
    let foreground_luminance = relative_luminance(composited);
    let background_luminance = relative_luminance(background);
    let lighter = foreground_luminance.max(background_luminance);
    let darker = foreground_luminance.min(background_luminance);
    (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(color: [f32; 3]) -> f32 {
    0.2126 * linearized(color[0]) + 0.7152 * linearized(color[1]) + 0.0722 * linearized(color[2])
}

fn linearized(channel: f32) -> f32 {
    if channel <= 0.03928 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn hud_entity_ids(app: &mut App) -> Vec<Entity> {
    let mut query = app.world_mut().query_filtered::<Entity, With<HudEntity>>();
    let mut entities = query.iter(app.world()).collect::<Vec<_>>();
    entities.sort_by_key(|entity| entity.index());
    entities
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn text(app: &App, entity: Entity) -> String {
    if let Some(text) = app.world().get::<Text>(entity) {
        return text.0.clone();
    }

    app.world()
        .get::<TextSpan>(entity)
        .expect("HUD text entity should have Text or TextSpan")
        .0
        .clone()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
