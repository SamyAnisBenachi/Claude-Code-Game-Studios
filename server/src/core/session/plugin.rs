// server/src/core/session/plugin.rs -- Game Session System plugin scaffold.

use bevy::prelude::*;
use shared::card::ClassId;

use crate::core::session::{PlayerSessionData, PlayerSessions, SessionConfig, SessionReady};

pub struct GameSessionPlugin;

impl Plugin for GameSessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSessions>()
            .add_observer(initialise_player_sessions);
    }
}

pub fn initialise_player_sessions(
    _trigger: On<SessionReady>,
    session: Res<SessionConfig>,
    mut sessions: ResMut<PlayerSessions>,
) {
    sessions.players.clear();
    for player_id in session.players() {
        let class = session
            .class_map
            .get(&player_id)
            .copied()
            .unwrap_or(ClassId::Neutral);
        sessions.players.insert(
            player_id,
            PlayerSessionData {
                class,
                class_locked: false,
            },
        );
    }
}
