"""Regression tests for the stale-pyc guard in Run-AutoplaySmoke.ps1 (PROMPT 1802).

Root cause captured: PROMPT 1801 live-verify failed because
``tools/autoplay/__pycache__/driver.cpython-312.pyc`` predated PROMPT 1794
(win_capture integration).  Python executed the stale bytecode; the driver log
had zero ``win32_capture:`` lines.

These tests are purely static — they read the launcher script as text and
verify the contract.  No GUI, no Bevy launch, no Cargo, no Python subprocess.

Run with:
    pytest tests/tools/autoplay/test_launcher_stale_pyc_guard.py -v
"""
from __future__ import annotations

from pathlib import Path

# ---------------------------------------------------------------------------
# Locate the launcher under test
# ---------------------------------------------------------------------------

_REPO_ROOT = Path(__file__).resolve().parents[3]
_LAUNCHER  = _REPO_ROOT / "tools" / "autoplay" / "Run-AutoplaySmoke.ps1"


def _launcher_text() -> str:
    return _LAUNCHER.read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


def test_launcher_stale_pyc_guard_launcher_exists() -> None:
    # Arrange / Act / Assert
    assert _LAUNCHER.exists(), f"Run-AutoplaySmoke.ps1 not found at {_LAUNCHER}"


def test_launcher_stale_pyc_guard_python_flag_b_present() -> None:
    """Python must be invoked with -B so bytecode cache is never read."""
    # Arrange
    text = _launcher_text()
    # Act / Assert
    assert "'-B'" in text or '"-B"' in text, (
        "Run-AutoplaySmoke.ps1 must pass '-B' to the Python invocation "
        "(PROMPT 1802 stale-pyc guard). Found neither '-B' nor \"-B\" in the script."
    )


def test_launcher_stale_pyc_guard_env_var_set() -> None:
    """PYTHONDONTWRITEBYTECODE must be set to '1' before the driver runs."""
    # Arrange
    text = _launcher_text()
    # Act / Assert
    assert "PYTHONDONTWRITEBYTECODE" in text, (
        "Run-AutoplaySmoke.ps1 must set $env:PYTHONDONTWRITEBYTECODE = '1' "
        "(PROMPT 1802 stale-pyc guard)."
    )


def test_launcher_stale_pyc_guard_cache_cleanup_present() -> None:
    """__pycache__ removal logic must appear before the driver Start-Process call."""
    # Arrange
    text = _launcher_text()
    # Act
    cache_cleanup_present = "__pycache__" in text and "Remove-Item" in text
    # Assert
    assert cache_cleanup_present, (
        "Run-AutoplaySmoke.ps1 must contain __pycache__ removal logic "
        "(Remove-Item ... __pycache__) before invoking the Python driver "
        "(PROMPT 1802 stale-pyc guard)."
    )


def test_launcher_stale_pyc_guard_cleanup_before_driver() -> None:
    """Cache cleanup lines must appear before the Start-Process driver invocation."""
    # Arrange
    text = _launcher_text()
    # Act
    cache_idx  = text.find("__pycache__")
    driver_idx = text.find("Start-Process -FilePath $Python")
    # Assert
    assert cache_idx != -1,  "__pycache__ reference not found in launcher"
    assert driver_idx != -1, "Start-Process $Python invocation not found in launcher"
    assert cache_idx < driver_idx, (
        "__pycache__ cleanup must appear BEFORE the Start-Process driver invocation; "
        f"found __pycache__ at char {cache_idx}, driver Start-Process at {driver_idx}"
    )
