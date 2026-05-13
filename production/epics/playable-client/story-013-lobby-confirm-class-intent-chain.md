# Story 013: Lobby ConfirmClass Intent Chain -- Production Fix

> **Epic**: Playable Client
> **Story ID**: S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001
> **Status**: Draft -- Sprint 12 draft Must Have (Cluster B3); NOT activated
> **Layer**: Friend-Game Lobby Input -- Production Repair
> **Type**: Integration (production code change in lobby input system + test
> un-`#[ignore]`)
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 12 (draft per PROMPT 793 at `origin/main@8a8451e`; NOT yet activated)
> **Authored**: 2026-05-14 by PROMPT 795 (producer + qa-lead, worktree `work/sprint-12-must-story-authoring`)
> **Authoring source-of-truth**: `origin/main@f72cc60` (PROMPT 793 Sprint 12 draft plan + PROMPT 794 story-019 slug correction).

---

## Status / No-Claim Banner

This story is authored as a Sprint 12 draft Must Have. Sprint 12 is **NOT
activated**; activation happens via `/sprint-plan sprint-12` in a separate
prompt. PROMPT 795 (this authoring run) does NOT:

- Activate Sprint 12.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md`.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any session-state file.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify the ignored test
  (`tests/integration/playable_client/native_operator_controls_test.rs:106`).
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 disposition (`closed-with-conditions`) and Sprint 11 disposition
(`closed-with-conditions` per PROMPT 792) remain unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**No optimistic client-side class-lock authority is introduced or proposed
by this story or by any disposition pathway recorded in "Acceptance
Criteria"**. ADR-002 binding: the production fix emits `ConfirmClass` as
an intent that travels through the lobby C2S/S2C protocol, NOT a
client-side state mutation.

---

## Context

Sprint 11 D-5 triage evidence
(`production/qa/evidence/sprint-11-ignored-d5-triage.md`, Cluster B3, row 85)
retained the test
`test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
at `tests/integration/playable_client/native_operator_controls_test.rs:106`
with the PROMPT 750 D-5 owner comment:

> `#[ignore = "PROMPT 750 D-5 follow-on: ConfirmClass intent not emitted
> alongside SelectClass -- input chain stops at SelectClass; needs lobby
> input system investigation (revealed after D-1 fix)"]`

After the D-1 fix landed (PROMPT 750 era), the lobby input chain emits
`SelectClass` correctly when the operator clicks a class button, but the
follow-on `ConfirmClass` intent that the integration test asserts is **not
emitted**. The investigation note recorded in the D-5 owner comment is
explicit: "input chain stops at `SelectClass`".

This is a **production-fix story**, not a test-relocation or
fixture-cleanup. The cluster B3 disposition per the triage doc is
`needs-repair-story` (production fix), not `needs-design-decision`. The
binary decision space for this story is much narrower:

- **Primary path (production fix)**: locate the lobby input system that
  consumes the `SelectClass` intent and add the production-driven
  emission of the `ConfirmClass` intent (either same-tick via a chained
  intent-write, or via the production event chain that the existing
  `SessionReady` flow uses).
- **Fallback path (test redesign + production gate)**: only if
  diagnosis surfaces that `ConfirmClass` is not the right next-intent
  shape after the D-1 fix (e.g., a deliberate UX change made the
  two-intent chain a one-intent flow); in that case, the test is
  rewritten to assert the correct post-D-1 lobby behaviour and a
  production-design write-up is recorded.

The D-5 owner-comment language ("input chain stops at `SelectClass`")
strongly suggests the primary path is correct -- the D-1 fix landed
without rewiring the next-intent emission, and the test simply caught
the regression. The implementation prompt MUST verify this before
landing the production change (no client-side optimism allowed in the
fallback either).

**Primary sources**:

- `production/qa/evidence/sprint-11-ignored-d5-triage.md` (Cluster B3, row 85)
- `production/sprints/sprint-12.md` (Sprint 12 draft Must Have row
  `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`, line 125)
