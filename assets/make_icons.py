"""Generate Kalk's source app icon and tray icons.

Motif: a person rising — head dot over an upward chevron ("get up!").
"""
import sys
from PIL import Image, ImageDraw

out_dir = sys.argv[1]
S = 4096  # supersample, then downscale


def glyph(draw, size, color, inset):
    """A person rising: head dot above an upward chevron (arms/shoulders)."""
    w = size - 2 * inset
    stroke = int(w * 0.17)
    r = stroke / 2
    cx = size / 2
    head_r = w * 0.15
    head_cy = inset + head_r
    top = inset + w * 0.50
    arm_y = inset + w * 0.92
    arm_dx = w * 0.42
    draw.ellipse([cx - head_r, head_cy - head_r, cx + head_r, head_cy + head_r], fill=color)
    pts = [(cx - arm_dx, arm_y), (cx, top), (cx + arm_dx, arm_y)]
    draw.line(pts, fill=color, width=stroke, joint="curve")
    for x, y in pts:
        draw.ellipse([x - r, y - r, x + r, y + r], fill=color)


# App icon: rounded square, warm gradient, white glyph.
img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
grad = Image.new("RGBA", (S, S))
gd = ImageDraw.Draw(grad)
c1, c2 = (255, 122, 69), (232, 62, 82)  # orange → coral
for y in range(S):
    t = y / (S - 1)
    gd.line([(0, y), (S, y)], fill=tuple(int(a + (b - a) * t) for a, b in zip(c1, c2)) + (255,))
mask = Image.new("L", (S, S), 0)
ImageDraw.Draw(mask).rounded_rectangle([int(S * .1)] * 2 + [int(S * .9)] * 2, radius=int(S * .18), fill=255)
img.paste(grad, (0, 0), mask)
glyph(ImageDraw.Draw(img), S, (255, 255, 255, 255), inset=int(S * 0.27))
img.resize((1024, 1024), Image.LANCZOS).save(f"{out_dir}/app-icon.png")

# macOS tray: black glyph on transparent, used as a template image so the
# menu bar recolours it for light/dark mode.
t = Image.new("RGBA", (S, S), (0, 0, 0, 0))
glyph(ImageDraw.Draw(t), S, (0, 0, 0, 255), inset=int(S * 0.06))
t.resize((64, 64), Image.LANCZOS).save(f"{out_dir}/tray-template.png")

# Windows tray: the full-colour app icon, which reads on light and dark taskbars.
img.resize((64, 64), Image.LANCZOS).save(f"{out_dir}/tray-color.png")
print("ok")
