# Sprint 17 — S17-SERVER-START-OF-TURN-DEBUG-001 — Implementing-worker evidence

> **Prompt**: PROMPT 1104
> **Date**: 2026-05-18
> **Story**: `production/epics/server/story-003-start-of-turn-debug-downgrade.md`
> **Branch**: `work/s17-server-start-of-turn-debug` from `origin/main@ff47075`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s17-server-start-of-turn-debug`
> **Source audit**: `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md` AUDIT-1076-15
> **QA-plan row**: `production/qa/qa-plan-sprint-17.md` Row 8
>
> **Status**: implementing-worker evidence only. The BLOCKING gate is the
> Sprint 17 smoke harness with an actual two-client session (a later prompt).
> This row's `/dev-story` worker does NOT run `/smoke-check`, `/team-qa`,
> `/gate-check`, or `/story-done`.

---

## Change summary (AC1 / AC4)

Single-line `tracing::warn!` → `tracing::debug!` substitution at the
`start_of_turn_dispatch_system` call site:

- **File**: `server/src/feature/keyword/observers.rs`
- **Lines changed**: 66 only (macro name)
- **Message text**: unchanged (`"start_of_turn_dispatch_system not yet implemented: keyword dispatch deferred to future story"`)
- **System body**: unchanged (still the same `MessageReader<DraftStarted>` stub)
- **`target:` field**: unchanged (`"server::game"`)

Diff:

```diff
 pub fn start_of_turn_dispatch_system(mut draft_started: MessageReader<DraftStarted>) {
     if draft_started.read().next().is_some() {
-        tracing::warn!(
+        tracing::debug!(
             target: "server::game",
             "start_of_turn_dispatch_system not yet implemented: keyword dispatch deferred to future story"
         );
     }
 }
```

AC1 grep (post-edit):

```
$ grep -rn 'start_of_turn_dispatch_system not yet implemented' server/src/
server/src/feature/keyword/observers.rs:68:            "start_of_turn_dispatch_system not yet implemented: keyword dispatch deferred to future story"
```

The surrounding macro at line 66 is `tracing::debug!` (verified by file Read).
The four sibling observer `tracing::warn!` calls (`on_unit_appeared`,
`on_final_blow_dealt`, `on_start_of_turn`, `on_end_of_turn`) at lines 13, 44,
51, 58 are **out of scope** for this row and remain unchanged.

## Out-of-scope verification (AC5 / AC6 / AC8 / AC9)

`git status --porcelain` on the worker branch shows only:

- `server/src/feature/keyword/observers.rs` (the single-line macro change)
- `production/qa/evidence/sprint-17-start-of-turn-debug/` (this evidence dir)

Zero changes under: `client/`, `shared/`, `server/src/core/rsm/`,
`tests/integration/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`,
`Trunk.toml`, `production/sprint-status.yaml`, `production/sprints/`,
`production/stage.txt`, `production/session-state/`,
`production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
`production/qa/team-qa-*.md`, `production/gate-checks/`,
`docs/architecture/adr-*.md`, `.claude/`, `CLAUDE.md`.

`start_of_turn_dispatch_system` itself is **not implemented** by this row —
the dispatch logic remains deferred.

## Cargo gate (AC10)

`cargo check -p server` under the Sprint 15+ Cargo resource policy:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Result:

```
    Checking server v0.1.0 (D:\_DEV\claude-code-game-studios-worktrees\s17-server-start-of-turn-debug\server)
    Finished `dev` profile [optimized] target(s) in 10.76s
```

Zero new compile errors. Zero new warnings on the touched file. RUSTFLAGS
honoured (no `+ debuginfo` in the "Finished" line).

D: free space before cargo: 762.25 GB (well above 40 GB threshold; no
target-dir cleanup performed).

## Launch-log evidence (AC2 / AC3 — non-binding; per QA-plan Row 8)

Two `cargo run -p server` launches captured against the post-edit binary. Each
launch was bounded by a 12-second timer (the server is a long-running headless
binary) and stopped via `taskkill /F /T`.

### Default filter — `default-launch.log`

Captured with `RUST_LOG` unset. Total content: 5 INFO lines (server boot:
plugin init, asset load, `AppState::Lobby` entry), followed by the cargo
runner's stderr (`Finished ... Running ...`).

```powershell
Select-String -Path default-launch.log -Pattern 'not yet implemented' -SimpleMatch | Measure-Object | % Count
# -> 0
Select-String -Path default-launch.log -Pattern 'WARN.*start_of_turn_dispatch_system not yet implemented' | Measure-Object | % Count
# -> 0
```

**AC2 partial-evidence**: zero `WARN` lines containing the audit phrase. The
audit (`AUDIT-1076-15`) measured 6 such lines per game session
(`server.log` lines 67, 109, 161, 240, 293, 337 in PROMPT 1076 run-7) at the
default filter, so the absence of the WARN at boot is consistent with the
downgrade. The **binding** gate remains the Sprint 17 smoke harness with an
actual two-client session driving real `DraftStarted` traffic — a later
prompt's scope, not this worker's.

### `RUST_LOG=debug` filter — `debug-launch.log`

Captured with `$env:RUST_LOG='debug'`. Identical surface content to
`default-launch.log` (same 5 INFO lines + cargo runner stderr).

```powershell
Select-String -Path debug-launch.log -Pattern 'not yet implemented' -SimpleMatch | Measure-Object | % Count
# -> 0
```

**AC3 caveat — empirically unverifiable at boot**: the
`start_of_turn_dispatch_system` only emits the log line when a `DraftStarted`
message arrives (function body: `if draft_started.read().next().is_some()`).
At server boot with no client connected, no `DraftStarted` fires, so the
log line does not trigger at any filter level. Two independent reasons leave
the debug-log empty of the phrase:

1. **No `DraftStarted` at boot** — same as above; the system body is unchanged
   and still gated on the message reader. AC3 evidence requires a live
   session, which is the Sprint 17 smoke harness's scope, not this worker's.
2. **Pre-existing tracing-subscriber wiring (out of this row's scope)** —
   `server/src/main.rs:50-52` initialises tracing via
   `tracing_subscriber::fmt().with_timer(...).init()`, which uses the builder's
   `init()` path. Unlike the standalone `tracing_subscriber::fmt::init()`
   wrapper, the builder path does **not** auto-wire
   `EnvFilter::from_default_env()`, so `RUST_LOG=debug` is currently ignored
   by the server binary. This is a pre-existing wiring gap, independent of
   `warn! → debug!`, and outside this row's scope. AUDIT-1076-15's "RUST_LOG
   raises it back" recommendation depends on filling that gap separately
   (candidate follow-up: a future ops-hardening row to add
   `.with_env_filter(EnvFilter::from_default_env())` to the subscriber
   builder). Not claimed or modified by this row.

These caveats do not affect AC1 / AC4 / AC5 / AC6 / AC8 / AC9 / AC10 — the
source diff alone proves the macro substitution, the message text is
unchanged, and no other files are touched.

## ACs not claimed by this worker

- **AC2** — fully binding only on Sprint 17 smoke harness (later prompt).
  This evidence is partial: WARN absent at boot, but the audit's
  6-WARNs-per-session measurement was per game session, not per boot.
- **AC3** — empirically untriggerable at boot (see caveats above); the
  source diff is the proof that the macro is now `debug!`. A live session
  driving `DraftStarted` is needed to observe the `DEBUG` line at the wire;
  scheduled for the Sprint 17 smoke prompt.
- **AC7** — preserved by avoiding any closure-claim text in commit / evidence;
  the commit message will explicitly NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, or any other accept-risk
  disposition.

## Files in this evidence dir

- `default-launch.log` — `cargo run -p server` with `RUST_LOG` unset (12 s
  bounded; killed via `taskkill /F /T`).
- `debug-launch.log` — `cargo run -p server` with `RUST_LOG=debug` (12 s
  bounded; killed via `taskkill /F /T`).
- `evidence.md` — this document.

## Carried-forward conditions (preserved, NOT claimed closed by this row)

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish → Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- `start_of_turn_dispatch_system` implementation remains **deferred**.
- All AUDIT-1076-* findings outside AUDIT-1076-15 preserved as open.
- All SOURCE-1077-* findings preserved.
- All 24 PROMPT 1022 audit findings preserved.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry preserved.
