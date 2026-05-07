# Story 006: Native Friend-Game Operator Controls

> **Epic**: Playable Client
> **Status**: Ready - backlog preparation only
> **Layer**: Polish / Native Client Operator UX
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 9 active / ready

## Context

This story makes the native friend-game client operator-playable through real
player controls. Story authoring did not implement code, close Sprint 8, run
smoke, run QA sign-off, run a gate, or claim full manual/browser/native
two-client completion.

Current source of truth for this prep is `origin/main` at
`f6864b3190fb9609eee8cd72e13b212c10a8fdf8`.

The live friend-game player guide found that current native launch can show the
lobby, but manual progression is blocked or unreliable:

- Digits `0` through `3` both type into the room-code input and alter requested
  slot.
- There is no clean mouse/button submit path for join.
- `DRAFT_INITIAL` purchase and ready/retract handlers exist internally, but a
  real operator path to buy a draft slot and Ready/Retract is not confirmed.
- `DRAFT_SHOP` purchase, refresh, ready, and retract handlers exist internally,
  but native operator input is incomplete.
- Placement has staging and submit handlers, but the full real player
  card/drag/click submit path is not confirmed operator-complete.
- This blocks native two-client manual route evidence even though controlled
  real-Lightyear tests cover the route.

**Primary sources**:

