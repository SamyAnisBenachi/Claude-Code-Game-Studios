# Story 011: Lobby Visual Chrome MVP — Class Carousel + Slot Panels + Room Code Chip

> **Epic**: Game Session System
> **Story ID**: S10-POLISH-003
> **Status**: In Progress
> **Layer**: Presentation (Polish)
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 10 active

## Context

This story closes the end-to-end visual chrome verification loop for the
lobby in the active friend-game route. The asset-wiring substrate landed
in PAW-006 (`production/epics/presentation-asset-wiring/story-006-lobby-portraits.md`,
done at `724470e`, integrated at `bb80b47`): `client/src/asset_wiring.rs`
exposes the lobby chrome path constants (per-class portrait selector
`lobby_portrait_asset(class_id)`, `LOBBY_PLAYER_SLOT_PANEL_ASSET`,
`LOBBY_ROOM_CODE_CHIP_ASSET`) and PAW-006 wired them into
`client/src/ui/lobby.rs` `spawn_lobby_ui_system` with a partial-App
integration test (`tests/integration/presentation/lobby_asset_wiring_test.rs`)
asserting non-default `ImageNode.image` handles on the 10 lobby chrome
entities (7 portraits + 2 slot panels + 1 chip).

S10-POLISH-003 verifies that the wired chrome **stays consumed** through
the actual `LobbyUiPlugin` `OnEnter(ClientState::Lobby)` spawn path
(not just a direct test-helper spawn copy as in PAW-006-d), preserves the
existing class-confirm + class-locked re-ack flow added at `be8b37d`,
preserves no-client-side-authority over class lock, and records one
manual evidence document of the friend-game route showing portraits and
slot panels rendered through the lobby. The substantive plumbing exists;
this story records that the friend-game build visibly looks like a
styled lobby rather than raw `bevy_ui` `Node` rectangles, captures a
manual screenshot of the active route, and adds an integration test
that drives through `OnEnter(ClientState::Lobby)` (not just a
`spawn_lobby_chrome` direct copy) to assert the lobby chrome entities
still hold non-default `ImageNode.image` handles.

This story does **not** add new asset authoring, change network
protocol, add client-side optimistic class authority, claim full asset
approval, claim final visual polish completion, claim public release
readiness, claim full playable-client manual QA, claim broad
Standard-tier accessibility completion, or close any S8 / Sprint 9
carried condition.

**Primary sources**:

- `production/sprints/sprint-10.md` (S10-POLISH-003 row, line 102)
- `production/epics/presentation-asset-wiring/story-006-lobby-portraits.md`
  (PAW-006 — the wiring substrate this story consumes)
- `design/ux/class-picker.md` (class picker UX spec — Lobby
  Anticipation phase)
- `docs/architecture/adr-021-presentation-layer-architecture.md`
  (ADR-021 — `bevy_ui` for screen-space UI; `ImageNode` for chrome)

**GDD, UX, and TR trace**:

- **UX**: `design/ux/class-picker.md` Lobby — class portrait, class
  name, slot panels, room code chip are all listed in the Component
  Inventory. The wired chrome is the visual surface of these
  components for the friend-game MVP.
- **TR-ID**: `TR-PAW-006` — *"Lobby class portraits, slot panels,
  room code chip wired"* (per PAW-006 acceptance criteria). The wiring
  substrate is owned by PAW-006 (done); this story closes the
  friend-game route verification loop on the same TR.
- **Related TR**: `TR-GSS-002` (lobby class selection / re-ack) —
  this story exercises the contract through the lobby UI but does
  not extend or modify it.

**ADR Governing Implementation**:

- **ADR-021** (Presentation Layer Architecture) — primary. Lobby is
  screen-space `bevy_ui`; chrome surfaces use `ImageNode`; never
  `Sprite` for these surfaces.
- **ADR-002** (Client-Server Authority) — class lock is server
  authoritative. Lobby UI presents `LobbyViewState.locked_class`
  driven by `S2CClassLocked`; the UI never writes `locked_class`
  optimistically.

**Engine**: Bevy 0.18 (Rust) | **Risk**: LOW (visual chrome
verification — no protocol or authority change; failure mode is
"lobby looks unstyled; no functional regression")

