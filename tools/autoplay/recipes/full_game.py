"""``full-game`` -- compose every phase recipe into one driven run.

The recipe assumes a bot peer is present so the driven client can
progress through DraftAuction / Placement / Resolution without a
second human. The bot peer is provided by PROMPT 1607's
``Start-BotVsBotSoak.ps1`` /  ``--bot-vs-bot-max-rounds`` lane.
That lane is NOT yet on main as of PROMPT 1609.

Detection contract:
  * Set ``CCGS_AUTOPLAY_BOT_ROOM_READY=1`` when the bot soak room is
    live and the autoplay-driven client is being launched against it.
  * If the env var is missing or not ``1``, this recipe writes a
    ``local.block`` row and the driver exits with code 4. The block
    row points the reader at PROMPT 1607 so they can either land the
    upstream prompt, run a bot-less sub-recipe (``lobby-create`` /
    ``class-select`` work without a bot), or pass the env var when
    the room has landed locally.

When unblocked, the recipe executes:
  ``lobby-create`` -> ``class-select`` -> ``draft-auction-probe``
  -> ``placement-drag-probe`` -> final resolution checkpoint.
"""

from __future__ import annotations

from . import class_select, draft_auction_probe, lobby_create, placement_drag_probe
from ._builder import RecipeBuilder, flatten

NAME = "full-game"
DESCRIPTION = "Composite recipe (lobby -> class -> draft/auction -> placement). Requires PROMPT 1607 bot-vs-bot soak room; emits BLOCKED otherwise."

BOT_ROOM_ENV = "CCGS_AUTOPLAY_BOT_ROOM_READY"
BLOCK_HINT = (
    "Set CCGS_AUTOPLAY_BOT_ROOM_READY=1 after launching `Start-BotVsBotSoak.ps1` "
    "(PROMPT 1607). Until that lane lands, run lobby-create / class-select / "
    "draft-auction-probe / placement-drag-probe individually against a human peer."
)


def build(ctx) -> list[dict]:
    if ctx.env.get(BOT_ROOM_ENV) != "1":
        b = RecipeBuilder(ctx.window_size)
        b.checkpoint("full-game-precheck", screenshot=False)
        b.block(
            reason=(
                "PROMPT 1607 bot-vs-bot soak room not advertised "
                f"({BOT_ROOM_ENV} != '1'); full-game cannot drive a peer-less client "
                "past DraftAuction without it."
            ),
            hint=BLOCK_HINT,
        )
        return b.build()

    # Stitch the sub-recipes back-to-back with monotonically increasing ticks.
    streams: list[list[dict]] = []
    cursor_tick = 1
    sub_builders = [
        ("lobby", lobby_create.build(ctx)),
        ("class", class_select.build(ctx)),
        ("draft-auction", draft_auction_probe.build(ctx)),
        ("placement", placement_drag_probe.build(ctx)),
    ]
    for label, actions in sub_builders:
        if not actions:
            continue
        first = actions[0]["tick"]
        offset = cursor_tick - first
        shifted = [{**a, "tick": a["tick"] + offset} for a in actions]
        streams.append(shifted)
        cursor_tick = shifted[-1]["tick"] + 4  # 4-tick settling gap between phases

    # Final composite checkpoint.
    tail = RecipeBuilder(ctx.window_size, start_tick=cursor_tick)
    tail.checkpoint("full-game-resolution")
    tail.clear_input()
    streams.append(tail.build())

    return flatten(*streams)