- `production/epics/playable-client/EPIC.md`
- `production/epics/playable-client/story-001-primary-client-bootstrap-fresh-lobby-entry.md`
- `production/epics/playable-client/story-002-live-draft-shop-hand-bridge.md`
- `production/epics/playable-client/story-003-real-end-to-end-loop-verification.md`
- `production/epics/playable-client/story-004-friend-game-result-endpoint-expansion.md`
- `production/epics/playable-client/story-005-draft-shop-auction-placement-resolution-loop-polish.md`
- `production/epics/presentation-layer/story-006-result-screen-mvp.md`
- `production/sprints/sprint-8.md`
- `production/sprint-status.yaml`
- `production/sprints/sprint-9.md`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/qa/evidence/captures/playable-client-real-e2e-loop/phase-captures.md`
- `design/ux/main-menu.md`
- `design/ux/class-picker.md`
- `design/ux/shop-auction-ui.md`
- `design/ux/hand-ui.md`
- `design/ux/result-screen.md`

**GDD and TR trace**:

- `design/gdd/network-protocol.md` / `TR-NP-001`: clients express intent
  through C2S messages only and the server owns game logic.
- `design/gdd/network-protocol.md` / `TR-NP-003`: connection starts with
  `C2SHello` and server response.
- `design/gdd/network-protocol.md` / `TR-NP-005`: invalid-phase C2S messages
  are discarded server-side.
- `design/gdd/network-protocol.md` / `TR-NP-007`: `C2SSubmitPlacement` is
  silent and `S2CPlacementReveal` is the only placement-close signal.
- `design/gdd/network-protocol.md` / `TR-NP-009`: `S2CResolutionEvent` arrives
  before the following reliable phase change.
- `design/gdd/network-protocol.md` / `TR-NP-011`: placement reveal is atomic for
  both players.
- `design/gdd/game-session-system.md` / `TR-GSS-001`, `TR-GSS-004`,
  `TR-GSS-007`: create/join/class/session-ready flow leads into the round loop.
- `design/gdd/card-data-pool.md` / `TR-CDP-010`: draft/shop payloads are
  reliable unicast before client phase/UI use.
- `design/gdd/shop-auction-ui.md` / `TR-SAU-006`: shop and auction panel
  transitions and input gating follow authoritative phase and timing behavior.
- `design/gdd/hand-ui.md` / `TR-HU-005`, `TR-HU-008`, `TR-PRES-001`: hand,
  placement, and economy views use server-projected card/economy data.
- `design/gdd/board-rendering.md` / `TR-BR-002`, `TR-BR-008`: board coordinate
  mapping and spawn highlights are client-side presentation over authoritative
  data.
- `design/gdd/round-state-machine.md` / `TR-RSM-007`, `TR-RSM-008`,
  `TR-RSM-009`: phase timers, game-over detection, and phase broadcasts remain
  server-authoritative.
- `design/gdd/hud.md` / `TR-HUD-009`: HUD remains frozen at `GAME_OVER`.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-003: Cargo Workspace Structure](../../../docs/architecture/adr-003-cargo-workspace-structure.md)
- [ADR-007: Placement Buffer](../../../docs/architecture/adr-007-placement-buffer.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md)
- [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md)
- [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md)
- [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

All referenced ADRs are Accepted.

**ADR Decision Summary**: Native operator controls may create local focus,
hover, drag, pending, and pressed-button state, but every server-owned outcome
must come from C2S intent followed by authoritative S2C or snapshot state. The
client must not locally create room membership, class lock, phase progression,
hand contents, shop contents, auction settlement, committed placement, result,
or teardown state.

**Engine**: Bevy 0.18 + Lightyear 0.26 + native primary client |
**Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and
`liv-bevy-lightyear` before editing any Lightyear or networking `.rs` file.
Operator controls must use Bevy 0.18 Required Components API, normal
`Interaction`/focus handling, and the existing presentation and UI plugin
ownership. Do not use deprecated bundle APIs, direct server imports, duplicate
Lightyear drains, or debug-only key paths as the only operator path.

**Lightyear Notes**: Final evidence must use one real local server and two real
native primary clients from the same commit. The manual path must send real C2S
messages and observe real S2C/snapshot convergence. Automated tests may cover
ECS input surfaces, but story completion cannot rely on direct `World`
injection, fake snapshots, or harness-only C2S sends as the manual proof.

**Control Manifest Rules (2026-05-05)**:

- Required: client state is read-only and server remains authoritative.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: Hand UI, HUD, and Shop/Auction UI read `PlayerEconomyView`.
- Required: `S2CResolutionEvent` stays before following phase changes on
  `ReliableChannel`.
- Required: UI and presentation work follow ADR-021 composition and ordering.
- Required: no duplicate S2C phase, economy, shop, auction, result, or
  placement drains.
- Guardrail: presentation steady-state stays below 1 ms per frame and
  phase-boundary spikes stay below 3 ms.

## Sprint 8 Scope Guard

This story preserves all Sprint 8 carried conditions and non-claims. It does
not close Sprint 8, run `/dev-story`, run `/story-done`, run smoke, run QA
sign-off, run a gate, or implement code during readiness authoring.

`QA-COND-0005` remains accepted risk for friend-game scope only and is not
verified Standard-tier accessibility completion. `QA-COND-0006` remains
accepted-risk/deferred and is not playtest evidence or fun-hypothesis
validation.

This story must not claim public, external, commercial, store, deployment,
release-candidate, or release readiness. It must not claim broad accessibility
completion, full playable-client manual QA, playtest validation,
fun-hypothesis validation, full game completion, or manual/browser/native
`GAME_OVER` completion unless a future evidence scope actually captures it.

## Scope

### In Scope

- Native lobby operator controls for room-code entry, requested-slot selection,
  Create Room, Join Room, class selection, and class confirmation.
- A focus model that separates text input from command shortcuts.
- Mouse/button paths for all required lobby actions.
- Native `DRAFT_INITIAL` controls for card purchase and Ready/Retract.
- Native `DRAFT_SHOP` controls for slot purchase, refresh, and Ready/Retract.
- Native auction controls for bidding through real `C2SPlaceBid`.
- Native placement controls for card select, drag or click staging, unstaging,
  and submit through real `C2SSubmitPlacement`.
- Evidence that operator controls send C2S intents and converge only from
  authoritative S2C/snapshot state.
- A native two-client manual smoke evidence path from lobby through the latest
  reachable friend-game endpoint, with exact blockers recorded.
- Explicit result/Return to Lobby dependency handling when `GAME_OVER` is
  reached before the result contract is implemented.

### Out of Scope

- Implementing this story during this docs-prep prompt.
- Sprint 9 activation, Sprint 8 close-out, smoke, QA sign-off, gate-check, or
  story-done updates.
- New server authority rules, new game modes, rematch, broad result-screen work,
  or result acknowledgement contract implementation unless separately scoped.
- Public release readiness, broad Standard-tier accessibility completion,
  full playable-client manual QA, playtest validation, fun-hypothesis
  validation, full game completion, or closure of `QA-COND-0005` or
  `QA-COND-0006`.
- Broad UI redesign, marketing/onboarding screens, art production, balance
  tuning, card content expansion, or unrelated reconnect polish.
- Client-side optimistic authority for any server-owned state.

## Acceptance Criteria

- [ ] **Room-code input is cleanly separated from shortcuts**: GIVEN the
      room-code field has text focus, WHEN the operator presses digits `0`
      through `9` or letters `A` through `Z`, THEN only the room-code input is
      updated, requested slot/class/command shortcuts do not fire, input is
      normalized to the server-supported room-code format, and length remains
      bounded.
- [ ] **Command shortcuts do not type into room code**: GIVEN the room-code
      field does not have text focus, WHEN the operator uses any supported
      shortcut, THEN the shortcut may trigger only its owning command and must
      not append characters to the room-code input.
- [ ] **Requested slot has an explicit control**: GIVEN the joiner is in the
      lobby, WHEN the operator changes the requested slot through the native UI,
      THEN the selected slot changes without mutating the room-code text and
      the displayed join target matches the value sent in `C2SJoinRoom`.
- [ ] **Host Create Room control exists**: GIVEN the host is connected in the
      lobby, WHEN the operator clicks or keyboard-activates Create Room, THEN
      the client sends exactly one `C2SCreateRoom` for the activation and does
      not display a room code, occupied slot, or session id until
      `S2CRoomCreated` or a snapshot supplies it.
- [ ] **Joiner Join Room control exists**: GIVEN the joiner has a non-empty room
      code and requested slot selected, WHEN the operator clicks or
      keyboard-activates Join Room, THEN the client sends exactly one
      `C2SJoinRoom` with the displayed room code and requested slot, keeps the
      input available on rejection, and waits for `S2CJoinAck` or
      `S2CJoinRejected` before changing confirmed room state.
- [ ] **Join control is disabled when invalid**: GIVEN the room-code input is
      empty or the client is already waiting on an in-flight join, WHEN the
      operator activates Join Room, THEN no duplicate or empty `C2SJoinRoom` is
      sent and visible feedback explains the current state.
- [ ] **Class select controls are operator-complete**: GIVEN the lobby class
      picker is visible, WHEN the operator clicks or keyboard-navigates to a
      class, THEN `C2SSelectClass` is sent only as a preview intent, the local
      selected-class display updates as pending UI only, and server-owned slot
      or lock state remains unchanged until S2C state arrives.
- [ ] **Class confirm control is operator-complete**: GIVEN a class is selected,
      WHEN the operator clicks or keyboard-activates Confirm Class, THEN exactly
      one `C2SConfirmClass` is sent, rejection keeps the picker usable, and
      confirmed/locked state is displayed only from `S2CClassLocked`,
      `S2CSlotUpdated`, `S2CClassesRevealed`, or snapshot state.
- [ ] **DRAFT_INITIAL purchase control exists**: GIVEN `DRAFT_INITIAL` is active
      and the player has an available offered card, WHEN the operator activates
      a card slot by click or keyboard focus, THEN the client sends
      `C2SPurchaseCard` for that card, marks only local pending feedback, and
      updates purchased/hand/economy state only from `S2CCardAcquired`,
      `S2CGoldUpdate`, `S2CDraftOffering`, or snapshot state.
- [ ] **DRAFT_INITIAL Ready/Retract control exists**: GIVEN `DRAFT_INITIAL` is
      active, WHEN the operator activates Ready or Retract, THEN the client
      sends `C2SSignalReady { retract: false }` or
      `C2SSignalReady { retract: true }` as appropriate, prevents duplicate
      spam while in flight, and derives ready/phase state only from
      authoritative S2C/snapshot convergence.
- [ ] **DRAFT_SHOP purchase control exists**: GIVEN `DRAFT_SHOP` is active and a
      shop slot contains a purchasable card, WHEN the operator activates the
      slot by click or keyboard focus, THEN the client sends `C2SPurchaseCard`,
      displays local pending feedback only, and updates shop slot, hand, and
      economy state only from `S2CCardAcquired`, `S2CShopSlots`,
      `S2CGoldUpdate`, or snapshot state.
- [ ] **DRAFT_SHOP refresh control exists**: GIVEN `DRAFT_SHOP` is active and
      refresh is available, WHEN the operator activates Refresh, THEN the client
      sends one `C2SRefreshShop`, disables duplicate activation while pending,
      and displays refreshed slots only after authoritative shop/economy S2C or
      snapshot state arrives.
- [ ] **DRAFT_SHOP Ready/Retract control exists**: GIVEN `DRAFT_SHOP` is active,
      WHEN the operator activates Ready or Retract, THEN the client sends the
      matching `C2SSignalReady` intent and does not locally advance the phase,
      hide shop state as committed, or alter readiness beyond pending UI.
- [ ] **Auction bid controls exist**: GIVEN `DRAFT_AUCTION` is active with an
      auction card and legal bid options, WHEN the operator activates a bid
      button or amount control, THEN the client sends exactly one `C2SPlaceBid`
      for the chosen amount, prevents duplicate in-flight sends, and derives
      accepted, rejected, leader, price, settlement, and card acquisition state
      only from authoritative S2C/snapshot state.
- [ ] **Auction controls are safely gated**: GIVEN the bid would be unaffordable,
      inactive, below minimum, late, or already in flight, WHEN the operator
      activates the control, THEN no invalid optimistic state is displayed and
      either no C2S is sent or the server rejection is surfaced from
      `S2CAuctionBidRejected`.
- [ ] **Placement card select and staging path exists**: GIVEN `PLACEMENT` is
      active and the hand contains playable cards, WHEN the operator clicks a
      card, drags a card, or keyboard-selects a card and chooses a valid target,
      THEN the client creates only local pending placement/ghost feedback,
      validates visible affordances against current authoritative hand,
      economy, spawn range, and board state, and does not create committed board
      units.
- [ ] **Placement unstage path exists**: GIVEN one or more cards are staged
      locally, WHEN the operator clicks a staged ghost, activates an unstage
      control, or drags the ghost back to the fan, THEN the local pending
      placement is removed, reserve/current mana preview updates locally, and
      no server-owned state changes.
- [ ] **Placement submit path exists**: GIVEN `PLACEMENT` is active, WHEN the
      operator activates Submit by click or keyboard while the staged set is
      valid or intentionally empty, THEN the client sends exactly one
      `C2SSubmitPlacement`, locks only the local submit affordance, and waits
      for `S2CPlacementReveal` or later snapshot/phase state before presenting
      committed placement or resolution.
- [ ] **Placement submit remains correct after correction**: GIVEN submit
      pre-validation blocks an overdrawn or invalid staged set, WHEN the
      operator unstages, adjusts reserve/current mana, or chooses a valid
      target, THEN the error clears from local validation and a later valid
      Submit sends only the corrected `C2SSubmitPlacement`.
- [ ] **Result and Return to Lobby are dependency-gated**: GIVEN the manual route
      reaches `GAME_OVER` before the result contract is implemented, WHEN
      evidence is written, THEN the story records an explicit dependency on
      [Presentation Story 006 Result Screen MVP](../presentation-layer/story-006-result-screen-mvp.md)
      and its result acknowledgement/data contract blockers instead of claiming
      Return to Lobby completion. If that dependency is resolved before this
      story is implemented, Return to Lobby must follow the accepted result
      contract, send or rely on `C2SAcknowledgeResult` only as specified there,
      and return local UI to the lobby/menu flow without optimistic server-owned
      teardown.
- [ ] **No client-side optimistic authority is introduced**: GIVEN any local
      create, join, class, purchase, refresh, ready, retract, bid, stage,
      unstage, submit, result, or return action, WHEN server-owned visible state
      changes, THEN the change is driven by S2C/snapshot state and not by local
      mutation of authoritative room, phase, hand, shop, auction, economy,
      placement, result, or teardown state.
- [ ] **Native two-client manual smoke path is documented**: GIVEN one real local
      server and two real native primary clients from the same commit, WHEN the
      operator follows the evidence script, THEN artifacts record launch
      commands, commit, port, host and joiner logs, screenshots or captures,
      actions taken, C2S/S2C observations where available, endpoint reached, and
      exact blockers.
- [ ] **Manual evidence has the correct claim boundary**: GIVEN the native
      two-client manual smoke evidence is reviewed, WHEN scope language is
      inspected, THEN it states that the evidence is internal friend-game
      operator evidence only and not public release readiness, broad
      accessibility completion, playtest validation, fun-hypothesis validation,
      full playable-client manual QA, or full game completion.
- [ ] **Regression commands pass or blockers are explicit**:
      `cargo test -p client --test playable_client_lobby_entry_test`,
      `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`,
      `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`,
      `cargo test -p client --test shop_auction_ui_shop_panel_test`,
      `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test`,
      `cargo test -p client --test hand_ui_placement_submit_core_test`,
      `cargo test -p client --test hand_ui_placement_unstaging_test`,
      `cargo test -p client --test hand_ui_placement_timer_test`,
      `cargo test -p server --test playable_client_real_e2e_loop_test`,
      `cargo test -p server --test playable_client_friend_game_result_endpoint_test`,
      `cargo check --workspace`, and `git diff --check` pass, or exact failing
      command output is recorded as a blocker.
- [ ] **Evidence document exists**:
      `production/qa/evidence/native-friend-game-operator-controls.md` records
      the native operator controls audit, manual two-client route, command
      results, screenshots/captures, defects, result dependency disposition, and
      all Sprint 8 carried conditions and non-claims.

## Likely Files Touched

- `client/src/ui/lobby.rs`
- `client/src/ui/shared.rs`
- `client/src/ui/shop_auction/mod.rs`
- `client/src/ui/hand/mod.rs`
- `client/src/presentation/board_rendering.rs`
- `client/src/presentation/mod.rs`
- `client/src/state/mod.rs`
- `client/src/network/mod.rs`
- `client/src/main.rs`
- `client/Cargo.toml`
- `server/Cargo.toml`
- `tests/integration/playable_client/native_operator_controls_test.rs`
- `tests/integration/playable_client/lobby_entry_test.rs`
- `tests/integration/playable_client/draft_shop_hand_bridge_test.rs`
- `tests/integration/shop_auction_ui/draft_initial_grid_test.rs`
- `tests/integration/shop_auction_ui/shop_panel_test.rs`
- `tests/integration/shop_auction_ui/auction_bid_buttons_test.rs`
- `tests/unit/hand-ui/placement_submit_core_test.rs`
- `tests/integration/hand-ui/placement_unstaging_test.rs`
- `tests/integration/hand-ui/placement_timer_test.rs`
- `tests/integration/playable_client/real_e2e_loop_test.rs`
- `tests/integration/playable_client/friend_game_result_endpoint_test.rs`
- `production/qa/evidence/native-friend-game-operator-controls.md`
- `production/qa/evidence/captures/native-friend-game-operator-controls/`

Server source files are not expected for this story unless native evidence
exposes an existing server-side rejection, validation, or result-contract bug.
Result acknowledgement, GAME_OVER reconnect payload, and Return to Lobby server
cleanup remain owned by the Presentation Story 006 result contract blockers or
the future story file that replaces the Sprint 9 draft contract rows.

## Implementation Notes

- Start by writing a native operator-control test around the actual Bevy
  `Interaction`, focus, and input resources instead of adding more debug-only
  key paths.
- Treat text input focus as an explicit UI mode. Digits and letters must not be
  global commands while the room-code field is active.
- Prefer visible native Bevy UI controls for Create Room, Join Room, requested
  slot, class selection, Confirm, purchase, refresh, Ready/Retract, bid,
  stage/unstage, and Submit. Keyboard accelerators may exist, but they cannot
  be the only operator path.
- Local pending state is allowed only for interaction feedback: focused,
  pressed, dragging, pending send, staged ghost, validation error, and submitted
  affordance. It must be visually distinct from authoritative state.
- Keep lobby commands in `LobbyCommand` or a similarly testable command layer,
  but ensure mouse/button activation writes the same command path as keyboard
  activation.
- Reuse existing Shop/Auction UI and Hand UI message types where possible:
  `ShopAuctionDraftReadyButtonClicked`, `ShopAuctionShopSlotClicked`,
  `ShopAuctionShopRefreshClicked`, `ShopAuctionShopReadyButtonClicked`,
  `ShopAuctionBidButtonClicked`, `HandFanCardClicked`, `GhostClickedEvent`,
  and `HandSubmitButtonClicked`.
- Do not add a second production receiver for `S2CPhaseChanged`,
  `S2CGoldUpdate`, shop messages, auction messages, or result messages. Read
  existing resources and message-drain outputs.
- Keep placement staging local until `C2SSubmitPlacement`; keep committed board
  state tied to `S2CPlacementReveal`, resolution messages, or snapshots.
- If manual evidence reaches `GAME_OVER` before the result contract is ready,
  record the dependency and stop the route cleanly rather than adding an
  ad-hoc local return flow.
- Preserve S8-QA-001-W1 honestly. This story is intended to close or narrow the
  native operator evidence gap later, but this docs-prep branch does not close
  it.

## Performance Budget

Native operator controls must preserve ADR-021 guardrails: presentation
steady-state below 1 ms per frame and phase-boundary spikes below 3 ms. Input
focus, hover, and button-state updates should be O(number of visible controls)
for the current surface and must not spawn/despawn steady-state UI every frame.
Manual evidence capture may record subjective stutter, but this story is not a
full performance profiling pass.

## QA Test Cases

- **Lobby room-code focus**
  - Given: the room-code input has focus.
  - When: digits `0` through `3` are typed.
  - Then: the room code changes and requested slot does not.

- **Lobby create and join buttons**
  - Given: host and joiner native clients are connected.
  - When: host clicks Create Room and joiner clicks Join Room with the shown
    code and slot.
  - Then: real C2S messages are sent and confirmed room state changes only from
    S2C.

- **Class select and confirm**
  - Given: both players are in the lobby.
  - When: each player selects and confirms a class by native UI controls.
  - Then: class lock and reveal state converge from S2C messages and the route
    enters `DRAFT_INITIAL`.

- **Draft initial operator path**
  - Given: `DRAFT_INITIAL` is active.
  - When: each operator buys an offered card and clicks Ready.
  - Then: purchase, hand, economy, ready, and next phase observations come from
    authoritative S2C/snapshot state.

- **Shop and auction operator path**
  - Given: `DRAFT_SHOP` or `DRAFT_AUCTION` is active.
  - When: operators buy, refresh, ready/retract, and bid through native
    controls.
  - Then: no action requires debug-only handlers and every committed result is
    S2C-driven.

- **Placement operator path**
  - Given: `PLACEMENT` is active with at least one playable card.
  - When: operators stage, unstage, correct, and submit placements through
    click/drag or keyboard controls.
  - Then: local staging is reversible and committed placement appears only
    after `S2CPlacementReveal` or later authoritative state.

- **Result dependency**
  - Given: the route reaches `GAME_OVER`.
  - When: result contract support is absent.
  - Then: evidence records the dependency on Presentation Story 006 result
    contract blockers and does not claim Return to Lobby completion.

- **Native two-client evidence**
  - Given: a local server and two native clients run from the same commit.
  - When: the manual script is executed.
  - Then: artifacts record the exact endpoint and every blocker without
    claiming public release readiness, full manual QA, playtest validation, or
    full game completion.

## Test Evidence

**Story Type**: Integration

**Required automated test target**:

- `tests/integration/playable_client/native_operator_controls_test.rs`
  - Registered as `playable_client_native_operator_controls_test`
  - Command: `cargo test -p client --test playable_client_native_operator_controls_test`

**Required manual native evidence document**:

- `production/qa/evidence/native-friend-game-operator-controls.md`

**Required capture artifact directory**:

- `production/qa/evidence/captures/native-friend-game-operator-controls/`

**Required regression commands**:

- `cargo test -p client --test playable_client_lobby_entry_test`
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`
- `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- `cargo test -p client --test shop_auction_ui_shop_panel_test`
- `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test`
- `cargo test -p client --test hand_ui_placement_submit_core_test`
- `cargo test -p client --test hand_ui_placement_unstaging_test`
- `cargo test -p client --test hand_ui_placement_timer_test`
- `cargo test -p server --test playable_client_real_e2e_loop_test`
- `cargo test -p server --test playable_client_friend_game_result_endpoint_test`
- `cargo check --workspace`
- `git diff --check`

**Final evidence expectations**:

- Exact commit, branch, build target, OS, native window size, local server port,
  and command summary.
- Confirmation that both native clients used the same build and a real local
  Lightyear server.
- Host and joiner action log from launch through the reached endpoint.
- Server, host-client, and joiner-client logs or summaries.
- Room-code focus and requested-slot separation evidence.
- Create Room and Join Room button evidence.
- Class selection and confirmation evidence.
- `DRAFT_INITIAL` purchase and Ready/Retract evidence.
- `DRAFT_SHOP` purchase, refresh, Ready/Retract evidence.
- Auction bid control evidence.
- Placement stage, unstage, correction, and submit evidence.
- Result/Return to Lobby dependency disposition if `GAME_OVER` is reached.
- No-optimistic-authority statement with examples checked.
- Defect table with severity, likely owner/system, workaround, and internal
  friend-game impact.
- Explicit non-claims for public release readiness, broad accessibility
  completion, playtest validation, fun-hypothesis validation, full
  playable-client manual QA, and full game completion.
- QA-COND-0005 and QA-COND-0006 impact statement.

**Status**: [ ] Not yet implemented or captured.

## Dependencies

- Depends on: [Story 001 Primary Client Bootstrap + Fresh Lobby Entry](story-001-primary-client-bootstrap-fresh-lobby-entry.md) - Complete.
- Depends on: [Story 002 Live Draft/Shop/Hand Bridge](story-002-live-draft-shop-hand-bridge.md) - Complete.
- Depends on: [Story 003 Real End-to-End Loop Verification](story-003-real-end-to-end-loop-verification.md) - Complete.
- Depends on: [Story 004 Friend-Game Result Endpoint Expansion](story-004-friend-game-result-endpoint-expansion.md) - Complete.
- Depends on: [Story 005 DRAFT_SHOP / Auction / Placement / Resolution Loop Polish](story-005-draft-shop-auction-placement-resolution-loop-polish.md) - Complete.
- Depends on: existing Hand UI, Shop/Auction UI, HUD, Board Rendering,
  Presentation Layer, Game Session, Network Protocol, Card Acquisition,
  Auction, Placement, RSM, and Economy behavior on `main`.
- Result/Return to Lobby dependency: [Presentation Story 006 Result Screen MVP](../presentation-layer/story-006-result-screen-mvp.md) and its result
  acknowledgement/data contract blockers. This dependency is satisfied only for
  the acceptance criterion that allows explicit dependency recording; it is not
  satisfied for a completed Return to Lobby implementation until the result
  contract blockers are resolved or accepted as MVP fallbacks.
- Sprint 9 draft rows `S9-RS-001` and `S9-RS-003` are planning rows only until
  Sprint 9 is explicitly activated and story files are created. This story must
  not treat those draft rows as active sprint state.

## Blockers

None for docs readiness. Result/Return to Lobby implementation is
dependency-gated as described above.

## Readiness Notes

**Implementation readiness verdict**: READY for a future explicitly activated
implementation scope.

This story is self-contained for native operator controls and evidence. It can
be assigned later without changing Sprint 8 status or activating Sprint 9.
Implementation must preserve the carried Sprint 8 conditions and must not claim
that this docs-prep branch closed S8-QA-001-W1, QA-COND-0005, or QA-COND-0006.
