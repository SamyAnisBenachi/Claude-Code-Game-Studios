# Story 008: Reconnect Snapshot Rebuild

> **Epic**: HUD
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-009`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-011: Reconnect + Snapshot](docs/architecture/adr-011-reconnect-snapshot.md)
**ADR Decision Summary**: `S2CGameSnapshot` triggers a full HUD rebuild in `MessageDrain` set — all 18 pre-pooled entities are written (no despawn/respawn). Snapshot bypasses `FROZEN` mode (if HUD is in FROZEN state, rebuild still runs, then FROZEN re-applies). After rebuild, HUD drains the deferred message queue normally.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `MessageReceiver<S2CGameSnapshot>` (Lightyear) — single drain, `MessageDrain` set. Rebuild is a synchronous multi-entity write within a single system body. All entity handles from `HudEntities` resource — no entity query needed if handles are pre-cached.

**Control Manifest Rules (Foundation + Presentation Layer)**:
- Required: `S2CGameSnapshot` always wins over FROZEN mode (snapshot rebuild runs, then FROZEN re-applies). Rebuild is synchronous within one system body. Pre-pooled entities reused — no spawn/despawn.
- Forbidden: Never send `S2CGameSnapshot` as broadcast (server rule — N/A here, but HUD must not request it).
- Guardrail: Snapshot rebuild must complete within the same frame — all zones populated before next render.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [x] **HUD-13** (BLOCKING): GIVEN HUD in any state (partially updated, stale, or mid-round), WHEN `S2CGameSnapshot` is received, THEN every HUD zone reflects the snapshot values within the same frame; no entity is despawned or re-spawned (pre-pooled entities reused).
- [ ] **HUD-14** (ADVISORY): GIVEN a player reconnects mid-match, WHEN `S2CGameSnapshot` is received, THEN no frame is observed where any zone shows a blank or stale value; confirmed by screenshot at the reconnect moment.
- [x] **HUD-27** (BLOCKING): GIVEN HUD in FROZEN mode (`S2CPhaseChanged(GAME_OVER)` received), WHEN `S2CGameSnapshot` arrives (simulating reconnect at GAME_OVER), THEN: (a) full rebuild runs from snapshot; (b) after rebuild, `HudMode == Frozen` immediately; (c) all label values reflect snapshot state; (d) a subsequent `S2CGoldUpdate` with a different gold value does NOT alter `GoldDisplayState.gold`.

---

## Implementation Notes

*Derived from ADR-021 and ADR-011 Implementation Guidelines:*

- `handle_snapshot_system` in `MessageDrain` set: drains `MessageReceiver<S2CGameSnapshot>`. For each snapshot received (reconnect may deliver multiple; last wins), synchronously write all 18 entities:
  1. Own gold: `GoldDisplayState{gold: snap.own.gold as f32, reserved_gold: snap.own.reserved_gold as f32, is_populated: true}`
  2. Opponent gold: `GoldDisplayState{gold: snap.opponent.gold as f32, reserved_gold: snap.opponent.reserved_gold as f32, is_populated: true}`
  3. Mana: numerator + denominator + reserve visibility
  4. Phase label text (derive from `snap.phase` using the Rule 5 mapping)
  5. Round counter text: `format!("R{}", snap.round_number)`
  6. All 10 dot states: set each `DotState` from `snap.objective_states[player][lane]`
  7. `HudMode`: set from `snap.phase` (LOBBY → Hidden, DRAFT_AUCTION → EconomyAuction, GAME_OVER → Frozen, else EconomyBasic)
  8. HUD root visibility: set from derived mode
- FROZEN bypass: snapshot handler does NOT check `HudMode` before running — it always rebuilds. After rebuild, `HudMode` is set to `Frozen` if `snap.phase == GAME_OVER`. The FROZEN gate in `handle_gold_update_system` and the dot Observer only gate incremental updates, not snapshot rebuilds.
- HIDDEN mode exception: if snapshot arrives before any `S2CPhaseChanged` (cold start), rebuild applies all values AND transitions HUD out of HIDDEN using `snap.phase`.
- Deferred queue: After snapshot rebuild, normal `MessageDrain` processing resumes. Any messages buffered during reconnect are drained from the deferred queue after `snapshot_sent = true` (server-side contract per ADR-011). HUD processes them normally after the snapshot.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 007]: FROZEN mode definition and incremental-update gate
- [Story 004]: Dot state machine and Observer (snapshot writes dot state directly, bypassing Observer)

---

## QA Test Cases

*Written by qa-lead at story creation.*

**HUD-13**: Full snapshot rebuild
  - Given: HUD in any state (partially updated, arbitrary `GoldDisplayState` values)
  - When: `S2CGameSnapshot{ own:{gold:20, mana:6, mana_cap:10, reserve_mana:0, reserved_gold:0}, opponent:{gold:15, reserved_gold:0}, phase:Placement, round:7, objective_states: [[false,false,true,false,false],[false,false,false,false,false]] }` processed
  - Then: Own gold `Text == "20g"`; opponent gold `Text == "15g"`; mana `Text == "6 / 10"`; `HudMode == EconomyBasic`; round counter `Text == "R7"`; `dots[0][2] == Destroyed`; all other dots `Alive`; no entity despawned
  - Edge cases: Snapshot with `phase = DraftAuction` → both gold labels switch to AUCTION_FORMAT; snapshot with `phase = GameOver` → `HudMode == Frozen` after rebuild

**HUD-27**: Snapshot bypasses FROZEN
  - Given: `HudMode == Frozen`; `GoldDisplayState.gold == 12.0`; dots in final state
  - When: `S2CGameSnapshot{own:{gold:5,...}, ..., phase:GameOver}` processed
  - Then: (a) Rebuild runs — own gold `GoldDisplayState.gold == 5.0`; (b) `HudMode == Frozen` after rebuild; (c) subsequent `S2CGoldUpdate{gold=999}` → `GoldDisplayState.gold` remains 5.0 (FROZEN gate active)
  - Edge cases: Snapshot arrives 3 frames after GAME_OVER — same result

**HUD-14 (Advisory — manual)**: No blank frames on reconnect
  - Setup: Mid-match reconnect scenario; snapshot contains all fields populated
  - Verify: Screenshot taken at the reconnect moment shows all 4 zones populated
  - Pass condition: No zone shows `"--g"` or blank text in the reconnect-instant screenshot

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/hud/reconnect_snapshot_rebuild_test.rs` — must exist and pass. Advisory: `production/qa/evidence/reconnect-snapshot-evidence.md` (screenshot).

