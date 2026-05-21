# PROMPT 1597 — BOT-FLOW-SERVER-QA-SNAPSHOT-AND-DECISION-LOG

## Summary

Adds the server-authoritative QA evidence channel for bot-driven flows
identified as missing items 2 and 3 in `reports/PROMPT-1594-bot-flow-inventory-followup.md`.

A new `server::feature::bot::qa_snapshot` module:

- Writes structured JSON **snapshots** of the observable game world (phase,
  RSM timers, both hands, board occupancy, economies, auction, objectives,
  per-bot RNG/timing, decision-log tail) on:
  - phase transitions (any change in `RoundState.phase`),
  - a periodic 10 s tick (configurable in code),
  - a best-effort dump on `AppExit` (graceful shutdown).
- Streams every `BotDecisionLog` append to a JSONL file, **flush()-ed on
  every line** so the file is durable even if the process dies between
  entries.
- Reads observable resources only (`Option<Res<…>>`) — no semantic gameplay
  mutation, no new protocol messages, no client surface, no replication.

All evidence lands under repo-root `dev-runs/` by default, matching the
"evidence under dev-runs" rule from the inventory follow-up.

## Source-of-truth and inputs

- Base: `origin/main@3a4603afe9b8b3cce40dca58e54366671a8db4b4` (fetched fresh).
- Worktree: `D:/Tmp/wt-1597` on branch `work/bot-flow-server-qa-snapshot-1597`.
- Inventory follow-up reference: `reports/PROMPT-1594-bot-flow-inventory-followup.md`
  (cited by the prompt; not present on `origin/main@3a4603af`).

## Files touched (vs origin/main)

```
server/src/feature/bot/mod.rs          (re-export the new module + plugin)
server/src/feature/bot/qa_snapshot.rs  (new — ~960 lines including tests)
server/src/main.rs                     (register BotQaSnapshotPlugin)
```

All paths fall inside the owned scope declared in the prompt:

- `server/src/feature/bot/**` ✓
- `server/src/main.rs` — plugin wire only (one `app.add_plugins(…)` call). No
  semantic edits; no other system or schedule changes.

No edits to:

- `client/**`, `shared/src/protocol.rs`,
- `production/sprint-status.yaml`, `production/session-state/**`,
  `production/stage.txt`, sprint activation/close-out files,
- workspace-level `Cargo.toml`, `Cargo.lock`, CI files,
- any existing test wiring.

## Activation contract

Mirrors the client-side `CCGS_QA_SNAPSHOT*` convention so the two
subsystems can be toggled independently:

| Env var | Purpose | Default |
|---|---|---|
| `CCGS_BOT_QA_SNAPSHOT` | `1` forces enabled, `0` disabled, unset = `cfg!(debug_assertions)` | dev: enabled, release: disabled |
| `CCGS_BOT_QA_SNAPSHOT_DIR` | Output directory for snapshot JSONs | `dev-runs/bot-qa-snapshots` |
| `CCGS_BOT_DECISION_LOG_PATH` | Path of JSONL bot decision log | `dev-runs/bot-decision-log.jsonl` |

The server binary does not parse CLI arguments today (only `SERVER_PORT` is
read in `network::mod`), so env vars are the project-conventional knob.
A future CLI overhaul can wrap these env vars without breaking compatibility.

Tests may insert a `BotQaSnapshotConfig { enabled: true, snapshot_dir, decision_log_path, periodic_interval_ms }`
directly before adding the plugin; `BotQaSnapshotPlugin::build` does not
overwrite a pre-inserted config (`if !world.contains_resource::<…>() { … }`).

## Snapshot file layout

Snapshot filename:

```
snapshot-<round:04>-<phase>-<trigger>-<timestamp_ms:013>-<sequence:06>.json
```

Triggers: `init` (first snapshot after activation), `phase` (RSM phase
change), `tick` (10 s periodic), `shutdown` (best-effort AppExit dump).

Sample evidence layout (post-startup, single 2-round game with one bot):

```
dev-runs/
├── bot-decision-log.jsonl                              # streamed, one JSON per line
└── bot-qa-snapshots/
    ├── snapshot-0000-lobby-init-0000000000123-000001.json
    ├── snapshot-0000-lobby-phase-0000000000456-000002.json     # Lobby → DraftInitial
    ├── snapshot-0001-draftinitial-tick-0000000010456-000003.json
    ├── snapshot-0001-draftinitial-phase-0000000012003-000004.json  # → Placement
    ├── snapshot-0001-placement-phase-0000000023017-000005.json     # → Resolution
    ├── snapshot-0001-resolution-phase-0000000028019-000006.json    # → DraftAuction (round 2)
    ├── snapshot-0002-draftauction-tick-0000000038019-000007.json
    └── snapshot-0002-gameover-shutdown-0000000048123-000008.json   # AppExit
```

