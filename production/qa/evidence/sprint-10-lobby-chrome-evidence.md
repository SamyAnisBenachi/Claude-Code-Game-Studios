# Sprint 10 — Lobby Visual Chrome MVP Evidence (S10-POLISH-003)

> **Story**: S10-POLISH-003 — Lobby visual chrome MVP
> **Story file**: `production/epics/game-session-system/story-011-lobby-visual-chrome-mvp.md`
> **Sprint**: Sprint 10 active
> **Build branch**: `work/s10-polish-003-lobby-visual-chrome`
> **Build commit**: recorded at integration time by orchestrator (cherry-pick SHA into `main`)
> **Captured by**: Claude Code worker — friend-game-lite paperwork pattern
> **Date**: 2026-05-10

## Friend-game scope statement

This document records the verification evidence for the Sprint 10 lobby
visual chrome MVP. It is **friend-game-lite** evidence: the substantive
asset-wiring landed in PAW-006 (`724470e`, integrated `bb80b47`) and is
already on `main`. This story closes the friend-game-route verification
loop on top of that wiring, with an integration test that drives through
the actual `LobbyUiPlugin` `OnEnter(ClientState::Lobby)` path (rather than
the direct-spawn copy used by the PAW-006 test fixture).

The following are **explicitly not claimed** by this evidence:

- No public-release readiness.
- No full asset approval — placeholder portrait PNGs from PAW-006 remain
  in use (PAW-TD-006-a accept-risk for friend-game scope).
- No Standard-tier accessibility completion. QA-COND-0005 remains
  accepted-risk friend-game scope.
- No playtest validation / fun-hypothesis claim. QA-COND-0006 remains
  accepted-risk / deferred.
- No client-side optimistic class authority added — class lock remains
  server-authoritative via `S2CClassLocked`; no edits to
  `send_lobby_commands_system` or `drain_lobby_s2c_system` were made
  in this story.
- No full playable-client manual QA, no full game completion, no broad
  Standard-tier accessibility completion.

## Acceptance criteria evidence

### AC-1 — Lobby spawn site consumes `asset_wiring.rs` constants

**Status**: PASS (audit-only, no code change required).

**Verification**:

```text
$ rg --pcre2 -nE '"art/|"assets/|\.png"|\.jpg"' client/src/ui/lobby.rs
(no matches)
```

The spawn sites at `client/src/ui/lobby.rs:870–935` import path constants
from `asset_wiring.rs`:

- `lobby_portrait_asset(class_id)` — selector returning one of the seven
  per-class `LOBBY_PORTRAIT_*_ASSET` constants.
- `LOBBY_PLAYER_SLOT_PANEL_ASSET` — slot panel chrome.
- `LOBBY_ROOM_CODE_CHIP_ASSET` — room code chip chrome.

No inline asset path string appears in the spawn code.

### AC-2 — No `Sprite` for lobby UI

**Status**: PASS (audit-only, no code change required).

**Verification**:

```text
$ rg -n '\bSprite\b|NodeBundle|ImageBundle|UiImage::new' client/src/ui/lobby.rs
(no matches)
```

The lobby is screen-space `bevy_ui` per ADR-021. No `Sprite` use in
`client/src/ui/lobby.rs`.

### AC-3 — Integration test asserts non-default `ImageNode.image` after `OnEnter(ClientState::Lobby)` via `LobbyUiPlugin`

**Status**: PASS.

**Test file**: `tests/integration/session/lobby_chrome_wiring_test.rs`
(NEW).

**Cargo entry**: `client/Cargo.toml [[test]] name = "lobby_chrome_wiring_test"`.

**Result** (run via `cargo rustc -p client --test lobby_chrome_wiring_test`
followed by direct binary invocation; the broken `*_harness.rs` bins
flagged in PROMPT 630 prevent a clean `cargo test --no-run`, so the
direct-binary path is the canonical run shape for this worker):

