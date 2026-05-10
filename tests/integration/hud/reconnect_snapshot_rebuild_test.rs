use std::time::Duration;

use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_tweening::{lens::TransformScaleLens, TweenAnim};
use client::card_animations::make_tween_anim;
use client::{
    presentation::{PlayerEconomyView, PresentationGameSnapshotMessage},
    state::{ClientState, CurrentClientPhase},
    ui::hud::{
        GoldDisplayState, HudEntities, HudMode, HudPlayerIds, HudPlugin, ManaDisplayState,
        ScoreboardDotState, HUD_DOTS_PER_ROW, HUD_ENTITY_COUNT,
    },
};
use shared::{
    card::ClassId,
    protocol::{
        BoardSnapshot, ObjectiveSnapshot, OpponentObjectiveSnapshot, PlayerSnapshot, RoundPhase,
        S2CGameSnapshot, S2CGoldUpdate,
    },
    session::PlayerId,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn full_snapshot_rebuild_populates_all_hud_zones_without_respawning_entities() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let before = hud_entities(&app);

    dirty_hud_state(&mut app);
    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::Placement,
            7,
            player_snapshot(
                player(1),
                ClassId::Iop,
                EconomyValues::new(20, 0, 6, 0, 10),
                [false; HUD_DOTS_PER_ROW],
                [false, false, true, false, false],
            ),
            player_snapshot(
                player(2),
                ClassId::Cra,
                EconomyValues::new(15, 0, 0, 0, 10),
                [false, false, true, false, false],
                [false; HUD_DOTS_PER_ROW],
            ),
        ),
    );
    app.update();

    let after = hud_entities(&app);
    assert_eq!(hud_entity_ids(before), hud_entity_ids(after));
    assert_eq!(
        count_with::<client::ui::hud::HudEntity>(&mut app),
        HUD_ENTITY_COUNT
    );
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyBasic);
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::Placement
    );
    assert_eq!(app.world().resource::<CurrentClientPhase>().round, 7);
    assert_eq!(text(&app, after.own_gold_parent), "20g");
    assert_eq!(text(&app, after.opponent_gold_parent), "15g");
    assert_eq!(text(&app, after.mana_label), "6 / 10");
    assert_eq!(text(&app, after.reserve_label), "");
    assert_eq!(text(&app, after.phase_label), "PLACEMENT");
    assert_eq!(text(&app, after.round_counter), "R7");
    assert_eq!(
        app.world().get::<Visibility>(after.root),
        Some(&Visibility::Visible)
    );

    for row in 0..2 {
        for lane in 0..HUD_DOTS_PER_ROW {
            let expected = row == 0 && lane == 2;
            assert_eq!(
                dot_state(&app, after.dots[row][lane]).destroyed,
                expected,
                "unexpected dot state at row {row}, lane {lane}"
            );
        }
    }
    assert!(app
        .world()
        .get::<TweenAnim>(after.own_gold_parent)
        .is_none());
    assert!(app.world().get::<TweenAnim>(after.mana_label).is_none());
}

#[test]
fn draft_auction_snapshot_rebuild_uses_reserved_gold_format() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::DraftAuction,
            4,
            player_snapshot(
                player(1),
                ClassId::Iop,
                EconomyValues::new(11, 4, 3, 1, 10),
                [false; HUD_DOTS_PER_ROW],
                [false; HUD_DOTS_PER_ROW],
            ),
            player_snapshot(
                player(2),
                ClassId::Cra,
                EconomyValues::new(8, 2, 0, 0, 10),
                [false; HUD_DOTS_PER_ROW],
                [false; HUD_DOTS_PER_ROW],
            ),
        ),
    );
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::EconomyAuction);
    assert_eq!(text(&app, entities.phase_label), "AUCTION");
    assert_eq!(text(&app, entities.own_gold_parent), "11g");
    assert_eq!(text(&app, entities.own_gold_span), " (4r)");
    assert_eq!(text(&app, entities.opponent_gold_parent), "8g");
    assert_eq!(text(&app, entities.opponent_gold_span), " (2r)");
    assert_eq!(text(&app, entities.mana_label), "3 / 10");
    assert_eq!(text(&app, entities.reserve_label), "+1 reserve");
}

#[test]
fn game_over_snapshot_bypasses_frozen_then_reapplies_incremental_gate() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    *app.world_mut().resource_mut::<HudMode>() = HudMode::Frozen;
    set_gold_state(&mut app, entities.own_gold_parent, 12, 0);
    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::GameOver,
            9,
            player_snapshot(
                player(1),
                ClassId::Iop,
                EconomyValues::new(5, 0, 2, 0, 10),
                [false, true, false, false, false],
                [true, false, false, false, false],
            ),
            player_snapshot(
                player(2),
                ClassId::Cra,
                EconomyValues::new(3, 0, 0, 0, 10),
                [true, false, false, false, false],
                [false, true, false, false, false],
            ),
        ),
    );
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(app.world().resource::<HudMode>(), &HudMode::Frozen);
    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 5.0);
    assert_eq!(text(&app, entities.own_gold_parent), "5g");
    assert_eq!(text(&app, entities.phase_label), "GAME OVER");
    assert_eq!(text(&app, entities.round_counter), "R9");
    assert!(dot_state(&app, entities.dots[0][0]).destroyed);
    assert!(dot_state(&app, entities.dots[1][1]).destroyed);

    write_gold_update(&mut app, gold_update(999, 9, 10, 0));
    app.update();

    assert_eq!(app.world().resource::<HudMode>(), &HudMode::Frozen);
    assert_eq!(gold_state(&app, entities.own_gold_parent).gold, 5.0);
    assert_eq!(text(&app, entities.own_gold_parent), "5g");
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

