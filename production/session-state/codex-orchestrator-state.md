# Codex Orchestrator State

Updated: 2026-05-02
Owner: Codex orchestration window

Purpose: durable coordination notes for parallel implementation. This file tracks
agent windows, pending story-done work, unlocks, and known blockers. It is not the
authoritative story status tracker; `production/sprint-status.yaml` remains the
source of truth for story status.

## Current Policy

- Do not block new implementation work on GitHub Actions unless CI reports a red
  failure that needs repair.
- New workers use one Git worktree and one branch per story:
  `D:\_DEV\claude-code-game-studios-worktrees\<story-id>` on
  `work/<story-id>-<short-slug>`.
- Workers run local Developer PowerShell checks, commit explicit owned paths,
  push their story branch, and report branch name, commit hash plus CI run if
  available.
- The root checkout stays reserved for orchestrator integration merges,
  story-done, CI triage, and state tracking.
- Story-done windows are serialized because they edit shared production files.
- Keep commits scoped. If the pre-commit hook blocks due to mixed files, unstage
  and re-add explicit owned paths.
- Existing shared-tree workers already launched before the worktree switch may
  finish normally; do not migrate them mid-story.

## Orchestrator Response Protocol

After every agent return from the user, the orchestrator must automatically:

1. Classify the returned window as `clear`, `keep open for repair/commit`, or
   `relaunch with corrected prompt`.
2. Update the durable orchestration state when implementation, closure, blockers,
   or unlocks changed.
3. Identify every newly unlocked story or blocker-clear task.
4. Provide new parallel launch prompts immediately, or state `nothing safe to
   launch in parallel` with the reason.

Standing throughput rule: keep exactly one serialized story-done window active
when closure work exists, and keep two to four implementation/blocker-clear
workers active whenever READY stories do not overlap on likely files or
architectural ownership.

Do not wait for the user to ask what can run in parallel. Assume prompts the
orchestrator provides are launched unless the user explicitly says otherwise.

## Live Windows Confirmed By User

- CA-005 worker: initial readiness run returned NEEDS WORK on stale manifest
  version and missing performance budget. Story fixed at `94267fb`; worker
  completed on branch `work/ca-005-purchase-flow` at `415384a`;
  cherry-picked into `main` at `c6141bc`; story-done committed at `a770db2`.
  Window can be cleared.
- OBJECTIVE-001 worker: initial readiness run returned NEEDS WORK on stale
  manifest and too-few acceptance criteria. Story fixed at `10d738f`; worker
  completed locally on branch `work/objective-001-state-model` at `0ca676d`;
  push was blocked by credentials, so root cherry-picked it into `main` at
  `38a5489`; story-done committed at `0b847cb`. Window can be cleared.
- COMBAT-001 worker: completed locally on branch
  `work/combat-001-resolve-combat-scaffold` at `311c6f0`; worker push was
  blocked by credentials, so root cherry-picked it into `main` at `01f831e`;
  story-done committed at `9589116`. Window can be cleared.
- HAND-UI-003 worker: launch prompt issued while CA-005 story-done is active;
  per user rule, assume launched unless contradicted. Completed on branch
  `work/hand-ui-003-phase-state-machine` at `c6e5504`; cherry-picked into
  `main` at `614e68e`. Worker checks passed; root `cargo fmt -p client --
  --check` and `git diff --check HEAD~1..HEAD` passed; story-done committed at
  `d55a3d5`. Window can be cleared.
- HUD-005 worker: launch prompt issued while CA-005 story-done is active; per
  user rule, assume launched unless contradicted. Completed on branch
  `work/hud-005-phase-transitions` at `5061728`; cherry-picked into `main` at
  `9104400`. Worker checks passed; root re-check hit long client compile
  timeout, not a test failure; story-done committed at `3230dce`. Window can
  be cleared.
- BOARD-005 worker: launch prompt issued while CA-005 story-done is active; per
  user rule, assume launched unless contradicted. Initial readiness returned
  NEEDS WORK only because the story embedded manifest version was stale
  (`2026-04-29` vs current `2026-05-01`); dependencies and ADRs passed.
  Story manifest was refreshed, so this window should retry readiness and then
  implement if READY. Correct ADR file is
  `docs/architecture/adr-007-placement-buffer.md`. Completed locally on branch
  `work/board-005-placement-buffer-phase-integration` at `4946ea2`; worker push
  was blocked by credentials, so root cherry-picked it into `main` at `86ecbcb`.
  Repair committed at `9175598`; story-done committed at `1dbd2cf`.
  Window can be cleared.

