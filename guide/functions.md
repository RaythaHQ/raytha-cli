# functions: JavaScript that runs on Raytha

A **Raytha Function** is a small JavaScript program stored in Raytha. Its trigger decides when it runs:

| trigger | runs when |
|---------|-----------|
| `http_request` | a request arrives (at its `--route-path`, such as `llms.txt`) |
| `liquid_template` | a Liquid template calls it |
| `content_item_created` / `content_item_updated` / `content_item_deleted` | a content item changes |

Needs the Manage System Settings permission.

## Commands

```bash
raytha function list
raytha function create llms.txt --trigger http_request --file llms.js --route-path llms.txt
raytha function get llms.txt --out llms.js
raytha function edit llms.txt --file llms.js
raytha function edit llms.txt --active false
raytha function revisions llms.txt
raytha function revert llms.txt <revision-id>
raytha function delete llms.txt --yes
```

- Developer names may contain dots and cannot change. A new function is active unless `--active false`.
- `edit` keeps every value you do not pass. Changing the code saves the old code as a revision, newest first in
  `revisions`; `revert` restores one and saves the current code as a new revision, so a revert can be undone.
- `--route-path ""` removes the public path. `--route-path` only applies to `http_request` functions.
- Work from files: keep `*.js` next to the theme in git, `function get --out` to start from what is live.
- After creating an `http_request` function, request its route and read the status; run `raytha check` for the
  rest of the site.