**Engine Notes**: Bevy 0.18 Required Components — `ImageNode { image:
handle, .. }` spawned directly via
`commands.spawn((Node { .. }, ImageNode::new(handle), ..))`. Path
constants are pulled from `client/src/asset_wiring.rs` via
`lobby_portrait_asset(class_id)`, `LOBBY_PLAYER_SLOT_PANEL_ASSET`, and
`LOBBY_ROOM_CODE_CHIP_ASSET`; no inline string literals for asset
paths in `client/src/ui/lobby.rs` spawn code. `NodeBundle` /
`ImageBundle` / `UiImage::new()` are forbidden in Bevy 0.18 (control
manifest Presentation Layer + Forbidden APIs table).

**Control Manifest Rules (2026-05-05)**:

- **Required**: Lobby chrome uses `ImageNode` — never `Sprite`.
  — source: ADR-021
- **Required**: Path constants from `asset_wiring.rs` (lobby class
  portrait, slot panel, room code chip). No inline string literals
  in `client/src/ui/lobby.rs` spawn code. — source: ADR-021 + PAW-006
- **Forbidden**: `NodeBundle` / `ImageBundle` / `UiImage::new()`.
  — source: ADR-021 + engine-reference
- **Forbidden**: `Sprite` for lobby UI surfaces. — source: ADR-021
- **Forbidden**: Client-side optimistic class authority. The lobby
  may locally preview a class via `selected_class` but never writes
  `locked_class` without a `S2CClassLocked` from the server.
  — source: ADR-002

---

## Scope

### In Scope

- Verify (read-only audit) that every spawn site in
  `client/src/ui/lobby.rs` for the following surfaces uses an
  `ImageNode` wired to a `client/src/asset_wiring.rs` path constant
  (no inline asset path strings):
  - `LobbyClassPortrait` per `ClassId` (7 entities)
  - `LobbyOwnSlotPanel`
  - `LobbyOpponentSlotPanel`
  - `LobbyRoomCodeChip`
- Add one integration test under
  `tests/integration/session/lobby_chrome_wiring_test.rs` that drives
  through `OnEnter(ClientState::Lobby)` via the actual
  `LobbyUiPlugin` (not a `spawn_lobby_chrome` direct copy) and
  asserts every lobby chrome entity carries a non-default
  `ImageNode.image` handle. The test must use the canonical
  partial-App fixture pattern (`MinimalPlugins`, `AssetPlugin::default()`,
  `init_asset::<Image>()`, `StatesPlugin`,
  `init_state::<ClientState>()`) and exercise the
  `OnEnter(ClientState::Lobby)` schedule.
- Run the friend-game route through `ClientState::Lobby` (browser
  or native client) and capture one manual evidence note showing
  the wired chrome rendered. Record the route + capture status
  at `production/qa/evidence/sprint-10-lobby-chrome-evidence.md`.
- Preserve the existing class-confirm + class-locked re-ack flow
  introduced at `be8b37d` — no edits to the C2S send path or the
  S2C drain path are in scope.

### Out of Scope

- No new asset authoring. Final art replacement is a future story;
  PAW-TD-006-a (placeholder portrait PNGs vs final art) is
  accept-risk for friend-game scope and remains so.
- No final visual polish (typography weight, spacing pixel-perfect
  alignment, animation easing curves). This is MVP wiring
  verification, not visual design pass.
- No claim of full asset approval — placeholder portrait PNGs from
  PAW-006 remain in use.
- No new protocol message, no protocol change, no Lightyear channel
  change.
- No client-side optimistic class authority.
- No carousel arrows, dot position indicator, Krosmic card name
  tooltips, or class-tinted card frame container — all listed in
  `design/ux/class-picker.md` Component Inventory but **not** in
  scope for this MVP wiring verification. Track as future polish.
