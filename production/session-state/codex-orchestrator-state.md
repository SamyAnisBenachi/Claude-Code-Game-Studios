# Codex Orchestrator State

Updated: 2026-05-07
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
- Delegation boundaries: delegate docs/readiness repairs, UX/docs repairs, asset
  batches, and worker `/dev-story` tasks to agents whenever their prompts can
  include scoped commit/push rules. Delegate `/story-done` too, but keep exactly
  one story-done active at a time because it edits shared closure files.
  Orchestrator keeps dependency decisions, window triage, prompt sequencing, and
  main-branch worker integration unless an explicit one-off integration prompt is
  issued. Do not let orchestrator become the bottleneck by committing this memory
  file for every minor return; batch memory updates unless a durable blocker,
  closure, integration, or policy change needs to persist.
- The orchestrator cannot approve reviewer-blocked actions inside another agent
  window on the user's behalf. When approval is needed, provide the exact short
  approval line the user should paste back into that same window.
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

Do not wait for the user to ask what can run in parallel. After every agent
return, explicitly state whether there are new prompts to launch now. Distinguish
`RELANCER`/repair prompts for an existing window from new launch prompts for a
new window. If new work is safe, provide the complete numbered prompts in the
same response; if not, state the blocker. Assume prompts the orchestrator
provides are launched unless the user explicitly says otherwise.

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

Final red-line prompt policy: when instructing workers about their final line,
do not paste literal HTML/CSS/ANSI/Markdown wrappers or bracket placeholders
such as `<span style="color:red">349: S9-BACKLOG-PREP: [STATUS]</span>`.
Workers sometimes copy those literally. Instead, write the requirement in plain
language: the last visible line must be red, must contain exactly
`PROMPT-NUMBER: TICKET-ID: STATUS`, and `STATUS` must be replaced with the true
outcome chosen by the worker. No line may follow it.

Prompt number policy: prompt numbers are global and monotonically increasing
within the orchestration conversation. Do not reset the index to 1 in later
answers. The last numbered response prompt issued was REPONSE 5, so the next
new launch prompt number is PROMPT 6 unless a later memory update records a
newer number.

Window instruction policy: use uppercase `CLEAR` or `REPONDRE`. `CLEAR` means
the user should close that window; do not add redundant wording like "do not
respond" after `CLEAR`. `REPONDRE` means the next prompt belongs in that same
existing window. If a prompt follows a `CLEAR` instruction, it is for a new
agent/window unless a specific `REPONDRE` line says otherwise.

## GitHub CLI And CI Notes

- 2026-05-07 CI auth repair: user GitHub CLI auth is valid in normal
  PowerShell and in Codex when commands run outside the sandbox. Sandboxed
  `gh auth status -h github.com` can still report an invalid default token
  because the sandbox cannot read the Windows keyring token. For CI triage,
  use escalated `gh` commands against the existing keyring auth; do not ask the
  user to relogin or store a plain-text token unless they explicitly approve
  that security tradeoff.
- 2026-05-07 latest `tests.yml` on `main` failed at `Run Cargo Tests` /
  `Check Board Rendering source guards` because Linux CI could not find
  `wayland-client` for `wayland-sys`. Local fix applied but not committed:
  `.github/workflows/tests.yml` now installs `libwayland-dev` with the other
  Linux packages. Older target run `25485272040` for commit `54681615` was also
  failed and is superseded by later failing `main` runs until this workflow fix
  is committed/pushed and CI reruns.

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

## Sprint 9 Active Coordination

- Prompt 409 activated Sprint 9 from current `origin/main`
  `879fd1dc4bd426d0d3ea4a985d73975755042c7c`. The active plan is
  `production/sprints/sprint-9.md`; `production/sprint-status.yaml` is the
  machine-readable source of truth.
- Sprint 9 activation was docs/status only. No `/dev-story`, `/story-done`,
  smoke, QA sign-off, `/team-qa`, `/gate-check`, implementation, or CI was run.
- S9-RS-001 is tracked as in progress only because worktree
  `D:\_DEV\claude-code-game-studios-worktrees\S9-RS-001` on branch
  `work/s9-rs-001-result-ack-contract` exists with uncommitted local
  session/network changes and is behind current `origin/main`. It is not
  integrated and must not be treated as complete.
- S9-CONTENT-001 / neutral card display placeholder pack is integrated on main
  at `424bcfa0b60cea5dba0d1cb920ac4a3221b9ae4f` as a supporting content/asset
  slice. No standalone story file exists and no `/story-done` was forced. No
  asset approval, full card production, full balance completion, public release
  readiness, or full game completion is claimed.
- S9-NATIVE-001 is ready for dev, not in progress. No active native operator
  controls implementation worker was found during activation.
- S9-RS-002 remains blocked until S9-RS-001 completes and is integrated.
  S9-RS-003 remains blocked until S9-RS-001 and S9-RS-002 complete and are
  integrated. S9-QA-001 remains blocked until operator/browser controls plus
  result flow are usable. S9-QA-002 remains blocked/backlog until evidence or a
  blocker record exists.
- Sprint 8 carried conditions remain active in Sprint 9: S8-QA-001-W1
  manual/browser `GAME_OVER` gap, `QA-COND-0005` friend-game-only accepted
  risk, `QA-COND-0006` accepted-risk/deferred, no public release readiness, no
  release-candidate readiness, no full game completion, no full playable-client
  manual QA, no broad accessibility completion, and no playtest/fun-hypothesis
  validation.

## Recently Implemented, Needs Formal Story-Done

