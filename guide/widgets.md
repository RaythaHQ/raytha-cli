# widgets: reusable page blocks

A widget is one block on a site page (hero, feature grid, pricing table). Its **widget template**
has two parts: Liquid markup and a **settings form** (field definitions). Editors fill the form;
the markup reads the values as `widget.settings.<fieldName>`.

## Built-in widgets

Every theme has: `hero`, `wysiwyg`, `imagetext`, `card`, `faq`, `cta`, `embed`, `contentlist`.
Inspect the current list and each one's settings:

```bash
raytha site-page widget-definitions
raytha widget-template list --theme raytha_default_theme
raytha widget-template get raytha_default_theme hero --out ./hero.liquid
```

Built-in names cannot be reused for your own widget templates; copy one under a new name to start.

## Create your own

1. Write the markup. `widget.settings.<name>` holds each field value; guard optional ones.

```liquid
{% assign cols = widget.settings.columns | default: 3 %}
<section class="py-5{% if widget.css_class %} {{ widget.css_class }}{% endif %}" {% if widget.html_id %}id="{{ widget.html_id }}"{% endif %}>
  <div class="container">
    {% if widget.settings.headline %}<h2 class="text-center mb-5">{{ widget.settings.headline }}</h2>{% endif %}
    <div class="row g-4">
      {% for tier in widget.settings.tiers %}
        <div class="col-md-{{ 12 | divided_by: cols }}">
          <div class="card h-100 p-4 text-center">
            <h3>{{ tier.name }}</h3>
            <p class="display-6">{{ tier.price }}</p>
            <div>{{ tier.features }}</div>
          </div>
        </div>
      {% endfor %}
    </div>
  </div>
</section>
```

2. Define the fields (a JSON array) and create the template:

```bash
raytha widget-template create my_theme pricing --label Pricing --file ./pricing.liquid --fields @pricing-fields.json
```

`pricing-fields.json`:

```json
[
  {"developerName": "headline", "label": "Headline", "fieldType": "single_line_text"},
  {"developerName": "columns", "label": "Columns", "fieldType": "number", "defaultValue": 3},
  {"developerName": "theme", "label": "Theme", "fieldType": "dropdown",
   "choices": [{"developerName": "light", "label": "Light"}, {"developerName": "dark", "label": "Dark"}]},
  {"developerName": "tiers", "label": "Tiers", "fieldType": "repeater", "subFields": [
    {"developerName": "name", "label": "Name", "fieldType": "single_line_text", "isRequired": true},
    {"developerName": "price", "label": "Price", "fieldType": "single_line_text"},
    {"developerName": "features", "label": "Features", "fieldType": "wysiwyg"}
  ]}
]
```

3. Update later with `raytha widget-template edit my_theme pricing --file ./pricing.liquid`. The
   settings form only changes when you pass `--fields`.

In a theme directory, put the markup in `widget-templates/pricing.liquid` and the form in
`widget-templates/pricing.json` as `{"label": "Pricing", "fields": [...]}`; `raytha theme push`
syncs both.

## Field types

List them with `raytha widget-template field-types`. Current set:

| fieldType | value stored in settings |
|-----------|--------------------------|
| `single_line_text`, `long_text`, `wysiwyg` | string (wysiwyg is HTML) |
| `number` | JSON number (not a string) |
| `checkbox` | JSON boolean |
| `date` | date string |
| `dropdown`, `radio` | one choice `developerName`; needs `choices` |
| `color` | hex colour such as `#1e293b` |
| `image` | URL string, usually from an upload |
| `repeater` | array of rows; needs `subFields` (no nested repeaters) |
| `content_type` | a content type developer name |
| `view` | a view id of the content type in the sibling field named by `contentTypeField` |

Rules: developer names start with a letter and use letters, digits, underscores (camelCase is fine
and usual); unique per template; every field needs a `label`; only dropdown/radio have `choices`;
only repeater has `subFields`. Values keep their JSON type: send `3`, not `"3"`.

## Placing widgets on a page

Settings are plain objects in the CLI; it stores them as the API expects.

```bash
raytha site-page widgets save <page-id> --section main --data '[{"widgetType":"pricing","settings":{"headline":"Plans","columns":2,"tiers":[{"name":"Free","price":"$0"},{"name":"Pro","price":"$20"}]}}]' --publish
raytha site-page widgets edit <page-id> <widget-id> --section main --settings '{"headline":"New headline"}' --merge --publish
```

Each widget: `widgetType` (alias `type`), `settings` (object), and optional placement `row`, `column`,
`columnSpan` (1-12), plus `cssClass`, `htmlId`, `customAttributes`. Without `row`, widgets stack
in order. Widgets with the same `row` sit side by side: give each a `column` (start, 0-11) and a
`columnSpan`, for example `column` 0 span 6 and `column` 6 span 6.

`raytha site-page widget-definitions` tells you every widget type available on the active theme with
its fields. Validation errors name the field and the problem (for example "Columns must be a number").
See `raytha guide site-pages` for whole-page flows.

## Gotchas

- Widget templates are per theme. A page can only use widget types that exist in the active theme.
- Widgets save to the page's draft. Publish with `--publish` or `raytha site-page publish <id>`.
- Changing a field's `developerName` orphans values already stored on pages for the old name.
- Keep widgets self-contained: CSS classes from the base layout's framework, no hidden dependence
  on other widgets, all copy in settings rather than hard-coded in markup.
