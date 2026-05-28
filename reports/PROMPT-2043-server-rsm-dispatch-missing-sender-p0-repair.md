# PROMPT 2043 — SERVER-RSM-DISPATCH-MISSING-SENDER-P0-REPAIR

**Branch:** `work/PROMPT-2043`
**Worktree:** `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2043`
**Source-of-truth base:** `origin/main@135ca0b0` (chain `a295db2a`)

## Failure Mode (pre-repair)

`server/src/network/rsm_dispatch.rs` exposed two systems
(`dispatch_phase_changed`, `dispatch_opponent_disconnected`) that take
`Option<ServerMultiMessageSender>` and `Query<&Server>`. The previous
implementation guarded the send call with `if let (Some(server), Some(sender))
= …` (line 31) and `let (Some(server_handle), Some(sender)) = … else continue`
(line 97). When either dependency was absent the system **silently consumed
the event and dropped the broadcast**:

- no log line of any level;
- no observable counter;
- the `MessageReader` cursor still advanced.

This is the exact failure shape PROMPT 2030 fingerprinted: a server-side
dispatch gap that surfaces upstream as a client/UI phase-sync defect.

## Repair

Owned file: **`server/src/network/rsm_dispatch.rs`**

1. **Introduced `DispatchReadiness` enum + pure `classify_dispatch_readiness`
   helper.** Four states: `Ready` / `MissingSender` / `MissingServer` /
   `Headless`. Pure → unit-testable without spinning lightyear.
2. **Introduced `RsmDispatchDiagnostics` resource** with four `u64` counters
   (`phase_changed_dropped_missing_sender`,
   `phase_changed_dropped_missing_server`,
   `opponent_disconnected_dropped_missing_sender`,
   `opponent_disconnected_dropped_missing_server`). Optional in both
   systems via `Option<ResMut<…>>`, so the resource stays opt-in and does
   not require touching `ServerNetworkPlugin` (outside owned scope) —
   tests insert it; production observers can insert it later.
3. **Rewired both dispatch systems** to classify readiness once per call,
   then branch:
   - `Ready` → real send (with the existing `tracing::error!` on send
     failure preserved).
   - `MissingSender` → `tracing::warn!` with structured fields (phase,
     round, timer_ms, readiness) + counter increment. Loud, never silent.
   - `MissingServer` → same shape, distinct warn text. Loud, never silent.
   - `Headless` (both absent) → `tracing::debug!`, no counter bump. This
     is the legitimate test/headless path the existing outbox-only flow
     relies on (see `rsm_timers_test.rs` etc.) and is deliberately
     non-noisy.
4. **Preserved every existing positive behaviour:** the `RsmNetworkOutbox`
   capture path is unchanged; `OpponentDisconnectNotice` recipient-empty
   skipping and missing `SessionConfig` / `PlayerConnectionMap` debug logs
   are unchanged.

## Regression Test

New file: **`server/tests/rsm_dispatch_missing_sender_test.rs`** (3 tests).

| Test | Pins |
|------|------|
| `test_rsm_dispatch_classify_readiness_all_four_combinations` | Pure helper returns every (server, sender) combination correctly. |
| `test_dispatch_phase_changed_headless_path_captures_in_outbox_and_does_not_increment_counters` | Running `dispatch_phase_changed` headless (no Server, no sender) leaves outbox populated (event was not silently dropped before reaching the branch) **and** leaves all drop counters at zero (headless is not a regression — only partial-wiring is). |
| `test_rsm_dispatch_diagnostics_resource_increments_are_observable_in_tests` | The diagnostics resource is queryable with stable field names and the increments are externally observable — the foundational affordance that turns the formerly silent drop into an asserted one. |

Inline unit module in `rsm_dispatch.rs` adds four more `classify_*` test
permutations as fast-path coverage.

## Validation

- **Path allowlist:** `git diff --cached --stat` confirms only two paths
  touched, both in owned scope:
  ```text
   server/src/network/rsm_dispatch.rs               | 219 ++++++++++++++++++++---
   server/tests/rsm_dispatch_missing_sender_test.rs | 129 +++++++++++++
  ```
  `.claude/settings.json` shows in `git status` as **unrelated session-start
  modification** (hooks file, untouched by this worker).
- **`git diff --check -- server/ reports/`:** clean (no whitespace damage in
  owned scope).
- **Focused build:** `cargo check -p server --tests` → green
  (`Finished dev profile … in 5m 05s`, exit 0).
- **Focused test:** `cargo test -p server --test rsm_dispatch_missing_sender_test`
  → **3 passed, 0 failed**.

  ```text
  running 3 tests
  test test_rsm_dispatch_classify_readiness_all_four_combinations ... ok
  test test_rsm_dispatch_diagnostics_resource_increments_are_observable_in_tests ... ok
  test test_dispatch_phase_changed_headless_path_captures_in_outbox_and_does_not_increment_counters ... ok
  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  ```

- **No broad cargo suite run** (per implementation rules).

## Out of Scope (not claimed)

- **Client phase-sync closure** — not addressed; this is server dispatch
  hardening only. PROMPT 2030's client-side fix stays independent.
- `ServerNetworkPlugin` is **not modified** to register
  `RsmDispatchDiagnostics`; doing so is a 1-line follow-up outside this
  worker's owned scope and does not change the structural-warn contract,
  only the counter visibility in production. Tests already exercise the
  observable contract; the warn log is the production-visible signal.

## Files Changed

- `server/src/network/rsm_dispatch.rs` — modified (added enum, helper,
  diagnostics resource, branch-aware dispatch in both systems, inline
  unit tests).
- `server/tests/rsm_dispatch_missing_sender_test.rs` — new (3 integration
  tests).
- `reports/PROMPT-2043-server-rsm-dispatch-missing-sender-p0-repair.md` —
  this report.

2043: SERVER-RSM-DISPATCH-MISSING-SENDER-P0-REPAIR: SHIPPED
