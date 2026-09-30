---
name: raytha
description: Build and manage a Raytha website (themes, Liquid templates, widgets, site pages, content types, content, media, menus) with the raytha CLI. Use when asked to create or change pages, themes, content or navigation on a Raytha site, or when RAYTHA_URL and RAYTHA_API_KEY are set.
---

# Raytha CLI

The `raytha` binary manages a Raytha site through its REST API. It prints JSON and ships its own
docs.

## Start here

```bash
raytha doctor                 # confirms RAYTHA_URL, RAYTHA_API_KEY, and permissions
raytha guide                  # list topics
raytha guide build-a-site     # the end-to-end recipe: read this before changing anything
```

Other topics: `overview`, `themes`, `liquid`, `widgets`, `content-types`, `site-pages`, `media`,
`errors`. The docs match the installed version, so prefer them over memory.

## Rules

- Configuration is `RAYTHA_URL` and `RAYTHA_API_KEY` (flags `--url`, `--api-key` override).
- stdout is one JSON document: `{"ok":true,"data":...}` or `{"ok":false,"error":{code,message,hint,...}}`.
  Exit codes: 0 ok, 2 usage, 3 auth or permission, 4 not found, 5 validation, 6 server.
  Follow `error.hint`. On exit 3 stop and tell the user which permission is missing.
- Work from files: `raytha theme pull raytha_default_theme ./site`, edit, then
  `raytha theme push ./site --dry-run` and `raytha theme push ./site`. Keep the directory in git.
- Destructive commands need `--yes`. Edits are partial; omitted flags keep their value.
- Site pages and widgets save to a draft; publish with `--publish` or `raytha site-page publish`.
- After changes, fetch the public URL and read the HTML. Liquid errors appear only at render time.
- If no command covers a need, `raytha spec --summary` lists API operations (paths and methods are
  reliable, request body schemas are not).
