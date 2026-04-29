# Epic: Lightyear Protocol & Verification Spike

> **Layer**: Foundation
> **GDD**: design/gdd/network-protocol.md
> **Architecture Module**: `shared/src/protocol.rs` + `server/src/network/` + `client/src/network/`
> **Status**: Ready
> **Stories**: 4 stories created — see table below
> **Priority**: ⭐ SPRINT 1 STORY 1.0 — Highest-risk de-risking item (TD sign-off 2026-04-29)

## Overview

Verifies that Lightyear 0.26 is usable as specified across all 12 ADRs before any gameplay code is written against it. The Lightyear 0.26 API is entirely post-training-cutoff (released January 2026) — channel registration syntax, `MessageSender`/`MessageReceiver` system params, `NetworkTarget::Single` unicast shape, and `ReplicationState` vs message-based boundaries must all be confirmed against `docs.rs` before implementation can proceed safely. This epic populates `shared/src/protocol.rs` with all C2S/S2C message types from the GDD, wires the Lightyear server and client plugins, establishes a WebSocket connection in CI, and completes the 20-item verification checklist from `docs/architecture/control-manifest.md`.

**Story sequencing:** Epic 1 (`workspace-and-shared-types`) scaffolds the `shared/src/protocol.rs` skeleton with one no-op `S2CHeartbeat` message. This epic fills in all remaining message types. Epic 1 must be complete before this epic's stories begin.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-008: Lightyear Channel Config | Two channels only: `ReliableChannel` (all game-state) + `UnreliableChannel` (heartbeat, auction timer ticks); assignment permanent per message type | **HIGH** |
| ADR-003: Cargo Workspace Structure | `shared/src/protocol.rs` is the single registration site; `register_protocol(app)` called by both `server/main.rs` and `client/main.rs` | MEDIUM |

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from ADR-008 and network-protocol.md. Run `/architecture-review` to register stable IDs before stories are written.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-NP-RELIABLE | All game-state messages route over `ReliableChannel` (ordered, guaranteed delivery) | ADR-008 ✅ |
| TR-NP-UNRELIABLE | High-frequency updates (`C2SHeartbeat`, auction timer ticks) route over `UnreliableChannel` | ADR-008 ✅ |
| TR-NP-UNICAST | `NetworkTarget::Single(ClientId)` unicast for per-player secrets (HiddenObjectives, shop slots, gold) | ADR-008 ✅ (via ADR-001) |
| TR-NP-SYMM | Server and client use identical message type definitions from `shared/`; divergence is a compile error | ADR-003 ✅ |
| TR-NP-ALL29 | All 29 ACs from network-protocol.md (28 BLOCKING) — full message type coverage, channel routing verified | ADR-008 ✅ |

## Scope

### Deliverables

**`shared/src/protocol.rs` — complete population**

All C2S* and S2C* message types from `design/gdd/network-protocol.md` Table A, defined with `#[derive(Serialize, Deserialize, Debug, Clone)]`. Channel assignments per ADR-008:

*Client-to-Server (ReliableChannel unless noted):*
- `C2SCreateRoom { mode: GameMode }`, `C2SJoinRoom { room_code: String, requested_slot: u8 }`
- `C2SSelectClass { class_id: ClassId }`, `C2SConfirmClass { class_id: ClassId }`
- `C2SSubmitPlacement { placements: Vec<PlacementEntry> }`
- `C2SHeartbeat` (UnreliableChannel)
- All others from GDD Table A

*Server-to-Client (ReliableChannel unless noted):*
- `S2CRoomCreated { room_code: String }`, `S2CJoinAck`, `S2CSlotUpdated`, `S2CClassLocked`, `S2CClassesRevealed`
- `S2CPhaseChanged { phase: RoundPhase, timer_ms: u32 }`
- `S2CSessionReady { session_config: SessionConfigSnapshot }`, `S2CSessionCancelled { reason: CancelReason }`
- `S2CObjectiveIdentities { identities: Vec<ObjectiveIdentity> }` (unicast — ADR-001)
- `S2CShopSlots { slots: Vec<Option<CardId>> }` (unicast)
- `S2CGoldUpdate { gold: u32, mana: u32 }` (unicast)
- `S2CPlacementReveal { all_placements: Vec<PlacementEntry> }` (broadcast)
- `S2CGameOver { reason: GameOverReason }`
- `S2CHeartbeat` (UnreliableChannel)
- All others from GDD Table A

