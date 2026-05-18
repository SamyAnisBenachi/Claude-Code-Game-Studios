# Story 021: S17-UI-HAND-B0004-CLEANUP-001 -- Hand UI Fan Root `B0004` Hierarchy Warning Cleanup

> **Epic**: Hand UI
> **Story ID**: S17-UI-HAND-B0004-CLEANUP-001
> **Status**: Draft -- Sprint 17 Nice to Have candidate (AUDIT-1076-14); NOT activated by this authoring run
> **Layer**: Hand UI / Presentation (ECS hierarchy structural fix)
> **Type**: Tech Debt -- structural ECS warning cleanup
> **Sprint**: Sprint 17 Nice to Have row per `production/sprints/sprint-17.md` §"Nice to Have". Activation is a separate explicit prompt (PROMPT 1093 pattern).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> **Estimated effort**: ~0.15d (single-surface hand UI hierarchy fix)
> **Source audit**: PROMPT 1076 `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md` §"Per-finding evidence" AUDIT-1076-14 (P3)

---

## Target Epic Justification

This story is filed under `production/epics/hand-ui/` rather
than `production/epics/ui-clean-pass/`. Justification:

- The repair surface is bounded to
  `client/src/ui/hand/fan_root*.rs` and
  `client/src/ui/hand/hand_bar*.rs` (per PROMPT 1076
  AUDIT-1076-14 minimal repair surface). Both files live under
  `client/src/ui/hand/`, which is the Hand UI epic's
  architecture module per its EPIC.md: "Architecture Module:
  `client/src/presentation/hand/` — `HandUiPlugin`".
- The Hand UI epic owns 20+ hand-specific stories (drag-state
  visuals, fan layout, placement timer, etc.). A hand-UI ECS
  hierarchy fix is the natural ownership.
- The UI Clean-Pass epic is reserved for cross-cutting / multi-
  surface UI structural refactors (modsplits, primitive
  ratifications, marker-split-across-modules). This row touches
  only one surface's ECS hierarchy.
