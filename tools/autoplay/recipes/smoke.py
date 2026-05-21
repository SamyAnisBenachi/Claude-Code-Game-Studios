"""``smoke`` -- minimal substrate probe (legacy from PROMPT 1595).

Preserved for backwards compatibility with `Run-AutoplaySmoke.ps1`.
"""

from __future__ import annotations

from ._builder import RecipeBuilder

NAME = "smoke"
DESCRIPTION = "Single input frame, clear, screenshot. Proves the RPC substrate."


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)
    cx, cy = b.centre()
    b._emit("autoplay/input", {
        "keys_down": ["KeyA"],
        "mouse_down": ["Left"],
        "cursor": {"screen": [cx, cy]},
    })
    b._next()
    b.clear_input()
    b.screenshot("smoke-driver")
    return b.build()
