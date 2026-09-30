#!/usr/bin/env bash
# Build the whole Aurora Observatory site on an empty Raytha, using only the `raytha` CLI.
#
#   RAYTHA_URL=http://localhost:5200 RAYTHA_API_KEY=... ./build.sh
#
# Needs: the release CLI (cargo build --release), python3, rsvg-convert and ImageMagick (for the generated art).
set -euo pipefail
cd "$(dirname "$0")/seed"
export RAYTHA_BIN="${RAYTHA_BIN:-$(cd ../.. >/dev/null && pwd)/target/release/raytha}"
"$RAYTHA_BIN" doctor >/dev/null

python3 01_schema.py       # 7 content types, 13 field types
python3 02_media.py        # procedurally drawn art -> media library
python3 03_theme.py        # compile + push the theme (layout, 20 templates, 18 widgets), activate it
python3 04_seed.py         # 70 content items, relationships resolved in dependency order
python3 05_views.py        # list views on the Aurora templates + 11 curated filtered views
python3 06_menus.py        # main + footer menus
python3 07_pages.py        # home, station, plan-your-trip, forecast, contact from custom widgets
