use bevy::prelude::*;
use lightyear::prelude::Replicate;
use shared::protocol::DraftPhase;

use crate::core::rsm::DraftStarted;
use crate::core::session::SessionConfig;
use crate::feature::objective::{
    HiddenObjectives, ObjectiveCounters, ObjectiveHp, ObjectiveSlot, OBJECTIVE_LANE_COUNT,
};
use crate::foundation::config::GameConfig;

/// Initializes objective slots on DRAFT_INITIAL entry.
///
/// Hidden fake identity assignment is intentionally left to the next objective
/// story; this system only creates visible HP slots and zeroed counters.
pub fn initialize_objectives_on_draft_initial(
    mut commands: Commands,
    mut draft_started: MessageReader<DraftStarted>,
    session: Option<Res<SessionConfig>>,
    config: Option<Res<GameConfig>>,
    existing_objectives: Query<Entity, With<ObjectiveSlot>>,
    mut hidden_objectives: ResMut<HiddenObjectives>,
    mut counters: ResMut<ObjectiveCounters>,
) {
    let should_initialize = draft_started
        .read()
        .any(|message| message.phase == DraftPhase::Initial);

    if !should_initialize {
        return;
    }

    let (Some(session), Some(config)) = (session, config) else {
        return;
    };

    for entity in existing_objectives.iter() {
        commands.entity(entity).despawn();
    }

    hidden_objectives.identities.clear();

    let players = session.players().collect::<Vec<_>>();
    counters.reset_for_players(players.iter().copied());

    for player in players {
        for lane in 1..=OBJECTIVE_LANE_COUNT {
            commands.spawn((
                ObjectiveHp {
                    hp: config.objective_hp,
                },
                ObjectiveSlot {
                    lane,
                    player,
                    destroyed: false,
                },
                Replicate::default(),
            ));
        }
    }
}
