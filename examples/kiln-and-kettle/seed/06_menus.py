#!/usr/bin/env python3
"""Main menu (with a dropdown) and footer menu. The stock `mainmenu` is rebuilt in place."""

from lib import log, rt

MAIN = [
    ("Shop", "/shop", []),
    ("Classes", "/classes", []),
    ("Tea room", "/tea-room", []),
    ("Journal", "/journal", []),
    ("The studio", "/studio", [("Meet the makers", "/makers"), ("Kiln calendar", "/kiln-calendar"), ("Commissions", "/commissions"),
                               ("Gift ideas", "/gifts"), ("Visit &amp; hours", "/visit")]),
]
FOOTER = [
    ("Shop", "/shop"), ("Classes", "/classes"), ("Tea room", "/tea-room"), ("Journal", "/journal"), ("Meet the makers", "/makers"),
    ("The studio", "/studio"), ("Kiln calendar", "/kiln-calendar"), ("Commissions", "/commissions"), ("Gift ideas", "/gifts"), ("Visit & hours", "/visit"),
]


def reset_menu(name, label):
    existing = {m["developerName"] for m in rt("menu", "list")["items"]}
    if name not in existing:
        rt("menu", "create", name, "--label", label)
    for it in rt("menu", "items", "list", name):
        if not it.get("parentNavigationMenuItemId"):
            rt("menu", "items", "delete", name, it["id"], "--yes")


reset_menu("mainmenu", "Main menu")
for label, link, kids in MAIN:
    top = rt("menu", "items", "create", "mainmenu", "--label", label.replace("&amp;", "&"), "--link", link)
    for klabel, klink in kids:
        rt("menu", "items", "create", "mainmenu", "--label", klabel.replace("&amp;", "&"), "--link", klink, "--parent", top["id"])
log(f"mainmenu: {len(MAIN)} items, {sum(len(k) for *_, k in MAIN)} nested")

reset_menu("footer", "Footer")
for label, link in FOOTER:
    rt("menu", "items", "create", "footer", "--label", label, "--link", link)
log(f"footer: {len(FOOTER)} items")
rt("menu", "set-main", "mainmenu")
