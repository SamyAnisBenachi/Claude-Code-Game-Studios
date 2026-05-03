# Codex Orchestrator State

Updated: 2026-05-03
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

Parallelism policy: maximize safe throughput even across future sprints. Do not
limit launch candidates to the current sprint if future Ready stories are
unblocked and have disjoint likely file ownership. Target 4 implementation
workers plus 1 docs/QA/tracker worker plus 1 serialized story-done window. Keep
only one story-done active because story-done edits shared production files.

Prompt formatting policy: every launch prompt shown to the user must start with
three red triangles and a number, for example
`🔺🔺🔺 PROMPT 1 -- HAND-UI-007 Placement Instant Staging`. Provide at least
three numbered prompts in a batch when three safe parallel tasks exist; if fewer
than three are safe, state why. Keep the existing status color convention:
`🟢` action/result, `🔵` verification, `🟡` attention/blocker, `🟣` queue/next.
Each prompt must include branch, worktree, scope limits, story-done prohibition,
shared tracker prohibition, and detailed commit-body requirements.

Window instruction policy: use uppercase `CLEAR` or `REPONDRE`. `CLEAR` means
the user should close that window; do not add redundant wording like "do not
respond" after `CLEAR`. `REPONDRE` means the next prompt belongs in that same
existing window. If a prompt follows a `CLEAR` instruction, it is for a new
agent/window unless a specific `REPONDRE` line says otherwise.

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

- HAND-UI-008: Placement Unstaging implemented on branch
  `work/hand-ui-008-placement-unstaging` at worker commit `743d660`; root
  integration landed on main at `552f80f`. Verification passed `cargo fmt -p
  client -- --check`, `cargo test -p client --test
  hand_ui_placement_unstaging_test` 4/4, placement submit/instant/drag
  regressions 13/13, `cargo check -p client`, and diff checks. No
  `/story-done` has run yet.
- HAND-UI-009: Placement Timer implemented on branch
  `work/hand-ui-009-placement-timer` at worker commit `7a8c173`; root
  integration landed on main at `a72bb0f`. Verification passed `cargo test -p
  client --test hand_ui_placement_timer_test` 4/4, surrounding Hand UI
  placement regressions 17/17, `cargo check -p client`, and diff checks. No
  `/story-done` has run yet.
- COMBAT-003: SS1 Placement Appearance trace repair landed on main at
  `e5c28d5`; implementation landed on main at `7fdf4fd`. Verification passed
  `cargo test -p server --test substep1_placement_test` 4/4, `cargo test -p
  server --test resolve_combat_scaffold_test` 4/4, `cargo test -p server
  --test modifier_stack_test` 7/7, `cargo check -p server`, and diff checks.
  No `/story-done` has run yet.
## Recently Closed

- AUC-006: Resolution & Settlement readiness repair landed on main at
  `6f9b54a`; implementation landed on main at `5461de6`; story-done closure
  committed at `61e69b4`. Verification passed `cargo test -p server --test
  auction_resolution_settlement_test` 3/3, `cargo test -p server --test
  auction_resolution_settlement_integration_test` 1/1, `cargo check -p
  server`, and diff checks. `production/sprint-status.yaml` had no matching
  AUC-006 row.
- BOARD-009: Prism Collection readiness docs landed on main at `7dab817`;
  implementation landed on main at `105e6b0`; story-done closure committed at
  `394d6c3`. Verification passed `cargo test -p server --test
  prism_collection_test` 6/6, related movement/trap regressions 14/14, and
  `cargo check -p server`. `production/sprint-status.yaml` had no matching
  BOARD-009 row.
- COMBAT-002: Combat Modifier Stack implemented on branch
  `work/combat-002-combat-modifier-stack` at worker commit `76ed813`; root
  integration landed on main and story-done closure was finalized at
  `fcf6615`. Verification passed `cargo test -p server --test
  modifier_stack_test` 7/7, `cargo test -p server --test
  game_config_defaults_test` 8/8, `cargo test -p server --test
  resolve_combat_scaffold_test` 3/3, `cargo check -p server`, and formatting
  checks. `production/sprint-status.yaml` had no matching COMBAT-002 row.
