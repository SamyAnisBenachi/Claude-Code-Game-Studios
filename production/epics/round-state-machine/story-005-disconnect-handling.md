# Story 005: Disconnect Handling

> **Epic**: Round State Machine
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/round-state-machine.md`
**Requirement**: TR-RSM-10
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-009 (RSM Phase State as ECS Resource)
**ADR Decision Summary**: `disconnect_trackers: HashMap<PlayerId, f32>` is a field on `RoundState`. Evaluated in a single pass per tick (mutual-disconnection invariant: both breach same tick → Draw). Mid-RESOLUTION deferral: if the grace timer breaches during RESOLUTION, GAME_OVER is deferred until RESOLUTION exits naturally; `AbortAuction` is emitted before GAME_OVER if phase is DRAFT_AUCTION. Grace duration from `GameConfig`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Lightyear `OnDisconnected` and `OnConnected` events — these are Lightyear-specific connection events, not standard Bevy events. Their exact type names and subscription pattern in Lightyear 0.26 must be verified with `liv-bevy-lightyear` skill before implementation. `liv-bevy-lightyear` skill is MANDATORY on all files in this story. `liv-bevy-018` skill also mandatory.

**Control Manifest Rules (Core layer)**:
- Required: `disconnect_trackers` field lives on `RoundState` (not a separate resource).
- Required: Single-pass evaluation per tick: all trackers are evaluated in one loop before any GAME_OVER decision — prevents split-brain on mutual disconnect.
- Required: Mid-RESOLUTION deferral: if GAME_OVER condition is detected while `phase == Resolution`, set a `pending_disconnect_game_over` flag; transition to GAME_OVER only after `ResolutionComplete` is processed.
- Required: `AbortAuction` event emitted before transitioning to GAME_OVER when `phase == DraftAuction`.
- Required: `disconnect_grace_seconds` sourced from `GameConfig`, not hardcoded (default: 30.0).
- Forbidden: Hardcoded `30.0` or any literal disconnect timeout value in `server/src/core/rsm/`.

---

## Acceptance Criteria

- [ ] `server/src/core/rsm/system.rs` defines `tick_disconnect_timers` system: subscribes to Lightyear `OnDisconnected` events; on disconnect, sets `rsm.disconnect_trackers.insert(player_id, 0.0)`; on reconnect (`OnConnected`), removes the player's entry from `disconnect_trackers`; ticks all active tracker values by `time.delta_secs()` each frame
- [ ] `tick_disconnect_timers` performs a single-pass evaluation each tick: after ticking all trackers, collects all players whose value exceeds `config.disconnect_grace_seconds` in one pass; determines outcome (single loser or Draw) from the collected set before modifying any state (RSM-37 invariant: both breach same tick → exactly one `GameOverEmitted { reason: Draw, loser: None }`)
- [ ] Single disconnect breach (one player exceeds grace): `GameOverEmitted { reason: Disconnection, loser: Some(player_id) }` emitted; `advance_phase` transitions to GAME_OVER
- [ ] Mutual disconnect breach (both players exceed grace in the same tick): exactly one `GameOverEmitted { reason: Draw, loser: None }` emitted; no double-transition
- [ ] Mid-RESOLUTION deferral: if disconnect grace is breached while `rsm.phase == Resolution`, do NOT immediately emit `GameOverEmitted`; instead set `rsm.pending_disconnect_outcome = Some(GameOverEmitted { ... })`; after `ResolutionComplete` is processed by `rsm_input_reader` (Story 003/004), check `pending_disconnect_outcome` before win condition evaluation; if set, emit the deferred `GameOverEmitted` and skip win condition (disconnect result takes precedence); `OnResolutionEnd` equivalent fires before GAME_OVER (RSM-35)
- [ ] DRAFT_AUCTION disconnect: emit `AbortAuction` event before transitioning to GAME_OVER; `AbortAuction` is defined in `server/src/core/rsm/events.rs` (or already defined in Story 001 if included in the catalog)
- [ ] `RoundState` gains a `pending_disconnect_outcome: Option<GameOverEmitted>` field (or equivalent deferred flag) — initialised to `None` in `RoundState::new()`
- [ ] `GameConfig` struct has `disconnect_grace_seconds: f32` field with default value `30.0` in `assets/config/game_config.ron`
- [ ] `tick_disconnect_timers` is registered in `RsmPlugin` and scheduled in the `Update` set; ordering: runs before `rsm_input_reader` so that a disconnect-triggered GAME_OVER is visible to `rsm_input_reader` in the same tick
- [ ] `tests/unit/rsm/rsm_disconnect_test.rs` passes all tests listed in the QA Test Cases section (RSM-23 through RSM-25, RSM-35, RSM-37)

---

## Implementation Notes

*Derived from ADR-009 and GDD Rule 13:*

**Lightyear `OnDisconnected` / `OnConnected` subscription:** In Lightyear 0.26, connection events are typically delivered as Bevy `Events<T>` or via dedicated Lightyear system parameters. The exact type names (`DisconnectedEvent`, `ClientDisconnected`, etc.) and the `PlayerId` extraction pattern must be verified against the Lightyear 0.26 API before writing any code. Activate `liv-bevy-lightyear` skill — it enforces correct Lightyear 0.26 API patterns. Do NOT assume Lightyear 0.14 or pre-0.26 event names.

**Single-pass evaluation pseudocode:**
```rust
fn tick_disconnect_timers(
    mut rsm: ResMut<RoundState>,
    // TODO(S1-05): verify Lightyear 0.26 disconnect event type — see liv-bevy-lightyear skill.
    // Lightyear 0.26 uses entity-per-connection model; disconnect may be an Observer trigger.
    // mut disconnected: MessageReader</* verify: Lightyear 0.26 disconnect type */>,
    // mut reconnected: MessageReader</* verify: Lightyear 0.26 reconnect type */>,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut game_over_writer: MessageWriter<GameOverEmitted>,
    mut abort_auction_writer: MessageWriter<AbortAuction>,
    mut advance: /* trigger mechanism */,
) {
    // 1. Update tracker map from connection events
    for event in disconnected.read() {
        rsm.disconnect_trackers.insert(event.player_id, 0.0);
    }
    for event in reconnected.read() {
        rsm.disconnect_trackers.remove(&event.player_id);
    }

    // 2. Tick all trackers
    let delta = time.delta_secs();
    for elapsed in rsm.disconnect_trackers.values_mut() {
        *elapsed += delta;
    }

    // 3. Single-pass: collect all breaching players
    let breaching: Vec<PlayerId> = rsm.disconnect_trackers
        .iter()
        .filter(|(_, &elapsed)| elapsed > config.disconnect_grace_seconds)
        .map(|(&pid, _)| pid)
        .collect();

    if breaching.is_empty() { return; }

    // 4. Determine outcome
    let outcome = if breaching.len() >= 2 {
        GameOverEmitted { reason: GameOverReason::Draw, loser: None }
    } else {
        GameOverEmitted { reason: GameOverReason::Disconnection, loser: Some(breaching[0]) }
    };

    // 5. Phase-specific handling
    if rsm.phase == RoundPhase::Resolution {
        rsm.pending_disconnect_outcome = Some(outcome); // Defer
    } else {
        if rsm.phase == RoundPhase::DraftAuction {
            abort_auction_writer.write(AbortAuction);
        }
        game_over_writer.write(outcome);
        // Trigger advance_phase → GameOver
    }
}
```

**Mid-RESOLUTION deferral interaction with Story 004:** In `rsm_input_reader`, when processing `ResolutionComplete`, check `rsm.pending_disconnect_outcome` first. If `Some`, emit that `GameOverEmitted` and transition to GAME_OVER, skipping the normal win condition check. If `None`, proceed with normal win condition evaluation (Story 004 logic). Clear `pending_disconnect_outcome` after handling.

**Browser note (GDD Rule 13):** The 30-second grace is intentional for WASM/browser targets. OS interrupts, antivirus scans, and tab switches routinely cause 3–6 second Lightyear connection gaps with no player action. Do not reduce the default below 30s without consulting the game designer.

**`AbortAuction` event:** Check whether `AbortAuction` was included in the Story 001 event catalog. If not, add it to `server/src/core/rsm/events.rs` and register it in `RsmPlugin`. It is an outbound RSM event: `#[derive(Event, Clone, Debug)] pub struct AbortAuction;`

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001-004: All prior RSM infrastructure
- Story 006: Network dispatch wiring
- Auction System (M2): Auction abort behavior when `AbortAuction` is received — this story only emits the event
- Game Session System (Epic 2): Session teardown on `GameOverEmitted`
- Team mode disconnect (future): "team loses" on single disconnect is a future extension; 1v1 only for M1

