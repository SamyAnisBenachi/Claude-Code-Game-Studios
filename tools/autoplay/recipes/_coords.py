"""Coordinate defaults for phase recipes (PROMPT 1609).

The autoplay status surface deliberately does NOT expose UI element
geometry — recipes cannot ask the client where the lobby Confirm
button is. To stay implementation-oriented but layout-tolerant,
each interactive recipe uses a default fractional coordinate that
can be overridden by an environment variable.

Override format:
    CCGS_AUTOPLAY_<KEY>=<fx>,<fy>

both values are floats in ``[0.0, 1.0]`` interpreted against the
client's logical window size. Setting the env var to anything that
fails to parse falls back to the default and the recipe writes a
``local.note`` row recording the parse failure.

The defaults below were chosen from the existing
``client/src/ui/**`` layouts (centre column / lower-third action
panel). They are not load-bearing — VERIFY-lane operators override
them when the live layout drifts.
"""

from __future__ import annotations

from typing import NamedTuple


class FracPoint(NamedTuple):
    fx: float
    fy: float


# Defaults: fractional logical-window positions.
DEFAULTS: dict[str, FracPoint] = {
    "LOBBY_CREATE_BTN": FracPoint(0.5, 0.55),
    "LOBBY_ADD_BOT_BTN": FracPoint(0.5, 0.72),
    "LOBBY_CONFIRM_BTN": FracPoint(0.5, 0.85),
    "CLASS_FIRST_CARD": FracPoint(0.25, 0.45),
    "CLASS_CONFIRM_BTN": FracPoint(0.5, 0.85),
    "SHOP_FIRST_SLOT": FracPoint(0.30, 0.45),
    "SHOP_CONFIRM_BTN": FracPoint(0.5, 0.85),
    "AUCTION_BID_BTN": FracPoint(0.5, 0.55),
    "AUCTION_READY_BTN": FracPoint(0.5, 0.85),
    "HAND_FIRST_CARD": FracPoint(0.35, 0.92),
    "BOARD_FIRST_CELL": FracPoint(0.5, 0.55),
    "SUBMIT_BTN": FracPoint(0.85, 0.92),
}


def resolve(key: str, env: dict[str, str]) -> tuple[FracPoint, str | None]:
    """Look up the fractional coordinate for ``key``.

    Returns ``(point, note)`` where ``note`` is a human-readable
    diagnostic if the env override could not be parsed (``None`` on
    clean default or successful override).
    """
    default = DEFAULTS[key]
    env_key = f"CCGS_AUTOPLAY_{key}"
    raw = env.get(env_key)
    if raw is None or raw.strip() == "":
        return default, None
    try:
        parts = raw.split(",")
        if len(parts) != 2:
            raise ValueError(f"expected 'fx,fy', got {raw!r}")
        fx = float(parts[0])
        fy = float(parts[1])
        if not (0.0 <= fx <= 1.0 and 0.0 <= fy <= 1.0):
            raise ValueError("fractions must be in [0,1]")
        return FracPoint(fx, fy), None
    except ValueError as err:
        return default, f"{env_key} parse failed ({err}); using default {default}"