## Tracker In-Progress But No Live Window Confirmed

These are marked `in-progress` in `production/sprint-status.yaml`, but the user
confirmed no corresponding agent window is currently running. Treat them as
stale/incomplete until explicitly relaunched or closed:
None currently tracked here.

## Recently Implemented, Needs Formal Story-Done

- BOARD-006: Charge Bonus Movement implemented locally on branch
  `work/board-006-charge-bonus-movement` at `874f28e`; worker push was blocked
  by external GitHub transfer approval, so root cherry-picked and amended the
  integration commit into `main` at `a04022b`. Root `cargo fmt -p server --
  --check`, `cargo test -p server --test charge_movement_test --test
  standard_movement_test`, `cargo check -p server`, and
  `git diff --check HEAD~1..HEAD` passed; story-done committed at `86612b7`.
- GSS-004: F4 SessionReady Predicate and Trigger implemented on branch
  `work/gss-004-f4-session-ready` at `9708147`; root cherry-picked it into
  `main` at `4d8cf60`. Worker checks passed full affected session/RSM/economy
  regression set plus full `cargo test -p server`; grep gates confirmed exactly
  one `On<SessionReady>` observer path. Story-done on 2026-05-02 returned
  BLOCKED: current GDD/TR require F4 to pass when
  `server_clock_now <= lobby_deadline`, but implementation uses strict `<` and
  fails exact-deadline equality. Needs one-line predicate repair plus boundary
  test before rerunning story-done.
- PRISM-001: Prism State Scaffold implemented on branch
  `work/prism-001-state-scaffold` at `6ecd421`; root cherry-picked it into
  `main` at `e093804`. Worker checks passed `cargo fmt -p server -- --check`,
  `cargo test -p server --test prism_state_scaffold_test`, `cargo check -p server`,
  and board/objective adjacent regression tests.
- HUD-004: Scoreboard Dot Observer implemented on branch
  `work/hud-004-scoreboard-dot-observer` at `fd9b4e8`; root cherry-picked it
  into `main` at `c30fc6a`. Worker checks passed
  `cargo fmt -p client -- --check`,
  `cargo test -p client --test scoreboard_dot_message_test`, HUD regression
  slice 21/21, and `cargo check -p client`. Root `cargo check -p client` and
  `git diff --check HEAD~1..HEAD` also passed after integration.
- CARD-ANIM-007: Damage Number Lifecycle implemented locally on branch
  `work/card-anim-007-damage-number-lifecycle` at `d49d274`; root cherry-picked
  it into `main` at `ca890fc`. Worker checks passed `cargo fmt -p client --
  --check`, `cargo test -p client --test card_animations_damage_number_test`,
  existing card animation regressions, and `cargo check -p client`. Root
  `cargo check -p client` and `git diff --check HEAD~1..HEAD` also passed after
  integration.
- CARD-ANIM-005: Placement Reveal Parallelism implemented locally on branch
  `work/card-anim-005-placement-reveal-parallelism` at `0c1d5fe`; root
  cherry-picked it into `main` at `5ccb988`. Worker checks passed
  `cargo fmt -p client -- --check`,
  `cargo test -p client --test card_animations_placement_reveal_test`,
  existing card animation regressions 30/30, and `cargo check -p client`.
  Root `cargo check -p client` and `git diff --check HEAD~1..HEAD` also passed
  after integration.
- PRISM-002: Deterministic Lane Rewards implemented on branch
  `work/prism-002-deterministic-lanes` at `8e9aaed`; root cherry-picked it into
  `main` at `65cb5a6`. Worker checks passed `cargo fmt -p server -- --check`,
  `cargo test -p server --test prism_deterministic_lanes_test`,
  `cargo test -p server --test prism_state_scaffold_test`, and
  `cargo check -p server`. Root `cargo check -p server` and
  `git diff --check HEAD~4..HEAD` passed after integration.
- HUD-006: Economy Auction Inline Gold implemented on branch
  `work/hud-006-economy-auction-inline-gold` at `6d6d90b`; root cherry-picked
  it into `main` at `92906d5`. Worker checks passed
  `cargo fmt -p client -- --check`,
  `cargo test -p client --test hud_economy_auction_inline_gold_test`,
  HUD regression slice 29/29, and `cargo check -p client`. Root
  `cargo check -p client` and `git diff --check HEAD~4..HEAD` passed after
  integration.
