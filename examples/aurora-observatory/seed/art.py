"""Procedural aurora artwork. Pure stdlib: writes SVG, rsvg-convert + ImageMagick make JPEGs.

Every image is deterministic for a given (name, seed), so the build is repeatable.
"""

from __future__ import annotations

import math
import random
import subprocess
from pathlib import Path

PALETTES = {
    "emerald": ["#37ffb0", "#1de9b6", "#00b8d4", "#7c4dff"],
    "violet": ["#b388ff", "#7c4dff", "#40c4ff", "#ff80ab"],
    "crimson": ["#ff5c8a", "#ff8a65", "#b388ff", "#37ffb0"],
    "glacier": ["#80deea", "#4dd0e1", "#82b1ff", "#b9f6ca"],
    "solar": ["#ffd166", "#ff8a65", "#ff5c8a", "#37ffb0"],
    "royal": ["#536dfe", "#7c4dff", "#18ffff", "#69f0ae"],
}
SKIES = {
    "emerald": ("#01040a", "#06222c", "#0b3b3a"),
    "violet": ("#030114", "#170a3c", "#2a1257"),
    "crimson": ("#0a0210", "#2a0a2e", "#4a1038"),
    "glacier": ("#010610", "#0a2440", "#124a66"),
    "solar": ("#060109", "#1f0f33", "#3a1544"),
    "royal": ("#01020d", "#0a1240", "#13247a"),
}


def _ridge(rng, w, base, amp, rough=0.55, steps=8):
    pts = [(0, base + rng.uniform(-amp, amp)), (w, base + rng.uniform(-amp, amp))]
    disp = amp
    for _ in range(steps):
        nxt = []
        for (x1, y1), (x2, y2) in zip(pts, pts[1:]):
            nxt.append((x1, y1))
            nxt.append(((x1 + x2) / 2, (y1 + y2) / 2 + rng.uniform(-disp, disp)))
        nxt.append(pts[-1])
        pts = nxt
        disp *= rough
    return pts


def _poly(pts, h):
    d = "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts)
    return d + f" L{pts[-1][0]:.1f},{h} L{pts[0][0]:.1f},{h} Z"


def _curtain(rng, w, horizon, color_a, color_b, idx, strength):
    """One aurora curtain: a glowing band + vertical rays along a wavy spine."""
    n = 40
    phase = rng.uniform(0, math.tau)
    freq = rng.uniform(0.8, 2.2)
    amp = rng.uniform(0.05, 0.14) * horizon
    base = horizon * rng.uniform(0.28, 0.55)
    height = horizon * rng.uniform(0.35, 0.6)
    tilt = rng.uniform(-0.12, 0.12) * horizon
    spine = []
    for i in range(n + 1):
        x = -w * 0.05 + (w * 1.1) * i / n
        t = i / n
        y = base + tilt * (t - 0.5) + amp * math.sin(phase + t * freq * math.tau) + amp * 0.4 * math.sin(phase * 1.7 + t * freq * 2.9 * math.tau)
        spine.append((x, y))
    top = " L".join(f"{x:.1f},{y - height * 0.15:.1f}" for x, y in spine)
    bot = " L".join(f"{x:.1f},{y + height:.1f}" for x, y in reversed(spine))
    band = f"M{top} L{bot} Z"
    gid = f"cg{idx}"
    rays = []
    for i in range(1, n * 2):
        t = i / (n * 2)
        x = -w * 0.05 + (w * 1.1) * t
        ys = spine[min(n, int(t * n))][1]
        length = height * rng.uniform(0.5, 1.15)
        op = rng.uniform(0.12, 0.5) * strength
        sw = rng.uniform(1.5, 6.5)
        rays.append(
            f'<line x1="{x:.1f}" y1="{ys - height * 0.1:.1f}" x2="{x + rng.uniform(-6, 6):.1f}" y2="{ys + length:.1f}" stroke="url(#{gid}r)" stroke-width="{sw:.1f}" opacity="{op:.2f}"/>'
        )
    defs = f"""
    <linearGradient id="{gid}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="{color_a}" stop-opacity="0"/>
      <stop offset="0.18" stop-color="{color_a}" stop-opacity="{0.55 * strength:.2f}"/>
      <stop offset="0.45" stop-color="{color_b}" stop-opacity="{0.35 * strength:.2f}"/>
      <stop offset="1" stop-color="{color_b}" stop-opacity="0"/>
    </linearGradient>
    <linearGradient id="{gid}r" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="{color_a}" stop-opacity="0"/>
      <stop offset="0.2" stop-color="#ffffff" stop-opacity="0.85"/>
      <stop offset="0.5" stop-color="{color_a}" stop-opacity="0.7"/>
      <stop offset="1" stop-color="{color_b}" stop-opacity="0"/>
    </linearGradient>"""
    layer = f"""
    <g filter="url(#blurBig)"><path d="{band}" fill="url(#{gid})"/></g>
    <g filter="url(#blurMid)">{''.join(rays)}</g>"""
    return defs, layer


