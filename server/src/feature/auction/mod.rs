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
    auction_tick_system, decrement_live_bidding_timer, defer_auction_outbox_for_reconnect,
    process_bid_batch, settle_expired_auction, AuctionAcceptedDispatch, AuctionBid,
    AuctionCardAcquiredDispatch, AuctionCardDrawFixture, AuctionNetworkOutbox,
    AuctionRejectionDispatch, AuctionSettledDispatch, S2CAuctionCard,
};
