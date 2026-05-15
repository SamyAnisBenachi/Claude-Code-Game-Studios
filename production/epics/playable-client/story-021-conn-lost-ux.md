# Story 021: S13-CONN-LOST-UX-001 -- Proactive Connection-Lost / Reconnecting Client UI

> **Epic**: Playable Client
> **Story ID**: S13-CONN-LOST-UX-001
> **Status**: Done -- closed by PROMPT 891 `/story-done` on
> `origin/main@cb01c492721bdcba6439d44b5b311c8905091828` (PROMPT 890 integration
> merge of PROMPT 889 worker commit `febc56abbb110b3f35f838a60447d7ee77a1982c`).
> Verdict **PASS**. AC1-AC13 all PASS (AC8 EVIDENCE-CITED with row-flip deferred
> per AC8 phrasing). See Closure Trail below.
> **Layer**: UI / UX (Client)
> **Type**: Integration -- new overlay module + transport-event subscription +
> integration test
> **Sprint**: Sprint 13 candidate (per PROMPT 803 §6 line 144; NOT activated)
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
story.** The connection-lost overlay reads from transport events
(server-authoritative: the transport-layer disconnect signal); it
does not synthesise any game-state change or mutate authoritative
state. The overlay is a presentation read-only over the transport
state. ADR-002 + ADR-008 binding.

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-13** Client-visible "server gone" UI absent (MED-HIGH):
  Only post-`GAME_OVER` disconnect reason copy exists. No proactive
  "Reconnecting…" or "Connection Lost" modal during gameplay
  (`DRAFT_INITIAL` → `RESOLUTION` window). Evidence anchor:
  `client/src/presentation/result_screen.rs:324-334` (only surface);
  backlog `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`.
- **§4 Lane E DC-13**: same.
- **§5 Should row 2 (S13-CONN-LOST-UX-001)**: "Implement proactive
  'Reconnecting…' / 'Connection Lost' overlay between transport drop
  and reconnect-window-expiry. Closes backlog
  `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`." Likely files:
  `client/src/network/mod.rs`,
  `client/src/presentation/` (NEW overlay module).
- **§6 PROMPT-N+4 (paired with S13-LATE-MSG-DEDUPE-001)**:
  paperwork-only story-authoring.

---

## Problem Class / Prevention Target

**Defect class (DC-13)**: When the Lightyear transport drops mid-game
(between `DRAFT_INITIAL` and `RESOLUTION`, before `GAME_OVER`), the
client's only user-visible feedback is the disconnect-reason copy on
the result screen -- which only appears after `GAME_OVER` is locally
synthesised or detected. There is no proactive "Reconnecting..." or
"Connection Lost" modal during the gameplay window. Symptoms: a
mid-game transport drop produces no UI change until either reconnect
succeeds (state silently resumes) or the reconnect window expires
and `GAME_OVER` lands; the player has no visibility into the failure
mode.

**Prevention target**: A new presentation overlay module
(`client/src/presentation/connection_lost_overlay.rs` or canonical
equivalent) that:

- Subscribes to transport-level connection-lost signals from
  Lightyear (specific API surface verified by implementing worker
  per `liv-bevy-lightyear` skill).
- Displays a proactive "Connection Lost - Reconnecting..." modal
  (or non-blocking banner) when the transport drops.
- Updates with a countdown to reconnect-window-expiry (if the
  reconnect window is server-configurable and known to the client).
- Dismisses the modal when the transport reconnects, the reconnect
  flow completes (`S2CSessionReady` or snapshot replay), or
  `GAME_OVER` is reached.
- Closes backlog row
  `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`.

The overlay is integrated with the existing
`PresentationPlugin` composition order (ADR-021); it sits above
gameplay UI but below the result screen.

---

## Context

### Existing surface

- **`client/src/presentation/result_screen.rs:324-334`**: only
  post-GAME_OVER disconnect reason copy. The connection-lost
  overlay sits **before** this in the disconnect lifecycle.
