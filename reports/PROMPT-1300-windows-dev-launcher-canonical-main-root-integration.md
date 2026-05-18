# PROMPT 1300 — Windows Dev Launcher Canonical-Main Repair Integration

## Status
READY_FOR_MAIN_LAND

## Summary
Cherry-picked the PROMPT 1290 functional + doc changes (`d8894d1a5ad2db9d707420bab0c4003f95687ff2`) onto a fresh worktree based on latest `origin/main` (`1345c6b8b1cbd543dbd63d279186c93924ca54db`). All later main commits (PROMPT 1281 fmt repair, PROMPT 1279/1288/1289 closeout chain, PROMPT 1285 Sprint 18 plan draft) are preserved — no fast-forward of main to the older worker branch.

## Inputs
- Source worker branch: `origin/work/windows-launcher-canonical-main-root-1290`
- Source worker commit: `d8894d1a5ad2db9d707420bab0c4003f95687ff2`
- Latest `origin/main` at integration: `1345c6b8b1cbd543dbd63d279186c93924ca54db`
- Worker branch base: `d73e25e49519a214f8fb0fefa1e78351ccd74795` (ancestor of `origin/main` — clean cherry-pick, no conflicts)

## Outputs
- Integration worktree: `D:\_DEV\claude-code-game-studios-worktrees\windows-launcher-canonical-main-integration-1300`
- Integration branch: `integrate/windows-launcher-canonical-main-root-1300`
- Integration commit: `a6b4eda` (cherry-pick of `d8894d1` with `-x` reference)
- Pushed to: `origin/integrate/windows-launcher-canonical-main-root-1300`

## Diff vs origin/main (4 files, +817 / -46)
```
M docs/setup/dev-two-button-launcher.md            117 +/-
A reports/PROMPT-1290-windows-dev-launcher-repo-root-canonical-main-repair.md  162 +
M tools/dev-launcher-app/src/main.rs               450 +/-
M tools/dev-launcher/build-launcher-exe.ps1        134 +/-
```

All paths are on the allowed list for this integration. Forbidden trees (`production/sprint-status.yaml`, `production/session-state/**`, `production/stage.txt`, `production/sprints/**`, `production/qa/**`, `client/**`, `server/**`, `shared/**`, `tests/**`, top-level `Cargo.*`) are untouched.

## Report file decision (reports/)
`reports/` is gitignored by default (`.gitignore:33`), but specific reports are explicitly tracked when intentionally committed by the source PROMPT — for example, `reports/PROMPT-1229-s18-qa-snapshot-placement-auction-state.md` is already tracked on `origin/main`. The worker commit `d8894d1` intentionally adds `reports/PROMPT-1290-...md` (162 lines of integration evidence), matching that convention, so it is kept in the integration commit.

## Preservation checks
- `production/sprints/sprint-18.md` still present, last touched by `1345c6b` (PROMPT 1285). Not modified or deleted.
- PROMPT 1289 closeout/QA evidence chain (commits `4efe800`, `ec6e5da`) untouched.
- `git log origin/main..HEAD` returns exactly one commit: `a6b4eda PROMPT-1290 launcher canonical-main sidecar repair`.
- `git diff --check origin/main HEAD` — clean (no whitespace errors, no conflict markers).

## Verification (launcher crate only)

Windows/MSVC Cargo resource policy applied:
```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

- `cargo fmt --manifest-path tools/dev-launcher-app/Cargo.toml -- --check`: **clean**
- `cargo test --manifest-path tools/dev-launcher-app/Cargo.toml`: **41/41 passed**, 0 failed, 0 ignored. Includes the 10 new resolution/branch-detection tests added by PROMPT 1290 (`resolve_repo_root_*`, `read_head_branch_*`, `canonical_repo_candidates_*`, etc.).

No full workspace tests were run — out of scope for this integration.

## Push policy
Per coordination rules ("workers use one worktree and one branch, push the worker branch only, and never push `main`"), the integration branch was pushed but `origin/main` was not fast-forwarded by this task.

- Branch pushed: `origin/integrate/windows-launcher-canonical-main-root-1300 → a6b4eda`
- Ready for main land via orchestrator/release authority.

## Final status line
1300: WINDOWS-DEV-LAUNCHER-CANONICAL-MAIN-REPAIR-INTEGRATION: READY_FOR_MAIN_LAND
