#!/usr/bin/env python3
"""Seed makers, pieces, workshops, teas and journal entries with `raytha content import`.

Each type is one JSON file. Attachments are `@file:` values (the CLI uploads them), and relationships are written as the
related item's primary field value, so the order below is the dependency order. Re-running skips types that already
have items.
"""

import json

from data_makers import MAKERS
from data_pieces import CARE, PIECES
from data_workshops import JOURNAL, TEAS, WORKSHOPS
from lib import ROOT, img_path, log, rt

BUILD = ROOT / ".build"


def f(name):
    return f"@file:{img_path(name)}"


def import_type(content_type, template, rows):
    if rt("content", "list", content_type, "--page-size", "1")["totalCount"] > 0:
        log(f"{content_type}: already seeded, skipping")
        return
    path = BUILD / f"{content_type}.json"
    path.write_text(json.dumps(rows, indent=1, ensure_ascii=False))
    res = rt("content", "import", content_type, "--file", str(path), "--template", template)
    log(f"{content_type}: {json.dumps(res)[:200]}")


makers = [
    {"name": m["name"], "role": m["role"], "bio": m["bio"], "pull_quote": m["quote"], "portrait": f(f"maker_{k}"), "crafts": m["crafts"],
     "joined_on": m["joined"], "years_at_wheel": m["years"], "signature_glaze": m["glaze"], "is_founder": m["founder"]}
    for k, m in MAKERS.items()
]
import_type("makers", "kk_detail_maker", makers)

pieces = [
    {"title": p["t"], "form": p["form"], "summary": p["sum"], "story": p["story"], "photo": f(f"piece_{p['t']}"), "firing": p["firing"],
     "clay_body": p["clay"], "glaze": p["glaze"], "glaze_colour": p["c"], "made_on": p["made"], "height_cm": p["h"], "width_cm": p["w"],
     "price": p["price"], "in_stock": p["stock"], "is_featured": p["feat"], "features": p["feats"], "maker": MAKERS[p["maker"]]["name"],
     "care": [{"step": s, "detail": d} for s, d in CARE[p.get("care", "default")]]}
    for p in PIECES
]
import_type("pieces", "kk_detail_piece", pieces)

workshops = [
    {"title": w["t"], "tagline": w["tag"], "overview": w["overview"], "banner": f(f"workshop_{w['t']}"), "level": w["level"], "format": w["fmt"],
     "starts_on": w["start"], "sessions": w["n"], "seats": w["seats"], "seats_taken": w["taken"], "price": w["price"], "is_open": w["open"],
     "accent": w["accent"], "includes": w["inc"], "instructor": MAKERS[w["who"]]["name"],
     "curriculum": [{"week": a, "topic": b, "outcome": c} for a, b, c in w["plan"]]}
    for w in WORKSHOPS
]
import_type("workshops", "kk_detail_workshop", workshops)

teas = [
    {"title": t["t"], "tea_type": t["type"], "origin": t["origin"], "tasting_notes": t["tasting"], "brewing_guide": t["guide"],
     "image": f(f"tea_{t['t']}"), "steep_temp_c": t["temp"], "steep_seconds": t["secs"], "price_per_pot": t["price"], "is_seasonal": t["seasonal"],
     "liquor_colour": t["liquor"], "flavour_notes": t["notes"], "served_in": t["cup"],
     "infusions": [{"infusion": a, "seconds": b, "note": c} for a, b, c in t["steps"]]}
    for t in TEAS
]
import_type("teas", "kk_detail_tea", teas)

journal = [
    {"title": j["t"], "category": j["cat"], "excerpt": j["ex"], "body": j["body"], "cover": f(f"journal_{j['t']}"), "published_on": j["date"],
     "reading_minutes": j["mins"], "is_featured": j["feat"], "mood": j["mood"], "tags": j["tags"], "author": MAKERS[j["author"]]["name"],
     "firing_log": [{"hour": a, "temperature": b, "note": c} for a, b, c in j["log"]]}
    for j in JOURNAL
]
import_type("journal", "kk_detail_journal", journal)
