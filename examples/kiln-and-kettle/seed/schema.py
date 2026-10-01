"""The Kiln & Kettle content model: five content types that between them use all 13 Raytha field types.

  makers     wysiwyg, long_text, single_line_text, attachment, multiple_select, date, number, color, checkbox, dropdown
  pieces     + radio, one_to_one_relationship (maker), repeater (care)
  workshops  + relationship (instructor), repeater (curriculum)
  teas       + relationship (served_in a piece), repeater (brewing steps)
  journal    + relationship (author), repeater (firing log)

`document()` returns the portable document accepted by `raytha schema import`.
"""

from __future__ import annotations


def ch(*pairs):
    """('wood', 'Wood-fired') or just 'Wood' -> choice objects."""
    out = []
    for p in pairs:
        dev, label = p if isinstance(p, tuple) else (p.lower().replace(" ", "_").replace("&", "and").replace("-", "_"), p)
        out.append({"developerName": dev, "label": label})
    return out


def f(dev, ftype, label, required=False, **kw):
    return {"developerName": dev, "label": label, "description": kw.pop("description", ""), "fieldType": ftype, "isRequired": required,
            "choices": kw.pop("choices", []), "relatedContentType": kw.pop("rel", None), "subFields": kw.pop("sub", [])}


def sub(dev, ftype, label, **kw):
    return {"developerName": dev, "label": label, "fieldType": ftype, **kw}


ROLES = ch(("master_potter", "Master potter"), ("glaze_chemist", "Glaze chemist"), ("wheel_teacher", "Wheel teacher"),
           ("tea_master", "Tea master"), ("studio_manager", "Studio manager"), ("apprentice", "Apprentice"))
CRAFTS = ch(("wheel", "Wheel throwing"), ("handbuilding", "Handbuilding"), ("slip_casting", "Slip casting"), ("raku", "Raku"),
            ("wood_firing", "Wood firing"), ("glaze_chemistry", "Glaze chemistry"), ("tea_ceremony", "Tea ceremony"),
            ("kintsugi", "Kintsugi repair"))
FORMS = ch(("bowl", "Bowl"), ("vase", "Vase"), ("mug", "Mug"), ("teapot", "Teapot"), ("plate", "Plate"),
           ("planter", "Planter"), ("lantern", "Lantern"), ("tea_cup", "Tea cup"), ("jug", "Jug"))
FIRINGS = ch(("oxidation", "Electric oxidation"), ("reduction", "Gas reduction"), ("wood", "Wood fired"), ("raku", "Raku"), ("salt", "Salt fired"))
CLAYS = ch(("stoneware", "Stoneware"), ("porcelain", "Porcelain"), ("terracotta", "Terracotta"), ("black_clay", "Black clay"))
FEATURES = ch(("dishwasher_safe", "Dishwasher safe"), ("food_safe", "Food safe"), ("microwave_safe", "Microwave safe"),
              ("oven_safe", "Oven safe"), ("one_off", "One of a kind"), ("signed", "Signed on the foot"))
LEVELS = ch(("beginner", "Beginner"), ("intermediate", "Intermediate"), ("advanced", "Advanced"), ("all_levels", "All levels"))
FORMATS = ch(("evening_class", "Evening class"), ("weekend_intensive", "Weekend intensive"), ("drop_in", "Drop-in session"),
             ("private", "Private lesson"), ("family", "Family morning"), ("tea_pairing", "Tea pairing"))
INCLUDES = ch(("clay", "All clay"), ("glazing", "Glazing"), ("firing", "Firing"), ("tea", "Tea and biscuits"),
              ("apron", "Apron to keep"), ("take_home", "Take-home piece"))
TEA_TYPES = ch(("green", "Green"), ("white", "White"), ("oolong", "Oolong"), ("black", "Black"), ("pu_erh", "Pu-erh"), ("herbal", "Herbal"))
ORIGINS = ch(("uji", "Uji, Japan"), ("fujian", "Fujian, China"), ("yunnan", "Yunnan, China"), ("darjeeling", "Darjeeling, India"),
             ("alishan", "Alishan, Taiwan"), ("assam", "Assam, India"), ("local", "Grown in Wales"), ("shizuoka", "Shizuoka, Japan"))
