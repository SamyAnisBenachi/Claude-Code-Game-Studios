//! Shared test helpers for integration tests across the workspace.
//!
//! ## `init_test_tracing`
//!
//! Installs a `tracing-subscriber` once per test process so integration tests
//! can see `tracing::*` output emitted by the systems under test. Output is
//! routed through `with_test_writer()` so it integrates with cargo's test
//! capture: visible only on test failure or when invoked with `--nocapture`.
//!
//! Idempotent — safe to call from many tests in the same binary; the
//! subscriber is installed at most once per process via `std::sync::Once`.
//!
//! ## Usage
//!
//! Because there is no top-level integration-test crate, include this file
//! via `#[path]` from any integration test. The parent crate that owns the
//! `[[test]]` entry must depend on `tracing-subscriber` (both `client` and
//! `server` already do, as of S11-TD-CLIENT-LOG-001 / PROMPT 647):
//!
//! ```ignore
//! #[path = "../../test_helpers.rs"]
//! mod test_helpers;
//!
//! #[test]
//! fn my_test() {
//!     test_helpers::init_test_tracing();
//!     // ... systems-under-test that emit tracing::info! / tracing::warn! ...
//! }
//! ```
//!
//! The default filter quiets wgpu / naga / bevy_ecs noise; override with
//! `RUST_LOG=...` (e.g. `RUST_LOG=client=debug cargo test --nocapture ...`).

use std::sync::Once;

static INIT: Once = Once::new();

/// Install a `tracing-subscriber` for the current test process if one is not
/// already installed. Idempotent — repeated calls are no-ops.
///
/// Reads `RUST_LOG` if set; otherwise applies a default filter that shows
/// `info` for app code and `warn` for renderer / shader noise.
pub fn init_test_tracing() {
    INIT.call_once(|| {
        use tracing_subscriber::fmt::time::UtcTime;
        use tracing_subscriber::EnvFilter;
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,wgpu=warn,wgpu_hal=warn,naga=warn,bevy_ecs=info")
        });
        // S13-OBS-WALLCLOCK-TIMESTAMPS-001 (PROMPT 837): UTC ISO-8601 (RFC 3339)
        // timer matches server/src/main.rs and client/src/main.rs so a
        // multi-process trace stitched together at the harness layer aligns at
        // sub-second precision.
        //
        // try_init returns Err if a subscriber is already set (e.g. by another
        // helper); the Once guard plus this fallback together ensure we never
        // panic on duplicate installation.
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_timer(UtcTime::rfc_3339())
            .with_test_writer()
            .try_init();
    });
}
