# Story 009: Register `S2CActivationRejected` + `ActivationRejectedReason` in Shared Protocol

> **Epic**: Lightyear Protocol & Verification Spike
> **Status**: Draft -- Sprint 18 candidate (`S18-PROTOCOL-S2CACTIVATIONREJECTED-REGISTER-001`); NOT activated
> **Layer**: Foundation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-18

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: NP-50 (wrong-phase rejection) + NP-55 (dispatcher no-op `S2CGoldUpdate` enforcement) protocol prerequisite. The two ACs cannot be implemented end-to-end while `S2CActivationRejected` and `ActivationRejectedReason` are absent from `shared/src/protocol.rs`. This story unblocks the rejection wire path only.
*(Requirement text lives in `design/gdd/network-protocol.md` Acceptance Criteria table NP-50, NP-55 and in the C2S/S2C tables and `ActivationRejectedReason` enum definition -- read fresh at review time.)*

**ADR Governing Implementation**: ADR-003: Cargo Workspace Structure; ADR-008: Lightyear Channel Config
**ADR Decision Summary**: Protocol message types live in `shared/src/protocol.rs` and are consumed by both client and server. `S2CActivationRejected` is a server-to-client reliable unicast message and must be added to `register_protocol(...)` on the existing `ReliableChannel`. No new channel is introduced. The enum `ActivationRejectedReason` is also part of the wire protocol -- it lives next to the message struct and is registered alongside.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: Use the Lightyear 0.26 message registration syntax already verified by Stories 001-004. No transport, plugin, or new channel work. The struct and enum are pure `serde` data; no Bevy components, no ECS hooks. This story uses the same `register_s2c::<M>(registry, ProtocolChannel::Reliable)` shape as the existing `S2CAuctionBidRejected` and `S2CPlacementRejected` neighbours.

**Control Manifest Rules (Foundation layer)**:
- Required: All C2S/S2C protocol types are defined once in `shared/src/protocol.rs` and registered through `register_protocol(...)`.
- Required: `S2CActivationRejected` is reliable + unicast (server only invokes it for the activating player); the registration call uses `ProtocolChannel::Reliable`.
- Required: Wire-protocol enums (`ActivationRejectedReason`) live next to their message struct in `shared/src/protocol.rs`; do not introduce a separate module.
- Forbidden: Do NOT implement the server card-activation dispatcher in this story. The `C2SActivateCard` drain in `server/src/network/mod.rs` remains log-only; no new `feature/activation/` module is created here.
- Forbidden: Do NOT implement the Hand UI `HandSlotState::ActivationLocked` drain or the `S2CActivationRejected` unlock path in `client/src/ui/hand/mod.rs` in this story. That work belongs to the Hand UI `story-012-activation-lock.md` follow-up (which remains BLOCKED until the future server dispatcher story lands).

---

> **Sequencing context (read before review):** `C2SActivateCard` is intentionally half-wired on `origin/main`. The client sends the message in DRAFT_SHOP (`client/src/ui/hand/mod.rs`), the server drains it as a `tracing::info!` log only (`server/src/network/mod.rs`), and there is no follow-on game-state mutation. This story lands the **protocol rejection path only** -- it does NOT close that half-wired gap. Two further stories are required (NOT scheduled here):
>
> 1. A future server card-activation dispatcher story that replaces the log-only drain with an authoritative dispatcher per NP-55 (sends `S2CGoldUpdate` automatically on every success path) and emits `S2CActivationRejected` on every rejection path (NP-50 + all `ActivationRejectedReason` variants).
> 2. The existing `production/epics/hand-ui/story-012-activation-lock.md` (Hand UI unlock path; remains BLOCKED until the server dispatcher exists).
>
> Reviewers MUST confirm this story does not regress that sequencing by quietly landing client drain code or server dispatcher code -- both are explicitly out of scope.

---

## Acceptance Criteria

