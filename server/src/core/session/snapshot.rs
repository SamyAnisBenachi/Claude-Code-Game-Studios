use bevy::prelude::World;
use shared::protocol::{PlayerSnapshot, S2CGameSnapshot};
use shared::session::PlayerId;

use crate::core::session::PlayerSessions;

/// Builds the player portion of a game snapshot from authoritative session state.
pub fn build_snapshot(player_id: PlayerId, world: &World) -> Option<S2CGameSnapshot> {
    let sessions = world.get_resource::<PlayerSessions>()?;
    let mut players = sessions
        .players
        .iter()
        .map(|(player_id, session)| PlayerSnapshot {
            player_id: *player_id,
            class_id: session.class,
        })
        .collect::<Vec<_>>();
    players.sort_by_key(|snapshot| snapshot.player_id.0);

    Some(S2CGameSnapshot { player_id, players })
}
