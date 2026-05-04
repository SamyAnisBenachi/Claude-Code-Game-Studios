# Codex Orchestrator State

Updated: 2026-05-04
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
- Worker launch prompts must include the pre-integration duty: before the final
  report, the worker rebases on latest `origin/main`, resolves conflicts inside
  its story worktree, reruns the full listed verification after the rebase,
  runs `git diff --check`, and pushes only the `work/...` branch if allowed.
  If worker branch push is blocked, the worker reports the local commit hash
  and final clean worktree status instead of attempting unsafe workarounds.
- Worker final reports must include: worktree path, branch, commit hash,
  changed files, exact checks run with results, blockers/notes, whether rebase
  happened, whether branch push succeeded, and final `git status`.
- If a worker touches shared protocol/config/Cargo workspace surfaces, its
  prompt must require `cargo check --workspace` after rebase. Otherwise use
  the narrow package check plus affected regressions.
- The root checkout stays reserved for orchestrator integration merges,
  story-done, CI triage, and state tracking.
- Root/orchestrator responsibilities after a worker return: only act after the
  user pastes the official return; update this memory file; integrate by
  cherry-pick/merge to `main`; run minimal trust checks rather than redoing the
  whole worker suite unless risk or shared surfaces require it; push `main`;
  then queue exactly one serialized `/story-done`.
- Workers must never push `main`, run `/story-done`, or edit
  `production/session-state/active.md`,
  `production/session-state/codex-orchestrator-state.md`, or
  `production/sprint-status.yaml` unless a prompt explicitly authorizes that
  specific tracking/closure work.
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
shared tracker prohibition, and detailed commit-body requirements. Do not
replace prompt triangles with colored circles; prompts keep the red triangle
prefix. Colored circles are only status/window labels immediately before the
word, for example `🟢 CLEAR` and `🟡 REPONDRE`.

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

None currently tracked here.

## Recent Planning / Readiness Updates

- COMBAT-008 / S5-10 RANGE Targeting readiness repair returned READY and was
  pushed at `b26f007`, updating `docs/architecture/tr-registry.yaml`,
  `production/epics/combat-resolution/EPIC.md`, and
  `production/epics/combat-resolution/story-008-range-targeting.md`. The repair
  updated the manifest version to `2026-05-01`, fixed TR coverage, aligned RNG
  guidance to intent-named `ServerRng` methods and strict
  `RangeEquidistantSelect` ordering, promoted equidistant RANGE selection to a
  blocking AC/QA case, expanded dependencies on Stories 004-007, and added the
  ADR-017 `<= 15 ms` RESOLUTION budget note. `/dev-story` is safe next with
  explicit authorization.
- COMBAT-005 readiness repair returned READY and was committed at `90ebdfb`.
  The story now traces `TR-CR-002`, `TR-CR-007`, `TR-CR-020`, and
  `TR-CR-021`, uses manifest `2026-05-01`, includes the ADR-017
  `<= 15 ms` RESOLUTION budget note, and clarifies minimal SS3 RANGE +
  FIRST STRIKE scope. A COMBAT-005 `/dev-story` prompt was given after this
  repair; by user default-launch rule, treat the implementation worker as
  launched unless the user says otherwise.
- AUC-008 readiness repair returned READY and was committed at `c20e503`.
  The story now uses manifest `2026-05-01`, removes stale ADR-013 wording,
  clarifies current Card Pool API ownership, removes stale `SharedNeutralPool`
  wording, and includes an auction tick performance/no-impact note. An
  AUC-008 `/dev-story` prompt was given after this repair; by user
  default-launch rule, treat the implementation worker as launched unless the
  user says otherwise.
- S5-21 planning artifacts for Board Rendering and Shop/Auction UI were
  written and pushed at `1c28e9c`: two EPIC.md files, 19 story files, and
  `production/epics/index.md`. This was planning/docs only; no code or
  session-state/sprint-status files were touched.
