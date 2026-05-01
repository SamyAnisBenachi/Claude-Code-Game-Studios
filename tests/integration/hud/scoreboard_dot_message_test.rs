use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_tweening::TweenAnim;
use client::{
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::{
        hud::{
            handle_hud_objective_update_system, HudConfig, HudEntities, HudEntity, HudPlayerIds,
            HudPlugin, HudSystemSet, ScoreboardDot, ScoreboardDotState, HUD_DOTS_PER_ROW,
            HUD_DOT_ROWS,
        },
        shared::{BoardLayout, HudObjectiveUpdate},
    },
};
use shared::{
    protocol::{RoundPhase, S2CPhaseChanged},
    session::PlayerId,
};

const EPSILON: f32 = 0.001;

#[derive(Resource)]
struct PendingHudObjectiveUpdate(Option<HudObjectiveUpdate>);

#[test]
fn scoreboard_dot_alignment_uses_board_layout_lane_projection() {
    let mut app = app_with_hud_in_session();
    assert_dot_centers_match_board_layout(&mut app);

    {
        let mut layout = app.world_mut().resource_mut::<BoardLayout>();
        layout.board_origin.x = 150.0;
        layout.lane_height = 37.5;
    }
    app.update();

    assert_dot_centers_match_board_layout(&mut app);
}

#[test]
fn hud_objective_update_destroys_only_target_dot_in_same_tick() {
    let mut app = app_with_hud_in_session();

    app.insert_resource(PendingHudObjectiveUpdate(Some(HudObjectiveUpdate {
        target_player_id: player(2),
        lane: 3,
    })));
    app.add_systems(
        Update,
        write_pending_hud_objective_update
            .in_set(HudSystemSet::MessageDrain)
            .before(handle_hud_objective_update_system)
            .run_if(in_state(ClientState::InSession)),
    );

    app.update();

    let entities = hud_entities(&app);
    for row in 0..HUD_DOT_ROWS {
        for lane_index in 0..HUD_DOTS_PER_ROW {
            let expected_destroyed = row == 0 && lane_index == 2;
            assert_eq!(
                dot_state(&app, entities.dots[row][lane_index]).destroyed,
                expected_destroyed,
                "unexpected dot state at row {row}, lane index {lane_index}"
            );
        }
    }

    let destroyed_dot = entities.dots[0][2];
    assert_eq!(
        app.world().get::<BackgroundColor>(destroyed_dot),
        Some(&BackgroundColor(Color::NONE))
    );
    assert!(app.world().get::<TweenAnim>(destroyed_dot).is_none());
}

#[test]
fn duplicate_and_oob_hud_objective_updates_do_not_change_other_dots() {
    let mut app = app_with_hud_in_session();

    write_update(&mut app, player(2), 3);
    app.update();
    let before = all_dot_states(&app);

    write_update(&mut app, player(2), 3);
    write_update(&mut app, player(2), 0);
    write_update(&mut app, player(2), 6);
    write_update(&mut app, player(2), u8::MAX);
    app.update();

    assert_eq!(all_dot_states(&app), before);
}

#[test]
fn scoreboard_never_stores_or_renders_objective_identity() {
    let mut app = app_with_hud_in_session();

    write_update(&mut app, player(2), 1);
    set_phase(&mut app, RoundPhase::GameOver, 12);
    app.update();

    assert!(!hud_text_contains(&mut app, "REAL"));
    assert!(!hud_text_contains(&mut app, "FAKE"));
    assert_eq!(
        count_with::<ScoreboardDotState>(&mut app),
        HUD_DOT_ROWS * HUD_DOTS_PER_ROW
    );
    assert_eq!(
        count_with::<ScoreboardDot>(&mut app),
        HUD_DOT_ROWS * HUD_DOTS_PER_ROW
    );
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HudPlugin);
    app.insert_resource(HudConfig {
        hud_margin_px: 12.0,
        hud_dot_diameter_px: 16.0,
        hud_tween_duration_ms: 300,
    });
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

fn write_pending_hud_objective_update(
    mut pending: ResMut<PendingHudObjectiveUpdate>,
    mut writer: MessageWriter<HudObjectiveUpdate>,
) {
    if let Some(update) = pending.0.take() {
        writer.write(update);
    }
}

fn write_update(app: &mut App, target_player_id: PlayerId, lane: u8) {
    app.world_mut()
        .resource_mut::<Messages<HudObjectiveUpdate>>()
        .write(HudObjectiveUpdate {
            target_player_id,
            lane,
        });
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

fn assert_dot_centers_match_board_layout(app: &mut App) {
    let entities = hud_entities(app);
    let layout = *app.world().resource::<BoardLayout>();
    let diameter = app.world().resource::<HudConfig>().hud_dot_diameter_px;

    for row in 0..HUD_DOT_ROWS {
        for lane_index in 0..HUD_DOTS_PER_ROW {
            let lane = lane_index as u8 + 1;
            let expected_center = layout
                .scoreboard_lane_center_x(lane)
                .expect("lane should be valid");
            let node = app
                .world()
                .get::<Node>(entities.dots[row][lane_index])
                .expect("dot should have Node");
            let Val::Px(left) = node.left else {
                panic!("dot left should be BoardLayout-derived px value");
            };
            assert_approx(left + diameter * 0.5, expected_center);
        }
    }
}

fn all_dot_states(app: &App) -> [[ScoreboardDotState; HUD_DOTS_PER_ROW]; HUD_DOT_ROWS] {
    let entities = hud_entities(app);
    std::array::from_fn(|row| std::array::from_fn(|lane| dot_state(app, entities.dots[row][lane])))
}

fn dot_state(app: &App, entity: Entity) -> ScoreboardDotState {
    *app.world()
        .get::<ScoreboardDotState>(entity)
        .expect("dot should have state")
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

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "expected {actual} to be within {EPSILON} of {expected}"
    );
}
