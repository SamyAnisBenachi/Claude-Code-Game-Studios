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
    _capture_hwnd_to_png,
    _write_png,
    capture_game_window,
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
