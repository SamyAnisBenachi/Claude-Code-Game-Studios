# Sprint 12 -- DRAFT (Polish stage)

> **Status**: `draft` -- authored 2026-05-13 by PROMPT 793 (producer +
> qa-lead roles, root checkout, no worktree). Source-of-truth at draft:
> `origin/main@8a8451e` (PROMPT 792 close-out commit
> `close-out(s11): Sprint 11 close-out disposition PASS-WITH-CONDITIONS`).
> Stage remains `Polish` (NOT Release). Sprint 11 disposition preserved:
> `closed-with-conditions` per PROMPT 792. PROMPT 793 paperwork-only
> draft: NO `/dev-story`, NO `/smoke-check`, NO `/team-qa`,
> NO `/gate-check`, NO `/story-done`, NO `/story-readiness`,
> NO `/qa-plan`, NO implementation, NO CI runs were performed.
> Sprint 12 is **NOT activated** by this draft; activation happens via
> `/sprint-plan sprint-12` in a separate prompt.
>
> **Start / end (provisional; locked at activation)**: 2026-06-18 ->
> 2026-07-01 (10 workdays). Continuous follow-on to Sprint 11
> (2026-06-04 -> 2026-06-17).
>
> **Release scope**: explicitly OUT of Sprint 12. PROMPT 761
> Polish->Release gate-check `FAIL` evidence remains preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`. Do **not**
> retry the Polish->Release gate-check until release-scope artifacts
> (final art, manual-QA sign-off, accessibility completion, playtest
> evidence) actually exist on `main`. Sprint 12 does **not** advance
> stage.

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`.
  Sprint 12 does NOT advance stage.
- Sprint 11 is `closed-with-conditions` (PROMPT 792); 6/6 Must Have
  done; 0/4 Should Have done; 0/6 Nice to Have done. All Should Have
  and Nice to Have rows were explicitly deferred forward to Sprint 12+
  planning by the Sprint 11 close-out (see
  `sprint_11_closeout.deferred_into_sprint_12_planning` in
  `production/sprint-status.yaml`). This draft pulls those deferrals
  plus the Cluster B follow-up slugs from the Sprint 11 D-5 triage
  evidence plus the follow-on diagnostic story 019 authored at
  worker commit `0fc05c3` (PROMPT 778) and present on `main`.
