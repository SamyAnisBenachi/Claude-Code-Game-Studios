// server/src/core/session/config.rs -- SessionReady configuration handoff.

use std::collections::HashMap;

use bevy::prelude::Resource;
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

use crate::core::session::state::{ClassSelections, SessionSlots, TeamId};

#[derive(Debug, Clone, Resource)]
pub struct SessionConfig {
    pub mode: GameMode,
    pub player_count: u8,
    pub team_map: HashMap<PlayerId, TeamId>,
    pub class_map: HashMap<PlayerId, ClassId>,
}

impl SessionConfig {
    pub fn players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.team_map.keys().copied()
    }
}

pub fn build_session_config(slots: &SessionSlots, selections: &ClassSelections) -> SessionConfig {
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

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: team_map.len() as u8,
        team_map,
        class_map,
    }
}