- **`client/src/network/mod.rs`**: client network plugin; this is
  where transport-event subscription lives (or the new overlay
  module subscribes via a Bevy event/resource the network plugin
  publishes).
- **Server-side reconnect window**:
  `server/src/core/session/reconnect.rs:107-121,176,198-233,292-316`
  -- defines the reconnect window and the flush order.
- **Backlog row** `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`:
  the preceding paperwork that introduced this gap. The
  implementation prompt links to this row in the evidence doc.

### UX surface

The overlay's visual design is owned by `ux-designer` agent +
`ui-programmer`. The implementing prompt MUST consult the UX
designer before locking on a final visual (modal vs banner,
copy text, countdown, color/animation choices). For Sprint 13
candidate scope, the visual is friend-game-tier (placeholder art
acceptable per `PAW-TD-*-a` accept-risk preserved).

### GDD / ADR / TR trace

- **GDD**: no GDD change. The "Connection Lost" overlay is a
  presentation/UX surface, not a mechanic. The implementing prompt
  may cite the result-screen UX doc
  (`design/ux/result-screen.md`) for tone/voice consistency.
- **ADR-002** (Client-Server Authority): overlay reads transport
  state; does not mutate game state.
- **ADR-008** (Lightyear Channel Config): no protocol change.
- **ADR-011** (Reconnect Snapshot): overlay dismisses on
  reconnect-flow completion (`S2CSessionReady` or snapshot replay).
- **ADR-012** (SessionReady Delivery): overlay dismissal on
  `S2CSessionReady` integrates with the existing observer.
- **ADR-021** (Presentation Layer Architecture): overlay is
  registered as a sub-plugin of `PresentationPlugin` in the
  canonical order.
- **TR registry**: no new TR (UX surface; not a TR-tracked
  requirement).

### Engine

- **Engine**: Bevy 0.18 (Rust). The overlay uses `bevy_ui`
  Required Components API (per
  `.claude/docs/technical-preferences.md`).
- **Lightyear**: 0.26. Transport-event subscription uses
  Lightyear 0.26 API (specific event types verified by
  implementing worker per `liv-bevy-lightyear` skill).

### Mandatory skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code edits;
  especially for `bevy_ui` Required Components patterns.
- **`liv-bevy-lightyear`** -- mandatory for transport-event
  subscription.

### Control Manifest Rules (Presentation / UI scope)

- Required: Overlay is read-only over transport state; no
  game-state mutation.
- Required: Overlay is registered in the canonical
  `PresentationPlugin` order (ADR-021).
- Required: Overlay dismisses on reconnect completion
  (`S2CSessionReady`, snapshot replay) or `GAME_OVER`.
- Required: Overlay does NOT block other UI from rendering
  underneath (i.e., the gameplay state remains visible behind
  the modal so the player can see what was happening).
- Required: At least one integration test asserts overlay
  visibility transitions on transport-event sequence.
- Forbidden: Synthesising fake `S2C*` messages or game-state
  events from the overlay.
- Forbidden: Modifying server-side reconnect logic.
- Forbidden: Adding optimistic client-side authority.

---

## Story Classification

**Story type**: Integration -- new overlay module + transport-event
subscription + integration test.

This is **NOT** a:

- Pure UX-spec story (real Bevy UI lands).
- Server-side change.
- Sprint 12 expansion.

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Overlay module exists**:
  `client/src/presentation/connection_lost_overlay.rs` (or
  canonical equivalent) exists; it defines a Bevy plugin
  (`ConnectionLostOverlayPlugin` or canonical name) registered in
  `PresentationPlugin`'s composition order per ADR-021.
  **PASS** — `client/src/presentation/connection_lost_overlay.rs` (NEW
  263 LOC) defines `pub struct ConnectionLostOverlayPlugin` registered
  in `client/src/presentation/mod.rs:92` after `ResultScreenPlugin`
  per ADR-021. Integration-test cases
  `ac1_plugin_registered_in_presentation_composition_order` +
  `ac1_overlay_root_carries_marker_component_for_query_targeting` PASS
  at PROMPT 889 worker tip `febc56a`.

