# Story 008: PLACEMENT Timer Multiplier Authority

> **Epic**: Game Session System
> **Status**: Complete
> **Layer**: Core / Networking / Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**GDD Requirements**:

- `design/gdd/game-session-system.md` Rule 14 and ACs GSS-42, GSS-43, GSS-44: during LOBBY before `SessionReady`, each occupied player slot may request a multiplayer-safe PLACEMENT timer multiplier; allowed Standard-tier values are `1x`, `1.5x`, `2x`, and `3x`; the effective neutral room/session value is the highest request capped at `3x`; no request contributes `1x`; `0.5x` does not lower the multiplayer value; the effective value is frozen into `SessionConfig.placement_timer_multiplier_effective` when `SessionReady` fires.
- `design/gdd/network-protocol.md` NP-59 and NP-60: `C2SSetPlacementTimerMultiplier` is valid only in LOBBY before `SessionReady`; `S2CSessionSettingsUpdated` carries only the neutral effective multiplier with no requester attribution; `S2CPhaseChanged(PLACEMENT).timer_duration_ms` carries the RSM-computed effective duration; `S2CGameSnapshot` carries the frozen neutral multiplier.
- `design/gdd/round-state-machine.md` Rule 9, RSM-29c, and RSM-39: the RSM selects the standard or auction-followup PLACEMENT base timer first, then multiplies that base by the frozen `SessionConfig.placement_timer_multiplier_effective`; at `3x`, a standard 10 second PLACEMENT timer becomes `30000ms`.
- `design/gdd/hand-ui.md` PLACEMENT timer contract: Hand UI displays the server-owned PLACEMENT timer from phase/snapshot data and must not apply a client-local Settings multiplier.

**TR IDs**: TR-GSS-011, TR-NP-015, TR-RSM-011.

**ADR Governing Implementation**: ADR-023 (Placement Timer Accessibility Authority), ADR-002 (Client-Server Authority), ADR-009 (RSM Phase State), ADR-012 (SessionReady Delivery), ADR-021 (Presentation Layer Architecture).

**ADR Decision Summary**: ADR-023 makes the PLACEMENT timer multiplier a server-authoritative lobby/session setting. The GSS owns request collection and neutral effective value computation before `SessionReady`; the effective value is frozen into `SessionConfig`; the RSM applies the frozen value to PLACEMENT timer duration; clients display the server-provided duration without local multiplication or requester attribution.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH

**Engine Notes**: `liv-bevy-018` is mandatory for Bevy ECS code and `liv-bevy-lightyear` is mandatory for Lightyear protocol/network code. The implementation must preserve the ADR-012 insert-before-trigger ordering for `SessionConfig` and the existing Lightyear reliable-channel registration pattern for new C2S/S2C messages.

**Control Manifest Rules (2026-05-05)**:

- Required: GSS negotiates the PLACEMENT timer multiplier in LOBBY before `SessionReady`.
- Required: Effective value is the highest requested multiplayer-safe value across players, capped at `3x`, and frozen into `SessionConfig.placement_timer_multiplier_effective`.
- Required: RSM applies the frozen multiplier after selecting the base PLACEMENT duration.
- Required: `S2CPhaseChanged.timer_duration_ms` carries the effective duration; clients never recompute it from local Settings.
- Required: Display the multiplier as a neutral room/session setting.
- Forbidden: Never expose `0.5x` as a multiplayer Standard-tier PLACEMENT timer value.
- Forbidden: Never attribute the effective timer multiplier to a player.
- Forbidden: Never let client-local Settings alter the active multiplayer PLACEMENT timer after `SessionReady`.

---

## Scope

### In Scope

- Shared protocol enum/message coverage for `PlacementTimerMultiplier`, `C2SSetPlacementTimerMultiplier`, `S2CSessionSettingsUpdated`, and the frozen multiplier field in reconnect snapshot state.
- GSS lobby/session state for per-player multiplier requests and neutral effective multiplier computation.
- GSS handling for `C2SSetPlacementTimerMultiplier` only during LOBBY before `SessionReady`.
- `SessionConfig.placement_timer_multiplier_effective` freeze before `SessionReady` is triggered.
- RSM PLACEMENT timer calculation using the frozen multiplier after standard vs auction-followup base timer selection.
- `S2CPhaseChanged.timer_duration_ms` carrying the effective PLACEMENT duration.
- Client network/state handling for neutral session settings and phase timer duration.
- Hand UI placement timer initialization from server-provided phase/snapshot duration instead of the local default `PlacementTimerConfig::placement_duration_ms = 10000`.
- Automated server, protocol, RSM, reconnect, client state, and Hand UI tests for the ADR-023 validation criteria.