*From GDD `design/gdd/network-protocol.md`, scoped to this story:*

- [ ] **NP-PROT-ACTREJ-1 — `S2CActivationRejected` defined**: `shared/src/protocol.rs` defines:

  ```rust
  pub struct S2CActivationRejected {
      pub entity_id: EntityId,
      pub reason: ActivationRejectedReason,
  }
  ```

  The struct derives `Serialize, Deserialize, Debug, Clone` (same shape as `S2CAuctionBidRejected` and `S2CPlacementRejected`). The field names `entity_id` and `reason` exactly match the GDD S2C table row (`network-protocol.md` C2/S2C table for `S2CActivationRejected`).

- [ ] **NP-PROT-ACTREJ-2 — `ActivationRejectedReason` enum**: `shared/src/protocol.rs` defines:

  ```rust
  pub enum ActivationRejectedReason {
      WrongPhase,
      EntityNotFound,
      NotInHand,
      InsufficientMana,
      InsufficientReserve,
      InvalidTarget,
      ActivationLimitReached,
  }
  ```

  Variant names and order match GDD `design/gdd/network-protocol.md` `enum ActivationRejectedReason` exactly (7 variants). The enum derives `Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq` (same derive shape as `BidRejectedReason`).

- [ ] **NP-PROT-ACTREJ-3 — registered on `ReliableChannel`**: `register_protocol(registry)` in `shared/src/protocol.rs` adds one new line:

  ```rust
  register_s2c::<S2CActivationRejected>(registry, ProtocolChannel::Reliable);
  ```

  alongside the existing S2C reliable registrations. No new channel is introduced. The registration is grouped with the other rejection messages (`S2CAuctionBidRejected`, `S2CPlacementRejected`).

