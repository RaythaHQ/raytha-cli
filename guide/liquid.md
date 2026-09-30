# liquid: templating in Raytha

Raytha renders templates with Liquid (the Fluid engine). Output is HTML. The authoritative
examples are the built-in templates: `raytha theme pull raytha_default_theme ./ref` and read them.

## Template kinds

| kind | what it renders | main variable |
|------|-----------------|---------------|
| base layout | the shell (head, header, footer). Contains `{% renderbody %}` | - |
| site page template | a page made of widgets in sections | `render_section("name")` |
| content item list | a view of content items | `Target` (the view) |
| content item detail | one content item | `Target` (the item) |
| widget template | one widget on a page | `widget` |

Non-layout templates do **not** contain a `{% layout %}` tag (Raytha fails at render time with
"Unknown tag 'layout'"). The parent layout is the template's `parent` setting (the `parent` field of the
sidecar JSON, or `--parent` on `web-template create`), and the layout outputs the child with
`{% renderbody %}`.

## Base layout skeleton

```liquid
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% if Target.PrimaryField %}{{ Target.PrimaryField }} | {% endif %}{{ CurrentOrganization.OrganizationName }}</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css">
</head>
<body>
  <nav>
    {% assign menu = get_main_menu() %}
    {% for item in menu.MenuItems %}
      <a href="{{ item.Url }}">{{ item.Label }}</a>
    {% endfor %}
  </nav>
  <main>{% renderbody %}</main>
  <footer>&copy; {{ CurrentOrganization.OrganizationName }}</footer>
</body>
</html>
```

The default theme uses Bootstrap 5 and Bootstrap Icons from a CDN. Replacing them with your own CSS
(inline `<style>`, or a theme media file) is fine.

## Site page template

```liquid
<section>{{ render_section("hero") }}</section>
<div class="container">{{ render_section("main") }}</div>
```

- `render_section("name")` renders the widgets placed in that section, ordered by row then column.
- Section names are discovered from the template source. Pages can only put widgets in sections the
  template renders.
- Options: `render_section("main", wrap=false)` (no row/column wrapper divs),
  `row_class="..."`, `col_class="..."`, `row_id`, `col_id`, `row_attributes`, `col_attributes`.
- On content item templates `render_section` returns an empty string.

## Global variables

```
{{ PathBase }}                               prefix when Raytha runs under a sub-path; use before links
{{ CurrentOrganization.OrganizationName }}   site name
{{ CurrentOrganization.WebsiteUrl }}
{{ CurrentUser.IsAuthenticated }}            public user info; also CurrentUser.IsAdmin
{{ CurrentUser.EmailAddress }}
```

## Content item detail (`Target` is the item)

```
{{ Target.PrimaryField }}                    the item's title-like field
{{ Target.RoutePath }}                       URL path
{{ Target.CreationTime | organization_time | date: "%B %e, %Y" }}
{{ Target.PublishedContent.<field> }}        custom fields by developer name
{{ ContentType.LabelPlural }} {{ ContentType.DeveloperName }}
```

Attachment fields: `{{ Target.PublishedContent.hero_image.Value | attachment_public_url }}` (see the
default detail template for the exact access pattern per field type).

## Content item list (`Target` is the view)

```
{{ Target.Label }}  {{ Target.TotalCount }}  {{ Target.PageNumber }}  {{ Target.TotalPages }}
{% for item in Target.Items %}
  <a href="{{ PathBase }}/{{ item.RoutePath }}">{{ item.PrimaryField }}</a>
{% endfor %}
```

Pagination uses `?pageNumber=N`; the default list template has a complete pager to copy.

## Functions Raytha adds

```liquid
{% assign menu = get_main_menu() %}                               menu.MenuItems: Label, Url, OpenInNewTab, CssClassName, Children
{% assign footer = get_menu("footer") %}                          by developer name
{% assign posts = get_content_items(ContentType="posts", Filter="", OrderBy="CreationTime desc", PageNumber=1, PageSize=3) %}
{% assign post = get_content_item_by_id(id) %}
{% assign ct = get_content_type_by_developer_name("posts") %}
```

`get_content_items` returns an object with `Items` (and `TotalCount`). Use it on the home page to show
the latest posts. Filters use the expression syntax in `raytha guide content-types`.

## Filters Raytha adds (plus all standard Liquid filters)

| filter | use |
|--------|-----|
| `attachment_public_url` | attachment object key to a download URL |
| `attachment_redirect_url` | attachment object key to a stable redirect URL |
| `organization_time` | convert a date to the site's time zone (pipe into `date:`) |
| `groupby: "Property"` | group items; yields `{key, items}` entries |
| `json` | serialize to JSON |

Standard filters such as `strip_html`, `truncate`, `escape`, `default`, `date`, `plus`, `minus`,
`size`, `sort`, `join`, and `downcase` work as in Shopify Liquid.

## Habits that avoid broken pages

- Guard optional values: `{% if x %}...{% endif %}` or `{{ x | default: "..." }}`.
- Escape user text with `escape` where it is not meant to be HTML. WYSIWYG fields are HTML: output
  them unescaped.
- Liquid errors surface when the page renders, not at push time. Push, fetch a page, read the HTML.
- Keep CSS and small scripts in the base layout or in theme media; page templates should be markup.
- Use `{{ PathBase }}` for internal links and `get_main_menu()` for navigation so the site works
  under any URL prefix.
