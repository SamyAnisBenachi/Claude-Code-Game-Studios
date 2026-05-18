// PROMPT 1211 -- S18 Opponent Disconnect Broadcast Repair
//
// Recipient-rule unit tests for `opponent_disconnect_recipients`.
// The dispatch system must send `S2COpponentDisconnected` to the *other*
// occupied session player(s), never to the disconnected player.

use std::collections::HashMap;

use lightyear::prelude::PeerId;
use server::core::session::{PlayerConnectionMap, SessionConfig};
use server::network::rsm_dispatch::opponent_disconnect_recipients;
use shared::card::ClassId;
use shared::protocol::{GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config_for(players: &[PlayerId]) -> SessionConfig {
    let mut team_map = HashMap::new();
    let mut class_map = HashMap::new();
    for (index, p) in players.iter().copied().enumerate() {
        team_map.insert(p, index as u8);
        class_map.insert(p, ClassId::Iop);
    }
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map,
        class_map,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
    }
}

fn connections_for(pairs: &[(PeerId, PlayerId)]) -> PlayerConnectionMap {
    let mut map = HashMap::new();
    for (peer, player_id) in pairs.iter().copied() {
        map.insert(peer, player_id);
    }
    PlayerConnectionMap(map)
}

#[test]
fn opponent_disconnect_recipients_excludes_disconnected_player() {
    let session = session_config_for(&[player(1), player(2)]);
    let connections = connections_for(&[
        (PeerId::Netcode(101), player(1)),
        (PeerId::Netcode(102), player(2)),
    ]);

    let recipients = opponent_disconnect_recipients(player(1), &session, &connections);
    assert_eq!(
        recipients,
        vec![PeerId::Netcode(102)],
        "surviving player (2) must be the sole recipient; disconnected player (1) must not be"
    );
    assert!(
        !recipients.contains(&PeerId::Netcode(101)),
        "disconnected player's PeerId must never appear in recipient list"
    );
}

#[test]
fn opponent_disconnect_recipients_returns_empty_when_no_other_players() {
    let session = session_config_for(&[player(1)]);
    let connections = connections_for(&[(PeerId::Netcode(101), player(1))]);

    let recipients = opponent_disconnect_recipients(player(1), &session, &connections);
    assert!(
        recipients.is_empty(),
        "no recipients when the disconnected player is the only one in the session"
    );
}

#[test]
fn opponent_disconnect_recipients_skips_unmapped_peer() {
    // Surviving player has no PeerId mapped (e.g., stale connection map mid-reconnect).
    let session = session_config_for(&[player(1), player(2)]);
    let connections = connections_for(&[(PeerId::Netcode(101), player(1))]);

    let recipients = opponent_disconnect_recipients(player(1), &session, &connections);
    assert!(
        recipients.is_empty(),
        "no recipients when the surviving player has no mapped PeerId"
    );
}

#[test]
fn opponent_disconnect_recipients_supports_multiple_survivors() {
    // 2v2 mode: one disconnect -> three survivors.
    let session = session_config_for(&[player(1), player(2), player(3), player(4)]);
    let connections = connections_for(&[
        (PeerId::Netcode(101), player(1)),
        (PeerId::Netcode(102), player(2)),
        (PeerId::Netcode(103), player(3)),
        (PeerId::Netcode(104), player(4)),
    ]);

    let recipients = opponent_disconnect_recipients(player(2), &session, &connections);
    let mut sorted: Vec<u64> = recipients
        .iter()
        .filter_map(|peer| match *peer {
            PeerId::Netcode(id) => Some(id),
            _ => None,
        })
        .collect();
    sorted.sort();
    assert_eq!(sorted, vec![101, 103, 104]);
    assert!(
        !recipients.contains(&PeerId::Netcode(102)),
        "disconnected player (2) must not appear as recipient"
    );
}
