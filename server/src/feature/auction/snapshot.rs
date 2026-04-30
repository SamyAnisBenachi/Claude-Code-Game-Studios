use shared::card::CardId;
use shared::session::PlayerId;

use crate::feature::auction::state::{AuctionPhase, AuctionState};

/// Reconnect-safe read model for the current auction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionSnapshot {
    /// Card currently being auctioned.
    pub card_id: CardId,
    /// Auction floor for the selected card's rarity.
    pub starting_price: u32,
    /// Zero when no bid has been accepted; otherwise the last accepted bid.
    pub last_accepted_bid: u32,
    /// Current auction leader, or `None` before the first accepted bid.
    pub current_leader: Option<PlayerId>,
    /// Authoritative remaining auction timer, in milliseconds.
    pub timer_remaining_ms: u32,
}

/// Build a reconnect snapshot from auction state.
///
/// Returns `None` in `Idle`. For defensive robustness, malformed non-idle
/// states without a card also return `None` instead of panicking.
pub fn auction_snapshot(state: &AuctionState) -> Option<AuctionSnapshot> {
    if state.phase == AuctionPhase::Idle {
        return None;
    }

    state.card_id.map(|card_id| AuctionSnapshot {
        card_id,
        starting_price: state.starting_price,
        last_accepted_bid: last_accepted_bid(state),
        current_leader: state.current_leader,
        timer_remaining_ms: state.timer_remaining_ms,
    })
}

fn last_accepted_bid(state: &AuctionState) -> u32 {
    if state.current_leader.is_some() {
        state.current_price
    } else {
        0
    }
}
