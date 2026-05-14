# Story 019: S13-OBS-WALLCLOCK-TIMESTAMPS-001 -- ISO-8601 UTC Wall-Clock Timestamps in Tracing Subscribers

> **Epic**: Playable Client
> **Story ID**: S13-OBS-WALLCLOCK-TIMESTAMPS-001
> **Status**: Draft -- Sprint 13 candidate; NOT activated; Sprint 12 is the
> active sprint
> **Layer**: Observability / Cross-Cutting
> **Type**: Integration -- subscriber-config edits in 3 files
> **Sprint**: Sprint 13 candidate (per PROMPT 803 §6 line 145; NOT activated)
> **Authored**: 2026-05-14 by PROMPT 804 (worktree
> `work/s13-runtime-hardening-story-authoring`)
> **Authoring source-of-truth**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12
> QA-plan commit). Sprint 12 active per PROMPT 798 at `origin/main@796851b`.

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 804. Sprint 12 remains the active sprint
(`status: active`) and must not be changed by this authoring run.

PROMPT 804 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md` or any other active sprint
  file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify `production/qa/qa-plan-sprint-12.md` or any other QA-plan file.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 dispositions unchanged. PROMPT 761 Polish->Release
gate-check FAIL evidence preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** The change is purely a subscriber-config tweak in three init
sites; no behaviour or authoritative state is touched. ADR-002 binding.

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-12** No wall-clock ISO-8601 timestamping in subscribers
  (HIGH for diagnostic): `tracing_subscriber::fmt().init()` (server
  `main.rs:87`) and equivalents on client/test -- no
  `.with_timer(..)`. Multi-machine log correlation impossible
  without external shell wrapper.
- **§4 Lane E DC-12**: `server/src/main.rs:87`,
  `client/src/main.rs:36`, `tests/test_helpers.rs:52` -- all
  `tracing_subscriber::fmt()...init()` without `.with_timer(..)`.
- **§5 Must row 8 (S13-OBS-WALLCLOCK-TIMESTAMPS-001)**: "Configure
  `tracing_subscriber::fmt().with_timer(UtcTime::rfc_3339())` (or
  equivalent) in server, client, and tests so multi-process logs
  can be aligned at ms precision". Verification: spot check by
  running server + two clients and asserting ISO-8601 UTC prefix
  on every line. Parallel-safe (three independent files; trivial
  to bundle).
- **§6 PROMPT-N+5 (paired with S13-OBS-TRACING-TARGETS-001)**:
  paperwork-only story-authoring.

---

## Problem Class / Prevention Target

**Defect class (DC-12)**: Three subscriber init sites in the
workspace (`server/src/main.rs`, `client/src/main.rs`,
`tests/test_helpers.rs`) configure `tracing_subscriber::fmt()`
without a wall-clock timer. The default fmt timer emits relative
seconds-since-process-start, which is useless for multi-process
correlation. Symptoms: when running server + two clients (e.g.,
the friend-game route or the two-client runtime harness), aligning
events across the three processes requires an external shell
wrapper or manual log post-processing.

**Prevention target**: Configure each subscriber init site with a
UTC ISO-8601 timer at millisecond precision:

```rust
use tracing_subscriber::fmt::time::UtcTime;

tracing_subscriber::fmt()
    .with_timer(UtcTime::rfc_3339())  // or rfc_3339_with_subseconds
    .init();
