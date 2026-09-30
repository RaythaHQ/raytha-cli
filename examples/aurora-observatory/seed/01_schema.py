#!/usr/bin/env python3
"""Create the content types and fields. Idempotent: skips what exists."""

from lib import CliError, jdump, log, rt
from schema import CONTENT_TYPES

existing = {c["developerName"] for c in rt("content-type", "list", "--all")["items"]}

for ct in CONTENT_TYPES:
    name = ct["name"]
    if name not in existing:
        log(f"content type {name}")
        rt(
            "content-type", "create", name,
            "--label-plural", ct["plural"], "--label-singular", ct["singular"],
            "--route-template", ct["route"], "--description", ct["description"],
        )
    else:
        rt(
            "content-type", "edit", name,
            "--label-plural", ct["plural"], "--label-singular", ct["singular"],
            "--route-template", ct["route"], "--description", ct["description"],
        )
    fields = {f["developerName"] for f in rt("content-type", "get", name)["contentTypeFields"]}
    # relabel the two fields every new type starts with
    # Relabelling needs `fields edit`, which currently 500s on the server (see RAYTHA_BUGS.md #1).
    for dev, label in [("title", ct["primary"][1]), ("content", ct["content_label"])] + ([("featured_image", "Cover image")] if name == "posts" else []):
        rt("content-type", "fields", "edit", name, dev, "--label", label, check=False)
    for dev, ftype, label, opts in ct["fields"]:
        if dev in fields:
            continue
        args = ["content-type", "fields", "create", name, dev, "--type", ftype, "--label", label]
        if "choices" in opts:
            args += ["--choices-json", jdump(opts["choices"])]
        if "related" in opts:
            args += ["--related-content-type", opts["related"]]
        if "sub" in opts:
            args += ["--sub-fields", jdump(opts["sub"])]
        log(f"  {name}.{dev} ({ftype})")
        rt(*args)

log("schema complete")