- This draft pulls candidates from: Sprint 11 close-out deferred items
  (the four Should Have rows + six Nice to Have rows); the five
  Cluster B retained D-5 ignored tests (`production/qa/evidence/sprint-11-ignored-d5-triage.md`);
  the follow-on diagnostic story 019
  (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`);
  the wider Sprint 11 backlog "not yet pulled" list
  (`production/sprints/sprint-11.md` lines 251-286).
- PR-SPRINT skipped -- Lean mode (no `production/review-mode.txt`).
- No Sprint 12 QA plan exists at draft time. A Sprint 12 QA plan must
  be authored via `/qa-plan sprint` after Sprint 12 story files exist
  and pass `/story-readiness`, and before any Polish gate-check or
  sprint close-out claim for Sprint 12.
- Sprint 12 explicitly does NOT claim public release readiness,
  release-candidate readiness, full game completion, broad
  Standard-tier accessibility completion, full playable-client manual
  QA, playtest / fun-hypothesis validation, final-art /
  asset-production completion, `S8-QA-001-W1` closure, or a
  Polish->Release retry. None of these can be added to Sprint 12 by
  activation; they require their own scope and gate evidence.

## Entry Conditions (must be true at activation)

- Sprint 11 row in `production/sprint-status.yaml` reads
  `closed-with-conditions` (already true at draft time per PROMPT 792).
- `production/stage.txt` still reads `Polish`.
- PROMPT 761 Polish->Release gate-check `FAIL` evidence is preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`.
- `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006` dispositions intact.
- Sprint 12 story files referenced below have either been authored and
  passed `/story-readiness`, or are explicitly held with a written
  blocker.

If any entry condition fails, Sprint 12 does NOT activate; producer
must revise scope before activation.

## Sprint Goal

Close the open-ended diagnostic from Sprint 11 (drag-runtime divergence
tighter-capture retest via story 019), retire the five retained
Cluster B D-5 `#[ignore]` tests via owner-named follow-up stories or
explicit design-decision gates, and drain the Sprint 11 Should Have /
Nice to Have backlog rows that were deferred forward by the Sprint 11
close-out (HUD timer eyeball visual check, client phase_changed
60Hz idempotency, server `init_pool` log guard, cargo disk-usage /
PDB-pressure investigation notes, orchestrator-root concurrent-session
lock pattern doc, `gh` CLI setup note, lobby "Confirming..." text
differentiation, intermittent R2 Placement runtime crash audit) --
without expanding into broad production, public release readiness,
broad accessibility completion, full playable-client manual QA,
playtest validation, full asset / content production, or full game
completion.

## Capacity (provisional)

- Total workdays: 10 (assumes 2-week sprint same as Sprint 10/11)
- Buffer (20%): 2 days reserved for diagnostic-capture friction
  (story 019 tighter-capture), fixture-cascade tail repair, and
  integration friction
- Available: **8 effective planned days**
- Planned Must Have scope: **~5.0 estimated days** (story 019 + 4
  Cluster B follow-ups)
- Should Have scope is conditional and must not displace Must Have
  closure.
- Nice to Have scope is documentation-tier and lands only when
  Should Have closure is on track.

---

## Tasks

> All IDs below are **draft S11-* / S12-*** tickets. They are NOT yet
> active `sprint-status.yaml` rows. Slugs prefixed `S11-` are carried
> forward unchanged from Sprint 11 close-out deferrals to preserve
> traceability (e.g., evidence cross-links in
> `sprint_11_closeout.deferred_into_sprint_12_planning`). Slugs
> prefixed `S12-` are net-new candidates surfaced in this draft.
> Promotion to active rows happens at activation via
> `/sprint-plan sprint-12`.

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001 | Drag-and-drop runtime divergence -- tighter-capture diagnostic-only retest per story 019 | client gameplay programmer + orchestrator | 1.50 | `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md` (follow-on from Sprint 11 `S11-DRAG-RUNTIME-RETEST-001` `cannot-reproduce`) | Story 019 `/story-readiness` passes; the upgraded `RUST_LOG=...,lightyear=debug,server::game=debug` invocation is exercised on a real two-client friend-game session; frame-level video captured for drag attempts A/B/C/D; S1-S5 truth-table locked at `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md` (NEW; do **not** overwrite the story-018 evidence file) with at least one PASS/FAIL row per column; disposition is exactly one of `{bug-reproduced, bug-fixed, cannot-reproduce, third-party-limitation}`; no production code changes land inside this story (any repair commit is delegated to a follow-on story); no optimistic client-side authority introduced (ADR-002 + ADR-009 binding). Inherits story-018 / story-019 no-claim banner verbatim. |
| S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001 | Cluster B2: `HudPlugin` `snapshot.phase -> CurrentClientPhase` bridge fixture gap -- design decision + repair | board-rendering test infra + HUD plugin owner + qa-lead | 0.75 | Sprint 11 D-5 triage evidence, Cluster B2 (`production/qa/evidence/sprint-11-ignored-d5-triage.md` line 84); PROMPT 762 candidate #5 | Story file authored at `production/epics/playable-client/story-XXX-fixture-hud-snapshot-phase-bridge.md` (NEW) and passes `/story-readiness`. Design decision recorded in story file: **expand `BoardRenderingPlugin`-only fixture to include `HudPlugin`** OR **relocate the `snapshot.phase -> CurrentClientPhase` assertion into a dedicated HUD test**. Test `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives` (`tests/integration/board_rendering/snapshot_spawn_test.rs:39`) is un-`#[ignore]`d under whichever path is chosen and passes. No production code modified outside of the test/fixture exception unless the chosen path explicitly requires it (write-up gates the production-code path). Original PROMPT 750 D-5 owner-comment removed only after the test passes. |
| S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001 | Cluster B3: lobby `ConfirmClass` intent not emitted alongside `SelectClass` -- production fix in lobby input system | client gameplay programmer (lobby input) + ux-designer | 1.00 | Sprint 11 D-5 triage evidence, Cluster B3 (`production/qa/evidence/sprint-11-ignored-d5-triage.md` line 85); PROMPT 762 candidate #7 | Story file authored at `production/epics/playable-client/story-XXX-lobby-confirm-class-intent-chain.md` (NEW) and passes `/story-readiness`. Investigation note recorded in story file: input chain stops at `SelectClass` after the D-1 fix; reason determined. Production fix lands in lobby input system so that `ConfirmClass` intent is emitted alongside `SelectClass` (or via a follow-on production-driven event chain) -- **no client-side optimistic class-lock authority added** (ADR-002 binding). Test `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands` (`tests/integration/playable_client/native_operator_controls_test.rs:106`) is un-`#[ignore]`d and passes. Integration test asserts the two-intent chain end-to-end. |
| S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001 | Cluster B4: `co_occupancy_offset` no longer panics on index 2 -- binary design decision | board-rendering owner + qa-lead | 0.50 | Sprint 11 D-5 triage evidence, Cluster B4 (`production/qa/evidence/sprint-11-ignored-d5-triage.md` line 86); PROMPT 762 candidate #3 | Story file authored at `production/epics/playable-client/story-XXX-cooccupancy-panic-guard-decision.md` (NEW) and passes `/story-readiness`. **Binary decision recorded in story file**: (a) panic-guard restored in production `co_occupancy_offset` with test re-armed as `#[should_panic(expected = "unit_index=2")]` OR (b) test rewritten to assert non-panic behaviour with production rationale captured. **Path (a) requires explicit production-design write-up before code change.** Resolution **must not** silently delete the `#[should_panic]` invariant without an explicit production-design write-up. Test `test_cooccupancy_index_two_panics_with_offending_index` (`tests/unit/board_rendering/status_icons_test.rs:167`) is un-`#[ignore]`d under whichever path is chosen and passes. |
| S11-TD-FIXTURE-D-RESIDUALS-001 | Cluster B1 + B5 umbrella: `BoardRenderingPlugin`-only ghost producer fixture gap (B1) **and** `ShopAuctionUiEntity` count drift (B5) | test infra + scaffold owner + qa-lead | 1.25 | Sprint 11 close-out deferral (Should Have, blocked); Sprint 11 D-5 triage evidence, Cluster B1 + B5 (`production/qa/evidence/sprint-11-ignored-d5-triage.md` lines 83 + 87); PROMPT 762 candidates #4 + #6 | Story file authored at `production/epics/playable-client/story-XXX-fixture-d-residuals.md` (NEW) and passes `/story-readiness`. **Producer decision recorded in story file**: keep this row as the umbrella for B1 + B5 **OR** split into per-test rows `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` (B1) and `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` (B5). Whichever path is chosen, each disposition is documented: B1 = fixture expansion to include `HandUiPlugin` pointer-to-drag bridge or scope the assertion to a `HandUiPlugin` fixture; B5 = update formula (57 -> 66) **OR** trim spawn (66 -> 57) -- recorded with rationale before code change. Tests `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui` (`tests/integration/board_rendering/ghost_preview_bridge_test.rs:147`) and `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` (`tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:25`) are un-`#[ignore]`d under whichever path is chosen and pass. PR-bundle keeps test-only changes unless the formula-vs-trim decision requires a production-code edit (then the edit is scoped narrowly with a written rationale). |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | HUD timer eyeball visual check (W2 carry from Sprint 10 smoke retry-7) | UI programmer | 0.25 | Sprint 11 close-out deferral (Should Have, blocked); smoke retry-7 W2 | Story file authored at `production/epics/playable-client/story-XXX-hud-timer-eyeball-visual-check.md` (NEW) and passes `/story-readiness`. Manual 2-client run validates timer countdown renders correctly for `DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s phases. Evidence: screenshot capture in `production/qa/evidence/sprint-12-hud-timer-visual-check/`. Cosmetic verification only; no production-code change unless an actual visual regression is found and a follow-on story is authored. Does NOT claim Standard-tier accessibility completion. |
| S11-HU-PHASE-IDEMPOTENCY-001 | Client `phase_changed=true` 60Hz idempotency | client gameplay programmer | 0.75 | Sprint 11 close-out deferral (Should Have, blocked); Wave 12 backlog | Story file authored at `production/epics/playable-client/story-XXX-client-phase-changed-idempotency.md` (NEW) and passes `/story-readiness`. Spurious `phase_changed=true` on every frame reduced to actual phase transitions only. Existing `S2CPhaseChanged` drain remains the single source of phase truth. Integration test asserts no `phase_changed=true` outside actual phase transition frames. **No client-side optimistic phase authority added** (ADR-002 + ADR-009 binding). |
| S11-SERVER-POOL-INIT-LOG-GUARD-001 | Server `init_pool` log emits before guard | server gameplay programmer | 0.25 | Sprint 11 close-out deferral (Should Have, blocked); Wave 12 backlog | Story file authored at `production/epics/server/story-XXX-init-pool-log-guard.md` (NEW) and passes `/story-readiness`. `init_pool` info-level log gated such that it does not emit before the initialization guard fires. Pattern matches W5 `acquisition_tick` fix from `ee27fb6`. Smoke / log evidence target: <50 emitted lines per session for the cold path. |
| S11-LOBBY-UX-CONFIRM-STATE-001 | Lobby "Confirming..." text differentiation (own-confirm-acked vs waiting-opponent) | UI programmer | 0.50 | Sprint 11 close-out deferral (Nice to Have, blocked, promoted) | Story file authored at `production/epics/playable-client/story-XXX-lobby-confirm-state.md` (NEW) and passes `/story-readiness`. Lobby UI text distinguishes the two states. **No client-side class-lock authority added** (ADR-002 binding). Integration test asserts text differentiation across the two states. Promoted from Sprint 11 Nice to Have to Sprint 12 Should Have because it shares review surface with `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001` (Cluster B3 Must Have) -- batching these two reduces re-review cost. |

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-TD-CARGO-DISK-USAGE-001 | Cargo workspace disk-usage reduction strategy | devops-engineer | 0.50 | Sprint 11 close-out deferral (Nice to Have, blocked); Wave 12 backlog (D: hit 100% 3x) | Story file authored at `production/epics/devops/story-XXX-cargo-workspace-disk-usage.md` (NEW) and passes `/story-readiness`. Investigation note at `docs/architecture/cargo-workspace-disk-usage.md` (NEW) documents current `target/` footprint per worktree, identifies trim candidates (shared target dir, prune debug symbols, sccache, etc.), recommends a single change to land in a follow-on story. **No build-script changes land in this story.** |
| S11-TD-CARGO-PDB-LIMIT-001 | Cargo PDB-size pressure investigation | devops-engineer | 0.25 | Sprint 11 close-out deferral (Nice to Have, blocked); Wave 12 backlog | Story file authored at `production/epics/devops/story-XXX-cargo-pdb-limit.md` (NEW) and passes `/story-readiness`. Document PDB-size impact on disk usage and CI runtime. Recommend Windows-side `split-debuginfo` / `strip` profile knobs for `[profile.dev]` or `[profile.test]`. **No profile changes land in this story.** |
| S11-OPS-ORCHESTRATOR-LOCK-001 | Orchestrator-root concurrent-session lock pattern | orchestrator | 0.25 | Sprint 11 close-out deferral (Nice to Have, blocked); Wave 12 backlog (2x sessions mutating main HEAD concurrently) | Story file authored at `production/epics/devops/story-XXX-orchestrator-lock.md` (NEW) and passes `/story-readiness`. Lock-file or convention documented at `.octogent/orchestrator-lock.md` (or appended to existing orchestrator docs) describing how to detect / avoid concurrent root-checkout writes. **No code lands; pattern is documented only.** |
| S11-OPS-GH-CLI-001 | `gh` CLI installation note for dev machine | orchestrator | 0.10 | Sprint 11 close-out deferral (Nice to Have, blocked); Wave 12 backlog (`gh` absent 3+ times) | Story file authored at `production/epics/devops/story-XXX-gh-cli-setup.md` (NEW) and passes `/story-readiness`. One paragraph in repo onboarding doc (or `docs/setup/dev-environment.md`) names `gh` as required, with install command. **No tooling changes land.** |
| S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 | Intermittent R2 Placement runtime crash audit | server gameplay programmer | 0.50 | Sprint 11 close-out deferral (Nice to Have, blocked); Wave 12 backlog (12:07 capture; not reproduced 13:28) | Story file authored at `production/epics/server/story-XXX-r2-placement-crash-audit.md` (NEW) and passes `/story-readiness`. Audit log emits enriched diagnostics around `Phase::Placement` round-2 transition. If a repro is captured during Sprint 12, a follow-on story is authored with the precise repro. **No fix is implemented in this story.** |

---

## Carryover from Sprint 11

| Source row (Sprint 11) | Disposition into Sprint 12 |
|------------------------|----------------------------|
| Cluster B D-5 ignored tests (5/5 retained on `main`) | Each gets a dedicated Sprint 12 Must Have row OR is folded under `S11-TD-FIXTURE-D-RESIDUALS-001` umbrella (B1, B5). B2 / B3 / B4 are dedicated Must Have rows. None silently dropped. |
| `story-019` follow-on diagnostic (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`; on `main` at `0fc05c3`) | Folded into `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001` (Must Have). |
| `S11-TD-FIXTURE-D-RESIDUALS-001` (Sprint 11 Should Have, blocked) | Pulled forward as Sprint 12 Must Have umbrella for B1 + B5 -- producer decision (umbrella vs split) recorded in the new story file. |
| `S11-HU-PHASE-IDEMPOTENCY-001` (Sprint 11 Should Have, blocked) | Pulled forward as Sprint 12 Should Have. |
| `S11-SERVER-POOL-INIT-LOG-GUARD-001` (Sprint 11 Should Have, blocked) | Pulled forward as Sprint 12 Should Have. |
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 11 Should Have, blocked; W2 carry from Sprint 10) | Pulled forward as Sprint 12 Should Have. |
| `S11-LOBBY-UX-CONFIRM-STATE-001` (Sprint 11 Nice to Have, blocked) | Promoted into Sprint 12 Should Have to batch with Cluster B3 lobby work. |
| `S11-TD-CARGO-DISK-USAGE-001` (Sprint 11 Nice to Have, blocked) | Pulled forward as Sprint 12 Nice to Have. |
| `S11-TD-CARGO-PDB-LIMIT-001` (Sprint 11 Nice to Have, blocked) | Pulled forward as Sprint 12 Nice to Have. |
| `S11-OPS-ORCHESTRATOR-LOCK-001` (Sprint 11 Nice to Have, blocked) | Pulled forward as Sprint 12 Nice to Have. |
| `S11-OPS-GH-CLI-001` (Sprint 11 Nice to Have, blocked) | Pulled forward as Sprint 12 Nice to Have. |
| `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001` (Sprint 11 Nice to Have, blocked) | Pulled forward as Sprint 12 Nice to Have. |

## Conditions Carried Forward Unchanged (NOT closed by Sprint 12)

Sprint 12 explicitly preserves and does NOT claim closure for any of:

- **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap
  remains OPEN.
- **`QA-COND-0005`** -- Standard-tier accessibility remains
  accepted-risk (friend-game scope only); Sprint 12 does NOT pursue
  Standard-tier accessibility completion.
- **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains
  accepted-risk / deferred; Sprint 12 does NOT pursue playtest
  evidence.
- **Placeholder / friend-game art scope** -- `PAW-TD-*-a`
  accept-risk on placeholder PNGs across PAW-002..PAW-006 remains in
  place; no final-art / asset-production completion is pursued.
- **PROMPT 683-era runtime divergence question** -- carried forward
  into story 019 tighter-capture; Sprint 12 does NOT claim closure of
  the question outside the four story-019 dispositions
  (`bug-reproduced`, `bug-fixed`, `cannot-reproduce`,
  `third-party-limitation`).
- **PROMPT 761 Polish->Release gate-check FAIL** -- preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO
  retry** is in scope for Sprint 12.

If any condition above changes during Sprint 12, it requires its own
separate story file and explicit disposition -- it cannot be silently
folded into another story.

## Wider Sprint 12 Backlog (not yet pulled into this draft)

The following S11-* / S12-* candidates remain in the broader backlog
and are **NOT scheduled** into this Sprint 12 draft. They may be
pulled by a producer revision before activation, or deferred to
Sprint 13:

- Optional splits for the Cluster B residuals umbrella:
  `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` (B1) and
  `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` (B5). If kept under the
  `S11-TD-FIXTURE-D-RESIDUALS-001` umbrella, both ride together; if
  split, each lifts to its own Must Have row.
- Server hardening test parity: `S11-TD-NET-001`, `S11-TD-NET-002`,
  `S11-TD-NET-003`.
- `S11-TD-PRISM-COV-001` -- Cluster 2C advisory coverage gap on
  `S2CPrismRewardDropped` + `S2CPrismRespawned`.
- `S11-TD-HARNESS-MESSAGES-001` -- 4 harness bins downstream from
  PROMPT 690 needing `add_message::<PlayerTeamMapUpdated>`.
- `S11-TD-HARNESS-HANDUI-ENTITIES-001` -- 2 harness bins downstream
  from PROMPT 690 needing `HandUiEntities`.
- `S11-TD-BOARD-RENDERING-SNAPSHOT-PHASE-COUPLING-001` (split from
  PROMPT 680 PARTIAL closure).
- `S11-TD-FIXTURE-MESSAGES-002` (wider exhaustive `add_message` sweep
  -- Option B from PROMPT 708).
- `S11-TD-CI-NORMALIZE-COMMENTS-001` (teach `normalize_source()` to
  strip Rust comments -- Option B from PROMPT 674 FAIL report).
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`.
- UI clean-pass 8-story milestone from PROMPT 685 audit:
  `S11-TD-UI-ZINDEX-LAYERS`, `S11-TD-UI-FLEX-STRIPS` +
  `S11-UX-HUD-TOP-STRIP-LAYOUT` + `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` +
  `S11-UX-HUD-OPP-FIGURINE`, `S11-UX-DRAFT-GRID-CENTERED-MODAL`,
  `S11-UX-AUCTION-FEATURED-CARD` + `S11-UX-AUCTION-FREE-GOLD-COUNTERS`,
  `S11-UX-LOBBY-CLASS-PICKER` + `S11-UX-LOBBY-BUTTON-HITTARGETS`,
  `S11-UX-BOARD-RENDERING-SPEC`, `S11-TD-UI-FONT-CONSTANTS`,
  `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`.
- Route-readability future-story candidates from
  `production/qa/evidence/sprint-10-route-readability-notes.md`:
  `S11-UX-LOBBY-ROOM-CODE-EYEBALL-001`,
  `S11-UX-LOBBY-OPP-SLOT-DISAMBIGUATION-001`,
  `S11-HU-DRAG-FEEDBACK-DIFFERENTIATION-001`,
  `S11-DRAFT-INITIAL-OVERLAY-EYEBALL-001`,
  `S11-UX-SHOP-SLOT-AFFORDANCE-001`,
  `S11-UX-SHOP-INLINE-GOLD-READ-ORDER-001`,
  `S11-UX-AUCTION-SETTLEMENT-VISUAL-EYEBALL-001`,
  `S11-UX-BOARD-STATUS-ICON-LEGEND-001`,
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`,
  `S11-UX-HUD-TIMER-URGENCY-VISUAL-001`,
  `S11-UX-RESULT-RETURN-TO-LOBBY-001`.

## Required Sprint 12 Story Docs

PROMPT 793 (this draft) did NOT author new story files. Before
`/dev-story` begins on any Must Have / Should Have / Nice to Have
row, the following story files must exist and pass `/story-readiness`:

| Planned ID | Required story file |
|------------|---------------------|
| S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001 | `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md` (EXISTS on `main` at `0fc05c3`; `/story-readiness` pending) |
| S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001 | `production/epics/playable-client/story-XXX-fixture-hud-snapshot-phase-bridge.md` (NEW) |
| S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001 | `production/epics/playable-client/story-XXX-lobby-confirm-class-intent-chain.md` (NEW) |
| S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001 | `production/epics/playable-client/story-XXX-cooccupancy-panic-guard-decision.md` (NEW) |
| S11-TD-FIXTURE-D-RESIDUALS-001 | `production/epics/playable-client/story-XXX-fixture-d-residuals.md` (NEW; umbrella OR split into B1 + B5 stories) |
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | `production/epics/playable-client/story-XXX-hud-timer-eyeball-visual-check.md` (NEW) |
| S11-HU-PHASE-IDEMPOTENCY-001 | `production/epics/playable-client/story-XXX-client-phase-changed-idempotency.md` (NEW) |
| S11-SERVER-POOL-INIT-LOG-GUARD-001 | `production/epics/server/story-XXX-init-pool-log-guard.md` (NEW) |
| S11-LOBBY-UX-CONFIRM-STATE-001 | `production/epics/playable-client/story-XXX-lobby-confirm-state.md` (NEW) |
| S11-TD-CARGO-DISK-USAGE-001 | `production/epics/devops/story-XXX-cargo-workspace-disk-usage.md` (NEW) |
| S11-TD-CARGO-PDB-LIMIT-001 | `production/epics/devops/story-XXX-cargo-pdb-limit.md` (NEW) |
| S11-OPS-ORCHESTRATOR-LOCK-001 | `production/epics/devops/story-XXX-orchestrator-lock.md` (NEW) |
| S11-OPS-GH-CLI-001 | `production/epics/devops/story-XXX-gh-cli-setup.md` (NEW) |
| S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 | `production/epics/server/story-XXX-r2-placement-crash-audit.md` (NEW) |

Until story files exist and pass `/story-readiness`, the corresponding
Sprint 12 rows in `production/sprint-status.yaml` (once activated) are
tracked as `blocked` by missing story docs.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Story 019 tighter-capture also dispositions `cannot-reproduce` | Medium | High | Time-box 1.5 days per story 019 §"Time-box". `cannot-reproduce` (second time) is a valid outcome and triggers escalation to Sprint 13 candidate authoring with expanded tracing scope. |
| Cluster B production-fix rows (B3 lobby intent chain, B4 panic-guard decision) expand into broader pattern reauthoring | Medium | Medium | Scope-cap at the named test + the named production module + one written rationale doc per row. Anything beyond gets a separate Sprint 13 candidate. |
| Sprint 12 expands into release-scope work | Medium | High | Release-scope is explicitly OUT of Sprint 12. PROMPT 761 FAIL evidence remains preserved. `/scope-check` enforced before activation. |
| Sprint 11 deferrals are silently dropped during Sprint 12 activation | Low | High | Carryover table above forces explicit preservation. Sprint 12 close-out cannot complete without an explicit disposition (done / deferred / dropped-with-rationale) for every deferred row. |
| `S8-QA-001-W1` is silently dropped if carry-over consolidation misses it | Medium | High | Conditions block above forces explicit preservation. Sprint 12 close-out paperwork must surface `S8-QA-001-W1` disposition explicitly. |
| `QA-COND-0005` (Standard-tier accessibility) is misrepresented as closed | Medium | High | Polish / friend-game scope language preserved on every Sprint 12 story; no Standard-tier accessibility work is in scope. |
| `QA-COND-0006` (playtest / fun-hypothesis validation) is misrepresented as closed | Medium | High | Sprint 12 evidence is friend-game / fixture / paperwork / diagnostic, never playtest validation. |
| Stage advances from `Polish` without release-scope artifacts | Low | High | `production/stage.txt` is explicitly out of Sprint 12 allowed-files list. No Polish->Release retry until release-scope artifacts exist. |
| Cluster B Must Have rows fight for the same review surface (lobby / board-rendering) | Medium | Medium | Sequence B2 (HUD bridge) and B4 (panic-guard) before B3 (lobby intent chain); promote `S11-LOBBY-UX-CONFIRM-STATE-001` from Nice to Have to Should Have to batch with B3 work. |
| Concurrent-session race on orchestrator-root (Wave 12 finding) damages Sprint 12 paperwork | Medium | Medium | `S11-OPS-ORCHESTRATOR-LOCK-001` Nice to Have row documents the pattern. Orchestrator should run only one shared-status writer at a time per the 2026-05-13 override. |

## Dependencies on External Factors

- `origin/main` source-of-truth must include `8a8451e` (Sprint 11
  close-out paperwork) before Sprint 12 activation.
- A local server and two real primary clients can be run for runtime
  evidence capture (browser/native) for
  `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001` (story 019) and
  `S11-HUD-TIMER-EYEBALL-VISUAL-001`.
- Operator must have OBS Studio / ShareX / OS screen recorder
  (30 fps minimum; 60 fps preferred) for story 019 frame-level video
  capture.
- No new asset authoring depends on this sprint; `PAW-TD-*-a`
  placeholder PNGs remain accepted-risk for friend-game scope.

## Definition of Done for this Sprint

- [ ] All Must Have tasks completed and integrated.
- [ ] All Must Have tasks pass acceptance criteria.
- [ ] Sprint 12 QA plan exists at
      `production/qa/qa-plan-sprint-12.md` (this is a Sprint 12
      prerequisite -- author via `/qa-plan sprint` before
      `/dev-story` begins).
- [ ] All Logic/Integration stories have passing unit/integration
      tests.
- [ ] `cargo test -p server` and `cargo test -p client` pass without
      regression vs. Sprint 11 close-out baseline
      (`origin/main@8a8451e` -- 1129 passed / 0 failed / 5 ignored).
- [ ] If any Cluster B follow-up closes its test, the workspace
      ignored count drops by 1 (or more) without introducing new
      undocumented `#[ignore]` markers.
- [ ] `/smoke-check sprint` passed for Sprint 12 (or recorded as PASS
      WITH WARNINGS with documented warnings -- any remaining Cluster B
      tests still ignored carry owner-named follow-up slugs or decision
      gates).
- [ ] `/team-qa sprint` produced an `APPROVED` or
      `APPROVED WITH CONDITIONS` sign-off report.
- [ ] No S1 or S2 bugs in delivered Must Have features.
- [ ] `production/sprint-status.yaml` reflects every Must Have story as
      `done` or explicitly `closed-with-conditions`.
- [ ] `S8-QA-001-W1`, `QA-COND-0005`, and `QA-COND-0006` retain their
      pre-Sprint-12 disposition unless actual separate closure evidence
      lands inside Sprint 12 scope.
- [ ] `production/stage.txt` is **unchanged** (remains `Polish`).
- [ ] No public release readiness, release-candidate readiness, full
      playable-client manual QA, full game completion, broad
      Standard-tier accessibility completion, playtest /
      fun-hypothesis validation, or full asset / content production
      is claimed.
- [ ] Sprint 12 close-out paperwork explicitly states what Sprint 13
      inherits (carries vs deferrals).

## QA Plan

No Sprint 12 QA plan exists at draft time (PROMPT 793). The producer
path is:

> Run `/qa-plan sprint` after Sprint 12 story docs exist and pass
> `/story-readiness`, and before `/dev-story` begins on any Must
> Have.

A Sprint 12 plan without a QA plan cannot pass any gate-check or
sprint close-out claim.

## Scope-Creep Guard

This sprint pulls work only from Sprint 11 close-out deferrals (4
Should Have + 6 Nice to Have rows), the 5 Cluster B retained D-5
ignored tests from
`production/qa/evidence/sprint-11-ignored-d5-triage.md`, and the
follow-on diagnostic story 019 already on `main`. New asset
authoring, broad accessibility completion, public release readiness,
full playtest validation, full playable-client manual QA, and full
game completion are explicitly out of scope.

Run `/scope-check hand-ui`, `/scope-check playable-client`,
`/scope-check hud`, `/scope-check shop-auction-ui`, and
`/scope-check server` before implementation begins to detect scope
creep on the polish stories.

## Verification For Activation (deferred)

Activation will require, at minimum:

- `git diff --check` on the activation commit.
- `git diff --cached --check` before commit.
- Sprint 11 row in `production/sprint-status.yaml` reads
  `closed-with-conditions` (already true at draft time per
  PROMPT 792).
- `production/sprint-status.yaml` Sprint 12 row written by
  `/sprint-plan sprint-12` activation, not manually.
- `production/stage.txt` unchanged at `Polish`.
- PROMPT 761 Polish->Release gate-check FAIL evidence preserved.
- This draft file is reviewed and locked.

This file is a **draft** as of PROMPT 793 (2026-05-13). PROMPT 793
wrote a `next_sprint:` draft block in
`production/sprint-status.yaml` (the existing Sprint 11
`stories:` block remains the authoritative active row set; Sprint 11
disposition remains `closed-with-conditions`; the `next_sprint:`
draft block does NOT touch the active `stories:` block). Sprint 11
disposition (`closed-with-conditions`) is preserved unchanged under
`sprint_11_closeout:` in the same file and in git history.

## PROMPT 793 Producer Note -- Top 5 Recommended Must Have

(Subject to user / producer revision before activation.)

1. **`S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`** (story 019) --
   HIGH; closes the open-ended runtime divergence question carried
   over from PROMPT 683 / 696 / 697 / 698 / 706 / 709 / 778. The
   `cannot-reproduce` outcome from Sprint 11 is itself the trigger
   for this story; second `cannot-reproduce` escalates to Sprint 13
   with expanded tracing scope.
2. **`S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`** (Cluster B2) --
   HIGH; design-decision row -- expand fixture vs relocate
   assertion; unblocks one ignored test.
3. **`S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`** (Cluster B3) --
   HIGH; production-fix likely (input chain stops at `SelectClass`);
   unblocks one ignored test and removes a friend-game lobby UX gap.
4. **`S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`** (Cluster B4) --
   HIGH; binary design decision (restore guard vs rewrite test); the
   `#[should_panic]` invariant cannot be silently deleted.
5. **`S11-TD-FIXTURE-D-RESIDUALS-001`** (Cluster B1 + B5 umbrella) --
   MEDIUM; umbrella row with producer decision (umbrella vs split)
   recorded in the new story file; unblocks two ignored tests.

## PROMPT 793 Producer Note -- Suggested First Parallel Batch After Sprint 12 Plan Exists

(Once Sprint 12 is activated by `/sprint-plan sprint-12` in a separate
prompt, with story files authored and `/story-readiness` passed, an
appropriate first parallel batch is:)

- **Lane A (story authoring)**: author the 4 new Cluster B story
  files (`S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`,
  `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`,
  `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`,
  `S11-TD-FIXTURE-D-RESIDUALS-001`) in parallel; they touch disjoint
  test files and modules and are safe to run truly in parallel.
- **Lane B (story 019 readiness)**: run `/story-readiness` on
  `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  (story file already on `main` at `0fc05c3`).
- **Lane C (Nice to Have docs)**: the four documentation-tier Nice
  to Have rows (`S11-TD-CARGO-DISK-USAGE-001`,
  `S11-TD-CARGO-PDB-LIMIT-001`, `S11-OPS-ORCHESTRATOR-LOCK-001`,
  `S11-OPS-GH-CLI-001`) can each be authored as separate small
  workers; they touch disjoint files and are safe to run truly in
  parallel.
- **Hold for serial**: `/qa-plan sprint`, `/smoke-check`,
  `/team-qa`, `/gate-check` and any close-out work. Per the
  2026-05-13 override, only one shared-status writer runs at a time.

## PROMPT 793 Producer Note -- Blockers / Missing Evidence

- No Sprint 12 QA plan exists yet. Must be authored via
  `/qa-plan sprint` before any `/dev-story` runs.
- No Sprint 12 story files exist for the 4 new Cluster B Must Haves,
  the 4 Should Haves, or the 5 Nice to Haves. They must be authored
  and pass `/story-readiness` before `/dev-story` runs.
- Story 019 (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`)
  exists on `main` at `0fc05c3` but is in `Draft` status; formal
  `/story-readiness` verdict pending.
- Sprint 12 dates are not locked. Producer should lock them at
  activation.
- The `S11-TD-FIXTURE-D-RESIDUALS-001` umbrella-vs-split producer
  decision must be recorded in the new story file before any worker
  dispatch.

## PROMPT 793 Authoring Trail

- 2026-05-13 -- PROMPT 793 -- Sprint 12 draft plan authored from
  Sprint 11 close-out carries (`sprint_11_closeout.deferred_into_sprint_12_planning`),
  the 5 Cluster B retained D-5 ignored tests
  (`production/qa/evidence/sprint-11-ignored-d5-triage.md`), and the
  follow-on diagnostic story 019
  (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`).
  Paperwork-only run on root checkout (no worktree). No code under
  `client/` / `server/` / `shared/` / `tests/` modified. No
  `production/stage.txt` change. No `.claude/settings.json` change
  (pre-existing in-tree modification preserved untouched). No
  `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check` run. Sprint 11 disposition
  (`closed-with-conditions` per PROMPT 792) preserved unchanged.
  Sprint 12 explicitly **not activated** by this draft.
