# PROMPT 1326 -- Windows Launcher Dedicated Main Checkout Main-Land

Status: LANDED (refresh + push to origin/main)

## Mode

Tooling main-land refresh. Executed inside a fresh worktree
(`D:/_DEV/claude-code-game-studios-worktrees/windows-launcher-main-land-1326`,
branch `work/windows-launcher-main-land-1326`); the orchestrator root checkout
(dirty with a staged Sprint 18 QA plan file) was not touched.

## Source refs

| Ref | Value |
|---|---|
| Source branch (input) | `origin/integrate/windows-launcher-dedicated-main-checkout-1322` |
| Source tip before refresh | `b34afe749f3d499bf01b1695dc3abb692bcf5b50` (matches expected) |
| `origin/main` at launch | `5c6e721771c09bf0ec27e85f2284a6b2302f0336` (PROMPT 1321 story-authoring combined main-land after qa-plan; matches expected) |
| Common base | `6e885b7a732a79ef29fd618908374d78402dc398` (PROMPT 1320 qa-plan main-land) |
| Refreshed branch tip (pre-push) | `ee8f545b77ca66737342c419439e891e3a425b3a` |
| Final `origin/main` tip (post-push) | `ee8f545b77ca66737342c419439e891e3a425b3a` |

## Content-disjoint check vs main since base

`git diff --stat 6e885b7..origin/main` returned only `production/epics/**` and
`reports/PROMPT-13{06,11,13,15,21}-*.md` (3721 insertions / 8 deletions across
15 files -- all Sprint 18 story-authoring docs).

`git diff --stat 6e885b7..origin/integrate/windows-launcher-dedicated-main-checkout-1322`
returned only the launcher allow-list (1408 insertions / 91 deletions across
7 files): `docs/setup/dev-two-button-launcher.md`,
`reports/PROMPT-130{9}`, `reports/PROMPT-131{6}`, `reports/PROMPT-1322`,
`tools/dev-launcher-app/src/main.rs`, `tools/dev-launcher/Start-TwoClients.ps1`,
`tools/dev-launcher/Update-LatestMain.ps1`.

No file overlap, so a content-disjoint rebase was the correct strategy.

## Refresh operation

Rebased `work/windows-launcher-main-land-1326` (initialized at source tip
`b34afe7`) onto current `origin/main` (`5c6e721`). Rebase applied all 3
commits with no conflicts:

```
ee8f545 report(prompt-1322): windows launcher dedicated main checkout refresh on 6e885b7 (no main push)
e21f4e9 report(prompt-1316): windows launcher dedicated main checkout integration
0c8c955 PROMPT-1309 launcher dedicated play/build checkout
5c6e721 report(prompt-1321): s18 story-authoring combined refresh + main-land after qa-plan (PROMPT 1320)   <-- origin/main
```

`git merge-base --is-ancestor origin/main HEAD` -> success, confirming
fast-forward eligibility for the push.

`git diff --check origin/main HEAD` -> clean (no whitespace / conflict marker
issues).

## Final allowed-file diff (vs current origin/main)

```
 docs/setup/dev-two-button-launcher.md              | 204 +
 reports/PROMPT-1309-windows-launcher-dedicated-main-checkout-repair.md       | 229 +
 reports/PROMPT-1316-windows-launcher-dedicated-main-checkout-integration.md  | 128 +
 reports/PROMPT-1322-windows-launcher-dedicated-main-checkout-refresh.md      | 174 +
 reports/PROMPT-1326-windows-launcher-dedicated-main-checkout-main-land.md    | <this file>
 tools/dev-launcher-app/src/main.rs                 | 507 +-
 tools/dev-launcher/Start-TwoClients.ps1            |  50 +-
 tools/dev-launcher/Update-LatestMain.ps1           | 207 +-
```

Every changed path is on the prompt's allow list. No file outside the allow
list was touched.

### Forbidden-surface scan (negative confirmation)

None of the following surfaces were modified in the diff vs origin/main:

- `production/sprint-status.yaml`, `production/session-state/**`,
  `production/stage.txt`, `production/sprints/**`, `production/qa/**`,
  `production/gate-checks/**`, `production/epics/**`
- `client/**`, `server/**`, `shared/**`, `tests/**` (no test files outside
  the inline `#[cfg(test)] mod tests` block in
  `tools/dev-launcher-app/src/main.rs`)
