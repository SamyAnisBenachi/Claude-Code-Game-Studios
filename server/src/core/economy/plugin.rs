// server/src/core/economy/plugin.rs -- Economy plugin registration.

use bevy::ecs::schedule::common_conditions::resource_exists;
use bevy::prelude::*;

use crate::core::economy::state::{InterestSnapshots, PlayerEconomies};
use crate::core::economy::system::{
    discard_current_mana_at_resolution_end, initialise_player_economies, on_draft_started,
    on_resolution_phase_entered, EconomySystemSet, S2CGoldBroadcast, S2CGoldUpdate,
};
use crate::core::rsm::advance_phase;
use crate::core::session::SessionConfig;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEconomies>()
            .init_resource::<InterestSnapshots>()
            .add_message::<S2CGoldUpdate>()
            .add_message::<S2CGoldBroadcast>()
            .add_observer(initialise_player_economies)
            .add_systems(Update, on_draft_started.after(advance_phase))
            .add_systems(
                Update,
                (
                    on_resolution_phase_entered,
                    discard_current_mana_at_resolution_end,
                )
                    .in_set(EconomySystemSet::ResolutionEnd)
                    .after(advance_phase)
                    .run_if(resource_exists::<SessionConfig>),
                // TODO M2: also order .after(ObjectiveSystemSet::ProcessDestructions).after(CombatSystemSet::ProcessKills).before(ResolutionCompleteEmitter)
            );
    }
}
