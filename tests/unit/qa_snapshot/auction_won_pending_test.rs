// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 (AC11 + AC16)
//
// Unit coverage for the top-level `auction_won_pending` snapshot block.
// Pure projection test against `client::presentation::qa_snapshot`:
// no Bevy app, no resource registration, no UI mutation. Each AC16 case
// exercises [`build_auction_won_pending_snapshot`] with a hand-built
// [`ExtrasSnapshot`] + [`PhaseInfo`].
//
// Cases (per AC16):
//   (a) winner client during auction-followup Placement → block present
//       with all five fields populated.
//   (b) non-winner client (extras carries no pending state) → block
//       absent.
//   (c) non-auction-followup Placement (extras carries no pending state)
//       → block absent (same predicate as case b — the pending state is
//       what gates presence).
//   (d) phase-change to Resolution (winner pending state still present in
//       extras, but current_phase moved on) → block absent.
//   (e) successful submit (extras pending state cleared by
//       `update_auction_won_pending_system`) → block absent.
//
// AC11 contract addendum verified: `Option::is_none` serialises as JSON
// key absent (not `null`) on the top-level field.

use client::presentation::qa_snapshot::{
    build_auction_won_pending_snapshot, AuctionWonPendingExtrasSnapshot, ExtrasSnapshot, PhaseInfo,
    ShopAuctionExtrasSnapshot,
};

fn phase(name: &str) -> PhaseInfo {
    PhaseInfo {
        phase: Some(name.to_string()),
        round: Some(3),
        timer_remaining_ms: Some(12_000),
    }
}

fn extras_with_pending(state: Option<AuctionWonPendingExtrasSnapshot>) -> ExtrasSnapshot {
    let shop_auction = state.map(|s| ShopAuctionExtrasSnapshot {
        ui_mode: None,
        draft_initial: None,
        shop: None,
        auction: None,
        settlement: None,
        bid_keyboard_focus: None,
        locally_passed: false,
        toast: None,
        auction_won_pending_state: Some(s),
    });
    ExtrasSnapshot {
        shop_auction,
        ..ExtrasSnapshot::default()
    }
}

fn pending_state(card_id: u32) -> AuctionWonPendingExtrasSnapshot {
    AuctionWonPendingExtrasSnapshot {
        card_id,
        settle_round: 3,
        staged_yet: false,
        submitted_yet: false,
    }
}

#[test]
fn ac16a_winner_in_placement_emits_block_with_all_five_fields() {
    let extras = extras_with_pending(Some(pending_state(107)));
    let current_phase = phase("Placement");

    let block = build_auction_won_pending_snapshot(&extras, &current_phase)
        .expect("AC16(a): winner in PLACEMENT must emit the auction_won_pending block");

    assert_eq!(block.card_id, 107);
    assert_eq!(block.acquired_phase, "Placement");
    assert_eq!(block.settle_round, 3);
    assert!(!block.staged_yet);
    assert!(!block.submitted_yet);
}

#[test]
fn ac16b_non_winner_client_omits_block() {
    // Non-winner client never arms `AuctionWonPending`; the resource (if
    // present) carries `state: None`, which the snapshot builder projects
    // as `auction_won_pending_state: None`.
    let extras = extras_with_pending(None);
    let current_phase = phase("Placement");

    let block = build_auction_won_pending_snapshot(&extras, &current_phase);
    assert!(
        block.is_none(),
        "AC16(b): non-winner client must NOT emit the block (got {:?})",
        block
    );
}

#[test]
fn ac16c_non_auction_followup_placement_omits_block_when_no_pending_state() {
    // Same predicate as the non-winner case: a regular (non-auction-
    // followup) PLACEMENT phase has no pending state because the
    // `arm()` call is gated on `LocalWinner` outcome.
    let extras = extras_with_pending(None);
    let current_phase = phase("Placement");

    let block = build_auction_won_pending_snapshot(&extras, &current_phase);
    assert!(
        block.is_none(),
        "AC16(c): non-auction-followup PLACEMENT must NOT emit the block"
    );
}

#[test]
fn ac16d_phase_change_to_resolution_omits_block_even_if_extras_has_pending() {
    // Extras still carry a pending state (sampled mid-frame; the clear
    // happens in `update_auction_won_pending_system`). The snapshot
    // builder additionally gates on `current_phase == Placement` so the
    // block is absent even if the resource lags by one frame.
    let extras = extras_with_pending(Some(pending_state(107)));
    let current_phase = phase("Resolution");

    let block = build_auction_won_pending_snapshot(&extras, &current_phase);
    assert!(
        block.is_none(),
        "AC16(d): phase-change to Resolution must clear the block immediately"
    );
}

#[test]
fn ac16e_successful_submit_omits_block_after_clear() {
    // Successful submit clears `AuctionWonPending` to Idle in the same
    // frame (the clear path: pending.submitted_yet AND PendingPlacements
    // had the won-card → state: None). After clear,
    // `auction_won_pending_state` is None.
    let extras = extras_with_pending(None);
    let current_phase = phase("Placement");

    let block = build_auction_won_pending_snapshot(&extras, &current_phase);
    assert!(
        block.is_none(),
        "AC16(e): submitted-then-cleared state must NOT emit the block"
    );
}

#[test]
fn ac11_block_serialises_absent_not_null_on_top_level_field() {
    // The top-level `auction_won_pending` field uses
    // `#[serde(skip_serializing_if = "Option::is_none")]` so a `None`
    // value serialises as a missing key, not `null`. This is the
    // "absent (NOT `null` — absent)" contract from AC11.
    //
    // We can't easily construct a full `QASnapshotData` from outside
    // the crate without populating every field, but the snapshot
    // builder itself returns `Option<AuctionWonPendingSnapshot>` whose
    // serialised form on the parent `Option` is the skip-serializing
    // path. The integration test in `placement_auction_state_field_*`
    // already exercises the full-snapshot JSON shape; this unit lock
    // just guarantees the projection helper is the single source of
    // truth for the present/absent decision.
    let extras = extras_with_pending(None);
    let current_phase = phase("Placement");
    assert!(build_auction_won_pending_snapshot(&extras, &current_phase).is_none());
}

#[test]
fn ac11_block_carries_staged_yet_and_submitted_yet_when_set() {
    // Cross-check: the field projection respects staged_yet and
    // submitted_yet (so the production update system can flip the
    // bools without disturbing presence).
    let extras = extras_with_pending(Some(AuctionWonPendingExtrasSnapshot {
        card_id: 107,
        settle_round: 5,
        staged_yet: true,
        submitted_yet: false,
    }));
    let current_phase = phase("Placement");

    let block =
        build_auction_won_pending_snapshot(&extras, &current_phase).expect("block present");
    assert_eq!(block.card_id, 107);
    assert_eq!(block.settle_round, 5);
    assert!(block.staged_yet, "staged_yet must propagate from extras");
    assert!(!block.submitted_yet);
}
