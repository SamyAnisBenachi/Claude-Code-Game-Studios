# Sprint 12 — S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001 — Evidence

> **Story**: `production/epics/playable-client/story-013-lobby-confirm-class-intent-chain.md`
> **Story ID**: S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001
> **Cluster**: B3 (per `production/qa/evidence/sprint-11-ignored-d5-triage.md` row 85)
> **Sprint**: 12 (active)
> **PROMPT**: 801
> **Author**: gameplay-programmer + qa-lead (worker on `work/s11-lobby-confirm-class-intent-chain`)
> **Source-of-truth at implementation**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12 QA plan commit)
> **Date**: 2026-05-14

---

## Status / No-Claim Banner

This evidence document records the implementation of story 013 under
PROMPT 801 (`/dev-story`). It is **paperwork-of-record for one
production-repair story scope**.

This evidence does **not** claim: public release readiness,
release-candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
validation (`QA-COND-0006`), full playable-client manual QA, two-client
GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production
completion, Sprint 12 close-out, Polish → Release gate retry, or stage
advance.

**No optimistic client-side class-lock authority was introduced** — the
ADR-002 session-id gate in `request_confirm_class` is preserved
unchanged. The fix is a test-fixture repair, not a production code
change to the gate. See "Disposition" below.

Sprint 10 disposition (`closed-with-conditions`), Sprint 11 disposition
(`closed-with-conditions` per PROMPT 792), and PROMPT 761 Polish→Release
gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` are
preserved unchanged.

---

## Investigation Note (AC1)

### Lobby input chain trace (post-D-1 fix)

The lobby UI surfaces two distinct buttons for the class-lock flow:

- `LobbyClassButton` (per-class preview button) — see
  `client/src/ui/lobby.rs:175` (component def) and `:958` (spawn).
- `LobbyConfirmClassButton` (separate confirm gesture) — see
  `client/src/ui/lobby.rs:177` (component def) and the spawn block that
  attaches it alongside `LobbyClassButton`.

The two-intent chain is wired through a single button-interaction
system:

1. **`lobby_button_interaction_system`** (`client/src/ui/lobby.rs:465`)
   queries `Changed<Interaction>` on every lobby button entity and
   dispatches:
   - `LobbyClassButton` press → `request_select_class` →
     `commands.write(LobbyCommand::SelectClass { class_id })` (line 784).
   - `LobbyConfirmClassButton` press → `request_confirm_class` →
     `commands.write(LobbyCommand::ConfirmClass { class_id })` (line 823),
     **gated** on `lobby.session_id.is_some()` (line 792) and
     `!input.class_confirm_in_flight` (line 804).

2. **`send_lobby_commands_system`** (`client/src/ui/lobby.rs:510`)
   reads `LobbyCommand` events and writes the corresponding C2S
   Lightyear messages on the reliable channel:
   - `LobbyCommand::SelectClass { class_id }` →
     `sender.send::<ReliableChannel>(C2SSelectClass { class_id })`
     (line 570).
   - `LobbyCommand::ConfirmClass { class_id }` →
     `sender.send::<ReliableChannel>(C2SConfirmClass { class_id })`
     (line 587).

### Why ConfirmClass was suppressed in the (previously-ignored) test

The PROMPT 750 D-5 owner comment read:

> `#[ignore = "PROMPT 750 D-5 follow-on: ConfirmClass intent not emitted
> alongside SelectClass — input chain stops at SelectClass; needs lobby
> input system investigation (revealed after D-1 fix)"]`

This was a **misdiagnosis**. The input chain does not stop at
`SelectClass`; both `LobbyCommand` events are dispatched correctly by
the same system, and the corresponding C2S writers are present. The
actual reason the test failed was:

- The test exercised lobby buttons in isolation, without simulating any
  server-to-client (S2C) round-trip.
- After pressing the JoinRoom button, the test never simulated the
  server's `S2CJoinAck` response that, in production, calls
  `apply_join_ack` (`client/src/ui/lobby.rs:380`) to set
  `lobby.session_id = Some(...)`.
- Consequently, `lobby.session_id` remained `None` at the moment the
  confirm button was pressed.
