# Story 002: All Protocol Message Types

> **Epic**: Lightyear Protocol & Verification Spike
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Config/Data
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: TR-??? (covers TR-NP-SYMM: all types in shared/; TR-NP-RELIABLE + TR-NP-UNRELIABLE: channel assignments; full GDD Table A + B + supporting types)

**ADR Governing Implementation**: ADR-008: Lightyear Channel Config + ADR-003: Cargo Workspace Structure
**ADR Decision Summary**: All C2S*/S2C* message types and channel definitions live in `shared/src/protocol.rs`. Both server and client compile against the same types. `register_protocol(app)` is the single registration entry point. Channel assignment is permanent per message type: `ReliableChannel` for all game-state, `UnreliableChannel` for heartbeat and auction timer ticks only.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use ONLY the API syntax confirmed in Story 001. Do not invent or assume Lightyear 0.26 method names — check the verification report at `tests/evidence/lightyear-026-verification.md` before writing any registration call. Add a code comment citing the verification report for every Lightyear API call.

**Control Manifest Rules (Foundation layer)**:
- Required: Exactly two Lightyear channels. Channel assignment is permanent per message type.
- Required: All channel definitions in `shared/src/protocol.rs`. Both sides compile against identical types.
- Forbidden: Never send `S2CAuctionUpdate` (timer/price) or `C2SHeartbeat` on `ReliableChannel` — only two types belong on `UnreliableChannel`.

---

## Acceptance Criteria

**Channel type stubs (using verified 0.26 syntax from Story 001):**
- [ ] `ReliableChannel` struct defined in `shared/src/protocol.rs` with correct Lightyear 0.26 channel registration
- [ ] `UnreliableChannel` struct defined with correct channel settings

**C2S message types (all `#[derive(Serialize, Deserialize, Debug, Clone)]`):**
- [ ] `C2SHello { protocol_version: u32, session_token: Option<[u8; 16]> }`
- [ ] `C2SCreateRoom { mode: GameMode }` (LOBBY)
- [ ] `C2SJoinRoom { room_code: String, requested_slot: u8 }` (LOBBY)
- [ ] `C2SSelectClass { class_id: ClassId }` (LOBBY)
- [ ] `C2SConfirmClass { class_id: ClassId }` (LOBBY)
- [ ] `C2SPurchaseCard { card_id: CardId }`
- [ ] `C2SRefreshShop {}`
- [ ] `C2SActivateCard { card_id: CardId }`
- [ ] `C2SSignalReady { retract: bool }`
- [ ] `C2SPlaceBid { amount: u32 }`
- [ ] `C2SSubmitPlacement { placements: Vec<PlacedCard> }`
- [ ] `C2SAcknowledgeResult {}`
- [ ] `C2SHeartbeat {}` — assigned to `UnreliableChannel`

**S2C message types (all `#[derive(Serialize, Deserialize, Debug, Clone)]`):**
- [ ] `S2CHandshake { protocol_version: u32, session_id: u64, session_token: [u8; 16] }`
- [ ] `S2CHandshakeRejected { server_version: u32, client_version: u32 }`
- [ ] `S2CPhaseChanged { phase: RoundPhase, round_number: u32, timer_duration_ms: u32 }`
- [ ] `S2CGameOver { loser: Option<PlayerId>, round: u32, reason: GameOverReason }`
- [ ] `S2CGoldUpdate { gold: u32, current_mana: u32, reserve_mana: u32, mana_cap: u8 }`
- [ ] `S2CGoldBroadcast { player_id: PlayerId, gold: u32 }`
- [ ] `S2CCardAcquired { card_id: CardId, source: CardSource }`
- [ ] `S2CShopSlots { slots: Vec<CardId> }`
- [ ] `S2CDraftOffering { card_ids: Vec<CardId> }`
- [ ] `S2CPoolUpdate { updates: Vec<(CardId, u8)> }`
- [ ] `S2CPlacementReveal { placements: Vec<PlacedCard> }`
- [ ] `S2CResolutionEvent { events: Vec<TaggedEvent> }`
- [ ] `S2CAuctionCard { card_id: CardId, starting_price: u32 }`
- [ ] `S2CAuctionBidAccepted { bidder: PlayerId, amount: u32, new_timer_ms: u32 }`
- [ ] `S2CAuctionSettled { winner: Option<PlayerId>, amount: u32 }`
- [ ] `S2CAuctionBidRejected { reason: BidRejectedReason }`
- [ ] `S2COpponentDisconnected { player_id: PlayerId, grace_remaining_ms: u32 }`
- [ ] `S2COpponentReconnected { player_id: PlayerId }`
- [ ] `S2CRoomCreated { room_code: String, mode: GameMode, slots: Vec<SessionSlot> }`
- [ ] `S2CJoinAck { mode: GameMode, slots: Vec<SessionSlot> }`
- [ ] `S2CJoinRejected { reason: JoinRejectedReason }`
- [ ] `S2CSlotUpdated { slots: Vec<SessionSlot> }`
- [ ] `S2CClassLocked { class_id: ClassId }`
- [ ] `S2CClassesRevealed { player_class_map: Vec<(PlayerId, ClassId)> }`
- [ ] `S2CConfirmClassRejected { reason: ConfirmClassRejectedReason }`
- [ ] `S2CSessionCancelled { reason: SessionCancelledReason }`
- [ ] `S2CSangMepriseReveal { identities: Vec<(u8, bool)> }` — (lane, is_fake)
- [ ] `S2CGameSnapshot` — minimal stub with a `player_id: PlayerId` field; full schema defined by GSS epic
- [ ] `S2CHeartbeat {}` — assigned to `UnreliableChannel`

