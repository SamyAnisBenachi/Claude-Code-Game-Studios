use shared::protocol::AuctionSnapshot;

use crate::feature::auction::state::{AuctionPhase, AuctionState};

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
