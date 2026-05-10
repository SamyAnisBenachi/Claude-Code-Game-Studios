use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    presentation::PlayerEconomyView,
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::hud::{
        GoldDisplayState, HudEntities, HudGoldBroadcastMessage, HudMode, HudPlayerIds, HudPlugin,
        ManaDisplayState,
    },
};
use shared::{
    protocol::{RoundPhase, S2CGoldBroadcast, S2CGoldUpdate, S2CPhaseChanged},
    session::PlayerId,
};

#[test]
fn same_tick_gold_update_wins_over_own_gold_broadcast() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::DraftShop, 3);
    app.update();

    write_gold_broadcast(&mut app, gold_broadcast(player(1), 12, 0));
    write_gold_update(&mut app, gold_update(15, 0, 10, 0));
    app.update();

    let entities = hud_entities(&app);
    let own_gold = gold_state(&app, entities.own_gold_parent);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(own_gold.gold, 15.0);
    assert_eq!(own_gold.reserved_gold, 0.0);
    assert_eq!(text(&app, entities.own_gold_parent), "15g");
    assert_eq!(mana_state(&app, entities.mana_label).mana_cap, 10);
}

#[test]
fn same_tick_own_broadcast_gold_is_ignored_while_reserved_gold_updates() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::DraftAuction, 3);
    app.update();

    write_gold_broadcast(&mut app, gold_broadcast(player(1), 99, 5));
    write_gold_update(&mut app, gold_update(15, 0, 10, 0));
    app.update();

    let entities = hud_entities(&app);
    let own_gold = gold_state(&app, entities.own_gold_parent);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyAuction);
    assert_eq!(own_gold.gold, 15.0);
    assert_eq!(own_gold.reserved_gold, 5.0);
    assert_eq!(text(&app, entities.own_gold_parent), "15g");
    assert_eq!(text(&app, entities.own_gold_span), " (5r)");
}

#[test]
fn own_gold_broadcast_alone_updates_only_reserved_gold() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftShop, 4);
    set_own_gold(&mut app, 10, 0);
    app.update();

    write_gold_broadcast(&mut app, gold_broadcast(player(1), 3, 2));
    app.update();

    let own_gold = gold_state(&app, entities.own_gold_parent);
    assert_eq!(own_gold.gold, 10.0);
    assert_eq!(own_gold.reserved_gold, 2.0);
    assert_eq!(text(&app, entities.own_gold_parent), "10g");
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

fn set_own_gold(app: &mut App, gold: u32, reserved_gold: u32) {
    let entities = hud_entities(app);
    let mut state = app
        .world_mut()
        .get_mut::<GoldDisplayState>(entities.own_gold_parent)
        .expect("own gold state should exist");
    state.gold = gold as f32;
    state.reserved_gold = reserved_gold as f32;
    state.is_populated = true;
}

fn write_gold_broadcast(app: &mut App, message: S2CGoldBroadcast) {
    app.world_mut()
        .resource_mut::<Messages<HudGoldBroadcastMessage>>()
        .write(HudGoldBroadcastMessage(message));
}

fn write_gold_update(app: &mut App, message: S2CGoldUpdate) {
    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .apply_gold_update(&message);
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

fn mana_state(app: &App, entity: Entity) -> ManaDisplayState {
    *app.world()
        .get::<ManaDisplayState>(entity)
        .expect("mana state should exist")
}

fn text(app: &App, entity: Entity) -> String {
    if let Some(text) = app.world().get::<Text>(entity) {
        return text.0.clone();
    }

    app.world()
        .get::<TextSpan>(entity)
        .expect("text or text span should exist")
        .0
        .clone()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