- AUC-004: Bid Validation Gate implemented locally on branch
  `work/auc-004-bid-validation-gate` at `59e086f`; root cherry-picked it into
  `main` at `5bd635e`. Worker resolved OQ9 as reachable and covered
  `LIVE_BIDDING` with `timer_remaining_ms == 0`. Root tests passed:
  `cargo test -p server --test auction_bid_validation_gate_test --test
  rsm_network_dispatch_test --test rsm_transitions_test`,
  `cargo test -p server rsm_disconnect`, `cargo check -p server`, and
  `git diff --check HEAD~2..HEAD`.
- RSM-006: Network Dispatch Wiring implemented on branch
  `work/rsm-006-network-dispatch-wiring` at `151d9e6`; root cherry-picked it
  into `main` at `894ea6b`. Worker checks passed `cargo fmt -p server --
  --check`, `cargo test -p server --test rsm_network_dispatch_test`,
  `cargo test -p server rsm_disconnect`, `cargo test -p server --test
  rsm_transitions_test`, `cargo check -p server`, and RSM grep gates. Root
  repeated the affected tests/checks after integration.
## Recently Closed

- CARD-ANIM-003: Simultaneous Track Animation implemented on branch
  `work/card-anim-003-simultaneous-track-animation` at `4f4d7c5`; cherry-picked
  into `main` at `066c1cd` after resolving a public export conflict with
  CARD-ANIM-005; story-done committed at `e46f704`.
- RSM-005: Disconnect Handling implemented locally on branch
  `work/rsm-005-disconnect-handling` at `8007ad1`; cherry-picked/rebased into
  `main` at `e4fb6a4`; repair committed at `b86b81b`; story-done committed at
  `9e9aa2f`.
- BOARD-006: Charge Bonus Movement implemented locally on branch
  `work/board-006-charge-bonus-movement` at `874f28e`; cherry-picked into
  `main` at `a04022b`; story-done committed at `86612b7`.
- ECO-005: Auction Reservation and Bid Validation implemented locally on branch
  `work/eco-005-auction-reservation-bid-validation` at `f8b69bc`; cherry-picked
  into `main` at `2108143`; story-done committed at `2f745bb`.
- BOARD-005: Placement Buffer Phase Integration implemented locally on branch
  `work/board-005-placement-buffer-phase-integration` at `4946ea2`; cherry-picked
  into `main` at `86ecbcb`; Lightyear ReliableChannel repair committed at
  `9175598`; story-done committed at `1dbd2cf`.
- HAND-UI-003: Phase State Machine implemented on branch
  `work/hand-ui-003-phase-state-machine` at `c6e5504`; cherry-picked into
  `main` at `614e68e`; story-done committed at `d55a3d5`.
- HUD-005: Phase Transitions implemented on branch
  `work/hud-005-phase-transitions` at `5061728`; cherry-picked into `main` at
  `9104400`; story-done committed at `3230dce`.
- OBJECTIVE-001: Objective State Model implemented locally on branch
  `work/objective-001-state-model` at `0ca676d`; cherry-picked into `main` at
  `38a5489`; story-done committed at `0b847cb`.
- COMBAT-001: Resolve Combat Scaffold implemented locally on branch
  `work/combat-001-resolve-combat-scaffold` at `311c6f0`; cherry-picked into
  `main` at `01f831e`; story-done committed at `9589116`.
- CA-005: Purchase Flow, Dead Slot, and CA18 Atomicity implemented on branch
  `work/ca-005-purchase-flow` at `415384a`; cherry-picked into `main` at
  `c6141bc`; story-done committed at `a770db2`.
- BOARD-004: Placement Occupancy implemented on branch
  `work/BOARD-004-placement-occupancy` at `224708d`; cherry-picked into `main`
  at `0c69612`; story-done committed at `9cfd0ad`.
- CA-004: Manual Refresh Cost implemented on branch `work/ca-004-refresh-cost`
  at `f26f738`; cherry-picked into `main` at `5cb53a8`; story-done committed
  at `dd6332e`.
