"""``idle`` -- status-only recipe for soak / observability."""

from __future__ import annotations

NAME = "idle"
DESCRIPTION = "No actions; ticks autoplay/status for soak / observability."


def build(ctx) -> list[dict]:
    return []
