"""File-ready poll for screenshot completion (PROMPT 1793).

Extracted as a standalone module so tests can import it without pulling in
driver.py's platform-specific dependencies (win_foreground, etc.).
"""
from __future__ import annotations

import time
from pathlib import Path
from typing import Callable

# Default poll parameters (PROMPT 1793 recommendation).
SCREENSHOT_POLL_INTERVAL: float = 0.1   # seconds between existence checks
SCREENSHOT_POLL_TIMEOUT: float = 3.0    # maximum seconds to wait


def wait_for_screenshot_file(
    path: Path,
    tick: int,
    log_fn: Callable[[str], None],
    poll_interval: float = SCREENSHOT_POLL_INTERVAL,
    timeout: float = SCREENSHOT_POLL_TIMEOUT,
) -> bool:
    """Poll until *path* exists with size > 0, or *timeout* elapses.

    The ``autoplay/screenshot`` RPC returns immediately after queuing the
    capture command; ``save_to_disk`` is async and may not have flushed the
    PNG by the time the driver continues.  This poll bridges that gap.

    Logs a clear success or timeout line via *log_fn* so driver.log provides
    unambiguous evidence of PNG write completion.

    Args:
        path: Absolute path to the expected PNG file.
        tick: Current driver tick number (included in log messages).
        log_fn: Callable matching the driver ``log`` signature.
        poll_interval: Seconds between existence checks.
        timeout: Maximum seconds to wait before giving up.

    Returns:
        True when the file is ready, False on timeout.
    """
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            if path.exists() and path.stat().st_size > 0:
                size = path.stat().st_size
                log_fn(
                    f"tick={tick} screenshot file ready: {path.name} ({size} bytes)"
                )
                return True
        except OSError:
            pass
        time.sleep(poll_interval)
    log_fn(
        f"tick={tick} WARNING screenshot file-ready poll timed out after {timeout}s: "
        f"path={path} (file may be missing or zero bytes)"
    )
    return False
