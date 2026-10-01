#!/usr/bin/env python3
"""Create or update the Raytha Functions from ../functions/*.js. Safe to re-run."""

from lib import ROOT, log, rt

FUNCTIONS = [
    # developer name, file, trigger, display name, public route path
    ("kiln_helpers", "kiln_helpers.js", "liquid_template", "Kiln helpers (money, glaze names)", None),
    ("pieces_api", "pieces_api.js", "http_request", "Shop as JSON", "pieces.json"),
    ("llms.txt", "llms_txt.js", "http_request", "llms.txt", "llms.txt"),
    ("sitemap_xml", "sitemap_xml.js", "http_request", "Sitemap", "sitemap.xml"),
    ("enquiry", "enquiry.js", "http_request", "Studio enquiry form", "enquiry"),
    ("note_new_piece", "note_new_piece.js", "content_item_created", "Note new pieces", None),
]

existing = {f["developerName"] for f in rt("function", "list", "--all")["items"]}
for dev, file, trigger, name, route in FUNCTIONS:
    path = str(ROOT / "functions" / file)
    if dev in existing:
        args = ["function", "edit", dev, "--file", path, "--name", name]
        if route:
            args += ["--route-path", route]
        rt(*args)
        log(f"updated function {dev}")
    else:
        args = ["function", "create", dev, "--trigger", trigger, "--file", path, "--name", name]
        if route:
            args += ["--route-path", route]
        rt(*args)
        log(f"created function {dev} ({trigger})")