### Out of Scope

- Settings/Accessibility screen construction, preference persistence, and full UI settings navigation.
- Colorblind modes, reduced motion, UI scale, input remapping, Help/tutorial persistence, and other QA-COND-0005 accessibility rows.
- Solo, custom, or debug faster-than-default timer options.
- General HUD accessibility metric remediation such as text size, contrast, target size, and visual evidence capture.
- `/story-done`, sprint status, session-state, assets, AGENTS.md, and code implementation work in this docs-only pass.

---

## Acceptance Criteria

- [ ] Shared protocol defines `PlacementTimerMultiplier` with exactly the multiplayer Standard-tier values `X1`, `X1_5`, `X2`, and `X3`. No multiplayer `X0_5` or equivalent faster-than-default variant is exposed through `C2SSetPlacementTimerMultiplier`.
- [ ] Shared protocol defines and registers `C2SSetPlacementTimerMultiplier { multiplier: PlacementTimerMultiplier }` on the appropriate C2S reliable path, valid only during LOBBY before `SessionReady`.
- [ ] Shared protocol defines and registers `S2CSessionSettingsUpdated { placement_timer_multiplier_effective: PlacementTimerMultiplier }` on `ReliableChannel`. The payload contains no `PlayerId`, requester field, requester index, connection id, or other attribution field.
- [ ] GSS stores multiplayer timer requests by player during LOBBY and treats absent requests as `X1`.
- [ ] GIVEN Player A requests `X1_5` and Player B requests `X3` before `SessionReady`, WHEN GSS recomputes the room/session setting, THEN `placement_timer_multiplier_effective == X3` and a reliable `S2CSessionSettingsUpdated { placement_timer_multiplier_effective: X3 }` is sent without requester attribution.
- [ ] GIVEN no occupied player has requested a PLACEMENT timer multiplier before `SessionReady`, WHEN GSS builds `SessionConfig`, THEN `SessionConfig.placement_timer_multiplier_effective == X1`.
- [ ] GIVEN any player attempts to submit a multiplayer timer request after `SessionReady`, WHEN the handler runs, THEN the request is silently discarded and the active `SessionConfig.placement_timer_multiplier_effective` is unchanged.
- [ ] GIVEN a serialized or future invalid value below `X1` is received by the multiplayer request path, WHEN GSS validates it, THEN the effective multiplayer value remains at least `X1`; the request never shortens another player's PLACEMENT window.
- [ ] `build_session_config` writes `placement_timer_multiplier_effective` into `SessionConfig` before `commands.trigger(SessionReady)` or the ADR-012 exclusive-system fallback trigger path runs.
- [ ] `SessionConfig.placement_timer_multiplier_effective` is immutable for the active match after `SessionReady`. Later Settings changes can only affect a future lobby/session.
- [ ] RSM PLACEMENT entry selects its base duration first: standard PLACEMENT uses `placement_timer_seconds * 1000`; auction-followup PLACEMENT uses `auction_followup_placement_timer_seconds * 1000`.
- [ ] GIVEN `SessionConfig.placement_timer_multiplier_effective == X3` and standard PLACEMENT base is 10000ms, WHEN RSM enters PLACEMENT, THEN the authoritative timer starts at 30000ms and the broadcast `S2CPhaseChanged { phase: Placement, timer_duration_ms: 30000, ... }` is queued.
- [ ] GIVEN `SessionConfig.placement_timer_multiplier_effective == X1_5` and auction-followup PLACEMENT base is 12000ms, WHEN RSM enters PLACEMENT, THEN the authoritative timer starts at 18000ms and `S2CPhaseChanged.timer_duration_ms == 18000`.
- [ ] `S2CGameSnapshot` includes the frozen neutral `placement_timer_multiplier_effective` for reconnect/session recovery. Snapshot construction strips secrets as before and does not attribute who requested the setting.
- [ ] Client inbound state records the neutral effective multiplier from `S2CSessionSettingsUpdated` and snapshot data as room/session state only. It does not mark any player as the requester.
- [ ] Hand UI initializes and resets its PLACEMENT timer from the server-provided phase/snapshot timer duration for PLACEMENT. It does not reset to the local 10000ms default when `S2CPhaseChanged.timer_duration_ms` provides a different PLACEMENT duration.
- [ ] Existing Hand UI urgency, grace-window, and submitted-checkmark behavior remains proportional to the server-provided duration and continues to satisfy Story 009 tests.
- [ ] `cargo test -p server --test placement_timer_multiplier_test` passes.
- [ ] `cargo test -p server --test rsm_placement_timer_multiplier_test` passes.
- [ ] `cargo test -p server --test reconnect_snapshot_test` includes frozen multiplier coverage and passes.
- [ ] `cargo test -p client --test hand_ui_server_timer_duration_test` passes.
- [ ] `cargo test -p client --test hand_ui_placement_timer_test` remains green.
- [ ] `cargo check --workspace` passes.
- [ ] `git diff --check` passes.

