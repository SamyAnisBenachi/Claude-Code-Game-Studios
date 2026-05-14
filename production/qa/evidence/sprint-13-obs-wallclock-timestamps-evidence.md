# Sprint 13 -- S13-OBS-WALLCLOCK-TIMESTAMPS-001 -- Implementation Evidence

> **Story**: `production/epics/playable-client/story-019-obs-wallclock-timestamps.md`
> **PROMPT**: 837 (`/dev-story` implementation)
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-obs-wallclock-timestamps`
> **Worker branch**: `work/s13-obs-wallclock-timestamps`
> **Source-of-truth at branch-create**: `origin/main@4f7ba78` (PROMPT 833
> `qa(s13): /story-done S11-SERVER-POOL-INIT-LOG-GUARD-001`).
> **Cross-link**: PROMPT 803 §3 DC-12; PROMPT 803 §5 Must row 8.

---

## No-Claim Banner (verbatim from story 019 file)

This story is **not** claiming: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full
playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), or
final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions are unchanged by this story.
PROMPT 761 Polish->Release gate-check FAIL evidence is preserved (no retry).

**No optimistic client-side authority is introduced or proposed by this
story.** The change is purely a subscriber-config tweak in three init sites
plus enabling the `tracing-subscriber` `time` feature in two `Cargo.toml`
files; no behaviour or authoritative state is touched. ADR-002 binding.

---

## Acceptance Criteria Verification

### AC1 -- Server subscriber config landed

`server/src/main.rs` (around the original line 87, now expanded for the new
`.with_timer(...)` call):

```rust
// MinimalPlugins does not include LogPlugin, so we initialise tracing here
// directly. This must come before App::new() so that all plugin startup
// messages are captured.
//
// S13-OBS-WALLCLOCK-TIMESTAMPS-001 (PROMPT 837): wall-clock UTC ISO-8601
// (RFC 3339) timer so multi-process logs from server + client + tests
// align at sub-second precision. Default fmt timer emits relative seconds
// since process start, which is useless for cross-process correlation.
tracing_subscriber::fmt()
    .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
    .init();
```

VERIFIED -- the subscriber init now calls `.with_timer(UtcTime::rfc_3339())`.

### AC2 -- Client subscriber config landed

`client/src/main.rs` (around the original line 36):

```rust
//
// S13-OBS-WALLCLOCK-TIMESTAMPS-001 (PROMPT 837): wall-clock UTC ISO-8601
// (RFC 3339) timer so multi-process logs from server + client + tests
// align at sub-second precision. Default fmt timer emits relative seconds
// since process start, which is useless for cross-process correlation.
#[cfg(not(target_arch = "wasm32"))]
{
    use tracing_subscriber::fmt::time::UtcTime;
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,wgpu=warn,wgpu_hal=warn,naga=warn,bevy_ecs=info")
    });
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_timer(UtcTime::rfc_3339())
        .init();
}
```

VERIFIED -- the desktop-only init block now adds `.with_timer(UtcTime::rfc_3339())`
to the existing `EnvFilter` configuration.

### AC3 -- Test subscriber config landed

`tests/test_helpers.rs` (around the original line 52, in `init_test_tracing`):

```rust
let _ = tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_timer(UtcTime::rfc_3339())
    .with_test_writer()
    .try_init();
```

VERIFIED -- the test harness subscriber now matches the server / client
timer config; `with_test_writer()` is preserved so cargo's test capture
behaviour is unchanged.

### AC4 -- Timer format consistent across three sites

All three sites construct the timer with the identical expression:

```text
tracing_subscriber::fmt::time::UtcTime::rfc_3339()
```

VERIFIED via `grep -rn "UtcTime::rfc_3339" server/src/main.rs client/src/main.rs tests/test_helpers.rs`:

```text
server/src/main.rs:91:            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
client/src/main.rs:41:                .with_timer(UtcTime::rfc_3339())
tests/test_helpers.rs:53:            .with_timer(UtcTime::rfc_3339())
```

### AC5 -- `Cargo.toml` feature flag added (required)

`tracing-subscriber`'s `UtcTime` type lives behind the `time` feature flag.
The feature was NOT previously enabled in `client/Cargo.toml` or
`server/Cargo.toml` (only `env-filter` was). PROMPT 837 enables it in both
crates:

```toml
# client/Cargo.toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "time"] }

# server/Cargo.toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "time"] }
```

Rationale (inline in evidence rather than as Cargo comment to avoid noise in
the small toml change): `UtcTime::rfc_3339()` is gated behind the
`tracing-subscriber/time` feature, which pulls in the `time` crate. The
incremental cost is the `time` crate (already a transitive dep of `rcgen`
via `lightyear_websocket`, so no NEW dependency-graph weight at runtime).

`shared/Cargo.toml` does NOT depend on `tracing-subscriber` and is
unchanged.

### AC6 -- Sample log output carries ISO-8601 UTC ms-precision timestamps

#### Pre-implementation baseline (before PROMPT 837)

`tracing_subscriber::fmt().init()` (server) and the env-filter-only init
(client / tests) emit a default timer that prints relative seconds since
process start, e.g.:

```text
  0.000123s  INFO server: Lanes and Lies server starting ...
