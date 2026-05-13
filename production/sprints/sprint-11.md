# Sprint 11 -- DRAFT (dates TBD at activation)

> **Status**: `draft` -- NOT active. Authored 2026-05-13 by PROMPT 764
> (producer role, root checkout, no worktree). Source-of-truth at draft time:
> `origin/main@a6132d7`. Stage remains `Polish`. Sprint 10 remains
> `closed-with-conditions` per PROMPT 763. No Sprint 11 activation, no
> `/dev-story`, no `/smoke-check`, no `/team-qa`, no `/gate-check`, no
> `/story-done`, no implementation, no CI runs were performed by PROMPT 764.
>
> **Provisional start / end**: TBD by `/sprint-plan sprint-11` at activation.
> Sprint 10 ran 2026-05-21 -> 2026-06-03; a continuous Sprint 11 would run
> approximately 2026-06-04 -> 2026-06-17 (10 workdays), but these dates are
> NOT locked.
>
> **Release scope**: explicitly OUT of Sprint 11. PROMPT 761 Polish->Release
> gate-check is `FAIL` and preserved as evidence. Do not retry Release until
> release-scope artifacts (final art, manual-QA sign-off, accessibility
> completion, playtest evidence) actually exist on `main`.

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`. Sprint
  11 does not advance stage.
- Sprint 10 is `closed-with-conditions`; 6/6 Must-Have and 2/3 Should-Have
  done; S10-TD-003 / S10-N1 / S10-N2 explicitly deferred into Sprint 11
  planning (this draft).
- This draft pulls candidates from: Sprint 10 close-out deferred items;
  PROMPT 762 Sprint 11 candidate backlog (9 entries); 11 ignored D-5 tests
  from smoke retry-7 W1; HUD timer eyeball visual check (W2);
  cargo disk / PDB / tooling candidates from Wave 12; drag-and-drop
  runtime unresolved retest gap (PROMPT 762 #1); UI clean-pass 8-story
  milestone from PROMPT 685 audit.
- PR-SPRINT skipped -- Lean mode (no `production/review-mode.txt`).
- No Sprint 11 QA plan exists at draft time. A Sprint 11 QA plan must be
  authored via `/qa-plan sprint` before any Polish gate-check or sprint
  close-out claim for Sprint 11.
- Sprint 11 explicitly does NOT claim public release readiness,
  release-candidate readiness, full game completion, broad Standard-tier
  accessibility completion, full playable-client manual QA, playtest /
  fun-hypothesis validation, or final-art / asset-production completion.

## Entry Conditions (must be true at activation)

- Sprint 10 row in `production/sprint-status.yaml` reads
  `closed-with-conditions` (already true at draft time per PROMPT 763).
- `production/stage.txt` still reads `Polish`.
- PROMPT 761 Polish->Release gate-check `FAIL` evidence is preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`.
- S8-QA-001-W1, QA-COND-0005, QA-COND-0006 dispositions are intact.
- Sprint 11 story files referenced below have either been authored and
  passed `/story-readiness`, or are explicitly held with a written
  blocker.

If any entry condition fails, Sprint 11 does not activate; producer must
revise scope before activation.

## Sprint Goal

Resolve the highest-severity friend-game runtime gap (drag-and-drop
runtime divergence), drain the pervasive fixture-design cascade that
generated the 11 ignored D-5 tests in Sprint 10, finish the carried Sprint
10 paperwork (S10-TD-003 doc hygiene tail + S10-N1 evidence index + S10-N2
route readability notes), and chip away at the operational tech debt
surfaced by Wave 12 (cargo disk / PDB pressure, server log spam,
intermittent placement crash audit) -- without expanding into broad
production, public release readiness, broad accessibility completion, full
playable-client manual QA, playtest validation, full asset / content
production, or full game completion.

## Capacity (provisional)

- Total workdays: 10 (assumes 2-week sprint same as Sprint 10)
- Buffer (20%): 2 days reserved for fixture-cascade tail repair, runtime
  retest evidence capture, and integration friction
