#!/usr/bin/env python3
"""Main navigation and footer menus."""

from lib import log, rt

MAIN = [
    ("Expeditions", "/expeditions"),
    ("Sky calendar", "/sky"),
    ("Field notes", "/journal"),
    ("Gallery", "/gallery"),
    ("The station", "/station"),
    ("Plan a trip", "/plan-your-trip"),
]
FOOTER = [
    ("Our guides", "/team"),
    ("Instruments", "/instruments"),
    ("Traveller voices", "/voices"),
    ("Hero shots", "/hero-shots"),
    ("Storm watch", "/storm-watch"),
    ("Tonight's forecast", "/forecast"),
    ("Contact", "/contact"),
]


def reset_menu(name, label, items):
    existing = {m["developerName"] for m in rt("menu", "list")["items"]}
    if name not in existing:
        rt("menu", "create", name, "--label", label)
    for it in rt("menu", "items", "list", name):
        rt("menu", "items", "delete", name, it["id"], "--yes")
    for text, link in items:
        rt("menu", "items", "create", name, "--label", text, "--link", link, "--css-class", "nav-link")
    log(f"menu {name}: {len(items)} items")


reset_menu("mainmenu", "Main menu", MAIN)
reset_menu("footer", "Footer", FOOTER)
rt("menu", "set-main", "mainmenu")
