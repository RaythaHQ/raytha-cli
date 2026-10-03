# build-a-site: from empty Raytha to a live website

Follow this order. Each step is idempotent enough to re-run after a fix.

## 0. Check the connection

```bash
raytha doctor
```

Read `data.permissions`. If a group is missing, `data.hint` names it. Stop and tell the user rather
than working around a missing permission.

## 1. Look at what exists

```bash
raytha theme list
raytha content-type list
raytha site-page list
raytha menu list
```

The active theme (`isActive: true`) is the one the public site renders with.

## 2. Start from the default theme as files

```bash
raytha theme pull raytha_default_theme ./site/my_theme
```

The default theme is `raytha_default_theme`. Read the pulled `web-templates/` and
`widget-templates/`: they are the most accurate reference for Liquid variables and widget settings,
and they include the built-in templates every theme needs (error pages, login pages). Edit
`theme.json` so `developerName` and `title` are your own, then edit the templates. Pushing creates
a new theme under that name; see `raytha guide themes`.

## 3. Design the look and the content model

- Decide the pages (home, about, contact, ...) and the repeating content (posts, products, team
  members, events). Repeating content becomes a content type; one-off pages become site pages.
- Edit the base layout first (`web-templates/raytha_html_base_layout.liquid`): head, fonts, colours,
  header/menu, footer. Every other template inherits it.
- Write widget templates for blocks you will reuse (hero, feature grid, pricing table, testimonial).
  See `raytha guide widgets`.

## 4. Push the theme and activate it

```bash
raytha theme push ./site/my_theme --dry-run
raytha theme push ./site/my_theme --activate
```

`push` creates or updates, never deletes unless `--prune`. A `push_incomplete` error lists which
files failed; fix them and push again.

## 5. Model the content

```bash
raytha content-type create posts --label-plural "Posts" --label-singular "Post"
raytha content-type fields create posts summary --type long_text --label Summary
raytha content-type fields create posts hero_image --type attachment --label "Hero image"
raytha content create posts --data '{"title":"Hello","content":"<p>First post</p>","summary":"Hi"}' --template raytha_html_content_item_detail
```

Give the content type a detail template and a list template that belong to your theme and allow
access for it (`raytha web-template edit my_theme my_post_detail --allow-new-content-types` or
`--content-types posts`). See `raytha guide content-types`.

## 6. Build pages

```bash
raytha site-page create --title Home --template raytha_html_home --home --sections @home.json
raytha site-page create --title About --template raytha_html_page_fullwidth --route-path about --sections @about.json
```

`--sections` is `{"hero":[{"widgetType":"hero","settings":{...}}],"content":[...]}`. The sections
named must be the ones the template renders with `render_section("name")`. See
`raytha guide site-pages` and `raytha site-page widget-definitions` for the settings each widget
accepts.

## 7. Navigation

```bash
raytha menu list
raytha menu items create main --label About --link /about
```

Use `raytha menu set-main <menu>` to choose which menu the `get_main_menu()` Liquid function returns.

## 8. Media

```bash
raytha theme media upload my_theme ./logo.svg
raytha media upload ./photo.jpg
```

Theme media is for the design (logo, css, fonts); the media library is for editorial images. Both
return a URL to put in templates or widget settings. See `raytha guide media`.

## 9. Verify

- `raytha site-page get <id>` shows sections and widgets and whether a draft is pending.
- Before publishing, render a template on the server: `raytha web-template preview <theme> <name> --view <id>`
  (or `--content-item <id>`, drafts included). Syntax errors show up earlier, in `raytha theme push --dry-run`,
  with `line` and `column`.
- Fetch the public page and look at it: `curl -s "$RAYTHA_URL/about"`. A Liquid error renders as an
  error page or empty block; check the HTML, then fix the template and push again.
- `raytha check` requests every public route (home, published views, items, site pages, an unknown path) and
  exits 6 if any fail. With a development server the 500 body names the failing template. Run it after every push.
- `raytha site-page publish <id>` if a draft is pending.

## Tips

- Work from files. Keep the theme directory in git. The server is a deploy target, not the source.
- Use `--dry-run` before `push`, especially with `--prune`.
- Change one thing, push, fetch the page, look. Do not write ten templates before the first test.
- When the CLI lacks a command, `raytha spec` lists the operation and its request body fields
  (see `raytha guide overview`).
