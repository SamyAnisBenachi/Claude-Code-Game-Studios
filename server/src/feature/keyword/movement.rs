#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerSide {
    PlayerA,
    PlayerB,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttractCollisionRule {
    AllowCoLocation,
    StopOneCellShort,
}

const MIN_CELL: i32 = 1;
const MAX_CELL: i32 = 8;

impl PlayerSide {
    fn advance_dir(self) -> i32 {
        match self {
            PlayerSide::PlayerA => 1,
            PlayerSide::PlayerB => -1,
        }
    }
}

fn clamp_cell(cell: i32) -> u8 {
    cell.clamp(MIN_CELL, MAX_CELL) as u8
}

/// Formula 1 from keyword-system.md.
///
/// REPEL pushes a unit toward its own side, which is the opposite of its
/// normal advance direction. The intermediate value must stay signed so
/// edge pushes cannot underflow `u8`.
pub fn repel_destination(target_cell: u8, owner: PlayerSide, x: u8) -> u8 {
    let push_direction = -owner.advance_dir();
    let destination = target_cell as i32 + push_direction * x as i32;

    clamp_cell(destination)
}

/// Standard forward movement using the same signed arithmetic as board F1.
pub fn advance_destination(current_cell: u8, owner: PlayerSide, cells: u8) -> u8 {
    let destination = current_cell as i32 + owner.advance_dir() * cells as i32;

    clamp_cell(destination)
}

/// Formula 2 friendly/default form from ADR-018.
///
/// Friendly targets may stop on the caster's cell. Enemy targets should use
/// `attract_enemy_destination`, which applies the one-cell-apart collision rule
/// from the current GDD.
pub fn attract_destination(caster_cell: u8, target_cell: u8, x: u8) -> u8 {
    attract_destination_with_rule(
        caster_cell,
        target_cell,
        x,
        AttractCollisionRule::AllowCoLocation,
    )
}

/// Formula 2 enemy-target form from keyword-system.md.
///
/// Opposing units can never occupy the same cell, so the target stops one cell
/// short of the caster when pulled as far as possible.
pub fn attract_enemy_destination(caster_cell: u8, target_cell: u8, x: u8) -> u8 {
    attract_destination_with_rule(
        caster_cell,
        target_cell,
        x,
        AttractCollisionRule::StopOneCellShort,
    )
}

pub fn attract_destination_with_rule(
    caster_cell: u8,
    target_cell: u8,
    x: u8,
    collision_rule: AttractCollisionRule,
) -> u8 {
    let caster = caster_cell as i32;
    let target = target_cell as i32;
    let distance = (caster - target).abs();
    let max_pull = match collision_rule {
        AttractCollisionRule::AllowCoLocation => distance,
        AttractCollisionRule::StopOneCellShort => (distance - 1).max(0),
    };
    let effective_pull = (x as i32).min(max_pull);
    let direction = (caster - target).signum();

    clamp_cell(target + direction * effective_pull)
}