- Sprint 17 plan row source allows either epic ("Target epic:
  production/epics/hand-ui/ or production/epics/ui-clean-pass/
  based on existing ownership. Choose one and justify."). The
  Hand UI epic is the natural ownership.

---

## Status / No-Claim Banner

This story is a Sprint 17 Nice to Have **candidate** authored by
PROMPT 1095. **No sprint is activated by this authoring run.**
PROMPT 1095 does NOT modify `production/sprint-status.yaml`,
`production/sprints/sprint-17.md`, `production/sprints/sprint-16.md`,
`production/stage.txt`, any `production/session-state/*` file, any
QA-plan / smoke / Team-QA / gate-check / release-check artifact
under `production/qa/`, any code under `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`,
`.github/`, or `Trunk.toml`. PROMPT 1095 does NOT run
`/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `cargo`,
`trunk`, or any CI command.

This story does **not** claim: public release readiness, release-
candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
validation (`QA-COND-0006`), full playable-client manual QA,
two-client GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-
production completion (`PAW-TD-*-a`), `Polish->Release` gate-check
retry, stage advance from Polish to Release, closure of the Sprint
12 story 019 underlying drag-runtime bug (specifically: this row is
ECS hierarchy hygiene; the audit notes "no functional bug evident
yet" — this is NOT a drag-runtime repair), closure of
`S11-HUD-TIMER-EYEBALL-VISUAL-001`, closure of any of the 24 PROMPT
1022 audit findings, closure of any SOURCE-1077-* finding, or
closure of any AUDIT-1076-* finding outside AUDIT-1076-14.

**No optimistic client-side authority is introduced or proposed.**
No protocol shape change. No new server-authoritative state. No
new C2S / S2C message.

Sprint 16 disposition preserved unchanged. Sprint 15 / 14 / 13 /
12 / 11 / 10 dispositions preserved unchanged. PROMPT 761 Polish
->Release gate-check `FAIL` preserved. `PAW-TD-*-a`,
`QA-COND-0005`, `QA-COND-0006`, `TQ-S12-C1..C7` preserved
verbatim.

---

## Source Finding

### AUDIT-1076-14 (P3) — B0004 hierarchy warning on Hand UI

- **Audit location**:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-14.
- **Severity**: P3.
- **Evidence**: every InSession entry (run-7 client-a:94,
  client-b:116).
- **Expected**: Bevy 0.18 entities with `GlobalTransform` should
  have a parent that also has `GlobalTransform`.
- **Actual**: Hand UI Fan Root entity carries `GlobalTransform`;
  its parent (HandBar entity) does NOT carry `GlobalTransform`.
  Bevy 0.18 emits `B0004` warning on every InSession entry.
- **State / log correlation**: structural ECS warning; may
  explain brittleness in hand layout under window resize but no
  functional bug evident yet.
- **Minimal repair surface** (audit recommendation):
  `client/src/ui/hand/fan_root*.rs`,
  `client/src/ui/hand/hand_bar*.rs`. Hand UI hierarchy aligned
  so HandBar carries `GlobalTransform` OR FanRoot is
  reparented.

---

## Problem Class / Prevention Target

**Defect class**: a child entity carrying `GlobalTransform` is
parented to an entity that does not carry `GlobalTransform`,
which violates the Bevy 0.18 invariant the engine documents via
B0004.

**Prevention target**: align the hierarchy so the invariant
holds. Two valid strategies per the audit:

1. **Strategy A — Give HandBar a `GlobalTransform`**: HandBar
   currently lacks `GlobalTransform`. Adding `GlobalTransform`
   (and `Transform`, since the Required Components API in Bevy
   0.18 auto-derives `GlobalTransform` from `Transform`) to
   HandBar resolves the warning. If HandBar is purely a bevy_ui
   `Node` and has no need for world-space transform, this
   strategy may not be appropriate; the worker re-verifies.
2. **Strategy B — Reparent FanRoot**: move FanRoot out from
   under HandBar to a parent that already carries
   `GlobalTransform`. This may require updating downstream
   queries that walk `ChildOf` chains from HandBar to FanRoot.

Implementing worker chooses one strategy and justifies. The audit
hints (but does not require) Strategy A based on "Hand UI Fan
Root entity with the GlobalTransform component has a parent (the
Hand UI HandBar entity) without GlobalTransform" — the strategy
that minimally fixes the invariant is to add `GlobalTransform`
to the parent. The Bevy 0.18 Required Components API may make
this a single-component-insert change.

---

## Context

### Existing surface

- **`client/src/ui/hand/fan_root*.rs`** — defines FanRoot entity
  spawn / structure. The implementing worker re-verifies the
  current shape at activation HEAD.
- **`client/src/ui/hand/hand_bar*.rs`** — defines HandBar entity
  spawn / structure. The implementing worker re-verifies the
  current shape.
- **Hand UI module split candidate** (`S16-TD-UI-HAND-MODSPLIT-001`,
  ui-clean-pass story 011 Draft): if this lands before Sprint 17
  activation, the FanRoot / HandBar spawn sites may live in
  per-surface submodules; the worker re-verifies.
- **Bevy 0.18 Required Components API**: `Transform` auto-
  derives `GlobalTransform` (per Bevy 0.15+ required-components
  migration). Inserting `Transform` on HandBar may be sufficient
  to make `GlobalTransform` appear automatically. The worker
  re-verifies under `liv-bevy-018`.
- **Existing hand UI test bins** under
  `tests/integration/hand-ui/` (or `tests/integration/hand_ui/`
  — re-verify). The implementing worker MAY extend an existing
  bin with a hierarchy invariant assertion or add a new bin.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md` does not specify the
  `GlobalTransform` invariant; this is engine-level. No GDD
  change in scope.
- **ADR-021** (Presentation Layer Architecture): no schedule
  change. Hierarchy structure is preserved or minimally
  realigned.
- **ADR-002** (Client-Server Authority): no change.
- **TR registry**: no new TR.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on every `.rs` edit. The
  Required Components API behaviour around `Transform` /
  `GlobalTransform` is post-cutoff (Bevy 0.15) and MUST be
  cross-referenced against the
  `docs/engine-reference/bevy/VERSION.md` migration guides. No
  Lightyear edits — `liv-bevy-lightyear` NOT required.

### Control Manifest Rules

- Required: the Bevy 0.18 `B0004` warning for FanRoot's parent
  hierarchy is gone after the fix.
- Required: existing hand UI behaviour is preserved — fan
  layout, drag-state visuals, placement staging, all current
  hand UI tests PASS unchanged.
- Required: the chosen strategy (A: give HandBar
  `GlobalTransform`; B: reparent FanRoot) is justified in the
  commit message.
- Required: smoke harness confirms zero `B0004` lines on the
  next Sprint 17 smoke against the default build profile.
- Required: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved.
- Forbidden: changing fan layout / drag-state / placement
  behaviour. This row is structural ECS hygiene only.
- Forbidden: modifying `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Forbidden: closing the Sprint 12 story 019 underlying drag-
  runtime bug. The audit notes "no functional bug evident yet"
  — this row is NOT a drag-runtime repair.
- Forbidden: closure of any AUDIT-1076-* finding outside
  AUDIT-1076-14.
- Forbidden: closure of any SOURCE-1077-* finding.
- Forbidden: closure of Sprint 11 story 018 retest or Sprint
  12 story 019 retest. Those drag-runtime questions remain
  `closed-with-conditions / cannot-reproduce` per
  `TQ-S12-C2`.

---

## Story Classification

**Story type**: **Integration** (multi-file hand UI hierarchy
realignment; touches FanRoot + HandBar spawn sites + possibly
their query consumers).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type" matrix, Integration stories require integration test OR
documented playtest (BLOCKING gate). This row delivers an
integration test that asserts the hierarchy invariant: HandBar
carries `GlobalTransform` (Strategy A) OR FanRoot's parent
carries `GlobalTransform` (Strategy B).

The Sprint 17 smoke also confirms the B0004 warning is gone in
post-implementation server / client logs — but the BLOCKING gate
is the integration test, not the smoke.

This is **NOT** a:

- Logic-only story (touches multiple files).
- Visual / feel story (no shader / VFX / animation).
- Final-art / accessibility / UI layout story.
- A drag-runtime repair (Sprint 12 story 019 question preserved).

---

## Dependencies and Parallelism

### Prerequisites

- None on `origin/main`.
- IF `S16-TD-UI-HAND-MODSPLIT-001` (UI Clean-Pass story 011,
  Draft) is activated and landed in Sprint 17, this row should
  serialise after it so the FanRoot / HandBar spawn sites are
  in their post-modsplit shape. The Sprint 17 plan does NOT
  activate `S16-TD-UI-HAND-MODSPLIT-001`; if it remains Draft,
  this row stands alone.

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must) | **PARTIAL** | both edit `client/src/ui/hand/mod.rs` or hand submodules. Serialise. |
| `S17-UI-CARD-SLOT-INSET-WIRING-001` (Should) | **YES** | disjoint (`design_tokens/card_slot.rs`). |
| `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (Should) | **PARTIAL** | both edit `client/src/ui/hand/`. Serialise. |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Should) | **YES** | disjoint (HUD). |
| `S17-UI-BID-BUTTON-PHASE-RACE-001` (Should) | **YES** | disjoint (shop_auction). |
| `S17-UI-MODAL-BLACK-SLAB-001` (conditional Must) | **YES** | disjoint (shop_auction). |
| `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (conditional Must) | **YES** | disjoint (shop_auction). |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` (Nice) | **YES** | disjoint. |
| `S17-SERVER-START-OF-TURN-DEBUG-001` (Nice) | **YES** | disjoint. |

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- B0004 warning is gone (default build)**: GIVEN
  the post-implementation client built with default features
  (`cargo build -p client` OR `trunk build`), WHEN launched and
  stderr/stdout is inspected through `InSession` entry, THEN
  there are ZERO `B0004` warning lines whose body cites Hand UI
  Fan Root / HandBar. Sprint 17 smoke (a later prompt) provides
  the cross-machine evidence; this AC is satisfied by the
  integration test (AC4 below) AND confirmed by the smoke.

