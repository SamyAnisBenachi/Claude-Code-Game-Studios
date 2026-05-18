use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::hud::{
        GoldDisplayState, HudEntities, HudMode, HudPlugin, HudRoot, ManaDisplayState,
        ScoreboardDotState,
    },
};
use shared::protocol::{RoundPhase, S2CGoldUpdate};

#[test]
fn lobby_to_draft_initial_shows_hud_and_preserves_alive_dots() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(app.world().resource::<HudMode>(), &HudMode::Hidden);
    assert_eq!(
        app.world().get::<Visibility>(entities.root),
        Some(&Visibility::Hidden)
    );

    set_phase(&mut app, RoundPhase::DraftInitial, 1);
    app.update();

    let entities = hud_entities(&app);
    assert!(app.world().get::<HudRoot>(entities.root).is_some());
    assert_eq!(
        app.world().get::<Visibility>(entities.root),
        Some(&Visibility::Visible)
    );
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    // PROMPT 1250 (S18-HUD-PHASE-CHIP-DISAMBIGUATION-001): DraftInitial
    // now reads as `KEEP-9` so the chip is unambiguous next to `SHOP`.
    assert_eq!(text(&app, entities.phase_label), "KEEP-9");
    assert_eq!(text(&app, entities.round_counter), "R1");
    assert_all_dots_alive(&app, entities);
}

#[test]
fn resolution_to_draft_shop_uses_basic_gold_format() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_gold(&mut app, entities.own_gold_parent, 11, 4);
    set_gold(&mut app, entities.opponent_gold_parent, 8, 2);
    set_text(&mut app, entities.own_gold_span, " (4r)");
    set_text(&mut app, entities.opponent_gold_span, " (2r)");

    set_phase(&mut app, RoundPhase::Resolution, 4);
    app.update();
    set_phase(&mut app, RoundPhase::DraftShop, 4);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    // PROMPT 1250 (S18-HUD-PHASE-CHIP-DISAMBIGUATION-001): DraftShop
    // now reads as `SHOP` so the chip stem is unique vs. `KEEP-9`.
    assert_eq!(text(&app, entities.phase_label), "SHOP");
    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, entities.own_gold_span), "");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, entities.opponent_gold_span), "");
}

#[test]
fn economy_auction_exit_to_draft_shop_clears_reserved_gold_spans() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_gold(&mut app, entities.own_gold_parent, 11, 4);
    set_gold(&mut app, entities.opponent_gold_parent, 8, 2);
    set_text(&mut app, entities.own_gold_parent, "11g");
    set_text(&mut app, entities.own_gold_span, " (4r)");
    set_text(&mut app, entities.opponent_gold_parent, "8g");
    set_text(&mut app, entities.opponent_gold_span, " (2r)");
    *app.world_mut().resource_mut::<HudMode>() = HudMode::EconomyAuction;

    set_phase(&mut app, RoundPhase::DraftShop, 5);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, entities.own_gold_span), "");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, entities.opponent_gold_span), "");
}

#[test]
fn placement_to_resolution_keeps_hud_visible_and_zones_intact() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::Placement, 6);
    app.update();
    let entities = hud_entities(&app);
    let reserve_visibility = *app
        .world()
        .get::<Visibility>(entities.reserve_label)
        .expect("reserve label visibility should exist");
    // PROMPT 1250: concise stems — Placement → `PLACE`, Resolution →
    // `RESOLVE`.
    assert_eq!(text(&app, entities.phase_label), "PLACE");

    set_phase(&mut app, RoundPhase::Resolution, 6);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(text(&app, entities.phase_label), "RESOLVE");
    assert_eq!(
        app.world().get::<Visibility>(entities.root),
        Some(&Visibility::Visible)
    );
    assert_core_hud_zones_visible(&app, entities);
    assert_eq!(
        app.world().get::<Visibility>(entities.reserve_label),
        Some(&reserve_visibility)
    );
}

#[test]
fn resolution_accepts_gold_updates() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::Resolution, 7);
    app.update();
    apply_own_gold_update(&mut app, gold_update(15, 5, 10, 0));
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 15.0);
    assert_eq!(text(&app, entities.own_gold_parent), "15g");
    assert_eq!(text(&app, entities.mana_label), "5 / 10");
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase, round_number: u32) {
    let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
    apply_phase_changed_message(
        shared::protocol::S2CPhaseChanged {
            phase,
            round_number,
            timer_duration_ms: 60_000,
        },
        &mut current,
    );
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

fn gold_update(gold: u32, current_mana: u32, mana_cap: u8, reserve_mana: u32) -> S2CGoldUpdate {
    S2CGoldUpdate {
        gold,
        current_mana,
        reserve_mana,
        mana_cap,
    }
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn set_gold(app: &mut App, entity: Entity, gold: u32, reserved_gold: u32) {
    let mut state = app
        .world_mut()
        .get_mut::<GoldDisplayState>(entity)
        .expect("gold state should exist");
    state.gold = gold as f32;
    state.reserved_gold = reserved_gold as f32;
    state.is_populated = true;
}

fn gold_state(app: &App, entity: Entity) -> GoldDisplayState {
    *app.world()
        .get::<GoldDisplayState>(entity)
        .expect("gold state should exist")
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

fn set_text(app: &mut App, entity: Entity, value: &'static str) {
    if let Some(mut text) = app.world_mut().get_mut::<Text>(entity) {
        text.0 = value.to_string();
        return;
    }

    app.world_mut()
        .get_mut::<TextSpan>(entity)
        .expect("text span should exist")
        .0 = value.to_string();
}

fn assert_all_dots_alive(app: &App, entities: HudEntities) {
    for row in entities.dots {
        for dot in row {
            assert_eq!(
                app.world().get::<ScoreboardDotState>(dot).copied(),
                Some(ScoreboardDotState { destroyed: false })
            );
        }
    }
}

fn assert_core_hud_zones_visible(app: &App, entities: HudEntities) {
    for entity in [
        entities.phase_label,
        entities.round_counter,
        entities.own_gold_parent,
        entities.own_gold_span,
        entities.opponent_gold_parent,
        entities.opponent_gold_span,
        entities.mana_label,
    ] {
        assert_eq!(
            app.world().get::<Visibility>(entity),
            Some(&Visibility::Visible)
        );
    }

    for row in entities.dots {
        for dot in row {
            assert_eq!(
                app.world().get::<Visibility>(dot),
                Some(&Visibility::Visible)
            );
        }
    }
}