```

(Exact API surface verified by the implementing worker against
`tracing-subscriber` version pinned in `Cargo.toml`; the
`liv-bevy-018` skill cross-references the current
`tracing-subscriber` docs at implementation time.)

After the change, every log line in server, client, and test runs
carries an ISO-8601 UTC timestamp with millisecond precision. The
Story 019 tighter-capture invocation no longer requires a
shell-wrapper UTC prefix. Multi-process correlation is trivial.

---

## Context

### Existing surface

- **`server/src/main.rs:87`**: server tracing init site (per PROMPT
  803 §4 Lane E).
- **`client/src/main.rs:36`**: client tracing init site.
- **`tests/test_helpers.rs:52`**: test tracing init site.
- **`Cargo.toml`**: `tracing-subscriber` is already a workspace
  dependency (per existing subscriber init calls). The `time`
  feature may need to be enabled if `UtcTime` requires it; the
  implementing worker confirms via
  `cargo tree -p tracing-subscriber -e features`.

### GDD / ADR / TR trace

- **No GDD change**: observability infrastructure.
- **ADR-002** (Client-Server Authority): unchanged.
- **TR registry**: no new TR.

### Engine

- **Engine**: Bevy 0.18 (Rust). All edits are in `.rs` source.
- **`tracing-subscriber`**: pinned version per `Cargo.toml`. The
  `UtcTime::rfc_3339()` API requires the `time` feature; the
  implementing worker enables it if not already enabled.

### Mandatory skills

- **`liv-bevy-018`** -- mandatory for `.rs` code edits.
- **`liv-bevy-lightyear`** -- mandatory only if the change touches
  any lightyear-adjacent subscriber config (it does not, per the
  three-file scope; but the skill remains active for any
  collateral edits).

### Control Manifest Rules (Observability scope)

- Required: All three subscriber init sites configure a UTC
  ISO-8601 timer at millisecond precision.
- Required: The timer format is consistent across all three sites
  (i.e., the same `UtcTime::rfc_3339()` or equivalent call).
- Required: After the change, sample log output from each of the
  three sites carries a valid ISO-8601 UTC timestamp at ms
  precision; a regex like `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z\b`
  matches every line.
- Required: Behaviour change is zero -- existing tests pass; no
  runtime semantics modified.
- Forbidden: Changing log message contents or levels.
- Forbidden: Removing the existing `.init()` call or replacing
  with a non-fmt subscriber.
- Forbidden: Adding any new logging output.

---

## Story Classification

**Story type**: Integration -- subscriber-config tweak in three
files.

This is **NOT** a:

- New-feature story.
- Refactor of tracing infrastructure beyond the timer config.
- Sprint 12 expansion.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Server subscriber config landed**: GIVEN the diff at
  `server/src/main.rs:87` (or the relevant line range post-edit),
  WHEN the subscriber init is inspected, THEN it calls
  `.with_timer(...)` with a UTC ISO-8601 timer at millisecond
  precision.

- [ ] **AC2 -- Client subscriber config landed**: same for
  `client/src/main.rs:36`.

- [ ] **AC3 -- Test subscriber config landed**: same for
  `tests/test_helpers.rs:52`.

- [ ] **AC4 -- Timer format consistent across three sites**: GIVEN
  the diff, WHEN the timer-construction expression is compared
  across the three files, THEN the three expressions are
  semantically identical (e.g., `UtcTime::rfc_3339()` everywhere).

- [ ] **AC5 -- `Cargo.toml` feature flag added if required**: GIVEN
  the implementing worker's check of `tracing-subscriber` feature
  flags, WHEN `UtcTime` requires the `time` feature and it is not
  already enabled, THEN the workspace `Cargo.toml` (or the
  relevant per-crate `Cargo.toml`) enables the feature with an
  inline rationale comment.

- [ ] **AC6 -- Sample log output carries ISO-8601 UTC ms-precision
  timestamps**: GIVEN the implementation commit, WHEN any one of
  (a) `cargo run --bin server`, (b) `cargo run --bin client`,
  (c) `cargo test --workspace --tests --no-fail-fast` produces
  log output, THEN every output line starts with a timestamp
  matching the regex
  `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z\b` (or the
  canonical ISO-8601 UTC ms-precision format produced by the
  chosen API). The evidence doc records sample lines from each
  binary.

- [ ] **AC7 -- Behaviour unchanged**: GIVEN `cargo test --workspace
  --tests --no-fail-fast` at the implementation commit, WHEN
  compared to the pre-implementation baseline, THEN no test
  regressions are observed (same pass/fail/ignored counts modulo
  Sprint 12 close-out deltas).

- [ ] **AC8 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed for any
  client-side mutation of authoritative state outside the
  shared phase sink, snapshot drainers, and S2C consumers,
  THEN no such mutation is present. ADR-002 binding.
  *Evidence*: text search for "no optimistic" in the evidence
  document.

- [ ] **AC9 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`,
  and `production/qa/qa-plan-sprint-12.md` are diffed, THEN
  none of them are modified under this story.

- [ ] **AC10 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-obs-wallclock-timestamps-evidence.md`
  (NEW). Records pre/post sample log lines from server, client,
  test runs; no-claim restatement; cross-link to PROMPT 803 §3
  DC-12.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `server/src/main.rs` | Subscriber init updated with `.with_timer(...)`. |
| `client/src/main.rs` | Same. |
| `tests/test_helpers.rs` | Same. |
| `Cargo.toml` (workspace or per-crate) | OPTIONAL: enable `time` feature on `tracing-subscriber` if required. |
| `production/qa/evidence/sprint-13-obs-wallclock-timestamps-evidence.md` | NEW evidence document per AC10. |
| This story file | Status updates per /story-readiness or /story-done if/when Sprint 13 activates. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for `.rs` code edits.
- **`liv-bevy-lightyear`** -- mandatory for any collateral lightyear-
  adjacent subscriber edits (not expected; included for completeness).

---

## Evidence Path

`production/qa/evidence/sprint-13-obs-wallclock-timestamps-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Pre/post sample log lines (~5 lines each) from server, client,
  and test runs.
- Regex verification output:
  `cargo run ... 2>&1 | head -50 | grep -cE '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z'`
  (post-impl: count matches line count).