- OBJECTIVE-004: Damage Interface implemented on branch
  `work/objective-004-damage-interface` at worker commit `33e0b9c`; root
  integration landed on main at `033c212`; story-done closure committed at
  `b9c6114`. Verification passed `cargo test -p server --test
  damage_interface_test` 7/7, `cargo check -p server`, and diff checks.
  `production/sprint-status.yaml` had no matching OBJECTIVE-004 row.
- HAND-UI-007: Placement Instant Staging implemented on branch
  `work/hand-ui-007-placement-instant-staging` at worker commit `7c3e76b`;
  root integration landed on main at `d3a16d1`; story-done closure committed
  at `6fe4313`. Verification passed `cargo test -p client --test
  hand_ui_placement_instant_staging_test`, `cargo check -p client --features
  ui_picking`, and `git diff --check`. `production/sprint-status.yaml` had no
  matching HAND-UI-007 row.
- BOARD-008: Objective Cell Detection readiness docs committed on worker branch
  `work/board-008-objective-cell-detection` at `7e8cf00`; implementation
  landed on main at `e4f76da` and `bb260c7`; initial `/story-done` was blocked
  by missing production SS6 wiring; repair landed at `b27db7d`; story-done
  closure committed at `57232e6`. Verification passed `cargo test -p server
  --test objective_detection_test` 5/5, `cargo test -p server --test
  resolve_combat_scaffold_test
  resolve_combat_ss6_emits_unit_at_objective_messages` 1/1, and `cargo check
  -p server`. `production/sprint-status.yaml` had no matching BOARD-008 row.
- ECO-006 / S4-14: Economy Network Dispatch readiness docs committed on worker
  branch `work/eco-006-network-dispatch-wiring` at `6645baa`; implementation
  committed at `f63c397`; root integration landed on main at `648790b` and
  `83317cb`; story-done closure committed at `c5739fa`. Verification passed
  `cargo test -p server --test economy_network_dispatch_test` 4/4, `cargo
  check --workspace`, `git diff --check`, and the `UnreliableChannel` grep gate
  in `server/src/network/economy_dispatch.rs`. Completion notes document the
  protocol-boundary `mana_cap` clamp from internal `u32` to wire `u8`.
  `production/sprint-status.yaml` had no matching ECO-006 / S4-14 row.
- CDP-005 / S3-10: Manual Refresh + Cost Escalation readiness docs landed on
  main at `037788b`; root implementation integration landed on main at
  `a3719fc`; story-done closure committed at `28c9f79`. Verification passed
  `cargo test -p server --test pool_manual_refresh_test` 7/7, `cargo test -p
  server --test card_acquisition_refresh_cost_test --test pool_session_ready_test
  --test pool_manual_refresh_test` 18/18 during integration, and `git diff
  --check` during closure. `production/sprint-status.yaml` moved S3-10 to
  `done` with `completed: "2026-05-03"`, making Sprint 3 19/19 complete.
- HAND-UI-006: Placement Drag Highlights implemented on branch
  `work/hand-ui-006-placement-drag-highlights` at worker commit `4381693`;
  root integration landed on main at `4491ecd`; story-done closure committed
  at `c772af1`. Verification passed `cargo fmt -p client -- --check`, `cargo
  test -p client --test hand_ui_placement_drag_highlights_test` 5/5, Hand UI
  regressions (`hand_ui_placement_submit_core_test`,
  `hand_ui_draft_initial_grid_test`, `hand_ui_phase_state_machine_test`)
  13/13, `cargo check -p client`, and diff checks. Completion notes document
  advisory visual overlay and outline pulse evidence. `production/sprint-
  status.yaml` had no matching row.
- CDP-004: Shop Refresh Subscriber SessionReady implemented on branch
  `work/cdp-004-shop-refresh-session-ready`; readiness docs landed on main at
  `daad9bf`, root implementation integration landed at `a8f2d75`, and
  story-done closure committed at `ea00d32`. Verification passed `cargo fmt -p
  server -- --check`, `cargo test -p server --test pool_session_ready_test`
  5/5, `cargo test -p server --lib core::pool::` 39/39, focused Card
  Acquisition regressions, focused Session/RSM/GameOver regressions, `cargo
  check -p server`, and diff checks. Completion moved `S3-07` to done in
  `production/sprint-status.yaml` with `completed: "2026-05-03"`.
