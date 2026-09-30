"""Content model for the Aurora Observatory site.

Seven content types that between them use every content field type Raytha offers:
single_line_text, long_text, wysiwyg, number, date, checkbox, dropdown, radio,
multiple_select, attachment, one_to_one_relationship, repeater and color.
"""


def ch(*pairs):
    """choices from ('developer_name', 'Label') pairs"""
    return [{"developerName": d, "label": l} for d, l in pairs]


REGIONS = ch(
    ("arctic_norway", "Arctic Norway"),
    ("finnish_lapland", "Finnish Lapland"),
    ("swedish_lapland", "Swedish Lapland"),
    ("iceland", "Iceland"),
    ("greenland", "Greenland"),
    ("svalbard", "Svalbard"),
    ("yukon", "Yukon"),
    ("alaska", "Alaska"),
    ("tasmania", "Tasmania"),
)

SEASONS = ch(
    ("sep_oct", "Sep – Oct"),
    ("nov_dec", "Nov – Dec"),
    ("jan_feb", "Jan – Feb"),
    ("mar_apr", "Mar – Apr"),
)

SKILLS = ch(
    ("aurora_photography", "Aurora photography"),
    ("astronomy", "Astronomy"),
    ("space_weather", "Space weather"),
    ("dog_sledding", "Dog sledding"),
    ("snowshoeing", "Snowshoeing"),
    ("wilderness_medicine", "Wilderness medicine"),
    ("folklore", "Folklore & storytelling"),
    ("instrument_engineering", "Instrument engineering"),
    ("sailing", "Sailing"),
    ("geology", "Geology"),
)

LANGUAGES = ch(
    ("english", "English"),
    ("norwegian", "Norwegian"),
    ("finnish", "Finnish"),
    ("swedish", "Swedish"),
    ("icelandic", "Icelandic"),
    ("spanish", "Spanish"),
    ("german", "German"),
    ("french", "French"),
    ("yoruba", "Yoruba"),
    ("tamil", "Tamil"),
)

LODGING = ch(
    ("lodge", "Wilderness lodge"),
    ("cabin", "Log cabin"),
    ("ship", "Expedition ship"),
    ("tent", "Heated tent camp"),
    ("glass_igloo", "Glass igloo"),
    ("station", "Observatory station"),
    ("hotel", "Town hotel"),
)

