use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use server::core::board::{
    build_unit_board_state, spawn_chacha_noir, spawn_la_gonflable, spawn_la_sacrifiee,
    spawn_madoll, spawn_mummy, spawn_seed, spawn_sinistro, BoardPosition, ClassTokenKind,
    ObjectiveAttachment, SeedMarker, SeedOwner, SourceClass, TokenUnit, UnitOwner, UnitStats,
};
use server::core::session::{build_snapshot, PlayerSessionData, PlayerSessions};
use shared::card::ClassId;
use shared::protocol::{UnitBoardLocation, UnitStatsSnapshot};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn spawn_with_commands(world: &mut World, spawn: impl FnOnce(&mut Commands) -> Entity) -> Entity {
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);
    let entity = spawn(&mut commands);
    queue.apply(world);
    entity
}

fn assert_token(
    world: &World,
    entity: Entity,
    kind: ClassTokenKind,
    source_class: ClassId,
    stats: Option<UnitStats>,
) {
    assert_eq!(world.get::<ClassTokenKind>(entity), Some(&kind));
    assert_eq!(
        world.get::<SourceClass>(entity),
        Some(&SourceClass(source_class))
    );
    assert!(world.get::<TokenUnit>(entity).is_some());

    match stats {
        Some(expected) => assert_eq!(world.get::<UnitStats>(entity), Some(&expected)),
        None => assert!(world.get::<UnitStats>(entity).is_none()),
    }
}

#[test]
fn test_all_token_spawns_have_source_class_and_marker() {
    let owner = player(1);
    let mut world = World::new();

    let mummy = spawn_with_commands(&mut world, |commands| spawn_mummy(commands, owner, 1, 2));
    let chacha = spawn_with_commands(&mut world, |commands| {
        spawn_chacha_noir(commands, owner, 2, 3)
    });
    let seed = spawn_with_commands(&mut world, |commands| spawn_seed(commands, owner, 3, 4));
    let madoll = spawn_with_commands(&mut world, |commands| spawn_madoll(commands, owner, 4, 5));
    let gonflable = spawn_with_commands(&mut world, |commands| {
        spawn_la_gonflable(commands, owner, 5, 6)
    });
    let sacrifiee = spawn_with_commands(&mut world, |commands| {
        spawn_la_sacrifiee(commands, owner, 1, 7)
    });
    let sinistro = spawn_with_commands(&mut world, |commands| spawn_sinistro(commands, owner, 2));

    assert_token(
        &world,
        mummy,
        ClassTokenKind::Mummy,
        ClassId::Xelor,
        Some(UnitStats::new(2, 2, 3, 0)),
    );
    assert_token(
        &world,
        chacha,
        ClassTokenKind::ChachaNoir,
        ClassId::Ecaflip,
        Some(UnitStats::new(2, 2, 6, 0)),
    );
    assert_token(&world, seed, ClassTokenKind::Seed, ClassId::Sadida, None);
    assert_token(
        &world,
        madoll,
        ClassTokenKind::Madoll,
        ClassId::Sadida,
        Some(UnitStats::new(3, 1, 3, 0)),
    );
    assert_token(
        &world,
        gonflable,
        ClassTokenKind::LaGonflable,
        ClassId::Sadida,
        Some(UnitStats::new(3, 2, 3, 0)),
    );
    assert_token(
        &world,
        sacrifiee,
        ClassTokenKind::LaSacrifiee,
        ClassId::Sadida,
        Some(UnitStats::new(2, 2, 3, 0)),
    );
    assert_token(
        &world,
        sinistro,
        ClassTokenKind::Sinistro,
        ClassId::Xelor,
        None,
    );

    let mut tokens = world.query::<&TokenUnit>();
    assert_eq!(tokens.iter(&world).count(), 7);
    assert!(world.get::<SeedMarker>(seed).is_some());
    assert_eq!(world.get::<SeedOwner>(seed), Some(&SeedOwner(owner)));
    assert_eq!(
        world.get::<ObjectiveAttachment>(sinistro),
        Some(&ObjectiveAttachment { lane: 2 })
    );
}

#[test]
fn test_standard_unit_has_no_source_class() {
    let mut world = World::new();
    let entity = world
        .spawn((
            UnitStats::new(4, 3, 2, 1),
            BoardPosition { lane: 2, cell: 3 },
            UnitOwner(player(1)),
        ))
        .id();

    assert!(world.get::<SourceClass>(entity).is_none());

    let snapshot = build_unit_board_state(entity, &world).expect("standard unit snapshots");
    assert_eq!(snapshot.source_class, None);
}

#[test]
fn test_unit_board_state_derives_source_class_for_tokens() {
    let owner = player(1);
    let mut world = World::new();
    let entity = spawn_with_commands(&mut world, |commands| spawn_madoll(commands, owner, 3, 2));

    let snapshot = build_unit_board_state(entity, &world).expect("token snapshots");

    assert_eq!(snapshot.owner_id, owner);
    assert_eq!(
        snapshot.location,
        UnitBoardLocation::BoardCell { lane: 3, cell: 2 }
    );
    assert_eq!(
        snapshot.stats,
        Some(UnitStatsSnapshot {
            hp: 3,
            atk: 1,
            mp: 3,
            ar: 0,
        })
    );
    assert_eq!(snapshot.source_class, Some(ClassId::Sadida));
}

#[test]
fn test_game_snapshot_includes_token_source_class() {
    let player_a = player(1);
    let mut world = World::new();
    let token = spawn_with_commands(&mut world, |commands| spawn_mummy(commands, player_a, 1, 1));
    let standard = world
        .spawn((
            UnitStats::new(4, 2, 3, 0),
            BoardPosition { lane: 2, cell: 2 },
            UnitOwner(player_a),
        ))
        .id();
    let mut sessions = PlayerSessions::default();
    sessions.players.insert(
        player_a,
        PlayerSessionData {
            class: ClassId::Xelor,
            class_locked: true,
        },
    );
    world.insert_resource(sessions);

    let snapshot = build_snapshot(player_a, &mut world).expect("snapshot builds");
    let token_state = snapshot
        .board
        .units
        .iter()
        .find(|unit| unit.unit_id == token.to_bits())
        .expect("token state exists");
    let standard_state = snapshot
        .board
        .units
        .iter()
        .find(|unit| unit.unit_id == standard.to_bits())
        .expect("standard state exists");

    assert_eq!(token_state.source_class, Some(ClassId::Xelor));
    assert_eq!(standard_state.source_class, None);
}

#[test]
fn test_miranda_control_transfer_does_not_change_source_class() {
    let player_a = player(1);
    let player_b = player(2);
    let mut world = World::new();
    let token = spawn_with_commands(&mut world, |commands| {
        spawn_madoll(commands, player_a, 4, 2)
    });

    world.entity_mut(token).insert(UnitOwner(player_b));

    assert_eq!(world.get::<UnitOwner>(token), Some(&UnitOwner(player_b)));
    assert_eq!(
        world.get::<SourceClass>(token),
        Some(&SourceClass(ClassId::Sadida))
    );
    let snapshot = build_unit_board_state(token, &world).expect("token snapshots");
    assert_eq!(snapshot.owner_id, player_b);
    assert_eq!(snapshot.source_class, Some(ClassId::Sadida));
}