- NP-005 / Placement Payload Shape Split is complete on `origin/main`. Worker branch
  `work/np-005-placement-payload-shape-split` returned at `103d27d`, pushed to
  `origin/work/np-005-placement-payload-shape-split`. Root cherry-picked the
  worker commit into `main` as `e65fcfe` (`NP-005 impl: split placement payload
  shapes`), resolving a conflict in `server/src/feature/combat/mod.rs` by
  combining NP-005 reveal payload conversion with COMBAT-011 placement reveal
  broadcast/enqueue behavior. Root added follow-up integration fix `48b4b5f`
  (`NP-005 integration: update resolution event test payload`) so
  `tests/integration/combat/resolution_event_log_test.rs` uses
  `AcceptedPlacement` rather than removed `PlacedCard`. Changed files include:
  `shared/src/protocol.rs`, `client/src/ui/hand/mod.rs`,
  `server/src/feature/board/mod.rs`, `server/src/feature/board/placement.rs`,
  `server/src/feature/combat/mod.rs`,
  `production/qa/evidence/placement-payload-shape-split-evidence.md`, and
  affected board/hand/combat tests. Root verification passed:
  `cargo fmt -p shared -- --check`, `cargo fmt -p server -- --check`,
  `cargo fmt -p client -- --check`, `cargo check -p shared`,
  `cargo test -p shared` 5/5, `cargo check -p server`, `cargo check -p client`,
  affected server tests 12/12, affected client tests 17/17,
  `cargo test -p server --test resolution_event_log_test` 3/3,
  `cargo check --workspace`, and `git diff --check origin/main..HEAD` before
  push. Story-done closure is pushed at `705defa`; do not relaunch the
  implementation worker or closure window.
- ECO-007 / Explicit Placement Mana Split API is complete on `origin/main`.
  Worker branch
  `work/eco-007-explicit-placement-mana-split-api` returned at `b1c678f`,
  pushed to `origin/work/eco-007-explicit-placement-mana-split-api`. Root
  cherry-picked the worker commit into `main` as `dacc7d3` (`ECO-007 impl:
  explicit placement mana split API`) and pushed it to `origin/main`. Changed
  files: `server/Cargo.toml`, `server/src/core/economy/api.rs`,
  `server/src/core/economy/mod.rs`, `server/src/core/economy/state.rs`, and
  `tests/unit/economy/explicit_placement_mana_split_test.rs`. Root verification
  passed: `cargo fmt -p server -- --check`,
  `cargo test -p server --test explicit_placement_mana_split_test` 6/6,
  `cargo test -p server --lib economy::api::tests` 20/20, economy adjacent
  regression bundle 26/26, `cargo check -p server`, and
  `git diff --check HEAD~1..HEAD`. Story-done closure is pushed at `a564d99`
  and the window can be cleared; do not relaunch the implementation worker.
- COMBAT-011 / S5-20 ResolutionEvent Log Completeness is integrated on
  `origin/main` and needs serialized `/story-done`. Worker branch
  `work/combat-011-resolution-event-log` returned at `06d5b17`, pushed to
  `origin/work/combat-011-resolution-event-log`. Root cherry-picked the worker
  commit into `main` as `73ad695` (`COMBAT-011 impl: Resolution event log
  serialization`) and pushed it to `origin/main`. Changed files:
  `shared/src/protocol.rs`, `server/src/feature/combat/mod.rs`,
  `server/src/feature/objective/system.rs`, `server/Cargo.toml`,
  `tests/integration/combat/resolution_event_log_test.rs`,
  `tests/integration/objective/resolution_sync_test.rs`,
  `tests/unit/combat/movement_collision_test.rs`, and
  `tests/unit/combat/substep1_placement_test.rs`. Root verification passed:
  `cargo fmt -p server -- --check`, `cargo fmt -p shared -- --check`,
  `cargo test -p server --test resolution_event_log_test` 3/3, combat/objective
  regression slice 45/45, `cargo check -p server`, `cargo check -p shared`,
  `cargo check --workspace`, and `git diff --check HEAD~1..HEAD`. Queue exactly
  one `/story-done` for
  `production/epics/combat-resolution/story-011-resolution-event-log.md`; do
  not relaunch the implementation worker.

## Recent Planning / Readiness Updates

- COMBAT-011 worker Hilbert returned final commit `06d5b17`, rebased, verified,
  and pushed `origin/work/combat-011-resolution-event-log`. Root integrated and
  pushed it to `origin/main` as `73ad695`. Worker window can be cleared; queue
  serialized story-done only.
- BOARD-RENDERING-003 readiness returned NEEDS WORK, not blocked. Story 001 and
  Story 002 are Complete; ADR-020/ADR-021 are Accepted; `TR-BR-003` is active.
  Gaps before `/dev-story`: story manifest is stale (`2026-05-01` vs current
  control manifest `2026-05-05`), GDD/TR trace is incomplete for snapshot
  rebuild / HP bars / objective identity isolation / missing-card fallback /
  pending state reconciliation, and the rendering performance budget note is
  missing. Repair docs-only, then rerun readiness; do not launch implementation
  yet.
- BOARD-RENDERING-003 docs-only readiness repair returned READY and was pushed
  at `171997b`. Changed file:
  `production/epics/board-rendering/story-003-snapshot-spawn-units-objectives-and-hp-bars.md`.
  Remaining blockers/gaps: none. `/dev-story` is safe next from readiness.
  Story 001/002 are Complete and referenced server replicated components exist
  locally. No code, worktrees, production session-state, sprint-status, or
  design assets were touched.
