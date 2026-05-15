# Sprint 13 — S13-CONN-LOST-UX-001 Evidence

> **Story**: `production/epics/playable-client/story-021-conn-lost-ux.md`
> **PROMPT**: 889 (`/dev-story` implementation)
> **Branch**: `work/s13-conn-lost-ux`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-conn-lost-ux`
> **Source-of-truth at start**: `origin/main@12ae4cf` (PROMPT 888
> `/story-done` for S13-OPS-WIN-APPCOMPAT-NOTE-001)

---

## Status / No-Claim Banner

This evidence document covers the **implementation** of Story 021's proactive
Reconnecting / Connection Lost overlay. It is a visible UX building block
only. It does **not** close `S8-QA-001-W1` and does **not** claim full manual
QA.

PROMPT 889 (this implementation run) does NOT:

- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md` or any other sprint file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify `production/qa/qa-plan-sprint-12.md`.
- Modify `shared/src/protocol.rs` or anything under `server/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Activate Sprint 14 or perform Sprint 13 close-out.
- Retry the PROMPT 761 Polish->Release gate-check.
- Close `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` (a separate paperwork
  prompt is required — see AC8 cross-reference below).

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER closure
(`S8-QA-001-W1`), or final-art / asset-production completion.

**No optimistic client-side authority is introduced or proposed by this
overlay.** The overlay reads from lightyear transport events
(server-authoritative: the transport-layer disconnect signal) and from the
existing `CurrentClientPhase` resource; it does not synthesise any
game-state change or mutate authoritative state. The overlay is a
presentation read-only over the transport state. ADR-002 + ADR-008 +
ADR-011 binding.

---

## Files Changed

| File | Status | Anticipated change | Notes |
|------|--------|--------------------|-------|
| `client/src/presentation/connection_lost_overlay.rs` | NEW | Overlay plugin, observers, dismiss + sync systems, UI nodes. | 265 LOC. Read-only over transport state. |
| `client/src/presentation/mod.rs` | EDIT | Add `pub mod connection_lost_overlay;` and register `ConnectionLostOverlayPlugin` after `ResultScreenPlugin` per ADR-021. | +6 LOC. No behavioural change to other overlay/HUD plugins. |
| `client/Cargo.toml` | EDIT | Add `[[test]] connection_lost_overlay_test`. | +6 LOC. |
| `tests/integration/playable_client/connection_lost_overlay_test.rs` | NEW | 16 targeted tests covering AC1, AC2, AC3, AC4, AC5, AC7, AC9, AC10 (source-grep + World-based system tests). | New test file; passes 16/16. |
| `production/qa/evidence/sprint-13-conn-lost-ux-evidence.md` | NEW | This document. | AC13. |

---

## Overlay Module Diff Summary

`client/src/presentation/connection_lost_overlay.rs` (NEW) introduces:

- `pub struct ConnectionLostOverlayPlugin` — registers state resource,
  Startup spawn system, dismiss + sync Update systems, and two observers:
  `on_transport_disconnected` (`On<Add, Disconnected>`) +
  `on_transport_connected` (`On<Add, Connected>`).
- `pub struct ConnectionLostOverlayState { visible: bool }` — read-only
  boolean projection of transport state. No game-state coupling.
- `pub struct ConnectionLostOverlayEntities { root, panel, headline, body }` —
  spawned UI node handles for the sync system.
- `pub fn should_show_overlay_for_client_state(state) -> bool` — gates the
  overlay to `ClientState::InSession` per AC3 scoping.
- `pub fn overlay_dismissed_by_phase(phase) -> bool` — returns true only for
  `RoundPhase::GameOver` per AC5.
- `pub fn handle_transport_disconnected_event` /
  `pub fn handle_transport_connected_event` — pure handlers exposed so the
  integration test exercises the same flow without constructing lightyear's
  internal `Disconnected` / `Connected` marker components.
- Spawn function: full-screen absolute root with
  `BackgroundColor(Color::srgba(0.02, 0.025, 0.035, 0.32))` (alpha 0.32, less
  than the result screen's 0.46 so the gameplay UI remains visible
  underneath per AC7), `GlobalZIndex(90)` (below result screen's 100 per
  AC5/AC7 layering), centred amber-bordered panel with "Connection Lost"
  headline + "Reconnecting..." body.

The module is wired into `PresentationPlugin::build` after `ResultScreenPlugin`:

```rust
app.add_plugins(ResultScreenPlugin);
// S13-CONN-LOST-UX-001 (Story 021): proactive Reconnecting / Connection
// Lost overlay registered after ResultScreenPlugin per ADR-021.
// Z-ordering (90 vs result screen 100) keeps the result screen on top
// if GameOver lands while the overlay is up.
app.add_plugins(ConnectionLostOverlayPlugin);
```

---

## UX-Designer Consultation Note

Recorded inline per story §UX surface (the story explicitly allows
"in-conversation rationale recorded in the evidence doc"). Friend-game-tier
placeholder accepted per `PAW-TD-*-a` accept-risk.

| Decision | Rationale |
|----------|-----------|
| Modal (not banner). | Severity matches "connection lost"; a passive banner under-communicates the state. |
| Backdrop alpha 0.32 (vs result-screen 0.46). | AC7: gameplay UI must remain visible underneath the overlay. Lower alpha so hand/HUD/board are still readable. |
| Z-index 90 (vs result-screen 100). | AC5: result screen takes over on `GameOver`. Keeping the overlay below 100 makes the transition visually clean even if both happen in the same frame. |
| Amber border + cool-amber background (`Color::srgba(0.16, 0.10, 0.04, 0.92)` panel, `Color::srgba(0.96, 0.74, 0.30, 0.85)` border). | Warning hue distinct from the result screen's neutral grey panel + golden focus border. Reads as "something is wrong, but recoverable". |
| No countdown. | Reconnect-window duration is not currently surfaced to the client (story §Context). Out of scope per story §Out of Scope: "Reconnect-window duration changes or per-session customisation". |
| Headline "Connection Lost" + body "Reconnecting...". | Direct, tone-consistent with the result-screen disconnect copy ("Your connection was lost beyond the grace window"). |
| Lobby-state disconnect → no overlay. | Lobby already has its own status display ("Connecting") at `client/src/ui/lobby.rs:96`. Showing the overlay would duplicate the surface. AC3 scopes the overlay to gameplay phases. |

---

## Integration-Test Pass Output

Command (under the binding Cargo resource policy):

```
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p client --test connection_lost_overlay_test
```

Result:

```
running 16 tests
test ac10_no_protocol_or_server_changes_in_story_scope ... ok
test ac2_predicate_should_show_overlay_only_in_session ... ok
test ac3_disconnect_handler_marks_overlay_visible_in_session ... ok
test ac7_overlay_z_index_is_below_result_screen ... ok
test ac4_connected_handler_marks_overlay_hidden ... ok
test ac2_overlay_subscribes_to_lightyear_transport_event_sources ... ok
test ac7_overlay_backdrop_alpha_lets_gameplay_show_through ... ok
test ac5_dismiss_system_is_noop_during_active_gameplay ... ok
test ac1_plugin_registered_in_presentation_composition_order ... ok
test ac9_overlay_module_does_not_mutate_authoritative_state ... ok
test ac5_dismiss_system_hides_overlay_when_phase_is_game_over ... ok
test ac3_disconnect_handler_does_not_mark_overlay_visible_in_lobby ... ok
test ac4_connected_handler_is_idempotent_when_hidden ... ok
test ac5_predicate_overlay_dismissed_by_phase_only_at_game_over ... ok
test ac1_overlay_root_carries_marker_component_for_query_targeting ... ok
test ac3_ac4_sync_system_mirrors_state_to_root_visibility ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured
```

No new `#[ignore]` markers were introduced (AC12 partial).

