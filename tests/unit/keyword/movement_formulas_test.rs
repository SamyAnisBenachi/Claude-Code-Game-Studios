use server::feature::keyword::movement::{
    attract_destination, attract_enemy_destination, repel_destination, PlayerSide,
};

#[test]
fn kw_029a_repel_player_a_clamps_at_cell_one() {
    assert_eq!(repel_destination(2, PlayerSide::PlayerA, 3), 1);
}

#[test]
fn kw_029b_repel_player_b_pushes_toward_cell_eight() {
    assert_eq!(repel_destination(5, PlayerSide::PlayerB, 2), 7);
}

#[test]
fn kw_029c_repel_uses_signed_intermediate_at_cell_one() {
    assert_eq!(repel_destination(1, PlayerSide::PlayerA, 6), 1);
}

#[test]
fn kw_029d_repel_player_b_clamps_at_cell_eight() {
    assert_eq!(repel_destination(8, PlayerSide::PlayerB, 6), 8);
}

#[test]
fn repel_destination_always_returns_board_cell() {
    for owner in [PlayerSide::PlayerA, PlayerSide::PlayerB] {
        for target_cell in [0, 1, 2, 5, 8, 9, u8::MAX] {
            for x in [0, 1, 6, u8::MAX] {
                let destination = repel_destination(target_cell, owner, x);
                assert!(
                    (1..=8).contains(&destination),
                    "destination {destination} for target_cell={target_cell}, owner={owner:?}, x={x}"
                );
            }
        }
    }
}

#[test]
fn kw_030a_attract_friendly_target_can_stop_on_caster_cell() {
    assert_eq!(attract_destination(3, 7, 6), 3);
}

#[test]
fn kw_030b_attract_enemy_target_stops_one_cell_short() {
    assert_eq!(attract_enemy_destination(3, 7, 6), 4);
}

#[test]
fn kw_079_attract_enemy_target_already_adjacent_does_not_move() {
    assert_eq!(attract_enemy_destination(2, 3, 6), 3);
}

#[test]
fn attract_co_located_target_is_noop() {
    assert_eq!(attract_destination(5, 5, 3), 5);
    assert_eq!(attract_enemy_destination(5, 5, 3), 5);
}

#[test]
fn attract_destination_never_overshoots_caster() {
    assert_eq!(attract_destination(7, 2, 20), 7);
    assert_eq!(attract_destination(2, 7, 20), 2);
    assert_eq!(attract_enemy_destination(7, 2, 20), 6);
    assert_eq!(attract_enemy_destination(2, 7, 20), 3);
}

#[test]
fn movement_formulas_are_deterministic() {
    let repel = repel_destination(5, PlayerSide::PlayerB, 2);
    let attract = attract_enemy_destination(3, 7, 6);

    assert_eq!(repel_destination(5, PlayerSide::PlayerB, 2), repel);
    assert_eq!(attract_enemy_destination(3, 7, 6), attract);
}
