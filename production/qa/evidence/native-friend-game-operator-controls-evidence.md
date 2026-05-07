# Native Friend-Game Operator Controls Evidence

Date: 2026-05-07
Branch: `work/native-friend-game-operator-controls`
Source baseline: `origin/main@836899c`
Story: `production/epics/playable-client/story-006-native-friend-game-operator-controls.md`

## Scope

This evidence covers the Sprint 9 native operator-controls implementation for
the internal friend-game route. It verifies that lobby, class, draft, shop,
auction, and placement controls are exposed through operator-driven keyboard or
Bevy UI interactions while preserving server authority.

This does not claim public release readiness, full playable-client manual QA,
playtest validation, broad accessibility completion, full game completion, or a
fully captured manual two-client route to `GAME_OVER`.

## Implementation Summary

- Lobby input now separates room-code text focus from command shortcuts.
  When the room-code field is focused, `J` and `0`/`1`/`2`/`3` type into the
  room code. When it is not focused, `J` joins and digits change requested slot.
- Lobby UI exposes explicit Bevy buttons for room-code focus, create room, join
  room, requested slot, class selection, and class confirmation.
- Draft initial and draft shop slot, ready/retract, and refresh controls now
  emit the existing outbound operator intents through real `Interaction`
  presses.
- Auction bid buttons remain routed through the existing bid path.
- Hand fan/grid/submit controls now emit existing placement messages through
  real `Interaction` presses. Clicking an active hand card during placement
  stages a conservative default target when the card type has a safe default;
  server-side validation remains authoritative.
- Result screen controls were not broadened in this story. Result endpoint
  behavior remains covered by the existing server endpoint test.

## Automated Verification

| Command | Result |
|---|---|
| `cargo test -p client --test playable_client_native_operator_controls_test` | PASS, 4 passed |
| `cargo test -p client --test playable_client_lobby_entry_test` | PASS, 6 passed |
| `cargo test -p client --test playable_client_draft_shop_hand_bridge_test` | PASS, 5 passed |
| `cargo test -p client --test playable_client_active_loop_ui_state_test` | PASS, 4 passed |
| `cargo test -p server --test playable_client_friend_game_result_endpoint_test` | PASS, 1 passed |
| `cargo check -p client` | PASS |
| `cargo fmt -p client -p server -- --check` | PASS |
| `git diff --check` | PASS before commit |

## Native Launch Sanity

Native command shape used:

```powershell
$env:SERVER_PORT='5097'
target\debug\server.exe

$env:SERVER_URL='ws://localhost:5097'
target\debug\client.exe
target\debug\client.exe
```

Observed result:

- One server process and two client processes stayed alive for the sanity
  window.
- No server, client A, or client B stderr/stdout panic output was produced.
- A second probe on port `5098` confirmed both client processes stayed alive
  and responding.

Visual inspection limit:

- This Codex execution context did not expose native window handles for the
  Bevy client processes (`MainWindowHandle = 0` for both clients), so this run
  cannot honestly claim that the visible lobby was observed through a native
  desktop window.
- The operator-control surface itself is verified by
  `playable_client_native_operator_controls_test`, which presses the Bevy UI
  controls directly and checks the resulting outbound intents.

Disposition: partial native launch sanity. Runtime process startup was stable,
but visual native-window confirmation remains a manual QA follow-up.
