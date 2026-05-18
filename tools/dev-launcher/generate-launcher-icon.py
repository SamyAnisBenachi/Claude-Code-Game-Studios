#!/usr/bin/env python3
"""generate-launcher-icon.py -- regenerate the CCGS Dev Launcher .ico asset.

Output: tools/dev-launcher-app/res/ccgs-dev-launcher.ico
        Multi-size ICO (16/32/48/64/128/256 px, PNG-encoded entries).

Rendered with Pillow only -- no external downloads, no external image
editor. The committed .ico is the source of truth for the EXE build; this
script exists for reproducibility so future tweaks can regenerate it
deterministically.

Usage:
    python tools/dev-launcher/generate-launcher-icon.py
    python tools/dev-launcher/generate-launcher-icon.py --out custom.ico
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path

from PIL import Image, ImageDraw

# Dark slate background reads as a developer tool rather than a game splash;
# warm amber accent + near-white lanes echo the lane-based card battle layout
# without committing to any specific in-game art.
BG_TOP = (40, 46, 60, 255)
BG_BOTTOM = (24, 28, 38, 255)
ACCENT = (255, 168, 38, 255)
LANE = (240, 244, 252, 255)
LANE_SHADOW = (0, 0, 0, 120)

SIZES = [16, 32, 48, 64, 128, 256]


def _rounded_mask(size: int, radius: int) -> Image.Image:
    """Return a single-channel mask for a rounded square of `size`."""
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    if radius <= 0:
        draw.rectangle([0, 0, size, size], fill=255)
    else:
        draw.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=255)
    return mask


def _vertical_gradient(size: int, top: tuple, bottom: tuple) -> Image.Image:
    """Render a top->bottom vertical gradient at `size`x`size`."""
    grad = Image.new("RGBA", (1, size), 0)
    for y in range(size):
        t = y / max(1, size - 1)
        r = int(top[0] + (bottom[0] - top[0]) * t)
        g = int(top[1] + (bottom[1] - top[1]) * t)
        b = int(top[2] + (bottom[2] - top[2]) * t)
        a = int(top[3] + (bottom[3] - top[3]) * t)
        grad.putpixel((0, y), (r, g, b, a))
    return grad.resize((size, size))


def render_launcher_icon(size: int) -> Image.Image:
    """Render the launcher icon at the requested square size."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))

    # Background: vertical gradient, optionally rounded. 16 px stays square so
    # the rounding does not collapse to mush.
    radius = 0 if size <= 16 else int(round(size * 0.18))
    bg = _vertical_gradient(size, BG_TOP, BG_BOTTOM)
    mask = _rounded_mask(size, radius)
    img.paste(bg, (0, 0), mask)

    draw = ImageDraw.Draw(img)

    # Three vertical lanes -- mirrors the lane-based card battle layout.
    lane_count = 3
    lane_w = max(1, int(round(size * 0.15)))
    lane_gap = max(1, int(round(size * 0.07)))
    total_w = lane_w * lane_count + lane_gap * (lane_count - 1)
    start_x = int(round((size - total_w) / 2))
    lane_top = int(round(size * 0.18))
    lane_bot = int(round(size * 0.72))
    lane_h = lane_bot - lane_top
    shadow_off = max(1, int(round(size * 0.015)))
    for i in range(lane_count):
        lx = start_x + i * (lane_w + lane_gap)
        draw.rectangle(
            [lx + shadow_off, lane_top + shadow_off,
             lx + shadow_off + lane_w, lane_top + shadow_off + lane_h],
            fill=LANE_SHADOW,
        )
        draw.rectangle(
            [lx, lane_top, lx + lane_w, lane_top + lane_h],
            fill=LANE,
        )

    # Amber accent bar across the lower third -- "ready to play" cue.
    bar_h = max(1, int(round(size * 0.11)))
    bar_y = size - bar_h - max(0, int(round(size * 0.06)))
    bar_x = int(round(size * 0.18))
    bar_w = size - 2 * bar_x
    draw.rectangle([bar_x, bar_y, bar_x + bar_w, bar_y + bar_h], fill=ACCENT)

    return img


def write_ico(out_path: Path, sizes: list[int]) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    base = render_launcher_icon(max(sizes))
    # Pillow's ICO encoder accepts a `sizes` kwarg and writes one PNG-encoded
    # ICONDIRENTRY per requested size, downsampled from the supplied base.
    base.save(
        out_path,
        format="ICO",
        sizes=[(s, s) for s in sizes],
        bitmap_format="png",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Destination .ico path. Defaults to tools/dev-launcher-app/res/ccgs-dev-launcher.ico",
    )
    args = parser.parse_args()

    if args.out is None:
        repo_root = Path(__file__).resolve().parents[2]
        out_path = repo_root / "tools" / "dev-launcher-app" / "res" / "ccgs-dev-launcher.ico"
    else:
        out_path = args.out.resolve()

    write_ico(out_path, SIZES)
    size_bytes = os.path.getsize(out_path)
    print(
        f"Wrote {out_path} ({size_bytes} bytes, "
        f"{len(SIZES)} sizes: {', '.join(str(s) for s in SIZES)})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