- Sprint 6 draft was produced as report-only and explicitly not written. Do
  not create `production/sprints/sprint-6.md` until S5-21 artifacts and the
  Sprint 5 combat spine are clearer.
- HAND-UI-010 / S5-17 story-readiness returned BLOCKED. Main blockers:
  server authoritative placement validation does not yet match the reserve /
  current-mana split spec, `PlacedCard` protocol shape is drifted between C2S
  and S2C reveal, and Hand UI lacks `current_mana` in its economy view. Do
  not implement HAND-UI-010 until the server/protocol/client data blockers are
  split or resolved. Follow-up blocker split returned read-only with no edits:
  docs-only readiness repair is not enough; create prerequisite implementation
  stories/repairs for (1) protocol shape split between submit/internal/reveal
  placement payloads, (2) server split-budget validation, duplicate card
  rejection, mandatory hand validation, target validation, and explicit
  reserve/current deduction, (3) C2S placement wiring from network message to
  `PlacementSubmissionReceived`, (4) client Hand UI `current_mana` economy
  mirror, and only then (5) HAND-UI-010 client validation gating for manual,
  timer, grace-expiry, and grace-drop submit paths. Do not run protocol split or
  combat-file changes casually in parallel with combat spine work; client
  economy mirror is the safest parallel slice after protocol API is stable.
- Shop/Auction UI UX readiness check returned that `design/ux/shop-auction-ui.md`
  is missing. It is advisory for Story 001 scaffold/formulas, but blocks
  visual/layout implementation and exact tooltip/layout work.
- Shop/Auction UI Story 001 readiness returned BLOCKED because
  `PresentationPlugin` / `PresentationSet` from ADR-021 do not exist yet.
  Presentation scaffold ownership analysis returned: create a new shared
  Presentation Layer story rather than hiding it in Board Rendering 001 or
  Shop/Auction UI 001. Proposed path:
  `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md`.
  Rationale: ADR-021 owns `PresentationPlugin`, `PresentationSet`, and
  `phase_sink_system` as cross-epic infrastructure, and its migration plan
  starts with `client/src/presentation/mod.rs` before BoardRenderingPlugin.
  Likely implementation surface: `client/src/presentation/mod.rs`,
  `client/src/lib.rs`, `client/src/main.rs`, existing HUD/Hand UI/Card
  Animations scheduling, and
  `tests/integration/presentation/presentation_plugin_scaffold_test.rs`.
  Board Rendering 001 and Shop/Auction UI 001 remain blocked until this shared
  story exists and is implemented.
- Presentation Layer Story 001 readiness returned READY. Traceability decision:
  ADR-021 plus current control manifest is sufficient because this is ADR-only
  presentation infrastructure and the TR registry is scoped to GDD technical
  requirements; no TR-PRES entry is required. Planning was committed at
  `514c1ad`; worker implementation returned local commit `e783a27` without a
  branch push, so root cherry-picked only that commit and pushed it to
  `origin/main` as `1c5c40f`. Story-done returned COMPLETE and was pushed at
  `d303155`, updating
  `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md`
  and `production/session-state/active.md`; no sprint-status row existed.
- Board Rendering Story 001 readiness returned NEEDS WORK. No trace,
  dependency, ADR, or manifest blocker was found. Required fixes are docs-only:
  add an explicit ADR-021 presentation performance/no-impact note and clarify
  `cell_to_world` invalid bounds/assert behavior inline (`lane 1..=5`,
  `cell 1..=8`, release `assert!`). The correct window action is `REPONDRE`
  with a readiness repair prompt, not `CLEAR`.
- Full-game asset coverage audit returned with verdict that coverage is not
  complete enough for full-game asset generation. No files were edited. Key
  blockers: manifest has 127 assets all `Needed`; main menu/lobby,
  settings/accessibility, game-over/result, combat VFX/audio, auction audio,
  keyword/status icons, Prism VFX/audio, and several UI interaction patterns
  are missing or underspecified; `design/ux/shop-auction-ui.md` is missing;
  accessibility conflicts need decisions, especially auction bid confirmation
  vs one-click preset bids; font choice and engine feasibility for 9-slice,
  blending, ImageScaleMode, particles, and OGG loops are not locked; manifest
  inconsistencies include ASSET-119, ASSET-071, class atlas dimensions, auction
  border over-baking, and board cell variant tinting. The full-game asset audit
  window can be cleared.
