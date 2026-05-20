# Codex Orchestrator State

## Current Handoff Snapshot (2026-05-20, pause after PROMPT-1539 launch)

This block supersedes earlier "current" snapshots until a later dated block is
added. The user requested no more agent launches for now and asked to save the
current orchestrator state plus generate a handoff document.

Source of truth:

- Root checkout: `D:\_DEV\Work\Claude-Code-Game-Studios`
- Root branch: `main`
- Root/source commit: `origin/main@78aa711b2a4a72aa4a93a8825e1a32682248eaa7`
  (`state: record 1537 bot lobby return`)
- Mainland queue: no pending/running entries at the handoff check.
- Root working tree caveat: `.claude/settings.json` is modified by dispatcher
  hook injection for the latest worker. Treat it as runtime churn; do not commit
  it unless intentionally updating dispatcher hooks.

Operational pause:

- Do not launch additional agents until the next orchestrator explicitly resumes
  the queue.
- Continue to use `project_id: "default"` for `gcs.dispatch`.
- Workers should use dedicated worktrees. Dispatcher may still report
  `workspaceMode: shared`; launch prompts must keep the explicit "create/switch
  to a dedicated worktree before editing" instruction.
- Cargo-heavy work remains isolated to VERIFY lanes. Do not make every
  implementation worker run Cargo.

Recently completed:

- `1518 AUCTION-WON-CARD-DISPOSITION-INTEGRATION-RECOVERY-VERIFY`:
  `ALREADY_LANDED`; report:
  `reports/PROMPT-1518-auction-won-card-disposition-integration-refresh-recovery.md`.
- `1518 AUCTION-WON-CARD-DISPOSITION-INTEGRATION-REFRESH`: duplicate rerun
  confirmed `DUPLICATE`; no action needed; report:
  `reports/PROMPT-1518-auction-won-card-disposition-integration-refresh-duplicate.md`.
- `1534 KROSMAGA-DEV-PROXY-PACK-MATERIALIZATION-STAGE2`: `SHIPPED` on branch
  `worker/prompt-1534-krosmaga-dev-proxy-stage2`, commit `f6988b34`, base
  `5358aed1`; integration refresh `1539` was launched.
- `1535 PLACEMENT-ACCEPTED-ACK-PROTOCOL-READINESS`: `NEEDS_WORK`; report:
  `reports/PROMPT-1535-placement-accepted-ack-protocol-readiness.md`. It
  recommends a serialized accepted-placement ACK implementation touching
  `shared/src/protocol.rs`, `server/src/feature/board/placement.rs`, and
  `client/src/ui/hand/mod.rs`.
- `1537 BOT-LOBBY-ADD-BOT-UI-WIRING`: `SHIPPED` and already landed; worker
  branch `worker/prompt-1537-bot-lobby-add-bot-ui`, commit `20abf970`;
  main carries return marker `78aa711b`; report path:
  `reports/PROMPT-1537-bot-lobby-add-bot-ui-wiring.md`.

Workers active or pending user DONE reports:

- No known active/pending worker windows remain from the pause ledger. Do not
  relaunch workers unless the next orchestrator explicitly resumes the queue.

Processed worker-return ledger (2026-05-20, no new agents launched):

| Prompt | Current state | Evidence | Written next step |
|---|---|---|---|
| `1518 AUCTION-WON-CARD-DISPOSITION-INTEGRATION-REFRESH` | DUPLICATE / already shipped. | User report: `reports/PROMPT-1518-auction-won-card-disposition-integration-refresh-duplicate.md`; worker branch `origin/worker/prompt-1513-auction-won-disposition` and original integration commit `f69bd595` are already ancestors of `origin/main@b531f499`. | Clear worker if open. No relaunch and no mainland enqueue needed. |
| `1509 HU-CHROME-02-HAND-FAN-REPAIR-INTEGRATION` | READY_FOR_MAINLAND_ENQUEUE reported after ledger update. | User reported full report at `reports/PROMPT-1509-hu-chrome-02-hand-fan-repair-integration.md`; prior mainland queue also shows `origin/integrate/hu-chrome-02-hand-fan-repair-1509` landed `1172e165..5d46b9a9`, and follow-up `1519` is GREEN/SHIPPED. | Clear worker after noting. Before any mainland enqueue, inspect report/branch against current `origin/main` to avoid duplicate or non-FF enqueue if content is already present. Do not relaunch. |
| `1530 SHOP-AUCTION-CARD-INSPECT-CONSUMER-WIRING` | SHIPPED on worker branch. | User report: branch `worker/prompt-1530-shop-auction-card-inspect-consumer-wiring` @ `b3743828`, base `5358aed1`; report in worker tree `reports/PROMPT-1530-shop-auction-card-inspect-consumer-wiring.md`; 3/3 unit tests pass. | Clear worker, then integration-refresh over current `origin/main` before mainland enqueue. |
| `1531 BOT-PARTICIPANT-ACTION-LOOP-WAVE1` | SHIPPED on worker branch. | User report and worker report `reports/PROMPT-1531-bot-participant-action-loop-wave1.md`; report says server bot action-loop tests 7/7 pass and broad live QA deferred. | Clear worker, inspect exact branch/commit from report or worktree, then integration-refresh over current `origin/main`. Do not overlap with bot UI/protocol owners without checking files. |
| `1532 RESOLUTION-REPLAY-VISUAL-MUTATION-FOLLOWUP` | SHIPPED on worker branch. | User report and worker report `reports/PROMPT-1532-resolution-replay-visual-mutation-followup.md`; client-only presentation mutations added. | Clear worker, integration-refresh over current `origin/main`, then schedule focused verify/live replay QA after landing. |
| `1533 QA-SNAPSHOT-OBSERVABILITY-FIELDS-FOLLOWUP` | SHIPPED on worker branch. | User report: branch `prompt-1533-qa-snapshot-observability-fields-followup` @ `4c3ece2c`, base `5358aed1`; report `reports/PROMPT-1533-qa-snapshot-observability-fields-followup.md`; snapshot field coverage test 15/15 pass. | Clear worker, integration-refresh over current `origin/main`; watch for conflicts with accepted-ACK fields from future `1535` implementation. |
| `1535 PLACEMENT-ACCEPTED-ACK-PROTOCOL-READINESS` | NEEDS_WORK readiness outcome; no implementation done. | Report on main: `reports/PROMPT-1535-placement-accepted-ack-protocol-readiness.md`. It proves accepted ACK is missing and recommends a serialized protocol/server/client implementation. | Clear worker if open. Later launch one implementation prompt only when `shared/src/protocol.rs`, `server/src/feature/board/placement.rs`, and `client/src/ui/hand/mod.rs` are free. |
| `1536 POST-1528-FOCUSED-VERIFY-LANE` | PARTIAL verify lane. | User report and worker report `reports/PROMPT-1536-post-1528-focused-verify-lane.md`. Report recommends two follow-ups: auction test serial-lock leak and hand inspect optional input resource. | Clear worker. Do not launch follow-ups during pause; next orchestrator should triage recommended fixes after current implementation branches are integrated or isolated. |
| `1537 BOT-LOBBY-ADD-BOT-UI-WIRING` | SHIPPED / already landed. | User reported worker branch `worker/prompt-1537-bot-lobby-add-bot-ui` commit `20abf970` pushed, already landed, with main return marker `78aa711b`; full report at `reports/PROMPT-1537-bot-lobby-add-bot-ui-wiring.md`. | Clear worker idempotently if still open. No integration-refresh or mainland enqueue needed unless later audit proves missing content. |
| `1538 RESULT-MULLIGAN-KROSMAGA-CHROME-POLISH` | SHIPPED on worker branch. | Worker report `reports/PROMPT-1538-result-mulligan-krosmaga-chrome-polish.md`; branch `worker/prompt-1538-result-mulligan-krosmaga-chrome-polish`, commit `93cfb255`, based on `38975b51`; 25/25 focused tests pass. | Clear worker, then integration-refresh over current `origin/main`; no mulligan module exists yet, so future mulligan-specific work remains deferred. |
| `1539 KROSMAGA-DEV-PROXY-STAGE2-INTEGRATION-REFRESH` | SHIPPED. | User report: `reports/PROMPT-1539-krosmaga-dev-proxy-stage2-integration-refresh.md`. | Clear worker, inspect integration branch/commit from report, then enqueue mainland only if FF-ready; otherwise refresh again. |

Handoff document:

- Current handoff file generated with the `handoff` skill:
  `D:\Tmp\GCS_ORCHESTRATOR_HANDOFF_2026-05-20.md`

Non-claims:

- No sprint close-out, release readiness, full-game-complete, final-art/legal
  clearance, accessibility completion, `S8-QA-001-W1` closure, or
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` closure is implied by this snapshot.

## Current Snapshot (2026-05-20, post-PROMPT-1528 mainland)

This block is the current restart/orchestration snapshot. It supersedes stale
historical "current source of truth" text below unless a later dated block
replaces it.

Source of truth:

- `origin/main@5358aed1a6075aca621936fd14f561be8fb854d3`
  (`PROMPT-1528 resolution-replay-mutation integration refresh` plus the
  post-1528 orchestrator-state refresh). The root checkout is expected to stay
  on local `main` fast-forwarded to this ref.
- Active stage remains Polish. Do not claim release readiness, full-game
  completion, final-art/legal clearance, accessibility completion, playtest
  validation, `S8-QA-001-W1` closure, or `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  closure from this snapshot.

Verified since the older 1472 planning snapshot:

- `1472 POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST`: PASS on
  `origin/main@56b5fc0c`. Verified lobby, class confirm, draft offers,
  shop slots, placement drag/drop with retry, submit acceptance, unit visible
  in Resolution, next draft, auction card/bid/settlement, QA snapshot and grid
  toggle evidence.
- `1473 DEV-LAUNCHER-CANONICAL-TWO-CLIENT-VERIFY`: PASS. Launcher rebuilt
  latest main, started server plus two clients from the canonical launcher
  workflow, wrote build provenance, and no launcher UI cropping was observed.
- `1475 KROSMAGA-UI-IMPLEMENTATION-LANE-MAP`: COMPLETE. Krosmaga-style work
  was split into file-owned UI lanes; dev-proxy assets remain reference/dev-only.
- `1476 MULTIPLAYER-PHASE-STATE-SYNC-GAP-AUDIT`: COMPLETE. Main remaining
  state risks are explicit placement-accepted ACK, accepted-unit visibility
  reliance on reveal/snapshot reconstruction, timer freshness, shop-offer
  recovery, and auction settlement payload completeness.
- `1477 RESOLUTION-COMBAT-LOOP-VISUAL-STATE-AUDIT`: COMPLETE. Server
  resolution flow is materially implemented, but client replay still needs
  visible mutation of movement, HP, removals, objective damage/destruction,
  gold rewards, and event comprehension.
- `1478 QA-SNAPSHOT-OBSERVABILITY-COMPLETENESS-AUDIT`: COMPLETE. Snapshot
  state is strong for local UI, shop, auction, placement intent, rendered board
  entities, and overlays; still weak for server-authoritative theoretical UI,
  real placement ACK lifecycle, precise pointer/focus, semantic rendered-label
  roles, image/card-art diagnostics, and pixel-level occlusion.
- `1479 KROSMAGA-ASSET-BINDING-ROLLOUT-PLAN`: COMPLETE. Asset path is staged:
  logical IDs -> dev-pack materialization tooling -> surface binding lanes ->
  original replacement. No Krosmaga payload may be copied into release assets.
- `1480 CARGO-VERIFY-LANE-SCHEDULER`: COMPLETE. Cargo-heavy checks stay in
  dedicated VERIFY lanes with max shared-target concurrency 1.
- `1519 POST-1511-HU-CHROME-VERIFY`: SHIPPED/GREEN. The HU-CHROME-02 gate is
  clear on main-line code; PROMPT 1510's fail was stale worktree/cache state,
  not current main.

Landed since the 1472 planning snapshot:

- `1518 AUCTION-WON-CARD-DISPOSITION-INTEGRATION-REFRESH`: MAIN-LANDED at
  `f69bd595`. Main now includes wire-authoritative
  `S2CAuctionSettled.card_id` and client use of the wire settlement card id
  instead of stale local auction state.
- `1523 CARD-INSPECT-HAND-DRAFT-INTEGRATION-REFRESH`: MAIN-LANDED at
  `a51f0ac7`. Main now includes right-click card inspect wiring for hand fan
  and DraftInitial grid surfaces; shop/auction inspect wiring remains a
  separate follow-up.
- `1526 BOT-ROOM-JOIN-LOOP-INTEGRATION-REFRESH-AFTER-1523`: MAIN-LANDED at
  `b82a341d`. Main now includes deterministic bot lobby class auto-confirm so
  bot rooms can advance past lobby once the human confirms.
- `1528 RESOLUTION-REPLAY-MUTATION-INTEGRATION-REFRESH-AFTER-1526`:
  MAIN-LANDED at `c3d847f1`. Main now includes the partial client replay
  cadence slice: resolution feedback applies at active AnimGroup cadence rather
  than intake-time burst.

Completed no-follow-up audits:

- `1515 CARD-COST-COMBAT-STAT-RENDERING-RE-AUDIT`: REDUNDANT.
- `1516 HUD-GHOST-GLYPH-LEGIBILITY-RE-AUDIT`: OBSOLETE.
- `1517 DEV-LAUNCHER-BUILD-PROVENANCE-MAINLAND-REFRESH`: NO-OP.

Current launch guidance:

- Default worker launch mode is `gcs.dispatch` with `workspace_mode: "worktree"`
  for implementation, integration, verify, audit, and report work. Use the
  shared/root checkout only when explicitly required; it must remain on `main`
  as the orchestrator source-of-truth workspace.
- 1472 is done; code repairs no longer need to wait for that old gate.
- HU-CHROME is green; prompts previously gated by 1506/1510 can proceed if
  file ownership is disjoint from active workers.
- Shop/auction is no longer held by 1518. The next useful shop/auction UI lane
  is inspect/consumer wiring and/or Krosmaga-style card product continuation,
  but keep it file-owned and separate from other `shop_auction` changes.
- Shared protocol is no longer held by 1518. New protocol changes still need a
  dedicated owner and should be serialized by branch/file ownership.
- Bot local/single-player remains deferred. Prioritize bot-as-room-participant:
  draft pick loop, placement failsafe, and auction pass.
- Krosmaga art remains dev-proxy/reference only. Continue implementation as
  Krosmaga-inspired hierarchy, pacing, readability, and layout; do not make
  release claims from Krosmaga proxy assets.
- Broad Cargo is still not part of implementation prompts. Use targeted VERIFY
  lanes and serialize shared-target Cargo pressure.

Current condensed task board:

- P0 gameplay: follow up the resolution replay partial slice with client-only
  visual mutations for unit movement/lane change, placed-unit dedupe,
  objective HP/destruction, gold-awarded HUD fanout, spawn-range/phase handoff
  coverage, and replay author/debug notes; then run the next live two-client
  multi-round retest.
- P0 multiplayer/state: explicit placement-accepted ACK remains a candidate if
  live evidence shows ambiguity; snapshot still needs real placement ACK
  lifecycle and richer pointer/semantic-label/image diagnostics.
- P1 bot: launch bot draft pick loop, bot placement failsafe, and bot auction
  pass as separate server-owned lanes now that lobby class auto-confirm is on
  main.
- P1 UI/Krosmaga: shop/auction card inspect consumer wiring remains open;
  continue board/objective physicality, HUD/lobby/result polish, and dev-proxy
  binding lanes guarded by provenance.
- P1 QA/observability: add missing snapshot fields from 1478 only as dedicated
  observability lanes; keep image-by-image forensic QA as the standard for
  human snapshot analysis.
- P2 sprint/paperwork: Sprint 18 story-done/smoke/team-QA/closeout remain
  serialized and should wait until active implementation branches are integrated
  or explicitly accepted as carried conditions.

## Current Operating Rules (2026-05-13 override)

This section is the current GCS orchestrator contract. It supersedes older
prompt-formatting, delimiter, close-out, and parallelism notes later in this
file. Later dated snapshots are historical unless they explicitly replace this
section.

Prompt display format correction (2026-05-15):

- This correction is authoritative for compaction/resume: after context
  compacts, re-read this section and ignore older triangle/fence/hash prompt
  snapshots below unless a newer dated rule explicitly replaces this one.
- Do not wrap worker launch prompts in 4-backtick fences, triangle headers,
  triangle closers, hash delimiters, or copied template wrappers.
- Put one plain disposition label directly above each action:
  - `🟢 CLEAR -- PROMPT N` for a worker window the user can close.
  - `🟡 REPONDRE -- PROMPT N` for text to send back to the same worker.
  - `🔴 RELANCER -- PROMPT N` for a corrected rerun/repair.
  - `🟣 NEW -- PROMPT N` for a new worker launch.
- For CLEAR, write one short human sentence below the label explaining what can
  be closed.
- For NEW/RELANCER/REPONDRE, put the worker prompt body immediately below the
  label. The first body line is `PROMPT N -- Task Title`.
- Use plain Markdown in the body; keep paths/commands in backticks where useful.
- The worker's final instruction remains one visible line:
  `N: TICKET-ID: STATUS`. STATUS is replaced by the real worker outcome, never
  hardcoded by the orchestrator, never `GREEN`/`YELLOW`, and no line follows it.

Current source of truth:

- **Authoritative correction for this header (PROMPT 798 update)**: current
  `origin/main` is `5029259` at PROMPT 798 entry (PROMPT 794-era docs commit
  `docs(octogent): RELANCER+PROMPT pairing, idle-trigger patch, slug-in-filename reports`
  on top of PROMPT 796 Sprint 12 Must Have story integration `487be6d`);
  PROMPT 798 will push one paperwork commit on top with the Sprint 12
  activation. PROMPT 798 promoted Sprint 12 from the `next_sprint:` draft
  block (PROMPT 793) to the top-level active sprint row in
  `production/sprint-status.yaml`: flipped `sprint: 11` -> `sprint: 12`,
  `status: closed-with-conditions` -> `status: active`, `start: 2026-06-04`
  -> `start: 2026-06-18`, `end: 2026-06-17` -> `end: 2026-07-01`; rewrote
  `goal:` / `scope:` for Sprint 12; rewrote `activation:` block for
  PROMPT 798; appended `carried_into_sprint_12:` block after
  `previous_sprint_closeout:`; replaced the Sprint 11 `stories:` block with
  the Sprint 12 `stories:` block (5 Must Have rows marked `ready` on the
  basis of PROMPT 794 READY for story 019 and PROMPT 797 PASS-WITH-NOTES /
  structurally READY for stories 012 / 013 / 014 / 015, all carrying the
  explicit blocker note that Sprint 12 QA plan is required before
  `/dev-story`; 4 Should Have rows marked `blocked` pending story files;
  5 Nice to Have rows marked `blocked` pending story files); removed the
  `next_sprint:` draft block (now superseded); appended a
  `sprint_12_activation:` block at end of file. Prepended an ACTIVATED
  banner to `production/sprints/sprint-12.md` above the PROMPT 793 DRAFT
  body. Verdict **PASS** — activation succeeds with the explicit
  precondition that Sprint 12 QA plan is still pending and required before
  `/dev-story`, all carried conditions are preserved unchanged, and no
  release / accessibility / playtest / manual-QA / final-art /
  `S8-QA-001-W1` closure is claimed. Sprint 11 disposition UNCHANGED
  (`closed-with-conditions` per PROMPT 792). Sprint 10 disposition
  UNCHANGED (`closed-with-conditions` per PROMPT 763). Stage UNCHANGED
  (`Polish`). PROMPT 761 Polish->Release gate-check `FAIL` preserved (no
  retry). `production/stage.txt` reads `Polish` and was NOT modified by
  PROMPT 798. `.claude/settings.json` working-tree modification preserved
  untouched. No public release readiness claim, no release-candidate
  readiness claim, no full-game-completion claim, no broad / Standard-tier
  accessibility-completion claim, no playtest / fun-hypothesis-validation
  claim, no full playable-client manual-QA claim, no final-art /
  asset-production claim, no `S8-QA-001-W1` closure, no Polish->Release
  retry, no stage advance from Polish to Release is authorised by this
  activation. (Prior: PROMPT 793 update — current
  `origin/main` was `8a8451e` at PROMPT 793 entry (PROMPT 792
  `close-out(s11): Sprint 11 close-out disposition PASS-WITH-CONDITIONS`);
  PROMPT 793 will push one paperwork commit on top with the Sprint 12
  draft plan. PROMPT 793 authored `production/sprints/sprint-12.md` (NEW
  Sprint 12 draft plan) and appended a `next_sprint:` draft block to
  `production/sprint-status.yaml`. **Sprint 12 is NOT activated by this
  draft**; activation happens via `/sprint-plan sprint-12` in a separate
  prompt. The Sprint 12 draft pulls forward (a) the Sprint 11 close-out
  deferrals (4 Should Have + 6 Nice to Have rows from
  `sprint_11_closeout.deferred_into_sprint_12_planning`), (b) the 5
  Cluster B retained D-5 ignored tests from
  `production/qa/evidence/sprint-11-ignored-d5-triage.md` (B1 board
  `GhostDragStartEvent` producer fixture gap, B2 HUD `snapshot.phase`
  bridge, B3 lobby `ConfirmClass` after `SelectClass` intent chain,
  B4 `co_occupancy_offset` panic-guard drift, B5 `ShopAuctionUiEntity`
  count drift), and (c) the follow-on diagnostic story 019
  (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`;
  on `main` at `0fc05c3`). Verdict **PASS-WITH-NOTES** — draft succeeds
  with the explicit non-activation precondition; producer review
  required before `/sprint-plan sprint-12` activation. Sprint 11
  disposition UNCHANGED (`closed-with-conditions` per PROMPT 792).
  Stage UNCHANGED (`Polish`). PROMPT 761 Polish->Release gate-check
  `FAIL` preserved (no retry). The 5 Cluster B retained D-5 ignored
  tests remain open as documented Sprint 12+ follow-ups / decision
  gates. No release claim, no release-candidate claim, no full-game
  claim, no broad / Standard-tier accessibility-completion claim, no
  playtest / fun-hypothesis validation claim, no full playable-client
  manual-QA claim, no final-art / asset-production claim, no
  `S8-QA-001-W1` closure, no Polish->Release retry, no Sprint 12
  activation is authorised by this draft. (Prior: PROMPT 792 update —
  current `origin/main` was `d19ea12` at PROMPT 792 entry (PROMPT 791
  `qa(team): Sprint 11 QA sign-off`); PROMPT 792 pushed one paperwork
  commit on top with the Sprint 11 close-out disposition.) PROMPT 792 flipped
  Sprint 11 top-level `status` in `production/sprint-status.yaml` from
  `active` to **`closed-with-conditions`** with verdict
  **PASS-WITH-CONDITIONS** on basis of 6/6 Must Have `done` + Sprint 11
  smoke `PASS-WITH-WARNINGS` (PROMPT 790, `1617352`) + Sprint 11 Team-QA
  `PASS-WITH-WARNINGS / APPROVED WITH CONDITIONS` (PROMPT 791, `d19ea12`).
  Should Have rows (4/4) and Nice to Have rows (6/6) remained `blocked`
  (no story files / no `/story-readiness`) and were **explicitly deferred**
  forward to Sprint 12+ planning by PROMPT 792 — none silently dropped, no
  new scope pulled in. Stage UNCHANGED (`Polish`). PROMPT 761
  Polish->Release gate-check `FAIL` preserved (no retry). The 5 Cluster B
  retained D-5 ignored tests remain open as documented Sprint 12+
  follow-ups / decision gates per
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`. Sprint 11 is
  now **closed-with-conditions**; no release claim, no
  release-candidate claim, no full-game claim, no broad / Standard-tier
  accessibility-completion claim, no playtest / fun-hypothesis validation
  claim, no full playable-client manual-QA claim, no final-art /
  asset-production claim, no `S8-QA-001-W1` closure, no Polish->Release
  retry is authorised by this close-out.
- Story and sprint status: `production/sprint-status.yaml`.
- Stage: `production/stage.txt`.
- Coordination memory: this file, using the latest dated block plus this
  override.
- Current verified state at this update: `origin/main@5029259` at
  PROMPT 798 entry (PROMPT 794-era docs commit
  `docs(octogent): RELANCER+PROMPT pairing, idle-trigger patch, slug-in-filename reports`
  on top of PROMPT 796 Sprint 12 Must Have story integration `487be6d`);
  PROMPT 798 will push one paperwork commit on top with the Sprint 12
  activation (`production/sprint-status.yaml` rewritten — top-level
  `sprint: 12`, `status: active`; Sprint 12 `stories:` block with 5
  Must Have `ready` + 4 Should Have `blocked` + 5 Nice to Have `blocked`
  rows; `next_sprint:` draft block removed; `sprint_12_activation:`
  block appended at end of file — plus `production/sprints/sprint-12.md`
  ACTIVATED banner prepended above the PROMPT 793 DRAFT body, plus
  `production/session-state/active.md` PROMPT 798 banner prepended,
  plus this `codex-orchestrator-state.md` update). Sprint 12 disposition
  **CHANGED**: `active` (Polish-stage; activated by PROMPT 798 with the
  precondition that the Sprint 12 QA plan must be authored via
  `/qa-plan sprint` before any `/dev-story` on the 5 Must Have rows;
  `/qa-plan sprint-12` is the next required prompt).
  Historical PROMPT 793 entry follows: `origin/main@8a8451e` at PROMPT
  793 entry (PROMPT 792 Sprint 11 close-out commit
  `close-out(s11): Sprint 11 close-out disposition PASS-WITH-CONDITIONS`);
  PROMPT 793 pushed one paperwork commit on top with the Sprint 12
  draft plan (`production/sprints/sprint-12.md` NEW + `next_sprint:`
  draft block appended to `production/sprint-status.yaml`).
  Sprint 12 historical disposition at PROMPT 793 entry: `draft`
  (Polish-stage; NOT activated by PROMPT 793; activation
  via `/sprint-plan sprint-12` happened in PROMPT 798). Sprint 12 draft
  Must Have rows (5): `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001` (story
  019 follow-on), `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001` (B2),
  `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001` (B3),
  `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001` (B4),
  `S11-TD-FIXTURE-D-RESIDUALS-001` (B1 + B5 umbrella). Sprint 12 draft
  Should Have rows (4): `S11-HUD-TIMER-EYEBALL-VISUAL-001` (W2 carry),
  `S11-HU-PHASE-IDEMPOTENCY-001`, `S11-SERVER-POOL-INIT-LOG-GUARD-001`,
  `S11-LOBBY-UX-CONFIRM-STATE-001` (promoted from Sprint 11 Nice to
  Have to batch with B3). Sprint 12 draft Nice to Have rows (5):
  `S11-TD-CARGO-DISK-USAGE-001`, `S11-TD-CARGO-PDB-LIMIT-001`,
  `S11-OPS-ORCHESTRATOR-LOCK-001`, `S11-OPS-GH-CLI-001`,
  `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`. Optional split
  candidates: `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` (B1
  split), `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` (B5 split).
  Sprint 11 disposition UNCHANGED (`closed-with-conditions` per PROMPT
  792). Sprint 11 `stories:` block UNCHANGED. Stage UNCHANGED
  (`Polish`). PROMPT 761 Polish->Release `FAIL` preserved (no retry).
  PROMPT 793 did NOT run `/dev-story`, `/story-readiness`,
  `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, `/qa-plan`. PROMPT 793 did NOT modify production
  code under `client/` / `server/` / `shared/` / `tests/`. PROMPT 793
  did NOT modify `production/stage.txt`, `.claude/settings.json`
  (pre-existing in-tree modification preserved untouched),
  `production/sprints/sprint-11.md`,
  `production/qa/qa-plan-sprint-11.md`,
  `production/qa/smoke-sprint-11-2026-05-13.md`,
  `production/qa/team-qa-sprint-11-2026-05-13.md`,
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`,
  `production/qa/evidence/sprint-11-drag-runtime-evidence.md`,
  `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`,
  `production/gate-checks/gate-polish-release-2026-05-12.md`, or
  `.octogent/`. No release claim. No release-candidate claim. No
  full-game-completion claim. No broad / Standard-tier
  accessibility-completion claim. No playtest / fun-hypothesis
  validation claim. No full playable-client manual-QA claim. No
  final-art / asset-production-completion claim. No `S8-QA-001-W1`
  closure. No Polish->Release retry. No Sprint 12 activation. Sprint
  12 close-out paperwork (when later authored by `/sprint-plan
  sprint-12`) will use `production/sprints/sprint-12.md` as the plan
  and the `next_sprint:` draft block as the source for the active
  `stories:` rows. (Prior: PROMPT 792 update — current `origin/main`
  was `d19ea12` at PROMPT 792 entry (PROMPT 791 Team-QA sign-off
  commit `qa(team): Sprint 11 QA sign-off`); PROMPT 792 pushed one
  paperwork commit on top with the Sprint 11 close-out disposition.
  Sprint 11 disposition **CHANGED** by PROMPT 792 (paperwork-only):
  flipped from `active` to **`closed-with-conditions`** (Polish-stage);
  6/6 Must Have rows `done`; Should Have (4/4 blocked) and Nice to Have
  (6/6 blocked) explicitly deferred forward to Sprint 12+ planning;
  Cluster B 5 retained D-5 ignored tests carried to Sprint 12+ backlog
  with named follow-ups per `production/qa/evidence/sprint-11-ignored-d5-triage.md`.
  PROMPT 792 did NOT run `/gate-check`, did NOT rerun smoke, did NOT run
  `/release-check`, did NOT run `/dev-story` / `/story-done` /
  `/story-readiness` / `/team-qa`. No release claim. No release-candidate
  claim. No full-game-completion claim. No broad / Standard-tier
  accessibility-completion claim. No playtest / fun-hypothesis validation
  claim. No full playable-client manual-QA claim. No final-art /
  asset-production claim. No `S8-QA-001-W1` closure. No retry of the
  Polish->Release gate-check. Sprint 11 close-out paperwork recorded
  under `sprint_11_closeout:` block in `production/sprint-status.yaml`
  (appended by PROMPT 792). Stage remains `Polish`,
  Sprint 10 `closed-with-conditions` per PROMPT 763 (2026-05-13), Sprint 11
  status `active` (PROMPT 773, 2026-05-13) as a Polish-stage sprint
  (`2026-06-04 -> 2026-06-17`) with plan at `production/sprints/sprint-11.md`
  and Sprint 11 QA plan on `main` at `production/qa/qa-plan-sprint-11.md`
  (PROMPT 774, 2026-05-13). PROMPT 761 `Polish->Release` gate-check `FAIL`
  preserved as evidence (no retry attempted). PROMPT 762 Sprint 11 candidate
  backlog capture folded into the Sprint 11 plan. Sprint 11 Must Have
  paperwork-carry deliverables landed on `main` (`0d19690` / `348084b` /
  `d3ee8df`); `/story-done` ran for `S11-DOC-HYGIENE-CARRY-001` in PROMPT
  780 (2026-05-13), for `S11-EVIDENCE-INDEX-CARRY-001` in PROMPT 781
  (2026-05-13), for `S11-DRAG-RUNTIME-RETEST-001` in PROMPT 783
  (2026-05-13), for `S11-TD-FIXTURE-HAND-UI-ONENTER-001` in PROMPT 785
  (2026-05-13), for `S11-ROUTE-READABILITY-CARRY-001` in PROMPT 786
  (2026-05-13), and for `S11-TD-IGNORED-D5-TRIAGE-001` in
  **PROMPT 789 (2026-05-13)**, flipping all six Sprint 11 Must Have rows
  from `ready` to `done`. PROMPT 778 /dev-story (2026-05-13) authored the
  drag-runtime evidence + follow-on diagnostic story at worker commit
  `0fc05c3` with disposition `PASS-CANNOT-REPRODUCE`; PROMPT 782
  (2026-05-13) integrated the worker to `main` at merge commit `3ca1aff`.
  PROMPT 779 /dev-story (2026-05-13) authored the Hand UI OnEnter
  fixture-cascade repair at worker branch
  `work/s11-hand-ui-onenter-fixture-repair`; PROMPT 784 (2026-05-13)
  integrated the worker to `main` at commit `d7f4103` (+1129 passed /
  -6 ignored at worker workspace; +390 passed / 0 failed / 5 ignored at
  PROMPT 784 client-crate verification). PROMPT 787 (2026-05-13) authored
  the read-only D-5 `#[ignore]` triage evidence
  `production/qa/evidence/sprint-11-ignored-d5-triage.md` (185 lines, 11/11
  accounted, 6 resolved + 5 retained); PROMPT 788 (2026-05-13) integrated
  the worker evidence to `main` at commit `1d96281`. All six Sprint 11
  Must Have rows are now `done`; the 5 retained Cluster B ignored tests
  (board `GhostDragStartEvent` producer fixture gap, HUD `snapshot.phase`
  bridge fixture gap, lobby `ConfirmClass` after `SelectClass` intent
  chain, `co_occupancy_offset` panic-guard drift, `ShopAuctionUiEntity`
  count drift) remain open as future stories or decision gates per the
  triage evidence; closing Sprint 11 is a separate orchestrator
  decision. The follow-on diagnostic story
  `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  is on `main` at `0fc05c3` but not yet activated into Sprint 11 active
  scope (separate `/sprint-plan sprint-11 --add story-019` prompt
  required).

Current next move:

- **Current authoritative next move (2026-05-18 launcher sidecar follow-up)**:
  PROMPT 1173 is the active official integration refresh for PROMPT 1170's
  Windows launcher repo-root sidecar repair. The user found a runtime parser
  issue before main-land: a BOM-prefixed sidecar comment is being parsed as
  the repo-root path. A direct `REPONDRE PROMPT-1173` has been sent requiring
  the BOM/comment parser fix and test before PASS. Wait for PROMPT 1173; if it
  returns a corrected integration branch, launch the serialized main-land
  prompt next. Do not use Octogent as truth, do not main-land the raw 1170
  branch, and do not use the root checkout for unrelated writes while it is
  still on the 1170 worker branch.

- **Authoritative next-move correction after PROMPT 793**: Sprint 12
  draft plan is now **AUTHORED** at `production/sprints/sprint-12.md`
  with a `next_sprint:` draft block appended to
  `production/sprint-status.yaml`. Sprint 12 is **NOT activated** by
  this draft; activation is a separate `/sprint-plan sprint-12`
  prompt that will write the active `stories:` rows. The primary
  next launchable prompts are: (1) `/sprint-plan sprint-12` to
  activate Sprint 12 (producer review of the draft required first);
  (2) story-file authoring + `/story-readiness` for the 4 new
  Cluster B Must Haves (`S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`,
  `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`,
  `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`,
  `S11-TD-FIXTURE-D-RESIDUALS-001`) + 4 Should Haves + 5 Nice to
  Haves; (3) `/story-readiness` on the existing story 019
  (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`;
  on `main` at `0fc05c3` but in `Draft` status); (4) `/qa-plan
  sprint-12` after story files exist and pass `/story-readiness`;
  (5) NO Polish->Release retry — preserved `FAIL` at
  `production/gate-checks/gate-polish-release-2026-05-12.md`.
  Sprint 11 disposition UNCHANGED (`closed-with-conditions` per
  PROMPT 792). Stage UNCHANGED (`Polish`). (Prior: after PROMPT 792
  — Sprint 11 close-out disposition is **DONE** —
  `production/sprint-status.yaml` top-level `status` flipped from `active`
  to **`closed-with-conditions`** with a `sprint_11_closeout:` block
  appended; Should/Nice rows deferred forward; carried conditions and
  non-claims preserved. The primary next launchable prompt is
  `/sprint-plan sprint-12` to open Sprint 12 planning and pull forward
  the deferred Should/Nice rows + Cluster B follow-up slugs + follow-on
  diagnostic `story-019` (currently on `main` at `0fc05c3` but not
  activated). Alternative next moves: author story files +
  `/story-readiness` for any deferred row (a precondition for
  activating it in Sprint 12). Do NOT retry `Polish->Release` —
  release-scope artifacts (final art, manual-QA sign-off, accessibility
  completion, playtest evidence) do not yet exist on `main`; PROMPT
  761 Polish->Release `FAIL` preserved.)
- Sprint 10 close-out paperwork is DONE (PROMPT 763). Sprint 10 disposition
  preserved at `production/sprint-status.yaml` `sprint_10_closeout:` block.
- Sprint 11 is ACTIVE as of PROMPT 773 (2026-05-13) as a Polish-stage
  sprint (`2026-06-04 -> 2026-06-17`). See `production/sprints/sprint-11.md`
  and `production/sprint-status.yaml` `sprint_11_activation:` block plus the
  `stories:` block (16 Sprint 11 rows). All six Sprint 11 Must Have rows
  (`S11-DOC-HYGIENE-CARRY-001`, `S11-EVIDENCE-INDEX-CARRY-001`,
  `S11-DRAG-RUNTIME-RETEST-001`, `S11-TD-FIXTURE-HAND-UI-ONENTER-001`,
  `S11-ROUTE-READABILITY-CARRY-001`, `S11-TD-IGNORED-D5-TRIAGE-001`)
  are now closed (`done`) by PROMPTs 780 / 781 / 783 / 785 / 786 / 789
  respectively. **PROMPT 789 does NOT close Sprint 11** — the 5
  retained Cluster B ignored tests remain open as future stories or
  decision gates per `production/qa/evidence/sprint-11-ignored-d5-triage.md`
  (B1 board `GhostDragStartEvent` producer fixture gap, B2 HUD
  `snapshot.phase` bridge fixture gap, B3 lobby `ConfirmClass` after
  `SelectClass` intent chain, B4 `co_occupancy_offset` panic-guard drift,
  B5 `ShopAuctionUiEntity` count drift). Sprint 11 Should Have / Nice to
  Have rows remain blocked pending story authoring + `/story-readiness`.
- Preserve the PROMPT 761 Release gate failure and all carried risks.
- Do not retry `Polish->Release` until release-scope artifacts exist.
- Next launchable prompts (Sprint 11 QA plan on `main` per PROMPT 774;
  `S11-DOC-HYGIENE-CARRY-001` closed by PROMPT 780;
  `S11-EVIDENCE-INDEX-CARRY-001` closed by PROMPT 781;
  `S11-DRAG-RUNTIME-RETEST-001` closed by PROMPT 783 with
  `PASS-CANNOT-REPRODUCE` disposition;
  `S11-TD-FIXTURE-HAND-UI-ONENTER-001` closed by PROMPT 785 with
  `PASS` disposition;
  `S11-ROUTE-READABILITY-CARRY-001` closed by PROMPT 786;
  `S11-TD-IGNORED-D5-TRIAGE-001` closed by PROMPT 789):
  (1) `/sprint-plan sprint-11 --add story-019` to activate the
  follow-on diagnostic story
  `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  into Sprint 11 active scope (separate prompt); (2) story file
  authoring + `/story-readiness` for any Cluster B follow-up
  (`S11-TD-FIXTURE-D-RESIDUALS-001` umbrella expansion already names B1
  and B5, plus new slugs `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`,
  `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`,
  `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`, and optional splits
  `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` and
  `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001`); (3) Sprint 11 close-out
  decision — only after Should Have / Nice to Have scope is either
  pulled into active scope and closed or explicitly deferred, and only
  after a Sprint 11 smoke + QA sign-off + (if release is asserted)
  Polish->Release gate-check retry.

### PROMPT 791 Sprint 11 Team-QA sign-off — verdict PASS-WITH-WARNINGS / APPROVED WITH CONDITIONS (2026-05-13)

Sprint 11 Team-QA / QA sign-off executed on root checkout (no worktree)
against `origin/main@1617352` (PROMPT 790 smoke evidence tip
`qa(smoke): Sprint 11 smoke check`). Evidence written at
`production/qa/team-qa-sprint-11-2026-05-13.md`. This is a Polish-stage
friend-game QA sign-off only — **NOT** a `/gate-check`, **NOT** a
`/release-check`, **NOT** a Sprint 11 close-out, **NOT** a release-readiness
claim, **NOT** a smoke rerun.

#### Preflight

- `git fetch origin` OK.
- `git rev-parse HEAD` == `git rev-parse origin/main` == `1617352`.
- `git status --short` shows pre-existing ` M .claude/settings.json` only.
  PROMPT 791 preserved this unstaged modification untouched (not staged,
  not committed).

#### Verdict

`PASS-WITH-WARNINGS` (Team-QA equivalent `APPROVED WITH CONDITIONS`).
Recommendation: **ready for Sprint 11 close-out with conditions** — close-out
itself is a separate orchestrator decision in a separate prompt.

- All Sprint 11 Must Have rows are `done` on `origin/main@1617352` (6/6).
- Smoke verdict `PASS-WITH-WARNINGS` (1129 passed / 0 failed / 5 ignored).
- The 5 ignored tests match the documented Cluster B retainers in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md` (B1 board
  `GhostDragStartEvent` producer fixture gap, B2 HUD `snapshot.phase`
  bridge fixture gap, B3 lobby `ConfirmClass` after `SelectClass` intent
  chain, B4 `co_occupancy_offset` panic-guard drift, B5
  `ShopAuctionUiEntity` count drift) — each with owner-named follow-up
  story slug or decision gate. No undocumented failure / no undocumented
  ignored test surfaced.
- Carried conditions preserved unchanged: `S8-QA-001-W1` OPEN,
  `QA-COND-0005` accepted-risk friend-game scope, `QA-COND-0006`
  accepted-risk / deferred, placeholder / friend-game art `PAW-TD-*-a`
  accept-risk, HUD-timer eyeball check W2 deferred.
- PROMPT 761 `Polish->Release` gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry
  attempted.

#### Conditions attached to this sign-off

- **TQ-S11-C1** — Sprint 11 close-out is a separate orchestrator decision;
  this sign-off does NOT close Sprint 11.
- **TQ-S11-C2** — The 5 Cluster B ignored tests must be tracked as Sprint
  12 (or later) backlog candidates; no row authorises immediate
  implementation under this sign-off.
- **TQ-S11-C3** — `S8-QA-001-W1` remains OPEN. Sign-off does NOT include
  manual / browser two-client GAME_OVER evidence.
- **TQ-S11-C4** — `QA-COND-0005` and `QA-COND-0006` remain accepted-risk /
  deferred. Sign-off does NOT include accessibility or playtest evidence.
- **TQ-S11-C5** — PROMPT 761 `Polish->Release` gate-check `FAIL` remains
  preserved; do NOT retry until release-scope artefacts exist on `main`.
- **TQ-S11-C6** — Placeholder / friend-game art scope (`PAW-TD-*-a`)
  remains accept-risk; no final-art / asset-production-completion claim.

#### Files changed by PROMPT 791

- `production/qa/team-qa-sprint-11-2026-05-13.md` (NEW — Team-QA sign-off
  evidence)
- `production/session-state/active.md` (banner prepended)
- `production/session-state/codex-orchestrator-state.md` (operating-rules
  `Current verified state` updated; PROMPT 791 disposition section
  prepended above PROMPT 790)
- `reports/PROMPT-791.md` (mandatory final report; NOT staged or
  committed)

Explicitly NOT touched by PROMPT 791: `.claude/settings.json`,
`client/`, `server/`, `shared/`, `tests/`, `production/sprint-status.yaml`,
`production/stage.txt`, `production/sprints/sprint-11.md`,
`production/qa/qa-plan-sprint-11.md`,
`production/qa/smoke-sprint-11-2026-05-13.md`,
`production/qa/evidence/sprint-11-ignored-d5-triage.md`,
`production/gate-checks/gate-polish-release-2026-05-12.md`.

#### Explicit non-claims

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005`
  unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk
  preserved)
- no `S8-QA-001-W1` closure
- no Polish→Release retry
- no Sprint 11 close-out

---

### PROMPT 790 Sprint 11 smoke check — verdict PASS-WITH-WARNINGS (2026-05-13)

Sprint 11 Polish / friend-game smoke check executed on root checkout (no
worktree) against `origin/main@18758b2` (PROMPT 789 integration tip
`story-done(s11): close ignored D-5 triage`). Evidence written at
`production/qa/smoke-sprint-11-2026-05-13.md`. This is a Polish-stage smoke
check only — **NOT** a `/gate-check`, **NOT** a `/team-qa` run, **NOT** a
`/release-check`, **NOT** a QA sign-off, **NOT** a Sprint 11 close-out.

#### Preflight

- `git fetch origin` OK.
- `git rev-parse HEAD` == `git rev-parse origin/main` ==
  `18758b25df209fa03cf9c0ba5237c7577ef33f8e`.
- `git status --short` shows pre-existing ` M .claude/settings.json` only.
  PROMPT 790 preserved this unstaged modification untouched per the
  operating contract (the file is **not** staged, **not** committed).
- D: free space ~222 GB / 1.3 TB (`df -h /d`). Sufficient for the workspace
  test suite. No `BLOCKED-DISK` reached.

#### Commands and results

| Command | Verdict |
|---|---|
| `cargo fmt --check` | PASS — exit 0, no output |
| `cargo check --workspace` | PASS — `Finished \`dev\` profile [optimized + debuginfo] target(s) in 1m 15s` |
| `cargo test --workspace --tests --no-fail-fast` | PASS-WITH-WARNINGS — aggregated **1129 passed / 0 failed / 5 ignored** across 189 binaries |
| `git diff --check` | informational CRLF advisory on `.claude/settings.json` only (not a whitespace error) |
| `git diff --cached --check` | empty (no staged changes) |

#### Ignored-test reconciliation (5 == documented Cluster B)

The 5 ignored tests reported by `cargo test --workspace --tests` exactly
match the 5 Cluster B retained D-5 tests documented at
`production/qa/evidence/sprint-11-ignored-d5-triage.md` lines 30-32 and
96-103:

1. `tests/integration/board_rendering/ghost_preview_bridge_test.rs ::
   br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui`
   (Cluster B1 — board `GhostDragStartEvent` producer fixture gap).
2. `tests/integration/board_rendering/snapshot_spawn_test.rs ::
   test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`
   (Cluster B2 — HUD `snapshot.phase` bridge fixture gap).
3. `tests/integration/playable_client/native_operator_controls_test.rs ::
   test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
   (Cluster B3 — lobby `ConfirmClass` after `SelectClass` intent chain).
4. `tests/unit/board_rendering/status_icons_test.rs ::
   test_cooccupancy_index_two_panics_with_offending_index`
   (Cluster B4 — `co_occupancy_offset` panic-guard drift).
5. `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs ::
   shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
   (Cluster B5 — `ShopAuctionUiEntity` count drift).

No undocumented ignored test surfaced; no regression on the Cluster A
(resolved by `S11-TD-FIXTURE-HAND-UI-ONENTER-001`) tests — those run and
pass in the workspace aggregate (1129 passed).

#### Verdict justification

Per `/smoke-check` skill verdict rules: automated test suite ran cleanly
with zero failures; remaining ignored tests are owner-named with landed
triage disposition. Verdict = **PASS-WITH-WARNINGS** (warning = the 5
documented D-5 ignored tests, not a regression).

#### Files changed by PROMPT 790

- `production/qa/smoke-sprint-11-2026-05-13.md` (NEW — Sprint 11 smoke
  evidence; the only artifact this prompt produces under `production/qa/`).
- `production/session-state/active.md` (PROMPT 790 banner prepended).
- `production/session-state/codex-orchestrator-state.md` (operating-rules
  `Current verified state` updated; this PROMPT 790 disposition section
  prepended above PROMPT 789).
- `reports/PROMPT-790.md` (mandatory final report; **not** staged or
  committed).

Explicitly **not** touched: `.claude/settings.json`, `client/`, `server/`,
`shared/`, `tests/`, `production/sprint-status.yaml`, `production/stage.txt`,
`production/sprints/sprint-11.md`,
`production/qa/evidence/sprint-11-ignored-d5-triage.md`, any other
`reports/` file.

#### Sprint 11 disposition

- Sprint 11 remains `active` (Polish-stage). All 6 Must Have rows remain
  `done` per PROMPTs 780 / 781 / 783 / 785 / 786 / 789.
- Stage remains `Polish`. PROMPT 761 Polish->Release gate `FAIL` preserved.
  No retry.
- Sprint 10 disposition unchanged (`closed-with-conditions` per PROMPT
  763).
- PROMPT 790 does **NOT** close Sprint 11. The 5 retained Cluster B
  ignored tests remain open as future stories / decision gates.

#### Non-claims (preserved)

No public release readiness, no release-candidate readiness, no full game
completion, no broad / Standard-tier accessibility completion
(`QA-COND-0005` unchanged), no playtest / fun-hypothesis validation
(`QA-COND-0006` unchanged), no full playable-client manual QA
(`S8-QA-001-W1` unchanged), no final-art / asset-production completion
(`PAW-TD-*-a` accept-risk preserved), no Sprint 11 close-out.

### PROMPT 789 /story-done Disposition — S11-TD-IGNORED-D5-TRIAGE-001 (2026-05-13)

Authoritative Sprint 11 row `S11-TD-IGNORED-D5-TRIAGE-001` closed by
`/story-done` in PROMPT 789. Source-of-truth at run: `origin/main@1d96281`
(PROMPT 788 integration commit `docs(qa): triage Sprint 11 D-5 ignored tests`).
Deliverable shipped at commit `1d96281` via PROMPT 787 worker + PROMPT 788
integration.

PROMPT 789 is paperwork-only `/story-done`-equivalent closure for a row that
has no standalone story file by design; closure runs against
`production/sprints/sprint-11.md` + `production/sprint-status.yaml` + the
landed triage evidence at
`production/qa/evidence/sprint-11-ignored-d5-triage.md`. No worker spawned.
No worktree opened. Root checkout only.

#### Deliverable provenance

- PROMPT 787 (2026-05-13): authored
  `production/qa/evidence/sprint-11-ignored-d5-triage.md` (185 lines)
  read-only against `origin/main@798ecc0`. Owner-named per-test
  disposition for the 11 D-5 `#[ignore]` tests surfaced by Sprint 10 smoke
  retry-7 W1. No test files modified; no production code touched.
- PROMPT 788 (2026-05-13): integrated the PROMPT 787 worker evidence to
  `main` at commit `1d96281` (single-file `+185` lines doc-only commit).

#### Accounting verdict — 11/11

- **Original total** (Sprint 10 smoke retry-7 W1,
  `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` lines 59-74): 11
  owner-named `#[ignore]` tests in 6 files.
- **Cluster A — resolved by `S11-TD-FIXTURE-HAND-UI-ONENTER-001`** (6
  tests): A1 `test_placement_exit_clears_stale_hand_timer_submit_and_pending_state`
  (`tests/integration/playable_client/active_loop_ui_state_test.rs`), A2
  `test_one_card_acquired_fanout_updates_hand_and_draft_pending_purchase`
  (`tests/integration/playable_client/draft_shop_hand_bridge_test.rs`), A3
  `test_one_draft_offering_fanout_updates_hand_grid_and_shop_grid`
  (`tests/integration/playable_client/draft_shop_hand_bridge_test.rs`), A4
  `test_shop_purchase_reconciles_hand_size_slots_and_shared_economy`
  (`tests/integration/playable_client/draft_shop_hand_bridge_test.rs`), A5
  `test_hand_pointer_controls_stage_unstage_and_submit_placement`
  (`tests/integration/playable_client/native_operator_controls_test.rs`),
  A6 `test_reserve_strip_input_does_not_mutate_player_economy_view`
  (`tests/integration/presentation/shared_economy_view_test.rs`). All
  un-`#[ignore]`d at PROMPT 784 integration commit `d7f4103` and closed
  by PROMPT 785 `/story-done` at `a8af79a`.
- **Cluster B — retained `#[ignore]` (5 tests)**: B1
  `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui`
  (`tests/integration/board_rendering/ghost_preview_bridge_test.rs:147`)
  — board `GhostDragStartEvent` producer fixture gap; B2
  `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`
  (`tests/integration/board_rendering/snapshot_spawn_test.rs:39`) — HUD
  `snapshot.phase` bridge fixture gap; B3
  `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
  (`tests/integration/playable_client/native_operator_controls_test.rs:106`)
  — lobby `ConfirmClass` after `SelectClass` intent chain (production
  lobby input investigation); B4
  `test_cooccupancy_index_two_panics_with_offending_index`
  (`tests/unit/board_rendering/status_icons_test.rs:167`) —
  `co_occupancy_offset` panic-guard drift; B5
  `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
  (`tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:25`) —
  `ShopAuctionUiEntity` count drift (actual=66, formula=57; +9 delta).
- **Roll-up**: 6 + 5 = **11**. None silently dropped.

#### Story acceptance-criterion verification (read-only against `origin/main@1d96281`)

- **AC1 — triage evidence file exists on main**:
  `production/qa/evidence/sprint-11-ignored-d5-triage.md` at commit
  `1d96281` (PROMPT 788 integration of PROMPT 787 worker authoring).
- **AC2 — 11/11 accounted**: evidence file totals table (lines 30-32) and
  roll-up table (lines 96-103) confirm 6 resolved + 5 retained = 11.
- **AC3 — 6 resolved tests linked to S11-TD-FIXTURE-HAND-UI-ONENTER-001 +
  PROMPT 779 / 784 / 785 evidence**: Cluster A table (lines 58-65) cites
  PROMPT 779 worker, PROMPT 784 integration commit `d7f4103`, PROMPT 785
  `/story-done` verdict at `a8af79a`, and the underlying evidence file
  `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
  `Per-fixture repair` table for each A-row.
- **AC4 — 5 retained tests carry owner-named disposition + follow-up
  path**: Cluster B table (lines 81-87) names owner + production system,
  classification (`needs-repair-story` for B1 / B3 vs.
  `needs-design-decision` for B2 / B4 / B5), proposed follow-up story
  slug, and decision gate. Follow-up slugs:
  `S11-TD-FIXTURE-D-RESIDUALS-001` umbrella OR new
  `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` split for B1; new
  `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001` for B2; new
  `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001` for B3; new
  `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001` for B4;
  `S11-TD-FIXTURE-D-RESIDUALS-001` umbrella OR new
  `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` split for B5.
- **AC5 — no evidence row claims the retained 5 are fixed**: each B-row
  carries `still ignored` state on `main` with the original PROMPT 750
  D-5 owner-named comment unchanged in the test source file.
- **AC6 — non-claims explicit**: evidence file lines 163-185 carry the
  full friend-game-lite non-claim ladder — public release readiness NOT
  claimed, release-candidate readiness NOT claimed, full game completion
  NOT claimed, broad / Standard-tier accessibility completion NOT claimed
  (`QA-COND-0005` unchanged), playtest / fun-hypothesis validation NOT
  claimed (`QA-COND-0006` unchanged), full playable-client manual QA NOT
  claimed (`S8-QA-001-W1` unchanged), final-art / asset-production
  completion NOT claimed (`PAW-TD-*-a` accept-risk unchanged), Sprint 11
  close-out NOT claimed, closure of any individual Cluster B ignored test
  NOT claimed.
- **AC7 — no row authorises immediate implementation**: each Cluster B
  follow-up slug explicitly requires its own story file +
  `/story-readiness` in a separate prompt before `/dev-story` can begin
  (evidence file § "Proposed follow-up story slugs" lines 106-132).
- **AC8 — Sprint 11 disposition preserved**:
  `production/sprints/sprint-11.md` untouched by PROMPT 789;
  `production/stage.txt` unchanged (`Polish`); the triage evidence file
  itself untouched by PROMPT 789 (closure paperwork only on top of the
  `1d96281` deliverable). Sprint 11 status remains `active` (Polish stage);
  Sprint 10 disposition remains `closed-with-conditions`.

#### Files changed by PROMPT 789

- `production/sprint-status.yaml`:
  - `S11-TD-IGNORED-D5-TRIAGE-001` row flipped `status: ready` → `status: done`.
  - `blocker:` cleared.
  - `completed: ""` → `completed: "2026-05-13"`.
  - PROMPT 787 + PROMPT 788 worker / integration note appended to `notes:`.
  - PROMPT 789 /story-done verdict note appended to `notes:`.
  - Top-of-file `updated:` annotation refreshed.
- `production/session-state/active.md`: PROMPT 789 banner prepended; PROMPT
  786 banner demoted to `PRIOR CURRENT STATE`.
- `production/session-state/codex-orchestrator-state.md`: `Updated:` header
  refreshed; `Current verified state` updated (HEAD `a8af79a` → `1d96281`,
  PROMPT 786 → PROMPT 789, six Sprint 11 Must Have rows closed); `Current
  next move` `Next launchable prompts` list updated (S11-TD-IGNORED-D5
  closed; Cluster B follow-up slugs enumerated; Sprint 11 close-out gate
  flagged); this PROMPT 789 disposition section prepended above PROMPT 786.
- `reports/PROMPT-789.md`: mandatory final report file (NOT staged or
  committed).

#### Working-tree state PROMPT 789 inherited

- `.claude/settings.json` was already modified in the working copy at PROMPT
  789 start; PROMPT 789 did NOT touch this file and explicitly excluded it
  from the staged set. The modification carries over outside the PROMPT 789
  commit.

#### Paperwork-only — explicit non-actions

PROMPT 789 did NOT:

- run `/dev-story`, `/story-readiness`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or any close-out skill;
- modify production code under `client/`, `server/`, `shared/`, or `tests/`;
- modify the triage evidence file
  `production/qa/evidence/sprint-11-ignored-d5-triage.md` (read-only
  verification only);
- modify `production/sprints/sprint-11.md`, `production/stage.txt`,
  `.claude/settings.json`, `reports/` (other than the mandatory
  `reports/PROMPT-789.md` final report file), `.claude/scheduled_tasks.lock`,
  or `.octogent/`;
- close Sprint 11 — the 5 retained Cluster B ignored tests remain open as
  future stories or decision gates; Sprint 11 Should Have / Nice to Have
  rows remain blocked pending story authoring + `/story-readiness`;
- claim closure of any individual Cluster B ignored test;
- claim Sprint 11 release-candidate readiness, public release readiness,
  full game completion, broad / Standard-tier accessibility completion
  (`QA-COND-0005` unchanged), playtest / fun-hypothesis validation
  (`QA-COND-0006` unchanged), full playable-client manual QA
  (`S8-QA-001-W1` unchanged), final-art / asset-production completion
  (`PAW-TD-*-a` accept-risk unchanged), Sprint 11 close-out, or a
  Polish->Release gate-check retry.

---

### PROMPT 786 /story-done Disposition — S11-ROUTE-READABILITY-CARRY-001 (2026-05-13)

Authoritative Sprint 11 row `S11-ROUTE-READABILITY-CARRY-001` closed by
`/story-done` in PROMPT 786. Source-of-truth at run: `origin/main@a8af79a`
(PROMPT 785 integration commit `story-done(s11): close hand-ui OnEnter fixture
repair`). Deliverable shipped at commit `d3ee8df` via PROMPT 772 (2026-05-13).

PROMPT 786 is paperwork-only `/story-done`-equivalent closure for a row that
has no standalone story file by design; closure runs against
`production/sprints/sprint-11.md` + `production/sprint-status.yaml` + the
landed evidence at `production/qa/evidence/sprint-10-route-readability-notes.md`.
No worker spawned. No worktree opened. Root checkout only.

#### Deliverable provenance

- PROMPT 772 (2026-05-13): authored
  `production/qa/evidence/sprint-10-route-readability-notes.md` at integration
  commit `d3ee8df`. Sprint 11 draft Must Have paperwork carry of deferred
  Sprint 10 nice-to-have `S10-N2` per PROMPT 763 close-out and PROMPT 764
  Sprint 11 draft plan. Concise rough-edge readability observations for the
  friend-game route; explicitly does **not** activate Sprint 11, mutate
  `production/sprint-status.yaml`, mutate `production/sprints/sprint-11.md`,
  or claim closure of any carried condition.
- PROMPT 773 (2026-05-13): activated Sprint 11 with this row marked `ready`
  (not `done`) per the no-invent-closure rule.

#### Story acceptance-criterion verification (read-only against `origin/main@a8af79a`)

- **AC1 — evidence file exists**: `production/qa/evidence/sprint-10-route-readability-notes.md`
  is on `main` at `d3ee8df` (PROMPT 772 commit).
- **AC2 — all eight friend-game routes covered**: Route 1 Lobby (4 rows),
  Route 2 Hand / Drag (3 rows), Route 3 Draft Grid / DRAFT_INITIAL (2 rows),
  Route 4 Shop / DRAFT_SHOP (3 rows), Route 5 Auction / DRAFT_AUCTION
  (3 rows), Route 6 Board / Placement + Resolution (3 rows), Route 7
  HUD / Timer (4 rows), Route 8 Result / Close-Out (3 rows).
- **AC3 — every observation classified**: classifications cover
  `already-tracked` (cross-references to existing Sprint 11 backlog rows),
  `future-story-candidate` (new slugs without a story file),
  `accepted-risk-friend-game` (explicit out-of-scope rows), and a `scope
  guard` Cross-Route Notes section calling out final-art accept-risk under
  `PAW-TD-*-a`.
- **AC4 — Non-Claims section explicit at lines 30-46**: public release
  readiness NOT claimed, release-candidate readiness NOT claimed, full game
  completion NOT claimed, broad / Standard-tier accessibility completion
  NOT claimed (`QA-COND-0005` remains accepted-risk friend-game scope),
  playtest / fun-hypothesis validation NOT claimed (`QA-COND-0006` remains
  accepted-risk / deferred), full playable-client manual QA NOT claimed,
  full manual / browser two-client GAME_OVER route NOT claimed
  (`S8-QA-001-W1` remains OPEN), final-art / asset-production completion
  NOT claimed (`PAW-TD-*-a` accept-risk preserved across PAW-002..PAW-006),
  Sprint 11 activation NOT claimed (PROMPT 772 ran before PROMPT 773
  activation), closure of any existing Sprint 10 carry or Sprint 11 row
  NOT claimed.
- **AC5 — no row authorises immediate implementation**: every
  `future-story-candidate` slug explicitly requires its own story file +
  `/story-readiness` in a separate prompt before `/dev-story` can begin
  (file lines 173-178 + § Authoring Disposition).
- **AC6 — Sprint 11 disposition preserved**: `production/sprints/sprint-11.md`
  untouched by PROMPT 786; `production/stage.txt` unchanged (`Polish`);
  the underlying evidence file itself untouched by PROMPT 786 (closure
  paperwork only on top of the `d3ee8df` deliverable).

#### Files changed by PROMPT 786

- `production/sprint-status.yaml`:
  - `S11-ROUTE-READABILITY-CARRY-001` row flipped `status: ready` → `status: done`.
  - `completed: ""` → `completed: "2026-05-13"`.
  - PROMPT 786 /story-done verdict note appended to `notes:`.
  - Top-of-file `updated:` annotation refreshed.
- `production/session-state/active.md`: PROMPT 786 banner prepended; PROMPT
  785 banner demoted to `PRIOR CURRENT STATE`.
- `production/session-state/codex-orchestrator-state.md`: `Current verified
  state` updated (HEAD `d7f4103` → `a8af79a`, PROMPT 785 → PROMPT 786,
  five rows closed); `Current next move` `Next launchable prompts` list
  updated; this PROMPT 786 disposition section prepended above PROMPT 785.
- `reports/PROMPT-786.md`: mandatory final report file (NOT staged or
  committed).

#### Working-tree state PROMPT 786 inherited

- `.claude/settings.json` was already modified in the working copy at PROMPT
  786 start; PROMPT 786 did NOT touch this file and explicitly excluded it
  from the staged set. The modification carries over outside the PROMPT 786
  commit.

#### Paperwork-only — explicit non-actions

PROMPT 786 did NOT:

- run `/dev-story`, `/story-readiness`, `/smoke-check`, `/team-qa`,
  `/gate-check`, sprint close-out, or release-check;
- modify production code under `client/`, `server/`, `shared/`, `tests/`;
- modify `production/sprints/sprint-11.md`, `production/stage.txt`,
  `production/qa/evidence/sprint-10-route-readability-notes.md`,
  `.claude/settings.json`, `.claude/scheduled_tasks.lock`, `.octogent/`,
  `.gitignore`, or any `reports/` file other than `reports/PROMPT-786.md`;
- mutate Sprint 10 close-out disposition (`closed-with-conditions` per
  PROMPT 763 preserved unchanged under `sprint_10_closeout:`);
- claim public release readiness, release-candidate readiness, full game
  completion, broad / Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, full playable-client manual QA, or final-art /
  asset-production completion;
- claim closure of `QA-COND-0005`, `QA-COND-0006`, `S8-QA-001-W1`, or any
  other carried condition;
- retry the PROMPT 761 Polish→Release gate-check.

#### Sprint 11 Must Have status after PROMPT 786

- **done**: `S11-DOC-HYGIENE-CARRY-001` (PROMPT 780),
  `S11-EVIDENCE-INDEX-CARRY-001` (PROMPT 781),
  `S11-DRAG-RUNTIME-RETEST-001` (PROMPT 783),
  `S11-TD-FIXTURE-HAND-UI-ONENTER-001` (PROMPT 785),
  `S11-ROUTE-READABILITY-CARRY-001` (PROMPT 786).
- **ready**: `S11-TD-IGNORED-D5-TRIAGE-001` (no story file yet; per-test
  triage doc target path is
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`).

#### Carried forward unchanged by PROMPT 786

- `S8-QA-001-W1` manual/browser two-client GAME_OVER gap (OPEN).
- `QA-COND-0005` Standard-tier accessibility (accepted-risk friend-game scope).
- `QA-COND-0006` playtest / fun-hypothesis validation (accepted-risk / deferred).
- 5 remaining ignored D-5 tests from smoke retry-7 W1 (folded into
  `S11-TD-IGNORED-D5-TRIAGE-001`).
- HUD timer eyeball visual check (W2; folded into
  `S11-HUD-TIMER-EYEBALL-VISUAL-001`).
- Placeholder / friend-game art scope (`PAW-TD-*-a` accept-risk on
  placeholder PNGs across PAW-002..PAW-006).
- PROMPT 683-era runtime divergence question preserved unchanged for
  follow-on story 019 (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`,
  on `main` at `0fc05c3`, not yet activated into Sprint 11 active scope).

### PROMPT 785 /story-done Disposition — S11-TD-FIXTURE-HAND-UI-ONENTER-001 (2026-05-13)

Authoritative Sprint 11 row `S11-TD-FIXTURE-HAND-UI-ONENTER-001` closed by
`/story-done` in PROMPT 785. Source-of-truth at run: `origin/main@d7f4103`
(PROMPT 784 integration of worker branch
`work/s11-hand-ui-onenter-fixture-repair` produced by PROMPT 779 /dev-story).

PROMPT 785 is paperwork-only `/story-done` paperwork on top of PROMPT 784's
integration. No worker spawned. No worktree opened. Root checkout only.

#### Worker provenance

- PROMPT 779 /dev-story (2026-05-13): dispatched the Hand UI OnEnter
  fixture-cascade repair from `origin/main@d36bbbd` (PROMPT 774 — Sprint
  11 QA plan). Worker branch: `work/s11-hand-ui-onenter-fixture-repair`.
  Worker disposition: PASS (all AC1-AC8 satisfied; full-workspace
  verification 1129 passed / 0 failed / 5 ignored against retry-7
  baseline 1123 passed / 0 failed / 11 ignored — delta +6 passed / -6
  ignored = the 6 cluster tests un-#[ignore]d).
- PROMPT 784 (2026-05-13): integrated the worker to `main` at commit
  `d7f4103` (single commit; no merge commit). Integration verification
  passed for the 4 affected integration test binaries individually
  (`shared_economy_view_test`,
  `playable_client_active_loop_ui_state_test`,
  `playable_client_draft_shop_hand_bridge_test`,
  `playable_client_native_operator_controls_test`) plus
  `cargo test -p client --no-fail-fast` (390 passed / 0 failed / 5
  ignored) plus `cargo fmt --check`. PROMPT 784 could not rerun the
  full workspace test post-integration because D: drive was full and
  `link.exe` failed with `LNK1180 insufficient disk space` —
  environment limitation explicitly recorded.

#### Story-011 acceptance-criterion verification (read-only against `origin/main@d7f4103`)

- **AC1 — Per-test disposition**: PASS. All 6 cluster tests un-#[ignore]d
  and passing under the repaired fixtures. Diff confirms removal of
  `#[ignore = "PROMPT 750 D-5 follow-on: spawn_hand_ui not firing ..."]`
  attributes at:
  - `tests/integration/playable_client/active_loop_ui_state_test.rs:225`
    (`test_placement_exit_clears_stale_hand_timer_submit_and_pending_state`)
  - `tests/integration/playable_client/draft_shop_hand_bridge_test.rs:71`
    (`test_one_draft_offering_fanout_updates_hand_grid_and_shop_grid`)
  - `tests/integration/playable_client/draft_shop_hand_bridge_test.rs:87`
    (`test_one_card_acquired_fanout_updates_hand_and_draft_pending_purchase`)
  - `tests/integration/playable_client/draft_shop_hand_bridge_test.rs:123`
    (`test_shop_purchase_reconciles_hand_size_slots_and_shared_economy`)
  - `tests/integration/playable_client/native_operator_controls_test.rs:214`
    (`test_hand_pointer_controls_stage_unstage_and_submit_placement`)
  - `tests/integration/presentation/shared_economy_view_test.rs:67`
    (`test_reserve_strip_input_does_not_mutate_player_economy_view`)
- **AC2 — Workspace ignored-count reduction OR owner-named disposition**:
  PASS. Workspace ignored count drops by 6 (11 -> 5) per PROMPT 779
  worker workspace verification. Each of the 5 remaining ignored tests
  carries an owner-named disposition comment pointing at a distinct
  non-`spawn_hand_ui` sibling-cluster cause (board `GhostDragStartEvent`
  producer; `HudPlugin` snapshot.phase bridge; lobby `ConfirmClass`
  intent chain; `co_occupancy_offset` panic guard;
  `ShopAuctionUiEntity` count drift). No silent `#[ignore]` retention.
- **AC3 — Reusable fixture helper**: PASS.
  `client::asset_wiring::enter_in_session_via_fixture` added at
  `client/src/asset_wiring.rs:420-453`, mirroring the
  `placeholder_assets_for_tests()` precedent (pub fn, no `#[cfg(test)]`
  gate; integration test binaries consume the library as a normal
  dependency). Called from all 4 repaired fixtures in place of the
  ad-hoc `NextState + run_update` block. No duplicated entry boilerplate.
- **AC4 — Pattern documentation**: PASS.
  `docs/architecture/test-fixture-patterns.md` (new, ~138 lines, single
  page). Covers: why the doc exists (silent-skip failure class), when
  to use the helper, what goes wrong without it (the
  `spawn_hand_ui` / `placeholder.is_none()` early-return chain), helper
  signature + behavior + pre-conditions, minimal example, side effects
  (does not also set `RoundPhase`; image handles are `Handle::default`),
  related precedent (`placeholder_assets_for_tests` from S10-TD-001
  Layer 3). Doc cross-links back to this story id and to story-009.
- **AC5 — `cargo test -p client --no-fail-fast` passes for repaired
  set**: PASS. PROMPT 784 integration verification: 390 passed / 0
  failed / 5 ignored.
- **AC6 — No production code modified**: PASS.
  `git show --stat d7f4103` confines the diff to:
  - `client/src/asset_wiring.rs` +48 (helper-only addition mirroring
    `placeholder_assets_for_tests` precedent — AC6 test-helper
    exception)
  - `docs/architecture/test-fixture-patterns.md` +138 (NEW — pattern
    doc)
  - `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
    +305 (NEW — evidence)
  - 4 integration test files (un-#[ignore] + helper call replacing
    ad-hoc `NextState + update` block)
  Zero changes under `server/src/`, `shared/src/`, or any non-test
  `client/src/` path.
- **AC7 — Sprint 11 disposition preserved**: PASS. Worker commit
  `d7f4103` did NOT modify `production/sprint-status.yaml`,
  `production/sprints/sprint-11.md`, or `production/stage.txt`. Stage
  remains `Polish`. PROMPT 785 /story-done paperwork flips the row to
  `done` in `production/sprint-status.yaml` only (a separate paperwork
  commit on top of `d7f4103`); no `production/sprints/sprint-11.md`
  or `production/stage.txt` mutation. No release / release-candidate /
  full-game / broad-accessibility / playtest / full-manual-QA /
  final-art claim.
- **AC8 — Evidence document populated**: PASS. 305 lines at
  `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
  covering diagnosis (S10-TD-001 Layer 3 cascade classification),
  per-test repair table (6 rows), 7th-sibling-test resolution (no
  7th — PROMPT 762 "7x" count was a counting artifact), sibling
  ignored tests table (5 remaining), pre/post test counts per binary
  + workspace, production source diff audit, Sprint 11 disposition
  preservation audit, pattern documentation cross-link, AC1-AC8
  sign-off table, verification commands run.

#### Verification commands run by PROMPT 785 itself (root checkout)

- `git fetch origin` — clean; HEAD == origin/main == `d7f4103`.
- `git status` — only modification is `.claude/settings.json` (forbidden
  territory; PROMPT 785 does not touch it).
- `git show --stat d7f4103` — confirms the 7-file scope and the diff
  shapes match the evidence document.
- `cargo fmt --check` — PASS.
- `git diff --check` — clean (only `.claude/settings.json` LF/CRLF
  warning, expected).
- `git diff --cached --check` — clean.
- Full-workspace `cargo test --workspace --tests --no-fail-fast` NOT
  rerun: D: drive has ~2 MB free (Get-PSDrive denied; `df -h /d`
  reports `Avail 2.2M`); link.exe would fail with LNK1180 exactly as
  PROMPT 784 reported. Environment limitation explicitly recorded.
  PROMPT 779 worker-side full-workspace count (1129 passed / 0 failed
  / 5 ignored) and PROMPT 784 client-crate integration count (390
  passed / 0 failed / 5 ignored) cited as authoritative post-integration
  verification.

#### Files mutated by PROMPT 785

- `production/sprint-status.yaml` — row
  `S11-TD-FIXTURE-HAND-UI-ONENTER-001` flipped `status: ready -> done`;
  `completed: "2026-05-13"`; `blocker:` cleared; appended PROMPT 779
  /dev-story note + PROMPT 785 /story-done verdict note with the full
  AC verification narrative. Top-of-file `updated:` annotation
  refreshed.
- `production/session-state/active.md` — PROMPT 785 CURRENT-STATE
  banner prepended above the prior PROMPT 783 banner. Prior banners
  preserved as historical.
- `production/session-state/codex-orchestrator-state.md` — operating
  rules updated (`Current verified state at this update` line and
  `Next launchable prompts` listing reflect PROMPT 785 closure of
  `S11-TD-FIXTURE-HAND-UI-ONENTER-001`); this PROMPT 785 disposition
  section prepended above the PROMPT 783 disposition section.

#### Forbidden / not-run by PROMPT 785

`/dev-story`, `/story-readiness`, `/smoke-check`, `/team-qa`,
`/gate-check`, `/qa-plan`. PROMPT 785 did NOT modify production code
under `client/`, `server/`, `shared/`, or `tests/`. PROMPT 785 did NOT
modify `production/stage.txt`, `production/sprints/sprint-11.md`,
`.claude/settings.json`, `.claude/scheduled_tasks.lock`, `.octogent/`,
or `.gitignore`. PROMPT 785 did NOT touch any file under `reports/`
other than the mandatory `reports/PROMPT-785.md` final report file
(written but NOT staged or committed).

No release claim. No release-candidate claim. No accessibility-completion
claim. No playtest-validation claim. No full-game-completion claim. No
final-art / asset-production-completion claim. No
full-playable-client-manual-QA claim. No Sprint 11 close-out claim. No
retry of the Polish->Release gate-check. No optimistic client-side
authority introduced (ADR-002 + ADR-009 binding).

#### Sprint 11 Must Have status after PROMPT 785

4/6 `done` (`S11-DOC-HYGIENE-CARRY-001`,
`S11-EVIDENCE-INDEX-CARRY-001`, `S11-DRAG-RUNTIME-RETEST-001`,
`S11-TD-FIXTURE-HAND-UI-ONENTER-001`); 2/6 `ready`
(`S11-TD-IGNORED-D5-TRIAGE-001`, `S11-ROUTE-READABILITY-CARRY-001`).
The remaining paperwork carry (`S11-ROUTE-READABILITY-CARRY-001`) has
its deliverable on `main` at `d3ee8df` and remains `ready` pending its
own `/story-done` prompt. `S11-TD-IGNORED-D5-TRIAGE-001` has no story
file yet; the per-test triage doc target path is
`production/qa/evidence/sprint-11-ignored-d5-triage.md` (per Sprint 11
QA plan); the 6 cluster rows just closed by this prompt are now
resolved cluster entries within the broader 11-test triage.

### PROMPT 783 /story-done Disposition — S11-DRAG-RUNTIME-RETEST-001 (2026-05-13)

Authoritative Sprint 11 row `S11-DRAG-RUNTIME-RETEST-001` closed by
`/story-done` in PROMPT 783. Source-of-truth at run: `origin/main@3ca1aff`
(PROMPT 782 merge integrating worker branch `work/s11-drag-runtime-retest`).
Worker deliverables verified on `main` at worker commit `0fc05c3` (PROMPT
778 /dev-story, 2026-05-13). PROMPT 778 worker disposition:
`PASS-CANNOT-REPRODUCE`. Story 018 acceptance-criterion verification
(read-only against `origin/main@3ca1aff`, deliverable commit `0fc05c3`):

- HU-DRAG-RT-01 — Runtime trace captured. **Deferred under
  cannot-reproduce disposition.** Story 018 §"Time-box" explicitly
  prescribes `cannot-reproduce` as a valid disposition when the
  1.0-day operator-driven two-client friend-game time-box cannot be
  exercised. PROMPT 778 was an automated CLI worker dispatch that
  cannot launch two browser tabs, manipulate `bevy_picking` pointers
  via mouse, or capture release-frame screenshots. The time-box was
  structurally unavailable. Static-code presence of S1-S5 emit sites
  was verified instead and recorded as code-evidence pointers in the
  truth-table.

- HU-DRAG-RT-02 — S1-S5 truth-table locked. **PASS.**
  `production/qa/evidence/sprint-11-drag-runtime-evidence.md` locks
  every row of the S1-S5 truth-table as `NOT-OBSERVED` across drag
  attempts A / B / C / D, with code-evidence pointers (file:line +
  `target:` string) for the emit-site presence. Code-evidence
  pointers from worker static verification: S1
  `client/src/ui/hand/mod.rs:2020` (`target: "drag_sprite_visible_flip"`);
  S2 `client/src/ui/hand/mod.rs:1901`
  (`target: "fan_active_default_drop"`); S3
  `client/src/ui/hand/mod.rs:2049`
  (`target: "placement_cursor_move"`); S4
  `client/src/card_animations/input_gating.rs:163`
  (`target: "drag_lift_tween_install"`); S5
  `client/src/presentation/board_rendering.rs:1709`
  (`target: "spawn_highlight_state_change"`); S5-callers L1640 /
  L1685 / L2622 (`target: "spawn_highlight_caller"`). Drag-ended
  gate widening from commit `cbb2565` (PROMPT 697) confirmed present
  at `client/src/ui/hand/mod.rs:2065`. Producer surface from commit
  `00ffe89` (PROMPT 696) confirmed present.

- HU-DRAG-RT-03 — Test-vs-runtime divergence dispositioned. **PASS.**
  Disposition is `cannot-reproduce` per story 018 §"Time-box". The
  PROMPT 683-era discrepancy (8 `C2SActivateCard` sends, zero
  `stage_or_update` events) is preserved as the **primary suspect
  for S5 release-branch → server** without being claimed as confirmed
  or refuted. Offending stage is **not named** because no row
  transitioned to FAIL in this run — every row was `NOT-OBSERVED`.

- HU-DRAG-RT-04 — Repair or follow-on authored. **PASS.** Follow-on
  diagnostic-only story authored at
  `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  (atomically by PROMPT 778 commit `0fc05c3`). It inherits the no-claim
  banner verbatim, inherits the §"Reproduction Recipe" with a
  tighter-capture protocol (adds `lightyear=debug` to the `RUST_LOG`
  chain, frame-level release-moment video capture, synchronised
  wall-clock timestamps for cross-client S2→S5 producer-consumer
  cross-check), names S5 as primary suspect, restates the "no
  optimistic client-side authority" prohibition, and is explicitly
  diagnostic-only — no repair commit may land inside the story under
  any disposition. Story 019 is currently `Draft`;
  `/story-readiness` is pending; activation into Sprint 11 active
  scope is a separate `/sprint-plan sprint-11 --add story-019`
  prompt.

- HU-DRAG-RT-05 — No production code changes in this story. **PASS.**
  Worker commit `0fc05c3` changed exactly 3 files (795 insertions,
  0 deletions): `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  (442 lines NEW), `production/qa/evidence/captures/sprint-11-drag-runtime/README.md`
  (36 lines NEW), `production/qa/evidence/sprint-11-drag-runtime-evidence.md`
  (317 lines NEW). `git diff --stat origin/main@3ca1aff..0fc05c3 -- client/ server/ shared/ tests/`
  returns EMPTY. Verified.

- HU-DRAG-RT-06 — No optimistic client-side authority introduced.
  **PASS.** Phrase "no optimistic client-side authority" present in
  both `production/qa/evidence/sprint-11-drag-runtime-evidence.md`
  and the follow-on story
  `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`.
  ADR-002 and ADR-009 lines preserved across the evidence file, the
  follow-on story, and any disposition pathway recorded therein.

- HU-DRAG-RT-07 — Non-claims preserved. **PASS.** Story 018
  §"Status / No-Claim Banner" restated verbatim in
  `production/qa/evidence/sprint-11-drag-runtime-evidence.md` §"Status
  / No-Claim Banner". The following are explicitly **NOT** claimed
  closed by this retest: public release readiness, release-candidate
  readiness, full game completion, broad / Standard-tier accessibility
  completion (`QA-COND-0005`), playtest / fun-hypothesis validation
  (`QA-COND-0006`), full playable-client manual QA, two-client
  GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production
  completion.

- HU-DRAG-RT-08 — Sprint 11 status/stage preserved. **PASS.** Worker
  commit `0fc05c3` did NOT modify `production/sprint-status.yaml`,
  `production/stage.txt`, or `production/sprints/sprint-11.md`.
  Verified by `git diff --stat origin/main@d36bbbd..0fc05c3 -- production/sprint-status.yaml production/stage.txt production/sprints/sprint-11.md`
  returning EMPTY. `production/stage.txt` reads `Polish`. Sprint 11
  `status: active`, activation by PROMPT 773, QA plan by PROMPT 774.

Files mutated by PROMPT 783:

- `production/sprint-status.yaml` — `S11-DRAG-RUNTIME-RETEST-001` row
  flipped `status: ready -> done`; `completed: "2026-05-13"`;
  `blocker: ""`; appended a PROMPT 778 /dev-story run note (worker
  branch, source-of-truth, commit, disposition, integration commit)
  and the PROMPT 783 /story-done verdict note with the full AC
  verification.

- `production/session-state/active.md` — PROMPT 783 CURRENT-STATE
  banner prepended above the prior PROMPT 781 banner. Prior banner
  preserved as historical.

- `production/session-state/codex-orchestrator-state.md` — current
  operating rules updated; this PROMPT 783 disposition section
  prepended above the PROMPT 781 disposition section.

- `reports/PROMPT-783.md` — mandatory final report file (the only
  `reports/` write in this run; not a substantive change to
  orchestrator state).

Forbidden / not-run by PROMPT 783: `/dev-story`, `/story-readiness`,
`/smoke-check`, `/team-qa`, `/gate-check`, `/qa-plan`. PROMPT 783 did
NOT modify production code under `client/` / `server/` / `shared/` /
`tests/`. PROMPT 783 did NOT modify `production/stage.txt`,
`production/sprints/sprint-11.md`, `.claude/settings.json`,
`.claude/scheduled_tasks.lock`, or `.octogent/`. No release claim.
No release-candidate claim. No accessibility-completion claim. No
playtest-validation claim. No full-game-completion claim. No
final-art / asset-production-completion claim. No
full-playable-client-manual-QA claim. No Sprint 11 close-out claim.
No retry of the Polish->Release gate-check. No optimistic
client-side authority introduced (ADR-002 + ADR-009 binding).

Carried forward unchanged: S8-QA-001-W1 manual/browser two-client
GAME_OVER gap (OPEN); QA-COND-0005 Standard-tier accessibility
(accepted-risk friend-game scope); QA-COND-0006 playtest /
fun-hypothesis validation (accepted-risk / deferred); 11 ignored
D-5 tests from smoke retry-7 W1 (folded into
`S11-TD-IGNORED-D5-TRIAGE-001` + `S11-TD-FIXTURE-HAND-UI-ONENTER-001`);
HUD timer eyeball visual check (W2; folded into
`S11-HUD-TIMER-EYEBALL-VISUAL-001`); placeholder / friend-game art
scope (PAW-TD-*-a accept-risk); PROMPT 683-era runtime divergence
question preserved unchanged for follow-on story 019.

Sprint 11 Must Have status after PROMPT 783: 3/6 `done`
(`S11-DOC-HYGIENE-CARRY-001`, `S11-EVIDENCE-INDEX-CARRY-001`,
`S11-DRAG-RUNTIME-RETEST-001`); 3/6 `ready`
(`S11-TD-FIXTURE-HAND-UI-ONENTER-001`,
`S11-TD-IGNORED-D5-TRIAGE-001`, `S11-ROUTE-READABILITY-CARRY-001`).

Next launchable prompts: (1) `/story-readiness
production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`
— formal verdict for `S11-TD-FIXTURE-HAND-UI-ONENTER-001`; (2)
`/story-done` for the remaining landed paperwork carry
(`S11-ROUTE-READABILITY-CARRY-001`) as a separate prompt; (3)
`/sprint-plan sprint-11 --add story-019` to activate the new
follow-on diagnostic story into Sprint 11 active scope (separate
prompt); (4) story file authoring for Should Have / Nice to Have
rows if pulled into active scope.

### PROMPT 781 /story-done Disposition — S11-EVIDENCE-INDEX-CARRY-001 (2026-05-13)

Authoritative Sprint 11 row `S11-EVIDENCE-INDEX-CARRY-001` closed by
`/story-done` in PROMPT 781. Source-of-truth at run: `origin/main@1bad399`.
Deliverable verified on `main` at `348084b` (PROMPT 771, 2026-05-13).
Acceptance-criterion verification (read-only against `origin/main@348084b`):

- AC1 — `production/qa/evidence/sprint-10-evidence-index.md` exists on
  `main` at `348084b` (PROMPT 771, 2026-05-13). Verified via
  `git show 348084b:production/qa/evidence/sprint-10-evidence-index.md`.
- AC2 — Records Sprint 10 disposition `closed-with-conditions` per PROMPT 763
  (linked through `production/sprint-status.yaml` `sprint_10_closeout:`
  block). Verified in the file header and Sprint 10 Headline table.
- AC3 — Records stage `Polish` (`production/stage.txt` unchanged). Verified
  in the file header and the Sprint 10 Headline `Stage after close-out`
  row.
- AC4 — Records smoke retry-7 `PASS WITH WARNINGS` (1123/1123 effective;
  11 ignored D-5 tests; HUD timer eyeball deferred) referencing
  `production/qa/smoke-sprint-10-2026-05-12-retry-7.md`. Verified in
  Sprint 10 Headline + Evidence File Map.
- AC5 — Records PROMPT 761 Polish->Release gate-check `FAIL` (0/13 required
  artefacts present) referencing
  `production/gate-checks/gate-polish-release-2026-05-12.md`. Verified in
  Sprint 10 Headline + the standing non-retry warning.
- AC6 — Records the Sprint 10 story / evidence map across Must Have
  (S10-PAW-001 sub-rolling PAW-002..PAW-006, S10-TD-001, S10-TD-002,
  S10-CARRY-001, S10-POLISH-001, S10-POLISH-002), Should Have
  (S10-POLISH-003, S10-TD-003 deferred, ECO-004), and Nice to Have
  (S10-N1 deferred, S10-N2 deferred) with integration commits and
  primary evidence paths. Verified in Per-Story Status tables +
  PAW-002..PAW-006 sub-table + Evidence File Map.
- AC7 — Records the three Sprint 10 deferred items (S10-TD-003, S10-N1,
  S10-N2) and their Sprint 11 carry IDs (`S11-DOC-HYGIENE-CARRY-001`,
  `S11-EVIDENCE-INDEX-CARRY-001`, `S11-ROUTE-READABILITY-CARRY-001`).
  Verified in the Deferred Items table.
- AC8 — Preserves carried conditions unchanged: S8-QA-001-W1 OPEN,
  QA-COND-0005 accepted-risk, QA-COND-0006 accepted-risk / deferred,
  11 ignored D-5 tests, HUD timer eyeball deferred, placeholder /
  friend-game art scope `PAW-TD-*-a` accept-risk. Verified in the
  Carried Conditions table.
- AC9 — Preserves friend-game-lite non-claims: no public release / no
  release-candidate / no full-game / no broad / Standard-tier
  accessibility / no playtest / fun-hypothesis / no full
  playable-client manual-QA / no final-art / no asset-production
  completion. Verified in the Non-Claims section.

Files mutated by PROMPT 781:

- `production/sprint-status.yaml` — `S11-EVIDENCE-INDEX-CARRY-001` row
  `status: ready -> done`; `completed: "2026-05-13"`; appended PROMPT 781
  /story-done verdict note preserving AC verification and every non-claim.
- `production/session-state/active.md` — PROMPT 781 CURRENT-STATE banner
  prepended above the PROMPT 780 banner; prior banners preserved as
  HISTORICAL.
- `production/session-state/codex-orchestrator-state.md` — current operating
  rules updated; this PROMPT 781 disposition section prepended above the
  PROMPT 780 disposition section.

Forbidden / not-run by PROMPT 781: `/dev-story`, `/story-readiness`,
`/smoke-check`, `/team-qa`, `/gate-check`, `/qa-plan`. PROMPT 781 did NOT
modify production code under `client/` / `server/` / `shared/` / `tests/`,
did NOT modify `production/stage.txt`, did NOT modify `production/sprints/sprint-11.md`,
did NOT modify `.claude/settings.json`, did NOT modify `reports/`, did NOT
modify `.octogent/`, did NOT modify `.claude/scheduled_tasks.lock`, did NOT
modify `.gitignore`. No public release claim. No release-candidate claim. No
full-game-completion claim. No broad / Standard-tier accessibility-completion
claim. No playtest / fun-hypothesis validation claim. No full playable-client
manual-QA claim. No final-art / asset-production-completion claim. No Sprint
10 close-out disposition modified. No Sprint 11 close-out claim. No retry of
the Polish->Release gate-check.

Sprint 11 Must Have status after PROMPT 781: 2/6 `done`
(`S11-DOC-HYGIENE-CARRY-001`, `S11-EVIDENCE-INDEX-CARRY-001`), 4/6 `ready`
(`S11-DRAG-RUNTIME-RETEST-001`, `S11-TD-FIXTURE-HAND-UI-ONENTER-001`,
`S11-TD-IGNORED-D5-TRIAGE-001`, `S11-ROUTE-READABILITY-CARRY-001`). All
carried conditions preserved unchanged (S8-QA-001-W1 OPEN, QA-COND-0005
accepted-risk, QA-COND-0006 accepted-risk / deferred, 11 ignored D-5 tests
from smoke retry-7 W1, HUD timer eyeball deferred, placeholder / friend-game
art scope PAW-TD-*-a accept-risk).

### PROMPT 780 /story-done Disposition — S11-DOC-HYGIENE-CARRY-001 (2026-05-13)

Authoritative Sprint 11 row `S11-DOC-HYGIENE-CARRY-001` closed by `/story-done`
in PROMPT 780. Source-of-truth at run: `origin/main@d36bbbd`. Deliverable
verified on `main` at `0d19690` (PROMPT 770, 2026-05-13). Acceptance-criterion
verification (read-only against `origin/main@0d19690`):

- AC1 — `docs/architecture/adr-011-reconnect-snapshot.md:173` reads
  `TR-NP-006: Live messages destined for the reconnecting player ...`
  (was `TR-NP-04`). Verified via `git show 0d19690 -- docs/architecture/adr-011-reconnect-snapshot.md`.
- AC2 — `docs/architecture/adr-011-reconnect-snapshot.md:810` traceability-matrix
  row reads `TR-NP-006 — Live messages held until snapshot delivered`
  (was `TR-NP-04`). Verified via the same diff.
- AC3 — `design/gdd/network-protocol.md` Rule 7 carries the
  `See docs/architecture/adr-011-reconnect-snapshot.md (ADR-011) ... mandatory
  send order (S2CHandshake → S2CGameSnapshot → S2CObjectiveIdentities →
  S2CPhaseChanged) ... ReconnectTracker.deferred_queue / snapshot_sent ...
  TR-NP-006` breadcrumb. Verified via the same diff.
- AC4 — No protocol or architecture decision changed; only literal ID
  corrections + a cross-reference breadcrumb. No normative wire or behavior
  text rewritten. Verified by inspecting the full diff of `0d19690`.
- AC5 — Doc-only sweep. No code under `client/` / `server/` / `shared/` /
  `tests/`. Verified via the file list of `0d19690` (only
  `docs/architecture/adr-011-reconnect-snapshot.md`,
  `design/gdd/network-protocol.md`,
  `production/session-state/active.md`,
  `production/session-state/codex-orchestrator-state.md`).

Files mutated by PROMPT 780:

- `production/sprint-status.yaml` — `S11-DOC-HYGIENE-CARRY-001` row
  `status: ready -> done`; `completed: "2026-05-13"`; appended PROMPT 780
  /story-done verdict note preserving AC verification and every non-claim.
- `production/session-state/active.md` — PROMPT 780 CURRENT-STATE banner
  prepended above the PROMPT 774 banner; prior banners preserved as
  HISTORICAL.
- `production/session-state/codex-orchestrator-state.md` — current operating
  rules updated; this PROMPT 780 disposition section appended.

Forbidden / not-run by PROMPT 780: `/dev-story`, `/story-readiness`,
`/smoke-check`, `/team-qa`, `/gate-check`, `/qa-plan`. PROMPT 780 did NOT
modify production code under `client/` / `server/` / `shared/` / `tests/`,
did NOT modify `production/stage.txt`, did NOT modify `production/sprints/sprint-11.md`,
did NOT modify `.claude/settings.json`, did NOT modify `reports/`, did NOT
modify `.octogent/`, did NOT modify `.claude/scheduled_tasks.lock`, did NOT
modify `.gitignore`. No public release claim. No release-candidate claim. No
full-game-completion claim. No broad / Standard-tier accessibility-completion
claim. No playtest / fun-hypothesis validation claim. No full playable-client
manual-QA claim. No final-art / asset-production-completion claim. No Sprint
11 close-out claim. No retry of the Polish->Release gate-check.

Sprint 11 Must Have status after PROMPT 780: 1/6 `done`
(`S11-DOC-HYGIENE-CARRY-001`), 5/6 `ready` (`S11-DRAG-RUNTIME-RETEST-001`,
`S11-TD-FIXTURE-HAND-UI-ONENTER-001`, `S11-TD-IGNORED-D5-TRIAGE-001`,
`S11-EVIDENCE-INDEX-CARRY-001`, `S11-ROUTE-READABILITY-CARRY-001`). All
carried conditions preserved unchanged (S8-QA-001-W1 OPEN, QA-COND-0005
accepted-risk, QA-COND-0006 accepted-risk / deferred, 11 ignored D-5 tests
from smoke retry-7 W1, HUD timer eyeball deferred, placeholder / friend-game
art scope PAW-TD-*-a accept-risk).

### Sprint 10 Polish Close-Out Disposition (PROMPT 763, 2026-05-13)

Sprint 10 was closed `closed-with-conditions` at `origin/main@a6132d7` as
Polish / friend-game-lite paperwork only. 6/6 Must-Have and 2/3 Should-Have
stories were already `done` on origin/main; the producer + qa-lead read-only
review pair both returned APPROVE_WITH_NOTES. The three remaining `ready`
rows were dispositioned as follows — they were NOT silently dropped:

- **S10-TD-003 Doc hygiene tech-debt sweep** → DEFERRED to Sprint 11 planning.
  Partially satisfied: `App::add_message` idempotency correction is on main
  (Bevy 0.18 fact verified at `bevy_app-0.18.1/src/sub_app.rs:358`).
  Outstanding: ADR-011 still contains literal `TR-NP-04` at
  `docs/architecture/adr-011-reconnect-snapshot.md:173` and `:810`; Network
  Protocol Rule 7 still lacks the `ADR-011` breadcrumb. Carry into Sprint 11.
- **S10-N1 Sprint 10 evidence index** → DEFERRED to Sprint 11 planning.
  Per-story evidence files exist (HUD chrome, shop/auction chrome, lobby
  chrome) but no `production/qa/evidence/sprint-10-evidence-index.md`
  aggregator was authored on origin/main.
- **S10-N2 Friend-game route readability notes** → DEFERRED to Sprint 11
  planning. No `sprint-10-readability*.md` or "route readability" file exists
  under `production/ux/`, `design/ux/`, or `production/qa/`.

All three are also recorded as deferred items in
`production/qa/team-qa-sprint-10-2026-05-11.md` Condition C-5 and
`production/gate-checks/gate-polish-release-2026-05-12.md` Recommendation 1.

The PROMPT 761 Polish->Release gate-check verdict `FAIL` is preserved as
evidence — do not retry the Polish->Release gate-check until release-scope
artifacts (final art, manual-QA sign-off, accessibility completion, playtest
evidence) actually exist on `main`.

Carried forward unchanged at close-out: S8-QA-001-W1 manual/browser
two-client GAME_OVER gap (open); QA-COND-0005 Standard-tier accessibility
(accepted-risk friend-game scope); QA-COND-0006 playtest/fun-hypothesis
validation (accepted-risk / deferred); 11 ignored D-5 tests pending owner
review (smoke retry-7 W1); HUD timer eyeball visual check deferred (smoke
retry-7 W2); placeholder / friend-game art scope (PAW-TD-*-a accept-risk on
placeholder PNGs).

Explicitly NOT claimed by this close-out: public release readiness,
release-candidate readiness, full game completion, broad Standard-tier
accessibility completion, playtest / fun-hypothesis validation, full
playable-client manual QA, final-art / asset-production completion.

Files touched by PROMPT 763: `production/sprint-status.yaml`,
`production/sprints/sprint-10.md`, `production/session-state/active.md`,
`production/session-state/codex-orchestrator-state.md`. No code under
`client/`, `server/`, `shared/`, `tests/`, no `.octogent/` changes, no
`production/stage.txt` change, no smoke / gate-check / QA sign-off /
`/dev-story` run, no Sprint 11 activation.

### Sprint 11 QA Plan Authoring (PROMPT 774, 2026-05-13)

PROMPT 774 authored the Sprint 11 QA plan as required by
`production/sprint-status.yaml` `sprint_11_activation.outstanding_before_dev_story[0]`.
Source-of-truth at authoring: `origin/main@07aafe2` (PROMPT 773's commit).

Scope and disposition:

- `production/qa/qa-plan-sprint-11.md` (NEW): covers all 16 Sprint 11 rows.
  6 Must Have: `S11-DRAG-RUNTIME-RETEST-001` (Integration — manual runtime
  evidence; story file at PROMPT 766 READY), `S11-TD-FIXTURE-HAND-UI-ONENTER-001`
  (Integration test-only; story file at PROMPT 767 content-ready, formal
  `/story-readiness` pending), `S11-TD-IGNORED-D5-TRIAGE-001` (Config/Data
  triage doc; no story file required per Sprint 11 plan), and three
  paperwork-carry rows tracked as `ready` with deliverables LANDED on `main`
  at `0d19690` / `348084b` / `d3ee8df` (`S11-DOC-HYGIENE-CARRY-001` /
  `S11-EVIDENCE-INDEX-CARRY-001` / `S11-ROUTE-READABILITY-CARRY-001`;
  `/story-done` NOT run, per the no-invent-closure rule). 4 Should Have rows
  tracked as conditional (blocked until story file + `/story-readiness`):
  `S11-TD-FIXTURE-D-RESIDUALS-001`, `S11-HU-PHASE-IDEMPOTENCY-001`,
  `S11-SERVER-POOL-INIT-LOG-GUARD-001`, `S11-HUD-TIMER-EYEBALL-VISUAL-001`.
  6 Nice to Have rows tracked as backlog-verification (blocked until story
  file authored): `S11-TD-CARGO-DISK-USAGE-001`, `S11-TD-CARGO-PDB-LIMIT-001`,
  `S11-OPS-ORCHESTRATOR-LOCK-001`, `S11-OPS-GH-CLI-001`,
  `S11-LOBBY-UX-CONFIRM-STATE-001`, `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`.

- Plan content: required evidence per story; required regression / test
  commands per story type (Logic / Integration / Visual / UI / Config-Data);
  manual runtime evidence expectations for `S11-DRAG-RUNTIME-RETEST-001`
  (S1-S5 grey-square attribution truth-table across drag-attempts A / B / C / D,
  4-way disposition `{bug-reproduced, bug-fixed, cannot-reproduce,
  third-party-limitation}`, `RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=info,server::game=info`
  capture, 1.0-day time-box) and for `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  (cosmetic screenshot evidence for `DraftInitial` 45s / `DraftShop` 30s /
  `Placement` 10-12s); pre-`/dev-story` prerequisites tracker; cross-cutting
  workspace gates (`cargo fmt --check`, `cargo test --workspace --tests
  --no-fail-fast`, workspace ignored-count regression check); smoke-test
  scope (verified via `/smoke-check sprint` in a separate prompt — not
  this plan); no playtest sessions required (QA-COND-0006 remains
  accept-risk / deferred); Definition of Done for the sprint.

- Carried conditions and non-claims preserved verbatim:
  S8-QA-001-W1 manual/browser two-client GAME_OVER gap (OPEN);
  QA-COND-0005 Standard-tier accessibility (accepted-risk friend-game
  scope); QA-COND-0006 playtest/fun-hypothesis validation (accepted-risk /
  deferred); 11 ignored D-5 tests carried until per-test disposition under
  `S11-TD-IGNORED-D5-TRIAGE-001`; HUD timer eyeball visual check (W2)
  carried until `S11-HUD-TIMER-EYEBALL-VISUAL-001` evidence captured;
  placeholder / friend-game art scope (PAW-TD-*-a accept-risk on placeholder
  PNGs). Explicitly NOT claimed: public release readiness, release-candidate
  readiness, full game completion, broad / Standard-tier accessibility
  completion, playtest / fun-hypothesis validation, full playable-client
  manual QA, final-art / asset-production completion.

- Unlock effect: with this plan on `main`, `/dev-story` is now authorised
  against any Sprint 11 row that **also** has (a) story file existing, and
  (b) `/story-readiness` PASS recorded. At this moment only
  `S11-DRAG-RUNTIME-RETEST-001` satisfies both gates; the playable-client
  fixture story has the file but the formal `/story-readiness` verdict is
  still pending in a separate prompt.

PROMPT 774 did NOT run `/dev-story`, `/story-readiness`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/story-done`. PROMPT 774 did NOT modify
production code under `client/`, `server/`, `shared/`, `tests/`. PROMPT 774
did NOT modify `production/sprint-status.yaml`, `production/sprints/sprint-11.md`,
`production/stage.txt`, `.claude/settings.json`, `reports/`,
`.claude/scheduled_tasks.lock`, or `.octogent/`. No release / release-candidate
/ full-game / broad-accessibility / playtest / full-manual-QA / final-art
claim. PROMPT 761 Polish->Release gate-check FAIL evidence preserved
unchanged. `production/stage.txt` reads `Polish` and is unchanged.

Files touched by PROMPT 774: `production/qa/qa-plan-sprint-11.md` (NEW),
`production/session-state/active.md` (banner prepended),
`production/session-state/codex-orchestrator-state.md` (this section).

### Sprint 11 Activation Paperwork (PROMPT 773, 2026-05-13)

PROMPT 773 activated Sprint 11 as a **Polish-stage** sprint (not Release).
Source-of-truth at activation: `origin/main@d3ee8df`. Activation policy and
scope:

- `production/sprint-status.yaml`: `sprint:` flipped from `10` to `11`;
  `status:` flipped from `closed-with-conditions` to `active`; `goal:`,
  `scope:`, `start:` (`2026-06-04`), `end:` (`2026-06-17`), `generated:`,
  `updated:` rewritten for Sprint 11; `stage:` UNCHANGED (`Polish`).
  `activation:` block rewritten for Sprint 11 (date 2026-05-13, prompt 773,
  source-of-truth `origin/main@d3ee8df`, basis enumerated, `not_release_activation`
  field added with explicit no-Release language). `previous_sprint_closeout:`
  block rewritten to summarise Sprint 10 close-out (PROMPT 763,
  `origin/main@a6132d7`, `closed-with-conditions`, full `carried_into_sprint_11:`
  list including S8-QA-001-W1 / QA-COND-0005 / QA-COND-0006 / 11 ignored D-5
  tests / HUD timer eyeball / placeholder art scope / explicit no-claims).
  `stories:` block: prior Sprint 10 rows removed (preserved in git history
  and summarised under `sprint_10_closeout:`); replaced with 16 Sprint 11
  rows — 6 Must Have, 4 Should Have, 6 Nice to Have. The prior `next_sprint:`
  draft block replaced with `sprint_11_activation:` recording the activation
  facts and the outstanding-before-`/dev-story` list (Sprint 11 QA plan, formal
  `/story-readiness` on the playable-client story, Should/Nice story files).
  `sprint_10_closeout:` block preserved unchanged. `presentation_asset_wiring:`,
  `coordination:`, `forbidden_runs_in_activation:`, `carried_conditions:` blocks
  preserved unchanged.
- Sprint 11 Must Have row dispositions: all six rows are `status: ready`.
  - `S11-DRAG-RUNTIME-RETEST-001` — `file: production/epics/hand-ui/story-018-drag-runtime-retest.md`
    (PROMPT 766, `/story-readiness` READY); blocker: Sprint 11 QA plan
    required before `/dev-story`.
  - `S11-TD-FIXTURE-HAND-UI-ONENTER-001` —
    `file: production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`
    (PROMPT 767, content-ready, formal readiness verdict pending); blocker:
    Sprint 11 QA plan + formal `/story-readiness` verdict.
  - `S11-TD-IGNORED-D5-TRIAGE-001` — no story file required per Sprint 11
    draft (triage doc authored during `/dev-story`); blocker: Sprint 11 QA
    plan required.
  - `S11-DOC-HYGIENE-CARRY-001` — deliverable LANDED at `0d19690`
    (PROMPT 770); `/story-done` NOT run — no-invent-closure rule applied.
  - `S11-EVIDENCE-INDEX-CARRY-001` — deliverable LANDED at `348084b`
    (PROMPT 771); `/story-done` NOT run — no-invent-closure rule applied.
  - `S11-ROUTE-READABILITY-CARRY-001` — deliverable LANDED at `d3ee8df`
    (PROMPT 772); `/story-done` NOT run — no-invent-closure rule applied.
- Sprint 11 Should Have / Nice to Have rows are `status: blocked` with a
  uniform blocker note: "No story file authored; /story-readiness pending;
  Sprint 11 QA plan also required before /dev-story." This tracks them
  without expanding scope.
- `production/sprints/sprint-11.md`: header flipped from
  `Sprint 11 -- DRAFT (dates TBD at activation)` to
  `Sprint 11 -- ACTIVE (Polish stage)`. Status line flipped from
  `draft / NOT active` to `active`. Dates locked
  (`2026-06-04 -> 2026-06-17`). Carry-deliverable-landed evidence and
  implementation-story-file authoring noted under the activation header.
  Closing paragraph rewritten to record PROMPT 773 activation.
- `production/session-state/active.md`: PROMPT 773 banner prepended above
  PROMPT 772 banner.

PROMPT 773 did NOT run `/dev-story`, `/story-readiness`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/story-done`, `/qa-plan`. PROMPT 773 did NOT
modify production code under `client/`, `server/`, `shared/`, `tests/`.
PROMPT 773 did NOT modify `production/stage.txt`, `.claude/settings.json`,
`reports/`, `.claude/scheduled_tasks.lock`, `.octogent/`. PROMPT 773 did NOT
modify the PROMPT 761 Polish->Release gate-check FAIL evidence; activation
is explicitly Polish, not Release.

Carried forward unchanged: S8-QA-001-W1 manual/browser two-client GAME_OVER
gap (OPEN); QA-COND-0005 Standard-tier accessibility (accepted-risk
friend-game scope); QA-COND-0006 playtest/fun-hypothesis validation
(accepted-risk / deferred); 11 ignored D-5 tests (folded into Must Haves);
HUD timer eyeball visual check (folded into Should Have); placeholder /
friend-game art scope (PAW-TD-*-a accept-risk). No public release
readiness, release-candidate readiness, full game completion, broad /
Standard-tier accessibility completion, playtest / fun-hypothesis
validation, full playable-client manual QA, or final-art /
asset-production completion is claimed.

Next launchable prompts after PROMPT 773:

1. `/qa-plan sprint` for Sprint 11 — required before any Sprint 11
   `/dev-story` runs.
2. `/story-readiness` on
   `production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`
   — formal verdict pending.
3. `/story-done` on the three landed paperwork carries
   (`S11-DOC-HYGIENE-CARRY-001`, `S11-EVIDENCE-INDEX-CARRY-001`,
   `S11-ROUTE-READABILITY-CARRY-001`), each as a separate prompt; safe to
   dispatch in parallel — they touch disjoint evidence files.

### Sprint 11 Route Readability Carry — `S11-ROUTE-READABILITY-CARRY-001` (PROMPT 772, 2026-05-13)

PROMPT 772 landed the doc-only `S11-ROUTE-READABILITY-CARRY-001` carry from
deferred Sprint 10 nice-to-have `S10-N2` (per PROMPT 763 close-out and PROMPT
764 Sprint 11 draft plan). Authored the friend-game route readability notes
file at `production/qa/evidence/sprint-10-route-readability-notes.md` (NEW)
covering eight routes: Lobby, Hand / Drag, Draft Grid, Shop, Auction, Board,
HUD / Timer, and Result / Close-Out. Each observation is captured as either an
`already-tracked` cross-reference to an existing Sprint 11 backlog row (e.g.
`S11-DRAG-RUNTIME-RETEST-001`, `S11-UX-DRAFT-GRID-CENTERED-MODAL`,
`S11-UX-AUCTION-FEATURED-CARD`, `S11-UX-AUCTION-FREE-GOLD-COUNTERS`,
`S11-UX-HUD-TOP-STRIP-LAYOUT`, `S11-UX-BOARD-RENDERING-SPEC`,
`S11-HUD-TIMER-EYEBALL-VISUAL-001`, `S11-LOBBY-UX-CONFIRM-STATE-001`,
`S8-QA-001-W1`) or a `future-story-candidate` slug that does NOT yet have a
story file. No row authorises immediate implementation; a separate prompt with
its own story + `/story-readiness` is required before any change lands.

The notes file explicitly preserves friend-game scope: it does NOT propose
broad Standard-tier accessibility completion, does NOT claim closure of
`QA-COND-0005` (Standard-tier accessibility, accepted-risk), does NOT claim
closure of `QA-COND-0006` (playtest / fun-hypothesis validation, accepted-risk
/ deferred), does NOT claim closure of `S8-QA-001-W1` (manual / browser
two-client GAME_OVER gap, OPEN), and does NOT claim closure of any other
carried condition. Final-art replacement remains accept-risk under `PAW-TD-*-a`.

Sprint 11 remains `draft / not_active`: `production/sprint-status.yaml`
`sprint:` is unchanged, `production/sprints/sprint-11.md` is unchanged,
`production/stage.txt` reads `Polish` and is unchanged, the PROMPT 761
Polish->Release gate-check `FAIL` is preserved as evidence, and Sprint 10
disposition stays `closed-with-conditions` per PROMPT 763. No code under
`client/` / `server/` / `shared/` / `tests/` modified. No smoke, gate-check,
QA sign-off, `/dev-story`, `/story-readiness`, or `/story-done` run. No
release artifact authored and no release claim.

With PROMPT 770 (`S11-DOC-HYGIENE-CARRY-001` landed at `0d19690`), PROMPT 771
(`S11-EVIDENCE-INDEX-CARRY-001` landed at `348084b`), and PROMPT 772
(`S11-ROUTE-READABILITY-CARRY-001`), all three Sprint 11 draft paperwork-carry
Must Haves derived from Sprint 10 deferrals now have their outstanding
deliverables on `main`. Marking the Sprint 11 rows as outstanding **vs** done
is a Sprint 11 activation-time decision — PROMPT 772 did NOT mutate
`production/sprint-status.yaml` or `production/sprints/sprint-11.md`. Files
touched by PROMPT 772: `production/qa/evidence/sprint-10-route-readability-notes.md`
(NEW), `production/session-state/active.md` (banner), and
`production/session-state/codex-orchestrator-state.md` (this section).

### Sprint 11 Evidence Index Carry — `S11-EVIDENCE-INDEX-CARRY-001` (PROMPT 771, 2026-05-13)

PROMPT 771 landed the doc-only `S11-EVIDENCE-INDEX-CARRY-001` carry from
deferred Sprint 10 nice-to-have `S10-N1` (per PROMPT 763 close-out and PROMPT
764 Sprint 11 draft plan). Authored the Sprint 10 evidence aggregator index at
`production/qa/evidence/sprint-10-evidence-index.md` (NEW). The aggregator
collates per-story status (Must / Should / Nice-to-Have) with integration
commit hashes and primary evidence paths; records the smoke retry-7 PASS WITH
WARNINGS at `production/qa/smoke-sprint-10-2026-05-12-retry-7.md`; records
the /team-qa APPROVED WITH CONDITIONS at
`production/qa/team-qa-sprint-10-2026-05-11.md`; records the PROMPT 761
Polish->Release gate-check `FAIL` at
`production/gate-checks/gate-polish-release-2026-05-12.md`; records the three
Sprint 10 deferred items (S10-TD-003, S10-N1, S10-N2) and their Sprint 11
draft carry IDs (`S11-DOC-HYGIENE-CARRY-001`, `S11-EVIDENCE-INDEX-CARRY-001`,
`S11-ROUTE-READABILITY-CARRY-001`); and preserves every carried condition
(S8-QA-001-W1 OPEN, QA-COND-0005 accepted-risk, QA-COND-0006
accepted-risk / deferred, 11 ignored D-5 tests from smoke retry-7 W1, HUD
timer eyeball visual check deferred from W2, placeholder / friend-game art
scope via PAW-TD-*-a accept-risk on placeholder PNGs) along with the standard
friend-game-lite non-claims (no release / no release-candidate / no full-game
completion / no broad / Standard-tier accessibility / no playtest validation /
no full manual QA / no final-art / asset-production claim). The aggregator is
read-only over the underlying evidence — it does not modify, supersede, or
reclassify any existing artefact. Authoritative status remains
`production/sprint-status.yaml`. Sprint 11 remains `draft / not_active`:
`production/sprint-status.yaml` `sprint:` is unchanged, `production/sprints/sprint-11.md`
is unchanged, `production/stage.txt` reads `Polish` and is unchanged, the
PROMPT 761 Polish->Release gate-check FAIL is preserved as evidence, and
Sprint 10 disposition stays `closed-with-conditions` per PROMPT 763. No code
under `client/` / `server/` / `shared/` / `tests/` modified. No smoke,
gate-check, QA sign-off, `/dev-story`, `/story-readiness`, `/story-done`, or
`/qa-plan` run. No release artifact authored and no release claim. Marking
the Sprint 11 row `done` vs outstanding is a Sprint 11 activation-time
decision — PROMPT 771 did NOT mutate `production/sprint-status.yaml` or
`production/sprints/sprint-11.md`. Files touched by PROMPT 771:
`production/qa/evidence/sprint-10-evidence-index.md` (NEW),
`production/session-state/active.md`,
`production/session-state/codex-orchestrator-state.md`.

### Sprint 11 Doc Hygiene Carry — `S11-DOC-HYGIENE-CARRY-001` (PROMPT 770, 2026-05-13)

PROMPT 770 landed the doc-only `S11-DOC-HYGIENE-CARRY-001` carry from
deferred `S10-TD-003` (PROMPT 763). Two literal `TR-NP-04` references in
`docs/architecture/adr-011-reconnect-snapshot.md` (lines 173 and 810) were
corrected to `TR-NP-006` — the TR-registry-canonical ID for the deferred-queue
/ snapshot-first / `snapshot_sent` invariant (`docs/architecture/tr-registry.yaml`
TR-NP-006 covering `NP-9, NP-16, NP-17, NP-18, NP-20, NP-21, NP-22`). Network
Protocol Rule 7 (`design/gdd/network-protocol.md`) gained an `ADR-011`
breadcrumb pointing at the full reconnect flow, mandatory send order, and the
`ReconnectTracker.deferred_queue` / `snapshot_sent` gating that enforces
TR-NP-006. No protocol or architecture decision is changed; no normative wire
or behavior text was rewritten. Sprint 11 remains `draft / not_active`:
`production/sprint-status.yaml` `sprint:` is unchanged, `production/sprints/sprint-11.md`
is unchanged, `production/stage.txt` reads `Polish` and is unchanged, the
PROMPT 761 Polish->Release gate-check FAIL is preserved as evidence, and
Sprint 10 disposition stays `closed-with-conditions` per PROMPT 763. No code
under `client/` / `server/` / `shared/` / `tests/` modified. No smoke,
gate-check, QA sign-off, `/dev-story`, `/story-readiness`, or `/story-done`
run. No release artifact authored and no release claim. Evidence is the diff
itself plus this paragraph (per the Sprint 10 row spec carried into Sprint 11).
Files touched by PROMPT 770: `docs/architecture/adr-011-reconnect-snapshot.md`,
`design/gdd/network-protocol.md`, `production/session-state/active.md`,
`production/session-state/codex-orchestrator-state.md`.

### Sprint 11 DRAFT Story Authoring — `S11-TD-FIXTURE-HAND-UI-ONENTER-001` (PROMPT 767, 2026-05-13)

PROMPT 767 authored the Sprint 11 draft Must Have story file for
`S11-TD-FIXTURE-HAND-UI-ONENTER-001` at
`production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`
(NEW). Sprint 11 remains `draft` per PROMPT 764; **Sprint 11 was NOT
activated by PROMPT 767**. `production/sprint-status.yaml` `sprint:`
field and active-row set are unchanged. `production/stage.txt` reads
`Polish` and is unchanged. `production/sprints/sprint-11.md` is
unchanged. PROMPT 761 Polish->Release gate-check FAIL evidence is
preserved.

Story scope (Layer 4 of the same fixture cascade that closed
`S10-TD-001` under `story-009-test-fixture-cascade-fail-repair.md`;
diagnosis + fixture-only repair):

- Identifies the cluster of ignored tests from smoke retry-7 W1
  (`production/qa/smoke-sprint-10-2026-05-12-retry-7.md` lines 60-74)
  whose owner-named `#[ignore]` comments point at the same root cause:
  `spawn_hand_ui` not firing on `OnEnter(InSession)` in `MinimalPlugins`
  fixtures → `HandUiEntities` never inserted → downstream entity-presence
  assertions fail. Six tests are explicitly enumerated; a seventh
  referenced in the PROMPT 759 closeout / PROMPT 762 candidate-backlog
  capture may have shifted disposition between retry-5 and retry-7, and
  is recorded as a "Cluster count note" for diagnosis to confirm or
  refute.
- Scopes repair to `tests/` plus a single `#[cfg(test)]`-gated test-only
  helper (precedent: `placeholder_assets_for_tests()` from S10-TD-001
  Layer 3) plus a pattern doc at
  `docs/architecture/test-fixture-patterns.md` (or appended location).
  AC6 enforces zero production code change in `client/src/`,
  `server/src/`, or `shared/src/` outside the helper exception. If
  diagnosis surfaces a production-runtime regression, the disposition
  is to author a separate follow-on production-fix story id and
  reference it from this story's evidence document — the production
  code change does NOT land under this story id.
- AC2 requires either (a) workspace ignored-count drop by N (= tests
  un-`#[ignore]`d) OR (b) explicit owner-named disposition comment on
  every retained `#[ignore]` pointing at the resolving story id (this
  story id, the referenced follow-on production-fix story id, or
  `S11-TD-IGNORED-D5-TRIAGE-001`). No silent retention.
- AC7 explicitly preserves Sprint 11 draft status:
  `production/sprint-status.yaml`, `production/sprints/sprint-11.md`,
  and `production/stage.txt` are not modified under this story.
- Evidence document slot reserved at
  `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
  for population by the implementation prompt(s).
- Story status authored as `Draft -- Sprint 11 draft Must Have, NOT
  activated`. `/story-readiness` is the next step **after** Sprint 11
  activation (separate prompt).

EPIC index update: `production/epics/playable-client/EPIC.md` Stories
table backfilled with rows 009 (S10-TD-001 Test-Fixture Cascade-Fail
Repair — Complete), 010 (S10-TD-002 Plugin Registration Audit), and
011 (the new S11 draft story). Rows 009 + 010 were authored
retroactively because the story files existed on disk but had not been
registered in the EPIC index. Status-line note updated to mention
Sprint 10 tech-debt + Sprint 11 draft tech-debt.

Sprint 11 Must Have story-file authoring status after PROMPT 767:

| Must Have ID | Required story file | Status |
|--------------|---------------------|--------|
| `S11-DRAG-RUNTIME-RETEST-001` | `production/epics/hand-ui/story-018-drag-runtime-retest.md` | ✅ Authored by PROMPT 766 |
| `S11-TD-FIXTURE-HAND-UI-ONENTER-001` | `production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md` | ✅ Authored by PROMPT 767 |
| `S11-TD-IGNORED-D5-TRIAGE-001` | No new story file required (triage doc) | n/a |
| `S11-DOC-HYGIENE-CARRY-001` | No new story file required (doc-only sweep) | n/a |
| `S11-EVIDENCE-INDEX-CARRY-001` | No new story file required (evidence aggregator) | n/a |
| `S11-ROUTE-READABILITY-CARRY-001` | No new story file required (notes file) | n/a |

Both Lane A story-authoring slots (PROMPT 766 + PROMPT 767) are now
filled. Remaining Sprint 11 Must Have artifacts (triage doc, doc-hygiene
sweep, evidence-index aggregator, route-readability notes) are
paperwork that lands at activation time via `/sprint-plan sprint-11` +
`/qa-plan sprint` + the subsequent `/dev-story` dispatches.

Files touched by PROMPT 767:
`production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`
(NEW),
`production/epics/playable-client/EPIC.md` (Stories table rows 009 +
010 + 011 added; status-line description updated),
`production/session-state/active.md` (PROMPT 767 banner prepended
above PROMPT 766 banner),
`production/session-state/codex-orchestrator-state.md` (this section).
No code under `client/`, `server/`, `shared/`, `tests/`. No
`.octogent/` change. No `.gitignore` change. No `production/stage.txt`
change. No `production/sprint-status.yaml` change. No
`production/sprints/sprint-11.md` change. No smoke / gate-check / QA
sign-off / `/dev-story` / `/story-done` run. No Sprint 11 activation.
No release artifact authored. No release claim.

### Sprint 11 DRAFT Story Authoring — `S11-DRAG-RUNTIME-RETEST-001` (PROMPT 766, 2026-05-13)

PROMPT 766 authored the Sprint 11 draft Must Have story file for
`S11-DRAG-RUNTIME-RETEST-001` at
`production/epics/hand-ui/story-018-drag-runtime-retest.md` (NEW). Sprint 11
remains `draft` per PROMPT 764; **Sprint 11 was NOT activated by PROMPT 766**.
`production/sprint-status.yaml` `sprint:` field and active-row set are
unchanged. `production/stage.txt` reads `Polish` and is unchanged.
`production/sprints/sprint-11.md` is unchanged. PROMPT 761 Polish->Release
gate-check FAIL evidence is preserved.

Story scope (runtime-evidence retest, NOT a code-change story):

- Defines the exact `RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=info,server::game=info` invocation for the runtime trace.
- Defines a manual two-client friend-game route with four drag-attempts:
  A (standard unit → BoardCell), B (Instant → fan plate), C (cancel onto
  empty space), D (invalid board cell).
- Defines the S1-S5 grey-square attribution truth-table (5 stages × 4
  drag-attempts = 20 cells to fill PASS / FAIL / NOT-OBSERVED + evidence
  pointer). Stages map to the 5 tracing sites landed at `7e0c663` per
  PROMPT 706 / 709.
- Acceptance criteria (`HU-DRAG-RT-01..08`) distinguish four disposition
  outcomes for the test-green/runtime-broken divergence:
  1. **Bug reproduced** — repro identified; follow-on repair story
     authored; **no repair commit lands inside this story**.
  2. **Bug fixed** — cumulative PROMPT 696 / 697 / 706 / 709 work
     resolved it; truth-table locked as PASS; evidence note records the
     disposition.
  3. **Cannot reproduce with evidence** — time-box exhausted (1.0 day);
     truth-table locked as best-effort with NOT-OBSERVED rows; follow-on
     diagnostic-only story authored with tighter capture spec.
  4. **Third-party / platform limitation** — divergence is browser /
     OS / GPU / input-device specific; documented with no-claim note.
- Explicitly forbids edits under `client/` / `server/` / `shared/` /
  `tests/` as part of `/dev-story` on this story (HU-DRAG-RT-05).
- Explicitly forbids introducing client-side optimistic authority for
  stage / activate / submit (HU-DRAG-RT-06; ADR-002 + ADR-009 binding).
- Preserves the no-claim banner (HU-DRAG-RT-07): no public release
  claim, no full manual QA, no Standard-tier accessibility, no playtest
  validation, no full game completion, no S8-QA-001-W1 / QA-COND-0005 /
  QA-COND-0006 closure.
- Preserves Sprint 11 draft status (HU-DRAG-RT-08): no edits to
  `production/sprint-status.yaml`, `production/stage.txt`, or
  `production/sprints/sprint-11.md`.

EPIC index update: `production/epics/hand-ui/EPIC.md` Stories table gained
row 018 with `Status: Draft (Sprint 11 not activated)` and ADRs
`ADR-021, ADR-002, ADR-009`. Dependency-order line gained
`017 → 018`. Counts note clarified — story 018 is a Sprint-11-draft
retest/paperwork row and is not folded into the active completion ratios;
stories 016 / 017 predate the last count refresh and are also not
folded — see those files for their authoritative status.

Sprint 11 Must Have story-file authoring status after PROMPT 766:

| Must Have ID | Required story file | Status |
|--------------|---------------------|--------|
| `S11-DRAG-RUNTIME-RETEST-001` | `production/epics/hand-ui/story-018-drag-runtime-retest.md` | ✅ Authored by PROMPT 766 |
| `S11-TD-FIXTURE-HAND-UI-ONENTER-001` | `production/epics/playable-client/story-XXX-spawn-hand-ui-fixture-cascade.md` | ⏳ Pending (Lane A second author in a separate prompt) |
| `S11-TD-IGNORED-D5-TRIAGE-001` | No new story file required (triage doc) | n/a |
| `S11-DOC-HYGIENE-CARRY-001` | No new story file required (doc-only sweep) | n/a |
| `S11-EVIDENCE-INDEX-CARRY-001` | No new story file required (evidence aggregator) | n/a |
| `S11-ROUTE-READABILITY-CARRY-001` | No new story file required (notes file) | n/a |

Files touched by PROMPT 766: `production/epics/hand-ui/story-018-drag-runtime-retest.md` (NEW), `production/epics/hand-ui/EPIC.md`, `production/session-state/active.md`, `production/session-state/codex-orchestrator-state.md`. No code under `client/`, `server/`, `shared/`, `tests/`. No `.octogent/` change. No `.gitignore` change. No `production/stage.txt` change. No `production/sprint-status.yaml` change. No `production/sprints/sprint-11.md` change. No smoke / gate-check / QA sign-off / `/dev-story` / `/story-done` run. No Sprint 11 activation. No release artifact authored. No release claim.

### Sprint 11 DRAFT Planning Artifacts (PROMPT 764, 2026-05-13)

Sprint 11 was drafted at `origin/main@a6132d7` as paperwork-only planning
artifacts. **Sprint 11 was NOT activated.** Sprint 10 disposition,
`production/stage.txt`, all carried conditions, and the PROMPT 761
Polish->Release gate-check FAIL evidence are unchanged.

Files touched by PROMPT 764: `production/sprints/sprint-11.md` (NEW),
`production/sprint-status.yaml` (`next_sprint:` block flipped from
`not_planned` to `draft` + `updated:` comment appended),
`production/session-state/active.md` (PROMPT 764 banner prepended above
the PROMPT 763 banner), `production/session-state/codex-orchestrator-state.md`
(this section + the Current Operating Rules `Current next move` update).

No code under `client/`, `server/`, `shared/`, `tests/`. No `.octogent/`
changes. No `.gitignore` change. No `production/stage.txt` change. No
smoke / gate-check / QA sign-off / `/dev-story` / `/story-done` run. No
Sprint 11 activation. No release artifact authored. No release claim.

#### Sprint 11 draft top 5 Must Have (PROMPT 764 producer recommendation)

1. `S11-DRAG-RUNTIME-RETEST-001` — HIGH; gameplay-blocking for
   friend-game runtime. Runtime trace never completed across PROMPT 696
   / 697 / 698 / 706 / 709. Locks the S1-S5 grey-square truth-table or
   authors a precise follow-on repro.
2. `S11-TD-FIXTURE-HAND-UI-ONENTER-001` — HIGH; pervasive fixture-design
   gap; 7x `spawn_hand_ui` not firing on `OnEnter(InSession)` in
   `MinimalPlugins` fixtures. Unblocks 7+ ignored tests + future ones.
3. `S11-TD-IGNORED-D5-TRIAGE-001` — HIGH; 11 owner-named `#[ignore]` D-5
   tests from smoke retry-7 W1 triaged per-test (fix / redesign /
   delete) with explicit rationale.
4. `S11-DOC-HYGIENE-CARRY-001` — MEDIUM; S10-TD-003 carry. ADR-011
   `TR-NP-04 -> TR-NP-006` literal corrections at
   `docs/architecture/adr-011-reconnect-snapshot.md:173` and `:810` +
   Rule 7 `ADR-011` breadcrumb in `design/gdd/network-protocol.md`.
5. `S11-EVIDENCE-INDEX-CARRY-001` — MEDIUM; S10-N1 carry. Author
   `production/qa/evidence/sprint-10-evidence-index.md` aggregator
   linking the per-story Sprint 10 evidence files.

(`S11-ROUTE-READABILITY-CARRY-001` is also Must Have as the third S10
carry — folds S10-N2 — but ranks 6th for capacity prioritisation.)

#### Sprint 11 draft Should Have

- `S11-TD-FIXTURE-D-RESIDUALS-001` — `ghost_preview_bridge_test`,
  `snapshot_spawn_test` phase routing, `status_icons` should-panic
  drift, `shop_auction_ui_plugin_scaffold_formulas_test` count drift
  57->66.
- `S11-HU-PHASE-IDEMPOTENCY-001` — client `phase_changed=true` 60Hz
  idempotency tightening.
- `S11-SERVER-POOL-INIT-LOG-GUARD-001` — `init_pool` log before guard
  (W5-fix pattern apply).
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` — smoke retry-7 W2 carry.

#### Sprint 11 draft Nice-to-Have

- `S11-TD-CARGO-DISK-USAGE-001`, `S11-TD-CARGO-PDB-LIMIT-001`,
  `S11-OPS-ORCHESTRATOR-LOCK-001`, `S11-OPS-GH-CLI-001`,
  `S11-LOBBY-UX-CONFIRM-STATE-001`,
  `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`.

#### Sprint 11 draft — wider backlog NOT scheduled into this draft

`S11-TD-NET-001/002/003`, `S11-TD-PRISM-COV-001`,
`S11-TD-HARNESS-MESSAGES-001`, `S11-TD-HARNESS-HANDUI-ENTITIES-001`,
`S11-TD-BOARD-RENDERING-SNAPSHOT-PHASE-COUPLING-001`,
`S11-TD-FIXTURE-MESSAGES-002`, `S11-TD-CI-NORMALIZE-COMMENTS-001`,
`S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`, ConfirmClass intent
chain, cooccupancy panic-guard drift, HudPlugin snapshot.phase fixture
gap, GhostDragStartEvent producer fixture gap, the PROMPT 685 UI
clean-pass 8-story milestone
(`S11-TD-UI-ZINDEX-LAYERS` / `S11-TD-UI-FLEX-STRIPS` /
`S11-UX-HUD-TOP-STRIP-LAYOUT` / `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` /
`S11-UX-HUD-OPP-FIGURINE` / `S11-UX-DRAFT-GRID-CENTERED-MODAL` /
`S11-UX-AUCTION-FEATURED-CARD` / `S11-UX-AUCTION-FREE-GOLD-COUNTERS` /
`S11-UX-LOBBY-CLASS-PICKER` / `S11-UX-LOBBY-BUTTON-HITTARGETS` /
`S11-UX-BOARD-RENDERING-SPEC` / `S11-TD-UI-FONT-CONSTANTS` /
`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`).

These remain in the broader backlog. Producer may pull them into the
draft before activation, or defer to Sprint 12.

#### Sprint 11 draft — suggested first parallel batch after activation

Once Sprint 11 is activated (via `/sprint-plan sprint-11`) and story
files for the two HIGH Must Haves are authored + `/story-readiness`
passes:

- Lane A (story authoring + triage doc skeleton): author
  `S11-DRAG-RUNTIME-RETEST-001` and `S11-TD-FIXTURE-HAND-UI-ONENTER-001`
  story files in parallel with the `S11-TD-IGNORED-D5-TRIAGE-001`
  triage-doc skeleton.
- Lane B (paperwork carries, truly parallel): dispatch
  `S11-DOC-HYGIENE-CARRY-001` (touches
  `docs/architecture/adr-011-*` + `design/gdd/network-protocol.md`),
  `S11-EVIDENCE-INDEX-CARRY-001` (touches
  `production/qa/evidence/sprint-10-evidence-index.md`), and
  `S11-ROUTE-READABILITY-CARRY-001` (touches
  `production/qa/evidence/sprint-10-route-readability-notes.md` or
  equivalent) as three separate small workers. Files are disjoint;
  safe under the 2026-05-13 override (only one shared-status writer at
  a time means `sprint-status.yaml` is OFF-limits for these workers).
- Hold for serial: `/qa-plan sprint`, `/smoke-check`, `/team-qa`,
  `/gate-check`, and all close-out work.

#### Sprint 11 draft — blockers / missing evidence flagged

- No Sprint 11 QA plan yet (`/qa-plan sprint` must run after story
  files exist).
- No Sprint 11 story files yet for the two HIGH Must Haves.
- Runtime trace for drag-and-drop divergence has never been captured
  end-to-end. `S11-DRAG-RUNTIME-RETEST-001` activation should specify
  the exact `RUST_LOG=...` invocation, the friend-game route to
  execute, and the expected truth-table form before worker dispatch.
- Sprint 11 dates are not locked. Producer should lock them at
  activation.

### Orchestrator Response Style

After every user-pasted agent return, lead with the action:

- `CLEAR -- PROMPT N` when the user can close the agent window and no reply is
  needed. Badge/color: green.
- `REPONDRE -- PROMPT N` when the user should paste a reply into that same
  window. Badge/color: yellow.
- `RELANCER -- PROMPT N` when the same work needs a corrected prompt or repair
  rerun. Badge/color: use a distinct repair color (red/orange if available).
- `NEW -- PROMPT N` above each new prompt the user should launch in a new agent
  window. Badge/color: purple.

Every prompt or agent-window disposition must have one of these state labels
directly above it. Use `NEW`, not a bare `PROMPT`, for newly launchable
parallel work.

Then state, briefly:

1. What changed.
2. Whether it is safe to clear, reply, repair, integrate, or launch new work.
3. Newly unlocked work, if any.
4. Exact next prompt(s), only if launchable now.

Keep responses operational. Do not bury the answer in narrative. If no safe
parallel work exists, say so and name the blocker.

Before ending any orchestrator response, explicitly ask: "What is the next
launchable step, and can any of it run safely in parallel?" If the response says
there is a next step, include the actual prompt block(s) in that same response.
Do not say "next step is X" and wait for the user to ask for the prompt. If the
next step is not launchable yet, name the blocker and do not emit a fake `NEW`.

### Parallelism

Maximize safe parallelism, but never invent work to fill a quota.

- Keep at most one `/story-done` or shared status writer active because it edits
  `production/sprint-status.yaml`, `production/session-state/active.md`, or
  story completion notes.
- Run two to four implementation/blocker-clear workers only when their file
  ownership and architecture ownership are disjoint.
- Docs/readiness/audit workers may run in parallel with implementation if they
  do not touch shared status files.
- Future-sprint work is allowed only when it is truly Ready, disjoint, and does
  not imply activating that sprint.
- CI/smoke/gate failures block release/close-out claims, not ordinary parallel
  implementation, unless the failure is directly caused by the pending work.

Root checkout is reserved for orchestration, integration, story-done, CI triage,
and state tracking. Implementation workers use one worktree and one branch per
story:

- Branch: `work/<story-id>-<short-slug>`.
- Worktree: `D:\_DEV\claude-code-game-studios-worktrees\<story-id>`.
- Workers push their branch, never `main`.

### Agent Roles And Skills

Use Game Studio roles explicitly in prompts:

- `ui-programmer`: Bevy UI, HUD, hand UI, lobby, shop/auction presentation.
- `gameplay-programmer`: server gameplay, economy, combat, RSM, acquisition.
- `network-programmer`: Lightyear protocol, client/server messages, reconnects.
- `qa-lead` or `qa-tester`: evidence, smoke/readiness audits, blocker records.
- `producer`: sprint planning, close-out disposition, scope decisions.
- `ux-designer`: interaction/readability diagnostics and UX docs.
- `art-director` or `technical-artist`: asset/art wiring and visual acceptance.
- `audio-director` or `sound-designer`: audio specs, sound bible, cue evidence.

Mandatory skills:

- Use `liv-bevy-018` before reading, reviewing, or editing Bevy `.rs` code.
- Use `liv-bevy-lightyear` before reading, reviewing, or editing Lightyear,
  multiplayer, protocol, channel, or network-message code.
- For read-only diagnostics, still name relevant skills so the worker uses the
  correct Bevy/Lightyear mental model.

Agent choice:

- Use broad Claude-style diagnostic agents for source-of-truth audits,
  read-only end-to-end diagnosis, UX/design review, and story/readiness docs.
- Use Codex-style implementation workers for scoped code changes, integration,
  story-done, and git hygiene.

When a prompt names Game Studio roles, state whether they are agents to spawn or
roles to perform locally. Avoid ambiguous shorthand such as `Agent: producer +
qa-lead`.

Preferred wording:

```text
Agent:
- Use Claude Code Game Studios agents if available:
  - producer for sprint close-out disposition
  - qa-lead for evidence/non-claims validation
- If spawning agents is not available, perform both roles locally.
- No Bevy/Lightyear code; no liv skill required.
```

Strict parallel review wording:

```text
Agent:
- Spawn CCGS producer and qa-lead agents in parallel for read-only review.
- Then apply the close-out edits locally from their combined verdict.
- No Bevy/Lightyear code; no liv skill required.
```

### Prompt Authoring Template

Every launch prompt should include only the sections that apply:

1. Title: `PROMPT N -- Short Task Name`.
2. Agent/skills: role plus mandatory skills.
3. Repo and mode:
   - implementation: branch + worktree off latest `origin/main`;
   - read-only diagnostic: root checkout, no writes, no branch;
   - story-done/integration: root checkout only.
4. Source of truth: exact branch/commit if known, otherwise latest
   `origin/main` verified at start.
5. Context: two to five bullets explaining why this task exists.
6. Owned files and forbidden files.
7. Investigation order, if the bug spans multiple systems.
8. Required implementation or documentation scope.
9. Verification:
   - workers run narrow targeted tests only;
   - root/orchestrator owns workspace smoke;
   - `cargo check --workspace` only when shared protocol/config/workspace
     surfaces changed or close-out requires it.
10. Commit/push policy:
   - no `main` push for workers;
   - stage explicit paths only;
   - no `/story-done`, smoke, gate-check, QA sign-off unless explicitly scoped.
11. Final report fields: branch, worktree, commit, changed files, checks, rebase
    yes/no, push yes/no, final status.
12. Last visible line rule.

For implementation prompts, include pre-integration duty:

- `git fetch origin`;
- rebase the worker branch on latest `origin/main`;
- rerun listed checks after rebase;
- `git diff --check origin/main...HEAD`;
- push only the worker branch.

For read-only diagnostics:

- Allow `git fetch origin` only to refresh refs.
- Forbid source/worktree edits, branch creation, commits, pushes, smoke, QA
  sign-off, gate-check, `/dev-story`, and `/story-done`.
- Require file/function/line evidence for every bug claim.
- If evidence is insufficient, report ranked suspects instead of certainty.

### Output Examples

Use these as style examples for future orchestrator windows.

Clear-only return:

```text
CLEAR -- PROMPT 762

Already committed at f27d888 and verified. No reply needed in that window.
```

Reply-to-existing-window return:

```text
REPONDRE -- PROMPT 761

Do not retry the Release gate. Record the FAIL as valid evidence, keep stage
Polish, and proceed to Sprint 10 closed-with-conditions paperwork.
```

Rerun/repair-existing-window return:

```text
RELANCER -- PROMPT 558

Use the corrected scope below in the same worker window. The prior prompt was too
broad and allowed shared tracker edits.
```

Short launch prompt:

```text
NEW -- PROMPT 763

PROMPT 763 -- Sprint 10 Polish Close-Out Disposition

Agent:
- Use Claude Code Game Studios agents if available:
  - producer for sprint close-out disposition
  - qa-lead for evidence/non-claims validation
- If spawning agents is not available, perform both roles locally.
- No Bevy/Lightyear code; no liv skill required.

Repo/mode:
- Root checkout only.
- Use latest origin/main as source of truth.

Context:
- Sprint 10 smoke retry-7 is PASS WITH WARNINGS.
- PROMPT 761 Polish->Release gate-check is FAIL.
- Stage remains Polish.
- Sprint 10 is still active.

Scope:
- Close Sprint 10 as Polish/friend-game closed-with-conditions.
- Preserve all carried risks and non-claims.
- Do not activate Sprint 11.

Allowed files:
- production/sprint-status.yaml
- production/session-state/active.md
- production/session-state/codex-orchestrator-state.md
- production/sprints/sprint-10.md only if its status/header must match.

Forbidden:
- client/, server/, shared/, tests/
- smoke, gate-check, QA sign-off, /dev-story, Release claims

Verification:
- git status --short --branch
- git diff --check
- git diff --cached --check before commit

Commit and push if scoped.

Last visible line uses:
763: SPRINT-10-POLISH-CLOSE-OUT-DISPOSITION: STATUS

Color the line by outcome. Use the true final status. No line after it.
```

Implementation worker prompt skeleton:

```text
PROMPT N -- Focused Implementation Title

Agent/skills:
- ui-programmer
- Mandatory: liv-bevy-018
- Add liv-bevy-lightyear if protocol/network messages are touched.

Repo/mode:
- Branch: work/<story-id>-<short-slug>
- Worktree: D:\_DEV\claude-code-game-studios-worktrees\<story-id>
- Base: latest origin/main

Scope:
- Owned files: <exact files/modules>
- Forbidden files: production/sprint-status.yaml,
  production/session-state/active.md,
  production/session-state/codex-orchestrator-state.md, unrelated code.

Task:
- Implement the smallest repair that satisfies the listed acceptance criteria.
- Do not broaden into adjacent bugs; report them separately.

Verification:
- Narrow targeted cargo test(s) only.
- cargo fmt --check
- cargo check -p <crate> --lib if production source changed.
- git diff --check origin/main...HEAD

Pre-integration duty:
- git fetch origin
- rebase on origin/main
- rerun listed checks
- push only the worker branch

Final report:
- worktree, branch, commit hash, changed files, checks, rebase yes/no,
  push yes/no, final git status, blockers.

Last visible line uses:
N: TICKET-ID: STATUS

Color the line by outcome. Use the true final status. No line after it.
```

Read-only diagnostic prompt skeleton:

```text
PROMPT N -- Runtime Bug E2E Diagnostic

Agent/skills:
- broad diagnostic agent
- Mandatory for Bevy reads: liv-bevy-018
- Mandatory for networking reads: liv-bevy-lightyear

Mode:
- Root checkout.
- Read-only diagnostic.
- No source/worktree writes. git fetch origin allowed only to refresh refs.
- No branch, commit, push, smoke, QA sign-off, gate-check, /dev-story, or
  /story-done.

Read first:
- AGENTS.md
- production/session-state/codex-orchestrator-state.md current override
- relevant story file and ACs
- relevant control-manifest / ADR / GDD references

Diagnose in order:
- UI/event path first
- network/protocol path second
- server/RSM path third
- existing-test coverage last

Deliver:
- proven root cause with file/function/line evidence, or ranked suspects with
  evidence gaps
- owner/story/AC classification
- minimal repair prompt(s), split by owner if needed

Last visible line uses:
N: TICKET-ID: STATUS

Color the line by outcome. Use the true final status. No line after it.
```

### Final Line Rule

Current convention for future prompts and orchestrator replies:

- One status line only.
- No delimiter line.
- No HTML/span/CSS/ANSI markup in the prompt text.
- Last visible line uses `N: TICKET-ID: STATUS`.
- `STATUS` is replaced by a real outcome word, never `STATUS`, never a color
  name such as `GREEN` or `YELLOW`.
- Color the entire status line by outcome when the interface supports color:
  green for DONE/COMPLETE/NO-OP/ACCEPTED RISK; yellow for PARTIAL/IN PROGRESS/
  WAITING/NEEDS REPAIR/WARNING; red for BLOCKED/FAILED.

Valid status words include DONE, COMPLETE, IN PROGRESS, WAITING USER, BLOCKED,
FAILED, NEEDS REPAIR, ACCEPTED RISK, NO-OP, and ALREADY DONE.

Prompt numbers are global and monotonically increasing. Use the latest number
recorded in the current conversation/state; do not reset to 1.

Updated: 2026-05-20 (PROMPT 1504 -- dev proxy pack validator tooling integration PASS registered. Source-of-truth before state note: `origin/main@d42cac14549451ce1e03b61543f68ea98f560b65`. Worker report on branch `origin/integrate/dev-proxy-pack-validator-tooling-1484` says PROMPT 1484 source commit `4084c76e7cf5d742eedff951a4dc147109557933` was cleanly cherry-picked onto integration branch `integrate/dev-proxy-pack-validator-tooling-1484`, with pushed integration tip `87d950cecc0ac33e3196ff0ce8e13de01faf02d1` (report records cherry-pick commit `ba715e6b`; integration tip includes the report commit). Worker validation: allowlist exact against its base (`reports/PROMPT-1484-dev-proxy-pack-validator-tooling.md` plus `tools/asset-provenance/**`), `git diff --check` PASS, cheap owned pytest suite PASS (`14 passed in 0.25s`), no Cargo/WASM/runtime smoke. Orchestrator post-fetch check confirms branch exists, but it is based on `origin/main@f6bf7a9a`; current main has newer orchestrator-state commits, so `git merge-base --is-ancestor origin/main origin/integrate/dev-proxy-pack-validator-tooling-1484` is false and `git diff --name-only origin/main..branch` includes `production/session-state/codex-orchestrator-state.md`. Deferred mainland register updated with `origin/integrate/dev-proxy-pack-validator-tooling-1484@87d950ce` as REFRESH-REQUIRED before `MAINLAND_ENQUEUE`; do not enqueue it as-is for fast-forward mainland. No main-land performed; no sprint advancement, no stage change, no story closure, no release readiness claim.)
Updated: 2026-05-20 (PROMPT 1493 -- resolution event visual replay story integration LANDED registered. Source-of-truth before state note: `origin/main@d84c6fc228fa99d3759d22d2600706db947608e0`. Worker report `reports/PROMPT-1493-resolution-event-visual-replay-story-integration.md` says PROMPT 1485 source branch `work/1485-resolution-event-visual-replay-mutation-story` commit `81255724685370de7028bc9107fb9f74feb20edb` was applied file-only onto integration branch `integrate/resolution-event-visual-replay-story-1493`, producing integration commit `cc093550e2bd357a82e74585cb71403c9b52ac62`, pushed to `origin/integrate/resolution-event-visual-replay-story-1493`. Worker fixed the known trailing blank-line-at-EOF issue in `production/epics/board-rendering/story-015-resolution-event-visual-replay-mutation.md`; allowlist exactly `production/epics/board-rendering/EPIC.md`, `production/epics/board-rendering/story-015-resolution-event-visual-replay-mutation.md`, `reports/PROMPT-1485-resolution-event-visual-replay-mutation-story.md`; `git diff --check --cached` PASS; no Cargo/test needed for paperwork/story integration. Branch base was `origin/main@f6bf7a9a`; current main has newer orchestrator-state commits, so deferred mainland register updated with `origin/integrate/resolution-event-visual-replay-story-1493@cc093550` as REFRESH-REQUIRED before `MAINLAND_ENQUEUE`; do not enqueue it as-is for fast-forward mainland. Story remains Draft / future Sprint 19 / NOT activated. No main-land performed; no sprint advancement, no stage change, no story closure, no release readiness claim.)

Updated: 2026-05-20 (PROMPT 1501 -- result-screen hero/accounting Krosmaga polish integration SUCCESS registered. Source-of-truth before state note: `origin/main@3ef586c6c87a693fafac5b9f3bdf4e5f08050a06`. Worker report `reports/PROMPT-1501-result-screen-hero-accounting-krosmaga-polish-integration.md` says PROMPT 1481 source branch `origin/work/result-screen-hero-accounting-1481` commit `4d0f7443091c5dfa64818d22f68630dc7b0fc2eb` was cleanly cherry-picked onto integration branch `integrate/result-screen-hero-accounting-1481`, producing integration commit `ab3f8171953d1e022e5d04823c2f5d84212dbfe3`, pushed to `origin/integrate/result-screen-hero-accounting-1481`. Worker validation: allowlist exact against its base (`client/Cargo.toml`, `client/src/presentation/result_screen.rs`, `tests/integration/presentation/result_screen_hero_accounting_polish_test.rs`, `reports/PROMPT-1481-result-screen-hero-accounting-krosmaga-polish.md`), `git diff --check` PASS, Cargo deferred with source-worker evidence noted. Orchestrator post-fetch check confirms branch exists, but it is based on `origin/main@f6bf7a9a`; current main has newer orchestrator-state commits, so `git merge-base --is-ancestor origin/main origin/integrate/result-screen-hero-accounting-1481` is false and `git diff --name-only origin/main..branch` includes `production/session-state/codex-orchestrator-state.md`. Deferred mainland register updated with `origin/integrate/result-screen-hero-accounting-1481@ab3f8171` as REFRESH-REQUIRED before `MAINLAND_ENQUEUE`; do not enqueue it as-is for fast-forward mainland. No main-land performed; no sprint advancement, no stage change, no story closure, no release readiness claim.)

Updated: 2026-05-20 (PROMPT 1503 -- shared card inspect zoom primitive integration INTEGRATED registered. Source-of-truth before state note: `origin/main@f60ba0da0478e704d640d2ef098ceb930797df97`. Worker report `reports/PROMPT-1503-shared-card-inspect-zoom-primitive-integration.md` says PROMPT 1482 source branch `origin/work/shared-card-inspect-zoom-primitive-1482` commit `f69f9704` was cleanly cherry-picked onto integration branch `integrate/shared-card-inspect-zoom-primitive-1503`, producing integration commit `1d78d90d2651d6525a4e3e718eb5afd6869a8ea3`, pushed to `origin/integrate/shared-card-inspect-zoom-primitive-1503`. Worker validation: allowlist exact against its base (`client/src/ui/card_inspect.rs`, `client/src/ui/mod.rs`, `reports/PROMPT-1482-shared-card-inspect-zoom-primitive.md`), `git diff --check` PASS, broad Cargo deferred. Orchestrator post-fetch check confirms branch exists, but it is based on `origin/main@f6bf7a9a` and is now stale behind current main state notes; `git diff --name-only origin/main..origin/integrate/shared-card-inspect-zoom-primitive-1503` includes `production/session-state/codex-orchestrator-state.md` because the branch lacks the newer orchestrator-state commits. Deferred mainland register updated with `origin/integrate/shared-card-inspect-zoom-primitive-1503@1d78d90d` as REFRESH-REQUIRED before `MAINLAND_ENQUEUE`; do not enqueue it as-is for fast-forward mainland. No main-land performed; no sprint advancement, no stage change, no story closure, no release readiness claim.)

Updated: 2026-05-20 (PROMPT 1502 -- hand fan readability/playable-affordance Krosmaga polish integration PASS registered. Source-of-truth before state note: `origin/main@f6bf7a9a2bd0191a0e72ed8071be39a2b6e172e2`. Worker report `reports/PROMPT-1502-hand-fan-readability-playable-affordance-krosmaga-polish-integration.md` says PROMPT 1490 source branch `origin/work/PROMPT-1490-hand-fan-readability-recovery` commit `be3c0268` was cleanly cherry-picked onto fresh integration branch `integration/PROMPT-1502-hand-fan-readability-1490`, producing integration commit `6307984fae94bf00e686ee4961b5783dd12f9a5a`, pushed to `origin/integration/PROMPT-1502-hand-fan-readability-1490`. Path allowlist exactly: `client/src/ui/hand/mod.rs`, `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs`, `tests/unit/hand-ui/fan_layout_formula_test.rs`; `git diff --check` clean; broad Cargo/focused hand tests deferred per prompt policy. Add `origin/integration/PROMPT-1502-hand-fan-readability-1490@6307984f` to the deferred mainland register for future `MAINLAND_LIST` + `MAINLAND_ENQUEUE` once structured dispatch is exposed. No main-land performed by this state note; no sprint advancement, no stage change, no story closure, no release readiness claim.)

Updated: 2026-05-20 (ORCHESTRATOR FOLLOW-UP REGISTER -- post-contract follow-up launches and deferred mainland register. Source-of-truth after `git fetch origin`: `origin/main@af55ae34a4e7d2f45c21cb976835d7713199b8d2`, the direct-main commit `orchestrator: document gcs dispatch mainland contract` requested by the user. `gcs.dispatch` / `MAINLAND_*` still not exposed in the current tool list (`tool_search` returned 0 tools), so this session must use fallback labels for worker lifecycle actions and must NOT claim structured mainland enqueue. Follow-up worker actions prepared for fallback dispatch: CLEAR PROMPT 1490 (DONE repeated by worker/user); NEW PROMPT 1501 result-screen hero/accounting Krosmaga polish integration from `origin/work/result-screen-hero-accounting-1481@4d0f7443`; NEW PROMPT 1502 hand fan readability/playable-affordance Krosmaga polish integration from `origin/work/PROMPT-1490-hand-fan-readability-recovery@be3c0268`; NEW PROMPT 1503 shared card inspect zoom primitive integration from `origin/work/shared-card-inspect-zoom-primitive-1482@f69f9704`; NEW PROMPT 1504 dev proxy pack validator tooling integration from local branch `work/prompt-1484-dev-proxy-pack-validator@4084c76e`; RELANCER PROMPT 1493 resolution event visual replay story integration from local branch `work/1485-resolution-event-visual-replay-mutation-story@81255724`, fixing the known trailing blank-line-at-EOF issue before branch push. Deferred mainland register to process later via `MAINLAND_LIST` + `MAINLAND_ENQUEUE` once the structured tool is available: `origin/integ/krosmaga-proxy-logical-id-map-stage1-1494@3fcb1b8`, `origin/integrate/lobby-class-identity-confirm-cta-1495@691215d0`, `origin/integration/PROMPT-1496-shop-auction-polish@4395e98e`, `origin/integrate/hud-edge-chrome-phase-timer-1497@a961926f`, `origin/integration/PROMPT-1499-board-play-area-physicality@ad216f3e`, and `origin/integration/PROMPT-1500-qa-snapshot-1486@987f5cb0` (strict diff audit required before enqueue because prior local inspection saw unrelated dispatcher-doc commits reachable on that branch family). Cargo policy preserved: implementation/integration workers run only focused cheap validation; broad Cargo verification remains a separate VERIFY lane. No sprint advancement, no stage change, no story closure, no release readiness, no actual mainland enqueue, and no claim that the fallback-dispatched workers have completed.)

Updated: 2026-05-19 (ORCHESTRATOR CONTRACT UPDATE -- gcs-app structured dispatch and non-blocking worker policy. User explicitly clarified that this session is the orchestrator and must apply the new dispatch contract directly. Active orchestration side effects should prefer `gcs.dispatch` when exposed: `SPAWN`, `CLEAR`, `REPONDRE`, `RELANCER`, and `NEW` for worker lifecycle; `MAINLAND_LIST` before any main-land queue mutation; `MAINLAND_ENQUEUE` with `project_id`, `source_branch`, and `intent_id=mainland-<slug>-<prompt_n>` for serialized fast-forward-only main landings; `MAINLAND_CANCEL` only for pending queue entries. If structured `gcs.dispatch` is unavailable in a Codex session, fallback emoji labels remain valid for worker lifecycle actions only; the orchestrator must not claim a main-land queue item was submitted unless `MAINLAND_ENQUEUE` returned a queue id or an explicit Git operation was actually performed and reported. Current session note: tool discovery did not expose `gcs.dispatch`; `git fetch origin` could not be completed because sandbox write access to `.git/FETCH_HEAD` was denied and escalation was rejected by usage limit, so immediate state refresh used locally cached refs plus worker DONE reports only. Worker policy reinforced: implementation workers should not stall on push, protected-branch, GitHub export, rebase, or permission issues; they keep the local commit/branch, push any non-protected branch if allowed, and relay exact branch, commit, command, and blocker to the orchestrator. Cargo policy reinforced: broad Cargo checks/tests are split into dedicated VERIFY or serialized smoke/checkpoint lanes; implementation workers run focused validation only and should not all run Cargo concurrently. Active dispatcher naming: use `gcs-app` / `gcs.dispatch`; legacy dispatcher docs are historical unless re-enabled by a future current-state entry. Snapshot QA rule preserved: image-by-image forensic analysis with matching JSON state and timestamp-correlated logs, not shallow bulk analysis. A current `NEW PROMPT TEMPLATE` has been added to `AGENTS.md` and `CODEX.md` without deleting older templates. No sprint advancement, no stage change, no story closure, no release readiness, no main-land claim, and no worker completion claim made by this contract update.)

Updated: 2026-05-19 (ORCHESTRATOR QUEUE REFRESH -- post-placement repair main-land sweep. Source-of-truth verified locally from `origin/main@7b683ead043b9f20f915fd291a5dd25935a7a47d`, PROMPT 1471 `PROMPT-1471 refresh connection lost observability`, strict descendant of the latest placement/shop/HUD/snapshot repair chain. Landed/current on main since the stale root checkout: PROMPT 1399 placement-submit silent-noop repair; PROMPT 1410 board-picking drag-to-cell backend; PROMPT 1433 lobby confirm CTA reachability; PROMPT 1436 HUD opponent class/mana microbadge; PROMPT 1438 board-picking main-land; PROMPT 1439 bot foundation scaffold; PROMPT 1441 bot protocol room foundations; PROMPT 1444 drag cursor target trace integration; PROMPT 1450 auction leader perspective label integration; PROMPT 1451 shop slots client receive integration; PROMPT 1452 HUD phase timer countdown snapshot integration; PROMPT 1456 placement drag cursor board hit-test integration; PROMPT 1457 placement drag cursor board hit-test live verify PASS; PROMPT 1458 QA snapshot forensic field upgrade integration; PROMPT 1460 accepted placement unit visibility integration; PROMPT 1466 board grid overlay toggle integration; PROMPT 1463 HUD objective/timer readability main-land; PROMPT 1468 placement rejection recovery UX main-land; PROMPT 1470 shop-auction z-order/readability refresh; PROMPT 1471 connection-lost auction-overlay observability refresh. Current known worker queue: no active running worker is known from the latest relay state. PROMPT 1440 and PROMPT 1449 live verify failures remain historical evidence and are not full-flow closure blockers by themselves after 1457, but 1457 only verifies the board hit-test path; it does NOT validate the full two-client game loop. Next required orchestration action is PROMPT 1472 `POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST`, verify-only against latest `origin/main`, covering lobby confirm, shop offers, auction timer/leader labels, drag/drop placement, accepted unit visibility, rejection recovery, HUD timer/readability, board grid overlay, connection-lost observability if practical, and snapshot forensic fields. Snapshot QA rule reiterated: analyze every image individually with matching JSON state and timestamp-correlated logs; no bulk shallow pass. Until PROMPT 1472 reports, do not launch overlapping code repairs touching `client/src/ui/hand/mod.rs`, `client/src/presentation/board_rendering.rs`, `client/src/ui/shop_auction/mod.rs`, or `client/src/presentation/qa_snapshot.rs` unless the user explicitly chooses speed over diagnosis. Parallel-safe work remains read-only audits/specs/story authoring, bot roadmap/design work, and separate verification lanes. Cargo-heavy checks should be split into dedicated VERIFY prompts rather than assigned to every implementation worker. Sprint 18 remains active; stage remains Polish; `production/stage.txt` untouched. PROMPT 761 Polish->Release gate-check FAIL preserved with no retry. Non-claims preserved: no Sprint 18 close-out, no Sprint 19 activation, no public release readiness, no RC readiness, no full-game completion, no final-art/legal clearance, no QA-COND advancement, no PAW-TD closure, no `S8-QA-001-W1` closure, no `S11-HUD-TIMER-EYEBALL-VISUAL-001` LLM closure, no playtest validation, no broad accessibility completion.)

Updated: 2026-05-19 (PROMPT 1357 -- S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001 `/story-done` paperwork-only closure. Source-of-truth verified from `origin/main@516b6427ba18fbfd0a8a85fe2f382d22d59be320`, which is PROMPT 1370 main-land tip (`story-authoring-integrate(s19-hand-reserve-strip-cleanup): cherry-pick PROMPT 1351 story-027 onto origin/main@daa7759 (PROMPT 1370)`) and a strict fast-forward descendant of PROMPT 1239 worker commit `50b66adfbe30c50eb5e45130b718c70bde8b03a2` (`dev-story(s18-hand-idle-playable-affordance): surface idle Playable / Unaffordable hint per local fan slot (PROMPT 1239)`) + PROMPT 1243 integration commit `4c75cec72adb28e9b81d31ed0806f38336b661c3` that comprise the S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001 implementation lineage on `origin/main`. PROMPT 1357 closes Sprint 18 Should Have Row 2 `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001` on the strength of PROMPT 1324 row 2 readiness verdict `READY_FOR_STORY_DONE` and PROMPT 1356 readiness refresh re-confirming `READY_FOR_STORY_DONE` against the current closure source-of-truth `origin/main@516b642`. Per-AC outcomes: AC1..AC10 + AC12..AC16 + AC18 + AC19 PASS / PASS-BY-CONSTRUCTION; AC11 PASS (test bin `tests/integration/hand-ui/hand_ui_idle_playable_affordance_test.rs` on `origin/main@516b642` with 10 `#[test]` fns + 16 helpers; state driven via direct resource insertion; R1-independent; AC11 floor of 10 named cases met exactly); AC17 PASS-WITH-ADVISORY (documentation drift: AC17 names `hand_ui_plugin_scaffold_test.rs` which does not exist as an integration test bin; canonical scaffold tests live as `tests/unit/hand-ui/plugin_scaffold_test.rs` and the entity-count assertion is satisfied within the new `hand_ui_idle_playable_affordance_test.rs` bin's AC14 case inline -- mirrors PROMPT 1110 trailing-whitespace + PROMPT 1331 / 1354 test-path mismatch advisory precedent); AC20 ADVISORY-DEFERRED (`production/epics/hand-ui/EPIC.md` flip outside PROMPT 1357 allowed-writes scope per task spec; mirrors PROMPT 1354 AC18 disposition for sibling story-022 closure). Zero changes under `client/src/**`, `server/src/**`, `shared/src/**`, `tests/**` across PROMPT 1239 + 1243 + 1357 commits combined (AC13 + AC16 PASS). Sprint 18 row coverage at PROMPT 1357 closure: 3 of 12 Sprint 18 active rows DONE (Must Have 1 of 4 via PROMPT 1337 `S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001`; Should Have 2 of 6 via PROMPT 1331 reconciled by PROMPT 1346 `S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001` + PROMPT 1357 `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001`; Nice to Have 0 of 2); 9 rows preserved as their current status. Sprint 18 top-level `status: active` UNCHANGED; stage `Polish` UNCHANGED; `production/stage.txt` NOT modified. PROMPT 761 Polish->Release gate-check FAIL preserved with NO retry. Sprint 18 NOT closed-out by PROMPT 1357. Carried conditions preserved verbatim: `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked carry (no LLM `/story-done` authorised); `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent-row paperwork gap (no silent closure); `S8-QA-001-W1` OPEN; `QA-COND-0005` accept-risk; `QA-COND-0006` accept-risk; `PAW-TD-*-a` accept-risk across PAW-002..PAW-006; `TQ-S12-C1..C7` (TQ-S12-C7 NOT closed); PROMPT 683-era runtime divergence; Sprint 12 story 019 cannot-reproduce (PROMPT 1357 closure independent of the drag-runtime bug per AC11 R1-independent test design); PROMPT 1054 P1 UI snapshot visual retest BLOCKED-HUMAN-OPERATOR; R1 drag-pipeline-dead bug remains separate prompt; R2 mana-preview missing-feature owned by sibling PROMPT 1354 closure (DONE locally at PROMPT 1357 launch but NOT yet on `origin/main@516b642`; orchestrator expected to reconcile PROMPT 1354 + PROMPT 1357 entries together if PROMPT 1354 lands later). Non-claims preserved: no public release readiness, no RC readiness, no full game completion, no Polish->Release retry, no stage advance, no LLM closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, no silent closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent row, no closure of any AUDIT-1131-* / AUDIT-1076-* / SOURCE-1077-* / PROMPT 1022 / 1076 / 1077 finding outside concrete repairs already on origin/main, no Sprint 10 through Sprint 17 row reopen, no Sprint 17 close-out reopen / re-author / silent overwrite, no Sprint 18 close-out claim, no retroactive closure of any row not implemented on origin/main at the closure tip, no closure of R1 / R2 / AUDIT-1076-02 / AUDIT-1076-03, no EPIC.md flip (AC20 ADVISORY-DEFERRED). Paperwork-only `/story-done`; no cargo / trunk / CI command invoked. Cargo policy: N/A for this paperwork-only closure. Worktree `D:/_DEV/claude-code-game-studios-worktrees/prompt-1357-s18-hand-idle-affordance-story-done` on branch `story-done/s18-hand-idle-playable-affordance-1357` (base `origin/main@516b6427ba18fbfd0a8a85fe2f382d22d59be320`). Files changed by PROMPT 1357: `production/epics/hand-ui/story-023-hand-idle-playable-affordance.md` (Status banner `Draft -> Done` + Sprint / Active impl / Completed / Closure source-of-truth lines + AC1..AC19 `[x]` flipped from `[ ]` (AC20 `[ ]` preserved as ADVISORY-DEFERRED) + Completion Notes (PROMPT 1357) section + Closure Trail table + Conditions / non-claims lists + final status line) + `production/sprint-status.yaml` (Row 2 status `ready -> done` with closure metadata + `sprint_18_activation.active_set.should_have` row 2 annotated with `status_post_closure` + PROMPT 1357 entry appended as 3rd `sprint_18_story_done:` block preserving PROMPT 1337 + PROMPT 1331 entries verbatim above) + `production/session-state/active.md` (PROMPT 1357 banner prepended above PROMPT 1337 banner) + this `production/session-state/codex-orchestrator-state.md` paragraph prepended + `reports/PROMPT-1357-s18-hand-idle-playable-affordance-story-done.md` (mandatory final report; gitignored). `production/epics/hand-ui/EPIC.md` NOT modified (AC20 ADVISORY-DEFERRED).)

Updated: 2026-05-19 (PROMPT 1337 -- S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001 `/story-done` relaunch closure. Source-of-truth verified from `origin/main@72b89ca9702eed5fc9149b92a2d8b7cc1d56aad6`, which is PROMPT 1335 main-land tip for the PROMPT 1334 AC9 cross-link backfill (`docs(ux): backfill global-ui-design-spec cross-link to global-ui-layout-contract (PROMPT 1334)`) and a strict fast-forward descendant of PROMPT 1188 worker commit `c2eaab0` + PROMPT 1208 integration commit `ae8f7d1`. PROMPT 1337 relaunches `/story-done` for Sprint 18 Must Have Row 4 `S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001` after PROMPT 1327 returned `NEEDS_WORK` on the AC9 hard gap (`reports/PROMPT-1327-s18-ui-layout-contract-doc-and-lint-story-done.md`). PROMPT 1334 authored the missing cross-link bullet from `docs/ux/global-ui-design-spec.md` to `docs/ux/global-ui-layout-contract.md`; PROMPT 1335 main-landed at `72b89ca` (+5 lines / 0 deletions; body otherwise unchanged). PROMPT 1337 re-runs the per-AC walk against the post-1335 main tip and finds AC1..AC5 + AC7..AC16 PASS; AC6 ADVISORY (L4 chip-side static lint deferred per contract §10 false-positive-surface rationale; advisory preserved per PROMPT 1323 + PROMPT 1327 directive + PROMPT 1337 re-confirmation -- no new hard blocker). AC7 ±2-lines reading (window = current line + 2 preceding lines via `has_ac_comment_near` at lint:425) carried as advisory per PROMPT 1323 §4 grep-style design defensible. AC4 + AC11 + AC12 + AC16 trusted from PROMPT 1188 + 1208 commit lineage per the project's `/story-done` paperwork policy; no Cargo invocation by PROMPT 1337. Zero changes under `client/src/**`, `server/src/**`, `shared/src/**` across PROMPT 1188 + 1208 + 1334 + 1335 + 1337 commits combined (AC13 PASS). Sprint 18 row coverage at PROMPT 1337 closure: 1 of 4 Must Have row DONE (Row 4 S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001 closed by PROMPT 1337); 3 of 4 Must Have rows remain `ready` (S11-HUD-TIMER-EYEBALL-VISUAL-001 human-operator-blocked carry + S18-AUCTION-WON-CARD-DISPOSITION-001 + S18-UI-PLAY-AREA-CONTAINER-001); 6 Should Have + 2 Nice to Have rows remain `ready`. Sprint 18 top-level `status: active` UNCHANGED; stage `Polish` UNCHANGED; `production/stage.txt` NOT modified. PROMPT 761 Polish->Release gate-check FAIL preserved with NO retry. Sprint 18 NOT closed-out by PROMPT 1337. Carried conditions preserved verbatim: `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked carry (no LLM `/story-done` authorised); `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent-row paperwork gap (no silent closure); `S8-QA-001-W1` OPEN; `QA-COND-0005` accept-risk; `QA-COND-0006` accept-risk; `PAW-TD-*-a` accept-risk; `TQ-S12-C1..C7` (TQ-S12-C7 NOT closed); PROMPT 683-era runtime divergence; Sprint 12 story 019 cannot-reproduce; PROMPT 1054 P1 UI snapshot visual retest BLOCKED-HUMAN-OPERATOR. Non-claims preserved: no public release readiness, no RC readiness, no full game completion, no Polish->Release retry, no stage advance, no LLM closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, no silent closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent row, no closure of any AUDIT-1131-* / AUDIT-1076-* / SOURCE-1077-* / PROMPT 1022 / 1076 / 1077 finding outside concrete repairs already on origin/main, no Sprint 10 through Sprint 17 row reopen, no Sprint 17 close-out reopen / re-author / silent overwrite, no Sprint 18 close-out claim, no retroactive closure of any row not implemented on origin/main at the closure tip. Paperwork-only `/story-done` relaunch; no cargo / trunk / CI command invoked. Cargo policy: N/A for this paperwork-only closure. Worktree `D:/_DEV/claude-code-game-studios-worktrees/prompt-1337-story-done-relaunch` on branch `work/s18-ui-layout-contract-doc-and-lint-story-done-relaunch-1337` (base `origin/main@72b89ca9702eed5fc9149b92a2d8b7cc1d56aad6`). Files changed by PROMPT 1337: `production/sprint-status.yaml` (Row 4 status `ready -> done` + closure metadata + `sprint_18_story_done:` block appended at EOF) + `production/epics/ui-clean-pass/story-027-ui-layout-contract-doc-and-lint.md` (Status banner `Draft -> Done` + AC checkboxes + Completion Notes section) + `production/epics/ui-clean-pass/EPIC.md` (Story 027 row Status column updated) + `production/session-state/active.md` (PROMPT 1337 banner prepended above PROMPT 1301 banner) + this `production/session-state/codex-orchestrator-state.md` entry prepended + `reports/PROMPT-1337-s18-ui-layout-contract-doc-and-lint-story-done-relaunch.md` (gitignored).)

Updated: 2026-05-19 (PROMPT 1331 -- Sprint 18 S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001 `/story-done` paperwork closure. **First /story-done block of Sprint 18 by sequence; reconciled onto current `origin/main` by PROMPT 1346 after PROMPT 1337 already main-landed at `72b89ca` -- PROMPT 1337 paragraph preserved verbatim above this one.** Source-of-truth verified from `origin/main@4940a7bdcbf7189a6c1d7adb5cf87edc93022096`, which is PROMPT 1326 windows launcher main-land tip and a strict fast-forward descendant of PROMPT 1187 dev-story implementation tip `8eeb94e3244245850b044e83ffcfff4df0da835f`, PROMPT 1301 Sprint 18 activation tip `1345c6b8b1cbd543dbd63d279186c93924ca54db`, and PROMPT 1320 Sprint 18 QA plan main-land tip. Paperwork-only single-row Sprint 18 `/story-done` closure for `S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001` (Should Have, story 024 in `production/epics/ui-clean-pass/`) on the basis of: PROMPT 1187 dev-story commit `8eeb94e` (replace 8-child absolute-positioned 760×520 settings panel with bounded flex hierarchy `panel` (flex column, `Overflow::scroll_y`, `max_width`/`max_height Percent(92)`) → `header_row` → `body_row` (flex row, `flex_grow 1`) `category_column` (width 170 px) + `content_pane` (flex column, `flex_grow 1`, `Overflow::scroll_y`) → `footer_row` (`JustifyContent::SpaceBetween`); new `SETTINGS_PANEL_MIN_WIDTH_PX = 540.0` floor; `sync_settings_shell_visibility_system` rewrites width/min_width/max_width/max_height every frame; +218 / -49 in `client/src/ui/settings/mod.rs`; NEW `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` 324 lines / 8 `#[test]` declarations covering AC1..AC8 + marker counts + focus order; `client/Cargo.toml` +4 lines; 3 files / 498 insertions / 49 deletions total; cargo gate pass under Sprint 15+ Windows/MSVC Cargo resource policy). No separate `/integrate` prompt; impl landed pre-Sprint-18 activation and was inherited at the activation tip `1345c6b` per PROMPT 1301. PROMPT 1324 readiness audit row 6 verdict `READY_FOR_STORY_DONE` with minor test-path mismatch advisory (paperwork-only). AC1..AC13 PASS; AC7 PASS-STRUCTURAL + ADVISORY-EVIDENCE-DEFERRED. PROMPT 1324 test-path mismatch advisory preserved explicitly: spec path per story-024 `Owned files` + `production/qa/qa-plan-sprint-18.md` Row 10 is `tests/integration/settings/ui_scale_invariant_test.rs`; actual landed path is `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` (consistent with sibling `settings_shell_test.rs` + `timer_selector_test.rs`); PROMPT 1331 selects PROMPT 1324 discharge option (a) — accept the actual landed path; record explicitly in story Completion Notes + sprint-status.yaml `paperwork_advisory.test_path_mismatch:` sub-block + active.md banner + this paragraph + final report — NOT hidden. Story-024 narrative + qa-plan Row 10 spec path wording preserved verbatim (PROMPT 1331 forbidden from touching `production/qa/**` and `production/sprints/**`). Mirrors PROMPT 1110 "PROMPT 1106 evidence-file trailing-whitespace advisory" precedent. Optional `production/qa/evidence/sprint-18-settings-flex-relayout/` screenshot directory was not authored (story-024 `Owned files` lists it as optional; structural integration test is binding AC7 gate); ADVISORY recorded. PROMPT 1324 §3 paperwork mis-attribution diagnostic findings for `production/sprints/sprint-18.md:206` and `production/qa/qa-plan-sprint-18.md:653` (both cite "no explicit commit captured"; correct is `8eeb94e` per PROMPT 1187) are **outside PROMPT 1331 allowed-files scope** and NOT discharged here; a follow-on paperwork prompt MAY thread them. Sprint 18 progress after PROMPT 1331 (reconciled with PROMPT 1337 already on main): 2 of 12 active rows DONE (Must Have 1/4 via PROMPT 1337 row 4 + Should Have 1/6 via PROMPT 1331 + Nice to Have 0/2); 10 rows preserved as their current status. Sprint 18 disposition `active` UNCHANGED. Stage `Polish` UNCHANGED (`production/stage.txt` NOT modified). PROMPT 761 Polish->Release FAIL preserved with NO retry. Carried conditions preserved verbatim: `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked carry Sprint 13 → 14 → 15 → 16 → 17 → 18 (no LLM `/story-done` authorised); `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent-row paperwork gap (no silent closure); `S8-QA-001-W1` OPEN; `QA-COND-0005` accepted-risk (this row is a PRECONDITION, not closure); `QA-COND-0006` accepted-risk; `PAW-TD-*-a` accepted-risk; `TQ-S12-C1..C7` preserved (TQ-S12-C7 NOT closed); PROMPT 683-era runtime divergence preserved; Sprint 12 story 019 cannot-reproduce preserved; PROMPT 1054 BLOCKED-HUMAN-OPERATOR preserved; 24 PROMPT 1022 QA snapshot audit findings preserved; long-tail AUDIT-1076-* + SOURCE-1077-* findings outside concrete repairs already on `origin/main` preserved. Non-claims preserved: no public release readiness, no RC readiness, no full game completion, no Polish->Release retry, no stage advance, no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, no silent closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001`, no closure of any other Sprint 18 active row beyond PROMPT 1337 Row 4 already on main, no Sprint 10–17 row reopen, no Sprint 17 close-out reopen. Files changed by PROMPT 1331 (as reconciled by PROMPT 1346): `production/epics/ui-clean-pass/story-024-ui-settings-panel-flex-relayout.md` (Draft → Done; AC1..AC13 verdicts; Completion Notes + PROMPT 1324 test-path mismatch advisory + Test Evidence + Closure Trail) + `production/epics/ui-clean-pass/EPIC.md` (story 024 row Status flipped; PROMPT 1337 row 027 entry preserved verbatim) + `production/sprint-status.yaml` (row flip + `sprint_18_activation.should_have` annotated with `status_post_closure:` + PROMPT 1331 entry appended to existing PROMPT 1337 `sprint_18_story_done:` list at EOF) + `production/session-state/active.md` (PROMPT 1331 banner inserted below PROMPT 1337 banner) + this `production/session-state/codex-orchestrator-state.md` paragraph inserted below PROMPT 1337 paragraph + `reports/PROMPT-1346-s18-settings-panel-flex-reconcile.md` (gitignored). Files explicitly NOT touched: `client/`, `server/`, `shared/`, `tests/` (in particular `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` preserved verbatim; no rename to `tests/integration/settings/`), `Cargo.*`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt`, `production/sprints/**`, `production/qa/**`, `production/gate-checks/**`, `docs/architecture/adr-*.md`, all other story files. Cargo policy: N/A (paperwork-only; no `cargo` or `trunk` invocation). Worktree `.claude/worktrees/prompt-1346-s18-settings-panel-flex-reconcile` on branch `worktree-prompt-1346-s18-settings-panel-flex-reconcile` (base `origin/main@1e9548f23f7f19d3f8e14591b731cdfbbdd57874`). Original PROMPT 1331 worktree `.claude/worktrees/prompt-1331-settings-panel-flex-relayout-story-done` on branch `prompt-1331-s18-settings-panel-flex-relayout-story-done` (base `origin/main@4940a7b`) preserved at commit `e61114b76b36127acc9b7d9cbb96dd83583ed1e7`. Next launchable action: queue the next paperwork-only Sprint 18 `/story-done` rows in parallel — `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001` (impl `8d0a3d3`), `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001` (impl `50b66ad` + `4c75cec`), `S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001` (impl `671c677`), `S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001` (impl PROMPT 1186 `d75db1a` per PROMPT 1324 §3). EPIC.md serialisation between them; otherwise file-disjoint per PROMPT 1324 §2 cross-row matrix.)

Updated: 2026-05-18 (PROMPT 1301 -- Sprint 18 activation disposition. Source-of-truth verified from `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`, which is PROMPT 1292 Sprint 18 plan draft main-land tip and a strict fast-forward descendant of PROMPT 1291 closeout evidence reconcile main-land tip `4efe800` and PROMPT 1283 cargo-fmt drift repair main-land tip `d73e25e`. Paperwork-only Sprint 18 activation against PROMPT 1293 read-only activation readiness PASS verdict at `reports/PROMPT-1293-sprint-18-activation-readiness.md` (gitignored). Top-level `production/sprint-status.yaml` flipped: `sprint: 17 -> 18`; `status: closed-with-conditions -> active`; `stage: Polish` PRESERVED verbatim (`production/stage.txt` NOT modified). `sprint_18_activation:` block appended at EOF; Sprint 17 closeout (PROMPT 1279) and closeout evidence reconcile (PROMPT 1289 / PROMPT 1291 main-land `4efe800`) blocks preserved verbatim. PROMPT 761 `Polish -> Release` gate-check FAIL preserved with NO retry. Sprint 18 active set: 4 Must Have (S11-HUD-TIMER-EYEBALL-VISUAL-001 carry + S18-AUCTION-WON-CARD-DISPOSITION-001 + S18-UI-PLAY-AREA-CONTAINER-001 + S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001) + 6 Should Have (S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001 + S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001 + S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001 + S18-UI-CARD-ART-AND-LABEL-STRIP-001 + S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 + S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001) + 2 Nice to Have (S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001 + S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001) = 12 rows / ~5.4d. 1 Nice to Have plan candidate row DROPPED at activation: S18-UI-HAND-RESERVE-STRIP-CLEANUP-001 (0.2d; no story file on origin/main; recorded under `sprint_18_activation.dropped_rows:` as not activated / story-authoring-needed per Sprint 18 plan Section 2.3 constraint; re-evaluation deferred to Sprint 19 planning). Section 0 activation blockers discharged: PROMPT 1284 post-fmt smoke evidence through PROMPT 1289 / PROMPT 1291 reconcile; PROMPT 1289 reconcile on origin/main as ancestor of 1345c6b; Sprint 18 plan draft landed via PROMPT 1292 at 1345c6b. Next launchable action: `/qa-plan sprint-18` (mirrors Sprint 17 PROMPT 1100 precedent). NO `/dev-story` against Sprint 18 rows is authorised before the QA plan lands on `origin/main`. Parallel branch-only / audit prompts PROMPT 1294-1299 and integration PROMPT 1300 remain safe parallel work. Carried conditions preserved verbatim: S11-HUD-TIMER-EYEBALL-VISUAL-001 human-operator-blocked carry (no LLM `/story-done` authorised); S17-UI-HUD-OPP-MANA-CLEANUP-001 parent-row paperwork gap (no silent closure); S8-QA-001-W1 OPEN; QA-COND-0005 accepted-risk; QA-COND-0006 accepted-risk; PAW-TD-*-a accepted-risk; TQ-S12-C1..C7 preserved (TQ-S12-C7 NOT closed); PROMPT 683-era runtime divergence preserved; Sprint 12 story 019 cannot-reproduce preserved; PROMPT 1054 BLOCKED-HUMAN-OPERATOR preserved. Non-claims preserved: no public release readiness, no RC readiness, no full game completion, no Polish->Release retry, no stage advance, no LLM closure of S11-HUD-TIMER-EYEBALL-VISUAL-001, no silent closure of S17-UI-HUD-OPP-MANA-CLEANUP-001 parent row, no closure of any PROMPT 1022 / 1076 / 1077 finding outside concrete repairs already on origin/main, no Sprint 10 through Sprint 17 row reopen, no Sprint 17 close-out reopen / re-author / silent overwrite, no retroactive closure of any row not implemented on origin/main at the activation tip. Paperwork-only activation; no client/server/shared/tests/Cargo/.cargo/.github/Trunk.toml/stage.txt/qa-plan/gate-checks/qa-evidence/story file edit; no cargo or trunk command invoked. Cargo policy: N/A. Worktree `.claude/worktrees/prompt-1301-sprint-18-activation` on branch `worktree-prompt-1301-sprint-18-activation` (base `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`). Files changed by PROMPT 1301: `production/sprint-status.yaml` (top-level flips + scope/goal/start/end/generated refresh + stories block replacement + `sprint_18_activation:` block appended) + `production/sprints/sprint-18.md` (ACTIVATED banner prepended above DRAFT banner; plan body NOT rewritten) + `production/session-state/active.md` (PROMPT 1301 banner prepended) + this `production/session-state/codex-orchestrator-state.md` entry prepended + `reports/PROMPT-1301-sprint-18-activation.md` (gitignored).)

Updated: 2026-05-18 (PROMPT 1279 -- Sprint 17 close-out disposition. Source-of-truth verified from `origin/main@946ca392c94a4988e9c6b4483848233fe6323061`, which is PROMPT 1276 board-rendering message init main-land and includes the late smoke-repair lineage PROMPT 1272 `23d1c1b`, PROMPT 1275 `35a95d5`, and PROMPT 1274 `c94514f`. Sprint 17 top-level status is now `closed-with-conditions`; stage remains `Polish`; `production/stage.txt` is not modified; Sprint 18 is NOT activated; no Polish->Release retry, release/RC readiness, full-game completion, final-art completion, broad accessibility completion, playtest validation, or stage advance is claimed. Team-QA of record is PROMPT 1278 `APPROVED-WITH-CONDITIONS` and is integrated at `production/qa/team-qa-sprint-17-2026-05-18.md`. Sprint 17 row count at close-out: 7/9 done, 1/9 `in_progress` carried as a parent-row paperwork gap (`S17-UI-HUD-OPP-MANA-CLEANUP-001`; AC3 hand-reserve microbadge source repair is on origin/main via `c842668`, but no final `/story-done` paperwork closed the row), and 1/9 human-operator-blocked ready carry (`S11-HUD-TIMER-EYEBALL-VISUAL-001`; no LLM `/story-done` authorised). Initial PROMPT 1264 smoke failed; repair commits are now on main; PROMPT 1277 durable tracked smoke report artifact is still missing from `reports/PROMPT-1277*` and `production/qa/smoke-sprint-17*`, so that remains a close-out evidence condition. PROMPT 1278 smoke warnings are carried: `hand_ui_phase_transition_auto_submit_short_circuit` / `invalid_submit_state` after one staged card at Placement -> Resolution, and `RSM disconnect timer breach: grace window exceeded` after a later DraftShop disconnect. Carried conditions preserved: `S8-QA-001-W1` OPEN, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 683-era runtime divergence, Sprint 12 story 019 cannot-reproduce, PROMPT 1054 P1 UI snapshot visual retest blocked-human, and PROMPT 761 Polish->Release FAIL with no retry. Current next action is a Sprint 18 plan / activation flow prompt only; do not activate Sprint 18 without an explicit producer prompt.)

Updated: 2026-05-18 (PROMPT 1227 -- Orchestrator state refresh after PROMPT 1210 main-land and PROMPT 1226 launch. Source-of-truth verified from `origin/main@c61bab34b775e420ef752135bb50d49611eae379`, which is PROMPT 1210 `S18-PLACEMENT-DRAG-CURSOR-COORD-SPACE-001` integration tip. Stage remains `Polish`; Sprint 17 remains `active`; Sprint 18 is NOT activated by this update; no Sprint 17 close-out, no Polish->Release retry, no stage advance, and no release/RC readiness claim. PROMPT 1210 worker report file was not present locally, but pushed worker commit `origin/work/s18-placement-drag-cursor-coord-space-1210@a652418659705ff1662cbe771abece92fd6e5721` contained detailed scope/test evidence. The orchestrator integrated it in fresh worktree `D:\Tmp\ccgs-prompt-1225-placement-drag-cursor`, producing main commit `c61bab34b775e420ef752135bb50d49611eae379`. Verification: first cargo invocation used one stale target-name attempt and failed before compilation because some test target names were incomplete; rerun used registered target names and passed 27/27 targeted hand-ui tests under the Windows/MSVC Cargo resource policy. Diff checks clean. PROMPT 1210 was cleared through structured dispatch. PROMPT 1226 `S18-PLACEMENT-AUTO-SUBMIT-CLIENT-001` was officially launched through structured dispatch as the now-unblocked client-side follow-up after PROMPT 1209 server grace and PROMPT 1210 cursor coord-space. Its task is to queue one final `C2SSubmitPlacement` before local pending placements are cleared when `S2CPhaseChanged(Resolution)` arrives with local staged placements. Current next action: wait for PROMPT 1226, then integrate it serially before launching overlapping placement rejection protocol or additional hand-ui placement follow-ups. Current live rules preserved: use structured `gcs.dispatch` with `project_id: default`; workers cannot officially launch prompts; ignore Octogent/Octoagent/.octogent as flow truth; every REPONDRE/RELANCER payload is direct worker-facing relay text; keep main lands/shared-status writers serialized; include Windows/MSVC Cargo resource policy in any Cargo-capable prompt; if push-to-main is policy-blocked, commit/push branch and report instead of stalling.)

Updated: 2026-05-18 (PROMPT 1224 -- Orchestrator catch-up after post-compaction serialized main lands. Source-of-truth verified from `origin/main@dbacb85b282787825e93d150383672124e1c91d0`, which is PROMPT 1211 `S18-OPPONENT-DISCONNECT-BROADCAST` integration tip. Stage remains `Polish`; Sprint 17 remains `active`; Sprint 18 is NOT activated by this update; no Sprint 17 close-out, no Polish->Release retry, no stage advance, and no release/RC readiness claim. Main now includes the refreshed sequence `0680fb4` PROMPT 1219 / 1189 Sprint 18 UI roadmap story authoring, `f48583d` PROMPT 1209 server Placement deadline grace window, `671c677` PROMPT 1185 live-spawn UI viewport invariant harness, `6a18c78` PROMPT 1212 session-settings-on-join unicast, and `dbacb85` PROMPT 1211 opponent-disconnect broadcast send-site. Verification performed by orchestrator in fresh worktrees: PROMPT 1209 `cargo test -p server --test rsm_timers_test` 13/13 plus adjacent RSM/timer tests 24/24; PROMPT 1185 `cargo test -p client --test ui_viewport_live_test -- --nocapture` 8/8; PROMPT 1212 `placement_timer_multiplier_test` 9/9 + `room_create_join_test` 7/7; PROMPT 1211 `opponent_disconnect_dispatch_test` 4/4 + `rsm_disconnect_test` 18/18 + `protocol_completeness_invariant` 2/2. The shared `D:\_DEV\cargo-target\ccgs-msvc` target contained a live `server.exe` from the user's dev session, so server verification for PROMPT 1209 / 1212 / 1211 used sibling target `D:\_DEV\cargo-target\ccgs-msvc-test` with the same no-PDB Cargo policy rather than killing the session. PROMPT 1212 and PROMPT 1211 lacked local report files, but their pushed worker commits contained detailed scope/test evidence and were integrated only after fresh worktree verification. Current pending worker state: PROMPT 1210 placement drag cursor coord-space repair is still awaited; no local report or worker branch was visible at this refresh. Current next action: wait for PROMPT 1210, then integrate it serially before launching dependent client-side placement auto-submit or placement rejection protocol work. Continue using fresh worktrees for all integration/main-land work; root checkout is not a clean orchestrator checkout. Current live rules preserved: use structured `gcs.dispatch` when available, fallback labels only when dispatch transport is closed; workers cannot officially launch prompts; ignore Octogent/Octoagent/.octogent as flow truth; every REPONDRE/RELANCER payload is direct worker-facing relay text; keep main lands/shared-status writers serialized; include Windows/MSVC Cargo resource policy in any Cargo-capable prompt; if push-to-main is policy-blocked, commit/push branch and report instead of stalling.)

Updated: 2026-05-18 (Orchestrator runtime update after PROMPT 1170 + REPONDRE to PROMPT 1173. Source-of-truth verified from `origin/main@95504f3791ee53898fa9fd4ce4ff760cc3279a24`, which is PROMPT 1172 session-state refresh. Stage remains `Polish`; Sprint 17 remains `active`; this update does not close Sprint 17, activate Sprint 18, retry Polish->Release, advance stage, or claim release readiness. PROMPT 1170 `WINDOWS-LAUNCHER-REPO-ROOT-RESOLUTION-REPAIR` completed on branch `origin/work/windows-launcher-repo-root-sidecar-1170` at `7c6a73cf0f01a64585a73fdc24b66be979598b48` and the worker was cleared; that branch is not an ancestor of `origin/main` yet. PROMPT 1173 `WINDOWS-LAUNCHER-REPO-ROOT-RESOLUTION-INTEGRATION` was launched as the official branch-only integration refresh on top of latest main. Runtime test feedback from the user found a second launcher bug before main-land: `D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe` fails because `ccgs-dev-launcher.repo-root.txt` begins with a `U+FEFF` BOM before a `#` comment header, and the sidecar parser treats `# ccgs-dev-launcher.repo-root.txt` as the repo-root path instead of skipping it as a comment. A direct worker-facing `REPONDRE PROMPT-1173` relay was sent requiring the integration to strip/handle BOM before comment detection, add or update a unit test for a BOM-prefixed comment header, verify the generated sidecar, and not finish PASS unless the runtime-discovered issue is fixed or explicitly BLOCKED with evidence. Current next action: wait for PROMPT 1173; if it returns a corrected integration branch with checks passing, launch a serialized main-land prompt. Do not main-land the raw PROMPT 1170 branch. Immediate workaround for local testing remains setting `CCGS_REPO_ROOT=D:\_DEV\Work\Claude-Code-Game-Studios` before launching the EXE. Current live rules preserved: use structured `gcs.dispatch`; workers cannot launch official prompts; ignore Octogent/Octoagent/.octogent as flow truth; every REPONDRE/RELANCER is direct worker-facing relay text; keep main lands/shared-status writers serialized; include Windows/MSVC Cargo resource policy in any Cargo-capable prompt; if push-to-main is policy-blocked, commit/push branch and report instead of stalling.)

Updated: 2026-05-18 (PROMPT 1172 -- Orchestrator state refresh after the late Sprint 17 / future-Sprint-18 runtime-tooling wave. Source-of-truth verified by orchestrator from `origin/main@6e3a5bebb339767051e904a08fd6fdd8fb5415af`, which is PROMPT 1171 `LOBBY-EXISTING-ROOM-BROWSER-MAIN-LAND` and includes the strict main chain `840191f` PROMPT 1149 placement critical repair -> `8fa33eb` PROMPT 1166 shop/auction click backend + `ui_picking` default -> `6af57d8` PROMPT 1168 QA snapshot overlay-exclude -> `5646e5a` PROMPT 1169 native Windows two-button launcher EXE -> `6e3a5be` PROMPT 1171 lobby existing-room browser. Stage remains `Polish`; sprint-status still says Sprint 17 active; this refresh is session-state only and does not close Sprint 17, activate Sprint 18, retry Polish->Release, or claim release readiness. Main now contains: PROMPT 1163 placement critical client repair, PROMPT 1166 shop/auction click repair (default `ui_picking`; timer visual cleanup remains separate/outstanding), PROMPT 1168 QA snapshot overlay-exclude, PROMPT 1169 native Windows launcher EXE code (`tools/dev-launcher-app` and `tools/dev-launcher/build-launcher-exe.ps1`; target binaries remain untracked build outputs), and PROMPT 1171 lobby existing-room browser (`C2SListRooms`, `S2CRoomList`, `RoomListEntry`, server room list filtering, client Existing rooms panel + Refresh + row-click join). PROMPT 1160 / 1167 / 1171 means the lobby room browser is now on main; next runtime test should verify Client A creates a room, Client B sees it in Existing rooms without typing a code, row-click joins, and class confirm still works. Current active worker: PROMPT 1170 `WINDOWS-LAUNCHER-REPO-ROOT-RESOLUTION-REPAIR`, branch/root checkout `work/windows-launcher-repo-root-sidecar-1170`, with edits in `tools/dev-launcher-app/src/main.rs`, `tools/dev-launcher/build-launcher-exe.ps1`, and `docs/setup/dev-two-button-launcher.md`; it is repairing the bug where launching `D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe` resolves repo root as the target directory and cannot find `tools\dev-launcher\Update-LatestMain.ps1`. Expected 1170 fix shape: validate `CCGS_REPO_ROOT`, add/read a sidecar repo-root file next to the EXE, refuse target/debug as a silent fallback, and rebuild/test `dev-launcher-app`; after 1170 reports, integrate/refresh on top of `origin/main@6e3a5be` or newer before any main land. Root checkout is NOT a clean orchestrator checkout right now: it is on the 1170 worker branch and behind `origin/main` by one commit; do not use root for unrelated integration or shared-status writes. Use fresh worktrees for integration/main-land prompts. Current live-orchestrator rules still apply: only the orchestrator launches official prompts via structured `gcs.dispatch`; workers cannot launch prompts; ignore Octogent/Octoagent/.octogent as flow truth; every REPONDRE/RELANCER payload must be direct worker-facing relay text; keep one shared-status writer/main-land active at a time; include the Windows/MSVC Cargo resource policy in every prompt that may run Cargo; if push-to-main is policy-blocked, workers should push a branch/commit and report instead of stalling. Non-claims preserved: no Sprint 17 close-out, no Sprint 18 activation, no Polish->Release retry, no public release / RC readiness, no full-game completion, no broad accessibility completion, no playtest validation, no final-art completion, no `S8-QA-001-W1` closure, and no LLM closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`.)

Updated: 2026-05-18 (PROMPT 1125 -- Sprint 17 `S17-OPS-VULKAN-VALIDATION-GATING-001` `/story-done` closure; paperwork-only single-shared-status writer. Source-of-truth at closure: `origin/main@c300b141247307cbd0fbc7f507a175db308026b2` = PROMPT 1124 paperwork-only `/story-done` tip `story-done(s17): close S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 (PROMPT 1124)`. Source-side remediation for this row was delivered earlier by PROMPT 1103 worker `c34cc041759385b75d6356ebbae7e3f336cb85a5` `dev-story(s17): gate Vulkan validation behind cargo feature (S17-OPS-VULKAN-VALIDATION-GATING-001 / AUDIT-1076-18, PROMPT 1103)` + PROMPT 1109 integration `0cab9421bd11b86c05cd804d62739e2e13a55278` `integrate(s17): bring origin/main 5345164 (PROMPTS 1106 1113 1114 1115 1116 1117 1118 1119 1120 1121) forward into PROMPT 1103 Vulkan validation gating integration branch (PROMPT 1109)`. Strict fast-forward descendant chain (PROMPT 1109 `0cab942` -> PROMPT 1106 `30f166f` -> PROMPT 1107 `dc8adb6` -> PROMPT 1108 `72d56bc` -> PROMPT 1110 `9a9b1dc` -> PROMPT 1111 `4bd4f56` -> PROMPT 1112 `f2ba917` -> PROMPT 1114 `30c9e0f` -> PROMPT 1117 `2250add` -> PROMPT 1118 `29ad4c6` -> PROMPT 1119 `d35d24d` -> PROMPT 1120 `89ce149` -> PROMPT 1121 `5345164` -> PROMPT 1123 `74c25b6` -> PROMPT 1124 `c300b14`). Branch: `worker/vulkan-validation-gating-story-done-1125` (fresh worktree from `origin/main@c300b14`, NOT in-place on the root checkout's local `main` per the PROMPT 1123/1124-recorded branch-state anomaly where local-only commits sit on the root checkout's local main without push). Single-context paperwork-only `/story-done` run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. **Single-row closure**: `S17-OPS-VULKAN-VALIDATION-GATING-001` (Nice to Have; story 007 in `production/epics/devops/`; AUDIT-1076-18 P3 Vulkan validation-layer warning gating) closed on basis of PROMPT 1103 worker (Cargo feature gate `wgpu-validation = []` in `client/Cargo.toml` lines 34-40 OFF by default; `bevy::render::settings::{InstanceFlags, RenderCreation, WgpuSettings}` + `bevy::render::RenderPlugin` imports at `client/src/main.rs:9-10`; `let instance_flags = if cfg!(feature = "wgpu-validation") { InstanceFlags::from_build_config() } else { InstanceFlags::empty() }` at `client/src/main.rs:60-64`; `.set(RenderPlugin { render_creation: RenderCreation::Automatic(WgpuSettings { instance_flags, ..default() }), ..default() })` override on the `DefaultPlugins` builder at `client/src/main.rs:74`; worker chose feature gate over `cfg!(debug_assertions)` because AC1 requires zero `VK_LAYER_KHRONOS_validation` warnings on the default `cargo build -p client` invocation, which `cfg!(debug_assertions)` would not satisfy in dev builds; the feature gate is the only strategy that unambiguously satisfies AC1 + AC2 + the Sprint 17 plan wording 'gated on a cargo feature so prod / CI logs stay clean'; worker pushed `work/s17-vulkan-validation-gating` only — never main) + PROMPT 1109 integration (`--no-ff` merge of `origin/work/s17-vulkan-validation-gating@c34cc04` onto origin/main produced first tip `78cfc80`; origin/main moved underneath to `30f166f` then `5345164` during the build/push window; PROMPT 1109 forward-merged origin/main onto the integration branch (no destructive ops; no rebase; no force push) producing tip `0cab942` which was fast-forwarded onto both `integrate/s17-vulkan-validation-gating-1109` and `main`; `git diff origin/main...HEAD` after forward-merge: exactly 3 paths `client/Cargo.toml` + `client/src/main.rs` + `production/qa/evidence/sprint-17-vulkan-validation-gating/evidence.md`; zero changes under `server/`, `shared/`, `tests/integration/server/`; `cargo check -p client` PASS 11.32s + 11.66s after forward-merge; `cargo build -p client` (default features) PASS 59.58s + 1m10s after forward-merge; `cargo build -p client --features wgpu-validation` PASS 1m46s + 1m09s after forward-merge; `git diff --check origin/main...HEAD` / `git diff --cached --check` clean; `liv-bevy-018` review by PROMPT 1109: `RenderPlugin`/`WgpuSettings`/`InstanceFlags` path used (`bevy::render::settings::*`) is the correct Bevy 0.18 API surface; no deprecated Bundle types touched; default-plugin `.set(...)` override is the canonical 0.18 customisation pattern; cfg-gated feature flag with empty default is consistent with the existing `ui_picking` feature shape in the same file; Cargo.lock unchanged — feature toggles `InstanceFlags`, pulls no new dependency; PROMPT 1123 subsequently rebased PROMPT 1122 worker onto `origin/main@0cab942` mid-run before push so `wgpu-validation` feature + S17-OPS comment block at `client/Cargo.toml:34-40` auto-merged cleanly with marker-split `[[test]]` registrations). **AC1 PASS** (Vulkan validation flag OFF by default — PROMPT 1103 worker default-build non-interactive 8s launch grep `VK_LAYER_KHRONOS_validation` match count = 0; reverified by PROMPT 1109 integration `cargo build -p client` PASS). **AC2 PASS** (validation can be opted in via `--features wgpu-validation` — PROMPT 1103 worker opt-in launch grep match count = 1; documented opt-in behaviour on a host without the validation layer; reverified by PROMPT 1109 integration `cargo build -p client --features wgpu-validation` PASS). **AC3 ADVISORY-DEFERRED** (Sprint 17 smoke confirms zero validation warnings — Config / Data row classification per `.claude/docs/coding-standards.md` Test Evidence by Story Type matrix; smoke check pass is the ADVISORY gate for Config / Data rows; not BLOCKING for PROMPT 1125 closure; PROMPT 1125 paperwork-only closure proceeds on AC1 + AC2 + AC4..AC10 PASS; AC3 carries forward into the existing Sprint 17 smoke prompt scope; the default-build cargo build run by PROMPT 1109 integration is an empirically equivalent surrogate for the AC1 grep but the binding gate remains the Sprint 17 smoke harness). **AC4 PASS** (WGPU plugin still functions normally — Vulkan AdapterInfo + window creation + every client plugin loaded in both PROMPT 1103 worker launch logs; PROMPT 1109 integration cargo build also confirms; `liv-bevy-018` review confirms no other WGPU plugin configuration was altered). **AC5 PASS** (no new workspace Cargo dependency — workspace root `Cargo.toml` unchanged; `Cargo.lock` unchanged; only `client/Cargo.toml` touched). **AC6 PASS** (no protocol or server change — PROMPT 1109 `git diff` confirms zero changes under `server/`, `shared/`, `tests/integration/server/`). **AC7 PASS** (no accept-risk closure claimed — worker commit + evidence file + integration merge + PROMPT 1125 paperwork all explicitly preserve `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 761 Polish->Release FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all AUDIT-1076-* findings outside AUDIT-1076-18, all SOURCE-1077-* findings, all 24 PROMPT 1022 findings). **AC8 PASS** (Sprint 17 disposition preserved by worker + integration — PROMPT 1103 + PROMPT 1109 diffs touched zero files under `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md`; PROMPT 1125 is the first authorised modifier of `production/sprint-status.yaml` + `production/session-state/*` for this row). **AC9 PASS** (worker branch scope contained — PROMPT 1103 worker pushed `work/s17-vulkan-validation-gating` (`c34cc04`) only — never main; integration into `origin/main` performed separately by PROMPT 1109 via `integrate/s17-vulkan-validation-gating-1109` -> `0cab942`). **AC10 PASS-WORKER + PASS-INTEGRATION** (Cargo resource policy applied — PROMPT 1103 worker applied all 5 env vars `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` + `CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'` before every cargo invocation; D: free >= 760 GB at preflight; stray `target/` from a first env-unloaded invocation was removed before re-running under policy; PROMPT 1109 integration applied the same policy; D: free ~745 GB at integration start; build correctness gate unaffected throughout; PROMPT 1125 itself did NOT invoke Cargo). **Sprint 17 progress after PROMPT 1125**: 7 of 9 active rows DONE (S17-SERVER-START-OF-TURN-DEBUG-001 by PROMPT 1108 + S17-UI-CARD-SLOT-INSET-WIRING-001 by PROMPT 1110 + S17-UI-CARD-DISPLAY-ART-HELPER-001 by PROMPT 1117 + S17-UI-HAND-B0004-CLEANUP-001 by PROMPT 1120 + S17-UI-BID-BUTTON-PHASE-RACE-001 by PROMPT 1121 + S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 by PROMPT 1124 + S17-OPS-VULKAN-VALIDATION-GATING-001 by PROMPT 1125) + 1 of 9 active rows PARTIAL / IN_PROGRESS (S17-UI-HUD-OPP-MANA-CLEANUP-001 carried via PROMPT 1112 with AC3 carried; row remains OPEN, NOT closed by PROMPT 1125) + 1 row preserved as `ready` (S11-HUD-TIMER-EYEBALL-VISUAL-001 human-operator-blocked carry). Must Have 1/2 done + Should Have 3/4 done + 1 partial/4 + Nice to Have 3/3 done. **AUDIT-1076-18 discharged on origin/main by PROMPT 1109 integration** (the closure target of this row). Other AUDIT-1076-* findings preserved outside the already-discharged subset (14 PROMPT 1118/1120; 15 PROMPT 1107/1108; 10 + 16 PROMPT 1111); AUDIT-1076-17 remains OPEN carried with AC3 of `S17-UI-HUD-OPP-MANA-CLEANUP-001` PROMPT 1112 PARTIAL disposition. PROMPT 1125 does NOT discharge any AUDIT-1076-* finding outside AUDIT-1076-18. SOURCE-1077-* findings preserved outside already-discharged subset (01/02/03/04 by PROMPT 1114/1117; 06 by PROMPT 1106/1110; 08/09/16 by PROMPT 1123/1124; 10 by PROMPT 1119/1121); SOURCE-1077-05/07/11/12/13/14/15 deferred to Sprint 18+. PROMPT 1125 does NOT discharge any SOURCE-1077-* finding. **PROMPT 1112 AC3 hand reserve-strip carry (AUDIT-1076-17 floating `Reserve N + / Current N` microbadge in `client/src/ui/hand/mod.rs` reserve strip) preserved OPEN; PROMPT 1125 does NOT close it (semantically distinct surface vs Vulkan validation gating).** External `shop_auction_ui_plugin_scaffold_formulas_test` baseline drift `87 vs 82` preserved verbatim by PROMPT 1124 paperwork and NOT silently fixed by PROMPT 1125. **Files changed by PROMPT 1125**: `production/epics/devops/story-007-vulkan-validation-gating.md` (Status banner Draft -> Done; AC1..AC10 flipped to `[x]` except AC3 -> `[~]` ADVISORY-DEFERRED; Completion Notes section inserted before Closure Trail; Closure Trail extended with numbered closure trail PROMPT 1095 / 1097 / 1099 / 1100 / 1101 / 1103 / 1109 / 1125 + Conditions carried forward + Explicitly NOT claimed; final status line DRAFT -> DONE); `production/sprint-status.yaml` (S17-OPS-VULKAN-VALIDATION-GATING-001 row flipped `status: ready -> done` with `completed: 2026-05-18` + worker/integration/integrated_commit/story_done_prompt/evidence metadata + closure notes; `sprint_17_story_done:` PROMPT 1125 block appended after PROMPT 1124 entry and before `sprint_17_partial_disposition:` block with `stories_closed` covering AC1..AC10, `rows_not_closed_by_prompt_1125`, `conditions_carried_forward_unchanged`, `explicitly_not_claimed`, `files_changed_by_prompt_1125`, `forbidden_changes_observed`); `production/session-state/active.md` (PROMPT 1125 banner prepended above PROMPT 1124 banner); this file (PROMPT 1125 paragraph prepended above PROMPT 1124 paragraph); `reports/PROMPT-1125-s17-vulkan-validation-gating-story-done.md` (mandatory final report; `reports/` gitignored; not staged or committed). **Files explicitly NOT touched by PROMPT 1125**: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-vulkan-validation-gating/ preserved verbatim on `origin/main` via PROMPT 1109 integration), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 8 other Sprint 17 active row story files under `production/epics/`, all prior `sprint_N_*` blocks in `production/sprint-status.yaml` (PROMPT 1108 + PROMPT 1110 + PROMPT 1117 + PROMPT 1120 + PROMPT 1121 + PROMPT 1124 `sprint_17_story_done` entries and PROMPT 1112 `sprint_17_partial_disposition` entry preserved verbatim above and below this PROMPT 1125 entry), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`, root checkout (D:\_DEV\Work\Claude-Code-Game-Studios) — paperwork performed in a fresh worktree. No cargo / trunk / CI command invoked. **Non-claims preserved verbatim by PROMPT 1125**: no Sprint 17 close-out; no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry); no closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (PROMPT 1112 PARTIAL disposition with AC3 carried preserved verbatim; row remains OPEN); no discharge of PROMPT 1112 AC3 hand reserve-strip carry / AUDIT-1076-17 (semantically distinct surface); no fix of external `shop_auction_ui_plugin_scaffold_formulas_test` baseline drift (preserved verbatim); no real-art replacement of any placeholder-art (`PAW-TD-*-a` preserved); no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion; no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* finding outside AUDIT-1076-18 by PROMPT 1125; no closure of any SOURCE-1077-* finding by PROMPT 1125; no closure of any of the 24 PROMPT 1022 audit findings; no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen; no cleanup or reset of the root checkout's local divergent `main` by PROMPT 1125 (worktree-isolated paperwork; root checkout untouched). **Next launchable prompts**: AC3 carry-forward producer decision on `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Sprint 17 pull vs Sprint 18 deferral) before final `/story-done` closure of that row; Sprint 17 smoke harness prompt with two-client session (binding for AC3 carry-forward and for `S17-OPS-VULKAN-VALIDATION-GATING-001` AC3); Sprint 17 close-out paperwork once the partial / in_progress row converges; counter-reconciliation candidate story authoring against external `shop_auction_ui_plugin_scaffold_formulas_test` baseline drift (Sprint 18+ recommendation; NOT activated by PROMPT 1125). **Branch / push**: PROMPT 1125 commits `story-done(s17): close S17-OPS-VULKAN-VALIDATION-GATING-001 (PROMPT 1125)` on worker branch `worker/vulkan-validation-gating-story-done-1125`; pushes to `origin/main` if policy allows; if direct `main` push is blocked, the commit is pushed on the worker branch and the exact commit/branch reported, never force-pushed.)

Updated: 2026-05-18 (PROMPT 1124 -- Sprint 17 `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` `/story-done` closure; paperwork-only single-shared-status writer. Source-of-truth at closure: `origin/main@74c25b65fbb08da966c2a4e9607812ec34ae610e` = PROMPT 1123 integration tip `integrate(s17): merge PROMPT 1122 qa-snapshot marker-split into main (PROMPT 1123)` merging PROMPT 1122 worker `f4d0fda49e303c3c0b3d3cef57a37ff6de350ea8` `dev-story(s17-qa-snapshot-marker-split): per-sub-surface markers + visibility-aware counts + session-id snapshot prefix (PROMPT 1122)` onto `origin/main` via `--no-ff` merge (rebased mid-run by PROMPT 1123 onto `origin/main@0cab942` PROMPT 1109 Vulkan validation gating integration tip when concurrent integration landed; PROMPT 1109's `wgpu-validation` feature + S17-OPS comment block at `client/Cargo.toml:34-40` preserved verbatim alongside the marker-split test registration via clean `client/Cargo.toml` auto-merge). `origin/main` HEAD at the start of this PROMPT 1124 closure window is also `74c25b6`. Strict fast-forward descendant of `origin/main@d35d24d` PROMPT 1119 bid-button phase-race integration tip, of `origin/main@5345164` PROMPT 1121 bid-button story-done tip, of `origin/main@89ce149` PROMPT 1120 hand-fan-root B0004 story-done tip, of `origin/main@29ad4c6` PROMPT 1118 hand-fan-root B0004 integration tip, of `origin/main@2250add` PROMPT 1117 card-display art-helper story-done tip, of `origin/main@30c9e0f` PROMPT 1114 card-display art-helper integration tip, of `origin/main@f2ba917` PROMPT 1112 paperwork-only PARTIAL DISPOSITION tip, of `origin/main@4bd4f56` PROMPT 1111 PARTIAL integration tip, of `origin/main@9a9b1dc` PROMPT 1110 card-slot inset wiring story-done tip, of `origin/main@30f166f` PROMPT 1106 card-slot inset wiring integration tip, of `origin/main@72d56bc` PROMPT 1108 server start-of-turn-debug story-done tip, and of `origin/main@dc8adb6` PROMPT 1107 server warn->debug integration tip. Branch: `story-done/s17-qa-snapshot-marker-split-1124` (fresh worktree from `origin/main@74c25b6`, NOT in-place on the root checkout's local `main` due to the PROMPT 1123-reported branch-state anomaly where Bash-tool MSYS path-mangling on the initial `cd` collapsed backslash-quoted paths so the first merge attempt landed two local-only commits `893d16b` + parent on the root checkout's local main without push; recovery actions performed by PROMPT 1123; origin/main is authoritative and is at `74c25b6`; local-only commits never pushed; PROMPT 1124 paperwork performed on a fresh worktree to avoid acting on the local divergent tip; recommended cosmetic `git reset --hard origin/main` on root checkout is purely local-state hygiene and NOT performed by PROMPT 1124). Single-context paperwork-only `/story-done` run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. **Single-row closure**: `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (Should Have; story 019 in `production/epics/ui-clean-pass/`; SOURCE-1077-08 P2 + SOURCE-1077-09 P2 + SOURCE-1077-16 P3 bundle) closed on basis of PROMPT 1122 worker (per-sub-surface markers introduced — HUD: `HudTopStripRoot` + `HudBottomStripRoot` + `HudScoreboardDotRoot` + `HudDimOverlayRoot` at `client/src/ui/hud/mod.rs`; Hand: `HandBarRoot` + `HandDraftGridSlotRoot` + `PlacementActionPanelRoot` at `client/src/ui/hand/mod.rs` alongside pre-existing `HandFanRoot`; Shop/Auction: pre-existing `ShopAuctionPanelRoot` enum declared canonical per-sub-surface marker and consumed by `UiCountQueries`; universal markers `HudEntity` / `HandUiEntity` / `ShopAuctionUiEntity` marked `#[deprecated(since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001", note = …)]`; `UiCountQueries` extended with per-sub-surface visibility queries via `HandVisibilityQueries` + `ShopAuctionVisibilityQueries` sub-`SystemParam`s under the 16-field ceiling; `UiCounts` gains 17 new `*_visible` fields filtered by `is_visibility_visible` (true when `Visibility != Hidden`); `format_snapshot_id(counter, unix_millis, session_id: Option<u64>)` at `client/src/presentation/qa_snapshot.rs` emits `{session_id}-{counter:06}-{unix_millis}` post-handshake and `pre-session-{counter:06}-{unix_millis}` before handshake via new `QA_SNAPSHOT_PRE_SESSION_PREFIX = "pre-session"` constant; `short_id` surfaces the counter token under the new format; legacy `hud_entities` / `hand_ui_entities` / `shop_auction_entities` / `connection_lost_overlay_roots` / `result_screen_roots` retained as `#[deprecated]` alongside new `*_visible` fields (AC8 option (a)); `CCGS_QA_SNAPSHOT=1` env-var contract preserved verbatim — `QA_SNAPSHOT_ENV_VAR` constant + `from_env_values` behaviour unchanged; AC7 satisfied via worker-allowable fallback — fixture-injected `ClientSessionIdentity` with two distinct `session_id` values in a single-client test bin per story AC7 phrasing since running two clients during `/dev-story` is not feasible; PAW-TD-*-a preserved verbatim — no `.png` files touched; new test bin `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs` 11/11 PASS at worker; existing `qa_snapshot_overlay_test` bin 20/20 PASS at worker after `..UiCounts::default()` extension + worker-side `#[allow(deprecated)]`; 8 adjacent surface focused bins PASS at worker; `cargo check -p client` exit 0 with 82 intentional deprecation warnings; worker pushed `work/s17-qa-snapshot-marker-split` only — never main) + PROMPT 1123 integration (`--no-ff` merge onto `origin/main`, rebased mid-run onto `origin/main@0cab942` so PROMPT 1109 Vulkan validation gating preserved verbatim via clean `client/Cargo.toml` auto-merge; `git diff --name-only origin/main...HEAD` returns exactly 8 paths: `client/Cargo.toml` test-registration-only + `client/src/presentation/qa_snapshot.rs` + `client/src/ui/hand/mod.rs` + `client/src/ui/hud/mod.rs` + `client/src/ui/shop_auction/mod.rs` + `production/qa/evidence/sprint-17-qa-snapshot-marker-split/evidence.md` NEW + `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs` extended + `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs` NEW; zero changes under `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`; `cargo check -p client` PASS at integration tip (`Finished dev profile [optimized]`, 82 intentional deprecation warnings, zero errors, zero new non-deprecation warnings); 11/11 PASS on new test bin + 20/20 PASS on extended `qa_snapshot_overlay_test` + 47/47 PASS across 8 adjacent surface focused bins (`hud_plugin_scaffold_test 4/4`, `hand_ui_plugin_scaffold_test 3/3`, `hud_top_strip_layout_test 8/8`, `hud_bottom_strip_layout_test 8/8`, `shop_auction_ui_shop_panel_test 10/10`, `shop_auction_ui_auction_activation_test 8/8`, `shop_auction_ui_draft_initial_grid_test 10/10`, `hand_fan_root_b0004_hierarchy_test 1/1`) at integration tip; `git diff --check origin/main...HEAD` clean; integration tip pushed `0cab942..74c25b6` on `main` as strict fast-forward). **AC1..AC16 PASS** at integration tip — Per-sub-surface markers split (AC1); UiCountQueries consumes per-sub-surface markers + JSON emits per-sub-surface counts (AC2); visibility filter applied (AC3 — asserted by `hidden_visibility_excludes_marker_from_per_sub_surface_counts` + `inherited_visibility_counts_as_visible`); connection-lost overlay visible flag honours Visibility (AC4); result-screen overlay visible flag honours Visibility (AC5); snapshot ID prefix includes session_id / pre-session- (AC6 — asserted by `pre_session_prefix_used_when_session_id_is_none` + `session_id_prefix_used_when_session_id_is_some`); two-client capture does not alias snapshot directories (AC7 — fixture fallback per story AC7 phrasing); legacy universal counts preserved as `#[deprecated]` (AC8 option (a)); 11-test integration bin covers marker split (AC9); CCGS_QA_SNAPSHOT=1 env-var contract preserved (AC10); no protocol or server change (AC11 — diff scope confirmed); ADR-021 schedule preserved (AC12 — no new SystemSet); no accept-risk closure (AC13); Sprint 17 disposition preserved by worker + integration (AC14); worker branch scope contained (AC15); Cargo resource policy applied — PASS-WORKER + ADVISORY-INTEGRATION (AC16; PROMPT 1122 worker applied all 5 env vars before every cargo invocation; PROMPT 1123 integration encountered a one-call env-var propagation gap on the first `cargo check -p client` invocation where the `powershell -NoProfile -Command` wrapper's `$env:` block was eaten by the bash shell layer, so cargo built to worktree-local `target/` once at `[optimized + debuginfo]` ~1m42s; all subsequent invocations applied policy correctly via bash inline `VAR=... cargo ...` syntax; D: free remained > 718 GB throughout; build correctness gate unaffected — 87/87 targeted sub-tests + 2 cargo check invocations PASS at integration tip under both the first-call worktree-local and the subsequent policy-compliant routes; the deviation affected disk-cache placement only, not build correctness; advisory recorded explicitly per `cargo_resource_policy_advisory` block in `production/sprint-status.yaml` PROMPT 1124 `sprint_17_story_done:` + in the story-019 Completion Notes section + in this paragraph + in the `production/session-state/active.md` PROMPT 1124 banner + in the PROMPT 1124 final report; **NOT hidden as a product failure**; PROMPT 1124 itself does NOT invoke Cargo — paperwork-only closure). **External baseline drift preserved verbatim**: `shop_auction_ui_plugin_scaffold_formulas_test::shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` fails with `left: 87, right: 82` on the unmodified `origin/main@5345164` baseline before any PROMPT 1122 edit (confirmed by stashing PROMPT 1122 changes and re-running). Hand-tuned arithmetic in `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:50` drifted from the actual `ShopAuctionUiEntity`-tagged spawn count introduced by an upstream landing. PROMPT 1122 makes no edit to that formula; PROMPT 1123 integration preserved the drift verbatim (no conflict required touching the counts); PROMPT 1124 paperwork does NOT silently fix the counts and does NOT advance closure on the drift; recommend a separate follow-up story for counter reconciliation against the post-marker-split spawn tree (NOT activated or authored by PROMPT 1124); advisory recorded explicitly per `external_baseline_drift_advisory` block in `production/sprint-status.yaml` PROMPT 1124 `sprint_17_story_done:` + in the story-019 Completion Notes section + in this paragraph + in the `production/session-state/active.md` PROMPT 1124 banner + in the PROMPT 1124 final report; **explicitly NOT hidden**. **Sprint 17 progress after PROMPT 1124**: 6 of 9 active rows DONE (S17-SERVER-START-OF-TURN-DEBUG-001 by PROMPT 1108 + S17-UI-CARD-SLOT-INSET-WIRING-001 by PROMPT 1110 + S17-UI-CARD-DISPLAY-ART-HELPER-001 by PROMPT 1117 + S17-UI-HAND-B0004-CLEANUP-001 by PROMPT 1120 + S17-UI-BID-BUTTON-PHASE-RACE-001 by PROMPT 1121 + S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 by PROMPT 1124) + 1 of 9 active rows PARTIAL / IN_PROGRESS (S17-UI-HUD-OPP-MANA-CLEANUP-001 carried via PROMPT 1112 with AC3 carried) + 2 rows preserved as `ready`. Must Have 1/2 done + Should Have 3/4 done + 1 partial / 4 + Nice to Have 2/3 done. Rows preserved as `ready` and NOT closed or partial by PROMPT 1124: `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; no LLM `/story-done` authorised per 2026-05-17 orchestrator decision); `S17-OPS-VULKAN-VALIDATION-GATING-001` (1 of 1 remaining Nice to Have; source-side discharge of AUDIT-1076-18 on origin/main via PROMPT 1103/1109 integration acknowledged but row paperwork closure not in scope for PROMPT 1124). **SOURCE-1077-08 + SOURCE-1077-09 + SOURCE-1077-16 discharged on origin/main by PROMPT 1123 integration** (the closure target of this row). Other SOURCE-1077-* findings: SOURCE-1077-01/02/03/04 discharged by PROMPT 1114/1117 closure; SOURCE-1077-06 by PROMPT 1106/1110; SOURCE-1077-10 by PROMPT 1119/1121; SOURCE-1077-05/07/11/12/13/14/15 deferred to Sprint 18+. PROMPT 1124 does NOT discharge any SOURCE-1077-* finding outside SOURCE-1077-08/09/16. All AUDIT-1076-* findings preserved outside AUDIT-1076-14 (discharged PROMPT 1118/1120), AUDIT-1076-15 (discharged PROMPT 1107/1108), AUDIT-1076-18 (source-side discharged PROMPT 1103/1109), AUDIT-1076-10 + AUDIT-1076-16 (discharged PROMPT 1111; AUDIT-1076-17 remains OPEN carried with AC3 of `S17-UI-HUD-OPP-MANA-CLEANUP-001`). **PROMPT 1112 AC3 hand reserve-strip carry (AUDIT-1076-17 floating `Reserve N + / Current N` microbadge in `client/src/ui/hand/mod.rs` reserve strip) preserved OPEN; PROMPT 1124 does NOT close it (semantically distinct surface: hand reserve-strip per-card microbadge vs QA snapshot marker split tooling; PROMPT 1122 worker added per-sub-surface markers to `client/src/ui/hand/mod.rs` but did NOT alter the reserve-strip mana-microbadge surface; the story file, PROMPT 1123 integration report, and this paperwork all carry this distinction explicitly).** **Files changed by PROMPT 1124**: `production/epics/ui-clean-pass/story-019-qa-snapshot-marker-split.md` (Status banner Draft -> Done; AC1..AC16 flipped to `[x]`; Completion Notes section inserted before Closure Trail containing PROMPT 1122 worker + PROMPT 1123 integration outcome + Test evidence + Cargo resource policy (AC16) advisory + Per-AC outcome + External / baseline drift advisory; Closure Trail extended with numbered closure trail commits (PROMPT 1095 / 1097 / 1099 / 1100 / 1122 / 1123 / 1124) and additional Conditions carried forward unchanged entries covering PROMPT 1108 / 1110 / 1112 / 1117 / 1120 / 1121 + PROMPT 1109 Vulkan gating preservation + PROMPT 1112 AC3 carry preserved OPEN + SOURCE-1077-* + AUDIT-1076-* disposition trace + external baseline drift preservation; final status line DRAFT -> DONE); `production/sprint-status.yaml` (S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 row flipped `status: ready -> done` with `completed: 2026-05-18` + worker/integration/integrated_commit/story_done_prompt/evidence metadata + closure notes; `sprint_17_story_done:` PROMPT 1124 block appended after PROMPT 1121 entry and before `sprint_17_partial_disposition:` block with `stories_closed` covering AC1..AC16, `rows_not_closed_by_prompt_1124`, `cargo_resource_policy_advisory`, `external_baseline_drift_advisory`, `conditions_carried_forward_unchanged`, `explicitly_not_claimed`, `files_changed_by_prompt_1124`, `forbidden_changes_observed`); `production/session-state/active.md` (PROMPT 1124 banner prepended above PROMPT 1121 banner); this file (PROMPT 1124 paragraph prepended above PROMPT 1121 paragraph); `reports/PROMPT-1124-s17-qa-snapshot-marker-split-story-done.md` (mandatory final report; `reports/` gitignored; not staged or committed). **Files explicitly NOT touched by PROMPT 1124**: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-qa-snapshot-marker-split/ preserved verbatim on `origin/main` via PROMPT 1123 integration), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 8 other Sprint 17 active row story files under `production/epics/`, all prior `sprint_N_*` blocks in `production/sprint-status.yaml` (PROMPT 1108 + PROMPT 1110 + PROMPT 1117 + PROMPT 1120 + PROMPT 1121 `sprint_17_story_done` entries and PROMPT 1112 `sprint_17_partial_disposition` entry preserved verbatim above and below this PROMPT 1124 entry), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1124 paperwork-only closure. **Non-claims preserved verbatim by PROMPT 1124**: no Sprint 17 close-out; no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry); no closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (PROMPT 1112 PARTIAL disposition with AC3 carried preserved verbatim; row remains OPEN); no closure of `S17-OPS-VULKAN-VALIDATION-GATING-001` sprint-status row (source-side discharge on origin/main via PROMPT 1103/1109 acknowledged; row paperwork closure deferred to separate prompt); no discharge of PROMPT 1112 AC3 hand reserve-strip carry / AUDIT-1076-17 (semantically distinct surface); no fix of external `shop_auction_ui_plugin_scaffold_formulas_test` baseline drift (preserved verbatim); no real-art replacement of any placeholder-art (`PAW-TD-*-a` preserved); no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion; no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* finding by PROMPT 1124; no closure of any SOURCE-1077-* finding by PROMPT 1124 outside SOURCE-1077-08/09/16 (which were discharged by PROMPT 1123 integration; this row's `/story-done` paperwork records the closure but does not re-discharge); no closure of any of the 24 PROMPT 1022 audit findings (this row improves the tool that captured them; it does NOT retest or close them); no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen. **Next launchable prompts**: per-row `/story-readiness` rerun against PROMPT 1124 closure HEAD for the remaining ready row `S17-OPS-VULKAN-VALIDATION-GATING-001` before its `/story-done` paperwork (source-side already discharged via PROMPT 1109; row closure is a clean follow-up); AC3 carry-forward producer decision on `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Sprint 17 pull vs Sprint 18 deferral) before final `/story-done` closure of that row; counter-reconciliation candidate story authoring against the external `shop_auction_ui_plugin_scaffold_formulas_test` baseline drift (Sprint 18+ recommendation; NOT activated by PROMPT 1124); Sprint 17 smoke harness prompt with two-client session. **Branch / push**: PROMPT 1124 commits `story-done(s17): close S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 (PROMPT 1124)` on worker branch `story-done/s17-qa-snapshot-marker-split-1124`; pushes to `origin/main` if policy allows; if direct `main` push is blocked, the commit is pushed on the worker branch and the exact commit/branch reported, never force-pushed.)

Updated: 2026-05-18 (PROMPT 1121 -- Sprint 17 `S17-UI-BID-BUTTON-PHASE-RACE-001` `/story-done` closure; paperwork-only single-shared-status writer. Source-of-truth at closure: `origin/main@d35d24d01ebc59e387ba749883596aa1f29418e4` = PROMPT 1119 integration tip `integrate(s17): merge PROMPT 1116 bid-button phase-race into main (PROMPT 1119)` merging PROMPT 1116 worker `6f8f5ae169a3ca89568a0c8bea49326eb818ec73` `dev-story(s17-bid-button-phase-race): spawn-state Loading… + HiddenLeading chrome override (PROMPT 1116)` onto `origin/main` via no-ff merge (rebased mid-run by PROMPT 1119 onto PROMPT 1118 integration tip `origin/main@29ad4c6cb066f61d82db02820d42612e66a97256` when concurrent integration landed). `origin/main` HEAD at the start of this PROMPT 1121 closure window is `89ce1498953c362a86d70037bbc71f7452fdcab5` (PROMPT 1120 `story-done(s17): close S17-UI-HAND-B0004-CLEANUP-001 (PROMPT 1120)`), a strict fast-forward descendant of `d35d24d`. Strict fast-forward descendant of `origin/main@29ad4c6` PROMPT 1118 hand-fan-root B0004 cleanup integration tip, of `origin/main@30c9e0f` PROMPT 1114 card-display art-helper integration tip, of `origin/main@2250add` PROMPT 1117 card-display art-helper story-done tip, of `origin/main@f2ba917` PROMPT 1112 paperwork-only PARTIAL DISPOSITION tip, of `origin/main@4bd4f56` PROMPT 1111 PARTIAL integration tip, of `origin/main@9a9b1dc` PROMPT 1110 card-slot inset wiring story-done tip, of `origin/main@30f166f` PROMPT 1106 card-slot inset wiring integration tip, of `origin/main@72d56bc` PROMPT 1108 server start-of-turn-debug story-done tip, and of `origin/main@dc8adb6` PROMPT 1107 server warn->debug integration tip. Branch: in-place edits on the primary checkout (`main`). Single-context paperwork-only `/story-done` run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. **Single-row closure**: `S17-UI-BID-BUTTON-PHASE-RACE-001` (Should Have; SOURCE-1077-10 P2 bid-button text-spawn / chrome-asset race) closed on basis of PROMPT 1116 worker (three concrete source edits in `client/src/ui/shop_auction/mod.rs`: new `pub const AUCTION_BID_BUTTON_LOADING_LABEL = "Loading…"` at module scope; `Text::new("")` → `Text::new(AUCTION_BID_BUTTON_LOADING_LABEL)` at spawn site; `sync_auction_panel_system` text branch surfaces pending label when `card_id.is_none()`; chrome apply site falls back to `Handle::<Image>::default()` when `auction_bid_chrome_state` returns `None`; `auction_bid_chrome_state` now returns `Option<BidButtonChromeState>` with `HiddenLeading => None` branch — `Normal`/`Disabled` mappings preserved verbatim; AC3 strategy = both (a) `Visibility::Hidden` AND (b) `Handle::<Image>::default()` chrome simultaneously; chrome override narrowed to `HiddenLeading` only to preserve `chrome_wiring_test` invariant; `assets/art/ui/auction/ui_bid_button_disabled.png` NOT modified — `PAW-TD-*-a` preserved; new test bin `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs` 5/5 PASS at worker; worker pushed `work/s17-bid-button-phase-race` only — never main) + PROMPT 1119 integration (no-ff merge onto `origin/main`; `git diff --name-only origin/main..HEAD` returns exactly 4 paths: `client/Cargo.toml` test-registration-only + `client/src/ui/shop_auction/mod.rs` + `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs` NEW + `production/qa/evidence/sprint-17-bid-button-phase-race/evidence.md` NEW; zero changes under `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`; `cargo check -p client` PASS in 7.04s with `Finished dev profile [optimized]`; 5/5 PASS on new test bin + 42/42 PASS across 7 adjacent bid-button bins (`auction_activation_test 8/8`, `auction_bid_buttons_test 9/9`, `auction_bid_target_focus_test 4/4`, `auction_feedback_test 6/6`, `auction_lead_loss_state_test 4/4`, `auction_settlement_test 7/7`, `chrome_wiring_test 4/4`) at integration tip; `git diff --check origin/main...HEAD` clean; integration tip pushed `29ad4c6..d35d24d` on `main` as strict fast-forward). **AC1 PASS** (spawn-state text is `"Loading…"` -- new `pub const AUCTION_BID_BUTTON_LOADING_LABEL` at `client/src/ui/shop_auction/mod.rs:40`; spawn site `Text::new(AUCTION_BID_BUTTON_LOADING_LABEL)` at `mod.rs:4883`; asserted by `s17_phase_race_ac1_spawn_state_text_is_loading_label`). **AC2 PASS** (text updates to numeric bid amounts on `S2CAuctionCard` arrival -- `sync_auction_panel_system` keeps Loading label only while `card_id.is_none()`; existing TR-SAU-002 numeric formula drives text after drain; asserted by `s17_phase_race_ac2_text_updates_to_numeric_on_auction_card_arrival`). **AC3 PASS** (`HiddenLeading` chrome / visibility override -- strategy = both (a) `Visibility::Hidden` AND (b) `Handle::<Image>::default()` chrome simultaneously; `auction_bid_chrome_state` now returns `Option<BidButtonChromeState>` with `HiddenLeading => None`; chrome apply site falls back to `Handle::<Image>::default()` when mapper returns `None`; asserted by `s17_phase_race_ac3_ac5_hidden_leading_clears_chrome_and_hides_row`). **AC4 PASS** (visible `?` does not surface during phase-entry race -- bid buttons carry Loading label while `card_id.is_none()`; chrome not forced to `Disabled` at spawn; asserted by `s17_phase_race_ac4_draft_auction_without_card_keeps_loading_or_hidden`). **AC5 PASS** (visible `?` does not surface during `HiddenLeading` -- covered by same `s17_phase_race_ac3_ac5_hidden_leading_clears_chrome_and_hides_row` test; row hidden via Visibility + chrome handle = `default()` so baked-`?` PNG is not on the entity). **AC6 PASS** (existing PROMPT 1042 Pass affordance preserved -- `shop_auction_ui_auction_bid_buttons_test 9/9` PASS at integration tip). **AC7 PASS** (`auction_bid_chrome_state` `Normal`/`Disabled` mappings preserved -- only `HiddenLeading => None` branch is new; asserted by `s17_phase_race_ac7_chrome_mapping_preserved_for_enabled_and_disabled_states`). **AC8 PASS** (`ui_bid_button_disabled.png` not modified -- PROMPT 1119 integration `git diff --name-only origin/main..HEAD` touches zero `.png` files; `PAW-TD-*-a` preserved). **AC9 PASS** (integration test bin authored -- `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs` NEW; 5/5 pass at integration tip; covers AC1, AC2, AC3, AC4, AC5, AC7 against a real Bevy 0.18 `App`). **AC10 PASS** (no protocol or server change -- PROMPT 1119 integration diff returns exactly 4 paths; zero changes under `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`). **AC11 PASS** (ADR-021 schedule preserved -- no new `SystemSet`, no schedule wiring change; `cargo check -p client` at integration tip PASS in 7.04s). **AC12 PASS** (no accept-risk closure -- worker commit + evidence.md + integration merge commit + this PROMPT 1121 paperwork all explicitly preserve `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 761 `Polish->Release` FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all `AUDIT-1076-*` findings, all `SOURCE-1077-*` findings outside SOURCE-1077-10, all 24 PROMPT 1022 findings, the PROMPT 1112 AC3 hand reserve-strip carry preserved OPEN; `A11Y-ST-12` NOT advanced; final-art replacement of the baked-`?` PNG NOT pursued; Standard-tier hit-target conformance NOT pursued; playtest validation NOT pursued). **AC13 PASS** (PROMPT 1116 worker + PROMPT 1119 integration diffs touched zero files under `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md`; PROMPT 1121 is the first authorised modifier of `production/sprint-status.yaml` + `production/session-state/*` for this row). **AC14 PASS** (PROMPT 1116 worker pushed `work/s17-bid-button-phase-race` (`6f8f5ae`) only -- never main; files changed at worker time: `client/src/ui/shop_auction/mod.rs` (three edits per AC1/AC3/AC7), `client/Cargo.toml` (single additive `[[test]]` block at line ~513), new test bin, evidence document; integration into `origin/main` performed separately by PROMPT 1119 via `integrate/s17-bid-button-phase-race-1119` -> `d35d24d`). **AC15 PASS-WORKER + ADVISORY-INTEGRATION** (PROMPT 1116 worker applied all 5 Cargo resource policy env vars `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` + `CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'` before every cargo invocation; build line printed `Finished dev profile [optimized]` (no `+ debuginfo`), confirming the policy was applied; D: free ≈ 774 GB at worker session start. PROMPT 1119 integration encountered a one-call env-var propagation gap on the first `cargo check -p client` invocation: the `$env:CARGO_TARGET_DIR=...` block issued through the Bash tool did not propagate, so that one call used the integration worktree-local `target/` rather than `D:\_DEV\cargo-target\ccgs-msvc`. All subsequent invocations used bash inline env-var syntax that successfully exported the variables. Resource impact: a few GB local to the worktree; D: free remained > 770 GB throughout. The build correctness gate the integration prompt required is **unaffected** -- all 47/47 targeted sub-tests + 2 cargo check invocations PASS against the merged tree. Recorded explicitly here, in the `production/sprint-status.yaml` PROMPT 1121 `sprint_17_story_done:` `cargo_resource_policy_advisory:` block + `batch_note`, in the story-019 Completion Notes section, in the `production/session-state/active.md` PROMPT 1121 banner, and in the PROMPT 1121 final report; **NOT hidden as a product failure**. PROMPT 1121 itself does NOT invoke Cargo (paperwork-only closure)). **Sprint 17 progress after PROMPT 1121**: 5 of 9 active rows DONE (S17-SERVER-START-OF-TURN-DEBUG-001 by PROMPT 1108 + S17-UI-CARD-SLOT-INSET-WIRING-001 by PROMPT 1110 + S17-UI-CARD-DISPLAY-ART-HELPER-001 by PROMPT 1117 + S17-UI-HAND-B0004-CLEANUP-001 by PROMPT 1120 + S17-UI-BID-BUTTON-PHASE-RACE-001 by PROMPT 1121) + 1 of 9 active rows PARTIAL / IN_PROGRESS (S17-UI-HUD-OPP-MANA-CLEANUP-001 carried via PROMPT 1112 with AC3 carried) + 3 rows preserved as `ready`. Must Have 1/2 done + Should Have 2/4 done + 1 partial / 4 + Nice to Have 2/3 done. Rows preserved as `ready` and NOT closed or partial by PROMPT 1121: `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; no LLM `/story-done` authorised per 2026-05-17 orchestrator decision); `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (1 of remaining 1 Should Have); `S17-OPS-VULKAN-VALIDATION-GATING-001` (1 of remaining 1 Nice to Have). **SOURCE-1077-10 (P2) discharged on origin/main by PROMPT 1119 integration** (the closure target of this row). Other SOURCE-1077-* findings: SOURCE-1077-01/02/03/04 discharged by PROMPT 1114/1117 closure; SOURCE-1077-06 by PROMPT 1106/1110; SOURCE-1077-08/09/16 reserved for `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`; SOURCE-1077-05/07/11/12/13/14/15 deferred to Sprint 18+. PROMPT 1121 does NOT discharge any SOURCE-1077-* finding outside SOURCE-1077-10. All AUDIT-1076-* findings preserved outside AUDIT-1076-14 (discharged PROMPT 1118/1120), AUDIT-1076-15 (discharged PROMPT 1107/1108), AUDIT-1076-10 + AUDIT-1076-16 (discharged PROMPT 1111; AUDIT-1076-17 remains OPEN carried with AC3 of `S17-UI-HUD-OPP-MANA-CLEANUP-001`). **PROMPT 1112 AC3 hand reserve-strip carry (AUDIT-1076-17 floating `Reserve N + / Current N` microbadge in `client/src/ui/hand/mod.rs` reserve strip) preserved OPEN; PROMPT 1121 does NOT close it (semantically distinct surface: hand reserve-strip per-card microbadge vs auction bid-button chrome / spawn-state behaviour; the story file, PROMPT 1119 integration report, and this paperwork all carry this distinction explicitly).** **Files changed by PROMPT 1121**: `production/epics/shop-auction-ui/story-019-bid-button-phase-race.md` (Status banner Draft -> Done; AC1..AC15 flipped to `[x]`; Closure Trail section replaced with Completion Notes containing PROMPT 1116 worker + PROMPT 1119 integration outcome + Test evidence + Cargo resource policy advisory + Per-AC outcome + Closure trail (commits) numbered list + Conditions carried forward + Explicitly NOT claimed; final status line DRAFT -> DONE); `production/sprint-status.yaml` (S17-UI-BID-BUTTON-PHASE-RACE-001 row flipped `status: ready -> done` with `completed: 2026-05-18` + worker/integration/integrated_commit/story_done_prompt/evidence metadata + closure notes; `sprint_17_story_done:` PROMPT 1121 block appended after PROMPT 1120 entry and before `sprint_17_partial_disposition:` block); `production/session-state/active.md` (PROMPT 1121 banner prepended above PROMPT 1120 banner); this file (PROMPT 1121 paragraph prepended above PROMPT 1120 paragraph); `reports/PROMPT-1121-s17-bid-button-phase-race-story-done.md` (mandatory final report; `reports/` gitignored; not staged or committed). **Files explicitly NOT touched by PROMPT 1121**: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-bid-button-phase-race/ preserved verbatim on `origin/main` via PROMPT 1119 integration), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 8 other Sprint 17 active row story files under `production/epics/`, all prior `sprint_N_*` blocks in `production/sprint-status.yaml` (PROMPT 1108 + PROMPT 1110 + PROMPT 1117 + PROMPT 1120 `sprint_17_story_done` entries and PROMPT 1112 `sprint_17_partial_disposition` entry preserved verbatim above and below this PROMPT 1121 entry), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1121 paperwork-only closure. **Non-claims preserved verbatim by PROMPT 1121**: no Sprint 17 close-out; no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry); no closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (PROMPT 1112 PARTIAL disposition with AC3 carried preserved verbatim; row remains OPEN); no discharge of PROMPT 1112 AC3 hand reserve-strip carry / AUDIT-1076-17 (semantically distinct surface); no real-art replacement of `ui_bid_button_disabled.png` (`PAW-TD-*-a` preserved); no closure of `A11Y-ST-12` (Shop / Auction UI epic stories 005 and 011 own bid-button focus / target size accessibility); no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion; no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* finding by PROMPT 1121; no closure of any SOURCE-1077-* finding by PROMPT 1121 outside SOURCE-1077-10 (which was discharged by PROMPT 1119 integration; this row's `/story-done` paperwork records the closure but does not re-discharge); no closure of any of the 24 PROMPT 1022 audit findings; no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen. **Next launchable prompts**: per-row `/story-readiness` reruns against PROMPT 1121 closure HEAD for the 2 remaining `ready` Sprint 17 active rows (`S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`, `S17-OPS-VULKAN-VALIDATION-GATING-001`) + AC3 carry-forward producer decision on `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Sprint 17 pull vs Sprint 18 deferral) before final `/story-done` closure of that row; per-row `/dev-story` + integration + `/story-done` paperwork for the remaining `ready` rows; Sprint 17 smoke harness prompt with two-client session. **Branch / push**: PROMPT 1121 commits `story-done(s17): close S17-UI-BID-BUTTON-PHASE-RACE-001 (PROMPT 1121)` on `main`; pushes to `origin/main` if policy allows; if direct `main` push is blocked, the commit is pushed on a worker branch and the exact commit/branch reported, never force-pushed.)

Updated: 2026-05-18 (PROMPT 1120 -- Sprint 17 `S17-UI-HAND-B0004-CLEANUP-001` `/story-done` closure; paperwork-only single-shared-status writer. Source-of-truth at closure: `origin/main@d35d24d01ebc59e387ba749883596aa1f29418e4` = PROMPT 1119 integration tip `integrate(s17): merge PROMPT 1116 bid-button phase-race into main (PROMPT 1119)`, a strict fast-forward descendant of the PROMPT 1118 integration tip `origin/main@29ad4c6cb066f61d82db02820d42612e66a97256` (`integrate(s17): merge PROMPT 1115 hand-fan-root B0004 cleanup into main (PROMPT 1118)`) which is the integration tip for the row this prompt closes. PROMPT 1119 (bid-button phase-race integration) landed during the PROMPT 1120 closure window after the initial origin/main read at 29ad4c6; PROMPT 1120 rebased its in-place paperwork edits onto the new tip via `git pull --rebase --autostash`. PROMPT 1119 touched only `client/`, `tests/`, `evidence/` paths; zero file overlap with PROMPT 1120 paperwork-only edits. PROMPT 1118 merged PROMPT 1115 worker `535450d101fb034f5946896f303533f4ce4f6435` `dev-story(s17-hand-fan-root-b0004): Strategy A — Transform on HandBar (PROMPT 1115)` onto `origin/main` via no-ff merge; strict fast-forward descendant of `origin/main@30c9e0f` PROMPT 1114 card-display art-helper integration tip, of `origin/main@2250add` PROMPT 1117 card-display art-helper story-done tip, of `origin/main@f2ba917` PROMPT 1112 paperwork-only PARTIAL DISPOSITION tip, of `origin/main@4bd4f56` PROMPT 1111 PARTIAL integration tip, of `origin/main@9a9b1dc` PROMPT 1110 card-slot inset wiring story-done tip, of `origin/main@30f166f` PROMPT 1106 card-slot inset wiring integration tip, of `origin/main@72d56bc` PROMPT 1108 server start-of-turn-debug story-done tip, and of `origin/main@dc8adb6` PROMPT 1107 server warn->debug integration tip. Branch: `story-done/s17-hand-b0004-cleanup-1120` from base `origin/main@29ad4c6`; in-place edits on the primary checkout. Single-context paperwork-only `/story-done` run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. **Single-row closure**: `S17-UI-HAND-B0004-CLEANUP-001` (Nice to Have; AUDIT-1076-14 P3 B0004 hierarchy warning) closed on basis of PROMPT 1115 worker (Strategy A applied: single-component `Transform::default()` insert on the `HandBar` strip entity at `client/src/ui/hand/mod.rs:3122` inside `spawn_hand_ui`; Bevy 0.18 Required Components API auto-derives `GlobalTransform` from `Transform`, silencing the `B0004` hierarchy warning AUDIT-1076-14 cited on every InSession entry; new test bin `tests/integration/hand-ui/hand_fan_root_b0004_hierarchy_test.rs` 1/1 PASS; worker pushed `work/s17-hand-fan-root-b0004-cleanup` only -- never main) + PROMPT 1118 integration (no-ff merge onto `origin/main`; `git diff --name-only origin/main..HEAD` returns exactly 4 paths: `client/Cargo.toml` test-registration-only + `client/src/ui/hand/mod.rs` + `tests/integration/hand-ui/hand_fan_root_b0004_hierarchy_test.rs` NEW + `production/qa/evidence/sprint-17-hand-fan-root-b0004/evidence.md` NEW; zero changes under `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`; `cargo check -p client` PASS in 10.43s with `Finished dev profile [optimized]`; 1/1 PASS on new test bin + 10 adjacent hand UI bins PASS at integration tip; `git diff --check origin/main...HEAD` clean; integration tip pushed `2250add..29ad4c6` on `main`). **AC1 PASS** (B0004 warning gone (default build) -- covered by AC4 in-test invariant; cross-machine smoke evidence remains later-prompt scope and is NOT claimed here). **AC2 PASS** (`HandBar` carries `Transform::default()` -> Required Components inserts `GlobalTransform`; asserted in-test by `hand_fan_root_parent_carries_global_transform`). **AC3 PASS** (PROMPT 1115 worker ran 24 adjacent hand UI bins green; PROMPT 1118 integration re-ran 10 of those + the new bin against the merged tree, all PASS). **AC4 PASS** (`tests/integration/hand-ui/hand_fan_root_b0004_hierarchy_test.rs` NEW; 1/1 pass; asserts `HandFanRoot` carries `GlobalTransform`, its `ChildOf` parent matches the `HandBar` marker locking the audit's exact edge, and the parent carries both `Transform` and `GlobalTransform`). **AC5 PASS** (Sprint 12 story 019 `closed-with-conditions / cannot-reproduce` disposition per `TQ-S12-C2` preserved; the audit's `no functional bug evident yet` note honoured -- this row is ECS hygiene, not a drag-runtime repair). **AC6 PASS** (zero changes under `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`; no protocol-shape change). **AC7 PASS** (ADR-021 schedule preserved; no new `SystemSet`, no schedule wiring change; cargo check at integration tip PASS). **AC8 PASS** (worker commit + evidence.md + integration merge commit + this PROMPT 1120 paperwork all explicitly preserve `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 761 `Polish->Release` FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all AUDIT-1076-* outside AUDIT-1076-14, all SOURCE-1077-*, all 24 PROMPT 1022 findings, and the PROMPT 1112 AC3 hand reserve-strip carry preserved OPEN). **AC9 PASS** (PROMPT 1115 worker + PROMPT 1118 integration diffs touched zero files under `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md`; PROMPT 1120 is the first authorised modifier of `production/sprint-status.yaml` + `production/session-state/*` for this row). **AC10 PASS** (PROMPT 1115 worker pushed `work/s17-hand-fan-root-b0004-cleanup` (`535450d`) only -- never main; files changed at worker time: `client/src/ui/hand/mod.rs` one-line `Transform::default()` insert + explanatory comment block, `client/Cargo.toml` single additive `[[test]]` block, new test bin, evidence document; integration into `origin/main` performed separately by PROMPT 1118 via `integrate/s17-hand-b0004-cleanup-1118` -> `29ad4c6`). **AC11 PASS** (PROMPT 1115 worker + PROMPT 1118 integration both applied the 5 Cargo resource policy env vars `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` + `CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'` before every cargo invocation via PowerShell launcher scripts; both build lines printed `Finished dev profile [optimized]` / `Finished test profile [optimized]` (no `+ debuginfo`), confirming the policy was applied; D: free 774 GB at PROMPT 1118 integration start (well above the 50 GB preflight threshold); PROMPT 1120 itself does NOT invoke Cargo). **Sprint 17 progress after PROMPT 1120**: 4 of 9 active rows DONE (S17-SERVER-START-OF-TURN-DEBUG-001 by PROMPT 1108 + S17-UI-CARD-SLOT-INSET-WIRING-001 by PROMPT 1110 + S17-UI-CARD-DISPLAY-ART-HELPER-001 by PROMPT 1117 + S17-UI-HAND-B0004-CLEANUP-001 by PROMPT 1120) + 1 of 9 active rows PARTIAL / IN_PROGRESS (S17-UI-HUD-OPP-MANA-CLEANUP-001 carried via PROMPT 1112 with AC3 carried) + 4 rows preserved as `ready`. Must Have 1/2 done + Should Have 1 done + 1 partial / 4 + Nice to Have 2/3 done. Rows preserved as `ready` and NOT closed or partial by PROMPT 1120: `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; no LLM `/story-done` authorised per 2026-05-17 orchestrator decision); `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`, `S17-UI-BID-BUTTON-PHASE-RACE-001` (2 of remaining 3 Should Have); `S17-OPS-VULKAN-VALIDATION-GATING-001` (1 of remaining 1 Nice to Have). **AUDIT-1076-14 (P3) discharged on origin/main by PROMPT 1118 integration**. Other AUDIT-1076-* findings preserved as open / report-only outside AUDIT-1076-15 (discharged PROMPT 1107/1108) and AUDIT-1076-10 + AUDIT-1076-16 (discharged PROMPT 1111; AUDIT-1076-17 remains OPEN carried with AC3 of `S17-UI-HUD-OPP-MANA-CLEANUP-001`). **PROMPT 1112 AC3 hand reserve-strip carry (AUDIT-1076-17 floating `Reserve N + / Current N` microbadge in `client/src/ui/hand/mod.rs` reserve strip) preserved OPEN; PROMPT 1120 does NOT close it (semantically distinct surface: hand reserve-strip per-card microbadge vs HandBar/FanRoot ECS hierarchy invariant; the story file, evidence document, and integration report all carry this distinction explicitly).** **Files changed by PROMPT 1120**: `production/epics/hand-ui/story-021-hand-fan-root-b0004-hierarchy.md` (Status banner Draft -> Done; AC1..AC11 flipped to `[x]`; Completion Notes section added with PROMPT 1115 worker + PROMPT 1118 integration outcome + Test evidence + Cargo resource policy (AC11) + Per-AC outcome + Closure trail (commits) numbered list; Closure Trail section replaced; final status line DRAFT -> DONE); `production/sprint-status.yaml` (S17-UI-HAND-B0004-CLEANUP-001 row flipped `status: ready -> done` with `completed: 2026-05-18` + worker/integration/integrated_commit/story_done_prompt/evidence metadata + closure notes; `sprint_17_story_done:` PROMPT 1120 block appended after PROMPT 1117 entry and before `sprint_17_partial_disposition:` block with stories_closed entry covering AC1..AC11 outcomes, rows_not_closed_by_prompt_1120 enumerating the other 8 Sprint 17 active rows, conditions_carried_forward_unchanged + explicitly_not_claimed + files_changed_by_prompt_1120 + forbidden_changes_observed sections); `production/session-state/active.md` (PROMPT 1120 banner prepended above PROMPT 1117 banner); this file (PROMPT 1120 paragraph prepended above PROMPT 1117 paragraph); `reports/PROMPT-1120-s17-hand-b0004-cleanup-story-done.md` (mandatory final report; `reports/` gitignored; not staged or committed). **Files explicitly NOT touched by PROMPT 1120**: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-hand-fan-root-b0004/ preserved verbatim on `origin/main` via PROMPT 1118 integration), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 8 other Sprint 17 active row story files under `production/epics/` (S11-HUD-TIMER-EYEBALL-VISUAL-001 story 014 untouched; S17-UI-HUD-OPP-MANA-CLEANUP-001 story 018 PROMPT 1112 PARTIAL disposition preserved verbatim; S17-UI-CARD-DISPLAY-ART-HELPER-001 story 017 PROMPT 1117 closure preserved verbatim), all prior `sprint_N_*` blocks in `production/sprint-status.yaml` (including PROMPT 1108 + PROMPT 1110 + PROMPT 1117 `sprint_17_story_done` entries and the PROMPT 1112 `sprint_17_partial_disposition` entry which are preserved verbatim above and below this new PROMPT 1120 entry), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1120 paperwork-only closure. **Non-claims preserved verbatim by PROMPT 1120**: no Sprint 17 close-out; no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry); no closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (PROMPT 1112 PARTIAL disposition with AC3 carried preserved verbatim; row remains OPEN); no discharge of PROMPT 1112 AC3 hand reserve-strip carry / AUDIT-1076-17 (semantically distinct surface); no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion; no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* finding outside AUDIT-1076-14 by PROMPT 1120; no closure of any SOURCE-1077-* finding by PROMPT 1120 (SOURCE-1077-01/02/03/04 discharged by PROMPT 1114 integration via PROMPT 1117); no closure of any of the 24 PROMPT 1022 audit findings; no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen. **Next launchable prompts**: per-row `/story-readiness` reruns against PROMPT 1120 closure HEAD for the 3 remaining `ready` Sprint 17 active rows (S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001, S17-UI-BID-BUTTON-PHASE-RACE-001, S17-OPS-VULKAN-VALIDATION-GATING-001) + AC3 carry-forward producer decision on `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Sprint 17 pull vs Sprint 18 deferral) before final `/story-done` closure of that row; per-row `/dev-story` + integration + `/story-done` paperwork for the remaining `ready` rows; Sprint 17 smoke harness prompt with two-client session. **Branch / push**: PROMPT 1120 commits `story-done(s17): close S17-UI-HAND-B0004-CLEANUP-001 (PROMPT 1120)` on branch `story-done/s17-hand-b0004-cleanup-1120` from base `origin/main@29ad4c6`; pushes to `origin/main` if policy allows; if direct `main` push is blocked, the commit is pushed on the worker branch and the exact commit/branch reported, never force-pushed.)

Updated: 2026-05-18 (PROMPT 1117 -- Sprint 17 `S17-UI-CARD-DISPLAY-ART-HELPER-001` `/story-done` closure; paperwork-only single-shared-status writer. Source-of-truth at closure: `origin/main@30c9e0f6d7b867d25d3f8ba5d273c2f1890b02a7` = PROMPT 1114 integration tip `integrate(s17): merge PROMPT 1113 card-display art-helper into main (PROMPT 1114)` merging PROMPT 1113 worker `4f577d68610e5231a94385634d828edd913a1f4e` `dev-story(s17-card-display-art-helper): lift helper to single owner + remove leak + chrome preservation + existence-check probe (PROMPT 1113)` onto `origin/main` via no-ff merge; strict fast-forward descendant of `origin/main@f2ba917` PROMPT 1112 paperwork-only PARTIAL DISPOSITION tip, of `origin/main@4bd4f56` PROMPT 1111 PARTIAL integration tip, of `origin/main@9a9b1dc` PROMPT 1110 card-slot inset wiring story-done tip, of `origin/main@30f166f` PROMPT 1106 card-slot inset wiring integration tip, of `origin/main@72d56bc` PROMPT 1108 server start-of-turn-debug story-done tip, and of `origin/main@dc8adb6` PROMPT 1107 server warn->debug integration tip. Branch: `story-done/s17-card-display-art-helper-1117` from base `origin/main@30c9e0f`; in-place edits on the primary checkout. Single-context paperwork-only `/story-done` run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. **Single-row closure**: `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must Have, PROMPT 1077 P0/P1 bundle absorbing SOURCE-1077-01 + SOURCE-1077-02 + SOURCE-1077-03 + SOURCE-1077-04) closed on basis of PROMPT 1113 worker (lifted `apply_card_display_art` + `clear_card_display_art` to `client/src/asset_wiring.rs` as `pub fn`; deleted verbatim copies in `client/src/ui/shop_auction/mod.rs` + `client/src/ui/hand/mod.rs`; both modules `use crate::asset_wiring::{apply_card_display_art, clear_card_display_art, ...}`; chrome preservation via do-not-touch-`ImageNode`-on-Err/Clear strategy; `resolve_card_display_art` returns `Result<String, ...>` -- `Box::leak` removed; `CardDisplayArtAsset.path: String` (was `&'static str`); new `probe_card_display_art_paths` system on `OnEnter(ClientState::InSession)` warns with `art_id` + `path` per missing file + `MissingCardArtWarnings` resource counts warnings for test observability + documented `"missing"` sentinel routes through placeholder without warn; 6/6 unit tests in `tests/unit/asset_wiring/card_display_art_helper_test.rs` + 8/8 integration tests in `tests/integration/presentation/card_display_art_chrome_preservation_test.rs` + 4 adjusted existing test files for `&'static str` -> `String` signature change; worker pushed `work/s17-card-display-art-helper` only -- never main) + PROMPT 1114 integration (no-ff merge onto `origin/main`; `git diff --name-only origin/main..HEAD` returns exactly 11 paths: `client/Cargo.toml` (test bin registration only; no feature flag, no dependency, no Cargo feature-surface change) + `client/src/asset_wiring.rs` + `client/src/ui/hand/mod.rs` + `client/src/ui/shop_auction/mod.rs` + `production/qa/evidence/sprint-17-card-display-art-helper/evidence.md` NEW + `tests/integration/hand-ui/draft_initial_grid_test.rs` + `tests/integration/playable_client/draft_shop_hand_bridge_test.rs` + `tests/integration/presentation/card_display_art_chrome_preservation_test.rs` NEW + `tests/integration/shop_auction_ui/auction_activation_test.rs` + `tests/integration/shop_auction_ui/shop_panel_test.rs` + `tests/unit/asset_wiring/card_display_art_helper_test.rs` NEW; zero changes under `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`; `cargo check -p client` EXIT 0 clean dev build ~58s; `cargo check -p client --tests` EXIT 0 clean test build ~9s; 11 targeted test bins PASS 102/102 sub-tests total at integration tip; `git diff --check origin/main...HEAD` clean; integration tip pushed `f2ba917..30c9e0f` on `main`). **AC1 PASS** (single owner site: `grep -rn 'fn apply_card_display_art' client/src/ shared/src/` -> 1 match at `client/src/asset_wiring.rs:594`; `grep -rn 'fn clear_card_display_art' client/src/ shared/src/` -> 1 match at `client/src/asset_wiring.rs:627`; SOURCE-1077-02 discharged). **AC2 PASS** (slot-well chrome survives missing card art via do-not-touch-`ImageNode`-on-Err/Clear strategy; test `shop_slot_chrome_survives_missing_card_art_apply` in `card_display_art_chrome_preservation_test.rs`; SOURCE-1077-01 discharged). **AC3 PASS** (no `Box::leak`: `grep -rn 'Box::leak' client/src/ shared/src/` -> zero functional matches under `client/src/` outside the historical doc-comment at `asset_wiring.rs:553` describing the prior bug; resolver returns `Result<String, ...>`; SOURCE-1077-03 discharged). **AC4 PASS** (`probe_card_display_art_paths` registered on `OnEnter(ClientState::InSession)`; emits `warn!` with `art_id` + constructed path per missing file; missing art does not panic; falls through to documented placeholder; chrome from AC2 remains intact; `MissingCardArtWarnings` resource counts warnings; SOURCE-1077-04 discharged). **AC5 PASS** (`shop_slot_happy_path_apply_sets_card_art_binding` test; hand fan slot subtree keeps chrome `ImageNode` independently via existing `hand_ui_asset_wiring_test.rs` sweep). **AC6 PASS** (`shop_slot_chrome_survives_clear` test). **AC7 PASS** (`resolve_missing_sentinel_routes_to_placeholder` unit + `missing_sentinel_resolves_to_placeholder_via_apply` integration; no `warn!` for documented `"missing"` sentinel). **AC8 PASS** (6/6 unit tests). **AC9 PASS** (8/8 integration tests). **AC10 PASS** (fixture art-id coverage via `probe_does_not_warn_for_documented_missing_sentinel` + `probe_records_warning_count_resource_on_session_entry`). **AC11 PASS** (ADR-021 schedule preserved; no new `SystemSet`; helper consumers' `PresentationSet` placement unchanged; probe slots into existing `OnEnter(InSession)`; cargo check at integration tip PASS). **AC12 PASS** (zero changes under `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`; no protocol-shape change). **AC13 PASS** (worker commit + integration merge commit + this PROMPT 1117 paperwork all explicitly preserve `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a` specifically `PAW-TD-004-a` card art placeholder, `TQ-S12-C1..C7`, PROMPT 761 `Polish->Release` FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all AUDIT-1076-*, all SOURCE-1077-* outside the four bundled here, 24 PROMPT 1022 findings; final-art replacement of the 16 production card art files explicitly out of scope; Standard-tier accessibility not pursued; playtest validation not pursued). **AC14 PASS** (PROMPT 1113 worker + PROMPT 1114 integration diffs touched zero files under `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md`; PROMPT 1117 is the first authorised modifier of `production/sprint-status.yaml` + `production/session-state/*` for this row). **AC15 PASS** (PROMPT 1113 worker pushed `work/s17-card-display-art-helper` only -- never main; integration into `origin/main` performed separately by PROMPT 1114 via `integrate/s17-card-display-art-helper-1114` -> `30c9e0f`). **AC16 PASS-WORKER + ADVISORY-INTEGRATION**: PROMPT 1113 worker applied all 5 Cargo resource policy env vars (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` + `CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before every cargo invocation per evidence.md AC16 row + worker report; D: free 800.5 GB at preflight (well above 50 GB minimum). PROMPT 1114 integration encountered a PowerShell/Bash env-var propagation gap: the `$env:CARGO_TARGET_DIR=...` block issued through the Bash tool did not propagate on the first call; the first cargo test invocation built into the integration worktree-local `target/` rather than `D:\_DEV\cargo-target\ccgs-msvc`. Resource impact: a few GB local to the worktree; D: drive remained ~744 GB free; no cleanup required; no stale child dirs touched under `D:\_DEV\cargo-target\ccgs-msvc`. The build correctness gate the integration prompt required is **unaffected** -- all 11 targeted test bins PASS 102/102 sub-tests total + 2 cargo check invocations PASS against the merged tree. Recorded explicitly as a process / policy advisory note in `production/epics/ui-clean-pass/story-017-card-display-art-helper-bundle.md` Completion Notes section, in the `production/sprint-status.yaml` `sprint_17_story_done:` PROMPT 1117 `batch_note` + `cargo_resource_policy_advisory:` block + row notes, in the `production/session-state/active.md` PROMPT 1117 banner, in this paragraph, and in the PROMPT 1117 final report; **NOT hidden as a product failure**. PROMPT 1117 itself does NOT invoke Cargo (paperwork-only closure). **Sprint 17 progress after PROMPT 1117**: 3 of 9 active rows DONE (S17-SERVER-START-OF-TURN-DEBUG-001 by PROMPT 1108 + S17-UI-CARD-SLOT-INSET-WIRING-001 by PROMPT 1110 + S17-UI-CARD-DISPLAY-ART-HELPER-001 by PROMPT 1117) + 1 of 9 active rows PARTIAL / IN_PROGRESS (S17-UI-HUD-OPP-MANA-CLEANUP-001 carried via PROMPT 1112 with AC3 carried) + 5 rows preserved as `ready`. Must Have 1/2 done + Should Have 1 done + 1 partial / 4 + Nice to Have 1/3 done. Rows preserved as `ready` and NOT closed or partial by PROMPT 1117: `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; no LLM `/story-done` authorised per 2026-05-17 orchestrator decision); `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`, `S17-UI-BID-BUTTON-PHASE-RACE-001` (2 of remaining 3 Should Have); `S17-OPS-VULKAN-VALIDATION-GATING-001`, `S17-UI-HAND-B0004-CLEANUP-001` (2 of remaining 2 Nice to Have). **SOURCE-1077-01 + SOURCE-1077-02 + SOURCE-1077-03 + SOURCE-1077-04 all discharged on origin/main by PROMPT 1114 integration**. Other PROMPT 1077 findings preserved as open / report-only outside the Sprint 17 active row absorption set (SOURCE-1077-06 discharged by PROMPT 1106/1110; SOURCE-1077-08/09/16 bundled into `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`; SOURCE-1077-10 owned by `S17-UI-BID-BUTTON-PHASE-RACE-001`; SOURCE-1077-05/07/11/12/13/14/15 deferred to Sprint 18+). All AUDIT-1076-* findings preserved outside AUDIT-1076-15 (discharged PROMPT 1107/1108) and AUDIT-1076-10 + AUDIT-1076-16 (discharged PROMPT 1111; AUDIT-1076-17 remains OPEN carried with AC3 of `S17-UI-HUD-OPP-MANA-CLEANUP-001`). **Files changed by PROMPT 1117**: `production/epics/ui-clean-pass/story-017-card-display-art-helper-bundle.md` (Status banner Draft -> Done; AC1..AC16 flipped to `[x]`; Completion Notes section added with PROMPT 1113 worker + PROMPT 1114 integration + Cargo resource policy advisory sub-section + per-AC outcome list + closure trail (commits) numbered list; final status line DRAFT -> DONE); `production/sprint-status.yaml` (S17-UI-CARD-DISPLAY-ART-HELPER-001 row flipped `status: ready -> done` with `completed: 2026-05-18` + worker/integration/integrated_commit/story_done_prompt/evidence metadata + closure notes + Cargo resource policy advisory note; `sprint_17_story_done:` PROMPT 1117 block appended after PROMPT 1110 entry and before `sprint_17_partial_disposition:` block with stories_closed entry covering AC1..AC16 outcomes, rows_not_closed_by_prompt_1117 enumerating the other 8 Sprint 17 active rows, cargo_resource_policy_advisory block, conditions_carried_forward_unchanged + explicitly_not_claimed + files_changed_by_prompt_1117 + forbidden_changes_observed sections); `production/session-state/active.md` (PROMPT 1117 banner prepended above PROMPT 1112 banner); this file (PROMPT 1117 paragraph prepended above PROMPT 1112 paragraph); `reports/PROMPT-1117-s17-card-display-art-helper-story-done.md` (mandatory final report; `reports/` gitignored; not staged or committed). **Files explicitly NOT touched by PROMPT 1117**: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**` (sprint-17 plan body NOT rewritten), `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-card-display-art-helper/ preserved verbatim on `origin/main` via PROMPT 1114 integration), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 8 other Sprint 17 active row story files under `production/epics/` (`S11-HUD-TIMER-EYEBALL-VISUAL-001` story 014 untouched; `S17-UI-HUD-OPP-MANA-CLEANUP-001` story 018 PROMPT 1112 PARTIAL disposition preserved verbatim), all prior `sprint_N_*` blocks in `production/sprint-status.yaml` (including the PROMPT 1108 + PROMPT 1110 `sprint_17_story_done` entries and the PROMPT 1112 `sprint_17_partial_disposition` entry which are preserved verbatim above and below this new PROMPT 1117 entry), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1117 paperwork-only closure. **Non-claims preserved verbatim by PROMPT 1117**: no Sprint 17 close-out; no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry); no closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (PROMPT 1112 PARTIAL disposition with AC3 carried preserved verbatim; row remains OPEN); no per-surface card-slot primitive migration of any consumer surface (HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST remain Sprint 17+ Backlog under `S17-UI-CARD-SLOT-MIGRATION-*`); no real-art production for any of the 16 production card art files (`PAW-TD-*-a` placeholder-art accept-risk preserved); no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion; no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* finding by PROMPT 1117; no closure of any SOURCE-1077-* finding outside the four bundled here (SOURCE-1077-01/02/03/04 discharged by PROMPT 1114); no closure of any of the 24 PROMPT 1022 audit findings; no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen. **Next launchable prompts**: per-row `/story-readiness` reruns against PROMPT 1117 closure HEAD for the 5 remaining `ready` Sprint 17 active rows + AC3 carry-forward producer decision on `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Sprint 17 pull vs Sprint 18 deferral) before final `/story-done` closure of that row; per-row `/dev-story` + integration + `/story-done` paperwork for the remaining `ready` rows; Sprint 17 smoke harness prompt with two-client session. **Branch / push**: PROMPT 1117 commits `story-done(s17): close S17-UI-CARD-DISPLAY-ART-HELPER-001 (PROMPT 1117)` on branch `story-done/s17-card-display-art-helper-1117` from base `origin/main@30c9e0f`; pushes to `origin/main` if policy allows; if direct `main` push is blocked, the commit is pushed on the worker branch and the exact commit/branch reported, never force-pushed.)

---

Updated: 2026-05-18 (PROMPT 1112 -- Sprint 17 `S17-UI-HUD-OPP-MANA-CLEANUP-001` PARTIAL DISPOSITION paperwork; paperwork-only single-shared-status writer; **NOT** a `/story-done` closure. Source-of-truth at disposition: `origin/main@4bd4f569bf0f8e54a18b6f1a9c95336aefff34d9` = PROMPT 1111 integration tip `integrate(s17): merge PROMPT 1105 HUD class-reveal projection (PARTIAL, AC3 carried) into main (PROMPT 1111)` merging PROMPT 1105 worker `c6b7d70a2733c1fa3b0af271c8e309397cf592a6` `dev-story(s17-hud-opp-mana-cleanup): HUD class-reveal projection for opp figurine + OPP label (PROMPT 1105)` onto `origin/main` via no-ff merge; strict fast-forward descendant of `origin/main@30f166f` PROMPT 1106 card-slot inset wiring integration tip, of `origin/main@9a9b1dc` PROMPT 1110 card-slot inset wiring story-done tip, of `origin/main@72d56bc` PROMPT 1108 server start-of-turn-debug story-done tip, and of `origin/main@dc8adb6` PROMPT 1107 server warn->debug integration tip. Branch: `paperwork/s17-hud-opp-mana-partial-disposition-1112` from base `origin/main@4bd4f56`; in-place edits on the primary checkout. Single-context paperwork-only PARTIAL DISPOSITION run; no spawned CCGS subagents; no `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. **Single-row PARTIAL disposition (not closure)**: `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Should Have, AUDIT-1076-10 + AUDIT-1076-16 + AUDIT-1076-17 bundle) recorded as Partial / In Progress on the basis of PROMPT 1105 worker (HUD class-reveal projection: new resource `HudClassReveal { local: Option<ClassId>, opponent: Option<ClassId> }` + three new systems in `client/src/ui/hud/mod.rs` -- `sync_class_reveal_from_lobby_view_system` MessageDrain reading `Res<LobbyViewState>.revealed_classes` + `Res<ClientSessionIdentity>` and skipping while `HudMode::Frozen` so incremental lobby reveals cannot overwrite during GAME_OVER per TR-HUD-009; `sync_class_reveal_from_snapshot_system` MessageDrain after `handle_game_snapshot_system` reading `MessageReader<PresentationGameSnapshotMessage>` and always running so `S2CGameSnapshot` reconnect rebuilds remain authoritative even at GAME_OVER per ADR-011; `sync_class_reveal_hud_system` StateSync after `sync_gold_text_system` and `sync_figurine_image_system` applying `HudClassReveal` to opp figurine `ImageNode` via `hud_figurine_asset(opp_class)` and to `opponent_gold_prefix` `Text` via `format_opp_class_display(opp_class)` returning `"OPP {ClassId:?}"`; local cache short-circuits redundant writes; 8 new integration tests in `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs`; worker pushed `work/s17-hud-opp-mana-cleanup` only -- never main; worker chose prefix entity (not value entity) for OPP label re-skin to preserve opponent-gold readout contract guarded by `reconnect_snapshot_rebuild_test.rs`) + PROMPT 1111 integration (no-ff merge onto `origin/main`; `git diff --name-only origin/main...HEAD` returns exactly 4 paths: `client/Cargo.toml` + `client/src/ui/hud/mod.rs` + `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs` NEW + `production/qa/evidence/sprint-17-hud-opp-mana-cleanup/evidence.md` NEW; `cargo check -p client` OK 8.78s 0 errors 0 warnings; `cargo test -p client --test hud_opp_figurine_label_mana_repaint_test` PASS 8/8; 6 sibling HUD test bins PASS 27/27 sub-tests; `git diff --check` + `git diff --cached --check` clean; integration tip `4bd4f56` pushed to origin/main fast-forward `9a9b1dc..4bd4f56`; **PROMPT 1111 explicitly did NOT touch `client/src/ui/hand/mod.rs` and recorded AC3 carried in the no-ff merge commit message**). **AC1 PASS** (opp figurine `ImageNode` re-skins on `S2CClassesRevealed` via `hud_figurine_asset(opp_class)`; test `ac1_opponent_figurine_reskins_on_classes_revealed` PASS; AUDIT-1076-10 functionally discharged on origin/main). **AC2 PASS** (OPP `opponent_gold_prefix` `Text` re-skins via `format_opp_class_display(opp_class)`; test `ac2_opp_text_label_reskins_on_classes_revealed` PASS; AUDIT-1076-16 functionally discharged on origin/main). **AC3 NOT DELIVERED -- EXPLICITLY CARRIED** (AUDIT-1076-17 mana microbadge removal; the floating `Reserve N + / Current N` microbadge spawn site lives in `client/src/ui/hand/mod.rs` reserve strip -- `spawn_reserve_strip` ~L3505, per-card `Reserve N Current N` text ~L3530, updater ~L4108-L4110 -- semantically distinct from HUD canonical `MANA n / N` strip which is preserved unchanged; the reserve strip is a hand-ui per-card overlay, not a HUD-owned widget; `client/src/ui/hand/` is on the story's "Forbidden files" list for the HUD-owned worker; PROMPT 1105 correctly invoked the story's worker-contract pause-and-escalate branch; PROMPT 1111 integration explicitly did NOT touch `client/src/ui/hand/mod.rs` and recorded AC3 carried; AUDIT-1076-17 remains OPEN on origin/main). **AC4 PASS** (`sync_class_reveal_hud_system` scheduled in `PresentationSet::StateSync`; no `Animator` / tween). **AC5 PASS** (`sync_class_reveal_from_snapshot_system` MessageDrain after `handle_game_snapshot_system`; always runs so `S2CGameSnapshot` reconnect rebuilds remain authoritative at GAME_OVER per ADR-011). **AC6 PASS** (`sync_class_reveal_from_lobby_view_system` skips while `HudMode::Frozen`; snapshot path always runs; FROZEN-on-GAME_OVER preserved per TR-HUD-009 + Sprint 14 story 017 AC6 binding). **AC7 PASS** (re-skin reads `Res<LobbyViewState>.revealed_classes` + `Res<ClientSessionIdentity>` + `S2CGameSnapshot` drain only; no spawned-unit / lane-state / observation-derived inference; ADR-002 + ADR-012 + Sprint 14 story 017 AC8 binding preserved). **AC8 PASS** (`HudClassReveal` carries class identity only; no objective identity / `was_fake` data flows to OPP label or opp figurine; ADR-001 invariant preserved; defence-in-depth grep in PROMPT 1105 evidence). **AC9 PASS** (`tests/integration/hud/opp_figurine_label_mana_repaint_test.rs` NEW, 8 tests covering AC1 / AC2 / AC4 / AC5 / AC6 / AC7 / AC8 + opp figurine marker singleton guard; AC3 microbadge-removal coverage explicitly NOT included, carries forward with AC3). **AC10 PASS** (integration diff 4 paths total, zero under `server/` / `shared/` / `tests/integration/server/`; no protocol-shape change). **AC11 PASS** (new systems slot into existing `PresentationSet::MessageDrain` + `PresentationSet::StateSync` per ADR-021; no new schedule wiring; `cargo check -p client` PASS at integration tip). **AC12 PASS** (worker + integration + this PROMPT 1112 partial-disposition paperwork all explicitly preserve `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a` specifically `PAW-TD-004-a` opp figurine placeholder, `TQ-S12-C1..C7`, PROMPT 761 `Polish->Release` FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all AUDIT-1076-* outside the three bundled with AC3 carried, all SOURCE-1077-*, all 24 PROMPT 1022 findings; final-art replacement of opp figurine remains out of scope; Standard-tier hit-target conformance on OPP label NOT pursued; playtest validation NOT pursued). **AC13 PASS** (worker + integration diffs touched zero files under `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md`; PROMPT 1112 is the first authorised modifier of `production/sprint-status.yaml` + `production/session-state/*` for this row, applying paperwork-only partial-disposition edits). **AC14 PASS** (worker branch `work/s17-hud-opp-mana-cleanup` scoped to 4 files; never `main`; `client/src/state/mod.rs` `apply_classes_revealed` reducer NOT modified -- worker chose to read via `Res<LobbyViewState>` instead of extending the reducer). **AC15 PASS** (worker + integration both applied 5 Cargo resource policy env vars `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` + `CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'` before every cargo invocation; integration cargo gates OK; D: free ~745 GB at start; PROMPT 1112 itself does NOT invoke cargo, paperwork-only). **AC16 DEFERRED** (HUD epic story count refresh deferred to future `/story-done` paperwork; gated on AC3 closure via follow-up hand-ui row OR producer explicit accept-into-Sprint-18 of AC3 carry-forward; PROMPT 1112 does NOT modify `production/epics/hud/EPIC.md`). **Follow-up candidate slug for the carried AC3**: `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` (preferred per PROMPT 1111 recommendation; Sprint 17 is in Polish and AC3 is a hand-ui cleanup decision rather than a hot defect); alternative `S17-UI-HAND-RESERVE-STRIP-CLEANUP-001` only if a producer explicitly authorises pulling AC3 forward as a separate single-row Sprint 17 story before close-out and Sprint 17 has remaining capacity; **PROMPT 1112 records the slug as candidate only and does NOT author or activate the row**. **Row status**: `S17-UI-HUD-OPP-MANA-CLEANUP-001` flipped `status: ready -> in_progress` with `partial_disposition: "AC3 carried"` note and full worker / integration / `integrated_commit` / `partial_disposition_prompt` metadata in `production/sprint-status.yaml`; row remains OPEN / NOT closed; closure deferred per AC3 carry gating; **NOT marked completed/done**. **Sprint 17 progress after PROMPT 1112**: 2 of 9 active rows DONE (S17-SERVER-START-OF-TURN-DEBUG-001 by PROMPT 1108 + S17-UI-CARD-SLOT-INSET-WIRING-001 by PROMPT 1110) + 1 of 9 active rows PARTIAL / IN_PROGRESS (S17-UI-HUD-OPP-MANA-CLEANUP-001 here) + 6 rows preserved as `ready`; Must Have 0/2 done + Should Have 1 done + 1 partial / 4 + Nice to Have 1/3 done. Rows preserved as `ready` and NOT closed or marked partial by PROMPT 1112: `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; no LLM `/story-done` authorised per 2026-05-17 orchestrator decision); `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must Have); `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`, `S17-UI-BID-BUTTON-PHASE-RACE-001` (2 of remaining 3 Should Have); `S17-OPS-VULKAN-VALIDATION-GATING-001`, `S17-UI-HAND-B0004-CLEANUP-001` (2 of remaining 2 Nice to Have). **AUDIT-1076-10 + AUDIT-1076-16 functionally discharged on origin/main by PROMPT 1111 integration**; AUDIT-1076-17 remains OPEN / explicitly carried with AC3. All other AUDIT-1076-* preserved as open / report-only outside AUDIT-1076-15 (discharged PROMPT 1107 / 1108) and AUDIT-1076-10 / 16 (discharged PROMPT 1111). All SOURCE-1077-* preserved as open / report-only outside SOURCE-1077-06 (discharged PROMPT 1106 / 1110). **Files changed by PROMPT 1112**: `production/epics/hud/story-018-opp-figurine-mana-cleanup.md` (Status Draft -> Partial / In Progress; AC1 + AC2 + AC4..AC15 [x] with per-AC delivery evidence annotations; AC3 [~] with explicit `NOT DELIVERED -- EXPLICITLY CARRIED` rationale and follow-up candidate slugs; AC16 [ ] with deferred-to-/story-done rationale; new Partial Integration Notes section added before Closure Trail covering disposition + AC status table + AC3 carry classification + follow-up candidate + files changed + forbidden changes observed; Closure Trail section updated to reference PROMPT 1112; final status line DRAFT -> PARTIAL); `production/sprint-status.yaml` (S17-UI-HUD-OPP-MANA-CLEANUP-001 row in stories: block flipped `status: ready -> in_progress` with `partial_disposition` note + worker / integration / `integrated_commit` / `partial_disposition_prompt` metadata + 6 notes describing PARTIAL disposition + AC3 carry classification + follow-up candidate + integration gates + non-claims; `sprint_17_partial_disposition:` block appended at EOF following the `sprint_17_story_done:` precedent pattern with `stories_partial` entry covering AC1..AC16 outcomes, `ac3_carry_classification` block, `rows_not_closed_or_partial_by_prompt_1112` enumerating the other 8 Sprint 17 active rows, `follow_up_candidate_for_ac3` block, `conditions_carried_forward_unchanged` + `explicitly_not_claimed` + `files_changed_by_prompt_1112` + `forbidden_changes_observed` sections); `production/session-state/active.md` (PROMPT 1112 banner prepended above PROMPT 1110 banner); this file (PROMPT 1112 paragraph prepended above PROMPT 1110 paragraph); `reports/PROMPT-1112-s17-hud-opp-mana-partial-disposition.md` (mandatory final report; `reports/` gitignored; not staged or committed). **Files explicitly NOT touched by PROMPT 1112**: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**` (sprint-17 plan body NOT modified), `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-hud-opp-mana-cleanup/ preserved verbatim on origin/main via PROMPT 1111 integration), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 7 other Sprint 17 active row story files under `production/epics/` (`S11-HUD-TIMER-EYEBALL-VISUAL-001` story 014 untouched), all prior `sprint_N_*` blocks in `production/sprint-status.yaml` (including the PROMPT 1108 + PROMPT 1110 `sprint_17_story_done` entries which are preserved verbatim above the new PROMPT 1112 `sprint_17_partial_disposition` entry), no candidate story file authored for `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` or `S17-UI-HAND-RESERVE-STRIP-CLEANUP-001` (slugs recorded as candidates only), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1112 paperwork-only partial disposition. **Non-claims preserved verbatim by PROMPT 1112**: no Sprint 17 close-out; no `/story-done` closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (row remains OPEN / Partial; closure deferred per AC3 carry gating); no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry); no authoring or activation of the follow-up candidate slugs; no discharge of AUDIT-1076-17 (mana microbadge remains in `client/src/ui/hand/mod.rs`); no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion (`PAW-TD-*-a` preserved); no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* by PROMPT 1112 (AUDIT-1076-10 + AUDIT-1076-16 functionally discharged by PROMPT 1111 integration tip; AUDIT-1076-17 remains OPEN / carried); no closure of any SOURCE-1077-* by PROMPT 1112; no closure of any of the 24 PROMPT 1022 audit findings; no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen. **Next launchable prompts**: per-row `/story-readiness` reruns against PROMPT 1112 disposition HEAD `4bd4f56` for each of the remaining 6 ready Sprint 17 active rows; per-row `/dev-story` + integration + `/story-done` paperwork for each; producer decision on AC3 carry-forward (Sprint 17 pull vs Sprint 18 deferral) before final `/story-done` closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001`; if AC3 is pulled into Sprint 17, the producer authors `S17-UI-HAND-RESERVE-STRIP-CLEANUP-001` story file and the orchestrator sequences `/story-readiness` + `/dev-story` + integration + `/story-done` for it; alternatively the producer authors `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` for Sprint 18+ planning and `/story-done` closes the HUD row as Done-with-AC3-carried-to-S18 with the explicit producer decision recorded. **Branch / push**: PROMPT 1112 commits `paperwork(s17): record S17-UI-HUD-OPP-MANA-CLEANUP-001 PARTIAL disposition + AC3 carry (PROMPT 1112)` on branch `paperwork/s17-hud-opp-mana-partial-disposition-1112` from base `origin/main@4bd4f56`; pushes to `origin/main` if allowed; if direct `main` push is blocked, the commit is pushed on the worker branch and the exact commit/branch reported, never force-pushed.)

---

# PROMPT 1110 Paragraph -- Sprint 17 S17-UI-CARD-SLOT-INSET-WIRING-001 /story-done

Updated: 2026-05-18 (PROMPT 1110 -- Sprint 17 `S17-UI-CARD-SLOT-INSET-WIRING-001` `/story-done` closure; paperwork-only single-shared-status writer. Source-of-truth at closure: `origin/main@30f166fb9b718bdb5a6a904da0d66cdcc9685f15` = PROMPT 1106 integration tip `integrate(s17): merge PROMPT 1102 card-slot inset wiring into main (PROMPT 1106)` merging PROMPT 1102 worker `55c0dab11ab7572e0cb88827c3ed5f3b241c0fe8` `dev-story(s17): wire card-slot image/text inset + GlobalZIndex (PROMPT 1102 S17-UI-CARD-SLOT-INSET-WIRING-001)` onto `origin/main` via no-ff merge; strict fast-forward descendant of `origin/main@dc8adb6` PROMPT 1107 server warn->debug integration tip (landed in parallel and disjoint with PROMPT 1102 client/test/qa-evidence diff) and of `origin/main@72d56bc` PROMPT 1108 S17-SERVER-START-OF-TURN-DEBUG-001 story-done closure tip. Branch: `story-done/s17-card-slot-inset-wiring-1110` from base `origin/main@72d56bc`; in-place edits on the primary checkout. Single-context paperwork-only `/story-done` run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. Single-row closure: `S17-UI-CARD-SLOT-INSET-WIRING-001` (Should Have, SOURCE-1077-06) closed on basis of PROMPT 1102 worker (two net-additive sibling builders extending the Sprint 16 PROMPT 1074 card-slot primitive: `card_slot_image_inset_node(kind) -> (Node, GlobalZIndex)` at `client/src/ui/design_tokens/card_slot.rs:687` + `card_slot_text_inset_node(kind) -> (Node, GlobalZIndex)` at line 728; both emit `PositionType::Absolute` `Node`s whose `left/right/top/bottom` read verbatim from `card_slot_geometry(kind).{image_inset_px,text_inset_px}` and a `GlobalZIndex` from `card_slot_geometry(kind).z_layer` -- `UI_BASE (300)` for `HandFan`/`DraftGrid`/`ShopSlot`/`AuctionFeatured` and `UI_OVERLAY (400)` for `BoardStagedGhost`; `card_slot_node` body and `card_slot_geometry` constants UNCHANGED; no consumer-surface migration; 8 new `s17_*` integration tests in `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` + 3 new `s17_*` inline tests in `card_slot.rs` `#[cfg(test)] mod tests` block; cargo check 6.41s + cargo test 27/27 + cargo test 9/9 PASS under Sprint 15+ Cargo resource policy via `.ps1` wrappers; D: free ~761.8 GB) + PROMPT 1106 integration (no-ff merge onto `origin/main`; origin/main advanced ff47075 -> dc8adb6 mid-integration due to parallel PROMPT 1107, integration branch reset + re-merged with no conflict; `git diff --name-only origin/main..HEAD` returns exactly 3 paths -- the new builders + tests + new evidence.md; `cargo check -p client` PASS + `cargo test -p client --test ui_clean_pass_card_slot_primitive_test` PASS 27/27 + `cargo test -p client --lib card_slot` PASS 9/9; liv-bevy-018 review notes Bevy 0.18 idioms (Required Components API; no deprecated `*Bundle`; no `unwrap()`; no client-side RNG); integration tip fast-forward pushed `dc8adb6..30f166f` on `main`). AC1 PASS (image inset builder at line 687 verified by PROMPT 1110 file Read on origin/main@30f166f). AC2 PASS (text inset builder at line 728). AC3 PASS (GlobalZIndex wired from geometry; UI_BASE 300 + UI_OVERLAY 400 per integration test + inline test). AC4 PASS (fallback clause; catalog does not expose padding; doc-comment added + no Node.padding). AC5 PASS (PROMPT 1067/1074 shop-slot Phase 1 migration remains green; cargo test 27/27 + cargo test 9/9 at integration tip). AC6 PASS (8 new s17_* integration tests + 3 new s17_* inline tests cover (a) per-variant image-inset edges + (b) per-variant text-inset edges + (c) per-variant GlobalZIndex + (d) variant set coverage; plus 3 defensive guards against SOURCE-1077-06 regression). AC7 PASS (integration diff confirms ZERO changes under client/src/ui/hand/, client/src/ui/shop_auction/auction_*, client/src/ui/shop_auction/draft_*, client/src/presentation/board_rendering.rs). AC8 PASS (no card_slot_geometry constant change; 14 named per-kind constants UNCHANGED). AC9 PASS (no App::add_systems introduced; ADR-021 schedule preserved). AC10 PASS (zero changes under server/, shared/, tests/integration/server/; no protocol-shape change). AC11 PASS (PROMPT 1102 worker commit message + evidence.md + PROMPT 1106 integration report §Non-claims preserve S8-QA-001-W1, QA-COND-0005, QA-COND-0006, PAW-TD-*-a, TQ-S12-C1..C7, PROMPT 761 Polish->Release FAIL, S11-HUD-TIMER-EYEBALL-VISUAL-001 carry, all AUDIT-1076-*, all SOURCE-1077-* outside SOURCE-1077-06, 24 PROMPT 1022 findings). AC12 PASS (PROMPT 1102 worker + PROMPT 1106 integration touched only client/src/ui/design_tokens/card_slot.rs + tests/integration/ui_clean_pass/card_slot_primitive_test.rs + new evidence.md; production/sprint-status.yaml, production/sprints/sprint-17.md, production/stage.txt, production/session-state/*, the Sprint 17 QA plan, production/qa/smoke-*.md, production/qa/team-qa-*.md, production/gate-checks/*, docs/architecture/adr-*.md all preserved unchanged across worker + integration; PROMPT 1110 paperwork is the first authorised modifier of production/sprint-status.yaml + production/session-state/* for this row). AC13 PASS (PROMPT 1102 worker pushed work/s17-card-slot-inset-wiring; did NOT push main; integration into origin/main performed separately by PROMPT 1106 via integrate/s17-card-slot-inset-wiring-1106 -> 30f166f; optional docs/ux/global-ui-design-spec.md amendment not authored, scope kept tight). **AC14 PASS-WORKER + ADVISORY-INTEGRATION**: PROMPT 1102 worker applied all 5 Cargo resource policy env vars (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` + `CARGO_INCREMENTAL=0` + `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`); cargo check Finished `dev` profile in 6.41s with zero warnings; D: free ~761.8 GB recorded. PROMPT 1106 integration attempted the same policy but the first `cargo test` invocation built into the integration worktree-local `target/` because the bash-tool-issued `$env:CARGO_TARGET_DIR=...` PowerShell-syntax env-var block was not interpreted on the first call; subsequent calls correctly routed to `D:\_DEV\cargo-target\ccgs-msvc`. The build correctness gate the integration prompt required is **unaffected** -- all 27/27 integration tests + 9/9 inline tests PASS against the merged tree. Recorded explicitly as a process / policy advisory note in `production/epics/ui-clean-pass/story-018-card-slot-inset-wiring.md` Completion Notes section + AC14 verdict + the `production/sprint-status.yaml` `sprint_17_story_done:` `batch_note` + the row's `notes:` list + this paragraph; **NOT hidden as a product failure**. PROMPT 1110 itself does NOT invoke Cargo. **PROMPT 1106 evidence-file trailing-whitespace advisory** preserved explicitly: `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md` inherits two pre-existing trailing-whitespace lines (lines 92 and 107, both inside a Markdown bullet list) from the PROMPT 1102 source branch. PROMPT 1106 integration `git diff --check origin/main...HEAD` flagged both lines. The condition is inside a documentation artifact, not code; PROMPT 1106 did not rewrite the evidence file to preserve worker-authored provenance; PROMPT 1110 preserves this verbatim and does NOT rewrite or strip the trailing whitespace (evidence.md is already on `origin/main` via PROMPT 1106 integration; PROMPT 1110 does not modify files already on origin/main via integration). Surfaced explicitly in the story-018 Completion Notes section sub-heading 'PROMPT 1106 evidence-file trailing-whitespace advisory', in the `production/sprint-status.yaml` `sprint_17_story_done:` PROMPT 1110 `documentation_artifact_advisory:` block + row notes, in the `production/session-state/active.md` PROMPT 1110 banner, in this paragraph, and in the PROMPT 1110 final report -- **NOT hidden**. Sprint 17 progress after PROMPT 1110: 2 of 9 active rows done (Must Have 0/2 + Should Have 1/4 + Nice to Have 1/3). Rows preserved as `ready` and NOT closed by PROMPT 1110: `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; no LLM `/story-done` authorised per 2026-05-17 orchestrator decision); `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must Have); `S17-UI-HUD-OPP-MANA-CLEANUP-001`, `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`, `S17-UI-BID-BUTTON-PHASE-RACE-001` (3 of 4 Should Have); `S17-OPS-VULKAN-VALIDATION-GATING-001`, `S17-UI-HAND-B0004-CLEANUP-001` (2 of 3 Nice to Have). Per-surface migration of HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST remains Sprint 17+ Backlog under the `S17-UI-CARD-SLOT-MIGRATION-*` family; PROMPT 1110 ratifies the primitive only and does NOT migrate any consumer surface. Files changed by PROMPT 1110: `production/epics/ui-clean-pass/story-018-card-slot-inset-wiring.md` (Status Draft -> Done; AC1..AC13 [x]; AC14 [x] PASS-WORKER + ADVISORY-INTEGRATION; Completion Notes section added with PROMPT 1102 + PROMPT 1106 + PROMPT 1106 trailing-whitespace advisory sub-section + Test Evidence; Closure Trail populated; final status line DRAFT -> DONE); `production/sprint-status.yaml` (S17-UI-CARD-SLOT-INSET-WIRING-001 row flipped `status: ready -> done` with `completed: 2026-05-18` + worker/integration/evidence metadata + closure note + PROMPT 1106 trailing-whitespace advisory note; `sprint_17_story_done:` block appended after PROMPT 1108 entry with full PROMPT 1110 disposition including AC1..AC14 outcomes, `rows_not_closed_by_prompt_1110` enumerating 7 remaining Sprint 17 active rows, `documentation_artifact_advisory` block, conditions / explicitly-not-claimed / files-changed / forbidden-changes-observed sections); `production/session-state/active.md` (PROMPT 1110 banner prepended above PROMPT 1108 banner); this file (PROMPT 1110 paragraph prepended above PROMPT 1108 paragraph); `reports/PROMPT-1110-s17-card-slot-inset-wiring-story-done.md` (mandatory final report; `reports/` gitignored; not staged or committed). Files explicitly NOT touched by PROMPT 1110: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**` (sprint-17 plan body NOT rewritten), `production/qa/qa-plan-sprint-17.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-card-slot-inset-wiring/ preserved verbatim on `origin/main` via PROMPT 1106 integration including the two pre-existing trailing-whitespace lines), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 8 other Sprint 17 active row story files under `production/epics/` (S11-HUD-TIMER-EYEBALL-VISUAL-001 story 014 untouched), all prior `sprint_N_*` blocks in `production/sprint-status.yaml` (including the PROMPT 1108 sprint_17_story_done entry which is preserved verbatim above the new PROMPT 1110 entry), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1110 paperwork-only closure. Non-claims preserved verbatim by PROMPT 1110: no Sprint 17 close-out; no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`; no per-surface migration of any consumer surface (HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST remain Sprint 17+ Backlog under `S17-UI-CARD-SLOT-MIGRATION-*`); no rewrite or stripping of the two pre-existing trailing-whitespace lines in evidence.md; no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion (`PAW-TD-*-a` preserved); no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* finding (AUDIT-1076-15 was discharged by PROMPT 1107 / 1108; not by PROMPT 1110); no closure of any SOURCE-1077-* finding outside SOURCE-1077-06 (PROMPT 1110 discharges SOURCE-1077-06 only); no closure of any of the 24 PROMPT 1022 audit findings; no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen. Next launchable prompts: per-row `/story-readiness` reruns against PROMPT 1110 closure HEAD `30f166f` for each of the remaining 7 Sprint 17 active rows; per-row `/dev-story` + integration + `/story-done` paperwork for each; Sprint 17 smoke harness prompt with two-client session to bind smoke evidence for the remaining UI rows. Branch / push: PROMPT 1110 commits `story-done(s17): close S17-UI-CARD-SLOT-INSET-WIRING-001 (PROMPT 1110)` (or analogous message) on branch `story-done/s17-card-slot-inset-wiring-1110` from base `origin/main@72d56bc`; pushes to `origin/main` if allowed; if direct `main` push is blocked, the commit is pushed on the worker branch and the exact commit/branch reported, never force-pushed.)

---

Updated: 2026-05-18 (PROMPT 1108 -- Sprint 17 `S17-SERVER-START-OF-TURN-DEBUG-001` `/story-done` closure; paperwork-only single-shared-status writer. Source-of-truth at closure: `origin/main@dc8adb6a2c67a975fb241639f2b242000e7db926` = PROMPT 1107 integration tip `integrate(s17): merge PROMPT 1104 server start_of_turn_dispatch_system warn->debug into main (PROMPT 1107)` merging PROMPT 1104 worker `b26beab18f3a707f66713bae1198378d4d15d09f` `tech-debt(server): downgrade start_of_turn_dispatch_system warn -> debug (S17-SERVER-START-OF-TURN-DEBUG-001, PROMPT 1104)` onto `origin/main` via no-ff merge; strict fast-forward descendant of `origin/main@ff47075` PROMPT 1100 Sprint 17 QA plan tip, `origin/main@cb62a9e` PROMPT 1099 Sprint 17 activation tip, and `origin/main@bc3db29` PROMPT 1097 net-new Sprint 17 story authoring batch integration tip. Branch: `story-done/s17-server-start-of-turn-debug-1108` from base `origin/main@dc8adb6`; in-place edits on the primary checkout. Single-context paperwork-only `/story-done` run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness` invoked. Single-row closure: `S17-SERVER-START-OF-TURN-DEBUG-001` (Nice to Have) closed on basis of PROMPT 1104 worker (single-line `tracing::warn!` -> `tracing::debug!` substitution at `server/src/feature/keyword/observers.rs:66` inside `start_of_turn_dispatch_system`; message text + `target` field + system body unchanged; cargo gate pass under Sprint 15+ Cargo resource policy via `.ps1` wrappers; partial launch-log evidence captured in `production/qa/evidence/sprint-17-start-of-turn-debug/`) + PROMPT 1107 integration (no-ff merge onto `origin/main`; `git diff --name-only origin/main...HEAD` returns 2 paths only -- the 1-line `.rs` edit plus the new `evidence.md`; `cargo check -p server` passed exit 0 2m07s). AC1 PASS (macro is `tracing::debug!`, text/location/system body unchanged; PROMPT 1108 file Read on `origin/main@dc8adb6` re-verifies). AC4 PASS (PROMPT 1107 diff disjoint; system body unchanged; dispatch logic remains deferred). AC5 PASS (zero changes under `client/`, `shared/`, `tests/integration/`). AC6 PASS (zero changes under `server/src/core/rsm/`; ADR-009 + ADR-010 unchanged). AC7 PASS (commit + evidence carry explicit no-claim sections preserving `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 761 FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all AUDIT-1076-* outside AUDIT-1076-15, all SOURCE-1077-*, 24 PROMPT 1022 findings). AC8 PASS (PROMPT 1104 worker + PROMPT 1107 integration touched only `server/src/feature/keyword/observers.rs` + `evidence.md`; `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, `production/session-state/*`, the Sprint 17 QA plan, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`, `docs/architecture/adr-*.md` all preserved unchanged across worker + integration; PROMPT 1108 paperwork is the first authorised modifier of `production/sprint-status.yaml` + `production/session-state/*` for this row). AC9 PASS (PROMPT 1104 worker pushed `work/s17-server-start-of-turn-debug`; did NOT push `main`; integration into `origin/main` performed separately by PROMPT 1107 via `integrate/s17-server-start-of-turn-debug-1107` -> `dc8adb6`). AC2 PARTIAL (advisory non-BLOCKING for Config / Data row classification per `.claude/docs/coding-standards.md` "Test Evidence by Story Type" matrix; PROMPT 1104 default-launch.log captured zero audit-phrase WARN at 12-second boot bound; binding two-client smoke gate remains the Sprint 17 smoke prompt scope). AC3 PARTIAL (advisory; empirically untriggerable at boot per evidence.md caveats -- no `DraftStarted` at boot + pre-existing `server/src/main.rs:50-52` `tracing_subscriber` wiring does NOT auto-wire `EnvFilter::from_default_env()` so `RUST_LOG=debug` is currently ignored; both out of this row's scope; source diff alone proves macro is now `debug!`). **AC10 PASS-WORKER + ADVISORY-DEVIATION-INTEGRATION**: PROMPT 1104 worker applied all 5 Cargo resource policy env vars (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` + `CARGO_INCREMENTAL=0` + `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`) via `.ps1` wrappers; cargo check Finished 10.76s with RUSTFLAGS honoured no `+ debuginfo`; D: free 762.25 GB. PROMPT 1107 integration prompt's PowerShell-syntax env-var block (`$env:CARGO_TARGET_DIR=...`) was issued through the Bash tool which does not interpret `$env:` assignments; Bash emitted "command not found" for each line and the 5 policy env vars were **NOT** exported during the integration `cargo check -p server` invocation; build ran with default `dev` profile into the integration worktree's local `target/` rather than `D:\_DEV\cargo-target\ccgs-msvc`. The build correctness gate the integration prompt required is **unaffected** -- `cargo check -p server` passed exit 0 2m07s against the merged tree. Recorded explicitly as a process / policy advisory note in `production/epics/server/story-003-start-of-turn-debug-downgrade.md` Completion Notes section + AC10 verdict + the `production/sprint-status.yaml` `sprint_17_story_done:` `batch_note` + the row's `notes:` list + this paragraph; **NOT hidden as a product failure**. PROMPT 1108 itself does NOT invoke Cargo. Sprint 17 progress after PROMPT 1108: 1 of 9 active rows done (Must Have 0/2 + Should Have 0/4 + Nice to Have 1/3). Rows preserved as `ready` and NOT closed by PROMPT 1108: `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; no LLM `/story-done` authorised per 2026-05-17 orchestrator decision); `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must Have); `S17-UI-HUD-OPP-MANA-CLEANUP-001`, `S17-UI-CARD-SLOT-INSET-WIRING-001`, `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`, `S17-UI-BID-BUTTON-PHASE-RACE-001` (4 Should Have); `S17-OPS-VULKAN-VALIDATION-GATING-001`, `S17-UI-HAND-B0004-CLEANUP-001` (2 of 3 Nice to Have). `start_of_turn_dispatch_system` implementation remains **deferred** (this row only changed the log macro; the system body, registration, schedule, and per-`DraftStarted` trigger are unchanged). Pre-existing `tracing_subscriber` wiring gap (`server/src/main.rs:50-52` builder init without `.with_env_filter(EnvFilter::from_default_env())`) preserved as candidate Sprint 18+ ops-hardening row; NOT closed by PROMPT 1108. Files changed by PROMPT 1108: `production/epics/server/story-003-start-of-turn-debug-downgrade.md` (Status Draft -> Done; AC1 + AC4..AC9 [x]; AC2 + AC3 [~] PARTIAL advisory; AC10 [~] PASS-WORKER + ADVISORY-DEVIATION-INTEGRATION; Completion Notes section added with PROMPT 1104 + PROMPT 1107 + Cargo resource policy advisory deviation + Test Evidence; Closure Trail populated; final status line DRAFT -> DONE); `production/sprint-status.yaml` (S17-SERVER-START-OF-TURN-DEBUG-001 row flipped `status: ready -> done` with `completed: 2026-05-18` + worker/integration/evidence metadata + closure note + Cargo resource policy advisory rationale; `sprint_17_story_done:` block appended at EOF with full PROMPT 1108 disposition); `production/session-state/active.md` (PROMPT 1108 banner prepended above PROMPT 1100 banner); this file (PROMPT 1108 paragraph prepended above PROMPT 1100 paragraph); `reports/PROMPT-1108-s17-server-start-of-turn-debug-story-done.md` (mandatory final report; `reports/` gitignored; not staged or committed). Files explicitly NOT touched by PROMPT 1108: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt` (remains `Polish`), `production/sprints/**` (sprint-17 plan body NOT rewritten), `production/qa/qa-plan-sprint-17.md` (Row 8 launch-log evidence references preserved verbatim), `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/qa/evidence/*` (sprint-17-start-of-turn-debug/ preserved verbatim on `origin/main` via PROMPT 1107 integration), `production/gate-checks/**` (PROMPT 761 Polish->Release FAIL preserved with NO retry), `docs/architecture/adr-*.md`, all 8 other Sprint 17 active row story files under `production/epics/`, all prior `sprint_N_*` blocks in `production/sprint-status.yaml`, `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1108 paperwork-only closure. Non-claims preserved verbatim by PROMPT 1108: no Sprint 17 close-out; no closure of any other Sprint 17 active row; no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`; no implementation of `start_of_turn_dispatch_system` (dispatch remains deferred); no Sprint 17 smoke harness or two-client session run; no fix of pre-existing `tracing_subscriber` wiring gap; no public release readiness; no RC readiness; no full game completion; no broad / Standard-tier accessibility completion (`QA-COND-0005` preserved); no playtest validation (`QA-COND-0006` preserved); no full manual QA; no two-client `GAME_OVER` closure (`S8-QA-001-W1` OPEN); no final-art completion (`PAW-TD-*-a` preserved); no Polish->Release retry (PROMPT 761 FAIL preserved); no stage advance (`production/stage.txt` reads `Polish` unchanged); no `TQ-S12-C7` closure; no closure of any AUDIT-1076-* finding outside AUDIT-1076-15; no closure of any SOURCE-1077-* finding; no closure of any of the 24 PROMPT 1022 audit findings; no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen. Next launchable prompts: per-row `/story-readiness` reruns against PROMPT 1108 closure HEAD `dc8adb6` for each of the remaining 8 Sprint 17 active rows; per-row `/dev-story` + integration + `/story-done` paperwork for each; Sprint 17 smoke harness prompt with two-client session to bind AC2 evidence for this row + smoke evidence for the rest. Branch / push: PROMPT 1108 commits `story-done(s17): close S17-SERVER-START-OF-TURN-DEBUG-001 (PROMPT 1108)` (or analogous message) on branch `story-done/s17-server-start-of-turn-debug-1108` from base `origin/main@dc8adb6`; pushes to `origin/main` if allowed; if direct `main` push is blocked, the commit is pushed on the worker branch and the exact commit/branch reported, never force-pushed.)

---

Updated: 2026-05-18 (PROMPT 1100 -- Sprint 17 `/qa-plan sprint-17` authoring; paperwork-only single-shared-status writer. Source-of-truth at authoring: `origin/main@cb62a9e4c23d18e89d886b525c92e0274aa038f9` = PROMPT 1099 Sprint 17 activation commit `activate(s17): flip Sprint 17 from draft to active (PROMPT 1099)`; strict fast-forward descendant of `origin/main@bc3db29` PROMPT 1097 net-new Sprint 17 story authoring batch integration tip and of `origin/main@e6a6e11` PROMPT 1090 Sprint 17 plan draft commit base. PROMPT 1100 authors `production/qa/qa-plan-sprint-17.md` NEW covering all 9 active Sprint 17 rows (1 Must Have human-operator-blocked carry `S11-HUD-TIMER-EYEBALL-VISUAL-001` + 1 Must Have non-conditional `S17-UI-CARD-DISPLAY-ART-HELPER-001` + 4 Should Have `S17-UI-HUD-OPP-MANA-CLEANUP-001` / `S17-UI-CARD-SLOT-INSET-WIRING-001` / `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` / `S17-UI-BID-BUTTON-PHASE-RACE-001` + 3 Nice to Have `S17-OPS-VULKAN-VALIDATION-GATING-001` / `S17-SERVER-START-OF-TURN-DEBUG-001` / `S17-UI-HAND-B0004-CLEANUP-001`). Per-row test-type classification per `.claude/docs/coding-standards.md` "Test Evidence by Story Type" matrix: Manual / Visual (row 1; human-operator-blocked; no LLM `/story-done` authorised) + Logic + Integration (row 2 card-display-art helper bundle: helper dedup + leak fix + existence check = Logic + slot-well chrome preservation = Integration) + Integration (rows 3 hud opp/mana + 5 qa snapshot marker split + 6 bid-button phase race + 9 hand B0004) + Logic (row 4 card-slot inset wiring; design-tokens leaf) + Config / Data (rows 7 vulkan validation gating + 8 server start-of-turn debug downgrade; both log-only with default-log + debug-log launch evidence). All non-claims preserved verbatim: `S8-QA-001-W1` OPEN, `QA-COND-0005` / `QA-COND-0006` accepted-risk, `PAW-TD-*-a` preserved across PAW-002..PAW-006, `TQ-S12-C1..C7` preserved (`TQ-S12-C7` AppCompat informational condition explicitly NOT closed), PROMPT 683-era runtime divergence preserved, PROMPT 761 `Polish->Release` FAIL preserved with NO retry, PROMPT 1054 P1 UI snapshot retest `BLOCKED-HUMAN-OPERATOR` preserved, 24 PROMPT 1022 QA snapshot audit findings preserved report-only, long-tail AUDIT-1076-05 / 08 / 11 + SOURCE-1077-05 / 07 / 11 / 12 / 13 / 14 / 15 deferred to Sprint 18+, all five dropped conditional Must Have rows preserved on `origin/main` unchanged. Stage `Polish` UNCHANGED; `production/stage.txt` NOT modified. **No `/dev-story` authorised before this QA plan lands on `origin/main` AND each non-human row's `/story-readiness` rerun returns READY against the post-`/qa-plan` HEAD.** Per-row `/story-readiness` for the 8 non-human Sprint 17 rows is the next launchable prompt sequence (PROMPT 1101 onward). PROMPT 1100 commits on worker branch `qa-plan/sprint-17-1100` from base `origin/main@cb62a9e`; push target worker branch only; orchestrator integrates separately. Cargo policy: N/A for this paperwork-only authoring (no `cargo` / `trunk` / CI command invoked). Files changed: `production/qa/qa-plan-sprint-17.md` NEW + `production/session-state/active.md` PROMPT 1100 banner prepended above PROMPT 1099 banner + this file PROMPT 1100 paragraph prepended above PROMPT 1099 paragraph + `reports/PROMPT-1100-sprint-17-qa-plan.md` mandatory final report file (`reports/` gitignored; not staged or committed). Files explicitly NOT touched by PROMPT 1100: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt`, `production/sprints/**` (sprint-17 plan body NOT rewritten), `production/epics/**`, `production/gate-checks/**`, `production/sprint-status.yaml` (top-level + `stories:` block + `sprint_17_activation:` block + all prior `sprint_N_*` blocks preserved verbatim — existing schema does not require per-sprint QA-plan metadata distinct from `sprint_17_activation.qa_plan_found` + `qa_plan_note` which referenced PROMPT 1100 as the authoring run; PROMPT 1100 leaves those fields unchanged so the activation history reads correctly through time; a Sprint 17 close-out prompt MAY add a `sprint_17_qa_plan:` block analogous to `sprint_N_closeout:` pattern, deferred to close-out), `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`.)

Updated: 2026-05-18 (PROMPT 1099 -- Sprint 17 activation; paperwork-only shared-status writer. Source-of-truth at activation: `origin/main@bc3db291fb2e9b840c986b68ea8899664bba94b6` = PROMPT 1097 paperwork-only main integration tip merging PROMPT 1095 net-new Sprint 17 story authoring batch into main; strict fast-forward descendant of `origin/main@e6a6e11` PROMPT 1090 Sprint 17 plan draft commit base and of `origin/main@fec13ff` PROMPT 1088 Sprint 16 close-out main integration tip. `production/sprint-status.yaml` top-level flipped `sprint: 16 -> 17` and `status: closed-with-conditions -> active`; `stage: Polish` UNCHANGED; `production/stage.txt` NOT modified (remains `Polish`). PROMPT 761 `Polish->Release` gate-check FAIL preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` with NO retry. Sprint 17 is NOT a `Polish->Release` activation. Five draft conditional Must Have rows DROPPED at activation because their repairs landed on `origin/main` between PROMPT 1090 draft (`e6a6e11`) and PROMPT 1099 activation (`bc3db29`): `S17-UI-MODAL-BLACK-SLAB-001` (PROMPT 1080 / 1083 / 1094); `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (PROMPT 1085 / 1094); `S17-UI-PLACEMENT-PERSPECTIVE-001` (PROMPT 1086 / 1092); `S17-UI-LOBBY-CLASS-ART-CONFIRM-001` (PROMPT 1081 / 1087 / 1089; placeholder PNGs only, real-art deferred to Sprint 18+ under `PAW-TD-*-a`); `S17-SERVER-AUCTION-TIMER-001` (PROMPT 1091 worker `4b5d751` + PROMPT 1091 integration `e3c91d5`). Sprint 17 final 9-row active set (2 Must Have + 4 Should Have + 3 Nice to Have; ~2.65d / 7.5d capacity): Must Have -- `S11-HUD-TIMER-EYEBALL-VISUAL-001` 0.25d conditional human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry (no LLM `/story-done` authorised; story file unchanged since Sprint 13) + `S17-UI-CARD-DISPLAY-ART-HELPER-001` 0.75d (SOURCE-1077-01 / 02 / 03 / 04 bundle); Should Have -- `S17-UI-HUD-OPP-MANA-CLEANUP-001` 0.5d + `S17-UI-CARD-SLOT-INSET-WIRING-001` 0.25d + `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` 0.25d + `S17-UI-BID-BUTTON-PHASE-RACE-001` 0.25d; Nice to Have -- `S17-OPS-VULKAN-VALIDATION-GATING-001` 0.15d + `S17-SERVER-START-OF-TURN-DEBUG-001` 0.1d + `S17-UI-HAND-B0004-CLEANUP-001` 0.15d. All 9 story files exist on `origin/main` at activation HEAD `bc3db29` (PROMPT 1097 integrated 8 net-new Sprint 17 stories authored by PROMPT 1095; story-014 HUD timer file unchanged since Sprint 13). Next launchable prompt: PROMPT 1100 `/qa-plan sprint-17` (authored ONLY after PROMPT 1099 activation lands on `origin/main`; no `/dev-story` authorised before the QA plan exists). Per-row `/story-readiness` reruns against Sprint 17 activation HEAD `bc3db29` follow PROMPT 1100 for the 8 net-new rows. Non-claims preserved verbatim: no public release readiness, no RC readiness, no full game completion, no `QA-COND-0005` Standard-tier accessibility advancement, no `QA-COND-0006` playtest validation advancement, no full playable-client manual QA, no `S8-QA-001-W1` closure (remains OPEN), no `PAW-TD-*-a` final-art completion, no `Polish->Release` gate-check retry, no stage advance, no Sprint 12 story 019 underlying drag-runtime bug fix, no `TQ-S12-C7` closure, no LLM closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, no pixel-level closure of PROMPT 1054 P1 UI snapshot visual retest, no closure of any PROMPT 1076 / 1077 finding without a concrete repair on `origin/main`, no closure of any of the 24 PROMPT 1022 QA snapshot audit findings, no Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen, no Sprint 16 close-out reopen or re-author. Files changed by PROMPT 1099: `production/sprint-status.yaml` (top-level flip + stories block + `sprint_17_activation:` block at EOF replacing `next_sprint_17_draft:`; all prior `sprint_N_*` blocks preserved verbatim above), `production/sprints/sprint-17.md` (ACTIVATED banner prepended above PROMPT 1090 DRAFT banner), `production/session-state/active.md` (PROMPT 1099 banner prepended above PROMPT 1090 banner), `production/session-state/codex-orchestrator-state.md` (this PROMPT 1099 paragraph prepended above PROMPT 1090 paragraph), `reports/PROMPT-1099-sprint-17-activation.md` (mandatory final report; `reports/` is gitignored). Forbidden paths NOT touched by PROMPT 1099: `production/stage.txt`, `production/qa/**`, `production/gate-checks/**`, `production/epics/**`, `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `.octogent/`, `.claude/settings.json`, `.claude/scheduled_tasks.lock`, Sprint 16/15/14/13/12/11/10 plan files, all prior `sprint_N_closeout:` / `sprint_N_activation:` / `sprint_N_story_done:` blocks. No cargo / trunk / CI command invoked. Cargo policy: N/A for this paperwork-only activation. Branch / push: PROMPT 1099 commits the activation paperwork on branch `activate/sprint-17-1099` from base `origin/main@bc3db291fb2e9b840c986b68ea8899664bba94b6`; push target: worker branch only; never `main`.).

---

Updated: 2026-05-18 (PROMPT 1090 -- Sprint 17 Plan Draft; paperwork-only shared-status writer. Source-of-truth at authoring: `origin/main@fec13ffc3723d9d68afdda4b6e4bf62af5d6da2a` = PROMPT 1088 Sprint 16 close-out main integration tip (`integrate(s16): merge Sprint 16 close-out paperwork into main (PROMPT 1088)`) merging PROMPT 1082 close-out paperwork (`860e08e`) into `origin/main`. Branch: `sprint-plan/sprint-17-draft-1090` from base `origin/main@fec13ff`; worktree `D:/_DEV/claude-code-game-studios-worktrees/sprint-17-plan-draft-1090`. Single-context paperwork-only Sprint 17 plan draft authoring run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness`, `/story-done` invoked by PROMPT 1090. **Sprint 17 is NOT activated by PROMPT 1090.** Top-level `production/sprint-status.yaml` `sprint: 16 / status: closed-with-conditions / stage: Polish` preserved verbatim; only a `next_sprint_17_draft:` block is appended at EOF following the `sprint_16_closeout:` block. Stage UNCHANGED `Polish`; `production/stage.txt` NOT modified by PROMPT 1090. PROMPT 761 Polish->Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; NO retry attempted by PROMPT 1090. Inputs read: PROMPT 1076 latest user-test log / snapshot deep audit (`reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`, 18 findings: 3 P0 + 9 P1 + 4 P2 + 5 P3); PROMPT 1077 UI / state source consistency deep audit (`reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`, 16 findings: 2 P0 + 4 P1 + 9 P2 + 1 P3); PROMPT 1083 modal integration report (`reports/PROMPT-1083-client-phase-modal-black-slab-integration.md`, PASS at integration time, NOT on `origin/main` -- worker `cbc11b2` on `origin/work/client-phase-modal-black-slab-1080`, integration `e4bbca3` on `origin/integrate/client-phase-modal-black-slab-1083`); PROMPT 1087 lobby class art + Confirm-class integration (49/49 PASS, 5 files M3/A2, integration `eec2a91` on `origin/integrate/lobby-class-art-confirm-1087`, NOT on `origin/main`); PROMPT 1089 lobby class art main-push attempt (`NEEDS-REFRESH`: integration not ancestor of `fec13ff` because of PROMPT 1082 + PROMPT 1088 close-out paperwork; non-FF would lose close-out paperwork; placeholder asset md5 gaps noted: lobby-portrait-png x7 + slot-panel + room-code-chip + board-sprite x7). PROMPT 1085 (client shop / auction surface paint + intent repair) worker on `origin/work/client-shop-auction-surface-paint-1085` and PROMPT 1086 (client placement perspective + invalid-drop feedback repair) worker on `origin/work/client-placement-perspective-submit-1086` are present only as worker branches; no integration prompts; no PROMPT 1085 or PROMPT 1086 reports present at draft time. PROMPT 1079 (`c5b0d04`) + PROMPT 1084 (`dd7f5d3`) server placement buffer log + spawn integration ARE on `origin/main`; AUDIT-1076-02 server placement-buffer race + AUDIT-1076-03 spawn-loop counter mismatch treated as **partially landed** server side; client-side residual gap AUDIT-1076-09 + PROMPT 1079 client residual risk #2 remains a Sprint 17 candidate. Sprint 17 plan shape: focused runtime / UI repair sprint prioritising runtime-playability blockers and visible UX fixes ahead of broad UI architecture debt. Must Have candidates (~5.0d upper bound / ~1.0d lower bound; row count depends on which in-flight repairs land on origin/main before activation): (1) `S11-HUD-TIMER-EYEBALL-VISUAL-001` 0.25d Sprint 13 -> 14 -> 15 -> 16 -> 17 human-operator-blocked carry (conditional; dropped if closed on origin/main before activation; no LLM /story-done authorised); (2) `S17-UI-MODAL-BLACK-SLAB-001` 0.25d paperwork OR 1.0d worker re-run conditional (AUDIT-1076-01 P0; PROMPT 1080 / 1083); (3) `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` 1.5d conditional (AUDIT-1076-04 P1 + AUDIT-1076-13 P2; PROMPT 1085; depends on modal repair landing); (4) `S17-UI-PLACEMENT-PERSPECTIVE-001` 1.0d conditional (AUDIT-1076-09 P1 UX + PROMPT 1079 client residual risk #2; PROMPT 1086); (5) `S17-UI-LOBBY-CLASS-ART-CONFIRM-001` 0.25d paperwork OR 1.5d worker re-run conditional (AUDIT-1076-06 P1 + AUDIT-1076-07 P1; PROMPT 1087 / 1089; placeholder PNGs only; real-art deferred to Sprint 18+); (6) `S17-UI-CARD-DISPLAY-ART-HELPER-001` 0.75d non-conditional bundled row (SOURCE-1077-01 P0 + SOURCE-1077-02 P0 + SOURCE-1077-03 P1 + SOURCE-1077-04 P1; dedup `apply_card_display_art` to single owner + preserve slot-well chrome + drop `Box::leak` + existence check). Should Have (~1.75d): HUD opponent figurine + OPP label + mana microbadge dedup (AUDIT-1076-10/16/17); auction state-machine timer latency repair (AUDIT-1076-12 conditional); card-slot primitive image/text inset wiring (SOURCE-1077-06); QA snapshot marker split + visibility-aware counts (SOURCE-1077-08/09/16); bid-button phase-entry race cleanup (SOURCE-1077-10). Nice to Have (~0.4d): Vulkan validation gating (AUDIT-1076-18); start_of_turn_dispatch warn -> debug (AUDIT-1076-15); Hand UI B0004 hierarchy warning cleanup (AUDIT-1076-14). Deferred to Sprint 18+ explicitly: per-surface card-slot primitive migration siblings (S17-UI-CARD-SLOT-MIGRATION-HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST family; producer may pull one row into Sprint 17 if capacity allows); real-art production for 7 lobby portraits + slot-panel + room-code-chip + 7 board sprites (PAW-TD-*-a accept-risk preserved); 24 PROMPT 1022 QA snapshot audit findings; 12 remaining PROMPT 1077 SOURCE-* structural findings (SOURCE-1077-05/07/11/12/13/14/15) not absorbed by Sprint 17 active rows; Sprint 11/12/13 server hardening backlog; PROMPT 803 §5 Should/Nice rows not pulled into Sprint 13/14/15/16; Tier 2 cosmetic captures bundle; PROMPT 1076 long-tail (AUDIT-1076-05 P1 giant blurry ? glyph; AUDIT-1076-08 P1 TO PLACE ART placeholder; AUDIT-1076-11 P2 Resolution phase no visualisation) -- reassess after Sprint 17 Must Have rows land. Next launchable prompts after PROMPT 1090: PROMPT 1091a modal main-push refresh (rebase `e4bbca3` onto `fec13ff` + re-run 10 focused tests + push integration branch + paperwork main-push); PROMPT 1091b shop / auction surface paint integration (worker -> integration + tests + main-push paperwork; sequenced AFTER PROMPT 1091a); PROMPT 1091c client placement perspective integration (worker -> integration + tests + main-push paperwork; may run in parallel with PROMPT 1091a); PROMPT 1091d lobby class art main-push refresh (rebase `eec2a91` onto current origin/main + resolve placeholder-asset md5 gaps + re-run 49 tests + paperwork main-push); PROMPT 1092 story-authoring prompts for Sprint 17 net-new candidate rows (each story file embeds source AUDIT-1076-* or SOURCE-1077-* ID and minimal repair surface from the audit); PROMPT 1093 Sprint 17 activation (mirrors PROMPT 1064 pattern; audits origin/main at activation to drop conditional rows whose in-flight repair has landed); PROMPT 1094 /qa-plan sprint-17 authored ONLY after PROMPT 1093 (NO /dev-story before QA plan lands). Files changed by PROMPT 1090: `production/sprints/sprint-17.md` (NEW; Sprint 17 DRAFT plan body); `production/sprint-status.yaml` (`next_sprint_17_draft:` block appended at EOF only; top-level `sprint: 16 / status: closed-with-conditions / stage: Polish` preserved verbatim; all prior `sprint_N_closeout:` / `sprint_N_activation:` / `sprint_N_story_done:` blocks preserved verbatim above); `production/session-state/active.md` (PROMPT 1090 banner prepended above PROMPT 1082 banner); `production/session-state/codex-orchestrator-state.md` (this PROMPT 1090 paragraph prepended above PROMPT 1082 paragraph); `reports/PROMPT-1090-Sprint-17-Plan-Draft.md` (mandatory final report; `reports/` is gitignored). Files explicitly NOT touched by PROMPT 1090: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt`, `production/sprints/sprint-16.md` / `sprint-15.md` / `sprint-14.md` / `sprint-13.md` / `sprint-12.md` / `sprint-11.md` / `sprint-10.md`, `production/qa/*` (no Sprint 17 QA plan authored by PROMPT 1090), `production/gate-checks/*` (PROMPT 761 Polish->Release FAIL preserved), `production/epics/**` (no story files authored by PROMPT 1090; story authoring deferred to PROMPT 1092), `production/sprint-status.yaml` top-level `sprint: / status: / stage:` (preserved verbatim `sprint: 16 / status: closed-with-conditions / stage: Polish`), `production/sprint-status.yaml` `sprint_16_closeout:` / `sprint_16_activation:` / `sprint_16_story_done:` PROMPT 1072 + PROMPT 1074 entries / earlier `sprint_N_*` blocks (all preserved verbatim above `next_sprint_17_draft:`), `production/sprint-status.yaml` `stories:` block (4 Sprint 16 rows preserved verbatim including the open `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry), `.octogent/`, `.claude/scheduled_tasks.lock`, `.claude/settings.json`. No cargo / trunk / CI command invoked. **Cargo policy: N/A** for PROMPT 1090 paperwork-only Sprint 17 plan draft. Non-claims preserved verbatim by PROMPT 1090: NO Sprint 17 activation; NO Sprint 17 sprint-status active row; NO Sprint 16 close-out reopen or re-author; NO closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry; no LLM `/story-done` authorised by PROMPT 1090); NO public release readiness; NO RC readiness; NO full game completion; NO broad / Standard-tier accessibility completion (`QA-COND-0005` accept-risk preserved); NO playtest / fun-hypothesis validation (`QA-COND-0006` accept-risk preserved); NO full playable-client manual QA; NO two-client `GAME_OVER` closure (`S8-QA-001-W1` remains OPEN); NO final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved; Sprint 17 lobby class art row uses placeholder PNGs only); NO Polish->Release gate-check retry (PROMPT 761 `FAIL` preserved); NO stage advance from Polish to Release (`production/stage.txt` NOT modified); NO underlying drag-runtime bug fix (Sprint 12 story 019 `cannot-reproduce` preserved); NO closure of `TQ-S12-C7`; NO pixel-level closure of PROMPT 1054 P1 UI snapshot visual retest; NO pixel-level QA snapshot capture for the Sprint 16 card-slot primitive shop-panel bundles at 1366x768 / 1920x1080 (story 009 AC6 PARTIAL preserved); NO closure of any of the 24 PROMPT 1022 QA snapshot audit findings; NO closure of any PROMPT 1076 finding for which a concrete repair is not on origin/main at activation time; NO closure of any PROMPT 1077 finding outside the Sprint 17 Must Have S17-UI-CARD-DISPLAY-ART-HELPER-001 bundle and Should Have S17-UI-CARD-SLOT-INSET-WIRING-001 + S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 + S17-UI-BID-BUTTON-PHASE-RACE-001 rows; NO closure of `S8-QA-001-W1`; NO Sprint 15 row reopen; NO Sprint 14 / 13 / 12 / 11 / 10 row reopen; NO Sprint 16 row reopen; NO `/dev-story` / `/story-readiness` / `/story-done` / `/smoke-check` / `/team-qa` / `/gate-check` / `/release-check` / `/qa-plan` run by PROMPT 1090; NO implementation work, NO cargo / trunk / build / test run, NO Polish->Release retry, NO Sprint 17 activation. Branch / push: PROMPT 1090 commits `sprint-plan(s17): Sprint 17 plan draft -- NOT activated (PROMPT 1090)` (or analogous message) on branch `sprint-plan/sprint-17-draft-1090` from base `origin/main@fec13ffc3723d9d68afdda4b6e4bf62af5d6da2a`; push target: worker branch only; never `main`. If branch push is blocked, report exact blocker. **Late-breaking update at PROMPT 1090 commit time**: between authoring and commit `origin/main` advanced 5 commits to `e6a6e11b7c3359e076dd1e3c71d47015fa1cf739` (7f10b42 PROMPT 1081 lobby class art worker + eec2a91 PROMPT 1087 lobby integration + d87939c PROMPT 1086 client placement perspective fix + d51e246 PROMPT 1089 refresh merge + e6a6e11 PROMPT 1092 client placement perspective + submit feedback integration); branch rebased onto e6a6e11 before commit. PROMPT 1086 client placement perspective + invalid-drop feedback repair is NOW on `origin/main` -> conditional Must Have row S17-UI-PLACEMENT-PERSPECTIVE-001 (1.0d) PRE-DROPPED at draft commit time; AUDIT-1076-09 client-side gap discharged on `origin/main`. PROMPT 1087 / 1089 lobby class art + Confirm-class button repair is NOW on `origin/main` -> conditional Must Have row S17-UI-LOBBY-CLASS-ART-CONFIRM-001 (0.25d-1.5d) PRE-DROPPED at draft commit time; AUDIT-1076-06 + AUDIT-1076-07 lobby gaps discharged on `origin/main` (placeholder PNGs only; real-art deferred Sprint 18+ under `PAW-TD-*-a` accept-risk). Remaining conditional Must Have rows at commit time: S17-UI-MODAL-BLACK-SLAB-001 (PROMPT 1080 worker `cbc11b2` + PROMPT 1083 integration `e4bbca3` NOT on `origin/main`) + S17-UI-SHOP-AUCTION-SURFACE-PAINT-001 (PROMPT 1085 worker only; NOT on `origin/main`; depends on modal repair landing). Sprint 17 Must Have scope at commit time shrinks to ~2.75d-3.5d (HUD timer 0.25d + modal repair 0.25d-1.0d + shop / auction surface paint 1.5d + card-display-art helper bundle 0.75d). Next launchable prompts list at commit time: PROMPT 1091c (client placement perspective integration) + PROMPT 1091d (lobby class art main-push refresh) SUPERSEDED by the landed PROMPT 1086 + PROMPT 1087 / 1089 commits and may be dropped; PROMPT 1091a (modal main-push refresh) + PROMPT 1091b (shop / auction surface paint integration) remain priority next-launchable prompts. The conditional-row mechanism in the Sprint 17 draft body handles row drops at activation; no row revisions to the Sprint 17 draft body required by this late-breaking update.)

---

Updated: 2026-05-18 (PROMPT 1082 -- Sprint 16 Close-Out Disposition; paperwork-only shared-status writer. Source-of-truth at close-out: `origin/main@dd7f5d32c420ab92bd42f61d95d4db4470d07d28` = PROMPT 1084 server placement buffer + spawn integration tip (`integrate(s16): server placement buffer + spawn integration (PROMPT 1084)`); strict fast-forward descendant of `origin/main@f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` which is the Sprint 16 source-of-truth at PROMPT 1078 Team-QA review-of-record. The post-f8eac30 commits on origin/main are c652b46 (chore(env): enable bypassPermissions + author default-to-acting protocol) + e4c7f2f (revert(claude-md): drop Default-to-acting Mode section) + cbc11b2 (fix(ui): hide DraftInitial modal outside DraftInitial, PROMPT 1080) + e4bbca3 (integrate(s16): merge client-phase-modal black-slab repair, PROMPT 1083) + c5b0d04 (fix(server-placement): log rejections, fix LaneWide spawn count, filter reveal, PROMPT 1079) + dd7f5d3 (integrate(s16): server placement buffer + spawn integration, PROMPT 1084). These are .claude/CLAUDE.md edits + PROMPT 1079/1080 server placement + UI modal repair code, file-disjoint from the Sprint 16 4-row active set + the Sprint 16 QA evidence files. Branch: `closeout/sprint-16-1082` from base `origin/main@dd7f5d3`; worktree `D:/_DEV/claude-code-game-studios-worktrees/sprint-16-closeout-1082`. Single-context paperwork-only close-out run; no spawned CCGS subagents; no `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/story-readiness`, `/story-done` invoked by PROMPT 1082. Sprint 16 disposition flipped `active -> closed-with-conditions` on `production/sprint-status.yaml` top-level line 24. Basis: 3 of 4 Sprint 16 active rows closed (Must Have 0/1 + Should Have 1/1 + Nice to Have 2/2). Sole open row `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have, story 014, 0.25d, human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 carry; promoted Should -> Must in Sprint 15 by PROMPT 988) remains `status: ready` / `human_operator_blocked: true` and is carried forward unchanged; closure remains gated on real human-operator screenshot capture across DraftInitial 45s / DraftShop 30s / Placement 10-12s phases per story file ACs and cannot be auto-closed by an LLM `/story-done`. The orchestrator explicitly decided on 2026-05-17 to continue without human input and defer human visual testing later; human-operated HUD timer eyeball capture is NOT a stop-the-line blocker for close-out under PROMPT 1082; allowed to carry to Sprint 17 if no human-operator slot opens. Preserved verbatim by PROMPT 1082: smoke PROMPT 1075 `PASS-WITH-WARNINGS` at commit `56655fc8c20c1aad8485f2de41c656cbb7c96900` on `origin/qa/sprint-16-smoke-check-1075` (evidence file `production/qa/smoke-sprint-16-2026-05-18.md` lives on smoke branch; NOT integrated under `production/qa/` on `origin/main` by PROMPT 1082; cargo aggregate 1464 passed / 0 failed / 0 ignored / 0 measured / 0 filtered across 223 binaries; environment-only Windows `CARGO_TARGET_DIR` contention workaround documented -- live game-binary contention forced sibling target dir `D:/_DEV/cargo-target/ccgs-msvc-smoke-1075`; no code regression; accepted by PROMPT 1078 without remediation); Team-QA PROMPT 1078 `APPROVED-WITH-CONDITIONS` at commit `70f6b2345890dfc83ac09755f7622d918f47df36` on `origin/qa/sprint-16-team-qa-1078` (evidence file `production/qa/team-qa-sprint-16-2026-05-18.md` lives on team-qa branch; NOT integrated under `production/qa/` on `origin/main` by PROMPT 1082; strict fast-forward descendant of `origin/main@f8eac30`; merge-base == HEAD; 13 carry conditions enumerated; all are existing carry conditions; none closed by Team-QA). Stage UNCHANGED `Polish`; `production/stage.txt` NOT modified by PROMPT 1082. PROMPT 761 Polish->Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; NO retry attempted by PROMPT 1082; NO retry in scope. Sprint 17 NOT activated by PROMPT 1082; no `next_sprint_17_draft:` pointer created; `production/sprints/sprint-16.md` plan body NOT rewritten (only CLOSED-WITH-CONDITIONS banner prepended above prior PROMPT 1064 ACTIVATED + PROMPT 1024 DRAFT banners). Conditions carried forward unchanged: `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked carry (allowed to carry to Sprint 17; no LLM /story-done authorised), PROMPT 1054 P1 UI snapshot visual retest `BLOCKED-HUMAN-OPERATOR` deferred, `S8-QA-001-W1` OPEN (story 017 AC12 forbid-auto-closure preserved through Sprint 16), `QA-COND-0005` Standard-tier accessibility accepted-risk (friend-game scope), `QA-COND-0006` playtest / fun-hypothesis validation accepted-risk / deferred, `PAW-TD-*-a` placeholder-art accept-risk across PAW-002..PAW-006, PROMPT 683-era runtime divergence preserved (no third same-scope retest per `TQ-S12-C2`), Sprint 12 story 019 cannot-reproduce preserved (no underlying drag-runtime bug fix claim), `TQ-S12-C1..C7` preserved verbatim (`TQ-S12-C7` AppCompat informational condition explicitly NOT closed by `S15-OPS-APPCOMPAT-MANIFEST-001` row closure; the manifest row is an ops hygiene robustness improvement, the informational condition closure is a separate decision outside Sprint 16 scope), Sprint 15 closed-with-conditions per PROMPT 1056 preserved unchanged, Sprint 14 closed-with-conditions per PROMPT 987 preserved unchanged, Sprint 13 / 12 / 11 / 10 dispositions preserved unchanged, all 3 closed Sprint 16 /story-done closures (PROMPT 1072 AppCompat + dead-code two-row batch + PROMPT 1074 card-slot primitive single-row closure) preserved verbatim on `origin/main`, `sprint_16_activation:` (PROMPT 1064) block preserved verbatim, Sprint 16 QA plan (PROMPT 1066) `production/qa/qa-plan-sprint-16.md` preserved verbatim on `origin/main`. PROMPT 1076 (latest user-test log/snapshot deep audit) + PROMPT 1077 (UI state source consistency deep audit) findings are post-Sprint inputs unless integrated separately; NOT claimed as repairs by PROMPT 1082. PROMPT 1079 / 1080 / 1083 / 1084 server-placement + UI modal repair commits on origin/main are file-disjoint from the Sprint 16 4-row active set + the Sprint 16 QA evidence files; NOT pulled into Sprint 16 close-out scope by PROMPT 1082. Files changed by PROMPT 1082: `production/sprint-status.yaml` (top-level `status:` flipped `active -> closed-with-conditions` on line 24; `# Last close-out:` comment on line 9 refreshed with PROMPT 1082 narrative + demoted PROMPT 1056 narrative to `# Previous:` line 10; `updated:` comment on line 12 refreshed with PROMPT 1082 narrative + demoted PROMPT 1064 narrative to `# Previous:` continuation; `sprint_16_closeout:` block appended at EOF following `sprint_15_closeout` / `sprint_14_closeout` / `sprint_13_closeout` / `sprint_12_closeout` / `sprint_11_closeout` / `sprint_10_closeout` pattern, positioned after `sprint_16_story_done:` PROMPT 1072 + PROMPT 1074 entries; no `next_sprint_17_draft:` block created); `production/sprints/sprint-16.md` (CLOSED-WITH-CONDITIONS banner prepended above prior PROMPT 1064 ACTIVATED banner + PROMPT 1024 DRAFT banner; plan body NOT rewritten); `production/session-state/active.md` (PROMPT 1082 close-out banner prepended above PROMPT 1074 banner); `production/session-state/codex-orchestrator-state.md` (this PROMPT 1082 paragraph prepended above PROMPT 1074 paragraph); `reports/PROMPT-1082-Sprint-16-Close-Out-Disposition.md` (mandatory final report; `reports/` is gitignored). Files explicitly NOT touched by PROMPT 1082: `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt`, `production/qa/qa-plan-sprint-16.md`, `production/qa/smoke-*.md` (PROMPT 1075 smoke evidence file lives on smoke branch only), `production/qa/team-qa-*.md` (PROMPT 1078 Team-QA evidence file lives on team-qa branch only), `production/qa/evidence/*` (sprint-16-ui-card-slot-primitive/ + sprint-16-appcompat-manifest-evidence.md + sprint-16-workspace-dead-code-warning/ preserved verbatim), `production/gate-checks/*` (PROMPT 761 Polish->Release FAIL preserved), any Sprint 16/15/14/13/12/11/10 story file under `production/epics/`, any prior `sprint_N_closeout:` / `sprint_N_activation:` / `sprint_N_story_done:` block, `sprint_16_activation:`, `sprint_16_story_done:` PROMPT 1072 + PROMPT 1074 entries, stories block, `.octogent/`, `.claude/scheduled_tasks.lock`, `.claude/settings.json`. No cargo / trunk / CI command invoked. Cargo policy: N/A for PROMPT 1082 paperwork-only close-out. Non-claims preserved verbatim by PROMPT 1082: no public release readiness, no RC readiness, no full game completion, no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged), no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged), no full playable-client manual QA (`S8-QA-001-W1` unchanged), no two-client `GAME_OVER` closure (`S8-QA-001-W1` remains OPEN), no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved), no Polish->Release gate-check retry (PROMPT 761 `FAIL` preserved), no stage advance from Polish to Release, no underlying drag-runtime bug fix (Sprint 12 story 019 cannot-reproduce preserved), no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry; closure remains gated on human-operator screenshot capture; no LLM `/story-done` authorised by PROMPT 1082), no pixel-level closure of PROMPT 1054 P1 UI snapshot visual retest, no pixel-level QA snapshot capture for the Sprint 16 card-slot primitive shop-panel bundles at 1366x768 / 1920x1080 (story 009 AC6 PARTIAL preserved; QA snapshot bundles remain human-operator-deferred via the `S15-QA-SNAPSHOT-DEFAULT-DEV` flow), no closure of `S8-QA-001-W1` / `TQ-S12-C7`, no closure of the three per-surface card-slot migration siblings (`S16-UI-CARD-SLOT-MIGRATION-HAND-001` / `-AUCTION-001` / `-BOARD-GHOST-001`), no closure of any of the 24 PROMPT 1022 QA snapshot audit findings, no PROMPT 1076 / 1077 audit repairs, no Sprint 17 activation, no Sprint 17 sprint-status active row, no `next_sprint_17_draft:` pointer creation, no Sprint 16 row reopen (the 3 closed Sprint 16 rows preserved unchanged), no Sprint 15 row reopen (the 4 closed Sprint 15 rows preserved unchanged), no Sprint 14 / 13 / 12 / 11 / 10 row reopen. Next launchable prompts: human-operator HUD timer eyeball screenshot capture session for `S11-HUD-TIMER-EYEBALL-VISUAL-001` (allowed to carry to Sprint 17; no LLM closure authorised); paperwork integration of Sprint 16 smoke + Team-QA evidence files into `production/qa/` on `origin/main` (separate documentation-completeness prompt; verdicts already preserved as branch-commit pointers in this close-out); Sprint 17 sprint plan draft authoring (separate paperwork prompt; NOT authored by PROMPT 1082); PROMPT 1076 / 1077 audit findings consumption into Sprint 17+ story authoring (separate paperwork). Branch / push: PROMPT 1082 commits `closeout(s16): close Sprint 16 with deferred human visual conditions (PROMPT 1082)` on branch `closeout/sprint-16-1082` from base `origin/main@dd7f5d32c420ab92bd42f61d95d4db4470d07d28`; push target: worker branch only; never `main`. If main push is needed by local convention, stop and report that orchestrator/human approval is required.)

---
---

## Archive pointer

Historical chronicle (pre-2026-05-18 `Updated:` paragraphs and all
`## Current verified state` sections back to 2026-04-14) moved to
`production/session-state/archive/orchestrator-archive-2026-05.md`
on 2026-05-20 to reduce per-turn boot context cost. Read that file
on-demand if forensic on older PROMPTs is needed.
