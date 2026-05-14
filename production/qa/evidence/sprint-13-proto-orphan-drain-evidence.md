# Sprint 13 — S13-PROTO-ORPHAN-DRAIN-001 Evidence

> **Story**: `production/epics/lightyear-protocol-verification/story-008-protocol-orphan-drain.md`
> **Implementation prompt**: PROMPT 852 (2026-05-14)
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-protocol-orphan-drain`
> **Worker branch**: `work/s13-protocol-orphan-drain`
> **Source-of-truth at start**: `origin/main@25573e6` (PROMPT 849 integration of `S13-PROTO-INVARIANT-001`).

---

## No-Claim Banner (verbatim from Story 008)

> This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
> activated by PROMPT 804. Sprint 12 remains the active sprint
> (`status: active` per `production/sprint-status.yaml` at
> `origin/main@b5eef0d`) and must not be changed by this authoring run.
> Activation of Sprint 13 happens via a separate `/sprint-plan sprint-13`
> prompt after Sprint 12 close-out.
>
> **No optimistic client-side authority is introduced or proposed by this
> story.** Each orphan disposition (drain vs delete) lands a server-
> authoritative drain (client-side draining server-broadcast S2C state
> read-only into client view) or removes the unused message; no
> disposition allows the client to mutate authoritative state outside the
> existing shared phase sink / snapshot / S2C consumer pattern.

This implementation prompt (PROMPT 852) does **not** modify
`production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
`production/stage.txt`, `production/qa/qa-plan-sprint-12.md`, or any
`production/session-state/*` file (AC10 binding). It does **not** run
`/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
`/release-check`, or `/qa-plan`. Sprint 12 disposition
(`closed-with-conditions` per PROMPT 817) and Sprint 11 disposition
(`closed-with-conditions` per PROMPT 792) and Sprint 10 disposition
(`closed-with-conditions` per PROMPT 763) remain unchanged.

---

## Per-orphan disposition table

| Orphan | Direction | Path chosen | Drain/handler location | Notes |
|---|---|---|---|---|
| `S2CHeartbeat` | S2C | **B — Delete** | n/a | No producer or consumer ever existed; GDD Rule 8 only specifies `C2SHeartbeat`. Type + channel registration removed in `shared/src/protocol.rs`; `tests/integration/network/e2e_websocket_test.rs` updated to drop the S2C-unreliable assertions (C2SHeartbeat C→S unreliable + S2CHandshakeRejected reliable still asserted). |
| `S2COpponentDisconnected` | S2C | **A — Drain** | `client/src/presentation/mod.rs::drain_opponent_connection_messages` → `OpponentConnectionView` | Server-side broadcast sender remains absent from the workspace (per Story 008 disposition, "out-of-scope for this story"); allowlisted as missing-send in the invariant test with a follow-on reference. |
| `S2COpponentReconnected` | S2C | **A — Drain** | `client/src/presentation/mod.rs::drain_opponent_connection_messages` → `OpponentConnectionView` | Live producer at `server/src/core/session/reconnect.rs:54-58,231`. Drain clears the disconnect indicator on receipt; paired with `S2COpponentDisconnected` in the same system per the story. |
| `S2CPoolUpdate` | S2C | **B — Delete** | n/a | No producer or consumer. GDD Table A row replaced with a deletion-note HTML comment; cross-references in §VI and §VIII updated to point at `S2CGameSnapshot.PlayerSnapshot.pool_snapshot`. |
| `S2CPrismRespawned` | S2C | **A — Drain** | `client/src/presentation/mod.rs::drain_prism_lifecycle_messages` → `PrismLifecycleView::last_respawn` | Live producer at `server/src/feature/prism/system.rs:513-531`. Drain updates a read-only presentation view; never mutates authoritative state. |
| `S2CPrismRewardDropped` | S2C | **A — Drain** | `client/src/presentation/mod.rs::drain_prism_lifecycle_messages` → `PrismLifecycleView::pending_rewards_lost` | Live producer at `server/src/feature/prism/system.rs:467-497` + `server/src/core/session/reconnect.rs:755-779` (reconnect replay). Drain appends each event to a read-only view. |
| `S2CSangMepriseReveal` | S2C | **C — Defer (split)** | n/a (drain pending) | Live producer at `server/src/core/session/reconnect.rs:54,479-490,998-1005`. Drain wiring deferred until the "Sang Méprise reveal mechanism" ADR is Accepted (`.claude/docs/technical-preferences.md` Pending ADRs). Allowlisted in the invariant test with follow-on `S14-PROTO-SANG-MEPRISE-DRAIN-001`. |
| `S2CSessionCancelled` | S2C | **A — Drain** | `client/src/presentation/mod.rs::drain_session_lifecycle_messages` → `SessionLifecycleView` | Live producer set: `server/src/core/session/system.rs:2075,2143`, `state.rs:126,234,240,252`, `reconnect.rs:581-593` (deferred replay). Drain records the cancellation reason; read-only. |
| `C2SRequestSnapshot` | C2S | **A — Handler** | `server/src/core/session/snapshot_request.rs::handle_request_snapshot` | New exclusive system in `GameSessionPlugin` (LiveMessages set). Reuses `build_game_snapshot` (ADR-011 binding). Rate-limited by new `GameConfig::snapshot_cooldown_ms` (default 5000ms, per GDD Table A). Legacy `handle_c2s_message` stub TODO removed from `server/src/main.rs`. |
| `C2SClassChoice` (additional) | C2S | **Allowlisted (out of scope)** | n/a | Surfaced by PROMPT 845 beyond the 9 PROMPT 803 §4 Lane A named orphans. Server drain at `server/src/lobby/handler.rs:15`; client lobby uses `C2SSelectClass` + `C2SConfirmClass` instead. Disposition deferred to `S14-PROTO-CLASSCHOICE-DISPOSITION-001`. |

---

## Changed files

`git diff --name-only origin/main...HEAD` (PROMPT 852 worker tip):

| Path | Change |
|---|---|
| `shared/src/protocol.rs` | Path B deletions of `S2CHeartbeat` and `S2CPoolUpdate` (type defs + channel registration); inline deletion notes added at registration call sites. |
| `shared/src/config.rs` | Added `snapshot_cooldown_ms: u32` (default 5000) to `GameConfig` with doc comment citing network-protocol.md Table A. |
| `assets/config/game_config.ron` | Added matching `snapshot_cooldown_ms: 5000` entry. |
| `design/gdd/network-protocol.md` | Removed `S2CPoolUpdate` Table A row + deletion note; updated §VI "Card Data & Pool" inventory + §VIII Source matrix + §IX cross-reference note. |
| `client/src/state/mod.rs` | New resources: `OpponentConnectionView`, `PrismLifecycleView`, `SessionLifecycleView`. New apply-functions: `apply_opponent_disconnected_message`, `apply_opponent_reconnected_message`, `apply_prism_respawned_message`, `apply_prism_reward_dropped_message`, `apply_session_cancelled_message`. |
| `client/src/presentation/mod.rs` | Added `drain_opponent_connection_messages`, `drain_prism_lifecycle_messages`, `drain_session_lifecycle_messages` systems in the existing `MessageDrain` set; registered the three new resources in `PresentationPlugin::build`. |
| `server/src/core/session/mod.rs` | Added `pub mod snapshot_request;` and re-exported `handle_request_snapshot` + `SnapshotRequestCooldowns`. |
| `server/src/core/session/snapshot_request.rs` | NEW — exclusive `handle_request_snapshot` system, `SnapshotRequestCooldowns` resource, cooldown math + per-player tracking. |
| `server/src/core/session/plugin.rs` | Imported `handle_request_snapshot` + `SnapshotRequestCooldowns`; registered the resource and scheduled the system in `SessionSystemSet::LiveMessages`. |
| `server/src/main.rs` | Removed the legacy `handle_c2s_message` stub function + its TODO block. |
| `tests/invariants/protocol_completeness_test.rs` | Removed `#[ignore]` from `protocol_completeness_assert_send_and_drain_sites`; updated docstring + companion-test S2C-count assertion (32 instead of 34 after Path B deletions); added `AllowlistEntry` + `MissingSide` types and the 3-entry allowlist (`S2CSangMepriseReveal` drain — Path C; `C2SClassChoice` send — out-of-scope; `S2COpponentDisconnected` send — follow-on per story disposition). |
| `tests/integration/network/e2e_websocket_test.rs` | Removed `S2CHeartbeat` references (Path B); the unreliable C2S + reliable S2C roundtrip checks remain intact. |
| `tests/integration/presentation/protocol_orphan_drain_test.rs` | NEW — 6 tests covering lifecycle drain application + prism drain application + single-drainer source guards + PresentationPlugin resource init. |
| `tests/integration/session/request_snapshot_handler_test.rs` | NEW — 5 tests covering cooldown math, plugin wiring, single-drainer source guard, and legacy stub removal. |
| `client/Cargo.toml` | Registered `presentation_protocol_orphan_drain_test`. |
| `server/Cargo.toml` | Registered `request_snapshot_handler_test`. |
| `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md` | NEW (this document). |

