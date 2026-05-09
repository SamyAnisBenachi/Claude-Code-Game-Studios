// server/src/core/pool/plugin.rs - CardPoolPlugin (ADR-006)

use bevy::prelude::*;

use crate::core::pool::state::{InitialDraftOffering, ManualRefreshCount, PlayerPools, ShopSlots};
use crate::core::pool::system::{
    clear_pool_session_resources_on_game_over, initialize_player_pools_on_draft_started,
};
use crate::core::rsm::advance_phase;
use crate::core::session::{SessionConfig, SessionSystemSet};
use crate::foundation::config::{CardCatalog, GameConfig};

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
            .configure_sets(
                Update,
                CardPoolSet::Lifecycle
                    .after(advance_phase)
                    .run_if(resource_exists::<SessionConfig>)
                    .run_if(resource_exists::<CardCatalog>)
                    .run_if(resource_exists::<GameConfig>),
            )
            // Explicit deferred-command flush between SessionSystemSet::LobbyEval
            // (which inserts SessionConfig via Commands) and CardPoolSet::Lifecycle
            // (whose run_if gate reads SessionConfig). Without this flush, the gate
            // is evaluated before the command buffer is applied, causing the gate
            // to fail on frame N and the DraftStarted message to expire before
            // the gate ever passes — leaving PlayerPools uninitialized.
            //
            // Bevy 0.18: `ApplyDeferred` is the system marker (free function
            // `apply_deferred` was removed in 0.17). Auto-insert sync points
            // do not cover `run_if`-gated set entry, so an explicit flush is
            // required here.
            .add_systems(
                Update,
                ApplyDeferred
                    .after(SessionSystemSet::LobbyEval)
                    .before(CardPoolSet::Lifecycle),
            )
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