fn dirty_hud_state(app: &mut App) {
    let entities = hud_entities(app);
    set_gold_state(app, entities.own_gold_parent, 99, 7);
    set_gold_state(app, entities.opponent_gold_parent, 88, 6);
    {
        let mut mana = app
            .world_mut()
            .get_mut::<ManaDisplayState>(entities.mana_label)
            .expect("mana state should exist");
        mana.current_mana = 1;
        mana.mana_cap = 2;
        mana.reserve_mana = 3;
        mana.is_populated = true;
    }
    app.world_mut()
        .entity_mut(entities.own_gold_parent)
        .insert(active_tween());
    app.world_mut()
        .entity_mut(entities.mana_label)
        .insert(active_tween());
}

fn active_tween() -> TweenAnim {
    make_tween_anim(bevy_tweening::Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(300),
        TransformScaleLens {
            start: Vec3::ONE,
            end: Vec3::splat(1.25),
        },
    ))
}

fn snapshot(
    phase: RoundPhase,
    round_number: u32,
    own: PlayerSnapshot,
    opponent: PlayerSnapshot,
) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number,
        phase,
        timer_remaining_ms: Some(12_000),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

#[derive(Clone, Copy)]
struct EconomyValues {
    gold: u32,
    reserved_gold: u32,
    current_mana: u32,
    reserve_mana: u32,
    mana_cap: u8,
}

impl EconomyValues {
    fn new(
        gold: u32,
        reserved_gold: u32,
        current_mana: u32,
        reserve_mana: u32,
        mana_cap: u8,
    ) -> Self {
        Self {
            gold,
            reserved_gold,
            current_mana,
            reserve_mana,
            mana_cap,
        }
    }
}

fn player_snapshot(
    player_id: PlayerId,
    class_id: ClassId,
    economy: EconomyValues,
    own_destroyed: [bool; HUD_DOTS_PER_ROW],
    opponent_destroyed: [bool; HUD_DOTS_PER_ROW],
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id,
        gold: economy.gold,
        reserved_gold: economy.reserved_gold,
        current_mana: economy.current_mana,
        reserve_mana: economy.reserve_mana,
        spawn_range_cells: 1,
        mana_cap: economy.mana_cap,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: objective_snapshots(own_destroyed),
        opponent_objectives: opponent_objective_snapshots(opponent_destroyed),
    }
}

fn objective_snapshots(destroyed: [bool; HUD_DOTS_PER_ROW]) -> Vec<ObjectiveSnapshot> {
    destroyed
        .into_iter()
        .enumerate()
        .map(|(index, is_destroyed)| ObjectiveSnapshot {
            lane: index as u8 + 1,
            hp: if is_destroyed { 0 } else { 3 },
            is_real: false,
            is_destroyed,
        })
        .collect()
}

fn opponent_objective_snapshots(
    destroyed: [bool; HUD_DOTS_PER_ROW],
) -> Vec<OpponentObjectiveSnapshot> {
    destroyed
        .into_iter()
        .enumerate()
        .map(|(index, is_destroyed)| OpponentObjectiveSnapshot {
            lane: index as u8 + 1,
            hp: if is_destroyed { 0 } else { 3 },
            is_destroyed,
            was_fake: None,
        })
        .collect()
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<PresentationGameSnapshotMessage>>()
        .write(PresentationGameSnapshotMessage(snapshot));
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

fn set_gold_state(app: &mut App, entity: Entity, gold: u32, reserved_gold: u32) {
    let mut state = app
        .world_mut()
        .get_mut::<GoldDisplayState>(entity)
        .expect("gold state should exist");
    state.gold = gold as f32;
    state.reserved_gold = reserved_gold as f32;
    state.is_populated = true;
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn hud_entity_ids(entities: HudEntities) -> Vec<Entity> {
    let mut ids = vec![
        entities.root,
        entities.phase_label,
        entities.round_counter,
        entities.own_gold_parent,
        entities.own_gold_span,
        entities.opponent_gold_parent,
        entities.opponent_gold_span,
        entities.mana_label,
        entities.reserve_label,
    ];
    ids.extend(entities.dots.into_iter().flatten());
    ids
}

fn gold_state(app: &App, entity: Entity) -> GoldDisplayState {
    *app.world()
        .get::<GoldDisplayState>(entity)
        .expect("gold state should exist")
}

fn dot_state(app: &App, entity: Entity) -> ScoreboardDotState {
    *app.world()
        .get::<ScoreboardDotState>(entity)
        .expect("dot should have state")
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

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