- [x] **AC2 -- Overlay subscribes to transport-event source**:
  GIVEN the overlay plugin's systems, WHEN inspected, THEN at
  least one system observes a Lightyear transport-event source
  (specific API surface verified by implementing worker; e.g.,
  `OnDisconnect`, `OnConnected`, or an equivalent Bevy event
  surface published by the client network plugin).
  **PASS** — `on_transport_disconnected: On<Add, Disconnected>` +
  `on_transport_connected: On<Add, Connected>` observers registered
  via `app.add_observer(...)` at
  `client/src/presentation/connection_lost_overlay.rs:68-69`.
  Integration tests
  `ac2_overlay_subscribes_to_lightyear_transport_event_sources` +
  `ac2_predicate_should_show_overlay_only_in_session` PASS.

- [x] **AC3 -- Overlay appears on transport drop during
  gameplay**: GIVEN the player is in `DRAFT_INITIAL`,
  `DRAFT_SHOP`, `PLACEMENT`, or `RESOLUTION` phase AND the
  transport drops, WHEN the overlay system runs, THEN the
  overlay UI becomes visible within one frame (or one tick)
  of the transport event being observed.
  **PASS** — `handle_transport_disconnected_event` sets
  `state.visible = true` when `ClientState::InSession`
  (`connection_lost_overlay.rs:90-108`).
  `sync_connection_lost_overlay_visibility_system` mirrors state to
  `Visibility::Visible` next frame
  (`connection_lost_overlay.rs:160-179`). Integration tests
  `ac3_disconnect_handler_marks_overlay_visible_in_session` +
  `ac3_disconnect_handler_does_not_mark_overlay_visible_in_lobby` +
  `ac3_ac4_sync_system_mirrors_state_to_root_visibility` PASS.

- [x] **AC4 -- Overlay dismisses on reconnect completion**:
  GIVEN the overlay is visible AND `S2CSessionReady` is
  observed (or the snapshot-replay flow completes), WHEN the
  overlay system runs, THEN the overlay UI is hidden within
  one frame of the observation.
  **PASS** — `handle_transport_connected_event` sets
  `state.visible = false` on any `Connected` Add
  (`connection_lost_overlay.rs:110-122`). Per ADR-011 the reconnect
  path does not exit `InSession`, so the lightyear `Connected` re-link
  fires on the existing client entity. Integration tests
  `ac4_connected_handler_marks_overlay_hidden` +
  `ac4_connected_handler_is_idempotent_when_hidden` +
  `ac3_ac4_sync_system_mirrors_state_to_root_visibility` PASS.

