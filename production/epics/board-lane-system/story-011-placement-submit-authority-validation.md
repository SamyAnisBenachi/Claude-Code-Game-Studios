# Story 011: Placement Submit Authority Validation

> **Epic**: Board / Lane System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-011`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*
**Readiness dependency status**: NP-005 is Complete at main integration `705defa`; ECO-007 is Complete at main integration `a564d99`; `TR-BLS-011` is active; ADR-007, ADR-019, ADR-002, and ADR-008 are Accepted.

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture; ADR-019: Economy Resource Architecture; ADR-002: Client-Server Authority; ADR-008: Lightyear Channel Config
**ADR Decision Summary**: Placement submissions are authoritative server-side batches. The server validates phase, sender ownership, hand membership, duplicate card IDs, target legality, occupancy/spawn rules, and explicit current/reserve mana budgets before writing to `PendingPlacements`. Rejections are silent and all-or-nothing. Mana is deducted at PLACEMENT close, not at submit receipt.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Uses Bevy `MessageReader`/`MessageWriter` for internal handoff and Lightyear `MessageReceiver<C2SSubmitPlacement>` for network input. Use `liv-bevy-018` and `liv-bevy-lightyear` before implementation.

**Control Manifest Rules (Feature layer)**:
- Required: Placement validation is all-or-nothing per player batch.
- Required: Pending placements are plain Rust data; no ECS entity spawn during PLACEMENT.
- Required: Mana validation uses the Economy explicit split API from `ECO-007`.
- Required: Mana deduction happens at PLACEMENT close before `S2CPlacementReveal`.
- Forbidden: Never send an S2C rejection for invalid `C2SSubmitPlacement`.

---

## Readiness Gates Confirmed

This story is ready to open because:

- `production/epics/lightyear-protocol-verification/story-005-placement-payload-shape-split.md` is Complete and provides the split `C2SSubmitPlacement` / `S2CPlacementReveal` protocol payloads.
- `production/epics/economy-system/story-007-explicit-placement-mana-split-api.md` is Complete and provides `validate_explicit_mana_split` / `apply_explicit_mana_split`.

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md` and `design/gdd/network-protocol.md`, scoped to this story:*

- [x] **BL-35 / TR-BLS-011 - authority gate**: Given any `C2SSubmitPlacement`, when the server handles it, then the handler resolves sender `ClientId` to authoritative `PlayerId`, phase-gates to PLACEMENT, and silently discards unknown-sender or wrong-phase messages.

- [x] **BL-35 / TR-BLS-011 - hand ownership**: Given a submitted `card_id` that is not in the submitting player's authoritative `PlayerHands`, when validation runs, then the entire batch is silently discarded and `PendingPlacements[player].is_final` remains false.

- [x] **BL-35 / TR-BLS-011 - duplicate card IDs**: Given the same `card_id` appears more than once in one submitted batch, when validation runs, then the entire batch is silently discarded, no S2C response is sent, and no pending placement is written.

- [x] **BL-35 / TR-BLS-011 - target legality**: Given any submitted `PlayTarget` with lane/cell/objective/unit data outside the authoritative board constraints, when validation runs, then the entire batch is silently discarded.

- [x] **BL-35 / TR-BLS-011 - spawn and occupancy legality**: Given a minion outside current spawn range, an occupied personal minion slot, duplicate trap/structure/field occupancy, or any other Board/Lane placement rule failure, when validation runs, then the entire batch is silently discarded.

- [x] **BL-35 / TR-BLS-011 - explicit mana budget validation**: Given submitted entries with `current_mana_spend` / `reserve_mana_spend`, when validation runs, then Board/Lane calls Economy's explicit split validation for each card and the whole batch aggregate. Current and reserve overdraw each reject the full batch.

- [x] **BL-36 / TR-BLS-011 - accepted batch write**: Given all checks pass, when validation completes, then exactly one `PlayerSubmission` is written to `PendingPlacements` for the submitting player with `is_final = true`, preserving per-card explicit split values for PLACEMENT close.

- [x] **BL-36 / TR-BLS-011 - deduction at close**: Given accepted pending placements exist, when `close_placement_phase` runs, then it applies Economy explicit split deductions before `S2CPlacementReveal` is enqueued and before any ECS unit entity is spawned.

---

## Implementation Notes

*Derived from ADR-007 Implementation Guidelines and ADR-019 API boundaries:*

Recommended data flow:

```text
MessageReceiver<C2SSubmitPlacement>
  -> resolve sender player
  -> phase gate
  -> reject duplicate submit if is_final already true
  -> validate batch against PlayerHands/CardCatalog/BoardState/PlayerEconomies
  -> write PendingPlacements only if every check passes
```