- HAND-UI-002: Fan Layout Formula implemented on branch
  `work/hand-ui-002-fan-layout-formula` at `da0fe3a`; cherry-picked into
  `main` at `047aff9`; story-done committed at `b4ca7e9`.
- BOARD-003: Spawn Range Validation implemented on branch
  `work/BOARD-003-spawn-range-validation` at `bf39342`; cherry-picked into
  `main` at `9c38083`; story-done committed at `cb642a6`.
- KW-005: Shield Scope implemented on branch `work/kw-005-shield-scope` at
  `a1a824b`; cherry-picked into `main` at `0b610fd`; story-done committed at
  `f055051`.
- HUD-003: Phase Label/Round Counter implemented on branch
  `work/hud-003-phase-label-round-counter` at `52a3605`; cherry-picked into
  `main` at `ce76a88`; story-done committed at `a3bbf92`.
- CARD-ANIM-006: Objective Stagger Reveal implemented on branch
  `work/card-anim-006-objective-stagger-reveal` at `effcef2`; cherry-picked into
  `main` at `8d641b9`; story-done committed at `4e38abf`.
- HUD-002: Gold/Mana Display implemented on branch
  `work/hud-002-gold-mana-display` at `0c00a44`; cherry-picked into `main` at
  `3eaf578`; story-done committed at `4e16bf9`.
- CARD-ANIM-008: Input Gating implemented on branch
  `work/card-anim-008-input-gating` at `0d75fb0`; cherry-picked into `main` at
  `9308bf3`; story-done committed at `d0365d9`.
- KW-004: STUN State implemented on branch `work/kw-004-stun-state` at
  `7543293`; cherry-picked into `main` at `b8b1287`; story-done committed at
  `87eb37c`.
- BOARD-002: Standard Unit Movement implemented on branch
  `work/board-002-standard-unit-movement` at `4a76028`; cherry-picked into
  `main` at `0d8e41c`; story-done committed at `ffe0ca6`.
- CARD-ANIM-009: CI Boundary Enforcement implemented on branch
  `work/card-anim-009-ci-boundary-enforcement` at `55b5331`; cherry-picked into
  `main` at `75e11ea`; story-done committed at `30bff20`.
- CARD-ANIM-004: AnimQueue Resolution Drain implemented on branch
  `work/card-anim-004-animqueue-resolution-drain` at `2ecd58f`; merged into
  `main` at `b7204e5`; story-done committed at `aec3b7f`.
- HAND-UI-001: Plugin Scaffold implemented on branch
  `work/hand-ui-001-plugin-scaffold` at `9f28a2a`; cherry-picked into `main` at
  `7c603e0`; story-done committed at `342b343`.
- CA-006: Card Acquisition External Bypass implemented on branch
  `work/ca-006-external-bypass` at `6af1137`; merged into `main`; story-done
  committed at `1ddd7b6`.
- CA-003: Card Acquisition Draw Pipeline implemented on branch
  `work/ca-003-draw-pipeline` at `c6200f0`; merged into `main` at `98cb52a`;
  story-done committed at `74f7aff`.
- BOARD-001: Board Grid Initialization implemented on branch
  `work/board-001-grid-initialization` at `7d38a34`; merged into `main` at
  `6e5d80b`; story-done committed at `e58533d`.
- HUD-001: implemented at `b04748b`; Bevy 0.18 BorderColor fix at `cbce522`;
  test harness fix at `95b58ae`; story-done closed after
  `hud_plugin_scaffold_test` and `cargo check -p client` passed locally.
- S3-08: Economy Interest Snapshot & Resolution End implemented on branch
  `work/s3-08-economy-interest-snapshot` at `db61102`; merged into `main` at
  `4961356`; story-done committed at `4f838b6`.
- CA-001: implemented at `05dc190`; story-done committed and pushed at
  `c4c3fa9`.
- AUC-003: implemented at `44afdb5`; story-done committed and pushed at
  `579db68`.
- CS-002: implemented at `20b24fa`; story-done committed and pushed at
  `bd3487a`.
- KW-002: implemented at `7fe9b5d`; tracking claim pushed at `699c227`;
  story-done committed and pushed at `765ecfc`.
- CARD-ANIM-001: implemented at `23fad70`; story-done committed and pushed at
  `ab7d56f`.
- S3-06: E2E WebSocket Roundtrip implemented at `a32a3df`; HUD Bevy 0.18 WASM
  blocker fixed at `cbce522`; story-done committed and pushed at `57159e9`.
  Note: sprint-status marks S3-06 done but still has owner
  `codex-s3-06-websocket`; clean this in a later tracker hygiene pass if needed.