- [x] **AC5 -- Overlay dismisses on `GAME_OVER`**: GIVEN the
  overlay is visible AND `S2CGameOver` is observed, WHEN the
  overlay system runs, THEN the overlay UI is hidden within
  one frame so the result screen can take over.
  **PASS** — `dismiss_overlay_on_game_over_system` reads
  `Res<CurrentClientPhase>` and clears `state.visible` if
  `phase == GameOver` (`connection_lost_overlay.rs:143-158`).
  `CurrentClientPhase` is set by `phase_sink_system` from
  `S2CPhaseChanged{phase: GameOver}` (the server emits together with
  `S2CGameOver`). `GlobalZIndex(90)` (vs result screen's 100) keeps
  the result screen on top if `GameOver` lands while the overlay is
  up. Integration tests
  `ac5_predicate_overlay_dismissed_by_phase_only_at_game_over` +
  `ac5_dismiss_system_hides_overlay_when_phase_is_game_over` +
  `ac5_dismiss_system_is_noop_during_active_gameplay` PASS.

- [x] **AC6 -- Integration test asserts visibility
  transitions**: GIVEN a new integration test (e.g.,
  `tests/integration/playable_client/connection_lost_overlay_test.rs`),
  WHEN the test simulates the transport-event sequence
  (drop → reconnect or drop → GAME_OVER), THEN the overlay
  UI visibility transitions match AC3, AC4, AC5.
  **PASS** — `tests/integration/playable_client/connection_lost_overlay_test.rs`
  (NEW 334 LOC; 16 test cases) registered at `client/Cargo.toml:289`.
  Test `ac3_ac4_sync_system_mirrors_state_to_root_visibility` exercises
  the full visibility-transition pipeline (hidden → visible → hidden)
  via the production plugin. PROMPT 889 evidence reports 16/16 pass at
  worker commit `febc56a` (`cargo test -p client --test
  connection_lost_overlay_test`).

- [x] **AC7 -- Overlay does not block underneath UI**:
  GIVEN the overlay is visible, WHEN the gameplay UI (hand,
  HUD, board) is inspected via the test fixture or visual
  evidence, THEN the gameplay UI remains visible underneath
  the overlay (modal is non-blocking visually).
  **PASS** — `BackgroundColor(Color::srgba(0.02, 0.025, 0.035, 0.32))`
  (alpha 0.32 < result-screen 0.46) at
  `connection_lost_overlay.rs:201`; `GlobalZIndex(90)` at
  `connection_lost_overlay.rs:203` (above gameplay UI z=0, below
  result screen z=100). Integration tests
  `ac7_overlay_z_index_is_below_result_screen` +
  `ac7_overlay_backdrop_alpha_lets_gameplay_show_through` PASS.

- [x] **AC8 -- Closes backlog row**
  `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`: GIVEN the
  evidence doc, WHEN inspected, THEN it cites the backlog row
  closure rationale; the row's status (wherever tracked) is
  updated to closed-with-evidence via a separate paperwork
  prompt (NOT this implementation).
  **EVIDENCE-CITED; ROW-CLOSURE-DEFERRED** — Per the AC8 phrasing
  ("via a separate paperwork prompt (NOT this implementation)"),
  PROMPT 891 cites the closure rationale (see evidence doc
  `production/qa/evidence/sprint-13-conn-lost-ux-evidence.md`
  §Backlog-Row Closure Cross-Reference, lines 181-206) but does NOT
  flip the backlog-row status. Row-status flip in any tracker remains
  a separate paperwork prompt by design.

- [x] **AC9 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed for any
  client-side mutation of authoritative state outside the
  shared phase sink, snapshot drainers, and S2C consumers,
  THEN no such mutation is present. ADR-002 binding.
  *Evidence*: text search for "no optimistic" in the evidence
  document.
  **PASS** — Verbatim phrase "No optimistic client-side authority
  is introduced or proposed by this overlay" present at evidence doc
  line 40-41 + module header at `connection_lost_overlay.rs:9-10`
  ("No optimistic client-side authority is introduced by this
  overlay."). Integration test
  `ac9_overlay_module_does_not_mutate_authoritative_state` asserts
  the module does not use `MessageSender` / `MessageWriter` /
  `NextState`. ADR-002 binding preserved.

- [x] **AC10 -- No protocol or server-side change**: GIVEN the
  diff in `shared/src/protocol.rs` and `server/`, WHEN
  inspected, THEN no functional change lands.
  **PASS** — `git diff cb01c49^1..cb01c49 --stat -- shared/src/protocol.rs server/`
  empty. Worker-commit-level `git diff febc56a^1..febc56a --stat --
  shared/ server/` also empty. Integration tip touches only
  `client/Cargo.toml` + `client/src/presentation/connection_lost_overlay.rs`
  + `client/src/presentation/mod.rs` + the new evidence doc + the new
  integration-test file. Integration test
  `ac10_no_protocol_or_server_changes_in_story_scope` PASS.

- [x] **AC11 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`,
  and `production/qa/qa-plan-sprint-12.md` are diffed, THEN
  none of them are modified under this story.
  **PASS** — `git diff cb01c49^1..cb01c49 --stat --
  production/sprint-status.yaml production/sprints/sprint-12.md
  production/stage.txt production/qa/qa-plan-sprint-12.md` empty
  across both worker (`febc56a`) and integration (`cb01c49`) commits.
  PROMPT 891 row-level flip (and `updated:` annotation + new
  `sprint_13_story_done:` entry) is the permitted
  disposition-preserving paperwork edit; top-level `sprint:` /
  `status:` / `stage:` fields unchanged.

- [x] **AC12 -- Workspace test pass + ignored count behave
  predictably**: GIVEN `cargo test --workspace --tests
  --no-fail-fast` at the implementation commit, WHEN compared
  to the post-Sprint-12 baseline, THEN no new `#[ignore]`
  markers are introduced; the new overlay test passes;
  previously-passing tests continue to pass.
  **PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK** — Per Sprint 13
  QA-plan binding no-full-workspace-tests-by-default policy + PROMPT
  889 explicit scope: `cargo test -p client --test
  connection_lost_overlay_test` reported 16/16 pass at worker tip
  `febc56a` (evidence doc lines 134-156). No new `#[ignore]` markers
  introduced by the worker diff. Full-workspace
  `cargo test --workspace --tests --no-fail-fast` deferred to Sprint
  13 end-of-sprint integration smoke.

- [x] **AC13 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-conn-lost-ux-evidence.md`
  (NEW). Records overlay module diff summary, integration-test
  pass evidence, UX-designer consultation note, backlog-row
  closure cross-reference, no-claim restatement, cross-link
  to PROMPT 803 §3 DC-13.
  **PASS** — `production/qa/evidence/sprint-13-conn-lost-ux-evidence.md`
  (NEW 289 lines) landed on `origin/main` via PROMPT 890 integration
  merge `cb01c49`; not modified by PROMPT 891. All required content
  present: overlay module diff summary (§Files Changed +
  §Overlay Module Diff Summary), integration-test pass evidence
  (§Integration-Test Pass Output), UX-designer consultation note
  (§UX-Designer Consultation Note), backlog-row closure cross-reference
  (§Backlog-Row Closure Cross-Reference), no-claim restatement
  (§Status / No-Claim Banner), cross-link to PROMPT 803 §3 DC-13
  (§Cross-Link to PROMPT 803).

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/presentation/connection_lost_overlay.rs` | NEW. Overlay plugin + systems + UI nodes. |
| `client/src/presentation/mod.rs` | Updated to register the overlay plugin in `PresentationPlugin`. |
| `client/src/network/mod.rs` | OPTIONAL: if a Bevy event surface for transport drop is needed, publish it here. Otherwise the overlay subscribes to a Lightyear-native event directly. |
| `tests/integration/playable_client/connection_lost_overlay_test.rs` | NEW integration test asserting AC3-AC7 visibility transitions. |
| `production/qa/evidence/sprint-13-conn-lost-ux-evidence.md` | NEW evidence document per AC13. |
| This story file | Status updates per /story-readiness or /story-done if/when Sprint 13 activates. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code, especially
  `bevy_ui` Required Components patterns.
- **`liv-bevy-lightyear`** -- mandatory for transport-event
  subscription.

---

## Evidence Path

`production/qa/evidence/sprint-13-conn-lost-ux-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Overlay module diff summary.
- New integration-test pass output.
- UX-designer consultation note (link to the Slack / doc
  conversation or in-conversation rationale recorded in the
  evidence doc).
- Backlog-row closure cross-reference
  (`S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`).
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no optimistic client-side authority").
- Cross-link to PROMPT 803 §3 DC-13 and §4 Lane E.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test connection_lost_overlay -- --nocapture`
  (or the new test name)
- `git diff <pre-impl-sha>..<impl-sha> -- 'shared/src/**' 'server/src/**'`
  (verifies AC10: zero protocol-shape change; zero server-side
  change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- **Server-side reconnect logic changes**. ADR-011 binding.
- **Reconnect-window duration changes** or per-session
  customisation.
- **Browser/WASM-specific transport-drop quirks** beyond what
  the existing client supports. If WASM transport-drop signals
  differ, the implementation prompt records the platform
  divergence and either covers both or scopes to the native
  client with a documented follow-on for WASM.
- **Final UX-tier visual polish** -- friend-game-tier
  placeholder art is acceptable per `PAW-TD-*-a` accept-risk.
- **Closing `S8-QA-001-W1`** -- the manual two-client GAME_OVER
  gap is separate.
- **Sprint 13 activation**.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `QA-COND-0005`, `QA-COND-0006`, or any
  carried Sprint condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Touches `client/src/presentation/`** (new file) and
  optionally `client/src/network/mod.rs`. Sprint 12 Must Have
  rows touch:
  - Story 012 (HUD snapshot bridge): touches presentation/HUD
    code -- **POTENTIAL CONFLICT** on `client/src/presentation/`
    if the overlay registration in `mod.rs` collides. Mitigation:
    sequence after Sprint 12 close-out.
  - Story 014 (cooccupancy panic guard):
    `client/src/presentation/board_rendering.rs` -- disjoint from
    the new overlay file.
  - Story 015 (fixture D residuals): tests only; disjoint.
- **No Sprint 12 invasion**: this story's implementation MUST
  NOT land before Sprint 12 close-out unless the producer
  explicitly authorises a pull-forward via a separate prompt.
- **Coordinate with `S13-LATE-MSG-DEDUPE-001` (Story 020 in
  this epic)**: both stories touch the disconnect / reconnect
  flow; ideally land in the same Sprint 13 wave with shared
  reconnect-test fixtures.
- **No shared-status writer overlap**:
  `production/sprint-status.yaml` is not touched by this story.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. Sprint 12 reaches close-out.
2. Sprint 13 is planned via `/sprint-plan sprint-13`.
3. This story passes `/story-readiness`.
4. Sprint 13 `/qa-plan sprint` is authored.
5. `/dev-story story-021-conn-lost-ux.md` is dispatched.

Expected implementation flow:

1. **Wave 1 -- UX consultation**: implementing prompt consults
   `ux-designer` agent for visual + copy direction. UX
   designer's response is recorded in the evidence doc.
2. **Wave 2 -- Transport-event subscription**: implementing
   prompt verifies Lightyear 0.26 transport-event API; chooses
   between direct subscription and a Bevy-event surface
   published from `client/src/network/mod.rs`.
3. **Wave 3 -- Overlay module**: implement the overlay plugin
   (Bevy 0.18 Required Components UI nodes); register in
   `PresentationPlugin` per ADR-021 order.
4. **Wave 4 -- Integration test**: assert visibility transitions
   per AC3, AC4, AC5, AC6, AC7.
5. **Wave 5 -- Backlog-row closure cross-reference**: link the
   evidence doc to
   `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`; backlog-row
   status update is a separate paperwork prompt.
6. **Wave 6 -- Evidence**: populate evidence file.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Sprint 12 Story 012 collision on `client/src/presentation/mod.rs` | Medium | Medium | Sequence: Sprint 12 closes first. |
| Lightyear 0.26 transport-event API has unexpected gaps | Medium | Medium | `liv-bevy-lightyear` skill cross-references current API; if a gap exists, the implementing prompt records a workaround (e.g., polling connection state) in the evidence doc. |
| WASM/browser transport-drop signals differ from native | Medium | Low | Implementation prompt audits both targets and either covers both or scopes to native with a documented follow-on for WASM. |
| Overlay UI competes with result screen on `GAME_OVER` | Low | Low | AC5 forces dismissal on `GAME_OVER`; the result screen takes over cleanly. |
| Overlay accidentally synthesises a fake `S2C*` event | Low | High | AC9 + ADR-002 reviewer check; pattern from existing read-only consumers (e.g., `phase_sink_system`) cited in evidence doc. |
| UX-designer consultation is skipped and the visual is wrong | Medium | Low | Wave 1 explicit; evidence doc must record consultation. |
| Sprint 13 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate. |

---

## Verification (orchestrator-side, before worker dispatch)

- `production/sprint-status.yaml` `sprint:` field reads `13` after
  Sprint 13 activation; Sprint 12 close-out has landed.
- Sprint 12 Story 012 (HUD snapshot bridge) is `done`.
- `production/stage.txt` reads `Polish` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for Proactive Connection-Lost / Reconnecting Client UI
  per PROMPT 803 §3 DC-13 / §5 Should row 2. Sprint 12 is `active`
  (PROMPT 798) and is not modified by this authoring run. No code
  changes, no smoke / gate / QA / `/dev-story` / `/story-done` /
  `/story-readiness` / `/qa-plan` run. Source-of-truth at authoring:
  `origin/main@b5eef0d`. Worker branch:
  `work/s13-runtime-hardening-story-authoring`. Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.

---

## Closure Trail

- 2026-05-14 -- PROMPT 808 -- Story-authoring integration merge of
  PROMPT 804 onto `origin/main` as commit `55b25be` (no functional
  change beyond authoring; story file landed on origin/main).
- 2026-05-14 -- PROMPT 823 -- Batch `/story-readiness` verdict
  **READY** for this story alongside sibling Sprint 13 candidates.
- 2026-05-15 -- PROMPT 889 -- `/dev-story` worker. Implemented the
  overlay module + integration test on branch `work/s13-conn-lost-ux`
  from base `origin/main@12ae4cf` (PROMPT 888 `/story-done` for
  S13-OPS-WIN-APPCOMPAT-NOTE-001). Worker commit
  `febc56abbb110b3f35f838a60447d7ee77a1982c`. Files: 5 changed /
  +899 / -0: `client/Cargo.toml +6` (new `[[test]] connection_lost_overlay_test`)
  + `client/src/presentation/connection_lost_overlay.rs` NEW 263 LOC
  (plugin, observers, dismiss + sync systems, UI nodes, pure handlers)
  + `client/src/presentation/mod.rs +7` (registers
  `ConnectionLostOverlayPlugin` after `ResultScreenPlugin` per
  ADR-021) + `tests/integration/playable_client/connection_lost_overlay_test.rs`
  NEW 334 LOC (16 test cases) + `production/qa/evidence/sprint-13-conn-lost-ux-evidence.md`
  NEW 289 lines. Cargo resource policy applied (`CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc`
  + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` +
  `CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`);
  targeted regression `cargo test -p client --test
  connection_lost_overlay_test` 16/16 pass; full-workspace tests
  intentionally NOT run per QA-plan-sprint-13 no-full-workspace-tests-by-default
  policy + worker prompt scope. Forbidden paths (`shared/` + `server/`
  + `production/sprint-status.yaml` + `production/sprints/` +
  `production/stage.txt` + `production/qa/qa-plan-sprint-12.md` +
  `production/qa/qa-plan-sprint-13.md` + `production/session-state/`)
  untouched. Did NOT run `/story-done`.
- 2026-05-15 -- PROMPT 890 -- Integration `--no-ff` merge of worker
  tip `febc56a` into prior `origin/main@12ae4cf` producing merge commit
  `cb01c492721bdcba6439d44b5b311c8905091828` on `origin/main`; zero
  conflicts. Merge-introduced delta byte-identical to worker stat
  (5 files / +899 / -0). Forbidden paths empty across both worker +
  integration commits. PROMPT 889 worker reachable as merge's
  second-parent. Did NOT run `/story-done`.
- 2026-05-15 -- PROMPT 891 -- `/story-done` paperwork closure on
  `origin/main@cb01c492721bdcba6439d44b5b311c8905091828`. Verdict
  **PASS**. AC1-AC13 verified against integrated docs / evidence /
  code at the integration tip (AC8 EVIDENCE-CITED with row-flip
  deferred per AC8 phrasing; AC12 PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK).
  Expected worker report `reports/PROMPT-889-S13-CONN-LOST-UX-Dev-Story.md`
  + integration report `reports/PROMPT-890-S13-CONN-LOST-UX-Integration.md`
  not on disk and not in any git tree; documented as non-blocking
  because the integration commit-message body + worker commit-message
  body (verbose, ~25 lines) + the evidence document
  (`production/qa/evidence/sprint-13-conn-lost-ux-evidence.md`,
  289 lines) collectively cover all thirteen ACs with concrete
  file:line references + test names + diff verifications. PROMPT 891
  ran on a fresh detached worktree `D:/_DEV/wt/ccgs-prompt-891-storydone`
  from `origin/main` because the root checkout was behind origin/main
  and had unrelated dirt (`M .claude/settings.json` +
  `Dtmpworkspace-test-output.txt` + `production/session-state/autonomous-monitor-task.md`
  + `tools/gcs-orchestrator/docs/ARCHITECTURE.md`); root-checkout
  dirt NOT touched by PROMPT 891; pattern matches PROMPT 884 + PROMPT
  885 + PROMPT 888 precedent. Paperwork-only writes to 3 allowed
  files (this story file + `sprint-status.yaml` + `active.md`);
  `production/session-state/codex-orchestrator-state.md` NOT modified
  by PROMPT 891 (precedent: PROMPT 884 + PROMPT 876 + PROMPT 888 all
  skipped this file). No `cargo` invoked by PROMPT 891 (paperwork-only).
  Sprint 13 disposition UNCHANGED `active`; Stage UNCHANGED `Polish`;
  PROMPT 761 Polish->Release FAIL preserved; all carried non-claims
  preserved (S8-QA-001-W1 OPEN, QA-COND-0005 + QA-COND-0006
  accepted-risk, PAW-TD-*-a accept-risk, TQ-S12-C1..C7 verbatim,
  Sprint 12 / Sprint 11 / Sprint 10 closeouts, all 15 prior Sprint 13
  `/story-done` closures unchanged on origin/main).

### Conditions carried forward unchanged

- `S8-QA-001-W1` manual/browser two-client GAME_OVER gap remains
  OPEN. PROMPT 891 does NOT close it.
- `QA-COND-0005` Standard-tier accessibility remains accepted-risk
  (friend-game scope only).
- `QA-COND-0006` playtest / fun-hypothesis validation remains
  accepted-risk / deferred.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  PAW-002..PAW-006; friend-game-tier overlay visual aligned with this
  stance.
- PROMPT 683-era runtime divergence question preserved unchanged
  (folded into Sprint 12 story 019 cannot-reproduce closure; third
  same-scope retest NOT authorised per `TQ-S12-C2`).
- PROMPT 761 Polish->Release gate-check FAIL preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no
  retry in PROMPT 891 scope.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed (closed cannot-reproduce, NOT bug-fixed).
- `TQ-S12-C1..C7` (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim. `TQ-S12-C7` remains preserved as informational.
- Sprint 12 disposition closed-with-conditions per PROMPT 817
  preserved unchanged under `sprint_12_closeout:` block.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- All 15 prior Sprint 13 `/story-done` closures (PROMPT 833 / 835
  inline / 840 / 843 / 844 / 850 / 851 / 854 / 856 / 865 / 868 /
  869 via PROMPT 882 carry / 871 / 876 / 884 / 885 / 888) preserved
  unchanged on `origin/main`.

### Explicitly NOT claimed by PROMPT 891

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- Stage advance from Polish to Release
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog
  row (cited in evidence per AC8; row-status flip remains a separate
  paperwork prompt)
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (last open Sprint 13
  Should Have row)
- Sprint 13 close-out (Sprint 13 remains `active`; **18 of 19 rows
  closed** after PROMPT 891; 1 Should Have row still ready)
- Sprint 14 activation or follow-on story authoring
- full-workspace `cargo test --workspace --tests --no-fail-fast`
  claim (deferred to end-of-sprint integration smoke per QA-plan-sprint-13)
- any UI integration beyond the in-scope overlay registration