- `request_confirm_class` correctly early-returned via the
  `if lobby.session_id.is_none()` gate (line 792), logging
  `"lobby_ui_confirm_button_state: blocked — no active session_id
  (premature confirm)"` (verified at test re-run; see "Pre-Fix Failure
  Log" below).

### Why no production code change is needed

The `session_id.is_none()` gate is **binding per ADR-002**
(Client-Server Authority): the client is not authoritative on
class-lock state and must not emit confirm intents before the server
has acknowledged the join. Removing the gate, or replacing it with a
client-side optimistic mutation, would violate ADR-002. The story file
("No optimistic client-side class-lock authority is introduced or
proposed by this story or by any disposition pathway") and AC5
explicitly forbid that direction.

Therefore the production code is correct as written, and the failing
assertion is an artefact of an incomplete test fixture — the fixture
exercises the lobby UI in isolation but omits the server round-trip
that production requires.

### Disposition chosen

**Fallback path (test redesign + production gate preserved)** per
story 013 "Decision space" section and Sprint 12 QA plan
"Decision space (narrow; primary path strongly preferred)" subsection.

- The **production gate is preserved** unchanged in
  `client/src/ui/lobby.rs:792` (and `:804`).
- The **test fixture is updated** to mirror the production S2CJoinAck
  round-trip by setting `lobby.session_id = Some("XY9".to_string())`
  immediately after the JoinRoom button press and clearing
  `input.join_in_flight = false`. The test still asserts the two-intent
  chain (`SelectClass` AND `ConfirmClass`) end-to-end, exactly as
  authored.

`ConfirmClass` remains the right next-intent shape; the UX is a
two-button chain (class-select + explicit confirm), not a one-button
shortcut. No protocol shape change is needed; the `C2SConfirmClass`
message exists at `shared/src/protocol.rs:431` and is registered on the
reliable channel at `shared/src/protocol.rs:62` (verified by
`/story-readiness` PROMPT 797 and re-verified by PROMPT 801).

---

## Production Diff Summary (AC2)

**Production diff (`client/src/`)**: NONE.

The fix does not touch `client/src/ui/lobby.rs` or any other
`client/src/` file. The lobby input system is correct as written; the
ADR-002 session-id gate is preserved unchanged.

**Test diff (`tests/integration/playable_client/`)**:

- `tests/integration/playable_client/native_operator_controls_test.rs`:
  - Added `LobbyViewState` to the `client::ui::lobby` import set
    (line 17-21).
  - Removed `#[ignore = "PROMPT 750 D-5 follow-on: ..."]` from
    `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
    (was at line 106; now removed).
  - Inserted an S2CJoinAck round-trip simulation block (a direct
    mutation of `LobbyViewState::session_id = Some("XY9".to_string())`
    and `LobbyInputState::join_in_flight = false`) between the JoinRoom
    button press and the class/confirm button presses. The block
    includes an inline comment referencing ADR-002 and the
    `request_confirm_class` gate.

**No protocol shape change** under PROMPT 801.

**Diff is scoped** to the test file only; no broader rework.

---

## Test Evidence (AC3, AC4, AC6, AC7)

### Pre-fix (ignored) — workspace baseline at `origin/main@b5eef0d`

```text
cargo test -p client --test playable_client_native_operator_controls_test \
    test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands -- --ignored
```

Result: `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands ... FAILED`

Captured warning trace (key line):

```text
WARN client::ui::lobby: lobby_ui_confirm_button_state: blocked — no
active session_id (premature confirm) can_confirm=false
session_id=None local_player_id=None class_id=Xelor
```

The assertion failure was confirmed: the test expected
`vec![LobbyCommand::SelectClass{..}, LobbyCommand::ConfirmClass{..}]`
but observed only `vec![LobbyCommand::SelectClass{..}]` — the
`ConfirmClass` variant was correctly suppressed by the production
session-id gate because the test fixture did not simulate the
`S2CJoinAck` round-trip.

### Post-fix — same command without `--ignored`

```text
cargo test -p client --test playable_client_native_operator_controls_test \
    test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands
```

Result: `test test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands ... ok`
(1 passed; 0 failed; 0 ignored).

The integration test now asserts the full two-intent chain
end-to-end — both `C2SSelectClass` (via `LobbyCommand::SelectClass`) and
`C2SConfirmClass` (via `LobbyCommand::ConfirmClass`) are emitted by the
lobby button click sequence, with the production session-id gate
preserved (AC4 satisfied).

### Per-target file pass — full operator-controls suite

```text
cargo test -p client --test playable_client_native_operator_controls_test --no-fail-fast
```

Result:

```text
running 5 tests
test test_lobby_room_code_textbox_click_selects_and_accepts_text_input ... ok
test test_lobby_room_code_focus_separates_text_from_shortcuts ... ok
test test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands ... ok
test test_hand_pointer_controls_stage_unstage_and_submit_placement ... ok
test test_shop_auction_pointer_controls_emit_operator_intents ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Client crate (full) pass — AC3 / AC6

```text
cargo test -p client --no-fail-fast
```

Result (aggregated): `passed: 391, failed: 0, ignored: 4`.

Drop of 1 from the Sprint 11 close-out client-side baseline of 5
ignored. No new undocumented `#[ignore]` marker introduced by this
story (AC6 satisfied for the client crate slice).

### Workspace pass — AC6

```text
cargo test --workspace --tests --no-fail-fast
```

Result (aggregated, all crates): `passed: 1366, failed: 0, ignored: 4`.

Sprint 11 close-out baseline (per
`production/qa/evidence/sprint-11-ignored-d5-triage.md`): 5 retained
Cluster B `#[ignore]` tests on `origin/main`. Post-PROMPT-801 count: 4.
Drop of 1 = the `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands`
un-ignored under this story. No new undocumented `#[ignore]` marker
introduced. AC6 satisfied workspace-wide.

### `#[ignore]` removal (AC7)

The PROMPT 750 D-5 owner comment was removed only after the test
passed under its current assertion shape. A grep over the test file
confirms no remaining `PROMPT 750 D-5` reference:

```text
grep -n "PROMPT 750 D-5\|#\\[ignore" \
    tests/integration/playable_client/native_operator_controls_test.rs
```

→ (no matches in the test file under PROMPT 801).

---

## No-Optimism Audit (AC5)

The production diff under PROMPT 801 is empty in `client/src/`. The
`request_confirm_class` gate (`client/src/ui/lobby.rs:792`) is
preserved unchanged. The test fixture's direct mutation of
`LobbyViewState::session_id` is the *test mirror of the S2CJoinAck
round-trip*; it is not a production code path and does not introduce
any new code path that mutates class-lock state before
`S2CClassConfirmed` (or `S2CClassLocked`) is observed.

**No optimistic client-side class-lock authority is introduced by this
story.**

Search confirmation:

```text
grep -rn "class_locked\|locked_class" client/src/
```

The only writes to `lobby.locked_class` remain inside
`apply_class_locked` (the S2C consumer at
`client/src/ui/lobby.rs:392`). No new write site introduced.

---

## Sprint 12 Disposition Preservation Audit (AC8)

Files NOT modified by PROMPT 801:

- `production/sprint-status.yaml`
- `production/sprints/sprint-12.md`
- `production/sprints/sprint-11.md`
- `production/stage.txt`
- `production/session-state/active.md`
- `production/session-state/codex-orchestrator-state.md`
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- `production/qa/qa-plan-sprint-12.md`
- `production/qa/qa-plan-sprint-11.md`
- `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- `.claude/settings.json`
- `client/src/ui/lobby.rs`
- `shared/src/protocol.rs`

Stage remains `Polish`. Sprint 12 disposition (active per PROMPT 798)
is preserved unchanged. Sprint 11 disposition
(`closed-with-conditions` per PROMPT 792) preserved unchanged.

---

## Carry-Conditions Preserved

- `S8-QA-001-W1` manual/browser two-client GAME_OVER gap — OPEN
  (unchanged by this story).
- `QA-COND-0005` Standard-tier accessibility — accepted-risk
  friend-game scope (unchanged).
- `QA-COND-0006` playtest / fun-hypothesis validation — accepted-risk /
  deferred (unchanged).
- PAW placeholder-art accept-risk (`PAW-TD-*-a`) — unchanged.
- PROMPT 683-era runtime divergence question — preserved (folded into
  story 019 tighter-capture; not in scope for this story).
- PROMPT 761 Polish→Release gate-check FAIL — preserved (no retry in
  this story).

---

## Cross-Links

- Story file:
  `production/epics/playable-client/story-013-lobby-confirm-class-intent-chain.md`
- Sprint 12 QA plan section:
  `production/qa/qa-plan-sprint-12.md` → `### S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001 (story 013) — Cluster B3`
- Sprint 11 D-5 triage row:
  `production/qa/evidence/sprint-11-ignored-d5-triage.md` → Cluster B3 row 85.
- ADR-002 binding:
  `docs/architecture/adr-002-client-server-authority.md` — preserved.
- ADR-008 binding:
  `docs/architecture/adr-008-lightyear-channel-config.md` — `C2SConfirmClass`
  registered on `ReliableChannel` at `shared/src/protocol.rs:62`.
- ADR-012 binding:
  `docs/architecture/adr-012-session-ready-delivery.md` — unchanged.

---

## Authoring Trail

- 2026-05-14 — PROMPT 801 (`/dev-story`) — Worker on worktree
  `work/s11-lobby-confirm-class-intent-chain` (branch
  `work/s11-lobby-confirm-class-intent-chain`). Source-of-truth at
  entry: `origin/main@b5eef0d`. Disposition: **fallback path** (test
  redesign + production gate preserved). No production code modified.
  Test fixture in
  `tests/integration/playable_client/native_operator_controls_test.rs`
  updated to mirror S2CJoinAck round-trip; `#[ignore]` and PROMPT 750
  D-5 owner comment removed after the test passed. Evidence document
  authored.
