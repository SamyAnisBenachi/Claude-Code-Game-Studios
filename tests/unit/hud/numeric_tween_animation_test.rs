use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_tweening::{PlaybackState, TweenAnim, TweenState};
use client::{
    presentation::PlayerEconomyView,
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::hud::{
        GoldDisplayState, GoldTweenTarget, HudConfig, HudEntities, HudGoldBroadcastMessage,
        HudPlayerIds, HudPlugin, ManaDisplayState, ManaTweenTarget,
    },
};
use shared::{
    protocol::{RoundPhase, S2CGoldBroadcast, S2CGoldUpdate, S2CPhaseChanged},
    session::PlayerId,
};

const EPSILON: f32 = 0.01;

#[test]
fn gold_update_tweens_display_target_to_authoritative_value_within_300ms() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftShop, 4);
    set_gold_display(&mut app, entities.own_gold_parent, 5.0, 0.0);
    app.update();

    write_gold_update(&mut app, gold_update(15, 4, 10, 0));
    run_for(&mut app, Duration::ZERO);

    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 15.0);
    assert_eq!(gold_target(&app, entities.own_gold_parent).gold, 5.0);
    assert_eq!(text(&app, entities.own_gold_parent), "5g");
    assert_active_tween(&app, entities.own_gold_parent);

    run_for(&mut app, Duration::from_millis(150));
    let halfway = gold_target(&app, entities.own_gold_parent).gold;
    assert!(
        halfway > 5.0 && halfway < 15.0,
        "expected in-flight gold target between 5 and 15, got {halfway}"
    );

    run_for(&mut app, Duration::from_millis(150));
    assert_approx(gold_target(&app, entities.own_gold_parent).gold, 15.0);
    assert_eq!(text(&app, entities.own_gold_parent), "15g");
}

#[test]
fn gold_tween_cancel_replace_restarts_from_current_interpolated_value() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftShop, 4);
    set_gold_display(&mut app, entities.own_gold_parent, 5.0, 0.0);
    app.update();

    write_gold_update(&mut app, gold_update(15, 4, 10, 0));
    run_for(&mut app, Duration::ZERO);
    run_for(&mut app, Duration::from_millis(120));
    let replacement_start = gold_target(&app, entities.own_gold_parent).gold;

    write_gold_update(&mut app, gold_update(20, 4, 10, 0));
    run_for(&mut app, Duration::ZERO);

    assert_approx(
        gold_target(&app, entities.own_gold_parent).gold,
        replacement_start,
    );
    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 20.0);
    assert_active_tween(&app, entities.own_gold_parent);

    run_for(&mut app, Duration::from_millis(150));
    let restarted_halfway = gold_target(&app, entities.own_gold_parent).gold;
    assert!(
        restarted_halfway > replacement_start && restarted_halfway < 20.0,
        "expected restarted tween between {replacement_start} and 20, got {restarted_halfway}"
    );

    run_for(&mut app, Duration::from_millis(150));
    assert_approx(gold_target(&app, entities.own_gold_parent).gold, 20.0);
    assert_eq!(text(&app, entities.own_gold_parent), "20g");
}

#[test]
fn opponent_reserved_gold_tweens_in_inline_auction_format() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftAuction, 6);
    set_gold_display(&mut app, entities.opponent_gold_parent, 8.0, 0.0);
    app.update();

    write_gold_broadcast(&mut app, gold_broadcast(player(2), 8, 4));
    run_for(&mut app, Duration::ZERO);
    run_for(&mut app, Duration::from_millis(150));

    let reserved_halfway = gold_target(&app, entities.opponent_gold_parent).reserved_gold;
    assert!(
        reserved_halfway > 0.0 && reserved_halfway < 4.0,
        "expected reserved gold target between 0 and 4, got {reserved_halfway}"
    );
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");

    run_for(&mut app, Duration::from_millis(150));
    assert_approx(
        gold_target(&app, entities.opponent_gold_parent).reserved_gold,
        4.0,
    );
    assert_eq!(text_span(&app, entities.opponent_gold_span), " (4r)");
}

