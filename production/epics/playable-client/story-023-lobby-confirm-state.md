# Story 023: S11-LOBBY-UX-CONFIRM-STATE-001 -- Lobby "Confirming..." Text Differentiation

> **Epic**: Playable Client
> **Story ID**: S11-LOBBY-UX-CONFIRM-STATE-001
> **Status**: Draft -- Sprint 13 candidate (Should Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: Lobby UI / UX (Client)
> **Type**: Integration -- targeted lobby UI text edit + integration test
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; Sprint 11
> promotion-from-Nice-to-Have to batch with Sprint 12 Cluster B3 lobby
> work, which landed as story 013 at commit `d8d0196`); NOT activated
> **Authored**: 2026-05-14 by PROMPT 819
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 819. Sprint 12 is closed-with-conditions per PROMPT
817 and is not changed by this authoring run.

PROMPT 819 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**No client-side class-lock authority is introduced or proposed by this
story.** Class-lock state remains server-authoritative via the
`S2CClassLocked` reliable broadcast; this story differentiates the
client's "Confirming..." text presentation only. ADR-002 binding is
reinforced by Sprint 12 story 013's lobby ConfirmClass intent-chain fix
(landed at `d8d0196`), which preserved ADR-002 + ADR-008 + ADR-012.

---

## Source Finding

- Sprint 11 close-out raised this row as a UX gap: the lobby
  "Confirming..." text does not differentiate between (a) the local
  player's confirm has been acknowledged by the server but the
  opponent has not yet confirmed, and (b) the local player has not
  yet confirmed.
- Sprint 11 promoted it from Nice-to-Have to Sprint 12 Should Have
  to batch with the Cluster B3 lobby ConfirmClass intent-chain work
  (Sprint 12 story 013).
- Sprint 12 closed with story 013 done at `d8d0196` but
  `S11-LOBBY-UX-CONFIRM-STATE-001` deferred; PROMPT 817 carried it
  forward to Sprint 13 planning.

---

## Problem Class / Prevention Target

**Defect class**: Lobby UI shows a single "Confirming..." text for
two distinct states:

- State A -- "I have confirmed; waiting for opponent": the local
  player's `C2SConfirmClass` has been acknowledged by `S2CClassLocked`
  (own player); the opponent's `S2CClassLocked` has not yet arrived.
- State B -- "I have not yet confirmed": the local player has not
  yet sent `C2SConfirmClass`.

Symptom: the player cannot tell from the UI whether the system is
waiting on them or on the opponent.

**Prevention target**: Differentiate the two states with distinct
copy. Concrete copy variants are owned by the `ux-designer` agent
(implementation prompt MUST consult before locking on a final
wording); a placeholder pair such as:

- State A: "Waiting for opponent..."
- State B: "Confirm your class to continue"

is acceptable for friend-game scope. The two states map cleanly to
local + server-acked state machines without introducing client
authority.

---

## Context

### Existing surface

- **`client/src/ui/lobby.rs`** (or canonical equivalent): the lobby
  UI text source. The intent-chain bug previously here was fixed by
  Sprint 12 story 013 at `d8d0196`; this story sits **after** that
  fix.
- **`shared/src/protocol.rs`**: `C2SConfirmClass`, `S2CClassLocked`,
  `S2CSessionReady`. **Not modified** by this story.
- **Lobby state**: client-side lobby state machine reads
  `S2CClassLocked` for both own player and opponent.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/game-session-system.md` (lobby flow);
  `design/gdd/hand-ui.md` (UX text owned by ux-designer).
- **ADR-002** (Client-Server Authority): no client-side class-lock
  authority added.
- **ADR-008** (Lightyear Channel Configuration): no channel change.
- **ADR-012** (SessionReady Delivery): no change to the SessionReady
  Observer.
- **TR registry**: no new TR (UX text differentiation only).

### UX surface

The two text variants are owned by `ux-designer` + `ui-programmer`.
The implementing prompt MUST consult ux-designer before locking on
final copy. Friend-game-tier placeholder text is acceptable per
`PAW-TD-*-a` accept-risk preserved.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` (any `.rs` edit in the lobby
  UI module).

### Control Manifest Rules

- Required: Two distinct text variants render in the two distinct
  lobby states.
- Required: At least one integration test asserts the text
  differentiation across the two states.
- Required: No client-side class-lock authority introduced.
- Required: ux-designer consultation note recorded in evidence.
- Forbidden: Synthesising a fake `S2CClassLocked` on the client to
  short-circuit the lobby state machine.
- Forbidden: Modifying the canonical lobby state machine to allow
  client-side state changes without server messages.
- Forbidden: Server-side change.

---

## Story Classification

**Story type**: Integration -- targeted lobby UI text edit +
integration test.

This is **NOT** a:

- Pure UX-spec story (real client UI text lands).
- Server-side change.
- Protocol change.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- ux-designer consultation recorded**: GIVEN the
  implementation prompt's first ux-designer interaction, WHEN the
  final text variants are chosen, THEN the consultation note and
  chosen copy are recorded in the evidence document.

- [ ] **AC2 -- Two distinct text variants render**: GIVEN the lobby
  UI at runtime, WHEN the local lobby state is "waiting for
  opponent's confirm" (State A), THEN the text reads the State A
  variant. WHEN the local lobby state is "local player has not
  confirmed yet" (State B), THEN the text reads the State B variant.

- [ ] **AC3 -- Integration test asserts text differentiation**:
  GIVEN a new or extended integration test (e.g.,
  `tests/integration/playable_client/lobby_confirm_state_text_test.rs`),
  WHEN the test drives both lobby states, THEN it asserts the
  rendered text matches the State A variant in State A and the
  State B variant in State B.

- [ ] **AC4 -- No client-side class-lock authority added**: GIVEN
  the implementation diff, WHEN reviewed, THEN no client-side
  mutation of class-lock state outside `S2CClassLocked` /
  `S2CSessionReady` drains is present. ADR-002 binding (reinforced
  by Sprint 12 story 013 fallback path).

- [ ] **AC5 -- No protocol or server-side change**: GIVEN the diff
  in `shared/src/protocol.rs` and `server/`, WHEN inspected, THEN
  no functional change lands.

- [ ] **AC6 -- Sprint 12 story 013 fallback preserved**: GIVEN the
  Sprint 12 story 013 fallback path (where duplicate same-class
  confirm returns `S2CClassLocked` re-ack), WHEN the lobby state
  reaches the duplicate-confirm condition, THEN the new text
  differentiation still renders correctly (i.e., the re-ack lands
  the local player in State A, displaying the State A variant).

- [ ] **AC7 -- Workspace test pass**: GIVEN `cargo test --workspace
  --tests --no-fail-fast` at the implementation commit, WHEN
  compared to the post-Sprint-12 baseline, THEN no new `#[ignore]`
  markers are introduced; the new test passes; previously-passing
  tests continue to pass.

- [ ] **AC8 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them are
  modified by this story.

- [ ] **AC9 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-lobby-confirm-state-evidence.md`
  (NEW). Records the diff summary, the chosen text variants, the
  ux-designer consultation note, the integration-test pass output,
  no-claim restatement (including "no client-side class-lock
  authority added"), and a cross-link to Sprint 12 story 013.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/lobby.rs` (canonical path verified by implementing worker) | Edited to render two distinct text variants based on lobby state. |
| `tests/integration/playable_client/lobby_confirm_state_text_test.rs` | NEW integration test asserting AC3. |
| `production/qa/evidence/sprint-13-lobby-confirm-state-evidence.md` | NEW evidence document per AC9. |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for `.rs` edits.
- **`liv-bevy-lightyear`** -- mandatory only if the touched lobby
  code imports `lightyear` directly (it likely does for the
  `S2CClassLocked` consumer wired by Sprint 12 story 013).

---

## Evidence Path

`production/qa/evidence/sprint-13-lobby-confirm-state-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- Diff summary for `client/src/ui/lobby.rs` edits.
- Chosen final text variants (State A + State B).
- ux-designer consultation note (link or in-document rationale).
- New integration-test pass output.
- Confirmation that Sprint 12 story 013 duplicate-confirm fallback
  path still works under the new text (AC6).
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no client-side class-lock authority added").
- Cross-link to Sprint 12 story 013 (`d8d0196`).
- Cross-link to Sprint 12 close-out deferral row.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test lobby_confirm_state_text -- --nocapture`
  (or the new test name)
- `git diff <pre-impl-sha>..<impl-sha> -- 'shared/src/**' 'server/src/**'`
  (verifies AC5: zero protocol-shape change; zero server-side
  change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- Server-side class-lock change.
- Adding a third lobby state.
- Changing the lobby flow for new joins, rejoins, or class swaps
  (outside the existing Sprint 12 story 013 fallback path).
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- Standard-tier accessibility of the lobby UI (friend-game scope
  only).
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Touches `client/src/ui/lobby.rs`. Sprint 13 Should Have row
  `S13-LATE-MSG-DEDUPE-001` (story 020) historically had file-scope
  conflict on the same file with Sprint 12 story 013; both have
  closed/landed. This story sequences cleanly after Sprint 13 row
  `S13-LATE-MSG-DEDUPE-001` to avoid a same-file double-edit collision
  if both are activated in the same sprint.
- Sequences after Sprint 13 Must Have row
  `S13-OBS-TRACING-TARGETS-001` (story 018) if both touch lobby
  emission sites.