- `tests/integration/playable_client/native_operator_controls_test.rs:106`
  (the test in question, with PROMPT 750 D-5 owner comment preserved on
  `origin/main@f72cc60`)
- ADR-002 / ADR-012 governing the C2S/S2C class-lock + session-ready
  flow (binding for the production fix shape).

**GDD, UX, and TR trace**:

- `design/gdd/game-session-system.md` -- TR-GSS-001, TR-GSS-004,
  TR-GSS-007 cover the create / join / class-lock / session-ready
  intent chain. The `SelectClass` -> `ConfirmClass` -> `SessionReady`
  flow is the canonical shape governed by these TRs.
- `design/ux/lobby.md` (if present) -- the two-intent class confirm
  flow is the spec; deviation requires a UX design write-up.
- No new TR is added by this story. The repair restores existing
  TR-GSS-004 coverage that regressed silently after the D-1 fix.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
  -- the production fix emits `ConfirmClass` as an intent that
  travels through C2S to the server; the server is authoritative on
  class-lock state and emits `S2CClassConfirmed` (or equivalent) back.
  **No client-side optimistic class-lock state mutation.**
- [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md)
  -- the `SessionReady` Observer enters the round loop on the same
  frame after both clients' confirm intents are acknowledged.
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
  -- the `ConfirmClass` C2S message rides the reliable channel
  (game-state / control class), not the unreliable channel.

**Engine**: Bevy 0.18 (Rust) + Lightyear 0.26 | **Risk**: MEDIUM
(production code change in lobby input system; touches the C2S intent
chain; small surface area but the change is gameplay-visible)

**Engine Notes**: Bevy 0.18 + Lightyear 0.26 lobby input pattern: the
lobby UI emits `Pointer<Press>` -> intent enum (a Bevy event or
message) -> intent-consumer system that writes the C2S Lightyear
message via `MessageWriter<C2S*>`. The intent-emission system fires on
the UI button click; the intent-consumer system fires on the next
schedule tick. Verify both phases for the `ConfirmClass` step -- the
D-1 fix may have rewired one but not the other.

**Mandatory skills**:
- `liv-bevy-018` -- any read/review/edit of Bevy `.rs` code touched.
- `liv-bevy-lightyear` -- the production fix touches Lightyear C2S
  message emission for the lobby class-lock flow.

**Control Manifest Rules (2026-05-05)**:
- Required: `ConfirmClass` intent must travel through the canonical
  C2S Lightyear path (reliable channel).
- Required: Server is authoritative on class-lock state; the
  `S2CClassConfirmed` (or equivalent) response is the only legal
  trigger for client-side class-lock UI state changes.
- Forbidden: Client-side optimistic class-lock mutation. The class
  button visual state may reflect the *intent in flight* (e.g., a
  "Confirming..." text), but the *class-lock state* itself is server-
  authoritative.
- Forbidden: Replacing `ConfirmClass` with a one-intent shortcut
  unless a UX design write-up explicitly approves the protocol
  change.

---

## Story Classification

**Story type**: Production repair (lobby input system) + test
un-`#[ignore]` after repair lands.

This is **NOT** a:

- Fixture-cleanup story (the test fixture is correct; the production
  code is missing the next-intent emission).
- Decision-only story (the decision space is narrow; the production
  fix is the primary path per Cluster B3 disposition language).
- Evidence-only story (an executable production code change is the
  primary artefact).

---

## Scope

### In Scope

- **Diagnosis**: locate the lobby input system that consumes
  `SelectClass` after the D-1 fix and identify where the
  `ConfirmClass` intent emission should occur. Record findings in the
  evidence document.
- **Production fix**: emit the `ConfirmClass` intent at the correct
  point in the lobby input chain. The fix is narrowly scoped to the
  lobby input system (`client/src/ui/lobby/` or equivalent module --
  verified at implementation time). The two acceptable shapes are:
  - **Same-tick chained intent**: the `SelectClass` consumer writes
    both the `SelectClass` C2S message AND the `ConfirmClass` C2S
    message in the same tick when the operator's click on the class
    button is the canonical "confirm" gesture (verify against UX
    spec).
  - **Separate confirm-button intent**: the `ConfirmClass` intent
    fires on a separate UI gesture (e.g., explicit Confirm button
    click) after `SelectClass` lands. If diagnosis surfaces that the
    lobby UI already has a separate confirm button whose input wire
    is broken, the fix is to repair that wire.
