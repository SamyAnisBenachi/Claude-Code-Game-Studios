use bevy::prelude::*;
use lightyear::prelude::Replicate;
use shared::protocol::DraftPhase;

use crate::core::rsm::DraftStarted;
use crate::core::session::SessionConfig;
use crate::feature::objective::{
    HiddenObjectives, ObjectiveCounters, ObjectiveHp, ObjectiveSlot, OBJECTIVE_LANE_COUNT,
};
use crate::foundation::config::GameConfig;
use crate::foundation::rng::ServerRng;

const LOSS_THRESHOLD: u32 = 2;

/// Initializes objective slots on DRAFT_INITIAL entry.
///
/// Hidden fake identity assignment remains server-only in `HiddenObjectives`.
pub fn initialize_objectives_on_draft_initial(
    mut commands: Commands,
    mut draft_started: MessageReader<DraftStarted>,
    session: Option<Res<SessionConfig>>,
    config: Option<Res<GameConfig>>,
    rng: Option<ResMut<ServerRng>>,
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

    let Some(mut rng) = rng else {
        error!("Objective initialization refused: ServerRng resource is missing");
        return;
    };

    if let Err(error) = validate_objective_config(&config.0) {
        error!("Objective initialization refused: {error}");
        return;
    }

    for entity in existing_objectives.iter() {
        commands.entity(entity).despawn();
    }

    let players = session.players().collect::<Vec<_>>();
    counters.reset_for_players(players.iter().copied());
    assign_fake_objectives(
        &mut rng,
        &players,
        config.fake_count as usize,
        &mut hidden_objectives,
    );

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

/// Validates Objective System invariants that must hold before DRAFT_INITIAL.
pub fn validate_objective_config(config: &shared::config::GameConfig) -> Result<(), String> {
    if config.fake_count < 1 {
        return Err(
            "fake_count must be >= 1: D5 would otherwise consume assignment seeds without a valid fake count"
                .to_string(),
        );
    }

    let lane_count = u32::from(OBJECTIVE_LANE_COUNT);
    let max_fake_count = lane_count - LOSS_THRESHOLD;
    if config.fake_count > max_fake_count {
        return Err(format!(
            "fake_count must be <= lane_count - loss_threshold ({max_fake_count}); got {}",
            config.fake_count
        ));
    }

    if config.objective_hp < 1 {
        return Err("objective_hp must be >= 1: objectives cannot spawn destroyed".to_string());
    }

    Ok(())
}

/// Assigns hidden objective identities in ascending player order.
pub fn assign_fake_objectives(
    rng: &mut ServerRng,
    players: &[shared::session::PlayerId],
    fake_count: usize,
    hidden: &mut HiddenObjectives,
) {
    hidden.identities.clear();

    let mut ordered_players = players.to_vec();
    ordered_players.sort_by_key(|player| player.0);

    for player in ordered_players {
        assign_fakes_for_player(rng, player, fake_count, hidden);
    }
}

fn assign_fakes_for_player(
    rng: &mut ServerRng,
    player: shared::session::PlayerId,
    fake_count: usize,
    hidden: &mut HiddenObjectives,
) {
    let mut lanes = (1..=OBJECTIVE_LANE_COUNT).collect::<Vec<_>>();
    let (first_fake, second_fake) = rng.assign_fake_objectives(rng_player_id(player));
    let mut fake_lanes = Vec::with_capacity(fake_count);

    for candidate in [first_fake, second_fake] {
        if fake_lanes.len() >= fake_count {
            break;
        }
        if let Some(position) = lanes.iter().position(|lane| *lane == candidate) {
            fake_lanes.push(lanes.remove(position));
        }
    }

    while fake_lanes.len() < fake_count {
        if lanes.is_empty() {
            break;
        }
        fake_lanes.push(lanes.remove(0));
    }

    for lane in fake_lanes {
        hidden.identities.insert((player, lane), true);
    }

    for lane in lanes {
        hidden.identities.insert((player, lane), false);
    }
}

fn rng_player_id(player: shared::session::PlayerId) -> u32 {
    u32::try_from(player.0).unwrap_or(u32::MAX)
}
