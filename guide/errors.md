# errors: what went wrong and how to recover

Every failure prints one JSON document on stdout and exits non-zero:

```json
{"ok": false, "error": {"code": "validation_failed", "message": "...", "hint": "...", "fields": {"Title": ["'Title' must not be empty."]}, "status": 400}}
```

- `code`: stable machine-readable name (below). Branch on this, not on the message.
- `message`: what the server or CLI said.
- `hint`: the next command or fix. Read it first.
- `fields`: per-field validation messages when there are any; names are exactly what Raytha
  reports (for example `Widgets[0].SettingsJson`).
- `status`: the HTTP status when the failure came from the server.

## Exit codes

| exit | class | what to do |
|------|-------|------------|
| 2 | usage / config | Fix the command line or set `RAYTHA_URL` and `RAYTHA_API_KEY`. Nothing reached the server. |
| 3 | auth / permission | Key is wrong (`unauthorized`) or its admin lacks a permission (`forbidden`). Stop; ask the user. |
| 4 | not found | Wrong name or id. List what exists (the hint names the command). |
| 5 | validation | The server rejected the input. Read `fields` and `message`, fix, retry. |
| 6 | server / network | Retry reads once or twice. For writes, check state (`get`) before retrying. |

## Codes

| code | meaning and fix |
|------|-----------------|
| `usage` | Bad flags or arguments. `message` says which; `raytha <command> --help` shows the syntax. |
| `config_error` | `RAYTHA_URL` and/or `RAYTHA_API_KEY` unset or invalid. Every missing one is listed at once. |
| `timeout`, `network_error` | No answer or connection failed (exit 6). Check the URL and that the site is up; raise `--timeout` for big uploads. |
| `unauthorized` | 401. Key rejected or URL points at a different site. Keys are shown once when created. |
| `forbidden` | 403. The key's admin lacks the permission named in `hint`. `raytha doctor` lists what works. |
| `not_found` | 404. Check spelling; developer names are lowercase with underscores. |
| `validation_failed` | 400. Input invalid; see `fields`. |
| `invalid_identifier` | 422. An id was malformed; copy ids exactly from list/get output. |
| `conflict` | 409. Already exists or in use. |
| `payload_too_large` | 413. Upload over the server limit. |
| `rate_limited` | 429. Wait a few seconds and retry. |
| `server_error` | 5xx. Retry once; otherwise the server logs have the cause. |
| `redirected` | The server redirected the API call. `RAYTHA_URL` is probably `http://` when the site forces `https://`, or has a wrong host. Use the final URL. |
| `push_incomplete` | `theme push` applied what it could; `fields.report` lists failures. Fix and push again. |

## Common validation causes

- **Base layout without `{% renderbody %}`**: a template marked as a base layout must contain it.
- **Developer name invalid or taken**: use lowercase letters, digits, underscores. Deleted content type
  fields cannot reuse their name.
- **Site page template not in the active theme**: activate the theme or pick a template from it
  (`raytha web-template list --theme <active-theme>`).
- **Section not in the template**: sections are the names in `render_section("...")` calls.
- **Widget type unknown**: check `raytha site-page widget-definitions`.
- **Widget setting wrong type**: numbers must be JSON numbers, checkboxes booleans, dropdown values one
  of the choice developer names.
- **Content type has no usable template**: give a template access (`--content-types` or
  `--allow-new-content-types` on `web-template edit`).
- **Widget field definitions invalid**: duplicate names, missing labels, dropdown without choices,
  sub-fields on a non-repeater.

## Debugging a page that does not look right

1. `raytha site-page get <id>` or `raytha content get <type> <id>`: is it published, which template?
2. Fetch the public URL (`curl -s "$RAYTHA_URL/path"`) and read the HTML. Liquid errors show up
   here, not at push time.
3. `raytha web-template get <theme> <name> --out /tmp/t.liquid` to see what is live.
4. Change one thing, push, fetch again.

If a needed operation has no command, `raytha spec --summary` lists every API operation. Paths and
methods are right; request body schemas are not reliable, so probe with a small request first.
