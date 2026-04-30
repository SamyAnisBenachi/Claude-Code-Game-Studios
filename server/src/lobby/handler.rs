use std::collections::HashMap;

use bevy::prelude::*;
use lightyear::prelude::*;
use shared::card::ClassId;
use shared::protocol::C2SClassChoice;
use shared::session::PlayerId;

use crate::core::rsm::{RoundPhase, RoundState};
use crate::core::session::PlayerSessions;

#[derive(Resource, Default)]
pub struct PlayerConnectionMap(pub HashMap<PeerId, PlayerId>);

/// Sole drainer for `MessageReceiver<C2SClassChoice>`.
pub fn handle_class_choice(
    rsm: Res<RoundState>,
    connections: Res<PlayerConnectionMap>,
    mut sessions: ResMut<PlayerSessions>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SClassChoice>)>,
) {
    let in_lobby = rsm.phase == RoundPhase::Lobby;

    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            if !in_lobby {
                continue;
            }

            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                continue;
            };

            apply_class_choice(&mut sessions, player_id, msg);
        }
    }
}

pub fn apply_class_choice(sessions: &mut PlayerSessions, player_id: PlayerId, msg: C2SClassChoice) {
    let Some(player) = sessions.players.get_mut(&player_id) else {
        return;
    };

    if player.class_locked || msg.class == ClassId::Neutral {
        return;
    }

    player.class = msg.class;
}
