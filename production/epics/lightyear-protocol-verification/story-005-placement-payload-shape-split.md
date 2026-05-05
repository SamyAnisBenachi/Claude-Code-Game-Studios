# Story 005: Placement Payload Shape Split

> **Epic**: Lightyear Protocol & Verification Spike
> **Status**: Ready
> **Layer**: Foundation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: `TR-NP-013`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*

**ADR Governing Implementation**: ADR-003: Cargo Workspace Structure; ADR-008: Lightyear Channel Config; ADR-002: Client-Server Authority; ADR-007: Placement Buffer and Simultaneous Reveal Architecture
**ADR Decision Summary**: Protocol message types live in `shared/src/protocol.rs` and are consumed by both client and server. `C2SSubmitPlacement` travels on `ReliableChannel`, expresses player intent only, and must not reuse the S2C reveal payload shape. The server remains authoritative and validates the full batch before writing to `PendingPlacements`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use the Lightyear 0.26 message registration syntax already verified by Stories 001-004. No new channel is introduced; this is a payload shape change on the existing reliable C2S/S2C message types.

**Control Manifest Rules (Foundation layer)**:
- Required: All C2S/S2C protocol types are defined once in `shared/src/protocol.rs` and registered through `register_protocol(app)`.
- Required: `C2SSubmitPlacement` remains on `ReliableChannel`.
- Required: Submit and reveal placement payloads are distinct types; do not reuse a shared `PlacedCard` struct for both directions.
- Forbidden: Do not expose placement mana split fields in `S2CPlacementReveal`.

---

## Acceptance Criteria

*From GDD `design/gdd/network-protocol.md`, scoped to this story:*

- [ ] **NP-58 / TR-NP-013 - C2S submit payload**: `C2SSubmitPlacement` uses `placements: Vec<PlacedCardSubmit>`, where each submit entry contains:
  - `card_id: CardId`
  - `target: PlayTarget`
  - `current_mana_spend: u32`
  - `reserve_mana_spend: u32`

- [ ] **NP-58 / TR-NP-013 - S2C reveal payload**: `S2CPlacementReveal` uses `placements: Vec<PlacedCardReveal>`, where each reveal entry contains owner/card/target data needed for rendering and combat setup, but omits `current_mana_spend` and `reserve_mana_spend`.

- [ ] **NP-58 / TR-NP-013 - no ambiguous shared placement type**: `shared/src/protocol.rs` no longer uses one `PlacedCard` type for both `C2SSubmitPlacement` and `S2CPlacementReveal`. Any internal server placement struct is separate from protocol structs.

- [ ] **NP-58 / TR-NP-013 - explicit split invariant**: `PlacedCardSubmit.current_mana_spend + PlacedCardSubmit.reserve_mana_spend` is documented as the intended total payment for that card. Server validation of the invariant is out of scope for this story and owned by Board/Lane Story 011.

- [ ] `register_protocol(app)` registers the renamed/split placement types without adding any new Lightyear channel.

---

## Implementation Notes

*Derived from ADR-003, ADR-008, and the amended Network Protocol GDD:*

Use direction-specific names:

```rust
pub struct C2SSubmitPlacement {
    pub placements: Vec<PlacedCardSubmit>,
}

pub struct PlacedCardSubmit {
    pub card_id: CardId,
    pub target: PlayTarget,
    pub current_mana_spend: u32,
    pub reserve_mana_spend: u32,
}

pub struct S2CPlacementReveal {
    pub placements: Vec<PlacedCardReveal>,
}

pub struct PlacedCardReveal {
    pub owner_id: PlayerId,
    pub card_id: CardId,
    pub target: PlayTarget,
}
```

`PlacedCardSubmit` is player intent. `PlacedCardReveal` is authoritative reveal data. The server may define a third internal struct for `PendingPlacements`; that struct belongs to Board/Lane Story 011.

Keep compatibility fallout explicit: every compiler error caused by the rename should be resolved by updating call sites to the correct direction-specific type, not by adding aliases that keep the ambiguous `PlacedCard` usage alive.

---

## Out of Scope

- Economy validation or deduction (`ECO-007`)
- Server authority validation and pending-buffer writes (`BLS-011`)
- Client-side economy mirror (`PRES-002`)
- HAND-UI-010 submit pre-validation
- Any gameplay behavior change beyond protocol type shape

---

## QA Test Cases

- **Protocol compile coverage**
  - Given: `shared/src/protocol.rs` after the split
  - When: `cargo check -p shared` runs
  - Then: both `C2SSubmitPlacement` and `S2CPlacementReveal` compile and register on `ReliableChannel`.

- **Type separation grep**
  - Given: protocol source after implementation
  - When: searching for `C2SSubmitPlacement { placements: Vec<PlacedCard> }` or `S2CPlacementReveal { placements: Vec<PlacedCard> }`
  - Then: no production-code occurrence remains.

- **Reveal privacy check**
  - Given: `PlacedCardReveal` definition
  - When: fields are inspected
  - Then: no `current_mana_spend`, `reserve_mana_spend`, or legacy `reserve_amount` field exists in the reveal payload.

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**:
- `cargo check -p shared`
- Grep evidence in `production/qa/evidence/placement-payload-shape-split-evidence.md`

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Lightyear Protocol Story 002 complete; current protocol registration scaffold present.
- Unlocks: `production/epics/board-lane-system/story-011-placement-submit-authority-validation.md`; HAND-UI-010 protocol prerequisite.
