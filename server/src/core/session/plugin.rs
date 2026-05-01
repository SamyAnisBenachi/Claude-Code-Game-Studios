// server/src/core/session/plugin.rs -- Game Session System plugin scaffold.

use bevy::prelude::*;
use shared::card::ClassId;

use crate::core::session::{
    handle_create_room, handle_join_room, ActiveSessions, PlayerConnectionMap, PlayerSessionData,
    PlayerSessions, RoomSessions, SessionConfig, SessionReady,
};

pub struct GameSessionPlugin;

impl Plugin for GameSessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSessions>()
            .init_resource::<ActiveSessions>()
            .init_resource::<PlayerConnectionMap>()
            .init_resource::<RoomSessions>()
            .add_systems(Update, (handle_create_room, handle_join_room));
    }
}

// Called by DraftStarted { phase: Initial } subscriber - NOT a SessionReady observer.
// Wired up in the class-selection story (S3-03) via RSM-owned DraftStarted signal.
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
