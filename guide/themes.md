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

- `parent`: the base layout this template extends. The template itself starts with
  `{% layout 'raytha_html_base_layout' %}`. Push creates parents before children.
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