---

## Implementation Notes

**Multiplier math**: Implement multiplier application through enum-owned integer ratios, not floating-point display strings. Suggested mapping: `X1 = 1/1`, `X1_5 = 3/2`, `X2 = 2/1`, `X3 = 3/1`. Compute with a widened integer type and downcast after bounds checking. The current base durations in the GDD are whole seconds, so all required effective durations are whole milliseconds.

**Protocol privacy**: `S2CSessionSettingsUpdated` is a room/session status message, not a social or accessibility disclosure. Keep requester identity out of message payloads, client state, and UI-facing copy. Server internals can store per-player requests only to compute the effective value.

**SessionReady boundary**: The multiplier must be resolved before `SessionConfig` is inserted. Preserve ADR-012 ordering: insert `SessionConfig`, insert `ServerRng`, then trigger `SessionReady`. The RSM reads only the frozen resource after that point.

**RSM duration source**: The RSM owns PLACEMENT phase duration. Client presentation reads `S2CPhaseChanged.timer_duration_ms` and snapshot timer data. No client system should multiply a local Settings value into the active timer.

**Current Hand UI gap**: The existing Hand UI timer reset path uses `PlacementTimerConfig::placement_duration_ms = 10000`. This story replaces that active-session source with server phase/snapshot duration for PLACEMENT while preserving the local config only for tests or fallback paths that do not claim multiplayer authority.

**Performance Budget**: No performance impact expected. GSS recomputes the effective value on lobby setting changes and at `SessionReady`; RSM performs one integer multiply when entering PLACEMENT. There is no per-frame network traffic added by this story.

---

## Out of Scope Guardrails

- Do not build a Settings screen or preference persistence layer in this story.
- Do not reclassify or close QA-COND-0005 in this story.
- Do not implement colorblind palette modes, reduced motion, UI scaling, or input remapping in this story.
- Do not modify sprint status or session-state files during this story.
- Do not introduce a local-only multiplayer timer override in client UI.

---

## QA Test Cases

- **GSS-42 / NP-59: highest requested multiplier wins**
  - Given: Two occupied players in LOBBY; Player A sends `C2SSetPlacementTimerMultiplier { multiplier: X1_5 }`; Player B sends `C2SSetPlacementTimerMultiplier { multiplier: X3 }`
  - When: GSS recomputes session settings
  - Then: Effective multiplier is `X3`; exactly one neutral `S2CSessionSettingsUpdated` with `X3` is emitted; no requester identity appears in the message

- **GSS-43: default multiplier**
  - Given: LOBBY reaches `SessionReady` with no timer requests
  - When: `build_session_config` runs
  - Then: `SessionConfig.placement_timer_multiplier_effective == X1`

- **GSS-44: frozen active match**
  - Given: `SessionReady` has fired with `SessionConfig.placement_timer_multiplier_effective == X2`
  - When: Any player sends another timer preference request during DRAFT_INITIAL, DRAFT_SHOP, PLACEMENT, RESOLUTION, or GAME_OVER
  - Then: Active `SessionConfig.placement_timer_multiplier_effective` remains `X2`; no active phase duration changes

- **RSM-39: standard PLACEMENT duration**
  - Given: `SessionConfig.placement_timer_multiplier_effective == X3` and standard PLACEMENT base is `10000ms`
  - When: RSM enters PLACEMENT
  - Then: Authoritative placement timer starts at `30000ms`; `S2CPhaseChanged.timer_duration_ms == 30000`

- **RSM-29c: auction-followup PLACEMENT duration**
  - Given: Prior phase was DRAFT_AUCTION, auction-followup PLACEMENT base is `12000ms`, and frozen multiplier is `X1_5`
  - When: RSM enters PLACEMENT
  - Then: Authoritative placement timer starts at `18000ms`; `S2CPhaseChanged.timer_duration_ms == 18000`

- **NP-60: reconnect snapshot**
  - Given: A player reconnects after `SessionReady`
  - When: `S2CGameSnapshot` is built
  - Then: The snapshot includes the frozen neutral multiplier and no requester attribution