- Full card art coverage audit returned with verdict that card art coverage is
  not sufficient yet. No files were edited. Current `assets/data/cards.json`
  has 8 unique `art_id`s, but none has a unique per-card illustration spec and
  `assets/art/cards/` does not exist, so no display/zoom card illustration
  PNGs exist. Catalog planning is inconsistent (`~298` vs `~315` usable cards)
  and registry plans 2 Prism spell cards absent from `cards.json`. The hand UI
  asset spec currently conflicts with layered composition by describing baked
  full card faces while ASSET-055..059 define frame/badges/text/type layers.
  Required direction: per-card art should be illustration-only; frame, badges,
  text, type/rarity, hover/ghost, and state overlays compose at runtime. Generate
  or spec now only after fixing the layered-composition wording: 8 current
  per-card specs, 16 display/zoom PNG targets, and missing current type/rarity
  icons (`trap_epic`, `field_legendary`, `order_rare`,
  `doubleface_uncommon`). Defer the full ~315-card catalog until roster count,
  card IDs, and art IDs are reconciled. Card art audit window can be cleared.
  UI/audio/VFX coverage audit remains pending unless the user provides its
  return.
- Full-game asset manifest/spec expansion was written and committed at
  `bbde404` as `asset-spec: expand full-game manifest coverage`. The commit is
  asset-only under `design/assets/**`, with ASSET-001 through ASSET-234
  continuous, no duplicates/missing IDs, 20 allowed asset files, and 8 current
  per-card illustration specs. Validation passed `git diff --check` after EOF
  cleanup and staged allowlist verification. No assets or code were generated.
  Remaining design questions: optional bid confirmation UX, GAME_OVER/result
  screen UX, full ~315-card roster/art IDs, and accessibility/settings details.
- Asset audit all returned read-only on 2026-05-04 with verdict
  NON-COMPLIANT for production asset readiness. Runtime scan found 57 assets
  total: 55 PNGs under `assets/art`, `assets/data/cards.json`, and
  `assets/config/game_config.ron`; `assets/audio`, `assets/vfx`, and
  `assets/shaders` directories are missing. Only 17 manifest rows have
  file-presence credit, and all 55 PNGs are temporary placeholders, not
  production-approved final art. P0 missing groups after the board PNG batch:
  Board Rendering blocking SFX, current 8-card display/zoom pairs plus four
  type/rarity icons, hand shaders, Hand UI audio, and shared display fonts. Next
  production batch recommendation: hand current-card pack, shared font/shader
  pack, then minimal board/hand audio.
- Board Blocking Asset Batch completed and was pushed at `9f8060b`
  (`BR-M2 assets: add blocking board sprites`). It generated 17 PNGs under
  `assets/art/board`, `assets/art/ui/board`, `assets/art/characters`, and
  `assets/art/vfx/board`: lane labels 01-05, idle/active cell nodes,
  unknown/real objectives, facedown trap, Player A/B unit bases, unit
  placeholder, HP white pixel, and objective real flash frames 01-03. Validation
  passed: all PNG dimensions matched spec, files are `Format32bppArgb`, alpha
  was verified, HP pixel is fully opaque, `git diff --check` passed, and final
  status was clean. No code, audio, card art, production session files, or
  sprint-status files were touched. Follow-up asset-status reconciliation was
  pushed at `631a5fd`, updating only `design/assets/asset-manifest.md` and
  `design/assets/specs/board-rendering-assets.md` to mark the 17 PNGs as
  generated/file-present placeholders only, with no `Done`, `Approved`, or
  production-ready claims. Remaining non-blocking spec flags: ASSET-026 separate
  sprite vs runtime tint, ASSET-035 `ui_` prefix on a world-space board sprite,
  ASSET-039 64x96 canvas despite manifest/spec naming saying 48x64, and
  ASSET-044 odd-width HP bar exception.
