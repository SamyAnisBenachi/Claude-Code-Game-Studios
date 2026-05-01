use bevy::prelude::{Commands, Entity};
use shared::card::ClassId;
use shared::session::PlayerId;

use crate::core::board::{
    BoardPosition, ClassTokenKind, ObjectiveAttachment, SeedMarker, SeedOwner, SourceClass,
    TokenUnit, UnitOwner, UnitStats,
};

pub fn spawn_mummy(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    commands
        .spawn((
            UnitStats::new(2, 2, 3, 0),
            BoardPosition { lane, cell },
            UnitOwner(owner),
            SourceClass(ClassId::Xelor),
            TokenUnit,
            ClassTokenKind::Mummy,
        ))
        .id()
}

pub fn spawn_chacha_noir(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    commands
        .spawn((
            UnitStats::new(2, 2, 6, 0),
            BoardPosition { lane, cell },
            UnitOwner(owner),
            SourceClass(ClassId::Ecaflip),
            TokenUnit,
            ClassTokenKind::ChachaNoir,
        ))
        .id()
}

pub fn spawn_seed(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    commands
        .spawn((
            SeedMarker,
            BoardPosition { lane, cell },
            SeedOwner(owner),
            SourceClass(ClassId::Sadida),
            TokenUnit,
            ClassTokenKind::Seed,
        ))
        .id()
}

pub fn spawn_madoll(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    commands
        .spawn((
            UnitStats::new(3, 1, 3, 0),
            BoardPosition { lane, cell },
            UnitOwner(owner),
            SourceClass(ClassId::Sadida),
            TokenUnit,
            ClassTokenKind::Madoll,
        ))
        .id()
}

pub fn spawn_la_gonflable(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    commands
        .spawn((
            UnitStats::new(3, 2, 3, 0),
            BoardPosition { lane, cell },
            UnitOwner(owner),
            SourceClass(ClassId::Sadida),
            TokenUnit,
            ClassTokenKind::LaGonflable,
        ))
        .id()
}

pub fn spawn_la_sacrifiee(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    commands
        .spawn((
            UnitStats::new(2, 2, 3, 0),
            BoardPosition { lane, cell },
            UnitOwner(owner),
            SourceClass(ClassId::Sadida),
            TokenUnit,
            ClassTokenKind::LaSacrifiee,
        ))
        .id()
}

pub fn spawn_sinistro(commands: &mut Commands, owner: PlayerId, objective_lane: u8) -> Entity {
    commands
        .spawn((
            ObjectiveAttachment {
                lane: objective_lane,
            },
            UnitOwner(owner),
            SourceClass(ClassId::Xelor),
            TokenUnit,
            ClassTokenKind::Sinistro,
        ))
        .id()
}
