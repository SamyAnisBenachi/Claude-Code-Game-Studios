// Scaffold systems consumed by downstream stories.
#![allow(dead_code)]

use bevy::prelude::*;
use shared::protocol::DraftPhase;

use crate::core::pool::state::{
    InitialDraftOffering, ManualRefreshCount, PlayerPool, PlayerPools, ShopSlots,
};
use crate::core::rsm::{DraftStarted, GameOverEmitted};
use crate::core::session::SessionConfig;
use crate::foundation::config::{CardCatalog, GameConfig};

pub fn initialize_player_pools_on_draft_started(
    mut draft_started: MessageReader<DraftStarted>,
    session: Res<SessionConfig>,
    catalog: Res<CardCatalog>,
    config: Res<GameConfig>,
    mut pools: ResMut<PlayerPools>,
) {
    tracing::info!(
        "initialize_player_pools_on_draft_started: entered (session=true, catalog=true, config=true)"
    );

    for message in draft_started.read() {
        if message.phase != DraftPhase::Initial {
            continue;
        }

        pools.pools.clear();
        for player in session.players() {
            pools
                .pools
                .insert(player, PlayerPool::initialize(&catalog.cards, &config.0));
        }
    }
}

pub fn clear_pool_session_resources_on_game_over(
    mut game_over: MessageReader<GameOverEmitted>,
    mut pools: ResMut<PlayerPools>,
    mut shop_slots: ResMut<ShopSlots>,
    mut draft_offering: ResMut<InitialDraftOffering>,
    mut refresh_count: ResMut<ManualRefreshCount>,
) {
    let mut saw_game_over = false;
    for _message in game_over.read() {
        saw_game_over = true;
    }

    if !saw_game_over {
        return;
    }

    pools.pools.clear();
    shop_slots.0.clear();
    draft_offering.0.clear();
    refresh_count.0.clear();
}