- Audio production handoff returned and was committed at `64cfd7f`, creating
  `design/assets/production-handoffs/audio-production-queue-2026-05-04.md`.
  It covers 37 audio assets grouped as board/objective blocking SFX, Hand UI
  current interaction SFX, combat SS3/SS4/SS6 SFX, auction SFX, and class/lobby
  SFX. No audio files were generated and no manifest statuses were changed.
  The handoff notes Bevy 0.18 audio API verification and several sound-design
  confirmation points.

## Recently Closed

- Server RNG 003 / S5-18: Determinism Proof & Session Reset readiness repair
  landed at `46e1ee2`; direct story-done closure committed at `2dcb237`.
  Verification recorded: `cargo test -p server foundation::rng::tests` 19/19
  and `cargo check -p server`. Story-done updated
  `production/epics/server-rng/story-003-determinism-session-reset.md`,
  `production/session-state/active.md`, and sprint-status row S5-18.
  Verdict was COMPLETE WITH NOTES because RNG source comments still use
  shorthand TR-RNG-04/RNG13/RNG15 while story and registry trace are correct.
- COMBAT-005: First Strike readiness repair landed at `90ebdfb`; worker
  implementation returned at `912fa45`; root integrated and pushed it at
  `e1733be`; story-done closure committed at `621fef5`. Verification passed
  `cargo test -p server --test substep3_first_strike_test` 5/5, `cargo check
  -p server`, root focused/adjacent checks, and diff checks. Story-done
  updated `production/epics/combat-resolution/story-005-substep3-first-strike.md`,
  `production/session-state/active.md`, and sprint-status row S5-07. Advisory:
  SS4 dead removal/gold chain remains COMBAT-006 scope.
- AUC-008: Pool Integration readiness repair landed at `c20e503`; worker
  implementation returned at `d1e8cba`; root integrated and pushed it at
  `c6a1d86`; story-done closure committed at `9ff607e`. Verification passed
  `cargo test -p server --test auction_pool_integration_test` 5/5,
  `cargo check --workspace`, root auction/card-pool/RNG checks, and diff
  checks. Story-done updated
  `production/epics/auction-system/story-008-pool-integration.md`,
  `production/session-state/active.md`, and sprint-status row S5-14.
- COMBAT-006 / S5-08 readiness repair returned READY and was pushed at
  `acafff6`. Worker implementation returned at `ea43240` on
  `work/combat-006-dead-removal-death-chain-kill-gold`; root fast-forwarded
  and pushed that commit to `origin/main`. Verification passed
  `cargo fmt -p server -- --check`,
  `cargo test -p server --test substep4_dead_removal_test` 3/3,
  adjacent combat regressions 25/25, `cargo check -p server`, and
  `git diff --check`. Story-done returned COMPLETE WITH NOTES and was pushed at
  `e3e1cd7`, updating
  `production/epics/combat-resolution/story-006-substep4-dead-removal.md`,
  `production/session-state/active.md`, and sprint-status row S5-08. Advisory:
  implementation uses Bevy `EntityEvent` dispatch via `world.trigger(...)`
  rather than manifest wording `world.trigger_targets(...)`, with tests and
  compile confirming targeted synchronous DEATH-chain behavior.
- OBJECTIVE-006 / S5-15 readiness repair returned READY at `f6cdf58`. Worker
  implementation returned at `c07be2c` on
  `work/objective-006-d4-fake-reward-draw`, but the branch was behind current
  `origin/main` after COMBAT-006 integration, so root cherry-picked only the
  OBJECTIVE-006 commit and pushed it to `origin/main` as `d46e812`. Verification
  passed `cargo fmt -p server -- --check`,
  `cargo test -p server --test fake_reward_test` 8/8,
  `cargo test -p server --test consequence_path_test --test damage_interface_test`
  14/14, `cargo check -p server`, and `git diff --check`. Story-done returned
  COMPLETE WITH NOTES and was pushed at `8142875`, updating
  `production/epics/objective-system/story-006-d4-fake-reward-draw.md`,
  `production/session-state/active.md`, and sprint-status row S5-15. Advisory:
  current TR-OBJ-005 registry text is narrower than story/GDD coverage, but
  implementation and tests cover OS-15, OS-22, and OS-27.
