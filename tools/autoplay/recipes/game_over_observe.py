"""``game-over-observe`` -- observe the GameOver / result screen.

This recipe is a pure observation recipe: it does not drive any input.
It waits for the game to reach the GameOver / result screen after combat
has reduced an objective's HP to zero and emits checkpoints so reviewers
can confirm that a complete game run was recorded.

Steps:
  1. checkpoint ``game-over-wait-start`` (screenshot=False) — marks the
     beginning of the wait window without spending a screenshot budget;
  2. soak for a polling window (``CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS``,
     default 120) to allow the server to finish the final resolution and
     push the GameOver / result-screen transition to the client;
  3. checkpoint ``game-over-screen`` (screenshot) — documents whatever is
     on screen after the soak; a human reviewer confirms whether the result
     screen is visible;
  4. additional short soak (``CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS``,
     default 30) for any winner-confirmation animation to settle;
  5. checkpoint ``winner-confirmed`` (screenshot) — final screenshot of
     the fully settled result state.

Detection note:
  ``autoplay/status`` does not expose the game-phase name. The recipe
  cannot positively confirm the client is on the GameOver screen. If the
  game has not reached GameOver within the soak window the screenshots will
  show the pre-GameOver state; the recipe records this as PASS (no block)
  but the reviewer will see no result screen. Use a longer soak or check
  that ``--bot-vs-bot-max-rounds`` is set on the server (Gap F in the
  PROMPT-1625 gap audit) to bound the run length.

Overrides:
    CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS         (integer >= 1, default 120)
        Ticks to wait before the ``game-over-screen`` checkpoint.
    CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS  (integer >= 1, default 30)
        Additional ticks to wait before the ``winner-confirmed`` checkpoint.
"""

from __future__ import annotations

from ._builder import RecipeBuilder

NAME = "game-over-observe"
DESCRIPTION = (
    "Passive GameOver-screen observation: soak + screenshot + soak + screenshot. "
    "No input driven. Two screenshots (game-over-screen, winner-confirmed). "
    "Override soak lengths via CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS (default 120) "
    "and CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS (default 30)."
)

_SOAK_ENV = "CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS"
_RESULT_SOAK_ENV = "CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS"
_DEFAULT_SOAK = 120
_DEFAULT_RESULT_SOAK = 30


def _parse_ticks(b: RecipeBuilder, env_key: str, default: int, env: dict[str, str]) -> int:
    raw = env.get(env_key, "").strip()
    if not raw:
        return default
    try:
        parsed = int(raw)
        if parsed < 1:
            raise ValueError("must be >= 1")
        return parsed
    except ValueError as err:
        b.note(f"{env_key} parse failed ({err}); using default {default} ticks")
        return default


def build(ctx) -> list[dict]:
    b = RecipeBuilder(ctx.window_size)

    soak = _parse_ticks(b, _SOAK_ENV, _DEFAULT_SOAK, ctx.env)
    result_soak = _parse_ticks(b, _RESULT_SOAK_ENV, _DEFAULT_RESULT_SOAK, ctx.env)

    b.checkpoint("game-over-wait-start", screenshot=False)
    b.status_only(soak)
    b.checkpoint("game-over-screen")
    b.status_only(result_soak)
    b.checkpoint("winner-confirmed")
    b.clear_input()
    return b.build()
