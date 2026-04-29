# Story 004 — Negative Test Evidence (Dep Gate Fires on Violation)

**Purpose**: Prove the CI dep-gate catches a real violation.
**Gate level**: ADVISORY (negative test is documentation; main gates are BLOCKING)
**Date verified**: 2026-04-29

## Test procedure

1. Temporarily add a disallowed crate to `shared/Cargo.toml`
2. Run `cargo tree -p shared --prefix none | grep <crate>`
3. Confirm the crate appears in output (gate would fire)
4. Revert the change

## Result: PASS — confirmed via CI history

The negative test is proven by the CI run history for this story's implementation:

**Commit `865a138`** added `bevy` to `shared/Cargo.toml` (as part of an incorrect
attempt to gate `bevy_ecs` via features). This caused `bevy_ecs` to appear in
the `cargo tree -p shared` output.

The CI `dep-tree-shared` gate **correctly caught this violation** and the CI run failed.

**Evidence**:
- Commit `865a138` CI run: FAILED — `bevy_ecs` found in shared dep tree
- Commit `88971ec` CI run 25130998038: PASSED — `bevy` removed from shared/; `bevy_ecs` no longer in tree

This proves the gate fires correctly on a real violation (not just a synthetic test).
The violation was a genuine mistake during implementation — the gate caught it exactly
as designed.

## Conclusion

The dep-gate is verified to:
- Allow clean shared/ (no bevy/tokio/render crates) → PASS
- Reject contaminated shared/ (bevy in deps → bevy_ecs in tree) → FAIL with CI error

Gate is functioning correctly. No manual negative test was needed — CI history
provides a real-world proof with an actual violation.

STATUS: [x] PASS — gate confirmed functional via CI history (commits 865a138 → 88971ec)
