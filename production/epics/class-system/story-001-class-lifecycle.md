# Story 001: Class Lifecycle — PlayerSessions Scaffold

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: `PlayerSessions` is a server-only `Resource` (HashMap<PlayerId, PlayerSessionData>) storing `class: ClassId` and `class_locked: bool`. Class selection via `C2SClassChoice` Lightyear message (LOBBY phase only). `all_classes_chosen()` gate refuses LOBBY→DRAFT_INITIAL until every player has a non-Neutral class. `lock_all_classes()` called atomically by RSM before emitting `LobbyComplete`. `PlayerSnapshot.class_id` added to `S2CGameSnapshot` (resolves NP-1).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `C2SClassChoice` must derive `lightyear::prelude::Message` — NOT `bevy::prelude::Message`. Both traits exist simultaneously in this project. Use the fully-qualified path.
- `MessageReceiver<C2SClassChoice>.receive_messages()` — Lightyear 0.26 server receive API. Verify exact method name against `docs/engine-reference/bevy/current-best-practices.md` before implementing. Wrap in `server/src/lobby/handler.rs` — single file to update if API shifts.
- `EventWriter`/`EventReader` do not exist in Bevy 0.17+. Class lifecycle signals (`LobbyComplete`) use `MessageWriter::write()` registered with `app.add_message::<T>()`.
- ADR-014 is NOT yet in the control manifest (manifest covers ADR-001–012). Apply Feature Layer rules from the manifest generically; class-specific rules come from ADR-014 directly.

**Control Manifest Rules (Feature Layer)**:
- Required: Feature systems subscribe to Core phase Messages; never observe `RoundState` directly — ADR-010
- Required: Phase-gate pattern in every C2S handler: `if round_state.phase != LOBBY { return; }` — ADR-002, ADR-009
- Forbidden: Never let Feature systems call Core/Foundation systems directly — ADR-010
- Guardrail: Server tick budget ≤ 5ms steady state — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, scoped to this story:*

- [ ] **CS-AC-01** GIVEN a lobby with two players, WHEN Player A selects Xelor and clicks Ready, THEN Player A's `class` field is locked to `Xelor` on the server and subsequent class-change messages from Player A are rejected.
- [ ] **CS-AC-02** GIVEN both players have locked their class, WHEN the RSM transitions LOBBY → DRAFT_INITIAL, THEN every active player's `class` field is `Some(C)` — no player may have `class = None` (`ClassId::Neutral`).
- [ ] **CS-AC-03a** GIVEN both players' classes are locked and the RSM has transitioned to DRAFT_INITIAL, WHEN any player receives `S2CGameSnapshot`, THEN the `PlayerSnapshot` for each player contains a `class_id` field equal to that player's locked class.

---

## Implementation Notes

*Derived from ADR-014 Decision §1 and §2:*

**`PlayerSessions` Resource** — file: `server/src/core/session/state.rs`

```rust
#[derive(Resource, Default)]
pub struct PlayerSessions {
    pub players: HashMap<PlayerId, PlayerSessionData>,
}

#[derive(Default, Clone, Debug)]
pub struct PlayerSessionData {
    /// ClassId::Neutral = "not yet chosen" sentinel. Unreachable at phase >= DRAFT_INITIAL.
    pub class: ClassId,
    pub class_locked: bool,
    // Future fields added by Economy ADR (gold, current_mana, reserve), Card Acq ADR (hand)
}
```

**Key methods on `PlayerSessions`**:
- `class_of(&self, player_id: PlayerId) -> ClassId` — O(1) HashMap lookup; panics with message if player not registered
- `is_locked(&self, player_id: PlayerId) -> bool`
- `all_classes_chosen(&self) -> bool` — returns true iff all players have `class != ClassId::Neutral`
- `lock_all_classes(&mut self)` — sets `class_locked = true` for all; `debug_assert!` fires if any player still has `ClassId::Neutral`

**`C2SClassChoice` handler** — file: `server/src/lobby/handler.rs`
- Sole drainer of `MessageReceiver<C2SClassChoice>` — exactly one system drains this receiver (forbidden pattern from ADR-013/ADR-014: registering a second drain starves this handler silently)
- Logic: `if player.class_locked { continue; }` / `if msg.class == ClassId::Neutral { continue; }` / else `player.class = msg.class`

