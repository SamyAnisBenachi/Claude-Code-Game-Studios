use std::collections::HashMap;

use server::core::session::SessionConfig;
use server::feature::board::{requires_spawn_range_validation, validate_spawn_range, BoardConfig};
use shared::card::{CardType, ClassId};
use shared::protocol::GameMode;
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config() -> SessionConfig {
    let player_a = player(1);
    let player_b = player(2);

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player_a, 0), (player_b, 1)]),
        class_map: HashMap::from([(player_a, ClassId::Iop), (player_b, ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn validates(target_cell: u8, player: PlayerId, fakes_destroyed: u8) -> bool {
    validate_spawn_range(
        target_cell,
        player,
        fakes_destroyed,
        &session_config(),
        &BoardConfig::default(),
    )
}

#[test]
fn bl_5_player_a_default_range_rejects_cell_2() {
    assert!(!validates(2, player(1), 0));
    assert!(!validates(3, player(1), 0));
    assert!(validates(1, player(1), 0));
    assert!(!validates(4, player(1), 0));
}

#[test]
fn bl_5b_player_b_default_range_rejects_cell_7() {
    assert!(!validates(7, player(2), 0));
    assert!(!validates(6, player(2), 0));
    assert!(validates(8, player(2), 0));
    assert!(!validates(5, player(2), 0));
}

#[test]
fn bl_6_player_a_range_expands_after_one_fake_destroyed() {
    assert!(validates(2, player(1), 1));
    assert!(!validates(3, player(1), 1));
    assert!(validates(3, player(1), 2));
}

#[test]
fn bl_6b_player_b_range_expands_after_one_fake_destroyed() {
    assert!(validates(7, player(2), 1));
    assert!(!validates(6, player(2), 1));
    assert!(validates(6, player(2), 2));
}

#[test]
fn bl_7_spawn_range_validation_only_applies_to_minions() {
    assert!(requires_spawn_range_validation(CardType::Minion));
    assert!(!requires_spawn_range_validation(CardType::Structure));
    assert!(!requires_spawn_range_validation(CardType::Trap));
}

#[test]
fn unknown_players_are_rejected_silently() {
    assert!(!validates(1, player(99), 0));
}

#[test]
fn fakes_destroyed_is_clamped_to_max_spawn_expansion() {
    assert!(validates(3, player(1), 9));
    assert!(!validates(4, player(1), 9));
    assert!(validates(6, player(2), 9));
    assert!(!validates(5, player(2), 9));
}
