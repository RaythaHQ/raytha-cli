# CLI roadmap: features that make agents faster at building sites

Ideas from building the Aurora Observatory demo (`examples/aurora-observatory`). Everything here can be done in
this repo without changing Raytha. Items that need server work are in [`raytha-feature-requests.md`](raytha-feature-requests.md).

Priority: **P1** removes the most wasted effort, **P2** clear win, **P3** nice to have.

## P1

### `raytha apply`: one declarative manifest for a whole site

**Problem.** The demo needed seven ordered scripts (schema, media, theme, items, views, menus, pages). Agents
must learn the ordering rules (relationships need ids, views need templates, templates need content types).

**Proposal.** `raytha apply site.json|dir [--dry-run] [--prune]`.
- Covers content types and fields, items, views, menus and site pages.
- Idempotent, with a diff in dry-run, like `theme push`.
- Resolves relationships by slug or primary field instead of id.
- Uploads attachments from file paths and substitutes the object key.
- Applies in dependency order and reports per-resource results.

### `raytha check`: verify the live site

**Problem.** Liquid errors return empty 500 bodies, so a push that looks fine can produce a dead site. The demo
was verified with hand-written curl loops.

**Proposal.** `raytha check [--routes auto]`.
- Enumerates every public route: views, items, site pages, the 404.
- Requests each one, then reports status, size and leftover `{{`/`{%` text.
- When the server is local, tails the log for the exception and prints it.
- Exit code 6 on any failure so agents can loop on it.

### `raytha theme init`

**Problem.** A new API-created theme has the built-in templates but no working view bindings, and the built-ins
still use the default layout. The demo needed a custom compile step and a view-swap workaround.

**Proposal.** `raytha theme init <name> [--from raytha_default_theme]`.
- Creates the theme, copies the built-in `raytha_html_*` templates, and re-parents them onto your layout.
- Rebinds existing views (client-side workaround: create replacement views, delete old ones) until Raytha offers
  a real fix.
- Optionally activates the theme.

## P2

### Build-time macros in `theme push`

**Problem.** Every themed site needs the same preprocessing, which the demo did in `03_theme.py`.

**Proposal.** Built into `push`:
- `@@media:file.css@@` resolved to the uploaded object key (upload theme media first, replace by name).
- `{{ x | @label:field }}` expanded to a `replace` chain from the field's choices, since dropdown `.Text`
  returns developer names.
- Optional include-style partials (`@@include:chips.liquid@@`) so agents stop copy-pasting pager/chip markup.

### `raytha theme lint`

Offline checks before push: unknown tags (`layout`, already caught), unresolved macros, `render_section` names
not matching the page's sections, content types referenced in sidecars that do not exist, widget field names
used in the widget Liquid but missing from its JSON, `{% ... %}` imbalance.

### Compact output by default

**Problem.** `content-type views list` returns ~60 KB of field metadata per call, and `site-page get` embedded
whole template sources (slimmed in this repo already).

**Proposal.** Default to compact projections on every `list` and `get` (id, name, routes, status), with `--full`
or `--fields a,b,c` for the raw payload. Keep `--raw` for the exact API response.

### `raytha schema export`

Dump content types, fields, choices, sub-fields, relationships and view routes as JSON or a one-page markdown
summary, suitable for pasting into an agent's context before it writes templates.

### Friendlier `content create` / `content import`

- `--data @file.json` already works. Add `content import <type> items.jsonl` for batches with progress and
  per-row errors.
- Local validation against the schema first (unknown field, bad choice, wrong type) with the allowed values in the
  error.
- Accept `"@file:./img.jpg"` for attachments and `"@ref:guides/Ingrid-Solheim"` for relationships.

## P3

### `raytha preview <path> [--png]`

Fetch a public route and, when headless Chromium is available, save a full-page screenshot and print console
errors. Visual inspection was the only way to catch layout bugs in the demo.

### Error hints that name the cause

Map known failures to specific hints instead of the generic "Retry once": missing theme binding, unknown filter
`type`, `get_menu` on a missing menu, unresolvable `contentTypes` in a sidecar. Read the server log path from
config when the server is local.

### `guide recipes`

A guide topic of worked Liquid and CLI snippets that exist today only in this repo's example: relationships
(`{% capture %}` id compare), repeaters, label mapping, list pages with client-side filter chips, pagination,
custom widgets that take a `view` field. Keep the snippets covered by the guide parse test so they stay valid.

### `raytha doctor --deep`

Extend `doctor` to report active theme, whether every published view has a template in it, stock sample content
still present, and menus referenced by the layout but missing.
