// server/src/core/economy/plugin.rs -- Economy plugin registration.

use bevy::prelude::*;

use crate::core::economy::state::{InterestSnapshots, PlayerEconomies};
use crate::core::economy::system::{
    initialise_player_economies, on_draft_started, S2CGoldBroadcast, S2CGoldUpdate,
};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEconomies>()
            .init_resource::<InterestSnapshots>()
            .add_message::<S2CGoldUpdate>()
            .add_message::<S2CGoldBroadcast>()
            .add_observer(initialise_player_economies)
            .add_systems(Update, on_draft_started);
    }
}