- OBJECTIVE-003: Identity Unicast Delivery implemented on branch
  `work/objective-003-identity-unicast-delivery`; readiness docs landed on
  main at `b3fbe3e`, root implementation integration landed at `aa84947`, and
  story-done closure committed at `4327b7b`. Verification passed `cargo fmt -p
  server -- --check`, `cargo test -p server --test
  objective_identity_unicast_test --test reconnect_snapshot_test` 10/10,
  `cargo test -p server --test objective_state_test` 4/4, objective/reconnect
  regressions, `cargo check -p server`, and diff checks. Completion notes
  document advisory GDD drift where older prose still mentions
  `ObjectiveIdentity` owner replication, while current TR/ADR require reliable
  unicast and never-replication. `production/sprint-status.yaml` had no
  matching row.
- PRISM-005: Respawn Cycle implemented on branch
  `work/prism-005-respawn-cycle` at worker commit `1c4ccbe`; root
  integration landed on main at `edb0a43`; story-done closure committed at
  `ef6b4ad`. Verification passed `cargo fmt -p server -- --check`, `cargo fmt
  --all -- --check`, `cargo test -p server --test prism_respawn_cycle_test`
  5/5, Prism regressions (`prism_state_scaffold_test`,
  `prism_deterministic_lanes_test`, `prism_lane3_rng_test`,
  `prism_hand_full_network_test`) 23/23, `cargo check -p server`, `cargo check
  --workspace`, and diff checks. Completion notes document advisory
  `TR-PRI-006` wording drift around `collected_this_round_grace` versus the
  current structural timing guarantee. `production/sprint-status.yaml` had no
  matching row.
- AUC-005: Accepted Bid Reservation implemented on branch
  `work/auc-005-accepted-bid-reservation`. Readiness metadata landed on main at
  `a55b6a0`; implementation landed on main at `ecdbf4a`; story-done closure
  committed at `2b61243`. Verification passed `cargo fmt -p server --
  --check`, `cargo test -p server --test accepted_bid_reservation_test` 5/5,
  `cargo test -p server --test auction_state_scaffold_test` 7/7, adjacent
  auction/economy regressions (`auction_phase_entry_test`,
  `auction_abort_handler_test`, `auction_bid_validation_gate_test`,
  `auction_reservation_test`) 23 passed with 1 existing ignored settlement
  test, `cargo check -p server`, and diff checks. Completion notes document
  advisory extracted-seam evidence instead of an `App::new()` harness and
  advisory broadcast ownership wording drift. `production/sprint-status.yaml`
  had no matching row.
- HUD-008: Reconnect Snapshot HUD Rebuild implemented on branch
  `work/hud-008-reconnect-snapshot-rebuild` at `778828d`; root integration
  landed at `d8971f4`; story-done closure committed at `07f477f`.
  Verification passed `cargo test -p client --test
  reconnect_snapshot_rebuild_test`, `cargo check -p client`, `cargo fmt -p
  client -- --check`, and `git diff --check`. Completion notes document
  advisory TR-HUD-009 registry narrowness versus broader HUD rebuild behavior
  in the current HUD GDD, plus missing screenshot/manual evidence for HUD-14.
  `production/sprint-status.yaml` had no matching row.
- GSS-007: Reconnect Snapshot implemented on branch
  `work/gss-007-reconnect-snapshot-schema-builder` at `8d3a91b`; root
  integration landed at `8e7c5b5`; repair committed at `32643e9`; story-done
  closure committed at `7378e28`. The repair filled the ADR-011 reconnect
  handshake flow, timeout/rejection handling, deferred queue flush, opponent
  reconnect broadcast, Sang Meprise restore, reconnect guard regressions, and
  missing `reconnect_snapshot_test` evidence. Verification passed `cargo fmt
  --all -- --check`, `cargo test -p server --test snapshot_secret_strip_test`,
  `cargo test -p server --test reconnect_snapshot_test`, `cargo test -p server
  --test game_over_teardown_test`, `cargo test -p server --test
  prism_hand_full_network_test`, `cargo check -p server`, `cargo check
  --workspace`, and diff checks. HUD-008 files were not touched by the repair
  or closure.
