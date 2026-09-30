#!/usr/bin/env python3
"""Compile the theme and push it with the CLI.

Compile step (plain text substitution, so the theme sources stay readable):
  @@media:file.css@@     -> object key of the uploaded theme media file
  {{ x | @region }}      -> chained `replace` filters turning developer names into labels (from schema.py)
  @@chips:field@@        -> filter buttons (All + every choice); @@chips_only:field@@ omits All
  @@pager@@              -> pagination markup for list templates
"""

import json
import re
import shutil
import sys

from lib import ROOT, log, rt
from schema import CONTENT_TYPES, LODGING

THEME = "aurora"
SRC = ROOT / "theme" / THEME
OUT = ROOT / ".build" / "theme"

# label/choice lookups by the name used in templates
CHOICES = {}
for ct in CONTENT_TYPES:
    for dev, ftype, _label, opts in ct["fields"]:
        if "choices" in opts:
            CHOICES.setdefault(dev, opts["choices"])
CHOICES.update(
    region=CHOICES["region"],
    season=CHOICES["seasons"],
    skill=CHOICES["specialties"],
    language=CHOICES["languages"],
    tag=CHOICES["tags"],
    condition=CHOICES["conditions"],
    lodging=LODGING,
)


def label_chain(name):
    parts = [f"replace: '{c['developerName']}', '{c['label']}'" for c in CHOICES[name]]
    return " | " + " | ".join(parts)


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
        log(f"creating theme {THEME}")
        rt("theme", "create", THEME, "--title", "Aurora Observatory", "--description", "Dark-sky expedition outfitter and observatory theme")

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
    (OUT / "theme.json").write_text(json.dumps({"title": "Aurora Observatory", "developerName": THEME, "description": "Dark-sky expedition outfitter and observatory theme"}))
    for sub in ("web-templates", "widget-templates"):
        for f in sorted((SRC / sub).iterdir()):
            body = f.read_text()
            (OUT / sub / f.name).write_text(compile_text(body, media) if f.suffix == ".liquid" else body)

    # Raytha's built-in templates (login, errors, list/detail fallbacks...) must exist in every theme:
    # views and content types bind to them. Take the stock ones from the default theme, re-parent them
    # onto the Aurora layout, and let anything Aurora overrides win.
    default = ROOT / ".build" / "default"
    if default.exists():
        shutil.rmtree(default)
    rt("theme", "pull", "raytha_default_theme", str(default), "--no-media")
    for f in sorted((default / "web-templates").iterdir()):
        if (OUT / "web-templates" / f.name).exists():
            continue
        if f.suffix == ".json":
            meta = json.loads(f.read_text())
            if meta.get("parent") == "raytha_html_base_layout" and f.stem != "raytha_html_base_login_layout":
                meta["parent"] = "aurora_base_layout"
            meta["contentTypes"] = []
            (OUT / "web-templates" / f.name).write_text(json.dumps(meta, indent=2))
        else:
            shutil.copy(f, OUT / "web-templates" / f.name)

    args = ["theme", "push", str(OUT), "--no-media"]
    if "--dry-run" in sys.argv:
        args.append("--dry-run")
    else:
        args.append("--activate")
    res = rt(*args, check=False)
    print(json.dumps({"summary": res.get("summary"), "changed": [a for a in res.get("actions", []) if a["action"] != "unchanged"]}, indent=1))
    if res and res.get("ok") is False:
        raise SystemExit(1)


main()
