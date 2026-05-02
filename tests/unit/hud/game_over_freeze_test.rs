use std::time::Duration;

use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_tweening::{lens::TransformScaleLens, TweenAnim};
use client::card_animations::make_tween_anim;
use client::{
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::{
        hud::{
            GoldDisplayState, HudEntities, HudEntity, HudMode, HudPlayerIds, HudPlugin,
            ScoreboardDotState,
        },
        shared::HudObjectiveUpdate,
    },
};
use shared::{
    protocol::{RoundPhase, S2CPhaseChanged},
    session::PlayerId,
};

#[test]
fn game_over_freezes_hud_and_rejects_objective_updates() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::Resolution, 14);
    set_gold(&mut app, entities.own_gold_parent, 12);
    app.update();

    write_objective_update(&mut app, player(2), 2);
    app.update();
    let before_lane_1 = dot_state(&app, entities.dots[0][0]);
    let before_lane_2 = dot_state(&app, entities.dots[0][1]);

    set_phase(&mut app, RoundPhase::GameOver, 14);
    app.update();
    write_objective_update(&mut app, player(2), 1);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::Frozen);
    assert_eq!(text(&app, entities.phase_label), "GAME OVER");
    assert_eq!(text(&app, entities.round_counter), "R14");
    assert_eq!(
        app.world().get::<Visibility>(entities.round_counter),
        Some(&Visibility::Visible)
    );
    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 12.0);
    assert_eq!(text(&app, entities.own_gold_parent), "12g");
    assert_eq!(dot_state(&app, entities.dots[0][0]), before_lane_1);
    assert_eq!(dot_state(&app, entities.dots[0][1]), before_lane_2);
    assert!(!hud_text_contains(&mut app, "REAL"));
    assert!(!hud_text_contains(&mut app, "FAKE"));
}

#[test]
fn game_over_snap_rerenders_gold_state_and_cancels_gold_tween() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::Resolution, 14);
    app.update();
    set_gold(&mut app, entities.own_gold_parent, 15);
    set_text(&mut app, entities.own_gold_parent, "7g");
    insert_active_gold_tween(&mut app, entities.own_gold_parent);

    set_phase(&mut app, RoundPhase::GameOver, 14);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::Frozen);
    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 15.0);
    assert_eq!(text(&app, entities.own_gold_parent), "15g");
    assert!(app
        .world()
        .get::<TweenAnim>(entities.own_gold_parent)
        .is_none());
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

fn set_gold(app: &mut App, entity: Entity, gold: u32) {
    let mut state = app
        .world_mut()
        .get_mut::<GoldDisplayState>(entity)
        .expect("gold state should exist");
    state.gold = gold as f32;
    state.is_populated = true;
}

fn set_text(app: &mut App, entity: Entity, value: &'static str) {
    app.world_mut()
        .get_mut::<Text>(entity)
        .expect("text should exist")
        .0 = value.to_string();
}

fn write_objective_update(app: &mut App, target_player_id: PlayerId, lane: u8) {
    app.world_mut()
        .resource_mut::<Messages<HudObjectiveUpdate>>()
        .write(HudObjectiveUpdate {
            target_player_id,
            lane,
        });
}

fn insert_active_gold_tween(app: &mut App, entity: Entity) {
    app.world_mut()
        .entity_mut(entity)
        .insert(make_tween_anim(bevy_tweening::Tween::new(
            EaseFunction::Linear,
            Duration::from_millis(300),
            TransformScaleLens {
                start: Vec3::ONE,
                end: Vec3::splat(1.25),
            },
        )));
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn gold_state(app: &App, entity: Entity) -> GoldDisplayState {
    *app.world()
        .get::<GoldDisplayState>(entity)
        .expect("gold state should exist")
}

fn dot_state(app: &App, entity: Entity) -> ScoreboardDotState {
    *app.world()
        .get::<ScoreboardDotState>(entity)
        .expect("dot state should exist")
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("HUD text entity should have Text")
        .0
        .clone()
}

fn hud_text_contains(app: &mut App, needle: &str) -> bool {
    let mut text_query = app
        .world_mut()
        .query_filtered::<&Text, (With<HudEntity>, With<Text>)>();
    if text_query
        .iter(app.world())
        .any(|text| text.0.contains(needle))
    {
        return true;
    }

    let mut span_query = app
        .world_mut()
        .query_filtered::<&TextSpan, (With<HudEntity>, With<TextSpan>)>();
    span_query
        .iter(app.world())
        .any(|span| span.0.contains(needle))
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