### Snapshot JSON shape (abbreviated)

```jsonc
{
  "schema_version": 1,
  "trigger": "phase_transition",          // init | phase_transition | periodic | graceful_shutdown
  "timestamp_ms": 12003,
  "sequence": 4,
  "round": {
    "phase": "Placement",
    "round_number": 1,
    "draft_ready_players": [{ "0": 7 }, { "0": 9223372036854775808 }],
    "submissions_received": [],
    "disconnect_trackers": [],
    "timers_ms": {
      "placement": 19850, "placement_grace": null,
      "draft_initial": null, "draft_shop": null,
      "auction_safety": null, "resolution_safety": null
    }
  },
  "session": { "mode": "OneVOne", "player_count": 2, "players": [ … ], "placement_timer_multiplier": "X1" },
  "auction": null,                         // present only while an auction is live
  "economies": [ { "player": {…}, "gold": 7, "current_mana": 2, … } ],
  "hands":     [ { "player": {…}, "size": 3, "cards": [ {…}, {…}, {…} ] } ],
  "board":     { "minion_count": 1, "trap_count": 0, "structure_count": 0, "field_count": 0,
                 "per_player_minions": [ { "player": {…}, "occupied_lanes": [3] } ] },
  "objectives": [ { "player": {…}, "lane": 1, "hp": 30, "destroyed": false }, … ],
  "bots":      [ { "player": {…}, "difficulty": "mvp", "rng_seed": 9223372036854775808,
                   "rng_word_counter": 2, "last_decision_at_ms": 11400,
                   "class_choice": "Iop", "next_decision_at_ms": null, "failsafe_deadline_ms": null } ],
  "decision_log_total": 5,
  "decision_log_tail": [
    { "round_number": 0, "phase": "Lobby", "bot_player_id": {…}, "decision": { "kind": "class_confirmed" }, … },
    { "round_number": 1, "phase": "DraftInitial", "bot_player_id": {…}, "decision": { "kind": "draft_ready" }, … },
    { "round_number": 1, "phase": "DraftAuction", "bot_player_id": {…},
      "decision": { "kind": "auction_pass", "reason": "phase_not_live_bidding" }, … }
  ]
}
```

### Decision-log JSONL shape

One `BotDecisionEntry` per line, in append order, `flush()`-ed after every
write so the file is durable even if the process is killed between events:

```jsonl
{"round_number":0,"phase":"Lobby","bot_player_id":{"0":9223372036854775808},"timestamp_ms":50,"seed":9223372036854775808,"seed_word_counter":0,"legal_action_count":6,"decision":{"kind":"class_confirmed"}}
{"round_number":1,"phase":"DraftInitial","bot_player_id":{"0":9223372036854775808},"timestamp_ms":1200,"seed":9223372036854775808,"seed_word_counter":0,"legal_action_count":null,"decision":{"kind":"draft_ready"}}
{"round_number":1,"phase":"DraftAuction","bot_player_id":{"0":9223372036854775808},"timestamp_ms":7400,"seed":9223372036854775808,"seed_word_counter":1,"legal_action_count":1,"decision":{"kind":"auction_bid","card_id":{"0":42},"amount":4,"valuation":5}}
```

The streamer keeps a `BufWriter<File>` resident in `BotQaSnapshotState` and
flushes after every append; it reopens only when the configured path
changes (covered by a dedicated unit test).

## Implementation notes

### Architecture

- `BotQaSnapshotConfig` (resource) — env-built, deterministic
  `from_env_values()` constructor for tests.
- `BotQaSnapshotState` (resource) — `last_phase`, `next_periodic_ms`,
  `decision_log_offset`, `sequence`, lazily-opened `BufWriter<File>` for the
  decision log, `shutdown_dump_done` re-entrancy guard.
- `BotQaSnapshotPlugin` registers:
  - `bot_qa_snapshot_writer_system` on `Update`,
  - `bot_decision_log_streamer_system` on `Update`,
  - `bot_qa_snapshot_shutdown_system` on `Last`.

The plugin is always registered; every system early-returns when the
config is disabled, so production servers pay zero observable cost.