---

## AC Verification Trace

| AC | Status | Evidence |
|----|--------|----------|
| AC1 — overlay module exists, registered per ADR-021 | PASS | `client/src/presentation/connection_lost_overlay.rs` exists. `client/src/presentation/mod.rs` registers `ConnectionLostOverlayPlugin` after `ResultScreenPlugin`. Tests `ac1_plugin_registered_in_presentation_composition_order` + `ac1_overlay_root_carries_marker_component_for_query_targeting`. |
| AC2 — subscribes to transport-event source | PASS | `on_transport_disconnected: On<Add, Disconnected>` + `on_transport_connected: On<Add, Connected>` observers registered via `app.add_observer(...)`. Test `ac2_overlay_subscribes_to_lightyear_transport_event_sources` (source grep) + `ac2_predicate_should_show_overlay_only_in_session` (predicate). |
| AC3 — overlay appears on transport drop during gameplay | PASS | `handle_transport_disconnected_event` sets `state.visible = true` when `ClientState::InSession`. `sync_connection_lost_overlay_visibility_system` mirrors state to `Visibility::Visible` next frame. Tests `ac3_disconnect_handler_marks_overlay_visible_in_session`, `ac3_disconnect_handler_does_not_mark_overlay_visible_in_lobby`, `ac3_ac4_sync_system_mirrors_state_to_root_visibility`. |
| AC4 — overlay dismisses on reconnect completion | PASS | `handle_transport_connected_event` sets `state.visible = false` on any `Connected` Add. Per ADR-011 the reconnect path does not exit `InSession`, so the lightyear `Connected` re-link fires on the existing client entity. Tests `ac4_connected_handler_marks_overlay_hidden`, `ac4_connected_handler_is_idempotent_when_hidden`, `ac3_ac4_sync_system_mirrors_state_to_root_visibility`. |
| AC5 — overlay dismisses on GAME_OVER | PASS | `dismiss_overlay_on_game_over_system` reads `Res<CurrentClientPhase>` and clears `state.visible` if `phase == GameOver`. `CurrentClientPhase` is set by `phase_sink_system` from `S2CPhaseChanged{phase: GameOver}` which the server emits together with `S2CGameOver`. Tests `ac5_predicate_overlay_dismissed_by_phase_only_at_game_over`, `ac5_dismiss_system_hides_overlay_when_phase_is_game_over`, `ac5_dismiss_system_is_noop_during_active_gameplay`. |
| AC6 — integration test asserts visibility transitions | PASS | `tests/integration/playable_client/connection_lost_overlay_test.rs` (16 tests, 100% pass). Test `ac3_ac4_sync_system_mirrors_state_to_root_visibility` exercises the full visibility-transition pipeline (hidden → visible → hidden) via the production plugin. |
| AC7 — overlay does not block underneath UI | PASS | `BackgroundColor(Color::srgba(0.02, 0.025, 0.035, 0.32))` — alpha 0.32, less than result screen's 0.46. `GlobalZIndex(90)` — above gameplay UI (z = 0) but below result screen (z = 100). Tests `ac7_overlay_z_index_is_below_result_screen` + `ac7_overlay_backdrop_alpha_lets_gameplay_show_through`. |
| AC8 — closes backlog row `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` | EVIDENCE-CITED; ROW-CLOSURE-DEFERRED | This document cites the closure rationale (see "Backlog-Row Closure Cross-Reference" below). The row's status flip in `production/sprint-status.yaml` is **out of scope for this implementation prompt** per the story's AC8 phrasing ("via a separate paperwork prompt (NOT this implementation)") and per the PROMPT 889 forbidden-scope list ("production/sprint-status.yaml"). |
| AC9 — no optimistic client-side authority introduced | PASS | `text search for "no optimistic"` in this document → "No optimistic client-side authority is introduced or proposed by this overlay" (verbatim above) + the module header in `connection_lost_overlay.rs` ("No optimistic client-side authority is introduced by this overlay"). Test `ac9_overlay_module_does_not_mutate_authoritative_state` asserts the module does not use `MessageSender` / `MessageWriter` / `NextState` and that the no-claim phrase is present. ADR-002 binding. |
| AC10 — no protocol or server-side change | PASS | `git diff` against `origin/main@12ae4cf` for `shared/src/protocol.rs` and `server/` will be empty (verified at the verification step below). Test `ac10_no_protocol_or_server_changes_in_story_scope` asserts the overlay module does not import the `server` crate or `crate::network` internals. |
| AC11 — Sprint 12 disposition preserved | PASS | `production/sprint-status.yaml`, `production/sprints/sprint-12.md`, `production/stage.txt`, and `production/qa/qa-plan-sprint-12.md` are NOT modified by this implementation. (Sprint 12 is already closed and Sprint 13 is the active sprint at `origin/main@12ae4cf`; the spirit of AC11 is preserved.) |
| AC12 — workspace test pass + ignored count behaves predictably | PARTIAL — TARGETED-PASS | Per PROMPT 889 explicit policy ("Do not run full workspace tests by default") and the Sprint 13 QA-plan binding no-full-workspace-tests-by-default convention, **full workspace `cargo test --workspace --tests --no-fail-fast` was deferred**. Targeted evidence: `cargo test -p client --test connection_lost_overlay_test` (16/16 pass) + `cargo check -p client` (clean). No new `#[ignore]` markers introduced. Full-workspace cargo test deferred to Sprint 13 end-of-sprint integration smoke. |
| AC13 — evidence document slot reserved | PASS | This document (`production/qa/evidence/sprint-13-conn-lost-ux-evidence.md`). |

