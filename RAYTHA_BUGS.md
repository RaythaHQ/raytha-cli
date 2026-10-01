# Raytha bugs and rough edges found while building the Aurora Observatory and Kiln & Kettle demos

Found against Raytha 2.6.3 by driving the v1 API through the `raytha` CLI. Re-tested against 2.6.4
(working tree with the fixes) on 2026-09-30: all original items are resolved and removed from this file. A
second pass later that day confirmed the two follow-ups (bulk delete by ids, root-relative attachment URLs). The
only thing left is one unconfirmed report from the re-test.

Severity: **bug** = wrong behaviour, **nit** = papercut.

## Resolved in 2.6.4 (for the record)

Field edit/reorder/delete 500 (was #1), no content type delete (#2), body-less admin POST 415 (#3),
`HEAD` on media (#4), no media delete (#5), views not bound to a new theme (#6; `views settings` now
needs `--template` and creates the binding, and `POST /Themes/{theme}/match-web-templates` exists),
unknown filter type 500 (#7, now a 400 naming the valid types), `get_menu` on a missing menu (#8, returns
nil), list view 500 when an item has no template in the active theme (#9), empty bodies on template errors
(#10; `POST /WebTemplates/validate` and parse-checking on create/edit, and runtime errors now carry a message),
`multiple_select` and relationship `eq` filters (#11), host-absolute URLs (#12; theme media and
`attachment_redirect_url` are both root-relative now). Also new in the v1 API: `DELETE /ContentItems/{type}/items`
with `{"ids": [...]}` (rejects empty or foreign ids and trashes nothing on failure; the CLI exposes it as
`content delete-many`) and `AssignContentItemTemplates`.

## Found building Kiln & Kettle (Raytha 2.0.0, `feature/2.0-admin-spa`, 2026-10-01)

### 1. bug: every base layout is rejected ("Unknown tag 'renderbody'")

`POST /WebTemplates`, `PUT /WebTemplates/{id}` and `POST /WebTemplates/validate` all answer `Unknown tag 'renderbody'`
for any template that contains `{% renderbody %}`, so a base layout can no longer be created or edited, and every
template that names it as a parent fails with a `resource not found` cascade. The tag is not a Fluid tag: at render
time `WebTemplateExtensions.RENDERBODY_REGEX` substitutes the child's content into the layout as plain text before
Fluid ever sees it, but `LiquidTemplateParser.GetSyntaxError` (the new parse check) hands the raw source to Fluid.
The same layouts saved fine before the parse check existed.

Fix: blank the tag with same-length whitespace before parsing, so positions stay true. The patch is in
`examples/kiln-and-kettle/raytha-renderbody-fix.patch` (7 lines in `LiquidTemplateParser.cs`). The CLI also blanks the
tag before `web-template validate`, but cannot do anything about create and edit.

### 2. nit: a view filter condition with a non-GUID `id` reports "The request field is required"

`schema import` / `views settings` with a filter condition whose `id` is not a GUID (for example `"id": "price"`)
answers a 400 whose only message is `The request field is required`. The model binder fails on the `Guid` and the
real cause is lost. Naming the property (`filter.conditions[0].id must be a GUID`) would save a lot of guessing.

### 3. nit: `resource not found` hides the real cause when a parent layout failed to save

When the base layout fails (see #1), pushing a child template says `resource not found` for the parent. Reporting
`parent template "kk_base_layout" does not exist` would point at the layout instead of the child.

### 4. nit: `Target.Id == related.Id` is always false in Liquid

On a detail page, `{% if p.PublishedContent.maker.Id == Target.Id %}` never matched in the maker template, although
the maker's own page printed the right id and the other item's relationship printed its related item's id. The two
`Id` values seem to be different types in the Liquid model, so Fluid's `==` treats them as unequal. Comparing `RoutePath` (or `PrimaryField`) works; fixing the comparison would make "items that point at
this item" loops natural to write.

## 5. [unconfirmed] Active theme appeared to clear after deleting a temporary theme

Once, during the re-test, the sequence `theme create zz_theme`, `theme activate zz_theme`, bind a view,
`theme activate aurora`, `theme delete zz_theme` left the site with no active theme: view routes returned 404 and
`views settings` answered `No active theme found` until `theme activate aurora` was run again (the `theme list`
output showed `aurora` as not active). I repeated the same sequence five more times, with and without view
bindings and public page requests in between, and could not reproduce it. Logging it in case the cause is a race
or a cache around `OrganizationSettings.ActiveThemeId`.
