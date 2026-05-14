#!/usr/bin/env python3
"""Generate DevDock icons: concentric listening-pulse mark.

Produces:
  - src-tauri/icons/tray.png            monochrome template for macOS menu bar
  - src-tauri/icons/32x32.png           full-color app icon set
  - src-tauri/icons/128x128.png
  - src-tauri/icons/128x128@2x.png      (256x256)
  - src-tauri/icons/icon.png            (512x512)
  - src-tauri/icons/Square*Logo.png     (Windows store sizes; harmless on macOS)

Antialiased via 4x supersampling.
"""

from __future__ import annotations

from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

OUT_DIR = Path(__file__).resolve().parents[1] / "src-tauri" / "icons"

# Brand palette
BG_TOP = (12, 16, 24)       # #0c1018  near-black with slight blue
BG_BOTTOM = (24, 30, 42)    # #181e2a
ACCENT_CORE = (134, 239, 172)   # #86efac  lighter mint core
ACCENT_INNER = (34, 197, 94)    # #22c55e  green
ACCENT_OUTER = (22, 163, 74)    # #16a34a  darker green
GLOW = (134, 239, 172)


def supersample(size: int, sup: int = 4):
    return Image.new("RGBA", (size * sup, size * sup), (0, 0, 0, 0)), sup


def draw_pulse(
    canvas: Image.Image,
    sup: int,
    *,
    color_inner: tuple[int, int, int, int],
    color_outer: tuple[int, int, int, int],
    color_core: tuple[int, int, int, int],
    stroke_pt: float,
    core_radius_pt: float,
    inner_radius_pt: float,
    outer_radius_pt: float,
    cx: float,
    cy: float,
):
    """Draw the concentric pulse: outer ring, inner ring, solid center dot."""
    d = ImageDraw.Draw(canvas)

    def ellipse(r: float, color, width: float):
        cxs, cys = cx * sup, cy * sup
        rs = r * sup
        ws = max(1, round(width * sup))
        d.ellipse(
            (cxs - rs, cys - rs, cxs + rs, cys + rs),
            outline=color,
            width=ws,
        )

    def filled(r: float, color):
        cxs, cys = cx * sup, cy * sup
        rs = r * sup
        d.ellipse((cxs - rs, cys - rs, cxs + rs, cys + rs), fill=color)

    ellipse(outer_radius_pt, color_outer, stroke_pt)
    ellipse(inner_radius_pt, color_inner, stroke_pt)
    filled(core_radius_pt, color_core)


def make_template(size: int = 64) -> Image.Image:
    """macOS menu bar template: black silhouette with alpha, no background.

    Tauri renders this as a template image on macOS (alpha-only), so colors
    are ignored and the system applies the appropriate menu bar tint.
    """
    canvas, sup = supersample(size)

    # Optical centering: nudge up a touch so the mark sits on the menu bar baseline
    cx = size / 2.0
    cy = size / 2.0 - 0.5

    # Sizes tuned for ~22pt menu bar height
    stroke = max(1.4, size * 0.045)
    core_r = size * 0.085
    inner_r = size * 0.21
    outer_r = size * 0.36

    BLACK = (0, 0, 0, 255)
    BLACK_FAINT = (0, 0, 0, 170)  # outer ring slightly fainter for pulse feel

    draw_pulse(
        canvas,
        sup,
        color_inner=BLACK,
        color_outer=BLACK_FAINT,
        color_core=BLACK,
        stroke_pt=stroke,
        core_radius_pt=core_r,
        inner_radius_pt=inner_r,
        outer_radius_pt=outer_r,
        cx=cx,
        cy=cy,
    )

    return canvas.resize((size, size), Image.LANCZOS)


