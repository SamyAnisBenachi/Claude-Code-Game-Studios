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
  -> ``placement-drag-probe`` -> resolution observation
  -> (optionally) game-over observation.

Post-placement observation (PROMPT 1641):
  By default the recipe chains a resolution-observe soak after
  placement to produce screenshot evidence of the combat/resolution
  phase without requiring a complete game. Opt-out by setting
  ``CCGS_AUTOPLAY_FULL_GAME_RESOLUTION=0``.

  GameOver observation is opt-in (off by default) because reaching
  GameOver requires natural HP drain which is not yet bounded by a
  server-side max-rounds flag. Enable with
  ``CCGS_AUTOPLAY_FULL_GAME_GAMEOVER=1``. This extends the run
  substantially; use only when a full game is expected to complete.

Env gates summary:
  CCGS_AUTOPLAY_BOT_ROOM_READY          – must be "1" to unblock recipe
  CCGS_AUTOPLAY_FULL_GAME_RESOLUTION    – "0" to skip resolution soak (default: on)
  CCGS_AUTOPLAY_FULL_GAME_GAMEOVER      – "1" to chain game-over soak (default: off)
  CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS   – soak length for resolution phase (default 60)
  CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS     – soak length before game-over screen (default 120)
  CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS – extra soak for winner anim (default 30)
"""

from __future__ import annotations

from . import (
    class_select,
    draft_auction_probe,
    game_over_observe,
    lobby_create,
    placement_drag_probe,
    resolution_observe,
)
from ._builder import RecipeBuilder, flatten

NAME = "full-game"
DESCRIPTION = (
    "Composite recipe (lobby -> class -> draft/auction -> placement -> resolution soak). "
    "Requires PROMPT 1607 bot-vs-bot soak room; emits BLOCKED otherwise. "
    "Resolution observation on by default; GameOver opt-in via "
    "CCGS_AUTOPLAY_FULL_GAME_GAMEOVER=1."
)

BOT_ROOM_ENV = "CCGS_AUTOPLAY_BOT_ROOM_READY"
RESOLUTION_ENV = "CCGS_AUTOPLAY_FULL_GAME_RESOLUTION"
GAMEOVER_ENV = "CCGS_AUTOPLAY_FULL_GAME_GAMEOVER"

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

    want_resolution = ctx.env.get(RESOLUTION_ENV, "1") != "0"
    want_gameover = ctx.env.get(GAMEOVER_ENV, "0") == "1"

    # Stitch the sub-recipes back-to-back with monotonically increasing ticks.
    streams: list[list[dict]] = []
    cursor_tick = 1

    core_phases = [
        ("lobby", lobby_create.build(ctx)),
        ("class", class_select.build(ctx)),
        ("draft-auction", draft_auction_probe.build(ctx)),
        ("placement", placement_drag_probe.build(ctx)),
    ]

    if want_resolution:
        core_phases.append(("resolution", resolution_observe.build(ctx)))

    if want_gameover:
        core_phases.append(("game-over", game_over_observe.build(ctx)))

    for label, actions in core_phases:
        if not actions:
            continue
        first = actions[0]["tick"]
        offset = cursor_tick - first
        shifted = [{**a, "tick": a["tick"] + offset} for a in actions]
        streams.append(shifted)
        cursor_tick = shifted[-1]["tick"] + 4  # 4-tick settling gap between phases

    # Final composite checkpoint — summarises whatever phases ran.
    tail = RecipeBuilder(ctx.window_size, start_tick=cursor_tick)
    if want_gameover:
        tail.checkpoint("full-game-complete")
    elif want_resolution:
        tail.checkpoint("full-game-post-resolution")
    else:
        tail.checkpoint("full-game-post-placement")
    tail.clear_input()
    streams.append(tail.build())

    return flatten(*streams)
