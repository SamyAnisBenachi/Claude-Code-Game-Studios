// server/src/core/session/config.rs -- SessionReady configuration handoff.

use std::collections::HashMap;

use bevy::prelude::Resource;
use shared::card::ClassId;
use shared::protocol::{GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;

use crate::core::session::state::{
    ClassSelections, PlacementTimerMultiplierRequests, SessionSlots, TeamId,
};

#[derive(Debug, Clone, Resource)]
pub struct SessionConfig {
    pub mode: GameMode,
    pub player_count: u8,
    pub team_map: HashMap<PlayerId, TeamId>,
    pub class_map: HashMap<PlayerId, ClassId>,
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
}

impl SessionConfig {
    pub fn players(&self) -> impl Iterator<Item = PlayerId> {
        let mut players = self.team_map.keys().copied().collect::<Vec<_>>();
        players.sort_by_key(|player| player.0);
        players.into_iter()
    }
}

pub fn build_session_config(slots: &SessionSlots, selections: &ClassSelections) -> SessionConfig {
    build_session_config_with_settings(slots, selections, None)
}

pub fn build_session_config_with_settings(
    slots: &SessionSlots,
    selections: &ClassSelections,
    placement_timer_requests: Option<&PlacementTimerMultiplierRequests>,
) -> SessionConfig {
    let mut team_map = HashMap::new();
    let mut class_map = HashMap::new();

    for slot in &slots.0 {
        let Some(player) = slot.player else {
            continue;
        };
        let Some(class_id) = slot.class else {
            panic!(
                "build_session_config: slot {} for player {:?} has no class confirmed -- invariant violation",
                slot.index, player
            );
        };

        debug_assert_eq!(
            selections.0.get(&player).copied(),
            Some(class_id),
            "build_session_config: class selections must mirror occupied slot classes"
        );

        team_map.insert(player, slot.team);
        class_map.insert(player, class_id);
    }

    let placement_timer_multiplier_effective =
        effective_placement_timer_multiplier(slots, placement_timer_requests);

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: team_map.len() as u8,
        team_map,
        class_map,
        placement_timer_multiplier_effective,
    }
}

pub fn effective_placement_timer_multiplier(
    slots: &SessionSlots,
    placement_timer_requests: Option<&PlacementTimerMultiplierRequests>,
) -> PlacementTimerMultiplier {
    let Some(requests) = placement_timer_requests else {
        return PlacementTimerMultiplier::X1;
    };

    slots
        .0
        .iter()
        .filter_map(|slot| slot.player)
        .filter_map(|player| requests.0.get(&player).copied())
        .max()
        .unwrap_or(PlacementTimerMultiplier::X1)
        .min(PlacementTimerMultiplier::X3)
}
