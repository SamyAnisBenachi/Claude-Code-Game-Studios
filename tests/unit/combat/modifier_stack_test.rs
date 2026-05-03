use server::feature::combat::modifier_stack::{
    apply_combat_modifier_stack, CombatResult, UnitSnapshot,
};
use shared::card::{Keyword, SimpleKeyword, UnitType};
use shared::config::GameConfig;
use shared::session::PlayerId;

fn config() -> GameConfig {
    GameConfig::default()
}

fn simple(keyword: SimpleKeyword) -> Keyword {
    Keyword::Simple(keyword)
}

fn unit(atk: u8, ar: u8, unit_type: UnitType, keywords: Vec<Keyword>) -> UnitSnapshot {
    UnitSnapshot {
        unit_id: 1,
        player: PlayerId(0),
        lane: 1,
        cell: 1,
        atk,
        hp: 10,
        ar,
        mp: 0,
        unit_type,
        keywords,
        leader_atk_bonus: 0,
    }
}

#[test]
fn test_cr12_damage_floor_returns_zero() {
    let attacker = unit(3, 0, UnitType::Neutral, vec![]);
    let defender = unit(0, 5, UnitType::Neutral, vec![]);

    let result = apply_combat_modifier_stack(&attacker, &defender, &config());

    assert_eq!(
        result,
        CombatResult {
            net_damage: 0,
            ar_attacker_combat: 0,
        }
    );
}

#[test]
fn test_cr13_resistance_reduces_attack_before_armor() {
    let attacker = unit(4, 0, UnitType::Neutral, vec![]);
    let defender = unit(
        0,
        1,
        UnitType::Neutral,
        vec![Keyword::ResistanceX { value: 2 }],
    );

    let result = apply_combat_modifier_stack(&attacker, &defender, &config());

    assert_eq!(result.net_damage, 1);
}

#[test]
fn test_cr14_armor_piercing_ignores_armor_after_resistance() {
    let attacker = unit(
        3,
        0,
        UnitType::Neutral,
        vec![simple(SimpleKeyword::ArmorPiercing)],
    );
    let defender = unit(
        0,
        4,
        UnitType::Neutral,
        vec![Keyword::ResistanceX { value: 1 }],
    );

    let result = apply_combat_modifier_stack(&attacker, &defender, &config());

    assert_eq!(result.net_damage, 2);
}

#[test]
fn test_cr15_type_advantage_adds_configured_attack_and_combat_armor() {
    let attacker = unit(3, 2, UnitType::Blade, vec![]);
    let defender = unit(0, 3, UnitType::Arcane, vec![]);

    let result = apply_combat_modifier_stack(&attacker, &defender, &config());

    assert_eq!(result.net_damage, 1);
    assert_eq!(result.ar_attacker_combat, 1);
    assert_eq!(attacker.atk, 3);
    assert_eq!(attacker.ar, 2);
}

#[test]
fn test_cr42_vulnerability_increases_effective_attack_before_armor() {
    let attacker = unit(3, 0, UnitType::Neutral, vec![]);
    let defender = unit(
        0,
        1,
        UnitType::Neutral,
        vec![Keyword::VulnerabilityX { value: 2 }],
    );

    let result = apply_combat_modifier_stack(&attacker, &defender, &config());

    assert_eq!(result.net_damage, 4);
}

#[test]
fn test_cr43_silence_strips_attacker_keywords_for_this_combat() {
    let attacker = unit(
        5,
        0,
        UnitType::Neutral,
        vec![
            simple(SimpleKeyword::Silence),
            simple(SimpleKeyword::FirstStrike),
            simple(SimpleKeyword::ArmorPiercing),
        ],
    );
    let defender = unit(0, 4, UnitType::Neutral, vec![]);

    let result = apply_combat_modifier_stack(&attacker, &defender, &config());

    assert_eq!(result.net_damage, 1);
}

#[test]
fn test_stunned_attacker_deals_no_damage() {
    let attacker = unit(5, 0, UnitType::Neutral, vec![simple(SimpleKeyword::Stun)]);
    let defender = unit(0, 0, UnitType::Neutral, vec![]);

    let result = apply_combat_modifier_stack(&attacker, &defender, &config());

    assert_eq!(result.net_damage, 0);
    assert_eq!(result.ar_attacker_combat, 0);
}
