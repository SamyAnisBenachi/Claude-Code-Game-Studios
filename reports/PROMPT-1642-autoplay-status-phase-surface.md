# PROMPT 1642 — Autoplay Status Phase Surface

**Branch:** `work/autoplay-status-phase-surface-1642`
**Commit:** `1c87b83a`
**Base:** `origin/main @ e67a3488`

## What was done

Added phase/screen state to the `autoplay/status` RPC response so recipes can
assert on game phase without relying on screenshots or human review.

### Changes — `client/src/autoplay.rs` (only file touched)

| Area | Change |
|------|--------|
| `AUTOPLAY_RPC_VERSION` | Bumped `1` → `2` (surface contract changed) |
| Schema string | `autoplay_status_v1` → `autoplay_status_v2` |
| `AutoplayStatusSnapshot` | Added `phase_label: Option<String>`, `round: Option<u32>`, `client_state_label: Option<String>` |
| `publish_status_system` | Added `Option<Res<CurrentClientPhase>>` and `Option<Res<State<ClientState>>>` params; populates the three new snapshot fields each frame |
| `render_status_json` | Emits the three new fields as JSON (null-safe) |
| `capabilities_json` | Added `status_fields` block documenting the three new keys |
| Unit tests (×3) | `render_status_json_includes_phase_fields`, `render_status_json_phase_null_when_absent`, `capabilities_json_lists_status_fields` |

### New JSON fields in `autoplay/status`

```json
{
  "schema": "autoplay_status_v2",
  "phase_label": "Placement",
  "round": 3,
  "client_state_label": "InSession",
  ...
}
```

- `phase_label` — `Debug` name of `RoundPhase` (e.g. `"Placement"`, `"DraftAuction"`, `"GameOver"`). `null` before the first `S2CPhaseChanged` lands.
- `round` — round number from `CurrentClientPhase`. `null` until first phase change.
- `client_state_label` — `"Lobby"` or `"InSession"` from `State<ClientState>`. `null` if the state resource is absent.

### Design decisions

- Both resources are read as `Option<Res<…>>` — the harness is never gated by whether game state has been registered yet (safe for early-frame polling, harness test environments, etc.).
- Read-only: no ECS writes, no gameplay mutation. Invariants from `docs/autoplay.md` are preserved.
- Import added: `use crate::state::{ClientState, CurrentClientPhase};` — within-crate, no new Cargo dependencies.

## Validation

| Gate | Result |
|------|--------|
| `git diff --check` | PASS — no whitespace errors |
| `cargo test --package client --features autoplay-remote -- autoplay::` | PASS — exit code 0; all autoplay unit tests pass |
| Pre-existing `hud_phase_transitions_test` compile error (`ScoreboardDotState` missing `known` field) | Pre-existing, unrelated to this change, owned by other worker |

## VERIFY lane

```sh
cargo test --package client --features autoplay-remote -- autoplay::
```

Expected: all 10 autoplay tests pass (7 existing + 3 new).

1642: AUTOPLAY-STATUS-PHASE-SURFACE: SHIPPED