- HAND-UI-005: Placement Submit Core implemented on branch
  `work/hand-ui-005-placement-submit-core` at `c547056c`; root integration
  landed at `1c798f0`; story-done closure committed at `c8222d2`. Verification
  passed `cargo test -p client --test hand_ui_placement_submit_core_test` 5/5,
  `cargo check -p client`, and `cargo fmt -p client -- --check`. Completion
  notes document the advisory that coverage verifies the Hand UI local
  message/outbox seam and optional `MessageSender<C2SSubmitPlacement>` path,
  not a full live Lightyear transport session.
- PRISM-004: Hand-Full Prism Network Staging implemented on branch
  `work/prism-004-hand-full-network` at `e823d66`; root integration landed at
  `8c77982`; story-done closure committed at `e7776b0`. Verification passed
  `cargo test -p server --test prism_hand_full_network_test` 7/7, `cargo check
  -p server`, `cargo fmt --all -- --check`, and `git diff --check`.
  Completion notes document the advisory TR-PRI-004 registry drift where Lane 3
  hand-full still says to emit `S2CPrismRewardDropped`, while current GDD/story
  behavior says it must not emit that message.
- GSS-005: Lobby Disconnect Dual-Signal Cancel implemented on branch
  `work/gss-005-lobby-disconnect-dual-signal` at `77640bf`; root integration
  landed at `15fe812`, story-done initially blocked on room-session timeout
  semantics, then repair/closure committed at `19071b5`. The fix changed
  `lobby_timeout_check` so a room session cancels when `now > lobby_deadline.0`
  and F4 is false, including the filled/class-locked-after-deadline regression.
  Verification passed `cargo fmt -p server -- --check`, `cargo test -p server
  --test dual_signal_disconnect_test --test lobby_timeout_test` 8/8, and
  `cargo check -p server`. `production/sprint-status.yaml` had no matching row.
- GSS-006: Game-Over Teardown implemented on branch
  `work/gss-006-game-over-teardown` at `37a237c`; root integration landed at
  `d5f835e`; story-done closure committed at `a49e422`. Verification passed
  `cargo test -p server --test game_over_teardown_test` 4/4 and `cargo check
  -p server`. Completion notes record advisory wording drift around
  `S2CGameOver.loser: Option<PlayerId>` for draw support and live Lightyear
  delivery being code-verified while the integration test covers the outbox
  path. `production/sprint-status.yaml` had no matching row.
- OBJ-002: Fake Assignment and Config Guards implemented on branch
  `work/objective-002-fake-assignment-config-guards` at `24bf21b`; root
  integration landed at `536ccc8`; story-done closure committed at `88f3fe2`.
  Verification passed `cargo test -p server --test fake_assignment_test` 5/5
  and `git diff --check` for the closure files. Completion notes document the
  advisory split between pre-Lobby `validate_game_config` invalid-config exits
  and the Objective System DRAFT_INITIAL defensive guard. `production/sprint-
  status.yaml` had no matching row.
- HAND-UI-004: DRAFT_INITIAL Grid Flow implemented on branch
  `work/hand-ui-004-draft-initial-grid` at `b2ad5db`; root cherry-picked it
  into `main` at `561d2fd`; story-done committed at `f610054`. Verification
  passed `cargo test -p client --test hand_ui_draft_initial_grid_test` 5/5 and
  `cargo check -p client`. Completion notes document local Bevy message/outbox
  usage instead of live Lightyear wiring, missing CardAtlas lookup, and
  TR-HU-005 timer/budget scope drift as advisory.
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
- BOARD-007: Trap Trigger Mechanics implemented on branch
  `work/board-007-trap-trigger-mechanics` at `2daaa76`; root cherry-picked and
  amended it into `main` at `fd13f2a`; story-done committed at `dc8b80a`.
  Verification passed `cargo test -p server --test trap_trigger_test` 4/4 and
  `cargo fmt -p server -- --check`. Completion notes document the BL-31
  `World::run_system_once` harness choice as advisory because it directly
  covers the lane-change commit system.
