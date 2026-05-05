use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::board_rendering::rendering_constants::{
    STATUS_ICON_LOCAL_Z, STATUS_ICON_TOP_RIGHT_X_OFFSET, Z_UNITS,
};
use client::presentation::board_rendering::{
    co_occupancy_offset, status_icon_slot_translation, BoardRenderingPlugin, BoardUnit,
    BoardUnitCard, BoardUnitOwner, BoardUnitStats, CardAtlas, PlayerTeamMap, StatusEffectKey,
    StatusEffectVisual, StatusEffectsList, StatusIcon, StatusOverflowBadge,
};
use client::presentation::LaneCell;
use client::state::{ClientGameSnapshotMessage, ClientState};
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
fn test_status_icons_sort_by_tier_duration_and_overflow() {
    let mut app = app_in_session();
    install_test_atlas(&mut app);

    let unit = spawn_unit_with_statuses(
        &mut app,
        vec![
            StatusEffectVisual::timed(StatusEffectKey::Haste, 1),
            StatusEffectVisual::timed(StatusEffectKey::Leader, 2),
            StatusEffectVisual::timed(StatusEffectKey::Stun, 3),
            StatusEffectVisual::untimed(StatusEffectKey::Shield),
        ],
    );
    app.update();

    let icons = status_icons(&mut app, unit);
    assert_eq!(icons.len(), 3);
    assert_eq!(icons[0].key, StatusEffectKey::Shield);
    assert_eq!(icons[0].slot, 0);
    assert_eq!(icons[0].display_tier, 1);
    assert_eq!(icons[1].key, StatusEffectKey::Stun);
    assert_eq!(icons[1].slot, 1);
    assert_eq!(icons[2].key, StatusEffectKey::Leader);
    assert_eq!(icons[2].slot, 2);

    let overflow = overflow_badges(&mut app, unit);
    assert_eq!(overflow.len(), 1);
    assert_eq!(overflow[0].slot, 3);
    assert_eq!(overflow[0].hidden_count, 1);

    for (index, icon) in status_icon_entities(&mut app, unit).into_iter().enumerate() {
        let transform = app.world().get::<Transform>(icon).unwrap();
        let expected = status_icon_slot_translation(index as u8);
        assert!((transform.translation.x - expected.x).abs() <= 0.5);
        assert!((transform.translation.y - expected.y).abs() <= 0.5);
        assert_eq!(transform.translation.z, STATUS_ICON_LOCAL_Z);

        let sprite = app.world().get::<Sprite>(icon).unwrap();
        let atlas = app.world().resource::<CardAtlas>();
        assert_eq!(sprite.image, atlas.board_elements_image);
        assert_eq!(
            sprite.texture_atlas.as_ref().unwrap().layout,
            atlas.board_elements_layout
        );
    }
}

#[test]
fn test_tier_two_equal_duration_sorts_deterministically_by_status_key() {
    let mut app = app_in_session();
    install_test_atlas(&mut app);

    let unit = spawn_unit_with_statuses(
        &mut app,
        vec![
            StatusEffectVisual::untimed(StatusEffectKey::Haste),
            StatusEffectVisual::untimed(StatusEffectKey::Silence),
            StatusEffectVisual::untimed(StatusEffectKey::Stun),
        ],
    );
    app.update();

    let icons = status_icons(&mut app, unit);
    assert_eq!(icons.len(), 3);
    assert_eq!(icons[0].key, StatusEffectKey::Stun);
    assert_eq!(icons[1].key, StatusEffectKey::Silence);
    assert_eq!(icons[2].key, StatusEffectKey::Haste);
}

#[test]
fn test_snapshot_cooccupancy_offsets_allied_units_by_unit_id() {
    let mut app = app_in_session();
    install_test_atlas(&mut app);
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(player(1), 0);
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(player(2), 0);

    write_snapshot(
        &mut app,
        snapshot_with_units(vec![
            unit_state(20, player(2), 2, 4),
            unit_state(10, player(1), 2, 4),
        ]),
    );
    app.update();

    let layout = *app.world().resource::<client::presentation::BoardLayout>();
    let cell_center = layout.cell_to_world(2, 4);
    let first = unit_transform(&mut app, 10);
    let second = unit_transform(&mut app, 20);

    assert!((first.translation.x - (cell_center.x - 4.0)).abs() <= 0.01);
    assert!((second.translation.x - (cell_center.x + 4.0)).abs() <= 0.01);
    assert!((second.translation.x - first.translation.x - 8.0).abs() <= 0.01);
}