NOTES = ch(("floral", "Floral"), ("honey", "Honey"), ("toasted", "Toasted"), ("mineral", "Mineral"), ("citrus", "Citrus"),
           ("stone_fruit", "Stone fruit"), ("smoke", "Smoke"), ("vegetal", "Vegetal"), ("malty", "Malty"), ("spice", "Spice"))
CATEGORIES = ch(("kiln_notes", "Kiln notes"), ("glaze_lab", "Glaze lab"), ("studio_life", "Studio life"), ("tea_stories", "Tea stories"), ("how_to", "How-to"))
TAGS = ch(("firing", "Firing"), ("glaze", "Glaze"), ("clay", "Clay"), ("tea", "Tea"), ("community", "Community"), ("tools", "Tools"),
          ("seasonal", "Seasonal"), ("experiments", "Experiments"))

CONTENT_TYPES = [
    {
        "developerName": "makers", "labelPlural": "Makers", "labelSingular": "Maker",
        "description": "The potters, glaze chemists and tea people of the studio.",
        "defaultRouteTemplate": "makers/{PrimaryField}", "primaryField": "name",
        "fields": [
            f("name", "single_line_text", "Name", True),
            f("role", "dropdown", "Role", True, choices=ROLES),
            f("bio", "wysiwyg", "Biography"),
            f("pull_quote", "long_text", "Pull quote"),
            f("portrait", "attachment", "Portrait"),
            f("crafts", "multiple_select", "Crafts", choices=CRAFTS),
            f("joined_on", "date", "Joined the studio"),
            f("years_at_wheel", "number", "Years at the wheel"),
            f("signature_glaze", "color", "Signature glaze colour"),
            f("is_founder", "checkbox", "Founder"),
        ],
    },
    {
        "developerName": "pieces", "labelPlural": "Pieces", "labelSingular": "Piece",
        "description": "Finished ceramics for sale: one of a kind and small runs.",
        "defaultRouteTemplate": "shop/{PrimaryField}", "primaryField": "title",
        "fields": [
            f("title", "single_line_text", "Title", True),
            f("form", "dropdown", "Form", True, choices=FORMS),
            f("summary", "long_text", "Summary"),
            f("story", "wysiwyg", "The story"),
            f("photo", "attachment", "Photograph"),
            f("firing", "radio", "Firing", choices=FIRINGS),
            f("clay_body", "dropdown", "Clay body", choices=CLAYS),
            f("glaze", "single_line_text", "Glaze"),
            f("glaze_colour", "color", "Glaze colour"),
            f("made_on", "date", "Made on"),
            f("height_cm", "number", "Height (cm)"),
            f("width_cm", "number", "Width (cm)"),
            f("price", "number", "Price (GBP)", True),
            f("in_stock", "checkbox", "In stock"),
            f("is_featured", "checkbox", "Featured"),
            f("features", "multiple_select", "Good to know", choices=FEATURES),
            f("maker", "one_to_one_relationship", "Made by", rel="makers"),
            f("care", "repeater", "Care instructions", sub=[
                sub("step", "single_line_text", "Step", isRequired=True), sub("detail", "long_text", "Detail")]),
        ],
    },
    {
        "developerName": "workshops", "labelPlural": "Workshops", "labelSingular": "Workshop",
        "description": "Classes and sessions you can book at the studio.",
        "defaultRouteTemplate": "classes/{PrimaryField}", "primaryField": "title",
        "fields": [
            f("title", "single_line_text", "Title", True),
            f("tagline", "long_text", "Tagline"),
            f("overview", "wysiwyg", "Overview"),
            f("banner", "attachment", "Banner image"),
            f("level", "radio", "Level", True, choices=LEVELS),
            f("format", "dropdown", "Format", True, choices=FORMATS),
            f("starts_on", "date", "First session"),
            f("sessions", "number", "Number of sessions"),
            f("seats", "number", "Seats"),
            f("seats_taken", "number", "Seats taken"),
            f("price", "number", "Price (GBP)", True),
            f("is_open", "checkbox", "Open for booking"),
            f("accent", "color", "Accent colour"),
            f("includes", "multiple_select", "Included", choices=INCLUDES),
            f("instructor", "one_to_one_relationship", "Led by", rel="makers"),
            f("curriculum", "repeater", "Session by session", sub=[
                sub("week", "single_line_text", "Session", isRequired=True), sub("topic", "single_line_text", "Topic"),
                sub("outcome", "long_text", "What you make")]),
        ],
    },
    {
        "developerName": "teas", "labelPlural": "Teas", "labelSingular": "Tea",
        "description": "The tea room menu.",
        "defaultRouteTemplate": "tea-room/{PrimaryField}", "primaryField": "title",
        "fields": [
            f("title", "single_line_text", "Name", True),
            f("tea_type", "radio", "Type", True, choices=TEA_TYPES),
            f("origin", "dropdown", "Origin", choices=ORIGINS),
            f("tasting_notes", "long_text", "Tasting notes"),
            f("brewing_guide", "wysiwyg", "Brewing guide"),
            f("image", "attachment", "Photograph"),
            f("steep_temp_c", "number", "Water temperature (C)"),
            f("steep_seconds", "number", "First steep (seconds)"),
            f("price_per_pot", "number", "Price per pot (GBP)", True),
            f("is_seasonal", "checkbox", "Seasonal"),
            f("liquor_colour", "color", "Liquor colour"),
            f("flavour_notes", "multiple_select", "Flavour notes", choices=NOTES),
            f("served_in", "one_to_one_relationship", "Served in", rel="pieces"),
            f("infusions", "repeater", "Infusions", sub=[
                sub("infusion", "single_line_text", "Infusion", isRequired=True), sub("seconds", "number", "Seconds"),
                sub("note", "single_line_text", "What to expect")]),
        ],
    },
    {
        "developerName": "journal", "labelPlural": "Journal", "labelSingular": "Journal entry",
        "description": "Field notes from the kiln, the glaze lab and the tea room.",
        "defaultRouteTemplate": "journal/{PrimaryField}", "primaryField": "title",
        "fields": [
            f("title", "single_line_text", "Title", True),
            f("category", "dropdown", "Category", True, choices=CATEGORIES),
            f("excerpt", "long_text", "Excerpt"),
            f("body", "wysiwyg", "Body"),
            f("cover", "attachment", "Cover image"),
            f("published_on", "date", "Published on"),
            f("reading_minutes", "number", "Reading time (minutes)"),
            f("is_featured", "checkbox", "Featured"),
            f("mood", "color", "Mood colour"),
            f("tags", "multiple_select", "Tags", choices=TAGS),
            f("author", "one_to_one_relationship", "Written by", rel="makers"),
            f("firing_log", "repeater", "Firing log", sub=[
                sub("hour", "single_line_text", "Hour", isRequired=True), sub("temperature", "number", "Temperature (C)"),
                sub("note", "single_line_text", "Note")]),
        ],
    },
]

# label lookup used by the theme compiler: {fieldDeveloperName: [choices]}
CHOICES = {}
for _ct in CONTENT_TYPES:
    for _f in _ct["fields"]:
        if _f["choices"]:
            CHOICES.setdefault(_f["developerName"], _f["choices"])
CHOICES["category"] = CATEGORIES


def document(views=None):
    """The `schema import` document. `views` maps content type -> list of view dicts (added on the second pass)."""
    cts = []
    for ct in CONTENT_TYPES:
        item = {k: v for k, v in ct.items()}
        item["views"] = (views or {}).get(ct["developerName"], [])
        cts.append(item)
    return {"schemaVersion": 1, "contentTypes": cts}