```

(Reproducible by reverting `server/src/main.rs:87` to `tracing_subscriber::fmt().init()`
and re-running `cargo run -p server`.)

#### Post-implementation server sample

Captured at the worker tip (commit recorded in the final report) by running:

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
timeout 10 cargo run -p server 2>&1 | head -25
```

Output (ANSI styling stripped for clarity):

```text
2026-05-14T13:13:33.3663018Z  INFO server: Lanes and Lies server starting — authority model: ADR-002
2026-05-14T13:13:33.3671169Z  INFO server::foundation::config: AppState::Loading — requesting game_config.ron and cards.json
2026-05-14T13:13:33.3676927Z  INFO lightyear_websocket::server: Server WebSocket starting at 0.0.0.0:5000
2026-05-14T13:13:33.3809762Z  INFO server::foundation::config: Both assets loaded — transitioning to AppState::ConfigValidation
2026-05-14T13:13:33.3812901Z  INFO server::foundation::config: Assets loaded: GameConfig + CardCatalog (16 cards) — transitioning to AppState::Lobby
```

Every line begins with an ISO-8601 UTC timestamp at sub-second (sub-ms)
precision followed by `Z`. The format produced by `UtcTime::rfc_3339()` is
RFC 3339 with 7-digit fractional-second precision (100-ns ticks) -- richer
than the AC6 ms-floor regex requires.

The AC6 regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z\b` (strict 3-digit
fractional) is a subset of the canonical RFC 3339 format produced by
`UtcTime::rfc_3339()`. AC6 explicitly allows "the canonical ISO-8601 UTC
ms-precision format produced by the chosen API"; `UtcTime::rfc_3339()` is
the chosen API and produces a superset (sub-ms precision) which trivially
satisfies the alignment requirement.

#### Post-implementation client sample

Not run in this evidence pass because the client binary requires a window
and a Vulkan-capable backend. The format is identical -- the subscriber
builder expression is byte-for-byte equivalent between server and client
(verified by AC4) -- and `cargo check -p client` passes (recorded below),
so the same prefix is emitted at runtime.

#### Post-implementation test-harness sample

Exercised by the new test
`tests/unit/observability/wallclock_timer_test.rs` (registered as
`observability_wallclock_timer_test` in `server/Cargo.toml`). The test:

1. Builds a `tracing_subscriber::fmt()` subscriber with the same
   `.with_timer(UtcTime::rfc_3339())` builder call used by all three
   production sites, plus a `CapturedWriter` accumulator.
2. Emits `tracing::info!("wallclock_timer_marker_event")` under that
   subscriber via `tracing::subscriber::with_default`.
3. Asserts the captured output begins with
   `YYYY-MM-DD T HH:MM:SS` followed by `.<subseconds>Z` or directly `Z`.
4. Walks the fractional-second digits and confirms termination at `Z`.
5. Verifies the event payload survives in the formatted line.

Command + result:

```text
$ cargo test -p server --test observability_wallclock_timer_test -- --nocapture
    Finished `test` profile [optimized] target(s) in 1m 07s
     Running ..\tests\unit\observability\wallclock_timer_test.rs
running 1 test
test test_wallclock_timer_emits_iso_8601_utc_prefix ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

VERIFIED -- AC6 satisfied for the canonical format path.

### AC7 -- Behaviour unchanged

`cargo check --workspace --all-targets` at the worker tip:

```text
    Checking shared v0.1.0 ...
    Checking server v0.1.0 ...
    Checking client v0.1.0 ...
warning: function `count_with_image_node` is never used
  --> client\..\tests\integration\presentation\hand_ui_asset_wiring_test.rs:43:4
   (pre-existing dead_code warning unrelated to PROMPT 837)
    Finished `dev` profile [optimized] target(s) in 20.38s
```

No new warnings or errors introduced by PROMPT 837. The single pre-existing
`dead_code` warning in `hand_ui_asset_wiring_test.rs:43` is in a file
untouched by this story.

Per the Sprint 13 QA plan §"Required Regression/Test Commands" and the
no-full-workspace-tests-by-default policy, PROMPT 837 did NOT run
`cargo test --workspace --tests --no-fail-fast`. The targeted Logic test
above plus the cross-crate `cargo check` covers the AC7 behaviour-unchanged
gate for the subscriber-config scope; orchestrator integration-merge will
run the full workspace test sweep before flipping `/story-done`.

### AC8 -- No optimistic client-side authority introduced

VERIFIED. The PROMPT 837 diff scope is:

- `server/src/main.rs` -- subscriber init in the server `main` binary; no
  game state read or written.
- `client/src/main.rs` -- subscriber init in the desktop client `main`
  binary; no game state read or written.
- `tests/test_helpers.rs` -- subscriber init in the shared test harness; no
  game state read or written.
