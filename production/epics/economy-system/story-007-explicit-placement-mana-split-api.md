# Story 007: Explicit Placement Mana Split API

> **Epic**: Economy System
> **Status**: Complete
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: `TR-ECO-009`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*

**ADR Governing Implementation**: ADR-019: Economy Resource Architecture; ADR-002: Client-Server Authority
**ADR Decision Summary**: `PlayerEconomies` is the server-only authority for gold/current/reserve mana. All field mutations go through `server/src/core/economy/api.rs`. Existing auto-split spend APIs remain valid for non-placement card plays; placement requires an explicit current/reserve split that mirrors `PlacedCardSubmit`.

**Engine**: Bevy 0.18 | **Risk**: LOW
**Engine Notes**: This is pure Rust API work in `server/src/core/economy/api.rs` plus unit tests. No Lightyear receiver/sender is needed in this story. Use `liv-bevy-018` for any Bevy `Resource` imports touched by tests.

**Control Manifest Rules (Core layer)**:
- Required: All `PlayerEconomy` field mutations go through `economy/api.rs`.
- Required: Explicit placement split validation checks current and reserve pools independently.
- Required: Existing auto-split `validate_spend` / `apply_spend` behavior remains unchanged for non-placement calls.
- Forbidden: Do not make Board/Lane assign `economy.current_mana` or `economy.reserve_mana` directly.

---

## Acceptance Criteria

*From GDD `design/gdd/economy-system.md`, scoped to this story:*

- [ ] **EC27 / TR-ECO-009 - explicit split validation succeeds**: Given `current_mana = 3`, `reserve_mana = 2`, and a card cost of `5`, when `validate_explicit_mana_split(economy, cost=5, current=3, reserve=2)` is called, then validation succeeds.

- [ ] **EC27 / TR-ECO-009 - current overdraw rejected**: Given `current_mana = 2`, `reserve_mana = 5`, and cost `5`, when the requested split is `current=3`, `reserve=2`, then validation returns a current-mana insufficiency error and neither pool is modified.

- [ ] **EC27 / TR-ECO-009 - reserve overdraw rejected**: Given `current_mana = 5`, `reserve_mana = 1`, and cost `5`, when the requested split is `current=3`, `reserve=2`, then validation returns a reserve-mana insufficiency error and neither pool is modified.

- [ ] **EC27 / TR-ECO-009 - split sum must equal cost**: Given any economy state, when `current + reserve != cost`, then validation rejects the spend as an invalid split before inspecting pool affordability.

- [ ] **EC28 / TR-ECO-009 - exact deduction**: Given validation succeeded for `current=3`, `reserve=2`, when `apply_explicit_mana_split(economy, current=3, reserve=2)` is called, then `current_mana` decreases by exactly 3 and `reserve_mana` decreases by exactly 2.

- [ ] Existing `validate_spend` and `apply_spend` unit tests still pass, proving normal non-placement auto-split behavior is unchanged.

---

## Implementation Notes

*Derived from ADR-019 Key Interfaces and the amended Economy GDD:*

Add explicit placement APIs alongside the existing auto-split APIs:

```rust
pub fn validate_explicit_mana_split(
    economy: &PlayerEconomy,
    cost: u32,
    current_mana_spend: u32,
    reserve_mana_spend: u32,
) -> Result<(), SpendError>;

pub fn apply_explicit_mana_split(
    economy: &mut PlayerEconomy,
    current_mana_spend: u32,
    reserve_mana_spend: u32,
);
```

Recommended `SpendError` additions:

```rust
InvalidManaSplit,
InsufficientCurrentMana,
InsufficientReserveMana,
```

`validate_explicit_mana_split` is the only API Board/Lane should call for `C2SSubmitPlacement` validation. `apply_explicit_mana_split` is called at PLACEMENT close, after the batch has already been accepted. Do not fold this into the existing auto-split API; placement must preserve the player's explicit reserve allocation from the submit payload.

Zero-cost cards are valid only with `current_mana_spend = 0` and `reserve_mana_spend = 0`.

---

## Out of Scope

- Protocol payload shape (`NP-005`)
- Board/Lane C2S handler, duplicate card rejection, target validation, pending-buffer writes (`BLS-011`)
- Client `PlayerEconomyView` (`PRES-002`)
- Hand UI validation gate (`HAND-UI-010`)

---

## QA Test Cases

- **Explicit split valid**
  - Given: `current_mana = 3`, `reserve_mana = 2`, `cost = 5`
  - When: validate and apply explicit split `current=3`, `reserve=2`
  - Then: validation returns `Ok(())`; after apply, both mana pools are 0.

- **Invalid sum rejected**
  - Given: `cost = 5`
  - When: validate split `current=4`, `reserve=0`
  - Then: `Err(InvalidManaSplit)` and economy values are unchanged.

- **Pool-specific rejection**
  - Given: only one pool is insufficient
  - When: validation runs
  - Then: the returned error identifies the deficient pool.

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/economy/explicit_placement_mana_split_test.rs` must exist and pass
- Existing economy API regression tests must pass

**Status**: [x] Exists and passes (`cargo test -p server --test explicit_placement_mana_split_test`)

---

## Dependencies

- Depends on: Economy Story 001 complete (`PlayerEconomy` and API scaffold).
- Unlocks: `production/epics/board-lane-system/story-011-placement-submit-authority-validation.md`; HAND-UI-010 server-validation prerequisite.

---

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 6/6 passing.
**Verification**: Current `main` includes worker commit `b1c678f3400f4f7c8047adffd3f6d0b775bb5c29` and integration commit `dacc7d38bc300f89ac13b689e5abbb17f69a1fed`. `validate_explicit_mana_split` accepts exact current/reserve split requests, rejects invalid sums before affordability checks, rejects current and reserve overdraw independently without mutation, and supports zero-cost cards only with a zero split. `apply_explicit_mana_split` deducts the submitted current and reserve amounts exactly, without auto-split recomputation. Existing `validate_spend` / `apply_spend` auto-split behavior remains covered by the economy API tests.
**Test Evidence**: `cargo test -p server --test explicit_placement_mana_split_test` passed 6/6. `cargo test -p server economy::api::tests` passed for the matching economy API tests. Adjacent economy regressions passed 26/26 across `auction_reservation_test`, `economy_draft_subscriber_test`, `economy_interest_snapshot_test`, `economy_round_trace_test`, and `economy_network_dispatch_test`. `cargo check -p server` passed. `git diff --check` passed.
**Deviations**: None blocking. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
**Code Review**: Lean mode; QL-TEST-COVERAGE and LP-CODE-REVIEW gates skipped.
**Tech Debt**: None logged.