- BOARD-RENDERING-004 readiness returned NEEDS WORK, not blocked by hard
  dependencies. Story 001/002 are Complete, ADR-021 is Accepted, and Hand UI
  ghost message types exist. Gaps before `/dev-story`: manifest is stale
  (`2026-05-01` vs current control manifest `2026-05-05`), requirement trace is
  wrong/incomplete because `TR-BR-002` maps to BoardLayout rather than ghost
  lifecycle/spawn highlights, spawn range scope conflicts with Story 009
  deferral, GDD Rule 8 coverage is missing `GhostDragStartEvent` and
  variant-specific ghost behavior ACs, and the rendering/PLACEMENT-loop
  performance budget note is missing. Repair docs-only, then rerun readiness.
- BOARD-RENDERING-004 docs-only repair commit `edfe87f` was already present
  locally above `origin/main` when integrating NP-005 and was pushed with the
  NP-005 integration batch. It changes only
  `production/epics/board-rendering/story-004-ghost-preview-hand-ui-bridge.md`.
  Treat this as observed until the owning agent's official repair/readiness
  report is pasted.
- SAU-002 readiness returned NEEDS WORK, not hard-blocked. ADR-015/ADR-021 are
  Accepted, `TR-SAU-006` is active, Story 001 is Complete, and draft offering /
  network dispatch dependencies appear Complete. Gaps before `/dev-story`:
  manifest stale (`2026-05-01` vs `2026-05-05`), ADR-021/control-manifest rules
  not noted, Bevy 0.18 and Lightyear engine notes too thin, performance budget
  missing, dependency wording should name exact completed paths such as Card
  Acquisition Story 002 and Card Data Pool Story 006, and tooltip
  placement/persistence needs clear Out-of-Scope placement. Repair docs-only,
  then rerun readiness.
- SAU-004 readiness returned NEEDS WORK, not hard-blocked. GDD path exists,
  `TR-SAU-006` is active, ADR-013/ADR-021 are Accepted, Story 001 is Complete,
  acceptance criteria and evidence path are clear, and no unresolved markers or
  asset references were found. Gaps before `/dev-story`: manifest stale
  (`2026-05-01` vs current `2026-05-05`), control manifest rules not cited
  (`phase_sink_system` / `CurrentClientPhase`, no sub-plugin direct phase
  drain, Bevy UI Required Components, single-drain Lightyear handling), engine
  notes too thin for Bevy 0.18 / Lightyear 0.26, and performance budget missing
  for UI state/timer/message-driven activation. Repair docs-only, then rerun
  readiness.
- Shop/Auction UI UX review finished with verdict NEEDS REVISION. It is not a
  major redesign and does not block SAU-001, but it blocks clean final
  visual/accessibility handoff and SAU-009. Full blockers: bid accessibility
  conflict (`design/accessibility-requirements.md` requires bid confirmation
  while UX/GDD/stories use immediate preset bid buttons), stale
  `interaction-patterns.md` Auction Bid Input assumptions plus missing patterns
  for components invented by the UX spec, unresolved hand tray / bottom
  resources / panel vertical split (`OQ-SAU-UX-2`), unresolved tooltip storage,
  missing `S2CCardAcquired` in data requirements, underspecified DRAFT_SHOP
  confirmed-purchase / empty-dead slot behavior, and stale SAU-009 tracker text
  saying `design/ux/shop-auction-ui.md` does not exist. Safe now: SAU-001,
  SAU-002 core, SAU-003 core, SAU-004, and SAU-007 state/logic. SAU-005/006 are
  safe only if the team explicitly keeps immediate preset bids and repairs
  accessibility docs accordingly. Recommended repair changeset: edit
  `design/ux/shop-auction-ui.md`, `design/ux/interaction-patterns.md`, and
  `design/accessibility-requirements.md`; optional later tracker hygiene for
  `production/epics/shop-auction-ui/story-009-visual-evidence-layout-and-accessibility.md`.
- Shop/Auction UX repair returned and was pushed at `a3739db` (`docs: repair
  shop auction UX review blockers`). Changed files:
  `design/ux/shop-auction-ui.md`, `design/ux/interaction-patterns.md`,
  `design/accessibility-requirements.md`, and
  `production/epics/shop-auction-ui/story-009-visual-evidence-layout-and-accessibility.md`.
  Resolved blockers: immediate preset bid buttons kept with misclick
  mitigations, stale Auction Bid Input assumptions replaced by Auction Bid
  Button, missing Shop/Auction patterns added, tooltip storage resolved to local
  preferences/localStorage, `S2CCardAcquired` data requirement added,
  DRAFT_SHOP confirmed-purchase and empty/dead slot behavior defined, toast
  duration / `REFRESH · 1g` / open-time performance criterion / numeric
  localization 40% expansion / vertical HUD-hand-panel contract added, and
  SAU-009 stale "UX spec does not exist" wording replaced. Remaining blockers:
  SAU-009 still needs renderable panel states from Stories 002-007 and evidence
  capture; YOU ARE LEADING idle-window playtest risk remains open; help/pause
  retrieval for dismissed tutorials remains low-priority open UX. Safe
  afterward: SAU-001 remains safe, SAU-005/006 are no longer blocked by bid
  confirmation conflict, and SAU-009 now waits on implementation/evidence
  prerequisites rather than missing UX repair docs. Verification passed:
  `git diff --check` and `git diff --cached --check`.
- HAND-UI-010 prerequisite docs changeset returned and was pushed at `935f090`
  (`docs: add HAND-UI-010 prerequisite stories`). It created/updated the
  cross-epic blocker chain without touching production session-state or
  sprint-status. New prerequisite stories: NP-005 Placement Payload Shape Split
  (`production/epics/lightyear-protocol-verification/story-005-placement-payload-shape-split.md`,
  Status Ready, `TR-NP-013`), ECO-007 Explicit Placement Mana Split API
  (`production/epics/economy-system/story-007-explicit-placement-mana-split-api.md`,
  Status Ready, `TR-ECO-009`), BLS-011 Placement Submit Authority Validation
  (`production/epics/board-lane-system/story-011-placement-submit-authority-validation.md`,
  Status Blocked on NP-005 + ECO-007, `TR-BLS-011`), and PRES-002 Shared Economy
  View (`production/epics/presentation-layer/story-002-shared-economy-view.md`,
  Status Ready, `TR-PRES-001`). `production/epics/hand-ui/story-010-submit-prevalidation.md`
  is now marked Blocked on Story 005, NP-005, ECO-007, BLS-011, and PRES-002.
  Final blocker chain: `NP-005 + ECO-007 -> BLS-011`; `BLS-011 + PRES-002 ->
  HAND-UI-010`.