- **Test un-`#[ignore]`d** under whichever production shape is
  chosen. The test
  `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
  asserts the two-intent chain end-to-end -- after the production fix
  lands, the test should pass under its existing assertion shape.
- **Original PROMPT 750 D-5 owner comment removed** only after the
  test passes.
- **Evidence document slot reserved** at
  `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md`
  (NEW; populated by the implementation prompt).

### Out of Scope

- **No client-side optimistic class-lock authority**. ADR-002 binding.
  The fix MUST route `ConfirmClass` through the C2S/S2C protocol;
  client-side class-lock visual state changes only after
  `S2CClassConfirmed` (or equivalent) is observed.
- **No protocol shape change**. The `C2SConfirmClass` message shape
  must already exist in `shared/src/protocol.rs` (verify at
  `/story-readiness` time). If the protocol message does not yet
  exist, that is a separate follow-on story and this story is
  blocked.
- No expansion to other Cluster B residuals (B1/B2/B4/B5) -- each is
  scoped to its own Sprint 12 Must Have story.
- No expansion to broader lobby UX rework. The Sprint 12 Should Have
  row `S11-LOBBY-UX-CONFIRM-STATE-001` (lobby "Confirming..." text
  differentiation) is a separate story; this story is the *intent
  chain* repair, not the UI text differentiation.
- No Sprint 12 activation. No `production/stage.txt` modification. No
  `production/sprint-status.yaml` modification. No
  `production/sprints/sprint-12.md` modification under this story
  authoring prompt.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, QA sign-off, or close-out under this story authoring
  prompt.
- No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or any
  carried Sprint 11 / Sprint 10 condition.
- No claim of public release readiness, release-candidate readiness,
  full playable-client manual QA, full game completion, broad
  Standard-tier accessibility completion, playtest / fun-hypothesis
  validation, or final-art / asset-production completion.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-12.md:125` Sprint 12 draft Must Have
row. ACs below are draft and become binding at Sprint 12 activation.)

- [ ] **AC1 -- Investigation note recorded**: GIVEN the implementation
      commit, WHEN the evidence document is read, THEN the lobby
      input chain post-D-1 is described: which system consumes
      `SelectClass`, why `ConfirmClass` is not emitted today, what
      the intended emission point is, and (if applicable) the UX spec
      shape that the fix conforms to.

- [ ] **AC2 -- Production fix lands in lobby input system**: GIVEN
      the implementation commit, WHEN the production diff is filtered
      to `client/src/` (lobby input module), THEN the production
      change emits the `ConfirmClass` C2S intent through the
      canonical Lightyear path. Diff is scoped to the lobby input
      module and its directly coupled types; no broader rework.

- [ ] **AC3 -- Test un-`#[ignore]`d and passes**: GIVEN the
      implementation commit, WHEN
      `cargo test -p client --test native_operator_controls` (or the
      equivalent `cargo test` invocation) is run, THEN
      `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
      passes without `#[ignore]` tagging. Pre/post pass count
      recorded in the evidence document.

- [ ] **AC4 -- Integration test asserts the two-intent chain
      end-to-end**: GIVEN the implementation commit, WHEN the test
      assertion set is reviewed, THEN the test verifies that both
      `C2SSelectClass` AND `C2SConfirmClass` (or equivalent canonical
      names; verify against `shared/src/protocol.rs`) are written by
      the lobby button click sequence. The test does NOT assert any
      client-side class-lock state change before
      `S2CClassConfirmed` is observed.