---

## Verification — targeted checks

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 — clean |
| `cargo check -p shared` | exit 0 — clean |
| `cargo check -p server` | exit 0 — clean |
| `cargo check -p client` | exit 0 — clean |
| `cargo check --tests -p client` | exit 0 — clean (one pre-existing dead-code warning in `hand_ui_asset_wiring_test.rs` unrelated to this story) |
| `cargo check --tests -p server` | exit 0 — clean |
| `cargo test -p shared --test protocol_completeness_invariant` | **2/2 PASS** (`protocol_manifest_parser_discovers_registered_messages` + `protocol_completeness_assert_send_and_drain_sites`, the latter no longer `#[ignore]`-gated; allowlist consumes the 3 retained orphans) |
| `cargo test -p client --test presentation_protocol_orphan_drain_test` | **6/6 PASS** |
| `cargo test -p server --test request_snapshot_handler_test` | **5/5 PASS** |
| `git diff --check origin/main...HEAD` | exit 0 — no whitespace errors |

Full workspace tests intentionally not run (per dispatch directive: "Do not run full workspace tests" + QA-plan-sprint-13 `no-full-workspace-tests-by-default` policy). Story 008 AC8 ("workspace test count and ignored count behave predictably") will be re-verified at sprint integration time; the invariant test's `#[ignore]` removal is the load-bearing change here and is verified above.

