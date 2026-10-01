#!/usr/bin/env python3
"""Regenerate tests/fixtures/openapi-v1.json from a running Raytha (default http://localhost:5200)."""
import json
import os
import sys
import urllib.request

base = (sys.argv[1] if len(sys.argv) > 1 else os.environ.get("RAYTHA_URL", "http://localhost:5200")).rstrip("/")
spec = json.load(urllib.request.urlopen(f"{base}/raytha/api/v1/swagger.json"))
old = json.load(open("tests/fixtures/openapi-v1.json"))
paths = {}
for path in sorted(p for p in spec["paths"] if p.startswith("/raytha/api/v1/")):
    ops = {}
    for method, op in spec["paths"][path].items():
        if method not in ("get", "post", "put", "delete", "patch", "head"):
            continue
        ops[method] = {
            "operationId": op.get("operationId"),
            "parameters": [{"in": p["in"], "name": p["name"]} for p in op.get("parameters", [])],
        }
    paths[path] = ops
json.dump({"note": old["note"], "paths": paths}, open("tests/fixtures/openapi-v1.json", "w"), indent=1)
open("tests/fixtures/openapi-v1.json", "a").write("\n")
print(f"{len(paths)} paths")
