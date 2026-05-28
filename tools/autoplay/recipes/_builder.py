"""Reusable primitives for autoplay recipes (PROMPT 1609).

These primitives compose into the ``list[dict]`` action stream the
driver consumes one entry per tick. Coordinates are always logical
window coordinates (the same space ``autoplay/status.cursor_logical``
reports); helpers ``frac_x`` / ``frac_y`` resolve fractional positions
against the live ``window_size``.

Hard rule: helpers MUST NOT synthesise gameplay verbs or invent new
RPC methods. The complete allowlist is enforced in
``tools/autoplay/driver.py``.
"""

from __future__ import annotations

from typing import Iterable


class RecipeBuilder:
    """Ordered, tick-driven action builder.

    Each primitive advances ``self.tick`` so callers can chain calls
    without juggling indices. ``hold_ticks`` parameters express how
    many driver ticks an input frame persists before the matching
    release.
    """

    def __init__(self, window_size: tuple[float, float], start_tick: int = 1) -> None:
        self.window_size = window_size
        self.tick = start_tick
        self.actions: list[dict] = []

    # --- internal -----------------------------------------------------
    def _emit(self, method: str, params: dict | None = None) -> None:
        self.actions.append({"tick": self.tick, "method": method, "params": params or {}})

    def _next(self, ticks: int = 1) -> None:
        if ticks < 1:
            raise ValueError("ticks must be >= 1")
        self.tick += ticks

    # --- coordinate helpers ------------------------------------------
    def frac(self, fx: float, fy: float) -> tuple[float, float]:
        w, h = self.window_size
        return (fx * w, fy * h)

    def centre(self) -> tuple[float, float]:
        return self.frac(0.5, 0.5)

    # --- low-level primitives ----------------------------------------
    def wait(self, ticks: int = 1) -> "RecipeBuilder":
        """Skip ``ticks`` driver ticks without emitting any action."""
        self._next(ticks)
        return self

    def status_only(self, ticks: int = 1) -> "RecipeBuilder":
        """Alias for :meth:`wait` to document soak/observability intent."""
        return self.wait(ticks)

    def cursor(self, x: float, y: float) -> "RecipeBuilder":
        self._emit("autoplay/input", {"cursor": {"screen": [x, y]}})
        self._next()
        return self

    def key_down(self, *keys: str) -> "RecipeBuilder":
        self._emit("autoplay/input", {"keys_down": list(keys)})
        self._next()
        return self

    def key_up(self, *keys: str) -> "RecipeBuilder":
        self._emit("autoplay/input", {"keys_up": list(keys)})
        self._next()
        return self

    def press(self, key: str, hold_ticks: int = 1) -> "RecipeBuilder":
        """Press ``key`` for ``hold_ticks`` ticks then release."""
        self.key_down(key)
        if hold_ticks > 1:
            self.wait(hold_ticks - 1)
        self.key_up(key)
        return self

    def mouse_down(self, button: str = "Left") -> "RecipeBuilder":
        self._emit("autoplay/input", {"mouse_down": [button]})
        self._next()
        return self

    def mouse_up(self, button: str = "Left") -> "RecipeBuilder":
        self._emit("autoplay/input", {"mouse_up": [button]})
        self._next()
        return self

    def click(self, x: float, y: float, button: str = "Left", hold_ticks: int = 1) -> "RecipeBuilder":
        """Cursor to (x,y), press the button, hold, release."""
        self.cursor(x, y)
        self.mouse_down(button)
        if hold_ticks > 1:
            self.wait(hold_ticks - 1)
        self.mouse_up(button)
        return self

    def drag(
        self,
        src: tuple[float, float],
        dst: tuple[float, float],
        button: str = "Left",
        steps: int = 3,
    ) -> "RecipeBuilder":
        """Cursor-down at ``src``, glide through ``steps`` intermediate cursor
        moves, then release at ``dst``. Useful for the placement drag/drop
        path which Bevy UI picking treats as a sustained drag gesture.
        """
        if steps < 1:
            raise ValueError("drag steps must be >= 1")
        self.cursor(*src)
        self.mouse_down(button)
        sx, sy = src
        dx, dy = dst
        for i in range(1, steps + 1):
            t = i / float(steps + 1)
            self.cursor(sx + (dx - sx) * t, sy + (dy - sy) * t)
        self.cursor(dx, dy)
        self.mouse_up(button)
        return self

    def clear_input(self) -> "RecipeBuilder":
        self._emit("autoplay/clear_input")
        self._next()
        return self

    def screenshot(self, reason: str) -> "RecipeBuilder":
        self._emit("autoplay/screenshot", {"reason": reason})
        self._next()
        return self

    # --- driver-local pseudo-primitives ------------------------------
    def checkpoint(
        self,
        label: str,
        screenshot: bool = True,
        settle_ticks: int = 3,
    ) -> "RecipeBuilder":
        """Emit a labelled checkpoint row to ``checkpoints.jsonl`` (driver-side)
        and, by default, request a screenshot named after the checkpoint.
        Used to delimit major phases in a recipe's timeline so reviewers
        can locate the lobby/draft/placement boundary without grepping
        ``driver-timeline.jsonl``.

        ``settle_ticks`` (default 3) inserts idle driver ticks between the
        checkpoint event and the screenshot request so the Bevy renderer has
        time to produce a frame that reflects the current game state.  At the
        default driver rate of 10 Hz each tick is ~100 ms, giving 300 ms of
        settle time.  Pass ``settle_ticks=0`` to restore the immediate
        (pre-PROMPT-1766) behaviour.  See GAP-SCR-01 / PROMPT 1763 for the
        stale-frame bug this parameter addresses.
        """
        self._emit("local.checkpoint", {"label": label, "screenshot": bool(screenshot)})
        self._next()
        if screenshot:
            if settle_ticks > 0:
                self._next(settle_ticks)
            self._emit("autoplay/screenshot", {"reason": f"checkpoint:{label}"})
            self._next()
        return self

    def note(self, message: str) -> "RecipeBuilder":
        """Write a free-form annotation into ``checkpoints.jsonl``.

        Useful when a recipe wants to record "skipped lobby-create
        because env X was unset" without bailing out.
        """
        self._emit("local.note", {"message": message})
        self._next()
        return self

    def block(self, reason: str, hint: str | None = None) -> "RecipeBuilder":
        """Signal that the recipe cannot proceed; driver writes the row,
        marks the run BLOCKED, and exits with code 4. Used by the
        ``full-game`` recipe when an upstream prerequisite (PROMPT 1607
        bot-vs-bot soak room) is missing.
        """
        params: dict = {"reason": reason}
        if hint is not None:
            params["hint"] = hint
        self._emit("local.block", params)
        self._next()
        return self

    def poll_phase(self, label: str, max_ticks: int = 30) -> "RecipeBuilder":
        """Emit a phase-polling pseudo-action.

        The driver will call ``autoplay/status`` repeatedly, checking
        ``status["phase"]``, until the value matches ``label`` or
        ``max_ticks`` polls have elapsed.  On timeout the driver logs a
        warning and emits a checkpoint row with ``timed_out=True``, then
        continues rather than aborting — replacing a brittle ``wait()``
        with a semantic phase gate that resolves as early as possible.

        ``max_ticks`` (default 30) caps how many status polls are issued
        at the driver's configured Hz rate (~100 ms each at 10 Hz →
        default 3 s window).  Pass a larger value for slow phase
        transitions.
        """
        self._emit("local.poll_phase", {"label": label, "max_ticks": int(max_ticks)})
        self._next()
        return self

    # --- finalize -----------------------------------------------------
    def build(self) -> list[dict]:
        return list(self.actions)


def flatten(*streams: Iterable[dict]) -> list[dict]:
    """Concatenate multiple action streams preserving tick order."""
    out: list[dict] = []
    for s in streams:
        out.extend(s)
    out.sort(key=lambda a: a["tick"])
    return out