`pub fn register_protocol(app: &mut App)` — registers all types and channels. Exact API syntax verified against Lightyear 0.26 docs.rs during this spike.

**`server/src/network/`**
- `ServerNetworkPlugin` — configures Lightyear server transport (WebSocket, port from GameConfig or env var)
- `OnConnected` / `OnDisconnected` handler stubs
- `MessageReceiver<C2SCreateRoom>` and other receiver system stubs (one per C2S message type)

**`client/src/network/`**
- `ClientNetworkPlugin` — configures Lightyear client transport (WebSocket URL from env or config)
- `MessageSender<C2SCreateRoom>` and other sender stubs (one per C2S message type)

**Lightyear 0.26 verification checklist** (from `docs/architecture/control-manifest.md`, all 20 items)
- Items 1–12: ADR-008 channel/API checks — channel registration syntax, `MessageSender`/`MessageReceiver` type params, `NetworkTarget::Single(ClientId)` unicast shape, `ReplicationState` boundary, `reliable_ordered` guarantee confirmation
- Items 13–20: cross-ADR checks spanning ADR-011 (reconnect snapshot), ADR-012 (SessionReady observer ordering), ADR-007 (session lifecycle), and engine-reference confirms
- Each item: verify against current Lightyear 0.26 docs.rs + annotate with "CONFIRMED" or "DIFFERS — see note" in a verification report at `tests/evidence/lightyear-026-verification.md`

**End-to-end connection test** (CI)
- Server and client binaries connect over WebSocket localhost
- Client sends `C2SHeartbeat`; server receives it; server sends `S2CHeartbeat`; client receives it
- Test exits 0 if both directions confirmed; non-zero on any failure

## Definition of Done

- All C2S*/S2C* message types from network-protocol.md Table A defined in `shared/src/protocol.rs`
- `register_protocol(app)` compiles and registers all types on both server and client
- All 20 checklist items in `tests/evidence/lightyear-026-verification.md` annotated (CONFIRMED or DIFFERS with resolution)
- End-to-end WebSocket connection test passes in CI (client → server → client round-trip)
- ADR-012 open condition from session state acknowledged: before GSS+RSM integration story can be marked Ready, unit test for `Commands::trigger()` flush ordering for `SessionReady` Observer must pass; if test fails, add `apply_deferred` to `.chain()` in `RsmPlugin::build()`
- `S2CObjectiveIdentities` unicast path verified: `NetworkTarget::Single(ClientId)` compiles and routes correctly (ADR-001)

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 ⭐ | [Lightyear 0.26 Verification Spike](story-001-lightyear-026-verification-spike.md) | Integration | Ready | ADR-008, ADR-012 |
| 002 | [All Protocol Message Types](story-002-all-protocol-message-types.md) | Config/Data | Ready | ADR-008, ADR-003 |
| 003 | [Server & Client Network Plugins](story-003-server-client-network-plugins.md) | Integration | Ready | ADR-008, ADR-003 |
| 004 | [End-to-End WebSocket Round-Trip Test](story-004-e2e-websocket-roundtrip.md) | Integration | Ready | ADR-008 |

> Story sequence: 001 → 002 → 003 → 004 (linear chain).
> **Story 001 is a hard gate** — no other story in this epic or any Core/Feature networking epic may start until it is Done.

## Next Step

Run `/sprint-plan new` — all 4 foundation epics have stories. Sprint 1 Story 1.0 = `story-001-lightyear-026-verification-spike.md`.