#[test]
fn mana_update_tweens_current_cap_and_reserve_display_values() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftShop, 4);
    set_gold_display(&mut app, entities.own_gold_parent, 5.0, 0.0);
    set_mana_display(&mut app, entities.mana_label, 4, 8, 1);
    app.update();

    write_gold_update(&mut app, gold_update(5, 6, 10, 3));
    run_for(&mut app, Duration::ZERO);
    run_for(&mut app, Duration::from_millis(150));

    let target = mana_target(&app, entities.mana_label);
    assert!(target.current_mana > 4.0 && target.current_mana < 6.0);
    assert!(target.mana_cap > 8.0 && target.mana_cap < 10.0);
    assert!(target.reserve_mana > 1.0 && target.reserve_mana < 3.0);

    run_for(&mut app, Duration::from_millis(150));
    let target = mana_target(&app, entities.mana_label);
    assert_approx(target.current_mana, 6.0);
    assert_approx(target.mana_cap, 10.0);
    assert_approx(target.reserve_mana, 3.0);
    assert_eq!(text(&app, entities.mana_label), "6 / 10");
    assert_eq!(text(&app, entities.reserve_label), "+3 reserve");
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.insert_resource(HudConfig {
        hud_margin_px: 12.0,
        hud_dot_diameter_px: 16.0,
        hud_tween_duration_ms: 300,
    });
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

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
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

fn set_gold_display(app: &mut App, entity: Entity, gold: f32, reserved_gold: f32) {
    {
        let mut state = app
            .world_mut()
            .get_mut::<GoldDisplayState>(entity)
            .expect("gold state should exist");
        state.gold = gold;
        state.reserved_gold = reserved_gold;
        state.is_populated = true;
    }
    {
        let mut target = app
            .world_mut()
            .get_mut::<GoldTweenTarget>(entity)
            .expect("gold tween target should exist");
        target.gold = gold;
        target.reserved_gold = reserved_gold;
        target.is_populated = true;
    }
}

fn set_mana_display(
    app: &mut App,
    entity: Entity,
    current_mana: u32,
    mana_cap: u32,
    reserve_mana: u32,
) {
    {
        let mut state = app
            .world_mut()
            .get_mut::<ManaDisplayState>(entity)
            .expect("mana state should exist");
        state.current_mana = current_mana;
        state.mana_cap = mana_cap;
        state.reserve_mana = reserve_mana;
        state.is_populated = true;
    }
    {
        let mut target = app
            .world_mut()
            .get_mut::<ManaTweenTarget>(entity)
            .expect("mana tween target should exist");
        target.current_mana = current_mana as f32;
        target.mana_cap = mana_cap as f32;
        target.reserve_mana = reserve_mana as f32;
        target.is_populated = true;
    }
}

fn write_gold_update(app: &mut App, message: S2CGoldUpdate) {
    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .apply_gold_update(&message);
}

fn write_gold_broadcast(app: &mut App, message: S2CGoldBroadcast) {
    app.world_mut()
        .resource_mut::<Messages<HudGoldBroadcastMessage>>()
        .write(HudGoldBroadcastMessage(message));
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

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn gold_state(app: &App, entity: Entity) -> GoldDisplayState {
    *app.world()
        .get::<GoldDisplayState>(entity)
        .expect("gold state should exist")
}

fn gold_target(app: &App, entity: Entity) -> GoldTweenTarget {
    *app.world()
        .get::<GoldTweenTarget>(entity)
        .expect("gold tween target should exist")
}

fn mana_target(app: &App, entity: Entity) -> ManaTweenTarget {
    *app.world()
        .get::<ManaTweenTarget>(entity)
        .expect("mana tween target should exist")
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("text should exist")
        .0
        .clone()
}

fn text_span(app: &App, entity: Entity) -> String {
    app.world()
        .get::<TextSpan>(entity)
        .expect("text span should exist")
        .0
        .clone()
}

fn assert_active_tween(app: &App, entity: Entity) {
    let animator = app
        .world()
        .get::<TweenAnim>(entity)
        .expect("numeric tween should be attached to the HUD label entity");
    assert_eq!(animator.playback_state, PlaybackState::Playing);
    assert_eq!(animator.tween_state(), TweenState::Active);
}

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {actual} to be within {EPSILON} of {expected}"
    );
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