### Trigger logic (`bot_qa_snapshot_writer_system`)

Snapshot fires when **any** of:

1. `last_phase` is `None` and the world has a `RoundState` → `Initial` trigger.
2. `last_phase != current_phase` → `PhaseTransition` trigger.
3. `now_ms >= next_periodic_ms` → `Periodic` trigger.

After every write the system records the new phase and arms
`next_periodic_ms = now + periodic_interval_ms` so phase-transition writes
also reset the periodic clock (avoiding back-to-back snapshots within a
single tick).

The system gates on **`bot_count >= 1`** so a fully-human session does not
generate evidence files for a channel that has nothing to document.

### Best-effort graceful shutdown (`bot_qa_snapshot_shutdown_system`)

Runs in the `Last` schedule and consumes `MessageReader<AppExit>`. The
first time it observes any `AppExit` event it writes a final snapshot with
`trigger = "graceful_shutdown"`, then sets `shutdown_dump_done = true` so
subsequent `AppExit` events do not duplicate the dump.

This catches `App::run()` returning cleanly (config validation failure,
explicit `AppExit::Success` from a system, etc.). It is **not** a robust
SIGKILL/Ctrl+C trap on Windows — the streaming JSONL is the durable
fallback when the process disappears without an `AppExit` event. The
prompt explicitly defers a true Windows-aware Ctrl+C handler.

### Determinism & safety

- Every I/O failure is logged at `warn` and the system continues. Evidence
  is diagnostic, not authoritative.
- The streamer guards against a poisoned `Mutex` (`PoisonError::into_inner`)
  to recover write capability across panics in a sibling thread.
- The snapshot reads only `Option<Res<…>>` so missing-resource test
  scaffolds compile without changes.
- `BTreeMap` was tempted for board-occupancy aggregation but `PlayerId`
  does not implement `Ord`; the module sorts via `.player.0` at output time
  so JSON output is stable across runs.

## Validation

- **Path allowlist review**: PASS — `git diff --name-only origin/main..HEAD`
  yields three files, all inside the owned scope (see *Files touched* above).
- **`git diff --check`**: PASS — clean.
- **`cargo check -p server --lib`**: PASS — 37.9 s.
- **`cargo check -p server --bin server`**: PASS — 11.7 s.
- **Focused unit tests**: PASS — 12 new tests under
  `feature::bot::qa_snapshot::tests`, all green, run in 0.01 s. Tests cover:
  - env-var parsing (enabled/disabled/dev-default/invalid values, path overrides),
  - snapshot filename derivation (round/phase/trigger/timestamp/sequence),
  - decision-entry JSON variant serialisation (incl. `Purchased { source: ShopPurchase }`),
  - JSONL append + flush + handle reuse,
  - JSONL reopen on path change,
  - `write_snapshot_to_disk` creating nested directories + pretty-printing,
  - end-to-end `assemble_snapshot` populating bots / economies / hands / decision-log tail from observed resources,
  - decision-log tail cap at `DECISION_LOG_TAIL_CAP = 64` (full count preserved in `decision_log_total`).
- **Regression check on existing bot tests**: PASS — `cargo test -p server
  --lib feature::bot` reports 19/19 (12 new + 7 existing), 0.00 s suite time.
- **Broad cargo suites**: DEFERRED per prompt rule
  ("Do not run broad Cargo. … broad verification deferred to VERIFY lane").

## Out-of-scope check

No edits to:

- `client/**`, `shared/**` (qa_snapshot reads `shared::session::PlayerId`,
  `shared::card::{CardId, ClassId}`, `shared::protocol::{CardSource, RoundPhase}`
  but does not modify their definitions).
- `production/sprint-status.yaml`, `production/session-state/**`,
  `production/sprints/**`, `production/qa/**`, `production/stage.txt`.
- Workspace `Cargo.toml`, `Cargo.lock`, CI files.
- Existing `[[test]]` blocks in `server/Cargo.toml` (the 12 new tests live
  in `#[cfg(test)] mod tests` inside `qa_snapshot.rs`, run via
  `cargo test -p server --lib`, so no `[[test]]` plumbing was needed).

## Push status

Local branch `work/bot-flow-server-qa-snapshot-1597` will be pushed to
origin as part of the commit step. Branch is FF over `origin/main@3a4603af`.

---

1597: BOT-FLOW-SERVER-QA-SNAPSHOT-AND-DECISION-LOG: SHIPPED
