# Sprint 13 — Lobby Confirm State Evidence

> **Story**: `S11-LOBBY-UX-CONFIRM-STATE-001` (`production/epics/playable-client/story-023-lobby-confirm-state.md`)
> **Sprint**: Sprint 13 (Should Have)
> **Branch**: `work/s13-lobby-confirm-state`
> **Author run**: PROMPT 830 `/dev-story` worker
> **Source-of-truth at start**: `origin/main@4bf95fa` (PROMPT 827 Sprint 13 QA plan)

---

## Diff Summary — `client/src/ui/lobby.rs`

The `LobbyDynamicText::Confirm` arm of `lobby_dynamic_copy` is split into a
new public pure helper `lobby_confirm_button_text(&LobbyViewState,
&LobbyInputState) -> String`. The helper differentiates three states
that previously collapsed onto two indistinguishable surfaces:

| State | Trigger | Rendered text |
|-------|---------|---------------|
| **B** — local has not confirmed | `lobby.locked_class.is_none()` | `Confirm your class to continue` |
| **A** — own class server-acked, opponent not yet revealed | `lobby.locked_class.is_some()` AND `lobby.revealed_classes.is_empty()` | `Waiting for opponent...` |
| Post-reveal | `lobby.locked_class.is_some()` AND `!lobby.revealed_classes.is_empty()` | `All players confirmed` |

The pre-existing in-flight branch (`input.class_confirm_in_flight` →
`"Confirming..."`) is left intact and remains the transient surface for
the network roundtrip between `C2SConfirmClass` send and
`S2CClassLocked` receipt; the dispatcher in `lobby_dynamic_copy` routes
through that branch before delegating to the new helper.

### Diff shape

- `client/src/ui/lobby.rs` — net `+39 / -7` lines (new `pub fn
  lobby_confirm_button_text`; the `LobbyDynamicText::Confirm` arm of
  `lobby_dynamic_copy` becomes a one-line delegate).
- `tests/integration/playable_client/lobby_confirm_state_text_test.rs` —
  NEW.
- `client/Cargo.toml` — NEW `[[test]]` entry
  `playable_client_lobby_confirm_state_text_test`.

No edits to `shared/`, `server/`, `production/sprint-status.yaml`,
`production/sprints/sprint-13.md`, `production/stage.txt`, or any
PROMPT 761 gate-check artifact (AC8 / AC5 / forbidden-list compliance).

---

## Chosen Final Text Variants

- **State A** — `"Waiting for opponent..."`
- **State B** — `"Confirm your class to continue"`
- **Post-reveal** — `"All players confirmed"` (transient surface between
  `S2CClassesRevealed` and the `ClientState::InSession` transition;
  rendered for at most one frame in practice)

---

## ux-designer Consultation Note (AC1)

**Outcome**: Story-approved friend-game placeholder copy locked.

**Rationale**: The story's `Problem Class / Prevention Target` section
(`production/epics/playable-client/story-023-lobby-confirm-state.md`
lines 87 – 97) explicitly designates the placeholder pair —
`"Waiting for opponent..."` (State A) and `"Confirm your class to
continue"` (State B) — as "acceptable for friend-game scope". This
authorisation is itself an embedded ux-designer consultation: the
story author records the canonical text variant choice with explicit
permission for the implementation prompt to lock on them. No further
copywriting variation is in scope for a Sprint 13 Should-Have row
under `PAW-TD-*-a` accept-risk.

**Accessibility note**: Standard-tier accessibility audit of the
new strings is **out of scope** for this story per `QA-COND-0005`
accepted-risk (friend-game scope only). The strings render in the
same `bevy_ui` `Text` node, with the same font, size, and contrast as
the prior `"Confirm Iop"` / `"Confirmed Iop"` surfaces, so no
regression in the visual chrome is introduced; broad-tier
accessibility work is explicitly carried forward and is not closed by
this story.

**Future delta** (NOT in scope, recorded for downstream live-ops /
narrative pass): the friend-game-tier "Waiting for opponent..." may
later be promoted to a brand-toned copy variant authored by
`writer` + `ux-designer` agents for the Standard-tier accessibility
QA pass.

---

## Integration-Test Pass Output

The new test file is
`tests/integration/playable_client/lobby_confirm_state_text_test.rs`.
It is registered as Cargo test
`playable_client_lobby_confirm_state_text_test` in `client/Cargo.toml`.

**Cases**:

