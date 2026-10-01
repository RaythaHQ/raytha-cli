# Raytha feature requests for agent-driven site building

Gaps in Raytha itself that make API-only workflows harder than they need to be (status checked against 2.6.8).
Defects are listed separately in [`../RAYTHA_BUGS.md`](../RAYTHA_BUGS.md); this file covers capabilities.

**Status at 2.6.8.** Everything in P1 shipped. Of P2 and P3, still open: field selection on list endpoints other than
views, attachment upload inline with item creation, a theme-activation webhook, and a content type rename.
CLI-side ideas are in [`cli-roadmap.md`](cli-roadmap.md).

Priority: **P1** blocks or badly slows API-only clients, **P2** clear win, **P3** nice to have.

## P1

### Bind views to a theme from the v1 API

**Shipped in 2.6.4 and 2.6.7:** `POST /themes/{theme}/match-web-templates`, `POST /themes/{theme}/duplicate` and
`GET /BackgroundTasks/{id}` to poll the jobs (CLI: `theme match-templates`, `theme duplicate`, `task get|wait`,
`--wait`). Original request follows.

Only `BeginDuplicateTheme` and `BeginMatchWebTemplates` create the view-to-template bindings for a theme, and
both are admin-cookie endpoints. Every API-created theme therefore leaves existing views unusable (see bug #6).
Ask: expose `POST /themes/{id}/match-web-templates` in v1, expose `POST /themes/{id}/duplicate`, or have theme
creation and activation bind all views to the theme's built-in list template automatically.

### Template validation and readable render errors

**Shipped in 2.6.6:** `POST /WebTemplates/validate` and save-time checks return line and column; failed renders name
the template; `GET /WebTemplates/{id}/render-preview` renders with an item or view (CLI: `web-template validate`,
`web-template preview`, and `theme push --dry-run` validates).

A Liquid error returns an empty 500/400 body, and pushing a template never parses it (bug #10). Ask:
- `POST /web-templates/validate` (or validate on create and update) returning the parse error with line and column.
- In development mode, an error body that names the template and the failing tag.
- A `GET /web-templates/{id}/render-preview` (or `?preview=` on a route) that returns the rendered HTML or the
  exception, so agents can verify without a public request.

### Missing v1 operations

**Shipped in 2.6.4:** content type delete, media delete, bulk content delete and bulk template assignment (CLI:
`content-type delete`, `media delete`, `content delete-many`, `content assign-template`). Only the rename item remains open.

- Delete a content type (a fresh install's sample `posts` type cannot be removed by an API client).
- Delete media library items.
- Bulk trash/delete content items, to clear sample data in one call.
- A content-type `rename` that works (field edits return 500, bug #1).

## P2

### Batch content endpoints

**Shipped in 2.6.7:** `POST /ContentItems/{type}/batch` (up to 500 items, per-item results through a polled
background task, relationships by id, route path or primary field value, and rows may refer to each other). CLI:
`content import`. Original request follows.

`POST /contentitems/{type}/batch` accepting an array of items with per-item results, and relationships given as
route path or primary field as well as id. Seeding 70 items took 70 round trips and required dependency ordering
by hand.

### Field selection and compact responses

**Partly shipped:** view endpoints are compact by default (`compact=false` expands; CLI `views list --full`). List
endpoints for items, site pages and content types still return everything, so the CLI trims `site-page list` itself.
Original request follows.

List and get endpoints return the full schema for each item or view (a views list is about 60 KB). Ask for
`?fields=id,label,routePath` or `?compact=true`, and omit `contentType.contentTypeFields` from view responses
unless requested. Matters for agents with limited context.

### Filterable multi-select and relationship fields

**Shipped (verified on 2.6.8):** `contains(seasons, 'jan_feb')` on multiple_select, `lead_guide eq '<id>'` on
relationships, and negations that include items without a value. Original request follows.

Add operators for `multiple_select` (`contains`, `notcontains`) and make relationship equality work (bug #11),
so views such as "expeditions running in January" or "notes by this guide" can be saved views instead of template
loops.

### Schema export/import

**Shipped in 2.6.8:** `GET /ContentTypes/export` and `POST /ContentTypes/import?dryRun=` (CLI: `schema export`,
`schema import`). Original request follows.

`GET /contenttypes/export` returning every type, field, choice and view as one JSON document, and a matching import.
Makes site definitions portable and diffable, and gives agents one call to learn the schema.

### Attachment upload inline with item creation

Accept multipart or a `{"$upload": ...}` value on attachment fields so one call creates the media item and sets
the field, instead of upload, read the object key, create the item.

## P3

### Public, relative media URLs

**Shipped:** media URLs are stored relative to the site. Original request follows.

Theme media and attachment helpers should emit root-relative URLs (bug #12). Optionally add `attachment_url`
variants that stay relative by design.

### Safer Liquid helpers for missing data

**Shipped in 2.6.8:** `get_content_item_by_id`, `get_content_type_by_developer_name` and `get_content_items` return nil
for missing data, and `attachment_url` exists. Original request follows.

`get_menu("missing")` should return an empty menu (bug #8) and `get_content_item_by_id` with a bad id should stay nil.
A Liquid `try`/`default` helper for objects would also help, because one missing object currently kills the page.

### Template dependency reporting

**Shipped in 2.6.8:** `GET /Themes/{theme}/usage` (CLI: `theme usage`), and deleting a web template that site pages
use is refused. Original request follows.

`GET /themes/{id}/usage` listing which views, items and pages bind each template, so agents can tell what a
deleted or renamed template will break.

### Webhook or event for theme activation

Emit a webhook on theme activation and view publish, so external tools (CDN purge, link checkers) can react
without polling.
