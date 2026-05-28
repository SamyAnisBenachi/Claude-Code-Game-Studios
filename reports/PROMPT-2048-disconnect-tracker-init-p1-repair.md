# PROMPT 2048 — Disconnect Tracker Init P1 Repair

## Verdict

**Root cause:** the disconnect tracker initialization path is **correct**; the
P1-001 forensic finding ("snapshots show seconds_since_disconnect around 30000s
from early game for both players, implying disconnect trackers initialize as
disconnected/stale") was a **snapshot-field-labelling bug**, not a tracker
init bug. The fix renames the misleading field and locks the contract with
focused tests so the same misread cannot recur.

## Evidence

`server/src/core/rsm/state.rs:46`:

    pub disconnect_trackers: HashMap<PlayerId, u32>,

The map stores **milliseconds remaining before GAME_OVER per player**, per
`design/gdd/round-state-machine.md` (Rule 13) and ADR-009. Init/refresh paths
all insert the **full grace window** as the healthy value:

- `server/src/core/rsm/transitions.rs:947` `reset_disconnect_trackers_for_session`:
  on session start, every session player is seeded with `grace_ms`
  (`disconnect_grace_seconds × 1000`; default `30 × 1000 = 30_000`).
- `server/src/core/rsm/transitions.rs:154` (on `PlayerDisconnected`): inserts
  `grace_ms` (or keeps existing remaining if already tracked).
- `server/src/core/rsm/transitions.rs:181`/`188` (on `PlayerReconnected` /
  `PlayerHeartbeat`): resets entry to `grace_ms`.
- `server/src/core/rsm/transitions.rs:209` (tick loop): decrements
  `remaining_ms` by `delta_ms`; breach at zero triggers GAME_OVER.

For a healthy/connected player, the value stays at or near `30_000` for the
entire match (heartbeats arrive every ~5000 ms and reset it). For a
disconnected player the value ticks **down** toward zero.

The QA snapshot side serialized this raw `u32` as
`seconds_since_disconnect: u32` (`server/src/feature/bot/qa_snapshot.rs:294`
pre-repair), which was **semantically wrong on three counts**:

1. The unit is milliseconds, not seconds.
2. It is "remaining before game-over", not "since the disconnect event".
3. The healthy initial state therefore looked like "30000 seconds since
   disconnect" (~ 8h33m of stale tracker) to any forensic reader — exactly
   the misread that drove the P1-001 bug filing.

The legitimate stored value `30000` is a **healthy fresh-tracker reading**,
not a stale tracker. No additional reset/init path was needed.

## Repair

`server/src/feature/bot/qa_snapshot.rs`:

- `DisconnectTrackerEntry.seconds_since_disconnect` -> `grace_ms_remaining`
  with a full doc-comment explaining the semantics (initialized to grace
  window on connect/heartbeat; only ticks down while disconnected; a value
  equal to the full grace window indicates a healthy player, **not** a
  stale tracker).
- `BOT_QA_SNAPSHOT_SCHEMA_VERSION` bumped `1 -> 2` with a doc-comment
  pointing at PROMPT 2048 so downstream evidence parsers detect the wire
  rename.
- `assemble_snapshot` mapping updated to populate the new field name.
- Existing `write_snapshot_to_disk_creates_dir_and_pretty_json` test
  updated to assert `schema_version: 2`.

### New focused tests (`server/src/feature/bot/qa_snapshot.rs` tests module)

1. `fresh_session_disconnect_trackers_report_full_grace_window` — seeds
   two players into `RoundState.disconnect_trackers` with the full grace
   window (mirrors `reset_disconnect_trackers_for_session`) and asserts
   both snapshot entries read `grace_ms_remaining == 30_000`. Proves
   the snapshot reports fresh/connected players as healthy, not stale.
2. `only_disconnected_players_accrue_below_full_grace` — mixes a
   connected player (`30_000`) and a disconnected player whose tracker has
   ticked down to `25_000` and verifies that only the disconnected one
   reports a value below the full grace window. Proves the accrual
   contract: connected -> stays full; disconnected -> ticks down toward
   GAME_OVER.
3. `disconnect_tracker_entry_serializes_as_grace_ms_remaining` — locks the
   on-wire field name. Asserts the JSON contains `grace_ms_remaining` and
   does **not** contain the legacy `seconds_since_disconnect`. Future
   renames must trip this test, forcing a schema bump.

## Owned-scope review

Only file modified inside the owned scope:

- `server/src/feature/bot/qa_snapshot.rs` — disconnect-tracker forensic
  serialization is the observation surface for the server disconnect
  tracker; the rename and the regression tests belong here.

Not touched (per allowlist):

- Client UI / autoplay / hand / drag / asset / result-screen / session-state.
- `server/src/core/rsm/state.rs`, `transitions.rs`, `core/session/*` — the
  init path is already correct; no edit needed.

Pre-existing modification `.claude/settings.json` was already in the
worktree on entry (hook-config drift, not part of this repair) and is
left untouched.

## Validation

- `git diff --check -- server/src/feature/bot/qa_snapshot.rs` — clean (no
  whitespace defects in owned diff).
- Path allowlist — owned diff confined to `server/src/feature/bot/qa_snapshot.rs`.
- **Focused-test compilation blocked by environment**: the D: drive holding
  this worktree was at 0 bytes free on entry; `cargo check -p server --lib`
  aborted with `os error 112 (There is not enough space on the disk)`
  partway through `lightyear_replication` / `windows-sys` / `serde_core`.
  `cargo clean` freed 1.7 GiB but the full server-lib check still needs more
  headroom than is available on D:. The edits themselves are mechanical
  (field rename + schema-version bump + three new tests that only use
  `RoundState::new()` + `HashMap::insert` + already-imported
  `serde_json::to_string`) and pass static review. **Exact blocker reported
  per the prompt contract**; rerun
  `cargo test -p server feature::bot::qa_snapshot::` after freeing D: to
  confirm.

## Behavioural impact

Server semantics unchanged. The change is observational: forensic readers
of `bot-qa-snapshot-*.json` will now see `grace_ms_remaining: 30000` for
healthy players (clearly "30 seconds of grace remaining"), and a value
below the grace window only for actually-disconnected players. The
schema-version bump (`1 -> 2`) signals the field rename so downstream
evidence tooling can detect the contract change.

## Final status line

2048: DISCONNECT-TRACKER-INIT-P1-REPAIR: SHIPPED-BLOCKED-VALIDATION
