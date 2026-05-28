"""Windows-native window capture backend for autoplay evidence screenshots.

PROMPT 1794: Implements GDI/PrintWindow capture of the composed CCGS/Bevy
window so evidence screenshots reflect what is actually visible on screen,
rather than Bevy's GPU backbuffer (which is near-black when the window is
offscreen or not actively composited by DWM).

No external dependencies — pure stdlib ctypes + zlib + struct.

Public API:
    is_available() -> bool
    capture_game_window(output_path, log) -> bool

Internal helpers (exposed for unit-test injection):
    _capture_hwnd_to_png(hwnd, output_path, log, *, user32, gdi32) -> bool
    _write_png(path, width, height, rgb_rows) -> None

The HWND discovery reuses _list_visible_windows and _find_candidate from
win_foreground so the title-hint list stays in one place.
"""
from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path
from typing import Callable

_IS_WINDOWS = sys.platform == "win32"

# GDI / user32 constants
_PW_RENDERFULLCONTENT = 2  # PrintWindow: capture DWM-composited content (Win8+)
_DIB_RGB_COLORS = 0
_BITMAPINFOHEADER_SIZE = 40


# ---------------------------------------------------------------------------
# Minimal PNG encoder — no PIL/Pillow required
# ---------------------------------------------------------------------------

def _png_chunk(tag: bytes, data: bytes) -> bytes:
    payload = tag + data
    crc = zlib.crc32(payload) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + payload + struct.pack(">I", crc)


def _write_png(path: Path, width: int, height: int, rgb_rows: list[bytes]) -> None:
    """Write *rgb_rows* (one bytes per row, 3 bytes per pixel RGB) as a PNG."""
    ihdr_data = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    # Filter type 0 (None) prepended to every scan line
    raw_pixels = b"".join(b"\x00" + row for row in rgb_rows)
    idat_data = zlib.compress(raw_pixels, level=6)
    with path.open("wb") as fh:
        fh.write(b"\x89PNG\r\n\x1a\n")
        fh.write(_png_chunk(b"IHDR", ihdr_data))
        fh.write(_png_chunk(b"IDAT", idat_data))
        fh.write(_png_chunk(b"IEND", b""))


# ---------------------------------------------------------------------------
# GDI capture — user32/gdi32 are injectable for unit tests
# ---------------------------------------------------------------------------

