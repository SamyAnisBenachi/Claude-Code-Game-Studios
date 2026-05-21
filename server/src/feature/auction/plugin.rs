use bevy::ecs::schedule::common_conditions::resource_exists;
use bevy::prelude::*;

use crate::core::rsm::{AuctionSettled, DraftStarted, RsmSet};
use crate::core::session::{reconnect_snapshot_system, SessionSystemSet};
use crate::feature::auction::{
    auction_tick_system, clear_auction_pool_on_game_over, initialize_auction_pool_on_draft_started,
    AuctionCardDrawFixture, AuctionPool, AuctionState, PendingBotBids, S2CAuctionCard,
};

/// Auction system ordering labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuctionSet {
    Tick,
}

/// Registers server-authoritative auction state and scheduling.
pub struct AuctionPlugin;

impl Plugin for AuctionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AuctionState::default())
            .init_resource::<AuctionPool>()
            .init_resource::<AuctionCardDrawFixture>()
            // PROMPT 1598: bot bid funnel — initialized here so any feature
            // (bot action loop today, future AI agents tomorrow) can push
            // bids that flow through `process_bid_batch` alongside humans.
            .init_resource::<PendingBotBids>()
            .add_message::<AuctionSettled>()
            .add_message::<DraftStarted>()
            .add_message::<S2CAuctionCard>()
            .configure_sets(Update, AuctionSet::Tick.before(RsmSet::Tick))
            .add_systems(
                Update,
                (
                    initialize_auction_pool_on_draft_started,
                    clear_auction_pool_on_game_over,
                )
                    .in_set(AuctionSet::Tick),
            )
            .add_systems(
                Update,
                auction_tick_system
                    .in_set(AuctionSet::Tick)
                    .in_set(SessionSystemSet::LiveMessages)
                    .after(initialize_auction_pool_on_draft_started)
                    .after(clear_auction_pool_on_game_over)
                    .after(reconnect_snapshot_system)
                    .run_if(resource_exists::<crate::foundation::config::CardCatalog>)
                    .run_if(resource_exists::<crate::foundation::config::GameConfig>),
            );
    }
}