def make_app_icon(size: int) -> Image.Image:
    """Full-color app icon on a dark rounded square."""
    canvas, sup = supersample(size)
    S = size * sup

    # Rounded square background with a subtle vertical gradient.
    # Apple-style corner radius ~22.4% of side.
    radius = int(S * 0.224)
    bg = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg)
    bg_draw.rounded_rectangle((0, 0, S - 1, S - 1), radius=radius, fill=BG_TOP + (255,))

    # Vertical gradient overlay
    grad = Image.new("RGBA", (1, S))
    for y in range(S):
        t = y / max(1, S - 1)
        r = round(BG_TOP[0] + (BG_BOTTOM[0] - BG_TOP[0]) * t)
        g = round(BG_TOP[1] + (BG_BOTTOM[1] - BG_TOP[1]) * t)
        b = round(BG_TOP[2] + (BG_BOTTOM[2] - BG_TOP[2]) * t)
        grad.putpixel((0, y), (r, g, b, 255))
    grad = grad.resize((S, S))

    # Mask the gradient by the rounded square
    mask = Image.new("L", (S, S), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, S - 1, S - 1), radius=radius, fill=255)
    bg = Image.composite(grad, Image.new("RGBA", (S, S), (0, 0, 0, 0)), mask)

    canvas.paste(bg, (0, 0), bg)

    # Faint outermost glow ring
    cx = size / 2.0
    cy = size / 2.0
    stroke = max(1, size * 0.022)

    glow_layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow_layer)
    outer_glow_r = (size * 0.46) * sup
    gd.ellipse(
        (cx * sup - outer_glow_r, cy * sup - outer_glow_r,
         cx * sup + outer_glow_r, cy * sup + outer_glow_r),
        outline=ACCENT_OUTER + (90,),
        width=max(1, round(stroke * sup)),
    )
    if S >= 256:
        glow_layer = glow_layer.filter(ImageFilter.GaussianBlur(radius=S * 0.012))
    canvas = Image.alpha_composite(canvas, glow_layer)

    # Three actual rings + core
    draw_pulse(
        canvas,
        sup,
        color_inner=ACCENT_INNER + (255,),
        color_outer=ACCENT_OUTER + (220,),
        color_core=ACCENT_CORE + (255,),
        stroke_pt=size * 0.028,
        core_radius_pt=size * 0.085,
        inner_radius_pt=size * 0.21,
        outer_radius_pt=size * 0.34,
        cx=cx,
        cy=cy,
    )

    # Soft glow halo around the core
    if size >= 64:
        halo = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        hd = ImageDraw.Draw(halo)
        halo_r = (size * 0.12) * sup
        hd.ellipse(
            (cx * sup - halo_r, cy * sup - halo_r,
             cx * sup + halo_r, cy * sup + halo_r),
            fill=GLOW + (110,),
        )
        halo = halo.filter(ImageFilter.GaussianBlur(radius=S * 0.018))
        canvas = Image.alpha_composite(canvas, halo)

    # Top inner highlight for depth (subtle)
    if size >= 128:
        hl = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        hd = ImageDraw.Draw(hl)
        hd.rounded_rectangle(
            (S * 0.06, S * 0.05, S * 0.94, S * 0.5),
            radius=int(radius * 0.85),
            fill=(255, 255, 255, 14),
        )
        hl = hl.filter(ImageFilter.GaussianBlur(radius=S * 0.01))
        canvas = Image.alpha_composite(canvas, hl)

    return canvas.resize((size, size), Image.LANCZOS)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # Template menu bar icon (alpha-only black). Make it large; macOS scales down.
    tray = make_template(64)
    tray.save(OUT_DIR / "tray.png")
    print("wrote icons/tray.png (64x64 template)")

    # App icon set
    targets = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
    }
    for name, sz in targets.items():
        make_app_icon(sz).save(OUT_DIR / name)
        print(f"wrote icons/{name} ({sz}x{sz})")

    # .icns / .ico are placeholders pointing at icon.png/32x32.png so the bundle
    # config resolves. For production builds, regenerate with:
    #   pnpm tauri icon src-tauri/icons/icon.png
    (OUT_DIR / "icon.icns").write_bytes((OUT_DIR / "icon.png").read_bytes())
    (OUT_DIR / "icon.ico").write_bytes((OUT_DIR / "32x32.png").read_bytes())
    print("wrote icons/icon.icns (placeholder)")
    print("wrote icons/icon.ico (placeholder)")

    print("\nFor proper platform icons, run:")
    print("  pnpm tauri icon src-tauri/icons/icon.png")


if __name__ == "__main__":
    main()
