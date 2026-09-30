#!/usr/bin/env bash
# Live smoke test of the raytha CLI against a REAL Raytha instance.
#
# It creates (and mostly removes) data: a theme, a content type, a content item, a site page.
# Content types cannot be deleted through the API, so one `smoke_<id>` content type is left behind.
# Run it only against a disposable instance (for example a local Raytha on its own database and
# port), never against a site whose data matters.
#
#   RAYTHA_URL=http://localhost:5299 RAYTHA_API_KEY=... RAYTHA_SMOKE_OK=1 scripts/smoke.sh
#
# Needs: bash, jq, and a built binary (cargo build) or `raytha` on PATH (override with RAYTHA=...).
set -euo pipefail

: "${RAYTHA_URL:?set RAYTHA_URL}"
: "${RAYTHA_API_KEY:?set RAYTHA_API_KEY}"
if [[ "${RAYTHA_SMOKE_OK:-}" != "1" ]]; then
  echo "Refusing to run: this writes to $RAYTHA_URL. Set RAYTHA_SMOKE_OK=1 to confirm it is disposable." >&2
  exit 2
fi

here="$(cd "$(dirname "$0")/.." && pwd)"
RAYTHA="${RAYTHA:-}"
if [[ -z "$RAYTHA" ]]; then
  if [[ -x "$here/target/debug/raytha" ]]; then RAYTHA="$here/target/debug/raytha"; else RAYTHA="raytha"; fi
fi

id="smoke_$(date +%s)"
work="$(mktemp -d)"
page_id=""
item_id=""
pass=0

step() { printf '\n== %s\n' "$*"; }
ok() { pass=$((pass + 1)); printf 'ok: %s\n' "$*"; }
# rt <args...>: run raytha, fail unless {"ok":true}, print the data.
rt() {
  local out
  out="$("$RAYTHA" "$@")" || { echo "FAILED: raytha $*" >&2; echo "$out" >&2; exit 1; }
  echo "$out" | jq -e '.ok == true' >/dev/null || { echo "FAILED: raytha $*" >&2; echo "$out" >&2; exit 1; }
  echo "$out" | jq -c '.data'
}

cleanup() {
  set +e
  [[ -n "$page_id" ]] && "$RAYTHA" site-page delete "$page_id" --yes >/dev/null
  if [[ -n "$item_id" ]]; then
    "$RAYTHA" content delete "$id" "$item_id" --yes >/dev/null
    "$RAYTHA" content purge "$id" "$item_id" --yes >/dev/null
  fi
  "$RAYTHA" theme delete "$id" --yes >/dev/null
  rm -rf "$work"
}
trap cleanup EXIT

step "doctor"
rt doctor | jq -e '.authenticated == true' >/dev/null && ok "authenticated"

step "theme pull -> push (new theme, not activated)"
rt theme pull raytha_default_theme "$work/theme" --no-media >/dev/null
jq --arg id "$id" '.developerName = $id | .title = $id' "$work/theme/theme.json" > "$work/theme.json.new"
mv "$work/theme.json.new" "$work/theme/theme.json"
rt theme push "$work/theme" --no-media | jq -e '.summary' >/dev/null && ok "first push"
second="$(rt theme push "$work/theme" --no-media)"
echo "$second" | jq -e '(.summary.create // 0) == 0 and (.summary.update // 0) == 0' >/dev/null \
  && ok "second push is a no-op"

step "web template round trip"
rt web-template get "$id" raytha_html_base_layout --out "$work/layout.liquid" >/dev/null
echo "<!-- smoke -->" >> "$work/layout.liquid"
rt web-template edit "$id" raytha_html_base_layout --file "$work/layout.liquid" >/dev/null
rt web-template get "$id" raytha_html_base_layout | jq -e '.content | contains("<!-- smoke -->")' >/dev/null \
  && ok "template edit persisted"

step "content type, fields, item"
rt content-type create "$id" --label-plural "Smoke ${id}" --label-singular "Smoke" >/dev/null
rt content-type fields create "$id" summary --type long_text --label Summary >/dev/null
item="$(rt content create "$id" --data '{"title":"Smoke item","content":"<p>hi</p>","summary":"s"}' --draft)"
item_id="$(echo "$item" | jq -r '.id')"
rt content edit "$id" "$item_id" --data '{"summary":"merged"}' --merge --draft >/dev/null
rt content get "$id" "$item_id" | jq -e '.' >/dev/null && ok "content create/edit/get"
rt content list "$id" --all | jq -e '.items | length >= 1' >/dev/null && ok "content list"

step "site page (needs a template in the ACTIVE theme)"
page="$(rt site-page create --title "Smoke $id" --template raytha_html_page_fullwidth --draft \
  --sections '{"main":[{"type":"wysiwyg","settings":{"content":"<p>smoke</p>"}}]}')"
page_id="$(echo "$page" | jq -r '.id')"
rt site-page get "$page_id" | jq -e '.widgets.main | length == 1' >/dev/null && ok "site page with a widget"

step "spec"
rt spec --summary | jq -e '.operations | length > 10' >/dev/null && ok "spec"

printf '\nAll smoke checks passed (%d).\n' "$pass"