def _motif(kind, rng, w, h, horizon, accent, glow):
    if kind == "pines":
        out = []
        for i in range(rng.randint(14, 22)):
            x = rng.uniform(-20, w + 20)
            s = rng.uniform(0.6, 1.8) * h / 9
            y = horizon + rng.uniform(0, (h - horizon) * 0.55)
            tiers = 4
            parts = []
            for t in range(tiers):
                ww = s * (0.55 + t * 0.32)
                yy = y - s * 2.8 + t * s * 0.7
                parts.append(f"M{x - ww:.1f},{yy + s * 0.95:.1f} L{x:.1f},{yy - s * 0.4:.1f} L{x + ww:.1f},{yy + s * 0.95:.1f} Z")
            out.append(f'<path d="{" ".join(parts)}" fill="#02060a"/>')
        return "".join(out)
    if kind == "cabin":
        cx = rng.uniform(w * 0.3, w * 0.7)
        cy = horizon + (h - horizon) * 0.55
        s = h / 7
        return (
            f'<rect x="{cx - s:.1f}" y="{cy - s * 0.75:.1f}" width="{s * 2:.1f}" height="{s * 1.1:.1f}" fill="#04080d"/>'
            f'<path d="M{cx - s * 1.25:.1f},{cy - s * 0.7:.1f} L{cx:.1f},{cy - s * 1.6:.1f} L{cx + s * 1.25:.1f},{cy - s * 0.7:.1f} Z" fill="#02050a"/>'
            f'<rect x="{cx - s * 0.45:.1f}" y="{cy - s * 0.45:.1f}" width="{s * 0.5:.1f}" height="{s * 0.45:.1f}" fill="#ffd27a" filter="url(#blurSm)"/>'
            f'<rect x="{cx - s * 0.45:.1f}" y="{cy - s * 0.45:.1f}" width="{s * 0.5:.1f}" height="{s * 0.45:.1f}" fill="#ffe7b0"/>'
            f'<ellipse cx="{cx:.1f}" cy="{cy + s * 0.45:.1f}" rx="{s * 2.4:.1f}" ry="{s * 0.28:.1f}" fill="#ffd27a" opacity="0.10"/>'
        )
    if kind == "tent":
        out = []
        for i in range(rng.randint(2, 3)):
            cx = rng.uniform(w * 0.15, w * 0.85)
            cy = horizon + (h - horizon) * rng.uniform(0.45, 0.8)
            s = h / rng.uniform(9, 13)
            col = rng.choice(["#ff9e4a", "#ffb86b", "#7dffcf"])
            out.append(
                f'<path d="M{cx - s:.1f},{cy:.1f} L{cx:.1f},{cy - s * 1.25:.1f} L{cx + s:.1f},{cy:.1f} Z" fill="{col}" opacity="0.16" filter="url(#blurMid)"/>'
                f'<path d="M{cx - s:.1f},{cy:.1f} L{cx:.1f},{cy - s * 1.25:.1f} L{cx + s:.1f},{cy:.1f} Z" fill="#03070c"/>'
                f'<path d="M{cx - s * 0.35:.1f},{cy:.1f} L{cx:.1f},{cy - s * 0.65:.1f} L{cx + s * 0.35:.1f},{cy:.1f} Z" fill="{col}" opacity="0.85"/>'
            )
        return "".join(out)
    if kind == "dome":
        cx = rng.uniform(w * 0.35, w * 0.65)
        cy = horizon + (h - horizon) * 0.35
        r = h / 9
        return (
            f'<rect x="{cx - r * 1.1:.1f}" y="{cy:.1f}" width="{r * 2.2:.1f}" height="{r * 0.9:.1f}" fill="#03070c"/>'
            f'<path d="M{cx - r:.1f},{cy:.1f} A{r:.1f},{r:.1f} 0 0 1 {cx + r:.1f},{cy:.1f} Z" fill="#05090f" stroke="#1a2c3a" stroke-width="2"/>'
            f'<path d="M{cx - r * 0.12:.1f},{cy - r * 0.98:.1f} L{cx + r * 0.12:.1f},{cy - r * 0.98:.1f} L{cx + r * 0.2:.1f},{cy:.1f} L{cx - r * 0.2:.1f},{cy:.1f} Z" fill="{accent}" opacity="0.65"/>'
            f'<line x1="{cx:.1f}" y1="{cy - r:.1f}" x2="{cx + r * 0.3:.1f}" y2="{cy - r * 5:.1f}" stroke="{glow}" stroke-width="2" opacity="0.7" filter="url(#blurSm)"/>'
        )
    if kind == "sled":
        out = []
        for i in range(3):
            x = w * (0.3 + i * 0.16)
            y = horizon + (h - horizon) * (0.5 + 0.06 * i)
            s = h / 26
            out.append(f'<ellipse cx="{x:.1f}" cy="{y:.1f}" rx="{s * 1.6:.1f}" ry="{s * 0.6:.1f}" fill="#03060a"/><circle cx="{x + s * 1.3:.1f}" cy="{y - s * 0.3:.1f}" r="{s * 0.5:.1f}" fill="#03060a"/>')
        out.append(f'<line x1="{w * 0.2:.1f}" y1="{horizon + (h - horizon) * 0.6:.1f}" x2="{w * 0.85:.1f}" y2="{horizon + (h - horizon) * 0.66:.1f}" stroke="#d9f3ff" stroke-width="1.2" opacity="0.25"/>')
        return "".join(out)
    if kind == "ice":
        out = []
        for i in range(rng.randint(5, 9)):
            x = rng.uniform(0, w)
            ww = rng.uniform(40, 130) * (w / 1600)
            hh = rng.uniform(60, 190) * (h / 900)
            y = h
            out.append(
                f'<path d="M{x:.1f},{y:.1f} L{x + ww * 0.2:.1f},{y - hh:.1f} L{x + ww * 0.55:.1f},{y - hh * 0.6:.1f} L{x + ww:.1f},{y:.1f} Z" fill="#0a1c2a" stroke="{glow}" stroke-width="1.5" stroke-opacity="0.35"/>'
            )
        return "".join(out)
    return ""