- PRESENTATION-001 readiness/planning landed at `514c1ad`; worker
  implementation returned local commit `e783a27` on
  `work/presentation-001-plugin-set-phase-sink`, but branch push was rejected.
  The worker branch was behind current `origin/main`, so root cherry-picked
  only the presentation commit and pushed it to `origin/main` as `1c5c40f`.
  Verification passed `cargo fmt -p client -- --check`,
  `cargo test -p client --test presentation_plugin_scaffold_test` 3/3,
  adjacent HUD/Hand/Card Animations regressions 16/16,
  `cargo check -p client`, `git diff --check`, and the grep guard for exactly
  one `MessageReceiver<S2CPhaseChanged>` in `client/src`. Story-done returned
  COMPLETE and was pushed at `d303155`, updating the story file and
  `production/session-state/active.md`. No sprint-status row existed. Do not
  relaunch the PRESENTATION-001 worker.
- COMBAT-007 / S5-09 readiness repair returned READY and was pushed at
  `041e92f`, updating `docs/architecture/tr-registry.yaml`, the combat epic,
  and story 007 trace/manifest. Worker implementation returned at `2c8f752`
  on `work/combat-007-standard-combat-shield-counterattack`; root
  fast-forwarded and pushed that commit to `origin/main`. Verification passed
  `cargo fmt -p server -- --check`,
  `cargo test -p server --test substep6_combat_shield_counterattack_test` 7/7,
  adjacent combat regressions 13/13, `cargo check -p server`, and
  `git diff --check`. Story-done returned COMPLETE WITH NOTES and was pushed at
  `ee1a036`, updating
  `production/epics/combat-resolution/story-007-substep6-combat-shield-counterattack.md`,
  `production/session-state/active.md`, and sprint-status row S5-09. Advisory:
  current GDD CR-21 wording says COUNTERATTACK fires before SHIELD absorption,
  while active TR-CR-006, the story, and ADR-017 specify after incoming damage
  or SHIELD absorption; implementation follows active TR/story contract.
- BOARD-RENDERING-001 worker returned at `43ace3e` on
  `work/board-rendering-001-plugin-scaffold-layout-atlas`. The worker branch
  was behind current `origin/main`, so root cherry-picked only the board
  rendering commit and pushed it to `origin/main` as `b5abcd5`. Verification
  passed `cargo fmt -p client -- --check`,
  `cargo test -p client --test board_rendering_plugin_scaffold_test` 9/9,
  touched adjacent tests 14/14, `cargo check -p client`, and `git diff --check`.
  Story-done returned COMPLETE WITH NOTES and was pushed at `e2d81d9`, updating
  `production/epics/board-rendering/story-001-plugin-scaffold-board-layout-card-atlas.md`
  and `production/session-state/active.md`; no sprint-status row existed.
  Advisory: TR-BR-002 registry wording says Vec3, while current GDD/ADR and
  implementation use Vec2.
- COMBAT-004: Movement + Collision readiness repair landed on main at
  `c2fc0d3`; implementation landed on main at `408c34a`; story-done closure
  committed at `52caa45`; follow-up story scope clarification committed at
  `3ef7bab`. Verification passed `cargo test -p server --test
  movement_collision_test` 5/5, `cargo check -p server`, and diff checks.
  `production/sprint-status.yaml` row S5-06 was updated by story-done.
  Later SS3/SS6 attack/damage clauses were recorded as advisory/out-of-scope.