- [ ] **AC5 -- No client-side optimistic class-lock authority
      introduced**: GIVEN the implementation commit, WHEN the
      production diff is reviewed, THEN no client-side state mutation
      writes the class-lock state before `S2CClassConfirmed` (or
      equivalent) is observed. ADR-002 binding. *Evidence*: text
      search for the phrase "no optimistic" in the evidence document
      and absence of client-side class-lock mutation outside the
      `S2CClassConfirmed` consumer.

- [ ] **AC6 -- Workspace ignored count drops by 1**: GIVEN Sprint 11
      close-out baseline of 5 retained Cluster B `#[ignore]` tests on
      `origin/main`, WHEN
      `cargo test --workspace --tests --no-fail-fast` is run at the
      implementation commit, THEN the workspace ignored count drops
      by 1 (relative to the baseline) and no new undocumented
      `#[ignore]` marker is introduced.

- [ ] **AC7 -- Original PROMPT 750 D-5 owner comment removed only
      after the test passes**: GIVEN the implementation commit, WHEN
      `tests/integration/playable_client/native_operator_controls_test.rs`
      is read, THEN no PROMPT 750 D-5 owner comment for this test
      remains.

- [ ] **AC8 -- Sprint 12 disposition preserved**: GIVEN the
      implementation commit, WHEN `production/sprint-status.yaml`,
      `production/sprints/sprint-12.md`, and `production/stage.txt`
      are diffed, THEN none of them are modified under this story.
      Sprint 12 activation disposition is preserved. Stage remains
      `Polish`. Sprint 11 disposition (`closed-with-conditions`) is
      unchanged.

- [ ] **AC9 -- Evidence document slot reserved**: GIVEN this story
      file, WHEN the evidence-doc path is checked, THEN a slot is
      reserved at
      `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md`
      for population by the implementation prompt. Authoring of the
      evidence file itself is deferred to the implementation prompt.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. `/sprint-plan sprint-12` activates Sprint 12 (separate prompt).
2. This story passes `/story-readiness` (separate prompt).
3. Sprint 12 `/qa-plan sprint` is authored (separate prompt).
4. `/dev-story story-013-lobby-confirm-class-intent-chain.md` is
   dispatched (separate prompt).

Expected implementation flow:

1. **Wave 1 -- Diagnosis**: the implementation prompt reads the lobby
   input module, traces `SelectClass` consumption post-D-1 fix, and
   identifies the exact production-code site where `ConfirmClass`
   emission is missing. Records findings in the evidence document.
2. **Wave 2 -- Protocol verification**: verifies that the
   `C2SConfirmClass` (or canonically named) message shape exists in
   `shared/src/protocol.rs`. If it does not, the story is blocked
   and a separate follow-on story is authored to add the protocol
   shape.
3. **Wave 3 -- Production fix**: lands the narrowest possible diff
   that emits the `ConfirmClass` intent through the canonical C2S
   Lightyear path. No client-side optimistic mutation.
4. **Wave 4 -- Test un-`#[ignore]` + pass**: runs the integration
   test; verifies the two-intent chain end-to-end. Removes the
   PROMPT 750 D-5 owner comment.
5. **Wave 5 -- Evidence doc**: populates
   `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md`
   with the investigation note, production diff summary, pre/post
   pass counts, no-claim restatement, and cross-link to Cluster B3
   row 85 in the triage doc.

The diagnosis wave MUST distinguish between two failure shapes:

- **Intent-chain wire gap** (expected shape): the lobby input system
  consumes `SelectClass` correctly but never emits `ConfirmClass`.
  Fix: add the emission at the correct point.
- **UX-design intentional change**: a deliberate post-D-1 UX change
  made the two-intent chain a one-intent flow. Fix: rewrite the
  test to assert the new correct behaviour AND record a UX design
  write-up. (Lower probability per the D-5 owner-comment language.)

---

## Performance Budget

N/A -- lobby input system fix is one frame of UI interaction; no
hot-path code changed.

---

## QA Test Cases

(Draft -- becomes binding at Sprint 12 activation. Sprint 12 QA plan
authored via `/qa-plan sprint` will pull from this set.)

