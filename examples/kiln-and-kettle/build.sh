#!/usr/bin/env bash
# Build the whole Kiln & Kettle site on an empty Raytha, using only the `raytha` CLI.
#
#   RAYTHA_URL=http://localhost:5200 RAYTHA_API_KEY=... ./build.sh
#
# Needs: the release CLI (cargo build --release), python3, rsvg-convert and ImageMagick (for the generated art).
# Heads up: saving a base layout (one containing {% renderbody %}) is rejected by Raytha 2.0.0 as it stands, see
# RAYTHA_BUGS.md #1. Apply raytha-renderbody-fix.patch to the server first.
set -euo pipefail
cd "$(dirname "$0")/seed"
export RAYTHA_BIN="${RAYTHA_BIN:-$(cd ../../.. >/dev/null && pwd)/target/release/raytha}"
"$RAYTHA_BIN" doctor >/dev/null

python3 00_functions.py    # 6 Raytha functions: pieces.json, llms.txt, sitemap.xml, enquiry, helpers, new-piece event
python3 01_schema.py       # 5 content types, 13 field types
python3 02_media.py        # procedurally drawn art -> media library
python3 03_theme.py        # duplicate the default theme, compile + push the kiln theme, activate it
python3 04_seed.py         # 63 content items, relationships resolved by primary field
python3 05_views.py        # list views on the Kiln templates + curated filtered views
python3 06_menus.py        # main (with a dropdown) + footer menus
python3 07_pages.py        # home, studio, visit, kiln calendar, commissions, gifts from custom widgets
"$RAYTHA_BIN" check --pretty
