use bevy::prelude::{Entity, Or, With, World};
use shared::protocol::{EntityId, UnitBoardLocation, UnitBoardState, UnitStatsSnapshot};
use shared::session::PlayerId;

use crate::core::board::{
    BoardPosition, ObjectiveAttachment, SeedOwner, SourceClass, UnitOwner, UnitStats,
};

pub fn build_unit_board_state(entity: Entity, world: &World) -> Option<UnitBoardState> {
    let owner_id = owner_id(entity, world)?;
    let location = location(entity, world)?;
    let stats = world
        .get::<UnitStats>(entity)
        .map(|stats| UnitStatsSnapshot {
            hp: stats.hp,
            atk: stats.atk,
            mp: stats.mp,
            ar: stats.ar,
        });
    let source_class = world.get::<SourceClass>(entity).map(|source| source.0);

    Some(UnitBoardState {
        unit_id: entity.to_bits() as EntityId,
        owner_id,
        location,
        stats,
        source_class,
    })
}

pub fn build_unit_board_states(world: &mut World) -> Vec<UnitBoardState> {
    let mut query =
        world.query_filtered::<Entity, Or<(With<BoardPosition>, With<ObjectiveAttachment>)>>();
    let entities = query.iter(world).collect::<Vec<_>>();
    let mut units = entities
        .into_iter()
        .filter_map(|entity| build_unit_board_state(entity, world))
        .collect::<Vec<_>>();
    units.sort_by_key(|unit| unit.unit_id);
    units
}

fn owner_id(entity: Entity, world: &World) -> Option<PlayerId> {
    world
        .get::<UnitOwner>(entity)
        .map(|owner| owner.0)
        .or_else(|| world.get::<SeedOwner>(entity).map(|owner| owner.0))
}

fn location(entity: Entity, world: &World) -> Option<UnitBoardLocation> {
    if let Some(position) = world.get::<BoardPosition>(entity) {
        return Some(UnitBoardLocation::BoardCell {
            lane: position.lane,
            cell: position.cell,
        });
    }

    world.get::<ObjectiveAttachment>(entity).map(|attachment| {
        UnitBoardLocation::ObjectiveAttachment {
            lane: attachment.lane,
        }
    })
}
