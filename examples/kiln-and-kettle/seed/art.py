"""Procedural ceramics: vessels, portraits and studio scenes drawn as SVG, rasterised to JPEG.

Everything is built from a few primitives so the whole catalogue of ~90 images costs no downloads and is
reproducible: the same data always yields the same pixels.
"""

from __future__ import annotations

import math
import random
import subprocess
from pathlib import Path


# ---------------------------------------------------------------- colour helpers
def _rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def shade(h, k):
    """k < 1 darkens, k > 1 lightens towards white."""
    r, g, b = _rgb(h)
    if k <= 1:
        r, g, b = (int(c * k) for c in (r, g, b))
    else:
        t = min(k - 1, 1)
        r, g, b = (int(c + (255 - c) * t) for c in (r, g, b))
    return f"#{r:02x}{g:02x}{b:02x}"


def mix(a, b, t):
    ra, rb = _rgb(a), _rgb(b)
    return "#" + "".join(f"{int(x + (y - x) * t):02x}" for x, y in zip(ra, rb))


# ---------------------------------------------------------------- vessel profiles
# Each profile is a list of (y, half_width) from the rim down to the foot, in a box 0..1 tall.
# 'rim' is the half width of the opening used for the interior ellipse.
FORMS = {
    "bowl": dict(h=300, pts=[(0, 290), (0.18, 275), (0.5, 215), (0.8, 140), (1, 105)], foot=(105, 0.07)),
    "vase": dict(h=620, pts=[(0, 78), (0.14, 70), (0.3, 120), (0.52, 230), (0.78, 215), (0.94, 140), (1, 120)], foot=(120, 0.03)),
    "mug": dict(h=360, pts=[(0, 160), (0.12, 163), (0.9, 150), (1, 140)], foot=(140, 0.03), handle="mug"),
    "teapot": dict(h=360, pts=[(0, 140), (0.1, 150), (0.35, 235), (0.75, 245), (0.96, 170), (1, 150)], foot=(150, 0.03), handle="teapot"),
    "plate": dict(h=110, pts=[(0, 400), (0.25, 380), (0.7, 250), (1, 220)], foot=(220, 0.1), flat=True),
    "planter": dict(h=420, pts=[(0, 250), (0.1, 255), (0.12, 235), (0.9, 170), (1, 160)], foot=(160, 0.03), plant=True),
    "lantern": dict(h=620, pts=[(0, 120), (0.06, 150), (0.12, 168), (0.9, 190), (1, 195)], foot=(195, 0.02), cut=True),
    "tea_cup": dict(h=230, pts=[(0, 150), (0.25, 150), (0.8, 118), (1, 100)], foot=(100, 0.12)),
    "jug": dict(h=560, pts=[(0, 100), (0.08, 112), (0.22, 100), (0.4, 190), (0.72, 215), (0.96, 150), (1, 135)], foot=(135, 0.03), handle="jug"),
}


def _smooth(pts):
    """Smooth path through points: quadratic curves through midpoints."""
    d = f"M{pts[0][0]:.1f},{pts[0][1]:.1f}"
    for i in range(1, len(pts) - 1):
        mx = (pts[i][0] + pts[i + 1][0]) / 2
        my = (pts[i][1] + pts[i + 1][1]) / 2
        d += f" Q{pts[i][0]:.1f},{pts[i][1]:.1f} {mx:.1f},{my:.1f}"
    d += f" L{pts[-1][0]:.1f},{pts[-1][1]:.1f}"
    return d


