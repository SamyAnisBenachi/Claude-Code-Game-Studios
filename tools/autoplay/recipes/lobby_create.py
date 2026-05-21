"""``lobby-create`` -- drive the lobby Create + Confirm path.

Steps (UI-driven, no semantic verbs):
  1. wait for status (handled by driver before recipes start);
  2. checkpoint ``lobby-loaded`` (screenshot);
  3. click the Create button (overridable via CCGS_AUTOPLAY_LOBBY_CREATE_BTN);
  4. wait a short settling window;
  5. click Confirm (overridable via CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN);
  6. checkpoint ``lobby-confirmed`` (screenshot).

If the live layout has moved (the defaults are centre-column / lower-
third), override the two env vars rather than editing this file.
"""

from __future__ import annotations

from ._builder import RecipeBuilder
from ._coords import resolve

NAME = "lobby-create"
DESCRIPTION = "Lobby flow: click Create, wait, click Confirm. Two checkpoints (loaded, confirmed)."


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)
    b.checkpoint("lobby-loaded")

    create, create_note = resolve("LOBBY_CREATE_BTN", ctx.env)
    if create_note:
        b.note(create_note)
    cx, cy = b.frac(create.fx, create.fy)
    b.click(cx, cy)
    b.wait(8)  # let the confirm panel mount

    confirm, confirm_note = resolve("LOBBY_CONFIRM_BTN", ctx.env)
    if confirm_note:
        b.note(confirm_note)
    fx, fy = b.frac(confirm.fx, confirm.fy)
    b.click(fx, fy)
    b.wait(4)

    b.checkpoint("lobby-confirmed")
    b.clear_input()
    return b.build()
