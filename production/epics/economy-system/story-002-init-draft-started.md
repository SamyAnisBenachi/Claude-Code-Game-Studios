# Story 002: Initialization and DraftStarted Subscriber

> **Epic**: Economy System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: `TR-ECO-02` (mana ramp), `TR-ECO-04` (gold income on DRAFT entry)

> ⚠️ TR-IDs are informal — `docs/architecture/tr-registry.yaml` is empty. Populate via `/architecture-review` before `/story-done`.

**ADR Governing Implementation**: [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md)
**ADR Decision Summary**: Economy subscribes to `DraftStarted { round, phase }` via `MessageReader<DraftStarted>`, scheduled `.after(advance_phase)`. Round 1 (`phase: Initial`) grants `starting_gold=5` (already set at init); subsequent rounds apply `baseline + interest`. `PlayerEconomies` is inserted at `SessionReady` via Bevy Observer trigger.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM
**Engine Notes**:
- `MessageReader::read()` — `EventReader` no longer exists in Bevy 0.17+. Use `MessageReader<DraftStarted>` from `server::core::rsm::events`.
- `SessionReady` is a Bevy Observer Event (`#[derive(Event)]`), not a buffered Message. Subscribe via `app.observe(on_session_ready)`, NOT `app.add_message::<SessionReady>()`.
- `app.add_message::<T>()` registration is required for all buffered message types.
- System scheduling: `on_draft_started.after(advance_phase)` — required or messages are missed.

**Control Manifest Rules (Core Layer)**:
- Required: `MessageReader::read()` for `DraftStarted` consumption (not `EventReader`)
- Required: `app.add_message::<DraftStarted>()` registration in plugin
- Required: `on_draft_started.after(advance_phase)` scheduling
- Required: `SessionReady` subscription via `app.observe(on_session_ready)` (Observer, not MessageReader)
- Forbidden: `EventWriter`, `EventReader`, `Events<T>`, `app.add_event::<T>()`

---

## Acceptance Criteria

*From GDD `design/gdd/economy-system.md`, Rules 2, 4, 6:*

- [ ] **EC12** `PlayerEconomies` resource inserted at `SessionReady` with one entry per player; each entry: `gold=5`, `current_mana=0`, `reserve_mana=0`, `mana_cap=config.mana_cap` (default 10)
- [ ] **EC12b** `InterestSnapshots` resource inserted at `SessionReady` as empty `HashMap`
- [ ] **EC-ROUND1** `DraftStarted { round:1, phase: Initial }` → `on_draft_started` applies mana ramp (`current_mana = min(1, mana_cap) = 1`); gold NOT incremented (remains at initialized value of 5; no baseline+interest on round 1)
- [ ] **EC-MANA-RAMP** `DraftStarted { round:3, phase: Shop }` → `current_mana = min(3, 10) = 3`
- [ ] **EC-MANA-CAP-RAMP** `DraftStarted { round:15, phase: Shop }`, `mana_cap=10` → `current_mana = 10` (capped)
- [ ] **EC-MANA-CAP-ELEVATED** `DraftStarted { round:15, phase: Shop }`, `mana_cap=12` → `current_mana = 12`
- [ ] **EC6 (event side)** `reserve_mana=7` before `DraftStarted` fires → `reserve_mana=7` unchanged after `on_draft_started` (mana ramp does not touch reserve). Setup: call `add_reserve(&mut economy, 7)` before event fires
- [ ] **EC15** `DraftStarted { round:2, phase: Shop }` with `InterestSnapshots[player]=8` and `gold=8` → `gold = 8 + floor(8/5) + 2 = 11`; `InterestSnapshots[player]` entry removed after processing (snapshot consumed exactly once)
- [ ] **EC-NO-SNAPSHOT** `DraftStarted { round:2, phase: Shop }` with no snapshot entry → `gold += 0 + gold_baseline_per_round`; no panic
- [ ] **EC-INTEREST-MAX** Snapshot=10 → interest=2 (maximum); `gold += 4` (2+2)
- [ ] **EC-S2C-COUNT** After `on_draft_started` runs for a 2-player world, exactly 2 `S2CGoldUpdate` messages enqueued (one per player), and exactly 2 `S2CGoldBroadcast` messages enqueued (checked via `MessageReader` drain count)
- [ ] **Integration round trace**: Simulate round 1→2→3 with pinned expected gold values:
  - R1: init `gold=5`; `DraftStarted { round:1, phase:Initial }` → `gold=5` (unchanged), `current_mana=1`
  - R1 end: snapshot `gold=5`; R2: `DraftStarted { round:2, phase:Shop }` → `gold = 5 + floor(5/5) + 2 = 8`, `current_mana=2`
  - R2 end: snapshot `gold=8`; R3: `DraftStarted { round:3, phase:Shop }` → `gold = 8 + floor(8/5) + 2 = 11`, `current_mana=3`
  - All assertions must pass; `current_mana = min(R, mana_cap)` confirmed at each DRAFT entry

---

## Implementation Notes

*From ADR-010 and EPIC.md scope:*

**File**: `server/src/core/economy/system.rs`

```rust
// on_draft_started signature (Bevy 0.18):
fn on_draft_started(
    mut economies: ResMut<PlayerEconomies>,
    mut snapshots: ResMut<InterestSnapshots>,
    config: Res<GameConfig>,
    session: Res<SessionConfig>,
    mut reader: MessageReader<DraftStarted>,
    mut gold_update: MessageWriter<S2CGoldUpdate>,
    mut gold_broadcast: MessageWriter<S2CGoldBroadcast>,
)
```

