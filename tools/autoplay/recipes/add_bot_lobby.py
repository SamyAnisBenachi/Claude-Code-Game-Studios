"""``add-bot-lobby`` -- drive the lobby Create + Add Bot + Confirm path.

Steps (UI-driven, no semantic verbs):
  1. check CCGS_DEBUG_UI=1; emit ``local.block`` and exit if unset --
     the Add Bot control is a debug-only affordance (PROMPT 1603);
  2. checkpoint ``lobby-loaded`` (screenshot);
  3. click the Create button (overridable via CCGS_AUTOPLAY_LOBBY_CREATE_BTN);
  4. wait a settling window for the bot-controls container to mount;
  5. click Add Bot (overridable via CCGS_AUTOPLAY_LOBBY_ADD_BOT_BTN);
  6. wait for the server round-trip to seat the bot;
  7. checkpoint ``bot-added`` (screenshot);
  8. click Confirm (overridable via CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN);
  9. checkpoint ``lobby-confirmed`` (screenshot).

The Add Bot control is only rendered when ``CCGS_DEBUG_UI=1`` is present in
the process environment (client/src/ui/lobby.rs ``is_debug_ui_enabled``).
If that var is absent the recipe emits ``local.block`` rather than clicking
a button that does not exist.

Override any coordinate via the matching env var (``fx,fy`` in [0,1]):
  CCGS_AUTOPLAY_LOBBY_CREATE_BTN
  CCGS_AUTOPLAY_LOBBY_ADD_BOT_BTN
  CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN
"""

from __future__ import annotations

from ._builder import RecipeBuilder
from ._coords import resolve

NAME = "add-bot-lobby"
DESCRIPTION = (
    "Lobby flow: click Create, click Add Bot, click Confirm. "
    "Requires CCGS_DEBUG_UI=1; emits BLOCKED otherwise. "
    "Three checkpoints (lobby-loaded, bot-added, lobby-confirmed)."
)

_DEBUG_UI_ENV = "CCGS_DEBUG_UI"
_BLOCK_HINT = (
    "Set CCGS_DEBUG_UI=1 when launching the client to expose the Add Bot "
    "control (client/src/ui/lobby.rs `is_debug_ui_enabled`). "
    "Without it the Add Bot button is never spawned and this recipe cannot "
    "seat a bot. Use the `lobby-create` recipe for a human-vs-human lobby."
)


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)

    # Guard: Add Bot UI is debug-gated; block early rather than clicking air.
    if ctx.env.get(_DEBUG_UI_ENV) != "1":
        b.checkpoint("add-bot-lobby-precheck", screenshot=False)
        b.block(
            reason=(
                f"Add Bot control requires {_DEBUG_UI_ENV}=1 "
                "(client/src/ui/lobby.rs `is_debug_ui_enabled`); "
                "env var is absent or not '1'."
            ),
            hint=_BLOCK_HINT,
        )
        return b.build()

    b.checkpoint("lobby-loaded")

    create, create_note = resolve("LOBBY_CREATE_BTN", ctx.env)
    if create_note:
        b.note(create_note)
    cx, cy = b.frac(create.fx, create.fy)
    b.click(cx, cy)
    b.wait(8)  # let the room be created and bot-controls container mount

    add_bot, add_bot_note = resolve("LOBBY_ADD_BOT_BTN", ctx.env)
    if add_bot_note:
        b.note(add_bot_note)
    ax, ay = b.frac(add_bot.fx, add_bot.fy)
    b.click(ax, ay)
    b.wait(6)  # let C2SAddBot round-trip complete and slot update propagate

    b.checkpoint("bot-added")

    confirm, confirm_note = resolve("LOBBY_CONFIRM_BTN", ctx.env)
    if confirm_note:
        b.note(confirm_note)
    fx, fy = b.frac(confirm.fx, confirm.fy)
    b.click(fx, fy)
    b.wait(4)

    b.checkpoint("lobby-confirmed")
    b.clear_input()
    return b.build()