- CDP-006: Network Dispatch Wiring readiness/trace repair landed on main at
  `e03dfcb`; test/evidence implementation landed on main at `4d21482`;
  story-done closure committed at `199a9b3`. Verification passed `cargo test
  -p server --test shop_dispatch_test` 5/5, `cargo test -p server --test
  reconnect_snapshot_test acquisition_unicast_helpers_defer_while_snapshot_pending`
  1/1, `cargo check -p server`, and diff checks. `production/sprint-status.yaml`
  had no matching CDP-006 row.
- BOARD-010: Displacement Keywords readiness repair landed on main at
  `40b3176`; initial implementation landed on main at `e49f5a2`; enemy
  ATTRACT repair landed at `61e7042`; story-done closure committed at
  `6021787`. Verification passed `cargo test -p server --test
  displacement_keywords_test` 9/9, `cargo check -p server`, and diff checks.
  `production/sprint-status.yaml` had no matching BOARD-010 row. Stale story
  BL-24/QA wording was recorded as an advisory while implementation/tests
  follow current GDD one-cell-short behavior.
- HAND-UI-011: Reserve Mana Strip implementation landed on main at `bd8c15b`;
  story-done closure committed at `d203c3b`. Verification passed `cargo test
  -p client --test hand_ui_reserve_mana_strip_test` 3/3, `cargo check -p
  client`, and diff checks. `production/sprint-status.yaml` had no matching
  HAND-UI-011 row. TR-HU-004 and VA-9 differences were recorded as advisory
  notes.
- AUC-007: Auction Plugin Scheduling readiness repair landed on main at
  `f96a524`; implementation landed on main at `ea5d88d`; story-done closure
  committed at `a5eaadc`. Verification passed the targeted auction/economy
  test set 39/39 with 1 pre-existing ignored AUC-006 edge test, `cargo check
  -p server`, and diff checks. `production/sprint-status.yaml` had no matching
  AUC-007 row. AU1-b-network remains recorded as a deferred/open sprint-review
  note pending ADR-008 FIFO integration evidence.
- OBJECTIVE-005: Destruction Consequence Path readiness repair landed on main
  at `e903c69`; implementation landed on main at `cf93a8d`; story-done
  closure committed at `b5ecd56`. Verification passed `cargo test -p server
  --test consequence_path_test` 7/7, `cargo check -p server`, and diff checks.
  `production/sprint-status.yaml` had no matching OBJECTIVE-005 row.
- COMBAT-003: SS1 Placement Appearance trace repair landed on main at
  `e5c28d5`; implementation landed on main at `7fdf4fd`; story-done closure
  committed at `bcc73bb`. Verification passed `cargo test -p server --test
  substep1_placement_test` 4/4, `cargo check -p server`, and diff checks.
  `production/sprint-status.yaml` had no matching COMBAT-003 row.
- HAND-UI-009: Placement Timer implemented on branch
  `work/hand-ui-009-placement-timer` at worker commit `7a8c173`; root
  integration landed on main at `a72bb0f`; story-done closure committed at
  `8344521`. Verification passed `cargo test -p client --test
  hand_ui_placement_timer_test` 4/4 and `cargo check -p client`.
  `production/sprint-status.yaml` had no matching HAND-UI-009 row.
- HAND-UI-008: Placement Unstaging implemented on branch
  `work/hand-ui-008-placement-unstaging` at worker commit `743d660`; root
  integration landed on main at `552f80f`; story-done closure committed at
  `90c2c3a`. Verification passed `cargo test -p client --test
  hand_ui_placement_unstaging_test` 4/4 and `cargo check -p client`.
  `production/sprint-status.yaml` had no matching HAND-UI-008 row.
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

None currently queued.

Run only one story-done at a time.

## Launch Blocks / Wait Conditions

