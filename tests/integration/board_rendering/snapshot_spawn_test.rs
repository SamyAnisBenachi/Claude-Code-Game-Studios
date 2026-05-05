use std::time::Duration;

use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_tweening::{lens::TransformPositionLens, PlaybackState, Tween, TweenAnim};
use client::card_animations::{
    make_tween_anim, AnimGroup, AnimQueue, CardAnimationsPlugin, PendingObjectiveDestroyedEvents,
    PendingPhaseChange, StagedObjectiveRevealQueue,
};
use client::presentation::board_rendering::rendering_constants::{
    HEALTH_BAR_LOCAL_Z, Z_HEALTH_BARS, Z_UNITS,
};
use client::presentation::board_rendering::{
    hp_bar_visual, BoardRenderState, BoardRenderingConfig, BoardRenderingEntity,
    BoardRenderingPlugin, BoardSnapshotEntity, BoardUnit, BoardUnitCard, BoardUnitOwner,
    BoardUnitStats, CardAtlas, HpBarColor, HpBarFill, ObjectiveIdentityCache, StandingObjective,
    StandingObjectiveHp, HP_BAR_WHITE_PIXEL_FRAME_INDEX, OBJECTIVE_UNKNOWN_FRAME_INDEX,
    UNIT_PLACEHOLDER_FRAME_INDEX,
};
use client::presentation::LaneCell;
use client::state::{ClientGameSnapshotMessage, ClientState, CurrentClientPhase};
use shared::card::{CardId, ClassId};
use shared::protocol::{
    BoardSnapshot, ObjectiveSnapshot, PlayerSnapshot, RoundPhase, S2CGameSnapshot,
    UnitBoardLocation, UnitBoardState, UnitStatsSnapshot,
};
use shared::session::PlayerId;

const KNOWN_CARD: CardId = CardId(10);
const KNOWN_CARD_FRAME: usize = 7;
const KNOWN_CARD_MAX_HP: u8 = 5;

#[test]
fn test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives() {
    let mut app = app_in_session();
    install_test_atlas(&mut app);

    let stale = app
        .world_mut()
        .spawn((
            BoardRenderingEntity,
            BoardSnapshotEntity,
            Transform::default(),
        ))
        .id();
    let tween_entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            make_tween_anim(transform_tween(Duration::from_secs(1))),
        ))
        .id();
    *app.world_mut().resource_mut::<AnimQueue>() =
        AnimQueue::from_groups(vec![AnimGroup::new(1, 100, Vec::new())]);
    app.world_mut()
        .resource_mut::<PendingPhaseChange>()
        .set_phase(RoundPhase::DraftShop);
    app.world_mut()
        .resource_mut::<PendingObjectiveDestroyedEvents>()
        .push(1, stale);
    app.world_mut()
        .resource_mut::<StagedObjectiveRevealQueue>()
        .push(1, Timer::from_seconds(1.0, TimerMode::Once));
    app.world_mut()
        .resource_mut::<ObjectiveIdentityCache>()
        .insert(player(1), 1, true);

    write_snapshot(
        &mut app,
        snapshot_with_units(
            RoundPhase::Placement,
            vec![
                unit(101, player(1), 2, 3, Some(KNOWN_CARD), 2),
                unit(102, player(2), 4, 6, Some(KNOWN_CARD), 5),
            ],
        ),
    );
    app.update();

    assert!(app.world().get_entity(stale).is_err());
    assert_eq!(app.world().resource::<AnimQueue>().groups.len(), 0);
    assert!(app.world().resource::<PendingPhaseChange>().is_none());
    assert!(app
        .world()
        .resource::<PendingObjectiveDestroyedEvents>()
        .is_empty());
    assert!(app
        .world()
        .resource::<StagedObjectiveRevealQueue>()
        .is_empty());
    assert!(app.world().resource::<ObjectiveIdentityCache>().is_empty());
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::Placement
    );
    assert_eq!(
        *app.world().resource::<BoardRenderState>(),
        BoardRenderState::Placement
    );

    let tween = app.world().get::<TweenAnim>(tween_entity).unwrap();
    assert_eq!(tween.playback_state, PlaybackState::Paused);

    let mut units = app.world_mut().query::<(
        Entity,
        &BoardUnit,
        &BoardUnitOwner,
        &BoardUnitCard,
        &BoardUnitStats,
        &LaneCell,
        &Transform,
        &Sprite,
    )>();
    let unit_rows = units.iter(app.world()).collect::<Vec<_>>();

    assert_eq!(unit_rows.len(), 2);
    assert!(unit_rows.iter().any(
        |(_, unit, owner, card, stats, lane_cell, transform, sprite)| {
            unit.unit_id == 101
                && owner.0 == player(1)
                && card.card_id == Some(KNOWN_CARD)
                && card.frame_index == KNOWN_CARD_FRAME
                && !card.used_missing_art_fallback
                && stats.hp_current == 2
                && stats.hp_max == KNOWN_CARD_MAX_HP
                && *lane_cell == &LaneCell { lane: 2, cell: 3 }
                && transform.translation.z == Z_UNITS
                && sprite.texture_atlas.as_ref().unwrap().index == KNOWN_CARD_FRAME
        }
    ));
    assert!(unit_rows.iter().any(
        |(_, unit, owner, card, stats, lane_cell, transform, sprite)| {
            unit.unit_id == 102
                && owner.0 == player(2)
                && card.card_id == Some(KNOWN_CARD)
                && stats.hp_current == 5
                && *lane_cell == &LaneCell { lane: 4, cell: 6 }
                && transform.translation.z == Z_UNITS
                && sprite.texture_atlas.as_ref().unwrap().index == KNOWN_CARD_FRAME
        }
    ));

    assert_eq!(objective_count(&mut app), 10);
}