- `cargo test --workspace --tests --no-fail-fast` pre/post output
  showing AC7 behaviour-unchanged.
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no optimistic client-side authority").
- Cross-link to PROMPT 803 §3 DC-12 and Story 019 (Hand UI
  tighter-capture).

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo run --bin server 2>&1 | head -20`
  (manual inspection for ISO-8601 UTC ms-precision timestamps)
- `cargo run --bin client 2>&1 | head -20`
  (same)
- `cargo test --workspace --tests --no-fail-fast 2>&1 | head -50 | grep -cE '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}'`
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- **Adding module-scoped tracing targets**. Scoped to
  `S13-OBS-TRACING-TARGETS-001` (Story 018 in this epic).
- **Replacing `tracing_subscriber::fmt()` with a different
  subscriber** (e.g., `tracing-bunyan-formatter` or a JSON
  formatter). The change is minimal: add `.with_timer(...)`.
- **CI integration changes** (log aggregation, log routing).
- **Persisting logs to disk by default**. The harness
  (`S13-TWO-CLIENT-RUNTIME-HARNESS-001`) handles log capture; this
  story only ensures the timestamp format is correlation-ready.
- **Sprint 13 activation**.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  or any carried Sprint condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Touches `server/src/main.rs`, `client/src/main.rs`,
  `tests/test_helpers.rs`** (and optionally `Cargo.toml`). Sprint
  12 Must Have rows do not touch these files:
  - Story 012/013/014/015/019 (Sprint 12) touch fixture / lobby /
    board-rendering test files and the hand-ui tighter-capture
    evidence file; **disjoint from this story's file scope**.
- **No Sprint 12 invasion**: this story's implementation is
  parallel-safe with Sprint 12 close-out paperwork, but the
  default policy is "wait for Sprint 12 close-out before Sprint
  13 implementation lands" to avoid muddying the diff. If the
  producer opts to pull this story forward (e.g., to unblock
  Sprint 12 Story 019 tighter-capture without a shell-wrapper),
  the pull-forward is a separate prompt with explicit producer
  authorisation.
- **Coordinate with `S13-OBS-TRACING-TARGETS-001` (Story 018 in
  this epic)**: ideally land both in the same Sprint 13 wave.
- **Coordinate with `S13-TWO-CLIENT-RUNTIME-HARNESS-001` (Story
  017 in this epic)**: the harness's UTC-prefixing shim becomes
  redundant after this story lands; the harness can be simplified
  in a follow-on.
- **No shared-status writer overlap**: `production/sprint-status.yaml`
  is not touched by this story.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. Sprint 12 reaches close-out (or producer-authorised pull-forward).
2. Sprint 13 is planned via `/sprint-plan sprint-13`.
3. This story passes `/story-readiness`.
4. Sprint 13 `/qa-plan sprint` is authored.
5. `/dev-story story-019-obs-wallclock-timestamps.md` is dispatched.

Expected implementation flow:

1. **Wave 1 -- Feature-flag check**: confirm
   `tracing-subscriber` has the `time` feature; enable it in
   `Cargo.toml` if missing (AC5).
2. **Wave 2 -- Three-file edit**: add `.with_timer(UtcTime::rfc_3339())`
   to each of the three subscriber init sites.
3. **Wave 3 -- Verification**: run server, client, tests; spot-check
   sample log lines; verify AC6 regex matches.
4. **Wave 4 -- Behaviour check**: `cargo test --workspace --tests
   --no-fail-fast`.
5. **Wave 5 -- Evidence**: populate evidence file.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `tracing-subscriber` `time` feature is gated and adds non-trivial dependency weight | Low | Low | Feature flag is small (pulls in `time` crate which is already widely depended on). Implementation prompt records the dependency-graph impact in the evidence doc. |
| Test output timestamp format breaks a downstream log-parsing script | Low | Low-Medium | No known downstream parsers depend on the relative-seconds format; evidence doc records this. |
| WASM/browser logs use a different subscriber path that doesn't get the timer | Medium | Low | `client/src/main.rs:36` covers the dev/desktop client; the WASM build may use a different init path (`tracing-wasm` or similar). Implementation prompt audits and applies the same timer config to the WASM path if applicable. |
| Sprint 13 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate. |

---

## Verification (orchestrator-side, before worker dispatch)

- `production/sprint-status.yaml` `sprint:` field reads `13` after
  Sprint 13 activation (or producer-authorised pull-forward).
- `production/stage.txt` reads `Polish` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for ISO-8601 UTC Wall-Clock Timestamps per PROMPT 803
  §3 DC-12 / §5 Must row 8. Sprint 12 is `active` (PROMPT 798) and
  is not modified by this authoring run. No code changes, no
  smoke / gate / QA / `/dev-story` / `/story-done` / `/story-readiness` /
  `/qa-plan` run. Source-of-truth at authoring: `origin/main@b5eef0d`.
  Worker branch: `work/s13-runtime-hardening-story-authoring`.
  Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.