---

## QA Test Cases

Each test uses `World::new()` + simulated Lightyear disconnect events (stub `OnDisconnected` events injected directly). No live Lightyear session required.

**RSM-23 — Single disconnect exceeds grace:**
- Given: `RoundState { phase: Placement, disconnect_trackers: {} }`; `config.disconnect_grace_seconds = 30.0`
- When: `disconnect_trackers[player_a]` set to `30.001` directly (bypasses Lightyear event for unit test); `tick_disconnect_timers` runs with delta=0
- Then: `GameOverEmitted { reason: Disconnection, loser: Some(player_a) }` written; `rsm.phase == GameOver`
- Edge cases: tracker = 30.0 exactly → must NOT trigger (strict `>`); tracker = 29.999 → must NOT trigger

**RSM-24 — Reconnect within grace:**
- Given: `disconnect_trackers[player_a] = 0.0` inserted (simulating disconnect)
- When: `tick_disconnect_timers` runs with cumulative delta of 15.0s; tracker removed (simulating reconnect); 20.0 more seconds tick
- Then: no `GameOverEmitted` written at any point; `rsm.phase` unchanged
- Edge cases: re-disconnect after reconnect starts a fresh 0.0 tracker entry

**RSM-25 — Boundary: exactly disconnect_grace_seconds survives:**
- Given: `disconnect_trackers[player_a] = 30.0` (exactly at boundary)
- When: `tick_disconnect_timers` runs (delta=0, tracker already at 30.0)
- Then: no `GameOverEmitted` written; condition is `elapsed > 30.0` (strict greater-than, not `>=`)
- Edge cases: 30.0 + f32::EPSILON → triggers; verify comparison operator is `>` not `>=`

