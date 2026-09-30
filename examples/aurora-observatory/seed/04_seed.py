#!/usr/bin/env python3
"""Seed all Aurora Observatory content through `raytha content create`.

Idempotent: created ids are remembered in .build/ids.json, so re-runs only add what is missing.
Order matters because relationships store the target item's id:
guides -> expeditions -> sky events -> field notes -> photographs -> instruments -> voices.
"""

import json

from data_expeditions import EXPEDITIONS
from data_people import GUIDES, INSTRUMENTS, VOICES
from data_sky import EVENTS, PHOTOS, POSTS
from lib import ROOT, jdump, log, rt

BUILD = ROOT / ".build"
MEDIA = json.loads((BUILD / "media.json").read_text())
IDS_FILE = BUILD / "ids.json"
IDS = json.loads(IDS_FILE.read_text()) if IDS_FILE.exists() else {}


def save():
    IDS_FILE.write_text(json.dumps(IDS, indent=1))


def create(kind, key, template, title, fields):
    """Create one item unless it already exists; remember its id."""
    tag = f"{kind}:{key}"
    if tag in IDS:
        return IDS[tag]
    data = {"title": title, **{k: v for k, v in fields.items() if v is not None}}
    item = rt("content", "create", kind, "--template", template, "--data", jdump(data))
    IDS[tag] = item["id"]
    save()
    log(f"  {tag} -> {item['id']}")
    return item["id"]


def remove_probes():
    for kind in ("expeditions", "guides"):
        for it in rt("content", "list", kind, "--all")["items"]:
            if it["primaryField"].startswith("Probe"):
                log(f"deleting probe {kind}/{it['primaryField']}")
                rt("content", "delete", kind, it["id"], "--yes")


def m(name):
    return MEDIA[name]


def seed_guides():
    log("guides")
    for key, g in GUIDES.items():
        create("guides", key, "aurora_detail_guide", g["title"], {
            "content": g["bio"],
            "role": g["role"],
            "quote": g["quote"],
            "portrait": m(f"guide_{key}"),
            "accent": g["accent"],
            "years_experience": g["years"],
            "base_region": g["region"],
            "specialties": g["skills"],
            "languages": g["langs"],
            "is_lead_guide": g["lead"],
            "links": [{"platform": p, "handle": h} for p, h in g["links"]],
        })


def seed_expeditions():
    log("expeditions")
    for key, e in EXPEDITIONS.items():
        create("expeditions", key, "aurora_detail_expedition", e["title"], {
            "content": e["story"],
            "tagline": e["tagline"],
            "summary": e["summary"],
            "hero_image": m(f"exp_{key}"),
            "accent": e["accent"],
            "region": e["region"],
            "difficulty": e["difficulty"],
            "seasons": e["seasons"],
            "departure_date": e["departure"],
            "duration_nights": e["nights"],
            "price_usd": e["price"],
            "group_size": e["group"],
            "aurora_odds": e["odds"],
            "is_featured": e["featured"],
            "lead_guide": IDS[f"guides:{e['guide']}"],
            "itinerary": [
                {"day": d, "title": t, "details": det, "lodging": lod, "distance_km": km, "aurora_night": night}
                for d, t, det, lod, km, night in e["itinerary"]
            ],
            "moments": [{"caption": c, "photo": m(g)} for c, g in e["moments"]],
        })


def seed_events():
    log("sky events")
    for i, ev in enumerate(EVENTS):
        create("sky_events", f"{i:02d}", "aurora_detail_event", ev["title"], {
            "content": ev["body"],
            "event_type": ev["type"],
            "starts_on": ev["start"],
            "ends_on": ev["end"],
            "visibility": ev["vis"],
            "expected_kp": ev["kp"],
            "livestream": ev["stream"],
            "viewing_tip": ev["tip"],
            "event_color": ev["color"],
            "poster": m(f"evt_{i:02d}"),
            "best_expedition": IDS.get(f"expeditions:{ev['exp']}") if ev["exp"] else None,
            "schedule": [{"time": t, "activity": a, "public": p} for t, a, p in ev["schedule"]],
        })


def seed_posts():
    log("field notes")
    for i, p in enumerate(POSTS):
        create("posts", f"{i:02d}", "aurora_detail_post", p["title"], {
            "content": p["body"],
            "deck": p["deck"],
            "featured_image": m(f"post_{i:02d}"),
            "author": IDS[f"guides:{p['author']}"],
            "published_on": p["on"],
            "category": p["category"],
            "mood": p["mood"],
            "tags": p["tags"],
            "reading_minutes": p["mins"],
            "pull_quote": p["pull"],
            "is_featured": p["featured"],
            "palette": p["palette"],
            "from_expedition": IDS.get(f"expeditions:{p['exp']}") if p["exp"] else None,
        })


PALETTE_HEX = {"emerald": "#37ffb0", "violet": "#a58bff", "crimson": "#ff5d7a", "glacier": "#7fe3ff", "solar": "#ffd166", "royal": "#6c8cff"}


def seed_photos():
    log("photographs")
    for key, title, _size, palette, _motif, region, date, exposure, iso, lens, cond, kp, hero, who, caption, _extra in PHOTOS:
        create("photographs", key, "aurora_detail_photo", title, {
            "content": f"<p>{caption}</p>",
            "photo": m(key),
            "caption": caption,
            "location": region,
            "captured_on": date,
            "exposure_seconds": exposure,
            "iso": iso,
            "lens": lens,
            "dominant_color": PALETTE_HEX[palette],
            "conditions": cond,
            "kp_at_capture": kp,
            "is_hero": hero,
            "photographer": IDS[f"guides:{who}"],
        })


def seed_instruments():
    log("instruments")
    for key, it in INSTRUMENTS.items():
        create("instruments", key, "aurora_detail_instrument", it["title"], {
            "content": it["body"],
            "kind": it["kind"],
            "photo": m(f"inst_{key}"),
            "status": it["status"],
            "installed_on": it["installed"],
            "public_data": it["public"],
            "glow": it["glow"],
            "cadence_seconds": it["cadence"],
            "operator": IDS[f"guides:{it['operator']}"],
            "specs": [{"label": l, "value": v, "notable": n} for l, v, n in it["specs"]],
        })


def seed_voices():
    log("voices")
    for i, v in enumerate(VOICES):
        create("voices", f"{i:02d}", "aurora_detail_voice", v["name"], {
            "content": v["story"],
            "quote": v["quote"],
            "hometown": v["town"],
            "rating": str(v["rating"]),
            "tint": v["tint"],
            "avatar": m(f"voice_{i:02d}"),
            "travelled_on": IDS[f"expeditions:{v['exp']}"],
            "verified": v["verified"],
        })


remove_probes()
seed_guides()
seed_expeditions()
seed_events()
seed_posts()
seed_photos()
seed_instruments()
seed_voices()
log(f"done: {len(IDS)} items")
