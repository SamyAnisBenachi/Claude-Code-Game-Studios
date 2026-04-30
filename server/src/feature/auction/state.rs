use bevy::prelude::Resource;
use shared::card::CardId;
use shared::session::PlayerId;

/// Server-authoritative auction state machine state.
///
/// `auction_tick_system` is the only system that may hold `ResMut<AuctionState>`
/// once auction processing is implemented.
#[derive(Resource, Debug, Clone)]
pub struct AuctionState {
    /// Current auction phase.
    pub phase: AuctionPhase,
    /// `None` in `Idle`; set after the auction card is selected.
    pub card_id: Option<CardId>,
    /// Auction floor for the selected card's rarity.
    pub starting_price: u32,
    /// Starting price until the first accepted bid; last accepted bid thereafter.
    pub current_price: u32,
    /// `None` until the first bid is accepted.
    pub current_leader: Option<PlayerId>,
    /// Authoritative remaining auction timer, in milliseconds.
    pub timer_remaining_ms: u32,
}

/// Auction state-machine phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionPhase {
    /// No auction is active.
    Idle,
    /// The server is selecting and announcing an auction card.
    Selecting,
    /// Bids are accepted and the timer is counting down.
    LiveBidding,
    /// Settlement is executing synchronously.
    Resolving,
}

impl Default for AuctionState {
    fn default() -> Self {
        Self {
            phase: AuctionPhase::Idle,
            card_id: None,
            starting_price: 0,
            current_price: 0,
            current_leader: None,
            timer_remaining_ms: 0,
        }
    }
}