- PRES-002 / Presentation Layer Story 002 readiness returned READY with no file
  changes or commit. Story:
  `production/epics/presentation-layer/story-002-shared-economy-view.md`.
  `TR-PRES-001` is active; manifest version `2026-05-05` matches the control
  manifest; ADR-021, ADR-002, ADR-008, and ADR-019 are Accepted;
  Presentation Story 001 is Complete; `S2CGoldUpdate`, `S2CGameSnapshot`, and
  `PlayerSnapshot` exist; no unresolved markers or asset references were found.
  `/dev-story` is safe next. Future implementation should use `liv-bevy-018`,
  plus `liv-bevy-lightyear` if touching `MessageReceiver<S2CGoldUpdate>` or
  other Lightyear drain code. HAND-UI-010 remains blocked until PRES-002 is
  implemented, not just readiness-cleared.
- ECO-007 / Economy Story 007 Explicit Placement Mana Split API readiness
  returned READY with no file changes or commit. Story:
  `production/epics/economy-system/story-007-explicit-placement-mana-split-api.md`.
  `TR-ECO-009` is active; EC27/EC28 trace to the Economy GDD; ADR-019 and
  ADR-002 are Accepted; manifest version `2026-05-05` matches the control
  manifest; Economy Story 001 is Complete; the test evidence path is defined.
  `/dev-story` is safe next. HAND-UI-010 still depends on NP-005, BLS-011, and
  PRES-002 in addition to ECO-007 completion.
- NP-005 / Lightyear Protocol Verification Story 005 Placement Payload Shape
  Split readiness returned READY with no file changes or commit. Story:
  `production/epics/lightyear-protocol-verification/story-005-placement-payload-shape-split.md`.
  `TR-NP-013` is active; manifest version `2026-05-05` matches the control
  manifest; ADR-002, ADR-003, ADR-007, and ADR-008 are Accepted; dependency
  Story 002 is Complete; `shared/src/protocol.rs` and current protocol
  registration scaffold exist. `/dev-story` is safe next. Future implementation
  should use `liv-bevy-lightyear`, plus `liv-bevy-018` if any Bevy-importing
  Rust file is touched. Evidence requirements: `cargo check -p shared` plus
  grep evidence in
  `production/qa/evidence/placement-payload-shape-split-evidence.md`.
- SHOP-AUCTION-UI-001 and BOARD-RENDERING-002 story-done closures were committed
  together at `3222be0` after both official story-done reports were received.
  The combined closure was required because both story-done windows had written
  extracts into `production/session-state/active.md`. Changed files:
  `production/epics/shop-auction-ui/story-001-plugin-scaffold-panel-tree-and-formulas.md`,
  `production/epics/board-rendering/story-002-board-grid-camera-and-z-layers.md`,
  and `production/session-state/active.md`. No sprint-status row existed for
  either story, so `production/sprint-status.yaml` stayed unchanged. Both
  windows can be cleared.
- Settings/Accessibility UX design returned and was pushed at `093b62a`,
  creating `design/ux/settings-accessibility.md` only. Validation passed
  `git diff --check` and a path-specific check. Open questions recorded in the
  spec: timer multiplier authority, `0.5x` timer option, settings persistence
  layer, Bevy/browser semantic accessibility support, reserved shortcut list,
  brightness/gamma implementation, audio Master bus, and tutorial/help
  ownership. Blocker status: none for the UX spec. Implementation still needs
  decisions on timer multiplier authority, preference storage, and semantic
  accessibility support. Asset/spec implications: settings UI controls/patterns,
  accessibility preview strip assets, audio bus support, help/tutorial prompt
  registry, and interaction-pattern additions.
- Result Screen UX design returned and was pushed at `399bb34`, creating
  `design/ux/result-screen.md` only. Validation passed `git diff --check`.
  Open blockers captured: full post-game opponent objective reveal needs server
  data, GAME_OVER reconnect needs result payload or `S2CGameOver` resend,
  rematch protocol is undefined, and `C2SAcknowledgeResult` timing needs a
  session-system decision. Downstream unlocked: result screen UI
  implementation, post-game summary / GAME_OVER reconnect protocol story,
  rematch flow story, full objective reveal payload work, and result overlay /
  objective reveal interaction patterns/assets.
- Current card type/rarity HUD icon batch returned and was pushed at `43bd1f8`,
  generating four files under `assets/art/ui/hand/`:
  `ui_icon_trap_epic_default_hud.png`,
  `ui_icon_field_legendary_default_hud.png`,
  `ui_icon_order_rare_default_hud.png`, and
  `ui_icon_doubleface_uncommon_default_hud.png`. Used imagegen/built-in image
  generation path, not API CLI. Validation passed: all four are 24x24 PNG RGBA
  (`Format32bppArgb`), transparent corners, nonzero alpha coverage, and
  `git diff --check` passed with only unrelated CRLF warnings. No code,
  manifest, production session-state, or sprint-status files were touched.