- Workspace `Cargo.toml` / `Cargo.lock` (only tool-local
  `tools/dev-launcher-app/Cargo.toml` was touched in the original launcher
  commit, and it was unchanged by this main-land; the workspace manifest /
  lock are untouched)
- `docs/architecture/**`, `design/**`, `.claude/**`

## Verification

### Dedicated-main-checkout repair invariants (code inspection)

Reviewed `tools/dev-launcher-app/src/main.rs` against the prompt's required
invariants:

| Invariant | Evidence |
|---|---|
| Launcher does not use mutable orchestrator root if its checkout is on a work/* branch | Sidecar acceptance gate: `read_head_branch` is required to return `Some("main")`; any other branch (e.g. `work/...`) causes fall-through to the canonical-candidate list (constants at lines 48-53, logic around 962-1046; tested by `resolve_repo_root_sidecar_on_worker_branch_falls_to_canonical`). |
| Launcher resolves or creates a dedicated stable main checkout for "Rebuild Latest Main" and "Start Two-Client Play Session" | Separate `PLAY_REPO_DEFAULT = D:\_DEV\ccgs-play-main` plus `CCGS_PLAY_REPO_ROOT` (primary) and `CCGS_CANONICAL_MAIN_ROOT` (legacy alias) env overrides; `resolve_play_root_*` reports status (`OnMain`, `OtherBranch`, `Detached`, `Missing`, `Invalid`) and is passed to both scripts via `-PlayRepoRoot` (PowerShell wiring at line 780). Update-LatestMain.ps1 / Start-TwoClients.ps1 own creation/refresh of the worktree; the launcher EXE never destructively switches the orchestrator root. |
| Valid `CCGS_REPO_ROOT` and sidecar behavior remain sane | Env > Sidecar (only on main) > Canonical fallback > exe walkup, matching pre-1290 docs. Covered by `resolve_repo_root_prefers_env_when_valid`, `resolve_repo_root_falls_through_invalid_env_to_sidecar_on_main`, `resolve_repo_root_sidecar_on_main_is_accepted_without_canonical_fallback`, and `resolve_repo_root_uses_exe_walkup_when_sidecar_absent`. |

### Launcher-app test suite

Ran with the Windows/MSVC Cargo policy explicitly in env before the first
Cargo command:

```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

Confirmation that the policy was honored: build output reported
`Running unittests src\main.rs (D:\_DEV\cargo-target\ccgs-msvc\debug\deps\ccgs_dev_launcher-0427248ed90834db.exe)`.

```
cargo test --bin ccgs-dev-launcher
...
test result: ok. 57 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.01s
```

All 57 tests passing covers sidecar parse, env override / fallback,
worker-branch rejection, play-root resolution & status, evidence-dir parsing,
log truncation, and diagnostics formatting.

### git diff --check

`git diff --check origin/main HEAD` -> exit 0, no output. Clean.

## Push

This is explicit orchestrator main-land authorization. The branch is a
strict descendant of `origin/main`, so the push is a fast-forward.

Executed:

```
git push origin work/windows-launcher-main-land-1326:refs/heads/main
```

Outcome: success. `origin/main` advanced from
`5c6e721771c09bf0ec27e85f2284a6b2302f0336` to
`ee8f545b77ca66737342c419439e891e3a425b3a` (the 3 launcher commits cleanly
applied on top). No force flag was used.

## Non-claims

- Did not run the launcher EXE end-to-end against a live two-client play
  session; verification was confined to unit tests + code inspection per
  the prompt's "Run launcher-app tests or equivalent targeted verification".
- Did not modify the orchestrator root checkout (still on
  `mainland/s18-server-dead-state-hygiene-1315` with a staged Sprint 18 QA
  plan); the worktree-based main-land deliberately avoided that dirty tree.
- Did not author or modify any sprint/epic/story/QA/gate-check files; the
  launcher repair is content-disjoint from the Sprint 18 story-authoring
  work that landed in PROMPT 1321.
- Did not change workspace `Cargo.toml` or `Cargo.lock`; the only Rust
  manifest change in the underlying source branch is to
  `tools/dev-launcher-app/Cargo.toml`, which was already in place and was
  not retouched here.
- Did not exercise the play-root worktree creation path
  (Update-LatestMain.ps1 git-worktree-add) in this session.

## Final status line

```
1326: WINDOWS-LAUNCHER-DEDICATED-MAIN-CHECKOUT-MAIN-LAND: LANDED
```
