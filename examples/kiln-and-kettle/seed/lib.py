"""Thin wrapper: every site mutation goes through the `raytha` CLI binary."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = os.environ.get("RAYTHA_BIN") or str(ROOT.parent.parent / "target" / "release" / "raytha")


class CliError(RuntimeError):
    def __init__(self, args, env):
        self.args_, self.env = args, env
        super().__init__(f"raytha {' '.join(map(str, args))[:160]} -> {json.dumps(env.get('error'))[:600]}")


def rt(*args, stdin: str | None = None, check: bool = True):
    """Run the CLI, return the `data` payload of the JSON envelope."""
    proc = subprocess.run([BIN, *map(str, args)], input=stdin, capture_output=True, text=True)
    out = proc.stdout.strip()
    try:
        env = json.loads(out)
    except json.JSONDecodeError:
        if check:
            raise RuntimeError(f"non-JSON output from raytha {args}: {out[:300]} {proc.stderr[:300]}")
        return None
    if not env.get("ok"):
        if check:
            raise CliError(args, env)
        return env
    return env["data"]


def log(msg: str) -> None:
    print(msg, file=sys.stderr, flush=True)


def jdump(value) -> str:
    return json.dumps(value, ensure_ascii=False)


def img_path(name: str) -> Path:
    """Where 02_media.py renders the image called `name` (names may contain spaces and punctuation)."""
    import re

    return ROOT / ".build" / "img" / (re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-") + ".jpg")