- [ ] **AC2 -- Hierarchy invariant holds (Strategy A or B)**:
  GIVEN the post-implementation hand UI ECS world, WHEN
  inspected via the integration test (AC4), THEN EITHER:
  (a) FanRoot's parent (HandBar) carries `GlobalTransform` (and
  `Transform` per the Bevy 0.18 Required Components API), OR
  (b) FanRoot has been reparented to a parent that already
  carries `GlobalTransform`.
  The chosen strategy is documented in the commit message.

- [ ] **AC3 -- Existing hand UI tests continue to PASS**: GIVEN
  the existing test bins under `tests/integration/hand-ui/`
  (or `tests/integration/hand_ui/` — re-verify; per Hand UI
  epic EPIC.md Stories table the existing bins cover fan
  layout, drag-state visuals, placement, etc.), WHEN run via
  `cargo test -p client --test <bin>` for each, THEN every
  test previously PASS at activation HEAD remains PASS. No
  fan layout, drag-state, placement, or staging assertion is
  regressed.

- [ ] **AC4 -- Integration test asserts hierarchy invariant**:
  GIVEN `tests/integration/hand-ui/hand_fan_root_globaltransform_test.rs`
  (NEW; or new assertions in an existing hand UI test bin —
  worker's choice, justified in commit), WHEN run, THEN it
  asserts:
  (a) After spawning the Hand UI tree (via `HandUiPlugin` in a
  `App::new()` fixture), the FanRoot entity carries
  `GlobalTransform`.
  (b) The FanRoot entity's `ChildOf` parent carries
  `GlobalTransform`.
  (c) The assertion shape covers the audit invariant
  precisely — "child with GlobalTransform must have parent
  with GlobalTransform".

- [ ] **AC5 -- Sprint 12 story 019 disposition preserved**:
  GIVEN the commit message and any evidence document, WHEN
  inspected, THEN they explicitly do NOT claim the underlying
  drag-runtime bug from Sprint 12 story 019 is fixed. The
  audit notes "no functional bug evident yet" — this row is
  ECS hygiene, not a drag-runtime repair. The
  `closed-with-conditions / cannot-reproduce` disposition is
  preserved verbatim per `TQ-S12-C2`.

- [ ] **AC6 -- No protocol or server change**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN
  there are zero changes under `server/`, `shared/`, or
  `tests/integration/server/`. The implementation is client-side
  only.

- [ ] **AC7 -- ADR-021 schedule preserved**: GIVEN `cargo build
  -p client` under the Cargo resource policy, WHEN run, THEN no
  new system-set or schedule wiring is introduced. The hand UI
  systems remain in their existing `PresentationSet` slots.

- [ ] **AC8 -- No accept-risk closure claimed**: GIVEN the
  commit message and any evidence document, WHEN inspected,
  THEN they explicitly do NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, or any other
  accept-risk disposition. Release readiness, accessibility
  completion, playtest validation, final-art completion, stage
  advance, Polish->Release gate-check retry, and Sprint 12
  story 019 / Sprint 11 story 018 retest closure are
  explicitly out of scope.

- [ ] **AC9 -- Sprint 17 disposition preserved**: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/*`, `production/gate-checks/*`, and
  `docs/architecture/adr-*.md` are diffed, THEN none are modified
  by this story's `/dev-story` worker.

- [ ] **AC10 -- Worker branch scope contained**: GIVEN the
  worker branch (slug recommendation:
  `work/s17-hand-fan-root-b0004-hierarchy`), WHEN inspected,
  THEN it pushes only the worker branch — never `main`. Files
  changed at worker time are scoped to
  `client/src/ui/hand/fan_root*.rs`,
  `client/src/ui/hand/hand_bar*.rs`, optionally any downstream
  query consumer if Strategy B (reparent) is chosen, and the
  new / extended hand UI test bin under
  `tests/integration/hand-ui/` (or `tests/integration/hand_ui/`).

- [ ] **AC11 -- Cargo resource policy applied for every Cargo
  command**: future implementation MUST set the Cargo resource
  policy env vars (`CARGO_TARGET_DIR=
  D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before
  every `cargo check` / `cargo test` invocation on Windows /
  MSVC. Story authoring (PROMPT 1095) does NOT invoke Cargo.

---

## Implementation Notes

### Owned files (likely change set)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/hand/fan_root*.rs` (audit-time location; re-verify at activation HEAD) | Strategy A: no change. Strategy B: reparent FanRoot. |
| `client/src/ui/hand/hand_bar*.rs` (audit-time location) | Strategy A: insert `Transform` (auto-derives `GlobalTransform` via Required Components API in Bevy 0.18). Strategy B: no change. |
| Any downstream query consumer that walks `ChildOf` from HandBar (Strategy B only) | Update parent reference if FanRoot is reparented. |
| `tests/integration/hand-ui/hand_fan_root_globaltransform_test.rs` (NEW; or new assertions in an existing bin) | AC4 hierarchy invariant assertion. |
| `production/qa/evidence/sprint-17-hand-fan-root-b0004/evidence.md` (NEW, optional, by `/dev-story` worker) | Optional evidence document. |

### Forbidden files

- Everything under `server/`, `shared/`.
- Everything under `tests/integration/server/`,
  `tests/unit/server/`, `tests/integration/lightyear*`,
  `tests/unit/lightyear*`.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files.
- `docs/architecture/adr-*.md`.
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.
- The Sprint 12 story 019 file
  (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`)
  is NOT modified by this row. Disposition preserved verbatim.

### Cargo resource policy

Per the binding Sprint 15+ QA plan precedent, every `cargo`
invocation on Windows / MSVC MUST set the five env vars under
AC11.

### Target citations

- Sprint 17 plan row source:
  `production/sprints/sprint-17.md` §"Nice to Have" row
  `S17-UI-HAND-B0004-CLEANUP-001`.
- Source audit:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-14.
- Bevy migration reference:
  `docs/engine-reference/bevy/VERSION.md` (Required Components
  API, `Transform` auto-derives `GlobalTransform`).

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a
   fresh worktree (suggested slug
   `work/s17-hand-fan-root-b0004-hierarchy`).
2. Read this story file end-to-end before any code change.
3. Re-verify the audit-time hand UI module shape by reading
   `client/src/ui/hand/fan_root*.rs` and
   `client/src/ui/hand/hand_bar*.rs`. If
   `S16-TD-UI-HAND-MODSPLIT-001` (ui-clean-pass story 011) has
   landed by activation, the spawn sites may have moved.
4. Re-verify the Bevy 0.18 Required Components behaviour for
   `Transform` / `GlobalTransform` under `liv-bevy-018`. The
   API may have shifted since the Sprint 14 cutoff date.
5. Pick a strategy (A: give HandBar `GlobalTransform`; B:
   reparent FanRoot). Justify in the commit message. Default
   recommendation: Strategy A (insert `Transform` on HandBar);
   the Required Components API in Bevy 0.18 auto-derives
   `GlobalTransform` from `Transform`, so this is a single-
   component-insert change.
6. Activate `liv-bevy-018` skill before any `.rs` edit. Do NOT
   activate `liv-bevy-lightyear`.
7. Set the Cargo resource policy env vars per AC11 before every
   `cargo check` / `cargo test` invocation.
8. Run `cargo check -p client` and the targeted hand UI test
   bins under the Cargo resource policy; confirm zero new
   warnings on touched files AND zero regression in existing
   hand UI tests (AC3).
9. Push the worker branch (never `main`).
10. Stop. Closure paperwork is later prompts' scope.

The worker MUST NOT:

- Change fan layout, drag-state visuals, placement staging
  behaviour, or any other hand UI behaviour. This row is
  hierarchy hygiene only.
- Modify `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Modify Sprint 12 story 019 file or its disposition. The
  drag-runtime question remains `closed-with-conditions /
  cannot-reproduce`.
- Modify Cargo / Trunk / CI files.
- Modify `production/sprint-status.yaml`, `production/sprints/`,
  `production/stage.txt`, `production/session-state/`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation
  (targeted hand UI bins only).
- Run `trunk` or any CI command.
- Push to `main`.
- Claim closure of `S11-DRAG-RUNTIME-RETEST-001` (Sprint 11
  story 018), `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  (Sprint 12 story 019), `TQ-S12-C2`, or any AUDIT-1076-*
  finding outside AUDIT-1076-14.
- Claim release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, final-art
  completion, or stage advance.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to the files this
worker owns under `client/src/ui/hand/` plus the new test bin.
The worker MUST NOT block on workspace-wide compilation errors
introduced by other in-flight Sprint 17 workers' branches. Per
the Parallelism summary above, file-overlap with the helper
bundle and the qa_snapshot marker split rows requires
orchestrator-side serialisation.

### Relay / reporting expectation for future workers

Final status line:

```
N: S17-UI-HAND-B0004-CLEANUP-001: STATUS
```

where `N` is the prompt number that ran `/dev-story`.

---

## Closure Trail

Closure trail is appended by future `/story-readiness`,
`/dev-story`, and `/story-done` prompts. No closure trail is
authored by PROMPT 1095.

### Conditions carried forward unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim (including `TQ-S12-C2`
  "no third same-scope drag-runtime retest").
- Sprint 11 story 018 (`S11-DRAG-RUNTIME-RETEST-001`)
  `closed-with-conditions` preserved.
- Sprint 12 story 019 (`S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`)
  `closed-with-conditions / cannot-reproduce` preserved.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-
  operator-blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT
  closed by this row.

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Closure of any AUDIT-1076-* finding outside AUDIT-1076-14.
- Closure of any SOURCE-1077-* finding.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Closure of Sprint 11 story 018 or Sprint 12 story 019 retest
  questions. The drag-runtime question remains
  `closed-with-conditions / cannot-reproduce` per `TQ-S12-C2`.
- Sprint 17 close-out.
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.

`021: S17-UI-HAND-B0004-CLEANUP-001: DRAFT`