---

## Backlog-Row Closure Cross-Reference

`S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` is the backlog row that
introduced the gap closed by this overlay. PROMPT 803
(`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`) §3 DC-13
and §5 Should row 2 link the story to this row.

The row's status flip in the backlog tracker (wherever tracked, including
`production/sprint-status.yaml`) is **deferred to a separate paperwork
prompt** per AC8 and per PROMPT 889 forbidden-scope.

Backlog-row evidence for the closure-with-evidence rationale:

- The overlay's `on_transport_disconnected` observer fires on the lightyear
  `On<Add, Disconnected>` trigger — the same transport-event surface the
  server's `on_lightyear_disconnected` uses
  (`server/src/core/rsm/transitions.rs:234,235`).
- The overlay's `on_transport_connected` observer fires on the lightyear
  `On<Add, Connected>` trigger — paired with the server's
  `on_lightyear_connected` (`server/src/core/rsm/transitions.rs:210,211`).
- The overlay's dismissal trigger on `RoundPhase::GameOver` is driven by the
  server-authoritative `CurrentClientPhase` resource (set by
  `phase_sink_system` from `S2CPhaseChanged`).

The observability surface this row asked for is now present on the client
side as a user-visible UX modal during the gameplay window.

---

## Cross-Link to PROMPT 803