def vessel(form, glaze="#c4572f", glaze2=None, interior=None, clay="#d9b99b", uid="v", seed=1, speckle=0.0, plant="#6d8f5e", cx=0, base=0, scale=1.0, drip=0.38):
    """A single vessel as an SVG <g> centred at x=cx, standing on y=base (units: px at scale 1)."""
    spec = FORMS[form]
    rng = random.Random(f"{uid}:{seed}")
    H = spec["h"]
    top = base - H
    ry_k = 0.15
    left, right = [], []
    for t, hw in spec["pts"]:
        y = top + t * H
        left.append((cx - hw, y))
        right.append((cx + hw, y))
    bhw = spec["pts"][-1][1]
    rim_hw = spec["pts"][0][1]
    ry_rim = max(10, rim_hw * ry_k) if not spec.get("flat") else rim_hw * 0.17
    ry_base = bhw * ry_k
    # silhouette: down the left, around the foot, up the right
    d = _smooth(left)
    d += f" A{bhw:.1f},{ry_base:.1f} 0 0 0 {right[-1][0]:.1f},{right[-1][1]:.1f}"
    d += " " + _smooth(right[::-1]).replace("M", "L", 1)
    d += f" A{rim_hw:.1f},{ry_rim:.1f} 0 0 0 {left[0][0]:.1f},{left[0][1]:.1f} Z"
    c = glaze
    c2 = glaze2 or glaze
    interior = interior or shade(glaze, 0.55)
    gid, cid = f"g{uid}", f"c{uid}"
    wide = max(h for _, h in spec["pts"])
    out = [f'<g transform="translate(0 0)">']
    # contact shadow
    out.append(f'<ellipse cx="{cx}" cy="{base + 6}" rx="{wide * 1.05:.0f}" ry="{wide * 0.16:.0f}" fill="#2a1a10" opacity="0.28" filter="url(#blur18)"/>')
    # handle behind body
    handle = spec.get("handle")
    hshape = ""
    if handle == "mug":
        hshape = f'<path d="M{cx + 150},{top + H * 0.22} C{cx + 330},{top + H * 0.12} {cx + 330},{top + H * 0.78} {cx + 148},{top + H * 0.72}" fill="none" stroke="url(#{gid}h)" stroke-width="42" stroke-linecap="round"/>'
    elif handle == "teapot":
        hshape = (
            f'<path d="M{cx + 235},{top + H * 0.3} C{cx + 400},{top + H * 0.15} {cx + 410},{top + H * 0.85} {cx + 225},{top + H * 0.78}" fill="none" stroke="url(#{gid}h)" stroke-width="40" stroke-linecap="round"/>'
            f'<path d="M{cx - 200},{top + H * 0.55} C{cx - 290},{top + H * 0.52} {cx - 330},{top + H * 0.3} {cx - 360},{top + H * 0.12}" fill="none" stroke="url(#{gid}h)" stroke-width="62" stroke-linecap="round"/>'
        )
    elif handle == "jug":
        hshape = f'<path d="M{cx + 105},{top + H * 0.12} C{cx + 330},{top + H * 0.05} {cx + 350},{top + H * 0.7} {cx + 205},{top + H * 0.62}" fill="none" stroke="url(#{gid}h)" stroke-width="44" stroke-linecap="round"/>'
    if spec.get("plant"):
        leaves = ""
        for i in range(9):
            a = -math.pi / 2 + (i - 4) * 0.28 + rng.uniform(-0.08, 0.08)
            ln = rng.uniform(260, 420)
            x0, y0 = cx + (i - 4) * 14, top + 20
            x1, y1 = x0 + math.cos(a) * ln, y0 + math.sin(a) * ln
            nx, ny = -math.sin(a) * 46, math.cos(a) * 46
            col = shade(plant, rng.uniform(0.7, 1.25))
            leaves += f'<path d="M{x0:.0f},{y0:.0f} Q{(x0 + x1) / 2 + nx:.0f},{(y0 + y1) / 2 + ny:.0f} {x1:.0f},{y1:.0f} Q{(x0 + x1) / 2 - nx:.0f},{(y0 + y1) / 2 - ny:.0f} {x0:.0f},{y0:.0f} Z" fill="{col}"/>'
        out.append(leaves)
    out.append(hshape)
    if handle == "teapot":
        # lid + knob sit on the rim
        out.append(f'<path d="M{cx - 150},{top + 6} C{cx - 150},{top - 90} {cx + 150},{top - 90} {cx + 150},{top + 6} Z" fill="url(#{gid})"/>')
        out.append(f'<ellipse cx="{cx}" cy="{top - 78}" rx="34" ry="26" fill="url(#{gid})"/>')
    out.append(f'<defs><linearGradient id="{gid}" x1="0" x2="1" y1="0" y2="0"><stop offset="0" stop-color="{shade(c, 0.55)}"/><stop offset=".2" stop-color="{shade(c, 0.9)}"/><stop offset=".42" stop-color="{shade(c, 1.28)}"/><stop offset=".65" stop-color="{c}"/><stop offset="1" stop-color="{shade(c, 0.5)}"/></linearGradient>'
               f'<linearGradient id="{gid}h" x1="0" x2="1" y1="0" y2="1"><stop offset="0" stop-color="{shade(c, 1.15)}"/><stop offset="1" stop-color="{shade(c, 0.6)}"/></linearGradient>'
               f'<linearGradient id="{gid}v" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.16"/><stop offset=".5" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity="0.32"/></linearGradient>'
               f'<clipPath id="{cid}"><path d="{d}"/></clipPath></defs>')
    out.append(f'<path d="{d}" fill="url(#{gid})"/>')
    # dipped second glaze with drips
    if glaze2:
        dip = top + H * drip
        wave = f"M{cx - wide - 20:.0f},{top - 20:.0f} L{cx + wide + 20:.0f},{top - 20:.0f} L{cx + wide + 20:.0f},{dip:.0f}"
        x = cx + wide + 20
        while x > cx - wide - 20:
            seg = rng.uniform(30, 70)
            drop = rng.choice([0, 0, 18, 46, 90])
            wave += f" Q{x - seg / 2:.0f},{dip + drop + 24:.0f} {x - seg:.0f},{dip + drop * 0.4:.0f}"
            x -= seg
        wave += " Z"
        out.append(f'<g clip-path="url(#{cid})"><path d="{wave}" fill="{glaze2}"/><path d="{wave}" fill="url(#{gid})" opacity="0.55" style="mix-blend-mode:multiply"/></g>')
    # speckle + throwing rings + vertical shading
    rings = "".join(f'<path d="M{cx - wide - 10:.0f},{top + H * t:.0f} h{2 * wide + 20:.0f}" stroke="#000" stroke-opacity="0.045" stroke-width="3"/>' for t in [i / 22 for i in range(2, 22)])
    dots = ""
    if speckle:
        for _ in range(int(speckle * 260)):
            x = cx + rng.uniform(-wide, wide)
            y = top + rng.uniform(0, H)
            dots += f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{rng.uniform(1.1, 3.2):.1f}" fill="#2b1a10" opacity="{rng.uniform(0.25, 0.7):.2f}"/>'
    out.append(f'<g clip-path="url(#{cid})">{rings}{dots}<rect x="{cx - wide - 20:.0f}" y="{top - 30:.0f}" width="{2 * wide + 40:.0f}" height="{H + 80:.0f}" fill="url(#{gid}v)"/>'
               f'<path d="M{cx - wide * 0.55:.0f},{top + H * 0.12:.0f} q-20,{H * 0.3:.0f} 0,{H * 0.55:.0f}" stroke="#fff" stroke-opacity="0.35" stroke-width="16" stroke-linecap="round" fill="none" filter="url(#blur6)"/></g>')
    # cutouts for lanterns (glowing)
    if spec.get("cut"):
        glow = ""
        for r_ in range(5):
            for k in range(5):
                x = cx + (k - 2) * 78 + (r_ % 2) * 28 - 14
                y = top + H * 0.2 + r_ * 70
                glow += f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{16 + (k + r_) % 3 * 3}" fill="#ffd27a"/><circle cx="{x:.0f}" cy="{y:.0f}" r="34" fill="#ffb347" opacity="0.35" filter="url(#blur6)"/>'
        out.append(glow)
    # rim: outer lip, interior well
    out.append(f'<ellipse cx="{cx}" cy="{top}" rx="{rim_hw}" ry="{ry_rim:.1f}" fill="{shade(c, 1.12)}"/>')
    out.append(f'<ellipse cx="{cx}" cy="{top + 2}" rx="{rim_hw - 10}" ry="{max(ry_rim - 8, 4):.1f}" fill="{interior}"/>')
    out.append(f'<path d="M{cx - rim_hw + 14},{top + 3} A{rim_hw - 14},{max(ry_rim - 8, 4):.1f} 0 0 1 {cx + rim_hw - 14},{top + 3}" stroke="#000" stroke-opacity="0.35" stroke-width="5" fill="none" filter="url(#blur6)"/>')
    # unglazed foot ring
    fw, fh = spec["foot"][0], spec["foot"][1]
    out.append(f'<path d="M{cx - fw},{base - 10} h{2 * fw} v10 a{fw},{fw * ry_k} 0 0 1 -{2 * fw},0 Z" fill="{clay}"/><ellipse cx="{cx}" cy="{base - 10}" rx="{fw}" ry="{fw * ry_k * 0.9:.0f}" fill="{shade(clay, 0.85)}" opacity="0"/>')
    out.append("</g>")
    body = "".join(out)
    if scale != 1.0:
        body = f'<g transform="translate({cx * (1 - scale):.1f} {base * (1 - scale):.1f}) scale({scale})">{body}</g>'
    return body