- S3-04: RSM Timers + Input Reader implemented at `eff5cf9`; blocker fixed at
  `ec6f433`/`61e45ad`; story-done committed at `1045dbc`.
- S3-05: RSM Win Condition and Game Over implemented at `5bf6bde`; story-done
  committed at `4d745a8`.
- CA-002: Card Acquisition Draft Initial implemented at `2c6c65b`; story-done
  committed at `79d5024`. `production/sprint-status.yaml` has no CA-002 entry,
  so the closeout updated only the story file and session state.
- KW-003: First Strike and Haste implemented at `874d86b`; story-done was
  absorbed into asset commit `bee8b47`, with acceptance checkbox/test-note
  cleanup finalized in a follow-up closure commit. `production/sprint-status.yaml`
  has no KW-003 entry, so the closeout updated only the story file and session
  state.
- CARD-ANIM-002: Tween Cancel/Replace Lifecycle implemented at `1354d5a` and
  merged into `main` at `e9103d9`; story-done closed after lifecycle tests,
  paired scaffold+lifecycle tests, and `cargo check -p client` passed locally.

## Story-Done Queue

1. GSS-004 repair + story-done
2. PRISM-001
3. HUD-004
4. CARD-ANIM-007
5. PRISM-002
6. HUD-006
7. CARD-ANIM-005
8. AUC-004
9. RSM-006

Run only one story-done at a time.

## Launch Blocks / Wait Conditions

- GSS-004: implemented and integrated but story-done is BLOCKED. Repair F4
  deadline equality (`now <= lobby_deadline`) and add exact-boundary test before
  rerunning closure. Do not launch further SessionReady/RSM session stories until
  closure confirms the single-observer gate.
- PRISM-001: implemented and integrated; pending story-done. PRISM-002 is also
  implemented and integrated, but PRISM-003+ should wait until PRISM-001/002 close.
- HUD-004: implemented and integrated; pending story-done before launching HUD
  stories that depend on scoreboard objective dots.
- CARD-ANIM-007: implemented and integrated; pending story-done.
- CARD-ANIM-005: implemented and integrated; pending story-done.
- PRISM-002: implemented and integrated; pending story-done.
- HUD-006: implemented and integrated; pending story-done.
- AUC-004: implemented and integrated; pending story-done.
- RSM-006: implemented and integrated; pending story-done.
- AUC-005+ follow normal sequencing after AUC-004 story-done.
- GSS-005+ and other RSM/session/disconnect work should still be staged
  carefully because GSS-004 awaits story-done confirmation of the SessionReady
  single-observer gate.
- Prism gates are resolved; PRISM-003+ follow normal sequencing after PRISM-001
  and PRISM-002 story-done.

## Next Parallel Launch Candidates

Batch launched:
- GSS-004: integrated but story-done BLOCKED; repair should stay in the same
  window.
- PRISM-001: integrated, pending story-done.
- HUD-004: integrated, pending story-done.
- CARD-ANIM-007: integrated, pending story-done.
- PRISM-002: integrated, pending story-done.
- HUD-006: integrated, pending story-done.
- CARD-ANIM-005: integrated, pending story-done.
- AUC-004: integrated, pending story-done.
- RSM-006: integrated, pending story-done.

Current active windows by user default-launch rule:
- GSS-004 repair/story-done remains open after the F4 exact-deadline blocker.
- CS-003, BOARD-007, and HUD-007 are considered launched from the latest prompts.
AUC-004 and RSM-006 have returned and are integrated. Do not launch PRISM-003
until PRISM-001 and PRISM-002 close.

## Resolved Design Gates

- OQ-KS9: resolved in `design/gdd/combat-resolution.md` via `f8ceafd`.
- OQ-HUD-05: resolved in HUD story 004 via `64b0cfd`; HUD story 004 still has
  other blockers and should not be implemented yet.
- KW-SC-1: `On<UnitDied>` observer param compile probe passed with
  `cargo check -p server`; no permanent files were needed.

## Current Dirty-Tree Notes

As of the asset sorting pass, generated art assets were moved into `assets/art/`
and committed. `.codex-tmp/` is ignored as a local scratch workspace. Use
worktree mode for all new code workers.
