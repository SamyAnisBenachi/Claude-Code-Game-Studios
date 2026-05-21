"""``class-select`` -- pick a class card and confirm.

Steps:
  1. checkpoint ``class-select-loaded``;
  2. click the first class card slot (CCGS_AUTOPLAY_CLASS_FIRST_CARD);
  3. wait for the selection highlight to mount;
  4. click Confirm (CCGS_AUTOPLAY_CLASS_CONFIRM_BTN);
  5. checkpoint ``class-confirmed``.
"""

from __future__ import annotations

from ._builder import RecipeBuilder
from ._coords import resolve

NAME = "class-select"
DESCRIPTION = "Class selection: click first card, click Confirm. Two checkpoints."


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)
    b.checkpoint("class-select-loaded")

    card, card_note = resolve("CLASS_FIRST_CARD", ctx.env)
    if card_note:
        b.note(card_note)
    cx, cy = b.frac(card.fx, card.fy)
    b.click(cx, cy)
    b.wait(6)

    confirm, confirm_note = resolve("CLASS_CONFIRM_BTN", ctx.env)
    if confirm_note:
        b.note(confirm_note)
    fx, fy = b.frac(confirm.fx, confirm.fy)
    b.click(fx, fy)
    b.wait(4)

    b.checkpoint("class-confirmed")
    b.clear_input()
    return b.build()
