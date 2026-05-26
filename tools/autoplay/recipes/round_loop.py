"""``round-loop`` -- drive multiple full rounds via sub-recipe composition.

Stitches existing recipes across N game rounds:
  1. ``full-game`` (lobby → class → draft/auction → placement, round 1)
  2. ``resolution-observe`` (round 1 resolution)
  3. For each additional round k in [2 .. ROUND_LOOP_COUNT]:
       a. ``draft-auction-probe`` (shop + auction)
       b. ``placement-drag-probe`` (placement + submit)
       c. ``resolution-observe`` (resolution)
  4. ``game-over-observe`` (final result screen)

This recipe does NOT drive any inter-round transition UI (e.g. a
"Continue" or "Next Round" button). It relies on the server advancing
the phase automatically after placement-submit. If the server requires a
client click to advance, the subsequent observation windows will show the
wrong phase; bracketing screenshots let a human reviewer detect the
mismatch.

Detection contract:
  ``autoplay/status`` does not expose the current game-phase name; the
  recipe cannot confirm mid-loop phase transitions. Screenshots are the
  primary evidence artefacts for human review.

Prerequisites / blocks:
  * ``CCGS_AUTOPLAY_BOT_ROOM_READY=1`` must be set — same gate as
    ``full-game``. Without a bot peer the lobby never confirms.
  * ``full-game``, ``resolution-observe``, ``game-over-observe``,
    ``draft-auction-probe``, and ``placement-drag-probe`` must all be
    present in the recipe registry (all landed on main as of PROMPT 1636).

Env-var overrides:
    CCGS_AUTOPLAY_BOT_ROOM_READY          (must equal '1')
        Advertise that the bot-vs-bot soak room is running.
    CCGS_AUTOPLAY_ROUND_LOOP_COUNT        (integer >= 1, default 2)
        Total rounds to drive. The first round uses ``full-game``; each
        additional round runs draft-auction-probe → placement-drag-probe
        → resolution-observe. A value of 1 drives only round 1 (same
        coverage as full-game + resolution-observe + game-over-observe).
    CCGS_AUTOPLAY_ROUND_SETTLE_TICKS      (integer >= 1, default 4)
        Gap ticks inserted between sub-recipe phase boundaries to give the
        client time to settle before the next recipe begins.
"""

from __future__ import annotations

from . import (
    draft_auction_probe,
    full_game,
    game_over_observe,
    placement_drag_probe,
    resolution_observe,
)
from ._builder import RecipeBuilder

NAME = "round-loop"
DESCRIPTION = (
    "Multi-round composite: full-game -> resolution-observe x N rounds "
    "-> game-over-observe. "
    "Requires CCGS_AUTOPLAY_BOT_ROOM_READY=1. "
    "Configure loop count via CCGS_AUTOPLAY_ROUND_LOOP_COUNT (default 2)."
)

_BOT_ROOM_ENV = "CCGS_AUTOPLAY_BOT_ROOM_READY"
_LOOP_COUNT_ENV = "CCGS_AUTOPLAY_ROUND_LOOP_COUNT"
_SETTLE_ENV = "CCGS_AUTOPLAY_ROUND_SETTLE_TICKS"
_DEFAULT_LOOP_COUNT = 2
_DEFAULT_SETTLE = 4

_BLOCK_HINT = (
    "Set CCGS_AUTOPLAY_BOT_ROOM_READY=1 after launching "
    "`Start-BotVsBotSoak.ps1` (PROMPT 1607 / PROMPT 1603). "
    "Without a bot peer the lobby never advances past the waiting screen."
)


def _parse_int(b: RecipeBuilder, key: str, default: int, env: dict[str, str]) -> int:
    raw = env.get(key, "").strip()
    if not raw:
        return default
    try:
        parsed = int(raw)
        if parsed < 1:
            raise ValueError("must be >= 1")
        return parsed
    except ValueError as err:
        b.note(f"{key} parse failed ({err}); using default {default}")
        return default


def _shift(actions: list[dict], offset: int) -> list[dict]:
    """Return a copy of *actions* with every tick shifted by *offset*."""
    return [{**a, "tick": a["tick"] + offset} for a in actions]


def _stitch(
    streams: list[list[dict]],
    settle: int,
) -> tuple[list[dict], int]:
    """Concatenate streams with *settle*-tick gaps; return (flat_list, next_tick)."""
    flat: list[dict] = []
    cursor = 1
    for actions in streams:
        if not actions:
            continue
        first = actions[0]["tick"]
        offset = cursor - first
        shifted = _shift(actions, offset)
        flat.extend(shifted)
        cursor = shifted[-1]["tick"] + settle
    return flat, cursor


def build(ctx) -> list[dict]:
    b_pre = RecipeBuilder(ctx.window_size)

    if ctx.env.get(_BOT_ROOM_ENV) != "1":
        b_pre.checkpoint("round-loop-precheck", screenshot=False)
        b_pre.block(
            reason=(
                f"Bot-vs-bot soak room not advertised ({_BOT_ROOM_ENV} != '1'); "
                "round-loop cannot drive past the lobby waiting screen without a bot peer."
            ),
            hint=_BLOCK_HINT,
        )
        return b_pre.build()

    loop_count = _parse_int(b_pre, _LOOP_COUNT_ENV, _DEFAULT_LOOP_COUNT, ctx.env)
    settle = _parse_int(b_pre, _SETTLE_ENV, _DEFAULT_SETTLE, ctx.env)

    # Collect all phase streams in order.
    streams: list[list[dict]] = []

    # Round 1: full-game drives lobby → class → draft/auction → placement.
    streams.append(full_game.build(ctx))

    # Round 1 resolution observation.
    streams.append(resolution_observe.build(ctx))

    # Additional rounds (2 .. loop_count): draft/auction → placement → resolution.
    for round_idx in range(2, loop_count + 1):
        # Checkpoint between rounds (no screenshot — just a timeline marker).
        marker = RecipeBuilder(ctx.window_size)
        marker.checkpoint(f"round-{round_idx}-start", screenshot=False)
        streams.append(marker.build())

        streams.append(draft_auction_probe.build(ctx))
        streams.append(placement_drag_probe.build(ctx))
        streams.append(resolution_observe.build(ctx))

    # Final game-over observation.
    streams.append(game_over_observe.build(ctx))

    flat, next_tick = _stitch(streams, settle)

    # Closing checkpoint + clear.
    tail = RecipeBuilder(ctx.window_size, start_tick=next_tick)
    tail.checkpoint("round-loop-complete")
    tail.clear_input()
    flat.extend(tail.build())

    return flat
