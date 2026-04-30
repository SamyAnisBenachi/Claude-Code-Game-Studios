# Story 001: PrismState Scaffold and Session Lifecycle

> **Epic**: Prism System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/prism-system.md`
**Requirement**: `TR-PRI-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-016: Prism System Architecture — State Ownership, Schedule Slot, and Hand-Write API
**ADR Decision Summary**: `PrismState` is a `#[derive(Resource, Default)]` type with exclusive `ResMut` access in `resolve_prism_draws` only. `DiscardLog` and `AuditLog` are server-only Resources for test-inspectable discard and RNG audit tracking. Ten `PrismPresence` entities (one per player×lane) are spawned at session start and replicated to clients via Lightyear `UnreliableChannel`. All three resources are inserted at session start and removed on `GameOverEmitted`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `PrismState`: `#[derive(Resource, Default)]` — stable across Bevy 0.15–0.18; no deprecated derives
- `DiscardLog`, `AuditLog`: `#[derive(Resource, Default)]` — same
- `PrismPresence`: `#[derive(Component, Clone, Serialize, Deserialize)]` — Lightyear-replicated component
- `Replicate` component for PrismPresence entities: Lightyear 0.26 per-entity client scoping API must be verified against `docs.rs/lightyear/0.26` before spawning (ADR-016 Verification Required item 2)
- Session lifecycle: insert resources in PrismPlugin reaction to `SessionReady`; despawn `PrismPresence` entities and remove resources on `GameOverEmitted`
- `app.add_message::<PrismCollected>()` is owned by the Board/Lane System plugin — confirm this registration exists before `resolve_prism_draws` can compile

**Control Manifest Rules (Feature layer — from ADR-016):**
- Required: `ResMut<PrismState>` appears exclusively in `resolve_prism_draws` — code-review gate on every Prism PR
- Required: PrismState, DiscardLog, AuditLog inserted at session start (triggered by `SessionReady` / `ResolutionPhaseEntered` wiring); removed on `GameOverEmitted`
- Required: Ten `PrismPresence` entities spawned at session start; despawned on `GameOverEmitted`
- Forbidden: `EventWriter<T>` / `EventReader<T>` — use `MessageWriter<T>` / `MessageReader<T>` for buffered server-internal messages; `#[derive(Event)]` + `Observer` for one-shot reactive triggers

---

## Acceptance Criteria

*From GDD `design/gdd/prism-system.md`, scoped to this story:*

- [ ] **PS-08** — GIVEN `resolve_prism_draws` is called with no `PrismCollected` message in the `MessageReader` buffer for a given `(player, lane)`, WHEN the function runs, THEN `collected[lane][player]` remains `false` and no reward is granted for that lane.
- [ ] **PS-07b** — GIVEN a prism token was collected in RESOLUTION N (`collected[lane][player] == true`) AND no full-set respawn has occurred, WHEN RESOLUTION N+1's sub-step 5 completes with the same WALL unit still at that spawn cell (no PrismCollected message emitted for that lane), THEN `collected[lane][player]` remains `true`, no `PrismCollected` message fires for that lane, and no reward is granted — confirming per-lane collected state persists across RESOLUTIONs until full-set respawn.
- [ ] **PS-15** — GIVEN any prism token is collected (any lane, any player), WHEN the player's resource totals are read after RESOLUTION, THEN the player's gold total is unchanged — prisms grant zero gold. Economy System not in call chain.

---

## Implementation Notes

*Derived from ADR-016 Decision Section 1 (PrismState) and Section 4 (PrismPresence):*

