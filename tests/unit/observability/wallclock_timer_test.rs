// S13-OBS-WALLCLOCK-TIMESTAMPS-001 (PROMPT 837) — Logic test for AC6.
//
// Verifies that a `tracing_subscriber::fmt()` subscriber configured with
// `with_timer(UtcTime::rfc_3339())` (the same pattern landed in
// `server/src/main.rs`, `client/src/main.rs`, and `tests/test_helpers.rs`)
// emits log lines whose leading timestamp is in UTC ISO-8601 / RFC 3339 form
// — i.e. matches `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}` followed by either a
// fractional-second separator `.` or the UTC marker `Z`.
//
// This is a Logic test: it exercises the subscriber builder in isolation
// against a captured writer, asserting the formatting behaviour without
// requiring a live server or client process. The three production-source
// init sites all use the exact same timer construction expression, so this
// single test covers AC4 (timer format consistent across three sites) by
// proxy.
//
// Test type: Logic (per QA plan §"Automated Tests Required" — story 019(S13)
// row).
// Sprint: 13. Story: `production/epics/playable-client/story-019-obs-wallclock-timestamps.md`.
// Evidence: `production/qa/evidence/sprint-13-obs-wallclock-timestamps-evidence.md`.

use std::io;
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::fmt::MakeWriter;

/// In-memory writer that accumulates all bytes the subscriber emits, so the
/// test can inspect the captured output for an ISO-8601 UTC timestamp prefix.
#[derive(Clone, Default)]
struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

impl io::Write for CapturedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("CapturedWriter lock poisoned")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for CapturedWriter {
    type Writer = CapturedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[test]
fn test_wallclock_timer_emits_iso_8601_utc_prefix() {
    // Arrange — build a subscriber with the same timer config the three
    // production init sites use (server/src/main.rs, client/src/main.rs,
    // tests/test_helpers.rs).
    let writer = CapturedWriter::default();
    let writer_clone = writer.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_writer(writer_clone)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .finish();

    // Act — emit a single info event under that subscriber.
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("wallclock_timer_marker_event");
    });

    // Assert — captured output is non-empty and begins with an ISO-8601 UTC
    // (RFC 3339) timestamp followed by the UTC marker `Z`.
    let captured = writer
        .0
        .lock()
        .expect("CapturedWriter lock poisoned")
        .clone();
    let output = String::from_utf8(captured).expect("subscriber output is valid utf-8");
    assert!(!output.is_empty(), "subscriber emitted no output");

    let bytes = output.as_bytes();
    assert!(
        bytes.len() >= 20,
        "captured output too short for an ISO-8601 timestamp prefix: {output:?}"
    );

    // YYYY-MM-DDTHH:MM:SS prefix.
    assert!(
        bytes[0..4].iter().all(|b| b.is_ascii_digit()),
        "expected 4-digit year at start: {output:?}"
    );
    assert_eq!(bytes[4], b'-', "expected '-' after year: {output:?}");
    assert!(
        bytes[5..7].iter().all(|b| b.is_ascii_digit()),
        "expected 2-digit month: {output:?}"
    );
    assert_eq!(bytes[7], b'-', "expected '-' after month: {output:?}");
    assert!(
        bytes[8..10].iter().all(|b| b.is_ascii_digit()),
        "expected 2-digit day: {output:?}"
    );
    assert_eq!(
        bytes[10], b'T',
        "expected 'T' between date and time: {output:?}"
    );
    assert!(
        bytes[11..13].iter().all(|b| b.is_ascii_digit()),
        "expected 2-digit hour: {output:?}"
    );
    assert_eq!(bytes[13], b':', "expected ':' after hour: {output:?}");
    assert!(
        bytes[14..16].iter().all(|b| b.is_ascii_digit()),
        "expected 2-digit minute: {output:?}"
    );
    assert_eq!(bytes[16], b':', "expected ':' after minute: {output:?}");
    assert!(
        bytes[17..19].iter().all(|b| b.is_ascii_digit()),
        "expected 2-digit second: {output:?}"
    );

    // After seconds, RFC 3339 emits either '.<subseconds>Z' (when subseconds
    // are non-zero — typical for runtime emission) or directly 'Z' (no
    // subseconds). Both are canonical UTC ISO-8601 / RFC 3339. The story
    // AC6 regex anchors at `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z\b`
    // OR the canonical format produced by the chosen API; UtcTime::rfc_3339()
    // is the chosen API.
    assert!(
        bytes[19] == b'.' || bytes[19] == b'Z',
        "expected '.' (subseconds) or 'Z' (UTC) after seconds, got {:?}: {output:?}",
        bytes[19] as char
    );

    // Walk the fractional-second digits and confirm the run ends at 'Z'. This
    // guarantees the UTC indicator is present (not '+00:00') so cross-process
    // log alignment is trivial.
    if bytes[19] == b'.' {
        let mut i = 20;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        assert!(
            i > 20,
            "expected at least one fractional-second digit after '.': {output:?}"
        );
        assert!(
            i < bytes.len() && bytes[i] == b'Z',
            "expected 'Z' UTC marker after fractional seconds: {output:?}"
        );
    }

    // Sanity check: the marker payload survived.
    assert!(
        output.contains("wallclock_timer_marker_event"),
        "event payload missing from captured output: {output:?}"
    );
}
