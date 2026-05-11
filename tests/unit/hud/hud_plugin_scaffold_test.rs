use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::hud::{
    GoldDisplayState, HudConfig, HudEntities, HudEntity, HudPlugin, HudRoot, HudTimerBar,
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
fn hud_entities_never_contain_timer_components_or_timer_text() {
    let mut app = app_with_hud_in_session();

    assert_eq!(count_hud_timer_named_entities(&mut app), 0);
    assert!(!hud_text_contains_timer_value(&mut app));
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

fn count_hud_timer_named_entities(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<&Name, (With<HudEntity>, With<Name>, Without<HudTimerBar>)>();
    query
        .iter(app.world())
        .filter(|name| name.to_string().to_ascii_lowercase().contains("timer"))
        .count()
}

fn hud_text_contains_timer_value(app: &mut App) -> bool {
    let mut text_query = app
        .world_mut()
        .query_filtered::<&Text, (With<HudEntity>, With<Text>)>();
    if text_query
        .iter(app.world())
        .any(|text| contains_timer_value(&text.0))
    {
        return true;
    }

    let mut span_query = app
        .world_mut()
        .query_filtered::<&TextSpan, (With<HudEntity>, With<TextSpan>)>();
    span_query
        .iter(app.world())
        .any(|span| contains_timer_value(&span.0))
}

fn contains_timer_value(value: &str) -> bool {
    for (index, character) in value.char_indices() {
        if !character.is_ascii_digit() {
            continue;
        }

        let suffix_start = index
            + value[index..]
                .chars()
                .take_while(char::is_ascii_digit)
                .map(char::len_utf8)
                .sum::<usize>();
        let suffix = &value[suffix_start..];
        if suffix.starts_with("ms") || suffix.starts_with("sec") || suffix.starts_with('s') {
            return true;
        }
    }

    false
}
