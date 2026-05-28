"""Unit tests for tools/autoplay/win_capture.py (PROMPT 1794).

All tests run without a GUI, without a live Bevy client, and without Cargo.
Win32 API boundaries are mocked via unittest.mock.

Covers:
  - is_available() platform guard
  - _write_png() produces a valid PNG byte stream
  - capture_game_window() non-Windows no-op
  - capture_game_window() no window found
  - capture_game_window() unexpected exception swallowed
  - capture_game_window() happy path delegates to _capture_hwnd_to_png
  - _capture_hwnd_to_png() GetWindowRect failure
  - _capture_hwnd_to_png() zero-size window guard
  - _capture_hwnd_to_png() GetDC failure
  - _capture_hwnd_to_png() CreateCompatibleDC failure
  - _capture_hwnd_to_png() CreateCompatibleBitmap failure
  - _capture_hwnd_to_png() PrintWindow failure with both flags
  - _capture_hwnd_to_png() GetDIBits failure
  - _capture_hwnd_to_png() happy path writes a valid PNG
  - Structural: driver.py imports win_capture + calls _win32_capture

Run with:
    pytest tests/tools/autoplay/test_win32_capture.py -v
"""
from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

import win_capture  # noqa: E402
from win_capture import (  # noqa: E402
    _capture_hwnd_bitblt_to_png,
    _capture_hwnd_to_png,
    _write_png,
    capture_game_window,
    capture_game_window_desktop_bitblt,
    is_available,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _log() -> tuple[list[str], object]:
    lines: list[str] = []
    return lines, lines.append


def _make_rect_mock(left=0, top=0, right=1280, bottom=720):
    """Return a mock ctypes.wintypes.RECT with .right/.bottom/.left/.top."""
    r = MagicMock()
    r.left = left
    r.top = top
    r.right = right
    r.bottom = bottom
    return r


def _make_fake_user32(
    *,
    getrect_ok: bool = True,
    getdc_ok: bool = True,
    printwindow_ok: int = 1,
    width: int = 4,
    height: int = 2,
) -> MagicMock:
    u = MagicMock()
    u.GetWindowRect.return_value = int(getrect_ok)
    u.GetDC.return_value = 0x100 if getdc_ok else 0
    u.ReleaseDC.return_value = 1
    u.PrintWindow.return_value = printwindow_ok
    # Store dimensions for rect side-effect
    u._width = width
    u._height = height
    return u


def _make_fake_gdi32(
    *,
    create_dc_ok: bool = True,
    create_bmp_ok: bool = True,
    getdibits_lines: int = 2,
    width: int = 4,
    height: int = 2,
) -> MagicMock:
    g = MagicMock()
    g.CreateCompatibleDC.return_value = 0x200 if create_dc_ok else 0
    g.CreateCompatibleBitmap.return_value = 0x300 if create_bmp_ok else 0
    g.SelectObject.return_value = 0x400
    g.DeleteObject.return_value = 1
    g.DeleteDC.return_value = 1
    g.GetDIBits.return_value = getdibits_lines
    return g


def _make_pixel_buf_side_effect(width: int, height: int):
    """Return a side-effect that fills the pixel buffer with non-zero BGRA data."""
    def _fill(_mem_dc, _bitmap, _start, _lines, pixel_buf, _bi, _mode):
        import ctypes
        # Fill with a mid-grey pixel (B=128, G=64, R=32, A=255)
        for i in range(width * height):
            pixel_buf[i * 4] = 128    # B
            pixel_buf[i * 4 + 1] = 64  # G
            pixel_buf[i * 4 + 2] = 32  # R
            pixel_buf[i * 4 + 3] = 255  # A
        return height
    return _fill


# ---------------------------------------------------------------------------
# 1. is_available()
# ---------------------------------------------------------------------------

class TestIsAvailable:
    def test_win32_capture_is_available_returns_bool(self):
        result = is_available()
        assert isinstance(result, bool)

    def test_win32_capture_is_available_false_on_non_windows(self, monkeypatch):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", False)
        assert is_available() is False

    def test_win32_capture_is_available_true_on_windows(self, monkeypatch):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        assert is_available() is True


# ---------------------------------------------------------------------------
# 2. _write_png() — stdlib encoder
# ---------------------------------------------------------------------------

class TestWritePng:
    def test_win32_capture_write_png_produces_png_signature(self, tmp_path):
        # Arrange
        path = tmp_path / "out.png"
        rgb_rows = [bytes([255, 0, 0, 0, 255, 0])]  # 2 RGB pixels, 1 row

        # Act
        _write_png(path, width=2, height=1, rgb_rows=rgb_rows)

        # Assert
        data = path.read_bytes()
        assert data[:8] == b"\x89PNG\r\n\x1a\n", "output must start with PNG signature"

    def test_win32_capture_write_png_produces_non_empty_file(self, tmp_path):
        path = tmp_path / "out.png"
        rgb_rows = [bytes([0, 0, 0] * 4)] * 4  # 4x4 black image
        _write_png(path, width=4, height=4, rgb_rows=rgb_rows)
        assert path.stat().st_size > 0

    def test_win32_capture_write_png_ihdr_contains_correct_dimensions(self, tmp_path):
        path = tmp_path / "out.png"
        width, height = 7, 3
        rgb_rows = [bytes([0] * (width * 3))] * height
        _write_png(path, width=width, height=height, rgb_rows=rgb_rows)

        data = path.read_bytes()
        # IHDR chunk: offset 8 = length(4) + "IHDR"(4) + width(4) + height(4)
        w = struct.unpack(">I", data[16:20])[0]
        h = struct.unpack(">I", data[20:24])[0]
        assert w == width
        assert h == height

    def test_win32_capture_write_png_iend_chunk_present(self, tmp_path):
        path = tmp_path / "out.png"
        _write_png(path, width=1, height=1, rgb_rows=[bytes([0, 0, 0])])
        data = path.read_bytes()
        assert b"IEND" in data

    def test_win32_capture_write_png_color_type_is_rgb(self, tmp_path):
        path = tmp_path / "out.png"
        _write_png(path, width=1, height=1, rgb_rows=[bytes([0, 0, 0])])
        data = path.read_bytes()
        # IHDR: 8 (sig) + 4 (len) + 4 (tag) + 4 (w) + 4 (h) + 1 (depth) + 1 (color) = offset 25
        color_type = data[25]
        assert color_type == 2, "color type must be 2 (RGB truecolor)"

    def test_win32_capture_write_png_idat_is_zlib_decompressible(self, tmp_path):
        path = tmp_path / "out.png"
        rgb_rows = [bytes([200, 100, 50])]
        _write_png(path, width=1, height=1, rgb_rows=rgb_rows)

        data = path.read_bytes()
        # Find IDAT chunk: after IHDR (8+4+4+13+4=33 bytes)
        idat_start = 33
        idat_len = struct.unpack(">I", data[idat_start:idat_start + 4])[0]
        idat_data = data[idat_start + 8: idat_start + 8 + idat_len]
        decompressed = zlib.decompress(idat_data)
        # First byte of first row must be filter type 0
        assert decompressed[0] == 0, "filter byte must be 0 (None filter)"


# ---------------------------------------------------------------------------
# 3. capture_game_window() — top-level entry point
# ---------------------------------------------------------------------------

class TestCaptureGameWindow:
    def test_win32_capture_game_window_noop_on_non_windows(self, monkeypatch, tmp_path):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", False)
        lines, log = _log()
        result = capture_game_window(tmp_path / "out.png", log)
        assert result is False
        assert any("non-Windows" in l for l in lines)

    def test_win32_capture_game_window_emits_log_on_non_windows(self, monkeypatch, tmp_path):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", False)
        lines, log = _log()
        capture_game_window(tmp_path / "out.png", log)
        assert len(lines) >= 1

    def test_win32_capture_game_window_returns_false_when_no_window(self, monkeypatch, tmp_path):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            monkeypatch.setattr(
                "win_foreground._list_visible_windows", lambda _u: []
            )
            lines, log = _log()
            result = capture_game_window(tmp_path / "out.png", log)
        assert result is False
        assert any("no CCGS/Bevy window found" in l for l in lines)

    def test_win32_capture_game_window_swallows_unexpected_exception(self, monkeypatch, tmp_path):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)

        def _boom():
            raise RuntimeError("simulated failure")

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            monkeypatch.setattr(
                "win_foreground._list_visible_windows",
                lambda _u: (_ for _ in ()).throw(RuntimeError("boom")),  # type: ignore[arg-type]
            )
            lines, log = _log()
            result = capture_game_window(tmp_path / "out.png", log)
        assert result is False
        assert any("unexpected error" in l for l in lines)

    def test_win32_capture_game_window_calls_capture_hwnd_on_match(self, monkeypatch, tmp_path):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        fake_windows = [(0xAB01, "Lanes and Lies")]
        monkeypatch.setattr("win_foreground._list_visible_windows", lambda _u: fake_windows)

        captured_hwnds: list[int] = []

        def _fake_capture(hwnd, path, log, *, user32=None, gdi32=None):
            captured_hwnds.append(hwnd)
            return True

        monkeypatch.setattr(win_capture, "_capture_hwnd_to_png", _fake_capture)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            lines, log = _log()
            result = capture_game_window(tmp_path / "out.png", log)

        assert result is True
        assert captured_hwnds == [0xAB01]

    def test_win32_capture_game_window_logs_matched_title(self, monkeypatch, tmp_path):
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        fake_windows = [(0xAB01, "Lanes and Lies")]
        monkeypatch.setattr("win_foreground._list_visible_windows", lambda _u: fake_windows)
        monkeypatch.setattr(win_capture, "_capture_hwnd_to_png", lambda *a, **k: True)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            lines, log = _log()
            capture_game_window(tmp_path / "out.png", log)

        assert any("Lanes and Lies" in l for l in lines)


