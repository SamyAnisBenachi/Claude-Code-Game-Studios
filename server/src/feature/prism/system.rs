use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Replicate};
use shared::protocol::DraftPhase;

use crate::core::rsm::{DraftStarted, GameOverEmitted};
use crate::core::session::SessionConfig;
use crate::feature::prism::{
    AuditLog, DiscardLog, PrismCollected, PrismLaneKey, PrismPresence, PrismState, MAX_PLAYERS,
    PRISM_LANE_COUNT,
};

/// Initializes session-scoped Prism resources and replicated presence entities.
pub fn initialize_prism_session(
    mut commands: Commands,
    mut draft_started: MessageReader<DraftStarted>,
    session: Option<Res<SessionConfig>>,
    existing_prisms: Query<Entity, With<PrismLaneKey>>,
) {
    let should_initialize = draft_started
        .read()
        .any(|message| message.phase == DraftPhase::Initial);

    if !should_initialize {
        return;
    }

    let Some(session) = session else {
        return;
    };

    for entity in existing_prisms.iter() {
        commands.entity(entity).despawn();
    }

    commands.insert_resource(PrismState::default());
    commands.insert_resource(DiscardLog::default());
    commands.insert_resource(AuditLog::default());

    for player in session.players().take(MAX_PLAYERS) {
        for lane in 1..=PRISM_LANE_COUNT as u8 {
            commands.spawn((
                PrismLaneKey { player, lane },
                PrismPresence { collected: false },
                Replicate::to_clients(NetworkTarget::All),
            ));
        }
    }
}

/// Removes all session-scoped Prism state at game over.
pub fn cleanup_prism_session(
    mut commands: Commands,
    mut game_over: MessageReader<GameOverEmitted>,
    prism_entities: Query<Entity, With<PrismLaneKey>>,
) {
    if game_over.read().next().is_none() {
        return;
    }

    for entity in prism_entities.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<PrismState>();
    commands.remove_resource::<DiscardLog>();
    commands.remove_resource::<AuditLog>();
}

/// Scaffold Prism resolver.
///
/// Reward routing, stale-message discard, RNG draws, and respawn mutation land in
/// follow-up Prism stories. This scaffold intentionally drains no-op collection
/// input without touching Economy state, preserving existing collected flags when
/// the buffer is empty.
pub fn resolve_prism_draws(
    _prism_state: Option<ResMut<PrismState>>,
    _discard_log: Option<ResMut<DiscardLog>>,
    _audit_log: Option<ResMut<AuditLog>>,
    mut collected: MessageReader<PrismCollected>,
) {
    for _message in collected.read() {}
}
