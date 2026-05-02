// server/src/core/session/plugin.rs -- Game Session System plugin scaffold.

use bevy::prelude::*;
use shared::card::ClassId;

use crate::core::rsm::{advance_phase, on_session_ready, PlayerHeartbeat};
use crate::core::session::{
    evaluate_session_ready, handle_confirm_class, handle_create_room, handle_game_over_teardown,
    handle_join_room, handle_lobby_disconnect, handle_lobby_heartbeat, handle_select_class,
    lobby_timeout_check, tick_lobby_heartbeats, ActiveSessions, ClassPreviews, ClassSelections,
    PlayerConnectionMap, PlayerSessionData, PlayerSessions, RoomSessions, ServerRngFactory,
    SessionConfig, SessionNetworkOutbox, SessionSystemSet,
};

pub struct GameSessionPlugin;

impl Plugin for GameSessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSessions>()
            .init_resource::<ActiveSessions>()
            .init_resource::<ClassPreviews>()
            .init_resource::<ClassSelections>()
            .init_resource::<PlayerConnectionMap>()
            .init_resource::<RoomSessions>()
            .init_resource::<ServerRngFactory>()
            .init_resource::<SessionNetworkOutbox>()
            .add_message::<PlayerHeartbeat>()
            .add_systems(
                Update,
                (
                    handle_create_room,
                    handle_join_room,
                    handle_select_class,
                    handle_confirm_class,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_lobby_heartbeat,
                    tick_lobby_heartbeats,
                    lobby_timeout_check,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                evaluate_session_ready
                    .in_set(SessionSystemSet::LobbyEval)
                    .after(lobby_timeout_check)
                    .before(advance_phase),
            )
            .add_systems(Update, handle_game_over_teardown.after(advance_phase))
            .add_observer(on_session_ready)
            .add_observer(handle_lobby_disconnect);
    }
}

// Called by a future DraftStarted { phase: Initial } subscriber - NOT a
// SessionReady observer. RSM is the only SessionReady observer path.
pub fn initialise_player_sessions(
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
