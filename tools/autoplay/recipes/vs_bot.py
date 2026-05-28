"""``vs-bot`` -- composite recipe for Story 004: full game loop via the Add Bot lobby.

Differs from ``full-game`` in exactly one place: the lobby entry point.
``full-game`` uses ``lobby-create`` (human-vs-human wait); ``vs-bot`` uses
``add-bot-lobby`` (Create + Add Bot + Confirm) so the session is immediately
seatable by the server-side bot participant without a second human client.

Phase sequence:
  add-bot-lobby -> class-select -> draft-auction-probe
    -> placement-drag-probe -> resolution-observe
    -> (optionally) game-over-observe

Prerequisites / env gates:
  CCGS_DEBUG_UI=1            -- required by ``add-bot-lobby`` (exposes the Add Bot
                                 button); emits BLOCKED if absent.
  CCGS_AUTOPLAY_BOT_ROOM_READY=1  -- required so the server soak room has a bot
                                 participant to seat; emits BLOCKED if absent.

Optional overrides (same semantics as ``full-game``):
  CCGS_AUTOPLAY_VS_BOT_RESOLUTION   -- "0" to skip resolution soak (default: on)
  CCGS_AUTOPLAY_VS_BOT_GAMEOVER     -- "1" to chain game-over soak (default: off)
  CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS    -- soak length for resolution phase
  CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS      -- soak length before game-over screen
  CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS -- extra soak for winner animation

All coordinate overrides used by the child recipes are honoured unchanged.
"""

from __future__ import annotations

from . import (
    add_bot_lobby,
    class_select,
    draft_auction_probe,
    game_over_observe,
    placement_drag_probe,
    resolution_observe,
)
from ._builder import RecipeBuilder, flatten

NAME = "vs-bot"
DESCRIPTION = (
    "Composite recipe (add-bot-lobby -> class -> draft/auction -> placement -> resolution soak). "
    "Requires CCGS_DEBUG_UI=1 AND CCGS_AUTOPLAY_BOT_ROOM_READY=1; emits BLOCKED otherwise. "
    "Resolution observation on by default; GameOver opt-in via "
    "CCGS_AUTOPLAY_VS_BOT_GAMEOVER=1."
)

_DEBUG_UI_ENV = "CCGS_DEBUG_UI"
_BOT_ROOM_ENV = "CCGS_AUTOPLAY_BOT_ROOM_READY"
_RESOLUTION_ENV = "CCGS_AUTOPLAY_VS_BOT_RESOLUTION"
_GAMEOVER_ENV = "CCGS_AUTOPLAY_VS_BOT_GAMEOVER"

_BLOCK_HINT_DEBUG_UI = (
    "Set CCGS_DEBUG_UI=1 when launching the client to expose the Add Bot button "
    "(client/src/ui/lobby.rs `is_debug_ui_enabled`). "
    "Without it the Add Bot button is never spawned and this recipe cannot seat a bot."
)

_BLOCK_HINT_BOT_ROOM = (
    "Set CCGS_AUTOPLAY_BOT_ROOM_READY=1 after launching `Start-BotVsBotSoak.ps1` "
    "(PROMPT 1607). Without a bot peer the session cannot advance past class-select."
)


def build(ctx) -> list[dict]:
    # Guard 1: Add Bot UI is debug-gated.
    if ctx.env.get(_DEBUG_UI_ENV) != "1":
        b = RecipeBuilder(ctx.window_size)
        b.checkpoint("vs-bot-precheck", screenshot=False)
        b.block(
            reason=(
                f"Add Bot control requires {_DEBUG_UI_ENV}=1 "
                "(client/src/ui/lobby.rs `is_debug_ui_enabled`); "
                "env var is absent or not '1'."
            ),
            hint=_BLOCK_HINT_DEBUG_UI,
        )
        return b.build()

    # Guard 2: bot-vs-bot soak room must be advertised.
    if ctx.env.get(_BOT_ROOM_ENV) != "1":
        b = RecipeBuilder(ctx.window_size)
        b.checkpoint("vs-bot-precheck", screenshot=False)
        b.block(
            reason=(
                f"Bot soak room not advertised ({_BOT_ROOM_ENV} != '1'); "
                "vs-bot cannot drive a peer-less session past class-select."
            ),
            hint=_BLOCK_HINT_BOT_ROOM,
        )
        return b.build()

    want_resolution = ctx.env.get(_RESOLUTION_ENV, "1") != "0"
    want_gameover = ctx.env.get(_GAMEOVER_ENV, "0") == "1"

    # Stitch sub-recipes back-to-back with monotonically increasing ticks.
    streams: list[list[dict]] = []
    cursor_tick = 1

    core_phases = [
        ("add-bot-lobby", add_bot_lobby.build(ctx)),
        ("class", class_select.build(ctx)),
        ("draft-auction", draft_auction_probe.build(ctx)),
        ("placement", placement_drag_probe.build(ctx)),
    ]

    if want_resolution:
        core_phases.append(("resolution", resolution_observe.build(ctx)))

    if want_gameover:
        core_phases.append(("game-over", game_over_observe.build(ctx)))

    for _label, actions in core_phases:
        if not actions:
            continue
        first = actions[0]["tick"]
        offset = cursor_tick - first
        shifted = [{**a, "tick": a["tick"] + offset} for a in actions]
        streams.append(shifted)
        cursor_tick = shifted[-1]["tick"] + 4  # 4-tick settling gap between phases

    # Final composite checkpoint.
    tail = RecipeBuilder(ctx.window_size, start_tick=cursor_tick)
    if want_gameover:
        tail.checkpoint("vs-bot-complete")
    elif want_resolution:
        tail.checkpoint("vs-bot-post-resolution")
    else:
        tail.checkpoint("vs-bot-post-placement")
    tail.clear_input()
    streams.append(tail.build())

    return flatten(*streams)
