# Raytha bugs and rough edges found while building the Aurora Observatory demo

Found against Raytha 2.6.3 by driving the v1 API through the `raytha` CLI. Re-tested against 2.6.4
(working tree with the fixes) on 2026-09-30: all 13 original items are resolved, one only partly (a remainder is
item 2 below). They are removed from this file. What is left is below, plus a new defect and an unconfirmed report
from the re-test.

Severity: **bug** = wrong behaviour, **nit** = papercut.

## Resolved in 2.6.4 (for the record)

Field edit/reorder/delete 500 (was #1), no content type delete (#2), body-less admin POST 415 (#3),
`HEAD` on media (#4), no media delete (#5), views not bound to a new theme (#6; `views settings` now
needs `--template` and creates the binding, and `POST /Themes/{theme}/match-web-templates` exists),
unknown filter type 500 (#7, now a 400 naming the valid types), `get_menu` on a missing menu (#8, returns
nil), list view 500 when an item has no template in the active theme (#9), empty bodies on template errors
(#10; `POST /WebTemplates/validate` and parse-checking on create/edit, and runtime errors now carry a message),
`multiple_select` and relationship `eq` filters (#11), theme media URLs (#12, theme media is now root-relative).
Also new in the v1 API: bulk item delete and `AssignContentItemTemplates`.

## 1. [bug] `DELETE /ContentItems/{type}/items` crashes, and takes no filter

`DeleteContentItems.Handler.PrimaryFieldValue` throws `RuntimeBinderException: Cannot convert type
'System.Text.Json.JsonElement' to 'System.Collections.Generic.Dictionary<string,object>'` (500) on any type that
has items, because it casts the stored JSON dynamically. No items were removed (counts unchanged afterwards), so
it fails before saving.

Design issue while you are in there: the controller action
(`ContentItemsController.DeleteContentItems`) takes no body and no filter, so a successful call would trash
**every** item of the type. That is a dangerous default for an agent-facing API. Consider requiring `ids` or a
`filter` (and an explicit `all=true` to empty a type).

## 2. [nit] `attachment_redirect_url` still emits host-absolute URLs

Theme media (`attachment_public_url`) is now root-relative (`/_static-files/...`), but content attachments
rendered with `{{ item.field.Value | attachment_redirect_url }}` still produce
`http://localhost:5200/raytha/media-items/objectkey/<key>`. Behind a reverse proxy that does not forward the host,
images point at the wrong origin. Make it root-relative like theme media (the `Website URL` setting can still
prefix it for emails and feeds).

## 3. [unconfirmed] Active theme appeared to clear after deleting a temporary theme

Once, during the re-test, the sequence `theme create zz_theme`, `theme activate zz_theme`, bind a view,
`theme activate aurora`, `theme delete zz_theme` left the site with no active theme: view routes returned 404 and
`views settings` answered `No active theme found` until `theme activate aurora` was run again (the `theme list`
output showed `aurora` as not active). I repeated the same sequence five more times, with and without view
bindings and public page requests in between, and could not reproduce it. Logging it in case the cause is a race
or a cache around `OrganizationSettings.ActiveThemeId`.
