// server/src/core/pool/plugin.rs — CardPoolPlugin (ADR-006)
//
// Story 001 scope: register PlayerPools resource only.
// Story 003: adds ShopSlots, InitialDraftOffering, ManualRefreshCount.
// Story 004: adds systems (on_session_ready_init, on_shop_refresh_needed).
// Story 005: adds on_manual_refresh system.
// Story 006: adds network dispatch systems.

use bevy::prelude::*;

use crate::core::pool::state::{InitialDraftOffering, ManualRefreshCount, PlayerPools, ShopSlots};

pub struct CardPoolPlugin;

impl Plugin for CardPoolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerPools>()
            .init_resource::<ShopSlots>()
            .init_resource::<InitialDraftOffering>()
            .init_resource::<ManualRefreshCount>();
        // Additional resources and systems added in Stories 003–006.
    }
}