- Current 8-card illustration batch returned and was pushed at `330806f`,
  generating 16 PNGs: one `assets/art/cards/zoom/card_<art_id>_art_zoom.png`
  at 240x360 and one
  `assets/art/cards/display/card_<art_id>_art_display.png` at 120x180 for each
  of the 8 current `assets/data/cards.json` art IDs. Skipped files: none; no
  filename collisions, so no numeric suffixes were needed. Validation passed:
  PNG signatures valid, bit depth 8 / color type 6 RGBA for all files,
  dimensions match requested sizes, `git diff --check` passed with only
  unrelated CRLF warnings. No manifest/code edits; production session-state and
  sprint-status untouched.
- Remaining Board Rendering placeholder PNG batch returned and was pushed at
  `9fa1908`, generating remaining board PNG placeholders under
  `assets/art/board/`, `assets/art/ui/board/`, and `assets/art/vfx/board/`.
  Generated files included `env_board_background_default.png`,
  `env_cell_node_inactive_board.png`, `env_cell_node_invalid_board.png`,
  `env_objective_fake_crack_board.png`, `env_prism_idle_board.png`,
  `ui_structure_token_default_board.png`, `ui_field_wash_lane_default.png`,
  `ui_field_wash_lane_default_1.png`,
  `ui_field_badge_icon_default_hud.png`,
  `vfx_objective_attack_ring_loop.png`,
  `vfx_spawn_range_pulse_loop.png`, and
  `vfx_prism_collect_shimmer_01.png`. Skipped: none. ASSET-037 produced two
  spec variants; the second used `_1` due to the no-overwrite rule. Validation
  passed: PNG signatures valid, color type 6 RGBA / `Format32bppArgb`,
  dimensions match spec, transparent assets have alpha/transparent corners,
  ASSET-037 variants are 512x128 with top 48px transparent and max alpha 51,
  `git diff --check` and `git diff --cached --check` passed with only unrelated
  CRLF warnings. Manifest assets were not marked Done.
- Shop/Auction UI PNG asset pack returned and was pushed at `efb95a1`,
  generating 18 PNGs under `assets/art/ui/shop_auction/` and
  `assets/art/vfx/shop_auction/`: gold coin 48/24, rarity gems rare/epic/
  legendary display variants, shop slot highlight, auction panel background,
  auction border tiers 1-4, gold particle loop, gold bloom loop, prism flash,
  and bid pulse ring loop. Validation passed: 18/18 files are PNG
  `Format32bppArgb` with alpha channel, dimensions match spec, transparent
  assets have valid alpha ranges, panel background is fully opaque RGBA,
  chroma-key fringe scan passed after cleanup, `git diff --check` passed, and
  `git diff --cached --check -- assets/art/ui/shop_auction assets/art/vfx/shop_auction`
  passed. Skipped: none; no filename suffixes needed. No audio, code, manifest,
  sprint-status, or session-state files were committed.
- Hand UI UX design returned and was pushed at `fc95a84`, creating
  `design/ux/hand-ui.md` only. Validation passed `git diff --check`; no code,
  production session-state, or sprint-status files were touched. Open questions
  captured: keyboard/focus scope vs Sprint 1 "no keybindings" GDD note, shallow
  fan row reconciliation between GDD fan formula and HUD/art readability
  guidance, missing `S2CActivationRejected`, reserve strip final width (`96px`
  GDD vs `104px` implementation evidence), and card zoom resolution / atlas
  sharing asset-pipeline questions. Blockers: Story 012 remains blocked by
  missing `S2CActivationRejected`; Story 013 keeps the reconnect timer-zero
  design question open. Unblocks hand/shop panel layout evidence and later Hand
  UI visual polish.
- Combat Log UX spec returned and was pushed at `959e38a`, creating
  `design/ux/combat-log.md` only. Validation passed: index verified as only
  `design/ux/combat-log.md`, `git diff --cached --check` passed, and
  `git diff --check` passed with only unrelated CRLF warnings. The follow-up
  orchestration tracking commit `89a8756` is also in `origin/main` history and
  recorded the COMBAT-011 approval blocker. No blocker remains for the combat
  log UX spec window.
- COMBAT-011 / S5-20 ResolutionEvent Log Completeness readiness repair/recheck
  returned READY. Initial repair was pushed at `92b5826`, updating
  `docs/architecture/tr-registry.yaml` and
  `production/epics/combat-resolution/story-011-resolution-event-log.md`:
  manifest `2026-05-01`, trace repaired to active `TR-CR-014` and `TR-CR-015`,
  `TR-CR-015` expanded to cover CR-32 content completeness, and story scope
  clarified as integration/protocol completion rather than verification-only.
  Follow-up commit `ce6a7aa` is also on `origin/main` and aligns event names to
  current protocol/story wording (`SubStepBegin` satisfying GDD SubStepEntry,
  `UnitRemoved` satisfying GDD UnitRemovedRecord). `/dev-story` is safe next
  with explicit authorization and must use `liv-bevy-018` plus
  `liv-bevy-lightyear` if it touches Bevy combat, shared protocol, Lightyear
  registration, or sending.
- COMBAT-010 / S5-19 Persistent Keyword States readiness returned BLOCKED on
  stale LEADER timing and trace/story gaps. Follow-up read-only analysis found
  the current Combat/Keyword GDDs authoritative: LEADER snapshot is post-SS1,
  before SS2, after all SS1 APPEARANCE effects resolve; ADR-018 and
  `docs/architecture/control-manifest.md` are stale derivatives from the older
  before-SS1 rule. This was already decided by OQ-KS9 and committed at
  `f8ceafd` on 2026-05-01. Required repair sequence before `/dev-story`:
  update ADR-018 Part 5 for post-SS1/pre-SS2 LEADER snapshot, update ADR-018
  Part 6 and the control manifest so OUTNUMBERED recomputes only after SS4
  `ChainDeathBuffer` drains, then repair Story 010 manifest/TR/LEADER timing,
  add OUTNUMBERED boundary/flip AC, add the ADR-017 `<= 15 ms` RESOLUTION
  budget note, and keep S2CResolutionEvent completeness in COMBAT-011. Docs-only
  repair returned READY and was pushed at `6326dc9`, updating ADR-017, ADR-018,
  the control manifest, TR registry, combat epic, and Story 010. `/dev-story` is
  safe next with explicit authorization.
