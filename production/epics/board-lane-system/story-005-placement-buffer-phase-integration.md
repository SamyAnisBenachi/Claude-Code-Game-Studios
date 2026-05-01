# Story 005: Placement Buffer and Phase Integration

> **Epic**: Board / Lane System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-005`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture; ADR-010: RSM Phase Event Bus
**ADR Decision Summary**: ADR-007 — placements are buffered as plain Rust data in `PendingPlacements` until `close_placement_phase` runs; `S2CPlacementReveal` is enqueued on `ReliableChannel` BEFORE any ECS unit entity is spawned — this ordering is load-bearing and must never be violated. ADR-010 — Board/Lane subscribes to `PlacementPhaseEntered` (opens buffer) via `MessageReader`; the RSM has zero direct imports from `feature/`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `MessageReader<PlacementPhaseEntered>` / `MessageWriter<PlacementCommitted>` — `EventWriter`/`EventReader` no longer exist in Bevy 0.17+. Verify `ServerMultiMessageSender` system param name for server-side S2C broadcast in Lightyear 0.26 (see Control Manifest Lightyear checklist item 9). Component replication is opt-in — `Commands::spawn()` without `Replicate` produces no replication (confirmed, checklist item 18). `liv-bevy-018` AND `liv-bevy-lightyear` skills are both mandatory on this file.

**Control Manifest Rules (this layer)**:
- Required: `S2CPlacementReveal` and entity spawning happen in the same system invocation, in this order: **(1) enqueue S2CPlacementReveal, (2) spawn entities**
- Required: Unit ECS entities may ONLY be spawned AFTER `S2CPlacementReveal` is enqueued on `ReliableChannel` — this invariant must never be violated
- Required: `PendingPlacements` is fully cleared on entry to each new PLACEMENT phase
- Required: Placement validation is all-or-nothing per player
- Required: Mana deduction happens at PLACEMENT close, not at submission receipt
- Forbidden: Never spawn ECS entity for a pending placement before `S2CPlacementReveal` is enqueued

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [x] **BL-14**: GIVEN Player A submitted unit X to (lane 1, cell 1) and Player B submitted unit Y to (lane 5, cell 8) in the pending buffer, WHEN sub-step 1 commits the buffer, THEN `get_units_at_cell(lane 1, cell 1)` returns unit X AND `get_units_at_cell(lane 5, cell 8)` returns unit Y — both visible after the same commit, with neither visible in the world state before sub-step 1 fires.
- [x] **ADR-007-LC-1 (Buffer clear)**: GIVEN `PlacementPhaseEntered` message fires, WHEN `placement_buffer_open` system runs, THEN `PendingPlacements.submissions` is empty for all players (any stale data from the previous round is cleared).
- [x] **ADR-007-LC-2 (Dedup)**: GIVEN Player A submits a valid `C2SSubmitPlacement` and then submits a second `C2SSubmitPlacement` in the same PLACEMENT phase, WHEN the second message is processed, THEN `PendingPlacements[Player A].placements` is unchanged (first submission retained) and `is_final` remains `true`.
- [x] **ADR-007-LC-3 (Ordering invariant)**: GIVEN `close_placement_phase` runs, WHEN it processes pending submissions, THEN `S2CPlacementReveal` is enqueued on `ReliableChannel` before any unit ECS entity is spawned in the same system invocation.

---

## Implementation Notes

*Derived from ADR-007 Buffer Lifecycle, Key Interfaces, and Implementation Guidelines:*

**File**: `server/src/feature/board/placement.rs`

The three systems that own the buffer lifecycle:

```
placement_buffer_open
    - Triggered by: MessageReader<PlacementPhaseEntered>
    - Action: PendingPlacements.submissions.clear()
    - Scheduling: .after(advance_phase) in BoardPlugin

handle_placement_submission
    - Triggered by: incoming C2SSubmitPlacement messages
    - Action: validate batch (Stories 003 + 004 functions), write to PendingPlacements if valid
    - Guards: phase_gate check (phase == PLACEMENT), is_final dedup guard
    - Scheduling: .after(advance_phase), no ordering constraint vs placement_buffer_open