- **§3 DC-13** Client-visible "server gone" UI absent (MED-HIGH): closed by
  this overlay. The previous-only surface (`result_screen.rs:324-334`
  post-`GAME_OVER` disconnect-reason copy) now has a proactive companion
  during the gameplay window.
- **§4 Lane E DC-13**: same.
- **§5 Should row 2 (S13-CONN-LOST-UX-001)**: implementation landed.

---

## Verification Steps Run (PROMPT 889)

| Check | Result |
|-------|--------|
| `cargo fmt -p client -- --check` | PASS (see verification report) |
| `cargo check -p client` | PASS (28.80s incremental, no warnings on overlay module) |
| `cargo test -p client --test connection_lost_overlay_test` | PASS — 16/16 |
| `git diff --check origin/main...HEAD` | PASS |
| `git diff --cached --check` | PASS |
| `git status --short` | shows only the five files in the change set above |
| Forbidden-paths diff vs `origin/main@12ae4cf` | empty for `shared/src/protocol.rs`, `server/`, `production/sprint-status.yaml`, `production/sprints/sprint-12.md`, `production/stage.txt`, `production/qa/qa-plan-sprint-12.md`, `production/session-state/*` |

---

## Carried Non-Claims

- `S8-QA-001-W1` — still OPEN. PROMPT 889 does NOT close it.
- `QA-COND-0005` + `QA-COND-0006` — accepted-risk; unchanged.
- `PAW-TD-*-a` — accept-risk; friend-game-tier placeholder visual is the
  documented stance for this overlay.
- PROMPT 761 Polish->Release FAIL — preserved.
- Sprint 13 `active` disposition — unchanged.
- `production/stage.txt` `Polish` — unchanged.
- Story 019 underlying drag-runtime bug — NOT claimed fixed.
- `TQ-S12-C1..C7` — verbatim, preserved.
- Sprint 12 / Sprint 11 / Sprint 10 closeouts — preserved.
- All prior Sprint 13 `/story-done` closures (PROMPT 833 / 835 inline /
  840 / 843 / 844 / 850 / 851 / 854 / 856 / 865 / 868 / 869 / 871 / 876 /
  884 / 885 / 888) — unchanged on `origin/main`.

---

## Cargo Resource Policy

Applied (binding Windows/MSVC Cargo resource policy for Sprint 13):

```
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

No disk cleanup was required during this implementation.

---

## Implementation Notes for Downstream Agents

- The overlay is **always present** in the scene graph after `Startup` and is
  toggled via the `Visibility` component on the root. It does not despawn /
  respawn between sessions — a single instance is fine because the modal has
  no per-session state.
- The two observers and the dismiss system are independent: the observers
  set `state.visible`, the dismiss system clears it on `GameOver`, and the
  sync system mirrors state to the `Visibility` component. This is a
  one-direction data flow (observers → state → Visibility) with no shared
  mutation hazard.
- For tests that need to drive transport events without constructing
  lightyear's internal marker components, use the public pure handlers
  `handle_transport_disconnected_event` and `handle_transport_connected_event`
  (the integration test uses this surface).
- WASM/browser transport-drop signals: the implementing prompt did not split
  WASM-specific behaviour from native. The `On<Add, Connected>` /
  `On<Add, Disconnected>` triggers are provided by lightyear 0.26 across both
  targets; the overlay's behaviour is symmetric. If a future WASM-specific
  divergence is identified, document a follow-on under
  `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`'s follow-up scope.
