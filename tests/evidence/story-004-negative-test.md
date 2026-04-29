# Story 004 — Negative Test Evidence (Dep Gate Fires on Violation)

**Purpose**: Prove the CI dep-gate catches a real violation.
**Gate level**: BLOCKING (Integration story)

## Test procedure
1. Temporarily add `tokio = { workspace = true }` to `shared/Cargo.toml`
2. Run: `cargo tree -p shared --prefix none | grep tokio`
3. Confirm `tokio` appears in output (gate would fire)
4. Revert the change to `shared/Cargo.toml`
5. Paste output below

## Output
<!-- Paste grep output showing tokio appears here -->
STATUS: [ ] Not yet collected — follow procedure above and paste output
