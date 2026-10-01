# CLI roadmap: features that make agents faster at building sites

Ideas from building the Aurora Observatory demo (`examples/aurora-observatory`). Server-side asks are in
[`raytha-feature-requests.md`](raytha-feature-requests.md). Status checked against Raytha 2.6.8.

Priority: **P1** removes the most wasted effort, **P2** clear win, **P3** nice to have.

## Done

| Idea | Now |
|------|-----|
| `raytha check` | `check`: requests home, published views, items, site pages and an unknown path; exit 6 on failure; the failing body (it names the template on a development server) is in the report. |
| `raytha schema export` | `schema export [--out FILE] [--summary]` and `schema import FILE [--dry-run]`, on the server's export/import endpoints. |
| Friendlier `content import` | `content import <type> --file rows.json\|jsonl`: batches of 500, waits for the job, lists failed rows by index. `@file:` values upload attachments, in `content create/edit --data` too. |
| `@ref:` for relationships | Moot. The server resolves relationships by id, route path or primary field value, so rows hold plain values. |
| Compact output (partly) | Views are compact on the server (`--full` expands). `site-page list` is trimmed by the CLI (`--full` expands). |
| Background jobs | `task get\|wait`, and `--wait` on `theme duplicate` and `theme match-templates`. |
| Theme usage | `theme usage [--template NAME \| --unused]`. |
| Functions | `function list\|get\|create\|edit\|delete\|revisions\|revert`, with a `functions` guide topic. |

## Moot

### `raytha theme init`

`theme duplicate <source> <new>` copies templates, widget templates, media and view bindings, so a new design starts
from a working theme and the replacement-view workaround is no longer needed. `theme match-templates` covers
switching an existing site.

## Open

### P1: `raytha apply`: one declarative manifest for a whole site

Much smaller than it was. `schema import` creates content types, fields and views in one call, and `content import`
seeds items with relationships resolved by the server. What is left to cover: menus and items, site pages with their
widgets, theme push, and media uploads, in dependency order, idempotent, with a dry-run diff and per-resource
results. Proposal: `raytha apply site.json|dir [--dry-run]` as a thin driver over the commands that exist.

### P2: Build-time macros in `theme push`

Every themed site needs the same preprocessing, which the demo did in `03_theme.py`.
- `@@media:file.css@@` resolved to the uploaded object key (upload theme media first, replace by name).
- `{{ x | @label:field }}` expanded to a `replace` chain from the field's choices, since dropdown `.Text` returns
  developer names.
- Optional include-style partials (`@@include:chips.liquid@@`).

### P2: `raytha theme lint`

Offline checks before push: unknown tags (`layout`, already caught; syntax errors are caught by `push --dry-run`),
unresolved macros, `render_section` names not matching the page's sections, content types referenced in sidecars
that do not exist, widget field names used in the widget Liquid but missing from its JSON.

### P2: Compact output for the remaining lists

`content list` and `content-type get` still return every field. Add `--fields a,b,c` and a compact default there
too; keep `--full` for the raw payload.

### P2: Local validation in `content import`

Check rows against `schema export` before sending (unknown field, bad choice, wrong type) and print the allowed
values. The server already rejects each bad row with its field name, so this saves a round trip rather than adding
safety.

### P3: `raytha preview <path> [--png]`

`web-template preview` returns server-rendered HTML. A public-route variant that also saves a full-page screenshot
and prints console errors, when headless Chromium is available, would catch layout bugs the HTML cannot show.

### P3: Error hints that name the cause

Mostly covered now that render failures name the template and give a position. Left: hints for a missing theme
binding, an unknown filter `type`, and unresolvable `contentTypes` in a sidecar.

### P3: `guide recipes`

Worked Liquid and CLI snippets from the demo: relationships (`{% capture %}` id compare), repeaters, label mapping,
list pages with filter chips, pagination, custom widgets that take a `view` field. Cover them with the guide parse
test.

### P3: `raytha doctor --deep`

Report the active theme, whether every published view has a template in it, stock sample content still present, and
templates nothing uses (`theme usage --unused`). Overlaps with `check`; keep `doctor` for configuration and
permissions.
