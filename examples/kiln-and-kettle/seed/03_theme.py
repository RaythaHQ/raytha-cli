#!/usr/bin/env python3
"""Duplicate the default theme into `kiln`, overlay the Kiln & Kettle design, validate with a dry run, push and activate.

Compile step (plain text substitution, so the theme sources stay readable):
  @@media:file.css@@     -> object key of the uploaded theme media file
  {{ x | @form }}        -> chained `replace` filters turning choice developer names into labels (from schema.py)
  @@chips:field@@        -> filter buttons (All + every choice); @@chips_only:field@@ omits All
  @@pager@@              -> pagination markup for list templates
"""

import json
import re
import shutil
import sys

from lib import ROOT, jdump, log, rt
from schema import CHOICES

THEME = "kiln"
SRC = ROOT / "theme" / THEME
BUILD = ROOT / ".build"
OUT = BUILD / "theme"


def label_chain(name):
    return " | " + " | ".join(f"replace: '{c['developerName']}', '{c['label']}'" for c in CHOICES[name])


def chips(name, with_all):
    btns = ['<button class="on" data-f="all">All</button>'] if with_all else []
    btns += [f'<button data-f="{c["developerName"]}">{c["label"]}</button>' for c in CHOICES[name]]
    return "".join(btns)


PAGER = """{% if Target.TotalPages > 1 %}
    <nav class="pager" aria-label="Pages">
      <a class="{% if Target.PreviousDisabledCss %}off{% endif %}" href="{{ PathBase }}/{{ Target.RoutePath }}?pageNumber={{ Target.PageNumber | minus: 1 }}">&larr;</a>
      {% for i in (Target.FirstVisiblePageNumber..Target.LastVisiblePageNumber) %}{% if i == Target.PageNumber %}<span class="cur">{{ i }}</span>{% else %}<a href="{{ PathBase }}/{{ Target.RoutePath }}?pageNumber={{ i }}">{{ i }}</a>{% endif %}{% endfor %}
      <a class="{% if Target.NextDisabledCss %}off{% endif %}" href="{{ PathBase }}/{{ Target.RoutePath }}?pageNumber={{ Target.PageNumber | plus: 1 }}">&rarr;</a>
    </nav>
    {% endif %}"""


def compile_text(text, media):
    text = re.sub(r"@@media:([\w.\-]+)@@", lambda m: media[m.group(1)], text)
    text = re.sub(r"@@chips:(\w+)@@", lambda m: chips(m.group(1), True), text)
    text = re.sub(r"@@chips_only:(\w+)@@", lambda m: chips(m.group(1), False), text)
    text = text.replace("@@pager@@", PAGER)
    text = re.sub(r"\|\s*@(\w+)", lambda m: label_chain(m.group(1)), text)
    leftover = re.findall(r"@@[\w:.\-]+@@", text)
    if leftover:
        raise SystemExit(f"unresolved macros: {leftover}")
    return text


def main():
    themes = {t["developerName"] for t in rt("theme", "list", "--all")["items"]}
    if THEME not in themes:
        log(f"duplicating the default theme into {THEME}")
        rt("theme", "duplicate", "raytha_default_theme", THEME, "--title", "Kiln & Kettle", "--description", "Warm paper, kiln-fire accents: a ceramics studio and tea room", "--wait")

    # theme media: replace by name so edits to the css/js always land
    existing = {m["fileName"]: m for m in rt("theme", "media", "list", THEME)}
    media = {}
    for f in sorted((SRC / "media").iterdir()):
        if f.name in existing:
            rt("theme", "media", "delete", THEME, existing[f.name]["id"], "--yes")
        res = rt("theme", "media", "upload", THEME, str(f))
        media[f.name] = res["objectKey"]
        log(f"media {f.name} -> {res['objectKey']}")

    if OUT.exists():
        shutil.rmtree(OUT)
    (OUT / "web-templates").mkdir(parents=True)
    (OUT / "widget-templates").mkdir()
    (OUT / "theme.json").write_text(json.dumps({"title": "Kiln & Kettle", "developerName": THEME, "description": "Warm paper, kiln-fire accents: a ceramics studio and tea room"}))

    # my own sources win; for stock templates I replace (error pages) keep the stock sidecar, re-parented below
    stock = BUILD / "stock"
    if stock.exists():
        shutil.rmtree(stock)
    rt("theme", "pull", "raytha_default_theme", str(stock), "--no-media")

    for sub in ("web-templates", "widget-templates"):
        for f in sorted((SRC / sub).iterdir()):
            body = f.read_text()
            (OUT / sub / f.name).write_text(compile_text(body, media) if f.suffix == ".liquid" else body)

    for f in sorted((stock / "web-templates").iterdir()):
        target = OUT / "web-templates" / f.name
        if f.suffix == ".json":
            meta = json.loads(f.read_text())
            if target.exists():
                continue  # mine
            if meta.get("parent") == "raytha_html_base_layout":
                meta["parent"] = "kk_base_layout"
            meta["contentTypes"] = []
            target.write_text(json.dumps(meta, indent=2))
        elif not target.exists():
            shutil.copy(f, target)
    # a template I override needs the stock sidecar when I did not ship one
    for f in sorted((OUT / "web-templates").glob("*.liquid")):
        side = f.with_suffix(".json")
        if not side.exists():
            raise SystemExit(f"{f.name} has no sidecar")

    dry = rt("theme", "push", str(OUT), "--no-media", "--dry-run", check=False)
    summary = dry.get("summary") if isinstance(dry, dict) else None
    log(f"dry run: {json.dumps(summary)}")
    if isinstance(dry, dict) and dry.get("ok") is False:
        print(json.dumps(dry, indent=1))
        raise SystemExit(1)

    args = ["theme", "push", str(OUT), "--no-media", "--activate"]
    if "--dry-run" in sys.argv:
        return
    res = rt(*args, check=False)
    print(jdump({"summary": res.get("summary"), "changed": [a for a in res.get("actions", []) if a["action"] != "unchanged"][:12]}))
    if res and res.get("ok") is False:
        raise SystemExit(1)


main()