```text
running 5 tests
test test_lobby_class_portraits_carry_non_default_image_node_after_on_enter_lobby ... ok
test test_lobby_room_code_chip_carries_non_default_image_node_after_on_enter_lobby ... ok
test test_lobby_portrait_per_class_path_matches_asset_wiring_selector ... ok
test test_lobby_own_slot_panel_carries_non_default_image_node_after_on_enter_lobby ... ok
test test_lobby_opponent_slot_panel_carries_non_default_image_node_after_on_enter_lobby ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

### AC-4 — Per-class portrait path matches selector

**Status**: PASS.

Covered by `test_lobby_portrait_per_class_path_matches_asset_wiring_selector`
sub-test (one of the 5 sub-tests in AC-3). The test pre-computes the
expected handle for each `ClassId` variant by calling
`asset_server.load(lobby_portrait_asset(class_id))` and asserts that the
spawned `LobbyClassPortrait` entity's `ImageNode.image` matches.

### AC-5 — PAW-006 regression preserved

**Status**: ADVISORY — pre-existing breakage surfaced, not introduced
by this story.

The PAW-006 test (`tests/integration/presentation/lobby_asset_wiring_test.rs`)
fails to compile on `origin/main@4cb02f3` (HEAD at worktree creation)
with 12 instances of `error[E0596]: cannot borrow *world as mutable, as it
is behind a & reference` — the test calls `let world = app.world();`
(immutable) and then `world.query::<...>()` which requires `&mut World`
under the Bevy 0.18 API. This is a Bevy version-bump regression that
predates this story.

This story does **not** modify
`tests/integration/presentation/lobby_asset_wiring_test.rs`. The failure
is recorded here so that a subsequent triage story can repair it (one-line
fix per occurrence: `app.world()` → `app.world_mut()`). The new test
authored by this story (`tests/integration/session/lobby_chrome_wiring_test.rs`)
correctly uses `app.world_mut()` and passes 5/5 sub-tests.

This finding sits alongside the pre-existing `hud_asset_wiring_test`
0/6, `hud_plugin_scaffold_test` 3/4, and broken `*_harness.rs` bin
findings already documented in PROMPT 630's S10-POLISH-001 closure.

### AC-6 — Class-confirm + re-ack flow preserved (no client-side authority added)

**Status**: PASS.

This story's diff for `client/src/ui/lobby.rs` is **empty** (audit-only
verification; the substantive wiring landed in PAW-006). The
`send_lobby_commands_system` (lines 451–513) and `drain_lobby_s2c_system`
(lines 208–301) paths are unchanged. The PROMPT 622 Finding D hardening
(`5da3768`) and the `be8b37d` class-confirm re-ack flow remain intact.

**Verification**: `git diff origin/main..HEAD -- client/src/ui/lobby.rs`
returns no diff on the `work/s10-polish-003-lobby-visual-chrome` branch.

### AC-7 — Manual evidence document recorded

**Status**: PASS (this document).

**Build commit SHA**: branch HEAD at push time will be the worker's
final commit; orchestrator records the cherry-pick / integration SHA on
`main` separately during the `/story-done` paperwork pass.

**Route step**: Lobby entry → class portraits visible → room code chip
visible → slot panels visible. Verified via the integration test
through `OnEnter(ClientState::Lobby)` (AC-3); manual screenshot capture
**deferred** per friend-game-lite paperwork pattern (matches
S10-POLISH-002's AC-3 / AC-7 deferral pattern at `fb30734`).

**Capture status**: deferred. Rationale: friend-game-lite scope; the
substantive verification surface for the lobby chrome handle resolution
is the integration test, which now exercises the actual
`LobbyUiPlugin` `OnEnter(ClientState::Lobby)` path. A future polish
pass owns the manual screenshot when final art lands.

**No-claim language** (cross-reference): see "Friend-game scope
statement" header at the top of this document.

## Notes

- **Scope reminder**: This is a Should Have story, not Must Have. The
  Sprint 10 close-out gate is unaffected by deferred AC-7 capture.
- **Test fixture pattern**: Mirrors
  `tests/integration/shop_auction_ui/chrome_wiring_test.rs` (S10-POLISH-002 /
  `fb30734`) — `MinimalPlugins` + `AssetPlugin::default()` +
  `init_asset::<Image>()` + `StatesPlugin` + `init_state::<ClientState>()`
  + plugin add — adapted for `LobbyUiPlugin` and `ClientState::Lobby`. One
  additional resource (`ButtonInput::<KeyCode>::default()`) is inserted
  to satisfy `lobby_keyboard_input_system`'s `Res<ButtonInput<KeyCode>>`
  parameter, since `MinimalPlugins` does not pull in `InputPlugin` (Bevy
  0.18 gates input behind its own plugin/feature). This is a test-only
  resource insertion; production builds receive `ButtonInput` via
  `DefaultPlugins`.
- **Pre-existing breakage surfaced** (NOT regression from this story):
  - `lobby_asset_wiring_test.rs` (PAW-006) — 12 × `error[E0596]` on
    `app.world()` → `world.query::<...>()` pattern (Bevy 0.18 API change
    not folded into the PAW-006 test file at PAW-006 time). Fix is a
    one-line `world()` → `world_mut()` edit per occurrence; recommend
    separate triage story.
  - The pre-existing `hud_asset_wiring_test` 0/6, `hud_plugin_scaffold_test`
    3/4, and broken `*_harness.rs` bins findings from PROMPT 630 also
    surface here as a continuing reminder; not modified by this story.
