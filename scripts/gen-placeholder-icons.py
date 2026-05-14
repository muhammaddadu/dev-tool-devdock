#!/usr/bin/env python3
"""Generate placeholder PNG icons for Tauri bundle config.

These are intentionally minimal solid-color rounded squares so `pnpm tauri dev`
and `pnpm tauri build` resolve. Replace with real icons by running:

    pnpm tauri icon src-tauri/icons/icon.svg

once you have a real source asset.
"""

from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

OUT_DIR = Path(__file__).resolve().parents[1] / "src-tauri" / "icons"

# Brand palette (sRGB)
BG = (11, 13, 18)          # #0b0d12
PANEL = (31, 41, 55)       # #1f2937
DOT_GREEN = (34, 197, 94)  # #22c55e
DOT_AMBER = (245, 158, 11) # #f59e0b
DOT_RED = (239, 68, 68)    # #ef4444


def write_png(path: Path, size: int) -> None:
    """Write a simple PNG: rounded background, panel, three traffic-light dots."""
    radius = max(2, size // 8)
    panel_pad_x = max(1, size // 6)
    panel_pad_y = max(1, size // 4)
    panel_radius = max(1, size // 16)

    pixels = bytearray(size * size * 4)

    def set_px(x: int, y: int, rgb: tuple[int, int, int]) -> None:
        if 0 <= x < size and 0 <= y < size:
            i = (y * size + x) * 4
            pixels[i] = rgb[0]
            pixels[i + 1] = rgb[1]
            pixels[i + 2] = rgb[2]
            pixels[i + 3] = 255

    def in_rounded_rect(x: int, y: int, x0: int, y0: int, x1: int, y1: int, r: int) -> bool:
        if x < x0 or x > x1 or y < y0 or y > y1:
            return False
        # Corner tests
        for cx, cy in ((x0 + r, y0 + r), (x1 - r, y0 + r), (x0 + r, y1 - r), (x1 - r, y1 - r)):
            if (x < cx) and (y < cy) and (cx == x0 + r) and (cy == y0 + r):
                return (x - cx) ** 2 + (y - cy) ** 2 <= r * r
            if (x > cx) and (y < cy) and (cx == x1 - r) and (cy == y0 + r):
                return (x - cx) ** 2 + (y - cy) ** 2 <= r * r
            if (x < cx) and (y > cy) and (cx == x0 + r) and (cy == y1 - r):
                return (x - cx) ** 2 + (y - cy) ** 2 <= r * r
            if (x > cx) and (y > cy) and (cx == x1 - r) and (cy == y1 - r):
                return (x - cx) ** 2 + (y - cy) ** 2 <= r * r
        return True

    # Background rounded rect
    for y in range(size):
        for x in range(size):
            if in_rounded_rect(x, y, 0, 0, size - 1, size - 1, radius):
                set_px(x, y, BG)

    # Panel
    px0, py0 = panel_pad_x, panel_pad_y
    px1, py1 = size - 1 - panel_pad_x, size - 1 - panel_pad_y
    for y in range(py0, py1 + 1):
        for x in range(px0, px1 + 1):
            if in_rounded_rect(x, y, px0, py0, px1, py1, panel_radius):
                set_px(x, y, PANEL)

    # Three dots (only on sizes >= 32 to stay legible)
    if size >= 32:
        dot_r = max(2, size // 28)
        cy = py0 + max(2, size // 14)
        spacing = max(4, size // 12)
        cx_base = px0 + max(2, size // 14)
        for i, color in enumerate((DOT_GREEN, DOT_AMBER, DOT_RED)):
            cx = cx_base + i * spacing
            for y in range(cy - dot_r, cy + dot_r + 1):
                for x in range(cx - dot_r, cx + dot_r + 1):
                    if (x - cx) ** 2 + (y - cy) ** 2 <= dot_r * dot_r:
                        set_px(x, y, color)

    # Encode PNG manually (no PIL dependency)
    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)

    raw = bytearray()
    stride = size * 4
    for y in range(size):
        raw.append(0)
        raw += pixels[y * stride : (y + 1) * stride]
    idat = zlib.compress(bytes(raw), 9)

    png = sig + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")
    path.write_bytes(png)


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    targets = [
        (OUT_DIR / "32x32.png", 32),
        (OUT_DIR / "128x128.png", 128),
        (OUT_DIR / "128x128@2x.png", 256),
        (OUT_DIR / "icon.png", 512),
    ]
    for path, size in targets:
        write_png(path, size)
        print(f"wrote {path.relative_to(OUT_DIR.parents[1])} ({size}x{size})")

    # Placeholder .icns / .ico — Tauri will still warn but bundle paths exist.
    # Real builds should run: `pnpm tauri icon src-tauri/icons/icon.svg`
    (OUT_DIR / "icon.icns").write_bytes((OUT_DIR / "icon.png").read_bytes())
    (OUT_DIR / "icon.ico").write_bytes((OUT_DIR / "32x32.png").read_bytes())
    print("wrote icon.icns (placeholder = icon.png)")
    print("wrote icon.ico (placeholder = 32x32.png)")
    print("\nFor real icons, run: pnpm tauri icon src-tauri/icons/icon.svg")
    return 0


if __name__ == "__main__":
    sys.exit(main())
