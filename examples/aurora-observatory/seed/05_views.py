#!/usr/bin/env python3
"""Publish the default views on the Aurora list templates and add curated filtered views."""

import uuid

from lib import jdump, log, rt

# content type -> (route, template, sort field, direction, page size)
DEFAULTS = {
    "guides": ("team", "aurora_list_guides", "title", "asc", 24),
    "expeditions": ("expeditions", "aurora_list_expeditions", "departure_date", "asc", 24),
    "sky_events": ("sky", "aurora_list_events", "starts_on", "asc", 24),
    "posts": ("journal", "aurora_list_posts", "published_on", "desc", 12),
    "photographs": ("gallery", "aurora_list_gallery", "captured_on", "desc", 24),
    "instruments": ("instruments", "aurora_list_instruments", "title", "asc", 24),
    "voices": ("voices", "aurora_list_voices", "title", "asc", 24),
}

# content type, view developer name, label, route, field, operator, value, sort field, direction
EXTRA = [
    ("expeditions", "featured_expeditions", "Featured expeditions", "featured-expeditions", "is_featured", "true", None, "departure_date", "asc"),
    ("expeditions", "gentle_expeditions", "Gentle expeditions", "gentle-expeditions", "difficulty", "eq", "gentle", "price_usd", "asc"),
    ("expeditions", "polar_crossings", "Polar crossings", "polar-crossings", "difficulty", "eq", "extreme", "departure_date", "asc"),
    ("sky_events", "storm_watch", "Storm watch", "storm-watch", "event_type", "eq", "storm_watch", "starts_on", "asc"),
    ("sky_events", "live_sky", "Live from the station", "live-sky", "livestream", "true", None, "starts_on", "asc"),
    ("posts", "science_desk", "Science desk", "science-desk", "category", "eq", "science", "published_on", "desc"),
    ("posts", "featured_notes", "Editor's picks", "editors-picks", "is_featured", "true", None, "published_on", "desc"),
    ("photographs", "hero_shots", "Hero shots", "hero-shots", "is_hero", "true", None, "kp_at_capture", "desc"),
    ("instruments", "online_instruments", "Online right now", "online-instruments", "status", "eq", "online", "title", "asc"),
    ("guides", "lead_guides", "Lead guides", "lead-guides", "is_lead_guide", "true", None, "years_experience", "desc"),
    ("voices", "five_star_voices", "Five-star voices", "five-star-voices", "rating", "eq", "5", "title", "asc"),
]

LIST_TEMPLATE = {ct: d[1] for ct, d in DEFAULTS.items()}


def sort(ct, vid, field, direction):
    rt("content-type", "views", "sort", ct, vid, "--data", jdump({"developerName": field, "showColumn": True, "orderByDirection": direction}))


for ct, (route, tpl, field, direction, size) in DEFAULTS.items():
    views = rt("content-type", "views", "list", ct)["items"]
    current = next((v for v in views if v["developerName"] == f"{ct}_all"), None)
    if current is None:
        # The stock view predates this theme, so it has no binding to the theme's list templates
        # (RAYTHA_BUGS.md #6). Swap it for a fresh view, which is bound to the active theme.
        old = views[0]
        current = rt("content-type", "views", "create", ct, f"{ct}_all", "--label", old["label"], "--description", old.get("description") or "")
        for v in views:
            rt("content-type", "views", "delete", ct, v["id"], "--yes")
    vid = current["id"]
    rt("content-type", "views", "settings", ct, vid, "--published", "true", "--route-path", route, "--template", tpl, "--page-size", str(size))
    sort(ct, vid, field, direction)
    log(f"default view {ct} -> /{route}")

existing = {(ct, v["developerName"]): v for ct in DEFAULTS for v in rt("content-type", "views", "list", ct)["items"]}
for ct, dev, label, route, field, op, value, sfield, sdir in EXTRA:
    view = existing.get((ct, dev)) or rt("content-type", "views", "create", ct, dev, "--label", label, "--description", f"{label} (curated by the CLI seed)")
    vid = view["id"]
    cond = {"id": str(uuid.uuid4()), "type": "filter_condition", "groupOperator": "AND", "field": field, "conditionOperator": op}
    if value is not None:
        cond["value"] = value
    rt("content-type", "views", "filter", ct, vid, "--conditions", jdump([cond]))
    rt("content-type", "views", "settings", ct, vid, "--published", "true", "--route-path", route, "--template", LIST_TEMPLATE[ct], "--page-size", "24")
    sort(ct, vid, sfield, sdir)
    log(f"view {ct}/{dev} -> /{route}")
