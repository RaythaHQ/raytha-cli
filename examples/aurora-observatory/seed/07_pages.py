#!/usr/bin/env python3
"""Build the site pages out of Aurora's custom widgets. Re-runnable: pages are matched by route path."""

import json

from lib import jdump, log, rt

MEDIA = json.loads(open(__file__.replace("07_pages.py", "../.build/media.json")).read())


def img(name):
    return f"/raytha/media-items/objectkey/{MEDIA[name]}"


def view_id(ct, dev):
    return next(v["id"] for v in rt("content-type", "views", "list", ct)["items"] if v["developerName"] == dev)


def w(kind, **settings):
    return {"widgetType": f"aurora_{kind}", "settings": settings}


FAQ = [
    ("Do I need to be fit to join an expedition?", "<p>Every trip has an honest difficulty rating. <strong>Gentle</strong> trips involve short walks on packed snow; <strong>demanding</strong> ones mean several hours a day outdoors at minus twenty. Your guide will tell you which side of that line you sit on before you book.</p>"),
    ("What are the real odds of seeing the aurora?", "<p>We publish the percentage of past departures that saw a visible display on at least one night. Nobody can promise the sky, so every itinerary builds in spare nights and a stand-by vehicle.</p>"),
    ("What do I need to bring?", "<p>Warm layers, a headlamp with a red mode and a sense of patience. We lend expedition parkas, boots, mitts and tripods. The <em>Packing for Minus Thirty</em> field note has the full list.</p>"),
    ("Can I take photos with my phone?", "<p>Modern phones handle a bright display on night mode. For the faint stuff bring a camera with a fast wide lens; the photography masterclass departure teaches exactly this.</p>"),
    ("What if the weather is bad?", "<p>Clouds are the only real enemy. Our guides read three weather models and move the group up to 300 km in a night to find clear sky.</p>"),
]

