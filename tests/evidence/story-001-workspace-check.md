# Test Evidence: Story 001 — Cargo Workspace Scaffolding

**Date**: 2026-04-29
**Story**: production/epics/workspace-and-shared-types/story-001-cargo-workspace-scaffolding.md
**Story Type**: Integration

## Evidence

### cargo check --workspace

Run command: `cargo check --workspace`

Expected: Exit code 0, zero warnings.

> NOTE: Run this manually after completing the scaffolding and paste the output here.
> The story cannot be marked Done until this check passes and output is recorded.

```
STATUS: Rust/Cargo not installed on design machine — run on a build machine.

Commands to run (paste full output below each):

  cd d:\_DEV\claude-code-game-studios
  cargo check --workspace
  # Expected: exit 0, zero warnings

  cargo tree -p shared --prefix none | grep -E "bevy_ecs|tokio|rand_chacha"
  # Expected: NO output (none of these crates in shared/)

  cargo tree -p client --prefix none | grep server
  # Expected: NO output (server crate not in client tree)

[paste cargo check --workspace output here]

[paste cargo tree -p shared grep output here — "no output" is PASS]

[paste cargo tree -p client grep output here — "no output" is PASS]
```

### Cross-crate isolation
- `cargo tree -p client` must NOT contain `server`
- `cargo tree -p shared` must NOT contain `bevy_ecs`, `tokio`, `rand_chacha`

> Run these checks and record results here before /story-done.

## AC Checklist

- [ ] Cargo.toml workspace root: members, resolver, workspace.package, profile.release, profile.dev
- [ ] shared/Cargo.toml: bevy serialize only, lightyear shared only
- [ ] shared/src/lib.rs: pub mod card/config/protocol
- [ ] server/Cargo.toml: headless bevy, correct deps
- [ ] server/src/main.rs: compilable + mod foundation/core/feature
- [ ] server subdirs: foundation/, core/, feature/ with mod.rs
- [ ] client/Cargo.toml: browser bevy, no tokio, no rand_chacha
- [ ] client/src/main.rs: compilable + mod network/state/ui
- [ ] client subdirs: network/, state/, ui/ with mod.rs
- [ ] client/index.html: valid Trunk entry point
- [ ] cargo check --workspace: zero warnings [MANUAL — record output above]
- [ ] bevy_asset_loader: decision documented (comment in client/Cargo.toml)