- Sprint 5 plan exists at `production/sprints/sprint-5.md` and was committed
  at `e8455f1`. It explicitly includes a revalidation note for pull-forward
  launched before the formal Sprint 5 plan existed:
  - AUC-007 Auction Plugin Scheduling
  - HAND-UI-011 Reserve Mana Strip
  - BOARD-010 Displacement Keywords
  - CDP-006 Network Dispatch Wiring
  Treat those items as Sprint 5 in-flight only after their worker readiness
  repairs return READY and are reconciled during integration/story-done.
  AUC-007 has returned READY, integrated at `f96a524`/`ea5d88d`, and closed
  at `a5eaadc`; HAND-UI-011 is integrated at `bd8c15b` and closed at
  `d203c3b`; BOARD-010 is integrated at `40b3176`/`e49f5a2`, repaired at
  `61e7042`, and closed at `6021787`; CDP-006 is integrated at
  `e03dfcb`/`4d21482` and closed at `199a9b3`. All listed pull-forward
  revalidation items are now integrated and story-done closed.
- Sprint 5 QA plan exists at `production/qa/qa-plan-sprint-5-2026-05-04.md`
  and was committed at `f71a736`.
- `production/sprint-status.yaml` was rebuilt for Sprint 5 at `b932290`.
  COMBAT-004 story-done later updated S5-06 to done at `52caa45`.
- S5-21 Board Rendering and Shop/Auction UI planning artifacts exist at
  `1c28e9c`; sprint hygiene cleanup at `625c416` marked S5-21 done. S5-22
  duplicate/stale Card Data Pool cleanup was also closed at `625c416`, retiring
  stale duplicate Ready story files and marking the canonical Card Data Pool
  epic/stories complete in docs/status. Presentation Layer Story 001 is closed
  at `d303155`, Board Rendering Story 001 is implemented/closed at `b5abcd5` /
  `e2d81d9`, and Shop/Auction UI Story 001 readiness is READY after `f847020`.
  Next UI implementation candidate after current story-done work is Shop/Auction
  UI Story 001.
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
- Full-game asset coverage audit, card art coverage audit, and UI/audio/VFX
  coverage audit have returned. The full-game asset manifest/spec expansion was
  committed at `bbde404`; no asset audit window remains active.
- HAND-UI-009, HAND-UI-008, AUC-006, BOARD-009, COMBAT-002, OBJECTIVE-004,
  BOARD-008, and HAND-UI-007 have returned and are integrated or closed as
  noted; do not relaunch their implementation workers.

Current active windows by user default-launch rule:
- COMBAT-008 readiness repair returned READY and was pushed at `b26f007`.
  Readiness window can be cleared. COMBAT-008 implementation worker returned at
  `dd7bd50`; root fast-forwarded and pushed `main` to that commit. Worker
  window can be cleared. Story-done returned COMPLETE WITH NOTES and was pushed
  at `6f6b40b`, updating the story, active session state, and S5-10 sprint row.
  Story-done window can be cleared.
- OBJECTIVE-007 implementation worker returned; root cherry-picked only the
  implementation because the branch was behind current `main` and pushed
  integration commit `1c8ef2a`. Story-done returned COMPLETE WITH NOTES and
  was pushed at `90e9b8f`, updating the story, active session state, and S5-16
  sprint row. Window can be cleared.
- BOARD-RENDERING-001 implementation worker returned; root cherry-picked and
  pushed only the board rendering commit as `b5abcd5` because the branch was
  behind current main; story-done returned and pushed closure at `e2d81d9`.
  Window can be cleared.
- COMBAT-007 implementation worker returned, root integrated and pushed commit
  `2c8f752`; story-done returned and pushed closure at `ee1a036`. Window can
  be cleared.
- COMBAT-006 implementation worker returned, root integrated and pushed commit
  `ea43240`; story-done returned and pushed closure at `e3e1cd7`. Window can
  be cleared.
- OBJECTIVE-006 implementation worker returned; root cherry-picked and pushed
  only the objective commit as `d46e812` because the worker branch was behind
  current main. Story-done returned and pushed closure at `8142875`. Window can
  be cleared.
- COMBAT-005 story-done returned, wrote closure files, and pushed closure at
  `621fef5`; window can be cleared.