**Income formula per player** (for each `DraftStarted` message read):
1. `current_mana = min(round, mana_cap)` — always applies
2. If `phase == DraftPhase::Initial` (round 1): skip gold income (gold already initialized to 5)
3. Else: `interest = min(snapshots.0.remove(&player).unwrap_or(0) / config.interest_threshold_gold, config.interest_max_bonus)`; `apply_gold_award(&mut economy, config.gold_baseline_per_round + interest)`
4. Enqueue `S2CGoldUpdate { player, gold, current_mana, reserve_mana, mana_cap }` (unicast)
5. Enqueue `S2CGoldBroadcast { player, gold }` (broadcast)

**`on_session_ready` Observer** (called by Bevy Observer when GSS triggers `SessionReady`):
- Iterates `session.team_map.keys()` to insert `PlayerEconomy { gold: config.starting_gold, current_mana: 0, reserve_mana: 0, mana_cap: config.mana_cap, reserved_gold: 0 }` per player
- Inserts empty `InterestSnapshots`

**Plugin registration** in `EconomyPlugin::build()`:
```rust
app.add_message::<S2CGoldUpdate>();
app.add_message::<S2CGoldBroadcast>();
app.observe(on_session_ready);
app.add_systems(Update, on_draft_started.after(advance_phase));
```

**`HashMap::get` lookups**: use `.ok_or(SpendError::PlayerNotFound)` propagation, never `.unwrap()`.

---

## Out of Scope

*Handled by neighbouring stories:*

- [Story 001]: `PlayerEconomy` struct, all API functions, `SpendError` enum
- [Story 003]: `on_resolution_phase_entered` (takes interest snapshot); `discard_current_mana_at_resolution_end`
- [Story 004]: `handle_kill_award`, `handle_objective_award`
- [Story 006]: Lightyear dispatch of `S2CGoldUpdate`/`S2CGoldBroadcast` messages (enqueued here, dispatched there)

---

## QA Test Cases

*Written by qa-lead at story creation.*

- **EC12 — Initialization**
  - Given: `World::new()` with `PlayerEconomies::default()`, `SessionConfig { players: [A, B] }`, `GameConfig { starting_gold:5, mana_cap:10, .. }`
  - When: `SessionReady` observer fires; `on_session_ready` runs
  - Then: `PlayerEconomies[A]` = `{ gold:5, current_mana:0, reserve_mana:0, mana_cap:10, reserved_gold:0 }`; `PlayerEconomies[B]` same; `InterestSnapshots` is empty
  - Edge cases: single-player session → 1 entry

- **EC6 (event side) — reserve untouched by mana ramp**
  - Given: After init, `add_reserve(&mut economies.0[A], 7)` called directly; `DraftStarted { round:2, phase:Shop }` written
  - When: `on_draft_started` runs
  - Then: `PlayerEconomies[A].reserve_mana == 7`; `PlayerEconomies[A].current_mana == 2`
  - Edge cases: `reserve_mana=0` stays 0

- **EC15 — Round 2 gold income with interest**
  - Given: `PlayerEconomies[A].gold = 8`; `InterestSnapshots[A] = 8`; `GameConfig { gold_baseline_per_round:2, interest_threshold_gold:5, interest_max_bonus:2, .. }`; `DraftStarted { round:2, phase:Shop }` written
  - When: `on_draft_started` runs
  - Then: `PlayerEconomies[A].gold == 11`; `InterestSnapshots.0.contains_key(&A) == false` (consumed)
  - Edge cases: snapshot=4 → interest=0 → gold+=2; snapshot=10 → interest=2 → gold+=4

- **EC-ROUND1 — Round 1 Initial phase: no gold increment**
  - Given: `PlayerEconomies[A].gold = 5` (post-init); `InterestSnapshots` empty; `DraftStarted { round:1, phase:Initial }` written
  - When: `on_draft_started` runs
  - Then: `PlayerEconomies[A].gold == 5`; `PlayerEconomies[A].current_mana == 1`
  - Edge cases: confirm no double-income trap

- **EC-NO-SNAPSHOT — missing snapshot treated as interest=0**
  - Given: `PlayerEconomies[A].gold = 5`; `InterestSnapshots` empty; `DraftStarted { round:2, phase:Shop }`
  - When: `on_draft_started` runs
  - Then: `PlayerEconomies[A].gold == 7` (`5 + 0 + 2`); no panic
  - Edge cases: snapshot key present but value=0 → same result

- **Integration round trace (R1→R2→R3)**
  - Given: 1-player World; init `gold=5`, `mana_cap=10`
  - When: Run R1 DRAFT (round:1, Initial) → snapshot 5 → R2 DRAFT (round:2, Shop) → snapshot 8 → R3 DRAFT (round:3, Shop)
  - Then: After R1 DRAFT: `gold=5`, `current_mana=1`; After R2 DRAFT: `gold=8`, `current_mana=2`; After R3 DRAFT: `gold=11`, `current_mana=3`
  - Edge cases: Interest threshold boundary at round 2 (5g → interest=1); round 3 (8g → interest=1 again)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/economy/draft_started_test.rs` — must exist and pass
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 must be DONE (needs all API functions)
- Depends on: S1-04 (Protocol Skeleton + CI Gates) must be DONE — provides `DraftStarted`, `S2CGoldUpdate`, `S2CGoldBroadcast` types in `shared/src/protocol.rs` ✅ Done
- Unlocks: Story 003 (needs `PlayerEconomies` resource + `InterestSnapshots`)