- **Hand UI server timer source**
  - Given: Client receives `S2CPhaseChanged { phase: Placement, timer_duration_ms: 30000, ... }`
  - When: Hand UI enters PLACEMENT
  - Then: `PlacementTimer.remaining_ms` initializes from `30000`, not the local 10000ms config default; urgency and grace-window logic still work

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:

- `tests/unit/session/placement_timer_multiplier_test.rs` covers GSS request handling, highest-request wins, default `X1`, no post-`SessionReady` mutation, and neutral `S2CSessionSettingsUpdated`.
- `tests/unit/rsm/rsm_placement_timer_multiplier_test.rs` covers standard and auction-followup effective PLACEMENT durations.
- `tests/integration/session/reconnect_snapshot_test.rs` includes frozen multiplier snapshot coverage.
- `tests/integration/hand-ui/placement_timer_test.rs` remains green for urgency, grace-window, and checkmark behavior.
- `tests/integration/hand-ui/server_timer_duration_test.rs` or equivalent client test proves Hand UI uses server-provided PLACEMENT duration.
- `cargo check --workspace` and `git diff --check` pass.

**Status**: [x] Complete and passing.

---

## Dependencies

- Depends on: `production/epics/game-session-system/story-001-lobby-scaffold.md` (Complete) for `SessionConfig` and session resources.
- Depends on: `production/epics/game-session-system/story-002-room-create-join.md` (Complete) for occupied slot/player mapping.
- Depends on: `production/epics/game-session-system/story-004-f4-session-ready.md` (Complete) for the `SessionReady` freeze boundary.
- Depends on: `production/epics/game-session-system/story-007-reconnect-snapshot.md` (Complete) for snapshot construction.
- Depends on: `production/epics/hand-ui/story-009-placement-timer.md` (Complete) for existing PLACEMENT timer presentation behavior.
- Depends on: ADR-023 Accepted as of 2026-05-05.
- Unlocks: Settings/Accessibility preference UI can later send timer multiplier requests before `SessionReady`; QA-COND-0005 can mark the PLACEMENT timer extension sub-gap implemented after this story has passing evidence.

## Completion Notes

**Completed**: 2026-05-05
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 24/24 verified. Protocol values are `X1`, `X1_5`, `X2`, and `X3` only; C2S/S2C settings messages are registered on the reliable path; `S2CSessionSettingsUpdated` and snapshots expose no requester identity; GSS request handling is LOBBY-only and highest-request-wins; `SessionConfig.placement_timer_multiplier_effective` freezes before `SessionReady`; RSM applies the frozen multiplier after selecting standard vs auction-followup PLACEMENT base duration; `S2CPhaseChanged.timer_duration_ms` carries the effective duration; Hand UI initializes from the server-provided phase duration and preserves urgency/grace/checkmark behavior.
**Deviations**: None blocking. One named evidence target was satisfied by an equivalent client regression: `tests/integration/hand-ui/placement_timer_test.rs::hu_24_placement_timer_uses_server_phase_duration` covers server-provided PLACEMENT duration instead of creating a separate `hand_ui_server_timer_duration_test.rs`.
**Test Evidence**: `cargo fmt -p shared -- --check`; `cargo fmt -p server -- --check`; `cargo fmt -p client -- --check`; `cargo check -p shared`; `cargo check -p server`; `cargo check -p client`; `cargo test -p shared`; `cargo test -p server --test placement_timer_multiplier_test`; `cargo test -p server --test rsm_placement_timer_multiplier_test`; `cargo test -p server --test reconnect_snapshot_test`; `cargo test -p server --test game_config_defaults_test`; `cargo test -p server --test session_ready_test`; `cargo test -p server --test rsm_timers_test`; `cargo test -p client --test hand_ui_placement_timer_test`; `cargo test -p client --test presentation_plugin_scaffold_test`; `git diff --check`.
**QA-COND-0005 Impact**: PLACEMENT timer-extension sub-gap is implemented and verified by GSS-008 evidence at `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`. QA-COND-0005 remains Open for the remaining Standard-tier accessibility gaps.
**Code Review**: Skipped - lean mode (`production/review-mode.txt` absent).
**QA Coverage Gate**: Skipped - lean mode (`production/review-mode.txt` absent).
**Implementation**: Worker commit `d31b98d60b0921f01017b4427b9193c5e7383ed8` was cherry-picked onto current `main` as integration commit `4b505af`.
**Scope Notes**: `production/sprint-status.yaml` was not updated because S6-04 tracks the whole QA-COND-0005 remediation condition, not only the PLACEMENT timer-extension sub-gap. `design/assets/**`, `AGENTS.md`, and `production/session-state/codex-orchestrator-state.md` were not touched.
