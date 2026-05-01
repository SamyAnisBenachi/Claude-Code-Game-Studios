//! Server-authoritative auction feature scaffold.
// Story AUC-001 defines types before Story 007 wires the plugin into the
// server binary, so this module is intentionally unused at this stage.
#![allow(dead_code, unused_imports)]

pub mod snapshot;
pub mod state;
pub mod system;

pub use snapshot::{auction_snapshot, AuctionSnapshot};
pub use state::{AuctionPhase, AuctionState};
pub use system::{auction_tick_system, AuctionCardDrawFixture, S2CAuctionCard};
