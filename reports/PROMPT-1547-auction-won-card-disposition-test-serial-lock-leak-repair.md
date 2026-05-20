# PROMPT 1547 — AUCTION-WON-CARD-DISPOSITION-TEST-SERIAL-LOCK-LEAK-REPAIR

- Source-of-truth: `origin/main@f341d6c5156eb22544a05c1834d7179f560bf317`.
- Worker worktree: `D:/tmp/wt-1547` (branch `work/auction-disposition-test-lock-1547`).
- Shared target dir: `D:/tmp/wt-1547/target` (`CARGO_TARGET_DIR`).

## Root cause (test-only)

`tests/integration/auction/auction_won_card_disposition_test.rs` holds three
sibling integration tests against the auction settle path. All three call
`enter_auction(...)` + `settle_expired_auction` indirectly, which emits the
shared `server::game event = "auction_settled"` tracing line with a
per-test `card_id`:

- Case A → `card_id = 107`
- Case B → `card_id = 207`
- AC13   → `card_id = 307`

A global `tracing_subscriber::Registry` + `CaptureLayer` is installed once
via `install_capture_subscriber()` and writes to a shared
`OnceLock<Arc<Mutex<Vec<CapturedEvent>>>>`. Case A and Case B serialize
their critical section with `let _serial = test_serial_lock();` and bracket
their `app.update()` with `take_captured()` calls (clear → emit → read).

`ac13_won_card_persists_in_hand_across_settle_with_no_submission` did
**not** install the capture subscriber and did **not** join the serial
guard. Under default multi-thread `cargo test`, AC13 can run between Case
A's clear and read, leaking its `card_id=307` event into the shared queue.
`find_auction_settled_event(&captured)` returns the first match — AC13's —
which causes Case A's `card_id` field assertion (expected `"107"`, actual
`"307"`) to fail at line 314. PROMPT 1536's verify lane captured the exact
failure stdout and reproduced the pass under `--test-threads=1`,
confirming a test-isolation defect rather than a product regression.

## Fix

Single test file, single function head edit — no product code changed, no
new test framework dependencies, no Cargo edits. AC13 now joins the same
serial discipline as Case A / Case B:

```rust
#[test]
fn ac13_won_card_persists_in_hand_across_settle_with_no_submission() {
    // PROMPT 1547 — `enter_auction` + `settle_expired_auction` emit the
    // shared `server::game event = "auction_settled"` tracing line. Without
    // joining the same serial guard + capture subscriber used by Case A/B,
    // this test's `card_id=307` event can leak into the capture queue while
    // a sibling test sits between its `take_captured()` clear and read,
    // flipping their `card_id` field assertion under default multi-thread
    // `cargo test`. Hold the lock for the whole body even though we don't
    // read captured events here.
    install_capture_subscriber();
    let _serial = test_serial_lock();
    take_captured();
    let winner = player(1);
    ...
}
```

The lock guard binds to `_serial` and is held for the full test body,
covering every `app.update()` that might emit. `install_capture_subscriber()`
keeps callsite Interest cache consistent with sibling tests (idempotent
via `SUBSCRIBER_INSTALL: OnceLock<()>`). `take_captured()` clears any
spillover from a prior test in the same binary so AC13's `app.update()`
runs against a clean queue.

The fix is deterministic and avoids sleep/time-based gating. Only AC13
needed the guard — Case A / Case B already had it.

## Files changed

- `tests/integration/auction/auction_won_card_disposition_test.rs`
  — added 3-line guard + comment block on `ac13_…` test head (lines 418–432
  region after edit).

## Focused validation

Worktree: `D:/tmp/wt-1547`, `CARGO_TARGET_DIR=D:/tmp/wt-1547/target`.

```
cargo test -p server --test auction_won_card_disposition_test
running 3 tests
test ac13_won_card_persists_in_hand_across_settle_with_no_submission ... ok
test case_a_winner_settle_grants_card_and_emits_ac10_trace_line ... ok
test case_b_no_winner_settle_grants_no_card_and_emits_ac10_trace_line ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.04s
```

Default `cargo test` parallelism — no `--test-threads=1` needed. The exact
PROMPT 1536 failure mode (`case_a` reads `card_id="307"` instead of
`"107"`) is no longer reproducible.

Broader Cargo suites deferred to a VERIFY lane per orchestrator policy.

## Allowlist + diff hygiene

- Edits limited to owned scope (`tests/integration/auction/auction_won_card_disposition_test.rs`
  + this report).
- `git diff --check` clean in the worker worktree (no whitespace
  violations). Root checkout had pre-existing unrelated dirt
  (`.claude/settings.json`) not touched by this lane.

## Branch / commit / push

- Branch: `work/auction-disposition-test-lock-1547`
- Commit: see DONE summary
- Push status: see DONE summary

## Final line

1547: AUCTION-WON-CARD-DISPOSITION-TEST-SERIAL-LOCK-LEAK-REPAIR: SHIPPED