# ---------------------------------------------------------------------------
# 4. _capture_hwnd_to_png() — GDI pipeline, mocked user32/gdi32
# ---------------------------------------------------------------------------

class TestCaptureHwndToPng:
    def _rect_side_effect(self, width, height):
        """Return a side_effect for GetWindowRect that fills the RECT mock."""
        import ctypes
        import ctypes.wintypes

        def _fill(hwnd, rect_ptr):
            # byref delivers the RECT object; access through _obj_
            obj = rect_ptr._obj
            obj.left = 0
            obj.top = 0
            obj.right = width
            obj.bottom = height
            return 1

        return _fill

    def _make_gdi_happy(self, width=4, height=2) -> tuple[MagicMock, MagicMock]:
        """Return (user32, gdi32) mocks that produce a successful capture."""
        import ctypes
        user32 = MagicMock()
        user32.GetWindowRect.return_value = 1
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        user32.PrintWindow.return_value = 1

        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        def _getdibits(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            for i in range(width * height):
                pixel_buf[i * 4] = 128
                pixel_buf[i * 4 + 1] = 64
                pixel_buf[i * 4 + 2] = 32
                pixel_buf[i * 4 + 3] = 255
            return height

        gdi32.GetDIBits.side_effect = _getdibits
        return user32, gdi32

    def _set_rect(self, user32, width, height):
        import ctypes
        import ctypes.wintypes

        def _fill_rect(hwnd, byref_rect):
            inner = byref_rect._obj
            inner.left = 0
            inner.top = 0
            inner.right = width
            inner.bottom = height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect

    def test_win32_capture_hwnd_returns_false_when_getrect_fails(self, tmp_path):
        user32 = MagicMock()
        user32.GetWindowRect.return_value = 0
        user32.GetDC.return_value = 0
        user32.ReleaseDC.return_value = 1
        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0
        gdi32.DeleteDC.return_value = 1

        lines, log = _log()
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)
        assert result is False
        assert any("GetWindowRect failed" in l for l in lines)

    def test_win32_capture_hwnd_returns_false_when_getdc_fails(self, tmp_path, monkeypatch):
        import ctypes
        import ctypes.wintypes

        user32 = MagicMock()
        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0
        gdi32.DeleteDC.return_value = 1

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, 100, 100
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0  # DC failure
        user32.ReleaseDC.return_value = 1

        lines, log = _log()
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)
        assert result is False
        assert any("GetDC failed" in l for l in lines)

    def test_win32_capture_hwnd_returns_false_on_zero_size_window(self, tmp_path):
        import ctypes
        import ctypes.wintypes

        user32 = MagicMock()
        gdi32 = MagicMock()
        gdi32.DeleteDC.return_value = 1

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, 0, 0  # zero size
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1

        lines, log = _log()
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)
        assert result is False
        assert any("zero-size" in l for l in lines)

    def test_win32_capture_hwnd_returns_false_when_printwindow_fails_both_flags(self, tmp_path):
        import ctypes
        import ctypes.wintypes

        user32 = MagicMock()
        gdi32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, 4, 2
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        user32.PrintWindow.return_value = 0  # always fails

        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        lines, log = _log()
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)
        assert result is False
        assert any("PrintWindow failed" in l for l in lines)

    def test_win32_capture_hwnd_returns_false_when_getdibits_returns_zero(self, tmp_path):
        import ctypes
        import ctypes.wintypes

        user32 = MagicMock()
        gdi32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, 4, 2
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        user32.PrintWindow.return_value = 1

        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1
        gdi32.GetDIBits.return_value = 0  # failure

        lines, log = _log()
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)
        assert result is False
        assert any("GetDIBits" in l for l in lines)

    def test_win32_capture_hwnd_happy_path_writes_png_file(self, tmp_path):
        import ctypes
        import ctypes.wintypes

        width, height = 4, 2
        user32 = MagicMock()
        gdi32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, width, height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        user32.PrintWindow.return_value = 1

        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        def _getdibits(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            for i in range(width * height):
                pixel_buf[i * 4] = 50   # B
                pixel_buf[i * 4 + 1] = 100  # G
                pixel_buf[i * 4 + 2] = 150  # R
                pixel_buf[i * 4 + 3] = 255  # A
            return height

        gdi32.GetDIBits.side_effect = _getdibits

        out = tmp_path / "capture.png"
        lines, log = _log()
        result = _capture_hwnd_to_png(0x1001, out, log, user32=user32, gdi32=gdi32)

        assert result is True
        assert out.exists(), "PNG file must be written"
        assert out.stat().st_size > 0, "PNG file must not be empty"
        assert out.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n", "must be a valid PNG"

    def test_win32_capture_hwnd_happy_path_logs_success(self, tmp_path):
        import ctypes
        import ctypes.wintypes

        width, height = 2, 2
        user32 = MagicMock()
        gdi32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, width, height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        user32.PrintWindow.return_value = 1

        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        def _getdibits(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            for i in range(width * height):
                pixel_buf[i * 4 + 2] = 200
            return height

        gdi32.GetDIBits.side_effect = _getdibits

        out = tmp_path / "capture.png"
        lines, log = _log()
        _capture_hwnd_to_png(0x1001, out, log, user32=user32, gdi32=gdi32)
        assert any("PNG written" in l for l in lines)

    def test_win32_capture_hwnd_logs_printwindow_retry_on_flag_failure(self, tmp_path):
        import ctypes
        import ctypes.wintypes

        width, height = 2, 2
        user32 = MagicMock()
        gdi32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, width, height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        # First call (PW_RENDERFULLCONTENT) fails, second (flags=0) succeeds
        user32.PrintWindow.side_effect = [0, 1]

        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        def _getdibits(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            return height

        gdi32.GetDIBits.side_effect = _getdibits

        out = tmp_path / "capture.png"
        lines, log = _log()
        _capture_hwnd_to_png(0x1001, out, log, user32=user32, gdi32=gdi32)

        # The retry log line must appear
        assert any("PW_RENDERFULLCONTENT" in l and "retrying" in l for l in lines)


# ---------------------------------------------------------------------------
# 5. Structural checks: driver.py wired up correctly (PROMPT 1794)
# ---------------------------------------------------------------------------

class TestDriverWin32CaptureWiring:
    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_win32_capture_driver_imports_win_capture(self):
        assert "from win_capture import" in self._DRIVER_SOURCE, (
            "driver.py must import from win_capture (PROMPT 1794)"
        )

    def test_win32_capture_driver_imports_capture_game_window(self):
        assert "_win32_capture" in self._DRIVER_SOURCE, (
            "driver.py must alias capture_game_window as _win32_capture"
        )

    def test_win32_capture_driver_calls_win32_capture_in_screenshot_branch(self):
        src = self._DRIVER_SOURCE
        screenshot_idx = src.index('method == "autoplay/screenshot"')
        capture_idx = src.index("_win32_capture(")
        assert capture_idx > screenshot_idx, (
            "_win32_capture call must appear after the autoplay/screenshot check"
        )

    def test_win32_capture_driver_win32_shot_path_uses_tick(self):
        assert "win32_tick_" in self._DRIVER_SOURCE, (
            "Win32 capture filename must embed the tick number (win32_tick_...)"
        )

    def test_win32_capture_driver_win32_capture_placed_after_ensure_foreground(self):
        src = self._DRIVER_SOURCE
        fg_idx = src.index("ensure_foreground(log)")
        capture_idx = src.index("_win32_capture(")
        assert capture_idx > fg_idx, (
            "_win32_capture must be called after ensure_foreground "
            "so the window is foregrounded before capture"
        )


# ---------------------------------------------------------------------------
# 6. TestCaptureHwndRestoreBeforeCapture (PROMPT 1803)
# ---------------------------------------------------------------------------

class TestCaptureHwndRestoreBeforeCapture:
    """Tests that ShowWindow and SetForegroundWindow are called before PrintWindow."""

    def _make_happy_user32_with_restore(self, width=4, height=2) -> MagicMock:
        """Return a user32 mock that succeeds and tracks ShowWindow/SetForeground calls."""
        user32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, width, height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        user32.PrintWindow.return_value = 1
        user32.ShowWindow.return_value = 1
        user32.SetForegroundWindow.return_value = 1
        return user32

    def _make_happy_gdi32(self, width=4, height=2) -> MagicMock:
        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        def _getdibits(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            for i in range(width * height):
                pixel_buf[i * 4] = 128
                pixel_buf[i * 4 + 1] = 64
                pixel_buf[i * 4 + 2] = 32
                pixel_buf[i * 4 + 3] = 255
            return height

        gdi32.GetDIBits.side_effect = _getdibits
        return gdi32

    def test_win32_capture_hwnd_calls_show_window_before_printwindow(self, tmp_path):
        # Arrange
        user32 = self._make_happy_user32_with_restore()
        gdi32 = self._make_happy_gdi32()
        call_order: list[str] = []

        original_sw = user32.ShowWindow
        original_pw = user32.PrintWindow

        def _track_sw(hwnd, cmd):
            call_order.append("ShowWindow")
            return 1

        def _track_pw(hwnd, dc, flags):
            call_order.append("PrintWindow")
            return 1

        user32.ShowWindow.side_effect = _track_sw
        user32.PrintWindow.side_effect = _track_pw

        lines, log = _log()

        # Act
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is True
        sw_pos = call_order.index("ShowWindow")
        pw_pos = call_order.index("PrintWindow")
        assert sw_pos < pw_pos, "ShowWindow must be called before PrintWindow"

    def test_win32_capture_hwnd_logs_show_window_result(self, tmp_path):
        # Arrange
        user32 = self._make_happy_user32_with_restore()
        gdi32 = self._make_happy_gdi32()
        lines, log = _log()

        # Act
        _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert any("ShowWindow" in l for l in lines), (
            "A log line containing 'ShowWindow' must appear"
        )

    def test_win32_capture_hwnd_logs_setforegroundwindow_result(self, tmp_path):
        # Arrange
        user32 = self._make_happy_user32_with_restore()
        gdi32 = self._make_happy_gdi32()
        lines, log = _log()

        # Act
        _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert any("SetForegroundWindow" in l for l in lines), (
            "A log line containing 'SetForegroundWindow' must appear"
        )

    def test_win32_capture_hwnd_proceeds_even_when_setforeground_returns_zero(self, tmp_path):
        # Arrange: SetForegroundWindow returns 0 (may fail in non-interactive sessions)
        user32 = self._make_happy_user32_with_restore()
        user32.SetForegroundWindow.return_value = 0
        gdi32 = self._make_happy_gdi32()
        lines, log = _log()

        # Act
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert: capture must still succeed
        assert result is True, "Capture must succeed even when SetForegroundWindow returns 0"
        assert any("SetForegroundWindow ret=0" in l for l in lines), (
            "Must log the zero return value"
        )


# ---------------------------------------------------------------------------
# 7. TestCaptureHwndPixelHash (PROMPT 1803)
# ---------------------------------------------------------------------------

class TestCaptureHwndPixelHash:
    """Tests the pixel hash logging after a successful GetDIBits."""

    def _make_happy_user32(self, width=4, height=2) -> MagicMock:
        user32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, width, height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        user32.PrintWindow.return_value = 1
        user32.ShowWindow.return_value = 1
        user32.SetForegroundWindow.return_value = 1
        return user32

    def _make_gdi32_with_fill(self, width, height, fill_value: int = 128) -> MagicMock:
        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        def _getdibits(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            for i in range(width * height * 4):
                pixel_buf[i] = fill_value
            return height

        gdi32.GetDIBits.side_effect = _getdibits
        return gdi32

    def test_win32_capture_hwnd_logs_pixel_hash_on_success(self, tmp_path):
        # Arrange
        width, height = 4, 2
        user32 = self._make_happy_user32(width, height)
        gdi32 = self._make_gdi32_with_fill(width, height, fill_value=100)
        lines, log = _log()

        # Act
        result = _capture_hwnd_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is True
        assert any("pixel_hash=" in l for l in lines), (
            "A log line containing 'pixel_hash=' must appear after successful capture"
        )

    def test_win32_capture_hwnd_pixel_hash_differs_between_captures(self, tmp_path):
        # Arrange: two captures with different pixel data
        width, height = 4, 2

        user32_a = self._make_happy_user32(width, height)
        gdi32_a = self._make_gdi32_with_fill(width, height, fill_value=50)
        lines_a, log_a = _log()

        user32_b = self._make_happy_user32(width, height)
        gdi32_b = self._make_gdi32_with_fill(width, height, fill_value=200)
        lines_b, log_b = _log()

        # Act
        _capture_hwnd_to_png(0x1001, tmp_path / "cap_a.png", log_a, user32=user32_a, gdi32=gdi32_a)
        _capture_hwnd_to_png(0x1002, tmp_path / "cap_b.png", log_b, user32=user32_b, gdi32=gdi32_b)

        # Extract hash values from log lines
        def _extract_hash(lines: list[str]) -> str:
            for l in lines:
                if "pixel_hash=" in l:
                    # e.g. "win32_capture: pixel_hash=0x12345678 width=..."
                    for token in l.split():
                        if token.startswith("pixel_hash="):
                            return token.split("=", 1)[1]
            return ""

        hash_a = _extract_hash(lines_a)
        hash_b = _extract_hash(lines_b)

        # Assert
        assert hash_a != "", "First capture must log a pixel_hash"
        assert hash_b != "", "Second capture must log a pixel_hash"
        assert hash_a != hash_b, (
            f"Captures with different pixel data must produce different hashes "
            f"(got hash_a={hash_a}, hash_b={hash_b})"
        )


# ---------------------------------------------------------------------------
# 8. TestDriverWin32CaptureOrchestration (PROMPT 1803)
# ---------------------------------------------------------------------------

class TestDriverWin32CaptureOrchestration:
    """Structural tests for driver.py capture orchestration additions."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_captures_win32_return_value(self):
        # Assert: driver.py assigns the return value of _win32_capture
        assert "_win32_ok = _win32_capture(" in self._DRIVER_SOURCE, (
            "driver.py must capture the return value: _win32_ok = _win32_capture(...)"
        )

    def test_driver_logs_win32_capture_result(self):
        # PROMPT 1813 renamed the label to win32_printwindow= to distinguish it
        # from the new desktop_bitblt= label.
        assert "win32_printwindow=" in self._DRIVER_SOURCE, (
            "driver.py must log win32_printwindow=OK/FAILED after the _win32_capture call"
        )

    def test_driver_has_dwm_settle_sleep_after_ensure_foreground(self):
        # Assert: time.sleep(0.12) appears between ensure_foreground and _win32_capture
        src = self._DRIVER_SOURCE
        fg_idx = src.index("ensure_foreground(log)")
        capture_idx = src.index("_win32_ok = _win32_capture(")
        sleep_idx = src.index("time.sleep(0.12)", fg_idx)
        assert fg_idx < sleep_idx < capture_idx, (
            "time.sleep(0.12) must appear after ensure_foreground and before _win32_capture"
        )


# ---------------------------------------------------------------------------
# 9. TestCaptureHwndBitblt (PROMPT 1813) — desktop BitBlt backend unit tests
# ---------------------------------------------------------------------------

class TestCaptureHwndBitblt:
    """Unit tests for _capture_hwnd_bitblt_to_png (desktop_bitblt backend)."""

    def _make_happy_user32(self, width: int = 4, height: int = 2) -> MagicMock:
        user32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 10, 20, 10 + width, 20 + height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100   # desktop_dc
        user32.ReleaseDC.return_value = 1
        return user32

    def _make_happy_gdi32(self, width: int = 4, height: int = 2) -> MagicMock:
        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0x300
        gdi32.SelectObject.return_value = 0x400
        gdi32.BitBlt.return_value = 1  # SRCCOPY success
        gdi32.DeleteObject.return_value = 1
        gdi32.DeleteDC.return_value = 1

        def _getdibits(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            for i in range(width * height):
                pixel_buf[i * 4] = 80      # B
                pixel_buf[i * 4 + 1] = 120  # G
                pixel_buf[i * 4 + 2] = 200  # R
                pixel_buf[i * 4 + 3] = 255  # A
            return height

        gdi32.GetDIBits.side_effect = _getdibits
        return gdi32

    def test_desktop_bitblt_returns_false_when_getrect_fails(self, tmp_path):
        # Arrange
        user32 = MagicMock()
        user32.GetWindowRect.return_value = 0
        user32.GetDC.return_value = 0
        user32.ReleaseDC.return_value = 1
        gdi32 = MagicMock()
        gdi32.DeleteDC.return_value = 1

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("GetWindowRect failed" in l for l in lines)

    def test_desktop_bitblt_returns_false_when_window_is_minimised(self, tmp_path):
        # Arrange: zero-size rect (minimised window)
        user32 = MagicMock()

        def _fill_zero(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, 0, 0
            return 1

        user32.GetWindowRect.side_effect = _fill_zero
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1
        gdi32 = MagicMock()
        gdi32.DeleteDC.return_value = 1

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("zero-size window" in l for l in lines)

    def test_desktop_bitblt_returns_false_when_getdc_null_fails(self, tmp_path):
        # Arrange
        user32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 0, 0, 100, 100
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0   # GetDC(NULL) fails
        user32.ReleaseDC.return_value = 1
        gdi32 = MagicMock()
        gdi32.DeleteDC.return_value = 1

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("GetDC(NULL) failed" in l for l in lines)

    def test_desktop_bitblt_returns_false_when_create_compatible_dc_fails(self, tmp_path):
        # Arrange
        user32 = self._make_happy_user32()
        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0   # failure
        gdi32.DeleteDC.return_value = 1

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("CreateCompatibleDC failed" in l for l in lines)

    def test_desktop_bitblt_returns_false_when_create_bitmap_fails(self, tmp_path):
        # Arrange
        user32 = self._make_happy_user32()
        gdi32 = MagicMock()
        gdi32.CreateCompatibleDC.return_value = 0x200
        gdi32.CreateCompatibleBitmap.return_value = 0  # failure
        gdi32.DeleteDC.return_value = 1
        gdi32.DeleteObject.return_value = 1

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("CreateCompatibleBitmap failed" in l for l in lines)

    def test_desktop_bitblt_returns_false_when_bitblt_fails(self, tmp_path):
        # Arrange
        user32 = self._make_happy_user32()
        gdi32 = self._make_happy_gdi32()
        gdi32.BitBlt.return_value = 0  # BitBlt failure

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("BitBlt failed" in l for l in lines)

    def test_desktop_bitblt_returns_false_when_getdibits_returns_zero(self, tmp_path):
        # Arrange
        user32 = self._make_happy_user32()
        gdi32 = self._make_happy_gdi32()
        gdi32.GetDIBits.side_effect = None
        gdi32.GetDIBits.return_value = 0  # failure

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("GetDIBits returned" in l for l in lines)

    def test_desktop_bitblt_returns_false_for_all_zero_pixels(self, tmp_path):
        # Arrange: blank image (all pixels zero) even though BitBlt succeeded
        user32 = self._make_happy_user32(width=4, height=2)
        gdi32 = self._make_happy_gdi32(width=4, height=2)

        def _getdibits_blank(_mem_dc, _bmp, _start, _nlines, pixel_buf, _bi, _mode):
            # Leave pixel_buf all-zero
            return 2

        gdi32.GetDIBits.side_effect = _getdibits_blank

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is False
        assert any("blank image detected" in l for l in lines)

    def test_desktop_bitblt_happy_path_writes_valid_png(self, tmp_path):
        # Arrange
        width, height = 4, 2
        user32 = self._make_happy_user32(width, height)
        gdi32 = self._make_happy_gdi32(width, height)
        out = tmp_path / "bitblt_cap.png"
        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, out, log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is True
        assert out.exists(), "PNG file must be written"
        assert out.stat().st_size > 0
        assert out.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"

    def test_desktop_bitblt_happy_path_logs_pixel_hash(self, tmp_path):
        # Arrange
        width, height = 4, 2
        user32 = self._make_happy_user32(width, height)
        gdi32 = self._make_happy_gdi32(width, height)
        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert result is True
        assert any("pixel_hash=" in l for l in lines)

    def test_desktop_bitblt_happy_path_logs_png_written(self, tmp_path):
        # Arrange
        width, height = 4, 2
        user32 = self._make_happy_user32(width, height)
        gdi32 = self._make_happy_gdi32(width, height)
        lines, log = _log()

        # Act
        _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert
        assert any("PNG written" in l for l in lines)

    def test_desktop_bitblt_uses_screen_origin_for_bitblt(self, tmp_path):
        """BitBlt source X/Y must be the window's screen-space left/top."""
        # Arrange: window positioned at screen (50, 80)
        width, height = 4, 2
        user32 = MagicMock()

        def _fill_rect(hwnd, byref_rect):
            r = byref_rect._obj
            r.left, r.top, r.right, r.bottom = 50, 80, 50 + width, 80 + height
            return 1

        user32.GetWindowRect.side_effect = _fill_rect
        user32.GetDC.return_value = 0x100
        user32.ReleaseDC.return_value = 1

        gdi32 = self._make_happy_gdi32(width, height)
        bitblt_calls: list[tuple] = []

        original_bitblt = gdi32.BitBlt

        def _track_bitblt(dst_dc, dst_x, dst_y, w, h, src_dc, src_x, src_y, rop):
            bitblt_calls.append((src_x, src_y))
            return 1

        gdi32.BitBlt.side_effect = _track_bitblt

        lines, log = _log()

        # Act
        _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert: BitBlt source must use window screen-space origin
        assert len(bitblt_calls) == 1, "BitBlt must be called exactly once"
        src_x, src_y = bitblt_calls[0]
        assert src_x == 50, f"BitBlt source X must be rect.left=50, got {src_x}"
        assert src_y == 80, f"BitBlt source Y must be rect.top=80, got {src_y}"

    def test_desktop_bitblt_releases_desktop_dc_on_failure(self, tmp_path):
        """ReleaseDC(None, desktop_dc) must be called even when BitBlt fails."""
        # Arrange
        width, height = 4, 2
        user32 = self._make_happy_user32(width, height)
        gdi32 = self._make_happy_gdi32(width, height)
        gdi32.BitBlt.return_value = 0  # force failure

        lines, log = _log()

        # Act
        result = _capture_hwnd_bitblt_to_png(0x1001, tmp_path / "out.png", log, user32=user32, gdi32=gdi32)

        # Assert: ReleaseDC called (finally block executed)
        assert result is False
        user32.ReleaseDC.assert_called()


# ---------------------------------------------------------------------------
# 10. TestCaptureGameWindowDesktopBitblt (PROMPT 1813) — public API tests
# ---------------------------------------------------------------------------

class TestCaptureGameWindowDesktopBitblt:
    """Unit tests for capture_game_window_desktop_bitblt (public entry point)."""

    def test_desktop_bitblt_public_noop_on_non_windows(self, monkeypatch, tmp_path):
        # Arrange
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", False)
        lines, log = _log()

        # Act
        result = capture_game_window_desktop_bitblt(tmp_path / "out.png", log)

        # Assert
        assert result is False
        assert any("non-Windows" in l for l in lines)

    def test_desktop_bitblt_public_returns_false_when_no_window_found(self, monkeypatch, tmp_path):
        # Arrange
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            monkeypatch.setattr("win_foreground._list_visible_windows", lambda _u: [])
            lines, log = _log()

            # Act
            result = capture_game_window_desktop_bitblt(tmp_path / "out.png", log)

        # Assert
        assert result is False
        assert any("no CCGS/Bevy window found" in l for l in lines)

    def test_desktop_bitblt_public_swallows_unexpected_exception(self, monkeypatch, tmp_path):
        # Arrange
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            monkeypatch.setattr(
                "win_foreground._list_visible_windows",
                lambda _u: (_ for _ in ()).throw(RuntimeError("boom")),
            )
            lines, log = _log()

            # Act
            result = capture_game_window_desktop_bitblt(tmp_path / "out.png", log)

        # Assert
        assert result is False
        assert any("unexpected error" in l for l in lines)

    def test_desktop_bitblt_public_calls_bitblt_hwnd_on_match(self, monkeypatch, tmp_path):
        # Arrange
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        fake_windows = [(0xBB01, "Lanes and Lies")]
        monkeypatch.setattr("win_foreground._list_visible_windows", lambda _u: fake_windows)

        captured_hwnds: list[int] = []

        def _fake_bitblt(hwnd, path, log, *, user32=None, gdi32=None):
            captured_hwnds.append(hwnd)
            return True

        monkeypatch.setattr(win_capture, "_capture_hwnd_bitblt_to_png", _fake_bitblt)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            lines, log = _log()

            # Act
            result = capture_game_window_desktop_bitblt(tmp_path / "out.png", log)

        # Assert
        assert result is True
        assert captured_hwnds == [0xBB01]

    def test_desktop_bitblt_public_logs_backend_prefix(self, monkeypatch, tmp_path):
        # Arrange
        monkeypatch.setattr(win_capture, "_IS_WINDOWS", True)
        fake_windows = [(0xBB01, "Lanes and Lies")]
        monkeypatch.setattr("win_foreground._list_visible_windows", lambda _u: fake_windows)
        monkeypatch.setattr(win_capture, "_capture_hwnd_bitblt_to_png", lambda *a, **k: True)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            lines, log = _log()
            capture_game_window_desktop_bitblt(tmp_path / "out.png", log)

        # Assert: all log lines must be prefixed with desktop_bitblt:
        assert any("desktop_bitblt:" in l for l in lines)


# ---------------------------------------------------------------------------
# 11. TestDriverDesktopBitbltFallback (PROMPT 1813) — driver.py wiring tests
# ---------------------------------------------------------------------------

class TestDriverDesktopBitbltFallback:
    """Structural tests: driver.py imports and wires desktop_bitblt fallback."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_imports_desktop_bitblt_capture(self):
        assert "capture_game_window_desktop_bitblt" in self._DRIVER_SOURCE, (
            "driver.py must import capture_game_window_desktop_bitblt from win_capture"
        )

    def test_driver_aliases_desktop_bitblt_capture(self):
        assert "_desktop_bitblt_capture" in self._DRIVER_SOURCE, (
            "driver.py must alias capture_game_window_desktop_bitblt as _desktop_bitblt_capture"
        )

    def test_driver_calls_desktop_bitblt_after_win32_failure(self):
        src = self._DRIVER_SOURCE
        # The fallback must be inside the "if not _win32_ok" guard
        assert "if not _win32_ok:" in src, (
            "driver.py must have 'if not _win32_ok:' guard before the BitBlt fallback"
        )
        win32_fail_idx = src.index("if not _win32_ok:")
        bitblt_call_idx = src.index("_desktop_bitblt_capture(")
        assert bitblt_call_idx > win32_fail_idx, (
            "_desktop_bitblt_capture must be called inside the 'if not _win32_ok:' block"
        )

    def test_driver_logs_desktop_bitblt_result(self):
        assert "desktop_bitblt=" in self._DRIVER_SOURCE, (
            "driver.py must log desktop_bitblt=OK/FAILED after the fallback call"
        )

    def test_driver_bitblt_shot_path_uses_tick(self):
        assert "bitblt_tick_" in self._DRIVER_SOURCE, (
            "BitBlt fallback filename must embed the tick number (bitblt_tick_...)"
        )

    def test_driver_win32_printwindow_log_uses_new_label(self):
        # PROMPT 1813 renamed the log label from "win32_capture" to "win32_printwindow"
        assert "win32_printwindow=" in self._DRIVER_SOURCE, (
            "driver.py must log win32_printwindow=OK/FAILED to distinguish from the bitblt backend"
        )

    def test_driver_bitblt_fallback_placed_after_win32_capture(self):
        src = self._DRIVER_SOURCE
        win32_idx = src.index("_win32_ok = _win32_capture(")
        bitblt_idx = src.index("_desktop_bitblt_capture(")
        assert bitblt_idx > win32_idx, (
            "desktop_bitblt fallback must appear after win32_printwindow attempt"
        )
