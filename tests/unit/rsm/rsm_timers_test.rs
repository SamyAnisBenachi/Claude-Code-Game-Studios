// Story 003: RSM timers + input reader
//
// Runnable tests live in server/tests/rsm_timers_test.rs and are executed by:
// cargo test -p server rsm_timers
//
// Coverage:
// - SessionReady starts DRAFT_INITIAL and its 45s timer.
// - DRAFT_INITIAL, DRAFT_SHOP, and PLACEMENT timers advance only their active phase.
// - DRAFT_SHOP ready signals and PLACEMENT submissions trigger early exits.
// - Stale AuctionSettled and ResolutionComplete messages are phase-guarded.