PAGES = {
    "home": dict(
        title="Aurora Observatory",
        template="aurora_home",
        home=True,
        sections={
            "hero": [w("hero", eyebrow="Tonight: Kp 5 and clear over Hallvarden", headline="Stand under", highlight="the living sky", tail="", subheadline="Small-group aurora expeditions led by the people who run the instruments. We chase the forecast, not the crowds, and we bring the science along.", primaryText="Find your expedition", primaryUrl="/expeditions", secondaryText="Tonight's forecast", secondaryUrl="/forecast", variant="aurora", align="left", minHeight=92, backgroundImage=img("gal_emerald_curtain"), showScrollCue=True, stats=[{"value": "96%", "label": "saw the aurora"}, {"value": "8", "label": "expert guides"}, {"value": "9", "label": "regions"}])],
            "ticker": [w("ticker", duration=60, items=[{"text": "Kp 5 storm watch over Arctic Norway tonight", "color": "#37ffb0"}, {"text": "Geminids peak on Dec 14: 120 meteors an hour", "color": "#ffd166"}, {"text": "Heimdall magnetometer reading calm", "color": "#7fe3ff"}, {"text": "Two places left on the Greenland Ice Cap Vigil", "color": "#ff5d7a"}, {"text": "New field note: Reading the forecast like a pilot", "color": "#a58bff"}])],
            "intro": [w("features", eyebrow="Why Aurora Observatory", heading="An outfitter that owns a telescope", intro="Most tour companies rent a view. We operate a research station, so our guides know what the sky will do hours before it does it.", centered=True, columns="3", items=[
                {"icon": "radar", "title": "Forecasts you can trust", "text": "Live magnetometer, solar wind and cloud data from our own instruments decide where the group sleeps tonight.", "color": "#37ffb0"},
                {"icon": "compass", "title": "Routes that move", "text": "No fixed hotel. We reposition up to 300 km in a night to find a clear patch of sky.", "color": "#7fe3ff"},
                {"icon": "camera", "title": "Photography coaching", "text": "Every trip includes hands-on long-exposure teaching, from phone night mode to full-frame rigs.", "color": "#ffd166"},
                {"icon": "telescope", "title": "Science in the loop", "text": "Hear how the solar wind becomes a curtain of light, from the physicists who log it.", "color": "#a58bff"},
                {"icon": "flame", "title": "Warm by design", "text": "Heated cabins, expedition parkas and hot food waiting at every viewpoint.", "color": "#ff5d7a"},
                {"icon": "snowflake", "title": "Small groups", "text": "Never more than fourteen guests, so the silence is part of the experience.", "color": "#6c8cff"},
            ])],
            "expeditions": [w("showcase", eyebrow="Featured departures", heading="Pick your latitude", intro="Hand-built routes from sailing a glass-still fjord to sleeping on the ice cap.", contentType="expeditions", viewId=view_id("expeditions", "featured_expeditions"), orderBy="departure_date asc", limit=6, layout="rail", ctaText="See all expeditions", ctaUrl="/expeditions")],
            "numbers": [w("stats", items=[{"value": 96, "suffix": "%", "label": "of guests saw the aurora"}, {"value": 14, "suffix": "", "label": "years running the station"}, {"value": 2400, "suffix": "+", "label": "travellers guided"}, {"value": 6, "suffix": "", "label": "live instruments"}])],
            "forecast": [w("forecast", eyebrow="Live from the station", heading="What the sky is doing right now", body="Kp measures how disturbed Earth's magnetic field is. Above 4 the lights reach most of the Arctic circle; at 7 they dance overhead.", kp=5, windSpeed=540, bz=-7, cloudCover=12, updated="2026-09-30", ctaText="Full forecast", ctaUrl="/forecast")],
            "story": [w("split", eyebrow="Masterclass", heading="Learn to photograph the lights", body="<p>Five nights on Svalbard's Skystation with a working astrophotographer. You leave with a portfolio and the technique to repeat it anywhere.</p>", bullets=[{"text": "Night landscape basics and manual exposure"}, {"text": "Stacking and noise reduction workflow"}, {"text": "Shooting timelapse of a full substorm"}], image=img("exp_masterclass"), flip=False, accent="#a58bff", buttonText="View the masterclass", buttonUrl="/expeditions")],
            "gallery": [w("gallery", eyebrow="From the archive", heading="Nights we will not forget", intro="Hero shots from guests and guides, filed under the conditions we captured them in.", limit=9, ctaText="Open the gallery", ctaUrl="/gallery")],
            "voices": [w("voices", eyebrow="Travellers say", heading="They came for the lights", intro="Verified reviews from recent departures.", limit=8, duration=70)],
            "cta": [w("cta", eyebrow="Ready?", heading="The next clear night is", highlight="already forecast", body="Places on every departure are capped at fourteen. Reserve early, and we will keep you posted on the solar wind until you fly.", primaryText="Plan your trip", primaryUrl="/plan-your-trip", secondaryText="Talk to a guide", secondaryUrl="/contact")],
        },
    ),
    "station": dict(
        title="The Station",
        template="aurora_page",
        sections={
            "hero": [w("hero", eyebrow="Hallvarden Geophysical Station", headline="The instruments behind", highlight="every forecast", subheadline="A converted weather station above the Arctic circle, home to six instruments and a rotating cast of physicists, photographers and dog-sled drivers.", primaryText="Meet the instruments", primaryUrl="/instruments", secondaryText="Meet the guides", secondaryUrl="/team", variant="violet", align="center", minHeight=70, backgroundImage=img("gal_dome_wakes"), showScrollCue=False, stats=[])],
            "top": [w("split", eyebrow="Our story", heading="From a weather hut to a research station", body="<p>In 2012 a retired meteorologist and two friends bought a decommissioned magnetometer hut. They wanted to know <em>when</em> the aurora would appear, not just hope for it.</p><p>The data turned out to be so good that guides started asking for it. Today the station feeds every expedition we run and shares its readings with the public.</p>", bullets=[{"text": "Open data: every public instrument streams to the web"}, {"text": "Guides rotate through shifts at the station"}, {"text": "Visitors welcome on Thursday open nights"}], image=img("inst_skald"), flip=True, accent="#37ffb0", buttonText="See the open nights", buttonUrl="/sky")],
            "main": [
                w("timeline", eyebrow="Timeline", heading="Twelve winters of watching", intro="The milestones that got us here.", items=[
                    {"when": "2012", "title": "The hut", "text": "A decommissioned magnetometer building is bought and heated for the first time.", "color": "#37ffb0"},
                    {"when": "2015", "title": "First guided night", "text": "Six guests, one borrowed van, and a Kp 6 storm nobody forecast.", "color": "#7fe3ff"},
                    {"when": "2019", "title": "Skald all-sky camera", "text": "The dome goes live and streams every night to a worldwide audience.", "color": "#a58bff"},
                    {"when": "2022", "title": "Open data programme", "text": "Magnetometer and camera feeds are released under a permissive licence.", "color": "#ffd166"},
                    {"when": "2026", "title": "Nine regions", "text": "Expeditions now run from Tasmania to Svalbard, all guided by station staff.", "color": "#ff5d7a"},
                ]),
                w("instruments", eyebrow="The kit", heading="Six instruments, always listening", intro="Hover a card for specs; open one to see the full data sheet.", limit=6),
                w("oval_map", eyebrow="Where we work", heading="Riding the auroral oval", body="The oval is a ring of peak activity around the magnetic pole. Our routes sit on it.", markers=[
                    {"name": "Hallvarden", "detail": "Station and Norway departures", "x": 52, "y": 28, "color": "#37ffb0", "url": "/expeditions"},
                    {"name": "Abisko", "detail": "Swedish Lapland", "x": 60, "y": 24, "color": "#7fe3ff", "url": "/expeditions"},
                    {"name": "Reykjavik", "detail": "Iceland", "x": 38, "y": 34, "color": "#a58bff", "url": "/expeditions"},
                    {"name": "Yukon", "detail": "Canada", "x": 18, "y": 40, "color": "#ffd166", "url": "/expeditions"},
                    {"name": "Svalbard", "detail": "Skystation", "x": 56, "y": 12, "color": "#ff5d7a", "url": "/expeditions"},
                ]),
                w("team", eyebrow="The crew", heading="People who stand outside all night", intro="Eight guides, each with a specialty and a story.", leadsOnly=False, limit=8),
            ],
            "bottom": [w("faq", eyebrow="Visiting", heading="Questions about the station", intro="", openFirst=True, items=[{"question": "Can I visit the station without booking a trip?", "answer": "<p>Yes. Thursday open nights are free; book a slot through the contact page.</p>"}, {"question": "Is the data really free?", "answer": "<p>The magnetometer and all-sky camera streams are open. Raw files are available on request.</p>"}])],
            "cta": [w("cta", eyebrow="Visit", heading="Come and meet the", highlight="instruments", body="Open nights run every Thursday from October to March.", primaryText="Book an open night", primaryUrl="/contact", secondaryText="Browse expeditions", secondaryUrl="/expeditions")],
        },
    ),
    "plan-your-trip": dict(
        title="Plan your trip",
        template="aurora_page",
        sections={
            "hero": [w("hero", eyebrow="Plan", headline="Choose how much", highlight="night you want", subheadline="Three ways to travel: a weekend vigil, a week-long expedition, or a full polar crossing.", primaryText="See prices", primaryUrl="#pricing", secondaryText="Ask a guide", secondaryUrl="/contact", variant="ember", align="left", minHeight=64, backgroundImage=img("gal_golden_hour"), showScrollCue=False, stats=[{"value": "$1,850", "label": "weekend from"}, {"value": "14", "label": "max group"}])],
            "top": [w("pricing", eyebrow="Pricing", heading="Three ways to go", intro="All prices per person, twin share. Flights not included.", plans=[
                {"name": "Weekend vigil", "price": 1850, "unit": "3 nights", "blurb": "A short, sharp chase for first-timers.", "features": "Heated cabin | Guide and driver | Photo coaching | Hot meals", "highlight": False, "badge": "", "color": "#7fe3ff", "cta": "See gentle trips", "url": "/gentle-expeditions"},
                {"name": "Signature expedition", "price": 4850, "unit": "7 nights", "blurb": "The full experience on the auroral oval.", "features": "Guided route | Dog sled or sailing | Station visit | Expedition parka loan | Private transfer", "highlight": True, "badge": "Most popular", "color": "#37ffb0", "cta": "See featured trips", "url": "/featured-expeditions"},
                {"name": "Polar crossing", "price": 9800, "unit": "11 nights", "blurb": "Ice cap, sled teams and a long polar night.", "features": "Expedition grade kit | Medic on call | Satellite tracker | Sleeping on the ice", "highlight": False, "badge": "", "color": "#ff5d7a", "cta": "See crossings", "url": "/polar-crossings"},
            ])],
            "main": [
                w("showcase", eyebrow="Easy first steps", heading="Gentle expeditions", intro="Short days, warm beds, big sky.", contentType="expeditions", viewId=view_id("expeditions", "gentle_expeditions"), orderBy="price_usd asc", limit=3, layout="grid", ctaText="", ctaUrl=""),
                w("faq", eyebrow="Before you book", heading="Frequently asked", intro="", openFirst=True, items=[{"question": q, "answer": a} for q, a in FAQ]),
            ],
            "bottom": [],
            "cta": [w("cta", eyebrow="Questions?", heading="Talk to someone who has", highlight="stood there", body="Every booking starts with a call from a guide.", primaryText="Contact us", primaryUrl="/contact", secondaryText="Read the field notes", secondaryUrl="/journal")],
        },
    ),
    "forecast": dict(
        title="Tonight's forecast",
        template="aurora_page",
        sections={
            "hero": [w("hero", eyebrow="Live", headline="Tonight over", highlight="Hallvarden", subheadline="Solar wind, magnetic field and cloud cover, refreshed by the station every few minutes.", primaryText="Open the calendar", primaryUrl="/sky", secondaryText="Storm watch", secondaryUrl="/storm-watch", variant="aurora", align="center", minHeight=58, backgroundImage=img("gal_crimson_substorm"), showScrollCue=False, stats=[])],
            "top": [w("forecast", eyebrow="Right now", heading="Kp 5: active", body="Enough to paint the sky from horizon to horizon on a clear night.", kp=5, windSpeed=540, bz=-7, cloudCover=12, updated="2026-09-30", ctaText="Book a place", ctaUrl="/plan-your-trip")],
            "main": [
                w("calendar", eyebrow="Coming up", heading="Sky events", intro="Meteor showers, comets and geomagnetic storm windows.", onlyStorms=False, limit=6, ctaText="Full sky calendar", ctaUrl="/sky"),
                w("calendar", eyebrow="Highest alert", heading="Storm watch", intro="Only the nights where a big geomagnetic storm is expected.", onlyStorms=True, limit=4, ctaText="", ctaUrl=""),
            ],
            "bottom": [w("gallery", eyebrow="What Kp 5 looks like", heading="Recent captures", intro="", limit=6, ctaText="Open the gallery", ctaUrl="/gallery")],
            "cta": [w("quote", quote="You do not watch the aurora. You stand inside it and forget you have a body.", author="Saoirse Keane", role="Guide, Iceland", accent="#37ffb0")],
        },
    ),
    "contact": dict(
        title="Contact",
        template="aurora_page",
        sections={
            "hero": [w("hero", eyebrow="Say hello", headline="Ask a guide", highlight="anything", subheadline="Booking, kit, fitness, photography, dietary needs. A human reads every message and replies within a day.", primaryText="watch@aurora-observatory.example", primaryUrl="mailto:watch@aurora-observatory.example", secondaryText="", secondaryUrl="", variant="violet", align="center", minHeight=50, backgroundImage=img("gal_quiet_blue"), showScrollCue=False, stats=[])],
            "top": [w("split", eyebrow="Find us", heading="Hallvarden Geophysical Station", body="<p>Sjøgata 12, 9008 Tromsø, Norway.</p><p>Open nights every Thursday, October to March, from 20:00. Wear boots; bring a thermos.</p>", bullets=[{"text": "watch@aurora-observatory.example"}, {"text": "+47 555 01 234"}, {"text": "Office hours: Mon-Fri 08:00-16:00 CET"}], image=img("gal_pine_cathedral"), flip=False, accent="#7fe3ff", buttonText="Email the station", buttonUrl="mailto:watch@aurora-observatory.example")],
            "main": [],
            "bottom": [],
            "cta": [w("cta", eyebrow="Or skip the email", heading="See what's", highlight="on this winter", body="", primaryText="Browse expeditions", primaryUrl="/expeditions", secondaryText="", secondaryUrl="")],
        },
    ),
}

existing = {p["routePath"]: p for p in rt("site-page", "list")["items"]}

# stock pages from the default theme would point at templates that are not in this theme's design
for route in ("about",):
    if route in existing and route not in PAGES:
        rt("site-page", "delete", existing[route]["id"], "--yes")
        del existing[route]

for route, page in PAGES.items():
    if route in existing:
        pid = existing[route]["id"]
        rt("site-page", "edit", pid, "--title", page["title"], "--template", page["template"])
        rt("site-page", "sections", pid, "--sections", jdump(page["sections"]), "--replace", "--publish")
        log(f"page /{route} updated")
        continue
    args = ["site-page", "create", "--title", page["title"], "--template", page["template"], "--route-path", route, "--sections", jdump(page["sections"])]
    if page.get("home"):
        args.append("--home")
    created = rt(*args)
    log(f"page /{route} -> {created.get('id')}")
