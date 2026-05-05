# QA-COND-0002: Ignored AUC-006 Auction Test

| Field | Value |
|---|---|
| ID | QA-COND-0002 |
| Kind | Test Debt |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Closed |
| Action State | N/A - Closed |
| Reported | 2026-05-05 |
| Source | Sprint 5 smoke report and QA sign-off |

## Summary

QA-COND-0002 is resolved after the AU19-a repair in commit `2bf7078`
(`fix auction abort resolving settlement guard`). The formerly ignored auction
abort settlement guard is now active and passing, and the related
auction/displacement regression batch has no ignored tests.

## Source Evidence

- `production/qa/smoke-2026-05-05.md` records the auction/displacement batch as
  passing with one ignored test.
- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records one auction abort
  test intentionally ignored for older AUC-006 settlement scope.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries the ignored auction test forward as an open QA condition.
- AU19-a repair commit: `2bf7078` (`fix auction abort resolving settlement
  guard`).

## Expected Closure Evidence

Satisfied by the first closure path: the ignored test was updated, unignored,
and passes in the relevant auction test target.

## Closure Evidence

Captured 2026-05-05 from `D:\_DEV\claude-code-game-studios`.

- `rg -n "#\\[ignore\\]" tests/unit/auction/auction_abort_handler_test.rs`
  returned no matches.
- `cargo --config "build.rustflags=['-C','link-arg=/DEBUG:NONE']" --config
  profile.dev.debug=0 --config profile.test.debug=0 test --target-dir
  C:\Users\Sam\.codex\memories\qa-cond-0002-target -p server --test
  auction_abort_handler_test` passed: 4 passed; 0 failed; 0 ignored.
- `cargo --config "build.rustflags=['-C','link-arg=/DEBUG:NONE']" --config
  profile.dev.debug=0 --config profile.test.debug=0 test --target-dir
  C:\Users\Sam\.codex\memories\qa-cond-0002-target -p server --test
  displacement_keywords_test --test auction_state_scaffold_test --test
  auction_phase_entry_test --test auction_reservation_test --test
  auction_bid_validation_gate_test --test auction_resolution_settlement_test
  --test auction_abort_handler_test` passed: 43 passed; 0 failed; 0 ignored
  across the auction/displacement regression batch.

Execution note: the default repo target on `D:` could not complete the Cargo
test link step because the drive had approximately 0.24 GB free and MSVC PDB
generation hit linker limits. The passing evidence used a writable `C:` target
directory and `/DEBUG:NONE` linker override only; no source code or Cargo
configuration files were edited.

## Current Blocker Status

Closed. QA-COND-0002 is no longer Sprint 6 validation test debt after the
AU19-a repair evidence confirmed no ignored auction abort tests and a passing
auction/displacement regression batch.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit auction code or tests.
- Does not change the ignored-test state.