- PRISM-003: Lane 3 RNG Draw Pipeline implemented on branch
  `work/prism-003-lane3-rng` at `4d5acf1`; root cherry-picked and amended it
  into `main` at `611baee`; story-done committed at `b4d9e04`. Verification
  passed `cargo test -p server --test prism_lane3_rng_test` 4/4 and `cargo
  check -p server`. Completion notes document stale `TR-PRI-004`
  `S2CPrismRewardDropped` wording and central `ServerRng.audit_log()` stub
  behavior as advisory.
- HUD-009: Same-Tick Gold Tie-Break implemented on branch
  `work/hud-009-same-tick-gold-tie-break` at `ed3d7fd`; root cherry-picked and
  amended it into `main` at `fdadbe6`; story-done committed at `7f3ecfa`.
  Verification passed `cargo test -p client --test same_tick_tie_break_test`
  3/3, `cargo check -p client`, and `cargo fmt -p client -- --check`.
  Completion notes document direct Bevy HUD message injection after the
  Lightyear drain seam as advisory.
- HUD-010: Numeric Tween Animation implemented on branch
  `work/hud-010-numeric-tween-animation` at `92f8677`; root cherry-picked and
  amended it into `main` at `609be61`; story-done committed at `d23ce6f`.
  Verification passed `cargo test -p client --test
  hud_numeric_tween_animation_test`, the focused HUD regression slice, `cargo
  check -p client`, `cargo fmt -p client -- --check`, and `git diff --check
  HEAD~1..HEAD`. Completion notes document missing screenshot/sign-off evidence
  for layout legibility as advisory.
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

1. HAND-UI-008:
   `production/epics/hand-ui/story-008-placement-unstaging.md`
   after integration commit `552f80f`.
2. HAND-UI-009:
   `production/epics/hand-ui/story-009-placement-timer.md`
   after integration commit `a72bb0f`.
3. COMBAT-003:
   `production/epics/combat-resolution/story-003-substep1-placement-appearance.md`
   after integration commit `7fdf4fd`.

Run only one story-done at a time.

## Launch Blocks / Wait Conditions

- Sprint 4 QA plan missing: resolved at `8578890` with
  `production/qa/qa-plan-sprint-4-2026-05-03.md`. S4-14 Economy Network
  Dispatch can now run story-readiness, but `production/sprint-status.yaml`
  still tracks Sprint 3 until the Sprint 4 handoff/reconcile is committed.
- PRISM-003 is unblocked by PRISM-001/002 closure. Remaining Prism story
  manifests were refreshed to 2026-05-01 in `7834e88`; PRISM-003 is now
  closed. PRISM-004 is closed at `e7776b0`; PRISM-005 is closed at `ef6b4ad`.
- GSS-005: closed at `19071b5`.
- GSS-006: closed at `a49e422`.
- GSS-007: closed at `7378e28`.
- AUC-005 is closed at `2b61243`; AUC-006 is closed at `61e69b4`;
  AUC-007+ follow normal sequencing.
- HUD-008: closed at `07f477f`. The original snapshot schema blocker was
  removed by integrated and closed GSS-007 code.
- Other RSM/session/disconnect work should avoid reopening the GSS-007
  reconnect snapshot contract unless it is explicitly scoped.
- Prism gates are resolved; PRISM-003+ follow normal sequencing after PRISM-001
  and PRISM-002 story-done.

## Next Parallel Launch Candidates

Batch launched:
- GSS-005: closed at `19071b5`.
- GSS-006: closed at `a49e422`.
- GSS-007: closed at `7378e28`.
- HUD-008: closed at `07f477f`.

Active implementation workers by default-launch rule:
- None known at this checkpoint. COMBAT-003, HAND-UI-009, HAND-UI-008,
  AUC-006, BOARD-009, COMBAT-002, OBJECTIVE-004, BOARD-008, and HAND-UI-007
  have returned and are integrated or closed as noted; do not relaunch their
  implementation workers.

Current active windows by user default-launch rule:
- COMBAT-003 returned, integrated into main at `e5c28d5` and `7fdf4fd`,
  verified, and is on origin/main. Window can be cleared. It now needs
  serialized `/story-done` after HAND-UI-009.
