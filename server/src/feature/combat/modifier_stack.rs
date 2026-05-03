use shared::card::{Keyword, SimpleKeyword, UnitType};
use shared::config::GameConfig;
use shared::protocol::EntityId;
use shared::session::PlayerId;

/// Unit stats frozen for a combat-resolution calculation.
///
/// The snapshot is plain data so modifier-stack tests can run without Bevy
/// `World` access. Future sub-step stories can build this from ECS components
/// at RESOLUTION entry.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitSnapshot {
    pub unit_id: EntityId,
    pub player: PlayerId,
    pub lane: u8,
    pub cell: u8,
    pub atk: u8,
    pub hp: u8,
    pub ar: u8,
    pub mp: u8,
    pub unit_type: UnitType,
    pub keywords: Vec<Keyword>,
    pub leader_atk_bonus: u8,
}

/// Result of one attacker-versus-defender modifier-stack pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatResult {
    pub net_damage: u8,
    pub ar_attacker_combat: u8,
}

/// Applies the combat modifier stack for one individual attack.
///
/// This function is intentionally pure: it reads only the two snapshots and
/// immutable config values, performs all arithmetic in `i32`, and returns the
/// computed damage plus the attacker's combat-only type-advantage AR bonus.
pub fn apply_combat_modifier_stack(
    attacker: &UnitSnapshot,
    defender: &UnitSnapshot,
    config: &GameConfig,
) -> CombatResult {
    let attacker_silenced = has_simple_keyword(attacker, SimpleKeyword::Silence);

    if !attacker_silenced && has_simple_keyword(attacker, SimpleKeyword::Stun) {
        return CombatResult {
            net_damage: 0,
            ar_attacker_combat: 0,
        };
    }

    let has_type_advantage = type_beats(attacker.unit_type, defender.unit_type);
    let atk_type_bonus = if has_type_advantage {
        config.type_advantage_atk_bonus as i32
    } else {
        0
    };
    let ar_attacker_combat = if has_type_advantage {
        config.type_advantage_ar_bonus
    } else {
        0
    };

    let atk_effective = clamp_to_u8(
        attacker.atk as i32
            + attacker.leader_atk_bonus as i32
            + atk_type_bonus
            + vulnerability_total(defender)
            - resistance_total(defender),
    );
    let armor_piercing =
        !attacker_silenced && has_simple_keyword(attacker, SimpleKeyword::ArmorPiercing);
    let ar_effective = if armor_piercing { 0 } else { defender.ar };
    let net_damage = clamp_to_u8(atk_effective as i32 - ar_effective as i32);

    CombatResult {
        net_damage,
        ar_attacker_combat,
    }
}

fn has_simple_keyword(unit: &UnitSnapshot, simple: SimpleKeyword) -> bool {
    unit.keywords
        .iter()
        .any(|keyword| matches!(keyword, Keyword::Simple(value) if *value == simple))
}

fn resistance_total(unit: &UnitSnapshot) -> i32 {
    unit.keywords
        .iter()
        .map(|keyword| match keyword {
            Keyword::ResistanceX { value } => *value as i32,
            _ => 0,
        })
        .sum()
}

fn vulnerability_total(unit: &UnitSnapshot) -> i32 {
    unit.keywords
        .iter()
        .map(|keyword| match keyword {
            Keyword::VulnerabilityX { value } => *value as i32,
            _ => 0,
        })
        .sum()
}

fn clamp_to_u8(value: i32) -> u8 {
    value.clamp(0, u8::MAX as i32) as u8
}

fn type_beats(attacker: UnitType, defender: UnitType) -> bool {
    matches!(
        (attacker, defender),
        (UnitType::Blade, UnitType::Arcane)
            | (UnitType::Arcane, UnitType::Shield)
            | (UnitType::Shield, UnitType::Blade)
    )
}
