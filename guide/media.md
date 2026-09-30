# media: images and files

Two separate stores:

| store | for | commands | reference in templates |
|-------|-----|----------|------------------------|
| theme media | the design: logo, favicon, css, js, fonts, background images | `raytha theme media ...`, or `media/` in a theme directory | the `url` from `theme media list` |
| media library | editorial files: photos, PDFs, post images | `raytha media ...` | object key stored in fields and settings |

## Theme media

```bash
raytha theme media upload my_theme ./logo.svg
raytha theme media list my_theme
raytha theme media delete my_theme <media-id>
```

`list` returns `id`, `fileName`, `contentType`, `length`, `objectKey` and `url` per file. Use the
`url` (a stable redirect URL on the site) in templates:

```liquid
<img src="https://example.com/raytha/...the url from list..." alt="Logo">
```

Files in a theme directory's `media/` folder upload automatically with `raytha theme push`; existing
files are matched by file name and skipped when the size is the same (`--replace-media` overwrites,
`--no-media` skips).

Prefer an inline `<style>` in the base layout for small CSS; upload larger css/js/fonts as theme media
and link them with the `url`.

## Media library

```bash
raytha media upload ./hero.jpg
raytha media list --all
raytha media get-url <object-key>
```

`upload` returns `{"objectKey": "...", "url": "..."}`. Keep the `objectKey`.

- Content attachment fields take the object key: `raytha content create posts --data '{"title":"Hi","hero_image":"<object-key>"}'`.
- In Liquid, turn an object key into a URL with a filter:
  `{{ Target.PublishedContent.hero_image.Value | attachment_public_url }}` (a storage URL) or
  `{{ key | attachment_redirect_url }}` (a stable URL on the site that redirects to the file). Use
  the redirect URL for anything you publish or cache.
- Widget `image` fields hold a URL string. Use the `url` from `upload`. Check that it is
  reachable from a browser; storage providers may return time-limited URLs, in which case set the
  image from a template using `attachment_redirect_url` with the object key instead.

## Practical rules

- Upload, then immediately record the key or URL in the content or widget settings. Do not rely on
  file names.
- Use web formats: `.jpg`/`.webp` for photos, `.svg` for logos and icons. Keep images reasonably
  small (the server rejects oversize uploads with `payload_too_large`).
- Put alt text in a field (or widget setting) and render it; do not hard-code it.
- There is no public stock-image source in Raytha. If the task needs imagery and none is provided,
  use a solid colour, gradient, or inline SVG; do not hotlink random images.