DEFS = """<defs>
<filter id="blur18" x="-30%" y="-80%" width="160%" height="260%"><feGaussianBlur stdDeviation="18"/></filter>
<filter id="blur6" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="6"/></filter>
<filter id="blur30" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="30"/></filter>
<filter id="paper" x="0" y="0" width="100%" height="100%"><feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="3"/><feColorMatrix values="0 0 0 0 0.2  0 0 0 0 0.12  0 0 0 0 0.06  0 0 0 0.09 0"/></filter>
</defs>"""


# ---------------------------------------------------------------- backdrops
BACKDROPS = {
    "sand": ("#efe3d0", "#d8c3a5", "#b89b78"),
    "sage": ("#dfe7da", "#b8c9b1", "#8aa585"),
    "dusk": ("#46506b", "#2f3750", "#1d2236"),
    "blush": ("#f2d9d0", "#e3b5a6", "#c98c78"),
    "ink": ("#2b2926", "#1c1a18", "#0f0e0d"),
    "ochre": ("#f0d18a", "#dca94b", "#b07d28"),
    "sky": ("#d6e4ec", "#a9c3d3", "#7a9ab0"),
    "clay": ("#d9a384", "#bd7a58", "#8f5236"),
}


def backdrop(w, h, kind, table=0.74, seed=1):
    a, b, c = BACKDROPS[kind]
    dark = kind in ("dusk", "ink")
    ty = h * table
    arch = ""
    if kind in ("sand", "sage", "blush", "sky", "ochre"):
        aw = w * 0.5
        arch = f'<path d="M{w / 2 - aw / 2:.0f},{ty:.0f} V{h * 0.32:.0f} A{aw / 2:.0f},{aw / 2:.0f} 0 0 1 {w / 2 + aw / 2:.0f},{h * 0.32:.0f} V{ty:.0f} Z" fill="{shade(a, 1.35)}" opacity="0.55"/>'
    sun = f'<circle cx="{w * 0.78:.0f}" cy="{h * 0.2:.0f}" r="{w * 0.12:.0f}" fill="{shade(a, 1.5) if not dark else shade(b, 1.5)}" opacity="0.5" filter="url(#blur30)"/>'
    return (
        f'<defs><linearGradient id="bgw" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="{a}"/><stop offset="1" stop-color="{b}"/></linearGradient>'
        f'<linearGradient id="bgt" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="{c}"/><stop offset="1" stop-color="{shade(c, 0.7)}"/></linearGradient></defs>'
        f'<rect width="{w}" height="{h}" fill="url(#bgw)"/>{arch}{sun}'
        f'<rect y="{ty:.0f}" width="{w}" height="{h - ty:.0f}" fill="url(#bgt)"/>'
        f'<rect y="{ty - 3:.0f}" width="{w}" height="6" fill="#000" opacity="0.1"/>'
    )


