use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    presentation::PlayerEconomyView,
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::hud::{
        CurrentManaShape, HudEntities, HudEntity, HudPlayerIds, HudPlugin, ManaShapeGeometry,
        ManaShapeKind, ReserveManaShape, CURRENT_MANA_BAR_HEIGHT_PX, CURRENT_MANA_BAR_WIDTH_PX,
        HUD_ENTITY_COUNT, RESERVE_MANA_DIAMOND_ROTATION_DEGREES, RESERVE_MANA_DIAMOND_SIZE_PX,
    },
};
use shared::{
    protocol::{RoundPhase, S2CGoldUpdate, S2CPhaseChanged},
    session::PlayerId,
};

#[test]
fn test_current_and_reserve_mana_expose_non_color_shape_geometry() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftShop, 3);
    apply_own_gold_update(&mut app, gold_update(8, 6, 10, 2));
    app.update();

    assert_eq!(text(&app, entities.mana_label), "6 / 10");
    assert_eq!(text(&app, entities.reserve_label), "+2 reserve");

    let current_geometry = shape_geometry(&app, entities.mana_label);
    assert_eq!(current_geometry.kind, ManaShapeKind::Bar);
    assert!(current_geometry.width_px > current_geometry.height_px);
    assert_eq!(current_geometry.rotation_degrees, 0.0);
    assert_eq!(
        node_px_width(&app, entities.mana_label),
        CURRENT_MANA_BAR_WIDTH_PX
    );
    assert_eq!(
        node_px_height(&app, entities.mana_label),
        CURRENT_MANA_BAR_HEIGHT_PX
    );
    assert!(app
        .world()
        .get::<CurrentManaShape>(entities.mana_label)
        .is_some());

    let reserve_geometry = shape_geometry(&app, entities.reserve_container);
    assert_eq!(reserve_geometry.kind, ManaShapeKind::Diamond);
    assert_eq!(reserve_geometry.width_px, reserve_geometry.height_px);
    assert_eq!(
        reserve_geometry.rotation_degrees,
        RESERVE_MANA_DIAMOND_ROTATION_DEGREES
    );
    assert_eq!(
        node_px_width(&app, entities.reserve_container),
        RESERVE_MANA_DIAMOND_SIZE_PX
    );
    assert_eq!(
        node_px_height(&app, entities.reserve_container),
        RESERVE_MANA_DIAMOND_SIZE_PX
    );
    assert_eq!(
        ui_rotation_degrees(&app, entities.reserve_container),
        RESERVE_MANA_DIAMOND_ROTATION_DEGREES
    );
    assert!(app
        .world()
        .get::<ReserveManaShape>(entities.reserve_container)
        .is_some());

    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_container),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&Visibility::Visible)
    );
}

#[test]
fn test_reserve_zero_hides_diamond_and_label_without_stale_text() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftShop, 3);
    apply_own_gold_update(&mut app, gold_update(8, 6, 10, 2));
    app.update();
    assert_eq!(text(&app, entities.reserve_label), "+2 reserve");
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_container),
        Some(&Visibility::Visible)
    );

    apply_own_gold_update(&mut app, gold_update(8, 6, 10, 0));
    app.update();

    assert_eq!(text(&app, entities.mana_label), "6 / 10");
    assert_eq!(text(&app, entities.reserve_label), "");
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_container),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn test_mana_shape_entities_are_prepooled_and_stable_across_updates() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);
    let current_shape = entities.mana_label;
    let reserve_shape = entities.reserve_container;

    set_phase(&mut app, RoundPhase::DraftShop, 3);
    for update in [
        gold_update(8, 6, 10, 2),
        gold_update(8, 4, 10, 1),
        gold_update(9, 7, 10, 0),
    ] {
        apply_own_gold_update(&mut app, update);
        app.update();
    }

    let after = hud_entities(&app);
    assert_eq!(after.mana_label, current_shape);
    assert_eq!(after.reserve_container, reserve_shape);
    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
    assert_eq!(count_with::<ManaShapeGeometry>(&mut app), 2);
    assert_eq!(count_with::<CurrentManaShape>(&mut app), 1);
    assert_eq!(count_with::<ReserveManaShape>(&mut app), 1);
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HudPlugin);
    app.insert_resource(HudPlayerIds {
        local_id: player(1),
        opponent_id: player(2),
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
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

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn gold_update(gold: u32, current_mana: u32, mana_cap: u8, reserve_mana: u32) -> S2CGoldUpdate {
    S2CGoldUpdate {
        gold,
        current_mana,
        reserve_mana,
        mana_cap,
    }
}

fn apply_own_gold_update(app: &mut App, message: S2CGoldUpdate) {
    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .apply_gold_update(&message);
}

fn shape_geometry(app: &App, entity: Entity) -> ManaShapeGeometry {
    *app.world()
        .get::<ManaShapeGeometry>(entity)
        .expect("mana shape geometry should exist")
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("text should exist")
        .0
        .clone()
}

fn node_px_width(app: &App, entity: Entity) -> f32 {
    val_px(
        app.world()
            .get::<Node>(entity)
            .expect("node should exist")
            .width,
    )
}

fn node_px_height(app: &App, entity: Entity) -> f32 {
    val_px(
        app.world()
            .get::<Node>(entity)
            .expect("node should exist")
            .height,
    )
}

fn ui_rotation_degrees(app: &App, entity: Entity) -> f32 {
    app.world()
        .get::<UiTransform>(entity)
        .expect("ui transform should exist")
        .rotation
        .as_degrees()
}

fn val_px(value: Val) -> f32 {
    match value {
        Val::Px(px) => px,
        other => panic!("expected Val::Px, got {other:?}"),
    }
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}