- Server RNG 003 story-done returned, wrote closure files, and pushed closure
  at `2dcb237`; window can be cleared.
- AUC-008 readiness repair returned READY and was committed at `c20e503`;
  that readiness window can be cleared. AUC-008 implementation worker returned
  complete at `d1e8cba`; root integrated and pushed it at `c6a1d86`. Window
  can be cleared. AUC-008 story-done returned, wrote closure files, and pushed
  closure at `9ff607e`; window can be cleared.
- S5-21 planning artifacts returned and were committed at `1c28e9c`; window
  can be cleared. Sprint hygiene S5-21/S5-22 cleanup returned and was committed
  at `625c416`; window can be cleared.
- Sprint 6 draft returned as report-only and was not written; window can be
  cleared.
- HAND-UI-010 readiness returned BLOCKED; window can be cleared and should
  not be relaunched until server/protocol/client data blockers are addressed.
  HAND-UI-010 blocker split returned read-only and confirmed new prerequisite
  implementation stories are needed; blocker split window can be cleared.
- Shop/Auction UX readiness check returned; window can be cleared.
- Shop/Auction UI Story 001 readiness returned BLOCKED on missing
  PresentationPlugin / PresentationSet; Presentation scaffold analysis returned
  and recommends a new shared Presentation Layer story. Window can be cleared.
- Presentation Layer Story 001 readiness returned READY; window can be cleared.
  Presentation implementation worker returned local commit `e783a27`; root
  cherry-picked and pushed it as `1c5c40f`; story-done returned and pushed
  closure at `d303155`. Window can be cleared.
- Board Rendering Story 001 readiness returned NEEDS WORK on missing
  presentation performance/no-impact note and ambiguous invalid
  `cell_to_world` bounds/assert wording. Use `REPONDRE` in that same window
  to repair the story, rerun readiness, and commit/push docs-only changes; do
  not create a worker or implement code until it returns READY.
- Full-game asset coverage audit returned: verdict says asset coverage is not
  complete enough for full-game asset generation; window can be cleared. Card
  art coverage audit returned: card art coverage is insufficient, current 8
  cards need per-card illustration specs/display/zoom PNG targets, global
  type/rarity icons are missing for current card types, and full catalog art is
  deferred until roster/art IDs are reconciled; window can be cleared.
- UI/audio/VFX coverage audit returned and led into the full-game asset
  manifest/spec expansion committed at `bbde404`; window can be cleared.
- Asset audit existing runtime files returned read-only with NON-COMPLIANT
  verdict and no edits; window can be cleared. The trailing `/review on my
  current changes` is not applicable to that audit because it made no changes.
- Board Blocking Asset Batch returned complete and was committed at `9f8060b`,
  generating the requested 17 PNG board/objective/unit-base/VFX files. Window
  can be cleared. Asset-status reconciliation returned and was committed at
  `631a5fd`, marking the 17 PNGs as generated/file-present placeholders only.
  That reconciliation window can also be cleared.
- Audio Production Handoff returned and was committed at `64cfd7f`; window can
  be cleared.
- COMBAT-004 story-done returned and was committed at `52caa45`; follow-up
  scope clarification committed at `3ef7bab`; window can be cleared.
- CDP-006 story-done returned and was committed at `199a9b3`; window can be
  cleared.
- BOARD-010 story-done returned and was committed at `6021787`; window can be
  cleared.
- HAND-UI-011 story-done returned and was committed at `d203c3b`; window can
  be cleared.
- AUC-007 story-done returned and was committed at `a5eaadc`; window can be
  cleared.
- OBJECTIVE-005 story-done returned and was committed at `b5ecd56`; window can
  be cleared.
- COMBAT-003 story-done returned and was committed at `bcc73bb`; window can
  be cleared.
- HAND-UI-009 story-done returned and was committed at `8344521`; window can
  be cleared.
- HAND-UI-008 story-done returned and was committed at `90c2c3a`; window can
  be cleared.
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
