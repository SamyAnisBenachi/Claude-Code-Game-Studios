"""``placement-drag-probe`` -- exercise the drag/drop placement path.

Drives the same low-level mouse drag a player produces when moving a
card from the hand strip to a board cell. The recipe does NOT verify
acceptance — that requires the QA snapshot / live observability lane
(see ``docs/autoplay.md``). It does emit checkpoints around each
drag so reviewers can locate the gesture in the timeline.

Steps:
  1. checkpoint ``placement-loaded``;
  2. drag from CCGS_AUTOPLAY_HAND_FIRST_CARD to CCGS_AUTOPLAY_BOARD_FIRST_CELL
     using ``RecipeBuilder.drag`` (multi-step cursor glide so Bevy UI
     picking sees a sustained drag);
  3. checkpoint ``placement-dragged``;
  4. click Submit (CCGS_AUTOPLAY_SUBMIT_BTN);
  5. checkpoint ``placement-submitted``.
"""

from __future__ import annotations

from ._builder import RecipeBuilder
from ._coords import resolve

NAME = "placement-drag-probe"
DESCRIPTION = "Drag from hand to board, click Submit. Three checkpoints (loaded, dragged, submitted)."


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)
    b.checkpoint("placement-loaded")

    hand, hand_note = resolve("HAND_FIRST_CARD", ctx.env)
    if hand_note:
        b.note(hand_note)
    board, board_note = resolve("BOARD_FIRST_CELL", ctx.env)
    if board_note:
        b.note(board_note)

    src = b.frac(hand.fx, hand.fy)
    dst = b.frac(board.fx, board.fy)
    b.drag(src, dst, steps=4)
    b.wait(4)
    b.checkpoint("placement-dragged")

    submit, submit_note = resolve("SUBMIT_BTN", ctx.env)
    if submit_note:
        b.note(submit_note)
    sx, sy = b.frac(submit.fx, submit.fy)
    b.click(sx, sy)
    b.wait(4)
    b.checkpoint("placement-submitted")
    b.clear_input()
    return b.build()