**RSM-35 — Mid-RESOLUTION deferral:**
- Given: `RoundState { phase: Resolution, pending_disconnect_outcome: None, disconnect_trackers: {player_a: 30.001} }`
- When: `tick_disconnect_timers` runs
- Then: no `GameOverEmitted` written immediately; `rsm.pending_disconnect_outcome == Some(...)` with `reason: Disconnection, loser: Some(player_a)`; `rsm.phase` still `Resolution`
- When (continued): `ResolutionComplete` injected; `rsm_input_reader` runs
- Then: `GameOverEmitted { reason: Disconnection, loser: Some(player_a) }` written; win condition evaluation skipped; `rsm.phase == GameOver`; `pending_disconnect_outcome` cleared
- Edge cases: both players disconnect mid-RESOLUTION → pending = Draw

**RSM-37 — Mutual disconnect → single Draw GameOverEmitted:**
- Given: `disconnect_trackers: { player_a: 30.001, player_b: 30.001 }`
- When: `tick_disconnect_timers` runs (single pass evaluates both)
- Then: exactly one `GameOverEmitted { reason: Draw, loser: None }` in queue; no `GameOverEmitted { reason: Disconnection }` written; `rsm.phase == GameOver` exactly once
- Edge cases: player_a = 30.001, player_b = 30.0 → only player_a breaches → single loser (not Draw)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: Automated unit tests — `tests/unit/rsm/rsm_disconnect_test.rs` must pass; paste `cargo test -p server rsm_disconnect` output into `tests/evidence/rsm-story-005-tests.md`
**Gate Level**: BLOCKING — all tests listed in QA Test Cases must pass before this story is Done
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 004 (win condition and game over) must be Done — `rsm_input_reader` must handle `pending_disconnect_outcome` before win condition check; `GameOverEmitted` and `advance_phase` GAME_OVER path must be implemented
- Unlocks: Story 006 (network dispatch wiring)
