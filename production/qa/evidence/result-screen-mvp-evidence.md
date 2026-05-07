# Result Screen MVP Evidence

Story: production/epics/presentation-layer/story-006-result-screen-mvp.md
Branch: work/result-screen-mvp
Implementation commit: work/result-screen-mvp HEAD
Date: 2026-05-07

## MVP Claims

- GAME_OVER opens a visible result overlay above the existing board/HUD surface.
- Result copy is derived from server-authored `S2CGameOver` data: victory, defeat, draw, no-result, disconnect, and missing-payload fallback are covered.
- Objective rows render five own lanes and five opponent lanes from the authoritative game snapshot.
- Own objective identities use the local player's private objective snapshot.
- Opponent objective identities remain `Unknown` while alive unless a server-authored reveal exists; destroyed opponent objectives use `was_fake` only when present.
- Return to Lobby sends `C2SAcknowledgeResult` and moves the client back to `ClientState::Lobby`.
- Rematch is not spawned, enabled, or wired.
- Keyboard MVP behavior is covered: Return to Lobby is the only focus target; Escape focuses it without exiting; Enter activates it.
- Reduced motion disables entry duration, row sequencing, and repeated flashes.

## HUD/Background Preservation

The overlay is spawned as a separate presentation UI root with `GlobalZIndex(100)` and a translucent background. It does not mutate HUD resources, HUD mode, scoreboard dots, gold state, or the existing GAME_OVER freeze path. Existing `hud_game_over_freeze_test` remains the regression gate for frozen HUD behavior.

## Objective Privacy Notes

Alive opponent rows render `Unknown` identity when no server-authoritative reveal is present. Fallback opponent snapshot data may supply HP/destroyed state for the summary, but it does not reveal real/fake identity.

## Layout/Accessibility Notes

The MVP uses bounded Bevy UI nodes: the panel is capped at 88 percent width, max 860 px, max 92 percent height, with wrapping objective columns and a fixed-size Return to Lobby button. The evidence is automated/unit level only; no browser/manual viewport claim is made for 1366x768, 1920x1080, or 150 percent UI scale.

## Validation

| Command | Result |
| --- | --- |
| `cargo test -p client --test result_screen_mvp_test` | PASS |
| `cargo test -p client --test hud_game_over_freeze_test` | PASS |
| `cargo test -p server --test playable_client_friend_game_result_endpoint_test` | PASS |
| `cargo check -p client` | PASS |
| `cargo fmt -p client -p server -- --check` | PASS |
| `git diff --check origin/main...HEAD` | PASS after worker commit |

## Explicit Non-Claims

- No rematch protocol.
- No broad post-game reveal.
- No playtest validation.
- No public release readiness.
- No full game completion claim.
- Sprint 8 carried conditions remain carried unless separately closed.
- Sprint 9 no-claims remain no-claims unless separately scoped.
