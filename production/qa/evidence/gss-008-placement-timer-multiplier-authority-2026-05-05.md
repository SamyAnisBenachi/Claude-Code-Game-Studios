# GSS-008 PLACEMENT Timer Multiplier Authority Evidence

| Field | Value |
|---|---|
| Story | `production/epics/game-session-system/story-008-placement-timer-multiplier-authority.md` |
| Date | 2026-05-05 |
| Worker commit | `d31b98d60b0921f01017b4427b9193c5e7383ed8` |
| Integration commit | `4b505af` |
| Verdict | PASS |
| QA condition impact | QA-COND-0005 PLACEMENT timer-extension sub-gap implemented and verified; QA-COND-0005 remains Open |

## Evidence Summary

GSS-008 implements the ADR-023 server-authoritative PLACEMENT timer multiplier path:

- Shared protocol exposes only multiplayer Standard-tier values `X1`, `X1_5`, `X2`, and `X3`.
- `0.5x` is not accepted as a multiplayer Standard-tier request.
- `C2SSetPlacementTimerMultiplier` and `S2CSessionSettingsUpdated` are registered on `ReliableChannel`.
- `S2CSessionSettingsUpdated` carries only the neutral effective multiplier and no requester identity.
- GSS stores per-player requests only before `SessionReady`, treats missing requests as `X1`, computes highest requested value, and caps at `X3`.
- The effective multiplier is frozen into `SessionConfig.placement_timer_multiplier_effective` before `SessionReady`.
- RSM selects the PLACEMENT base duration first, then applies the frozen multiplier.
- `S2CPhaseChanged.timer_duration_ms` carries the effective PLACEMENT duration.
- `S2CGameSnapshot` carries the frozen neutral multiplier without attribution.
- Hand UI initializes PLACEMENT timer state from the server-provided phase duration instead of the local 10 second fallback when phase data is present.

## Verification Commands

All commands passed on 2026-05-05:

- `cargo fmt -p shared -- --check`
- `cargo fmt -p server -- --check`
- `cargo fmt -p client -- --check`
- `cargo check -p shared`
- `cargo check -p server`
- `cargo check -p client`
- `cargo test -p shared`
- `cargo test -p server --test placement_timer_multiplier_test`
- `cargo test -p server --test rsm_placement_timer_multiplier_test`
- `cargo test -p server --test reconnect_snapshot_test`
- `cargo test -p server --test game_config_defaults_test`
- `cargo test -p server --test session_ready_test`
- `cargo test -p server --test rsm_timers_test`
- `cargo test -p client --test hand_ui_placement_timer_test`
- `cargo test -p client --test presentation_plugin_scaffold_test`
- `git diff --check`

## QA-COND-0005 Disposition

This evidence supports marking only the PLACEMENT timer-extension sub-gap as implemented and verified. It does not close QA-COND-0005 as a whole.

Remaining Standard-tier accessibility gaps, including colorblind modes, reduced motion, UI scaling, input remapping, cognitive supports, contrast/text-size verification, and other browser/WASM accessibility evidence, remain open for a later evidence/disposition pass.
