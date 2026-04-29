# Story 003: Class Selection and Reveal

> **Epic**: Game Session System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-session-system.md`
**Requirement**: TR-GSS-09 (deferred simultaneous reveal — `S2CClassesRevealed` broadcast only when all slots locked)

**ADR Governing Implementation**: ADR-008 (Lightyear Channel Config)
**ADR Decision Summary**: `S2CClassLocked` unicast on `ReliableChannel` to the locking player only. `S2CClassesRevealed` broadcast on `ReliableChannel` to all session participants once all occupied slots have confirmed. Preview selections are never broadcast — server only.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: No post-cutoff Bevy APIs in this story beyond `MessageSender` (Lightyear 0.26 — already gated by ADR-008 verification in Story 002). `liv-bevy-018` and `liv-bevy-lightyear` skills are mandatory on all `.rs` files.

**Control Manifest Rules (Core layer)**:
- Required: `handle_select_class` updates preview state only — never broadcasts, never writes to `ClassSelections`.
- Required: `handle_confirm_class` writes `SessionSlot.class = Some(class_id)` and `ClassSelections[player_id] = class_id` in the same system call (atomically from the ECS perspective).
- Required: `S2CClassesRevealed` is sent only once — after the second player confirms; idempotent guard on `ClassSelections` length.
- Forbidden: Re-locking with a different class after confirm. Any attempt must return `S2CClassLockRejected { reason: AlreadyLocked }`.

---

## Acceptance Criteria

- [ ] `handle_select_class` system exists in `server/src/core/session/system.rs` and:
  - Updates a `ClassPreviews(HashMap<PlayerId, ClassId>)` server-side resource (preview state, not persisted to `SessionSlot`)
  - Does NOT send any S2C message (preview is server-side only)
  - Does NOT write to `ClassSelections`
  - Sends `S2CSelectClassRejected { reason: SessionNotInLobby }` if `LobbyState != LobbyWaiting`
- [ ] `handle_confirm_class` system exists in `server/src/core/session/system.rs` and:
  - Rejects with `S2CClassLockRejected { reason: AlreadyLocked }` if `ClassSelections` already contains this `PlayerId` (idempotent guard — player cannot re-lock with a different class)
  - Rejects with `S2CClassLockRejected { reason: SessionNotInLobby }` if `LobbyState != LobbyWaiting`
  - On success: writes `SessionSlot.class = Some(class_id)` for the player's slot AND inserts `class_id` into `ClassSelections[player_id]` in the same system call
  - On success: sends `S2CClassLocked { player_id, class_id }` unicast to the locking player on `ReliableChannel`
  - After writing: checks if all occupied slots in `SessionSlots` have `class = Some(_)`; if true, broadcasts `S2CClassesRevealed { class_map: HashMap<PlayerId, ClassId> }` to all session participants on `ReliableChannel`
  - `S2CClassesRevealed` is sent exactly once — the idempotent guard (`AlreadyLocked` rejection on re-confirm) prevents a second broadcast
- [ ] `ClassPreviews(HashMap<PlayerId, ClassId>)` newtype resource is defined in `server/src/core/session/state.rs`
- [ ] `GameSessionPlugin` registers both handlers in the Bevy `Update` schedule
- [ ] `cargo check -p server` passes with zero warnings
- [ ] Unit tests in `tests/unit/session/class_reveal_test.rs` pass — see QA Test Cases

---

## Implementation Notes

*Derived from EPIC.md §Scope (handle_select_class, handle_confirm_class) and GDD Rule 7:*

**Preview vs confirm distinction**: `handle_select_class` tracks intent without committing. A player may change their preview any number of times before confirming. Only `handle_confirm_class` locks the choice and writes to `ClassSelections`. The preview resource (`ClassPreviews`) is cleared when the session ends (Story 006 teardown) or is cancelled (Story 005).

**All-locked detection**: After writing the second player's class, iterate `SessionSlots` and check that every slot where `player = Some(_)` also has `class = Some(_)`. If this condition holds, broadcast `S2CClassesRevealed`. Do this check inline in `handle_confirm_class` — do not schedule a separate system for this.

**`S2CClassesRevealed` payload**: The `class_map` field contains all player-to-class mappings for the session. This is the moment both players learn each other's class choice simultaneously (the bluff aspect of class selection). The server never reveals a player's class before this broadcast.

**Re-lock rejection semantics**: If Player A confirms `ClassId::Iop`, then sends `C2SConfirmClass { class_id: ClassId::Feca }`, the second message must be rejected with `AlreadyLocked`. The slot state is not mutated. This prevents mid-session class switching after the reveal.

**`ClassSelections` vs `ClassPreviews`**: `ClassSelections` feeds directly into `build_session_config` (Story 001) and is the source of truth for `SessionConfig.class_map`. `ClassPreviews` is ephemeral and has no downstream consumers beyond UX feedback (which is not in MVP scope at server level).

---

## Out of Scope

- `evaluate_session_ready` F4 predicate that reads `ClassSelections` (Story 004)
- Client-side class preview UI rendering
- Class availability validation (no "you can't pick the same class as your opponent" rule at MVP)

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: single player confirms — unicast only, no reveal**
  - Given: 2-slot session in `LobbyWaiting`; Player A has not confirmed; Player B has not confirmed
  - When: Player A sends `C2SConfirmClass { class_id: Iop }`
  - Then: `S2CClassLocked { player_id: A, class_id: Iop }` sent to A only; `S2CClassesRevealed` NOT sent; `ClassSelections` contains only A

- **AC: both players confirm — reveal broadcasts to all**
  - Given: Player A already confirmed `Iop`; Player B sends `C2SConfirmClass { class_id: Feca }`
  - When: `handle_confirm_class` processes B's message
  - Then: `S2CClassLocked { player_id: B }` unicast to B; `S2CClassesRevealed { class_map: {A: Iop, B: Feca} }` broadcast to both A and B; `ClassSelections.len() == 2`

- **AC: re-lock rejection**
  - Given: Player A has already confirmed `Iop` (in `ClassSelections`)
  - When: Player A sends `C2SConfirmClass { class_id: Feca }`
  - Then: `S2CClassLockRejected { reason: AlreadyLocked }` unicast to A; `ClassSelections[A]` is still `Iop`; no `S2CClassesRevealed` sent

- **AC: select (preview) does not broadcast**
  - Given: 2-slot session in `LobbyWaiting`
  - When: Player A sends `C2SSelectClass { class_id: Iop }` three times with different classes
  - Then: No S2C messages sent; `ClassPreviews[A]` holds the last class; `ClassSelections` is empty

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/session/class_reveal_test.rs` — all test cases passing
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (session types — `SessionSlots`, `ClassSelections`, `LobbyState`)
- Depends on: Story 002 (room create/join — session must exist and be in `LobbyWaiting` state for class selection to proceed)
- Unlocks: Story 004 (F4 predicate reads `ClassSelections` — all slots must be confirmable before readiness can be evaluated)
