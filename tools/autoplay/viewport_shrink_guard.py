"""Mid-run viewport / window-size guard for autoplay click actions.

PROMPT 1922: Prevents bot/autoplay from clicking outside the visible UI when
the game window shrinks, moves, or clips after the run has started.

The guard runs before every autoplay/input RPC that carries cursor screen
coordinates and enforces two invariants:

1. Minimum-size gate: window_logical_size must be >= (min_w, min_h).
   Below this threshold the game UI is clipped and click targets are
   unreliable even if their coordinates are technically "inside" the rect.

2. In-bounds gate: the target (x, y) must fall inside [0, w) x [0, h).
   Coordinates outside the window rect are offscreen and will miss UI elements.

Both checks use the live window_logical_size from the autoplay/status response
sampled just before the action is dispatched, so mid-run shrink events are
caught on the tick they first appear.

Public API
----------
check_viewport_size(status, min_w, min_h) -> (ok, reason)
    Pure check of window dimensions only.  No coordinate needed.

check_click_target(x, y, status) -> (ok, reason)
    Check that (x, y) falls within the current window rect.

check_before_input(params, status, min_w, min_h) -> (ok, reason)
    Combined check for an autoplay/input params dict: extracts cursor
    screen coords (if present) and runs both guards.  Returns (True, None)
    when input has no cursor field (key/mouse-button-only actions are not
    position-checked).

Minimum QA viewport constant
-----------------------------
MIN_QA_VIEWPORT_W / MIN_QA_VIEWPORT_H: the baseline from PROMPT 1894.
Pass these to check_before_input / check_viewport_size unless a recipe
overrides them.
"""
from __future__ import annotations

from typing import Any

MIN_QA_VIEWPORT_W: float = 1280.0
MIN_QA_VIEWPORT_H: float = 720.0


def _extract_window_size(status: Any) -> tuple[float, float] | None:
    """Return (w, h) from an autoplay/status dict, or None if unavailable."""
    if not isinstance(status, dict):
        return None
    size = status.get("window_logical_size")
    if not isinstance(size, (list, tuple)) or len(size) != 2:
        return None
    try:
        return float(size[0]), float(size[1])
    except (TypeError, ValueError):
        return None


def check_viewport_size(
    status: Any,
    min_w: float = MIN_QA_VIEWPORT_W,
    min_h: float = MIN_QA_VIEWPORT_H,
) -> tuple[bool, str | None]:
    """Check that the current window meets the minimum QA size.

    Returns (True, None) when the viewport is large enough.
    Returns (False, reason) when it is too small or the size is unavailable.
    """
    size = _extract_window_size(status)
    if size is None:
        return False, (
            "viewport_size_unknown: window_logical_size missing from status; "
            "cannot verify click target is in-bounds"
        )
    w, h = size
    if w < min_w or h < min_h:
        return False, (
            f"viewport_too_small: window={w:.0f}x{h:.0f} "
            f"below minimum={min_w:.0f}x{min_h:.0f}; "
            "game UI may be clipped — skipping click"
        )
    return True, None


def check_click_target(
    x: float,
    y: float,
    status: Any,
) -> tuple[bool, str | None]:
    """Check that (x, y) is within the current window rect.

    Returns (True, None) when the coordinate is inside [0, w) x [0, h).
    Returns (False, reason) when out of bounds or size is unavailable.
    """
    size = _extract_window_size(status)
    if size is None:
        # Size unknown; be conservative and block.
        return False, (
            f"click_target_unverifiable: window size unknown; "
            f"cannot confirm ({x:.1f},{y:.1f}) is in-bounds — skipping click"
        )
    w, h = size
    if x < 0 or y < 0 or x >= w or y >= h:
        return False, (
            f"click_target_offscreen: ({x:.1f},{y:.1f}) outside "
            f"window {w:.0f}x{h:.0f} — skipping click"
        )
    return True, None


def check_before_input(
    params: dict[str, Any],
    status: Any,
    min_w: float = MIN_QA_VIEWPORT_W,
    min_h: float = MIN_QA_VIEWPORT_H,
) -> tuple[bool, str | None]:
    """Combined pre-dispatch guard for an autoplay/input action.

    Extracts cursor screen coordinates from *params* (if present) and
    runs the viewport-size check followed by the in-bounds check.

    Returns (True, None) when the action is safe to dispatch.
    Returns (False, reason) when the action should be skipped with the
    given diagnostic reason recorded to checkpoints.jsonl.

    Actions with no cursor field (key_down / key_up / mouse_down /
    mouse_up without explicit cursor) pass through as (True, None)
    because they have no screen-space coordinate to validate.
    """
    cursor = params.get("cursor")
    if cursor is None:
        # No screen coordinate in this action — nothing to guard.
        return True, None

    screen = cursor.get("screen") if isinstance(cursor, dict) else None
    if not isinstance(screen, (list, tuple)) or len(screen) != 2:
        # Malformed cursor field — let the RPC layer handle it normally.
        return True, None

    try:
        x, y = float(screen[0]), float(screen[1])
    except (TypeError, ValueError):
        return True, None

    # Gate 1: viewport minimum size
    size_ok, size_reason = check_viewport_size(status, min_w, min_h)
    if not size_ok:
        return False, size_reason

    # Gate 2: coordinate in-bounds
    return check_click_target(x, y, status)
