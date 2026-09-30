# raytha: overview

`raytha` manages a Raytha site (themes, templates, widgets, pages, content, media, menus, users)
through its REST API. It is built for LLM agents: every command prints JSON, exit codes are stable,
and errors say how to fix the problem.

Start with `raytha doctor`, then `raytha guide build-a-site`.

## Configuration

Two environment variables. Flags override them.

```
RAYTHA_URL       site root, e.g. https://example.com   (--url)
RAYTHA_API_KEY   an admin API key                      (--api-key)
```

The key is created in the admin under Settings > Administrators > (admin) > API Keys. It has
exactly the permissions of that administrator. `raytha doctor` shows which command groups the key
can use.

## Output contract

- stdout is always exactly one JSON document. Success: `{"ok":true,"data":...}`.
  Failure: `{"ok":false,"error":{"code","message","hint","fields","status"}}`.
- Exceptions: `raytha guide` prints markdown, `--help` and `--version` print text.
- Nothing else is written to stdout. Progress and warnings never mix into the JSON.
- `--pretty` indents the JSON. Leave it off to save tokens; pipe through `jq` to slice.

## Exit codes

| code | meaning |
|------|---------|
| 0 | success |
| 2 | usage or configuration problem (bad flag, missing env var) |
| 3 | authentication or permission problem |
| 4 | not found |
| 5 | validation error (the server rejected the input; see `error.fields`) |
| 6 | server or network problem (safe to retry GETs; inspect before retrying writes) |

## Conventions

- Identify things by developer name where Raytha allows it (themes, content types, templates,
  menus) and by id otherwise (site pages, content items). Ids are short GUID strings.
- Developer names are lowercase letters, digits and underscores. The CLI normalizes
  `"My Theme"` to `my_theme` where it creates things.
- Long input: `--file path`, `--data @path`, or `--data -` for stdin. Prefer files for anything longer
  than a line. Flags that take JSON accept inline JSON, `@path`, or `-`.
- Lists return `{"items":[...],"totalCount":N,"page":1,"pageSize":50,"hasMore":bool}`. Use `--all`
  to fetch every page; `--page-size` sets the page size (default 50).
- Edits are partial: flags you omit keep their current value. The CLI reads the current object and
  merges before it writes.
- Destructive commands (`delete`, `purge`) require `--yes`. `theme push` takes `--dry-run`; run it
  first, especially with `--prune`.
- Drafts: site pages and content have a draft and a published version. Commands say whether they
  publish; the defaults are noted in each command's `--help`.

## Command map

```
raytha doctor                              connectivity, key, permissions
raytha guide [topic]                       these docs
raytha spec [--summary] [--path text]      live OpenAPI operations (for gaps only)

raytha theme ...                           list get create edit delete activate pull push media
raytha web-template ...                    layouts and page templates (liquid)
raytha widget-template ...                 reusable blocks with a settings form
raytha site-page ...                       pages made of widgets in sections
raytha content-type ...                    schema: fields and views
raytha content ...                         items: create edit publish list trash
raytha media ...                           site media library
raytha menu ...                            navigation menus and items
raytha user / user-group ...               public users and groups
```

Every group has `--help`. Topics: overview, build-a-site, themes, liquid, widgets, content-types,
site-pages, media, errors.

## Known quirks of the Raytha API

- The OpenAPI request-body schemas (`raytha spec`) are unreliable: several bodies share a name and
  show the wrong shape. Paths, methods and query parameters are right. Use the curated commands.
- There is no separate "publish" endpoint for content. `raytha content publish` re-saves the item
  as published.
- Saving widgets writes the page's draft. Publish the page afterwards (`--publish`).
- A site page's template must belong to the active theme.
- Editing a web template replaces its list of content types that may use it. The CLI carries the
  current list over unless you pass `--content-types`.
- A new content type needs a template that has "allow access for new content types" on before it
  renders publicly.