Validation should be pure enough to unit test without a live Lightyear session. Keep the network receiver thin; the core validator should accept plain data and return a structured internal result used only for debug/test assertions. Production rejection remains silent.

Do not deduct mana in the submit handler. The accepted split values must remain in pending data until `close_placement_phase`, where the existing reveal-before-spawn order still applies:

```text
1. collect PendingPlacements
2. apply_explicit_mana_split for accepted entries
3. enqueue S2CPlacementReveal
4. spawn ECS unit entities and add replication
5. emit PlacementCommitted
6. clear PendingPlacements
```

## Performance Budget

Placement submit validation must remain bounded by the submitted batch size plus constant-time lookups in authoritative player hand, card catalog, board state, and economy resources. The submit handler runs in the normal server gameplay loop and must stay within the ADR-002 steady-state server budget of `<= 5 ms` per tick; `close_placement_phase` work remains part of the existing phase-close path and must not violate the ADR-017 RESOLUTION batch budget of `<= 15 ms`.

---

## Out of Scope

- Protocol struct split (`NP-005`)
- Economy explicit split API implementation (`ECO-007`)
- Client `PlayerEconomyView` (`PRES-002`)
- HAND-UI-010 client-side pre-validation
- Visual feedback for invalid placement attempts

---

## QA Test Cases

- **Duplicate card rejection**
  - Given: one `C2SSubmitPlacement` contains the same `card_id` twice
  - When: the submit handler runs in PLACEMENT
  - Then: no pending placement is written; `is_final` remains false; no S2C message is enqueued.

- **Explicit reserve overdraw rejection**
  - Given: submitted entries total `reserve_mana_spend > player.reserve_mana`
  - When: validation runs
  - Then: the full batch is rejected and no mana is deducted.

- **Accepted batch persists until close**
  - Given: valid submitted entries
  - When: submit handler runs
  - Then: pending data records card, target, current spend, and reserve spend; economy values are unchanged until `close_placement_phase`.

- **Close deducts before reveal**
  - Given: accepted pending placements
  - When: `close_placement_phase` runs
  - Then: explicit mana deductions occur before reveal enqueue and entity spawn in the same close sequence.

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- Unit: `tests/unit/board-lane-system/placement_submit_authority_validation_test.rs`
- Integration or evidence doc: `production/qa/evidence/placement-submit-authority-validation-evidence.md`

**Status**: [x] Created and passing

---

## Dependencies

- Readiness gates confirmed: `NP-005` Complete; `ECO-007` Complete; ADR-007/ADR-019/ADR-002/ADR-008 Accepted; `TR-BLS-011` active.
- Depends on: `production/epics/lightyear-protocol-verification/story-005-placement-payload-shape-split.md` is Complete and provides split placement protocol payloads.
- Depends on: `production/epics/economy-system/story-007-explicit-placement-mana-split-api.md` is Complete and provides explicit current/reserve mana validation and deduction APIs.
- Unlocks: `production/epics/hand-ui/story-010-submit-prevalidation.md` server-authority prerequisite.

---

## Completion Notes

**Completed**: 2026-05-05
**Verdict**: COMPLETE
**Criteria**: 8/8 passing; sender authority/phase gate, hand ownership, duplicate card rejection, target bounds, spawn/occupancy legality, explicit split validation, accepted pending write, and close-phase deduction ordering verified.
**Test Evidence**: `tests/unit/board-lane-system/placement_submit_authority_validation_test.rs` exists and `cargo test -p server --test placement_submit_authority_validation_test` passed 8/8. `production/qa/evidence/placement-submit-authority-validation-evidence.md` exists. Adjacent regression commands passed: `cargo test -p server --test placement_buffer_test` 3/3 and `cargo test -p server --test explicit_placement_mana_split_test` 6/6.
**Verification**: Current `main` includes worker commit `d2d16312db93205d81613387e3aade18cbebd732` via main integration commit `7f034b3`. `drain_submit_placement_messages` resolves trusted sender identity before writing internal placement submissions; `process_placement_submission` silently rejects wrong phase, unknown player, duplicate final submission, missing hand ownership, duplicate card IDs, invalid targets, spawn/occupancy failures, and current/reserve overdraw before writing `PendingPlacements`. Accepted batches preserve per-card explicit split values until `close_placement_phase`, where aggregate explicit mana is deducted before `S2CPlacementReveal` and before unit spawn.
**Regression Evidence**: `cargo fmt -p server -- --check`, `cargo check -p server`, `cargo check --workspace`, and `git diff --check` passed.
**Deviations**: None. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
**Tech Debt**: None logged.
**Sprint Status**: Unchanged; no matching BLS-011 row exists in `production/sprint-status.yaml`.
**Next Recommended**: HAND-UI-010 can be rechecked now that the BLS-011 server-authority prerequisite is complete.
