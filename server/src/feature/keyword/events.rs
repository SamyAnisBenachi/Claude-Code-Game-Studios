use bevy::prelude::*;
use shared::keyword::KeywordPayload;
use shared::protocol::EntityId;

#[derive(Event, Clone, Debug)]
pub struct UnitAppeared {
    pub sub_step: u8,
}

#[derive(Event, Clone, Debug)]
pub struct UnitDied {
    pub attacker: Option<Entity>,
}

#[derive(Event, Clone, Debug)]
pub struct FinalBlowDealt {
    pub killed: Entity,
    pub sub_step: u8,
}

#[derive(Event, Clone, Debug)]
pub struct StartOfTurnTriggered;

#[derive(Event, Clone, Debug)]
pub struct EndOfTurnTriggered;

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct KeywordTriggered {
    pub source_unit_id: Option<EntityId>,
    pub sub_step: u8,
    pub payload: KeywordPayload,
}