def finish(w, h):
    return f'<rect width="{w}" height="{h}" filter="url(#paper)"/><rect width="{w}" height="{h}" fill="url(#vig)"/>'


VIG = '<defs><radialGradient id="vig" cx=".5" cy=".5" r=".75"><stop offset=".6" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#1a0f08" stop-opacity=".38"/></radialGradient></defs>'


def svg(w, h, body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{DEFS}{VIG}{body}</svg>'


# ---------------------------------------------------------------- compositions
def piece_photo(form, glaze, glaze2=None, bg="sand", seed=1, speckle=0.0, clay="#d9b99b", size=1200, accent_form=None):
    w = h = size
    s = size / 1200
    base = 900
    spec_h = FORMS[form]["h"]
    # scale tall forms down so they always fit with air above
    k = min(1.0, 620 / (spec_h + (90 if form == "teapot" else 0)))
    if form == "plate":
        k = 1.12
    body = backdrop(1200, 1200, bg, 0.74, seed)
    # a small companion object adds life to the still life
    rng = random.Random(seed)
    companion = ""
    if form not in ("plate", "lantern"):
        companion = vessel("tea_cup", mix(glaze, "#ffffff", 0.45), None, clay=clay, uid="c", seed=seed, cx=1010, base=base + 50, scale=0.34)
    body += companion
    body += vessel(form, glaze, glaze2, clay=clay, uid="m", seed=seed, speckle=speckle, cx=560 if companion else 600, base=base, scale=k)
    body += finish(1200, 1200)
    return svg(1200, 1200, body).replace(f'width="1200" height="1200"', f'width="{w}" height="{h}"', 1)


def tea_photo(liquor, tea_color, bg="sand", seed=1, w=1200, h=900):
    rng = random.Random(seed)
    body = backdrop(w, h, bg, 0.7, seed)
    base = 650
    # leaves scattered on the table
    for _ in range(46):
        x, y = rng.uniform(80, w - 80), rng.uniform(base + 30, h - 40)
        r = rng.uniform(0, 180)
        lw, lh = rng.uniform(14, 28), rng.uniform(5, 10)
        body += f'<ellipse cx="{x:.0f}" cy="{y:.0f}" rx="{lw:.0f}" ry="{lh:.0f}" transform="rotate({r:.0f} {x:.0f} {y:.0f})" fill="{shade(tea_color, rng.uniform(0.6, 1.2))}" opacity="0.9"/>'
    body += vessel("teapot", mix("#ffffff", tea_color, 0.25), None, uid="tp", seed=seed, cx=330, base=base + 20, scale=0.72)
    body += vessel("tea_cup", "#f3ecdf", None, interior=liquor, uid="tc", seed=seed, cx=830, base=base + 70, scale=1.05)
    # steam
    for i, x in enumerate((790, 840, 890)):
        body += f'<path d="M{x},{base - 190} C{x - 50},{base - 270} {x + 50},{base - 330} {x},{base - 420}" stroke="#fff" stroke-opacity="0.45" stroke-width="{14 - i * 2}" fill="none" stroke-linecap="round" filter="url(#blur6)"/>'
    body += finish(w, h)
    return svg(w, h, body)


def shelf_scene(w, h, items, bg="sand", seed=1, label_rows=1):
    """items: list of (form, glaze, glaze2) arranged along one or two shelves."""
    body = backdrop(w, h, bg, 0.5, seed)
    rows = [items[: len(items) // 2 + len(items) % 2], items[len(items) // 2 + len(items) % 2:]] if len(items) > 4 else [items]
    for ri, row in enumerate(rows):
        base = h * (0.48 if len(rows) > 1 else 0.72) + ri * h * 0.42
        if len(rows) > 1:
            body += f'<rect x="0" y="{base:.0f}" width="{w}" height="22" fill="#6b4a32"/><rect x="0" y="{base + 22:.0f}" width="{w}" height="14" fill="#000" opacity="0.18" filter="url(#blur6)"/>'
        gap = w / (len(row) + 1)
        for i, (form, g1, g2) in enumerate(row):
            k = min(0.8, (h * 0.34) / FORMS[form]["h"])
            body += vessel(form, g1, g2, uid=f"s{ri}{i}", seed=seed + i, cx=gap * (i + 1), base=base, scale=k)
    body += finish(w, h)
    return svg(w, h, body)


def wheel_scene(w, h, clay="#c79a74", accent="#c4572f", seed=1):
    body = backdrop(w, h, "clay", 0.72, seed).replace("<rect width", f'<rect opacity="1" width', 1)
    cx, base = w * 0.5, h * 0.74
    body += f'<ellipse cx="{cx}" cy="{base + 40}" rx="330" ry="46" fill="#000" opacity="0.3" filter="url(#blur18)"/>'
    body += f'<rect x="{cx - 70}" y="{base - 10}" width="140" height="140" fill="#3b3631"/><rect x="{cx - 190}" y="{base + 120}" width="380" height="36" rx="14" fill="#2b2724"/>'
    body += f'<ellipse cx="{cx}" cy="{base}" rx="290" ry="44" fill="#5b5650"/><ellipse cx="{cx}" cy="{base - 12}" rx="290" ry="44" fill="#7b756d"/><ellipse cx="{cx}" cy="{base - 14}" rx="240" ry="34" fill="#8b857c"/>'
    # clay mid-throw: a tall tapering form
    pts = [(0, 70), (0.2, 84), (0.55, 150), (0.85, 175), (1, 170)]
    H = 380
    top = base - 18 - H
    left = [(cx - hw, top + t * H) for t, hw in pts]
    right = [(cx + hw, top + t * H) for t, hw in pts]
    d = _smooth(left) + f" A170,26 0 0 0 {right[-1][0]},{right[-1][1]} " + _smooth(right[::-1]).replace("M", "L", 1) + f" A70,12 0 0 0 {left[0][0]},{left[0][1]} Z"
    body += f'<defs><linearGradient id="cl" x1="0" x2="1"><stop offset="0" stop-color="{shade(clay, 0.6)}"/><stop offset=".4" stop-color="{shade(clay, 1.2)}"/><stop offset="1" stop-color="{shade(clay, 0.55)}"/></linearGradient></defs>'
    body += f'<path d="{d}" fill="url(#cl)"/><ellipse cx="{cx}" cy="{top}" rx="70" ry="12" fill="{shade(clay, 0.5)}"/>'
    for i in range(14):
        body += f'<path d="M{cx - 190},{top + 30 + i * 26} q190,{12 + (i % 3) * 3} 380,0" stroke="#fff" stroke-opacity="0.07" stroke-width="3" fill="none"/>'
    # two hands pressing the clay from each side
    skin = "#e0b08c"
    for sx in (-1, 1):
        body += f'<path d="M{cx + sx * 420},{base - 220} C{cx + sx * 300},{base - 250} {cx + sx * 230},{base - 190} {cx + sx * 150},{base - 160} C{cx + sx * 130},{base - 120} {cx + sx * 170},{base - 100} {cx + sx * 240},{base - 120} C{cx + sx * 320},{base - 60} {cx + sx * 420},{base - 70} {cx + sx * 500},{base - 80} Z" fill="{skin}"/>'
        body += f'<path d="M{cx + sx * 420},{base - 220} L{cx + sx * 700},{base - 260} L{cx + sx * 700},{base - 60} L{cx + sx * 500},{base - 80} Z" fill="{accent}"/>'
    body += finish(w, h)
    return svg(w, h, body)


def tiles_scene(w, h, colors, seed=1, bg="ink"):
    rng = random.Random(seed)
    body = backdrop(w, h, bg, 0.9, seed)
    cols, rows = 6, 3
    tw, th = (w - 160) / cols, (h - 200) / rows
    for r in range(rows):
        for c in range(cols):
            base = colors[(r * cols + c) % len(colors)]
            x, y = 80 + c * tw, 100 + r * th
            gid = f"t{r}{c}"
            body += f'<defs><linearGradient id="{gid}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{shade(base, 1.2)}"/><stop offset=".6" stop-color="{base}"/><stop offset="1" stop-color="{shade(base, 0.65)}"/></linearGradient></defs>'
            body += f'<rect x="{x + 8:.0f}" y="{y + 10:.0f}" width="{tw - 22:.0f}" height="{th - 28:.0f}" rx="10" fill="#000" opacity="0.35" filter="url(#blur6)"/>'
            body += f'<rect x="{x + 6:.0f}" y="{y + 6:.0f}" width="{tw - 22:.0f}" height="{th - 28:.0f}" rx="10" fill="url(#{gid})"/>'
            # dipped band + speckles per tile
            body += f'<rect x="{x + 6:.0f}" y="{y + 6:.0f}" width="{tw - 22:.0f}" height="{(th - 28) * rng.uniform(0.25, 0.5):.0f}" rx="10" fill="{shade(colors[(r * cols + c + 3) % len(colors)], 1.0)}" opacity="0.8"/>'
            for _ in range(14):
                body += f'<circle cx="{x + 14 + rng.uniform(0, tw - 40):.0f}" cy="{y + 14 + rng.uniform(0, th - 44):.0f}" r="{rng.uniform(1, 2.6):.1f}" fill="#1a0f08" opacity="0.4"/>'
            body += f'<text x="{x + tw - 28:.0f}" y="{y + th - 32:.0f}" font-family="monospace" font-size="18" fill="#fff" opacity="0.55" text-anchor="end">{(r * cols + c + 1) * 7:03d}</text>'
    body += finish(w, h)
    return svg(w, h, body)


def kiln_scene(w, h, glow="#ff8a3c", seed=1):
    rng = random.Random(seed)
    body = f'<rect width="{w}" height="{h}" fill="#140d09"/>'
    cx, cy = w / 2, h * 0.58
    aw, ah = w * 0.55, h * 0.78
    body += f'<defs><radialGradient id="kg" cx=".5" cy=".65" r=".7"><stop offset="0" stop-color="#fff2c4"/><stop offset=".18" stop-color="{glow}"/><stop offset=".6" stop-color="#9c2e12"/><stop offset="1" stop-color="#2a0e06"/></radialGradient></defs>'
    body += f'<path d="M{cx - aw / 2:.0f},{h} V{cy - ah * 0.12:.0f} A{aw / 2:.0f},{ah * 0.55:.0f} 0 0 1 {cx + aw / 2:.0f},{cy - ah * 0.12:.0f} V{h} Z" fill="url(#kg)"/>'
    # firebrick courses
    for i in range(18):
        y = h * 0.1 + i * h * 0.055
        body += f'<path d="M{cx - aw / 2 - 6:.0f},{y:.0f} H{cx + aw / 2 + 6:.0f}" stroke="#2a0e06" stroke-opacity="0.35" stroke-width="3"/>'
    # stacked ware silhouettes glowing
    for i in range(7):
        form = rng.choice(["vase", "bowl", "jug", "mug", "tea_cup"])
        k = rng.uniform(0.28, 0.4)
        body += vessel(form, "#c24a1a", "#ffb15c", uid=f"k{i}", seed=seed + i, cx=cx - aw * 0.38 + i * aw * 0.127, base=h * 0.86 - (i % 2) * 0, scale=k, drip=0.3)
    # sparks
    for _ in range(60):
        body += f'<circle cx="{rng.uniform(w * 0.2, w * 0.8):.0f}" cy="{rng.uniform(h * 0.1, h * 0.9):.0f}" r="{rng.uniform(1, 3.4):.1f}" fill="#ffd27a" opacity="{rng.uniform(0.2, 0.9):.2f}"/>'
    body += f'<rect width="{w}" height="{h}" fill="url(#vig)"/>'
    return svg(w, h, body)


def tea_still(w, h, tea_color, bg="sage", seed=1):
    return tea_photo("#c9822b", tea_color, bg, seed, w, h)


# ---------------------------------------------------------------- portraits
HAIR = {
    "long": lambda cx, hy, r, c: f'<path d="M{cx - r * 1.05:.0f},{hy + r * 1.8:.0f} C{cx - r * 1.7:.0f},{hy - r * 2.8:.0f} {cx + r * 1.7:.0f},{hy - r * 2.8:.0f} {cx + r * 1.05:.0f},{hy + r * 1.8:.0f} Z" fill="{c}"/>',
    "bob": lambda cx, hy, r, c: f'<path d="M{cx - r * 1.1:.0f},{hy + r * 0.9:.0f} C{cx - r * 1.5:.0f},{hy - r * 2.3:.0f} {cx + r * 1.5:.0f},{hy - r * 2.3:.0f} {cx + r * 1.1:.0f},{hy + r * 0.9:.0f} Z" fill="{c}"/>',
    "curly": lambda cx, hy, r, c: "".join(f'<circle cx="{cx + math.cos(a) * r * 1.05:.0f}" cy="{hy - r * 0.1 + math.sin(a) * r * 1.15:.0f}" r="{r * 0.36:.0f}" fill="{c}"/>' for a in [math.pi + math.pi * i / 7 for i in range(8)]),
    "bun": lambda cx, hy, r, c: f'<circle cx="{cx:.0f}" cy="{hy - r * 1.45:.0f}" r="{r * 0.44:.0f}" fill="{c}"/>',
    "short": lambda cx, hy, r, c: "",
    "bald": lambda cx, hy, r, c: "",
}


def portrait(seed, bg, skin, hair, hair_style, garment, apron="#b5703f", glasses=False, beard=None, size=900):
    rng = random.Random(seed)
    s = size
    cx, hy, r = s / 2, s * 0.40, s * 0.15
    a, b, c = BACKDROPS[bg]
    body = backdrop(s, s, bg, 0.9, seed)
    # shelf of tiny pots behind
    for i, (form, g) in enumerate([("vase", "#c4572f"), ("bowl", "#27406b"), ("jug", "#8fb7a6"), ("mug", "#e0a526")]):
        body += vessel(form, g, None, uid=f"p{i}", seed=seed, cx=s * (0.12 + i * 0.25), base=s * 0.56 if i % 2 == 0 else s * 0.5, scale=0.17 if i % 2 == 0 else 0.15)
    body += f'<rect x="0" y="{s * 0.565:.0f}" width="{s}" height="12" fill="#6b4a32" opacity="0.9"/>'
    body += HAIR[hair_style](cx, hy, r, hair)
    body += f'<path d="M{s * 0.1:.0f},{s} C{s * 0.1:.0f},{s * 0.72} {s * 0.3:.0f},{s * 0.66} {cx:.0f},{s * 0.66} C{s * 0.7:.0f},{s * 0.66} {s * 0.9:.0f},{s * 0.72} {s * 0.9:.0f},{s} Z" fill="{garment}"/>'
    body += f'<path d="M{cx - r * 1.3:.0f},{s * 0.76:.0f} L{cx - r * 0.7:.0f},{s * 0.68:.0f} L{cx + r * 0.7:.0f},{s * 0.68:.0f} L{cx + r * 1.3:.0f},{s * 0.76:.0f} L{cx + r * 1.5:.0f},{s} L{cx - r * 1.5:.0f},{s} Z" fill="{apron}"/>'
    body += f'<rect x="{cx - r * 0.42:.0f}" y="{hy + r * 0.6:.0f}" width="{r * 0.84:.0f}" height="{s * 0.15:.0f}" rx="{r * 0.3:.0f}" fill="{skin}"/><rect x="{cx - r * 0.42:.0f}" y="{hy + r * 0.6:.0f}" width="{r * 0.84:.0f}" height="{s * 0.05:.0f}" fill="#000" opacity="0.2"/>'
    body += f'<ellipse cx="{cx:.0f}" cy="{hy:.0f}" rx="{r * 0.92:.0f}" ry="{r * 1.12:.0f}" fill="{skin}"/>'
    body += f'<ellipse cx="{cx - r * 0.92:.0f}" cy="{hy + 6:.0f}" rx="{r * 0.14:.0f}" ry="{r * 0.24:.0f}" fill="{skin}"/><ellipse cx="{cx + r * 0.92:.0f}" cy="{hy + 6:.0f}" rx="{r * 0.14:.0f}" ry="{r * 0.24:.0f}" fill="{skin}"/>'
    if beard:
        body += f'<path d="M{cx - r * 0.9:.0f},{hy + r * 0.1:.0f} C{cx - r * 0.9:.0f},{hy + r * 1.6:.0f} {cx + r * 0.9:.0f},{hy + r * 1.6:.0f} {cx + r * 0.9:.0f},{hy + r * 0.1:.0f} C{cx + r * 0.5:.0f},{hy + r * 0.55:.0f} {cx - r * 0.5:.0f},{hy + r * 0.55:.0f} {cx - r * 0.9:.0f},{hy + r * 0.1:.0f} Z" fill="{beard}"/>'
    body += (f'<ellipse cx="{cx - r * 0.36:.0f}" cy="{hy - r * 0.05:.0f}" rx="{r * 0.09:.0f}" ry="{r * 0.06:.0f}" fill="#1c1410"/><ellipse cx="{cx + r * 0.36:.0f}" cy="{hy - r * 0.05:.0f}" rx="{r * 0.09:.0f}" ry="{r * 0.06:.0f}" fill="#1c1410"/>'
             f'<path d="M{cx - r * 0.5:.0f},{hy - r * 0.26:.0f} q{r * 0.14:.0f},-{r * 0.1:.0f} {r * 0.28:.0f},0 M{cx + r * 0.22:.0f},{hy - r * 0.26:.0f} q{r * 0.14:.0f},-{r * 0.1:.0f} {r * 0.28:.0f},0" stroke="{hair}" stroke-width="{r * 0.06:.0f}" fill="none" stroke-linecap="round"/>'
             f'<path d="M{cx - r * 0.3:.0f},{hy + r * 0.5:.0f} q{r * 0.3:.0f},{r * 0.2:.0f} {r * 0.6:.0f},0" stroke="#7a2f2f" stroke-width="{r * 0.065:.0f}" fill="none" stroke-linecap="round" opacity="0.85"/>'
             f'<path d="M{cx:.0f},{hy + r * 0.05:.0f} q-{r * 0.08:.0f},{r * 0.28:.0f} {r * 0.02:.0f},{r * 0.3:.0f}" stroke="#000" stroke-opacity="0.18" stroke-width="{r * 0.05:.0f}" fill="none" stroke-linecap="round"/>')
    if glasses:
        body += (f'<circle cx="{cx - r * 0.38:.0f}" cy="{hy - r * 0.05:.0f}" r="{r * 0.27:.0f}" fill="#fff" fill-opacity=".15" stroke="#2b2118" stroke-width="{r * 0.05:.0f}"/>'
                 f'<circle cx="{cx + r * 0.38:.0f}" cy="{hy - r * 0.05:.0f}" r="{r * 0.27:.0f}" fill="#fff" fill-opacity=".15" stroke="#2b2118" stroke-width="{r * 0.05:.0f}"/>'
                 f'<path d="M{cx - r * 0.11:.0f},{hy - r * 0.05:.0f} h{r * 0.22:.0f}" stroke="#2b2118" stroke-width="{r * 0.05:.0f}"/>')
    # hair cap over the forehead
    if hair_style not in ("bald",):
        body += f'<path d="M{cx - r * 0.98:.0f},{hy - r * 0.15:.0f} C{cx - r * 1.1:.0f},{hy - r * 1.5:.0f} {cx + r * 1.1:.0f},{hy - r * 1.5:.0f} {cx + r * 0.98:.0f},{hy - r * 0.15:.0f} C{cx + r * 0.7:.0f},{hy - r * 0.75:.0f} {cx + r * 0.1:.0f},{hy - r * 0.9:.0f} {cx - r * 0.3:.0f},{hy - r * 0.8:.0f} C{cx - r * 0.7:.0f},{hy - r * 0.72:.0f} {cx - r * 0.92:.0f},{hy - r * 0.5:.0f} {cx - r * 0.98:.0f},{hy - r * 0.15:.0f} Z" fill="{hair}"/>'
    # clay smudges on the apron
    for _ in range(5):
        body += f'<ellipse cx="{cx + rng.uniform(-r, r):.0f}" cy="{s * rng.uniform(0.82, 0.96):.0f}" rx="{rng.uniform(8, 22):.0f}" ry="{rng.uniform(4, 9):.0f}" fill="#f0e2d0" opacity="0.5"/>'
    body += finish(s, s)
    return svg(s, s, body)


# ---------------------------------------------------------------- raster
def render(svg_text: str, out: Path, quality: int = 84) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    png = out.with_suffix(".png")
    subprocess.run(["rsvg-convert", "-o", str(png)], input=svg_text.encode(), check=True)
    subprocess.run(["magick", str(png), "-strip", "-interlace", "Plane", "-quality", str(quality), str(out)], check=True)
    png.unlink()
