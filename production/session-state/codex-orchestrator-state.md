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
- BOARD-007: Trap Trigger Mechanics implemented on branch
  `work/board-007-trap-trigger-mechanics` at `2daaa76`; root cherry-picked and
  amended it into `main` at `fd13f2a`. Worker checks passed
  `cargo fmt -p server -- --check`,
  `cargo test -p server --test trap_trigger_test`, board movement/placement
  regressions 34/34, and `cargo check -p server`. Root repeated fmt,
  `trap_trigger_test` 4/4, affected board regression slice 30/30, `cargo check
  -p server`, and `git diff --check HEAD~1..HEAD`. No Prism collection or RSM
  dispatch code touched.
- PRISM-003: Lane 3 RNG Draw Pipeline implemented on branch
  `work/prism-003-lane3-rng` at `4d5acf1`; root cherry-picked and amended it
  into `main` at `611baee`. Root repeated checks because worker output was
  truncated: `cargo fmt -p server -- --check`,
  `cargo test -p server --test prism_lane3_rng_test --test
  prism_state_scaffold_test --test prism_deterministic_lanes_test` passed 16/16,
  `cargo check -p server`, and `git diff --check HEAD~1..HEAD`. Network
  staging remains PRISM-004.
- HUD-009: Same-Tick Gold Tie-Break implemented on branch
  `work/hud-009-same-tick-gold-tie-break` at `ed3d7fd`; root cherry-picked and
  amended it into `main` at `fdadbe6` after worker push was blocked by approval
  policy. Root checks passed `cargo fmt -p client -- --check`,
  `cargo test -p client --test same_tick_tie_break_test --test
  hud_gold_mana_display_test --test hud_economy_auction_inline_gold_test --test
  hud_phase_transitions_test` 18/18, `cargo check -p client`, and `git diff
  --check HEAD~1..HEAD`.
- HUD-010: Numeric Tween Animation implemented on branch
  `work/hud-010-numeric-tween-animation` at `92f8677`; root cherry-picked and
  amended it into `main` at `609be61`. Root checks passed `cargo fmt -p client
  -- --check`, `cargo test -p client --test hud_numeric_tween_animation_test
  --test hud_gold_mana_display_test --test
  hud_economy_auction_inline_gold_test --test hud_game_over_freeze_test --test
  hud_phase_transitions_test` 21/21, extra HUD regression slice
  `same_tick_tie_break_test`, `hud_phase_label_round_counter_test`,
  `hud_plugin_scaffold_test`, and `scoreboard_dot_message_test` 17/17,
  `cargo check -p client`, and `git diff --check HEAD~1..HEAD`. Uses the
  current bevy_tweening 0.15 TweenAnim / PlaybackState / TweenState pattern.
- HAND-UI-004: DRAFT_INITIAL Grid Flow implemented on branch
  `work/hand-ui-004-draft-initial-grid` at `b2ad5db`; root cherry-picked it
  into `main` at `561d2fd`. Root checks passed `cargo fmt -p client --
  --check`, `cargo test -p client --test hand_ui_draft_initial_grid_test
  --test hand_ui_plugin_scaffold_test --test hand_ui_fan_layout_formula_test
  --test hand_ui_phase_state_machine_test` 16/16, `cargo check -p client`, and
  `git diff --check HEAD~1..HEAD`.
- GSS-005: Lobby Disconnect Dual-Signal Cancel implemented on branch
  `work/gss-005-lobby-disconnect-dual-signal` at `77640bf`; root cherry-picked
  and amended it into `main` at `15fe812`. Root checks passed
  `cargo fmt -p server -- --check`,
  `cargo test -p server --test dual_signal_disconnect_test --test
  lobby_timeout_test --test room_create_join_test --test class_reveal_test
  --test session_ready_test` 25/25,
  `cargo test -p server --test class_lifecycle_test --test
  session_ready_observer_test` 11/11, `cargo check -p server`, grep gate for no
  EventReader/EventWriter/Events in touched session/network paths, and `git diff
  --check HEAD~1..HEAD`.