#[test]
fn test_status_icon_global_x_inherits_cooccupancy_parent_offset() {
    let mut app = app_in_session();
    install_test_atlas(&mut app);
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(player(1), 0);
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(player(2), 0);

    write_snapshot(
        &mut app,
        snapshot_with_units(vec![
            unit_state(10, player(1), 2, 4),
            unit_state(20, player(2), 2, 4),
        ]),
    );
    app.update();

    let unit = unit_entity(&mut app, 20);
    app.world_mut()
        .entity_mut(unit)
        .insert(StatusEffectsList::new(vec![StatusEffectVisual::untimed(
            StatusEffectKey::Outnumbered,
        )]));
    app.update();
    app.update();
    app.update();

    let icon = status_icon_entities(&mut app, unit)[0];
    let unit_x = app.world().get::<Transform>(unit).unwrap().translation.x;
    let icon_local_x = app.world().get::<Transform>(icon).unwrap().translation.x;
    let child_parent = app.world().get::<ChildOf>(icon).unwrap().parent();
    let derived_icon_world_x = unit_x + icon_local_x;
    let first_unit_x = unit_transform(&mut app, 10).translation.x;

    assert!((unit_x - (first_unit_x + 8.0)).abs() <= 0.01);
    assert!((icon_local_x - STATUS_ICON_TOP_RIGHT_X_OFFSET).abs() <= 0.01);
    assert_eq!(child_parent, unit);
    assert!((derived_icon_world_x - (unit_x + STATUS_ICON_TOP_RIGHT_X_OFFSET)).abs() <= 0.01);
}

#[test]
#[should_panic(expected = "unit_index=2")]
fn test_cooccupancy_index_two_panics_with_offending_index() {
    let _ = co_occupancy_offset(2, 8.0);
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(TransformPlugin);
    app.add_plugins(StatesPlugin);
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

fn spawn_unit_with_statuses(app: &mut App, effects: Vec<StatusEffectVisual>) -> Entity {
    app.world_mut()
        .spawn((
            BoardUnit { unit_id: 1 },
            BoardUnitOwner(player(1)),
            BoardUnitCard {
                card_id: Some(KNOWN_CARD),
                frame_index: KNOWN_CARD_FRAME,
                used_missing_art_fallback: false,
            },
            BoardUnitStats {
                hp_current: 5,
                hp_max: 5,
                atk: 1,
                mp: 1,
                ar: 0,
            },
            LaneCell { lane: 1, cell: 1 },
            StatusEffectsList::new(effects),
            Transform::from_xyz(0.0, 0.0, Z_UNITS),
        ))
        .id()
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<ClientGameSnapshotMessage>>()
        .write(ClientGameSnapshotMessage(snapshot));
}

fn snapshot_with_units(units: Vec<UnitBoardState>) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number: 4,
        phase: RoundPhase::Placement,
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
        objectives: objectives(),
        opponent_objectives: Vec::new(),
    }
}

fn objectives() -> Vec<ObjectiveSnapshot> {
    (1..=5)
        .map(|lane| ObjectiveSnapshot {
            lane,
            hp: 5,
            is_real: true,
            is_destroyed: false,
        })
        .collect()
}

fn unit_state(unit_id: u64, owner_id: PlayerId, lane: u8, cell: u8) -> UnitBoardState {
    UnitBoardState {
        unit_id,
        owner_id,
        location: UnitBoardLocation::BoardCell { lane, cell },
        card_id: Some(KNOWN_CARD),
        stats: Some(UnitStatsSnapshot {
            hp: 5,
            atk: 2,
            mp: 3,
            ar: 0,
        }),
        source_class: None,
    }
}

fn status_icons(app: &mut App, unit: Entity) -> Vec<StatusIcon> {
    let mut icons = status_icon_entities(app, unit)
        .into_iter()
        .map(|entity| *app.world().get::<StatusIcon>(entity).unwrap())
        .collect::<Vec<_>>();
    icons.sort_by_key(|icon| icon.slot);
    icons
}

fn status_icon_entities(app: &mut App, unit: Entity) -> Vec<Entity> {
    let children = app.world().entity(unit).get::<Children>().unwrap();
    let mut icons = children
        .iter()
        .filter(|child| app.world().get::<StatusIcon>(*child).is_some())
        .collect::<Vec<_>>();
    icons.sort_by_key(|entity| app.world().get::<StatusIcon>(*entity).unwrap().slot);
    icons
}

fn overflow_badges(app: &mut App, unit: Entity) -> Vec<StatusOverflowBadge> {
    app.world()
        .entity(unit)
        .get::<Children>()
        .unwrap()
        .iter()
        .filter_map(|child| app.world().get::<StatusOverflowBadge>(child).copied())
        .collect()
}

fn unit_entity(app: &mut App, unit_id: u64) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &BoardUnit)>();
    query
        .iter(app.world())
        .find_map(|(entity, unit)| (unit.unit_id == unit_id).then_some(entity))
        .expect("unit should exist")
}

fn unit_transform(app: &mut App, unit_id: u64) -> Transform {
    let entity = unit_entity(app, unit_id);
    *app.world().get::<Transform>(entity).unwrap()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
