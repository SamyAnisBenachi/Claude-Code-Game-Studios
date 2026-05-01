#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerSide {
    PlayerA,
    PlayerB,
}

pub fn repel_destination(_target_cell: u8, _owner: PlayerSide, _x: u8) -> u8 {
    todo!()
}

pub fn attract_destination(_caster_cell: u8, _target_cell: u8, _x: u8) -> u8 {
    todo!()
}