CONTENT_TYPES = [
    {
        "name": "guides",
        "plural": "Field Guides",
        "singular": "Field Guide",
        "route": "team/{PrimaryField}",
        "description": "The people who lead expeditions and run the station.",
        "primary": ("title", "Name"),
        "content_label": "Biography",
        "fields": [
            ("role", "single_line_text", "Role", {}),
            ("quote", "long_text", "Personal motto", {}),
            ("portrait", "attachment", "Portrait", {}),
            ("accent", "color", "Accent colour", {}),
            ("years_experience", "number", "Years in the field", {}),
            ("base_region", "dropdown", "Home region", {"choices": REGIONS}),
            ("specialties", "multiple_select", "Specialties", {"choices": SKILLS}),
            ("languages", "multiple_select", "Languages", {"choices": LANGUAGES}),
            ("is_lead_guide", "checkbox", "Lead guide", {}),
            (
                "links",
                "repeater",
                "Elsewhere on the web",
                {
                    "sub": [
                        {"developerName": "platform", "label": "Platform", "fieldType": "dropdown", "choices": ch(("instagram", "Instagram"), ("mastodon", "Mastodon"), ("youtube", "YouTube"), ("website", "Website"), ("bluesky", "Bluesky"))},
                        {"developerName": "handle", "label": "Handle", "fieldType": "single_line_text"},
                    ]
                },
            ),
        ],
    },
    {
        "name": "expeditions",
        "plural": "Expeditions",
        "singular": "Expedition",
        "route": "expeditions/{PrimaryField}",
        "description": "Guided aurora-chasing journeys, from weekend vigils to polar crossings.",
        "primary": ("title", "Expedition name"),
        "content_label": "The story",
        "fields": [
            ("tagline", "single_line_text", "Tagline", {}),
            ("summary", "long_text", "Summary", {}),
            ("hero_image", "attachment", "Hero image", {}),
            ("accent", "color", "Accent colour", {}),
            ("region", "dropdown", "Region", {"choices": REGIONS}),
            (
                "difficulty",
                "radio",
                "Difficulty",
                {"choices": ch(("gentle", "Gentle"), ("moderate", "Moderate"), ("demanding", "Demanding"), ("extreme", "Extreme"))},
            ),
            ("seasons", "multiple_select", "Best seasons", {"choices": SEASONS}),
            ("departure_date", "date", "Next departure", {}),
            ("duration_nights", "number", "Nights", {}),
            ("price_usd", "number", "Price (USD)", {}),
            ("group_size", "number", "Group size (max)", {}),
            ("aurora_odds", "number", "Aurora odds (%)", {}),
            ("is_featured", "checkbox", "Featured on the home page", {}),
            ("lead_guide", "one_to_one_relationship", "Lead guide", {"related": "guides"}),
            (
                "itinerary",
                "repeater",
                "Day by day",
                {
                    "sub": [
                        {"developerName": "day", "label": "Day", "fieldType": "single_line_text"},
                        {"developerName": "title", "label": "Title", "fieldType": "single_line_text"},
                        {"developerName": "details", "label": "Details", "fieldType": "long_text"},
                        {"developerName": "lodging", "label": "Lodging", "fieldType": "dropdown", "choices": LODGING},
                        {"developerName": "distance_km", "label": "Distance (km)", "fieldType": "number"},
                        {"developerName": "aurora_night", "label": "Prime aurora night", "fieldType": "checkbox"},
                    ]
                },
            ),
            (
                "moments",
                "repeater",
                "Moments",
                {
                    "sub": [
                        {"developerName": "caption", "label": "Caption", "fieldType": "single_line_text"},
                        {"developerName": "photo", "label": "Photo", "fieldType": "attachment"},
                    ]
                },
            ),
        ],
    },
    {
        "name": "sky_events",
        "plural": "Sky Events",
        "singular": "Sky Event",
        "route": "sky/{PrimaryField}",
        "description": "Meteor showers, geomagnetic storm watches and other things worth staying up for.",
        "primary": ("title", "Event"),
        "content_label": "Description",
        "fields": [
            (
                "event_type",
                "dropdown",
                "Type",
                {"choices": ch(("meteor_shower", "Meteor shower"), ("storm_watch", "Geomagnetic storm watch"), ("comet", "Comet"), ("solstice", "Solstice vigil"), ("festival", "Festival"), ("equinox", "Equinox peak"))},
            ),
            ("starts_on", "date", "Starts", {}),
            ("ends_on", "date", "Ends", {}),
            ("visibility", "number", "Visibility score (1-10)", {}),
            ("expected_kp", "number", "Expected Kp index", {}),
            ("livestream", "checkbox", "Live-streamed from the station", {}),
            ("viewing_tip", "long_text", "Viewing tip", {}),
            ("event_color", "color", "Event colour", {}),
            ("poster", "attachment", "Poster", {}),
            ("best_expedition", "one_to_one_relationship", "Best expedition for it", {"related": "expeditions"}),
            (
                "schedule",
                "repeater",
                "Night schedule",
                {
                    "sub": [
                        {"developerName": "time", "label": "Time", "fieldType": "single_line_text"},
                        {"developerName": "activity", "label": "Activity", "fieldType": "single_line_text"},
                        {"developerName": "public", "label": "Open to visitors", "fieldType": "checkbox"},
                    ]
                },
            ),
        ],
    },
    {
        "name": "posts",  # the stock type is repurposed: the v1 API cannot delete content types
        "plural": "Field Notes",
        "singular": "Field Note",
        "route": "journal/{CurrentYear}/{PrimaryField}",
        "description": "Essays and dispatches from the field.",
        "primary": ("title", "Headline"),
        "content_label": "Article",
        "fields": [
            ("deck", "single_line_text", "Standfirst", {}),
            ("author", "one_to_one_relationship", "Author", {"related": "guides"}),
            ("published_on", "date", "Published", {}),
            (
                "category",
                "dropdown",
                "Category",
                {"choices": ch(("science", "Science"), ("photography", "Photography"), ("folklore", "Folklore"), ("gear", "Gear"), ("dispatch", "Dispatch"))},
            ),
            (
                "mood",
                "radio",
                "Mood",
                {"choices": ch(("calm", "Calm"), ("electric", "Electric"), ("eerie", "Eerie"), ("triumphant", "Triumphant"))},
            ),
            (
                "tags",
                "multiple_select",
                "Tags",
                {"choices": ch(("kp_index", "Kp index"), ("long_exposure", "Long exposure"), ("cold_weather", "Cold weather"), ("wildlife", "Wildlife"), ("history", "History"), ("sound", "Sound"), ("packing", "Packing"), ("myth", "Myth"))},
            ),
            ("reading_minutes", "number", "Minutes to read", {}),
            ("pull_quote", "long_text", "Pull quote", {}),
            ("is_featured", "checkbox", "Feature on home page", {}),
            ("palette", "color", "Palette colour", {}),
            ("from_expedition", "one_to_one_relationship", "Written on expedition", {"related": "expeditions"}),
        ],
    },
    {
        "name": "photographs",
        "plural": "Aurora Gallery",
        "singular": "Photograph",
        "route": "gallery/{PrimaryField}",
        "description": "A curated archive of auroras photographed from our stations and expeditions.",
        "primary": ("title", "Title"),
        "content_label": "Notes",
        "fields": [
            ("photo", "attachment", "Photograph", {}),
            ("caption", "long_text", "Caption", {}),
            ("location", "dropdown", "Location", {"choices": REGIONS}),
            ("captured_on", "date", "Captured on", {}),
            ("exposure_seconds", "number", "Exposure (s)", {}),
            ("iso", "number", "ISO", {}),
            ("lens", "single_line_text", "Lens", {}),
            ("dominant_color", "color", "Dominant colour", {}),
            (
                "conditions",
                "radio",
                "Sky conditions",
                {"choices": ch(("crystal_clear", "Crystal clear"), ("thin_cloud", "Thin cloud"), ("moonlit", "Moonlit"), ("storm", "Storm-level"))},
            ),
            ("kp_at_capture", "number", "Kp at capture", {}),
            ("is_hero", "checkbox", "Hero shot", {}),
            ("photographer", "one_to_one_relationship", "Photographer", {"related": "guides"}),
        ],
    },
    {
        "name": "instruments",
        "plural": "Instruments",
        "singular": "Instrument",
        "route": "instruments/{PrimaryField}",
        "description": "The sensors and telescopes that watch the sky.",
        "primary": ("title", "Instrument"),
        "content_label": "Overview",
        "fields": [
            (
                "kind",
                "dropdown",
                "Kind",
                {"choices": ch(("all_sky_camera", "All-sky camera"), ("magnetometer", "Magnetometer"), ("spectrograph", "Spectrograph"), ("telescope", "Telescope"), ("riometer", "Riometer"), ("lidar", "Lidar"))},
            ),
            ("photo", "attachment", "Photo", {}),
            (
                "status",
                "radio",
                "Status",
                {"choices": ch(("online", "Online"), ("calibrating", "Calibrating"), ("maintenance", "Maintenance"))},
            ),
            ("installed_on", "date", "Installed", {}),
            ("public_data", "checkbox", "Data is public", {}),
            ("glow", "color", "Display colour", {}),
            ("cadence_seconds", "number", "Sample interval (s)", {}),
            ("operator", "one_to_one_relationship", "Operator", {"related": "guides"}),
            (
                "specs",
                "repeater",
                "Specifications",
                {
                    "sub": [
                        {"developerName": "label", "label": "Label", "fieldType": "single_line_text"},
                        {"developerName": "value", "label": "Value", "fieldType": "single_line_text"},
                        {"developerName": "notable", "label": "Headline spec", "fieldType": "checkbox"},
                    ]
                },
            ),
        ],
    },
    {
        "name": "voices",
        "plural": "Voices",
        "singular": "Voice",
        "route": "voices/{PrimaryField}",
        "description": "Words from travellers who came back changed.",
        "primary": ("title", "Traveller"),
        "content_label": "Full story",
        "fields": [
            ("quote", "long_text", "Quote", {}),
            ("hometown", "single_line_text", "Hometown", {}),
            ("rating", "radio", "Rating", {"choices": ch(("5", "5 stars"), ("4", "4 stars"))}),
            ("tint", "color", "Card tint", {}),
            ("avatar", "attachment", "Photo", {}),
            ("travelled_on", "one_to_one_relationship", "Expedition", {"related": "expeditions"}),
            ("verified", "checkbox", "Verified traveller", {}),
        ],
    },
]
