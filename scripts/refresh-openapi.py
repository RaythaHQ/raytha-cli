#!/usr/bin/env python3
"""Regenerate tests/fixtures/openapi-v1.json from a running Raytha (default http://localhost:5200).

Keeps paths, methods, parameters and the top-level request-body property names. Raytha 2.0.0 gives
each command its own schema, so those names are the contract `Harness::assert_contract` checks.
"""
import json
import os
import sys
import urllib.request

NOTE = (
    "Snapshot of Raytha's OpenAPI v1 paths: method, operationId, parameters and the top-level "
    "request-body property names. Since Raytha 2.0.0 each command has its own schema, so these "
    "names are the contract."
)

def request_body(op, schemas):
    content = (op.get("requestBody") or {}).get("content") or {}
    if not content:
        return None
    content_type = "application/json" if "application/json" in content else next(iter(content))
    schema = (content.get(content_type) or {}).get("schema") or {}
    ref = schema.get("$ref")
    if ref:
        name = ref.rsplit("/", 1)[-1]
        props = list((schemas.get(name, {}).get("properties") or {}))
    else:
        name = None
        props = list((schema.get("properties") or {}))
    body = {"contentType": content_type, "properties": props}
    if name:
        body["schema"] = name
    return body


base = (sys.argv[1] if len(sys.argv) > 1 else os.environ.get("RAYTHA_URL", "http://localhost:5200")).rstrip("/")
req = urllib.request.Request(
    f"{base}/raytha/api/v1/swagger.json",
    headers={"User-Agent": "raytha-cli-refresh-openapi"},
)
spec = json.load(urllib.request.urlopen(req))
schemas = spec.get("components", {}).get("schemas", {})
paths = {}
for path in sorted(p for p in spec["paths"] if p.startswith("/raytha/api/v1/")):
    ops = {}
    for method, op in spec["paths"][path].items():
        if method not in ("get", "post", "put", "delete", "patch", "head"):
            continue
        entry = {
            "operationId": op.get("operationId"),
            "parameters": [{"in": p["in"], "name": p["name"]} for p in op.get("parameters", [])],
        }
        body = request_body(op, schemas)
        if body is not None:
            entry["requestBody"] = body
        ops[method] = entry
    paths[path] = ops
out = "tests/fixtures/openapi-v1.json"
json.dump({"note": NOTE, "paths": paths}, open(out, "w"), indent=1)
open(out, "a").write("\n")
print(f"{len(paths)} paths")