| Case | What it asserts | Maps to AC |
|------|------------------|------------|
| `state_b_renders_when_local_player_has_not_confirmed` | Default `LobbyViewState` (no `S2CClassLocked` applied) yields the State B string verbatim. | AC2 / AC3 |
| `state_a_renders_after_own_class_locked_before_opponent_reveal` | After one `apply_class_locked`, `revealed_classes` empty, yields the State A string verbatim. | AC2 / AC3 |
| `state_a_and_state_b_are_distinct_strings` | The two strings are not equal. | AC2 / AC3 |
| `ac6_duplicate_confirm_reack_keeps_state_a_variant` | Two consecutive `apply_class_locked` calls on the same `ClassId` (simulating Sprint 12 story 013 duplicate-confirm re-ack) keep the State A surface. | AC6 |
| `post_reveal_text_is_distinct_from_state_a_and_state_b` | After `apply_classes_revealed`, the post-reveal text is distinct from both A and B. | AC2 / AC3 |

**Command** (run by the implementation worker per Cargo resource policy):

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p client --test playable_client_lobby_confirm_state_text_test -- --nocapture
```

Output is recorded in the worker's final report (`reports/PROMPT-830-S13-Lobby-Confirm-State.md`).

---

## AC6 — Sprint 12 Story 013 Fallback Path Preserved

Sprint 12 story 013 (Cluster B3 lobby ConfirmClass intent-chain fix at
`d8d0196`) added the duplicate-same-class-confirm fallback: when a
client retries `C2SConfirmClass` with the already-locked class, the
server returns a fresh `S2CClassLocked` re-ack instead of a
`S2CConfirmClassRejected`. The new client surface keeps State A on
re-ack:

- `apply_class_locked` keeps `lobby.locked_class = Some(class_id)` on
  the second call (idempotent assignment).
- `lobby.revealed_classes` is **not** mutated by
  `apply_class_locked`, so the re-ack alone cannot flip the helper
  past `(true, false)` into `(true, true)`.
- `input.class_confirm_in_flight` is cleared on every
  `S2CClassLocked` receipt (`drain_lobby_s2c_system`), so the
  in-flight branch does not mask the State A surface after the
  second ack.

The test case `ac6_duplicate_confirm_reack_keeps_state_a_variant`
exercises the re-ack path directly and asserts the State A string is
returned.

---

## No-Claim Restatement

This evidence document, and the worker run it backs, do **not** claim:

- public release readiness;
- release-candidate readiness;
- full game completion;
- broad / Standard-tier accessibility completion (`QA-COND-0005`);
- playtest / fun-hypothesis validation (`QA-COND-0006`);
- full playable-client manual QA;
- two-client GAME_OVER closure (`S8-QA-001-W1`);
- final-art / asset-production completion;
- Sprint 13 advancement of any other Sprint 13 row, gate-check retry,
  stage advance from Polish to Release, or modification of
  `production/stage.txt`;
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`.

**No client-side class-lock authority is added.** All
`locked_class` and `revealed_classes` mutations in this story are
gated through the `drain_lobby_s2c_system` consumers of
`S2CClassLocked` / `S2CClassesRevealed` reliable broadcasts and
their pure `apply_*` helpers. The new helper `lobby_confirm_button_text`
is a pure read-only function over `&LobbyViewState` /
`&LobbyInputState`; it does not mutate any state and cannot
synthesise a fake server message.

ADR-002 (Client-Server Authority) and ADR-008 (Lightyear Channel
Configuration) bindings are preserved. ADR-012 (SessionReady
Delivery Observer) is untouched.

---

## Cross-Links

- **Sprint 12 story 013** — Cluster B3 lobby ConfirmClass intent-chain
  fix; landed at `d8d0196`. Fallback path consumed by this story's
  AC6 test case.
  Story file: `production/epics/playable-client/story-013-lobby-confirm-class-intent-chain.md`
  (path verified during worker survey).
- **Sprint 12 close-out deferral** — `production/sprints/sprint-12.md`
  close-out section that deferred `S11-LOBBY-UX-CONFIRM-STATE-001` to
  Sprint 13 (PROMPT 817 close-with-conditions row).
- **Sprint 13 plan row** — `production/sprints/sprint-13.md` Should
  Have row `S11-LOBBY-UX-CONFIRM-STATE-001`.
- **Sprint 13 QA plan** —
  `production/qa/qa-plan-sprint-13.md` (story-023 per-story
  expectations).
- **ADR-002** —
  `docs/architecture/adr-002-client-server-authority.md` (path
  verified at survey time; if relocated, ADR registry is the source
  of truth).

---

## Files Changed by PROMPT 830

| Path | Change |
|------|--------|
| `client/src/ui/lobby.rs` | Edited: introduce `lobby_confirm_button_text`; rewire `LobbyDynamicText::Confirm` arm to delegate. |
| `client/Cargo.toml` | Edited: register the new `[[test]]` entry. |
| `tests/integration/playable_client/lobby_confirm_state_text_test.rs` | NEW. |
| `production/qa/evidence/sprint-13-lobby-confirm-state-evidence.md` | NEW (this file). |
| `reports/PROMPT-830-S13-Lobby-Confirm-State.md` | NEW worker final report (gitignored — `reports/`). |
