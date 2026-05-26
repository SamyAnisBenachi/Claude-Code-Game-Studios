"""Autoplay recipe library (PROMPT 1609).

Each recipe is a small Python module that exposes:

    NAME: str
    DESCRIPTION: str
    def build(ctx: RecipeContext) -> list[dict]: ...

`build` returns a flat list of action dicts shaped
``{"tick": <int>, "method": <str>, "params": <dict>}`` ordered by tick.
Recipes MUST only emit:

* the low-level autoplay RPC methods (``autoplay/input``,
  ``autoplay/clear_input``, ``autoplay/screenshot``, ``autoplay/status``,
  ``autoplay/capabilities``); OR
* the driver-local pseudo-methods (``local.checkpoint``, ``local.note``,
  ``local.block``) — these never hit the RPC server; the driver consumes
  them directly.

The contract is enforced in `tools/autoplay/driver.py` via the
``ALLOWED_METHODS`` allowlist.

Adding a recipe:
    1. Create ``tools/autoplay/recipes/<slug>.py`` with ``NAME``,
       ``DESCRIPTION``, and ``build(ctx)``.
    2. Import it from ``REGISTRY`` below.
    3. Re-run ``python tools/autoplay/driver.py --list-recipes`` to
       confirm it appears.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

from . import (
    add_bot_lobby,
    class_select,
    draft_auction_probe,
    full_game,
    idle,
    lobby_create,
    placement_drag_probe,
    smoke,
)


@dataclass(frozen=True)
class RecipeContext:
    """State passed to each recipe's ``build`` callable.

    ``window_size`` is the latest ``window_logical_size`` returned by
    ``autoplay/status``; recipes use it to express coordinates as
    fractions of the visible area rather than hard-coding pixels.
    ``env`` exposes the process environment so recipes can detect
    upstream prerequisites (e.g. PROMPT 1607 bot-vs-bot soak room
    landed) without reaching for ``os`` directly.
    """

    window_size: tuple[float, float]
    env: dict[str, str]


Builder = Callable[[RecipeContext], list[dict]]


REGISTRY: dict[str, tuple[str, Builder]] = {
    smoke.NAME: (smoke.DESCRIPTION, smoke.build),
    idle.NAME: (idle.DESCRIPTION, idle.build),
    add_bot_lobby.NAME: (add_bot_lobby.DESCRIPTION, add_bot_lobby.build),
    lobby_create.NAME: (lobby_create.DESCRIPTION, lobby_create.build),
    class_select.NAME: (class_select.DESCRIPTION, class_select.build),
    draft_auction_probe.NAME: (draft_auction_probe.DESCRIPTION, draft_auction_probe.build),
    placement_drag_probe.NAME: (placement_drag_probe.DESCRIPTION, placement_drag_probe.build),
    full_game.NAME: (full_game.DESCRIPTION, full_game.build),
}


def names() -> list[str]:
    return sorted(REGISTRY.keys())


def get(name: str) -> tuple[str, Builder]:
    return REGISTRY[name]