### Pre-drain → post-drain invariant test summary

- **Pre-drain** (at `origin/main@25573e6`, per PROMPT 845 + PROMPT 849 evidence): `protocol_completeness_assert_send_and_drain_sites` was `#[ignore]`-gated and, when forced-run, panicked with **13 violations across 10 unique types** (all 9 PROMPT 803 §4 Lane A named orphans + `C2SClassChoice`).
- **Post-drain** (at PROMPT 852 worker tip): `protocol_completeness_assert_send_and_drain_sites` is no longer `#[ignore]`-gated and passes cleanly. The 3-row allowlist (`S2CSangMepriseReveal` drain, `C2SClassChoice` send, `S2COpponentDisconnected` send) carries each retained orphan with an inline rationale + follow-on reference per AC4 ("passes with a documented allowlist where each allowlist entry has an inline rationale + follow-on story reference").

---

## AC verification matrix

| AC | Status | Evidence |
|---|---|---|
| **AC1** — Per-orphan decisions recorded | ✔ | Story file's "Per-Orphan Decisions" section (lines 310-613) carries the 9 dispositions plus the `C2SClassChoice` allowlist (recorded in this evidence doc + invariant-test ALLOWLIST entry). The decision-recording commit (PROMPT 821) precedes PROMPT 852 implementation commits. |
| **AC2** — Path A drains land with single-drainer discipline | ✔ | `tests/integration/presentation/protocol_orphan_drain_test.rs::{lifecycle_cluster_drains_are_registered_exactly_once_in_production, prism_cluster_drains_are_registered_exactly_once_in_production}` + `tests/integration/session/request_snapshot_handler_test.rs::handle_request_snapshot_is_sole_production_drain` walk `client/src/` and `server/src/` and assert exactly one `MessageReceiver<T>` for each Path A message. All pass. |
| **AC3** — Path B deletions atomic across protocol + GDD + senders | ✔ | `S2CHeartbeat` and `S2CPoolUpdate` removed atomically: (a) type defs removed in `shared/src/protocol.rs`; (b) channel-binding `register_s2c::<T>` lines removed; (c) no senders existed; (d) `design/gdd/network-protocol.md` Table A + §VI + §VIII + §IX updated for `S2CPoolUpdate` (S2CHeartbeat had no GDD row to remove); (e) `tests/integration/network/e2e_websocket_test.rs` updated to drop the `S2CHeartbeat` references. |
| **AC4** — `S13-PROTO-INVARIANT-001` test flips to PASS | ✔ | `cargo test -p shared --test protocol_completeness_invariant` passes 2/2 with no `#[ignore]`. Three-row allowlist documented inline; each entry names a follow-on story. |
| **AC5** — Integration tests per cluster | ✔ | Lifecycle cluster (3 drains) covered by `presentation_protocol_orphan_drain_test::s2c_opponent_disconnect_and_reconnect_pair_apply_to_connection_view` + `..._session_cancelled_applies_to_session_lifecycle_view` + the single-drainer guard. Prism cluster covered by `..._s2c_prism_respawned_and_reward_dropped_apply_to_lifecycle_view` + its single-drainer guard. Snapshot-request cluster covered by `request_snapshot_handler_test::{snapshot_request_cooldown_blocks_inside_window_and_releases_after_threshold, ..._tracks_each_player_independently, game_session_plugin_installs_snapshot_request_cooldowns_resource, handle_request_snapshot_is_sole_production_drain}`. |
| **AC6** — No optimistic client-side authority introduced | ✔ | The phrase **"no optimistic client-side authority"** is preserved verbatim from the Story 008 banner in this evidence document. Client drains write only to new presentation-view resources (`OpponentConnectionView`, `PrismLifecycleView`, `SessionLifecycleView`); none mutate `CurrentClientPhase`, `ClientObjectiveIdentities`, `PlayerEconomyView`, or any other authoritative-mirror resource. Server stays sole authority on `S2CGameSnapshot` contents in `handle_request_snapshot`; ADR-002 / ADR-011 bindings preserved. |
| **AC7** — No channel-binding changes for retained messages | ✔ | `git diff origin/main...HEAD -- shared/src/protocol.rs` shows only `register_s2c::<S2CHeartbeat>` and `register_s2c::<S2CPoolUpdate>` lines removed (Path B deletions). Every retained `register_c2s::<T>` / `register_s2c::<T>` line retains its prior `ProtocolChannel::Reliable` / `ProtocolChannel::Unreliable` assignment (manually verified by reading the diff). ADR-008 binding preserved. |
| **AC8** — Workspace test count behaves predictably | ✔ (partial — story-prescribed targeted checks only) | The dispatch directive forbids full-workspace test runs; AC8's "compared to the post-`S13-PROTO-INVARIANT-001` baseline" comparison is deferred to sprint integration. The load-bearing assertion ("`protocol_completeness_test` reports PASS; no new `#[ignore]` markers introduced") is verified directly: `cargo test -p shared --test protocol_completeness_invariant` passes 2/2; no new `#[ignore]` attribute was added by PROMPT 852 (the existing one on `protocol_completeness_assert_send_and_drain_sites` was REMOVED). |
| **AC9** — If split chosen, per-message follow-on stories authored | n/a | Umbrella path chosen per PROMPT 821 (Story 008 "Producer Decision" §[x]); only `S2CSangMepriseReveal` is recorded as a Path C deferral. The Sprint 14 candidate story file for that deferral (`S14-PROTO-SANG-MEPRISE-DRAIN-001`) was NOT authored by PROMPT 852 per Story 008 Path C rationale ("PROMPT 821 does NOT author the Sprint 14 candidate story (paperwork-only run; story-file authoring is a separate paperwork prompt)"). |
| **AC10** — Sprint 12 disposition preserved | ✔ | `git diff --name-only origin/main...HEAD` does not include `production/sprint-status.yaml`, `production/sprints/sprint-12.md`, `production/sprints/sprint-13.md`, `production/stage.txt`, or `production/qa/qa-plan-sprint-12.md`. Stage = `Polish` unchanged. Sprint 11 + Sprint 10 dispositions unchanged. PROMPT 761 Polish→Release gate-check FAIL evidence at `production/gate-checks/gate-polish-release-2026-05-12.md` preserved (not in diff). |
| **AC11** — Evidence document slot reserved | ✔ | This file (`production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`) is NEW and contains all required evidence content per Story 008's evidence-doc spec. |

