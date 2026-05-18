use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::hud::{
    GoldDisplayState, HudConfig, HudEntities, HudEntity, HudPlugin, HudRoot, HudTimerCountdown,
    ScoreboardDot, HUD_ENTITY_COUNT,
};

#[test]
fn hud_initializes_exact_prepooled_entity_tree() {
    let mut app = app_with_hud_in_session();

    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
    assert_eq!(count_with::<GoldDisplayState>(&mut app), 2);
    assert_eq!(count_with::<ScoreboardDot>(&mut app), 10);

    let entities = *app.world().resource::<HudEntities>();
    assert_eq!(
        app.world()
            .get::<TextSpan>(entities.own_gold_span)
            .unwrap()
            .0,
        ""
    );
    assert_eq!(
        app.world()
            .get::<TextSpan>(entities.opponent_gold_span)
            .unwrap()
            .0,
        ""
    );
}

#[test]
fn hud_does_not_respawn_entities_on_subsequent_frames() {
    let mut app = app_with_hud_in_session();

    for _ in 0..3 {
        app.update();
        assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
        assert_eq!(count_with::<GoldDisplayState>(&mut app), 2);
        assert_eq!(count_with::<ScoreboardDot>(&mut app), 10);
    }
}

#[test]
fn hud_root_starts_hidden_before_any_phase_message() {
    let app = app_with_hud_in_session();
    let entities = app.world().resource::<HudEntities>();

    assert!(app.world().get::<HudRoot>(entities.root).is_some());
    assert_eq!(
        app.world().get::<Visibility>(entities.root),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn hud_pre_pools_a_phase_timer_countdown_entity_hidden_until_active() {
    // S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
    // UI-1129-06) — successor to the deleted
    // `hud_entities_never_contain_timer_components_or_timer_text`
    // assertion (which forbade a numeric timer text on the HUD). The
    // new contract requires exactly one `HudTimerCountdown` entity
    // pre-pooled at session entry, spawned hidden because no
    // `S2CPhaseChanged` has fired yet.
    let mut app = app_with_hud_in_session();

    let entities = *app.world().resource::<HudEntities>();
    assert_eq!(count_with::<HudTimerCountdown>(&mut app), 1);
    assert_eq!(
        app.world().get::<Visibility>(entities.timer_countdown),
        Some(&Visibility::Hidden),
        "pre-pooled countdown must start hidden — visibility is gated by PhaseTimerState",
    );
    assert_eq!(
        app.world()
            .get::<Text>(entities.timer_countdown)
            .map(|t| t.0.as_str()),
        Some(""),
        "pre-pooled countdown must start with empty text",
    );
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.insert_resource(HudConfig::default());
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}
