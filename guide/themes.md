# themes: a theme as a directory of files

A theme is a set of web templates (Liquid layouts and page templates), widget templates (reusable
blocks), and media (images, css, fonts). `raytha theme pull` turns one into files; `raytha theme push`
syncs files back. Work in files, not with one-off API calls.

## Directory layout

```
my_theme/
  theme.json                         {"title": "My theme", "developerName": "my_theme", "description": "..."}
  web-templates/
    raytha_html_base_layout.liquid   template source
    raytha_html_base_layout.json     optional sidecar (see below)
    my_landing.liquid
  widget-templates/
    pricing.liquid                   widget markup
    pricing.json                     {"label": "Pricing", "fields": [ ... ]}
  media/
    logo.svg                         uploaded as theme media
```

The file name (without extension) is the template's developer name: lowercase letters, digits,
underscores.

## Web template sidecar (`<name>.json`, all keys optional)

```json
{
  "label": "Landing page",
  "isBaseLayout": false,
  "parent": "raytha_html_base_layout",
  "allowAccessForNewContentTypes": false,
  "contentTypes": ["posts"]
}
```

- `parent`: the base layout this template extends. The template source does not repeat it: a
  `{% layout %}` tag fails at render time ("Unknown tag 'layout'"). Push creates parents before children.
- `isBaseLayout`: true for layouts. A base layout must contain `{% renderbody %}`; the CLI infers
  this on create when the tag is present.
- `contentTypes` / `allowAccessForNewContentTypes`: which content types may use the template.
  Needed for templates that render content items (list and detail views).
- No sidecar: create uses a humanized label and defaults; update leaves the remote values alone.

## Workflow

```bash
raytha theme pull raytha_default_theme ./my_theme
raytha theme push ./my_theme --dry-run
raytha theme push ./my_theme
raytha theme push ./my_theme --activate
raytha theme push ./my_theme --prune --dry-run
```

- Pull overwrites local files. Pull into a fresh directory, or commit first.
- Push is idempotent: it compares bodies and reports per item `create`, `update`, `unchanged`,
  `delete`, `skip` or `failed`. Re-running a finished push changes nothing.
- If some items fail, the command exits non-zero (the first failure's code) with error code
  `push_incomplete` and the full report in `error.fields.report`. Fix the failed files and push again; successful items are not repeated.
- `--prune` deletes remote templates and media that are not in the directory. Built-in templates
  (error pages, login pages, the default layout) are never deleted. Always `--dry-run` first.
- `--theme other_name` pushes the same directory to a different theme, useful for staging a copy.
- Media: push uploads new files and skips existing ones by name. `--replace-media` re-uploads files
  whose size differs. `--no-media` skips media entirely.
- `--activate` makes the theme live after a successful push.
- Creating a theme makes Raytha add all built-in templates itself. Pushing a pulled default theme
  to a new name therefore reports `unchanged`/`update` for those, and `create` only for your own.

## Checking a template and switching themes

```bash
raytha web-template validate --file layout.liquid     # or --content '<liquid>'; exits non-zero with the parse error
raytha web-template preview my_theme aurora_list_posts --view <view-id> --out /tmp/list.html
raytha theme match-templates new_theme --map aurora_list_posts=new_list_posts,aurora_detail_post=new_detail_post
```

- `validate` only parses the Liquid; it does not render it. Create, edit and `theme push --dry-run` run the same
  check, so syntax errors surface before anything is saved. Errors carry `error.line` and `error.column`.
- `preview` renders a saved template on the server and returns `{html, bytes, title}` (or writes `--out` and returns
  only the size and title). With no flag it renders against an empty target; `--content-item <id>` (drafts work) or
  `--view <id>` renders real data, and the two exclude each other. Runtime failures come back as
  `validation_failed` with the template name in `error.message`; runtime errors have no line or column. Preview is
  the way to check a page without publishing it: push, preview, fix, repeat.
- `match-templates` is for moving a live site to another theme without losing view and item bindings. Each
  `--map OLD=NEW` pairs a template of the **active** theme with one of the new theme (`--map-json` takes an object).
  It runs as a background job and then **makes the new theme active**. Only templates that are still unbound in the
  new theme are accepted, and an empty map just activates. Add `--wait` to block until the job finishes (without it
  you get the task id; `raytha task wait <id>` blocks later). Do not delete the theme before the job is done.

## Copying a theme and seeing what uses a template

```bash
raytha theme duplicate aurora aurora_v2 --wait       # templates, widget templates, media, view bindings
raytha theme usage aurora                            # per template: views, items, site pages, child templates
raytha theme usage aurora --unused                   # custom templates nothing uses (safe to delete)
raytha theme usage aurora --template aurora_base_layout
raytha task get <id>                                 # status of any background job
```

- `duplicate` is the safe way to start a new design: the copy keeps every view and item binding, so you do not need
  `match-templates` or the replacement-view workaround below. Edit the copy, `theme push --dry-run`, then
  `theme activate`.
- Check `theme usage` before deleting or renaming a template. Raytha refuses to delete a web template that a site
  page uses.

## Piece by piece

For a single change, skip the directory:

```bash
raytha web-template get my_theme my_landing --out ./my_landing.liquid
raytha web-template edit my_theme my_landing --file ./my_landing.liquid
raytha web-template create my_theme my_landing --file ./my_landing.liquid --parent raytha_html_base_layout
raytha widget-template edit my_theme pricing --file ./pricing.liquid
raytha theme list
raytha theme activate my_theme
```

## Things to know

- Only one theme is active. Site pages and content render with the active theme's templates.
  A site page's template must belong to the active theme.
- The default theme `raytha_default_theme` is a good base; pull it and change it, do not build layout
  plumbing from nothing.
- Editing a web template replaces its content-type access list. Push and `web-template edit` keep
  the current list unless you provide one.
- Theme media has a stable URL. `raytha theme media list <theme>` returns each file's `url`; use it
  in templates. See `raytha guide media`.
- Template errors show up when a page renders, not when you push. After pushing, fetch a page
  that uses the template and look at the HTML.

## Built-in templates and views (read before `theme create`)

- Every theme must contain Raytha's built-in templates (`raytha_html_*`: error pages, login, list/detail
  fallbacks, page layouts). `theme create` adds them; push a pulled copy of the default theme's built-ins with
  your overrides on top (re-parent them with the sidecar `parent`) rather than deleting them.
- Views and content items bind to templates **per theme**. Prefer `theme duplicate`, which carries the bindings
  over. Views created while another theme was active have no binding in a theme made some other way, and
  `content-type views settings` without `--template` answers 400 "Template is required". Fix with
  `views settings --template <list template>`, or `theme match-templates`, or: `views create` a replacement (it binds to the active theme's built-in list template), `views delete`
  the old one, then run `views settings`. Do this before you run `content create --template ...`, and delete
  stock sample items that have no template in the new theme: one such item makes its whole list view 500.
- After every push, run `raytha check`: it requests every public route (home, views, items, site pages, a missing
  page) and exits 6 when any fail. Template errors return empty bodies in production.