- `server/Cargo.toml`, `client/Cargo.toml` -- feature flag addition for
  `tracing-subscriber`; no source-code semantics.
- `tests/unit/observability/wallclock_timer_test.rs` -- new Logic test
  against the subscriber builder in isolation; no game state touched.
- `production/qa/evidence/sprint-13-obs-wallclock-timestamps-evidence.md`
  -- this evidence file.

**Phrase**: no optimistic.

No client-side mutation of authoritative state outside the shared phase
sink, snapshot drainers, and S2C consumers is added or modified. ADR-002
remains binding and unchanged.

### AC9 -- Sprint 12 disposition preserved

PROMPT 837 does NOT modify `production/sprint-status.yaml`,
`production/sprints/sprint-12.md`, `production/stage.txt`, or
`production/qa/qa-plan-sprint-12.md`. Cross-verified by
`git diff --stat origin/main...HEAD` (recorded in the final report).
Sprint 12 disposition `closed-with-conditions` (PROMPT 817) is preserved
unchanged.

### AC10 -- Evidence document slot reserved

This file is that evidence document.

---

## Files Changed by PROMPT 837

```text
client/Cargo.toml                                                            | 2 +-
client/src/main.rs                                                           | 11 +++++++++--
production/qa/evidence/sprint-13-obs-wallclock-timestamps-evidence.md        | (NEW)
server/Cargo.toml                                                            | 6 +++++-
server/src/main.rs                                                           | 9 ++++++++-
tests/test_helpers.rs                                                        | 8 ++++++--
tests/unit/observability/wallclock_timer_test.rs                             | (NEW)
```

No production source under `client/`, `server/`, or `shared/` is touched
beyond the three named subscriber init sites and the two `Cargo.toml`
feature additions.

---

## Verification Commands Run (PROMPT 837)

| Command | Result |
|---------|--------|
| `cargo fmt -p client -p server -- --check` | PASS (exit 0) |
| `cargo check -p server` | PASS (exit 0) |
| `cargo check -p client` | PASS (exit 0) |
| `cargo check --workspace --all-targets` | PASS (exit 0; one pre-existing unrelated warning) |
| `cargo test -p server --test observability_wallclock_timer_test` | PASS (1/1) |
| `cargo run -p server` (10s capture) | Emits 5+ ISO-8601 UTC lines, all matching AC6 format |
| `git diff --check origin/main...HEAD` | clean (recorded in final report) |

The Sprint 13 QA plan no-full-workspace-tests-by-default policy is honoured:
PROMPT 837's diff is a 3-line behavioural change in subscriber config plus
a new isolated Logic test, so `cargo test --workspace` is not run by the
worker. Integration-merge will run the full sweep.

---

## Cross-Links

- Story file: `production/epics/playable-client/story-019-obs-wallclock-timestamps.md`
- Source audit: `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`
  §3 DC-12, §4 Lane E, §5 Must row 8
- Sprint 13 QA plan: `production/qa/qa-plan-sprint-13.md` §"S13-OBS-WALLCLOCK-TIMESTAMPS-001"
  (story 019, playable-client epic)
- DISTINCT from Sprint 12 `hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  (closed `cannot-reproduce` per PROMPT 817); the Sprint 12 underlying
  drag-runtime bug is NOT claimed fixed by Sprint 13.

---

## Conditions Carried Forward Unchanged by PROMPT 837

- TQ-S12-C1..C7 preserved verbatim.
- TQ-S12-C2 binding: no third same-scope retest of Sprint 12 story 019
  (hand-ui) authorised; PROMPT 837 expands the diagnostic toolkit (UTC
  timestamps) but does NOT re-attempt the Sprint 12 capture.
- `S8-QA-001-W1` OPEN.
- `QA-COND-0005`, `QA-COND-0006` accepted-risk / deferred.
- `PAW-TD-*-a` accept-risk on placeholder PNGs.
- PROMPT 683-era runtime divergence question -- folded into Sprint 12 story
  019 (`closed-with-conditions / cannot-reproduce`); NOT claimed closed by
  PROMPT 837.
- PROMPT 761 Polish->Release gate-check `FAIL` -- preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; NO retry
  attempted by PROMPT 837.
- Sprint 12 / Sprint 11 / Sprint 10 closeout dispositions preserved.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 837 -- `/dev-story` implementation on worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s13-obs-wallclock-timestamps`,
  branch `work/s13-obs-wallclock-timestamps`, from `origin/main@4f7ba78`.
  Three subscriber init sites configured with `.with_timer(UtcTime::rfc_3339())`;
  `time` feature added to `tracing-subscriber` in `client/Cargo.toml` and
  `server/Cargo.toml`; new Logic test at
  `tests/unit/observability/wallclock_timer_test.rs` registered in
  `server/Cargo.toml`; evidence captured. Cargo resource policy applied
  (env vars set per Sprint 13 QA plan §"Cargo Resource Policy on
  Windows/MSVC"). No disk-pressure threshold hit, no cleanup performed.
