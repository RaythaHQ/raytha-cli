#!/usr/bin/env python3
"""Draw every image procedurally into .build/img, then upload the page-level ones to the media library.

Item images (pieces, makers, teas, workshops, journal covers) are NOT uploaded here: `content import`
uploads them itself through `@file:` values. Only the images that site pages and widgets point at go into
the media library, and their object keys are written to .build/media.json.
"""

import hashlib
import json
from concurrent.futures import ThreadPoolExecutor

import art
from data_makers import MAKERS
from data_pieces import PIECES
from data_workshops import JOURNAL, TEAS, WORKSHOPS
from lib import ROOT, img_path, log, rt

BUILD = ROOT / ".build"
IMG = BUILD / "img"
MAP = BUILD / "media.json"
IMG.mkdir(parents=True, exist_ok=True)

GLAZES = ["#b5482a", "#27406b", "#8fb7a6", "#e0a526", "#6b3f2e", "#e8dcc4", "#6f8f6a", "#e8a98a", "#3f8f7e", "#6a3a55"]
jobs = {}


def seed_of(key):
    return int(hashlib.sha1(key.encode()).hexdigest(), 16) % 9973


for p in PIECES:
    key = f"piece_{p['t']}"
    jobs[key] = lambda p=p, key=key: art.piece_photo(p["form"], p["c"], p["c2"], p["bg"], seed_of(key), p["speck"])
for k, m in MAKERS.items():
    po = m["portrait"]
    jobs[f"maker_{k}"] = lambda k=k, po=po: art.portrait(seed_of(k), po["bg"], po["skin"], po["hair"], po["style"], po["garment"], po["apron"], po["glasses"], po.get("beard"))
for t in TEAS:
    key = f"tea_{t['t']}"
    jobs[key] = lambda t=t, key=key: art.tea_photo(t["liquor"], t["leaf"], t["bg"], seed_of(key), 1200, 800)


def scene(kind, key, w, h, accent="#b5482a"):
    s = seed_of(key)
    if kind == "wheel":
        return art.wheel_scene(w, h, "#c79a74", accent, s)
    if kind == "kiln":
        return art.kiln_scene(w, h, "#ff8a3c", s)
    if kind == "tiles":
        return art.tiles_scene(w, h, GLAZES, s)
    if kind == "tea":
        return art.tea_photo("#d9902f", "#5a3a1a", "sage", s, w, h)
    return art.shelf_scene(w, h, [("vase", "#b5482a", "#2b2926"), ("bowl", "#27406b", "#e8dcc4"), ("jug", "#8fb7a6", None), ("mug", "#e0a526", None),
                                  ("teapot", "#2b2926", None), ("plate", "#e8dcc4", "#27406b"), ("tea_cup", "#b5482a", None), ("lantern", "#6b3f2e", None)], "sand", s)


for w in WORKSHOPS:
    key = f"workshop_{w['t']}"
    jobs[key] = lambda w=w, key=key: scene(w["art"], key, 1600, 900, w["accent"])
for j in JOURNAL:
    key = f"journal_{j['t']}"
    jobs[key] = lambda j=j, key=key: scene(j["art"], key, 1600, 900)

# page-level imagery (uploaded to the media library)
PAGE = {
    "hero_shelf": lambda: art.shelf_scene(2000, 1200, [("vase", "#b5482a", "#2b2926"), ("bowl", "#27406b", "#e8dcc4"), ("teapot", "#2b2926", None), ("jug", "#8fb7a6", None),
                                                       ("mug", "#e0a526", None), ("plate", "#e8dcc4", "#27406b"), ("tea_cup", "#b5482a", None), ("lantern", "#6b3f2e", None), ("planter", "#e0a526", None)], "sand", 11),
    "studio_kiln": lambda: art.kiln_scene(1600, 1000, "#ff8a3c", 21),
    "studio_wheel": lambda: art.wheel_scene(1600, 1000, "#c79a74", "#27406b", 22),
    "studio_tiles": lambda: art.tiles_scene(1600, 1000, GLAZES, 23),
    "studio_shelf": lambda: scene("shelf", "studio_shelf", 1600, 1000),
    "studio_tea": lambda: art.tea_photo("#d9902f", "#5a3a1a", "sage", 24, 1600, 1000),
    "studio_ember": lambda: art.kiln_scene(1600, 1000, "#ffb347", 25),
    "visit_exterior": lambda: art.shelf_scene(1600, 1000, [("lantern", "#6b3f2e", None), ("planter", "#e0a526", None), ("vase", "#b5482a", None), ("planter", "#6f8f6a", None), ("lantern", "#38342f", None)], "dusk", 26),
    "commission_sketch": lambda: art.tiles_scene(1600, 1000, GLAZES[::-1], 27),
    "kiln_still": lambda: art.kiln_scene(1600, 1000, "#ff7a2c", 28),
    "tea_banner": lambda: art.tea_photo("#c9822b", "#5a3a1a", "clay", 29, 1800, 900),
    "journal_banner": lambda: art.shelf_scene(1800, 900, [("jug", "#aeb4b4", "#e8dcc4"), ("vase", "#9eb6c9", None), ("mug", "#e8dcc4", None), ("bowl", "#c9a15e", "#7a5030")], "sage", 31),
    "classes_banner": lambda: art.wheel_scene(1800, 900, "#c79a74", "#b5482a", 32),
}
for name, fn in PAGE.items():
    jobs[name] = fn


def make(name):
    out = img_path(name)
    if not out.exists():
        art.render(jobs[name](), out, 82 if name in PAGE else 80)
    return name


log(f"rendering {len(jobs)} images")
with ThreadPoolExecutor(max_workers=4) as pool:
    list(pool.map(make, jobs))

media = json.loads(MAP.read_text()) if MAP.exists() else {}
todo = [n for n in PAGE if n not in media]
log(f"uploading {len(todo)} page images")
for n in todo:
    res = rt("media", "upload", str(img_path(n)))
    media[n] = res["objectKey"]
    MAP.write_text(json.dumps(media, indent=1))
log(f"media library holds {len(media)} page images; item images upload through content import")
