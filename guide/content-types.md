# content-types: modelling and managing content

A **content type** is a schema (like "Posts" or "Products"): fields, views, and templates. A
**content item** is one record. Use content types for anything repeating; use site pages for
one-off pages.

## Create the type

```bash
raytha content-type create posts --label-plural Posts --label-singular Post --route-template "{CurrentYear}/{PrimaryField}"
raytha content-type get posts
raytha content-type delete posts --yes   # also deletes every item of the type
```

A new type starts with two fields, `title` (single line text, the primary field) and `content`
(wysiwyg), and one public list view named like the type. `--route-template` builds each item's URL
from `{PrimaryField}` (slugified) and `{CurrentYear}`. Default is `{PrimaryField}`.

## The whole model at once

```bash
raytha schema export --summary                 # one line per field, for your own context
raytha schema export --out schema.json         # full document: types, fields, choices, views (diffable)
raytha schema import schema.json --dry-run     # what would be created or updated
raytha schema import schema.json
```

- `schema import` matches by developer name, creates what is missing, updates what differs, never deletes, and
  never changes a field's type. It applies all of it or none of it. The same document works on another site, so
  it is the quickest way to define five related content types in one call instead of dozens of `fields create`.
- Relationship fields name the related type (`"relatedContentType": "authors"`); views name their list template
  (`"template"`) in the active theme.
- `content-type views list|get` leave out each view's field definitions (the server's `compact`, on by default).
  For fields call `content-type get <type>` once or `schema export --summary`; `--full` puts them in every view.

## Fields

```bash
raytha content-type field-types
raytha content-type fields create posts summary --type long_text --label Summary
raytha content-type fields create posts category --type dropdown --label Category --choices "News,Guides,Releases"
raytha content-type fields create posts hero_image --type attachment --label "Hero image"
raytha content-type fields create posts author --type one_to_one_relationship --label Author --related-content-type authors
raytha content-type fields edit posts summary --label "Short summary"
raytha content-type fields reorder posts summary --position 2
raytha content-type fields delete posts summary --yes
```

Types: `single_line_text`, `long_text`, `wysiwyg`, `number`, `date`, `checkbox`, `dropdown`, `radio`,
`multiple_select`, `attachment`, `one_to_one_relationship`, `repeater`, `color`.
Dropdown, radio and multiple_select need `--choices "A,B"` (labels; developer names are derived) or
`--choices-json '[{"label":"A","developerName":"a"}]'`. Repeaters take `--sub-fields` JSON.

Field developer names: lowercase letters, digits, underscores. **A deleted field's developer name
cannot be reused**, so decide names before you create them.

`--primary-field <fieldDeveloperName>` on `content-type edit` chooses which field names the item and
its URL.

## Items

```bash
raytha content create posts --data '{"title":"Hello","content":"<p>First post</p>","summary":"Hi"}' --template raytha_html_content_item_detail
raytha content create posts --file ./post.json --draft
raytha content get posts <id>
raytha content edit posts <id> --data '{"summary":"Shorter"}' --merge
raytha content publish posts <id>
raytha content unpublish posts <id>
raytha content settings posts <id> --route-path blog/hello
raytha content get-by-path posts blog/hello
raytha content import posts --file items.jsonl --template aurora_detail_post
raytha content delete posts <id> --yes
raytha content delete-many posts --ids <id1>,<id2>,<id3> --yes
raytha content delete-many posts --ids-file ids.json --yes
raytha content assign-template posts --template aurora_detail_post --all
raytha content assign-template posts --template aurora_detail_post --ids <id1>,<id2>
raytha content trash posts
raytha content restore posts <id>
raytha content purge posts <id> --yes
```

- `import` creates many items in one batch (a JSON array or JSON Lines file, one object of field values per row,
  up to 500 per request; larger files are split). A relationship field can hold an item id, a route path, or the
  **primary field value** of the related item, and a row may refer to another row of the same file. The related
  item is created first, so seed authors and books in one file. A value `"@file:./img.jpg"` uploads that file and
  stores its object key (also in `content create --data` and `content edit --data`). Rows fail independently:
  the command exits 5 with `error.fields.report.items`, each failed row's `index` and `errors`, while the
  others are created. Re-import only the failed rows.
- `delete-many` trashes several items of one type in one call (all ids must belong to the type, or nothing is
  deleted). `--ids-file` takes a JSON array or ids separated by whitespace; `-` reads stdin. For example,
  `raytha content list posts --all | jq -r '.data.items[].id' | raytha content delete-many posts --ids-file - --yes`
  empties a type, so check the list first.
- `assign-template` re-points existing items at another detail template (name or `--template-id`). It needs
  `--ids`/`--ids-file` or `--all` (every item of the type), and the template must be allowed for the type.
