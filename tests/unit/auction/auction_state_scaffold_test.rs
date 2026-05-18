use server::feature::auction::{auction_snapshot, AuctionPhase, AuctionState};
use shared::card::CardId;
use shared::session::PlayerId;

fn state_with_phase(phase: AuctionPhase) -> AuctionState {
    AuctionState {
        phase,
        card_id: Some(CardId(7)),
        starting_price: 3,
        current_price: 3,
        current_leader: None,
        timer_remaining_ms: 12_000,
        live_bidding_deadline_elapsed_ms: None,
    }
}

#[test]
fn test_snapshot_returns_none_when_phase_is_idle() {
    let state = AuctionState::default();

    assert_eq!(auction_snapshot(&state), None);
}

#[test]
fn test_snapshot_returns_some_for_non_idle_phases_with_card() {
    for phase in [
        AuctionPhase::Selecting,
        AuctionPhase::LiveBidding,
        AuctionPhase::Resolving,
    ] {
        let state = state_with_phase(phase);

        assert!(auction_snapshot(&state).is_some());
    }
}

#[test]
fn test_snapshot_uses_zero_sentinel_when_no_bids_placed() {
    let state = AuctionState {
        phase: AuctionPhase::LiveBidding,
        card_id: Some(CardId(7)),
        starting_price: 3,
        current_price: 3,
        current_leader: None,
        timer_remaining_ms: 12_000,
        live_bidding_deadline_elapsed_ms: None,
    };

    let snapshot = auction_snapshot(&state).expect("live auction with card snapshots");

    assert_eq!(snapshot.card_id, CardId(7));
    assert_eq!(snapshot.starting_price, 3);
    assert_eq!(snapshot.last_accepted_bid, 0);
    assert_eq!(snapshot.current_leader, None);
    assert_eq!(snapshot.timer_remaining_ms, 12_000);
}

#[test]
fn test_snapshot_reflects_last_bid_and_leader_when_bids_exist() {
    let state = AuctionState {
        phase: AuctionPhase::LiveBidding,
        card_id: Some(CardId(3)),
        starting_price: 3,
        current_price: 7,
        current_leader: Some(PlayerId(1)),
        timer_remaining_ms: 5_500,
        live_bidding_deadline_elapsed_ms: None,
    };

    let snapshot = auction_snapshot(&state).expect("live auction with card snapshots");

    assert_eq!(snapshot.card_id, CardId(3));
    assert_eq!(snapshot.starting_price, 3);
    assert_eq!(snapshot.last_accepted_bid, 7);
    assert_eq!(snapshot.current_leader, Some(PlayerId(1)));
    assert_eq!(snapshot.timer_remaining_ms, 5_500);
}

#[test]
fn test_timer_remaining_ms_passes_through_unmodified() {
    for timer_remaining_ms in [0, 1, 19_999, 20_000] {
        let state = AuctionState {
            phase: AuctionPhase::LiveBidding,
            card_id: Some(CardId(9)),
            starting_price: 5,
            current_price: 5,
            current_leader: None,
            timer_remaining_ms,
            live_bidding_deadline_elapsed_ms: None,
        };

        let snapshot = auction_snapshot(&state).expect("live auction with card snapshots");

        assert_eq!(snapshot.timer_remaining_ms, timer_remaining_ms);
    }
}

#[test]
fn test_default_state_is_idle_with_zeroed_fields() {
    let state = AuctionState::default();

    assert_eq!(state.phase, AuctionPhase::Idle);
    assert_eq!(state.card_id, None);
    assert_eq!(state.starting_price, 0);
    assert_eq!(state.current_price, 0);
    assert_eq!(state.current_leader, None);
    assert_eq!(state.timer_remaining_ms, 0);
    assert_eq!(state.live_bidding_deadline_elapsed_ms, None);
}

#[test]
fn test_auction_state_stays_under_size_guardrail() {
    // PROMPT 1091 added `live_bidding_deadline_elapsed_ms: Option<u64>` to
    // anchor the auction settlement clock to wall-clock elapsed time. With
    // 8-byte alignment, this widens the struct past the original 64-byte
    // ceiling. 80 bytes is still well under any meaningful budget for a
    // single per-room resource and leaves headroom for one more u64-sized
    // field before re-evaluation.
    assert!(std::mem::size_of::<AuctionState>() <= 80);
}