- GSS-006: Game-Over Teardown implemented on branch
  `work/gss-006-game-over-teardown` at `37a237c`; root cherry-picked and
  amended it into `main` at `d5f835e`. Root checks passed `cargo fmt -p server
  -- --check`, `cargo test -p server --test game_over_teardown_test --test
  session_ready_test --test dual_signal_disconnect_test --test
  rsm_win_condition_test` 18/18, `cargo check -p server`, grep gate for no
  EventReader/EventWriter/Events< in touched session/RSM/network paths, and
  `git diff --check HEAD~1..HEAD`. Scope is teardown only; reconnect snapshot
  schema/building remains GSS-007/HUD-008 unblock work.
## Recently Closed

- CARD-ANIM-005: Placement Reveal Parallelism implemented locally on branch
  `work/card-anim-005-placement-reveal-parallelism` at `0c1d5fe`; root
  cherry-picked it into `main` at `5ccb988`; story-done committed at
  `265f34b`. Verification passed `cargo test -p client --test
  card_animations_placement_reveal_test` 9/9 and `cargo check -p client`.
  Completion notes document missing visual screenshot/sign-off evidence for
  CA-4b as advisory only.
- AUC-004: Bid Validation Gate implemented locally on branch
  `work/auc-004-bid-validation-gate` at `59e086f`; root cherry-picked it into
  `main` at `5bd635e`; story-done committed at `75b8998`. Worker resolved OQ9
  as reachable and covered `LIVE_BIDDING` with `timer_remaining_ms == 0`.
  Verification passed `cargo test -p server --test
  auction_bid_validation_gate_test` 9/9, adjacent regression tests 14 passed
  with 1 ignored future settlement test, `cargo fmt -p server -- --check`, and
  `cargo check -p server`. Completion notes document stale
  `C2SAuctionBid`/`C2SPlaceBid` wording as advisory.
- RSM-006: Network Dispatch Wiring implemented on branch
  `work/rsm-006-network-dispatch-wiring` at `151d9e6`; root cherry-picked it
  into `main` at `894ea6b`; story-done committed at `2f07c94`. Verification
  passed `cargo test -p server --test rsm_network_dispatch_test` 3/3 and
  `cargo check --workspace`. Completion notes document stale manifest,
  ResolutionTimeout wording, and `timer_duration_ms` doc drift as advisory.
- CS-003: Xelor Reserve Formulas implemented on branch
  `work/cs-003-xelor-reserve-formulas` at `e5aabd6`; root cherry-picked it into
  `main` at `3440b21`; story-done committed at `b940f70`. Verification passed
  `cargo test -p server --test xelor_reserve_test` 6/6, `cargo check -p
  server`, and `cargo fmt -p server -- --check`. Completion notes document
  stale manifest, stale file-path wording, and missing separate evidence doc as
  advisory only.
- HUD-007: Game Over Freeze implemented on branch
  `work/hud-007-game-over-freeze` at `926d35d`; root cherry-picked it into
  `main` at `862704f`; story-done committed at `cae0e45`. Verification passed
  `cargo test -p client --test hud_game_over_freeze_test` 2/2, adjacent HUD
  pack 14/14, and `cargo check -p client`. Completion notes document that the
  test covers post-GAME_OVER update rejection but not the exact story example
  values `999/888`.
- GSS-004: F4 SessionReady Predicate and Trigger implemented on branch
  `work/gss-004-f4-session-ready` at `9708147`; cherry-picked into `main` at
  `4d8cf60`; repair committed at `3c64b84`; story-done committed at `36ed875`.
- PRISM-001: Prism State Scaffold implemented on branch
  `work/prism-001-state-scaffold` at `6ecd421`; root cherry-picked it into
  `main` at `e093804`; story-done committed at `671caa2`.
- PRISM-002: Deterministic Lane Rewards implemented on branch
  `work/prism-002-deterministic-lanes` at `8e9aaed`; root cherry-picked it into
  `main` at `65cb5a6`; story-done committed at `2d1a4bf`.
- HUD-004: Scoreboard Dot Observer implemented on branch
  `work/hud-004-scoreboard-dot-observer` at `fd9b4e8`; root cherry-picked it
  into `main` at `c30fc6a`; story-done committed at `3c85ae1`.
- CARD-ANIM-007: Damage Number Lifecycle implemented locally on branch
  `work/card-anim-007-damage-number-lifecycle` at `d49d274`; root cherry-picked
  it into `main` at `ca890fc`; repaired against the current GDD F3 jitter table
  at `2b5ea8e`; story-done committed at `35ee469`.
- HUD-006: Economy Auction Inline Gold implemented on branch
  `work/hud-006-economy-auction-inline-gold` at `6d6d90b`; root cherry-picked
  it into `main` at `92906d5`; story-done committed at `cc205e3`.
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

