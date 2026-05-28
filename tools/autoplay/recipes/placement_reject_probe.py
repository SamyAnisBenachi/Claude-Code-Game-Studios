"""``placement-reject-probe`` -- exercise placement rejection and recovery.

Drives the client through the placement-rejection / recovery cycle:

  1. Drag hand card → BOARD_DEEP_CELL (upper-board, heuristically outside the
     player's spawn range, likely to trigger S2CPlacementRejected).
  2. Click Submit.
  3. Wait CCGS_AUTOPLAY_REJECT_SETTLE_TICKS (default 20 = 2 s @ 10 Hz) for the
     server round-trip and client re-render.
  4. Checkpoint ``placement-reject-feedback`` — screenshot captures the
     rejection banner / visual feedback state.
  5. Recover: drag from BOARD_DEEP_CELL → BOARD_FIRST_CELL (valid target).
  6. Click Submit (recovery path).
  7. Wait settle ticks.
  8. Checkpoint ``placement-reject-recovery-submitted``.

Observable limitations (TODO — require new app signals):
  * Autoplay has no ``autoplay/status`` phase-state or rejection-acknowledged
    signal.  Whether S2CPlacementRejected was actually received can only be
    confirmed by visual review of the ``placement-reject-feedback`` screenshot.
  * BOARD_DEEP_CELL default (fy=0.30) is heuristic.  If the cell falls within
    the player's spawn range for the active class/round it will be accepted, not
    rejected.  Override with ``CCGS_AUTOPLAY_BOARD_DEEP_CELL=fx,fy`` to target a
    known-invalid cell.
  * If no rejection occurs (valid cell hit accidentally), the recovery drag still
    exercises the double-submit path; this is valid evidence that the client
    handles a double-submit cleanly.
  * The recipe does not unstage via an explicit cancel button because none is
    defined in the current UI layout (PROMPT 1468: recovery is drag-retarget or
    unstage gesture, not a dedicated Cancel CTA).

Prerequisites:
  * Client must already be in the placement phase before the driver starts.
    Use ``vs-bot`` or ``full-game`` composites to reach placement automatically,
    or manually advance the game before running this recipe standalone.
"""

from __future__ import annotations

from ._builder import RecipeBuilder
from ._coords import resolve

NAME = "placement-reject-probe"
DESCRIPTION = (
    "Stage a card at a heuristically deep board cell (likely out of spawn range), "
    "submit, wait for rejection round-trip, then recover by retargeting to BOARD_FIRST_CELL "
    "and re-submitting. Verification relies on checkpoint screenshots; no programmatic "
    "rejection signal is available. "
    "Override CCGS_AUTOPLAY_BOARD_DEEP_CELL=fx,fy to target a known-invalid cell."
)

_REJECT_SETTLE_ENV = "CCGS_AUTOPLAY_REJECT_SETTLE_TICKS"
_DEFAULT_REJECT_TICKS = 20  # 2 s at 10 Hz — budget for server round-trip + client re-render


def _reject_settle_ticks(env: dict[str, str]) -> int:
    raw = env.get(_REJECT_SETTLE_ENV, "")
    if raw.strip():
        try:
            v = int(raw.strip())
            if v >= 1:
                return v
        except ValueError:
            pass
    return _DEFAULT_REJECT_TICKS


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)
    b.checkpoint("placement-reject-loaded")

    # Resolve coordinates with env overrides.
    hand, hand_note = resolve("HAND_FIRST_CARD", ctx.env)
    if hand_note:
        b.note(hand_note)
    deep, deep_note = resolve("BOARD_DEEP_CELL", ctx.env)
    if deep_note:
        b.note(deep_note)
    first, first_note = resolve("BOARD_FIRST_CELL", ctx.env)
    if first_note:
        b.note(first_note)
    submit, submit_note = resolve("SUBMIT_BTN", ctx.env)
    if submit_note:
        b.note(submit_note)

    settle = _reject_settle_ticks(ctx.env)

    # Step 1 — drag to deep / heuristically-invalid cell.
    src = b.frac(hand.fx, hand.fy)
    dst_deep = b.frac(deep.fx, deep.fy)
    b.drag(src, dst_deep, steps=4)
    b.wait(4)
    b.checkpoint("placement-reject-deep-staged")

    # Step 2 — submit (expect S2CPlacementRejected from server).
    sx, sy = b.frac(submit.fx, submit.fy)
    b.click(sx, sy)
    b.wait(settle)
    # TODO(observability): autoplay/status returns no rejection-state signal.
    # Confirm rejection by reviewing this checkpoint's screenshot for the
    # rejection banner (PROMPT 1468 UX: rejected badge, timer flag re-opened).
    b.note(
        "TODO: no autoplay/status rejection-state signal; "
        "rejection confirmed by screenshot review of placement-reject-feedback only."
    )
    b.checkpoint("placement-reject-feedback")

    # Step 3 — recovery: retarget to BOARD_FIRST_CELL then re-submit.
    # Per PROMPT 1468 the rejected batch is marked inactive; a new drag gesture
    # to a valid cell re-stages the card without a dedicated Cancel CTA.
    dst_first = b.frac(first.fx, first.fy)
    b.drag(dst_deep, dst_first, steps=4)
    b.wait(4)
    b.checkpoint("placement-reject-recovery-staged")

    b.click(sx, sy)
    b.wait(settle)
    b.checkpoint("placement-reject-recovery-submitted")

    b.clear_input()
    return b.build()
