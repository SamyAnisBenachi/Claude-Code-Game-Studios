use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::hud::{
        GoldDisplayState, HudEntities, HudEntity, HudMode, HudPlayerIds, HudPlugin,
        HUD_ENTITY_COUNT,
    },
};
use shared::{
    protocol::{RoundPhase, S2CGoldBroadcast, S2CPhaseChanged},
    session::PlayerId,
};

#[test]
fn draft_auction_entry_formats_both_gold_labels_inline_reserved_zero() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_gold(&mut app, entities.own_gold_parent, 11, 0);
    set_gold(&mut app, entities.opponent_gold_parent, 8, 0);
    set_phase(&mut app, RoundPhase::DraftShop, 3);
    app.update();

    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, entities.own_gold_span), "");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, entities.opponent_gold_span), "");

    set_phase(&mut app, RoundPhase::DraftAuction, 3);
    app.update();

    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyAuction);
    assert_eq!(text(&app, entities.phase_label), "AUCTION");
    assert_eq!(text(&app, entities.round_counter), "R3");
    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, entities.own_gold_span), " (0r)");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, entities.opponent_gold_span), " (0r)");
}

#[test]
fn opponent_gold_broadcast_rendering_adapts_to_hud_mode() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftShop, 3);
    app.update();
    apply_gold_broadcast(&mut app, gold_broadcast(player(2), 7, 0));
    app.update();

    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(text(&app, entities.opponent_gold_parent), "7g");
    assert_eq!(text(&app, entities.opponent_gold_span), "");

    set_phase(&mut app, RoundPhase::DraftAuction, 3);
    app.update();
    apply_gold_broadcast(&mut app, gold_broadcast(player(2), 7, 3));
    app.update();

    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyAuction);
    assert_eq!(text(&app, entities.opponent_gold_parent), "7g");
    assert_eq!(text(&app, entities.opponent_gold_span), " (3r)");
    assert_eq!(text_span_children(&app, entities.opponent_gold_parent), 1);
    assert_eq!(top_level_text_spans(&app, entities.root), 0);
    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
}

#[test]
fn auction_exit_clears_reserved_spans_without_despawning_them() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);
    let own_span = entities.own_gold_span;
    let opponent_span = entities.opponent_gold_span;

    set_gold(&mut app, entities.own_gold_parent, 11, 4);
    set_gold(&mut app, entities.opponent_gold_parent, 8, 2);
    set_phase(&mut app, RoundPhase::DraftAuction, 3);
    app.update();

    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, own_span), " (4r)");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, opponent_span), " (2r)");
    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);

    set_phase(&mut app, RoundPhase::DraftShop, 3);
    app.update();

    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, own_span), "");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, opponent_span), "");
    assert!(app.world().get::<TextSpan>(own_span).is_some());
    assert!(app.world().get::<TextSpan>(opponent_span).is_some());
    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
}

#[test]
fn reserved_gold_display_clamps_to_total_gold() {
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase(&mut app, RoundPhase::DraftAuction, 3);
    app.update();
    apply_gold_broadcast(&mut app, gold_broadcast(player(2), 7, 9));
    app.update();

    assert_eq!(text(&app, entities.opponent_gold_parent), "7g");
    assert_eq!(text(&app, entities.opponent_gold_span), " (7r)");
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

fn set_gold(app: &mut App, entity: Entity, gold: u32, reserved_gold: u32) {
    let mut state = app
        .world_mut()
        .get_mut::<GoldDisplayState>(entity)
        .expect("gold state should exist");
    state.gold = gold as f32;
    state.reserved_gold = reserved_gold as f32;
    state.is_populated = true;
}

fn apply_gold_broadcast(app: &mut App, message: S2CGoldBroadcast) {
    let ids = *app.world().resource::<HudPlayerIds>();
    let entities = hud_entities(app);
    let reserved_gold = message.reserved_gold.min(message.gold);

    if message.player_id == ids.opponent_id {
        let mut state = app
            .world_mut()
            .get_mut::<GoldDisplayState>(entities.opponent_gold_parent)
            .expect("opponent gold state should exist");
        state.gold = message.gold as f32;
        state.reserved_gold = reserved_gold as f32;
        state.is_populated = true;
    }

    if message.player_id == ids.local_id {
        let mut state = app
            .world_mut()
            .get_mut::<GoldDisplayState>(entities.own_gold_parent)
            .expect("own gold state should exist");
        state.reserved_gold = reserved_gold as f32;
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

fn text_span_children(app: &App, parent: Entity) -> usize {
    app.world()
        .get::<Children>(parent)
        .map(|children| {
            children
                .iter()
                .filter(|child| app.world().get::<TextSpan>(*child).is_some())
                .count()
        })
        .unwrap_or_default()
}

fn top_level_text_spans(app: &App, root: Entity) -> usize {
    app.world()
        .get::<Children>(root)
        .map(|children| {
            children
                .iter()
                .filter(|child| app.world().get::<TextSpan>(*child).is_some())
                .count()
        })
        .unwrap_or_default()
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