- COMBAT-009 / S5-11 Objective Damage + GAME_OVER readiness repair returned
  READY and was pushed at `05aca93`, updating
  `production/epics/combat-resolution/story-009-objective-damage-gameover.md`.
  The repair updated manifest `2026-05-01`, fixed stale TR placeholder wording,
  aligned GAME_OVER ownership to current RSM/ResolutionComplete/economy ordering,
  refreshed event/log wording to current protocol names, expanded dependencies
  on completed COMBAT-007/008 and OBJECTIVE-005/006/007, and confirmed expected
  evidence path `tests/unit/combat/objective_damage_gameover_test.rs`. `/dev-story`
  is safe next with explicit authorization.
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
- COMBAT-011 implementation returned and was integrated to `main` at `73ad695`.
  No COMBAT-011 implementation worker remains active; queue serialized
  story-done.
- Full-game asset coverage audit, card art coverage audit, and UI/audio/VFX
  coverage audit have returned. The full-game asset manifest/spec expansion was
  committed at `bbde404`; no asset audit window remains active.
- HAND-UI-009, HAND-UI-008, AUC-006, BOARD-009, COMBAT-002, OBJECTIVE-004,
  BOARD-008, and HAND-UI-007 have returned and are integrated or closed as
  noted; do not relaunch their implementation workers.

Current active windows by user default-launch rule:
- COMBAT-009 readiness repair returned READY and was pushed at `05aca93`.
  Readiness window can be cleared. COMBAT-009 implementation worker returned at
  `16398c6`; root fast-forwarded and pushed `main` to that commit. Worker
  window can be cleared. Story-done returned COMPLETE and was pushed at
  `dc91402`, updating the story, active session state, and S5-11 sprint row.
  Story-done window can be cleared. COMBAT-010 readiness repair returned READY
  and was pushed at `6326dc9`; readiness window can be cleared. COMBAT-010
  implementation worker returned at `7ab44a1`; root fast-forwarded and pushed
  `main` to that commit. Worker window can be cleared. Story-done returned
  COMPLETE and was pushed at `7e0a213`, updating the story, active session
  state, and S5-19 sprint row. COMBAT-010 story-done window can be cleared.
  COMBAT-011 readiness repair/recheck returned READY and was pushed at
  `92b5826`, with follow-up trace-name alignment at `ce6a7aa`; readiness window
  can be cleared. COMBAT-011 `/dev-story` is now safe to launch with
  `liv-bevy-018` and `liv-bevy-lightyear`. COMBAT-011 implementation worker
  returned at `06d5b17`; root integrated and pushed it as `73ad695`. Worker
  window can be cleared. Queue a serialized `/story-done` for
  `production/epics/combat-resolution/story-011-resolution-event-log.md`.
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
- HAND-UI-010 prerequisite docs writer returned and pushed `935f090`; window can
  be cleared. Next safe work is readiness/dev sequencing for NP-005, ECO-007,
  and PRES-002; BLS-011 waits for NP-005 + ECO-007.
- PRES-002 readiness returned READY; readiness window can be cleared. PRES-002
  `/dev-story` is safe to launch now and remains a prerequisite for HAND-UI-010.
- ECO-007 readiness returned READY; readiness window can be cleared. ECO-007
  `/dev-story` is safe to launch now and remains a prerequisite for BLS-011 and
  HAND-UI-010.
- NP-005 readiness returned READY; readiness window can be cleared. NP-005
  `/dev-story` is safe to launch now and is a prerequisite for BLS-011 and
  HAND-UI-010.
- NP-005 implementation worker returned at `103d27d`; root integrated and
  pushed it as `e65fcfe` plus integration fix `48b4b5f`. Worker window can be
  cleared. NP-005 story-done returned and pushed closure at `705defa`; closure
  window can be cleared.
- ECO-007 implementation worker returned at `b1c678f`; root integrated and
  pushed it as `dacc7d3`. Worker window can be cleared. Queue serialized
  `/story-done` for
  `production/epics/economy-system/story-007-explicit-placement-mana-split-api.md`.
- PRES-002 implementation worker returned at `58afb3b`; root integrated and
  pushed it as `8587fa9` plus integration fix `e14feb6` for the NP-005 placement
  payload split test helper. Worker window can be cleared. Verification passed:
  affected client bundle 27/27, `cargo check -p client`, `cargo fmt -p client
  -- --check`, PRES commit-range `git diff --check`, and the grep guard showing
  exactly one production `MessageReceiver<S2CGoldUpdate>` drain in
  `client/src/presentation/shared/economy_view.rs`. Queue serialized
  `/story-done` for
  `production/epics/presentation-layer/story-002-shared-economy-view.md`.
- Shop/Auction UX readiness check returned; window can be cleared.
- Shop/Auction UX review finish returned NEEDS REVISION; window can be cleared
  after recording repair scope. Use `REPONDRE` only if launching the UX repair
  changeset.
- Shop/Auction UX repair returned and pushed `a3739db`; window can be cleared.
  SAU-005/006 are no longer blocked by the bid-confirmation conflict. SAU-009
  remains blocked by implementation/evidence prerequisites.
