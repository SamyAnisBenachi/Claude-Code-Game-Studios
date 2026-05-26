"""``resolution-observe`` -- observe the Resolution phase with checkpoints.

This recipe is a pure observation recipe: it does not drive any input.
It waits for the resolution animation to play out and emits checkpoints
so reviewers can locate the phase boundary in the driver timeline.

Steps:
  1. checkpoint ``resolution-started`` (screenshot) — captured immediately
     when the recipe begins, assuming the client has just transitioned into
     Resolution (e.g. after ``placement-drag-probe`` submits);
  2. soak for a settling window (``CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS``,
     default 60) to let the combat animation and result splash play out;
  3. checkpoint ``resolution-complete`` (screenshot) — captured after the
     soak to document the settled post-resolution screen state.

Because the RPC surface (``autoplay/status``) does not expose the current
game-phase name, the recipe cannot positively confirm that the client is in
Resolution. If no visual change appears between the two screenshots the run
is still recorded as PASS at the recipe level; interpreting the evidence is
left to the human reviewer.

Override:
    CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS  (integer >= 1, default 60)
        How many driver ticks to wait between the two checkpoints.
"""

from __future__ import annotations

from ._builder import RecipeBuilder

NAME = "resolution-observe"
DESCRIPTION = (
    "Passive Resolution-phase observation: checkpoint + soak + checkpoint. "
    "No input driven. Two screenshots (resolution-started, resolution-complete). "
    "Override soak length via CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS (default 60)."
)

_SOAK_ENV = "CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS"
_DEFAULT_SOAK = 60


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)

    soak = _DEFAULT_SOAK
    raw = ctx.env.get(_SOAK_ENV, "").strip()
    if raw:
        try:
            parsed = int(raw)
            if parsed < 1:
                raise ValueError("must be >= 1")
            soak = parsed
        except ValueError as err:
            b.note(
                f"{_SOAK_ENV} parse failed ({err}); "
                f"using default {_DEFAULT_SOAK} ticks"
            )

    b.checkpoint("resolution-started")
    b.status_only(soak)
    b.checkpoint("resolution-complete")
    b.clear_input()
    return b.build()
