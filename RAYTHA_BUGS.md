# Raytha bugs and rough edges found while building the Aurora Observatory demo

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

## 1. [unconfirmed] Active theme appeared to clear after deleting a temporary theme

Once, during the re-test, the sequence `theme create zz_theme`, `theme activate zz_theme`, bind a view,
`theme activate aurora`, `theme delete zz_theme` left the site with no active theme: view routes returned 404 and
`views settings` answered `No active theme found` until `theme activate aurora` was run again (the `theme list`
output showed `aurora` as not active). I repeated the same sequence five more times, with and without view
bindings and public page requests in between, and could not reproduce it. Logging it in case the cause is a race
or a cache around `OrganizationSettings.ActiveThemeId`.
