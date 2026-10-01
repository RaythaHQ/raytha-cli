#!/usr/bin/env python3
"""Build the site pages out of the Kiln custom widgets. Re-runnable: pages are matched by route path."""

import json

from lib import ROOT, jdump, log, rt

MEDIA = json.loads((ROOT / ".build" / "media.json").read_text())


def img(name):
    return f"/raytha/media-items/objectkey/{MEDIA[name]}"


def view_id(ct, dev):
    return next(v["id"] for v in rt("content-type", "views", "list", ct)["items"] if v["developerName"] == dev)


def w(kind, **settings):
    return {"widgetType": f"kk_{kind}", "settings": settings}


def feat(icon, title, text, color):
    return {"icon": icon, "title": title, "text": text, "color": color}


HOURS = [("Monday", 1, "10:00 - 17:00"), ("Tuesday", 2, "10:00 - 17:00"), ("Wednesday", 3, "10:00 - 17:00"), ("Thursday", 4, "10:00 - 20:00 (late class night)"),
         ("Friday", 5, "10:00 - 17:00"), ("Saturday", 6, "09:30 - 18:00"), ("Sunday", 0, "11:00 - 16:00")]

PAGES = {
    "home": dict(
        title="Kiln & Kettle", template="kk_page", home=True,
        sections={
            "hero": [w("hero", eyebrow="Ashby Mill, Hay-on-Wye · est. 2014", headline="Pots you'll use,", highlight="tea you'll linger over",
                       subheadline="A working pottery and tea room in a Victorian bobbin mill. Everything on the shelves was made by one of seven people, fired in our own kilns, and is meant to be held every day.",
                       image=img("hero_shelf"), badgeNumber="1,280°", badgeText="the temperature our wood kiln reached on Sunday",
                       primaryText="Browse the shop", primaryUrl="/shop", secondaryText="Book a class", secondaryUrl="/classes",
                       stats=[{"value": "25", "label": "pieces on the shelves"}, {"value": "7", "label": "makers under one roof"}, {"value": "12", "label": "teas, poured daily"}])],
            "top": [w("marquee", label="Studio news", items=[{"text": "Wood kiln fires Thursday 19 November"}, {"text": "Wheel Basics: 2 seats left"}, {"text": "Hojicha is back for autumn"},
                                                              {"text": "Raku Fire Night is fully booked: join the list"}, {"text": "New: Evening Star lanterns"}]),
                    w("features", eyebrow="Why Kiln & Kettle", heading="A pottery with a kettle on", intro="We started with one kiln and a stubborn idea: the best way to understand a cup is to drink from it.",
                      centered=True, columns="3", items=[
                          feat("flame", "Fired by hand", "Electric, gas, salt, raku and a four-day wood kiln. Every piece is fired on site, and the fire is part of the finish.", "#b5482a"),
                          feat("cup", "Tea in our own cups", "The tea room serves every tea in the piece Haruto thinks suits it best. If you love the cup, it's usually for sale.", "#6f8f6a"),
                          feat("wheel", "Learn it yourself", "Evening courses, weekend intensives and raku nights, taught by people who throw for a living.", "#27406b")])],
            "main": [
                w("showcase", eyebrow="From the shelves", heading="Studio picks", intro="The pieces that make us stop what we're doing. New work lands every Friday after the kiln is opened.",
                  contentType="pieces", viewId=view_id("pieces", "studio_picks"), orderBy="price desc", limit=8, layout="rail", ctaText="See the whole shop", ctaUrl="/shop"),
                w("split", eyebrow="The studio", heading="A bobbin mill that learned to hold fire",
                  body="<p>Ashby Mill made cotton bobbins until 1971, then sat empty for forty years. Margo found it in 2014 with a leaking roof and a perfect north light.</p><p>Today it holds three kilns, a glaze lab, a tea room and a Thursday class that has been fully booked since 2019.</p>",
                  bullets=[{"text": "Open seven days, no booking needed to browse"}, {"text": "Tea room seats twenty and looks onto the kiln yard"}, {"text": "Dogs welcome. Biscuit is the manager."}],
                  image=img("studio_kiln"), accent="#b5482a", buttonText="Read our story", buttonUrl="/studio", tinted=True),
                w("process", eyebrow="How a pot is made", heading="Five steps, about five weeks", intro="Clay doesn't hurry. This is the journey every piece in the shop has taken.",
                  steps=[{"title": "Wedge", "text": "Air is pressed out of the clay by hand so it can't burst in the kiln."},
                         {"title": "Throw", "text": "On the wheel, a lump becomes a bowl in about four minutes."},
                         {"title": "Bisque", "text": "First firing to 1,000°C turns clay into pottery. Still porous, still pale."},
                         {"title": "Glaze", "text": "Dipped, poured or painted. This is where the colour comes from."},
                         {"title": "Fire", "text": "The second firing melts the glaze into glass. Ten hours, or four days for wood."}]),
                w("showcase", eyebrow="Learn with us", heading="Next on the wheel", intro="Courses and one-off sessions, with the seats left shown live.",
                  contentType="workshops", viewId=view_id("workshops", "open_classes"), orderBy="starts_on asc", limit=3, layout="grid", columns="3", ctaText="All classes", ctaUrl="/classes"),
                w("teaboard", kicker="Haruto's list", heading="What's in the pot today", note="Tap any tea for brewing times and a steep timer.", limit=8),
                w("quote", quote="Slow water makes sweet tea. Slow people make sweet pots.", name="Haruto Nakamura", role="Tea master", avatar=img("studio_tea")),
                w("stats", accent="#b5482a", items=[{"value": 12, "suffix": "", "label": "years of firing"}, {"value": 900, "suffix": "+", "label": "people taught to throw"},
                                                    {"value": 640, "suffix": "", "label": "glaze test tiles"}, {"value": 4, "suffix": "", "label": "days per wood firing"}]),
                w("gallery", eyebrow="Around the mill", heading="A few corners of the studio",
                  images=[{"image": img("studio_kiln"), "caption": "The anagama, mid-firing"}, {"image": img("studio_wheel"), "caption": "Thursday evening, wheel six"},
                          {"image": img("studio_tiles"), "caption": "The glaze library"}, {"image": img("studio_shelf"), "caption": "Friday's new work"},
                          {"image": img("studio_tea"), "caption": "Haruto's table"}, {"image": img("studio_ember"), "caption": "Ember"}]),
                w("showcase", eyebrow="The Journal", heading="Kiln notes & kettle talk", intro="The latest from the studio.", contentType="journal", orderBy="published_on desc", limit=3,
                  layout="grid", columns="3", ctaText="Read the Journal", ctaUrl="/journal"),
            ],
            "bottom": [],
            "cta": [w("cta", eyebrow="Come and say hello", heading="The kettle's on.", highlight="So is the kiln.", body="Open every day from Ashby Mill. Drop in for a cup, stay for a class.",
                      primaryText="Plan your visit", primaryUrl="/visit", secondaryText="Browse the shop", secondaryUrl="/shop")],
        },
    ),
    "studio": dict(
        title="The studio", template="kk_page",
        sections={
            "hero": [w("pagehero", crumb="The studio", eyebrow="About Kiln & Kettle", heading="Seven people,", highlight="three kilns, one very good kettle",
                       lede="We're a working pottery first and a tea room second, which is why the tea tastes better than it should.", image=img("studio_wheel"))],
            "top": [w("timeline", eyebrow="Our story", heading="Twelve years in a mill", intro="From a leaking roof and a second-hand gas kiln to a four-day wood firing twice a year.", tinted=True,
                      items=[{"when": "2014", "title": "Margo finds the mill", "text": "A derelict bobbin store with north light, three-phase power and no windows that closed."},
                             {"when": "2016", "title": "The glaze lab", "text": "Tobias arrives with a spreadsheet and a muffle furnace. Test tile number one is a disaster."},
                             {"when": "2018", "title": "The tea room opens", "text": "Haruto sets up four tables in the old packing room. It's still the best seat in the building."},
                             {"when": "2020", "title": "Raku in the yard", "text": "Odessa lights the first raku fire. The neighbours are told in advance."},
                             {"when": "2023", "title": "The anagama", "text": "Dewi and Margo hand-build a wood kiln from reclaimed firebrick. It fires four days, twice a year."},
                             {"when": "2026", "title": "Nine hundred students", "text": "Ines teaches her nine hundredth person to centre clay."}])],
            "main": [
                w("showcase", eyebrow="Meet the makers", heading="The hands", intro="Every piece in the shop names the person who made it.", contentType="makers", orderBy="years_at_wheel desc", limit=7, layout="rail", ctaText="All makers", ctaUrl="/makers"),
                w("quote", quote="A pot is finished when it stops asking you for more.", name="Margo Ashby", role="Founder", avatar=img("studio_kiln")),
                w("split", eyebrow="Materials", heading="Local clay, fired slowly", body="<p>Most of our stoneware comes from a pit forty miles away. We buy porcelain from Stoke and mix our own glazes from raw materials, weighed out in the glaze lab.</p>",
                  bullets=[{"text": "Lead-free, food-safe glazes tested against UK standards"}, {"text": "Studio-recycled water and clay scraps"}, {"text": "Wood for the anagama is reclaimed oak and ash"}],
                  image=img("studio_tiles"), flip=True, accent="#27406b", buttonText="Meet the glaze lab", buttonUrl="/journal"),
                w("stats", accent="#27406b", items=[{"value": 7, "suffix": "", "label": "makers"}, {"value": 3, "suffix": "", "label": "kilns"}, {"value": 640, "suffix": "", "label": "test tiles"}, {"value": 12, "suffix": "", "label": "teas on the menu"}]),
            ],
            "bottom": [],
            "cta": [w("cta", eyebrow="Work with us", heading="Want something", highlight="made for you?", body="Commissions, wedding gifts and full dinner sets.", primaryText="See commissions", primaryUrl="/commissions")],
        },
    ),
    "visit": dict(
        title="Visit & hours", template="kk_page",
        sections={
            "hero": [w("pagehero", crumb="Visit", eyebrow="Find us", heading="Come and", highlight="have a look", lede="The mill is at the end of Mill Lane, behind the old bridge. Look for the chimney and the lanterns.", image=img("visit_exterior"))],
            "top": [w("hours", eyebrow="Opening hours", heading="Open seven days", address="Ashby Mill, Mill Lane\nHay-on-Wye HR3 5AD\n+44 1497 555 0142",
                      note="Kiln days (the first Friday of every month) the yard may be closed to visitors for safety. The tea room stays open.", hours=[{"day": d, "dow": n, "open": o} for d, n, o in HOURS])],
            "main": [
                w("features", eyebrow="Before you come", heading="Good to know", columns="3", items=[
                    feat("leaf", "Getting here", "Ten minutes' walk from Hay town centre. Free parking behind the mill, with bike racks by the gate.", "#6f8f6a"),
                    feat("heart", "Accessible", "Step-free to the shop, the tea room and the ground-floor studio. An accessible toilet is by the kiln yard.", "#b5482a"),
                    feat("star", "Dogs and children", "Both welcome, on the understanding that one of them will end up sitting under the kiln bench.", "#e0a526")]),
                w("faq", eyebrow="Questions", heading="Things people ask", intro="If you don't see it here, use the form below.", items=[
                    {"question": "Do I need to book to visit the shop or tea room?", "answer": "<p>No. Walk in any time we're open. For groups of six or more, please tell us so we can set the big table.</p>"},
                    {"question": "Can I watch the potters work?", "answer": "<p>Yes. The studio is open behind a glass wall, and on Saturdays someone will talk you through what they're doing if you ask.</p>"},
                    {"question": "Do you post pieces?", "answer": "<p>Anywhere in the UK, wrapped in paper and recycled cardboard. International shipping by arrangement.</p>"},
                    {"question": "Is there food?", "answer": "<p>Tea-room cakes baked daily by a local baker, plus toasties at lunch. Everything is vegetarian.</p>"}]),
                w("enquiry", eyebrow="Say hello", heading="Questions, groups & special requests", body="<p>Use the form and one of us will reply within two working days.</p>", buttonText="Send to the studio",
                  topics=[{"label": "Visiting"}, {"label": "Groups & parties"}, {"label": "Shipping"}, {"label": "Something else"}]),
            ],
            "bottom": [],
            "cta": [],
        },
    ),
    "kiln-calendar": dict(
        title="Kiln calendar", template="kk_page",
        sections={
            "hero": [w("pagehero", crumb="Kiln calendar", eyebrow="Firing schedule", heading="When the", highlight="fire is lit", lede="Our kilns run to a calendar. Here's what's firing, and what a four-day wood firing looks like from the inside.", image=img("kiln_still"))],
            "top": [w("kilncurve", eyebrow="The anagama", heading="A wood firing in 96 hours", stage="Cone 12",
                      body="<p>The wood kiln climbs slowly for the first day so the water in the clay can escape as steam. Then it takes off. Hover the dots to see what we're doing at each point.</p>",
                      schedule=[{"hour": 0, "temperature": 20, "note": "Light the kiln"}, {"hour": 12, "temperature": 250, "note": "Slow candling"}, {"hour": 24, "temperature": 600, "note": "Quartz inversion, slow here"},
                                {"hour": 40, "temperature": 900, "note": "Ash starts to land"}, {"hour": 56, "temperature": 1100, "note": "Reduction begins"}, {"hour": 76, "temperature": 1240, "note": "Stoke every four minutes"},
                                {"hour": 88, "temperature": 1280, "note": "Peak. Cone 12 down"}, {"hour": 96, "temperature": 1280, "note": "Seal the kiln"}]),
                    w("timeline", eyebrow="Next up", heading="The firing calendar", intro="Dates move with the weather. We announce on Fridays.", items=[
                        {"when": "Thu 19 Nov", "title": "Anagama wood firing", "text": "Four days and nights. Join a shift with the Anagama Weekend."},
                        {"when": "Fri 20 Nov", "title": "Raku in the yard", "text": "Dusk. Bring a flask."},
                        {"when": "Every Friday", "title": "Electric glaze firing", "text": "Opens Saturday morning. New work on the shelves by lunch."},
                        {"when": "First Friday", "title": "Gas reduction", "text": "Eleven hours to cone 10. Our celadons and tenmokus come from this one."}])],
            "main": [w("showcase", eyebrow="Fresh from the kiln", heading="Wood-fired pieces", contentType="pieces", viewId=view_id("pieces", "wood_fired"), orderBy="title asc", limit=6, layout="rail", ctaText="Shop wood-fired", ctaUrl="/shop/wood-fired")],
            "bottom": [],
            "cta": [w("cta", eyebrow="Join a shift", heading="Stay up with the", highlight="fire", body="Four people only. Bring a sleeping bag and a taste for oak smoke.", primaryText="The Anagama Weekend", primaryUrl="/classes")],
        },
    ),
    "commissions": dict(
        title="Commissions", template="kk_page",
        sections={
            "hero": [w("pagehero", crumb="Commissions", eyebrow="Made for you", heading="A pot", highlight="with your name on the order", lede="Dinner sets, wedding gifts, a teapot for someone who has everything. We'll sketch it with you and make it on the wheel.", image=img("commission_sketch"))],
            "top": [w("process", eyebrow="How it works", heading="From conversation to kiln", intro="Most commissions take six to eight weeks.", steps=[
                {"title": "Tell us", "text": "Send the form: what, how many, for whom, and by when."}, {"title": "Sketch", "text": "We reply with drawings and a glaze palette from the library."},
                {"title": "Make", "text": "Thrown, trimmed and bisque fired. You get a photo at leather-hard."}, {"title": "Glaze", "text": "You choose the final colours on tiles from the lab."},
                {"title": "Fire & send", "text": "Fired, checked, wrapped and delivered or ready to collect."}])],
            "main": [
                w("pricing", eyebrow="Starting points", heading="What commissions cost", intro="Guide prices. Each is quoted individually.", tiers=[
                    {"name": "A single piece", "price": "from £90", "blurb": "A teapot, a large bowl or a vase.", "features": "<ul><li>One maker, one piece</li><li>Choice of glaze</li><li>Signed on the foot</li></ul>", "featured": False},
                    {"name": "A set", "price": "from £280", "blurb": "Four place settings or a tea set.", "features": "<ul><li>Dinner or tea set</li><li>Matched glazes</li><li>Gift packaging</li></ul>", "featured": True},
                    {"name": "A whole table", "price": "from £900", "blurb": "Wedding sets for up to twelve.", "features": "<ul><li>Platters, jugs, bowls</li><li>Personalised marks</li><li>Delivery included</li></ul>", "featured": False}]),
                w("enquiry", eyebrow="Start here", heading="Tell us what you're after", body="<p>A few lines is fine. We'll reply with questions, not a quote.</p>", buttonText="Start a commission",
                  topics=[{"label": "A single piece"}, {"label": "A set"}, {"label": "A whole table"}, {"label": "Not sure yet"}]),
            ],
            "bottom": [],
            "cta": [],
        },
    ),
    "gifts": dict(
        title="Gift ideas", template="kk_page",
        sections={
            "hero": [w("pagehero", crumb="Gift ideas", eyebrow="Presents worth unwrapping", heading="Gifts that get", highlight="used every day", lede="Under £50, ready to ship, or made just for them.")],
            "top": [w("showcase", eyebrow="Under £50", heading="Small, useful, lovely", contentType="pieces", viewId=view_id("pieces", "under_50"), orderBy="price asc", limit=8, layout="grid", columns="4"),
                    w("features", heading="Gift it well", centered=True, columns="3", items=[
                        feat("heart", "Wrapped by hand", "Every order is wrapped in paper with a handwritten note, free.", "#b5482a"),
                        feat("truck", "Ships across the UK", "Two working days, tracked, in recycled packaging.", "#27406b"),
                        feat("cup", "Add a pot of tea", "Haruto will pack 50 g of any tea on the menu.", "#6f8f6a")])],
            "main": [w("showcase", eyebrow="Ready to ship", heading="Can be with them by Friday", contentType="pieces", viewId=view_id("pieces", "ready_to_ship"), orderBy="CreationTime desc", limit=4, layout="grid", columns="4", ctaText="All ready-to-ship", ctaUrl="/shop/ready-to-ship")],
            "bottom": [],
            "cta": [w("cta", heading="Not sure what", highlight="they'd like?", body="A voucher can be spent in the shop, the tea room or on any class.", primaryText="Ask the studio", primaryUrl="/visit")],
        },
    ),
}

existing = {p["routePath"]: p for p in rt("site-page", "list")["items"]}

# the stock Home and About belong to the default theme's design
for route in ("about",):
    if route in existing and route not in PAGES:
        rt("site-page", "delete", existing[route]["id"], "--yes")
        del existing[route]

for route, page in PAGES.items():
    sections = jdump(page["sections"])
    if route in existing:
        pid = existing[route]["id"]
        rt("site-page", "edit", pid, "--title", page["title"], "--template", page["template"])
        rt("site-page", "sections", pid, "--sections", sections, "--replace", "--publish")
        log(f"page /{route} updated")
        continue
    args = ["site-page", "create", "--title", page["title"], "--template", page["template"], "--route-path", route, "--sections", sections]
    if page.get("home"):
        args.append("--home")
    created = rt(*args)
    log(f"page /{route} -> {created.get('id')}")