def scene(name: str, w: int, h: int, seed: int, palette: str = "emerald", motif: str = "pines", moon: bool = False, reflect: bool = False, intensity: float = 1.0) -> str:
    rng = random.Random(f"{name}:{seed}")
    colors = PALETTES[palette]
    sky = SKIES[palette]
    horizon = h * rng.uniform(0.62, 0.72)
    defs, layers = [], []
    for i in range(rng.randint(3, 5)):
        ca = colors[i % len(colors)]
        cb = colors[(i + 1 + rng.randint(0, 2)) % len(colors)]
        d, l = _curtain(rng, w, horizon, ca, cb, i, rng.uniform(0.55, 1.0) * intensity)
        defs.append(d)
        layers.append(l)
    stars = []
    for _ in range(int(w * h / 4200)):
        x, y = rng.uniform(0, w), rng.uniform(0, horizon * 0.98)
        r = rng.choice([0.5, 0.6, 0.7, 0.9, 1.1, 1.5])
        o = rng.uniform(0.25, 0.95)
        stars.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r}" fill="#fff" opacity="{o:.2f}"/>')
    for _ in range(int(w * h / 90000) + 3):
        x, y = rng.uniform(0, w), rng.uniform(0, horizon * 0.7)
        stars.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="2.4" fill="#cfe9ff" opacity="0.9" filter="url(#blurSm)"/>')
    moon_svg = ""
    if moon:
        mx, my, mr = w * rng.uniform(0.6, 0.85), h * rng.uniform(0.1, 0.25), h * 0.035
        moon_svg = f'<circle cx="{mx:.1f}" cy="{my:.1f}" r="{mr * 5:.1f}" fill="#bfe6ff" opacity="0.08" filter="url(#blurBig)"/><circle cx="{mx:.1f}" cy="{my:.1f}" r="{mr:.1f}" fill="#f4fbff"/><circle cx="{mx + mr * 0.35:.1f}" cy="{my - mr * 0.1:.1f}" r="{mr * 0.9:.1f}" fill="{sky[0]}" opacity="0.0"/>'
    back = _ridge(rng, w, horizon - h * 0.04, h * 0.09, rough=0.52)
    mid = _ridge(rng, w, horizon + h * 0.02, h * 0.06, rough=0.5)
    front = _ridge(rng, w, horizon + h * 0.1, h * 0.035, rough=0.5)
    # Snow-lit highlight on the far ridge (thin offset polygon).
    glow = colors[0]
    accent = colors[1]
    motif_svg = _motif(motif, rng, w, h, horizon, accent, glow)
    sky_group = f"""
    <rect width="{w}" height="{h}" fill="url(#sky)"/>
    {''.join(stars)}{moon_svg}
    {''.join(layers)}
    <rect x="0" y="{horizon - h * 0.08:.1f}" width="{w}" height="{h * 0.2:.1f}" fill="url(#haze)"/>
    <path d="{_poly(back, h)}" fill="#0a1a26"/>
    <path d="{_poly([(x, y + 4) for x, y in back], h)}" fill="#050d14"/>
    <path d="{_poly(mid, h)}" fill="#050b11"/>
    <path d="{_poly(front, h)}" fill="#02060a"/>
    {motif_svg}"""
    reflection = ""
    if reflect:
        reflection = f'<g transform="translate(0,{2 * horizon + h * 0.04:.1f}) scale(1,-1)" opacity="0.45" filter="url(#blurLake)" clip-path="url(#lake)"><use href="#skyscene"/></g>'
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="{sky[0]}"/><stop offset="0.62" stop-color="{sky[1]}"/><stop offset="1" stop-color="{sky[2]}"/>
    </linearGradient>
    <linearGradient id="haze" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="{colors[0]}" stop-opacity="0"/><stop offset="0.6" stop-color="{colors[0]}" stop-opacity="0.16"/><stop offset="1" stop-color="{colors[1]}" stop-opacity="0"/>
    </linearGradient>
    <radialGradient id="vig" cx="0.5" cy="0.45" r="0.85">
      <stop offset="0.55" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity="0.65"/>
    </radialGradient>
    <filter id="blurBig" x="-20%" y="-30%" width="140%" height="160%"><feGaussianBlur stdDeviation="{w / 60:.1f}"/></filter>
    <filter id="blurMid" x="-10%" y="-20%" width="120%" height="140%"><feGaussianBlur stdDeviation="{w / 420:.1f}"/></filter>
    <filter id="blurSm"><feGaussianBlur stdDeviation="{max(1.5, w / 600):.1f}"/></filter>
    <filter id="blurLake"><feGaussianBlur stdDeviation="0 {h / 160:.1f}"/></filter>
    <clipPath id="lake"><rect x="0" y="{horizon + h * 0.05:.1f}" width="{w}" height="{h}"/></clipPath>
    {''.join(defs)}
    <g id="skyscene">{sky_group}</g>
  </defs>
  <use href="#skyscene"/>
  {reflection}
  <rect width="{w}" height="{h}" fill="url(#vig)"/>
