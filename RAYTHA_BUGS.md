# Raytha bugs and rough edges found while building the Aurora Observatory demo

Found against Raytha 2.6.3 (`src/` at commit `0166874`) by driving the v1 API through the `raytha` CLI.
Severity: **bug** = wrong behaviour, **gap** = missing capability, **nit** = papercut.

## 1. [bug] `PUT /api/v1/contenttypes/{ct}/fields/{id}` returns 500 (also delete and reorder)

Any field edit fails:

```
raytha content-type fields edit guides title --label Name
=> 500 Writing values of 'CSharpVitamins.ShortGuid' is not supported for parameters having DataTypeName 'uuid'.
```

Cause: `ContentTypeConfigurationController.GetFieldId` sends
`GetContentTypeFields.Query { ContentTypeId = <ShortGuid> }`, and the handler compares
`p.ContentTypeId == request.ContentTypeId` (`GetContentTypeFields.cs`, ~line 47). EF hands the `ShortGuid`
straight to Npgsql, which cannot write it to a `uuid` parameter. Use `request.ContentTypeId.Guid`.
`DeleteContentTypeField` and `ReorderContentTypeField` share `GetFieldId`, so they hit the same path.

## 2. [gap] v1 API cannot delete a content type

`ContentTypes/{name}` has only `GET`/`PUT`. A fresh install ships a sample `posts` type that an API-only
workflow can neither delete nor rename; the demo repurposes it.

## 3. [nit] Admin API rejects a body-less `POST` with 415

`POST /raytha/api/admin/admins/{id}/api-keys` needs `Content-Type: application/json` and a body
(`{}`) even though the command takes no input. `curl -X POST` without both returns
`415 Mutating requests to the admin API must send Content-Type: application/json.`

## 4. [nit] `HEAD /raytha/media-items/objectkey/{key}` returns 404

The route is mapped with `MapGet` only, so link checkers and CDNs that probe with `HEAD` see a 404 while
`GET` answers `302`. Add `.MapMethods(..., new[] { "GET", "HEAD" }, ...)`.

## 5. [gap] v1 API cannot delete media library items

`MediaItems/{objectKey}` is `GET` only, so an agent that uploads a wrong image cannot remove it (the admin UI
can). Re-uploads leave orphans behind.

## 6. [bug] Views that predate a theme have no binding to it, so `views settings` returns 500

`PUT .../views/{id}/public-settings` (`EditPublicSettings.cs`, line ~139) does
`WebTemplateViewRelations.FirstAsync(wtr => wtr.ViewId == view.Id && wtr.WebTemplate.ThemeId == activeThemeId)`.
A theme made through the API (`POST /themes`) gets its built-in templates but no `WebTemplateViewRelation`
rows for the existing views. Only `BeginDuplicateTheme` and `BeginMatchWebTemplates` (both admin API) add them.
Result: `InvalidOperationException: Sequence contains no elements` and a 500, for every pre-existing view.

An API-only client can work around it by creating new views (`CreateView` binds the active theme's built-in
list template) and deleting the old ones, which is what `seed/05_views.py` does. Suggested fix: make
`EditPublicSettings` create the missing relation (or have theme creation/activation bind all views), and expose
`match-web-templates` in the v1 API.

## 7. [bug] Unknown filter condition type returns 500

`views filter` with `"type":"condition"` fails with `FilterConditionTypeNotFoundException` surfaced as a 500.
The valid values are `filter_condition` and `filter_condition_group`. The exception should map to a 400 that
lists the valid types (the error text does not even say what is allowed).

## 8. [bug] `get_menu("missing")` throws and takes the whole page down with it

`RenderEngine.GetMenuByDeveloperName` lets `NotFoundException` escape from the Liquid call. A layout that
references a footer menu that was not created renders an empty 500 for **every** page, including the 404
page, so the site looks dead with no hint in the response. It should return `nil`/an empty menu (like
`get_content_item_by_id` does for a missing id) or render an inline error.

## 9. [bug] A list view 500s when any item lacks a template for the active theme

`ContentItemListResult_RenderModel.GetProjection` indexes `webTemplateDeveloperNamesByContentItemId[ci.Id]`
(`RenderModels.cs`, line ~141). After switching to a new theme, the stock sample posts have no content-type
template relation in it, so `/posts` (or whichever view lists them) throws `KeyNotFoundException`. Use
`TryGetValue` and fall back to the built-in detail template (or skip the link).

## 10. [bug] Template errors produce an empty response body

A Liquid parse error (`Unknown tag 'layout'`, a bad function argument) or a missing `get_menu` yields a bare
`500` or `400` with `Content-Length: 0`, even when `IsDevelopmentMode` is on. The details only reach the server
log, so an API-driven agent pushing templates gets no feedback. A dev-mode error body (or a
`POST /web-templates/validate` endpoint) would remove most of the guesswork. A related gap: pushing a template
never parses it, so errors only appear on the first public request.

## 11. [gap] Filters on `multiple_select` and relationship fields are unusable

`seasons eq 'jan_feb'` throws `InvalidFilterException` (400, empty body). `lead_guide eq '<id>'` returns 0 rows.
There is also no `contains` for multi-selects in the operator list. Content lists that need "trips running in
January" have to filter in the template or client-side.

## 12. [nit] Theme media URLs are host-absolute

`{{ "key.css" | attachment_public_url }}` renders `http://localhost:5200/_static-files/key_name.css`, using the
request host instead of a root-relative path. Behind a reverse proxy without forwarded-host configuration the
theme CSS/JS points at the wrong origin.

## 13. [gap] No `content-type delete`, `media delete` or template validation in the v1 API

Also missing: moving an item between templates in bulk, and deleting the stock `Hello World` data in one call
(the demo trashes the stock posts item by item). Worth considering for an agent-facing surface.