**RSM LOBBY→DRAFT_INITIAL gate**:
- Gate predicate: `sessions.all_classes_chosen()` — if false, refuse transition; lobby stays in LOBBY
- On gate pass: `sessions.lock_all_classes()` → emit `LobbyComplete` Message

**Protocol additions** (file: `shared/src/protocol.rs` or `shared/src/snapshot.rs`):
- `C2SClassChoice { class: ClassId }` — derive `lightyear::prelude::Message, Serialize, Deserialize, Clone, Debug`
- `PlayerSnapshot.class_id: ClassId` — never `ClassId::Neutral` at phase ≥ DRAFT_INITIAL (resolves NP-1)

**Import rule**: Always import `ClassId` from `shared::card::ClassId` — never redefine it.

**Insertion lifecycle** (ADR-012 contract): `PlayerSessions` inserted before `commands.trigger(SessionReady)`; removed on `GameOverEmitted`.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 002: `SourceClass` component and token spawn functions — separate concern
- Story 003–008: Class effect formulas (Gelure, Xelorium, etc.) — depend on this scaffold being in place
- Economy fields (`gold`, `current_mana`, `reserve`) in `PlayerSessionData` — added by Economy System ADR/stories
- `garde_temps_used_this_game` counter — owned by Game Session System (not PlayerSessions)
- UI: class picker display, Garde-Temps gate visual feedback — Presentation layer

---

## QA Test Cases

*Logic story — automated test specs. Implement using `World::new()` (Bevy ECS tests); no Lightyear runtime needed for pure state assertions.*

- **AC CS-AC-01a**: Class choice accepted while unlocked
  - Given: `PlayerSessions` with player A at `class = ClassId::Neutral, class_locked = false`
  - When: `handle_class_choice` processes `C2SClassChoice { class: ClassId::Xelor }` for player A
  - Then: `sessions.players[A].class == ClassId::Xelor`; `class_locked` still `false`
  - Edge cases: `C2SClassChoice { class: ClassId::Neutral }` is silently discarded; `class` unchanged

- **AC CS-AC-01b**: Class change rejected when locked
  - Given: player A has `class = ClassId::Xelor, class_locked = true`
  - When: `handle_class_choice` processes `C2SClassChoice { class: ClassId::Iop }` for player A
  - Then: `sessions.players[A].class == ClassId::Xelor` (unchanged); no error response

- **AC CS-AC-02a**: Gate rejects transition when a player has Neutral class
  - Given: 2 players; player A `class = ClassId::Xelor`; player B `class = ClassId::Neutral`
  - When: `sessions.all_classes_chosen()` is called
  - Then: returns `false`; transition is refused; both players remain in LOBBY

- **AC CS-AC-02b**: Gate passes and locks when all players have non-Neutral class
  - Given: 2 players; player A `class = ClassId::Xelor`; player B `class = ClassId::Sacrier`
  - When: `sessions.all_classes_chosen()` called, then `sessions.lock_all_classes()` called
  - Then: both players have `class_locked = true`; subsequent `handle_class_choice` messages silently discarded
  - Edge cases: `lock_all_classes()` with any player at `ClassId::Neutral` triggers `debug_assert!` panic in debug builds

- **AC CS-AC-03a**: PlayerSnapshot contains class_id
  - Given: session with player A locked to `ClassId::Sacrier`
  - When: `build_snapshot(player_a_id, &world)` is called
  - Then: returned `PlayerSnapshot.class_id == ClassId::Sacrier`; never `ClassId::Neutral`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/class/class_lifecycle_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `workspace-and-shared-types` story-001 (provides `ClassId` in `shared/src/card.rs`) + `workspace-and-shared-types` story-002 (provides `C2SClassChoice` message type in `shared/src/protocol.rs`) — both must be DONE
- Depends on: `round-state-machine` story-001 (RSM State + Events Scaffold — provides `LobbyComplete` Message on ADR-010 event bus) — must be DONE
- Unlocks: Story 002 (`class_of()` needed for token spawn validation); Stories 003–009 (all class effects read `PlayerSessions`)