</svg>"""
    return svg


def render(svg: str, out: Path, quality: int = 84) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    png = out.with_suffix(".png")
    subprocess.run(["rsvg-convert", "-o", str(png)], input=svg.encode(), check=True)
    subprocess.run(["magick", str(png), "-strip", "-interlace", "Plane", "-quality", str(quality), str(out)], check=True)
    png.unlink()


def portrait(name: str, seed: int, palette: str, skin: str, hair: str, hair_style: str, garment: str, beanie: str | None = None, size: int = 800) -> str:
    rng = random.Random(f"{name}:{seed}")
    colors = PALETTES[palette]
    sky = SKIES[palette]
    s = size
    cx = s / 2
    hy = s * 0.42
    r = s * 0.16
    # shoulders
    shoulders = f'<path d="M{s * 0.12:.0f},{s} C{s * 0.12:.0f},{s * 0.72} {s * 0.3:.0f},{s * 0.66} {cx:.0f},{s * 0.66} C{s * 0.7:.0f},{s * 0.66} {s * 0.88:.0f},{s * 0.72} {s * 0.88:.0f},{s} Z" fill="{garment}"/>'
    collar = f'<path d="M{cx - r * 0.75:.0f},{s * 0.665:.0f} L{cx:.0f},{s * 0.76:.0f} L{cx + r * 0.75:.0f},{s * 0.665:.0f} Z" fill="{skin}" opacity="0.9"/>'
    neck = f'<rect x="{cx - r * 0.42:.0f}" y="{hy + r * 0.6:.0f}" width="{r * 0.84:.0f}" height="{s * 0.2:.0f}" rx="{r * 0.3:.0f}" fill="{skin}"/><rect x="{cx - r * 0.42:.0f}" y="{hy + r * 0.6:.0f}" width="{r * 0.84:.0f}" height="{s * 0.06:.0f}" fill="#000" opacity="0.18"/>'
    head = f'<ellipse cx="{cx:.0f}" cy="{hy:.0f}" rx="{r * 0.92:.0f}" ry="{r * 1.12:.0f}" fill="{skin}"/>'
    ears = f'<ellipse cx="{cx - r * 0.92:.0f}" cy="{hy + 6:.0f}" rx="{r * 0.14:.0f}" ry="{r * 0.24:.0f}" fill="{skin}"/><ellipse cx="{cx + r * 0.92:.0f}" cy="{hy + 6:.0f}" rx="{r * 0.14:.0f}" ry="{r * 0.24:.0f}" fill="{skin}"/>'
    eyes = (
        f'<ellipse cx="{cx - r * 0.36:.0f}" cy="{hy - r * 0.05:.0f}" rx="{r * 0.09:.0f}" ry="{r * 0.06:.0f}" fill="#10161d"/>'
        f'<ellipse cx="{cx + r * 0.36:.0f}" cy="{hy - r * 0.05:.0f}" rx="{r * 0.09:.0f}" ry="{r * 0.06:.0f}" fill="#10161d"/>'
        f'<path d="M{cx - r * 0.5:.0f},{hy - r * 0.26:.0f} q{r * 0.14:.0f},-{r * 0.1:.0f} {r * 0.28:.0f},0" stroke="{hair}" stroke-width="{r * 0.06:.0f}" fill="none" stroke-linecap="round"/>'
        f'<path d="M{cx + r * 0.22:.0f},{hy - r * 0.26:.0f} q{r * 0.14:.0f},-{r * 0.1:.0f} {r * 0.28:.0f},0" stroke="{hair}" stroke-width="{r * 0.06:.0f}" fill="none" stroke-linecap="round"/>'
        f'<path d="M{cx - r * 0.3:.0f},{hy + r * 0.5:.0f} q{r * 0.3:.0f},{r * 0.22:.0f} {r * 0.6:.0f},0" stroke="#7a2f2f" stroke-width="{r * 0.065:.0f}" fill="none" stroke-linecap="round" opacity="0.8"/>'
        f'<path d="M{cx:.0f},{hy + r * 0.05:.0f} q-{r * 0.08:.0f},{r * 0.28:.0f} {r * 0.02:.0f},{r * 0.3:.0f}" stroke="#000" stroke-opacity="0.18" stroke-width="{r * 0.05:.0f}" fill="none" stroke-linecap="round"/>'
    )
    if hair_style == "long":
        hair_svg = (
            f'<path d="M{cx - r * 1.05:.0f},{hy + r * 1.7:.0f} C{cx - r * 1.6:.0f},{hy - r * 2.7:.0f} {cx + r * 1.6:.0f},{hy - r * 2.7:.0f} {cx + r * 1.05:.0f},{hy + r * 1.7:.0f} L{cx + r * 0.75:.0f},{hy + r * 1.7:.0f} C{cx + r * 0.9:.0f},{hy - r * 0.2:.0f} {cx + r * 0.3:.0f},{hy - r * 0.9:.0f} {cx - r * 0.3:.0f},{hy - r * 0.95:.0f} C{cx - r * 0.85:.0f},{hy - r * 0.6:.0f} {cx - r * 0.9:.0f},{hy:.0f} {cx - r * 0.75:.0f},{hy + r * 1.7:.0f} Z" fill="{hair}"/>'
        )
        layered = hair_svg
        front_hair = f'<path d="M{cx - r * 0.98:.0f},{hy - r * 0.05:.0f} C{cx - r * 1.1:.0f},{hy - r * 1.55:.0f} {cx + r * 1.1:.0f},{hy - r * 1.55:.0f} {cx + r * 0.98:.0f},{hy - r * 0.05:.0f} C{cx + r * 0.8:.0f},{hy - r * 0.75:.0f} {cx + r * 0.1:.0f},{hy - r * 0.85:.0f} {cx - r * 0.25:.0f},{hy - r * 0.8:.0f} C{cx - r * 0.6:.0f},{hy - r * 0.72:.0f} {cx - r * 0.9:.0f},{hy - r * 0.45:.0f} {cx - r * 0.98:.0f},{hy - r * 0.05:.0f} Z" fill="{hair}"/>'
    elif hair_style == "short":
        layered = ""
        front_hair = f'<path d="M{cx - r * 0.98:.0f},{hy - r * 0.2:.0f} C{cx - r * 1.1:.0f},{hy - r * 1.5:.0f} {cx + r * 1.1:.0f},{hy - r * 1.5:.0f} {cx + r * 0.98:.0f},{hy - r * 0.2:.0f} C{cx + r * 0.75:.0f},{hy - r * 0.75:.0f} {cx + r * 0.2:.0f},{hy - r * 0.9:.0f} {cx - r * 0.3:.0f},{hy - r * 0.78:.0f} C{cx - r * 0.65:.0f},{hy - r * 0.7:.0f} {cx - r * 0.9:.0f},{hy - r * 0.55:.0f} {cx - r * 0.98:.0f},{hy - r * 0.2:.0f} Z" fill="{hair}"/>'
    elif hair_style == "curly":
        layered = "".join(
            f'<circle cx="{cx + math.cos(a) * r * 1.0:.0f}" cy="{hy - r * 0.15 + math.sin(a) * r * 1.1:.0f}" r="{r * 0.34:.0f}" fill="{hair}"/>'
            for a in [math.tau * i / 14 + math.pi for i in range(0, 8)]
        )
        front_hair = f'<path d="M{cx - r * 0.95:.0f},{hy - r * 0.3:.0f} C{cx - r * 0.8:.0f},{hy - r * 1.2:.0f} {cx + r * 0.8:.0f},{hy - r * 1.2:.0f} {cx + r * 0.95:.0f},{hy - r * 0.3:.0f} C{cx + r * 0.5:.0f},{hy - r * 0.7:.0f} {cx - r * 0.5:.0f},{hy - r * 0.7:.0f} {cx - r * 0.95:.0f},{hy - r * 0.3:.0f} Z" fill="{hair}"/>'
    elif hair_style == "bun":
        layered = f'<circle cx="{cx:.0f}" cy="{hy - r * 1.35:.0f}" r="{r * 0.42:.0f}" fill="{hair}"/>'
        front_hair = f'<path d="M{cx - r * 0.98:.0f},{hy - r * 0.1:.0f} C{cx - r * 1.05:.0f},{hy - r * 1.35:.0f} {cx + r * 1.05:.0f},{hy - r * 1.35:.0f} {cx + r * 0.98:.0f},{hy - r * 0.1:.0f} C{cx + r * 0.7:.0f},{hy - r * 0.8:.0f} {cx - r * 0.7:.0f},{hy - r * 0.8:.0f} {cx - r * 0.98:.0f},{hy - r * 0.1:.0f} Z" fill="{hair}"/>'
    else:  # bald / shaved
        layered = ""
        front_hair = ""
    beanie_svg = ""
    if beanie:
        beanie_svg = (
            f'<path d="M{cx - r * 1.0:.0f},{hy - r * 0.35:.0f} C{cx - r * 1.05:.0f},{hy - r * 1.55:.0f} {cx + r * 1.05:.0f},{hy - r * 1.55:.0f} {cx + r * 1.0:.0f},{hy - r * 0.35:.0f} Z" fill="{beanie}"/>'
            f'<rect x="{cx - r * 1.04:.0f}" y="{hy - r * 0.55:.0f}" width="{r * 2.08:.0f}" height="{r * 0.3:.0f}" rx="{r * 0.1:.0f}" fill="{beanie}"/>'
            f'<rect x="{cx - r * 1.04:.0f}" y="{hy - r * 0.55:.0f}" width="{r * 2.08:.0f}" height="{r * 0.3:.0f}" rx="{r * 0.1:.0f}" fill="#fff" opacity="0.12"/>'
            f'<circle cx="{cx:.0f}" cy="{hy - r * 1.5:.0f}" r="{r * 0.18:.0f}" fill="{beanie}"/>'
        )
    bg_aurora = scene(name + "-bg", s, s, seed, palette=palette, motif="pines", intensity=1.1)
    # Reuse the aurora scene as a background by embedding it.
    inner = bg_aurora.replace('<?xml version="1.0"?>', "")
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{s}" height="{s}" viewBox="0 0 {s} {s}">
  <defs><radialGradient id="pg" cx="0.5" cy="0.4" r="0.7"><stop offset="0" stop-color="{colors[0]}" stop-opacity="0.35"/><stop offset="1" stop-color="{sky[0]}" stop-opacity="0"/></radialGradient>
  <clipPath id="pc"><rect width="{s}" height="{s}"/></clipPath></defs>
  <g clip-path="url(#pc)">
    <svg x="0" y="0" width="{s}" height="{s}" viewBox="0 0 {s} {s}">{inner[inner.index('>') + 1:inner.rindex('</svg>')]}</svg>
    <rect width="{s}" height="{s}" fill="url(#pg)"/>
    {layered}{shoulders}{neck}{collar}{ears}{head}{eyes}{front_hair}{beanie_svg}
  </g>
</svg>"""
