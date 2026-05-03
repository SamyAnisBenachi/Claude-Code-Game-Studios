// server/src/core/economy/plugin.rs -- Economy plugin registration.

use bevy::ecs::schedule::common_conditions::resource_exists;
use bevy::prelude::*;

use crate::core::economy::state::{InterestSnapshots, PlayerEconomies};
use crate::core::economy::system::{
    on_draft_started, on_resolution_complete, AwardGold, EconomySystemSet, ManaCapIncreased,
    S2CGoldBroadcast, S2CGoldUpdate,
};
use crate::core::rsm::{advance_phase, rsm_input_reader};
use crate::core::session::SessionConfig;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEconomies>()
            .init_resource::<InterestSnapshots>()
            .add_message::<AwardGold>()
            .add_message::<ManaCapIncreased>()
            .add_message::<S2CGoldUpdate>()
            .add_message::<S2CGoldBroadcast>()
            .add_systems(Update, on_draft_started.after(advance_phase))
            .add_systems(
                Update,
                on_resolution_complete
                    .in_set(EconomySystemSet::ResolutionEnd)
                    .before(rsm_input_reader)
                    .run_if(resource_exists::<SessionConfig>),
            );
    }
}
