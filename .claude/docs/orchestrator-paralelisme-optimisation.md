# Orchestrator Parallelism Optimization

Rules to maximize parallel worker throughput, minimize wasted compute, and keep
caches warm across the multi-worker workflow. Apply to every worker prompt that
touches code, tests, or build artifacts.

## Agent Resource Policy

- Workers MUST investigate locally before editing.
- Investigation is **source-only by default**: `rg` / `git grep` / `Read` files.
- Workers MUST NOT run workspace-wide cargo commands.
- Workers MUST NOT run `cargo clean`.
- Use warm caches:
  - `RUSTC_WRAPPER=sccache` when available
  - Persistent worktree slots when assigned (do not destroy/recreate)

## Worker Verification

Run only the **narrowest relevant** command:

```
cargo test -p <crate> --test <test_file> <exact_test_name> -- --exact
```

Then:

```
cargo fmt --check
```

Only run crate-level check if production source changed:

```
cargo check -p <crate> --lib
```

**Forbidden for workers:**
- `cargo test --workspace --tests`
- `cargo check --workspace`
- `cargo test` (bare, no targeting)
- `cargo clean`

Example correct worker verify block:

```
Verify:
1. cargo test -p client --test accessibility_settings_photosensitivity_warning_test test_warning_appears_before_gameplay_exposure -- --exact
2. cargo fmt --check
3. Stop. Report NEEDS_ROOT_SMOKE.

Do not run cargo test --workspace --tests.
Root/orchestrator owns workspace smoke checks after cherry-pick/batch merge.
```

## Root Verification Policy

- Workers prove the exact failing test.
- Orchestrator-root runs **one** workspace smoke after integrating a batch.
- Never ask every worker to run workspace tests independently.

## Fixture Cascade Policy

If multiple failures share the same crate + same root cause + test-only fixture gap:
- Assign them to **one** worker as a mini-batch.
- Max 3–5 files.
- One targeted test per failing test file.
- One final root smoke by orchestrator after merge.

## Priority Application

The three highest-impact changes:

1. Remove `cargo test --workspace --tests` from worker prompts.
2. Force exact-target tests with `-p <crate> --test <file> <test_name> -- --exact`.
3. Keep caches warm: no `cargo clean`, persistent worktrees, `sccache` if available.

## Cross-references

- `.claude/docs/coordination-rules.md` — parallel task protocol, model tier assignments
- `.claude/docs/technical-preferences.md` — file extension routing, specialist agents
- `docs/engine-reference/bevy/VERSION.md` — mandatory skill activation