def _capture_hwnd_to_png(
    hwnd: int,
    output_path: Path,
    log: Callable[[str], None],
    *,
    user32=None,
    gdi32=None,
) -> bool:
    """Capture *hwnd* to *output_path* as PNG using PrintWindow + GetDIBits.

    *user32* and *gdi32* are injected for testability; when None the real
    ctypes.windll handles are used.  Only valid to call on Windows.
    """
    import ctypes
    import ctypes.wintypes  # noqa: PLC0415

    if user32 is None:
        user32 = ctypes.windll.user32  # type: ignore[attr-defined]
    if gdi32 is None:
        gdi32 = ctypes.windll.gdi32  # type: ignore[attr-defined]

    rect = ctypes.wintypes.RECT()
    if not user32.GetWindowRect(hwnd, ctypes.byref(rect)):
        log(f"win32_capture: GetWindowRect failed hwnd={hwnd:#010x}")
        return False

    width = rect.right - rect.left
    height = rect.bottom - rect.top
    if width <= 0 or height <= 0:
        log(
            f"win32_capture: zero-size window hwnd={hwnd:#010x} "
            f"w={width} h={height}"
        )
        return False

    hwnd_dc = user32.GetDC(hwnd)
    if not hwnd_dc:
        log(f"win32_capture: GetDC failed hwnd={hwnd:#010x}")
        return False

    try:
        mem_dc = gdi32.CreateCompatibleDC(hwnd_dc)
        if not mem_dc:
            log("win32_capture: CreateCompatibleDC failed")
            return False
        try:
            bitmap = gdi32.CreateCompatibleBitmap(hwnd_dc, width, height)
            if not bitmap:
                log("win32_capture: CreateCompatibleBitmap failed")
                return False
            try:
                old_obj = gdi32.SelectObject(mem_dc, bitmap)

                # PW_RENDERFULLCONTENT captures DWM-composited window contents.
                # Fall back to flags=0 (non-DWM) if the extended flag is refused.
                ok = user32.PrintWindow(hwnd, mem_dc, _PW_RENDERFULLCONTENT)
                if not ok:
                    log(
                        "win32_capture: PrintWindow(PW_RENDERFULLCONTENT) returned 0"
                        " — retrying flags=0"
                    )
                    ok = user32.PrintWindow(hwnd, mem_dc, 0)
                if not ok:
                    log(f"win32_capture: PrintWindow failed hwnd={hwnd:#010x}")
                    return False

                # BITMAPINFOHEADER: top-down 32-bit BGRA (biHeight negative)
                bi = struct.pack(
                    "<IiiHHIIiiII",
                    _BITMAPINFOHEADER_SIZE,
                    width,
                    -height,           # negative → top-down scan order
                    1,                 # biPlanes
                    32,                # biBitCount (BGRA)
                    0,                 # biCompression (BI_RGB)
                    width * height * 4,
                    0, 0, 0, 0,
                )
                bi_buf = ctypes.create_string_buffer(bi)
                pixel_buf = (ctypes.c_uint8 * (width * height * 4))()

                lines_copied = gdi32.GetDIBits(
                    mem_dc, bitmap, 0, height, pixel_buf, bi_buf, _DIB_RGB_COLORS
                )
                if lines_copied <= 0:
                    log(f"win32_capture: GetDIBits returned {lines_copied}")
                    return False

                # Convert GDI BGRA → PNG RGB (one row at a time)
                raw = bytes(pixel_buf)
                stride = width * 4
                rgb_rows: list[bytes] = []
                for row_idx in range(height):
                    start = row_idx * stride
                    row_rgb = bytearray(width * 3)
                    for col in range(width):
                        px = start + col * 4
                        row_rgb[col * 3] = raw[px + 2]      # R
                        row_rgb[col * 3 + 1] = raw[px + 1]  # G
                        row_rgb[col * 3 + 2] = raw[px]      # B
                    rgb_rows.append(bytes(row_rgb))

                output_path.parent.mkdir(parents=True, exist_ok=True)
                _write_png(output_path, width, height, rgb_rows)
                size = output_path.stat().st_size
                log(
                    f"win32_capture: PNG written {output_path.name} "
                    f"{width}x{height} ({size} bytes)"
                )
                return True

            finally:
                if old_obj:
                    gdi32.SelectObject(mem_dc, old_obj)
                gdi32.DeleteObject(bitmap)
        finally:
            gdi32.DeleteDC(mem_dc)
    finally:
        user32.ReleaseDC(hwnd, hwnd_dc)


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------

def is_available() -> bool:
    """Return True when Win32 capture is possible (Windows platform)."""
    return _IS_WINDOWS


def capture_game_window(
    output_path: Path,
    log: Callable[[str], None],
) -> bool:
    """Capture the CCGS/Bevy game window to *output_path* as PNG.

    Finds the game window via the same title hints used in win_foreground.
    Returns True on success, False on any failure (non-Windows, no window found,
    GDI error).  All outcomes emit at least one log line.
    """
    if not _IS_WINDOWS:
        log("win32_capture: non-Windows platform — skipping")
        return False
    try:
        import ctypes  # noqa: PLC0415

        # Import here (not at module top) so the module loads on non-Windows.
        from win_foreground import _find_candidate, _list_visible_windows  # noqa: PLC0415

        user32 = ctypes.windll.user32  # type: ignore[attr-defined]
        windows = _list_visible_windows(user32)
        candidate = _find_candidate(windows)
        if candidate is None:
            log(
                f"win32_capture: no CCGS/Bevy window found among "
                f"{len(windows)} visible windows — skipping capture"
            )
            return False
        hwnd, title = candidate
        log(f"win32_capture: found window title={title!r} hwnd={hwnd:#010x}")
        return _capture_hwnd_to_png(hwnd, output_path, log, user32=user32)
    except Exception as exc:  # noqa: BLE001
        log(f"win32_capture: unexpected error — skipping: {exc}")
        return False