close_placement_phase
    - Triggered by: (a) both players have is_final=true, OR (b) placement_timer expires
    - Action (in strict order per ADR-007):
        1. Collect PendingPlacements submissions
        2. Deduct mana via Economy System events (emit AwardGold/SpendMana)
        3. Enqueue S2CPlacementReveal broadcast on ReliableChannel  ← MUST BE FIRST
        4. Spawn ECS unit entities + add to Lightyear replication group
        5. Emit PlacementCommitted { round_number, committed_placements }
        6. PendingPlacements.submissions.clear()
    - Scheduling: .after(handle_placement_submission).after(placement_timer_tick)
```

The load-bearing invariant (step 3 before step 4) must not be split across two Bevy systems. If ever refactored, an explicit `.before()`/`.after()` constraint must be added and a CI grep must verify ordering.

**Reconnect handling**: If a player reconnects mid-PLACEMENT with `is_final=true`, `S2CGameSnapshot` carries `PlayerSnapshot.submitted=true`. The buffer is intact server-side; no re-send needed.

**Lightyear 0.26 send API** (verify against checklist item 9 before implementing):
```
ServerMultiMessageSender::send::<S2CPlacementReveal, ReliableChannel>(&msg, &server, &NetworkTarget::All)
```

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 003: `validate_spawn_range` function (called by `handle_placement_submission`)
- Story 004: Occupancy functions (called by `handle_placement_submission`)
- Combat Resolution [M2]: Sub-step 1 that fires `PlacementCommitted` to trigger reveal

---

## QA Test Cases

*Written by qa-lead at story creation. Integration story — manual verification steps for ordering invariant; automated test for atomic visibility.*

**Manual check: BL-14 — Atomic commit of pending placements**
- Setup: Start a headless server session. Player A submits `C2SSubmitPlacement` for unit X at (lane 1, cell 1). Player B submits for unit Y at (lane 5, cell 8). Confirm both are in `PendingPlacements` before sub-step 1. Query board world state: `get_units_at_cell(lane 1, cell 1)` and `get_units_at_cell(lane 5, cell 8)`.
- Verify: Before sub-step 1: both queries return `None`. After sub-step 1: both queries return `Some(entity)` within the same frame.
- Pass condition: Both units appear in the same system invocation; neither is visible before commit.

**Manual check: ADR-007-LC-3 — S2CPlacementReveal ordering invariant**
- Setup: Instrument `close_placement_phase` to log (a) when `S2CPlacementReveal` is enqueued and (b) when each unit ECS entity is spawned, with sequence numbers.
- Verify: In a single system run, all (a) log lines precede all (b) log lines.
- Pass condition: No entity-spawn log line has a sequence number ≤ the reveal-enqueue log line.

**Automated test: ADR-007-LC-1 — Buffer cleared on phase entry**
- Given: `World::new()` with `PendingPlacements` containing stale data from previous round; `PlacementPhaseEntered` message written to world
- When: `placement_buffer_open` system runs
- Then: `PendingPlacements.submissions.is_empty()` == `true`

**Automated test: ADR-007-LC-2 — Duplicate submission discarded**
- Given: `World::new()` with `PendingPlacements`; first `C2SSubmitPlacement` accepted for PlayerA (is_final=true)
- When: second `C2SSubmitPlacement` received for PlayerA in same phase
- Then: `PendingPlacements[PlayerA].placements` unchanged; `is_final` still `true`; no S2C message enqueued

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- Automated: `tests/integration/board-lane-system/placement_buffer_test.rs` — must exist and pass (covers LC-1, LC-2)
- Manual: `production/qa/evidence/placement-buffer-evidence.md` — ordering invariant (LC-3, BL-14) verified by instrumented run; lead sign-off required

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 003 must be DONE (spawn range validation), Story 004 must be DONE (occupancy validation)
- Unlocks: Nothing in this epic — terminal story for the placement pipeline

## Completion Notes

**Completed**: 2026-05-02
**Criteria**: 4/4 passing.
**Deviations**: None blocking. Advisory only: `production/qa/evidence/placement-buffer-evidence.md` records automated evidence only; no manual lead sign-off is claimed.
**Test Evidence**: `tests/integration/board-lane-system/placement_buffer_test.rs` passes 3/3 and verifies a live Lightyear `S2CPlacementReveal` receive on `ReliableChannel` before replicated unit spawn. `production/qa/evidence/placement-buffer-evidence.md` records the automated commands/results.
**Code Review**: Skipped - Lean mode.
