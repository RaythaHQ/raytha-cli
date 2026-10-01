#!/usr/bin/env python3
"""Define the five content types with one `raytha schema import` (dry run first), then drop the stock `posts` type."""

import json

from lib import ROOT, log, rt
from schema import document

BUILD = ROOT / ".build"
BUILD.mkdir(exist_ok=True)
path = BUILD / "schema.json"
path.write_text(json.dumps(document(), indent=1))

plan = rt("schema", "import", str(path), "--dry-run")
log(f"dry run: {json.dumps(plan.get('summary') or plan)[:300]}")
applied = rt("schema", "import", str(path))
log(f"imported: {json.dumps(applied.get('summary') or applied)[:300]}")

existing = {c["developerName"] for c in rt("content-type", "list")["items"]}
if "posts" in existing:
    # The stock "Posts" type (and its four demo items) is not part of this site.
    rt("content-type", "delete", "posts", "--yes")
    log("removed the stock posts type")