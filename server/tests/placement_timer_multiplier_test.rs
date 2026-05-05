use std::collections::HashMap;

use server::core::session::{
    apply_placement_timer_multiplier_request_batch, build_session_config_with_settings, LobbyState,
    PlacementTimerMultiplierRequests, SessionConfig, SessionSlot, SessionSlots,
};
use shared::card::ClassId;
use shared::protocol::{GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn ready_slots() -> SessionSlots {
    SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(player(1)),
            class: Some(ClassId::Iop),
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: Some(player(2)),
            class: Some(ClassId::Cra),
        },
    ])
}

fn ready_selections() -> server::core::session::ClassSelections {
    server::core::session::ClassSelections(HashMap::from([
        (player(1), ClassId::Iop),
        (player(2), ClassId::Cra),
    ]))
}

fn frozen_session_config(multiplier: PlacementTimerMultiplier) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player(1), 0), (player(2), 1)]),
        class_map: HashMap::from([(player(1), ClassId::Iop), (player(2), ClassId::Cra)]),
        placement_timer_multiplier_effective: multiplier,
    }
}

#[test]
fn session_multiplier_defaults_to_x1_when_players_make_no_requests() {
    let slots = ready_slots();
    let config = build_session_config_with_settings(&slots, &ready_selections(), None);

    assert_eq!(
        config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1
    );
}

#[test]
fn session_multiplier_highest_request_wins_and_update_has_no_identity() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests::default();

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::LobbyWaiting),
        None,
        Some(&slots),
        &mut requests,
        [
            (player(1), Some(PlacementTimerMultiplier::X1_5)),
            (player(2), Some(PlacementTimerMultiplier::X3)),
        ],
    )
    .expect("highest request should change the effective multiplier");

    assert_eq!(
        update.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X3
    );
    assert_eq!(
        requests.0.get(&player(1)),
        Some(&PlacementTimerMultiplier::X1_5)
    );
    assert_eq!(
        requests.0.get(&player(2)),
        Some(&PlacementTimerMultiplier::X3)
    );

    let value = serde_json::to_value(update).expect("settings update should serialize");
    assert!(value.get("placement_timer_multiplier_effective").is_some());
    assert!(value.get("requester").is_none());
    assert!(value.get("requester_id").is_none());
    assert!(value.get("player_id").is_none());
    assert!(value.get("connection_id").is_none());
}

#[test]
fn session_multiplier_invalid_or_unsupported_request_does_not_shorten_default() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests::default();
    let invalid = PlacementTimerMultiplier::from_standard_ratio(1, 2);

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::LobbyWaiting),
        None,
        Some(&slots),
        &mut requests,
        [(player(1), invalid)],
    );

    assert!(update.is_none());
    assert!(requests.0.is_empty());
    let config = build_session_config_with_settings(&slots, &ready_selections(), Some(&requests));
    assert_eq!(
        config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1
    );
}

#[test]
fn session_multiplier_freezes_into_session_config_before_session_ready() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests::default();
    requests.0.insert(player(1), PlacementTimerMultiplier::X1_5);
    requests.0.insert(player(2), PlacementTimerMultiplier::X2);

    let config = build_session_config_with_settings(&slots, &ready_selections(), Some(&requests));

    assert_eq!(
        config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X2
    );
}

#[test]
fn session_multiplier_ignores_post_session_ready_changes() {
    let slots = ready_slots();
    let frozen_config = frozen_session_config(PlacementTimerMultiplier::X2);
    let mut requests = PlacementTimerMultiplierRequests::default();

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::GameActive),
        Some(&frozen_config),
        Some(&slots),
        &mut requests,
        [(player(1), Some(PlacementTimerMultiplier::X3))],
    );

    assert!(update.is_none());
    assert!(requests.0.is_empty());
    assert_eq!(
        frozen_config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X2
    );
}

#[test]
fn session_multiplier_x1_request_clears_existing_player_request() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests(HashMap::from([(
        player(1),
        PlacementTimerMultiplier::X2,
    )]));

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::LobbyWaiting),
        None,
        Some(&slots),
        &mut requests,
        [(player(1), Some(PlacementTimerMultiplier::X1))],
    )
    .expect("clearing the only request should publish the default");

    assert_eq!(
        update.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1
    );
    assert!(requests.0.is_empty());
}
