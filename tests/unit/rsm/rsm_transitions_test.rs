// Story 002: advance_phase + F2 Ordering
//
// Runnable tests live in server/tests/rsm_transitions_test.rs and are executed by:
// cargo test -p server rsm_transitions
//
// Coverage:
// - RSM Formula F1: auction routing every third round, debug guard for round 0.
// - advance_phase double-transition guard via PhaseAdvanceRequest.expected_source.
// - All source phases are covered, including terminal GameOver no-op.
// - Draft entry events emit DraftStarted, per-player ShopRefreshNeeded,
//   optional AuctionPhaseEntered, and BroadcastPhaseChanged payloads.
// - Placement and Resolution entry payloads, submission clearing, and GameOver
//   payload/timer invariants.
