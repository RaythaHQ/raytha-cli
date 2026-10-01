#!/usr/bin/env python3
"""Publish the list views on the Kiln templates and add curated filtered views, all in one `raytha schema import`.

The same document as step 1 plus `views`: the default view of each type (routes /shop, /classes ...) and extra views that
filter and sort (routes like /shop/wood-fired). Importing again only changes what differs.
"""

import json
import uuid

from lib import ROOT, log, rt
from schema import document

CONTENT_COLUMNS = {
    "makers": ["name", "role", "years_at_wheel"],
    "pieces": ["title", "form", "price", "in_stock"],
    "workshops": ["title", "format", "starts_on", "price"],
    "teas": ["title", "tea_type", "price_per_pot"],
    "journal": ["title", "category", "published_on"],
}


def view(ct, dev, label, route, template, sort_field, direction, size, cond=None, description=""):
    filt = []
    if cond:
        field, op, value = cond
        filt = [{"id": str(uuid.uuid5(uuid.NAMESPACE_URL, f"{ct}/{dev}")), "parentId": None, "type": "filter_condition", "groupOperator": "and",
                 "field": field, "conditionOperator": op, "value": value}]
    return {"developerName": dev, "label": label, "description": description, "isPublished": True, "routePath": route, "template": template,
            "columns": CONTENT_COLUMNS[ct], "sort": [{"developerName": sort_field, "direction": direction}], "filter": filt,
            "defaultNumberOfItemsPerPage": size, "maxNumberOfItemsPerPage": 200, "ignoreClientFilterAndSortQueryParams": False}


VIEWS = {
    "makers": [view("makers", "makers", "Makers", "makers", "kk_list_makers", "name", "asc", 24)],
    "pieces": [
        view("pieces", "pieces", "Shop", "shop", "kk_list_pieces", "title", "asc", 30),
        view("pieces", "ready_to_ship", "Ready to ship", "shop/ready-to-ship", "kk_list_pieces", "price", "asc", 30, ("in_stock", "eq", "true")),
        view("pieces", "studio_picks", "Studio picks", "shop/studio-picks", "kk_list_pieces", "title", "asc", 30, ("is_featured", "eq", "true")),
        view("pieces", "wood_fired", "Wood-fired", "shop/wood-fired", "kk_list_pieces", "title", "asc", 30, ("firing", "eq", "wood")),
        view("pieces", "under_50", "Under £50", "shop/under-50", "kk_list_pieces", "price", "asc", 30, ("price", "lt", "50")),
        view("pieces", "mugs", "Mugs", "shop/mugs", "kk_list_pieces", "title", "asc", 30, ("form", "eq", "mug")),
    ],
    "workshops": [
        view("workshops", "workshops", "Classes", "classes", "kk_list_workshops", "starts_on", "asc", 20),
        view("workshops", "open_classes", "Open for booking", "classes/open", "kk_list_workshops", "starts_on", "asc", 20, ("is_open", "eq", "true")),
        view("workshops", "beginner_classes", "Beginner classes", "classes/beginners", "kk_list_workshops", "starts_on", "asc", 20, ("level", "eq", "beginner")),
        view("workshops", "weekend_intensives", "Weekend intensives", "classes/weekends", "kk_list_workshops", "starts_on", "asc", 20, ("format", "eq", "weekend_intensive")),
    ],
    "teas": [
        view("teas", "teas", "Tea room", "tea-room", "kk_list_teas", "title", "asc", 30),
        view("teas", "seasonal_teas", "Seasonal teas", "tea-room/seasonal", "kk_list_teas", "title", "asc", 30, ("is_seasonal", "eq", "true")),
        view("teas", "oolongs", "Oolongs", "tea-room/oolong", "kk_list_teas", "title", "asc", 30, ("tea_type", "eq", "oolong")),
    ],
    "journal": [
        view("journal", "journal", "Journal", "journal", "kk_list_journal", "published_on", "desc", 12),
        view("journal", "kiln_notes", "Kiln notes", "journal/kiln-notes", "kk_list_journal", "published_on", "desc", 12, ("category", "eq", "kiln_notes")),
        view("journal", "glaze_lab", "Glaze lab", "journal/glaze-lab", "kk_list_journal", "published_on", "desc", 12, ("category", "eq", "glaze_lab")),
    ],
}

path = ROOT / ".build" / "schema-with-views.json"
path.write_text(json.dumps(document(VIEWS), indent=1, ensure_ascii=False))
plan = rt("schema", "import", str(path), "--dry-run")
log(f"dry run: created {plan['created']}, updated {plan['updated']}, unchanged {plan['unchanged']}")
done = rt("schema", "import", str(path))
log(f"imported: created {done['created']}, updated {done['updated']}, unchanged {done['unchanged']}")
again = rt("schema", "import", str(path), "--dry-run")
log(f"second dry run (should be all unchanged): created {again['created']}, updated {again['updated']}")