#[test]
fn test_hp_bar_fill_thresholds_local_z_and_no_fill_tween() {
    let mut app = app_in_session();
    install_test_atlas(&mut app);

    assert_eq!(
        hp_bar_visual(5, 5, BoardRenderingConfig::default()),
        client::presentation::board_rendering::HpBarVisual {
            fill: 1.0,
            color: HpBarColor::Green,
        }
    );
    assert_eq!(
        hp_bar_visual(2, 5, BoardRenderingConfig::default()).color,
        HpBarColor::Yellow
    );
    assert_eq!(
        hp_bar_visual(1, 5, BoardRenderingConfig::default()).color,
        HpBarColor::Red
    );
    assert_eq!(
        hp_bar_visual(3, 10, BoardRenderingConfig::default()).color,
        HpBarColor::Yellow
    );

    write_snapshot(
        &mut app,
        snapshot_with_units(
            RoundPhase::DraftInitial,
            vec![unit(201, player(1), 1, 1, Some(KNOWN_CARD), 2)],
        ),
    );
    app.update();
    app.update();

    let unit_entity = single_unit(&mut app);
    let fill_entity = hp_fill_child(&app, unit_entity);
    let fill_transform = app.world().get::<Transform>(fill_entity).unwrap();
    let fill_sprite = app.world().get::<Sprite>(fill_entity).unwrap();

    assert!((fill_transform.scale.x - 0.4).abs() <= 0.01);
    assert_eq!(fill_transform.translation.z, HEALTH_BAR_LOCAL_Z);
    assert_ne!(fill_transform.translation.z, Z_HEALTH_BARS);
    assert_eq!(fill_sprite.color, HpBarColor::Yellow.tint());
    assert!(app.world().get::<TweenAnim>(fill_entity).is_none());
    assert_eq!(
        fill_sprite.texture_atlas.as_ref().unwrap().index,
        HP_BAR_WHITE_PIXEL_FRAME_INDEX
    );
}

#[test]
fn test_standing_objectives_use_unknown_frame_and_no_identity_components() {
    #[derive(Component)]
    struct RealObjective;
    #[derive(Component)]
    struct FakeObjective;
    #[derive(Component)]
    struct ObjectiveIdentity;
    #[derive(Component)]
    struct IsKnown;
    #[derive(Component)]
    struct IsTrue;

    let mut app = app_in_session();
    install_test_atlas(&mut app);

    let scratch = app
        .world_mut()
        .spawn((
            RealObjective,
            FakeObjective,
            ObjectiveIdentity,
            IsKnown,
            IsTrue,
        ))
        .id();

    write_snapshot(
        &mut app,
        snapshot_with_units(RoundPhase::DraftInitial, Vec::new()),
    );
    app.update();

    let mut objectives = app.world_mut().query::<(
        Entity,
        &StandingObjective,
        &StandingObjectiveHp,
        &LaneCell,
        &Sprite,
    )>();
    let rows = objectives.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 10);

    for (entity, _objective, _hp, _lane_cell, sprite) in rows {
        assert_eq!(
            sprite.texture_atlas.as_ref().unwrap().index,
            OBJECTIVE_UNKNOWN_FRAME_INDEX
        );
        let entity_ref = app.world().entity(entity);
        assert!(entity_ref.get::<RealObjective>().is_none());
        assert!(entity_ref.get::<FakeObjective>().is_none());
        assert!(entity_ref.get::<ObjectiveIdentity>().is_none());
        assert!(entity_ref.get::<IsKnown>().is_none());
        assert!(entity_ref.get::<IsTrue>().is_none());
    }

    app.world_mut().entity_mut(scratch).despawn();
}

