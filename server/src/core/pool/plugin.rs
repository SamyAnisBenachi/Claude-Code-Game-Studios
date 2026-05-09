// server/src/core/pool/plugin.rs - CardPoolPlugin (ADR-006)

use bevy::prelude::*;

use crate::core::pool::state::{InitialDraftOffering, ManualRefreshCount, PlayerPools, ShopSlots};
use crate::core::pool::system::{
    clear_pool_session_resources_on_game_over, initialize_player_pools_on_draft_started,
};
use crate::core::rsm::advance_phase;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardPoolSet {
    Lifecycle,
}

// Scaffold API consumed by downstream stories.
#[allow(dead_code)]
pub struct CardPoolPlugin;

impl Plugin for CardPoolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerPools>()
            .init_resource::<ShopSlots>()
            .init_resource::<InitialDraftOffering>()
            .init_resource::<ManualRefreshCount>()
            .configure_sets(Update, CardPoolSet::Lifecycle.after(advance_phase))
            .add_systems(
                Update,
                (
                    initialize_player_pools_on_draft_started,
                    clear_pool_session_resources_on_game_over,
                )
                    .in_set(CardPoolSet::Lifecycle),
            );
    }
}