- `--data` is an object of `developerName: value`. Dates are ISO strings; checkbox is a boolean;
  attachment fields take the object key from `raytha media upload`; relationships take the related
  item's id; multiple_select takes an array of choice developer names.
- `content create` publishes unless `--draft`. `--template` names a web template in the **active
  theme** used to render the item; the template must be allowed for this content type. The API
  requires a template, so without `--template` the CLI uses `raytha_html_content_item_detail`.
- Without `--merge`, `content edit` replaces the whole field set; with it only the keys you pass
  change. Prefer `--merge`.
- There is no separate publish endpoint in the API; `content publish` re-saves the current draft
  content as published.
- Delete moves to the trash; `purge` is permanent.

## Listing and filtering

```bash
raytha content list posts --all
raytha content list posts --filter "category eq 'News'" --order-by "CreationTime desc" --page-size 10
raytha content list posts --filter "contains(title, 'launch') and category eq 'News'"
raytha content list posts --search launch --view-id <view-id>
```

Filter syntax is OData-like: `field eq 'x'`, `ne`, `lt`, `le`, `gt`, `ge`, `contains(field,'x')`,
`startswith(field,'x')`, `endswith(field,'x')`, `not ...`, combined with `and` / `or` and
parentheses. Booleans compare as `field eq 'true'`. Order: `Field asc|desc`. Default page size is 50;
use `--all` to fetch every page.

## Views (public lists)

A view is a saved filter, sort and column set with a public URL. The list template renders it.

```bash
raytha content-type views list posts
raytha content-type views create posts featured --label Featured --duplicate-from <view-id>
raytha content-type views settings posts <view-id> --published --route-path blog --template raytha_html_content_item_list --page-size 12
raytha content-type views filter posts <view-id> --conditions @featured-filter.json
raytha content-type views sort posts <view-id> --data '{"developerName":"title","showColumn":true,"orderByDirection":"asc"}'
raytha content-type views set-home posts <view-id>
```

`views sort` takes `developerName` (a field), `showColumn`, and `orderByDirection` (`asc`, `desc` or empty
for none); `views sort-reorder` takes `developerName` and `newFieldOrder`. Setting `--published` and `--route-path` makes `/blog` render the list. Conditions JSON:

```json
[{"id":"1","type":"filter_condition","groupOperator":"AND","field":"category","conditionOperator":"eq","value":"News"}]
```

## Making content visible: the checklist

1. The content type exists and has the fields you render.
2. The active theme has list and detail web templates **allowed for this content type**
   (`--allow-new-content-types` or `--content-types posts` on `raytha web-template edit`).
3. The view is published with a route path and template; items are published with a template.
4. Fetch the public URL and check the HTML.

## Modelling advice for LLM-built sites

- One-off pages (home, about, contact) are site pages. Posts, products, events, team members,
  testimonials are content types.
- Put everything the design needs in fields (images, summary, category) instead of in the body.
- Use an attachment field for images; render with `attachment_public_url` in templates.
- Use a `category` dropdown or a relationship instead of free-text tags you would need to parse.
- Seed a few real-looking items so the list and detail templates can be checked end to end.

## Field value formats (what `--data` takes)

| field type | value |
|------------|-------|
| `single_line_text`, `long_text`, `wysiwyg`, `color` | string (`color` is `#rrggbb`) |
| `number` | number |
| `date` | `"2026-01-14"` |
| `checkbox` | `true` / `false` |
| `dropdown`, `radio` | one choice **developer name** (`"arctic_norway"`). Radio values that look like numbers are still strings (`"5"`) |
| `multiple_select` | array of choice developer names |
| `attachment` | the media item's **object key** from `raytha media upload` (`"<id>_<file>.jpg"`) |
| `one_to_one_relationship` | the related item's **id** |
| `repeater` | array of objects keyed by sub-field developer name; sub-fields may be single_line_text, long_text, wysiwyg, number, checkbox, date, dropdown, radio, color, attachment (no nesting, no relationships) |

In templates, dropdown and radio `.Text` also gives the developer name, not the label. Map names to labels
yourself (`{{ x | replace: 'arctic_norway', 'Arctic Norway' }}`), or generate the chain at build time.
A relationship exposes the related item (`.Id`, `.PrimaryField`, `.RoutePath`, `.PublishedContent`); compare ids
through `{% capture %}` to get string equality.

Filter `type` is `filter_condition` (or `filter_condition_group`). `multiple_select` fields filter with
`contains(seasons, 'jan_feb')` and relationship fields with `lead_guide eq '<item id>'`. Negations (`ne`,
`notcontains`, ...) also match items that have no value. A malformed filter tree is rejected with a 400.
