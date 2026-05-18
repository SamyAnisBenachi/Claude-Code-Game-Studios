use bevy::prelude::Resource;
use shared::card::CardId;
use shared::session::PlayerId;

/// Server-authoritative auction state machine state.
///
/// `auction_tick_system` is the only system that may mutably access this resource
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
    /// Authoritative remaining auction timer, in milliseconds. Decremented
    /// each `auction_tick_system` run via `time.delta()`. The absolute deadline
    /// in [`AuctionState::live_bidding_deadline_elapsed_ms`] acts as a safety
    /// net so the auction settles in bounded wall-clock time even when the
    /// system's per-tick delta accumulation under-counts (PROMPT 1091:
    /// observed 149s drain of a 20s timer in run-7, 2026-05-17).
    pub timer_remaining_ms: u32,
    /// Absolute wall-clock-elapsed time (in milliseconds, from
    /// `Time<Real>::elapsed()`) at which the auction must expire.
    ///
    /// `Some(_)` while in `LiveBidding` (set on phase entry, recomputed on
    /// every accepted bid). `None` otherwise. `None` also when the system
    /// has no `Time<Real>` resource (e.g., unit tests that bypass the schedule).
    ///
    /// PROMPT 1091: this field is the authoritative settlement clock. The
    /// per-tick `timer_remaining_ms` decrement remains for the broadcast
    /// contract, but settlement is gated on this absolute deadline. The
    /// anchor is `Time<Real>` (not `Time<Virtual>`) because
    /// `Time<Virtual>::max_delta` is capped at 250ms by default, which
    /// makes the Virtual delta under-count when `Update` fires sparsely —
    /// the exact root cause of the 149s LiveBidding stall reproduced in
    /// AUDIT-1076-12.
    pub live_bidding_deadline_elapsed_ms: Option<u64>,
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
            live_bidding_deadline_elapsed_ms: None,
        }
    }
}
