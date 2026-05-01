use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_tweening::TweenAnim;
use client::state::ClientState;
use client::ui::hud::{
    apply_gold_update_batch, GoldDisplayState, HudEntities, HudPlayerIds, HudPlugin,
    ManaDisplayState, ScoreboardDotState,
};
use shared::protocol::{S2CGoldBroadcast, S2CGoldUpdate};
use shared::session::PlayerId;

#[test]
fn basic_gold_mana_display_formats_and_reserve_visibility() {
    let mut app = app_with_hud_in_session();

    apply_own_gold_update(&mut app, gold_update(8, 6, 10, 2));
    apply_gold_broadcast(&mut app, gold_broadcast(player(2), 6, 0));
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.own_gold_parent), "8g");
    assert_eq!(text(&app, entities.opponent_gold_parent), "6g");
    assert_eq!(text(&app, entities.mana_label), "6 / 10");
    assert_eq!(text(&app, entities.reserve_label), "+2 reserve");
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&Visibility::Visible)
    );
}

#[test]
fn gold_update_changes_only_local_economy_readouts() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    apply_own_gold_update(&mut app, gold_update(5, 3, 8, 1));
    apply_gold_broadcast(&mut app, gold_broadcast(player(2), 3, 0));
    set_text(&mut app, entities.phase_label, "DRAFT");
    set_text(&mut app, entities.round_counter, "R2");
    app.world_mut()
        .get_mut::<ScoreboardDotState>(entities.dots[0][0])
        .expect("dot state should exist")
        .destroyed = true;
    app.update();

    let opponent_state_before = gold_state(&app, entities.opponent_gold_parent);
    let opponent_text_before = text(&app, entities.opponent_gold_parent);
    let phase_text_before = text(&app, entities.phase_label);
    let round_text_before = text(&app, entities.round_counter);
    let dot_state_before = *app
        .world()
        .get::<ScoreboardDotState>(entities.dots[0][0])
        .expect("dot state should exist");

    apply_own_gold_update(&mut app, gold_update(10, 4, 8, 0));
    app.update();

    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 10.0);
    assert_eq!(text(&app, entities.own_gold_parent), "10g");
    assert_eq!(text(&app, entities.mana_label), "4 / 8");
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        gold_state(&app, entities.opponent_gold_parent),
        opponent_state_before
    );
    assert_eq!(
        text(&app, entities.opponent_gold_parent),
        opponent_text_before
    );
    assert_eq!(text(&app, entities.phase_label), phase_text_before);
    assert_eq!(text(&app, entities.round_counter), round_text_before);
    assert_eq!(
        app.world()
            .get::<ScoreboardDotState>(entities.dots[0][0])
            .copied(),
        Some(dot_state_before)
    );
}

#[test]
fn mana_cap_denominator_updates_and_zero_reserve_hides_label() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    apply_own_gold_update(&mut app, gold_update(7, 4, 8, 2));
    app.update();
    assert_eq!(text(&app, entities.mana_label), "4 / 8");
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&Visibility::Visible)
    );

    apply_own_gold_update(&mut app, gold_update(7, 4, 10, 0));
    app.update();
    assert_eq!(text(&app, entities.mana_label), "4 / 10");
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&Visibility::Hidden)
    );

    apply_own_gold_update(&mut app, gold_update(7, 5, 3, 0));
    app.update();
    assert_eq!(text(&app, entities.mana_label), "5 / 3");
}

#[test]
fn cold_start_placeholders_distinguish_unpopulated_from_zero() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(text(&app, entities.own_gold_parent), "--g");
    assert_eq!(text(&app, entities.mana_label), "-- / --");
    assert_eq!(text(&app, entities.opponent_gold_parent), "--g");

    apply_own_gold_update(&mut app, gold_update(0, 0, 10, 0));
    app.update();

    assert_eq!(text(&app, entities.own_gold_parent), "0g");
    assert_eq!(text(&app, entities.mana_label), "0 / 10");
}

#[test]
fn mana_cap_zero_renders_without_panic() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    apply_own_gold_update(&mut app, gold_update(2, 0, 0, 0));
    app.update();

    assert_eq!(text(&app, entities.mana_label), "0 / 0");
}

#[test]
fn multi_update_collapse_applies_only_last_gold_update() {
    let mut gold = GoldDisplayState::default();
    let mut mana = ManaDisplayState::default();

    let applied = apply_gold_update_batch(
        vec![
            gold_update(7, 2, 10, 0),
            gold_update(9, 3, 10, 0),
            gold_update(11, 4, 10, 0),
        ],
        &mut gold,
        &mut mana,
    )
    .expect("batch should apply last update");

    assert_eq!(applied.gold, 11);
    assert_eq!(gold.gold, 11.0);
    assert_eq!(mana.current_mana, 4);

    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);
    assert!(app
        .world()
        .get::<TweenAnim>(entities.own_gold_parent)
        .is_none());
    apply_own_gold_update(&mut app, applied);
    app.update();
    assert_eq!(text(&app, entities.own_gold_parent), "11g");
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

fn gold_broadcast(player_id: PlayerId, gold: u32, reserved_gold: u32) -> S2CGoldBroadcast {
    S2CGoldBroadcast {
        player_id,
        gold,
        reserved_gold,
    }
}

fn apply_own_gold_update(app: &mut App, message: S2CGoldUpdate) {
    let entities = hud_entities(app);
    {
        let mut gold_state = app
            .world_mut()
            .get_mut::<GoldDisplayState>(entities.own_gold_parent)
            .expect("own gold state should exist");
        gold_state.gold = message.gold as f32;
        gold_state.is_populated = true;
    }
    {
        let mut mana_state = app
            .world_mut()
            .get_mut::<ManaDisplayState>(entities.mana_label)
            .expect("mana state should exist");
        mana_state.current_mana = message.current_mana;
        mana_state.mana_cap = message.mana_cap as u32;
        mana_state.reserve_mana = message.reserve_mana;
        mana_state.is_populated = true;
    }
}

fn apply_gold_broadcast(app: &mut App, message: S2CGoldBroadcast) {
    let ids = *app.world().resource::<HudPlayerIds>();
    let entities = hud_entities(app);

    if message.player_id == ids.opponent_id {
        let mut state = app
            .world_mut()
            .get_mut::<GoldDisplayState>(entities.opponent_gold_parent)
            .expect("opponent gold state should exist");
        state.gold = message.gold as f32;
        state.reserved_gold = message.reserved_gold as f32;
        state.is_populated = true;
    }

    if message.player_id == ids.local_id {
        let mut state = app
            .world_mut()
            .get_mut::<GoldDisplayState>(entities.own_gold_parent)
            .expect("own gold state should exist");
        state.reserved_gold = message.reserved_gold as f32;
    }
}

fn gold_state(app: &App, entity: Entity) -> GoldDisplayState {
    *app.world()
        .get::<GoldDisplayState>(entity)
        .expect("gold state should exist")
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("text should exist")
        .0
        .clone()
}

fn set_text(app: &mut App, entity: Entity, value: &'static str) {
    app.world_mut()
        .get_mut::<Text>(entity)
        .expect("text should exist")
        .0 = value.to_string();
}