- **Two-intent chain end-to-end**
  - Given: implementation commit set on `main` for this story.
  - When: the operator click sequence (create room, join room, select
    slot, click class button) is exercised via
    `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`.
  - Then: both `C2SSelectClass` AND `C2SConfirmClass` are written by
    the lobby input system in the canonical Lightyear flow. No
    client-side class-lock state mutation occurs before
    `S2CClassConfirmed` is observed.

- **Workspace ignored-count regression check**
  - Given: Sprint 11 close-out baseline (1129 passing / 5 ignored on
    `origin/main@8a8451e`).
  - When: `cargo test --workspace --tests --no-fail-fast` is run at
    the implementation commit.
  - Then: workspace ignored count is at most `5 - 1 = 4` and no new
    undocumented `#[ignore]` marker is introduced.

- **Production diff scope audit**
  - Given: the union diff of every commit in this story's trail.
  - When: paths under `client/src/` are filtered.
  - Then: the diff is scoped to the lobby input module and its
    directly coupled types. No broader rework lands.

- **No-optimism audit**
  - Given: the production diff.
  - When: the diff is reviewed for any client-side class-lock state
    mutation outside the `S2CClassConfirmed` consumer.
  - Then: no such mutation is present.

- **Sprint 12 disposition preservation audit**
  - Given: the implementation commit.
  - When: `production/sprint-status.yaml`,
    `production/sprints/sprint-12.md`, and `production/stage.txt` are
    diffed.
  - Then: none of them are modified under this story id.

---

## Test Evidence

**Story Type**: Integration (production code change + test
un-`#[ignore]`)

**Evidence path**: `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Investigation note (lobby input chain post-D-1; missing emission
  point; production-fix shape chosen).
- Production diff summary (file paths + line ranges; one-paragraph
  description of the fix).
- Pre/post `cargo test -p client` pass + ignored counts.
- Pre/post `cargo test --workspace --tests --no-fail-fast` pass +
  ignored counts.
- No-claim restatement (verbatim from this story file's "Status /
  No-Claim Banner" section), including the explicit "no optimistic
  client-side authority" line.
- Cross-link back to this story file and to Cluster B3 row 85 in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`.

**Required verification commands** (for the implementation prompt):

- `cargo test -p client --no-fail-fast`
- `cargo test --workspace --tests --no-fail-fast`
- `git diff <pre-impl-sha>..<impl-sha> -- 'client/src/**'` (scoped to
  lobby input module)
- `git diff --check` and `git diff --cached --check` before commit

**Status**: [ ] Captured and locked

---

## Owner / Classification

- **Owner**: client gameplay programmer (lobby input system) +
  ux-designer (intent chain spec verification) -- per Cluster B3
  row 85 in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`.
- **Estimated days**: 1.00 (per Sprint 12 draft row).
- **Story classification**: production repair + test un-`#[ignore]`.

## Dependencies

- **Depends on**: Sprint 12 activation via `/sprint-plan sprint-12`
  (separate prompt). This story remains `Draft` until then.
- **Depends on**: Sprint 11 D-5 triage doc
  (`production/qa/evidence/sprint-11-ignored-d5-triage.md`, Cluster B3,
  row 85) for owner / disposition / decision-gate language.
- **Depends on**: `C2SConfirmClass` (or canonically named) message
  shape existing in `shared/src/protocol.rs`. Verify at
  `/story-readiness` time; if absent, this story is blocked and a
  separate protocol-additions story is authored.
- **Depends on**: Sprint 12 QA plan authored via `/qa-plan sprint`
  (separate prompt) before `/dev-story` runs.
- **Coordinated with**: `S11-LOBBY-UX-CONFIRM-STATE-001` (Sprint 12
  Should Have promoted from Sprint 11 Nice to Have -- "Confirming..."
  text differentiation; shares lobby UX review surface, batched to
  reduce re-review cost).
- **Coordinated with**: `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`
  (Story 012 -- Cluster B2 HUD bridge fixture; no shared file scope).