---

## Cross-references

- **PROMPT 803 source finding** (audit roadmap): `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md` §3 DC-1 + §4 Lane A.
- **PROMPT 845 worker report** (invariant test landing): `reports/PROMPT-845-S13-PROTOCOL-COMPLETENESS-INVARIANT.md`.
- **PROMPT 849 integration report** (invariant test merged to main): `reports/PROMPT-849-S13-PROTOCOL-COMPLETENESS-INVARIANT-INTEGRATION.md`.
- **Story file**: `production/epics/lightyear-protocol-verification/story-008-protocol-orphan-drain.md`.
- **Pre-drain invariant evidence**: `production/qa/evidence/sprint-13-proto-invariant-evidence.md` (pre-drain test output).

---

## Cargo policy applied

```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

Applied to every Cargo invocation. No disk cleanup needed (D: drive had ample free space; no Cargo command failed on disk).

---

## Out of scope (explicitly NOT claimed by PROMPT 852)

- Public release readiness, release-candidate readiness, full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Full playable-client manual QA.
- Two-client `GAME_OVER` closure (`S8-QA-001-W1`).
- Final-art / asset-production completion.
- Sprint 13 `/story-done` row flip for `S13-PROTO-ORPHAN-DRAIN-001` (deferred to a future `/story-done` paperwork prompt; this worker does not edit `production/sprint-status.yaml`).
- Full-workspace `cargo test --workspace --tests --no-fail-fast` (deferred to sprint integration).
- `S2COpponentDisconnected` server-broadcast send-site (out-of-scope per story; allowlisted as follow-on).
- `S2CSangMepriseReveal` client drain (Path C deferral until reveal-mechanism ADR is Accepted).
- `C2SClassChoice` drain-vs-delete disposition (out-of-scope; allowlisted as follow-on).