1. BOARD-007
2. PRISM-003
3. HUD-009
4. HUD-010
5. HAND-UI-004
6. GSS-005
7. GSS-006

Run only one story-done at a time.

## Launch Blocks / Wait Conditions

- PRISM-003 is unblocked by PRISM-001/002 closure. Remaining Prism story
  manifests were refreshed to 2026-05-01 in `7834e88`; PRISM-003 is now
  implemented/integrated and pending story-done. PRISM-004 can run after
  PRISM-003 story-done because it depends on the Lane 3 call site.
- BOARD-007: implemented and integrated; pending story-done.
- PRISM-003: implemented and integrated; pending story-done.
- HUD-009: implemented and integrated; pending story-done.
- HUD-010: implemented and integrated; pending story-done.
- HAND-UI-004: implemented and integrated; pending story-done.
- GSS-005: implemented and integrated; pending story-done.
- GSS-006: implemented and integrated; pending story-done. Do not launch
  GSS-007 until GSS-005 and GSS-006 closure are clean, because GSS-007 is the
  reconnect snapshot schema/builder unblock for HUD-008 and touches the same
  session/protocol ownership.
- AUC-005+ follow normal sequencing after AUC-004 story-done.
- HUD-008 returned READY but blocked on missing full `S2CGameSnapshot` schema.
  Correct unblock path is GSS-005 -> GSS-006 -> GSS-007. GSS-005 and GSS-006
  are implemented/integrated and pending story-done; launch GSS-007 only after
  those closures settle.
- Other RSM/session/disconnect work should be staged carefully because GSS-005
  and GSS-006 are implemented and pending story-done.
- Prism gates are resolved; PRISM-003+ follow normal sequencing after PRISM-001
  and PRISM-002 story-done.

## Next Parallel Launch Candidates

Batch launched:
- BOARD-007: integrated, pending story-done.
- PRISM-003: integrated, pending story-done.
- HUD-009: integrated, pending story-done.
- HUD-010: integrated, pending story-done.
- HAND-UI-004: integrated, pending story-done.
- GSS-005: integrated, pending story-done.
- GSS-006: integrated, pending story-done.

Current active windows by user default-launch rule:
- PRISM-001 story-done returned and was committed at `671caa2`; window can be
  cleared. PRISM-002 story-done returned and was committed at `2d1a4bf`;
  window can be cleared. HUD-004 story-done returned and was committed at
  `3c85ae1`; window can be cleared. CARD-ANIM-007 is now the next serialized
  story-done candidate.
  Do not launch another story-done until it returns.
- BOARD-007 returned, integrated at `fd13f2a`, and now only needs serialized
  story-done.
- HUD-008 returned with no code changes; window can be cleared. It remains
  blocked until GSS-007 expands/builds `S2CGameSnapshot`.
- PRISM-003 returned, integrated at `611baee`, and now only needs serialized
  story-done.
- CARD-ANIM-007 story-done returned and was committed at `35ee469`; window can
  be cleared. HUD-006 story-done returned and was committed at `cc205e3`;
  window can be cleared. CARD-ANIM-005 story-done returned and was committed
  at `265f34b`; window can be cleared. AUC-004 story-done returned and was
  committed at `75b8998`; window can be cleared. RSM-006 story-done returned
  and was committed at `2f07c94`; window can be cleared. CS-003 story-done
  returned and was committed at `b940f70`; window can be cleared. HUD-007
  story-done returned and was committed at `cae0e45`; window can be cleared.
  BOARD-007 is now the next serialized story-done candidate.
- HAND-UI-004 returned, integrated at `561d2fd`, and now only needs serialized
  story-done.
- GSS-005 returned, integrated at `15fe812`, and now only needs serialized
  story-done.
- GSS-006 returned, integrated at `d5f835e`, and now only needs serialized
  story-done after GSS-005.
- HUD-010 returned, integrated at `609be61`, and now only needs serialized
  story-done after HUD-009.
- OBJ-002 returned NEEDS WORK only on stale manifest. Manifest refreshed to
  `2026-05-01` in `b8b9f26`; relaunch readiness/implementation and then treat
  it as active unless the user says it was not launched.
AUC-004, RSM-006, GSS-004, CS-003, and HUD-007 have returned and are
integrated/closed as noted above.

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
