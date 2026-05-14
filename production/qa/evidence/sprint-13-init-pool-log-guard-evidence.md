# Sprint 13 -- Server `init_pool` Log Guard Evidence

> **Story**: `S11-SERVER-POOL-INIT-LOG-GUARD-001`
> **Story file**: `production/epics/server/story-001-init-pool-log-guard.md`
> **Sprint**: Sprint 13 (Should Have)
> **Author**: PROMPT 829 (`/dev-story` implementation)
> **Source-of-truth at start**: `origin/main@4bf95fa` (PROMPT 827 Sprint 13 QA plan)
> **Worker branch**: `work/s13-server-pool-init-log-guard`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-server-pool-init-log-guard`

---

## No-Claim Restatement (verbatim from story)

PROMPT 829 implements the log-guard fix only. PROMPT 829 does **not**
claim: public release readiness, release-candidate readiness, full game
completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved. Stage remains
`Polish`. Sprint 13 disposition unchanged (`active` per PROMPT 826).

**No client-side authority is introduced or proposed by this story.**
The fix is server-side only: gate the `init_pool` info-level log on
the existing initialization guard. ADR-002 binding preserved.

---

## AC1 -- Source located (file:line evidence)

Pre-fix source — `server/src/core/pool/system.rs`:

```rust
// server/src/core/pool/system.rs (pre-fix, lines 14-37 at origin/main@4bf95fa)
pub fn initialize_player_pools_on_draft_started(
    mut draft_started: MessageReader<DraftStarted>,
    session: Res<SessionConfig>,
    catalog: Res<CardCatalog>,
    config: Res<GameConfig>,
    mut pools: ResMut<PlayerPools>,
) {
    tracing::info!(                                                  // <- line 21: pre-guard info!
        "initialize_player_pools_on_draft_started: entered (session=true, catalog=true, config=true)"
    );

    for message in draft_started.read() {                            // <- line 25: existing guard #1
        if message.phase != DraftPhase::Initial {                    // <- line 26: existing guard #2
            continue;
        }

        pools.pools.clear();
        for player in session.players() {
            pools
                .pools
                .insert(player, PlayerPool::initialize(&catalog.cards, &config.0));
        }
    }
}
```

- **`info!` call site (pre-fix)**: `server/src/core/pool/system.rs:21`
- **Existing initialization guard**: `server/src/core/pool/system.rs:25-28`
  (the `for message in draft_started.read() { if message.phase !=
  DraftPhase::Initial { continue; } ... }` block)

The pre-fix `info!` fires unconditionally every tick the system is
scheduled, regardless of whether any `DraftStarted` message exists or
whether the `phase != DraftPhase::Initial` continue-guard short-circuits
the body. This is the spam source.

---

## AC2 + AC3 -- Log relocated post-guard (W5 pattern match)

Post-fix source — `server/src/core/pool/system.rs`:

```rust
pub fn initialize_player_pools_on_draft_started(
    mut draft_started: MessageReader<DraftStarted>,
    session: Res<SessionConfig>,
    catalog: Res<CardCatalog>,
    config: Res<GameConfig>,
    mut pools: ResMut<PlayerPools>,
) {
    // Log point 1: system entry. Downgraded to debug! to avoid per-frame spam
    // (S11-SERVER-POOL-INIT-LOG-GUARD-001) — info-level retained only when an
    // actual DraftStarted::Initial message is drained below. Matches the
    // Sprint 11 W5 ee27fb6 acquisition_tick pattern.
    tracing::debug!(
        "initialize_player_pools_on_draft_started: entered (session=true, catalog=true, config=true)"
    );

    for message in draft_started.read() {
        if message.phase != DraftPhase::Initial {
            continue;
        }

        // Log point 2: fire info! only after the DraftPhase::Initial guard
        // permits work — the per-frame entry case happens every tick in idle
        // and was the spam source. W5-fix pattern.
        tracing::info!(
            "initialize_player_pools_on_draft_started: initializing PlayerPools on DraftStarted::Initial"
        );

        pools.pools.clear();
        for player in session.players() {
            pools
                .pools
                .insert(player, PlayerPool::initialize(&catalog.cards, &config.0));
        }
    }
}
```

### Diff summary (vs `origin/main@4bf95fa`)

- **`info!` -> `debug!` at entry**: the pre-guard log keeps its message
  string verbatim but is rescoped from `info!` to `debug!` (preserves
  diagnostic capture under `RUST_LOG=server=debug` while removing
  per-tick info-level spam).
- **New `info!` post-guard**: a new `info!` call is inserted *after* the
  `if message.phase != DraftPhase::Initial { continue; }` guard so it
  fires only when the function actually permits initialization work
  (the cold-path case).
- **Guard logic unchanged**: the `for message in draft_started.read() {
  if message.phase != DraftPhase::Initial { continue; } ... }` block is
  byte-identical to the pre-fix version. Forbidden change avoided.
- **No suppression**: the log remains diagnostically useful at both
  levels (debug = entry trace, info = actual init event). Forbidden
  suppression avoided.

### Cross-link to Sprint 11 W5 fix `ee27fb6`

Sprint 11 W5 fix (commit `ee27fb6 fix(observability): rescope
acquisition_tick per-frame log spam (PROMPT 681 /
S11-TD-SERVER-LOG-SPAM-001)`) applied the same pattern to
`server/src/feature/acquisition/system.rs::card_acquisition_tick_system`:

- Downgraded `tracing::info!("acquisition_tick: system entered")` to
  `tracing::debug!(...)` at the system entry.
- Gated the `tracing::info!("acquisition_tick: drained N
  ShopRefreshTriggered messages", ...)` call on `!is_empty()` so the
  info-level emission fires only when a message is actually drained.

PROMPT 829's fix to `initialize_player_pools_on_draft_started` is the
identical pattern: pre-guard `info!` downgraded to `debug!`; new
post-guard `info!` fires only when the `DraftPhase::Initial` guard
permits work. AC3 satisfied.

---

## AC4 -- Smoke / log evidence (cold-path emission count)

**Per QA-plan-sprint-13 policy**: end-of-sprint smoke is required and
runs at the integration / sprint close-out merge, not at per-row
implementation time. PROMPT 829 does NOT run `/smoke-check` (forbidden
by the prompt's scope and by `production/qa/qa-plan-sprint-13.md`'s
serialization policy: only one smoke run per sprint at a time).

**Static analysis of cold-path emission count**:

In the post-fix code, the `tracing::info!` at log point 2 fires **only**
inside the body of the `for message in draft_started.read()` loop AND
only after the `if message.phase != DraftPhase::Initial { continue; }`
guard. Therefore the info-level emission count per session is bounded
by:

```
N_info <= count(DraftStarted messages where phase == DraftPhase::Initial)
```

`DraftPhase::Initial` is a one-shot phase transition in the round-state
machine — it fires exactly **once** per session at draft start (see
`server/src/core/rsm/` for the RSM definition; one `DraftStarted` with
`phase: DraftPhase::Initial` is emitted per session at game-start).
Even pathologically (session restart, fixture re-run, double-init under
tests), N_info is bounded by the number of session restarts, which is
< 50 per cold-path session under any realistic test scenario.

**Conclusion**: AC4's `<50 emitted lines per session` target is
satisfied by construction. A runtime smoke capture confirming the
empirical count is deferred to the sprint-close integration smoke run.

---

## AC5 -- No client-side change

`git diff origin/main...HEAD -- 'client/**'` -> empty. AC5 satisfied.

## AC6 -- No protocol change

`git diff origin/main...HEAD -- 'shared/**'` -> empty. AC6 satisfied.

## AC7 -- Targeted server tests pass

Per QA-plan-sprint-13's binding "no-full-workspace-tests-by-default
policy for implementation workers (per-row narrowest BLOCKING command
only; orchestrator runs full-workspace gate at integration merge)",
PROMPT 829 runs the narrowest BLOCKING command matching the W5 ee27fb6
precedent: `cargo test -p server --lib`.

Results: see "Verification" section below.

`cargo test --workspace --tests --no-fail-fast` is the orchestrator's
integration-merge gate, not a per-row worker command. It is NOT run by
PROMPT 829.

## AC8 -- Sprint 13 disposition preserved

`git diff origin/main...HEAD --` against the forbidden-set:

- `production/sprint-status.yaml` -> not modified.
- `production/sprints/sprint-13.md` -> not modified.
- `production/stage.txt` -> not modified.
- `production/gate-checks/gate-polish-release-2026-05-12.md` -> not
  modified.

AC8 satisfied.

## AC9 -- Evidence document (this file)

`production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md` is
this file. AC9 satisfied.

---

## Files changed by PROMPT 829

| Path | Change |
|------|--------|
| `server/src/core/pool/system.rs` | One log relocation: pre-guard `info!` -> `debug!`; new `info!` added post-guard (lines 21-28 in new file). Guard logic unchanged. |
| `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md` | NEW (this file). |
| `reports/PROMPT-829-S13-SERVER-POOL-INIT-LOG-GUARD.md` | NEW (final report mirror; under gitignored `reports/`, not staged). |

No other files touched. `production/sprint-status.yaml`,
`production/sprints/sprint-13.md`, `production/stage.txt`,
`production/session-state/*`, `client/**`, `shared/**`, `tests/**`,
`Cargo.toml`, `Cargo.lock`, `.claude/*`, `.octogent/*` -- all unchanged
by PROMPT 829.

---

## Verification commands run by PROMPT 829

Results recorded in the final PROMPT-829 report.

- `cargo fmt -p server -- --check`
- `cargo check -p server`
- `cargo test -p server --lib` (W5 precedent — narrowest BLOCKING
  command per QA-plan-sprint-13's per-row policy)
- `git diff --check origin/main...HEAD`
- `git diff origin/main...HEAD -- 'client/**'` (AC5)
- `git diff origin/main...HEAD -- 'shared/**'` (AC6)

Cargo resource policy on Windows/MSVC applied for every Cargo
invocation:

```
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

---

## Cross-links

- Story: `production/epics/server/story-001-init-pool-log-guard.md`
- Sprint plan: `production/sprints/sprint-13.md`
- QA plan: `production/qa/qa-plan-sprint-13.md`
- Sprint 11 close-out source row:
  `production/sprint-status.yaml` — `S11-SERVER-POOL-INIT-LOG-GUARD-001`
- Sprint 12 close-out deferral (PROMPT 817):
  `production/sprints/sprint-12.md`
- W5 pattern reference: commit `ee27fb6` —
  `server/src/feature/acquisition/system.rs::card_acquisition_tick_system`