```rust
// server/feature/prism/state.rs

#[derive(Resource, Default)]
pub struct PrismState {
    /// [player_index][lane_index (0-based, lane 1 = index 0)] — true = collected
    pub collected: [[bool; 5]; MAX_PLAYERS],
    /// Transient per-RESOLUTION flag; set in Rule 8, cleared after Rule 9 fires
    pub pending_respawn: [bool; MAX_PLAYERS],
}

/// Test-facing discard record — produced by resolve_prism_draws on stale PrismCollected messages.
#[derive(Resource, Default)]
pub struct DiscardLog {
    pub entries: Vec<(PlayerId, u8)>,  // (player_id, lane)
}

/// RNG audit log — one entry per ServerRng::next_seed() call in resolve_prism_draws.
#[derive(Resource, Default)]
pub struct AuditLog {
    pub entries: Vec<PrismAuditEntry>,
}

pub struct PrismAuditEntry {
    pub player_id: PlayerId,
    pub lane: u8,
    pub seed_index: u32,
    pub result: Option<CardId>,  // None = pool exhausted
}
```

```rust
// server/feature/prism/components.rs

#[derive(Component, Clone, Debug)]
pub struct PrismLaneKey {
    pub player: PlayerId,
    pub lane: u8,  // 1–5
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct PrismPresence {
    pub collected: bool,
}
```

**Session lifecycle** in `PrismPlugin::build()`:
- Insert `PrismState::default()`, `DiscardLog::default()`, `AuditLog::default()` at session start
- Spawn 10 `PrismPresence` entities (player × lane pairs) with `Replicate` targeting `UnreliableChannel` — Lightyear 0.26 API must be verified before this step
- Remove all three resources + despawn 10 entities on `GameOverEmitted`

**Initial state** per GDD Rule 2: `collected[lane][player] = false` for all 5 lanes × all players. Achieved by `PrismState::default()` (array of bools default to `false`).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: The actual `resolve_prism_draws` reward routing for Lanes 1/2/4/5
- Story 003: Lane 3 RNG draw pipeline and AuditLog population
- Story 004: Hand-full rejection logic and S2CPrismRewardDropped staging
- Story 005: Full-set respawn cycle logic
- Story 006: prism_strike and prism_reserve play path (Card Acquisition + Objective/Economy System)

---

## QA Test Cases

*Written at story creation. The developer implements against these.*

- **PS-08**: No message → no state change
  - Given: `PrismState` initialized (`all collected = false`); `DiscardLog` and `AuditLog` empty; no `PrismCollected` messages in `MessageReader` buffer
  - When: `resolve_prism_draws` runs
  - Then: all `collected[lane][player] == false`; `hand.len()` unchanged; `AuditLog.entries` empty; `DiscardLog.entries` empty; no `S2CCardAcquired` staged
  - Edge cases: second consecutive RESOLUTION with empty buffer (state still holds from prior round)

- **PS-07b**: Per-lane collected state persists across RESOLUTIONs
  - Given: `collected[2][player_a] = true` (Lane 3 was collected in a prior RESOLUTION); no new `PrismCollected(player_a, 3)` message in buffer for this RESOLUTION
  - When: `resolve_prism_draws` runs for RESOLUTION N+1
  - Then: `collected[2][player_a]` remains `true`; hand unchanged; `AuditLog` unchanged; `DiscardLog` unchanged
  - Edge cases: `collected[2][player_a] = true` while `collected[0][player_a] = false` (mixed state across lanes)

- **PS-15**: No gold change on any collection
  - Given: `player_a.gold = G`; `PrismCollected(player_a, lane_1)` in buffer; hand < 10
  - When: Story 002's reward routing runs (can be tested together with PS-01 in Story 002 tests)
  - Then: `player_a.gold == G`; `Economy::spend_gold` not called; no `S2CGoldUpdate` staged for gold change
  - Note: This AC is verifiable as a side-effect assertion in Story 002's lane reward tests — include as a `assert_eq!(gold_before, gold_after)` invariant assertion

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/prism/state_scaffold_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: None — foundational story; all other Prism System stories depend on this one
- Depends on (external): Card Acquisition Story 001 (`state-scaffold`) must be Done — `PlayerHands` and `hand_push()` shared API must be defined and public before Prism System stories beyond this one can be implemented
- Unlocks: Stories 002, 003, 004, 005, 006