- SAU-003 readiness returned NEEDS WORK with no hard blockers. Window can be
  cleared after launching a docs-only repair for stale manifest `2026-05-01`,
  coarse/stale GDD trace, missing current control-manifest notes, missing
  UI/message-path performance note, and imprecise upstream dependency wording.
- SAU-002 readiness returned NEEDS WORK with no hard blockers. Window can be
  cleared after launching a docs-only repair for stale manifest `2026-05-01`,
  resolved tooltip UX scope, and missing current control-manifest notes for
  phase sink, `PlayerEconomyView`, single Lightyear drains, Bevy Required
  Components, and presentation performance budget.
- SHOP-AUCTION-UI-001 implementation/story-done windows and BOARD-RENDERING-002
  implementation/story-done windows can be cleared. Root committed and pushed
  both story-done closures together at `3222be0` after both official closure
  reports were received.
- Settings/Accessibility UX design returned and pushed `093b62a`; window can be
  cleared. Keep its open questions visible for future settings implementation.
- Result Screen UX design returned and pushed `399bb34`; window can be cleared.
  Keep its protocol/data blockers visible for future result/reconnect/rematch
  stories.
- Current card type/rarity HUD icon batch returned and pushed `43bd1f8`; window
  can be cleared.
- Current 8-card illustration batch returned and pushed `330806f`; window can be
  cleared.
- Remaining Board Rendering placeholder PNG batch returned and pushed `9fa1908`;
  window can be cleared.
- Shop/Auction UI PNG asset pack returned and pushed `efb95a1`; window can be
  cleared.
- Hand UI UX design returned and pushed `fc95a84`; window can be cleared.
- Combat Log UX design returned and pushed `959e38a`; window can be cleared.
- BOARD-RENDERING-003 readiness returned NEEDS WORK; readiness window can be
  cleared after launching a docs-only repair. Do not launch BOARD-003 dev-story
  until readiness returns READY.
- BOARD-RENDERING-003 readiness repair returned READY at `171997b`; repair
  window can be cleared. BOARD-003 `/dev-story` is safe to launch now.
- BOARD-RENDERING-004 readiness returned READY after docs repair `edfe87f`;
  readiness window can be cleared. BOARD-004 `/dev-story` is safe to launch.
- SAU-002 readiness returned NEEDS WORK; readiness window can be cleared after
  launching a docs-only repair. Do not launch SAU-002 dev-story until readiness
  returns READY.
- SAU-004 readiness returned NEEDS WORK; readiness window can be cleared after
  launching a docs-only repair. Do not launch SAU-004 dev-story until readiness
  returns READY.
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

## Live Orchestration Status

> Updated by the orchestrator after each window return. This is the source of
> truth for "what is running, what just closed, what is launchable next."
> Do NOT rely on conversation memory — always read this section first.

## Mandatory Skill Gates (enforced by user — never skip)

Every action in this project MUST use the Claude Game Studios defined skills:

| Step | Skill | When |
|---|---|---|
| Before every integration to main | `/code-review <files>` | After worker pushes branch |
| After every integration | `/story-done <story-file>` | Once on main |
| After a batch of stories | `/smoke-check` | Before close-out |
| Phase transitions | `/gate-check` | Never skip |
| After GDD edits | `/design-review` | Always |
| After cross-GDD changes | `/consistency-check` | Always |
| After ADR changes | `/architecture-review` | Always |

**Never substitute with:** raw bash/grep checks, manual file edits bypassing skills, skipping smoke-check, ad-hoc path existence checks.

### Last Sync: 2026-05-08 (reconnect/board fix wave + snapshot_sent root cause)

### CRITICAL CHAIN BLOCKING DRAFT_INITIAL
1. `defer_unicast_for_reconnect` always returns false → S2CDraftOffering silently discarded
2. Root cause: `initialise_reconnect_tracker` sets snapshot_sent=true for fresh players (should be false)
3. PROMPT 499 fixes the one-line bug — branch `work/snapshot-sent-fix` PUSHED
4. /code-review on reconnect.rs returned CHANGES REQUIRED (5 items unrelated to the fix line itself)
5. PROMPT 501 fixing the required items — IN FLIGHT
6. After 501 → re-run /code-review → if APPROVED → integrate snapshot_sent fix → DRAFT_INITIAL unblocked

### PROTOCOL ENFORCEMENT
Per `feedback_paw_review_flow.md`: NEVER merge until /code-review passes. Even for "one-line obvious" fixes. The protocol is strict and applies to all branches.

### ⚠️ DISK SPACE ALERT
D: drive hit 100% full from Rust build artifacts. Clean inactive worktree targets before launching new builds. Keep only active worktrees.

### origin/main HEAD: bd41df3

### Currently Running (windows still open)
- PROMPT 501 — reconnect-dedup-fix (server/src/core/session/reconnect.rs duplicate DeferredMessage variants)
- PROMPT 502 — paw-final-fixes (LobbyViewState dependency violation + BID_INCREMENTS to game_config.ron)
- PROMPT 504 — disk space cleanup (worktree targets)

### Worker branches PUSHED but BLOCKED on /code-review
- work/snapshot-sent-fix (PROMPT 499) — one-line snapshot_sent fix; /code-review CHANGES REQUIRED, blocked on PROMPT 501
- work/board-rendering-fixes-2 (PROMPT 500) — Pointer<Click> observers + 4 other fixes; needs /code-review
- work/hand-ui-node-lens-fix (PROMPT 493) — NodePositionLens + 4 other fixes; needs /code-review
- work/placeholder-assets-panic-fix (PROMPT 492) — already integrated as b92aa97 ✅
- work/paw-review-fixes (PROMPT 494) + work/paw-functional-fixes (PROMPT 497) — needs follow-up after 502 lands

