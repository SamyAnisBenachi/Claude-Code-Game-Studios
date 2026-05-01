use bevy::prelude::Component;
use shared::card::{CardId, ClassId};
use shared::session::PlayerId;

/// Absolute board cell occupied by a live unit or cell marker.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardPosition {
    pub lane: u8,
    pub cell: u8,
}

/// Objective lane attachment for non-cell token entities such as Sinistro.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectiveAttachment {
    pub lane: u8,
}

/// Current controller for a unit-like board entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitOwner(pub PlayerId);

/// Card definition backing a live board unit.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitCardRef(pub CardId);

/// Current controller for a seed marker.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeedOwner(pub PlayerId);

/// Logical combat stats for unit tokens.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitStats {
    pub hp: u8,
    pub atk: u8,
    pub mp: u8,
    pub ar: u8,
}

impl UnitStats {
    pub const fn new(hp: u8, atk: u8, mp: u8, ar: u8) -> Self {
        Self { hp, atk, mp, ar }
    }
}

/// Identifies the class that spawned this token entity.
///
/// Set at spawn time. Never mutated. Absent on non-token units.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceClass(pub ClassId);

/// Marker present on all class token entities.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenUnit;

/// Marker for Sadida seed cell hazards.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeedMarker;

/// Stable token kind marker for tests and later class-token behavior stories.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassTokenKind {
    Mummy,
    ChachaNoir,
    Seed,
    Madoll,
    LaGonflable,
    LaSacrifiee,
    Sinistro,
}