- Available: **8 effective planned days**
- Planned Must Have scope: **~6.5 estimated days**
- Should Have scope is conditional and must not displace Must Have closure.

---

## Tasks

> All IDs below are draft S11-* tickets. They are NOT active
> `sprint-status.yaml` rows. Promotion to active rows happens at
> activation via `/sprint-plan sprint-11`.

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-DRAG-RUNTIME-RETEST-001 | Drag-and-drop runtime divergence -- retest, lock S1-S5 grey-square truth table, repair if needed | client gameplay programmer + orchestrator | 1.50 | PROMPT 762 candidate #1 (HIGH, gameplay-blocking) | Runtime trace with `RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info` is captured for a real friend-game session; the S1-S5 grey-square attribution truth-table from PROMPT 698 / 706 / 709 is locked; test-vs-runtime divergence is identified, dispositioned (production fix OR test gap), and either a repair commit lands on `main` OR a concrete follow-on story is authored with a precise repro. No new optimistic client-side authority is introduced. Evidence at `production/qa/evidence/sprint-11-drag-runtime-evidence.md`. |
| S11-TD-FIXTURE-HAND-UI-ONENTER-001 | Pervasive fixture-design fix: 7x `spawn_hand_ui` not firing on `OnEnter(InSession)` in `MinimalPlugins` fixtures | server / client test infra | 1.50 | PROMPT 762 candidate #2 (HIGH; pervasive; unblocks 7+ tests + future) | The 7 `#[ignore]`d tests from smoke retry-7 W1 with the `spawn_hand_ui` / `OnEnter(InSession)` owner comment either (a) un-`#[ignore]` and pass under a corrected fixture pattern, or (b) are formally redesigned with a documented reason. A reusable fixture helper is authored under `tests/helpers/` (or equivalent) and the canonical pattern is documented in `docs/architecture/test-fixture-patterns.md` (or appended to an existing test-pattern doc). `cargo test -p server` and `cargo test -p client` pass for the un-ignored set. No production code in `server/`, `client/`, `shared/` is modified by this story. |
| S11-TD-IGNORED-D5-TRIAGE-001 | Owner-named `#[ignore]` triage -- 11 D-5 tests from smoke retry-7 | qa-lead + test owners | 1.00 | Smoke retry-7 W1 (`production/qa/smoke-sprint-10-2026-05-12-retry-7.md`) | Each of the 11 owner-named `#[ignore]` tests is dispositioned: (a) un-`#[ignore]` after fixture/code repair; (b) redesign + retain; (c) delete with rationale. Triage doc authored at `production/qa/evidence/sprint-11-ignored-d5-triage.md` with one row per test, owner, decision, follow-on story id if any. Tests covered by `S11-TD-FIXTURE-HAND-UI-ONENTER-001` are linked, not re-resolved. |
| S11-DOC-HYGIENE-CARRY-001 | Carry S10-TD-003 outstanding -- ADR-011 `TR-NP-04 -> TR-NP-006` corrections + Rule 7 `ADR-011` breadcrumb | orchestrator + architect | 0.25 | S10-TD-003 deferred (PROMPT 763) | `docs/architecture/adr-011-reconnect-snapshot.md:173` and `:810` literal `TR-NP-04` references are corrected to `TR-NP-006`. Network Protocol Rule 7 (`design/gdd/network-protocol.md`) gains an `ADR-011` breadcrumb. No protocol or architecture decision is changed. Evidence is the diff itself plus a one-paragraph note in `production/session-state/codex-orchestrator-state.md`. |
| S11-EVIDENCE-INDEX-CARRY-001 | Sprint 10 evidence aggregator index (S10-N1 carry) | orchestrator | 0.25 | S10-N1 deferred (PROMPT 763) | `production/qa/evidence/sprint-10-evidence-index.md` is authored, linking each Sprint 10 per-story evidence file (HUD chrome, shop/auction chrome, lobby chrome, test-fixture repair, plugin audit), each story's `/story-done` verdict, build/commit, manual route status, and explicit no-claim language. Does NOT modify Sprint 10 close-out disposition. |
| S11-ROUTE-READABILITY-CARRY-001 | Friend-game route readability notes (S10-N2 carry) | UI programmer + orchestrator | 0.25 | S10-N2 deferred (PROMPT 763) | A single concise notes file at `production/qa/evidence/sprint-10-route-readability-notes.md` (or `design/ux/friend-game-route-readability-notes.md`) lists rough-edge readability observations for HUD, hand, shop, auction, board, lobby. Fixes only proceed if they directly improve the active friend-game loop and do not expand into broad Standard-tier accessibility. Records explicit no-claim for QA-COND-0005 / QA-COND-0006. |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-TD-FIXTURE-D-RESIDUALS-001 | Residual fixture cluster from Wave 12 D-3 / D-4 / D-5 sweeps | test infra | 1.00 | Wave 12 backlog (codex-orchestrator-state.md L3939) | `board_rendering_ghost_preview_bridge_test` ghost-preview producer fixture gap, `board_rendering_snapshot_spawn_test` phase routing, `board_rendering_status_icons_test` should-panic drift, `shop_auction_ui_plugin_scaffold_formulas_test` count drift 57->66 are each dispositioned (fix OR redesign OR delete) with one disposition row per test. Each disposition documented; PR-bundle keeps test-only changes. |
| S11-HU-PHASE-IDEMPOTENCY-001 | Client `phase_changed=true` 60Hz idempotency | client gameplay programmer | 0.75 | Wave 12 backlog -- `S11-CLIENT-HAND-UI-PHASE-TRANSITION-IDEMPOTENCY-001` | Spurious `phase_changed=true` on every frame is reduced to actual phase transitions only. Existing `S2CPhaseChanged` drain remains the single source of phase truth. Integration test asserts no `phase_changed=true` outside actual phase transition frames. No client-side optimistic phase authority is added. |
| S11-SERVER-POOL-INIT-LOG-GUARD-001 | Server `init_pool` log emits before guard | server gameplay programmer | 0.25 | Wave 12 backlog | `init_pool` info-level log is gated such that it does not emit before the initialization guard fires. Pattern matches W5 acquisition_tick fix from `ee27fb6`. Smoke / log evidence: <50 emitted lines per session for the cold path. |
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | HUD timer eyeball visual check (W2 carry) | UI programmer | 0.25 | Smoke retry-7 W2 | Manual 2-client run validates timer countdown renders correctly for `DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s phases. Evidence: screenshot capture in `production/qa/evidence/sprint-11-hud-timer-visual-check/`. Cosmetic verification only. |

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-TD-CARGO-DISK-USAGE-001 | Cargo workspace disk-usage reduction strategy | devops-engineer | 0.50 | Wave 12 backlog (D: hit 100% 3x) | Investigation note at `docs/architecture/cargo-workspace-disk-usage.md` documents current `target/` footprint per worktree, identifies trim candidates (shared target dir, prune debug symbols, sccache, etc.), recommends a single change to land in a follow-on story. No build-script changes land in this story. |
| S11-TD-CARGO-PDB-LIMIT-001 | Cargo PDB-size pressure investigation | devops-engineer | 0.25 | Wave 12 backlog | Document PDB-size impact on disk usage and CI runtime. Recommend Windows-side `split-debuginfo` / `strip` profile knobs for `[profile.dev]` or `[profile.test]`. No profile changes land in this story. |
| S11-OPS-ORCHESTRATOR-LOCK-001 | Orchestrator-root concurrent-session lock pattern | orchestrator | 0.25 | Wave 12 backlog (2x sessions mutating main HEAD concurrently) | A lock-file or convention is documented at `.octogent/orchestrator-lock.md` (or appended to existing orchestrator docs) describing how to detect / avoid concurrent root-checkout writes. No code lands; pattern is documented only. |
| S11-OPS-GH-CLI-001 | `gh` CLI installation note for dev machine | orchestrator | 0.10 | Wave 12 backlog (`gh` absent 3+ times) | One paragraph in repo onboarding doc (or `docs/setup/dev-environment.md`) names `gh` as required, with install command. No tooling changes land. |
| S11-LOBBY-UX-CONFIRM-STATE-001 | Lobby "Confirming..." text differentiation (own-confirm-acked vs waiting-opponent) | UI programmer | 0.50 | Wave 8 backlog | Lobby UI text distinguishes the two states. No client-side class-lock authority is added. Integration test asserts text differentiation across the two states. |
| S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 | Intermittent R2 Placement runtime crash audit | server gameplay programmer | 0.50 | Wave 12 backlog (12:07 capture; not reproduced 13:28) | Audit log emits enriched diagnostics around `Phase::Placement` round-2 transition. If a repro is captured during Sprint 11, a follow-on story is authored with the precise repro. No fix is implemented in this story. |

---

## Carryover from Sprint 10

| Source row | Disposition into Sprint 11 |
|------------|----------------------------|
| S10-TD-003 Doc hygiene tech debt sweep (partial; deferred) | Folded into `S11-DOC-HYGIENE-CARRY-001` (Must Have). |
| S10-N1 Sprint 10 evidence index (deferred) | Folded into `S11-EVIDENCE-INDEX-CARRY-001` (Must Have). |
| S10-N2 Friend-game route readability notes (deferred) | Folded into `S11-ROUTE-READABILITY-CARRY-001` (Must Have). |
| 11 ignored D-5 tests (W1) | Folded into `S11-TD-IGNORED-D5-TRIAGE-001` + `S11-TD-FIXTURE-HAND-UI-ONENTER-001`. |
| HUD timer eyeball visual check (W2) | Folded into `S11-HUD-TIMER-EYEBALL-VISUAL-001`. |

## Conditions Carried Forward Unchanged (NOT closed by Sprint 11)

Sprint 11 explicitly preserves and DOES NOT claim closure for any of:

- **S8-QA-001-W1** -- manual / browser two-client GAME_OVER gap remains OPEN.
- **QA-COND-0005** -- Standard-tier accessibility remains accepted-risk
  (friend-game scope only); Sprint 11 does not pursue Standard-tier
  accessibility completion.
- **QA-COND-0006** -- playtest / fun-hypothesis validation remains
  accepted-risk / deferred; Sprint 11 does not pursue playtest evidence.
- **Placeholder / friend-game art scope** -- `PAW-TD-*-a` accept-risk on
  placeholder PNGs remains in place; no final-art / asset-production
  completion is pursued.

If any condition above changes during Sprint 11, it requires its own
separate story file and explicit disposition -- it cannot be silently
folded into another story.

## Wider Sprint 11 Backlog (not yet pulled into this draft)

The following S11-* candidates remain in the broader backlog and are
**NOT scheduled** into this Sprint 11 draft. They may be pulled by a
producer revision before activation, or deferred to Sprint 12:

- Server hardening test parity: `S11-TD-NET-001`, `S11-TD-NET-002`,
  `S11-TD-NET-003`.
- `S11-TD-PRISM-COV-001` -- Cluster 2C advisory coverage gap on
  S2CPrismRewardDropped + S2CPrismRespawned.
- `S11-TD-HARNESS-MESSAGES-001` -- 4 harness bins downstream from PROMPT
  690 needing `add_message::<PlayerTeamMapUpdated>`.
- `S11-TD-HARNESS-HANDUI-ENTITIES-001` -- 2 harness bins downstream from
  PROMPT 690 needing `HandUiEntities`.
- `S11-TD-BOARD-RENDERING-SNAPSHOT-PHASE-COUPLING-001` (split from PROMPT
  680 PARTIAL closure).
- `S11-TD-FIXTURE-MESSAGES-002` (wider exhaustive add_message sweep --
  Option B from PROMPT 708).
- `S11-TD-CI-NORMALIZE-COMMENTS-001` (teach `normalize_source()` to strip
  Rust comments -- Option B from PROMPT 674 FAIL report).
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`.
- ConfirmClass intent chain after SelectClass (PROMPT 762 candidate #7).
- cooccupancy panic-guard drift (PROMPT 762 candidate #3).
- ShopAuctionUiEntity count drift 57->66 (PROMPT 762 candidate #4 --
  partially covered by `S11-TD-FIXTURE-D-RESIDUALS-001`).
- HudPlugin snapshot.phase bridge fixture gap (PROMPT 762 candidate #5).
- GhostDragStartEvent producer fixture gap (PROMPT 762 candidate #6 --
  partially covered by `S11-TD-FIXTURE-D-RESIDUALS-001`).
- UI clean-pass 8-story milestone from PROMPT 685 audit:
  `S11-TD-UI-ZINDEX-LAYERS`, `S11-TD-UI-FLEX-STRIPS` +
  `S11-UX-HUD-TOP-STRIP-LAYOUT` + `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` +
  `S11-UX-HUD-OPP-FIGURINE`, `S11-UX-DRAFT-GRID-CENTERED-MODAL`,
  `S11-UX-AUCTION-FEATURED-CARD` + `S11-UX-AUCTION-FREE-GOLD-COUNTERS`,
  `S11-UX-LOBBY-CLASS-PICKER` + `S11-UX-LOBBY-BUTTON-HITTARGETS`,
  `S11-UX-BOARD-RENDERING-SPEC`, `S11-TD-UI-FONT-CONSTANTS`,
  `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`.

## Required Sprint 11 Story Docs

`/sprint-plan` (PROMPT 764) did not author new story files. Before
`/dev-story` begins on any Must Have, the following story files must
exist and pass `/story-readiness`:

| Planned ID | Required story file |
|------------|---------------------|
| S11-DRAG-RUNTIME-RETEST-001 | `production/epics/hand-ui/story-XXX-drag-runtime-retest.md` (NEW) |
| S11-TD-FIXTURE-HAND-UI-ONENTER-001 | `production/epics/playable-client/story-XXX-spawn-hand-ui-fixture-cascade.md` (NEW) |
| S11-TD-IGNORED-D5-TRIAGE-001 | No new story file required -- triage doc + per-test follow-on stories as needed |
| S11-DOC-HYGIENE-CARRY-001 | No new story file required -- doc-only sweep |
| S11-EVIDENCE-INDEX-CARRY-001 | No new story file required -- evidence aggregator |
| S11-ROUTE-READABILITY-CARRY-001 | No new story file required -- notes file |

Until story files exist and pass `/story-readiness`, the corresponding
Sprint 11 rows in `production/sprint-status.yaml` (once activated) are
tracked as blocked by missing story docs.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `S11-DRAG-RUNTIME-RETEST-001` retest is non-reproducible and burns capacity | Medium | High | Time-box runtime tracing to 1.0 day; if no repro, lock the S1-S5 truth-table as "best-effort" evidence and author a precise follow-on repro story. |
| Fixture-cascade fix (`S11-TD-FIXTURE-HAND-UI-ONENTER-001`) expands beyond the 7 tests into broader pattern reauthoring | Medium | Medium | Scope-cap at the 7 owner-named tests + one helper module + one pattern doc. Anything beyond gets a separate Sprint 12 candidate. |
| Sprint 11 expands into release-scope work | Medium | High | Release-scope is explicitly OUT of Sprint 11. PROMPT 761 FAIL evidence remains preserved. `/scope-check` enforced before activation. |
| Sprint 10 carry items (S10-TD-003 / S10-N1 / S10-N2) are silently dropped | Low | High | Three dedicated Must Have rows above; close-out cannot complete without all three reaching `done` or formal re-defer with written reason. |
| `S8-QA-001-W1` is silently dropped if carry-over consolidation misses it | Medium | High | Conditions block above forces explicit preservation. Sprint 11 close-out paperwork must surface S8-QA-001-W1 disposition explicitly. |
| `QA-COND-0005` (Standard-tier accessibility) is misrepresented as closed | Medium | High | Polish / friend-game scope language preserved on every Sprint 11 story; no Standard-tier accessibility work is in scope. |
| `QA-COND-0006` (playtest/fun-hypothesis validation) is misrepresented as closed | Medium | High | Sprint 11 evidence is friend-game / fixture / paperwork, never playtest validation. |
| Stage advances from `Polish` without release-scope artifacts | Low | High | `production/stage.txt` is explicitly out of Sprint 11 allowed-files list. No Polish->Release retry until release-scope artifacts exist. |
| Concurrent-session race on orchestrator-root (Wave 12 finding) damages Sprint 11 paperwork | Medium | Medium | `S11-OPS-ORCHESTRATOR-LOCK-001` documents the pattern. Orchestrator should run only one shared-status writer at a time per the 2026-05-13 override. |

## Dependencies on External Factors

- `origin/main` source-of-truth must include `a6132d7` (Sprint 10
  close-out paperwork) before Sprint 11 activation.
- A local server and two real primary clients can be run for runtime
  evidence capture (browser/native) for `S11-DRAG-RUNTIME-RETEST-001` and
  `S11-HUD-TIMER-EYEBALL-VISUAL-001`.
- No new asset authoring depends on this sprint; PAW-TD-*-a placeholder
  PNGs remain accepted-risk for friend-game scope.

## Definition of Done for this Sprint

- [ ] All Must Have tasks completed and integrated.
- [ ] All Must Have tasks pass acceptance criteria.
- [ ] Sprint 11 QA plan exists at `production/qa/qa-plan-sprint-11.md`
      (this is a Sprint 11 prerequisite -- author via `/qa-plan sprint`
      before `/dev-story` begins).
- [ ] All Logic/Integration stories have passing unit/integration tests.
- [ ] `cargo test -p server` and `cargo test -p client` pass without
      regression vs. Sprint 10 close-out baseline (`origin/main@a6132d7`).
- [ ] `/smoke-check sprint` passed for Sprint 11 (or recorded as PASS
      WITH WARNINGS with documented warnings).
- [ ] `/team-qa sprint` produced an `APPROVED` or `APPROVED WITH
      CONDITIONS` sign-off report.
- [ ] No S1 or S2 bugs in delivered Must Have features.
- [ ] `production/sprint-status.yaml` reflects every Must Have story as
      `done` or explicitly `closed-with-conditions`.
- [ ] `S8-QA-001-W1`, `QA-COND-0005`, and `QA-COND-0006` retain their
      pre-Sprint-11 disposition unless actual separate closure evidence
      lands inside Sprint 11 scope.
- [ ] `production/stage.txt` is **unchanged** (remains `Polish`).
- [ ] No public release readiness, release-candidate readiness, full
      playable-client manual QA, full game completion, broad Standard-tier
      accessibility completion, playtest / fun-hypothesis validation, or
      full asset / content production is claimed.
- [ ] Sprint 11 close-out paperwork explicitly states what Sprint 12
      inherits (carries vs deferrals).

## QA Plan

No Sprint 11 QA plan exists at draft time (PROMPT 764). The producer
path is:

> Run `/qa-plan sprint` after Sprint 11 story docs exist and pass
> `/story-readiness`, and before `/dev-story` begins on any Must Have.

A Sprint 11 plan without a QA plan cannot pass any gate-check or
sprint close-out claim.

## Scope-Creep Guard

This sprint pulls work only from Sprint 10 deferred items, the PROMPT 762
candidate backlog, the Wave 12 backlog, and the 11 ignored D-5 tests +
HUD timer eyeball check from smoke retry-7. New asset authoring, broad
accessibility completion, public release readiness, full playtest
validation, full playable-client manual QA, and full game completion are
explicitly out of scope.

Run `/scope-check hand-ui`, `/scope-check playable-client`,
`/scope-check hud`, and `/scope-check shop-auction-ui` before
implementation begins to detect scope creep on the polish stories.

## Verification For Activation (deferred)

Activation will require, at minimum:

- `git diff --check` on the activation commit.
- `git diff --cached --check` before commit.
- Sprint 10 row in `production/sprint-status.yaml` reads
  `closed-with-conditions` (already true at draft time per PROMPT 763).
- `production/sprint-status.yaml` Sprint 11 row written by
  `/sprint-plan sprint-11` activation, not manually.
- `production/stage.txt` unchanged at `Polish`.
- PROMPT 761 Polish->Release gate-check FAIL evidence preserved.
- This draft file is reviewed and locked.

This file is a **draft**. PROMPT 764 did NOT write an active Sprint 11
row to `production/sprint-status.yaml`. The `next_sprint` block in that
file is updated to point at this draft and to remain
`status: draft / not_active` until `/sprint-plan sprint-11` activates
Sprint 11 in a separate prompt.

## PROMPT 764 Producer Note -- Top 5 Recommended Must Have

(Subject to user / producer revision before activation.)

1. **S11-DRAG-RUNTIME-RETEST-001** -- HIGH; gameplay-blocking for
   friend-game runtime; the runtime trace was never completed across the
   PROMPT 696 / 697 / 698 / 706 / 709 chain.
2. **S11-TD-FIXTURE-HAND-UI-ONENTER-001** -- HIGH; pervasive
   fixture-design gap; unblocks 7+ ignored tests and prevents future
   ignore-creep.
3. **S11-TD-IGNORED-D5-TRIAGE-001** -- HIGH; 11 owner-named ignore-blocks
   are a known blocker on credible Sprint 11 smoke retries.
4. **S11-DOC-HYGIENE-CARRY-001** -- MEDIUM; small carry from S10-TD-003;
   correctness-only doc fix; trivial to land if not silently dropped.
5. **S11-EVIDENCE-INDEX-CARRY-001** -- MEDIUM; aggregates Sprint 10
   evidence that already exists per-story; small but explicitly named
   carry that cannot be silently dropped.

(`S11-ROUTE-READABILITY-CARRY-001` is also a Must Have carry but ranks
6th for capacity prioritisation -- it is intentionally above the cut for
"all three carries must close".)

## PROMPT 764 Producer Note -- Suggested First Parallel Batch After Sprint 11 Plan Exists

(Once Sprint 11 is activated by `/sprint-plan sprint-11` in a separate
prompt, with story files authored and `/story-readiness` passed, an
appropriate first parallel batch is:)

- **Lane A (story authoring)**: author the two new story files
  (`S11-DRAG-RUNTIME-RETEST-001`, `S11-TD-FIXTURE-HAND-UI-ONENTER-001`)
  in parallel with the triage doc skeleton for
  `S11-TD-IGNORED-D5-TRIAGE-001`.
- **Lane B (paperwork carries)**: `S11-DOC-HYGIENE-CARRY-001` +
  `S11-EVIDENCE-INDEX-CARRY-001` + `S11-ROUTE-READABILITY-CARRY-001`
  can each be dispatched as separate small workers; they touch disjoint
  files (`docs/architecture/adr-011-*` vs `production/qa/evidence/*`)
  and are safe to run truly in parallel.
- **Hold for serial**: `/qa-plan sprint`, `/smoke-check`, `/team-qa`,
  `/gate-check` and any close-out work. Per the 2026-05-13 override,
  only one shared-status writer runs at a time.

## PROMPT 764 Producer Note -- Blockers / Missing Evidence

- No Sprint 11 QA plan exists yet. Must be authored via `/qa-plan sprint`
  before any `/dev-story` runs.
- No Sprint 11 story files exist for the two HIGH Must Haves. They must be
  authored and pass `/story-readiness` before `/dev-story` runs.
- Runtime trace for drag-and-drop divergence has never been captured
  end-to-end (PROMPT 762 candidate #1). Activation of
  `S11-DRAG-RUNTIME-RETEST-001` should specify the precise
  `RUST_LOG=...` invocation, the friend-game route to execute, and the
  expected truth-table form before worker dispatch.
- Sprint 11 dates are not locked. Producer should lock them at activation.