- HAND-UI-009 returned, integrated into main at `a72bb0f`, verified, and is on
  origin/main. Window can be cleared. It now needs serialized `/story-done`
  after HAND-UI-008.
- HAND-UI-008 returned, integrated into main at `552f80f`, verified, and is on
  origin/main. Window can be cleared. It now needs serialized `/story-done`.
- BOARD-009 story-done returned and was committed at `394d6c3`; window can be
  cleared.
- AUC-006 story-done returned and was committed at `61e69b4`; window can be
  cleared.
- PRISM-005 story-done returned and was committed at `ef6b4ad`; window can be
  cleared.
- OBJECTIVE-003 story-done returned and was committed at `4327b7b`; window can
  be cleared.
- CDP-004 story-done returned and was committed at `ea00d32`; window can be
  cleared.
- HAND-UI-006 story-done returned and was committed at `c772af1`; window can be
  cleared.
- PRISM-001 story-done returned and was committed at `671caa2`; window can be
  cleared. PRISM-002 story-done returned and was committed at `2d1a4bf`;
  window can be cleared. HUD-004 story-done returned and was committed at
  `3c85ae1`; window can be cleared. CARD-ANIM-007 is now the next serialized
  story-done candidate.
  Do not launch another story-done until it returns.
- BOARD-007 returned, integrated at `fd13f2a`, and now only needs serialized
  story-done; story-done committed at `dc8b80a`. Window can be cleared.
- HUD-008 initially returned with no code changes while blocked on
  `S2CGameSnapshot`; after GSS-007 integration it returned implemented and is
  integrated at `d8971f4`; story-done committed at `07f477f`. Window can be
  cleared.
- PRISM-003 returned, integrated at `611baee`, and now only needs serialized
  story-done; story-done committed at `b4d9e04`. Window can be cleared.
- CARD-ANIM-007 story-done returned and was committed at `35ee469`; window can
  be cleared. HUD-006 story-done returned and was committed at `cc205e3`;
  window can be cleared. CARD-ANIM-005 story-done returned and was committed
  at `265f34b`; window can be cleared. AUC-004 story-done returned and was
  committed at `75b8998`; window can be cleared. RSM-006 story-done returned
  and was committed at `2f07c94`; window can be cleared. CS-003 story-done
  returned and was committed at `b940f70`; window can be cleared. HUD-007
  story-done returned and was committed at `cae0e45`; window can be cleared.
  BOARD-007 story-done returned and was committed at `dc8b80a`; window can be
  cleared. PRISM-003 story-done returned and was committed at `b4d9e04`;
  window can be cleared. HUD-009 story-done returned and was committed at
  `7f3ecfa`; window can be cleared. HUD-010 is now the next serialized
  story-done candidate.
- HAND-UI-004 returned, integrated at `561d2fd`, and now only needs serialized
  story-done; story-done committed at `f610054`. Window can be cleared.
- GSS-005 returned, integrated at `15fe812`, repaired/closed at `19071b5`, and
  its window can be cleared.
- GSS-006 returned, integrated at `d5f835e`, closed at `a49e422`, and its
  window can be cleared.
- HUD-010 returned, integrated at `609be61`, and now only needs serialized
  story-done after HUD-009; story-done committed at `d23ce6f`. Window can be
  cleared. HAND-UI-004 is now the next serialized story-done candidate.
- OBJ-002 returned NEEDS WORK only on stale manifest. Manifest refreshed to
  `2026-05-01` in `b8b9f26`; implementation integrated at `536ccc8`;
  story-done committed at `88f3fe2`. Window can be cleared.
- PRISM-004 returned, integrated at `8c77982`, story-done committed at
  `e7776b0`. Window can be cleared.
- HAND-UI-005 returned, integrated at `1c798f0`, story-done committed at
  `c8222d2`. Window can be cleared.
- GSS-007 returned, integrated at `8e7c5b5`, repaired at `32643e9`,
  story-done committed at `7378e28`. Window can be cleared.
- HUD-008 returned, integrated at `d8971f4`, story-done committed at
  `07f477f`. Window can be cleared.
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
