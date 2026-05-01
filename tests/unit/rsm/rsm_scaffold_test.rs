// Story 001: State and Events Scaffold
//
// Runnable tests live in server/tests/rsm_scaffold_test.rs and are executed by:
// cargo test -p server rsm_scaffold
//
// Coverage:
// - RoundState inserts cleanly into a fresh Bevy App and initializes to Lobby.
// - All timers initialize to None and tracking collections are empty.
// - RsmPlugin registers RoundState and all buffered Message resources,
//   including the Auction abort cleanup signal.
// - SessionReady is intentionally excluded from Messages<T>; it is an Observer
//   Event registered via app.observe(on_session_ready).
