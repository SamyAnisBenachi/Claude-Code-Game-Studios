"""Windows-native window capture backend for autoplay evidence screenshots.

PROMPT 1794: Implements GDI/PrintWindow capture of the composed CCGS/Bevy
window so evidence screenshots reflect what is actually visible on screen,
rather than Bevy's GPU backbuffer (which is near-black when the window is
offscreen or not actively composited by DWM).

PROMPT 1813: Adds desktop BitBlt fallback backend that captures composed desktop
pixels via GetDC(NULL) + BitBlt over the window screen rect.  This avoids
PrintWindow/DWM stale-buffer issues seen in PROMPT 1807/1809 live verifies.

No external dependencies — pure stdlib ctypes + zlib + struct.

Public API:
    is_available() -> bool
    capture_game_window(output_path, log) -> bool
    capture_game_window_desktop_bitblt(output_path, log) -> bool

Internal helpers (exposed for unit-test injection):
    _capture_hwnd_to_png(hwnd, output_path, log, *, user32, gdi32) -> bool
    _capture_hwnd_bitblt_to_png(hwnd, output_path, log, *, user32, gdi32) -> bool
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
_SRCCOPY = 0x00CC0020  # BitBlt raster-op: direct copy from source to destination

# ShowWindow / SetForegroundWindow constants
_SW_RESTORE = 9          # Restores a minimised window to its normal size/position
_SW_SHOWNOACTIVATE = 4   # Displays a window without activating it


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

    # Restore / foreground the window before capture so DWM composites it.
    sw_ret = user32.ShowWindow(hwnd, _SW_RESTORE)
    log(f"win32_capture: ShowWindow ret={sw_ret} hwnd={hwnd:#010x}")
    sfg_ret = user32.SetForegroundWindow(hwnd)
    log(f"win32_capture: SetForegroundWindow ret={sfg_ret} hwnd={hwnd:#010x}")

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
                pixel_hash = zlib.adler32(raw[:min(4096, len(raw))]) & 0xFFFFFFFF
                log(
                    f"win32_capture: pixel_hash={pixel_hash:#010x} "
                    f"width={width} height={height}"
                )
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
# Desktop BitBlt capture — GetDC(NULL) over the window screen rect
# ---------------------------------------------------------------------------

def _capture_hwnd_bitblt_to_png(
    hwnd: int,
    output_path: Path,
    log: Callable[[str], None],
    *,
    user32=None,
    gdi32=None,
) -> bool:
    """Capture *hwnd* via desktop BitBlt to *output_path* as PNG.

    Uses ``GetDC(NULL)`` (the desktop DC) as the source and ``BitBlt`` over the
    window's screen-space rectangle.  This reads composed desktop pixels
    directly from DWM's output, avoiding the PrintWindow stale-buffer issue
    where ``PrintWindow`` returns a frozen frame for backgrounded windows.

    Failure modes logged with prefix ``desktop_bitblt:``:
    - Invalid / minimised window rect  → "zero-size window"
    - ``GetDC(NULL)`` failure          → "GetDC(NULL) failed"
    - ``CreateCompatibleDC`` failure   → "CreateCompatibleDC failed"
    - ``CreateCompatibleBitmap``       → "CreateCompatibleBitmap failed"
    - ``BitBlt`` returns 0             → "BitBlt failed"
    - ``GetDIBits`` returns <= 0       → "GetDIBits returned N"
    - All pixels zero                  → "blank image detected"

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
        log(f"desktop_bitblt: GetWindowRect failed hwnd={hwnd:#010x}")
        return False

    width = rect.right - rect.left
    height = rect.bottom - rect.top
    if width <= 0 or height <= 0:
        log(
            f"desktop_bitblt: zero-size window hwnd={hwnd:#010x} "
            f"w={width} h={height} — window may be minimised"
        )
        return False

    # GetDC(NULL) acquires the desktop DC which contains composed screen pixels.
    desktop_dc = user32.GetDC(None)
    if not desktop_dc:
        log("desktop_bitblt: GetDC(NULL) failed — cannot acquire desktop DC")
        return False

    try:
        mem_dc = gdi32.CreateCompatibleDC(desktop_dc)
        if not mem_dc:
            log("desktop_bitblt: CreateCompatibleDC failed")
            return False
        try:
            # Bitmap must be compatible with the desktop DC (not mem_dc) so
            # its colour depth matches the screen surface.
            bitmap = gdi32.CreateCompatibleBitmap(desktop_dc, width, height)
            if not bitmap:
                log("desktop_bitblt: CreateCompatibleBitmap failed")
                return False
            try:
                old_obj = gdi32.SelectObject(mem_dc, bitmap)

                # BitBlt from desktop at the window's screen position.
                ok = gdi32.BitBlt(
                    mem_dc, 0, 0, width, height,
                    desktop_dc, rect.left, rect.top,
                    _SRCCOPY,
                )
                if not ok:
                    log(
                        f"desktop_bitblt: BitBlt failed hwnd={hwnd:#010x} "
                        f"src=({rect.left},{rect.top}) size=({width}x{height})"
                    )
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
                    log(f"desktop_bitblt: GetDIBits returned {lines_copied}")
                    return False

                raw = bytes(pixel_buf)

                # Detect blank (all-zero) image — indicates a capture failure
                # even when all API calls returned success.
                if not any(raw):
                    log(
                        f"desktop_bitblt: blank image detected hwnd={hwnd:#010x} "
                        f"({width}x{height}) — all pixels are zero"
                    )
                    return False

                pixel_hash = zlib.adler32(raw[:min(4096, len(raw))]) & 0xFFFFFFFF
                log(
                    f"desktop_bitblt: pixel_hash={pixel_hash:#010x} "
                    f"width={width} height={height}"
                )

                # Convert GDI BGRA → PNG RGB
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
                    f"desktop_bitblt: PNG written {output_path.name} "
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
        user32.ReleaseDC(None, desktop_dc)


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

    Backend: win32_printwindow (PrintWindow + GetDIBits).
    Finds the game window via the same title hints used in win_foreground.
    Returns True on success, False on any failure (non-Windows, no window found,
    GDI error).  All outcomes emit at least one log line.
    """
    if not _IS_WINDOWS:
        log("win32_printwindow: non-Windows platform — skipping")
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
                f"win32_printwindow: no CCGS/Bevy window found among "
                f"{len(windows)} visible windows — skipping capture"
            )
            return False
        hwnd, title = candidate
        log(f"win32_printwindow: found window title={title!r} hwnd={hwnd:#010x}")
        return _capture_hwnd_to_png(hwnd, output_path, log, user32=user32)
    except Exception as exc:  # noqa: BLE001
        log(f"win32_printwindow: unexpected error — skipping: {exc}")
        return False


def capture_game_window_desktop_bitblt(
    output_path: Path,
    log: Callable[[str], None],
) -> bool:
    """Capture the CCGS/Bevy game window using desktop BitBlt.

    Backend: desktop_bitblt (GetDC(NULL) + BitBlt over screen rect).
    Captures composed desktop pixels directly from the screen DC, bypassing
    PrintWindow/DWM stale-buffer issues.  Use as a fallback when
    win32_printwindow produces frozen or byte-identical frames.

    Returns True on success, False on any failure.  All outcomes emit at least
    one log line with the prefix ``desktop_bitblt:``.
    """
    if not _IS_WINDOWS:
        log("desktop_bitblt: non-Windows platform — skipping")
        return False
    try:
        import ctypes  # noqa: PLC0415

        from win_foreground import _find_candidate, _list_visible_windows  # noqa: PLC0415

        user32 = ctypes.windll.user32  # type: ignore[attr-defined]
        windows = _list_visible_windows(user32)
        candidate = _find_candidate(windows)
        if candidate is None:
            log(
                f"desktop_bitblt: no CCGS/Bevy window found among "
                f"{len(windows)} visible windows — skipping capture"
            )
            return False
        hwnd, title = candidate
        log(f"desktop_bitblt: found window title={title!r} hwnd={hwnd:#010x}")
        return _capture_hwnd_bitblt_to_png(hwnd, output_path, log, user32=user32)
    except Exception as exc:  # noqa: BLE001
        log(f"desktop_bitblt: unexpected error — skipping: {exc}")
        return False