- [ ] **NP-PROT-ACTREJ-4 — protocol-completeness invariant updates**: If a `protocol-completeness` integration / invariant test exists (per Story 007's invariant model) and lists registered C2S/S2C message types, this story updates that list to include `S2CActivationRejected` so the test continues to pass. If no such invariant test exists yet, this AC is N/A.

- [ ] **NP-PROT-ACTREJ-5 — round-trip serialization test**: A new test under `tests/integration/lightyear-protocol-verification/` (or alongside existing protocol roundtrip evidence) asserts that `S2CActivationRejected` round-trips through `bincode` (or whichever wire codec the existing protocol roundtrip tests use) for **every** `ActivationRejectedReason` variant -- 7 cases in total. Each case asserts the encoded message decodes to a struct equal to the input. No `unwrap()` in test bodies for production-style error handling; use `expect(...)` with a clear message.

- [ ] **NP-PROT-ACTREJ-6 — drain-gap not introduced**: The story MUST NOT touch any of:
  - `client/src/ui/hand/**` (no Hand UI drain)
  - `client/src/feature/**` (no new client feature module)
  - `server/src/network/mod.rs` (no replacement of the log-only `C2SActivateCard` drain)
  - `server/src/feature/**` (no new server activation dispatcher)

  Reviewers verify by `git diff --stat` against `origin/main` at story landing.

- [ ] **NP-PROT-ACTREJ-7 — half-wired note preserved**: After this story lands, `C2SActivateCard` remains end-to-end functionally inert (server drain is still log-only). This story unblocks the rejection-protocol wire path **only**; it does not deliver any user-visible activation behaviour and it does not close the audit finding PROMPT-1297 F1 (the server-side dead-handler). Story closure notes MUST state this explicitly so future producers do not mistake this for a fix of the broader activation gap.

---

## Implementation Notes

*Derived from ADR-003, ADR-008, and the Network Protocol GDD:*

The change is localised to `shared/src/protocol.rs`:

```rust
// Inside register_protocol(registry: &mut impl ProtocolRegistry):
//   ... existing s2c registrations ...
register_s2c::<S2CActivationRejected>(registry, ProtocolChannel::Reliable);
//   ... S2CPlacementRejected and neighbours unchanged ...

// Near the other S2C reject types and BidRejectedReason/PlacementRejectedReason:
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationRejectedReason {
    WrongPhase,
    EntityNotFound,
    NotInHand,
    InsufficientMana,
    InsufficientReserve,
    InvalidTarget,
    ActivationLimitReached,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct S2CActivationRejected {
    pub entity_id: EntityId,
    pub reason: ActivationRejectedReason,
}
```

Naming caveats:
- Use `InsufficientReserve` (not `InsufficientReserveMana`) -- this matches the GDD enum verbatim. The audit report PROMPT-1297 §1.5 listed the variant as `InsufficientReserveMana`; that label was a paraphrase, not the GDD definition. Cross-check against `design/gdd/network-protocol.md` enum block at the time of implementation.
- 7 variants exactly. Do NOT add a `CardSpecificFailure` variant -- it does not exist in the GDD enum and PROMPT-1297 §1.1 mentioning 8 variants was a transcription artefact.

Lightyear 0.26 registration shape was verified in Stories 001-002 (`tests/evidence/lightyear-026-verification.md`). No new verification work is needed for this story.

Server and client systems that will eventually consume `MessageReceiver<S2CActivationRejected>` / `MessageSender<S2CActivationRejected>` are **out of scope**. After this story lands, `cargo check -p shared`, `cargo check -p server`, and `cargo check -p client` MUST all still pass without any new sender/receiver query being added.

---

## Out of Scope

*Handled by neighbouring or future stories -- do not implement here:*

- **Server activation dispatcher** (NP-50 + NP-55 implementation). A future `card-activation` epic (or `card-acquisition` sub-epic) story will replace the log-only `C2SActivateCard` drain in `server/src/network/mod.rs` with an authoritative dispatcher. Slug placeholder: `S19-CARD-ACTIVATION-DISPATCHER-001`. Until that story lands, `C2SActivateCard` produces no game-state effect.
- **Hand UI activation-lock UI** (`production/epics/hand-ui/story-012-activation-lock.md` HU-28, HU-28b, HU-29). The Hand UI drain of `S2CActivationRejected` and the unlock path belong to that story and remain BLOCKED until the server dispatcher exists.
- **`CardType::Instant` honour-or-drop decision**. PROMPT-1297 §3 captured the broader F-02 question (honour the activation contract vs drop it). That decision belongs to producer + game-designer + lead-programmer and is explicitly NOT made by this story. This story only unblocks the protocol wire so the honour-the-contract option remains viable.
- **DRAFT_INITIAL activation surface** (PROMPT-1297 F5). The GDD lists `C2SActivateCard` as valid in DRAFT_INITIAL + DRAFT_SHOP, but no client surface emits it from DRAFT_INITIAL today. This story does not add or remove that gap.
- **Documentation revisions** to `design/gdd/hand-ui.md` OQ8 wording. The hand-ui GDD revision is owned by game-designer + network-programmer; this story does not edit `design/**`.
- **GDD revision to `ActivationRejectedReason` shape**. The enum is taken as-given. Any variant rename or addition (e.g. `CardSpecificFailure`) is a separate GDD revision.

---

## QA Test Cases

- **Compile coverage**
  - Given: `shared/src/protocol.rs` after this story lands
  - When: `cargo check -p shared`, `cargo check -p server`, and `cargo check -p client` run
  - Then: all three compile cleanly. The protocol registration adds exactly one new `register_s2c::<S2CActivationRejected>(...)` call.

- **Wire round-trip per variant**
  - Given: a `S2CActivationRejected { entity_id, reason }` constructed for each of the 7 `ActivationRejectedReason` variants (use a distinct `entity_id` per variant so a swap regression would be caught)
  - When: the message is serialised via `bincode::serialize(...)` and deserialised via `bincode::deserialize(...)` (or whichever codec the existing protocol roundtrip tests use)
  - Then: the decoded value `PartialEq`-equals the input. 7 BLOCKING test cases, one per variant.

- **Registration grep**
  - Given: the worker branch diff against `origin/main`
  - When: searching for `register_s2c::<S2CActivationRejected>` and `pub struct S2CActivationRejected` and `pub enum ActivationRejectedReason`
  - Then: each pattern matches exactly once inside `shared/src/protocol.rs` and zero times anywhere else.

- **Drain-gap not introduced**
  - Given: the worker branch diff
  - When: `git diff --stat origin/main -- client/src/ui/hand/ server/src/network/ server/src/feature/ client/src/feature/`
  - Then: zero lines changed. Confirms no Hand UI drain and no server dispatcher leaked into this story.

- **Half-wired `C2SActivateCard` invariant**
  - Given: the server module after this story lands
  - When: inspecting `server/src/network/mod.rs::receive_c2s_messages` (or whichever function handles `C2SActivateCard`)
  - Then: the body is still log-only (`tracing::info!`) with no `MessageWriter` / `EventWriter` / state mutation. ADVISORY -- this is a tripwire to catch accidental scope creep; not a behavioural test.

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**:
- `cargo check -p shared`, `cargo check -p server`, `cargo check -p client` -- all pass.
- New roundtrip test file under `tests/integration/lightyear-protocol-verification/` (filename to be chosen at implementation time; suggested: `s2c_activation_rejected_roundtrip_test.rs`).
- Grep evidence captured in `production/qa/evidence/s2c-activation-rejected-register-evidence.md`.

**Status**: [ ] Not yet created (story is Draft, Sprint 18 candidate, NOT activated).

---

## Dependencies

- Depends on: Story 002 (All Protocol Message Types) complete -- the registration scaffold and `register_protocol` flow this slots into. Already complete on `origin/main`.
- Unlocks (sequencing only -- this story does NOT activate them):
  - Future `card-activation` epic story `S19-CARD-ACTIVATION-DISPATCHER-001` (server NP-55 dispatcher).
  - `production/epics/hand-ui/story-012-activation-lock.md` HU-28 / HU-28b (still requires the dispatcher story landing first; this story only removes the **protocol** half of its blocker).

---

## Open Questions

| # | Question | Owner | Notes |
|---|---|---|---|
| OQ1 | Does an active `protocol-completeness` invariant test already exist on `origin/main`? Story 007 was authored as a Sprint 13 candidate and may not be activated. | Lead-programmer | If the test exists, NP-PROT-ACTREJ-4 must update its expected message list; otherwise the AC is N/A. Resolve at story-readiness time, not at story authoring. |
| OQ2 | Should the roundtrip test live alongside Story 005's evidence harness, or as a fresh `s2c_activation_rejected_roundtrip_test.rs` next to it? | Network-programmer | Either is acceptable; pick the placement that keeps the integration test runner discovery rules happy. Resolve at implementation time, not at story authoring. |

---

## Notes for the future card-activation epic author

When `S19-CARD-ACTIVATION-DISPATCHER-001` is authored, it MUST cite this story as a prerequisite and MUST NOT re-add the `S2CActivationRejected` registration -- doing so will collide with the registration this story lands. The dispatcher story is expected to:

1. Replace the log-only `C2SActivateCard` drain in `server/src/network/mod.rs` with a dispatcher that:
   - On success: automatically emits `S2CGoldUpdate` (no-op confirmation if no economy change) per NP-55. Card handlers MUST NOT emit `S2CGoldUpdate` themselves.
   - On rejection: emits `S2CActivationRejected { entity_id, reason }` with the appropriate `ActivationRejectedReason` per NP-50.
2. Cover all 7 `ActivationRejectedReason` variants with one BLOCKING integration test each.
3. Coordinate the dispatcher's plugin-register with the Hand UI activation-lock UI story (Story 012) so the unlock path lands in the same wave or the immediately following wave.