- No changes to `server/`, `shared/`, or other client UI plugins.
- No closure of S8-QA-001-W1, QA-COND-0005, or QA-COND-0006.
- No claim of public release readiness, full playable-client manual
  QA, full game completion, broad Standard-tier accessibility
  completion, or playtest/fun-hypothesis validation.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-10.md:102` S10-POLISH-003 row.)

- [x] **AC-1 Lobby spawn site consumes `asset_wiring.rs` constants**:
      GIVEN every spawn site under `client/src/ui/lobby.rs` for
      `LobbyClassPortrait`, `LobbyOwnSlotPanel`,
      `LobbyOpponentSlotPanel`, and `LobbyRoomCodeChip`, WHEN the
      spawn code is read, THEN the `ImageNode.image` handle is
      sourced from `lobby_portrait_asset(class_id)`,
      `LOBBY_PLAYER_SLOT_PANEL_ASSET`, or
      `LOBBY_ROOM_CODE_CHIP_ASSET` (from `asset_wiring.rs`); no
      inline asset path string literal appears in the spawn site.
      *Verification*: `grep -nE '"art/|"assets/|\.png"|\.jpg"' client/src/ui/lobby.rs`
      returns zero hits outside comments / docstrings.
- [x] **AC-2 No `Sprite` for lobby UI**: GIVEN the same file, WHEN
      the diff is filtered for `Sprite::` / `Sprite {` / `Sprite ` use,
      THEN zero hits are found in `client/src/ui/lobby.rs` (lobby is
      screen-space `bevy_ui`; board content is `Sprite`).
- [x] **AC-3 Integration test asserts non-default `ImageNode.image`
      after `OnEnter(ClientState::Lobby)` via `LobbyUiPlugin`**:
      GIVEN a partial-App test fixture configured with
      `MinimalPlugins`, `AssetPlugin::default()`,
      `init_asset::<Image>()`, `StatesPlugin`,
      `init_state::<ClientState>()`, and `LobbyUiPlugin`, WHEN
      `NextState::<ClientState>::set(ClientState::Lobby)` is
      transitioned, THEN every lobby chrome entity (7 class
      portraits + 2 slot panels + 1 room-code chip) carries a
      non-default `ImageNode.image` handle (handle ≠
      `Handle::<Image>::default()`).
- [x] **AC-4 Per-class portrait path matches selector**: GIVEN the
      same fixture as AC-3, WHEN the 7 spawned `LobbyClassPortrait`
      entities are queried by `class_id`, THEN each entity's
      `ImageNode.image` resolves to the handle returned by
      `asset_server.load(lobby_portrait_asset(class_id))` for its
      `class_id` (covers `Iop`, `Cra`, `Sacrier`, `Xelor`,
      `Ecaflip`, `Sadida`, `Neutral`).
- [~] **AC-5 PAW-006 regression preserved** (ADVISORY — pre-existing
      breakage; not introduced by this story; see evidence doc AC-5): GIVEN the existing
      `tests/integration/presentation/lobby_asset_wiring_test.rs`
      (introduced in PAW-006 / `724470e`), WHEN it is re-run after
      this story's changes, THEN it passes without modification (no
      PAW-006 regression; the existing test continues to assert the
      direct-spawn path while the new test asserts the
      `LobbyUiPlugin` `OnEnter` path).
- [x] **AC-6 Class-confirm + re-ack flow preserved
      (no client-side authority added)**: GIVEN the lobby's
      `send_lobby_commands_system` and `drain_lobby_s2c_system`
      paths from `be8b37d` and PROMPT 622 (Finding D — `5da3768`),
      WHEN the diff for this story is read, THEN no edit to either
      system is added; the C2S send path (`C2SConfirmClass`) and
      the S2C drain path (`S2CClassLocked` re-ack on duplicate
      class) remain unchanged.
- [x] **AC-7 Manual evidence document recorded** (capture deferred per
      friend-game-lite paperwork pattern; document authored at
      `production/qa/evidence/sprint-10-lobby-chrome-evidence.md`): GIVEN the
      friend-game lobby route capture, WHEN the evidence document
      at `production/qa/evidence/sprint-10-lobby-chrome-evidence.md`
      is read, THEN it records: build commit SHA, route step
      (lobby entry → class portraits visible → room code chip
      visible → slot panels visible), capture status (deferred or
      delivered, per friend-game-lite paperwork pattern), and the
      friend-game no-claims language (no public release readiness,
      no full asset approval, no Standard-tier accessibility, no
      playtest validation, no client-side class authority added).

---

## Implementation Notes

The substantive wiring landed in PAW-006 (`724470e`, integrated at
`bb80b47`). This story is the friend-game route verification +
integration test through the actual `LobbyUiPlugin` + manual evidence
loop on top of that wiring. Expected work shape:

1. **Audit pass** (read-only): grep `client/src/ui/lobby.rs` for
   inline asset path strings (`.png`, `.jpg`, `assets/`), for any
   `Sprite` use (forbidden for these panels), and for any
   `NodeBundle` / `ImageBundle` / `UiImage::new()` use (forbidden
   in Bevy 0.18). Record findings.
2. **Integration test add**: add
   `tests/integration/session/lobby_chrome_wiring_test.rs` that
   follows the partial-App fixture pattern from
   `tests/integration/shop_auction_ui/chrome_wiring_test.rs`
   (S10-POLISH-002 / `fb30734`):
   `MinimalPlugins`, `AssetPlugin::default()`,
   `init_asset::<Image>()`, `StatesPlugin`,
   `init_state::<ClientState>()`, `LobbyUiPlugin`. Transition to
   `ClientState::Lobby`, then assert non-default `ImageNode.image`
   on every lobby chrome entity. Add per-class assertion that the
   handle matches `lobby_portrait_asset(class_id)`.
3. **Cargo.toml [[test]] entry**: register the new test under
   `client/Cargo.toml`. No other Cargo mutation.
4. **Friend-game route capture**: launch a single-client local
   build, walk through lobby entry, screenshot the rendered chrome,
   and write
   `production/qa/evidence/sprint-10-lobby-chrome-evidence.md`. If
   no manual capture is feasible inside the worker, defer the
   capture per friend-game-lite paperwork pattern (record build
   SHA + no-claims language; mark capture as deferred — same
   pattern S10-POLISH-002 used for AC-3 / AC-7).
5. **Regression check**: re-run
   `cargo test -p client --test lobby_asset_wiring_test` to confirm
   PAW-006-d behaviour is intact.

If the audit pass finds zero violations (PAW-006 wiring is intact and
no later commit regressed it), the story collapses to: integration
test add + evidence-doc authoring.

The integration test fixture pattern from
`tests/integration/shop_auction_ui/chrome_wiring_test.rs`
(introduced in `fb30734`) is the canonical reference — mirror its
`MinimalPlugins` + `AssetPlugin` + `init_asset::<Image>()` +
`StatesPlugin` + `init_state` + plugin add shape and adapt it for
`LobbyUiPlugin` + `ClientState::Lobby`.

---

## Performance Budget

- **Presentation steady-state**: < 1 ms per frame (per ADR-021
  Performance Guardrails). Lobby chrome entities are spawned once
  on `OnEnter(ClientState::Lobby)` and despawned on
  `OnExit(ClientState::Lobby)`; no per-frame spawn / despawn.
- **Phase-boundary frame**: not applicable — lobby is pre-game; no
  in-game phase transitions touch this surface.
- No hot-path code changed by this story.

---

## QA Test Cases

(Source: `production/sprints/sprint-10.md:102` AC text.)

- **Asset path constant audit**
  - Given: post-implementation `client/src/ui/lobby.rs`.
  - When: `grep -nE '"art/|"assets/|\.png"|\.jpg"' client/src/ui/lobby.rs`
    is run.
  - Then: zero hits outside comments / docstrings.

- **Sprite / ImageNode boundary audit**
  - Given: post-implementation `client/src/ui/lobby.rs`.
  - When: `grep -n 'Sprite' client/src/ui/lobby.rs` is run.
  - Then: zero hits (lobby is `bevy_ui`).

- **Integration test through plugin OnEnter**
  - Given: partial-App fixture with `LobbyUiPlugin`.
  - When: `NextState::<ClientState>::set(ClientState::Lobby)` and
    `app.update()` are run.
  - Then: 7 portrait + 2 slot panel + 1 chip entities all carry a
    non-default `ImageNode.image` handle.

- **Per-class portrait path**
  - Given: same fixture as above.
  - When: each `LobbyClassPortrait` is queried by `class_id`.
  - Then: `ImageNode.image == asset_server.load(lobby_portrait_asset(class_id))`
    for each of the 7 variants.

- **PAW-006 regression**
  - Given: this story's diff.
  - When: `cargo test -p client --test lobby_asset_wiring_test` is
    run.
  - Then: passes without modification.

- **Class-confirm flow preserved**
  - Given: this story's diff.
  - When: filtered for edits to `send_lobby_commands_system` or
    `drain_lobby_s2c_system`.
  - Then: zero edits.

- **Manual screenshot capture**
  - Given: a built friend-game client.
  - When: a manual play-through reaches `ClientState::Lobby`.
  - Then: the screenshot is captured (or deferred per
    friend-game-lite paperwork pattern) and recorded in
    `production/qa/evidence/sprint-10-lobby-chrome-evidence.md`
    with build/commit SHA and the explicit no-claim language.

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/session/lobby_chrome_wiring_test.rs` (new)
- `cargo test -p client --test lobby_chrome_wiring_test`

**Required regression targets** (must remain green):

- `cargo test -p client --test lobby_asset_wiring_test`

**Required manual evidence path**:

- `production/qa/evidence/sprint-10-lobby-chrome-evidence.md` (new) —
  records build/commit SHA, capture status (delivered or deferred),
  and the explicit no-claim language listed in AC-7.

**Status**: [x] Implemented in this commit — 5/5 sub-tests PASS in
`tests/integration/session/lobby_chrome_wiring_test.rs`. PAW-006
regression target (`tests/integration/presentation/lobby_asset_wiring_test.rs`)
fails to compile with 12 × `error[E0596]` on `app.world()` →
`world.query::<...>()` — pre-existing Bevy 0.18 API breakage NOT
introduced by this story (see evidence doc AC-5).

---

## Files Modified

Anticipated diff (final shape may vary; this is the authoring-time
target):

| Path | Change |
|---|---|
| `tests/integration/session/lobby_chrome_wiring_test.rs` | NEW — partial-App fixture asserting non-default `ImageNode.image` on the 10 lobby chrome entities after `OnEnter(ClientState::Lobby)` via `LobbyUiPlugin`, plus per-class portrait path match. |
| `client/Cargo.toml` | Register `[[test]] name = "lobby_chrome_wiring_test"` entry. |
| `production/qa/evidence/sprint-10-lobby-chrome-evidence.md` | NEW — manual evidence document per AC-7. |
| `production/epics/game-session-system/story-011-lobby-visual-chrome-mvp.md` | THIS FILE — story authoring per Sprint 10 docs-only prerequisite (slot 011 in Game Session System epic was free). |

No protocol files, no `shared/` files, no `server/` files, no
`design/gdd/`, no `docs/architecture/` files are modified. No new
asset files are added. No edits to `client/src/ui/lobby.rs` are
expected (audit-only) — if the audit surfaces a violation, the patch
scope expands minimally.

---

## Dependencies

- Depends on: `production/epics/presentation-asset-wiring/story-006-lobby-portraits.md`
  (PAW-006) — Done. Provides the `LobbyClassPortrait`,
  `LobbyOwnSlotPanel`, `LobbyOpponentSlotPanel`, `LobbyRoomCodeChip`
  entities this story consumes.
  Verification: `git log --oneline 724470e bb80b47` returns both
  commits on `main`.
- Depends on: S10-PAW-001 PAW-006 row reaching `done` in
  `production/sprint-status.yaml` (closed at PROMPT 598-RETRY).
- Depends on: `client/src/asset_wiring.rs` `lobby_portrait_asset(class_id)`,
  `LOBBY_PLAYER_SLOT_PANEL_ASSET`, `LOBBY_ROOM_CODE_CHIP_ASSET` —
  present on `main` since `724470e`.
- Depends on: `client/src/ui/lobby.rs` `LobbyUiPlugin` —
  present on `main`, with the `be8b37d` class-confirm re-ack flow
  and the `5da3768` (PROMPT 622 Finding D) C2S hardening intact.
- Depends on: `tests/integration/shop_auction_ui/chrome_wiring_test.rs`
  fixture pattern — present on `main` since `fb30734`
  (S10-POLISH-002).
- Depends on: ADR-021 + ADR-002 Accepted (per
  `docs/architecture/control-manifest.md` header line 6).
- Unlocks: friend-game-route visible chrome MVP for the lobby
  (`/story-done` flips `production/sprint-status.yaml`
  S10-POLISH-003 → `done`).

---

## Readiness Notes

**Implementation readiness verdict at authoring time**: READY.

- Story file authored at the canonical Sprint 10 path agreed in
  `production/sprints/sprint-10.md:129`:
  `production/epics/game-session-system/story-011-lobby-visual-chrome-mvp.md`.
  The `story-011-` slot in the Game Session System epic is free
  (last existing story is `story-010-result-acknowledgement-cleanup-handshake.md`).
- All TR-IDs, ADR refs, control-manifest version, engine notes,
  test evidence path, out-of-scope, and dependency rows are
  embedded.
- PAW-006 wiring substrate is on `main`; the integration test
  exercises the `LobbyUiPlugin` `OnEnter` path rather than a
  direct-spawn copy — a new assertion surface, not a regression
  of the PAW-006 test.

---

## Definition of Done

This story is **ready to start** at authoring time, not yet
substantively complete. Done means:

- All acceptance criteria above checked.
- Automated test
  `tests/integration/session/lobby_chrome_wiring_test.rs` passing
  under `cargo test -p client --test lobby_chrome_wiring_test`.
- PAW-006 regression test
  (`cargo test -p client --test lobby_asset_wiring_test`) remains
  green.
- Manual evidence document at
  `production/qa/evidence/sprint-10-lobby-chrome-evidence.md`
  exists, records the friend-game route step, capture status
  (delivered or deferred), build/commit SHA, and the explicit
  no-claim language.
- `/story-done` flips `production/sprint-status.yaml`
  S10-POLISH-003 → `done`.
- No public-release readiness, full playable-client manual QA,
  full game completion, or broad Standard-tier accessibility
  completion is claimed at close.