### Closed Recently (this session)
- PAW-001 through PAW-006: all on main (cb4e178, 3734f19, bb80b47, f5b7a34, 2132129 etc.)
- ClientState dedup fix: f5b7a34 on main ✅
- Lightyear ReplicationSender fix: 14c937d on main ✅
- B0004 fan root + server tracing: 0cb0766 on main ✅
- Placeholder panic fix (PROMPT 492): branch pushed, NOT yet integrated — blocked by /code-review CHANGES REQUIRED (4 items, 3 being fixed by PROMPT 493)

### Current Work Flow
All PAW stories (002-006) are on main but have NOT had /story-done run yet.
All PAW files failed /code-review — fixes in PROMPT 493 + 494.
After both fix prompts complete:
  1. /code-review on fixed files
  2. If APPROVED → cherry-pick to main
  3. /story-done for each PAW-002 through PAW-006
  4. /smoke-check
  5. Rebuild + retest DRAFT_INITIAL (main fix: ClientState dedup f5b7a34 + ReplicationSender 14c937d)

### Pending /story-done (blocked until code-review passes)
- PAW-002: story-002-hand-ui-card-frames.md
- PAW-003: story-003-shop-auction-chrome.md
- PAW-004: story-004-hud-figurines-timer-dots.md
- PAW-005: story-005-board-unit-sprites.md
- PAW-006: story-006-lobby-portraits.md

### Active Blockers
- PROMPT 492 (placeholder panic fix): not integrated — 3 of 4 review items being fixed by PROMPT 493
- hand/mod.rs: CHANGES REQUIRED from /code-review — fix in PROMPT 493
- PAW-003 to PAW-006 + asset_wiring.rs: APPROVED WITH SUGGESTIONS (3 small required fixes) — lobby.rs:33 dead Startup to be fixed in PROMPT 494
- All /story-done for PAW-002 to PAW-006 blocked until /code-review passes on fixed files

### Closed Recently
- PROMPT 450 — DRAFT_INITIAL grid repair → worker commit `5b6d030` ✅ (incomplete fix — see 461 follow-up; worker forgot the BackgroundColor fallback I had requested)
- PROMPT 458 — cherry-pick 5b6d030 into main → integration commit `7431bdb` on origin/main ✅
- PROMPT 456 — S9-QA-002 story-done → main commit `10bfac9`, pushed to origin ✅
- PROMPT 457 — S9-AUDIO-001 audio bootstrap + timer urgency cue → worker commit `db7f1a9`, branch pushed ✅ (pending integration to main)

### Known Issues
- `hud_text_size_contrast_harness` test has a pre-existing rustc STATUS_STACK_BUFFER_OVERRUN crash on origin/main. Surfaced by PROMPT 457 verification. Unrelated to audio work. Investigate later if it blocks CI.

### Visual UI Implementation Gap (NEW — surfaced 2026-05-08)
Diagnostic confirms that beyond DRAFT_INITIAL slots, the entire game UI is in a raw/unstyled state:
- 0/242 design assets approved or wired into client
- HUD missing timer + class figurines + RESOLUTION dimming
- Hand UI logic complete but no visual chrome (frames, badges, panels)
- Shop/Auction UI missing all panel chrome (BLOCKER for visual playability)
- Lobby missing class carousel + portraits + slot indicators
- Stories were marked Complete on gameplay logic only — visual delivery never tracked as a separate axis

User decision pending on Option A (accept friend-game-lite raw UI), Option B (full visual polish sprint), or Option C (targeted asset wiring + shop/auction panels only).

### Sprint 9 Story Status
| Story | Status |
|---|---|
| S9-RS-001 | Done |
| S9-RS-002 | Done |
| S9-RS-003 | Done |
| S9-NATIVE-001 | Done with notes |
| SAU-008 | Done |
| S9-CONTENT-001 | Done (supporting) |
| S9-QA-002 | Done |
| S9-QA-001 | in-progress / partial — blocked by MANUAL-FG-001 (human operator GUI route) |
| S9-AUDIO-001 | worker pushed `db7f1a9`, pending integration to main |
| ECO-004 | conditional backlog — do not launch unless reward-loop issue surfaces |

### Active Blockers
- **MANUAL-FG-001** (S9-QA-001) — human operator must run two-client GAME_OVER route. Hybrid approach approved: PROMPT 459 covers automated E2E test; user does 5 manual screenshots; PROMPT 460 will consolidate into accepted-risk friend-game-lite QA evidence.

### Carried Conditions (do not close)
- S8-QA-001-W1 — open, full manual GAME_OVER route not captured
- QA-COND-0005 — accepted-risk (Standard-tier accessibility waived for friend-game)
- QA-COND-0006 — accepted-risk / deferred

### Next Available Prompt Numbers
- 460 — pending (consolidation of automated E2E + manual screenshots + accepted-risk evidence; emit when both 459 and screenshots are ready)
- 461+ — free

### Non-Claims Preserved
- no public release readiness, no release-candidate readiness, no full game completion
- no broad Standard-tier accessibility completion, no QA-COND-0005 closure
- no playtest/fun validation, no QA-COND-0006 closure
- no full playable-client manual QA, no full regression campaign
- no smoke/QA sign-off/gate-check/close-out unless explicitly run and evidenced

### Update Protocol for Orchestrator
After every window return:
1. Move the prompt from "Currently Running" to "Closed Recently" with its outcome commit hash.
2. Update Sprint 9 Story Status if the prompt closed/advanced a story.
3. Update Active Blockers if a blocker resolved or a new one surfaced.
4. Update Next Available Prompt Numbers when emitting a new prompt.
5. Bump "Last Sync" date.
6. Commit this file with message `state: update live orchestration status` whenever a non-trivial state change happens. Batch trivial changes — do not commit on every micro-return.
