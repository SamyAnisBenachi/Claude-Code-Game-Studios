# Epic: Economy System

> **Layer**: Core
> **GDD**: design/gdd/economy-system.md
> **Architecture Module**: `server/core/economy/` (full module — `state.rs`, `api.rs`, `system.rs`, `plugin.rs`)
> **Status**: Ready
> **Stories**: To be created — see Story Breakdown Hint below

## Overview

Implements the per-player currency state and the public spend/award API consumed by every other gameplay system. This epic owns `PlayerEconomy { gold, current_mana, reserve_mana, mana_cap }` per player, the gold interest snapshot taken at RESOLUTION end, the mana ramp formula `current_mana = min(round_number, mana_cap)` applied on DRAFT entry, the gold-reservation accounting that prevents double-spend during active auction bids, and the auto-split spending allocation (current first, reserve as overflow) for normal cards plus the strict reserve-only path for cards with "costs from reserve" text. The Economy System exposes a small, single-writer API — `validate_spend`, `apply_spend`, `apply_gold_award`, `reserve_gold`, `release_gold_reservation`, `increment_mana_cap`, `add_reserve` — and `PlayerEconomy` is mutated ONLY through this API. Economy never self-triggers: it subscribes to `DraftStarted` (Epic 1's RSM event bus) for the mana-ramp + gold-income sequence, and to `ResolutionPhaseEntered` for the interest snapshot timing. It emits no phase events of its own — only `S2CGoldUpdate` (unicast to owner) and `S2CGoldBroadcast` (gold totals are publicly visible per GDD Rule 6) for client-state mirror updates.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-010: RSM Phase Event Bus | Economy is the canonical `DraftStarted` subscriber; reads `phase: DraftPhase` field to select between `starting_gold = 5` (Initial) and `baseline + interest` (Auction/Shop); F2 emission ordering guarantees Economy runs before Card Pool's `ShopRefreshNeeded` subscriber | MEDIUM |

## Engine Risk: MEDIUM

Lower than RSM/GSS because Economy uses standard Bevy patterns: a `Resource` for state, `EventReader::read()` for subscribing to RSM events, plain function-call API for spend/award, `EventWriter::write()` for `S2CGoldUpdate` enqueue. The risks:

1. **`EventReader::read()` not `.iter()`** — `iter()` was renamed in Bevy 0.16. `liv-bevy-018` enforces this on every reader.
2. **`EventWriter::write()` not `.send()`** — `send()` was removed in Bevy 0.16.
3. **Subscriber ordering** — Economy's `DraftStarted` subscriber must run `.after(advance_phase)` so events written in the current frame are visible. This is the Epic 1 contract; Economy enforces it in its plugin scheduling.

`liv-bevy-018` skill is mandatory on every `.rs` file. `liv-bevy-lightyear` is mandatory wherever `S2CGoldUpdate` send code lives (the network dispatch system).

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from the GDD Acceptance Criteria.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-ECO-01 | Three independent currency pools per player: `current_mana`, `reserve_mana`, `gold` | ADR-010 (`PlayerEconomy` is mutated only via Economy API) |
| TR-ECO-02 | Mana ramp on DRAFT entry: `current_mana = min(round_number, mana_cap)` | ADR-010 ✅ (`DraftStarted` subscriber) |
| TR-ECO-03 | Mana cap default 10, max 12 (via fake-objective rewards); permanent for session, takes effect next DRAFT | GDD Rule 5; `increment_mana_cap` API |
| TR-ECO-04 | Gold income on DRAFT entry: `gold += baseline + interest`; round 1 grants `starting_gold = 5` instead | ADR-010 ✅ (`DraftStarted { phase: Initial }` branch) |
| TR-ECO-05 | Interest snapshot at RESOLUTION end: `interest = min(floor(g / 5), 2)` | ADR-010 ✅ (`ResolutionPhaseEntered` subscriber takes snapshot before `ResolutionComplete` fires) |
| TR-ECO-06 | Spend validation: `current_mana + reserve_mana >= cost` before accepting card play; auto-split current-first | GDD Rule 4 |
| TR-ECO-07 | Reserve-only cards: `reserve_mana >= cost`; `current_mana` does not substitute | GDD Rule 4 |
| TR-ECO-08 | Auction reservation: `reserve_gold` lowers shop-purchase ceiling; `release_gold_reservation` on outbid; `apply_spend(from_reserve=false)` on auction win | GDD Rule 7 (auction hand-full + reservation) |

## Scope

### Deliverables

**`server/src/core/economy/state.rs`**
- `PlayerEconomy { gold: u32, current_mana: u32, reserve_mana: u32, mana_cap: u32, reserved_gold: u32 }` — `#[derive(Clone, Debug)]`. Not directly a `Resource` — held inside `PlayerEconomies(HashMap<PlayerId, PlayerEconomy>)` resource so all per-player state is accessible from one borrow.
- `PlayerEconomies` resource: `HashMap<PlayerId, PlayerEconomy>`. Initialised on `SessionReady` (Epic 2) with one entry per player from `SessionConfig.team_map.keys()`. Per GDD EC12: `gold = 5, current_mana = 0, reserve_mana = 0, mana_cap = GameConfig.mana_cap` at init.
- `InterestSnapshots(HashMap<PlayerId, u32>)` resource — written at RESOLUTION end, read at next DRAFT entry, cleared after consumption.
- `SpendError` enum: `InsufficientFunds | ReserveOnlyButCurrentProvided | HandFull | WrongPhase` — returned by validation functions.

**`server/src/core/economy/api.rs`** (the public single-writer API)

```rust
pub fn validate_spend(
    economy: &PlayerEconomy,
    cost: u32,
    from_reserve_only: bool,
) -> Result<(), SpendError>;

pub fn apply_spend(
    economy: &mut PlayerEconomy,
    cost: u32,
    from_reserve_only: bool,
);   // auto-split if from_reserve_only=false; reserve-only deduction otherwise

pub fn apply_gold_award(economy: &mut PlayerEconomy, amount: u32);
pub fn add_reserve(economy: &mut PlayerEconomy, amount: u32);     // no cap; +1 from prism, +n from Xelor

pub fn reserve_gold(economy: &mut PlayerEconomy, amount: u32) -> Result<(), SpendError>;
pub fn release_gold_reservation(economy: &mut PlayerEconomy, amount: u32);

pub fn increment_mana_cap(economy: &mut PlayerEconomy, config: &GameConfig);  // capped at config.mana_cap_max (12); takes effect next DRAFT

pub fn can_afford_bid(economy: &PlayerEconomy, amount: u32) -> bool;
    // = (gold - reserved_gold) >= amount

pub fn can_afford_shop(economy: &PlayerEconomy, cost: u32) -> bool;
    // = (gold - reserved_gold) >= cost

pub fn discard_current_mana(economy: &mut PlayerEconomy);  // RESOLUTION end; current_mana = 0
```

**Invariants enforced by the API:**
- `PlayerEconomy` is mutated ONLY through these functions. Code review gate: `grep -r "economy\." server/src/feature/ | grep -E "\.(gold|current_mana|reserve_mana|mana_cap|reserved_gold) =" ` returns zero matches outside `economy/api.rs`.
- `reserved_gold` never exceeds `gold` (panic in debug if violated).
- `mana_cap` never exceeds `GameConfig.mana_cap_max` (default 12).
- All u32 arithmetic uses `saturating_sub` to prevent underflow.

**`server/src/core/economy/system.rs`**
- `on_draft_started` — `EventReader<DraftStarted>` subscriber. For each event:
  1. For all players in `Res<SessionConfig>.team_map`:
     - `apply_mana_ramp(player, round_number)` → `current_mana = min(round_number, mana_cap)`
     - If `phase == DraftPhase::Initial` AND `round_number == 1`: gold is already `5` from initialisation; no income applied this DRAFT entry.
     - Else: read `Res<InterestSnapshots>[player]`; compute `interest = min(floor(snap / interest_threshold_gold), interest_max_bonus)`; `apply_gold_award(player, gold_baseline_per_round + interest)`. Clear the snapshot entry.
  2. Enqueue `S2CGoldUpdate { gold, current_mana, reserve_mana, mana_cap }` unicast per player on `ReliableChannel`.
  3. Enqueue `S2CGoldBroadcast { player, gold }` broadcast (gold is publicly visible per GDD Rule 6).
- `on_resolution_phase_entered` — takes interest snapshot AT THE END of resolution. Implementation: the system runs `.after(combat_resolution_complete_marker)` (the marker is established by Combat Resolution in M2; for M1 we use a placeholder system label that runs after Objective System's destruction processing). Snapshot reads `gold` for each player and writes to `InterestSnapshots`. The snapshot is what `on_draft_started` consumes at next DRAFT entry — never recomputed from "current" gold.
- `discard_current_mana_at_resolution_end` — for each player: `current_mana = 0`. Runs in the same system step as the snapshot.
- `handle_kill_award` — `EventReader<UnitKilled>` (from Combat Resolution M2 — type defined now): `apply_gold_award(killer, kill_gold_reward)`; enqueue `S2CGoldUpdate` + `S2CGoldBroadcast`.
- `handle_objective_award` — `EventReader<ObjectiveDestroyed>` (from Objective System): if `attacker != target`: `apply_gold_award(attacker, objective_gold_reward)`; if `was_fake` and the RNG-rolled reward is "mana_cap +1": `increment_mana_cap(attacker)`. Self-inflicted objective damage (EC11) does NOT award gold.
- `handle_card_play_spend` — called by Board/Lane System on placement commit (NOT by this epic — Board/Lane calls `apply_spend` after `validate_spend` returns Ok). Documented contract; no system here.

**`server/src/core/economy/plugin.rs`**
- `EconomyPlugin`: registers `PlayerEconomies`, `InterestSnapshots`; subscribes `on_draft_started` `.after(advance_phase)` (Epic 1 ordering contract); subscribes resolution-end systems.

**Network dispatch wiring**
- A system in `server/src/network/` reads `EventReader<S2CGoldUpdate>` and sends unicast on `ReliableChannel` via `MessageSender<S2CGoldUpdate>` to the owning player.
- A system reads `EventReader<S2CGoldBroadcast>` and sends broadcast on `ReliableChannel`.
- Both message types are defined in `shared/src/protocol.rs` (`workspace-and-shared-types` Foundation epic).

**Tests**
- `tests/unit/economy/` — every Acceptance Criterion EC1–EC26 from the GDD has a passing unit test using `World::new()` + `PlayerEconomies` resource + manual `DraftStarted` event injection. No live RSM, no live Lightyear.
- Auto-split coverage: EC1, EC2, EC3, EC4 — auto-split, current-only, reserve-overflow, rejection.
- Reserve-only restriction: EC5.
- Reserve persistence across rounds: EC6.
- Gelure no-op + transfer: EC7, EC8 (Gelure spell calls `add_reserve(player, current_mana); current_mana = 0` — implemented in the spell handler, but the Economy's transactional contract is tested here).
- Mana cap increment: EC9, EC10 (saturate at 12).
- Self-inflicted objective: EC11 — no gold awarded.
- Initialisation: EC12.
- Interest formula: EC13, EC14, EC15 (8g → +1 interest → 11g after baseline).
- Kill / objective awards: EC16, EC17.
- Mana discard at RESOLUTION end: EC18.
- Auction bid validation + reservation: EC21, EC22, EC23.
- Refresh cost escalation: EC24, EC25, EC26 (this requires a per-DRAFT-phase refresh counter — the counter is owned by Card Data & Pool epic; Economy validates the dynamic cost passed in).
- Integration test: complete round 1 → 2 → 3 trace, asserting `current_mana = min(R, 10)` at each DRAFT entry, `gold` evolves correctly through interest, `S2CGoldUpdate` unicast at each DRAFT.

### Out of Scope (owned by other epics)

- `RoundState`, `advance_phase`, `DraftStarted` event definition: Epic 1 — Round State Machine. This epic only subscribes.
- `SessionConfig` (player roster, class_map): Epic 2 — Game Session System. Used as read-only `Res<SessionConfig>`.
- `GameConfig` field definitions and loading: `game-config-pipeline` Foundation epic.
- Card definitions, mana cost values per card: `workspace-and-shared-types` Foundation epic + Epic 4 — Card Data & Pool.
- Shop refresh cost escalation counter: Epic 4 — Card Data & Pool owns the counter; Economy only deducts the dynamic amount it's passed.
- Auction state machine, bid acceptance: M2 — Auction System.
- Combat resolution and `UnitKilled` event source: M2 — Combat Resolution.
- Objective destruction and `ObjectiveDestroyed` event source: Objective System epic.
- Class System spell handlers (Gelure, Miss Nuit reserve gain): M3 — Class System. Those handlers call `add_reserve` / `apply_spend`.

### Implementation Notes

**Interest snapshot timing precision** — The snapshot is taken AFTER all kill rewards and objective rewards have fired during RESOLUTION, but BEFORE `ResolutionComplete` is emitted (which the RSM reads to transition to next DRAFT). This timing is the F4 contract from the GDD: snapshot reflects "gold held at end of RESOLUTION" inclusive of all RESOLUTION-phase awards. Implementation pattern: a system labelled `EconomySystemSet::ResolutionEnd` runs `.after(ObjectiveSystemSet::ProcessDestructions)` and `.after(CombatSystemSet::ProcessKills)` and `.before(ResolutionCompleteEmitter)`. For M1, Combat is not implemented; for M1 acceptance, the snapshot system can run on a synthetic `ResolutionPhaseEntered` test event with manual gold setup.

**Refresh cost escalation owner clarification** — The GDD specifies "first refresh = 1g, second = 2g, etc., counter resets at DRAFT start". The counter (`ManualRefreshCount<PlayerId>`) is owned by Epic 4 — Card Data & Pool because it's intrinsic to the shop, not the economy. Card Pool's refresh handler reads the counter, computes cost, calls `validate_spend` + `apply_spend` on the Economy, then increments the counter. EC24/EC25/EC26 tests cross both epics; their implementation is split — Economy epic asserts `validate_spend` + `apply_spend` work for any cost; Card Pool epic asserts the counter behaviour. This epic ships the half it owns.

**`reserved_gold` and shop visibility** — Per GDD edge case "Reserved gold and shop purchases", `can_afford_shop` returns `(gold - reserved_gold) >= cost`. The client's HUD shows `gold` (the public value); client-side it is acceptable for shop card affordability to update reactively as bids land. The server is the only authority on the `(gold - reserved_gold)` ceiling.

**No `unwrap()`** — All `HashMap::get(player_id)` lookups use `.ok_or(SpendError::PlayerNotFound)` propagation, never `.unwrap()`. A missing entry indicates a bug (player not initialised at `SessionReady`); production handles it as a no-op with debug log.

## Definition of Done

- All deliverables above implemented and passing.
- All BLOCKING Acceptance Criteria EC1–EC26 from `economy-system.md` have passing unit tests in `tests/unit/economy/`.
- All Acceptance Criteria E1–E11 from the master GDD §8 (referenced from `lanes-and-lies-gdd.md`) that touch Economy have passing tests.
- `cargo check --workspace` green; zero warnings on `server/src/core/economy/**`.
- CI grep gate: direct field mutation outside `economy/api.rs` returns zero matches:
  `grep -rE "economy\.(gold|current_mana|reserve_mana|mana_cap|reserved_gold)\s*=" server/src/ | grep -v "core/economy/"` returns zero matches.
- CI grep gate: `grep -rE "EventWriter::send|\.iter\(\)" server/src/core/economy/ | grep -E "Event(Reader|Writer)"` returns zero matches.
- An integration test simulates round 1 → 2 → 3 with a fixed gold/mana trace and asserts every value at every DRAFT entry matches GDD formulas (Formula 1, 2, 3, 4).
- An integration test demonstrates `S2CGoldUpdate` unicast and `S2CGoldBroadcast` broadcast both fire on every DRAFT entry, with consistent values.
- `EconomyPlugin` registers cleanly in a headless Bevy `App` startup test.

## Story Breakdown Hint

Suggested decomposition (final story list to be authored via `/create-stories`):

1. **State + API scaffold** (Config/Data + Logic) — `state.rs`, `api.rs`, all single-writer functions; unit tests EC1–EC11 (auto-split, reserve-only, mana cap, self-inflicted) — pure-function tests, no Bevy app needed.
2. **Initialisation + DraftStarted subscriber** (Logic) — `PlayerEconomies` initialised on `SessionReady`; `on_draft_started` for round 1 (Initial) and round ≥ 2 (Auction/Shop with interest); tests EC12, EC15, EC18, integration round 1 → 2 trace.
3. **Interest snapshot + RESOLUTION end** (Logic) — `on_resolution_phase_entered` snapshot; mana discard; tests EC13, EC14.
4. **Awards (kill, objective, mana_cap)** (Logic) — `handle_kill_award`, `handle_objective_award`; tests EC16, EC17, plus mana_cap reward path.
5. **Auction reservation + bid validation** (Logic) — `reserve_gold`, `release_gold_reservation`, `can_afford_bid`, hand-full check; tests EC21, EC22, EC23.
6. **Network dispatch wiring** (Integration) — `S2CGoldUpdate` unicast, `S2CGoldBroadcast` broadcast; `liv-bevy-lightyear` mandatory; integration test asserts the right messages on the right channel with the right targets.

## Next Step

Run `/create-stories production/epics/economy-system/EPIC.md` to author the story files. Story 1 (State + API) can begin in parallel with Epic 1's Story 1 (RSM scaffold) — they are independent. Story 2 onward is gated on Epic 1's `DraftStarted` event being defined in `server/core/rsm/events.rs`.