#[test]
fn test_missing_card_art_uses_placeholder_and_keeps_hp_bar() {
    let mut app = app_in_session();
    install_test_atlas(&mut app);

    write_snapshot(
        &mut app,
        snapshot_with_units(
            RoundPhase::DraftInitial,
            vec![unit(301, player(1), 3, 4, Some(CardId(9999)), 3)],
        ),
    );
    app.update();

    let mut units = app
        .world_mut()
        .query_filtered::<(Entity, &BoardUnitCard, &Sprite), With<BoardUnit>>();
    let rows = units.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);

    let (unit_entity, card, sprite) = rows[0];
    assert_eq!(card.card_id, Some(CardId(9999)));
    assert!(card.used_missing_art_fallback);
    assert_eq!(
        sprite.texture_atlas.as_ref().unwrap().index,
        UNIT_PLACEHOLDER_FRAME_INDEX
    );
    assert!(app
        .world()
        .entity(unit_entity)
        .get::<Children>()
        .unwrap()
        .iter()
        .any(|child| app.world().get::<HpBarFill>(child).is_some()));
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(CardAnimationsPlugin);
    app.add_plugins(BoardRenderingPlugin);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn install_test_atlas(app: &mut App) {
    *app.world_mut().resource_mut::<CardAtlas>() =
        CardAtlas::default().with_unit_frame(KNOWN_CARD, KNOWN_CARD_FRAME, KNOWN_CARD_MAX_HP);
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<ClientGameSnapshotMessage>>()
        .write(ClientGameSnapshotMessage(snapshot));
}

fn snapshot_with_units(phase: RoundPhase, units: Vec<UnitBoardState>) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number: 4,
        phase,
        timer_remaining_ms: Some(20_000),
        players: vec![player_snapshot(player(1)), player_snapshot(player(2))],
        board: BoardSnapshot { units, ..default() },
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(player_id: PlayerId) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold: 0,
        reserved_gold: 0,
        current_mana: 0,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 1,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: objectives([true, false, true, false, true]),
        opponent_objectives: Vec::new(),
    }
}

fn objectives(real_flags: [bool; 5]) -> Vec<ObjectiveSnapshot> {
    real_flags
        .into_iter()
        .enumerate()
        .map(|(index, is_real)| ObjectiveSnapshot {
            lane: index as u8 + 1,
            hp: 5,
            is_real,
            is_destroyed: false,
        })
        .collect()
}

fn unit(
    unit_id: u64,
    owner_id: PlayerId,
    lane: u8,
    cell: u8,
    card_id: Option<CardId>,
    hp: u8,
) -> UnitBoardState {
    UnitBoardState {
        unit_id,
        owner_id,
        location: UnitBoardLocation::BoardCell { lane, cell },
        card_id,
        stats: Some(UnitStatsSnapshot {
            hp,
            atk: 2,
            mp: 3,
            ar: 0,
        }),
        source_class: None,
    }
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn transform_tween(duration: Duration) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        duration,
        TransformPositionLens {
            start: Vec3::ZERO,
            end: Vec3::X,
        },
    )
}

fn single_unit(app: &mut App) -> Entity {
    let mut query = app.world_mut().query_filtered::<Entity, With<BoardUnit>>();
    let rows = query.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    rows[0]
}

fn hp_fill_child(app: &App, unit: Entity) -> Entity {
    app.world()
        .entity(unit)
        .get::<Children>()
        .unwrap()
        .iter()
        .find(|child| app.world().get::<HpBarFill>(*child).is_some())
        .expect("unit should have HP fill child")
}

fn objective_count(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<StandingObjective>>();
    query.iter(app.world()).count()
}