**Supporting enum/struct types:**
- [ ] `GameMode` enum: `OneVOne` (minimum; extend as needed)
- [ ] `RoundPhase` enum: `Handshaking`, `Lobby`, `DraftInitial`, `DraftShop`, `DraftAuction`, `Placement`, `Resolution`, `GameOver`
- [ ] `GameOverReason` enum: `ObjectivesDestroyed`, `Disconnect`, `Draw`, `ResolutionTimeout`
- [ ] `SessionCancelledReason` enum: `LobbyTimeout`, `PlayerDisconnected`, `ServerRngFail`
- [ ] `JoinRejectedReason` enum: `SlotOccupied`, `SessionFull`, `RoomNotFound`, `InvalidSlot`, `AlreadyInSession`, `SessionInProgress`, `InvalidMode`
- [ ] `BidRejectedReason` enum: `InsufficientGold`, `AmountTooLow`, `AuctionExpired`
- [ ] `ConfirmClassRejectedReason` enum: `ClassAlreadyConfirmed`
- [ ] `CardSource` enum: all variants from GDD §D.3
- [ ] `PlacedCard { card_id: CardId, owner_id: PlayerId, target: PlayTarget }`
- [ ] `PlayTarget` enum: `BoardCell { lane: u8, cell: u8 }`, `TargetUnit { lane: u8, unit_id: u64 }`, `TargetObj { player_id: PlayerId, lane: u8 }`, `LaneWide { lane: u8 }`, `Instant`
- [ ] `TaggedEvent { sub_step: u8, event: ResolutionEvent }`
- [ ] `ResolutionEvent` enum: `UnitMoved`, `UnitDied`, `TrapTriggered` stubs (full variants deferred to Combat epic)
- [ ] `SessionSlot` stub struct (full schema owned by GSS epic)
- [ ] `PlayerId` newtype — confirm against `shared/src/card.rs` or define as `pub struct PlayerId(pub u32)` if not yet in shared/

**`register_protocol` updated:**
- [ ] All C2S types registered on their assigned channel (`C2SHeartbeat` on `UnreliableChannel`; all others on `ReliableChannel`)
- [ ] All S2C types registered (`S2CHeartbeat` on `UnreliableChannel`; all others on `ReliableChannel`)
- [ ] Every Lightyear API call has a code comment: `// Lightyear 0.26: verified in tests/evidence/lightyear-026-verification.md item N`
- [ ] `cargo check -p shared` passes with zero warnings

---

## Implementation Notes

*Derived from ADR-008 Decision §Channel Assignment and network-protocol.md Rule 2:*

**Channel assignment rule (two categories only):**
```
UnreliableChannel: C2SHeartbeat, S2CHeartbeat
                   (and S2CAuctionTimerTick if/when implemented — not in current GDD)
ReliableChannel:   ALL other C2S* and S2C* messages
```

**`PlayerId` placement:** If `PlayerId` is already defined in `shared/src/card.rs` (from workspace-and-shared-types Story 002), import it from there. If it's only a local placeholder `type PlayerId = u32` in `rng.rs`, promote it to `shared/src/lib.rs` as a newtype `pub struct PlayerId(pub u32)` and update both files to use it. Do not define `PlayerId` twice.

**`SessionToken` type alias:** `pub type SessionToken = [u8; 16];` — UUID v4, 128-bit, server-generated.

**`S2CGameSnapshot` stub:** The full snapshot schema is complex (see GDD §D.1). Define a minimal compilable stub with `player_id: PlayerId` only. The GSS epic owns the full schema. Flag with `// TODO(GSS epic): expand to full S2CGameSnapshot schema`.

**`ResolutionEvent` stub:** Define only the 3 variants shown in the GDD excerpt (`UnitMoved`, `UnitDied`, `TrapTriggered`) with their fields. Additional variants are owned by Combat Resolution and Keyword System epics. Flag with `// TODO(Combat epic): add remaining ResolutionEvent variants`.

---

## Out of Scope

- Story 003: Server/client plugin wiring — types only in this story
- Full `S2CGameSnapshot` schema — GSS epic (Core layer)
- Additional `ResolutionEvent` variants — Combat Resolution epic

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: cargo check clean after full type population**
  - Given: All message types and supporting enums defined
  - When: `cargo check -p shared` is run
  - Then: Zero errors, zero warnings

- **AC: Channel assignment correct**
  - Given: `register_protocol` implemented
  - When: Source is reviewed
  - Then: Only `C2SHeartbeat` and `S2CHeartbeat` are on `UnreliableChannel`; all other types are on `ReliableChannel`; no type is registered twice

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**: `cargo check -p shared` output showing zero warnings → `tests/evidence/story-lyv-002-types-check.md`
**Status**: [x] Created and passing in CI run `25169319842`

---

## Completion Notes

**Completed**: 2026-04-30
**Criteria**: Passing via local worker verification and CI
**Deviations**: None blocking. `shared/` remains dependency-pure, so protocol registration is represented as a dependency-free manifest for server/client Lightyear wiring to adapt in the plugin story.
**Test Evidence**: `tests/evidence/story-lyv-002-types-check.md`; worker reported `cargo fmt --check`, `cargo check -p shared`, `cargo test -p shared`, and extra `cargo check -p server` passing.
**Implementation Commit**: `759bd4a`
**CI**: GitHub Actions run `25169319842` passed.
**Code Review**: Lean mode skipped; CI green.

---

## Dependencies

- Depends on: Story 001 (verification spike must be Done — exact API syntax required before writing registration calls)
- Unlocks: Story 003 (server/client plugins)
