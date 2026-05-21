//! Server-authoritative auction feature.
#![allow(dead_code, unused_imports)]

pub mod plugin;
pub mod snapshot;
pub mod state;
pub mod system;

pub use plugin::{AuctionPlugin, AuctionSet};
pub use shared::protocol::AuctionSnapshot;
pub use snapshot::auction_snapshot;
pub use state::{AuctionPhase, AuctionState};
pub use system::{
    auction_tick_system, clear_auction_pool_on_game_over, decrement_live_bidding_timer,
    defer_auction_outbox_for_reconnect, enforce_live_bidding_deadline,
    initialize_auction_pool_on_draft_started, process_bid_batch, settle_expired_auction,
    AuctionAcceptedDispatch, AuctionBid, AuctionCardAcquiredDispatch, AuctionCardDrawFixture,
    AuctionNetworkOutbox, AuctionPool, AuctionRejectionDispatch, AuctionSettledDispatch,
    PendingBotBids, S2CAuctionCard,
};
