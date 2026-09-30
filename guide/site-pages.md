# site-pages: pages made of widgets

A **site page** is a page with its own URL whose body is widgets arranged in **sections**. Its web
template decides which sections exist (each `{{ render_section("name") }}` call); the page decides
which widgets go in them.

## Create a page in one command

```bash
raytha site-page widget-definitions
raytha web-template list --theme my_theme
raytha site-page create --title About --template raytha_html_page_fullwidth --route-path about --sections @about.json
```

`about.json`:

```json
{
  "main": [
    {"widgetType": "hero", "settings": {"headline": "About us", "subheadline": "Who we are", "backgroundColor": "#0f172a"}},
    {"widgetType": "wysiwyg", "settings": {"content": "<p>We build things.</p>"}},
    {"widgetType": "cta", "settings": {"headline": "Talk to us", "buttonText": "Contact", "buttonUrl": "/contact"}}
  ]
}
```

- `--template` is a web template developer name in the **active** theme. Built-in page templates:
  `raytha_html_home`, `raytha_html_page_fullwidth` (section `main`), `raytha_html_page_sidebar`
  (`main`, `sidebar`), `raytha_html_page_multi` (`hero`, `features`, `content`, `cta`).
  Your own templates work the same.
- With `--sections`, create builds the page as a draft, saves the widgets, then publishes. Add
  `--draft` to leave it unpublished for review.
- `--route-path` is the URL (`about`, `company/team`). Without one, the server derives it.
- `--home` makes it the site's home page.
- Sections named in `--sections` must exist in the template; the error says which do not.
- Widget settings are validated against the widget's fields. Errors list the field and problem.

## Read, edit, publish

```bash
raytha site-page list
raytha site-page get <id>
raytha site-page edit <id> --title "About Us"
raytha site-page settings <id> --route-path company/about
raytha site-page sections <id> --sections @about.json --publish
raytha site-page sections <id> --sections @about.json --replace --publish
raytha site-page widgets save <id> --section main --file ./main-widgets.json --publish
raytha site-page widgets edit <id> <widget-id> --section main --settings '{"headline":"New"}' --merge --publish
raytha site-page widgets delete <id> <widget-id> --section main --yes
raytha site-page publish <id>
raytha site-page unpublish <id>
raytha site-page discard-draft <id>
raytha site-page set-home <id>
raytha site-page delete <id> --yes
```

- `get` returns each widget's `settings` as an object plus draft and published state. Widget ids
  come from there.
- `sections` without `--replace` only replaces the sections you list; `--replace` also removes
  sections you omit.
- `widgets save --section s` replaces every widget in that section; send the complete list, or
  `[]` to empty it.
- Widget edits write to the draft. The live page changes only after `--publish` or `site-page publish`.
- `site-page edit` publishes the change unless you pass `--draft`.

## Layout inside a section

Each widget has `row`, `column` (0-11) and `columnSpan` (1-12). Omit them to stack widgets
vertically, one per row, full width. Side by side:

```json
{"main": [
  {"widgetType": "card", "row": 0, "column": 0, "columnSpan": 4, "settings": {"title": "One"}},
  {"widgetType": "card", "row": 0, "column": 4, "columnSpan": 4, "settings": {"title": "Two"}},
  {"widgetType": "card", "row": 0, "column": 8, "columnSpan": 4, "settings": {"title": "Three"}}
]}
```

Optional per-widget `cssClass`, `htmlId`, `customAttributes` flow to the widget markup as
`widget.css_class`, `widget.html_id`, `widget.custom_attributes`.

## Designing a home page

Pick or write a template with sections like `hero`, `features`, `content`, `cta`; put a `hero`
widget in `hero`, three `card` or `imagetext` widgets in `features`, a `contentlist` widget showing
the latest posts in `content`, and a `cta` widget at the end. Then:

```bash
raytha site-page create --title Home --template raytha_html_home --home --sections @home.json
```

Menus are separate: see `raytha menu --help` and `raytha guide build-a-site`.

## Gotchas

- The page's template must be in the active theme. After activating a different theme, pages that
  pointed at the old theme's templates need `raytha site-page edit <id> --template <name>`.
- A widget type must exist in the active theme (`raytha site-page widget-definitions`).
- Changing a page's template does not move widgets between sections that differ in name; check
  `site-page get` afterwards.
- After a publish, fetch the public URL to confirm it renders. Liquid errors appear only at render
  time.
