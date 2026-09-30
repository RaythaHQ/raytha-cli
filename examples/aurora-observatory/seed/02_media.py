#!/usr/bin/env python3
"""Generate every image procedurally, then upload to the media library with `raytha media upload`.

Writes .build/media.json: {name: objectKey}. Re-runs only upload what is missing.
"""

import hashlib
import json
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import art
from data_expeditions import EXPEDITIONS
from data_people import GUIDES, INSTRUMENTS, VOICES
from data_sky import EVENTS, PHOTOS, POSTS
from lib import ROOT, log, rt

BUILD = ROOT / ".build"
IMG = BUILD / "img"
MAP = BUILD / "media.json"
IMG.mkdir(parents=True, exist_ok=True)

jobs = {}  # name -> callable producing svg, output path


def scene_job(name, w, h, a):
    jobs[name] = lambda: art.scene(name, w, h, a["seed"], a["palette"], a["motif"], a.get("moon", False), a.get("reflect", False), a.get("intensity", 1.0))


for key, e in EXPEDITIONS.items():
    scene_job(f"exp_{key}", 1600, 900, e["art"])
for i, p in enumerate(POSTS):
    scene_job(f"post_{i:02d}", 1400, 800, p["art"])
for key, inst in INSTRUMENTS.items():
    scene_job(f"inst_{key}", 1200, 800, inst["art"])
for i, ev in enumerate(EVENTS):
    scene_job(f"evt_{i:02d}", 900, 1200, ev["art"])
for key, g in GUIDES.items():
    p = g["portrait"]
    jobs[f"guide_{key}"] = lambda key=key, p=p: art.portrait(f"guide-{key}", 7, p["palette"], p["skin"], p["hair"], p["style"], p["garment"], p["beanie"], 800)
for i, v in enumerate(VOICES):
    p = v["portrait"]
    jobs[f"voice_{i:02d}"] = lambda i=i, p=p: art.portrait(f"voice-{i}", 9, p["palette"], p["skin"], p["hair"], p["style"], p["garment"], p["beanie"], 480)


def make(name):
    out = IMG / f"{name}.jpg"
    if not out.exists():
        art.render(jobs[name](), out)
    return name


for ph in PHOTOS:  # stable seeds: python's hash() is salted per process
    key = ph[0]
    w, h = ph[2]
    seed = int(hashlib.sha1(key.encode()).hexdigest(), 16) % 97 + 3
    a = dict(seed=seed, palette=ph[3], motif=ph[4], **ph[15])
    scene_job(key, w, h, a)

log(f"rendering {len(jobs)} images")
with ThreadPoolExecutor(max_workers=4) as pool:
    list(pool.map(make, jobs))

media = json.loads(MAP.read_text()) if MAP.exists() else {}
todo = [n for n in jobs if n not in media]
log(f"uploading {len(todo)} images")
for n in todo:
    res = rt("media", "upload", str(IMG / f"{n}.jpg"))
    media[n] = res["objectKey"]
    MAP.write_text(json.dumps(media, indent=1))
log(f"media library: {len(media)} objects")
