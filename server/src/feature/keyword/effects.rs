use bevy::prelude::*;

pub fn apply_first_strike(_attacker: Entity, _target: Entity, _world: &mut World) {
    todo!()
}

pub fn check_shield_absorb(_unit: Entity, _sub_step: u8, _world: &World) -> bool {
    todo!()
}

pub fn apply_bodyguard_bond(_bodyguard: Entity, _protected: Entity, _world: &mut World) {
    todo!()
}

pub fn apply_repel(_target: Entity, _distance: u8, _world: &mut World) -> u8 {
    todo!()
}

pub fn apply_attract(_caster: Entity, _target: Entity, _distance: u8, _world: &mut World) -> u8 {
    todo!()
}

pub fn apply_teleport(_target: Entity, _lane: u8, _cell: u8, _world: &mut World) {
    todo!()
}

pub fn apply_change_lane(_target: Entity, _lane_delta: i8, _world: &mut World) {
    todo!()
}

pub fn check_irremovable(_target: Entity, _world: &World) -> bool {
    todo!()
}

pub fn check_counterattack_proximity(_world: &World, _defender: Entity, _attacker: Entity) -> bool {
    todo!()
}

pub fn apply_counterattack(
    _world: &mut World,
    _defender: Entity,
    _attacker: Entity,
    _sub_step: u8,
) {
    todo!()
}