- **Not coordinated with**: `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  (story 019 in hand-ui epic -- separate runtime-trace story; no
  shared file scope).

## Readiness Notes

**Implementation readiness verdict**: Draft -- substantive work has not
started. The live downstream gates are Sprint 12 activation, Sprint 12
QA plan authorship, and `/story-readiness` PASS on this file.

Pre-conditions for `/story-readiness` PASS:

- Sprint 12 is activated (`sprint:` field in
  `production/sprint-status.yaml` bumped + active row written) --
  **Pending separate `/sprint-plan sprint-12` prompt.**
- Sprint 12 QA plan exists at `production/qa/qa-plan-sprint-12.md` --
  **Pending separate `/qa-plan sprint` prompt.**
- `C2SConfirmClass` (or canonically named) message shape exists in
  `shared/src/protocol.rs` -- **Verify at `/story-readiness` time.**
- This story file is referenced from Sprint 12's active row in
  `production/sprint-status.yaml` after activation -- **Pending
  separate `/sprint-plan sprint-12` prompt.**

Open questions to resolve at `/story-readiness` time:

- Is the lobby UI's class-button click the canonical "select +
  confirm" gesture (same-tick chained intent), or is there a separate
  Confirm button after class selection (separate-tick chained
  intent)? Diagnosis at implementation time decides; the production
  fix shape conforms.
- Is the UX spec language in `design/ux/lobby.md` (or equivalent)
  current after the D-1 fix? If not, ux-designer review is required
  before the production fix lands.

---

## Files Anticipated To Be Modified (planning estimate, NOT binding)

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/lobby/*.rs` (path TBD by implementation; verify against current lobby module layout) | production fix: emit `ConfirmClass` C2S intent at the correct point in the lobby input chain. Diff scoped to the lobby input module. |
| `tests/integration/playable_client/native_operator_controls_test.rs` | un-`#[ignore]` `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`; remove PROMPT 750 D-5 owner comment after test passes. |
| `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md` (NEW) | evidence document per AC9 |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Diagnosis surfaces a UX-design intentional change rather than a missing emission | Low | Medium | Diagnosis wave records the UX spec shape; if intentional change is the disposition, story scope flips to test rewrite + UX write-up; no client-side optimism introduced under either disposition. |
| `C2SConfirmClass` protocol message does not exist | Low | High | `/story-readiness` open question forces verification; if absent, this story is blocked and a separate protocol-additions story is authored. |
| Production fix accidentally introduces client-side optimistic class-lock state | Medium | High | AC5 hard constraint + ADR-002 reviewer check + evidence-doc text-search for "no optimistic" phrase. |
| Production fix expands beyond the lobby input module into broader rework | Low | Medium | AC2 + AC4 + scope-cap language ("diff scoped to the lobby input module"). |
| Implementation prompt bundles a UI text change into this story (overlap with `S11-LOBBY-UX-CONFIRM-STATE-001`) | Medium | Medium | Out-of-Scope language explicit; UI text differentiation is the separate Should Have story. |
| Sprint 12 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate; this story stays `Draft` until activation. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator emitting the implementation
prompt, not for the worker:

- `production/sprint-status.yaml` `sprint:` field reads `sprint 12` and
  this story is referenced from the active row, OR the row is held with
  a written blocker.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-12.md` status block reads `active` (after
  separate `/sprint-plan sprint-12` activation prompt).
- The PROMPT 761 Polish->Release gate-check FAIL evidence at
  `production/gate-checks/gate-polish-release-2026-05-12.md` is
  preserved.
- `C2SConfirmClass` (or canonically named) message shape exists in
  `shared/src/protocol.rs`.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 795 -- Story file authored as a Sprint 12 draft
  Must Have for Cluster B3. Sprint 12 is `draft` (PROMPT 793) and not
  yet activated -- this story is **not yet activated** into the
  Sprint 12 active scope. Activation is a separate prompt
  (`/sprint-plan sprint-12`). No code changes, no smoke / gate / QA /
  `/dev-story` / `/story-done` / `/story-readiness` / `/qa-plan` run.
  Source-of-truth at authoring: `origin/main@f72cc60`.