**Status**: [x] Created and passing (`cargo test -p client --test reconnect_snapshot_rebuild_test`). Advisory screenshot evidence not yet created.

---

## Dependencies

- Depends on: Story 001 (entity pool), Story 002 (GoldDisplayState), Story 003 (phase label), Story 004 (dot state — snapshot writes dot state directly), Story 007 (FROZEN mode — snapshot bypasses it)
- Unlocks: None (final integration story for reconnect path)

## Completion Notes

**Completed**: 2026-05-03
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 2/3 directly covered and passing; HUD-13 and HUD-27 are covered by `tests/integration/hud/reconnect_snapshot_rebuild_test.rs`. HUD-14 remains advisory and untested because `production/qa/evidence/reconnect-snapshot-evidence.md` does not exist.
**Test Evidence**: `cargo test -p client --test reconnect_snapshot_rebuild_test` passed 3/3. `cargo check -p client` passed. `cargo fmt -p client -- --check` passed.
**Verification**: `client/src/ui/hud/mod.rs` drains `S2CGameSnapshot` through `MessageReceiver<S2CGameSnapshot>` into `HudGameSnapshotMessage`, handles the last snapshot in `HudSystemSet::MessageDrain`, writes cached `HudEntities` without respawning, updates mode/visibility/phase/round/gold/mana/dots, and runs before incremental gold and objective handlers.
**Notes**: Advisory only - HUD-14 screenshot/manual evidence is still missing. Advisory only - story references `TR-HUD-009`, whose current registry text focuses on FROZEN mode; the broader snapshot rebuild requirement remains present in current `design/gdd/hud.md` Rule 13 and the HUD-13/HUD-27 acceptance criteria. Lean mode skipped external QA/code-review gates.
**Tech Debt**: None logged.
**Sprint Status**: Unchanged per user instruction; no explicit `HUD-008` row exists in `production/sprint-status.yaml`.
**Next Recommended**: Create `production/qa/evidence/reconnect-snapshot-evidence.md` for HUD-14 visual sign-off, then continue the serialized closure queue with Hand UI Story 006 PLACEMENT Drag Highlights (`production/epics/hand-ui/story-006-placement-drag-highlights.md`) after readiness check.
